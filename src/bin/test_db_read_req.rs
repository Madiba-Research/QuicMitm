// use std::collections::HashMap;

// use std::collections::HashMap;

use futures::StreamExt;
use h3server::RequestInMONGO;
// use http::HeaderMap;
// use mongodb::{ bson::{doc, oid::ObjectId}, options::{ ClientOptions, FindOptions, ServerApi, ServerApiVersion }, Client, Collection };
use mongodb::{ bson::doc, options::{ ClientOptions, FindOptions, ServerApi, ServerApiVersion }, Client, Collection };

// use serde::{Deserialize, Serialize};
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt; // for write_all()


// fn hashmap_contain_word(h_map: &HashMap<String, String>) -> bool {
//     if let Some(encode_type) = h_map.get("content-encoding") {
//         return true;
//     }
//     false
// }




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


    // example: read request
    // let mut req_cursor = my_coll.find(
    //     doc! {
    //         "version": "HTTP/3",
    //         // "bodytype": "text/plain",
    //         "method": { "$ne": "GET" }
    //     }
    // ).await.unwrap();

    // let mut n = 0;

    // while let Some(Ok(req)) = req_cursor.next().await {
    //     if hashmap_contain_word(&req.header) {
    //         n  += 1;
    //         println!(
    //             "uri: {}\nmethod: {}\nversion: {}\nheader: {:?}\n",
    //             req.uri,
    //             req.method,
    //             req.version,
    //             req.header,
    //             // req.bodyplaintext
    //         );
    //     }
    // }

    // println!("with content-encoding: {}", n);
    
    // let result = tree_magic_mini::from_u8(&req.body);
    // println!("body type: {}", result);
    // // body type: text/plain
    // match String::from_utf8(req.body) {
    //     Ok(readable) => { println!("body: {}", readable) },
    //     Err(e) => { println!("error: {}", e) },
    // }


    // example: check a doc with specific id
    // error at req id Some(ObjectId("6727fa00994bb5ac9d41e600")) -> "content-type": "application/x-protobuf"
    // let obj_id = ObjectId::parse_str("67298676541e00d414fd78e4").unwrap();
    // let req = my_coll.find_one(
    //     doc! {
    //         "_id": obj_id
    //     }
    // ).await?
    // .expect("Missing req document.");
    // println!("{:?}", req);
    


    // example: add bodytype and bodyplaintext (for "text/plain" type) for each request
    // let mut cursor = my_coll.find(doc! {}).await?;
    // while let Some(Ok(req)) = cursor.next().await {
    //     let bodytype = tree_magic_mini::from_u8(&req.body);
    //     if bodytype == "text/plain" {
    //         // let readable = String::from_utf8(req.body).expect("cannot convert to readable");
    //         match String::from_utf8(req.body) {
    //             Ok(readable) => {
    //                 my_coll.update_one(
    //                     doc! { "_id": &req._id },
    //                     doc! { "$set": { "bodytype": bodytype.to_string(), "bodyplaintext": readable } }
    //                 ).await?;
    //             }

    //             Err(e) => {
    //                 println!("error at req id {:?} \nError: {}", req._id, e);
    //                 my_coll.update_one(
    //                     doc! { "_id": &req._id },
    //                     doc! {
    //                         "$set": { "bodytype": bodytype.to_string() + ":unreadable" }
    //                     }
    //                 ).await?;
    //             }
    //         }

    //     } else {
    //         my_coll.update_one(
    //             doc! { "_id": &req._id },
    //             doc! { "$set": { "bodytype": bodytype.to_string() } }
    //         ).await?;
    //     }
    // }



    // example: print first 100 data
    let find_options = FindOptions::builder().limit(20).build();
    let mut cursor2 = my_coll.find(doc! {}).with_options(find_options).await?;
    while let Some(Ok(req)) = cursor2.next().await {
        println!(
            "uri: {}\nmethod: {}\nversion: {}\nheader: {:?}\nbodytype: {:?}\n",
            req.uri,
            req.method,
            req.version,
            req.header,
            req.bodytype,
        );
    }



    // example: count document
    match my_coll.count_documents(doc! { "version": "HTTP/3" }).await {
        Ok(count) => { println!("total document num: {}", count) },
        Err(e) => { println!("error: {}", e) },
    }


    Ok(())
}