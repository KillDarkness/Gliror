use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub url: Option<String>,
    pub host: Option<String>,
    pub target_port: Option<u16>,
    pub attack_type: Option<String>,
    pub time: Option<u64>,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub random_ua: Option<bool>,
    pub data: Option<String>,
    pub proxy: Option<String>,
    pub concurrent: Option<u32>,
    pub delay: Option<u64>,
    pub ramp_up: Option<u64>,
    pub schedule: Option<String>,
    pub cluster_mode: Option<bool>,
    pub worker_id: Option<String>,
    pub coordinator_addr: Option<String>,
    pub total_workers: Option<usize>,
    pub port: Option<u16>,
    pub role: Option<String>,
    pub distribution_mode: Option<String>,
}
