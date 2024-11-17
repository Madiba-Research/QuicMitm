use std::{io, net::{IpAddr, Ipv4Addr, SocketAddr}, str::FromStr, sync::Arc};

use futures::future;
use h3::quic::BidiStream;

use h3server::{headers_to_hashmap, version_to_string, RequestInMONGO};
// use h3server::create_http_request_type;
use http::{request::Parts, Response};
use http_body_util::BodyExt;
// use http_body_util::Full;
use hyper::{server::conn::http1, server::conn::http2, service::service_fn};
use hyper_util::rt::TokioIo;
use mongodb::{options::{ClientOptions, ServerApi, ServerApiVersion}, Client, Database};
// use prost::Message;
use quinn::{crypto::rustls::QuicServerConfig, Endpoint, Incoming};
use rustls::{pki_types::{self}, RootCertStore, ServerConfig};

use tokio::{net::{TcpListener, TcpStream}, sync::OnceCell};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio::io::{split, copy};

use bytes::{Buf, Bytes};
use http_body_util::Full;


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



static MONGO_CLIENT: OnceCell<Arc<Client>> = OnceCell::const_new();

async fn init_mongo_client() -> Arc<Client> {
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse options");
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    let client = Client::with_options(client_options).expect("Failed to create client");
    Arc::new(client)
}

async fn get_mongo_client() -> Arc<Client> {
    // `get_or_init` will call `init_mongo_client` only once, even if accessed from multiple tasks.
    MONGO_CLIENT.get_or_init(init_mongo_client).await.clone()
}

async fn get_database() -> Database {
    get_mongo_client().await.database("requestdb")
}

static USING_QUIC: OnceCell<bool> = OnceCell::const_new();

static PACKAGE_NAME: OnceCell<String> = OnceCell::const_new();




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("need 3 args for the program");
        return Ok(());
    }

    if args[1] != "h2" && args[1] != "h2h3" {
        println!("the second arg cannot be other than 'h2' and 'h2h3'");
        return Ok(());
    }

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("default provider already set elsewhere");


    // for tcp usage
    let tcp_tls_acceptor = get_h2_config()?;

    // let tcp_listener = TcpListener::bind("127.0.0.1:443").await?;
    let tcp_listener = TcpListener::bind("172.30.143.95:443").await?;
    println!("Tcp binding finished");


    PACKAGE_NAME.set(args[2].to_string()).expect("Failed to set package_name for current proxy work");

    if args[1] == "h2h3" {
        USING_QUIC.set(true)?;
        // set tls for quic
        let server_config = get_h3_config()?;
        let endpoint = quinn::Endpoint::server(
            server_config,
            // SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 30, 143, 95)), 443),
        )?;
        println!("Quic binding finished");

        // set server
        let tcp_tls_task = tokio::spawn(process_tcp_request(tcp_listener, tcp_tls_acceptor));
        let quic_task = tokio::spawn(process_quic_request(endpoint));
        let _ = tokio::join!(tcp_tls_task, quic_task);
    } else {
        USING_QUIC.set(false)?;
        let tcp_tls_task = tokio::spawn(process_tcp_request(tcp_listener, tcp_tls_acceptor));
        let _ = tcp_tls_task.await;
    }


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
                    ::<TokioExecutor, TokioIo<tokio_rustls::client::TlsStream<TcpStream>>, Full<Bytes>>(TokioExecutor, server_io)
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
    mut server_send: hyper::client::conn::http2::SendRequest<Full<Bytes>>,
) -> Result<hyper::Response<hyper::body::Incoming>, Box<dyn std::error::Error + Sync + Send>> {

    // Result<hyper::Response<hyper::body::Incoming>, Box<dyn std::error::Error + Sync + Send>>
    // Result<hyper::Response<Full<Bytes>>, Box<dyn std::error::Error + Sync + Send>>
    
    // write into mongodb
    let (req_parts, req_body) = client_req.into_parts();

    let req_version = version_to_string(&req_parts.version);
    let req_hash_header = headers_to_hashmap(&req_parts.headers);
    let req_body_byte = req_body.collect().await?.to_bytes();
    let req_body_vec = req_body_byte.to_vec();

    // deal with data storage field
    let mut package_name = String::new();
    if let Some(p) = PACKAGE_NAME.get() {
        package_name = p.clone();
    }

    let using_quic = USING_QUIC.get().expect("cannot decide USING_H3");

    let doc = RequestInMONGO {
        _id: None,
        app: package_name.to_string(),
        withquic: using_quic.clone(),
        uri: req_parts.uri.to_string(),
        method: req_parts.method.to_string(),
        version: req_version,
        header: req_hash_header,
        body: req_body_vec,
        
        bodytype: None,
        bodyplaintext: None,
    };

    println!("in connecting db h2");
    let db_col = get_database().await.collection("httpreq");
    match db_col.insert_one(doc).await {
        Ok(rst) => { println!("insert rst: {:?}", rst) },
        Err(e) => { println!("insert err: {}", e) },
    };

    let req_to_server = hyper::Request::from_parts(req_parts, Full::new(req_body_byte));
    // println!("request sent h2");
    // println!("{:?}", req_to_server);

    let server_resp = server_send.send_request(req_to_server).await?;
    // println!("resp h2");
    // println!("{:?}", server_resp);

    Ok(server_resp)

    // let server_resp = server_send.send_request(req_to_server).await?;
    // let server_resp = server_resp.into_parts();
    // let res_collected = server_resp.1.collect().await?;
    // let res_bytes = res_collected.to_bytes();
    // println!("h2 response body: {:?}", String::from_utf8(res_bytes.to_vec()));
    // let new_server_resp = Response::from_parts(server_resp.0, Full::new(res_bytes));

    // Ok(new_server_resp)
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

    // check request, write into mongodb
    let (req_parts, req_body) = client_req.into_parts();

    let req_version = version_to_string(&req_parts.version);
    let req_hash_header = headers_to_hashmap(&req_parts.headers);
    let req_body_byte = req_body.collect().await?.to_bytes();
    let req_body_vec = req_body_byte.to_vec();

    // deal with data storage field
    let mut package_name = String::new();
    if let Some(p) = PACKAGE_NAME.get() {
        package_name = p.clone();
    }

    let using_quic = USING_QUIC.get().expect("cannot decide USING_H3");

    let doc = RequestInMONGO {
        _id: None,
        app: package_name.to_string(),
        withquic: using_quic.clone(),
        uri: req_parts.uri.to_string(),
        method: req_parts.method.to_string(),
        version: req_version,
        header: req_hash_header,
        body: req_body_vec,
        
        bodytype: None,
        bodyplaintext: None,
    };

    println!("in connecting db h1");
    let db_col = get_database().await.collection("httpreq");
    match db_col.insert_one(doc).await {
        Ok(rst) => { println!("insert rst: {:?}", rst) },
        Err(e) => { println!("insert err: {}", e) },
    };


    // let req_body_vec = req_body_byte.to_vec();
    // println!("h1 request part: {:?}\n h1 request body: {:?}", req_parts, String::from_utf8_lossy(&req_body_vec));

    let req_to_server = hyper::Request::from_parts(req_parts, Full::new(req_body_byte));
    // println!("request sent h1");
    // println!("{:?}", req_to_server);
    
    let server_resp = server_sender.send_request(req_to_server).await?;
    // println!("resp h1");
    // println!("{:?}", server_resp);


    Ok(server_resp)
    
}




// async fn proxy_tcp_tls_naive(
//     tcp_stream: TcpStream,
//     tls_acceptor: TlsAcceptor
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     match tls_acceptor.accept(tcp_stream).await {
//         Ok(client_tls_stream) => {
//             // obtain domain name from tls connection
//             let server_name_option = client_tls_stream.get_ref().1.server_name();

//             if let Some(server_name) = server_name_option {
//                 // println!("tcp_tls to server name: {}", server_name);
                
//                 // build connection to server
//                 let root_store = RootCertStore {
//                     roots: webpki_roots::TLS_SERVER_ROOTS.into(),
//                 };
//                 let mut proxy_config = rustls::ClientConfig::builder()
//                     .with_root_certificates(root_store)
//                     .with_no_client_auth();
//                 proxy_config.alpn_protocols= vec![H2.to_vec(), HTTP1_1.to_vec()];

//                 let server_port = String::from_str(server_name)? + ":443";
//                 let proxy_connector = TlsConnector::from(Arc::new(proxy_config));
//                 let server_domain = pki_types::ServerName::try_from(server_name)
//                     .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
//                     .to_owned();

//                 let server_tcp_stream = TcpStream::connect(server_port.clone()).await?;
//                 let server_tls_stream = proxy_connector.connect(server_domain, server_tcp_stream).await?;

//                 // wire the client-proxy, and proxy-server
//                 let (mut to_client_read, mut to_client_write) = split(client_tls_stream);
//                 let (mut to_server_read, mut to_server_write) = split(server_tls_stream);

//                 let upload_fut = copy(&mut to_client_read, &mut to_server_write);
//                 let download_fut = copy(&mut to_server_read, &mut to_client_write);

//                 let _ = tokio::join!(upload_fut, download_fut);

//                 // println!("tcp tls stream done");
//             }
//         }

//         Err(e) => { println!("TLS accepted error on tcp: {}", e); }
//     }

//     Ok(())
// }


async fn process_quic_request(endpoint: Endpoint) {
    while let Some(new_conn) = endpoint.accept().await {
        println!("quic accepting connection");
        tokio::spawn(proxy_quic_connection(new_conn));
    }
    endpoint.wait_idle().await;
}


async fn proxy_quic_connection(conn: Incoming) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let proxy_conn = conn.await?;

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
    let proxy_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(proxy_config)?,
    ));
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
        // println!("h3 client request: {:?}", client_req_stream.0);
        let req_parts = client_req_stream.0.clone().into_parts().0;

        // println!("h3 send request");
        // println!("{:?}", client_req_stream.0);

        let req_server_stream = send_request.send_request(client_req_stream.0).await?;
        
        tokio::spawn(
            handle_tunnel_stream(client_req_stream.1, req_server_stream, req_parts)
        );
    }

    let _ = drive.await;
    
    // loop {
    //     let client_stream = proxy_conn.accept_bi().await?;
    //     let server_stream = server_conn.open_bi().await?;
    //     tokio::spawn(handle_bi(client_stream, server_stream));
    // }
    Ok(())
}

async fn handle_tunnel_stream<T>(
    mut to_client_stream: h3::server::RequestStream<T, Bytes>,
    mut to_server_stream: h3::client::RequestStream<T, Bytes>,
    client_req_parts: Parts,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>>
where T: BidiStream<Bytes> {

    let mut req_body_vec = vec![];

    while let Some(mut chunk) = to_client_stream.recv_data().await? {
        let data = chunk.copy_to_bytes(chunk.remaining());
        req_body_vec.append(&mut data.to_vec());
        to_server_stream.send_data(data).await?;
    }
    to_server_stream.finish().await?;

    
    // write into mongodb
    let req_version = version_to_string(&client_req_parts.version);
    let req_hash_header = headers_to_hashmap(&client_req_parts.headers);

    // deal with data storage field
    let mut package_name = String::new();
    if let Some(p) = PACKAGE_NAME.get() {
        package_name = p.clone();
    }

    let using_quic = USING_QUIC.get().expect("cannot decide USING_H3");
    

    let doc = RequestInMONGO {
        _id: None,
        app: package_name.to_string(),
        withquic: using_quic.clone(),
        uri: client_req_parts.uri.to_string(),
        method: client_req_parts.method.to_string(),
        version: req_version,
        header: req_hash_header,
        body: req_body_vec,
        
        bodytype: None,
        bodyplaintext: None,
    };

    println!("in connecting db h3");
    let db_col = get_database().await.collection("httpreq");
    match db_col.insert_one(doc).await {
        Ok(rst) => { println!("insert rst: {:?}", rst) },
        Err(e) => { println!("insert err: {}", e) },
    };


    // let req_proto = create_http_request_type(
    //     client_req_parts.uri.to_string(),
    //     client_req_parts.method.to_string(),
    //     format!("{:?}", client_req_parts.version),
    //     format!("{:?}", client_req_parts.headers),

    //     Bytes::from(req_body_vec)
    // );
    // let req_proto_dump = req_proto.encode_to_vec();
    // global_write_file(req_proto_dump).await?;

    let server_resp = to_server_stream.recv_response().await?;

    // println!("resp h3");
    // println!("{:?}", server_resp);

    to_client_stream.send_response(server_resp).await?;

    

    while let Some(mut chunk) = to_server_stream.recv_data().await? {
        let data = chunk.copy_to_bytes(chunk.remaining());
        // println!("h3 received data from server: {}", String::from_utf8_lossy(data.clone().chunk()));
        to_client_stream.send_data(data).await?;
    }
    to_client_stream.finish().await?;

    Ok(())
}



// async fn handle_bi(
//     client_stream: (SendStream, RecvStream),
//     server_stream: (SendStream, RecvStream),
// ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {

//     let (mut to_client_send, mut to_client_recv) = client_stream;
//     let (mut to_server_send, mut to_server_recv) = server_stream;

//     let upload_task = handle_uni(to_client_recv, to_server_send, false);
//     let download_task = handle_uni(to_server_recv, to_client_send, false);

//     let _ = tokio::join!(upload_task, download_task);
//     Ok(())
// }


// async fn handle_uni(
//     mut recv_stream: RecvStream,
//     mut send_stream: SendStream,
//     print_data: bool
// ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
//     let recv_id = recv_stream.id();
//     let send_id = send_stream.id();

//     // if not work, try read and write instead of read_chunk and write_chunk
//     while let Some(chunk) = recv_stream.read_chunk(usize::MAX, true).await? {
        
//         if print_data {
//             let readable = chunk.bytes.to_vec();
//             println!("quic data from {} to {}: {} bytes", recv_id, send_id, readable.len());
//         };
        
//         match send_stream.write_chunk(chunk.bytes).await {
//             Ok(_) => { println!("uni write good"); },
//             Err(e) => {println!("write error: {}", e)},
//         };
//     }

//     send_stream.finish()?;
//     println!("stream ends");
//     Ok(())
// }




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

