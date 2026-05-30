/// AnvilCraft – CurseForge / Modrinth Modpack Importer
///
/// Supported formats:
///   • CurseForge export ZIP  → contains `manifest.json`
///   • Modrinth export ZIP    → contains `modrinth.index.json`
///
/// Both flows:
///   1. Extract the ZIP to a temp directory.
///   2. Parse the manifest to discover the mod loader + Minecraft version.
///   3. Create the instance folder structure under `instances/<slug>/.minecraft/`.
///   4. Download every listed mod concurrently using the respective API.
///   5. Copy the `overrides/` (CurseForge) or `overrides/` (Modrinth) folder.
///   6. Install the mod loader (NeoForge or Forge) headlessly if needed.
use crate::commands::versions::{install_neoforge, write_eula_txt};
use crate::models::{Instance, InstanceEngine, InstanceInstallProgress, InstanceSettings, InstanceState};
use chrono::Utc;
use futures_util::future::join_all;
use serde::Deserialize;
use slug::slugify;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;
use zip::ZipArchive;

// ── CurseForge API ────────────────────────────────────────────────────────────
// Public CurseForge API key (used for server-side tooling, widely distributed).
const CF_API_KEY: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";
const CF_API_BASE: &str = "https://api.curseforge.com/v1";

// ── Manifest structs ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CfManifest {
    minecraft: CfMinecraft,
    name: String,
    version: Option<String>,
    files: Vec<CfFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMinecraft {
    version: String,
    mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Deserialize)]
struct CfModLoader {
    id: String, // e.g. "neoforge-21.1.172" or "forge-47.2.0"
    primary: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CfFile {
    project_id: u64,
    file_id: u64,
    required: bool,
}

// Modrinth format
#[derive(Debug, Deserialize)]
struct MrIndex {
    name: String,
    #[serde(rename = "versionId")]
    version_id: Option<String>,
    dependencies: MrDependencies,
    files: Vec<MrFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MrDependencies {
    minecraft: String,
    #[serde(default)]
    neoforge: Option<String>,
    #[serde(default)]
    forge: Option<String>,
    #[serde(default)]
    fabric_loader: Option<String>,
    #[serde(default)]
    quilt_loader: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MrFile {
    path: String,
    downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    file_size: Option<u64>,
}

// ── Helper: copy directory recursively ───────────────────────────────────────

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// ── Helper: extract ZIP to temp dir ──────────────────────────────────────────

fn extract_zip_to_dir(zip_path: &Path, out_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let relative = entry
            .enclosed_name()
            .ok_or("Invalid zip entry path")?
            .to_path_buf();
        let out_path = out_dir.join(&relative);

        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
            fs::write(&out_path, data).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// ── Helper: create instance.json ─────────────────────────────────────────────

fn create_instance_json(
    instance_path: &Path,
    id: &str,
    name: &str,
    icon: &str,
    engine: InstanceEngine,
    mc_version: &str,
    slug: &str,
) -> Result<(), String> {
    let instance = Instance {
        id: id.to_string(),
        name: name.to_string(),
        icon: icon.to_string(),
        loader: engine,
        version: mc_version.to_string(),
        path: slug.to_string(),
        date_created: Utc::now(),
        last_played: None,
        state: InstanceState::Stopped,
        settings: InstanceSettings::default(),
        build: None,
    };
    let json = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    fs::write(instance_path.join("instance.json"), json).map_err(|e| e.to_string())?;
    Ok(())
}

// ── CurseForge: resolve download URL via API ─────────────────────────────────

async fn cf_get_download_url(client: &reqwest::Client, project_id: u64, file_id: u64) -> Result<String, String> {
    let url = format!("{}/mods/{}/files/{}/download-url", CF_API_BASE, project_id, file_id);
    let resp = client
        .get(&url)
        .header("x-api-key", CF_API_KEY)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        // Fallback: construct CDN URL manually
        // CurseForge CDN pattern: https://edge.forgecdn.net/files/<file_id_hi>/<file_id_lo>/<filename>
        // We can't know the filename here, so just return an error.
        return Err(format!("CF API returned {} for mod {}/{}", resp.status(), project_id, file_id));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let url_str = json["data"]
        .as_str()
        .ok_or_else(|| format!("No download URL for mod {}/{}", project_id, file_id))?
        .to_string();
    Ok(url_str)
}

// ── Tauri Command: import_curseforge_zip ─────────────────────────────────────

#[tauri::command]
pub async fn import_curseforge_zip(
    app: AppHandle,
    name: String,
    zip_path: String,
    icon: String,
    accept_eula: bool,
) -> Result<String, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data.join("instances");
    fs::create_dir_all(&instances_dir).map_err(|e| e.to_string())?;

    // ── Prepare instance directory ──────────────────────────────────────────
    let id = Uuid::new_v4().to_string();
    let mut slug = slugify(&name);
    let mut instance_path = instances_dir.join(&slug);
    let mut counter = 1u32;
    while instance_path.exists() {
        slug = format!("{}-{}", slugify(&name), counter);
        instance_path = instances_dir.join(&slug);
        counter += 1;
    }
    fs::create_dir_all(&instance_path).map_err(|e| e.to_string())?;

    // ── Extract ZIP to temp dir ─────────────────────────────────────────────
    let temp_dir = instance_path.join(".import_tmp");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: "Extracting modpack ZIP...".into(),
            progress: 2,
            total_size: None,
            downloaded: 0,
        },
    );

    extract_zip_to_dir(Path::new(&zip_path), &temp_dir)?;

    // ── Detect format: CurseForge vs Modrinth ──────────────────────────────
    let cf_manifest_path = temp_dir.join("manifest.json");
    let mr_index_path = temp_dir.join("modrinth.index.json");

    if cf_manifest_path.exists() {
        import_cf_manifest(app, id.clone(), name, icon, accept_eula, instance_path, temp_dir, slug, cf_manifest_path).await
    } else if mr_index_path.exists() {
        import_mr_index(app, id.clone(), name, icon, accept_eula, instance_path, temp_dir, slug, mr_index_path).await
    } else {
        let _ = fs::remove_dir_all(&temp_dir);
        Err("No recognized modpack manifest found (manifest.json or modrinth.index.json)".into())
    }
}

// ── CurseForge flow ───────────────────────────────────────────────────────────

async fn import_cf_manifest(
    app: AppHandle,
    id: String,
    name: String,
    icon: String,
    accept_eula: bool,
    instance_path: PathBuf,
    temp_dir: PathBuf,
    slug: String,
    manifest_path: PathBuf,
) -> Result<String, String> {
    // Parse manifest
    let raw = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: CfManifest = serde_json::from_str(&raw).map_err(|e| format!("Invalid manifest.json: {}", e))?;

    let mc_version = manifest.minecraft.version.clone();
    let primary_loader = manifest.minecraft.mod_loaders.iter().find(|l| l.primary).or_else(|| manifest.minecraft.mod_loaders.first());

    // Determine engine
    let (engine, loader_version) = if let Some(loader) = primary_loader {
        let id_lower = loader.id.to_lowercase();
        if id_lower.starts_with("neoforge-") {
            let ver = loader.id.trim_start_matches("neoforge-").trim_start_matches("NeoForge-").to_string();
            (InstanceEngine::NeoForge, Some(ver))
        } else if id_lower.starts_with("forge-") {
            let ver = loader.id.trim_start_matches("forge-").trim_start_matches("Forge-").to_string();
            (InstanceEngine::Forge, Some(ver))
        } else if id_lower.starts_with("fabric-") {
            (InstanceEngine::Fabric, None)
        } else {
            (InstanceEngine::Vanilla, None)
        }
    } else {
        (InstanceEngine::Vanilla, None)
    };

    // Create instance.json early so the UI can show the instance
    create_instance_json(&instance_path, &id, &name, &icon, engine.clone(), &mc_version, &slug)?;

    // Create .minecraft/mods dir
    let mc_dir = instance_path.join(".minecraft");
    fs::create_dir_all(mc_dir.join("mods")).map_err(|e| e.to_string())?;

    // ── Download mods concurrently ──────────────────────────────────────────
    let total_files = manifest.files.len();
    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: format!("Downloading {} mods via CurseForge API...", total_files),
            progress: 5,
            total_size: None,
            downloaded: 0,
        },
    );

    let client = Arc::new(
        reqwest::Client::builder()
            .user_agent("AnvilCraft/1.0")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| e.to_string())?
    );

    let mc_dir_arc = Arc::new(mc_dir.clone());
    let app_arc = Arc::new(app.clone());
    let id_arc = Arc::new(id.clone());

    // Process mods in batches of 10 to avoid overwhelming the API
    let files: Vec<CfFile> = manifest.files.into_iter().filter(|f| f.required).collect();
    let total = files.len();
    let mut downloaded_count = 0usize;

    for batch in files.chunks(10) {
        let tasks: Vec<_> = batch.iter().map(|file| {
            let client = Arc::clone(&client);
            let mc_dir = Arc::clone(&mc_dir_arc);
            let file = file.clone();

            async move {
                let url = cf_get_download_url(&client, file.project_id, file.file_id).await?;
                let filename = url.split('/').last().unwrap_or("mod.jar").to_string();
                let dest = mc_dir.join("mods").join(&filename);

                let req = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?
                    .error_for_status()
                    .map_err(|e| e.to_string())?;

                let bytes = req.bytes().await.map_err(|e| e.to_string())?;
                fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
                Result::<String, String>::Ok(filename)
            }
        }).collect();

        let results = join_all(tasks).await;
        for res in results {
            downloaded_count += 1;
            let progress = 5 + (downloaded_count * 70 / total.max(1)) as u64;
            match res {
                Ok(filename) => {
                    let _ = app_arc.emit(
                        "install-progress",
                        InstanceInstallProgress {
                            id: id_arc.to_string(),
                            step: format!("Downloaded mod: {}", filename),
                            progress,
                            total_size: None,
                            downloaded: downloaded_count as u64,
                        },
                    );
                }
                Err(e) => {
                    // Non-fatal: log and continue
                    eprintln!("[AnvilCraft] Mod download failed: {}", e);
                }
            }
        }
    }

    // ── Copy overrides ──────────────────────────────────────────────────────
    let overrides_dir = temp_dir.join("overrides");
    if overrides_dir.exists() {
        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: "Copying overrides...".into(),
                progress: 80,
                total_size: None,
                downloaded: 0,
            },
        );
        copy_dir_recursive(&overrides_dir, &mc_dir)?;
    }

    // ── EULA ────────────────────────────────────────────────────────────────
    write_eula_txt(mc_dir.join("eula.txt"), accept_eula)?;

    // ── Install mod loader ──────────────────────────────────────────────────
    if let (InstanceEngine::NeoForge, Some(nf_ver)) = (&engine, &loader_version) {
        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: format!("Installing NeoForge {}...", nf_ver),
                progress: 85,
                total_size: None,
                downloaded: 0,
            },
        );
        install_neoforge(&app, &id, nf_ver, &mc_dir, accept_eula).await?;
    }
    // Note: Forge installation could be added here in a future update

    // ── Cleanup temp dir ────────────────────────────────────────────────────
    let _ = fs::remove_dir_all(&temp_dir);

    // ── Done ────────────────────────────────────────────────────────────────
    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: "Done".into(),
            progress: 100,
            total_size: None,
            downloaded: 0,
        },
    );

    Ok(id)
}

// ── Modrinth flow ─────────────────────────────────────────────────────────────

async fn import_mr_index(
    app: AppHandle,
    id: String,
    name: String,
    icon: String,
    accept_eula: bool,
    instance_path: PathBuf,
    temp_dir: PathBuf,
    slug: String,
    index_path: PathBuf,
) -> Result<String, String> {
    let raw = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let index: MrIndex = serde_json::from_str(&raw).map_err(|e| format!("Invalid modrinth.index.json: {}", e))?;

    let mc_version = index.dependencies.minecraft.clone();

    // Determine engine
    let (engine, loader_version) = if let Some(nf_ver) = &index.dependencies.neoforge {
        (InstanceEngine::NeoForge, Some(nf_ver.clone()))
    } else if let Some(_forge_ver) = &index.dependencies.forge {
        (InstanceEngine::Forge, None)
    } else if index.dependencies.fabric_loader.is_some() {
        (InstanceEngine::Fabric, None)
    } else if index.dependencies.quilt_loader.is_some() {
        (InstanceEngine::Quilt, None)
    } else {
        (InstanceEngine::Vanilla, None)
    };

    // Create instance.json
    create_instance_json(&instance_path, &id, &name, &icon, engine.clone(), &mc_version, &slug)?;

    let mc_dir = instance_path.join(".minecraft");
    fs::create_dir_all(mc_dir.join("mods")).map_err(|e| e.to_string())?;

    let total_files = index.files.len();
    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: format!("Downloading {} files via Modrinth CDN...", total_files),
            progress: 5,
            total_size: None,
            downloaded: 0,
        },
    );

    let client = Arc::new(
        reqwest::Client::builder()
            .user_agent("AnvilCraft/1.0")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| e.to_string())?
    );

    let total = index.files.len();
    let mut downloaded_count = 0usize;

    // Modrinth files can go to different paths (e.g. "mods/mod.jar" or "config/something.cfg")
    for batch in index.files.chunks(8) {
        let tasks: Vec<_> = batch.iter().map(|file| {
            let client = Arc::clone(&client);
            let mc_dir = mc_dir.clone();
            let url = file.downloads.first().cloned().unwrap_or_default();
            let rel_path = file.path.clone();

            async move {
                if url.is_empty() {
                    return Err(format!("No download URL for {}", rel_path));
                }
                let dest = mc_dir.join(&rel_path);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                let req = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?
                    .error_for_status()
                    .map_err(|e| e.to_string())?;

                let bytes = req.bytes().await.map_err(|e| e.to_string())?;
                fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
                Ok::<String, String>(rel_path)
            }
        }).collect();

        let results = join_all(tasks).await;
        for res in results {
            downloaded_count += 1;
            let progress = 5 + (downloaded_count * 70 / total.max(1)) as u64;
            match res {
                Ok(p) => {
                    let _ = app.emit(
                        "install-progress",
                        InstanceInstallProgress {
                            id: id.clone(),
                            step: format!("Downloaded: {}", p),
                            progress,
                            total_size: None,
                            downloaded: downloaded_count as u64,
                        },
                    );
                }
                Err(e) => {
                    eprintln!("[AnvilCraft] Modrinth file download failed: {}", e);
                }
            }
        }
    }

    // Copy overrides
    let overrides_dir = temp_dir.join("overrides");
    if overrides_dir.exists() {
        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: "Copying overrides...".into(),
                progress: 80,
                total_size: None,
                downloaded: 0,
            },
        );
        copy_dir_recursive(&overrides_dir, &mc_dir)?;
    }

    // EULA
    write_eula_txt(mc_dir.join("eula.txt"), accept_eula)?;

    // Install loader
    if let (InstanceEngine::NeoForge, Some(nf_ver)) = (&engine, &loader_version) {
        let _ = app.emit(
            "install-progress",
            InstanceInstallProgress {
                id: id.clone(),
                step: format!("Installing NeoForge {}...", nf_ver),
                progress: 85,
                total_size: None,
                downloaded: 0,
            },
        );
        install_neoforge(&app, &id, nf_ver, &mc_dir, accept_eula).await?;
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.clone(),
            step: "Done".into(),
            progress: 100,
            total_size: None,
            downloaded: 0,
        },
    );

    Ok(id)
}
