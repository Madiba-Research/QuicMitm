// use rusqlite::Connection;

// use std::{
//     collections::{HashMap, HashSet},
//     str::FromStr,
// };
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

// use sqlx::types::chrono::{DateTime, NaiveDateTime};
// use sqlx::{sqlite::SqliteConnectOptions, Acquire, Pool, Sqlite, SqlitePool};

// use crate::events::JobScheduler;

// use crate::events::SocketHook;

// pub struct DB {
//     pool: Pool<Sqlite>,
// }

// table! {
//     leaks (id) {
//         id -> Integer,
//         leak -> Text,
//         is_send -> Bool,
//         is_encrypted -> Bool,
//         connection_id -> Integer,
//     }
// }
// table! {
//     connections (id){
//         id -> Integer,
//         pkg_name -> Text,
//         src_addr -> Text,
//         destination_addr -> Text,
//         importance -> Integer,
//         tid -> Integer,
//         pid -> Integer,
//         target -> Text,
//         ts -> Timestamp,
//     }
// }

// type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// impl DB {
//     pub async fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
//         let options = SqliteConnectOptions::from_str(path)?.create_if_missing(true);
//         let pool = SqlitePool::connect_with(options).await?;

//         sqlx::query("CREATE TABLE IF NOT EXISTS connections (id INTEGER PRIMARY KEY, pkg_name TEXT NOT NULL, src_addr TEXT NOT NULL, destination_addr TEXT, importance INTEGER NOT NULL, tid INTEGER NOT NULL, pid INTEGER NOT NULL, target TEXT, ts TIMESTAMP NOT NULL)")
//             .execute(&pool).await
//             .unwrap();
//         sqlx::query("CREATE TABLE IF NOT EXISTS leaks (id INTEGER PRIMARY KEY, leak TEXT NOT NULL, is_send BOOL NOT NULL, is_encrypted BOOL NOT NULL, connection_id INTEGER NOT NULL, FOREIGN KEY(connection_id) REFERENCES connections(id))")
//             .execute(&pool).await
//             .unwrap();
//         Ok(Self { pool })
//     }
//     pub async fn insert_network_connections(
//         &self,
//         app_name: &str,
//         network_connections: HashMap<NetworkConnection, HashSet<Leak>>,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         let mut pool_conn = self.pool.acquire().await?;
//         let conn = pool_conn.acquire().await?;

//         for (connection, leaks) in network_connections {
//             let Some(ts) = ts_int_to_naive_datetime(connection.ts) else {
//                 return Err("Failed to convert timestamp to datetime".into());
//             };
//             let insert_query = r#"
//                 INSERT INTO connections (pkg_name, src_addr, destination_addr, importance, tid, pid, target, ts)
//                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#;
//             let id = sqlx::query(insert_query)
//                 .bind(app_name)
//                 .bind(connection.src_addr)
//                 .bind(connection.destination_addr)
//                 .bind(connection.importance)
//                 .bind(connection.tid)
//                 .bind(connection.pid)
//                 .bind(connection.target)
//                 .bind(ts)
//                 .execute(&mut *conn)
//                 .await?
//                 .last_insert_rowid();
//             for leak in leaks {
//                 let insert_query = r#"
//                     INSERT INTO leaks (leak, is_send, is_encrypted, connection_id)
//                     VALUES ($1, $2, $3, $4)"#;
//                 sqlx::query(insert_query)
//                     .bind(leak.leak)
//                     .bind(leak.is_send)
//                     .bind(leak.is_encrypted)
//                     .bind(id)
//                     .execute(&mut *conn)
//                     .await?;
//             }
//         }
//         Ok(())
//     }
// }

#[derive(Hash, Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct Leak {
    pub leak: String,
    pub is_send: bool,
    pub is_encrypted: bool,
    pub connection_id: i32,
}

// pub fn ts_int_to_naive_datetime(ts: u128) -> Option<NaiveDateTime> {
//     DateTime::from_timestamp((ts / 1000) as i64, (ts % 1000) as u32).map(|t| t.naive_local())
// }

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConnection {
    pub src_addr: String,
    pub destination_addr: Option<String>,
    pub importance: i32,
    pub tid: i32,
    pub pid: i32,
    pub target: Option<String>,
    pub ts: u128,
    pub leaks: HashSet<Leak>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct BackgroundWork {
    pub class: String,
    pub method: String,
    pub ts: u128,
    pub parent: Option<(String, String)>,
    pub trigger: Option<Trigger>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RuntimeRegisteredReceiver {
    pub class: String,
    pub action: Vec<String>,
    pub parent: Option<(String, String)>,
    pub importance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkInfo {
    pub runtime_registered_receiver: Vec<RuntimeRegisteredReceiver>,
    pub background_work: Vec<BackgroundWork>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Trigger {
    BroadcastReceiver(IntentInfo, bool), // bool is dynamic
    Service(IntentInfo, ServiceMethod),
    Job(JobScheduleInfo),
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct JobScheduleInfo {
    // pub class: String,
    // pub method: String,
    // pub ts: u128,
    // pub parent: Option<(String, String)>,
    pub id:i64,
    pub time_driven_job: Option<TimeDrivenJob>,
    pub event_driven_job: Option<EventDrivenJob>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct TimeDrivenJob {
    pub min_latency_millis: i64, // minimum time to wait before running the job
    pub max_execution_delay_millis: i64, // maximum time to wait before running the job
    pub interval_millis: i64,    // interval between each run
    pub flex_millis: i64,        // maximum time to wait before running the job
    pub initial_backoff_millis: i64, // initial backoff time
}

impl TimeDrivenJob {
    pub fn time_slot(&self) -> u8 {
        if self.interval_millis == 0 {
            match self.min_latency_millis {
                0..=10000 => {
                    return 0;
                }
                10001..=60000 => {
                    // 1 minute
                    return 1;
                }
                60001..=1800000 => {
                    // 30 minutes
                    return 2;
                }
                _ => {
                    return 3;
                }
            }
        } else {
            match self.interval_millis {
                0..=10000 => {
                    return 4;
                }
                10001..=60000 => {
                    // 1 minute
                    return 5;
                }
                60001..=1800000 => {
                    // 30 minutes
                    return 6;
                }
                _ => {
                    return 7;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct EventDrivenJob {
    pub required_network_capabilities: Vec<i32>, // network capabilities required to run the job
    pub required_network_transport_types: Vec<i32>, // network transport types required to run the job
    pub is_require_battery_not_low: bool,           // is battery not low
    pub is_require_charging: bool,                  // is charging
    pub is_require_device_idle: bool,               // is device idle
    pub is_require_storage_not_low: bool,           // is storage not low
    pub trigger_content_max_delay: i64,             // maximum delay before running the job
    pub trigger_content_update_delay: i64,          // delay before running the job
    pub trigger_content_uris: String,               // content uris
    pub is_expedited: bool,                         // is expedited
    pub is_important_while_foreground: bool,        // is important while foreground
}

// impl EventDrivenJob {

// }

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ServiceMethod {
    OnBind(bool),
    OnRebind(bool),
    // OnCreate,
    OnStartCommand(bool),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IntentInfo {
    pub action: String,
    pub data: String,
    pub component: String,
    pub extras: Vec<(String, String)>,
    pub intent_package: String,
    pub alarm_info: Option<AlarmInfo>,
}

impl IntentInfo {
    pub fn eq_no_alarm(&self, intent: &Self) -> bool {
        self.action == intent.action
            && self.data == intent.data
            && self.component == intent.component
            && self.extras == intent.extras && self.intent_package == intent.intent_package
    }
}
#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct AlarmInfo {
    pub trigger_at_millis: i64,
    pub window_millis: i64,
    pub interval_millis: i64,
}

// com.deliveryno
// registerReceiver