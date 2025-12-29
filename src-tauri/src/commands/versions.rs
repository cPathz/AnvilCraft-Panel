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

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if let Some(size) = total_size {
            let progress = (downloaded as f64 / size as f64 * 100.0) as u64;
            // Emit sparingly
            if downloaded % (1024 * 1024) == 0 || downloaded == size {
                // Log every MB
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

            let _ = app.emit(
                "install-progress",
                InstanceInstallProgress {
                    id: id.to_string(),
                    step: format!("Downloading..."),
                    progress,
                    total_size: Some(size),
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
