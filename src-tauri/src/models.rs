use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};

// Global map to store running server processes
pub struct ChildProcessMap(pub Arc<Mutex<HashMap<String, Child>>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceInstallProgress {
    pub id: String,
    pub step: String,
    pub progress: u64,
    pub total_size: Option<u64>,
    pub downloaded: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InstanceEngine {
    Vanilla,
    Paper,
    Fabric,
    Forge,
    NeoForge,
    Quilt,
    Spigot,
    Purpur,
    Folia,
    Velocity,
    Waterfall,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InstanceState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceSettings {
    pub min_ram: u64, // MB
    pub max_ram: u64, // MB
    pub port: u16,
    pub args: String,
    pub jar_file: String,
    pub java_path: Option<String>,
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            min_ram: 1024,
            max_ram: 4096,
            port: 25565,
            args: String::new(),
            jar_file: String::from("server.jar"),
            java_path: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub loader: InstanceEngine,
    pub version: String,
    pub path: String, // Slug path relative to instances dir
    pub date_created: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
    pub state: InstanceState,
    #[serde(default)]
    pub settings: InstanceSettings,
    #[serde(default)]
    pub build: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Addon {
    pub file_name: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub size: u64,
    pub last_modified: i64, // Unix timestamp
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddonCache {
    pub last_scan: i64,
    pub addons: Vec<Addon>,
}
