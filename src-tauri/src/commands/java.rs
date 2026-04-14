use crate::models::InstanceInstallProgress;
use futures_util::StreamExt;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};

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

    let versions = vec![8, 11, 16, 17, 21, 25];
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
