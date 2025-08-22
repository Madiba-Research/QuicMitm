use base64::{prelude::BASE64_STANDARD, Engine};
use flate2::read::GzDecoder;
// use regex::Regex;
// use serde::{ser, Deserialize, Serialize};
use serde::{Deserialize, Serialize};
use std::{io::Read, str::FromStr};

use crate::db::{EventDrivenJob, TimeDrivenJob};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SocketAddr {
    ip: String,
    port: u16,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SchedProcessFork {
    pub parent_pid: i64,
    pub parent_tid: i64,
    pub child_pid: i64,
    uid: i64,
    pub ts: u128,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TaskInfo {
    pub parent_pid: i64,
    pub parent_tid: i64,
    uid: i64,
    pub ts: u128,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SocketAddrV6 {
    uid: i64,
    pub pid: i64,
    pub tid: i64,
    protocol: u8,
    r#type: u8,
    left_socket_addr: SocketAddr,
    right_socket_addr: SocketAddr,
    pub ts: u128,
}
const BACKGROUND_WORK_METHODS: [&str; 7] = [
    "onReceive",
    "doWork",
    "onStartJob",
    "onStartCommand",
    "onBind",
    "onRebind",
    "onCreate",
];
impl SocketAddrV6 {
    pub fn get_local_addr(&self) -> Option<(std::net::Ipv4Addr, u16)> {
        std::net::Ipv6Addr::from_str(&self.left_socket_addr.ip)
            .ok()
            .and_then(|x| x.to_ipv4())
            .map(|x| (x, self.left_socket_addr.port))
    }
    pub fn get_remote_addr(&self) -> Option<(std::net::Ipv4Addr, u16)> {
        std::net::Ipv6Addr::from_str(&self.right_socket_addr.ip)
            .ok()
            .and_then(|x| x.to_ipv4())
            .map(|x| (x, self.right_socket_addr.port))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SocketAddrV4 {
    uid: i64,
    pub pid: i64,
    pub tid: i64,
    protocol: u8,
    r#type: u8,
    left_socket_addr: SocketAddr,
    right_socket_addr: SocketAddr,
    pub ts: u128,
}

impl SocketAddrV4 {
    pub fn get_local_addr(&self) -> Option<(std::net::Ipv4Addr, u16)> {
        std::net::Ipv4Addr::from_str(&self.left_socket_addr.ip)
            .ok()
            .map(|x| (x, self.left_socket_addr.port))
    }
    pub fn get_remote_addr(&self) -> Option<(std::net::Ipv4Addr, u16)> {
        std::net::Ipv4Addr::from_str(&self.right_socket_addr.ip)
            .ok()
            .map(|x| (x, self.right_socket_addr.port))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Events {
    JobScheduler(JobScheduler),
    OnStartJob(OnStartJob),
    Intent(Intent),
    OnReceiveHook(OnReceiveHook),
    OnCreate(OnCreate),
    GetBroadcast(GetBroadcast),
    DoFinal(DoFinal),
    DoFinalByte(DoFinalByte),
    DoFinalByteInt(DoFinalByteInt),
    DoFinalByteIntInt(DoFinalByteIntInt),
    DoFinalByteIntIntByte(DoFinalByteIntIntByte),
    DoFinalByteIntIntByteInt(DoFinalByteIntIntByteInt),
    DoFinalByteBufferByteBuffer(DoFinalByteBufferByteBuffer),
    InitIntCertificate(InitIntCertificate),
    InitIntKey(InitIntKey),
    UpdateByte(UpdateByte),
    UpdateByteIntInt(UpdateByteIntInt),
    UpdateByteIntIntByte(UpdateByteIntIntByte),
    UpdateByteIntIntByteInt(UpdateByteIntIntByteInt),
    UpdateByteBufferByteBuffer(UpdateByteBufferByteBuffer),
    UpdateAADByte(UpdateAADByte),
    UpdateAADByteIntInt(UpdateAADByteIntInt),
    UpdateAADByteBuffer(UpdateAADByteBuffer),
    StartForeground(StartForeground),
    OnStartCommand(OnStartCommand),
    OnBind(OnBind),
    SetImpl(SetImpl),
    SetAlarmClock(SetAlarmClock),
    PendingIntentGet(PendingIntentGet),
    ScheduleInternal(ScheduleInternal),
    RequestWork(RequestWork),
    DoWork(DoWork),
    GenericHook(GenericHook),
    RegisterReceiver(RegisterReceiver),
    DeviceID(DeviceID),
    Canary(Canary),
    StartThreadHook(StartThreadHook),
    SchedProcessFork(SchedProcessFork),
    SchedProcessExit(TaskInfo),
    TaskNewtask(TaskInfo),
    // TaskRename(TaskInfo),
    SocketAddrV6(SocketAddrV6),
    SocketAddrV4(SocketAddrV4),
    SocketHook(SocketHook),
    ConnectHook(ConnectHook),
    #[serde(skip_deserializing)]
    Other(String),
}

impl Events {
    pub fn get_uid(&self) -> i64 {
        match self {
            Events::JobScheduler(e) => e.uid,
            Events::OnStartJob(e) => e.uid,
            Events::Intent(e) => e.uid,
            Events::OnReceiveHook(e) => e.uid,
            Events::OnCreate(e) => e.uid,
            Events::GetBroadcast(e) => e.uid,
            Events::DoFinal(e) => e.uid,
            Events::DoFinalByte(e) => e.uid,
            Events::DoFinalByteInt(e) => e.uid,
            Events::DoFinalByteIntInt(e) => e.uid,
            Events::DoFinalByteIntIntByte(e) => e.uid,
            Events::DoFinalByteIntIntByteInt(e) => e.uid,
            Events::DoFinalByteBufferByteBuffer(e) => e.uid,
            Events::InitIntCertificate(e) => e.uid,
            Events::InitIntKey(e) => e.uid,
            Events::UpdateByte(e) => e.uid,
            Events::UpdateByteIntInt(e) => e.uid,
            Events::UpdateByteIntIntByte(e) => e.uid,
            Events::UpdateByteIntIntByteInt(e) => e.uid,
            Events::UpdateByteBufferByteBuffer(e) => e.uid,
            Events::UpdateAADByte(e) => e.uid,
            Events::UpdateAADByteIntInt(e) => e.uid,
            Events::UpdateAADByteBuffer(e) => e.uid,
            Events::StartForeground(e) => e.uid,
            Events::OnStartCommand(e) => e.uid,
            Events::OnBind(e) => e.uid,
            Events::SetImpl(e) => e.uid,
            Events::SetAlarmClock(e) => e.uid,
            Events::PendingIntentGet(e) => e.uid,
            Events::ScheduleInternal(e) => e.uid,
            Events::RequestWork(e) => e.uid,
            Events::DoWork(e) => e.uid,
            Events::GenericHook(e) => e.uid,
            Events::RegisterReceiver(e) => e.uid,
            Events::DeviceID(e) => e.uid,
            Events::Canary(e) => e.uid,
            Events::StartThreadHook(e) => e.uid,
            Events::SchedProcessFork(e) => e.uid,
            Events::SchedProcessExit(e) => e.uid,
            Events::TaskNewtask(e) => e.uid,
            // Events::TaskRename(e) => e.uid,
            Events::SocketAddrV6(e) => e.uid,
            Events::SocketAddrV4(e) => e.uid,
            Events::SocketHook(e) => e.uid,
            Events::ConnectHook(e) => e.uid,
            Events::Other(_) => 0,
        }
    }
    pub fn get_stacktrace(&self) -> Box<dyn StackTrace + '_> {
        let x: Box<dyn StackTrace> = match self {
            Events::JobScheduler(e) => Box::new(e),
            Events::OnStartJob(e) => Box::new(e),
            Events::Intent(e) => Box::new(e),
            Events::OnReceiveHook(e) => Box::new(e),
            Events::OnCreate(e) => Box::new(e),
            Events::GetBroadcast(e) => Box::new(e),
            Events::DoFinal(e) => Box::new(e),
            Events::DoFinalByte(e) => Box::new(e),
            Events::DoFinalByteInt(e) => Box::new(e),
            Events::DoFinalByteIntInt(e) => Box::new(e),
            Events::DoFinalByteIntIntByte(e) => Box::new(e),
            Events::DoFinalByteIntIntByteInt(e) => Box::new(e),
            Events::DoFinalByteBufferByteBuffer(e) => Box::new(e),
            Events::InitIntCertificate(e) => Box::new(e),
            Events::InitIntKey(e) => Box::new(e),
            Events::UpdateByte(e) => Box::new(e),
            Events::UpdateByteIntInt(e) => Box::new(e),
            Events::UpdateByteIntIntByte(e) => Box::new(e),
            Events::UpdateByteIntIntByteInt(e) => Box::new(e),
            Events::UpdateByteBufferByteBuffer(e) => Box::new(e),
            Events::UpdateAADByte(e) => Box::new(e),
            Events::UpdateAADByteIntInt(e) => Box::new(e),
            Events::UpdateAADByteBuffer(e) => Box::new(e),
            Events::StartForeground(e) => Box::new(e),
            Events::OnStartCommand(e) => Box::new(e),
            Events::OnBind(e) => Box::new(e),
            Events::SetImpl(e) => Box::new(e),
            Events::SetAlarmClock(e) => Box::new(e),
            Events::PendingIntentGet(e) => Box::new(e),
            Events::ScheduleInternal(e) => Box::new(e),
            Events::RequestWork(e) => Box::new(e),
            Events::DoWork(e) => Box::new(e),
            Events::GenericHook(e) => Box::new(e),
            Events::RegisterReceiver(e) => Box::new(e),
            Events::DeviceID(e) => Box::new(e),
            Events::Canary(e) => Box::new(e),
            Events::StartThreadHook(e) => Box::new(e),
            Events::SchedProcessFork(_) => return Box::new(None::<u8>),
            Events::SchedProcessExit(_) => return Box::new(None::<u8>),
            Events::TaskNewtask(_) => return Box::new(None::<u8>),
            // Events::TaskRename(e) => e.uid,
            Events::SocketAddrV6(_) => return Box::new(None::<u8>),
            Events::SocketAddrV4(_) => return Box::new(None::<u8>),
            Events::SocketHook(e) => Box::new(e),
            Events::ConnectHook(e) => Box::new(e),
            Events::Other(_) => return Box::new(None::<u8>),
        };
        x
        // let o = BASE64_STANDARD.decode(&st).ok()?;
        // let mut result = Vec::new();
        // GzDecoder::new(o.as_slice()).read_to_end(&mut result).ok()?;
        // String::from_utf8(result)
        //     .ok()
        //     .map(|x| x.lines().map(|x| x.to_string()).collect::<Vec<String>>())
    }
    pub fn get_ptid(&self) -> (i64, i64) {
        match self {
            Events::JobScheduler(e) => (e.pid, e.tid),
            Events::OnStartJob(e) => (e.pid, e.tid),
            Events::Intent(e) => (e.pid, e.tid),
            Events::OnReceiveHook(e) => (e.pid, e.tid),
            Events::OnCreate(e) => (e.pid, e.tid),
            Events::GetBroadcast(e) => (e.pid, e.tid),
            Events::DoFinal(e) => (e.pid, e.tid),
            Events::DoFinalByte(e) => (e.pid, e.tid),
            Events::DoFinalByteInt(e) => (e.pid, e.tid),
            Events::DoFinalByteIntInt(e) => (e.pid, e.tid),
            Events::DoFinalByteIntIntByte(e) => (e.pid, e.tid),
            Events::DoFinalByteIntIntByteInt(e) => (e.pid, e.tid),
            Events::DoFinalByteBufferByteBuffer(e) => (e.pid, e.tid),
            Events::InitIntCertificate(e) => (e.pid, e.tid),
            Events::InitIntKey(e) => (e.pid, e.tid),
            Events::UpdateByte(e) => (e.pid, e.tid),
            Events::UpdateByteIntInt(e) => (e.pid, e.tid),
            Events::UpdateByteIntIntByte(e) => (e.pid, e.tid),
            Events::UpdateByteIntIntByteInt(e) => (e.pid, e.tid),
            Events::UpdateByteBufferByteBuffer(e) => (e.pid, e.tid),
            Events::UpdateAADByte(e) => (e.pid, e.tid),
            Events::UpdateAADByteIntInt(e) => (e.pid, e.tid),
            Events::UpdateAADByteBuffer(e) => (e.pid, e.tid),
            Events::StartForeground(e) => (e.pid, e.tid),
            Events::OnStartCommand(e) => (e.pid, e.tid),
            Events::OnBind(e) => (e.pid, e.tid),
            Events::SetImpl(e) => (e.pid, e.tid),
            Events::SetAlarmClock(e) => (e.pid, e.tid),
            Events::PendingIntentGet(e) => (e.pid, e.tid),
            Events::ScheduleInternal(e) => (e.pid, e.tid),
            Events::RequestWork(e) => (e.pid, e.tid),
            Events::DoWork(e) => (e.pid, e.tid),
            Events::GenericHook(e) => (e.pid, e.tid),
            Events::DeviceID(e) => (e.pid, e.tid),
            Events::Canary(e) => (e.pid, e.tid),
            Events::SchedProcessFork(e) => (e.parent_pid, e.parent_pid),
            Events::SchedProcessExit(e) => (e.parent_tid, e.parent_tid),
            Events::TaskNewtask(e) => (e.parent_pid, e.parent_tid),
            // Events::TaskRename(e) => e.uid,
            Events::SocketAddrV6(e) => (e.pid, e.tid),
            Events::SocketAddrV4(e) => (e.pid, e.tid),
            Events::SocketHook(e) => (e.pid, e.tid),
            Events::ConnectHook(e) => (e.pid, e.tid),
            Events::StartThreadHook(e) => (e.pid, e.tid),
            Events::RegisterReceiver(e) => (e.pid, e.tid),
            Events::Other(_) => (0, 0),
        }
    }

    pub fn timestamp(&self) -> u128 {
        match self {
            Events::JobScheduler(e) => e.ts,
            Events::OnStartJob(e) => e.ts,
            Events::Intent(e) => e.ts,
            Events::OnReceiveHook(e) => e.ts,
            Events::OnCreate(e) => e.ts,
            Events::GetBroadcast(e) => e.ts,
            Events::DoFinal(e) => e.ts,
            Events::DoFinalByte(e) => e.ts,
            Events::DoFinalByteInt(e) => e.ts,
            Events::DoFinalByteIntInt(e) => e.ts,
            Events::DoFinalByteIntIntByte(e) => e.ts,
            Events::DoFinalByteIntIntByteInt(e) => e.ts,
            Events::DoFinalByteBufferByteBuffer(e) => e.ts,
            Events::InitIntCertificate(e) => e.ts,
            Events::InitIntKey(e) => e.ts,
            Events::UpdateByte(e) => e.ts,
            Events::UpdateByteIntInt(e) => e.ts,
            Events::UpdateByteIntIntByte(e) => e.ts,
            Events::UpdateByteIntIntByteInt(e) => e.ts,
            Events::UpdateByteBufferByteBuffer(e) => e.ts,
            Events::UpdateAADByte(e) => e.ts,
            Events::UpdateAADByteIntInt(e) => e.ts,
            Events::UpdateAADByteBuffer(e) => e.ts,
            Events::StartForeground(e) => e.ts,
            Events::OnStartCommand(e) => e.ts,
            Events::OnBind(e) => e.ts,
            Events::SetImpl(e) => e.ts,
            Events::SetAlarmClock(e) => e.ts,
            Events::PendingIntentGet(e) => e.ts,
            Events::ScheduleInternal(e) => e.ts,
            Events::RequestWork(e) => e.ts,
            Events::DoWork(e) => e.ts,
            Events::GenericHook(e) => e.ts,
            Events::DeviceID(e) => e.ts,
            Events::Canary(e) => e.ts,
            Events::SchedProcessFork(e) => e.ts,
            Events::SchedProcessExit(e) => e.ts,
            Events::TaskNewtask(e) => e.ts,
            // Events::TaskRename(e) => e.ts,
            Events::SocketAddrV6(e) => e.ts,
            Events::SocketAddrV4(e) => e.ts,
            Events::SocketHook(e) => e.ts,
            Events::ConnectHook(e) => e.ts,
            Events::StartThreadHook(e) => e.ts,
            Events::RegisterReceiver(e) => e.ts,
            Events::Other(_) => 0,
        }
    }
    pub fn importance(&self) -> u64 {
        match self {
            Events::JobScheduler(e) => e.importance,
            Events::OnStartJob(e) => e.importance,
            Events::Intent(e) => e.importance,
            Events::OnReceiveHook(e) => e.importance,
            Events::OnCreate(e) => e.importance,
            Events::GetBroadcast(e) => e.importance,
            Events::DoFinal(e) => e.importance,
            Events::DoFinalByte(e) => e.importance,
            Events::DoFinalByteInt(e) => e.importance,
            Events::DoFinalByteIntInt(e) => e.importance,
            Events::DoFinalByteIntIntByte(e) => e.importance,
            Events::DoFinalByteIntIntByteInt(e) => e.importance,
            Events::DoFinalByteBufferByteBuffer(e) => e.importance,
            Events::InitIntCertificate(e) => e.importance,
            Events::InitIntKey(e) => e.importance,
            Events::UpdateByte(e) => e.importance,
            Events::UpdateByteIntInt(e) => e.importance,
            Events::UpdateByteIntIntByte(e) => e.importance,
            Events::UpdateByteIntIntByteInt(e) => e.importance,
            Events::UpdateByteBufferByteBuffer(e) => e.importance,
            Events::UpdateAADByte(e) => e.importance,
            Events::UpdateAADByteIntInt(e) => e.importance,
            Events::UpdateAADByteBuffer(e) => e.importance,
            Events::StartForeground(e) => e.importance,
            Events::OnStartCommand(e) => e.importance,
            Events::OnBind(e) => e.importance,
            Events::SetImpl(e) => e.importance,
            Events::SetAlarmClock(e) => e.importance,
            Events::PendingIntentGet(e) => e.importance,
            Events::ScheduleInternal(e) => e.importance,
            Events::RequestWork(e) => e.importance,
            Events::DoWork(e) => e.importance,
            Events::GenericHook(e) => e.importance,
            Events::DeviceID(e) => e.importance,
            Events::Canary(e) => e.importance,
            Events::SchedProcessFork(_e) => u64::MAX,
            Events::SchedProcessExit(_e) => u64::MAX,
            Events::TaskNewtask(_e) => u64::MAX,
            // Events::TaskRename(e) => e.ts,
            Events::SocketAddrV6(_e) => u64::MAX,
            Events::SocketAddrV4(_e) => u64::MAX,
            Events::SocketHook(e) => e.importance,
            Events::ConnectHook(e) => e.importance,
            Events::StartThreadHook(e) => e.importance,
            Events::RegisterReceiver(e) => e.importance,
            Events::Other(_) => u64::MAX,
        }
    }

    // pub fn is_cryptographic_api(&self) -> bool{
    //     match self {
    //         Events::DoFinal(_) => true,
    //         Events::DoFinalByte(_) => true,
    //         Events::DoFinalByteInt(_) => true,
    //         Events::DoFinalByteIntInt(_) => true,
    //         Events::DoFinalByteIntIntByte(_) => true,
    //         Events::DoFinalByteIntIntByteInt(_) => true,
    //         Events::DoFinalByteBufferByteBuffer(_) => true,
    //         Events::InitIntCertificate(_) => true,
    //         Events::InitIntKey(_) => true,
    //         Events::UpdateByte(_) => true,
    //         Events::UpdateByteIntInt(_) => true,
    //         Events::UpdateByteIntIntByte(_) => true,
    //         Events::UpdateByteIntIntByteInt(_) => true,
    //         Events::UpdateByteBufferByteBuffer(_) => true,
    //         Events::UpdateAADByte(_) => true,
    //         Events::UpdateAADByteIntInt(_) => true,
    //         Events::UpdateAADByteBuffer(_) => true,
    //         _ => false,
    //     }
    // }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq,Hash, Eq)]
pub struct JobScheduler {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub job_id: i64,
    pub backoff_policy: i64,
    pub clip_data: String,
    pub clip_grant_flags: i64,
    pub estimated_network_download_bytes: i64,
    pub estimated_network_upload_bytes: i64,
    pub flex_millis: i64,
    pub initial_backoff_millis: i64,
    pub interval_millis: i64,
    pub max_execution_delay_millis: i64,
    pub min_latency_millis: i64,
    pub minimum_network_chunk_bytes: i64,
    pub network_type: i64,
    pub priority: i64,
    pub service: String,
    pub required_network_capabilities: Vec<i32>,
    pub required_network_transport_types: Vec<i32>,
    pub trigger_content_max_delay: i64,
    pub trigger_content_update_delay: i64,
    pub trigger_content_uris: String,
    pub is_expedited: bool,
    pub is_important_while_foreground: bool,
    pub is_periodic: bool,
    pub is_persisted: bool,
    pub is_prefetch: bool,
    pub is_require_battery_not_low: bool,
    pub is_require_charging: bool,
    pub is_require_device_idle: bool,
    pub is_require_storage_not_low: bool,
    pub pid: i64,
    pub tid: i64,
    pub uid: i64,
    pub package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

impl JobScheduler {
    pub fn into_event(&self) -> Option<EventDrivenJob> {
        let j = self;
        let mut conditions = Vec::new();
        conditions.push(!j.required_network_capabilities.is_empty());
        conditions.push(!j.required_network_transport_types.is_empty());
        conditions.push(j.is_require_battery_not_low);
        conditions.push(j.is_require_charging);
        conditions.push(j.is_require_device_idle);
        conditions.push(j.is_require_storage_not_low);
        conditions.push(j.trigger_content_max_delay != 0);
        conditions.push(j.trigger_content_update_delay != 0);
        conditions.push(!j.trigger_content_uris.is_empty());
        conditions.push(j.is_expedited);
        conditions.push(j.is_important_while_foreground);
        if conditions.iter().any(|c| c == &true) {
            Some(EventDrivenJob {
                required_network_capabilities: j.required_network_capabilities.clone(),
                required_network_transport_types: j.required_network_transport_types.clone(),
                is_require_battery_not_low: j.is_require_battery_not_low,
                is_require_charging: j.is_require_charging,
                is_require_device_idle: j.is_require_device_idle,
                is_require_storage_not_low: j.is_require_storage_not_low,
                trigger_content_max_delay: j.trigger_content_max_delay,
                trigger_content_update_delay: j.trigger_content_update_delay,
                trigger_content_uris: j.trigger_content_uris.clone(),
                is_expedited: j.is_expedited,
                is_important_while_foreground: j.is_important_while_foreground,
            })
        } else {
            None
        }
    }
    pub fn into_time(&self) -> Option<TimeDrivenJob> {
        let j = self;
        let mut conditions = Vec::new();
        conditions.push(j.min_latency_millis != 0);
        conditions.push(j.max_execution_delay_millis != 0);
        conditions.push(j.interval_millis != 0);
        conditions.push(j.flex_millis != 0);
        conditions.push(j.initial_backoff_millis != 0);
        if conditions.iter().any(|c| c == &true) {
            Some(TimeDrivenJob {
                min_latency_millis: j.min_latency_millis,
                max_execution_delay_millis: j.max_execution_delay_millis,
                interval_millis: j.interval_millis,
                flex_millis: j.flex_millis,
                initial_backoff_millis: j.initial_backoff_millis,
            })
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OnStartJob {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub job_id: i64,
    pub pid: i64,
    pub ts: u128,
    pub tid: i64,
    uid: i64,
    package_name: String,
    start_ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Intent {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub action: String,
    pub component: String,
    pub data: String,
    pub extras: Vec<(String, String)>,
    pub intent_package: String,
    pub pid: i64,
    pub tid: i64,
    pub uid: i64,
    pub package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct OnReceiveHook {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub action: String,
    pub component: String,
    pub data: String,
    pub extras: Vec<(String, String)>,
    pub intent_package: String,
    start_ts: u128,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct OnCreate {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GetBroadcast {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    request_code: i64,
    pub action: String,
    pub component: String,
    pub data: String,
    pub extras: Vec<(String, String)>,
    pub intent_package: String,
    flags: i64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoFinal {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub ret: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoFinalByte {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    pub ret: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoFinalByteInt {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_output: String,
    arg_output_offset: i64,
    #[serde(deserialize_with = "null_to_default")]
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoFinalByteIntInt {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    arg_input_offset: i64,
    arg_input_len: i64,
    pub ret: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoFinalByteIntIntByte {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    arg_input_offset: i64,
    arg_input_len: i64,
    pub arg_output: String,
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoFinalByteIntIntByteInt {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    arg_input_offset: i64,
    arg_input_len: i64,
    pub arg_output: String,
    arg_output_offset: i64,
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoFinalByteBufferByteBuffer {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    pub arg_output: String,
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct InitIntCertificate {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_opcode: i64,
    pub arg_certificate: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct InitIntKey {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_opcode: i64,
    pub arg_key: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateByte {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    pub ret: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateByteIntInt {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    arg_input_offset: i64,
    arg_input_len: i64,
    pub ret: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateByteIntIntByte {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    arg_input_offset: i64,
    arg_input_len: i64,
    pub arg_output: String,
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateByteIntIntByteInt {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    arg_input_offset: i64,
    arg_input_len: i64,
    pub arg_output: String,
    arg_output_offset: i64,
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateByteBufferByteBuffer {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    pub arg_output: String,
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateAADByte {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    pub ret: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateAADByteIntInt {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_input: String,
    arg_input_offset: i64,
    arg_input_len: i64,
    pub ret: String,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateAADByteBuffer {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub arg_src: String,
    ret: i64,
    pub algorithm: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct StartForeground {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    id: i64,
    r#type: i64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct OnStartCommand {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub action: String,
    pub component: String,
    pub data: String,
    pub extras: Vec<(String, String)>,
    pub intent_package: String,
    flags: i64,
    start_id: i64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct OnBind {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub action: String,
    pub component: String,
    pub data: String,
    pub extras: Vec<(String, String)>,
    pub intent_package: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SetImpl {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    r#type: i64,
    pub trigger_at_millis: i64,
    pub window_millis: i64,
    pub interval_millis: i64,
    flags: i64,
    pub operation_hashcode: i64,
    listener_class_name: String,
    listener_tag: String,
    alarm_clock_intent_hash_code: i64,
    alarm_clock_get_trigger_time: i64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SetAlarmClock {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    creator: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PendingIntentGet {
    class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    request_code: i64,
    pub action: String,
    pub component: String,
    pub data: String,
    pub extras: Vec<(String, String)>,
    pub intent_package: String,
    flags: i64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    pub package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ScheduleInternal {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub job_id: i64,
    pub work_id: String,
    pub pid: i64,
    pub tid: i64,
    pub uid: i64,
    pub package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct RequestWork {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    work_id: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DoWork {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub work_id: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GenericHook {
    pub class_name: String,
    pub method_name: String,
    pub hashcode: i64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct RegisterReceiver {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub register_class: String,
    pub actions: Vec<String>,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    pub package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DeviceID {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    device_id: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct StartThreadHook {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    new_jtid: u64,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Canary {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    pub canary: String,
    pub target: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

impl Canary {
    pub fn get_work_name(&self) -> Vec<(String, String)> {
        let Some(st) = self.get_stack_trace() else {
            return Vec::new();
        };
        let Some(indicator) = self.canary.chars().into_iter().next() else {
            return Vec::new();
        };
        match indicator {
            '1' => Self::extract_work_name(&st, "onReceive"),
            '2' => Self::extract_work_name(&st, "doWork"),
            '3' => Self::extract_work_name(&st, "onStartJob"),
            '4' => Self::extract_work_name(&st, "onStartCommand"),
            '5' => Self::extract_work_name(&st, "onBind"),
            '6' => Self::extract_work_name(&st, "onRebind"),
            '7' => Self::extract_work_name(&st, "onCreate"),
            _ => Vec::new(),
        }
    }
    fn extract_work_name(st: &Vec<String>, m: &str) -> Vec<(String, String)> {
        let mut classess = Vec::new();
        for s in st {
            let Some((class_name, method_name)) = s
                .split_once("(")
                .map(|s| s.0)
                .and_then(|x| x.rsplit_once("."))
            else {
                continue;
            };
            if method_name.starts_with(m) && !class_name.starts_with("LSPHooker_") {
                classess.push((class_name.to_string(), method_name.to_string()));
            }
        }
        classess
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SocketHook {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    local_addr: String,
    remote_addr: String,
    pub canary: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

impl SocketHook {
    pub fn get_local_addr(&self) -> Option<(std::net::Ipv4Addr, u16)> {
        let socket_addr = if self.local_addr.contains("/") {
            let (_, socket_addr) = self.local_addr.split_once("/")?;
            socket_addr
        } else {
            &self.local_addr
        };
        std::net::SocketAddr::from_str(socket_addr)
            .ok()
            .and_then(|s| match s {
                std::net::SocketAddr::V4(addr) => Some((addr.ip().to_owned(), addr.port())),
                std::net::SocketAddr::V6(addr) => {
                    addr.ip().to_owned().to_ipv4().map(|ip| (ip, addr.port()))
                }
            })
    }
    pub fn get_remote_addr(&self) -> Option<(std::net::Ipv4Addr, u16)> {
        let socket_addr = if self.remote_addr.contains("/") {
            let (_, socket_addr) = self.remote_addr.split_once("/")?;
            socket_addr
        } else {
            &self.remote_addr
        };
        std::net::SocketAddr::from_str(socket_addr)
            .ok()
            .and_then(|s| match s {
                std::net::SocketAddr::V4(addr) => Some((addr.ip().to_owned(), addr.port())),
                std::net::SocketAddr::V6(addr) => {
                    addr.ip().to_owned().to_ipv4().map(|ip| (ip, addr.port()))
                }
            })
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ConnectHook {
    class_name: String,
    method_name: String,
    pub hashcode: i64,
    local_addr: String,
    remote_addr: String,
    pub pid: i64,
    pub tid: i64,
    uid: i64,
    package_name: String,
    pub ts: u128,
    jtid: u64,
    pub importance: u64,
    pub stack_trace: String,
}

// pub struct StackTrace(pub <String>);

pub trait StackTrace {
    fn get_encoded_stacktrace(&self) -> Option<&str>;
    fn get_stack_trace(&self) -> Option<Vec<String>> {
        let st = self.get_encoded_stacktrace()?;
        let o = BASE64_STANDARD.decode(&st).ok()?;
        let mut result = Vec::new();
        GzDecoder::new(o.as_slice()).read_to_end(&mut result).ok()?;
        String::from_utf8(result)
            .ok()
            .map(|x| x.lines().map(|x| x.to_string()).collect::<Vec<String>>())
    }
    fn extract_canary_info(&self) -> Option<(Option<String>, u32, u32)> {
        self.get_stack_trace().as_ref().and_then(|st| {
            st.iter().find_map(|s| {
                if !s.starts_with("StackTracer_") {
                    return None;
                }
                let mut v = s
                    .split_once(".")
                    .map(|(v, _)| v.split("_"))?
                    .filter(|v| !v.is_empty() && v != &"StackTracer")
                    .collect::<Vec<&str>>();

                let tid = v.pop().and_then(|v| u32::from_str(v).ok())?;
                let pid = v.pop().and_then(|v| u32::from_str(v).ok())?;
                let canary = v.pop().map(|v| v.to_string());
                Some((canary, pid, tid))
            })
        })
    }
    fn get_parent_work(&self) -> Option<(String, String)> {
        self.get_stack_trace().as_ref().and_then(|st| {
            st.iter().find_map(|s| {
                if !s.starts_with("LSPHooker_") {
                    return None;
                }
                let (class, method) = s
                    .split_once("(")
                    .and_then(|(v, _)| v.split_once("_").and_then(|(_, v)| v.rsplit_once(".")))?;
                if !BACKGROUND_WORK_METHODS.contains(&method) {
                    return None;
                }
                Some((class.to_string(), method.to_string()))
            })
        })
    }
}

impl StackTrace for &JobScheduler {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}

impl StackTrace for &OnStartJob {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &Intent {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &OnReceiveHook {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &OnCreate {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &GetBroadcast {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoFinal {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoFinalByte {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoFinalByteInt {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoFinalByteIntInt {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoFinalByteIntIntByte {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoFinalByteIntIntByteInt {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoFinalByteBufferByteBuffer {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &InitIntCertificate {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &InitIntKey {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateByte {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateByteIntInt {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateByteIntIntByte {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateByteIntIntByteInt {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateByteBufferByteBuffer {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateAADByte {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateAADByteIntInt {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &UpdateAADByteBuffer {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &StartForeground {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &OnStartCommand {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &OnBind {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &SetImpl {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &SetAlarmClock {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &PendingIntentGet {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &ScheduleInternal {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &RequestWork {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DoWork {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &GenericHook {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &RegisterReceiver {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &DeviceID {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &Canary {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &StartThreadHook {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &SchedProcessFork {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        None
    }
}
impl StackTrace for &TaskInfo {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        None
    }
}
impl StackTrace for &SocketAddrV6 {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        None
    }
}
impl StackTrace for &SocketAddrV4 {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        None
    }
}
impl StackTrace for &SocketHook {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}
impl StackTrace for &ConnectHook {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        Some(self.stack_trace.as_str())
    }
}

impl<T> StackTrace for Option<T> {
    fn get_encoded_stacktrace(&self) -> Option<&str> {
        None
    }
}
use serde::Deserializer;
fn null_to_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let key = Option::<T>::deserialize(de)?;
    Ok(key.unwrap_or_default())
}