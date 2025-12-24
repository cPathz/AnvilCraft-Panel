// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use slug::slugify;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InstanceEngine {
    Vanilla,
    Paper,
    Fabric,
    Forge,
    NeoForge,
    Quilt,
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
    pub id: String,
    pub name: String,
    pub icon: String,
    pub loader: InstanceEngine,
    pub version: String,
    pub path: String, // Slug path relative to instances dir
    pub date_created: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
    pub state: InstanceState,
}

#[tauri::command]
async fn create_instance(
    app: tauri::AppHandle,
    name: String,
    loader: String,
    version: String,
    icon: String,
) -> Result<String, String> {
    // 1. Resolve AppData path
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data_dir.join("instances");

    if !instances_dir.exists() {
        fs::create_dir_all(&instances_dir).map_err(|e| e.to_string())?;
    }

    // 2. Generate Slug and unique ID
    let mut slug = slugify(&name);
    let id = Uuid::new_v4().to_string();

    // 3. Handle duplicate folders
    let mut instance_path = instances_dir.join(&slug);
    let mut counter = 1;
    while instance_path.exists() {
        slug = format!("{}-{}", slugify(&name), counter);
        instance_path = instances_dir.join(&slug);
        counter += 1;
    }

    // 4. Create Directory Structure
    fs::create_dir_all(&instance_path).map_err(|e| e.to_string())?;
    fs::create_dir_all(instance_path.join(".minecraft")).map_err(|e| e.to_string())?;

    // 5. Create Instance Metadata
    let engine = match loader.as_str() {
        "Fabric" => InstanceEngine::Fabric,
        "Forge" => InstanceEngine::Forge,
        "NeoForge" => InstanceEngine::NeoForge,
        "Quilt" => InstanceEngine::Quilt,
        "Paper" => InstanceEngine::Paper,
        _ => InstanceEngine::Vanilla,
    };

    let instance = Instance {
        id: id.clone(),
        name,
        icon,
        loader: engine,
        version,
        path: slug,
        date_created: Utc::now(),
        last_played: None,
        state: InstanceState::Stopped,
    };

    // 6. Save instance.json
    let json_path = instance_path.join("instance.json");
    let json_content = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    fs::write(json_path, json_content).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
async fn read_instances(app: tauri::AppHandle) -> Result<Vec<Instance>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data_dir.join("instances");

    if !instances_dir.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();
    let entries = fs::read_dir(instances_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            let json_path = path.join("instance.json");
            if json_path.exists() {
                let content = fs::read_to_string(json_path).map_err(|e| e.to_string())?;
                match serde_json::from_str::<Instance>(&content) {
                    Ok(instance) => instances.push(instance),
                    Err(e) => println!("Error parsing instance at {:?}: {}", path, e),
                }
            }
        }
    }

    // Sort by last played (descending) or date created
    instances.sort_by(|a, b| b.date_created.cmp(&a.date_created));

    Ok(instances)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_instance,
            read_instances
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
