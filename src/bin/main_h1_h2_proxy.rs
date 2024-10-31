use std::{io, net::{IpAddr, Ipv4Addr, SocketAddr}, str::FromStr, sync::Arc};

use futures::future;
use h3::quic::BidiStream;

// use http_body_util::Full;
use hyper::{server::conn::http1, server::conn::http2, service::service_fn};
use hyper_util::rt::TokioIo;
use quinn::{crypto::rustls::QuicServerConfig, Connection, Endpoint, Incoming, RecvStream, SendStream};
use rustls::{pki_types::{self, ServerName}, RootCertStore, ServerConfig};

use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio::io::{split, copy};

use bytes::{Buf, Bytes};

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

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("default provider already set elsewhere");

    // for tcp usage
    let tcp_tls_acceptor = get_h2_config()?;
    // let tcp_listener = TcpListener::bind("127.0.0.1:443").await?;
    let tcp_listener = TcpListener::bind("172.30.143.91:443").await?;
    println!("Tcp binding finished");

    // set tls for quic
    // let server_config = get_h3_config()?;
    // let endpoint = quinn::Endpoint::server(
    //     server_config,
    //     // SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443),
    //     SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 30, 143, 91)), 443),
    // )?;
    // println!("Quic binding finished");

    // set server
    let tcp_tls_task = tokio::spawn(process_tcp_request(tcp_listener, tcp_tls_acceptor));
    // let quic_task = tokio::spawn(process_quic_request(endpoint));
    // let _ = tokio::join!(tcp_tls_task, quic_task);
    let _ = tcp_tls_task.await;

    Ok(())
}


async fn process_tcp_request(
    tcp_listener: TcpListener,
    tls_acceptor: TlsAcceptor
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let tls_acceptor_clone = tls_acceptor.clone();
        tokio::spawn(proxy_tcp_tls(tcp_stream, tls_acceptor_clone));
        // tokio::spawn(proxy_tcp_tls_naive(tcp_stream, tls_acceptor_clone));
    }

    Ok(())
}


async fn proxy_tcp_tls(
    tcp_stream: TcpStream,
    tls_acceptor: TlsAcceptor
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match tls_acceptor.accept(tcp_stream).await {
        Ok(client_tls_stream) => {
            // obtain domain name from tls connection
            let Some(server_name) = client_tls_stream.get_ref().1.server_name() else { return Ok(()) };

            // build connection to server
            let root_store = RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.into(),
            };
            let mut proxy_config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            let Some(alpn) = client_tls_stream.get_ref().1.alpn_protocol()
                .map(|x|x.to_vec()) else { return Ok(()) };

            if alpn == H2 {
                proxy_config.alpn_protocols= vec![H2.to_vec()];

                let server_port = String::from_str(server_name)? + ":443";
                let proxy_connector = TlsConnector::from(Arc::new(proxy_config));
                let server_domain = pki_types::ServerName::try_from(server_name)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
                    .to_owned();

                let server_tcp_stream = TcpStream::connect(server_port.clone()).await?;
                let server_tls_stream = proxy_connector.connect(server_domain, server_tcp_stream).await?;
                let server_io = TokioIo::new(server_tls_stream);
                let (h2_server_send, h2_server_conn) = hyper::client::conn::http2::handshake
                    ::<TokioExecutor, TokioIo<tokio_rustls::client::TlsStream<TcpStream>>, hyper::body::Incoming>(TokioExecutor, server_io)
                    .await?;

                tokio::task::spawn(async move {
                    if let Err(err) = h2_server_conn.await {
                        println!("http2 Connection failed: {:?}", err);
                    }
                });

                let client_io = TokioIo::new(client_tls_stream);

                let _ = http2::Builder::new(TokioExecutor)
                    .serve_connection(client_io, service_fn(move |h2_req| {
                        handle_http2_tunnel(h2_req, h2_server_send.clone())
                    }))
                    .await;


            } else if alpn == HTTP1_1 {   

                proxy_config.alpn_protocols= vec![HTTP1_1.to_vec()];
                let proxy_config_clone = proxy_config.clone();
                let server_name_clone = String::from(server_name);

                let client_io = TokioIo::new(client_tls_stream);

                let _ = http1::Builder::new()
                    .serve_connection(client_io, service_fn(move |req_http1| {
                        handle_http1_tunnel(req_http1, server_name_clone.clone(), proxy_config_clone.clone())
                    }))
                    .await;

            } else {
                proxy_config.alpn_protocols= vec![alpn];

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
            }
            
        }

        Err(e) => { println!("TLS accepted error on tcp: {}", e); }
    }

    Ok(())
}


async fn handle_http2_tunnel(
    client_req: hyper::Request<hyper::body::Incoming>,
    mut server_send: hyper::client::conn::http2::SendRequest<hyper::body::Incoming>,
) -> Result<hyper::Response<hyper::body::Incoming>, Box<dyn std::error::Error + Sync + Send>> {

    let server_resq = server_send.send_request(client_req).await?;
    // todo!("convert the incoming to bytes");
    // let res_collected = server_res.into_body().collect().await?;
    // let res_bytes = res_collected.to_bytes();

    Ok(server_resq)
}

async fn handle_http1_tunnel(
    client_req: hyper::Request<hyper::body::Incoming>,
    server_name: String,
    proxy_config: rustls::ClientConfig,
) -> Result<hyper::Response<hyper::body::Incoming>, Box<dyn std::error::Error + Sync + Send>> {
    let server_port = server_name.clone() + ":443";

    let proxy_connector = TlsConnector::from(Arc::new(proxy_config));
    let server_domain = pki_types::ServerName::try_from(server_name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();

    let server_tcp_stream = TcpStream::connect(server_port.clone()).await?;
    let server_tls_stream = proxy_connector.connect(server_domain, server_tcp_stream).await?;

    let server_io = TokioIo::new(server_tls_stream);
    let (mut server_sender, server_conn) = hyper::client::conn::http1::handshake(server_io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = server_conn.await {
            println!("http1 Connection failed: {:?}", err);
        }
    });

    let server_resp = server_sender.send_request(client_req).await?;

    Ok(server_resp)
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




