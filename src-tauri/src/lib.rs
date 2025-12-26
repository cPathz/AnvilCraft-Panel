// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use slug::slugify;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::Manager;
use tauri::State;
use uuid::Uuid;

// Global map to store running server processes
struct ChildProcessMap(Arc<Mutex<HashMap<String, Child>>>);

// Helper for debug logging
// Helper for debug logging
fn log_debug(msg: &str) {
    println!("[DEBUG] {}", msg); // Write to stdout instead of file to avoid triggering file watcher restarts
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceInstallProgress {
    pub id: String,
    pub step: String,  // "Downloading", "Installing", "Done"
    pub progress: u64, // 0-100
    pub total_size: Option<u64>,
    pub downloaded: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceSettings {
    pub min_ram: u64, // MB
    pub max_ram: u64, // MB
    pub port: u16,
    pub args: String,
    pub jar_file: String,
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            min_ram: 2048,
            max_ram: 2048, // Default 2GB as requested
            port: 25565,
            args: String::new(),
            jar_file: String::from("server.jar"),
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

#[tauri::command]
async fn create_instance(
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
        "PaperMC" => InstanceEngine::Paper,
        "Spigot" => InstanceEngine::Spigot,
        "Purpur" => InstanceEngine::Purpur,
        "Folia" => InstanceEngine::Folia,
        "Velocity" => InstanceEngine::Velocity,
        "Waterfall" => InstanceEngine::Waterfall,
        _ => InstanceEngine::Vanilla,
    };

    // Extract build number if custom URL is provided
    let build = if let Some(url) = &custom_download_url {
        // Simple heuristic: try to get the number before .jar
        // e.g. https://.../paper-1.20.5-15.jar -> 15
        url.split('/').last().and_then(|filename| {
            if filename.ends_with(".jar") {
                let name = filename.trim_end_matches(".jar");
                // Split by '-' and take last part
                name.split('-').last().map(|s| s.to_string())
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

    // 6. Save instance.json
    let json_path = instance_path.join("instance.json");
    let json_content = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    fs::write(json_path, json_content).map_err(|e| e.to_string())?;

    // 7. Start Installation in Background
    let app_handle = app.clone();
    let instance_id = id.clone();
    let instance_version = version.clone();
    let instance_path_clone = instance_path.clone(); // Clone complete PathBuf
    let loader_engine = instance.loader.clone();
    let custom_download_url_clone = custom_download_url.clone();

    tauri::async_runtime::spawn(async move {
        // Emit initial "Creating files" event
        let _ = app_handle.emit(
            "install-progress",
            InstanceInstallProgress {
                id: instance_id.clone(),
                step: "Creating files...".into(),
                progress: 10, // Start at 10% to show activity
                total_size: None,
                downloaded: 0,
            },
        );
        // Small delay to let user see "Creating files"
        std::thread::sleep(std::time::Duration::from_millis(500));

        match loader_engine {
            InstanceEngine::Paper
            | InstanceEngine::Purpur
            | InstanceEngine::Spigot
            | InstanceEngine::Folia
            | InstanceEngine::Velocity
            | InstanceEngine::Waterfall => {
                // Determine project name based on engine
                let project = match loader_engine {
                    InstanceEngine::Purpur => "purpur",
                    InstanceEngine::Spigot => "spigot", // Note: Spigot usually not in Paper API, but if it is in my custom API it works
                    InstanceEngine::Folia => "folia",
                    InstanceEngine::Velocity => "velocity",
                    InstanceEngine::Waterfall => "waterfall",
                    _ => "paper",
                };

                if let Err(e) = install_project_server(
                    app_handle.clone(),
                    instance_path_clone.join(".minecraft"),
                    instance_version,
                    instance_id.clone(),
                    project.to_string(),
                    custom_download_url_clone,
                )
                .await
                {
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
            }
            _ => {
                if let Err(e) = install_vanilla(
                    &app_handle,
                    &instance_id,
                    &instance_version,
                    &instance_path_clone,
                )
                .await
                {
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
            }
        }
    });

    Ok(id)
}

async fn install_vanilla(
    app: &tauri::AppHandle,
    id: &str,
    version: &str,
    path: &std::path::Path,
) -> Result<(), String> {
    // 1. Get Version Details URL
    let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let manifest: VersionManifest = reqwest::get(manifest_url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let version_info = manifest
        .versions
        .into_iter()
        .find(|v| v.id == version)
        .ok_or("Version not found")?;

    // 2. Get Server JAR URL
    let details: VersionDetails = reqwest::get(&version_info.url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let server_download = details
        .downloads
        .server
        .ok_or("No server download available for this version")?;

    // 3. Download JAR
    let server_jar_path = path.join(".minecraft").join("server.jar"); // Inside .minecraft
    download_file(
        app,
        &server_download.url,
        &server_jar_path,
        id,
        Some(server_download.size),
    )
    .await?;

    // 4. Auto EULA
    auto_eula(&path.join(".minecraft"))?;

    // 5. Emit Done
    app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: "Done".to_string(),
            progress: 100,
            total_size: None,
            downloaded: 0,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn read_instances(
    app: tauri::AppHandle,
    state: State<'_, ChildProcessMap>,
) -> Result<Vec<Instance>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data_dir.join("instances");

    if !instances_dir.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();
    let entries = fs::read_dir(instances_dir).map_err(|e| e.to_string())?;

    let map = state.0.lock().map_err(|_| "Failed to lock state")?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            let json_path = path.join("instance.json");
            if json_path.exists() {
                let content = fs::read_to_string(json_path).map_err(|e| e.to_string())?;
                match serde_json::from_str::<Instance>(&content) {
                    Ok(mut instance) => {
                        // Sync state with actual process map
                        if map.contains_key(&instance.id) {
                            instance.state = InstanceState::Running;
                        } else {
                            // If it says Running/Starting but no process, it's Stopped (orphaned or app restart)
                            if instance.state == InstanceState::Running
                                || instance.state == InstanceState::Starting
                            {
                                instance.state = InstanceState::Stopped;
                                // Optional: Update JSON to reflect reality?
                                // Ideally yes, but let's just update in memory for UI for now.
                                // Actually better to fix the file so next read is clean.
                                // But simple read should be fast.
                            }
                        }
                        instances.push(instance);
                    }
                    Err(e) => println!("Error parsing instance at {:?}: {}", path, e),
                }
            }
        }
    }

    // Sort by last played (descending) or date created
    instances.sort_by(|a, b| b.date_created.cmp(&a.date_created));

    Ok(instances)
}

#[derive(Debug, Deserialize)]
struct LatestVersions {
    release: String,
    snapshot: String,
}

#[derive(Debug, Deserialize)]
struct VersionManifest {
    latest: LatestVersions,
    versions: Vec<VersionInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionInfo {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionDetails {
    downloads: VersionDownloads,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionDownloads {
    server: Option<VersionDownload>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionDownload {
    sha1: String,
    size: u64,
    url: String,
}

async fn download_file(
    app: &tauri::AppHandle,
    url: &str,
    path: &std::path::Path,
    id: &str,
    known_size: Option<u64>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("AnvilCraft/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    log_debug(&format!("Starting download: {}", url));
    log_debug(&format!("Known size passed: {:?}", known_size));

    // Emit connecting state
    app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: format!("Connecting: {}", url),
            progress: 0,
            total_size: known_size,
            downloaded: 0,
        },
    )
    .map_err(|e| e.to_string())?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    let total_size = response.content_length().or(known_size).filter(|&s| s > 0);
    log_debug(&format!(
        "Response content-length: {:?}",
        response.content_length()
    ));
    log_debug(&format!("Final total_size used: {:?}", total_size));

    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if let Some(size) = total_size {
            let progress = (downloaded as f64 / size as f64 * 100.0) as u64;
            app.emit(
                "install-progress",
                InstanceInstallProgress {
                    id: id.to_string(),
                    step: format!("Downloading: {}", url),
                    progress,
                    total_size: Some(size),
                    downloaded,
                },
            )
            .map_err(|e| e.to_string())?;
        } else {
            // Emulate progress based on approx 60MB (Paper/Vanilla jars are usually 30-60MB)
            // Cap at 99% so we don't show 100% prematurely
            let estimated_size = 60 * 1024 * 1024; // 60 MB
            let progress = ((downloaded as f64 / estimated_size as f64 * 100.0) as u64).min(99);

            app.emit(
                "install-progress",
                InstanceInstallProgress {
                    id: id.to_string(),
                    step: format!(
                        "Downloading... ({:.1} MB)",
                        downloaded as f64 / 1024.0 / 1024.0
                    ),
                    progress,
                    total_size: None,
                    downloaded,
                },
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Artificial delay for UI visibility
    std::thread::sleep(std::time::Duration::from_millis(800));

    // Force 100% progress event
    app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: format!("Downloading: {}", url),
            progress: 100,
            total_size: total_size,
            downloaded: downloaded,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn auto_eula(instance_path: &std::path::Path) -> Result<(), String> {
    let eula_path = instance_path.join("eula.txt");
    fs::write(eula_path, "eula=true").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn start_instance(
    app: tauri::AppHandle,
    state: State<'_, ChildProcessMap>,
    id: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    // Find instance path (re-reading json or using convention)
    // For now assuming we can find it by searching instances dir or passing path.
    // To be safe/fast let's read instances dir to find the ID.
    let instances_dir = app_data_dir.join("instances");
    let mut instance_path = PathBuf::new();
    let mut settings = InstanceSettings::default();

    if instances_dir.exists() {
        for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let json_path = entry.path().join("instance.json");
            if json_path.exists() {
                let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
                let instance: Instance =
                    serde_json::from_str(&content).map_err(|e| e.to_string())?;
                if instance.id == id {
                    instance_path = entry.path();
                    settings = instance.settings;
                    break;
                }
            }
        }
    }

    if !instance_path.exists() {
        return Err("Instance not found".to_string());
    }

    // Java Command construction
    // TODO: Verify Java installation or use bundled one. For now assuming 'java' in PATH.
    let server_jar = instance_path.join(".minecraft").join(&settings.jar_file);
    if !server_jar.exists() {
        return Err(format!("Server JAR '{}' not found", settings.jar_file));
    }

    let mut cmd = Command::new("java");
    cmd.current_dir(instance_path.join(".minecraft"));

    // Memory
    cmd.arg(format!("-Xms{}M", settings.min_ram));
    cmd.arg(format!("-Xmx{}M", settings.max_ram));

    // Encoding
    cmd.arg("-Dfile.encoding=UTF-8");

    // Custom JVM Args
    if !settings.args.is_empty() {
        for arg in settings.args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    // Jar
    cmd.arg("-jar");
    cmd.arg(&settings.jar_file);
    cmd.arg("nogui");

    // Server Port (if needed, usually in server.properties, but some server jars accept --port)
    // cmd.arg("--port");
    // cmd.arg(settings.port.to_string());
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;

    // Stream Stdout
    if let Some(stdout) = child.stdout.take() {
        let id_clone = id.clone();
        let app_clone = app.clone();
        let process_map = state.0.clone();
        let instances_dir_clone = instances_dir.clone();

        tauri::async_runtime::spawn(async move {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let _ = app_clone.emit("server-log", (id_clone.clone(), l));
                }
            }

            // Process has exited (stdout closed)

            // 1. Remove from map
            if let Ok(mut map) = process_map.lock() {
                map.remove(&id_clone);
            }

            // 2. Update state to Stopped
            // We use the synchronous update helper here.
            let _ = update_instance_state(&instances_dir_clone, &id_clone, InstanceState::Stopped);

            // 3. Notify Frontend
            let _ = app_clone.emit("instance-update", ()); // Generic update trigger
        });
    }

    // Stream Stderr (Prevent buffer deadlock & capture errors)
    if let Some(stderr) = child.stderr.take() {
        let id_clone = id.clone();
        let app_clone = app.clone();

        tauri::async_runtime::spawn(async move {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let _ = app_clone.emit("server-log", (id_clone.clone(), l));
                }
            }
        });
    }

    // Capture child in state
    state
        .0
        .lock()
        .map_err(|_| "Failed to lock state")?
        .insert(id.clone(), child);

    // Update State in JSON
    // Note: detailed state management usually needs to be more robust (checking if process died etc)
    // For now we just mark as Running.
    update_instance_state(&instances_dir, &id, InstanceState::Running)?;

    // Notify Frontend of state change
    app.emit("instance-update", ()).map_err(|e| e.to_string())?;

    Ok(())
}

fn update_instance_state(
    instances_dir: &std::path::Path,
    id: &str,
    new_state: InstanceState,
) -> Result<(), String> {
    for entry in fs::read_dir(instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let json_path = entry.path().join("instance.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            let mut instance: Instance =
                serde_json::from_str(&content).unwrap_or_else(|_| Instance {
                    id: "error".to_string(),
                    name: "".to_string(),
                    icon: "".to_string(),
                    loader: InstanceEngine::Vanilla,
                    version: "".to_string(),
                    path: "".to_string(),
                    date_created: Utc::now(),
                    last_played: None,
                    state: InstanceState::Error,
                    settings: InstanceSettings::default(),
                    build: None,
                });

            if instance.id == id {
                instance.state = new_state;
                let new_json =
                    serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
                fs::write(json_path, new_json).map_err(|e| e.to_string())?;
                break;
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn send_command(
    state: State<'_, ChildProcessMap>,
    id: String,
    command: String,
) -> Result<(), String> {
    let mut map = state.0.lock().map_err(|_| "Failed to lock state")?;
    if let Some(child) = map.get_mut(&id) {
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "{}", command).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn stop_instance(state: State<'_, ChildProcessMap>, id: String) -> Result<(), String> {
    let mut map = state.0.lock().map_err(|_| "Failed to lock state")?;
    if let Some(child) = map.get_mut(&id) {
        // Graceful stop
        if let Some(stdin) = child.stdin.as_mut() {
            // "stop" command is standard for Minecraft servers
            writeln!(stdin, "stop").map_err(|e| e.to_string())?;
        }
        // We do NOT remove from map here. We allow the process to exit naturally.
        // The log reader thread detects exit and removes it from map.
    }
    Ok(())
}

#[tauri::command]
async fn kill_instance(state: State<'_, ChildProcessMap>, id: String) -> Result<(), String> {
    let mut map = state.0.lock().map_err(|_| "Failed to lock state")?;
    if let Some(child) = map.get_mut(&id) {
        let _ = child.kill();
        map.remove(&id);
    }
    Ok(())
}

#[tauri::command]
async fn get_minecraft_versions(snapshots: bool) -> Result<Vec<String>, String> {
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let response = reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .json::<VersionManifest>()
        .await
        .map_err(|e| e.to_string())?;

    let versions: Vec<String> = response
        .versions
        .into_iter()
        .filter(|v| snapshots || v.version_type == "release")
        .map(|v| v.id)
        .collect();

    Ok(versions)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CustomMeta {
    project: String,
    generated_at: String,
    source: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CustomBuildInfo {
    number: u32,
    label: String,
    time: String,
    sha256: Option<String>,
    size: u64,
    url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CustomVersionInfo {
    #[serde(rename = "type")]
    version_type: String,
    channel: String,
    requires_java: Option<u32>,
    latest_build: CustomBuildInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CustomFamily {
    lifecycle: String,
    versions: HashMap<String, CustomVersionInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CustomApiResponse {
    meta: CustomMeta,
    families: HashMap<String, CustomFamily>,
}

#[tauri::command]
async fn get_project_versions(project: String) -> Result<Vec<String>, String> {
    let url = format!(
        "https://corpmore.com/anvilcraft/api/v1/{}.json",
        project.to_lowercase()
    );
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;

    let api_response = response
        .json::<CustomApiResponse>()
        .await
        .map_err(|e| format!("Failed to parse JSON for {}: {}", project, e))?;

    let mut versions = Vec::new();

    // Iterate over families and their versions to collect all available version IDs
    for (_, family) in api_response.families {
        for (version_id, _) in family.versions {
            versions.push(version_id);
        }
    }

    // Sort versions using a custom logic or semver if possible.
    // For now, simple reverse string sort works reasonably well for MC versions (1.20 > 1.10)
    // BUT string sort fails on 1.10 vs 1.2.
    // Let's implement a basic heuristic sort: Major.Minor.Patch
    versions.sort_by(|a, b| {
        // Helper to parse version string into a tuple of integers
        fn parse_version(v: &str) -> Vec<u32> {
            v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect()
        }

        let va = parse_version(a);
        let vb = parse_version(b);

        // Compare properly (b vs a for descending order)
        vb.cmp(&va)
    });

    Ok(versions)
}

#[tauri::command]
async fn open_instances_folder(app: tauri::AppHandle, slug: Option<String>) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut target_dir = app_data_dir.join("instances");

    if let Some(instance_slug) = &slug {
        target_dir = target_dir.join(instance_slug).join(".minecraft");
    }

    if !target_dir.exists() {
        // If specific path doesn't exist, try opening the instance root
        if let Some(instance_slug) = slug {
            target_dir = app_data_dir.join("instances").join(instance_slug);
            if !target_dir.exists() {
                // Fallback to instances root
                target_dir = app_data_dir.join("instances");
            }
        } else {
            fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&target_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&target_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&target_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn get_system_memory() -> u64 {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_memory();
    sys.total_memory()
}

#[tauri::command]
async fn save_instance_settings(
    app: tauri::AppHandle,
    instance_id: String,
    settings: InstanceSettings,
) -> Result<(), String> {
    // 1. Resolve AppData path
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    // We need to find the instance path. Since we don't have a DB, we iterate or assume structure.
    // Easier: Read all instances, find the one with ID, update, and write.

    // Wait, read_instances does scanning.
    // To be efficient, we might need a method to get instance path by ID.
    // But since `path` (slug) is not passed here, let's scan.
    // Let's implement a quick scan helper or just scan here.

    let instances_dir = app_data_dir.join("instances");
    if !instances_dir.exists() {
        return Err("No instances found".to_string());
    }

    let entries = fs::read_dir(&instances_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let json_path = path.join("instance.json");
            if json_path.exists() {
                let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
                if let Ok(mut instance) = serde_json::from_str::<Instance>(&content) {
                    if instance.id == instance_id {
                        // Found it! Update settings
                        instance.settings = settings;
                        let new_content =
                            serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
                        fs::write(json_path, new_content).map_err(|e| e.to_string())?;
                        return Ok(());
                    }
                }
            }
        }
    }

    Err("Instance not found".to_string())
}

#[tauri::command]
async fn delete_instance(
    app: tauri::AppHandle,
    state: State<'_, ChildProcessMap>,
    id: String,
) -> Result<(), String> {
    // 1. Check if running
    let is_running = {
        let map = state.0.lock().map_err(|_| "Failed to lock state")?;
        map.contains_key(&id)
    };

    if is_running {
        return Err("Cannot delete a running instance. Stop it first.".to_string());
    }

    // 2. Resolve Path
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data_dir.join("instances");
    let mut instance_path = PathBuf::new();

    if instances_dir.exists() {
        for entry in fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let json_path = entry.path().join("instance.json");
            if json_path.exists() {
                let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
                if let Ok(instance) = serde_json::from_str::<Instance>(&content) {
                    if instance.id == id {
                        instance_path = entry.path();
                        break;
                    }
                }
            }
        }
    }

    if !instance_path.exists() {
        return Err("Instance not found".to_string());
    }

    // 3. Delete Directory
    fs::remove_dir_all(instance_path).map_err(|e| e.to_string())?;

    Ok(())
}

async fn install_project_server(
    app: tauri::AppHandle,
    instance_dir: PathBuf,
    version: String,
    id: String,
    project: String,
    custom_download_url: Option<String>,
) -> Result<(), String> {
    let step_msg = if custom_download_url.is_some() {
        "Resolving link..."
    } else {
        "Fetching builds..."
    };

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: step_msg.into(),
            progress: 0,
            total_size: None,
            downloaded: 0,
        },
    );

    let (download_url, known_size) = if let Some(url) = custom_download_url {
        (url, None)
    } else {
        // 1. Fetch Project Metadata from Custom API
        let url = format!("https://corpmore.com/anvilcraft/api/v1/{}.json", project);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<CustomApiResponse>()
            .await
            .map_err(|e| format!("Failed to parse JSON for {}: {}", project, e))?;

        // 2. Find version info
        let mut target_version_info = None;
        for (_, family) in response.families {
            if let Some(v_info) = family.versions.get(&version) {
                target_version_info = Some(v_info.clone());
                break;
            }
        }

        let version_info = target_version_info
            .ok_or_else(|| format!("Version {} not found for project {}", version, project))?;

        // 3. Get URL and Size
        (
            version_info.latest_build.url.clone(),
            Some(version_info.latest_build.size),
        )
    };

    // 4. Download JAR
    let server_jar_path = instance_dir.join("server.jar");
    download_file(&app, &download_url, &server_jar_path, &id, known_size).await?;

    // 5. Auto EULA
    auto_eula(&instance_dir)?;

    // 6. Emit Done
    app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: "Done".to_string(),
            progress: 100,
            total_size: None,
            downloaded: 0,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ChildProcessMap(Arc::new(Mutex::new(HashMap::new()))))
        .invoke_handler(tauri::generate_handler![
            create_instance,
            read_instances,
            start_instance,
            stop_instance,
            kill_instance,
            send_command,
            get_minecraft_versions,
            get_project_versions,
            open_instances_folder,
            get_system_memory,
            save_instance_settings,
            delete_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
