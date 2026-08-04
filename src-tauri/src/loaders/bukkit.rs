//! Bukkit-family loaders (Paper, Spigot, Purpur, Folia).
//!
//! All four share the same install body (download a JAR for a given MC
//! version, rename to `server.jar`, write EULA). The differences are the
//! API endpoint and the per-loader metadata (min Java).
//!
//! - Paper, Folia: `api.papermc.io`
//! - Purpur: `api.purpurmc.org`
//! - Spigot: no public JSON API; falls through to PaperMC (which 404s) and
//!   surfaces the error. Users who need Spigot can supply a `custom_url`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Emitter;

use crate::loaders::common::{download_file, write_eula_txt};
use crate::loaders::types::{DownloadInfo, VersionMeta};
use crate::loaders::{LoaderCapabilities, LoaderCategory, LoaderStrategy};
use crate::models::{InstanceEngine, InstanceInstallProgress};

// ── API response structs (PaperMC / Purpur) ───────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct ProjectBuilds {
    #[allow(dead_code)]
    pub project_id: String,
    #[allow(dead_code)]
    pub project_name: String,
    #[allow(dead_code)]
    pub version: String,
    pub builds: Vec<BuildInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BuildInfo {
    pub build: u32,
    #[allow(dead_code)]
    pub time: String,
    #[allow(dead_code)]
    pub channel: String,
    #[allow(dead_code)]
    pub promoted: bool,
    #[allow(dead_code)]
    pub changes: Vec<ChangeInfo>,
    pub downloads: BuildDownloads,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChangeInfo {
    #[allow(dead_code)]
    pub commit: String,
    #[allow(dead_code)]
    pub summary: String,
    #[allow(dead_code)]
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct BuildDownloads {
    pub application: ApplicationDownload,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApplicationDownload {
    pub name: String,
    #[allow(dead_code)]
    pub sha256: String,
}

// ── Shared helpers ────────────────────────────────────────────────────────

fn bukkit_capabilities() -> LoaderCapabilities {
    LoaderCapabilities {
        supports_plugins: true,
        supports_mods: false,
        is_proxy: false,
        custom_url_supported: true,
    }
}

fn not_implemented_yet(name: &str) -> String {
    format!("{} version fetching is not yet implemented", name)
}

/// Resolve a concrete download for a PaperMC / Purpur project + MC version.
/// Returns the URL + filename of the latest build.
async fn resolve_build_download(
    project: &str,
    mc_version: &str,
) -> Result<DownloadInfo, String> {
    if project == "purpur" {
        let url = format!("https://api.purpurmc.org/v2/purpur/{}/latest", mc_version);
        let json: serde_json::Value = reqwest::get(&url)
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        let build = json["build"].as_str().ok_or("No build found for Purpur")?;
        let dl_url = format!(
            "https://api.purpurmc.org/v2/purpur/{}/{}/download",
            mc_version, build
        );
        Ok(DownloadInfo {
            url: dl_url,
            filename: format!("purpur-{}-{}.jar", mc_version, build),
            size: None,
        })
    } else {
        let url = format!(
            "https://api.papermc.io/v2/projects/{}/versions/{}/builds",
            project, mc_version
        );
        let data: ProjectBuilds = reqwest::get(&url)
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        let latest = data.builds.last().ok_or("No builds found")?;
        let dl_name = &latest.downloads.application.name;
        let dl_url = format!(
            "https://api.papermc.io/v2/projects/{}/versions/{}/builds/{}/downloads/{}",
            project, mc_version, latest.build, dl_name
        );
        Ok(DownloadInfo {
            url: dl_url,
            filename: dl_name.clone(),
            size: None,
        })
    }
}

/// Shared install body. `install_dir` is the `.minecraft` folder inside the
/// instance root — the JAR lands at `<install_dir>/server.jar`.
async fn install_inner(
    app: &tauri::AppHandle,
    id: &str,
    install_dir: &Path,
    project: &str,
    mc_version: &str,
    custom_url: Option<&str>,
    accept_eula: bool,
) -> Result<(), String> {
    let download = match custom_url {
        Some(url) => {
            let filename = url.split('/').last().unwrap_or("server.jar").to_string();
            DownloadInfo {
                url: url.to_string(),
                filename,
                size: None,
            }
        }
        None => resolve_build_download(project, mc_version).await?,
    };

    let jar_path = install_dir.join(&download.filename);
    if let Some(p) = jar_path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }

    let log_file = install_dir
        .parent()
        .unwrap_or(install_dir)
        .join("install.log");

    download_file(
        app,
        &download.url,
        &jar_path,
        id,
        download.size,
        Some(&log_file),
    )
    .await?;

    // Rename to server.jar for consistency (the default `jar_file` setting).
    let server_jar = install_dir.join("server.jar");
    if server_jar.exists() {
        std::fs::remove_file(&server_jar).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&jar_path, &server_jar).map_err(|e| e.to_string())?;

    write_eula_txt(install_dir.join("eula.txt"), accept_eula)?;

    // `install_dir` is `<target_dir>/.minecraft`; its parent is the
    // per-instance folder that `finalize_install` expects.
    crate::loaders::common::finalize_install(app, id, install_dir.parent().unwrap_or(install_dir));
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

// ── Paper ─────────────────────────────────────────────────────────────────

pub struct PaperLoader;

#[async_trait]
impl LoaderStrategy for PaperLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Paper
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Bukkit
    }
    fn display_name(&self) -> &'static str {
        "Paper"
    }
    fn min_java(&self) -> u8 {
        11
    }
    fn capabilities(&self) -> LoaderCapabilities {
        bukkit_capabilities()
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err(not_implemented_yet("Paper"))
    }

    async fn install(
        &self,
        app: &tauri::AppHandle,
        id: &str,
        target_dir: &Path,
        version: &VersionMeta,
        custom_url: Option<&str>,
        accept_eula: bool,
    ) -> Result<(), String> {
        let mc_dir = target_dir.join(".minecraft");
        install_inner(app, id, &mc_dir, "paper", &version.id, custom_url, accept_eula).await
    }
}

// ── Spigot ────────────────────────────────────────────────────────────────

pub struct SpigotLoader;

#[async_trait]
impl LoaderStrategy for SpigotLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Spigot
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Bukkit
    }
    fn display_name(&self) -> &'static str {
        "Spigot"
    }
    fn min_java(&self) -> u8 {
        8
    }
    fn capabilities(&self) -> LoaderCapabilities {
        bukkit_capabilities()
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err("Spigot does not provide a public build JSON API".to_string())
    }

    async fn install(
        &self,
        app: &tauri::AppHandle,
        id: &str,
        target_dir: &Path,
        version: &VersionMeta,
        custom_url: Option<&str>,
        accept_eula: bool,
    ) -> Result<(), String> {
        if custom_url.is_none() {
            return Err(
                "Spigot has no public build API. Provide a custom download URL.".to_string(),
            );
        }
        let mc_dir = target_dir.join(".minecraft");
        install_inner(app, id, &mc_dir, "spigot", &version.id, custom_url, accept_eula).await
    }
}

// ── Purpur ────────────────────────────────────────────────────────────────

pub struct PurpurLoader;

#[async_trait]
impl LoaderStrategy for PurpurLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Purpur
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Bukkit
    }
    fn display_name(&self) -> &'static str {
        "Purpur"
    }
    fn min_java(&self) -> u8 {
        11
    }
    fn capabilities(&self) -> LoaderCapabilities {
        bukkit_capabilities()
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err(not_implemented_yet("Purpur"))
    }

    async fn install(
        &self,
        app: &tauri::AppHandle,
        id: &str,
        target_dir: &Path,
        version: &VersionMeta,
        custom_url: Option<&str>,
        accept_eula: bool,
    ) -> Result<(), String> {
        let mc_dir = target_dir.join(".minecraft");
        install_inner(app, id, &mc_dir, "purpur", &version.id, custom_url, accept_eula).await
    }
}

// ── Folia ─────────────────────────────────────────────────────────────────

pub struct FoliaLoader;

#[async_trait]
impl LoaderStrategy for FoliaLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Folia
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Bukkit
    }
    fn display_name(&self) -> &'static str {
        "Folia"
    }
    fn min_java(&self) -> u8 {
        17
    }
    fn capabilities(&self) -> LoaderCapabilities {
        bukkit_capabilities()
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err(not_implemented_yet("Folia"))
    }

    async fn install(
        &self,
        app: &tauri::AppHandle,
        id: &str,
        target_dir: &Path,
        version: &VersionMeta,
        custom_url: Option<&str>,
        accept_eula: bool,
    ) -> Result<(), String> {
        let mc_dir = target_dir.join(".minecraft");
        install_inner(app, id, &mc_dir, "folia", &version.id, custom_url, accept_eula).await
    }
}
