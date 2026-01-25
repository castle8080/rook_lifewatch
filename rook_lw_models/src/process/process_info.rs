use serde::{Serialize, Deserialize};

use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub cmd: String,
    pub status: String,
    pub started: DateTime<FixedOffset>,
    pub memory: u64,
    pub cpu_usage_percent: f32,
    pub cpu_accumulated_time: u64,
    pub thread_kind: Option<String>,
}
