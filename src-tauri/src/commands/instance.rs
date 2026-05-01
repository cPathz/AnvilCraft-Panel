use crate::commands::versions::{install_project_server, install_vanilla};
use crate::models::{
    ChildProcessMap, Instance, InstanceEngine, InstanceInstallProgress, InstanceSettings,
    InstanceState, Addon, AddonCache, AddonAnalysis, AddonInstallItem
};
use chrono::Utc;
use slug::slugify;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use sysinfo::System;
use tauri::{Emitter, Manager, State};
use uuid::Uuid;
use zip::ZipArchive;

#[tauri::command]
pub async fn create_instance(
    app: tauri::AppHandle,
    name: String,
    loader: String,
    version: String,
    icon: String,
    custom_download_url: Option<String>,
    accept_eula: bool,
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

    // 6.5 handle EULA (moved to background install for Vanilla/Paper)
    // For local imports we still handle it here if it's not a background task

    // 7. Background Install
    let app_handle = app.clone();
    let instance_id = id.clone();
    let instance_version = version.clone();
    let instance_path_clone = instance_path.clone();
    let loader_engine = instance.loader.clone();
    let custom_url_clone = custom_download_url.clone();
    let accept_eula_clone = accept_eula;

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
                    accept_eula_clone,
                )
                .await
            }
            _ => {
                install_vanilla(
                    &app_handle,
                    &instance_id,
                    &instance_version,
                    &instance_path_clone,
                    accept_eula_clone,
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
                        if !map.contains_key(&instance.id) {
                            if instance.state == InstanceState::Running
                                || instance.state == InstanceState::Starting
                                || instance.state == InstanceState::Stopping
                            {
                                instance.state = InstanceState::Stopped;
                            }
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
pub async fn delete_instance(
    app: tauri::AppHandle,
    state: State<'_, ChildProcessMap>,
    id: String,
) -> Result<(), String> {
    // 0. Resolve Instance Path First
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

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

    if !target_path.exists() {
        return Ok(()); // Nothing to delete
    }

    // 1. Force Kill Orphan Processes (via Sysinfo)
    // Scans all processes for a java process running inside this instance folder
    {
        let sys = System::new_all();
        // sys.refresh_processes(); REMOVED (new_all refreshes everything)

        let target_path_str = target_path.to_string_lossy().to_string();

        for (pid, process) in sys.processes() {
            // process.name() is &str in newer or &OsStr in some?
            // Error: "no method to_lowercase for &OsStr". So it is &OsStr?
            // Wait, newer sysinfo returns &str for name().
            // If error says &OsStr, then it returns &OsStr?
            // Let's assume &OsStr and convert.
            if process
                .name()
                .to_string_lossy()
                .to_lowercase()
                .contains("java")
            {
                // Check CWD
                let cwd_match = process
                    .cwd()
                    .map_or(false, |p| p.to_string_lossy().contains(&target_path_str));

                // Check Arguments
                let args_match = process
                    .cmd()
                    .iter()
                    .any(|arg| arg.to_string_lossy().contains(&target_path_str));

                if cwd_match || args_match {
                    println!(
                        "Killing orphan process: {} ({})",
                        process.name().to_string_lossy(),
                        pid
                    );
                    process.kill();
                }
            }
        }
    }

    // 2. Cleanup ChildProcessMap (if still tracked)
    {
        let mut map = state.0.lock().map_err(|_| "Failed to lock state")?;
        if let Some(child) = map.get_mut(&id) {
            let _ = child.kill();
            map.remove(&id);
        }
    }

    // 3. Delete Files
    if target_path.exists() {
        // Retry logic for Windows file locks
        let mut attempts = 0;
        while attempts < 3 {
            match fs::remove_dir_all(&target_path) {
                Ok(_) => break,
                Err(e) => {
                    if attempts == 2 {
                        return Err(format!("Failed to delete after 3 attempts: {}", e));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
            attempts += 1;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn update_instance_name(
    app: tauri::AppHandle,
    id: String,
    name: String,
) -> Result<(), String> {
    if name.trim().len() > 30 {
        return Err("Name too long".to_string());
    }
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(mut inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    inst.name = name.clone();
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
pub async fn update_instance_version(
    app: tauri::AppHandle,
    id: String,
    version: String,
    build: Option<String>,
    loader: Option<String>,
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
                    inst.version = version;
                    if build.is_some() {
                        inst.build = build;
                    }
                    if let Some(l) = loader {
                        inst.loader = match l.as_str() {
                            "Fabric" => InstanceEngine::Fabric,
                            "Forge" => InstanceEngine::Forge,
                            "Paper" => InstanceEngine::Paper,
                            "Spigot" => InstanceEngine::Spigot,
                            "Purpur" => InstanceEngine::Purpur,
                            "Folia" => InstanceEngine::Folia,
                            "Velocity" => InstanceEngine::Velocity,
                            "Waterfall" => InstanceEngine::Waterfall,
                            "NeoForge" => InstanceEngine::NeoForge,
                            "Quilt" => InstanceEngine::Quilt,
                            _ => InstanceEngine::Vanilla,
                        };
                    }
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
pub async fn get_instance_addons_type(app: tauri::AppHandle, id: String) -> Result<String, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    if !instances_dir.exists() {
        return Ok("none".to_string());
    }

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    let dot_minecraft = entry.path().join(".minecraft");
                    let mods_path = dot_minecraft.join("mods");
                    let plugins_path = dot_minecraft.join("plugins");
                    
                    if mods_path.exists() && mods_path.is_dir() {
                        return Ok("mods".to_string());
                    }
                    if plugins_path.exists() && plugins_path.is_dir() {
                        return Ok("plugins".to_string());
                    }
                    return Ok("none".to_string());
                }
            }
        }
    }
    Ok("none".to_string())
}

fn extract_addon_metadata(path: &PathBuf) -> Option<Addon> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();
    let last_modified = metadata.modified().ok()?
        .duration_since(std::time::UNIX_EPOCH).ok()?
        .as_secs() as i64;
    let enabled = !file_name.ends_with(".disabled") || file_name.ends_with(".bkp") || file_name.ends_with(".bak") || file_name.ends_with(".old") || file_name.ends_with(".off");

    let mut name = file_name.clone();
    let mut version = "Unknown".to_string();
    let mut author: Option<String> = None;
    let mut description: Option<String> = None;

    // 1. Check for fabric.mod.json (Fabric)
    if let Ok(mut fabric_file) = archive.by_name("fabric.mod.json") {
        let mut content = String::new();
        if fabric_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = json.get("authors").and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| {
                    if v.is_string() { v.as_str() } else { v.get("name").and_then(|n| n.as_str()) }
                }).map(|s| s.to_string());
                description = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                let platform = "Fabric".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 2. Check for quilt.mod.json (Quilt)
    if let Ok(mut quilt_file) = archive.by_name("quilt.mod.json") {
        let mut content = String::new();
        if quilt_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let quat = json.get("quilt_loader").or(json.get("metadata"));
                if let Some(m) = quat {
                    if let Some(n) = m.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                    if let Some(v) = m.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                    author = m.get("contributors").and_then(|v| v.as_object()).and_then(|o| o.keys().next()).map(|s| s.to_string());
                    description = m.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
                let platform = "Quilt".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 3. Check for mods.toml (Forge 1.13+)
    if let Ok(mut forge_file) = archive.by_name("META-INF/mods.toml") {
        let mut content = String::new();
        if forge_file.read_to_string(&mut content).is_ok() {
            if let Ok(toml_val) = toml::from_str::<toml::Value>(&content) {
                if let Some(mods_array) = toml_val.get("mods").and_then(|v| v.as_array()) {
                    if let Some(mods) = mods_array.get(0) {
                        if let Some(n) = mods.get("displayName").and_then(|v| v.as_str()) { name = n.to_string(); }
                        if let Some(v) = mods.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                        author = mods.get("authors").and_then(|v| v.as_str()).map(|s| s.to_string());
                        description = mods.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let platform = "Forge".to_string();
                        return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
                    }
                }
            }
        }
    }

    // 4. Check for mcmod.info (Legacy Forge)
    if let Ok(mut mcmod_file) = archive.by_name("mcmod.info") {
        let mut content = String::new();
        if mcmod_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let mod_obj = if json.is_array() { json.get(0) } else { json.get("modList").and_then(|l| l.get(0)).or(Some(&json)) };
                if let Some(m) = mod_obj {
                    if let Some(n) = m.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                    if let Some(v) = m.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                    author = m.get("authorList").and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
                    description = m.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
                let platform = "Forge (Legacy)".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 5. Check for paper-plugin.yml (Modern Paper)
    if let Ok(mut paper_file) = archive.by_name("paper-plugin.yml") {
        let mut content = String::new();
        if paper_file.read_to_string(&mut content).is_ok() {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(n) = yaml.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = yaml.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = yaml.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
                description = yaml.get("description").and_then(|v| v.as_str()).map(|s: &str| s.to_string());
                let platform = "Paper".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 6. Check for plugin.yml (Spigot/Bukkit)
    if let Ok(mut plugin_file) = archive.by_name("plugin.yml") {
        let mut content = String::new();
        if plugin_file.read_to_string(&mut content).is_ok() {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(n) = yaml.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = yaml.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = yaml.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
                description = yaml.get("description").and_then(|v| v.as_str()).map(|s: &str| s.to_string());
                let platform = "Spigot".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 7. Check for bungee.yml (BungeeCord)
    if let Ok(mut bungee_file) = archive.by_name("bungee.yml") {
        let mut content = String::new();
        if bungee_file.read_to_string(&mut content).is_ok() {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(n) = yaml.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = yaml.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = yaml.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
                let platform = "Bungee".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }

    // 8. Check for velocity-plugin.json (Velocity)
    if let Ok(mut velocity_file) = archive.by_name("velocity-plugin.json") {
        let mut content = String::new();
        if velocity_file.read_to_string(&mut content).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("name").and_then(|v| v.as_str()) { name = n.to_string(); }
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) { version = v.to_string(); }
                author = json.get("authors").and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
                description = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                let platform = "Velocity".to_string();
                return Some(Addon { file_name, name, version, author, description, enabled, size, last_modified, platform });
            }
        }
    }
    
    None
}

fn get_addons_internal(
    target_dir: &Path,
    cache_path: &Path,
    force_scan: bool,
) -> Result<Vec<Addon>, String> {
    if !target_dir.exists() {
        return Ok(vec![]);
    }

    // 1. Load Cache
    let mut cache = if !force_scan && cache_path.exists() {
        let cache_content = fs::read_to_string(&cache_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<AddonCache>(&cache_content).unwrap_or(AddonCache { last_scan: 0, addons: vec![] })
    } else {
        AddonCache { last_scan: 0, addons: vec![] }
    };

    // 2. Scan Directory
    let mut files_to_scan = Vec::new();
    let mut current_addons = Vec::new();
    let mut cache_modified = false;

    for entry in fs::read_dir(target_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if file_name.ends_with(".jar") || file_name.ends_with(".disabled") || file_name.ends_with(".bkp") || file_name.ends_with(".bak") || file_name.ends_with(".old") || file_name.ends_with(".off") {
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let size = metadata.len();
                let last_modified = metadata.modified().map_err(|e| e.to_string())?
                    .duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?
                    .as_secs() as i64;

                // Check Cache
                if let Some(cached) = cache.addons.iter().find(|a| a.file_name == file_name && a.size == size && a.last_modified == last_modified) {
                    current_addons.push(cached.clone());
                } else {
                    files_to_scan.push(path.clone());
                    cache_modified = true;
                }
            }
        }
    }

    // 3. Parallel Scan for new/changed files
    if !files_to_scan.is_empty() {
        let new_addons: Vec<Addon> = files_to_scan.par_iter()
            .filter_map(|path: &PathBuf| extract_addon_metadata(path))
            .collect();
        current_addons.extend(new_addons);
        cache_modified = true;
    }

    // 4. Check for deletions
    if current_addons.len() != cache.addons.len() {
        cache_modified = true;
    }

    // 5. Update Cache File
    if cache_modified || force_scan {
        cache.addons = current_addons.clone();
        cache.last_scan = Utc::now().timestamp();
        let new_cache_json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
        fs::write(cache_path, new_cache_json).map_err(|e| e.to_string())?;
    }

    Ok(current_addons)
}


#[tauri::command]
pub async fn get_instance_addons(app: tauri::AppHandle, id: String, force_scan: bool) -> Result<Vec<Addon>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    let dot_minecraft = entry.path().join(".minecraft");
                    let mods_path = dot_minecraft.join("mods");
                    let plugins_path = dot_minecraft.join("plugins");
                    let cache_path = entry.path().join("addons_cache.json");

                    let target_dir = if mods_path.exists() {
                        mods_path
                    } else {
                        plugins_path
                    };

                    return get_addons_internal(&target_dir, &cache_path, force_scan);
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
                    inst.settings = settings.clone();
                    let new_json =
                        serde_json::to_string_pretty(&inst).map_err(|e| e.to_string())?;
                    fs::write(json_path, new_json).map_err(|e| e.to_string())?;

                    // Sync server.properties
                    let props_path = entry.path().join(".minecraft").join("server.properties");
                    let _ = sync_server_properties(&props_path, settings.port);

                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

#[tauri::command]
pub async fn get_instance_max_players(app: tauri::AppHandle, id: String) -> Result<u16, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    if instances_dir.exists() {
        for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let json_path = entry.path().join("instance.json");
            if json_path.exists() {
                let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
                if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                    if inst.id == id {
                        let props_path = entry.path().join(".minecraft").join("server.properties");
                        if props_path.exists() {
                            if let Ok(props) = fs::read_to_string(props_path) {
                                for line in props.lines() {
                                    if line.starts_with("max-players=") {
                                        if let Ok(p) = line
                                            .replace("max-players=", "")
                                            .trim()
                                            .parse::<u16>()
                                        {
                                            return Ok(p);
                                        }
                                    }
                                }
                            }
                        }
                        return Ok(20); // Default if not found
                    }
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

#[tauri::command]
pub async fn get_instance_port(app: tauri::AppHandle, id: String) -> Result<u16, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    if instances_dir.exists() {
        for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let json_path = entry.path().join("instance.json");
            if json_path.exists() {
                let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
                if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                    if inst.id == id {
                        let props_path = entry.path().join(".minecraft").join("server.properties");
                        if props_path.exists() {
                            if let Ok(props) = fs::read_to_string(props_path) {
                                for line in props.lines() {
                                    if line.starts_with("server-port=") {
                                        if let Ok(p) = line
                                            .replace("server-port=", "")
                                            .trim()
                                            .parse::<u16>()
                                        {
                                            return Ok(p);
                                        }
                                    }
                                }
                            }
                        }
                        return Ok(inst.settings.port);
                    }
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

fn sync_server_properties(path: &std::path::Path, port: u16) -> Result<(), String> {
    if !path.exists() {
        // Create basic template if it doesn't exist
        fs::write(
            path,
            format!("server-port={}\nquery.port={}\n", port, port),
        )
        .map_err(|e| e.to_string())?;
        return Ok(());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut found_port = false;
    let mut found_query = false;

    for line in lines.iter_mut() {
        if line.trim().starts_with("server-port=") {
            *line = format!("server-port={}", port);
            found_port = true;
        } else if line.trim().starts_with("query.port=") {
            *line = format!("query.port={}", port);
            found_query = true;
        }
    }

    if !found_port {
        lines.push(format!("server-port={}", port));
    }
    if !found_query {
        lines.push(format!("query.port={}", port));
    }

    fs::write(path, lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_instances_folder(app: tauri::AppHandle, slug: Option<String>) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let slug_val = slug.unwrap_or_default();
    let instance_root = app_data.join("instances").join(slug_val);
    let dot_minecraft = instance_root.join(".minecraft");

    let path = if dot_minecraft.exists() {
        dot_minecraft
    } else {
        instance_root
    };

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

#[tauri::command]
pub async fn detect_minecraft_version(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    let filename = path
        .file_name()
        .map_or("", |f| f.to_str().unwrap_or(""))
        .to_lowercase();

    if path.is_file() {
        // Try to read JAR
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

        // 1. Try version.json
        let version_id = {
            if let Ok(mut version_file) = archive.by_name("version.json") {
                let mut content = String::new();
                let _ = version_file.read_to_string(&mut content);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json["id"]
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| json["name"].as_str().map(|s| s.to_string()))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(v) = version_id {
            if filename.contains("-installer") {
                return Ok(format!("Forge/NeoForge Installer ({})", v));
            }
            return Ok(v);
        }

        // 2. Try patch.json (Fabric/Quilt)
        let patch_version = {
            if let Ok(mut patch_file) = archive.by_name("patch.json") {
                let mut content = String::new();
                let _ = patch_file.read_to_string(&mut content);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json["version"].as_str().map(|v| v.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(v) = patch_version {
            return Ok(v);
        }

        // 3. Special case for Forge installer filename if internal search fails
        if filename.contains("forge-") || filename.contains("neoforge-") {
            if let Some(v) = filename.split('-').nth(1) {
                if v.chars().all(|c| c.is_digit(10) || c == '.') {
                    return Ok(v.to_string());
                }
            }
        }
    } else if path.is_dir() {
        // ... (existing dir detection)
        let instance_json = path.join("instance.json");
        if instance_json.exists() {
            if let Ok(content) = fs::read_to_string(instance_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json["version"].as_str() {
                        return Ok(version.to_string());
                    }
                }
            }
        }

        let version_json = path.join("version.json");
        if version_json.exists() {
            if let Ok(content) = fs::read_to_string(version_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(id) = json["id"].as_str() {
                        return Ok(id.to_string());
                    }
                }
            }
        }
    }

    Err("Could not detect version automatically".to_string())
}

#[tauri::command]
pub async fn create_instance_from_path(
    app: tauri::AppHandle,
    name: String,
    icon: String,
    source_path: String,
    is_file: bool,
    version: String,
    accept_eula: bool,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data_dir.join("instances");

    if !instances_dir.exists() {
        fs::create_dir_all(&instances_dir).map_err(|e| e.to_string())?;
    }

    let mut slug = slugify(&name);
    let id = Uuid::new_v4().to_string();

    let mut instance_path = instances_dir.join(&slug);
    let mut counter = 1;
    while instance_path.exists() {
        slug = format!("{}-{}", slugify(&name), counter);
        instance_path = instances_dir.join(&slug);
        counter += 1;
    }

    fs::create_dir_all(&instance_path).map_err(|e| e.to_string())?;
    let dot_minecraft = instance_path.join(".minecraft");
    fs::create_dir_all(&dot_minecraft).map_err(|e| e.to_string())?;

    let mut jar_file = "server.jar".to_string();
    let source = Path::new(&source_path);
    let filename = source.file_name().unwrap_or_default().to_string_lossy();
    let is_installer = filename.contains("-installer");

    if is_file {
        if is_installer {
            // Copy installer to .minecraft
            let installer_dest = dot_minecraft.join(&*filename);
            fs::copy(source, &installer_dest).map_err(|e| e.to_string())?;

            // Background Installation
            let app_handle = app.clone();
            let instance_id = id.clone();
            let working_dir = dot_minecraft.clone();
            let installer_filename = filename.to_string();
            let instance_path_clone = instance_path.clone();
            let instance_path_json = instance_path.join("instance.json");
            tauri::async_runtime::spawn(async move {
                let _ = app_handle.emit(
                    "install-progress",
                    InstanceInstallProgress {
                        id: instance_id.clone(),
                        step: "Ejecutando instalador Forge...".into(),
                        progress: 20,
                        total_size: None,
                        downloaded: 0,
                    },
                );

                let mut cmd = std::process::Command::new("java");
                cmd.arg("-jar")
                    .arg(&installer_filename)
                    .arg("--installServer")
                    .current_dir(&working_dir)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());

                #[cfg(windows)]
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

                let child = cmd.spawn();

                match child {
                    Ok(mut child) => {
                        let stdout = child.stdout.take().unwrap();
                        let reader = BufReader::new(stdout);
                        let mut current_progress = 20;
                        let install_log_path = instance_path_clone.join("install.log");
                        let mut install_log = fs::File::create(&install_log_path).ok();
                        let mut last_emit = std::time::Instant::now();
                        let mut line_count = 0;

                        // Stream output to the UI
                        for line in reader.lines() {
                            if let Ok(l) = line {
                                line_count += 1;
                                let trimmed = l.trim();
                                if !trimmed.is_empty() {
                                    // Write to permanent log file
                                    if let Some(ref mut file) = install_log {
                                        let _ = writeln!(file, "{}", trimmed);
                                    }

                                    // Pseudo-progress: increment slightly up to 85%
                                    // Slow down increments to avoid reaching 85 too fast
                                    if line_count % 10 == 0 && current_progress < 85 {
                                        current_progress += 1;
                                    }

                                    // Throttle UI emissions: Every 200ms or for critical lines
                                    let is_critical = trimmed.contains("Patching")
                                        || trimmed.contains("Downloading")
                                        || trimmed.contains("Extracting")
                                        || trimmed.contains("Success")
                                        || trimmed.contains("Error")
                                        || trimmed.contains("Fail");

                                    if is_critical || last_emit.elapsed().as_millis() > 200 {
                                        let _ = app_handle.emit(
                                            "install-progress",
                                            InstanceInstallProgress {
                                                id: instance_id.clone(),
                                                step: trimmed.to_string(),
                                                progress: current_progress,
                                                total_size: None,
                                                downloaded: 0,
                                            },
                                        );
                                        last_emit = std::time::Instant::now();
                                    }
                                }
                            }
                        }

                        // Wait for completion
                        match child.wait() {
                            Ok(status) if status.success() => {
                                let _ = app_handle.emit(
                                    "install-progress",
                                    InstanceInstallProgress {
                                        id: instance_id.clone(),
                                        step: "Instalación completada, configurando...".into(),
                                        progress: 90,
                                        total_size: None,
                                        downloaded: 0,
                                    },
                                );

                                // Try to find the server jar or set up the bootstrapper
                                let mut final_jar = "server.jar".to_string();
                                if let Ok(entries) = fs::read_dir(&working_dir) {
                                    for entry in entries.flatten() {
                                        let name = entry.file_name().to_string_lossy().to_string();
                                        if (name.contains("forge") || name.contains("neoforge"))
                                            && name.ends_with(".jar")
                                            && !name.contains("installer")
                                        {
                                            final_jar = name;
                                            break;
                                        }
                                    }
                                }

                                // Update instance.json with detected jar
                                if let Ok(content) = fs::read_to_string(&instance_path_json) {
                                    if let Ok(mut inst) = serde_json::from_str::<Instance>(&content)
                                    {
                                        inst.settings.jar_file = final_jar;
                                        inst.loader =
                                            if installer_filename.to_lowercase().contains("neo") {
                                                InstanceEngine::NeoForge
                                            } else {
                                                InstanceEngine::Forge
                                            };
                                        if let Ok(new_json) = serde_json::to_string_pretty(&inst) {
                                            let _ = fs::write(&instance_path_json, new_json);
                                        }
                                    }
                                }

                                // Cleanup installer and log
                                let _ = fs::remove_file(working_dir.join(&installer_filename));
                                let _ = fs::remove_file(&install_log_path);

                                let _ = app_handle.emit(
                                    "install-progress",
                                    InstanceInstallProgress {
                                        id: instance_id,
                                        step: "Done".into(),
                                        progress: 100,
                                        total_size: None,
                                        downloaded: 0,
                                    },
                                );
                            }
                            Ok(status) => {
                                let _ = app_handle.emit(
                                    "install-progress",
                                    InstanceInstallProgress {
                                        id: instance_id,
                                        step: format!(
                                            "El instalador falló con código: {:?}",
                                            status.code()
                                        ),
                                        progress: 0,
                                        total_size: None,
                                        downloaded: 0,
                                    },
                                );
                            }
                            Err(e) => {
                                let _ = app_handle.emit(
                                    "install-progress",
                                    InstanceInstallProgress {
                                        id: instance_id,
                                        step: format!("Error al esperar al instalador: {}", e),
                                        progress: 0,
                                        total_size: None,
                                        downloaded: 0,
                                    },
                                );
                            }
                        }
                    }
                    Err(e) => {
                        let _ = app_handle.emit(
                            "install-progress",
                            InstanceInstallProgress {
                                id: instance_id,
                                step: format!("Error al iniciar el instalador: {}", e),
                                progress: 0,
                                total_size: None,
                                downloaded: 0,
                            },
                        );
                    }
                }
            });
        } else {
            // Normal JAR
            let dest = dot_minecraft.join("server.jar");
            fs::copy(source, dest).map_err(|e| e.to_string())?;
        }
    } else {
        // Copy Directory contents
        let options = fs_extra::dir::CopyOptions::new().content_only(true);
        fs_extra::dir::copy(source, &dot_minecraft, &options).map_err(|e| e.to_string())?;

        // Try to find if there is a jar already
        if let Ok(entries) = fs::read_dir(&dot_minecraft) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "jar" {
                        jar_file = entry.file_name().to_string_lossy().to_string();
                        break;
                    }
                }
            }
        }
    }

    let instance = Instance {
        id: id.clone(),
        name,
        icon,
        loader: InstanceEngine::Vanilla, // Initially vanilla, updated in BG if Forge
        version: version
            .replace("Forge/NeoForge Installer (", "")
            .replace(")", ""),
        path: slug,
        date_created: Utc::now(),
        last_played: None,
        state: InstanceState::Stopped,
        settings: InstanceSettings {
            jar_file,
            ..InstanceSettings::default()
        },
        build: None,
    };

    let json_path = instance_path.join("instance.json");
    let json_content = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    fs::write(json_path, json_content).map_err(|e| e.to_string())?;

    // handle EULA for local imports
    if accept_eula {
        use crate::commands::versions::write_eula_txt;
        write_eula_txt(instance_path.join(".minecraft").join("eula.txt"), true)?;
    }

    Ok(id)
}

#[tauri::command]
pub async fn open_instance_addons_folder(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    let dot_minecraft = entry.path().join(".minecraft");
                    let mods_path = dot_minecraft.join("mods");
                    let plugins_path = dot_minecraft.join("plugins");

                    let target = if mods_path.exists() {
                        mods_path
                    } else if plugins_path.exists() {
                        plugins_path
                    } else {
                        // Create plugins folder by default if none exist
                        fs::create_dir_all(&plugins_path).map_err(|e| e.to_string())?;
                        plugins_path
                    };

                    #[cfg(target_os = "windows")]
                    {
                        std::process::Command::new("explorer")
                            .arg(target)
                            .spawn()
                            .map_err(|e| e.to_string())?;
                    }
                    #[cfg(target_os = "linux")]
                    {
                        std::process::Command::new("xdg-open")
                            .arg(target)
                            .spawn()
                            .map_err(|e| e.to_string())?;
                    }
                    #[cfg(target_os = "macos")]
                    {
                        std::process::Command::new("open")
                            .arg(target)
                            .spawn()
                            .map_err(|e| e.to_string())?;
                    }
                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

#[tauri::command]
pub async fn analyze_instance_addons(
    app: tauri::AppHandle,
    id: String,
    source_paths: Vec<String>,
) -> Result<Vec<AddonAnalysis>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    // 1. Find Instance
    let mut instance_folder = PathBuf::new();
    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    instance_folder = entry.path();
                    break;
                }
            }
        }
    }

    if instance_folder.as_os_str().is_empty() {
        return Err("Instance not found".to_string());
    }

    let dot_minecraft = instance_folder.join(".minecraft");
    let mods_path = dot_minecraft.join("mods");
    let plugins_path = dot_minecraft.join("plugins");

    let target_dir = if mods_path.exists() {
        mods_path
    } else {
        plugins_path
    };

    // 2. Get current addons list for comparison (USING CACHE)
    let cache_path = instance_folder.join("addons_cache.json");
    let existing_addons = get_addons_internal(&target_dir, &cache_path, false)?;

    // 3. Analyze each source path (IN PARALLEL)
    let mut batch_seen: Vec<Addon> = Vec::new();
    
    // First, extract metadata for all sources in parallel
    let source_metas: Vec<(String, Option<Addon>)> = source_paths.par_iter()
        .map(|p: &String| (p.clone(), extract_addon_metadata(&PathBuf::from(p))))
        .collect();

    let mut results = Vec::new();
    for (path_str, metadata) in source_metas {
        let source_path = PathBuf::from(&path_str);
        if metadata.is_none() {
            results.push(AddonAnalysis {
                source_path: path_str.clone(),
                name: source_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                version: "N/A".into(),
                status: "invalid".into(),
                existing_filename: None,
                old_version: None,
                size: 0,
                last_modified: 0,
                platform: "Unknown".into(),
            });
            continue;
        }

        let meta = metadata.unwrap();
        let mut status = "valid".to_string();
        let mut existing_filename = None;
        let mut old_version = None;

        // A. Check for duplicates/updates within current selection (intra-batch)
        let mut found_in_batch = false;
        for seen in &batch_seen {
            if seen.name == meta.name {
                existing_filename = Some(seen.file_name.clone());
                old_version = Some(seen.version.clone());
                status = if seen.version == meta.version && seen.size == meta.size { "duplicate_selection".into() } else { "update_selection".into() };
                found_in_batch = true;
                break;
            }
        }

        if !found_in_batch {
            // B. Check for duplicates/updates against existing files on disk
            for existing in &existing_addons {
                if existing.name == meta.name {
                    existing_filename = Some(existing.file_name.clone());
                    old_version = Some(existing.version.clone());

                    if existing.version == meta.version
                        && existing.size == meta.size
                        && existing.last_modified == meta.last_modified
                    {
                        status = "duplicate".into();
                    } else {
                        status = "update".into();
                    }
                    break;
                }
            }
        }

        batch_seen.push(meta.clone());

        results.push(AddonAnalysis {
            source_path: path_str,
            name: meta.name,
            version: meta.version,
            status,
            existing_filename,
            old_version,
            size: meta.size,
            last_modified: meta.last_modified,
            platform: meta.platform,
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn install_instance_addons(
    app: tauri::AppHandle,
    id: String,
    items: Vec<AddonInstallItem>,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    // 1. Find Instance
    let mut instance_folder = PathBuf::new();
    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    instance_folder = entry.path();
                    break;
                }
            }
        }
    }

    if instance_folder.as_os_str().is_empty() {
        return Err("Instance not found".to_string());
    }

    let dot_minecraft = instance_folder.join(".minecraft");
    let mods_path = dot_minecraft.join("mods");
    let plugins_path = dot_minecraft.join("plugins");

    let target_dir = if mods_path.exists() {
        mods_path
    } else {
        if !plugins_path.exists() {
            fs::create_dir_all(&plugins_path).map_err(|e| e.to_string())?;
        }
        plugins_path
    };

    // 2. Process items
    for item in items {
        if item.action == "skip" {
            continue;
        }

        let source = Path::new(&item.source_path);
        if !source.exists() {
            continue;
        }

        // Handle replacement
        if let Some(existing) = item.existing_filename {
            let old_file = target_dir.join(existing);
            if old_file.exists() {
                fs::remove_file(old_file).map_err(|e| e.to_string())?;
            }
        }

        let file_name = source.file_name().ok_or("Invalid filename")?;
        let dest_path = target_dir.join(file_name);

        fs::copy(source, dest_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    enabled: bool,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    let dot_minecraft = entry.path().join(".minecraft");
                    let mods_path = dot_minecraft.join("mods");
                    let plugins_path = dot_minecraft.join("plugins");

                    let target_dir = if mods_path.exists() {
                        mods_path
                    } else {
                        plugins_path
                    };

                    let source_path = target_dir.join(&file_name);
                    if !source_path.exists() {
                        return Err("File not found".to_string());
                    }

                    let mut new_name = file_name.clone();
                    if enabled {
                        // Remove any "off" suffix and ensure it ends with .jar
                        for suffix in &[".disabled", ".bkp", ".bak", ".old", ".off"] {
                            if new_name.ends_with(suffix) {
                                new_name = new_name.replace(suffix, "");
                            }
                        }
                        if !new_name.ends_with(".jar") {
                            new_name.push_str(".jar");
                        }
                    } else {
                        // Add .disabled suffix
                        if !new_name.ends_with(".disabled") {
                            new_name.push_str(".disabled");
                        }
                    }

                    if new_name != file_name {
                        fs::rename(source_path, target_dir.join(new_name)).map_err(|e| e.to_string())?;
                    }
                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}

#[tauri::command]
pub async fn delete_instance_addon(
    app: tauri::AppHandle,
    id: String,
    file_name: String,
    delete_folder: bool,
) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");

    for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&content) {
                if inst.id == id {
                    let dot_minecraft = entry.path().join(".minecraft");
                    let mods_path = dot_minecraft.join("mods");
                    let plugins_path = dot_minecraft.join("plugins");

                    let target_dir = if mods_path.exists() {
                        mods_path
                    } else {
                        plugins_path
                    };

                    let file_path = target_dir.join(&file_name);
                    if file_path.exists() {
                        // 1. Detect possible folder before deleting the file (to have metadata if needed)
                        if delete_folder {
                            let addon_meta = extract_addon_metadata(&file_path);
                            let folder_names = vec![
                                file_name.replace(".jar", "").replace(".disabled", "").replace(".bkp", "").replace(".bak", "").replace(".old", "").replace(".off", ""),
                                addon_meta.map(|m| m.name).unwrap_or_default(),
                            ];

                            for f_name in folder_names {
                                if f_name.is_empty() { continue; }
                                let possible_dir = target_dir.join(&f_name);
                                if possible_dir.exists() && possible_dir.is_dir() {
                                    let _ = fs::remove_dir_all(possible_dir);
                                    break;
                                }
                            }
                        }

                        // 2. Delete the file
                        fs::remove_file(file_path).map_err(|e| e.to_string())?;
                    }
                    return Ok(());
                }
            }
        }
    }
    Err("Instance not found".to_string())
}
