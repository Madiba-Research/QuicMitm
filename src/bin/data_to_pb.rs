use std::{io::{Read, Write}, path::Path};

use futures::StreamExt;
// use h3::client::new;
// use hyper_util::server::conn;
use mongodb::{ bson::doc, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client, Collection };
use prost::Message;

use h3server::RecordInMONGODBv2;


include!(concat!(env!("OUT_DIR"), "/data.rs"));


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("need 2 args for the generating csv, the second arg as package name");
        return Ok(());
    }
    if args[2] != "h2" && args[2] != "h2h3" {
        println!("the 3rd arg can only be h2 or h2h3");
        return Ok(());
    }

    let package_name = args[1].clone();
    let with_quic = args[2] == "h2h3";

    let mongodb_uri = "mongodb://localhost:27017";
    let mut client_options = ClientOptions::parse(mongodb_uri).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    let my_coll: Collection<RecordInMONGODBv2> = client.database("requestdb2").collection("httpreq2");
    let mut cursor = my_coll
        .find(doc! {
            "app": &package_name,
            "withquic": with_quic,
        }).await?;

    // convert to pb message
    // prepare the NetworkData pb
    let mut pkg_name: String = "".to_string();
    let mut data_timestamp: u64 = 0;
    let mut record_vec: Vec<Record> = Vec::new();

    while let Some(Ok(record)) = cursor.next().await {
        
        // connection info pb
        let conn_info = record.conn_info_v2;
        let conn_info_pb = ConnectionInfo {
            source_address: conn_info.source_addr,
            destination_address: conn_info.dest_addr,
            tls: conn_info.is_tls,
            timestamp: conn_info.timestamp,
        };


        // solve mitm decode
        let mut req = record.request_v2;

        // request pb
        if req.header.contains_key("content-encoding") {
            let encoding = req.header.get("content-encoding").unwrap().to_lowercase();

            if encoding == "gzip" {
                let body_clone = req.body.clone();
                let mut decoder = flate2::read::GzDecoder::new(&body_clone[..]);
                let mut text = vec![];
                if let Ok(_) = decoder.read_to_end(&mut text) {
                    req.body = text;
                    req.header.remove_entry("content-encoding");
                }
            } else if encoding == "deflate" {
                let body_clone = req.body.clone();
                let mut decoder = flate2::read::DeflateDecoder::new(&body_clone[..]);
                let mut text = vec![];
                if let Ok(_) = decoder.read_to_end(&mut text) {
                    req.body = text;
                    req.header.remove_entry("content-encoding");
                }
            } else if encoding == "identity" {
                req.header.remove_entry("content-encoding");
            } else if encoding == "br" {
                let body_clone = req.body.clone();
                let mut decoder = brotli_decompressor::Decompressor::new(&body_clone[..], 4096);
                let mut text = vec![];
                if let Ok(_) = decoder.read_to_end(&mut text) {
                    req.body = text;
                    req.header.remove_entry("content-encoding");
                }
            } else if encoding == "zstd" {
                let body_clone = req.body.clone();
                if let Ok(mut decoder) = zstd::stream::read::Decoder::new(&body_clone[..]) {
                    let mut text = vec![];
                    if let Ok(_) = decoder.read_to_end(&mut text) {
                        req.body = text;
                        req.header.remove_entry("content-encoding");
                    }
                };
            } else {
                continue;
            }
        }

        let mut req_headers_pb = Vec::new();
        for (k, v) in req.header {
            let h_pb = Header { key: k, value: v };
            req_headers_pb.push(h_pb);
        }
        let req_trailers_pb: Vec<Header> = Vec::new();

        let req_pb = Request {
            method: req.method,
            path: req.path,
            headers: req_headers_pb,
            trailers: req_trailers_pb,
            body: req.body,
        };

        // make record_pb
        let websocket_pb: Vec<WebSocketMessage> = Vec::new();
        let record_pb = Record {
            request: Some(req_pb),
            response: None,
            websocket: websocket_pb,
            connection_info: Some(conn_info_pb),
        };

        // update NetworkData data
        pkg_name = record.app;
        data_timestamp = record.time_stamp;
        record_vec.push(record_pb);
    }

    // let session_id = data_timestamp.clone();
    let pkg = pkg_name.clone();

    // make NetworkData pb
    let network_data_pb = NetworkData {
        pkg_name,
        records: record_vec,
        timestamp: data_timestamp,
    };

    let _ = std::fs::create_dir_all("pbdata")?;
    let app_events = format!("pbdata/{}-{}.pb", pkg, args[2]);
    let app_events_path = Path::new(&app_events);
    
    // write to pb file
    let file = std::fs::File::create(&app_events_path)?;
    let mut buf = Vec::new();
    let _ = network_data_pb.encode(&mut buf)?;
    let mut writer = std::io::BufWriter::new(file);
    let _ = writer.write_all(&buf)?;

    let _ = writer.flush()?;
    

    Ok(())
}