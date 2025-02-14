use std::{collections::{HashMap, HashSet}, fs::File as stdfile, sync::Arc};
use futures::StreamExt;
use h3server::{data::AppData, leak};
use mongodb::{bson::doc, options::{ClientOptions, ServerApi, ServerApiVersion}, Client, Collection};
use prost::Message;
use serde::{Deserialize, Serialize};
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
    // path: event path
    let Some(event_path) = args.get(2) else {
        return Err("Please provide the path to the event directory".into());
    };
    let Some(session) = args.get(3) else {
        return Err("Please provide the session h2 or h2h3".into());
    };
    let Some(leak_file) = args.get(4) else {
        return Err("Please provide the path to the leak file".into());
    };
    let Some(app) = args.get(5) else {
        return Err("Please provide the app package name".into());
    };

    let leak_item_file = stdfile::open(leak_file)?;
    let leak_table = serde_json::from_reader::<_, HashMap<String, Vec<String>>>(leak_item_file).unwrap();

    let mut buf = Vec::new();
    let network_file_path = format!("{}/{}-{}.pb", network_path, app, session);

    if let Ok(mut file) = File::open(network_file_path).await {
        file.read_to_end(&mut buf).await?;
     }
    // if let Ok(mut file) = File::open(network_path).await {
    //    file.read_to_end(&mut buf).await?;
    // }

    // prepare data
    let data = h3server::data::NetworkData::decode(buf.as_slice())?;
    // prepare events
    let mut events = Vec::new();
    if let Ok(mut file) = File::open(event_path).await {
        let mut buf = String::new();
        file.read_to_string(&mut buf).await?;
        for l in buf.lines() {
            let event = serde_json::from_str::<h3server::events::Events>(l)?;
            events.push(event);
        }
    }
    let mut app_data = AppData {
        data,
        events,
        cryptographic: Default::default(),
        // we dont need device_id and session in this project
        device_id: Some("pixel6".to_string()),
        session: 0 as u64
    };
    app_data.load_cryptographic();
    // let mut app_data = SimplifiedAppData {
    //     data,
    //     cryptographic: Default::default(),
    // };
    
    
    // analysis main.rs line 44:
    let mut leak_table = leak_table.clone();
    let leak_items = leak::Leaks::from(leak_table);

    let network_leaks = leak_items.extract_leaks(&app_data.data, app_data.cryptographic.clone());
    
    // corresponding to main, line 59
    let mut network_connections = fake_get_network_connections(&network_leaks);

    // prepare mongodb
    let mongodb_uri = "mongodb://localhost:27017";
    let mut client_options = ClientOptions::parse(mongodb_uri).await?;
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    let my_coll: Collection<h3server::RecordInMONGODBv2> = client.database("requestdb2").collection("httpreq2");
    let mut cursor = my_coll
        .find(doc! {
            "app": app,
            "withquic": session == "h2h3",
        }).await?;

    let mut versioned_network_connections = Vec::<VersionedNetworkConnection>::new();
    while let Some(Ok(record)) = cursor.next().await {
        let ncs: Vec<_> = network_connections
            .iter()
            .filter(|c| c.ts == record.conn_info_v2.timestamp as u128)
            .cloned()
            .collect();
        if let Some(nc) = ncs.first() {
            versioned_network_connections.push(VersionedNetworkConnection {
                version: record.request_v2.version.clone(),
                src_addr: nc.src_addr.clone(),
                destination_addr: nc.destination_addr.clone(),
                importance: nc.importance,
                tid: nc.tid,
                pid: nc.pid,
                target: nc.target.clone(),
                ts: nc.ts,
                leaks: nc.leaks.clone(),
            });
        }
    }

    for (leaks, leak_network_info) in network_leaks {
        let candidates = versioned_network_connections
            .iter()
            .filter(|c| c.src_addr == leak_network_info.source_address)
            .cloned()
            .collect::<Vec<_>>();
        if let Some(item_index) =
            find_closest_network_connection_v(leak_network_info.timestamp as u128, candidates)
                .and_then(|closest| versioned_network_connections.iter().position(|c| c == &closest))
        {
            if let Some(entry) = versioned_network_connections.get_mut(item_index) {
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
        &versioned_network_connections,
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

pub fn find_closest_network_connection_v(
    ts: u128,
    network_connection: Vec<VersionedNetworkConnection>,
) -> Option<VersionedNetworkConnection> {
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


#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct VersionedNetworkConnection {
    pub version: String,
    pub src_addr: String,
    pub destination_addr: Option<String>,
    pub importance: i32,
    pub tid: i32,
    pub pid: i32,
    pub target: Option<String>,
    pub ts: u128,
    pub leaks: HashSet<h3server::db::Leak>,
}
