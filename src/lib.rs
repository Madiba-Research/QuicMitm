use std::collections::HashMap;

use http::Version;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestInMONGO {
    pub uri: String,
    pub method: String,
    pub version: String,
    
    pub header: HashMap<String, String>,
    pub body: Vec<u8>
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

