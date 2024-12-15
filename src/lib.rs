use std::collections::HashMap;

use http::Version;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};


// pub const IS_CONNECTING_DB: bool = false;


// pub mod test_field {

//     pub const WITH_QUIC: bool = true;

//     pub const APP: &[u8] = b"com.alibaba.intl.android.apps.poseidon";
//     pub const APP: &[u8] = b"com.amazon.mShop.android.shopping";
//     pub const APP: &[u8] = b"com.americasbestpics";
//     pub const APP: &[u8] = b"com.best.quick.browser";
//     pub const APP: &[u8] = b"com.cbs.app";
//     pub const APP: &[u8] = b"com.einnovation.temu";
//     pub const APP: &[u8] = b"com.google.android.apps.messaging";
//     pub const APP: &[u8] = b"com.google.android.apps.translate";
//     pub const APP: &[u8] = b"com.google.android.youtube";
//     pub const APP: &[u8] = b"com.instabridge.android";
//     pub const APP: &[u8] = b"com.netflix.mediaclient";
//     pub const APP: &[u8] = b"com.newleaf.app.android.victor";
//     pub const APP: &[u8] = b"com.pinterest";
//     pub const APP: &[u8] = b"com.radio.pocketfm";
//     pub const APP: &[u8] = b"com.reddit.frontpage";
//     pub const APP: &[u8] = b"com.weaver.app.prod";
//     pub const APP: &[u8] = b"gen.tech.impulse.android";
// }





#[derive(Serialize, Deserialize, Debug)]
pub struct RecordInMONGODBv2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,

    pub request_v2: RequestInMONGOv2,
    pub response_v2: ResponseInMONGOv2,
    pub conn_info_v2: ConnectionInfoInMONGODBv2,

    // info for NetworkData
    pub app: String,
    pub withquic: bool,

    pub time_stamp: u64,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionInfoInMONGODBv2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,

    pub source_addr: String,
    pub dest_addr: String,
    pub is_tls: bool,
    pub timestamp: u64,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RequestInMONGOv2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,

    pub method: String,
    pub path: String,

    pub header: HashMap<String, String>,
    pub body: Vec<u8>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseInMONGOv2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,

    pub status_code: u32,
    pub header: HashMap<String, String>,
    pub body: Vec<u8>,
}





#[derive(Serialize, Deserialize, Debug)]
pub struct RequestInMONGO {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,

    pub app: String,
    pub withquic: bool,

    pub uri: String,
    pub method: String,
    pub version: String,
    
    pub header: HashMap<String, String>,
    pub body: Vec<u8>,
    pub bodytype: Option<String>,
    pub bodyplaintext: Option<String>,

    pub tls: bool,
    pub time: u64,
}


#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RequestInCSV {
    pub _id: String,

    pub app: String,
    pub withquic: bool,

    pub uri: String,
    pub method: String,
    pub version: String,
    
    pub header: String,
    // body: Vec<u8>,
    pub bodytype: Option<String>,
    pub bodyplaintext: Option<String>,
}



pub fn headers_to_hashmap(headers: &http::HeaderMap) -> HashMap<String, String> {
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

pub fn version_to_string(version: &Version) -> String {
    match version {
        &Version::HTTP_09 => "HTTP/0.9".to_string(),
        &Version::HTTP_10 => "HTTP/1.0".to_string(),
        &Version::HTTP_11 => "HTTP/1.1".to_string(),
        &Version::HTTP_2 => "HTTP/2".to_string(),
        &Version::HTTP_3 => "HTTP/3".to_string(),
        _ => "Unknown HTTP Version".to_string(),
    }
}




