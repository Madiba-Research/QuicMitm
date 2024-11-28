use futures::StreamExt;
use h3server::RequestInMONGO;
use mongodb::{ bson::doc, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client, Collection };

use std::{fs::OpenOptions, io::{BufWriter, Write}};
// use serde_json;

use prost::Message;


include!(concat!(env!("OUT_DIR"), "/data.rs"));


// take raw data into protobuf: .pb file
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("need 2 args for the generating csv, the second arg as package name");
        return Ok(());
    }
    let package_name = args[1].clone();


    let uri = "mongodb://localhost:27017";
    let mut client_options = ClientOptions::parse(uri).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    let my_coll: Collection<RequestInMONGO> = client.database("requestdb").collection("httpreq");

    let mut cursor = my_coll
        .find(doc! {
            "app": &package_name
        }).await?;


    // follow the step from backchecker: preprocessing.py: line 176
    let data_dir = std::path::Path::new("pbdata");
    // Create the data directory if it doesn't exist
    if !data_dir.exists() {
        std::fs::create_dir(data_dir)?;
    }
    let pb_name = format!("{}.pb", &package_name);
    let file_path = data_dir.join(pb_name);
    let pb_file = OpenOptions::new()
        .write(true) // Open in write mode
        .create(true) // Create the file if it does not exist
        .append(true) // Append to the file, instead of overwriting
        .open(file_path)?;
    let mut writer = BufWriter::new(pb_file);


    while let Some(Ok(req)) = cursor.next().await {

        let req_method = req.method;
        let req_uri = req.uri;
        let req_headers = req.header;
        let req_body = req.body;

        let record_tls = req.tls;
        let record_timestamp = req.time;

        let mut req_headers_pb = Vec::new();
        for (k, v) in req_headers {
            let h_pb = Header { key: k, value: v };
            req_headers_pb.push(h_pb);
        }

        let req_trailers_pb: Vec<Header> = Vec::new();

        let request_pb = Request {
            method: req_method,
            path: req_uri,
            headers: req_headers_pb,
            trailers: req_trailers_pb,
            body: req_body,
        };

        let record_websocket: Vec<WebSocketMessage> = Vec::new();

        let conn_info_pb = ConnectionInfo {
            source_address: "None".to_string(),
            destination_address: "None".to_string(),
            tls: record_tls,
            timestamp: record_timestamp,
        };

        let record_pb = Record {
            request: Some(request_pb),
            response: None,
            websocket: record_websocket,
            connection_info: Some(conn_info_pb),
        };

        // write into file
        let mut buf = Vec::new();
        
        record_pb.encode_length_delimited(&mut buf).unwrap();
        writer.write_all(&buf)?;

    }

    writer.flush()?;

    Ok(())

}