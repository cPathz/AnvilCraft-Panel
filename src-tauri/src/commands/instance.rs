use crate::commands::versions::{install_project_server, install_vanilla};
use crate::models::{
    ChildProcessMap, Instance, InstanceEngine, InstanceInstallProgress, InstanceSettings,
    InstanceState,
};
use chrono::Utc;
use slug::slugify;
use std::fs;
use std::path::PathBuf;
use tauri::{Emitter, Manager, State};
use uuid::Uuid;

#[tauri::command]
pub async fn create_instance(
    app: tauri::AppHandle,
    name: String,
    loader: String,
    version: String,
    icon: String,
    custom_download_url: Option<String>,
) -> Result<String, String> {
    // 1. Resolve AppData path
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data_dir.join("instances");

    if !instances_dir.exists() {
        fs::create_dir_all(&instances_dir).map_err(|e| e.to_string())?;
    }

    // 2. Generate Slug
    let mut slug = slugify(&name);
    let id = Uuid::new_v4().to_string();

    // 3. Handle duplicates
    let mut instance_path = instances_dir.join(&slug);
    let mut counter = 1;
    while instance_path.exists() {
        slug = format!("{}-{}", slugify(&name), counter);
        instance_path = instances_dir.join(&slug);
        counter += 1;
    }

    // 4. Create Dir
    fs::create_dir_all(&instance_path).map_err(|e| e.to_string())?;
    fs::create_dir_all(instance_path.join(".minecraft")).map_err(|e| e.to_string())?;

    // 5. Engine
    let engine = match loader.as_str() {
        "Fabric" => InstanceEngine::Fabric,
        "Forge" => InstanceEngine::Forge,
        "Paper" => InstanceEngine::Paper,
        "Spigot" => InstanceEngine::Spigot,
        "Purpur" => InstanceEngine::Purpur,
        "Folia" => InstanceEngine::Folia,
        "Velocity" => InstanceEngine::Velocity,
        "Waterfall" => InstanceEngine::Waterfall,
        _ => InstanceEngine::Vanilla,
    };

    // Calculate build (simplified)
    let build = if let Some(url) = &custom_download_url {
        url.split('/').last().and_then(|f| {
            if f.ends_with(".jar") {
                f.trim_end_matches(".jar")
                    .split('-')
                    .last()
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
    } else {
        None
    };

    let instance = Instance {
        id: id.clone(),
        name,
        icon,
        loader: engine,
        version: version.clone(),
        path: slug,
        date_created: Utc::now(),
        last_played: None,
        state: InstanceState::Stopped,
        settings: InstanceSettings::default(),
        build,
    };

    // 6. Save JSON
    let json_path = instance_path.join("instance.json");
    let json_content = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    fs::write(json_path, json_content).map_err(|e| e.to_string())?;

    // 7. Background Install
    let app_handle = app.clone();
    let instance_id = id.clone();
    let instance_version = version.clone();
    let instance_path_clone = instance_path.clone();
    let loader_engine = instance.loader.clone();
    let custom_url_clone = custom_download_url.clone();

    tauri::async_runtime::spawn(async move {
        let _ = app_handle.emit(
            "install-progress",
            InstanceInstallProgress {
                id: instance_id.clone(),
                step: "Creating files...".into(),
                progress: 10,
                total_size: None,
                downloaded: 0,
            },
        );
        std::thread::sleep(std::time::Duration::from_millis(500));

        let result = match loader_engine {
            InstanceEngine::Paper
            | InstanceEngine::Purpur
            | InstanceEngine::Spigot
            | InstanceEngine::Folia
            | InstanceEngine::Velocity
            | InstanceEngine::Waterfall => {
                let project = match loader_engine {
                    InstanceEngine::Purpur => "purpur",
                    InstanceEngine::Spigot => "spigot",
                    InstanceEngine::Folia => "folia",
                    InstanceEngine::Velocity => "velocity",
                    InstanceEngine::Waterfall => "waterfall",
                    _ => "paper",
                };
                install_project_server(
                    app_handle.clone(),
                    instance_path_clone.join(".minecraft"),
                    instance_version,
                    instance_id.clone(),
                    project.to_string(),
                    custom_url_clone,
                )
                .await
            }
            _ => {
                install_vanilla(
                    &app_handle,
                    &instance_id,
                    &instance_version,
                    &instance_path_clone,
                )
                .await
            }
        };

        if let Err(e) = result {
            let _ = app_handle.emit(
                "install-progress",
                InstanceInstallProgress {
                    id: instance_id,
                    step: format!("Error: {}", e),
                    progress: 0,
                    total_size: None,
                    downloaded: 0,
                },
            );
        }
    });

    Ok(id)
}

#[tauri::command]
pub async fn read_instances(
    app: tauri::AppHandle,
    state: State<'_, ChildProcessMap>,
) -> Result<Vec<Instance>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    if !instances_dir.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();
    let map = state.0.lock().map_err(|_| "Failed to lock state")?;

    for entry in fs::read_dir(instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_dir() {
            let json_path = entry.path().join("instance.json");
            if json_path.exists() {
                if let Ok(content) = fs::read_to_string(json_path) {
                    if let Ok(mut instance) = serde_json::from_str::<Instance>(&content) {
                        // Sync state
                        if map.contains_key(&instance.id) {
                            instance.state = InstanceState::Running;
                        } else if instance.state == InstanceState::Running
                            || instance.state == InstanceState::Starting
                        {
                            instance.state = InstanceState::Stopped;
                        }
                        instances.push(instance);
                    }
                }
            }
        }
    }

    instances.sort_by(|a, b| b.date_created.cmp(&a.date_created));
    Ok(instances)
}

#[tauri::command]
pub async fn delete_instance(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    // Find instance path
    let mut target_path = PathBuf::new();
    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(json_path).unwrap_or_default();
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    target_path = entry.path();
                    break;
                }
            }
        }
    }

    if target_path.exists() {
        fs::remove_dir_all(target_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn update_instance_icon(
    app: tauri::AppHandle,
    id: String,
    icon: String,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(mut inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    inst.icon = icon.clone();
                    let new_json =
                        serde_json::to_string_pretty(&inst).map_err(|e| e.to_string())?;
                    fs::write(json_path, new_json).map_err(|e| e.to_string())?;
                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

#[tauri::command]
pub async fn save_instance_settings(
    instance_id: String,
    settings: InstanceSettings,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(mut inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == instance_id {
                    inst.settings = settings;
                    let new_json =
                        serde_json::to_string_pretty(&inst).map_err(|e| e.to_string())?;
                    fs::write(json_path, new_json).map_err(|e| e.to_string())?;
                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

#[tauri::command]
pub async fn open_instances_folder(app: tauri::AppHandle, slug: String) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = app_data.join("instances").join(slug);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
