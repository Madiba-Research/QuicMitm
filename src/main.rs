use std::{io, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc};

use quinn::{crypto::rustls::QuicServerConfig, Endpoint, Incoming};
use rustls::ServerConfig;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

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



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("default provider already set elsewhere");

    // for tcp usage
    let tcp_tls_acceptor = get_h2_config()?;
    // let tcp_listener = TcpListener::bind("127.0.0.1:443").await?;
    let tcp_listener = TcpListener::bind("172.30.143.77:443").await?;
    println!("Tcp binding finished");

    // set tls for quic
    let server_config = get_h3_config()?;
    let endpoint = quinn::Endpoint::server(
        server_config,
        // SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 30, 143, 77)), 443),
    )?;
    println!("Quic binding finished");

    // set server
    let tcp_tls_task = tokio::spawn(process_tcp_request(tcp_listener, tcp_tls_acceptor));
    let quic_task = tokio::spawn(process_quic_request(endpoint));
    tokio::join!(tcp_tls_task, quic_task);


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
    }
}


async fn proxy_tcp_tls(tcp_stream: TcpStream, tls_acceptor: TlsAcceptor) {
    match tls_acceptor.accept(tcp_stream).await {
        Ok(tls_stream) => {
            todo!("build tcp tls connection to server and deliver the traffic");

        }

        Err(e) => { println!("TLS accepted error on tcp: {}", e); }
    }
}


async fn process_quic_request(endpoint: Endpoint) {
    while let Some(new_conn) = endpoint.accept().await {
        println!("quic accepting connection");
        tokio::spawn(proxy_quic_connection(new_conn));
    }
    endpoint.wait_idle().await;
}

async fn proxy_quic_connection(conn: Incoming) {
    todo!("build quic connection to server and deliver the traffic");

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

    let quinn_server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap()));

    Ok(quinn_server_config)
}

