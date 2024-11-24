use std::{io, net::{IpAddr, Ipv4Addr, SocketAddr}, str::FromStr, sync::Arc};

use futures::future;
use h3::quic::BidiStream;


use quinn::{crypto::rustls::QuicServerConfig, Endpoint, Incoming, TransportConfig};
use rustls::{pki_types::{self}, RootCertStore, ServerConfig};

use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio::io::{split, copy};

use bytes::{Buf, Bytes};

use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;

mod cert_generate_util;

pub mod alpn {
    pub const H2: &[u8] = b"h2";
    pub const HTTP1_1: &[u8] = b"http/1.1";
    pub const HTTP3: &[u8] = b"h3";
    pub const HQ29: &[u8] = b"hq-29";
}

use alpn::H2;
use alpn::HTTP1_1;
use alpn::HTTP3;

#[derive(Clone)]
// An Executor that uses the tokio runtime.
pub struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    // tracing::subscriber::set_global_default(
    //     tracing_subscriber::FmtSubscriber::builder()
    //         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //         .finish(),
    // )
    // .unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("default provider already set elsewhere");

    // for tcp usage
    // let tcp_tls_acceptor = get_h2_config()?;
    // let tcp_listener = TcpListener::bind("127.0.0.1:443").await?;
    // let tcp_listener = TcpListener::bind("172.30.143.95:443").await?;
    // println!("Tcp binding finished");

    // set tls for quic
    let server_config = get_h3_config()?;

    // let mut endpoint_config = quinn::EndpointConfig::default();
    // // let ep_config = endpoint_config.max_udp_payload_size(1216).expect("fail set udp payload").clone();

    // let runtime = Arc::new(quinn::TokioRuntime);
    // let socket = std::net::UdpSocket::bind("172.30.143.58:443")?;
    // let endpoint = quinn::Endpoint::new(ep_config, Some(server_config), socket, runtime)
    //     .expect("cannot build endpoint");
    
    
    let endpoint = quinn::Endpoint::server(
        server_config,
        // SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 30, 143, 58)), 443),
    )?;
    
    println!("Quic binding finished");

    // set server
    // let tcp_tls_task = tokio::spawn(process_tcp_request(tcp_listener, tcp_tls_acceptor));
    let quic_task = tokio::spawn(process_quic_request(endpoint));
    quic_task.await?;
    // let _ = tokio::join!(tcp_tls_task, quic_task);

    Ok(())
}


async fn process_tcp_request(
    tcp_listener: TcpListener,
    tls_acceptor: TlsAcceptor
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let tls_acceptor_clone = tls_acceptor.clone();
        tokio::spawn(proxy_tcp_tls_naive(tcp_stream, tls_acceptor_clone));
    }

    Ok(())
}


async fn proxy_tcp_tls_naive(
    tcp_stream: TcpStream,
    tls_acceptor: TlsAcceptor
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match tls_acceptor.accept(tcp_stream).await {
        Ok(client_tls_stream) => {
            // obtain domain name from tls connection
            let server_name_option = client_tls_stream.get_ref().1.server_name();

            if let Some(server_name) = server_name_option {
                // println!("tcp_tls to server name: {}", server_name);
                
                // build connection to server
                let root_store = RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.into(),
                };
                let mut proxy_config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                proxy_config.alpn_protocols= vec![H2.to_vec(), HTTP1_1.to_vec()];

                let server_port = String::from_str(server_name)? + ":443";
                let proxy_connector = TlsConnector::from(Arc::new(proxy_config));
                let server_domain = pki_types::ServerName::try_from(server_name)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
                    .to_owned();

                let server_tcp_stream = TcpStream::connect(server_port.clone()).await?;
                let server_tls_stream = proxy_connector.connect(server_domain, server_tcp_stream).await?;

                // wire the client-proxy, and proxy-server
                let (mut to_client_read, mut to_client_write) = split(client_tls_stream);
                let (mut to_server_read, mut to_server_write) = split(server_tls_stream);

                let upload_fut = copy(&mut to_client_read, &mut to_server_write);
                let download_fut = copy(&mut to_server_read, &mut to_client_write);

                let _ = tokio::join!(upload_fut, download_fut);

                // println!("tcp tls stream done");
            }
        }

        Err(e) => { println!("TLS accepted error on tcp: {}", e); }
    }

    Ok(())
}


async fn process_quic_request(endpoint: Endpoint) {
    // while let Some(new_conn) = endpoint.accept().await {
    //     println!("quic accepting connection");
    //     // tokio::spawn(proxy_quic_connection(new_conn));
    //     tokio::spawn(async move {
    //         if let Err(e) = proxy_quic_connection(new_conn).await {
    //             println!("error in quic connection: {}", e);
    //         }
    //     });
    // }
    if let Some(new_conn) = endpoint.accept().await {
        println!("quic accepting connection");
        // tokio::spawn(proxy_quic_connection(new_conn));
        tokio::spawn(async move {
            if let Err(e) = proxy_quic_connection(new_conn).await {
                println!("error in quic connection: {}", e);
            }
        }).await;
    }
    endpoint.wait_idle().await;
}


async fn proxy_quic_connection(conn: Incoming) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    println!("before conn");
    let proxy_conn = conn.await?;
    println!("after conn");

    let server_domain = proxy_conn
        .handshake_data()
        .unwrap()
        .downcast::<quinn::crypto::rustls::HandshakeData>().unwrap()
        .server_name
        .map_or_else(|| "<none>".into(), |x| x);
    println!("quic to server name: {}", &server_domain);
    
    // set connection to the server
    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let mut proxy_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    proxy_config.enable_early_data = true;
    proxy_config.alpn_protocols = vec![HTTP3.to_vec()];

    let mut proxy_endpoint = h3_quinn::quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    let mut proxy_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(proxy_config)?,
    ));

    let mut transport_config = TransportConfig::default();
    transport_config.enable_segmentation_offload(false);
    proxy_config.transport_config(Arc::new(transport_config));

    proxy_endpoint.set_default_client_config(proxy_config);


    let server_domain_port = server_domain.clone() + ":443";
    let server_addr = tokio::net::lookup_host(server_domain_port)
        .await?
        .next()
        .ok_or("dns found no addresses")?;

    
    let server_conn = proxy_endpoint.connect(server_addr, &server_domain)?.await?;
    let h3_quinn_server_conn = h3_quinn::Connection::new(server_conn);
    
    let h3_proxy_conn: h3::server::Connection<h3_quinn::Connection, Bytes>  = h3::server::Connection::new(h3_quinn::Connection::new(proxy_conn)).await?;

    let _ = accept_bi_streams(h3_proxy_conn, h3_quinn_server_conn).await;
    
    proxy_endpoint.wait_idle().await;
    Ok(())
}


async fn accept_bi_streams(
    mut proxy_conn: h3::server::Connection<h3_quinn::Connection, Bytes>,
    h3_quinn_server_conn: h3_quinn::Connection,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {

    let (mut conn_driver, mut send_request) = h3::client::new(h3_quinn_server_conn).await?;
    let drive = async move {
        future::poll_fn(|cx| conn_driver.poll_close(cx)).await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    while let Some(client_req_stream) = proxy_conn.accept().await? {
        println!("h3 client request: {:?}", client_req_stream.0);
        let mut req_server_stream = send_request.send_request(client_req_stream.0).await?;
        
        tokio::spawn(handle_tunnel_stream(client_req_stream.1, req_server_stream));
    }

    drive.await;
    
    Ok(())
}

async fn handle_tunnel_stream<T>(
    mut to_client_stream: h3::server::RequestStream<T, Bytes>,
    mut to_server_stream: h3::client::RequestStream<T, Bytes>,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>>
where T: BidiStream<Bytes> {

    while let Some(mut chunk) = to_client_stream.recv_data().await? {
        let data = chunk.copy_to_bytes(chunk.remaining());
        to_server_stream.send_data(data).await?;
    }
    to_server_stream.finish().await?;

    let server_resp = to_server_stream.recv_response().await?;
    to_client_stream.send_response(server_resp).await?;

    while let Some(mut chunk) = to_server_stream.recv_data().await? {
        let data = chunk.copy_to_bytes(chunk.remaining());
        // println!("h3 received data from server: {}", String::from_utf8_lossy(data.clone().chunk()));
        to_client_stream.send_data(data).await?;
    }
    to_client_stream.finish().await?;

    Ok(())
}






fn get_h2_config() -> io::Result<TlsAcceptor> {

    let ca_cert_file = "democacert.pem";
    let ca_key_file = "democakey.pem";

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(cert_generate_util::DynamicCertResolver::new(ca_cert_file, ca_key_file)));


    config.alpn_protocols= vec![H2.to_vec(), HTTP1_1.to_vec()];

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    Ok(tls_acceptor)
}


fn get_h3_config() -> io::Result<quinn::ServerConfig> {

    let ca_cert_file = "democacert.pem";
    let ca_key_file = "democakey.pem";
    
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(cert_generate_util::DynamicCertResolver::new(ca_cert_file, ca_key_file)));

    
    config.alpn_protocols= vec![HTTP3.to_vec()];

    let mut quinn_server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap()));

    // set enable segmentation offload as false, so that every udp package has only 1 datagram
    let mut transport_config = TransportConfig::default();
    transport_config.enable_segmentation_offload(false);
    transport_config.datagram_send_buffer_size(0);

    quinn_server_config.transport_config(Arc::new(transport_config));

    Ok(quinn_server_config)
}

