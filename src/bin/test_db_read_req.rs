use std::collections::HashMap;

use h3server::RequestInMONGO;
use http::HeaderMap;
use mongodb::{ bson::doc, options::{ ClientOptions, Hint, ServerApi, ServerApiVersion }, Client, Collection };
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    // Replace the placeholder with your Atlas connection string
    let uri = "mongodb://localhost:27017";
    let mut client_options = ClientOptions::parse(uri).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    let my_coll: Collection<RequestInMONGO> = client.database("requestdb").collection("httpreq");


    // example: read a request
    let req = my_coll.find_one(
        doc! {
            "method": "POST"
        }
    ).await?
    .expect("Missing req document.");
    println!("req: {:?}", req);

    
    let result = tree_magic_mini::from_u8(&req.body);
    println!("body type: {}", result);
    // body type: text/plain
    match String::from_utf8(req.body) {
        Ok(readable) => { println!("body: {}", readable) },
        Err(e) => { println!("error: {}", e) },
    }

    Ok(())
}