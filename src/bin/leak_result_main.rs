use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format;

const LEAK_ITEMS: [(&str, &str); 29] = [
    // ("imei", "IMEI"),
    // ("serial", "Device Serial"),
    ("device_id", "Device ID"),
    ("adid", "Advertising ID"),
    ("bootloader", "Bootloader"),
    ("finger_print", "Fingerprint"),
    ("cpu", "CPU Model"),
    ("displayid", "Display ID"),
    ("device_name", "Device Name"),
    ("resolution", "Device Resolution"),
    ("abi", "Device ABI"),
    ("model", "Device Model"),
    ("timezone", "Device Timezone"),
    ("operator", "Operator"),
    // ("iccid", "ICCID"),
    // ("imsi", "IMSI"),
    ("wifi_ip", "Device WiFi IP"),
    ("wifi_ipv6", "Device WiFi IPv6"),
    ("gateway", "Default Gateway IP"),
    ("proxy_addr", "On Device Proxy Address"),
    ("my_wifi_essid", "Router ESSID"),
    ("my_wifi_bssid", "Router BSSID"),
    ("other_wifi_essid", "Neighbor Router ESSID"),
    ("other_wifi_bssid", "Neighbor Router BSSID"),
    ("gps4", "GPS ($\\leq$7 meter accuracy)"),
    ("gps3", "GPS (78 meter accuracy)"),
    ("gps2", "GPS (787 meter accuracy)"),
    ("device_apps", "List of Device Apps"),
    ("phone_number", "Phone Number"),
    ("call_history_number", "Call History Number"),
    ("contact_name", "Contacts Name"),
    ("contact_phone", "Contacts Number"),
    ("email", "Device Email"),
];

const DEVICE_INFO: [&str; 13] = [
    "imei",
    "serial",
    "device_id",
    "adid",
    "bootloader",
    "finger_print",
    "cpu",
    "displayid",
    "device_name",
    "resolution",
    "abi",
    "model",
    "timezone",
];

const NETWORK_INFO: [&str; 7] = [
    "operator",
    "iccid",
    "imsi",
    "wifi_ip",
    "wifi_ipv6",
    "gateway",
    "proxy_addr",
];

const NETWORK_LOCATION_INFO: [&str; 4] = [
    "my_wifi_essid",
    "my_wifi_bssid",
    "other_wifi_essid",
    "other_wifi_bssid",
];

const LOCATION_INFO: [&str; 3] = ["gps4", "gps3", "gps2"];

const USER_ASSETS: [&str; 6] = [
    "device_apps",
    "phone_number",
    "call_history_number",
    "contact_name",
    "contact_phone",
    "email",
];


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        return Err("Please provide the path to the leak analysis directory".into());
    };

    let Some(app) = args.get(2) else {
        return Err("Please provide the app name".into());
    };

    let h2h3_file_name = format!("{}/{}-h2h3.json", path, app);
    let h2h3_file = tokio::fs::read_to_string(h2h3_file_name).await?;
    let app_connections_h2h3: Vec<VersionedNetworkConnection> = serde_json::from_str(&h2h3_file)?;

    let h2_connections_h2h3: Vec<_> = app_connections_h2h3
        .iter()
        .filter(|c| c.version != "HTTP/3")
        .cloned()
        .collect();
    let h2_leaks_h2h3 = get_leak_set(&h2_connections_h2h3);

    let h3_connections_h2h3: Vec<_> = app_connections_h2h3
        .iter()
        .filter(|c| c.version == "HTTP/3")
        .cloned()
        .collect();
    let h3_leaks_h2h3 = get_leak_set(&h3_connections_h2h3);

    // in h2h3, compare h2 leak and h3 leak
    let (distinct_leak_h2_1, distinct_leak_h3_1, common_leak_1) = compare_two_leak_sets(&h2_leaks_h2h3, &h3_leaks_h2h3);
    


    let h2_file_name = format!("{}/{}-h2.json", path, app);
    let h2_file = tokio::fs::read_to_string(h2_file_name).await?;
    let app_connections_h2: Vec<VersionedNetworkConnection> = serde_json::from_str(&h2_file)?;
    let h2_leaks_h2 = get_leak_set(&app_connections_h2);
    // compare h3 leak and h2 leak in h2
    let (distinct_leak_h2_2, distinct_leak_h3_2, common_leak_2) = compare_two_leak_sets(&h2_leaks_h2, &h3_leaks_h2h3);

    // compare two runs of h2 and h2h3
    let h2h3_leaks = get_leak_set(&app_connections_h2h3);
    let (distionct_leak_h2h3_3, distinct_leak_h2_3, common_leak_3) = compare_two_leak_sets(&h2h3_leaks, &h2_leaks_h2);

    // collect h3 domain name
    let h3_domains: HashSet<String> = h3_connections_h2h3
        .iter()
        .filter_map(|c| c.destination_addr.clone())
        .collect();

    // write to json file
    let c1 = LeakComparison {
        distinct_leak1_num: distinct_leak_h2_1.len(),
        distinct_leak2_num: distinct_leak_h3_1.len(),
        common_leak_num: common_leak_1.len(),
        distinct_leak1: distinct_leak_h2_1,
        distinct_leak2: distinct_leak_h3_1,
        common_leak: common_leak_1,
    };
    let c2 = LeakComparison {
        distinct_leak1_num: distinct_leak_h2_2.len(),
        distinct_leak2_num: distinct_leak_h3_2.len(),
        common_leak_num: common_leak_2.len(),
        distinct_leak1: distinct_leak_h2_2,
        distinct_leak2: distinct_leak_h3_2,
        common_leak: common_leak_2,
    };
    let c3 = LeakComparison {
        distinct_leak1_num: distionct_leak_h2h3_3.len(),
        distinct_leak2_num: distinct_leak_h2_3.len(),
        common_leak_num: common_leak_3.len(),
        distinct_leak1: distionct_leak_h2h3_3,
        distinct_leak2: distinct_leak_h2_3,
        common_leak: common_leak_3,
    };

    let result = ResultOutput {
        // compare, in the h2h3 run, h2 leaks with h3 leaks
        h2h3: c1,
        // compare the h3 leaks in h2h3 run with all leaks in h2 run
        h3_h2: c2,
        // compare all the leaks in h2h3 run with all the leaks in h2 run
        h2h3_h2: c3,
        // the leaks of all h3 domains
        h3_domains,
    };

    let _ = std::fs::create_dir_all("leakresult")?;
    serde_json::to_writer(
        std::fs::File::create(format!("leakresult/{}.json", app)).unwrap(),
        &result,
    )?;

    Ok(())
}


fn get_leak_set(connections: &[VersionedNetworkConnection]) -> HashSet<String> {
    let mut leaks: HashSet<String> = HashSet::new();
    
    connections.iter().for_each(|c| {
        c.leaks.iter().for_each(|l| {
            leaks.insert(l.leak.clone());
        });
    });

    leaks
}


fn compare_two_leak_sets(leaks1: &HashSet<String>, leaks2: &HashSet<String>) -> (
    HashSet<String>,
    HashSet<String>,
    HashSet<String>,
) {
    let mut distinct_leak1: HashSet<String> = HashSet::new();
    let mut distinct_leak2: HashSet<String> = HashSet::new();
    let mut common_leak: HashSet<String> = HashSet::new();

    leaks1.iter().for_each(|l| {
        if leaks2.contains(l) {
            common_leak.insert(l.clone());
        } else {
            distinct_leak1.insert(l.clone());
        }
    });

    leaks2.iter().for_each(|l| {
        if !leaks1.contains(l) {
            distinct_leak2.insert(l.clone());
        }
    });

    (distinct_leak1, distinct_leak2, common_leak)
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


#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct LeakComparison {
    pub distinct_leak1_num: usize,
    pub distinct_leak2_num: usize,
    pub common_leak_num: usize,
    pub distinct_leak1: HashSet<String>,
    pub distinct_leak2: HashSet<String>,
    pub common_leak: HashSet<String>,   
}


#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct ResultOutput {
    pub h2h3: LeakComparison,
    pub h3_h2: LeakComparison,
    pub h2h3_h2: LeakComparison,
    pub h3_domains: HashSet<String>,
}