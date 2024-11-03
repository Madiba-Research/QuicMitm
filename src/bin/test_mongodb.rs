use std::collections::HashMap;

use http::HeaderMap;
use mongodb::{ bson::doc, options::{ ClientOptions, Hint, ServerApi, ServerApiVersion }, Client, Collection };
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Book {
    // _id: i32,
    title: String,
    author: String,
    header: HashMap<String, String>,
    body: Vec<u8>
}

fn headers_to_hashmap(headers: &HeaderMap) -> HashMap<String, String> {
    let mut hashmap = HashMap::new();
    
    for (key, value) in headers.iter() {
        // Convert HeaderName to String and HeaderValue to String
        if let Ok(key_str) = key.to_string().parse::<String>() {
            if let Ok(value_str) = value.to_str() {
                hashmap.insert(key_str, value_str.to_string());
            }
        }
    }
    
    hashmap
}



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

    // Send a ping to confirm a successful connection
    client.database("admin").run_command(doc! { "ping": 1 }).await?;
    println!("Pinged your deployment. You successfully connected to MongoDB!");

    let my_coll: Collection<Book> = client.database("db").collection("books");

    let mut headers = HeaderMap::new();

    headers.insert(http::header::HOST, "example.com".parse().unwrap());
    headers.insert(http::header::CONTENT_LENGTH, "123".parse().unwrap());

    

    // insert
    // let mut h: HashMap<String, String> = HashMap::new();
    let mut h = headers_to_hashmap(&headers);
    let b = b"bodybyte".to_vec();
    h.insert("h1".to_string(), "h1_key".to_string());
    h.insert("h2".to_string(), "h2_key".to_string());
    // let doc = Book {
    //     // _id: 8,
    //     title: "Atonement".to_string(),
    //     author: "Ian McEwan".to_string(),
    //     header: h,
    //     body: b,
    // };

    // let insert_one_result = my_coll.insert_one(doc).await?;
    // println!("Inserted document with _id: {}", insert_one_result.inserted_id);

    // read
    let book = my_coll.find_one(
        doc! {
            "title": "Atonement"
        }
    ).await?
    .expect("Missing 'Atonement' document.");
    println!("book: {:?}", book);

    // delete book
    // let res = my_coll
    //     .delete_many(doc! {"title": "Atonement"})
    //     .hint(Hint::Name("_id_".to_string()))
    //     .await
    //     .expect("failed delete");

    Ok(())
}
