// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InstanceEngine {
    Vanilla,
    Paper,
    Fabric,
    Forge,
    Spigot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InstanceState {
    Stopped,
    Starting,
    Running,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instance {
    pub uuid: String,
    pub name: String,
    pub motor: InstanceEngine,
    pub version: String,
    pub path: String,
    pub icon: Option<String>,
    pub ram_min: u32,
    pub ram_max: u32,
    pub port: u16,
    pub state: InstanceState,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
