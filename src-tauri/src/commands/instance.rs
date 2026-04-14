use crate::commands::versions::{install_project_server, install_vanilla};
use crate::models::{
    ChildProcessMap, Instance, InstanceEngine, InstanceInstallProgress, InstanceSettings,
    InstanceState,
};
use chrono::Utc;
use slug::slugify;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
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
pub async fn open_instances_folder(app: tauri::AppHandle, slug: String) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instance_root = app_data.join("instances").join(slug);
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
