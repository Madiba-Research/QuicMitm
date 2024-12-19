use std::{collections::{HashMap, HashSet}, fs::File as stdfile, sync::Arc};
use h3server::leak;
use prost::Message;
use tokio::{
    fs::{read_dir, File},
    io::AsyncReadExt,
};
// include!(concat!(env!("OUT_DIR"), "/data.rs"));




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    // path: data path
    let Some(network_path) = args.get(1) else {
        return Err("Please provide the path to the data directory".into());
    };
    let Some(session) = args.get(2) else {
        return Err("Please provide the session h2 or h2h3".into());
    };
    let Some(leak_file) = args.get(3) else {
        return Err("Please provide the path to the leak file".into());
    };
    let Some(app) = args.get(4) else {
        return Err("Please provide the path to the leak file".into());
    };

    let leak_item_file = stdfile::open(leak_file)?;
    let leak_table = serde_json::from_reader::<_, HashMap<String, Vec<String>>>(leak_item_file).unwrap();

    let mut buf = Vec::new();
    if let Ok(mut file) = File::open(network_path).await {
       file.read_to_end(&mut buf).await?;
    }
    let data = h3server::data::NetworkData::decode(buf.as_slice())?;
    let mut app_data = SimplifiedAppData {
        data,
        cryptographic: Default::default(),
    };
    
    // analysis main.rs line 40:
    let mut leak_table = leak_table.clone();
    let leak_items = leak::Leaks::from(leak_table);

    let network_leaks = leak_items.extract_leaks(&app_data.data, app_data.cryptographic.clone());
    
    // corresponding to main, line 59
    let mut network_connections = fake_get_network_connections(&network_leaks);

    for (leaks, leak_network_info) in network_leaks {
        let candidates = network_connections
            .iter()
            .filter(|c| c.src_addr == leak_network_info.source_address)
            .cloned()
            .collect::<Vec<_>>();
        if let Some(item_index) =
            find_closest_network_connection(leak_network_info.timestamp as u128, candidates)
                .and_then(|closest| network_connections.iter().position(|c| c == &closest))
        {
            if let Some(entry) = network_connections.get_mut(item_index) {
                entry.leaks.extend(leaks);
                entry.destination_addr = Some(leak_network_info.destination_address);
            }
        }
    }


    // output file
    // let app = args.get(4);
    println!("Network done {}:{}", app, session);

    let _ = std::fs::create_dir_all("leakanalysis")?;
    
    serde_json::to_writer(
        stdfile::create(format!(
            "leakanalysis/{}-{}.json",
            app, session
        ))
        .unwrap(),
        &network_connections,
    )
    .unwrap();

    Ok(())
}


fn fake_get_network_connections(
    network_leaks: &Vec<(HashSet<h3server::db::Leak>, h3server::data::ConnectionInfo)>
) -> Vec<h3server::db::NetworkConnection> {
    let mut network_connections = Vec::new();
    for (leaks, connection_info) in network_leaks {
        network_connections.push(h3server::db::NetworkConnection{
            src_addr: connection_info.source_address.clone(),
            destination_addr: None,
            importance: 0,
            tid: 0,
            pid: 0,
            target: None,
            ts: connection_info.timestamp.clone() as u128,
            leaks: leaks.clone(),
        });
    }
    network_connections
}

pub fn find_closest_network_connection(
    ts: u128,
    network_connection: Vec<h3server::db::NetworkConnection>,
) -> Option<h3server::db::NetworkConnection> {
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


pub struct SimplifiedAppData {
    // pub pkg: String,
    pub data: h3server::data::NetworkData,
    pub cryptographic: Arc<Vec<h3server::data::CryptographicRecord>>,
    // pub flows: HashMap<(i64, i64), Node>,
}

