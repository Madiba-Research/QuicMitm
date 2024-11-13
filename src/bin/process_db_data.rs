use std::{collections::{HashMap, HashSet}, io::Read};

use flate2::read::GzDecoder;
use futures::StreamExt;
use h3::client::new;
use h3server::{RequestInCSV, RequestInMONGO};
use mongodb::{ bson::{doc, oid::ObjectId}, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client, Collection };
use rustls::crypto::hash::Hash;

use csv::Writer;
use serde_json;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let uri = "mongodb://localhost:27017";
    let mut client_options = ClientOptions::parse(uri).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    let my_coll: Collection<RequestInMONGO> = client.database("requestdb").collection("httpreq");

    
    // 1. fill all data's bodytype
    if false {
        let mut cursor = my_coll.find(doc! {}).await?;

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

                } else {
                    my_coll.update_one(
                        doc! { "_id": &req._id },
                        doc! { "$set": { "bodytype": encoding } }
                    ).await?;
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
            "version": "HTTP/3",
        }
    ).await?;

    // 3. get all url from HTTP3 doc
    let mut http3_urls: HashSet<String> = HashSet::new();
    while let Some(Ok(req)) = http3_cursor.next().await {
        http3_urls.insert(req.uri.clone());
    }

    // 4. see if HTTP2 doc has same url
    let mut common_urls: HashSet<String> = HashSet::new();
    let url_filter = doc! {
        "uri": { "$in": Vec::from_iter(http3_urls.clone()) },
        "version": { "$in": ["HTTP/2", "HTTP/1.1"] },
    };
    let mut http2_cursor = my_coll.find(url_filter).await?;
    
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


    // 5. get all corresponding HTTP3 doc
    let mut http3_common_cursor = my_coll.find(
        doc! {
            "uri": { "$in": Vec::from_iter(common_urls.clone()) },
            "version": "HTTP/3",
        }
    ).await?;

    while let Some(Ok(req)) = http3_common_cursor.next().await {
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

    
    // 6. dump the matching http3 data into csv
    // at this time only write down those with plain text
    let mut wtr = Writer::from_path("h2_h3_table.csv")?;
    wtr.write_record(&[
        "_id",

        "app",
        "withquic",
    
        "uri",
        "method",
        "version",
        
        "header",
        "bodytype",
        "bodyplaintext"
    ])?;

    for u in Vec::from_iter(common_urls.clone()) {
        let mut h3_cursor = my_coll.find(
            doc! {
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
                "uri": u.clone(),
                "version": { "$in": ["HTTP/2", "HTTP/1.1"] },
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


    // todo: process http3 request for those not in common urls    


    Ok(())

}

