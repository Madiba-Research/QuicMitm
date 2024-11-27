use std::{collections::HashSet, io::Read};

use flate2::{bufread::DeflateDecoder, read::GzDecoder};
use futures::StreamExt;
use h3server::{RequestInCSV, RequestInMONGO};
use mongodb::{ bson::doc, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client, Collection };

use csv::Writer;
use serde_json;

pub mod data;


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

    


    Ok(())

}