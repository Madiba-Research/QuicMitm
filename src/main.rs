use std::{
    // ascii, fs, io,
    env, error, io::{self, ErrorKind}, net::{IpAddr, Ipv4Addr, SocketAddr}, str::{self, FromStr}, sync::Arc
};

// use anyhow::{anyhow, bail, Context, Ok, Result};
// use quinn::crypto::rustls::QuicServerConfig;
// use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use futures::future;
use quinn::Endpoint;
// use anyhow::Ok as OkAnyhow;
use rustls::{pki_types, server::ServerConfig};
use std::fs::File;
use std::io::{BufReader, prelude::*};

use h3::{client, error::ErrorLevel, quic::BidiStream, server::{self, RequestStream}};
use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use http::{header::CONTENT_TYPE, Request, StatusCode};

use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}, stream};
use hyper_util::rt::TokioIo;
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Empty, Full};
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


fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
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
    // let tcp_listener = TcpListener::bind("127.0.0.1:443").await?;
    let tcp_listener = TcpListener::bind("172.30.143.77:443").await?;
    println!("Tcp binding finished");
    // tokio::spawn(listen_tcp_request(tcp_listener, tcp_tls_acceptor));


    // set tls for quic
    let server_config = get_h3_config()?;

    // let mut server_config = get_h3_config()?;
    // let mut transport_config = quinn::TransportConfig::default();
    // transport_config.max_idle_timeout(None);
    // let transport_config = Arc::new(transport_config);
    // server_config.transport_config(transport_config);

    // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    // transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = quinn::Endpoint::server(
        server_config,
        // SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 30, 143, 77)), 443),
    )?;
    println!("Quic binding finished");

    // set server
    let http12_task = tokio::spawn(process_tcp_request(tcp_listener, tcp_tls_acceptor));
    // tokio::join!(http12_task);
    let http3_task = tokio::spawn(process_quic_request(endpoint));
    tokio::join!(http12_task, http3_task);
    
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


async fn h2_http_response(tcp_stream: TcpStream, tls_acceptor: TlsAcceptor) {
    match tls_acceptor.accept(tcp_stream).await {
        Ok(tls_accepted) => {

            let Some(alpn) = tls_accepted.get_ref().1.alpn_protocol()
                .map(|x|x.to_vec()) else { return };

            let io = TokioIo::new(tls_accepted);
            if alpn == alpn::H2 {

                println!("processed with h2");
                let _ = http2::Builder::new(TokioExecutor)
                    .serve_connection(io, service_fn(handle_proxy_http2))
                    .await;
                
            } else if alpn == alpn::HTTP1_1 {
                
                println!("processed with h1.1");
                let _ = http1::Builder::new()
                    .serve_connection(io, service_fn(handle_proxy_http1))
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



async fn handle_proxy_http2(
    req: HyperRequest<hyper::body::Incoming>,
) -> std::result::Result<HyperResponse<Incoming>, Box<dyn std::error::Error + Send + Sync>> {

    let dest_addr = host_addr(req.uri()).unwrap();

    // tunneling
    match tunnel_http2(req, dest_addr).await {
        Ok(server_resp) => {
            println!("proxy received from server");
            
            Ok(server_resp)

            // let headers = server_resp.headers().clone();

            // let mut res = HyperResponse::builder()
            //     .status(server_resp.status())
            //     .body(server_resp.into_body())
            //     .unwrap();
            // headers.into_iter().map(|(k, v)| res.headers_mut().insert(k.unwrap(), v) );
            // Ok(res)
        },
        Err(e) => { 
            println!("server io error: {}", e);
            Err(e)
        },
    }
}


async fn handle_proxy_http1(
    req: HyperRequest<hyper::body::Incoming>,
) -> std::result::Result<HyperResponse<Incoming>, Box<dyn std::error::Error + Send + Sync>> {

    let dest_domain = host_addr(req.uri()).unwrap();

    // tunneling
    match tunnel_http1(req, dest_domain).await {
        Ok(server_resp) => {
            println!("proxy received from server");
            
            Ok(server_resp)

            // let headers = server_resp.headers().clone();

            // let mut res = HyperResponse::builder()
            //     .status(server_resp.status())
            //     .body(server_resp.into_body())
            //     .unwrap();
            // headers.into_iter().map(|(k, v)| res.headers_mut().insert(k.unwrap(), v) );
            // Ok(res)
        },
        Err(e) => { 
            println!("server io error: {}", e);
            Err(e)
        },
    }
}


async fn tunnel_http2(
    client_req: HyperRequest<hyper::body::Incoming>,
    server_domain: String,
) -> std::result::Result<HyperResponse<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
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

    // hardcode test www.google.com, 172.217.13.196
    // hardcode test www.baidu.com 103.235.46.96
    // let server_addr_port = "172.217.13.174:443";
    let server_addr_port = server_domain.clone() + ":443";

    let connector: TlsConnector = TlsConnector::from(Arc::new(config));
    let domain = pki_types::ServerName::try_from(server_domain)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();

    let server_tcp_stream = TcpStream::connect(server_addr_port).await?;

    let server_tls_stream = connector.connect(domain, server_tcp_stream).await?;

    let server_io = TokioIo::new(server_tls_stream);
    // todo!("there is a weird problem: i am accepting http2 as the alpn, but the hyper handshake only allow http1");
    
    // http1
    let (mut server_sender, server_conn) = hyper::client::conn::http1::handshake(server_io).await?;
    // http2
    // let (mut server_sender, server_conn) = hyper::client::conn::http2::handshake(TokioExecutor, server_io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = server_conn.await {
            println!("Connection failed: {:?}", err);
        }
    });


    // let new_req = HyperRequest::builder()
    //     .method(client_req.method())
    //     .uri(client_req.uri())
    //     .version(client_req.version())
    //     .body(Empty::<Bytes>::new())?;

    let server_res = server_sender.send_request(client_req).await?;
    // todo!("convert the incoming to bytes");
    // let res_collected = server_res.into_body().collect().await?;
    // let res_bytes = res_collected.to_bytes();
    
    println!("good for server res");

    Ok(server_res)
}


async fn tunnel_http1(
    client_req: HyperRequest<hyper::body::Incoming>,
    server_domain: String,
) -> std::result::Result<HyperResponse<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
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
    // hardcode test www.google.com, 172.217.13.196
    // hardcode test www.baidu.com 103.235.46.96
    // let server_addr_port = "172.217.13.174:443";

    let server_addr_port = server_domain.clone() + ":443";

    let connector: TlsConnector = TlsConnector::from(Arc::new(config));
    let domain = pki_types::ServerName::try_from(server_domain)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();


    let server_tcp_stream = TcpStream::connect(server_addr_port).await?;

    let server_tls_stream = connector.connect(domain, server_tcp_stream).await?;

    let server_io = TokioIo::new(server_tls_stream);
    // http1
    let (mut server_sender, server_conn) = hyper::client::conn::http1::handshake(server_io).await?;
    // http2
    // let (mut server_sender, server_conn) = hyper::client::conn::http2::handshake(TokioExecutor, server_io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = server_conn.await {
            println!("Connection failed: {:?}", err);
        }
    });


    // let new_req = HyperRequest::builder()
    //     .method(client_req.method())
    //     .uri(client_req.uri())
    //     .version(client_req.version())
    //     .body(Empty::<Bytes>::new())?;

    let server_res = server_sender.send_request(client_req).await?;
    // todo!("convert the incoming to bytes");
    // let res_collected = server_res.into_body().collect().await?;
    // let res_bytes = res_collected.to_bytes();
    
    println!("good for server res");

    Ok(server_res)
}



async fn process_quic_request(endpoint: Endpoint) {
    while let Some(new_conn) = endpoint.accept().await {
        println!("accepting connection");
        tokio::spawn(h3_handle_connection(new_conn));
    }
    endpoint.wait_idle().await;
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
                            // println!("new request: {:#?}", req);
                            println!("new request on h3");
    
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
                            break;
                        }
                    }
                }
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
    let domain_dest = host_addr(req.uri()).unwrap();

    // tunneling
    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config.enable_early_data = true;
    config.alpn_protocols = vec![HTTP3.to_vec()];

    // let mut proxy_endpoint = h3_quinn::quinn::Endpoint::client("[::]:0".parse().unwrap())?;
    let mut proxy_endpoint = h3_quinn::quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    let config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(config)?,
    ));

    proxy_endpoint.set_default_client_config(config);

    // hardcode test www.google.com, 172.217.13.174
    // hardcode test www.baidu.com 103.235.46.96

    let domain = pki_types::ServerName::try_from(domain_dest)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();
    let domain_str = domain.clone().to_str().into_owned();


    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 217, 13, 174)), 443);
    // let dest_addr = host_addr(req.uri()).unwrap();
    let conn = proxy_endpoint.connect(server_addr, &domain_str)?.await?;

    println!("Proxy to server connection established");

    let quinn_conn = h3_quinn::Connection::new(conn);
    
    let (mut driver, mut send_request) = h3::client::new(quinn_conn).await?;

    let _drive = async move {
        future::poll_fn(|cx| driver.poll_close(cx)).await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let mut server_stream = send_request.send_request(req).await?;

    server_stream.finish().await?;

    println!("proxy receiving response ...");

    let resp = server_stream.recv_response().await?;

    stream.send_response(resp).await?;

    while let Some(mut chunk) = server_stream.recv_data().await? {
        // let mut out = tokio::io::stdout();
        // out.write_all_buf(&mut chunk).await?;
        // out.flush().await?;
        stream.send_data(chunk.copy_to_bytes(chunk.remaining())).await?;
    }

    // server_stream.finish().await?;

    // stream.finish().await?;

    // println!("stream finished");

    Ok(())

    // ===================================================================================
    // match (req.method(), req.uri().path()) {

    //     (&Method::GET, "/") | (&Method::GET, "/index.html") => {
    //         let resp = HyperResponse::builder()
    //             .status(200)
    //             .header(CONTENT_TYPE, "text/html")
    //             .header("Alt-Svc", "h3=\":443\"; ma=86400")
    //             .body(())
    //             .unwrap();

    //         if let Ok(_) = stream.send_response(resp).await {
    //             let html = read_html_file("index.html").unwrap();
    //             let mut buf = BytesMut::with_capacity(4096 * 10);
    //             buf.put(html.as_slice());
    //             stream.send_data(buf.freeze()).await?;
    //             stream.finish().await?;
    //         }   
    //     }

    //     (&Method::GET, "/testmsg") => {
    //         let resp = HyperResponse::builder()
    //             .header(CONTENT_TYPE, "application/json")
    //             .header("Alt-Svc", "h3=\":443\"; ma=86400")
    //             .body(())
    //             .unwrap();

    //         if let Ok(_) = stream.send_response(resp).await {
    //             let json_data = JsonMsg { message: String::from("Http3 API") };
    //             let json_body = serde_json::to_string(&json_data).unwrap();
    //             let mut buf = BytesMut::with_capacity(4096 * 10);
    //             buf.put(json_body.as_bytes());
    //             stream.send_data(buf.freeze()).await?; 
    //             stream.finish().await?;
    //         }
    //     }

    //     _ => {
    //         let resp = HyperResponse::builder()
    //             .status(StatusCode::NOT_FOUND)
    //             .body(())
    //             .unwrap();
    //         if let Ok(_) = stream.send_response(resp).await {
    //             stream.finish().await?;
    //         }
    //     }
    // }
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
