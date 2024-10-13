use std::{
    // ascii, fs, io,
    env, io, net::{IpAddr, Ipv4Addr, SocketAddr}, str, sync::Arc
};

// use anyhow::{anyhow, bail, Context, Ok, Result};
// use quinn::crypto::rustls::QuicServerConfig;
// use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use quinn::Endpoint;
// use anyhow::Ok as OkAnyhow;
use rustls::{pki_types, server::ServerConfig};
use std::fs::File;
use std::io::{BufReader, prelude::*};

use h3::{client, error::ErrorLevel, quic::BidiStream, server::{self, RequestStream}};
use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};

use bytes::{BufMut, Bytes, BytesMut};
use http::{header::CONTENT_TYPE, Request, StatusCode};

use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}, stream};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::{Bytes as HyperBytes, Incoming};
use hyper::Method;
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use std::convert::Infallible;
use tokio_rustls::{rustls, TlsAcceptor, TlsConnector};

use serde::Serialize;
// use serde_json;

use rustls::RootCertStore;


use alpn::H2;
use alpn::HTTP1_1;
use alpn::HTTP3;


pub mod alpn {
    pub const H2: &[u8] = b"h2";
    pub const HTTP1_1: &[u8] = b"http/1.1";
    pub const HTTP3: &[u8] = b"h3";
    pub const HQ29: &[u8] = b"hq-29";
}

mod cert_generate_util;


#[derive(Serialize)]
struct JsonMsg {
    message: String,
}


fn read_html_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}



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
    env::set_var("RUST_BACKTRACE", "1");

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("default provider already set elsewhere");
     
    // for tcp usage
    let tcp_tls_acceptor = get_h2_config()?;
    let tcp_listener = TcpListener::bind("127.0.0.1:443").await?;
    println!("Tcp binding finished");
    // tokio::spawn(listen_tcp_request(tcp_listener, tcp_tls_acceptor));


    // set tls for quic
    let server_config = get_h3_config()?;
    // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    // transport_config.max_concurrent_uni_streams(0_u8.into());
    let endpoint = quinn::Endpoint::server(
        server_config,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443),
    )?;
    println!("Quic binding finished");

    // set server
    let http12_task = tokio::spawn(process_tcp_request(tcp_listener, tcp_tls_acceptor));
    tokio::join!(http12_task);
    // let http3_task = tokio::spawn(process_quic_request(endpoint));
    // tokio::join!(http12_task, http3_task);
    
    Ok(())
}



async fn process_tcp_request(
    tcp_listener: TcpListener,
    tls_acceptor: TlsAcceptor
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> 
{
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let acceptor_clone = tls_acceptor.clone();
        tokio::spawn(h2_http_response(tcp_stream, acceptor_clone));
    }
}


async fn process_quic_request(endpoint: Endpoint) {
    while let Some(new_conn) = endpoint.accept().await {
        println!("accepting connection");
        tokio::spawn(h3_handle_connection(new_conn));
    }
    endpoint.wait_idle().await;
}


async fn h2_http_response(tcp_stream: TcpStream, tls_acceptor: TlsAcceptor) {
    match tls_acceptor.accept(tcp_stream).await {
        Ok(tls_accepted) => {
            // let (tcp_io, tls_conn, tls_state) = tls_accepted.in_inner_all();
            // let (_, tls_conn) = tls_accepted.get_ref();
            let Some(alpn) = tls_accepted.get_ref().1.alpn_protocol().map(|x|x.to_vec()) else {
                return
            };
            let io = TokioIo::new(tls_accepted);
            if alpn == alpn::H2 {
                // let tls_stream = TlsStream::new(tcp_io, tls_conn, tls_state);
                // let io = TokioIo::new(tls_stream);
                
                // let _ = http2::Builder::new(TokioExecutor)
                //     .serve_connection(io, service_fn(hello_http1_http2))
                //     .await;
                println!("processed with h2");
                let _ = http2::Builder::new(TokioExecutor)
                    .serve_connection(io, service_fn(handle_proxy_http1_http2))
                    .await;
                
            } else if alpn == alpn::HTTP1_1 {
                // let tls_stream = TlsStream::new(tcp_io, tls_conn, tls_state);
                // let io = TokioIo::new(tls_stream);
                
                println!("processed with h2");
                let _ = http1::Builder::new()
                    .serve_connection(io, service_fn(handle_proxy_http1_http2))
                    .await;
            } else {
                io.into_inner().get_mut().0.shutdown();
            }
        }
        Err(e) => {
            println!("TLS accepted error: {}", e);
        }
    }
}


// http task

fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}

async fn tunnel(
    client_req: HyperRequest<hyper::body::Incoming>,
    server_addr: String,
) -> std::result::Result<HyperResponse<hyper::body::Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    // connection from request
    // let client_io = TokioIo::new(client_io);

    // set to server tls connection
    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    
    // to server tcp, then to server tls

    // let server_addr_port = server_addr.clone() + ":443";
    // hardcode test www.baidu.com, 103.235.47.188
    let server_addr_port = "103.235.47.188:443";
    let connector: TlsConnector = TlsConnector::from(Arc::new(config));
    let domain = pki_types::ServerName::try_from(server_addr)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();

    
    println!("good for domain: {:?}", domain);

    let server_tcp_stream = TcpStream::connect(server_addr_port).await?;
    println!("good for tcp");
    let server_tls_stream = connector.connect(domain, server_tcp_stream).await?;
    println!("good for tls");
    let server_io = TokioIo::new(server_tls_stream);
    // http1
    // let (mut server_sender, _server_conn) = hyper::client::conn::http1::handshake(server_io).await?;
    // http2
    let (mut server_sender, _server_conn) = hyper::client::conn::http2::handshake(TokioExecutor, server_io).await?;
    println!("good for handshake");
    
    // send request to server
    // todo!("a little bit problem, got suspended at client_req");
    // println!("client req: {:?}", client_req);

    let new_req = HyperRequest::builder()
        .method(client_req.method())
        .uri(client_req.uri())
        .version(client_req.version())
        .body(Empty::<Bytes>::new())?;

    let server_res = server_sender.send_request(client_req).await?;
    println!("good for server res");

    Ok(server_res)
}

async fn handle_proxy_http1_http2(
    req: HyperRequest<hyper::body::Incoming>,
) -> std::result::Result<HyperResponse<Incoming>, Box<dyn std::error::Error + Send + Sync>> {

    let dest_addr = host_addr(req.uri()).unwrap();

    // tunneling
    let s = req.headers();
    let b = req.body();

    match tunnel(req, dest_addr).await {
        Ok(server_resp) => {
            println!("proxy received from server");
            // type convert, make valid response
            // let status = server_resp.status();
            let headers = server_resp.headers().clone();

            let mut res = HyperResponse::builder()
                .status(server_resp.status())
                .body(server_resp.into_body())
                .unwrap();
            headers.into_iter().map(|(k, v)| res.headers_mut().insert(k.unwrap(), v) );
            Ok(res)
        },
        Err(e) => { 
            println!("server io error: {}", e);
            Err(e)
        },
    }

}



async fn h3_handle_connection(new_conn: quinn::Incoming) {
    match new_conn.await {
        Ok(conn) => {
            let mut h3_conn: h3::server::Connection<h3_quinn::Connection, Bytes> =
                h3::server::Connection::new(h3_quinn::Connection::new(conn))
                    .await
                    .unwrap();

                loop {
                    match h3_conn.accept().await {
                        Ok(Some((req, stream))) => {
                            println!("new request: {:#?}", req);
    
                            tokio::spawn(async {
                                if let Err(e) = h3_handle_request(req, stream).await {
                                    println!("handling request failed: {}", e);
                                }
                            });
                        }
                        // indicating no more streams to be received
                        Ok(None) => {
                            // break;
                        }
                        Err(err) => {
                            println!("error on accept {}", err);
                            match err.get_error_level() {
                                ErrorLevel::ConnectionError => println!("ConnectionError"),
                                ErrorLevel::StreamError => println!("StreamError"),
                            }
                        }
                    }
                }
                
            // loop {
            //     match h3_conn.accept().await {
            //         Ok(Some((req, stream))) => {
            //             println!("new request: {:#?}", req);

            //             tokio::spawn(async {
            //                 if let Err(e) = handle_request(req, stream).await {
            //                     println!("handling request failed: {}", e);
            //                 }
            //             });
            //         }


            //         // indicating no more streams to be received
            //         Ok(None) => {
            //             break;
            //         }

            //         Err(err) => {
            //             println!("error on accept {}", err);
            //             match err.get_error_level() {
            //                 ErrorLevel::ConnectionError => break,
            //                 ErrorLevel::StreamError => continue,
            //             }
            //         }
            //     }
            // }
        }
        Err(e) => {
            println!("quic connection failed: {}", e)
        }
    }
}


async fn h3_handle_request<T>(
    req: Request<()>,
    mut stream: RequestStream<T, Bytes>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: BidiStream<Bytes>,
{
    match (req.method(), req.uri().path()) {

        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            let resp = HyperResponse::builder()
                .status(200)
                .header(CONTENT_TYPE, "text/html")
                .header("Alt-Svc", "h3=\":443\"; ma=86400")
                .body(())
                .unwrap();

            if let Ok(_) = stream.send_response(resp).await {
                let html = read_html_file("index.html").unwrap();
                let mut buf = BytesMut::with_capacity(4096 * 10);
                buf.put(html.as_slice());
                stream.send_data(buf.freeze()).await?;
                stream.finish().await?;
            }   
        }

        (&Method::GET, "/testmsg") => {
            let resp = HyperResponse::builder()
                .header(CONTENT_TYPE, "application/json")
                .header("Alt-Svc", "h3=\":443\"; ma=86400")
                .body(())
                .unwrap();

            if let Ok(_) = stream.send_response(resp).await {
                let json_data = JsonMsg { message: String::from("Http3 API") };
                let json_body = serde_json::to_string(&json_data).unwrap();
                let mut buf = BytesMut::with_capacity(4096 * 10);
                buf.put(json_body.as_bytes());
                stream.send_data(buf.freeze()).await?; 
                stream.finish().await?;
            }
        }

        _ => {
            let resp = HyperResponse::builder()
                .status(StatusCode::NOT_FOUND)
                .body(())
                .unwrap();
            if let Ok(_) = stream.send_response(resp).await {
                stream.finish().await?;
            }
        }
    }

    Ok(())
}


fn get_h2_config() -> io::Result<TlsAcceptor> {
    // let cert_file = "myservercert.pem";
    // let key_file = "myserverkey.pem";
    // let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file).unwrap()))
    //     .collect::<Result<Vec<_>, _>>()
    //     .unwrap();
    // let private_key = rustls_pemfile::private_key(&mut BufReader::new(
    //     &mut File::open(key_file).unwrap(),
    // ))
    //     .unwrap()
    //     .unwrap();
    // let mut config = ServerConfig::builder()
    //     .with_no_client_auth()
    //     .with_single_cert(certs, private_key)
    //     .unwrap();

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
    // let cert_file = "myservercert.pem";
    // let key_file = "myserverkey.pem";
    // let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file).unwrap()))
    //     .collect::<Result<Vec<_>, _>>()
    //     .unwrap();
    // let private_key = rustls_pemfile::private_key(&mut BufReader::new(
    //     &mut File::open(key_file).unwrap(),
    // ))
    //     .unwrap()
    //     .unwrap();
    // let mut config = ServerConfig::builder()
    //     .with_no_client_auth()
    //     .with_single_cert(certs, private_key)
    //     .unwrap();

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
