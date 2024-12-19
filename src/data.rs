use std::{collections::HashMap, sync::Arc};

use base64::{prelude::{BASE64_STANDARD, BASE64_STANDARD_NO_PAD}, Engine};
use prost::Message;
use tokio::{
    fs::{read_dir, File},
    io::AsyncReadExt,
};

use crate::{db::NetworkConnection, events::Events};

include!(concat!(env!("OUT_DIR"), "/data.rs"));

pub struct DataProvider {
    path: String,
    pub apps: HashMap<(String, u64), AppData>,
}

impl DataProvider {
    pub fn new(path: String) -> Self {
        Self {
            path,
            apps: Default::default(),
        }
    }
    pub async fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut dirs = read_dir(&self.path).await?;
        let mut sessions = Vec::new();
        let mut apps = Vec::new();
        while let Some(dir) = dirs.next_entry().await? {
            let file_path = dir.path();
            if dir.file_type().await.ok().filter(|f| f.is_file()).is_some() {
                if let Some(sess) = file_path
                    .as_path()
                    .file_name()
                    .and_then(|p| p.to_str())
                    .filter(|p| p.starts_with("dump-") && p.ends_with(".mitm"))
                    .and_then(|p| p.split("-").last())
                    .and_then(|s| s.split(".").next())
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    sessions.push(sess);
                }
            } else if let Some(file_name) = file_path
                .file_name()
                .and_then(|p| p.to_str())
                .map(|p| p.to_string())
            {
                apps.push(file_name);
            }
        }
        for session in sessions {
            for app in &apps {
                
                let mut device_id = None;
                let mut events = Vec::new();
                let network_path = format!(
                    "{}/{}/analysis_result/network-{}.pb",
                    self.path, app, session
                );
                let event_path = format!(
                    "{}/{}/analysis_result/events-{}.txt",
                    self.path, app, session
                );
                let event_missing_path = format!(
                    "{}/{}/analysis_result/events-missing-{}.txt",
                    self.path, app, session
                );
                let device_id_path = format!(
                    "{}/{}/analysis_result/device_id-{}.txt",
                    self.path, app, session
                );

                let mut buf = Vec::new();
                if let Ok(mut file) = File::open(network_path).await {
                    file.read_to_end(&mut buf).await?;
                }
                let data = NetworkData::decode(buf.as_slice())?;
                // dbg!(event_path.clone());
                if let Ok(mut file) = File::open(event_path).await {
                    let mut buf = String::new();
                    file.read_to_string(&mut buf).await?;
                    for l in buf.lines() {
                        let event = serde_json::from_str::<Events>(l)?;
                        events.push(event);
                    }
                }
                if let Ok(mut file) = File::open(event_missing_path).await {
                    let mut buf = String::new();
                    file.read_to_string(&mut buf).await?;
                    for l in buf.lines() {
                        let event = serde_json::from_str::<Events>(l)?;
                        events.push(event);
                    }
                }

                if let Ok(mut device_id_file) = File::open(device_id_path).await {
                    let mut buf = String::new();
                    device_id_file.read_to_string(&mut buf).await?;
                    device_id = Some(buf.trim().to_string());
                }

                let mut app_data = AppData {
                    // pkg: app.clone(),
                    data,
                    events,
                    cryptographic: Default::default(),
                    device_id,
                    session,
                    // flows: Default::default(),
                };
                app_data.load_cryptographic();
                
                // app_data.load_nodes();
                self.apps.insert((app.clone(), session), app_data);
            }
        }
        Ok(())
    }
}

pub struct AppData {
    // pub pkg: String,
    pub data: NetworkData,
    pub events: Vec<Events>,
    pub cryptographic: Arc<Vec<CryptographicRecord>>,
    pub device_id: Option<String>,
    pub session: u64,
    // pub flows: HashMap<(i64, i64), Node>,
}

impl AppData {
    // pub fn load_nodes(&mut self) {
    //     let mut fork_info = Vec::new();
    //     while let Some(e) = self.events.pop() {
    //         if let Events::SchedProcessFork(f) = &e {
    //             fork_info.push((f.parent_pid, f.parent_tid, f.child_pid, f.ts));
    //         }
    //         self.flows.entry(e.get_ptid()).or_default().events.push(e);
    //     }
    //     for ((ppid, ptid), v) in self.flows.iter_mut() {
    //         for (fork_parent_pid, fork_parent_tid, fork_child_pid, fork_ts) in &fork_info {
    //             if ppid == fork_parent_tid && ptid == fork_child_pid {
    //                 v.parent
    //                     .insert((*fork_parent_pid, *fork_parent_tid, *fork_ts));
    //             }
    //             if ppid == fork_parent_pid && ptid == fork_parent_tid {
    //                 v.children
    //                     .push((*fork_child_pid, *fork_parent_tid, *fork_ts));
    //             }
    //         }
    //     }
    //     for (_, v) in self.flows.iter_mut() {
    //         v.events.retain_mut(|e| {
    //             !matches!(
    //                 e,
    //                 Events::SchedProcessFork(_)
    //                     | Events::SchedProcessExit(_)
    //                     | Events::TaskNewtask(_)
    //             )
    //         });
    //     }
    // }
    pub fn load_cryptographic(&mut self) {
        let event_len = self.events.len();
        if event_len == 0 {
            return;
        }
        let mut do_finals = Vec::new();
        let mut records = Vec::new();
        let mut updates = Vec::new();
        let mut i = self.events.len() - 1;

        while i != 0 {
            let event = self.events.get(i).unwrap();
            match event {
                Events::DoFinal(e) => {
                    do_finals.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: Default::default(),
                        output: BASE64_STANDARD.decode(&e.ret).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::DoFinalByte(e) => {
                    do_finals.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.ret).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::DoFinalByteInt(e) => {
                    do_finals.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: Default::default(),
                        output: BASE64_STANDARD.decode(&e.arg_output).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::DoFinalByteIntInt(e) => {
                    do_finals.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.ret).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::DoFinalByteIntIntByte(e) => {
                    do_finals.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.arg_output).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::DoFinalByteIntIntByteInt(e) => {
                    do_finals.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.arg_output).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::DoFinalByteBufferByteBuffer(e) => {
                    do_finals.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.arg_output).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::InitIntCertificate(e) => {
                    records.push(CryptographicRecord::from(e.clone()));
                    self.events.swap_remove(i);
                }
                Events::InitIntKey(e) => {
                    records.push(CryptographicRecord::from(e.clone()));
                    self.events.swap_remove(i);
                }
                Events::UpdateByte(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.ret).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::UpdateByteIntInt(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.ret).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::UpdateByteIntIntByte(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.arg_output).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::UpdateByteIntIntByteInt(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.arg_output).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::UpdateByteBufferByteBuffer(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.arg_output).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::UpdateAADByte(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.ret).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::UpdateAADByteIntInt(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_input).unwrap_or_default(),
                        output: BASE64_STANDARD.decode(&e.ret).unwrap_or_default(),
                    });
                    self.events.swap_remove(i);
                }
                Events::UpdateAADByteBuffer(e) => {
                    updates.push(UnifiedCrypt {
                        timestamp: e.ts,
                        hashcode: e.hashcode,
                        input: BASE64_STANDARD.decode(&e.arg_src).unwrap_or_default(),
                        output: Default::default(),
                    });
                    self.events.swap_remove(i);
                }
                _ => {}
            }
            i -= 1;
        }

        updates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        do_finals.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for u in updates {
            let closest = find_closest(
                u.timestamp,
                records
                    .iter()
                    .filter(|r| r.hashcode == u.hashcode)
                    .collect::<Vec<&CryptographicRecord>>(),
            )
            .map(|r| (r.hashcode, r.timestamp));

            if let Some((hashcode, timestamp)) = closest {
                for r in records.iter_mut() {
                    if r.timestamp == timestamp && r.hashcode == hashcode {
                        if r.is_encryption {
                            r.plaintext.append(&mut u.input.clone());
                            r.ciphertext.append(&mut u.output.clone());
                        } else {
                            r.plaintext.append(&mut u.output.clone());
                            r.ciphertext.append(&mut u.input.clone());
                        }
                        r.timestamp = u.timestamp;
                    }
                }
            }
        }

        for u in do_finals {
            let closest = find_closest(
                u.timestamp,
                records
                    .iter()
                    .filter(|r| r.hashcode == u.hashcode)
                    .collect::<Vec<&CryptographicRecord>>(),
            )
            .map(|r| (r.hashcode, r.timestamp));

            if let Some((hashcode, timestamp)) = closest {
                for r in records.iter_mut() {
                    if r.timestamp == timestamp && r.hashcode == hashcode {
                        if r.is_encryption {
                            r.plaintext.append(&mut u.input.clone());
                            r.ciphertext.append(&mut u.output.clone());
                        } else {
                            r.plaintext.append(&mut u.output.clone());
                            r.ciphertext.append(&mut u.input.clone());
                        }
                    }
                }
            }
        }
        // for r in &records {
        //     println!("{:?}", r);
        //     println!("=====================");
        //     break;
        // }
        self.cryptographic = Arc::new(records);
    }
}

#[derive(Debug)]
pub struct CryptographicRecord {
    pub is_encryption: bool,
    pub plaintext: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub key: Vec<u8>,
    pub hashcode: i64,
    pub timestamp: u128,
    pub algorithm: String,
    pub importance: u64,
}

impl CryptographicRecord {
    pub fn contains(&self, data: &[u8]) -> bool {
        if self.plaintext == self.ciphertext
            || data.len() < 16
            || self.ciphertext.is_empty()
            || self.plaintext.is_empty()
        {
            return false;
        }
        let mut i = 0;
        while i + 16 <= self.ciphertext.len() {
            let chunk = &self.ciphertext[i..i + 16];

            if chunk.iter().any(|&b| b != 0) && crate::leak::is_sub(data, chunk) {
                return true;
            }            
            if chunk.iter().any(|&b| b != 0) && crate::leak::is_sub(data, BASE64_STANDARD_NO_PAD.encode(chunk).as_bytes()) {
                return true;
            }

            i += 16;
        }

        false
    }
}

impl From<crate::events::InitIntKey> for CryptographicRecord {
    fn from(init: crate::events::InitIntKey) -> Self {
        Self {
            timestamp: init.ts,
            algorithm: init.algorithm,
            hashcode: init.hashcode,
            key: BASE64_STANDARD.decode(init.arg_key).unwrap_or_default(),
            is_encryption: init.arg_opcode == 1,
            plaintext: Default::default(),
            ciphertext: Default::default(),
            importance: init.importance,
        }
    }
}
impl From<crate::events::InitIntCertificate> for CryptographicRecord {
    fn from(init: crate::events::InitIntCertificate) -> Self {
        Self {
            timestamp: init.ts,
            algorithm: init.algorithm,
            hashcode: init.hashcode,
            key: BASE64_STANDARD
                .decode(init.arg_certificate)
                .unwrap_or_default(),
            is_encryption: init.arg_opcode == 1,
            plaintext: Default::default(),
            ciphertext: Default::default(),
            importance: init.importance,
        }
    }
}

pub struct UnifiedCrypt {
    pub timestamp: u128,
    pub hashcode: i64,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
}

pub fn find_closest(ts: u128, crypt: Vec<&CryptographicRecord>) -> Option<&CryptographicRecord> {
    let mut smallest_diff = None;
    let mut pkg = None;

    for c in crypt {
        let diff = c.timestamp.abs_diff(ts);
        if (smallest_diff.is_none() || smallest_diff.filter(|s| diff < *s).is_some())
            && c.timestamp <= ts
        {
            smallest_diff = Some(diff);
            pkg = Some(c);
        }
    }
    pkg
}

pub fn find_closest_network_connection(
    ts: u128,
    network_connection: Vec<NetworkConnection>,
) -> Option<NetworkConnection> {
    let mut smallest_diff = None;
    let mut pkg = None;
    if network_connection.is_empty() {
        return None;
    }
    if network_connection.len() == 1 {
        return Some(network_connection[0].clone());
    }
    for c in network_connection {
        let diff = c.ts.abs_diff(ts);
        if (smallest_diff.is_none() || smallest_diff.filter(|s| diff < *s).is_some()) && c.ts <= ts
        {
            smallest_diff = Some(diff);
            pkg = Some(c);
        }
    }
    pkg
}

// smallest_diff = None
// pkg = None

// for (p,num) in numbers:
//     diff = abs(num - target)
//     if smallest_diff is None or diff < smallest_diff:
//         smallest_diff = diff
//         pkg = p

// return pkg