use std::{collections::HashSet, io::Read};

use flate2::{bufread::DeflateDecoder, read::GzDecoder};
use futures::StreamExt;
use h3server::{RequestInCSV, RequestInMONGO};
use mongodb::{ bson::doc, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client, Collection };

use csv::Writer;
use serde_json;




// Process db and generate csv, for further conversion into protobuf
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // get the package name
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

    
    // 1. fill all data's bodytype with specific app name
    if true {
        let mut cursor = my_coll
            .find(doc! {
                "app": &package_name
            }).await?;

        while let Some(Ok(req)) = cursor.next().await {
            if req.header.contains_key("content-encoding") {
                let encoding = req.header.get("content-encoding").unwrap();

                if encoding == "gzip" {
                    let body_clone = req.body.clone();
                    let mut decoder = GzDecoder::new(&body_clone[..]);
                    let mut text = vec![];
                    if let Ok(_) = decoder.read_to_end(&mut text) {
                        let bodytype = tree_magic_mini::from_u8(&text);
                        my_coll.update_one(
                            doc! { "_id": &req._id },
                            doc! { "$set": { "bodytype": bodytype.to_string() } }
                        ).await?;
                    }

                } else if encoding == "deflate" {
                    let body_clone = req.body.clone();
                    let mut decoder = DeflateDecoder::new(&body_clone[..]);
                    let mut text = vec![];
                    if let Ok(_) = decoder.read_to_end(&mut text) {
                        let bodytype = tree_magic_mini::from_u8(&text);
                        my_coll.update_one(
                            doc! { "_id": &req._id },
                            doc! { "$set": { "bodytype": bodytype.to_string() } }
                        ).await?;
                    }
                    
                } else {
                    // my_coll.update_one(
                    //     doc! { "_id": &req._id },
                    //     doc! { "$set": { "bodytype": encoding } }
                    // ).await?;
                }

            } else {
                let bodytype = tree_magic_mini::from_u8(&req.body);
                my_coll.update_one(
                    doc! { "_id": &req._id },
                    doc! { "$set": { "bodytype": bodytype.to_string() } }
                ).await?;
            }
        }
    }
    // content-encoding:
    // {"deflate", "union_sdk_encode", "identity", "br", "amz-1.0", "zstd", "msl_v1", "gzip"}



    // 2. get all HTTP3 doc
    let mut http3_cursor = my_coll.find(
        doc! {
            "app": &package_name,
            "version": "HTTP/3",
            // "withquic": true,
        }
    ).await?;

    // 3. get all url from HTTP3 doc, withquic as true inherently
    let mut http3_urls: HashSet<String> = HashSet::new();
    while let Some(Ok(req)) = http3_cursor.next().await {
        // Nov 14 todo: filter out the url not specific to app: beacon, playstore, google ads
        // if uri contains: ??? then continue
        http3_urls.insert(req.uri.clone());
        match req.bodytype.as_deref() {
            Some("application/gzip") => {
                
            },

            Some("text/plain") => {
                match String::from_utf8(req.body) {
                    Ok(plain_text) => {
                        my_coll.update_one(
                            doc! { "_id": &req._id },
                            doc! { "$set": { "bodyplaintext": plain_text } }
                        ).await?;
                    },
                    Err(_) => {},
                }
            },

            Some("application/octet-stream") => {

            },

            Some(&_) => { 
                
            },

            None => {

            },
        }
    }

    // 4. get doc with same url, in trail of withquic as false
    let mut common_urls: HashSet<String> = HashSet::new();
    let url_filter = doc! {
        "app": &package_name,
        "uri": { "$in": Vec::from_iter(http3_urls.clone()) },
        "withquic": false,
        // "version": { "$in": ["HTTP/2", "HTTP/1.1"] },
    };
    let mut http2_cursor = my_coll.find(url_filter).await?;
    // plain text body of these doc
    while let Some(Ok(req)) = http2_cursor.next().await {
        common_urls.insert(req.uri.clone());
        // todo decode
        // application/gzip
        // application/octet-stream
        // text/plain
        match req.bodytype.as_deref() {
            Some("application/gzip") => {
                
            },

            Some("text/plain") => {
                match String::from_utf8(req.body) {
                    Ok(plain_text) => {
                        my_coll.update_one(
                            doc! { "_id": &req._id },
                            doc! { "$set": { "bodyplaintext": plain_text } }
                        ).await?;
                    },
                    Err(_) => {},
                }
            },

            Some("application/octet-stream") => {

            },

            Some(&_) => { 
                
            },

            None => {

            },
        }
    }
    

    println!("= = = = = = = = = = =");

    
    // 6. dump the matching http3 data into csv
    // at this time only write down those with plain text
    let data_dir = std::path::Path::new("csvdata");
    // Create the data directory if it doesn't exist
    if !data_dir.exists() {
        std::fs::create_dir(data_dir)?;
    }
    let csv_name = format!("{}.csv", &package_name);
    let file_path = data_dir.join(csv_name);
    let mut wtr = Writer::from_path(file_path)?;


    for u in Vec::from_iter(http3_urls.clone()) {
        let mut h3_cursor = my_coll.find(
            doc! {
                "app": &package_name,
                "uri": u.clone(),
                "version": "HTTP/3",
            }
        ).await?;
        while let Some(Ok(req)) = h3_cursor.next().await {
            if !req.bodyplaintext.is_none() {
                let req_csv = RequestInCSV {
                    _id: req._id.unwrap().to_hex(),
                    app: req.app,
                    withquic: req.withquic,
                    uri: req.uri,
                    method: req.method,
                    version: req.version,
                    header: serde_json::to_string(&req.header).unwrap(),
                    bodytype: req.bodytype,
                    bodyplaintext: req.bodyplaintext,
                };
                wtr.serialize(req_csv)?;
            }
        }

        let mut h2_cursor = my_coll.find(
            doc! {
                "app": &package_name,
                "uri": u.clone(),
                // "version": { "$in": ["HTTP/2", "HTTP/1.1"] },
                "withquic": false,
            }
        ).await?;
        while let Some(Ok(req)) = h2_cursor.next().await {
            if !req.bodyplaintext.is_none() {
                let req_csv = RequestInCSV {
                    _id: req._id.unwrap().to_hex(),
                    app: req.app,
                    withquic: req.withquic,
                    uri: req.uri,
                    method: req.method,
                    version: req.version,
                    header: serde_json::to_string(&req.header).unwrap(),
                    bodytype: req.bodytype,
                    bodyplaintext: req.bodyplaintext,
                };
                wtr.serialize(req_csv)?;
            }
        }

    }
    // finish csv writing
    wtr.flush()?;


    Ok(())

}

