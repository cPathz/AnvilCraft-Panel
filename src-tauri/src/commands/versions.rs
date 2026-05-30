use crate::models::InstanceInstallProgress;
use chrono::Local;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use tauri::Emitter;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LatestVersions {
    release: String,
    snapshot: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

// Paper/Project Structs
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectVersionList {
    pub project_id: String,
    pub project_name: String,
    pub version_groups: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectBuilds {
    pub project_id: String,
    pub project_name: String,
    pub version: String,
    pub builds: Vec<BuildInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildInfo {
    pub build: u32,
    pub time: String,
    pub channel: String,
    pub promoted: bool,
    pub changes: Vec<ChangeInfo>,
    pub downloads: BuildDownloads,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChangeInfo {
    pub commit: String,
    pub summary: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildDownloads {
    pub application: DownloadInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadInfo {
    pub name: String,
    pub sha256: String,
}

// --- Commands ---

#[tauri::command]
pub async fn get_minecraft_versions(snapshots: bool) -> Result<Vec<String>, String> {
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
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

#[tauri::command]
pub async fn get_project_versions(project: String) -> Result<Vec<String>, String> {
    // Projects: paper, folia, velocity, waterfall, purpur
    // Purpur API is slightly different (https://api.purpurmc.org/v2/purpur)
    // Paper API: https://api.papermc.io/v2/projects/{project}

    let url = if project == "purpur" {
        "https://api.purpurmc.org/v2/purpur".to_string()
    } else {
        format!("https://api.papermc.io/v2/projects/{}", project)
    };

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if project == "purpur" {
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        // Purpur: { "project": "purpur", "versions": ["1.16.5", ...] }
        let versions = json["versions"]
            .as_array()
            .ok_or("Invalid Purpur API")?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect();
        Ok(versions)
    } else {
        let data = resp
            .json::<ProjectVersionList>()
            .await
            .map_err(|e| e.to_string())?;
        // Reverse to show newest first
        let mut v = data.versions;
        v.reverse();
        Ok(v)
    }
}

pub async fn download_file(
    app: &tauri::AppHandle,
    url: &str,
    path: &std::path::Path,
    id: &str,
    known_size: Option<u64>,
    log_path: Option<&std::path::Path>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("AnvilCraft/1.0")
        .timeout(std::time::Duration::from_secs(300)) // Increased timeout
        .build()
        .map_err(|e| e.to_string())?;

    println!("[DEBUG] Downloading: {}", url);
    let mut log_file = log_path.and_then(|p| {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
    });

    if let Some(ref mut f) = log_file {
        let _ = writeln!(
            f,
            "[{}] Starting download: {}",
            Local::now().format("%H:%M:%S"),
            url
        );
    }

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: format!("Connecting: {}", url),
            progress: 0,
            total_size: known_size,
            downloaded: 0,
        },
    );

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    let total_size = response.content_length().or(known_size).filter(|&s| s > 0);

    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    let mut last_emit = std::time::Instant::now();
    let mut last_progress = 0;
    let mut last_logged_mb = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let current_mb = downloaded / (1024 * 1024);
        let mut progress = 0;
        let mut is_finished = false;

        if let Some(size) = total_size {
            progress = (downloaded as f64 / size as f64 * 100.0) as u64;
            is_finished = downloaded == size;
        }

        // Log every MB
        if current_mb > last_logged_mb || is_finished {
            last_logged_mb = current_mb;
            if let Some(ref mut f) = log_file {
                let _ = writeln!(
                    f,
                    "[{}] Progress: {}% ({} bytes)",
                    Local::now().format("%H:%M:%S"),
                    progress,
                    downloaded
                );
            }
        }

        // Throttle UI events to prevent freezing the WebView
        if progress > last_progress || last_emit.elapsed().as_millis() > 50 || is_finished {
            last_progress = progress;
            last_emit = std::time::Instant::now();

            let _ = app.emit(
                "install-progress",
                InstanceInstallProgress {
                    id: id.to_string(),
                    step: format!("Downloading..."),
                    progress,
                    total_size,
                    downloaded,
                },
            );
        }
    }

    if let Some(ref mut f) = log_file {
        let _ = writeln!(
            f,
            "[{}] Download finished.",
            Local::now().format("%H:%M:%S")
        );
    }

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: "Finalizing download...".into(),
            progress: 100,
            total_size: total_size,
            downloaded,
        },
    );

    Ok(())
}

pub fn write_eula_txt(path: std::path::PathBuf, accept: bool) -> Result<(), String> {
    if !accept {
        return Ok(());
    }
    let timestamp = Local::now().format("%a %b %d %H:%M:%S %Z %Y").to_string();
    let content = format!(
        "#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n#{}\neula=true",
        timestamp
    );

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    fs::write(path, content).map_err(|e| e.to_string())
}

// Helpers
pub async fn install_vanilla(
    app: &tauri::AppHandle,
    id: &str,
    version: &str,
    path: &std::path::Path,
    accept_eula: bool,
) -> Result<(), String> {
    // 1. Get Manifest
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

    // 2. Get Details
    let details: VersionDetails = reqwest::get(&version_info.url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let server_dl = details.downloads.server.ok_or("No server download")?;

    // 3. Download
    let jar_path = path.join(".minecraft").join("server.jar");
    // Ensure dir exists
    if let Some(parent) = jar_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let log_file = path.join("install.log");
    download_file(
        app,
        &server_dl.url,
        &jar_path,
        id,
        Some(server_dl.size),
        Some(&log_file),
    )
    .await?;

    // 4. EULA
    if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&log_file) {
        let _ = writeln!(
            f,
            "[{}] Setting up EULA...",
            Local::now().format("%H:%M:%S")
        );
    }
    write_eula_txt(path.join(".minecraft").join("eula.txt"), accept_eula)?;

    // 5. Done
    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: "Done".into(),
            progress: 100,
            total_size: None,
            downloaded: 0,
        },
    );

    Ok(())
}

pub async fn install_project_server(
    app: tauri::AppHandle,
    install_dir: std::path::PathBuf,
    version: String,
    id: String,
    project: String,
    custom_url: Option<String>,
    accept_eula: bool,
) -> Result<(), String> {
    // Determine Download URL
    // If Custom URL is present, Use it.
    // Else, fetch latest build.

    let (download_url, filename) = if let Some(url) = custom_url {
        let fname = url.split('/').last().unwrap_or("server.jar").to_string();
        (url, fname)
    } else {
        // Fetch Build
        // Logic for Paper/Purpur/etc builds
        // Simplified for brevity: Assuming we implement get_builds logic here or similar.
        // For now, let's implement the fetching logic directly here to be self contained.

        if project == "purpur" {
            // Purpur: https://api.purpurmc.org/v2/purpur/{version}/latest
            let url = format!("https://api.purpurmc.org/v2/purpur/{}/latest", version);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let build = json["build"].as_str().ok_or("No build found")?;

            // Download: https://api.purpurmc.org/v2/purpur/{version}/{build}/download
            let dl_url = format!(
                "https://api.purpurmc.org/v2/purpur/{}/{}/download",
                version, build
            );
            (dl_url, format!("purpur-{}-{}.jar", version, build))
        } else {
            // Paper API
            let url = format!(
                "https://api.papermc.io/v2/projects/{}/versions/{}/builds",
                project, version
            );
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let data: ProjectBuilds = resp.json().await.map_err(|e| e.to_string())?;
            let latest = data.builds.last().ok_or("No builds found")?;

            let dl_name = &latest.downloads.application.name;
            let dl_url = format!(
                "https://api.papermc.io/v2/projects/{}/versions/{}/builds/{}/downloads/{}",
                project, version, latest.build, dl_name
            );
            (dl_url, dl_name.clone())
        }
    };

    let jar_path = install_dir.join(&filename);
    if let Some(p) = jar_path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }

    let log_file = install_dir
        .parent()
        .unwrap_or(&install_dir)
        .join("install.log");

    download_file(&app, &download_url, &jar_path, &id, None, Some(&log_file)).await?;

    // Rename to server.jar for consistency?
    if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&log_file) {
        let _ = writeln!(
            f,
            "[{}] Finalizing server.jar...",
            Local::now().format("%H:%M:%S")
        );
    }
    // Or update instance settings to point to this jar?
    // In create_instance we default settings.jar_file to "server.jar".
    // If we use "paper-ver-build.jar", we must update the instance settings.
    // BUT create_instance happens BEFORE this async task completes fully or concurrently.
    // Let's rename to server.jar to be simple and safe for now.

    let server_jar = install_dir.join("server.jar");
    if server_jar.exists() {
        fs::remove_file(&server_jar).map_err(|e| e.to_string())?;
    }
    fs::rename(jar_path, server_jar).map_err(|e| e.to_string())?;

    // EULA
    write_eula_txt(install_dir.join("eula.txt"), accept_eula)?;

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id,
            step: "Done".into(),
            progress: 100,
            total_size: None,
            downloaded: 0,
        },
    );

    Ok(())
}

// ─── NeoForge ────────────────────────────────────────────────────────────────

/// Helper to extract the "Minecraft Version" category from a NeoForge version string.
fn extract_neoforge_category(nf_version: &str) -> Option<String> {
    if nf_version.starts_with("1.") {
        nf_version.split('-').next().map(|s| s.to_string())
    } else {
        let parts: Vec<&str> = nf_version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse::<u32>().unwrap_or(0);
            if major >= 20 && major <= 21 {
                // Map 20.x and 21.x to 1.20.x and 1.21.x
                Some(format!("1.{}.{}", parts[0], parts[1]))
            } else {
                // For 26.x.x onwards, it uses the direct prefix.
                // E.g., 26.1.2.65-beta -> category 26.1.2
                // We join all parts except the last one (which is the build/beta number).
                let cat = parts[0..parts.len() - 1].join(".");
                Some(cat)
            }
        } else {
            None
        }
    }
}

/// Returns NeoForge versions for a given Minecraft version (e.g. "1.20.1" or "26.1.2").
/// Queries the Maven Metadata XML from the official NeoForged Maven repository.
#[tauri::command]
pub async fn get_neoforge_versions(mc_version: String, betas: bool) -> Result<Vec<String>, String> {
    // NeoForge versions have the format: <mc_major>.<mc_minor>.<mc_patch>-<neoforge_build>
    // The metadata XML for a MC version like 1.20.1 lives at:
    // https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

    let client = reqwest::Client::builder()
        .user_agent("AnvilCraft/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let xml = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // Parse <version> tags from XML using simple string scanning (no heavy XML parser needed)
    let mut versions: Vec<String> = Vec::new();

    for line in xml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
            let inner = trimmed
                .trim_start_matches("<version>")
                .trim_end_matches("</version>");
            
            let ext_mc = extract_neoforge_category(inner);
            
            if let Some(ver) = ext_mc {
                if ver == mc_version {
                    // Filter out betas if the user didn't request them
                    let is_beta = inner.contains("-beta");
                    if betas || !is_beta {
                        versions.push(inner.to_string());
                    }
                }
            }
        }
    }

    // Reverse so newest builds come first
    versions.reverse();
    Ok(versions)
}

/// Lists all Minecraft versions that have at least one NeoForge release.
#[tauri::command]
pub async fn get_neoforge_mc_versions() -> Result<Vec<String>, String> {
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

    let client = reqwest::Client::builder()
        .user_agent("AnvilCraft/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let xml = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let mut mc_versions: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for line in xml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
            let inner = trimmed
                .trim_start_matches("<version>")
                .trim_end_matches("</version>");
            
            let ext_mc = extract_neoforge_category(inner);

            if let Some(ver) = ext_mc {
                // Ensure it looks like a valid category version (e.g., numbers separated by dots)
                if ver.split('.').all(|p| p.parse::<u32>().is_ok()) {
                    mc_versions.insert(ver);
                }
            }
        }
    }

    let mut result: Vec<String> = mc_versions.into_iter().collect();
    result.reverse(); // newest first
    Ok(result)
}

/// Downloads the NeoForge installer and runs it headlessly to set up the server.
/// Emits `install-progress` events to the frontend during the process.
pub async fn install_neoforge(
    app: &tauri::AppHandle,
    id: &str,
    neoforge_version: &str,
    minecraft_dir: &std::path::Path,
    accept_eula: bool,
) -> Result<(), String> {
    use std::io::Write as IoWrite;
    use tauri::Emitter;
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    // NeoForge installer URL pattern:
    // https://maven.neoforged.net/releases/net/neoforged/neoforge/<version>/neoforge-<version>-installer.jar
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        neoforge_version, neoforge_version
    );

    let installer_path = minecraft_dir.join(format!("neoforge-{}-installer.jar", neoforge_version));

    // Ensure dir exists
    fs::create_dir_all(minecraft_dir).map_err(|e| e.to_string())?;

    // Step 1 – Emit "downloading installer"
    let _ = app.emit(
        "install-progress",
        crate::models::InstanceInstallProgress {
            id: id.to_string(),
            step: format!("Downloading NeoForge {} installer...", neoforge_version),
            progress: 0,
            total_size: None,
            downloaded: 0,
        },
    );

    let log_file = minecraft_dir
        .parent()
        .unwrap_or(minecraft_dir)
        .join("install.log");

    // Download installer JAR
    download_file(app, &installer_url, &installer_path, id, None, Some(&log_file)).await?;

    // Step 2 – Run installer headlessly
    let _ = app.emit(
        "install-progress",
        crate::models::InstanceInstallProgress {
            id: id.to_string(),
            step: format!("Ejecutando instalador NeoForge {}...", neoforge_version),
            progress: 50,
            total_size: None,
            downloaded: 0,
        },
    );

    if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&log_file) {
        let _ = writeln!(
            f,
            "[{}] Running NeoForge installer headlessly...",
            chrono::Local::now().format("%H:%M:%S")
        );
    }

    let mut std_cmd = std::process::Command::new("java");
    std_cmd.arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .current_dir(minecraft_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    std_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let mut cmd = tokio::process::Command::from(std_cmd);
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to run NeoForge installer: {}", e))?;

    // Drain stderr in a background task to prevent pipe deadlock
    let stderr_log = log_file.clone();
    let stderr_handle = if let Some(stderr) = child.stderr.take() {
        Some(tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut reader = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&stderr_log) {
                    let _ = writeln!(f, "[STDERR] {}", line);
                }
            }
        }))
    } else {
        None
    };

    // Read stdout and emit progress to UI
    if let Some(stdout) = child.stdout.take() {
        use tokio::io::AsyncBufReadExt;
        let mut reader = tokio::io::BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&log_file) {
                let _ = writeln!(f, "{}", line);
            }
            
            let mut short_line = line.clone();
            if short_line.len() > 60 {
                short_line.truncate(60);
                short_line.push_str("...");
            }

            let _ = app.emit(
                "install-progress",
                crate::models::InstanceInstallProgress {
                    id: id.to_string(),
                    step: format!("NeoForge: {}", short_line),
                    progress: 50,
                    total_size: None,
                    downloaded: 0,
                },
            );
        }
    }

    // Wait for stderr reader to finish
    if let Some(handle) = stderr_handle {
        let _ = handle.await;
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for NeoForge installer: {}", e))?;

    if !status.success() {
        return Err("NeoForge installer failed".into());
    }

    // Step 3 – EULA
    write_eula_txt(minecraft_dir.join("eula.txt"), accept_eula)?;

    // Cleanup installer JAR to save space
    let _ = fs::remove_file(&installer_path);
    // Also remove the installer log that NeoForge generates in the same dir
    let _ = fs::remove_file(
        minecraft_dir.join(format!("neoforge-{}-installer.jar.log", neoforge_version)),
    );

    // Step 4 – Done
    let _ = app.emit(
        "install-progress",
        crate::models::InstanceInstallProgress {
            id: id.to_string(),
            step: "Instalación completada".into(),
            progress: 90,
            total_size: None,
            downloaded: 0,
        },
    );

    Ok(())
}
