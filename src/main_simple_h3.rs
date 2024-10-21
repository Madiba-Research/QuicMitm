use std::{
    // ascii, fs, io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    // path::{self, Path, PathBuf},
    str,
    sync::Arc,
};

// use anyhow::{anyhow, bail, Context, Ok, Result};
// use quinn::crypto::rustls::QuicServerConfig;
// use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};


use rustls::server::ServerConfig;
use std::io::BufReader;
use std::fs::File;

use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};
use h3::{error::ErrorLevel, quic::BidiStream, server::RequestStream};

use bytes::{BufMut, Bytes, BytesMut};
use http::{Request, StatusCode};

use alpn::HQ29;
use alpn::HTTP3;

pub mod alpn {
    // pub const H2: &[u8] = b"h2";
    // pub const HTTP1_1: &[u8] = b"http/1.1";
    pub const HTTP3: &[u8] = b"h3";
    pub const HQ29: &[u8] = b"hq-29";
}


// fn main() {
//     let code = {
//         if let Err(e) = run() {
//             eprintln!("ERROR: {e}");
//             1
//         } else {
//             0
//         }
//     };
//     ::std::process::exit(code);
// }


// a simple hello world server using h3
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
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
    println!("port binding: {}", endpoint.local_addr()?);
    while let Some(new_conn) = endpoint.accept().await {
        
        println!("accepting connection");
        tokio::spawn(handle_connection(new_conn));
    }
    Ok(())
}


async fn handle_connection(new_conn: quinn::Incoming) {
    match new_conn.await {
        Ok(conn) => {
            let mut h3_conn: h3::server::Connection<h3_quinn::Connection, Bytes> = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                .await
                .unwrap();
            loop {
                match h3_conn.accept().await {
                    Ok(Some((req, stream))) => {
                        println!("new request: {:#?}", req);

                        tokio::spawn(async {
                            if let Err(e) = handle_request(req, stream).await {
                                println!("handling request failed: {}", e);
                            }
                        });
                    }

                    // indicating no more streams to be received
                    Ok(None) => {
                        break;
                    }

                    Err(err) => {
                        println!("error on accept {}", err);
                        match err.get_error_level() {
                            ErrorLevel::ConnectionError => break,
                            ErrorLevel::StreamError => continue,
                        }
                    }
                }
            }
        },
        Err(e) => {println!("quic connection failed: {}", e)},
    }
}



async fn handle_request<T>(
    req: Request<()>,
    mut stream: RequestStream<T, Bytes>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: BidiStream<Bytes>,
{

    let resp = http::Response::builder().body(()).unwrap();

    match stream.send_response(resp).await {
        Ok(_) => {
            println!("successfully respond to connection");
        }
        Err(err) => {
            println!("unable to send response to connection peer: {:?}", err);
        }
    }

    let mut buf = BytesMut::with_capacity(4096 * 10);
    buf.put(&b"Hello, world!\n"[..]);
    stream.send_data(buf.freeze()).await?;


    Ok(stream.finish().await?)
}



// async fn handle_connection(conn: quinn::Incoming) -> Result<()> {
//     let connection = conn.await?;

//     async {
//         println!("established");

//         // Each stream initiated by the client constitutes a new request.
//         loop {
//             let stream = connection.accept_bi().await;
//             let stream = match stream {
//                 Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
//                     println!("connection closed");
//                     return Ok(());
//                 }
//                 Err(e) => {
//                     return Err(e);
//                 }
//                 Ok(s) => s,
//             };
//             let fut = handle_request(stream);
//             tokio::spawn(
//                 async move {
//                     if let Err(e) = fut.await {
//                         println!("failed: {reason}", reason = e.to_string());
//                     }
//                 }
//             );
//         }
//     }
//     .await?;
//     Ok(())
// }



// async fn handle_request(
//     (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
// ) -> Result<()> {
//     let req = recv
//         .read_to_end(64 * 1024)
//         .await
//         .map_err(|e| anyhow!("failed reading request: {}", e))?;
//     let mut escaped = String::new();
//     for &x in &req[..] {
//         let part = ascii::escape_default(x).collect::<Vec<_>>();
//         escaped.push_str(str::from_utf8(&part).unwrap());
//     }
//     // Execute the request
//     let resp = process_get(&req).unwrap_or_else(|e| {
//         println!("failed: {}", e);
//         format!("failed to process request: {e}\n").into_bytes()
//     });
//     // Write the response
//     send.write_all(&resp)
//         .await
//         .map_err(|e| anyhow!("failed to send response: {}", e))?;
//     // Gracefully terminate the stream
//     send.finish().unwrap();
//     println!("complete");
//     Ok(())
// }



// fn process_get(x: &[u8]) -> Result<Vec<u8>> {
//     if x.len() < 4 || &x[0..4] != b"GET " {
//         bail!("missing GET");
//     }
//     if x[4..].len() < 2 || &x[x.len() - 2..] != b"\r\n" {
//         bail!("missing \\r\\n");
//     }
//     let x = &x[4..x.len() - 2];
//     let end = x.iter().position(|&c| c == b' ').unwrap_or(x.len());
//     let path = str::from_utf8(&x[..end]).context("path is malformed UTF-8")?;
//     let path = Path::new(&path);
//     let mut components = path.components();
//     match components.next() {
//         Some(path::Component::RootDir) => {}
//         _ => {
//             bail!("path must be absolute");
//         }
//     }
//     for c in components {
//         match c {
//             path::Component::Normal(x) => {
//                 real_path.push(x);
//             }
//             x => {
//                 bail!("illegal component in path: {:?}", x);
//             }
//         }
//     }
//     let data = fs::read(&real_path).context("failed reading file")?;
//     Ok(data)
// }



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
    config.alpn_protocols = vec![HQ29.to_vec(), HTTP3.to_vec()];

    config
}