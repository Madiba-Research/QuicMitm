use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use std::{io::BufReader, sync::Arc};
use std::fs::File;
use quinn::{Accept, ConnectError, ConnectionError, Incoming};
use quinn::{crypto::rustls::QuicServerConfig, Endpoint, TransportConfig};
use tokio::net::UdpSocket;
use rustls_pemfile;
use rustls::server::ServerConfig;


use alpn::HQ29;



pub mod alpn {
    // pub const H2: &[u8] = b"h2";
    // pub const HTTP1_1: &[u8] = b"http/1.1";
    // pub const HTTP3: &[u8] = b"h3";
    pub const HQ29: &[u8] = b"hq-29";
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + >> {

    // prepare udp
    let socket = UdpSocket::bind("127.0.0.1:443").await?;

    // set tls
    let local_cert = "localTestCert.pem";
    let local_key = "localTestKey.pem";
    let server_crypto = config_tls(local_cert, local_key);

    // set tls for quic
    let mut server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    let endpoint =
        quinn::Endpoint::server(server_config, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443))?;

    println!("listening on {}", endpoint.local_addr()?);
    loop {
        if let Some(conn) = endpoint.accept().await {
            let fut = handle_connection(conn);
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    println!("connection failed: {reason}", reason = e.to_string())
                }
            });
        }

    }

    Ok(())
}




async fn handle_connection(conn: quinn::Incoming) -> Result<(), ConnectionError> {
    if let Ok(connection) = conn.await {
        async {
            println!("established");
    
            // Each stream initiated by the client constitutes a new request.
            loop {
                let stream = connection.accept_bi().await;
                let stream = match stream {
                    Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                        println!("connection closed");
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(e);
                    }
                    Ok(s) => s,
                };
                let fut = handle_request(stream);
                tokio::spawn(
                    async move {
                        if let Err(e) = fut.await {
                            println!("failed: {reason}", reason = e.to_string());
                        }
                    }
                );
            }
        }
        .await?;

    } else {
        return Ok(())
    }
    Ok(())
}


sync fn handle_request()



fn config_tls(local_cert: &str, local_key: &str) -> ServerConfig {
    let cert_file = local_cert;
    let private_key_file = local_key;

    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file).unwrap()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let private_key = rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(private_key_file).unwrap()))
        .unwrap()
        .unwrap();
    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .unwrap();
    config.alpn_protocols = vec![HQ29.to_vec()];

    config
}
