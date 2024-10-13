use std::sync::Arc;

use bytes::Bytes;
use h3::client::SendRequest;
use http::Version;
use http_body_util::{BodyExt, Empty};
use hyper::body::{Body, Incoming};
use hyper::Request;
use hyper_util::rt::TokioIo;
use rustls::{pki_types, RootCertStore};
use tokio::io::{self, copy, split, stdin as tokio_stdin, stdout as tokio_stdout, AsyncWriteExt};
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

    let url = "www.baidu.com";

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let domain = pki_types::ServerName::try_from(url)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();

    let server_tcp_stream = TcpStream::connect("www.baidu.com:443").await?;
   
    let mut server_tls_stream = connector.connect(domain, server_tcp_stream).await?;

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
    // baidu.com only works on http1
    // let (mut sender, conn) = hyper::client::conn::http2::handshake(TokioExecutor, server_io).await?;
    let (mut sender, conn) = hyper::client::conn::http1::handshake(server_io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let req = Request::builder()
        .method("GET")
        .version(Version::HTTP_11)
        .header("Host", "www.baidu.com")
        .header("Connection", "close")
        .header("Accept-Encoding", "gzip, deflate, br, zstd")
        .body(Empty::<Bytes>::new())?;

    // println!("req: {:?}", req);

    let res = sender.send_request(req).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    Ok(())
}
