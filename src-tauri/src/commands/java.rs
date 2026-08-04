use crate::models::InstanceInstallProgress;
use futures_util::StreamExt;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};

const ZULU_TARGET_VERSIONS: &[u8] = &[12, 13, 14];

#[derive(Deserialize, Debug, Clone, serde::Serialize)]
pub struct JavaRuntimeInfo {
    pub version: u8,
    pub is_downloaded: bool,
    pub path: Option<String>,
}

#[tauri::command]
pub async fn get_available_java_versions(app: AppHandle) -> Result<Vec<JavaRuntimeInfo>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let runtimes_dir = app_data.join("runtimes").join("java");

    let versions = vec![8, 11, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26];
    let mut info = Vec::new();

    for v in versions {
        let v_dir = runtimes_dir.join(v.to_string());
        let mut is_downloaded = false;
        let mut java_path = None;

        if v_dir.exists() {
            // Find java.exe recursively in bin folders
            if let Some(path) = find_java_executable(&v_dir) {
                is_downloaded = true;
                java_path = Some(path.to_string_lossy().to_string());
            }
        }

        info.push(JavaRuntimeInfo {
            version: v,
            is_downloaded,
            path: java_path,
        });
    }

    Ok(info)
}

fn find_java_executable(dir: &Path) -> Option<PathBuf> {
    // Check common locations like bin/java.exe or just java.exe
    let exe_name = if cfg!(windows) { "java.exe" } else { "java" };

    // 1. Direct check in dir/bin
    let bin_path = dir.join("bin").join(exe_name);
    if bin_path.exists() {
        return Some(bin_path);
    }

    // 2. Recursive check (Adobeium ZIPs often have a nested folder like jdk8u412-b08-jre/)
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(found) = find_java_executable(&entry.path()) {
                    return Some(found);
                }
            }
        }
    }

    None
}

#[tauri::command]
pub async fn download_java_runtime(app: AppHandle, version: u8) -> Result<String, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let runtimes_dir = app_data.join("runtimes").join("java");
    let version_dir = runtimes_dir.join(version.to_string());

    if !runtimes_dir.exists() {
        fs::create_dir_all(&runtimes_dir).map_err(|e| e.to_string())?;
    }

    // 1. Get download URL from Adoptium API
    // https://api.adoptium.net/v3/binary/latest/{feature_version}/{release_type}/{os}/{arch}/{image_type}/{jvm_impl}/{heap_size}/{vendor}
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "aarch64"
    };

    let url = if version == 16 {
        format!(
            "https://api.adoptium.net/v3/binary/version/jdk-16.0.2+7/{}/{}/jdk/hotspot/normal/eclipse",
            os, arch
        )
    } else {
        format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse",
            version, os, arch
        )
    };

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Adoptium API returned error: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    let temp_zip = runtimes_dir.join(format!("java_{}.zip", version));
    let mut file = fs::File::create(&temp_zip).map_err(|e| e.to_string())?;

    // Emite progress
    let id = format!("java-download-{}", version);

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut file, &chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let progress = if total_size > 0 {
            (downloaded * 100 / total_size) as u64
        } else {
            0
        };

        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: format!("Downloading Java {}...", version),
                progress,
                total_size: Some(total_size),
                downloaded,
            },
        );
    }

    // 2. Extract
    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: format!("Extracting Java {}...", version),
            progress: 100,
            total_size: Some(total_size),
            downloaded: total_size,
        },
    );

    if version_dir.exists() {
        fs::remove_dir_all(&version_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;

    let zip_file = fs::File::open(&temp_zip).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;

    archive.extract(&version_dir).map_err(|e| e.to_string())?;

    // Cleanup
    let _ = fs::remove_file(&temp_zip);

    // Find the executable to return it
    if let Some(path) = find_java_executable(&version_dir) {
        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: "Done".into(),
                progress: 100,
                total_size: Some(total_size),
                downloaded: total_size,
            },
        );
        Ok(path.to_string_lossy().to_string())
    } else {
        Err("Java executable not found after extraction".to_string())
    }
}

#[derive(Deserialize, Debug, Clone, serde::Serialize)]
pub struct ZuluRuntimeInfo {
    pub version: u8,
    pub is_downloaded: bool,
    pub path: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ZuluPackage {
    download_url: String,
    name: String,
}

#[tauri::command]
pub async fn get_available_zulu_versions(app: AppHandle) -> Result<Vec<ZuluRuntimeInfo>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let runtimes_dir = app_data.join("runtimes").join("java");

    let mut info = Vec::new();

    for &v in ZULU_TARGET_VERSIONS {
        let v_dir = runtimes_dir.join(format!("zulu-{}", v));
        let mut is_downloaded = false;
        let mut java_path = None;

        if v_dir.exists() {
            if let Some(path) = find_java_executable(&v_dir) {
                is_downloaded = true;
                java_path = Some(path.to_string_lossy().to_string());
            }
        }

        info.push(ZuluRuntimeInfo {
            version: v,
            is_downloaded,
            path: java_path,
        });
    }

    Ok(info)
}

#[tauri::command]
pub async fn download_zulu_runtime(app: AppHandle, version: u8) -> Result<String, String> {
    if !ZULU_TARGET_VERSIONS.contains(&version) {
        return Err(format!("Zulu version {} is not available", version));
    }

    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let runtimes_dir = app_data.join("runtimes").join("java");
    let version_dir = runtimes_dir.join(format!("zulu-{}", version));

    if !runtimes_dir.exists() {
        fs::create_dir_all(&runtimes_dir).map_err(|e| e.to_string())?;
    }

    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "aarch64"
    };

    let api_url = format!(
        "https://api.azul.com/metadata/v1/zulu/packages?java_version={}&os={}&arch={}&archive_type=zip&java_package_type=jre&release_status=ga&availability_types=available",
        version, os, arch
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("Zulu API request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Zulu API returned error: {}", response.status()));
    }

    let packages: Vec<ZuluPackage> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Zulu response: {}", e))?;

    // Prefer the entry whose name does NOT include "-fx-" (JavaFX bundling adds weight we don't need)
    let pkg = packages
        .iter()
        .find(|p| !p.name.contains("-fx-"))
        .or_else(|| packages.first())
        .ok_or_else(|| format!("No Zulu package found for version {}", version))?;

    let id = format!("zulu-download-{}", version);

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: format!("Downloading Zulu Java {}...", version),
            progress: 0,
            total_size: None,
            downloaded: 0,
        },
    );

    let download_response = client
        .get(&pkg.download_url)
        .send()
        .await
        .map_err(|e| format!("Zulu download failed: {}", e))?;

    if !download_response.status().is_success() {
        return Err(format!(
            "Zulu download returned error: {}",
            download_response.status()
        ));
    }

    let total_size = download_response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = download_response.bytes_stream();

    let temp_zip = runtimes_dir.join(format!("zulu_{}.zip", version));
    let mut file = fs::File::create(&temp_zip).map_err(|e| e.to_string())?;

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Stream error: {}", e))?;
        std::io::Write::write_all(&mut file, &chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let progress = if total_size > 0 {
            (downloaded * 100 / total_size) as u64
        } else {
            0
        };

        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: format!("Downloading Zulu Java {}...", version),
                progress,
                total_size: Some(total_size),
                downloaded,
            },
        );
    }

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: format!("Extracting Zulu Java {}...", version),
            progress: 100,
            total_size: Some(total_size),
            downloaded: total_size,
        },
    );

    if version_dir.exists() {
        fs::remove_dir_all(&version_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;

    let zip_file = fs::File::open(&temp_zip).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;
    archive.extract(&version_dir).map_err(|e| e.to_string())?;

    let _ = fs::remove_file(&temp_zip);

    if let Some(path) = find_java_executable(&version_dir) {
        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: "Done".into(),
                progress: 100,
                total_size: Some(total_size),
                downloaded: total_size,
            },
        );
        Ok(path.to_string_lossy().to_string())
    } else {
        Err("Zulu Java executable not found after extraction".to_string())
    }
}

/// Find a Java executable to use for running installer JARs (NeoForge/Forge).
/// Searches in this order:
///   1. App's Java runtimes directory: {app_data_dir}/runtimes/java/{ver}/bin/java.exe
///   2. Recursive search inside app's runtimes (for nested folder structures
///      like Adoptium's jdk8u412-b08-jre/bin/java.exe)
///   3. System PATH (java.exe in any PATH directory)
///
/// Returns None if no Java executable can be found anywhere.
pub fn find_any_java_executable(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    // 1+2. App's Java installations
    if let Ok(app_data) = app.path().app_data_dir() {
        let java_dir = app_data.join("runtimes").join("java");
        if let Some(found) = find_java_executable(&java_dir) {
            return Some(found);
        }
    }

    // 3. System PATH
    find_java_in_path()
}

fn find_java_in_path() -> Option<std::path::PathBuf> {
    let exe = if cfg!(windows) { "java.exe" } else { "java" };
    let Ok(path_var) = std::env::var("PATH") else {
        return None;
    };
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(exe);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
