use std::sync::Arc;

use bytes::Bytes;
use h3::client::SendRequest;
use http::Version;
use http_body_util::{BodyExt, Empty};
use hyper::body::{Body, Incoming};
use hyper::Request;
use hyper_util::rt::TokioIo;
use rustls::{pki_types, RootCertStore};
use tokio::io;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;


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


type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[tokio::main]
async fn main() -> Result<()> {

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("default provider already set elsewhere");

    let url = "www.google.com";

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    
    // this is very import, dont forget to set alpn for proxy to the server
    config.alpn_protocols = vec![b"h2".to_vec()];

    let connector = TlsConnector::from(Arc::new(config));
    let domain = pki_types::ServerName::try_from(url)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();

    let server_tcp_stream = TcpStream::connect("www.google.com:443").await?;

    let server_tls_stream = connector.connect(domain, server_tcp_stream).await?;

    // server_tls_stream.write_all(concat!(
    //     "GET / HTTP/1.1\r\n",
    //     "Host: www.baidu.com\r\n",
    //     "Connection: close\r\n",
    //     "Accept-Encoding: gzip, deflate, br, zstd\r\n",
    //     "\r\n"
    // ).as_bytes()).await?;

    // let (mut reader, mut writer) = split(server_tls_stream);
    // let (mut stdin, mut stdout) = (tokio_stdin(), tokio_stdout());
    // tokio::select! {
    //     ret = copy(&mut reader, &mut stdout) => {
    //         ret?;
    //     },
    //     ret = copy(&mut stdin, &mut writer) => {
    //         ret?;
    //         writer.shutdown().await?
    //     }
    // }

    let server_io = TokioIo::new(server_tls_stream);

    // let executor = hyper_util::rt::tokio::TokioExecutor::new();

    let (mut sender, conn) = hyper::client::conn::http2::handshake(TokioExecutor, server_io).await?;
    // let (mut sender, conn) = hyper::client::conn::http1::handshake(server_io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let req = Request::builder()
        .uri("https://www.google.com")
        .header("user-agent", "hyper-client-http2")
        .version(hyper::Version::HTTP_2)
        .body(Empty::<Bytes>::new())?;

    println!("req: {:?}", req);

    let res = sender.send_request(req).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    let collected_body = res.into_body().collect().await?;
    let body = collected_body.to_bytes();

    print!("Body: {}", String::from_utf8_lossy(&body));

    Ok(())
}


// use core::panic;
// use http_body_util::{combinators::BoxBody, BodyExt, Empty};
// use hyper::body::Bytes;
// use hyper::Request;
// use hyper_util::rt::TokioIo;
// use std::{net::ToSocketAddrs, sync::Arc};
// use tokio::net::TcpStream;
// use tokio_rustls::{rustls, TlsConnector};

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Set log level to TRACE to see detailed information
//     rustls::crypto::ring::default_provider()
//         .install_default()
//         .expect("default provider already set elsewhere");

//     let domain = "www.google.com";
//     let uri = format!("https://{}/", domain);
//     let target_host = match format!("{}:443", domain).to_socket_addrs() {
//         Ok(socket_ip) => socket_ip.into_iter().next().unwrap(),
//         Err(e) => {
//             panic!("DNS resolution error: {}", e);
//         }
//     };

//     let mut root_cert_store = rustls::RootCertStore::empty();
//     root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

//     let mut config = rustls::ClientConfig::builder()
//         .with_root_certificates(root_cert_store)
//         .with_no_client_auth();
//     // config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
//     config.alpn_protocols = vec![b"h2".to_vec()];

//     let connector = TlsConnector::from(Arc::new(config));
//       let tcp_stream = TcpStream::connect(target_host).await?;
//     let tls_domain = rustls_pki_types::ServerName::try_from(domain)
//         .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid dnsname"))?
//         .to_owned();

//     let stream = connector.connect(tls_domain, tcp_stream).await?;
//     let io = TokioIo::new(stream);
//     let executor = hyper_util::rt::tokio::TokioExecutor::new();

//     let (mut sender, conn) = hyper::client::conn::http2::handshake(executor, io).await?;

//     tokio::task::spawn(async move {
//         if let Err(e) = conn.await {
//             println!("Error: {:?}", e);
//         }
//     });

//     let upstream_request = Request::builder()
//         .uri(uri)
//         .header("user-agent", "hyper-client-http2")
//         .version(hyper::Version::HTTP_2)
//         .body(Empty::<Bytes>::new())?;

//     let res = sender.send_request(upstream_request).await?;


//     let body = res.collect().await?.to_bytes();

//     println!("{}", String::from_utf8_lossy(&body));
//     Ok(())
// }