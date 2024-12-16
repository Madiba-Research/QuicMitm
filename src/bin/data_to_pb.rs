use futures::StreamExt;
use hyper_util::server::conn;
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
    if args[2] != "h2" || args[2] != "h2h3" {
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
    while let Some(Ok(record)) = cursor.next().await {
        // connection info pb
        let conn_info = record.conn_info_v2;
        let conn_info_pb = ConnectionInfo {
            source_address: conn_info.source_addr,
            destination_address: conn_info.dest_addr,
            tls: conn_info.is_tls,
            timestamp: conn_info.timestamp,
        };

        // req pb
        let req = record.request_v2;

        let mut req_headers_pb = Vec::new();
        for (k, v) in req.header {
            let h_pb = Header { key: k, value: v };
            req_headers_pb.push(h_pb);
        }
        let req_trailers_pb: Vec<Header> = Vec::new();

        todo!("body decode");
        let req_pb = Request {
            method: req.method,
            path: req.path,
            headers: req_headers_pb,
            trailers: req_trailers_pb,
            body: 
        };
    }


    Ok(())
}