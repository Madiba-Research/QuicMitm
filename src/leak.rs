use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use flate2::read::GzDecoder;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashSet;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::{
    collections::HashMap,
    io::Cursor,
    net::{Ipv4Addr, Ipv6Addr},
    num::ParseIntError,
    str::FromStr,
};

use crate::data::ConnectionInfo;
use crate::data::CryptographicRecord;
use crate::data::NetworkData;
use crate::db::Leak;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Leaks(pub HashMap<String, Vec<Vec<u8>>>);

impl From<HashMap<String, Vec<String>>> for Leaks {
    fn from(leaks: HashMap<String, Vec<String>>) -> Self {
        let mut leaks_map = HashMap::new();
        for (key, value) in leaks {
            let mut values = Vec::new();
            for v in value {
                if Leaks::is_mac_address(&v) {
                    if let Some(mac) = decode_hex(&v.replace(":", "")).ok() {
                        values.push(mac);
                    }
                }
                if v.contains(":") {
                    values.push(v.replace(":", "").as_bytes().to_vec());
                }
                if v.contains("-") {
                    values.push(v.replace("-", "").as_bytes().to_vec());
                }
                Self::hash(&v.as_bytes())
                    .iter()
                    .for_each(|h| values.push(h.to_vec()));
                Ipv4Addr::from_str(&v)
                    .ok()
                    .map(|ip| values.push(ip.octets().to_vec()));
                Ipv6Addr::from_str(&v)
                    .ok()
                    .map(|ip| values.push(ip.octets().to_vec()));
                f64::from_str(&v)
                    .ok()
                    .map(|f| values.push(f.to_be_bytes().to_vec()));
                f32::from_str(&v)
                    .ok()
                    .map(|f| values.push(f.to_be_bytes().to_vec()));
                i128::from_str(&v)
                    .ok()
                    .filter(|v| *v > (u32::MAX as i128))
                    .map(|i| {
                        values.push(
                            i.to_be_bytes()
                                .into_iter()
                                .skip_while(|x| *x == 0)
                                .collect::<Vec<u8>>(),
                        )
                    });

                if v.len() > 8 && v.len() % 2 == 0 {
                    decode_hex(&v).ok().map(|hex| values.push(hex));
                }
                values.push(v.to_lowercase().as_bytes().to_vec());
            }
            for i in values.clone() {
                values.push(STANDARD_NO_PAD.encode(&i).as_bytes().to_vec());
                values.push(
                    i.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<String>()
                        .as_bytes()
                        .to_vec(),
                );
            }
            // STANDARD_NO_PAD.encode(&values);
            leaks_map.insert(key, values);
        }
        Leaks(leaks_map)
    }
}

impl Leaks {
    // pub fn transform(self) -> Self {
    //     let mut leaks = HashMap::new();
    //     for leak_item in self.0 {}
    //     // let sample = vec!["sample_value".to_string()];
    //     // let mut leaks = HashMap::new();
    //     // leaks.insert("sample".to_string(), sample);
    //     Leaks(leaks)
    // }
    fn is_mac_address(s: &str) -> bool {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return false;
        }
        for part in parts {
            if part.len() != 2 || !part.chars().all(|c| c.is_digit(16)) {
                return false;
            }
        }

        true
    }
    pub fn hash(input: &[u8]) -> Vec<Vec<u8>> {
        let mut output = Vec::new();
        let o = sha2::Sha256::digest(input);
        output.push(o.to_vec());
        output.push(format!("{:x}", o).as_bytes().to_vec());
        let o = sha1::Sha1::digest(input);
        output.push(o.to_vec());
        output.push(format!("{:x}", o).as_bytes().to_vec());
        let o = md5::Md5::digest(input);
        output.push(o.to_vec());
        output.push(format!("{:x}", o).as_bytes().to_vec());
        output
    }
    pub fn contains(&self, chunk: &[u8], unarchive: bool) -> Vec<String> {
        if chunk.len() == 0 {
            return Vec::new();
        }
        let chunk = if unarchive {
            Self::unpack(chunk)
        } else {
            chunk.to_vec()
        };
        let lowercase_chunk = chunk
            .iter()
            .map(|&b| if b >= b'A' && b <= b'Z' { b + 32 } else { b })
            .collect::<Vec<u8>>();
        let mut result = Vec::new();
        for (key, leak_values) in &self.0 {
            for value in leak_values {
                if chunk.len() < value.len() {
                    continue;
                }
                if chunk == *value || lowercase_chunk == *value {
                    result.push(key.clone());
                    break;
                }

                if is_sub(&chunk, value) {
                    result.push(key.clone());
                    break;
                } else if is_sub(&lowercase_chunk, value) {
                    result.push(key.clone());
                    break;
                }
            }
        }
        result
    }

    fn unpack(chunk: &[u8]) -> Vec<u8> {
        let mimetype = tree_magic_mini::from_u8(chunk);
        let mut result = Vec::new();
        match mimetype {
            "application/gzip" => {
                _ = GzDecoder::new(chunk).read_to_end(&mut result);
            }
            "application/zip" => {
                if let Ok(mut c) = zip::ZipArchive::new(Cursor::new(chunk)) {
                    let file_names = c
                        .file_names()
                        .map(|v| v.to_owned())
                        .collect::<Vec<String>>();
                    for name in file_names {
                        result.extend_from_slice(name.as_bytes());
                        if let Ok(mut file) = c.by_name(&name) {
                            file.read_to_end(&mut result).ok();
                        }
                    }
                }
            }
            "application/zstd" => {
                if let Ok(mut c) = zstd::stream::read::Decoder::new(chunk) {
                    c.read_to_end(&mut result).ok();
                }
            }
            // "audio/mpeg" => {
            //     let rand = rand::random::<u64>();
            //     if let Ok(mut file) = std::fs::File::create(format!("/tmp/{}.mp3", rand)){
            //         file.write_all(chunk).ok();
            //     }
            // }
            // "video/mp4" => {
            //     let rand = rand::random::<u64>();
            //     if let Ok(mut file) = std::fs::File::create(format!("/tmp/{}.mp4", rand)){
            //         file.write_all(chunk).ok();
            //     }
            // }
            _ => {
                // println!("{:?}", mimetype);
                return chunk.to_vec();
            }
        }
        if result.is_empty() {
            return Self::unpack(&result);
        } else {
            result
        }
    }

    pub fn extract_leaks(
        &self,
        network_data: &NetworkData,
        cryptos: Arc<Vec<CryptographicRecord>>,
    ) -> Vec<(HashSet<Leak>, ConnectionInfo)> {
        // let leak_items = self;
        network_data
            .records
            .par_iter()
            .map(|record| {
                let Some(connection_info) = record.connection_info.clone() else {
                    return (HashSet::new(), ConnectionInfo::default());
                };
                let leaks = Mutex::new(HashSet::new());
                record.websocket.par_iter().for_each(|ws| {
                    for c in cryptos
                        .iter()
                        .filter(|c| c.contains(&ws.content) && (c.is_encryption == ws.from_client))
                        .collect::<Vec<&CryptographicRecord>>()
                    {
                        for leak in self.contains(&c.plaintext, true) {
                            leaks.lock().unwrap().insert(Leak {
                                leak,
                                is_send: ws.from_client,
                                is_encrypted: true,
                                connection_id: i32::MAX,
                            });
                        }
                    }
                    for leak in self.contains(&ws.content, true) {
                        leaks.lock().unwrap().insert(Leak {
                            leak,
                            is_send: ws.from_client,
                            is_encrypted: false,
                            connection_id: i32::MAX,
                        });
                    }
                });
                if let Some(req) = &record.request {
                    req.headers.par_iter().for_each(|h| {
                        let header_key_valye_binding = h.key.clone() + ": " + &h.value;
                        let header_key_valye = header_key_valye_binding.as_bytes();
                        for c in cryptos
                            .iter()
                            .filter(|c| c.contains(&header_key_valye) && c.is_encryption)
                            .collect::<Vec<&CryptographicRecord>>()
                        {
                            for leak in self.contains(&c.plaintext, true) {
                                leaks.lock().unwrap().insert(Leak {
                                    leak,
                                    is_send: true,
                                    is_encrypted: true,
                                    connection_id: i32::MAX,
                                });
                            }
                        }
                        for leak in self.contains(&header_key_valye, true) {
                            leaks.lock().unwrap().insert(Leak {
                                leak,
                                is_send: true,
                                is_encrypted: false,
                                connection_id: i32::MAX,
                            });
                        }
                    });

                    for c in cryptos
                        .iter()
                        .filter(|c| c.contains(&req.body) && c.is_encryption)
                        .collect::<Vec<&CryptographicRecord>>()
                    {
                        for leak in self.contains(&c.plaintext, true) {
                            leaks.lock().unwrap().insert(Leak {
                                leak,
                                is_send: true,
                                is_encrypted: true,
                                connection_id: i32::MAX,
                            });
                        }
                    }
                    for leak in self.contains(&req.body, true) {
                        leaks.lock().unwrap().insert(Leak {
                            leak,
                            is_send: true,
                            is_encrypted: false,
                            connection_id: i32::MAX,
                        });
                    }
                }
                if let Some(res) = &record.response {
                    res.headers.par_iter().for_each(|h| {
                        let header_key_valye_binding = h.key.clone() + ": " + &h.value;
                        let header_key_valye = header_key_valye_binding.as_bytes();
                        for c in cryptos
                            .iter()
                            .filter(|c| c.contains(&header_key_valye) && !c.is_encryption)
                            .collect::<Vec<&CryptographicRecord>>()
                        {
                            for leak in self.contains(&c.plaintext, false) {
                                leaks.lock().unwrap().insert(Leak {
                                    leak,
                                    is_send: false,
                                    is_encrypted: true,
                                    connection_id: i32::MAX,
                                });
                            }
                        }
                        for leak in self.contains(&header_key_valye, true) {
                            leaks.lock().unwrap().insert(Leak {
                                leak,
                                is_send: false,
                                is_encrypted: false,
                                connection_id: i32::MAX,
                            });
                        }
                    });

                    for c in cryptos
                        .iter()
                        .filter(|c| c.contains(&res.body) && !c.is_encryption)
                        .collect::<Vec<&CryptographicRecord>>()
                    {
                        for leak in self.contains(&c.plaintext, true) {
                            leaks.lock().unwrap().insert(Leak {
                                leak,
                                is_send: false,
                                is_encrypted: true,
                                connection_id: i32::MAX,
                            });
                        }
                    }
                    for leak in self.contains(&res.body, false) {
                        leaks.lock().unwrap().insert(Leak {
                            leak,
                            is_send: false,
                            is_encrypted: false,
                            connection_id: i32::MAX,
                        });
                    }
                }
                (leaks.into_inner().unwrap(), connection_info)
            })
            .collect::<Vec<(HashSet<Leak>, ConnectionInfo)>>()
    }
}
// pub fn is_sub<T: PartialEq>(mut haystack: &[T], needle: &[T]) -> bool {
//     if haystack.len() < needle.len() {
//         return false;
//     }
//     if needle.len() == 0 {
//         return true;
//     }
//     while !haystack.is_empty() {
//         if haystack.starts_with(needle) {
//             return true;
//         }
//         haystack = &haystack[1..];
//     }
//     false
// }
pub fn is_sub<T: PartialEq>(haystack: &[T], needle: &[T]) -> bool {
    let h_len = haystack.len();
    let n_len = needle.len();
    if n_len == 0 {
        return true;
    }
    if h_len < n_len {
        return false;
    }
    for i in 0..=(h_len - n_len) {
        let mut found = true;
        for j in 0..n_len {
            if haystack[i + j] != needle[j] {
                found = false;
                break;
            }
        }
        if found {
            return true;
        }
    }
    false
}
// uppercase_bytes
//         .iter()
//         .map(|&b| {
//             if b >= b'A' && b <= b'Z' {
//                 b + 32 // Convert uppercase to lowercase
//             } else {
//                 b
//             }
//         })
//         .collect();

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
