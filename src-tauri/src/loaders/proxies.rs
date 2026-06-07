//! Proxy loaders (Velocity, Waterfall, BungeeCord).
//!
//! - Velocity + Waterfall: same PaperMC API as the Bukkit family. The
//!   `ProjectVersionList` / build listing is identical; only the project
//!   name (`velocity` / `waterfall`) differs.
//! - BungeeCord: uses md-5's Jenkins JSON API at `ci.md-5.net`. There is no
//!   "MC version" parameter — BungeeCord releases are versioned by Jenkins
//!   build number. We list the last 30 successful builds and let the user
//!   pick one. The JAR is published as
//!   `bootstrap/target/BungeeCord.jar` under each build number.
//!
//! All three proxies override `stop_command` to return `"end"`, matching
//! the previous `commands/server.rs` hardcoded match.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Emitter;

use crate::loaders::common::{download_file, write_eula_txt};
use crate::loaders::types::{DownloadInfo, VersionMeta};
use crate::loaders::{LoaderCapabilities, LoaderCategory, LoaderStrategy};
use crate::models::{InstanceEngine, InstanceInstallProgress};

// ── Shared capabilities ────────────────────────────────────────────────────

fn proxy_capabilities() -> LoaderCapabilities {
    LoaderCapabilities {
        supports_plugins: true,
        supports_mods: false,
        is_proxy: true,
        custom_url_supported: true,
    }
}

fn not_implemented_yet(name: &str) -> String {
    format!("{} version fetching is not yet implemented", name)
}

// ── PaperMC API response structs (re-used by Velocity + Waterfall) ────────

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

/// Resolve a build download from PaperMC for a Velocity/Waterfall project.
async fn resolve_papermc_build(project: &str, mc_version: &str) -> Result<DownloadInfo, String> {
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

// ── BungeeCord Jenkins API ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JenkinsBuilds {
    builds: Vec<JenkinsBuildRef>,
}

#[derive(Debug, Deserialize)]
struct JenkinsBuildRef {
    number: u32,
    #[allow(dead_code)]
    url: Option<String>,
}

/// Fetch the last 30 BungeeCord build numbers from the SpigotMC Jenkins.
///
/// Uses `?tree=builds[number]` to minimise the payload, plus a `User-Agent`
/// header that some Jenkins instances require to return JSON instead of an
/// HTML error page. The response text is captured before deserialisation so
/// that any parse errors include the actual server reply.
async fn fetch_bungeecord_recent_builds() -> Result<Vec<u32>, String> {
    // The official BungeeCord CI is hosted on hub.spigotmc.org.
    // Using the `tree` parameter requests only the `builds[number]` field,
    // keeping the payload small and avoiding large HTML-embedded descriptions.
    let url = "https://hub.spigotmc.org/jenkins/job/BungeeCord/api/json?tree=builds[number]";

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 AnvilCraftPanel/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Jenkins request failed: {}", e))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read Jenkins response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "Jenkins returned HTTP {}: {}",
            status,
            &body[..body.len().min(200)]
        ));
    }

    let data: JenkinsBuilds = serde_json::from_str(&body).map_err(|e| {
        format!(
            "Failed to parse Jenkins JSON ({}). Response was: {}",
            e,
            &body[..body.len().min(300)]
        )
    })?;

    // Limit to 30 most recent builds. The JAR download will fail loudly
    // (404) if a specific build is broken, so no extra filtering needed.
    let mut numbers: Vec<u32> = data
        .builds
        .into_iter()
        .map(|b| b.number)
        .take(30)
        .collect();
    numbers.sort_unstable_by(|a, b| b.cmp(a)); // newest first
    Ok(numbers)
}

fn bungeecord_jar_url(build_number: u32) -> String {
    format!(
        "https://hub.spigotmc.org/jenkins/job/BungeeCord/{}/artifact/bootstrap/target/BungeeCord.jar",
        build_number
    )
}

// ── Shared install body for all three proxies ─────────────────────────────

async fn install_proxy(
    app: &tauri::AppHandle,
    id: &str,
    target_dir: &Path,
    project: &str,
    mc_version: &str,
    custom_url: Option<&str>,
    accept_eula: bool,
) -> Result<(), String> {
    let mc_dir = target_dir.join(".minecraft");
    std::fs::create_dir_all(&mc_dir).map_err(|e| e.to_string())?;

    let download = match custom_url {
        Some(url) => {
            let filename = url.split('/').last().unwrap_or("server.jar").to_string();
            DownloadInfo {
                url: url.to_string(),
                filename,
                size: None,
            }
        }
        None => resolve_papermc_build(project, mc_version).await?,
    };

    let jar_path = mc_dir.join(&download.filename);
    if let Some(p) = jar_path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }

    let log_file = mc_dir.parent().unwrap_or(&mc_dir).join("install.log");

    download_file(
        app,
        &download.url,
        &jar_path,
        id,
        download.size,
        Some(&log_file),
    )
    .await?;

    // Rename to server.jar (matches the default `jar_file` setting)
    let server_jar = mc_dir.join("server.jar");
    if server_jar.exists() {
        std::fs::remove_file(&server_jar).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&jar_path, &server_jar).map_err(|e| e.to_string())?;

    write_eula_txt(mc_dir.join("eula.txt"), accept_eula)?;

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

async fn install_bungeecord(
    app: &tauri::AppHandle,
    id: &str,
    target_dir: &Path,
    build_number: u32,
    custom_url: Option<&str>,
    accept_eula: bool,
) -> Result<(), String> {
    let mc_dir = target_dir.join(".minecraft");
    std::fs::create_dir_all(&mc_dir).map_err(|e| e.to_string())?;

    let download = match custom_url {
        Some(url) => {
            let filename = url.split('/').last().unwrap_or("BungeeCord.jar").to_string();
            DownloadInfo {
                url: url.to_string(),
                filename,
                size: None,
            }
        }
        None => DownloadInfo {
            url: bungeecord_jar_url(build_number),
            filename: "BungeeCord.jar".to_string(),
            size: None,
        },
    };

    let jar_path = mc_dir.join(&download.filename);
    if let Some(p) = jar_path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }

    let log_file = mc_dir.parent().unwrap_or(&mc_dir).join("install.log");

    download_file(
        app,
        &download.url,
        &jar_path,
        id,
        download.size,
        Some(&log_file),
    )
    .await?;

    // Rename to server.jar — the launcher always looks for this filename
    // (settings.jar_file default). Same convention used by Velocity/Waterfall.
    let server_jar = mc_dir.join("server.jar");
    if server_jar.exists() {
        std::fs::remove_file(&server_jar).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&jar_path, &server_jar).map_err(|e| e.to_string())?;

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

    let _ = accept_eula; // silence unused warning (BungeeCord has no EULA)
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
//  Velocity
// ═══════════════════════════════════════════════════════════════════════════

pub struct VelocityLoader;

#[async_trait]
impl LoaderStrategy for VelocityLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Velocity
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Proxies
    }
    fn display_name(&self) -> &'static str {
        "Velocity"
    }
    fn min_java(&self) -> u8 {
        17
    }
    fn capabilities(&self) -> LoaderCapabilities {
        proxy_capabilities()
    }
    fn stop_command(&self) -> &'static str {
        "end"
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err(not_implemented_yet("Velocity"))
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
        install_proxy(
            app,
            id,
            target_dir,
            "velocity",
            &version.id,
            custom_url,
            accept_eula,
        )
        .await
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Waterfall
// ═══════════════════════════════════════════════════════════════════════════

pub struct WaterfallLoader;

#[async_trait]
impl LoaderStrategy for WaterfallLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Waterfall
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Proxies
    }
    fn display_name(&self) -> &'static str {
        "Waterfall"
    }
    fn min_java(&self) -> u8 {
        11
    }
    fn capabilities(&self) -> LoaderCapabilities {
        proxy_capabilities()
    }
    fn stop_command(&self) -> &'static str {
        "end"
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err(not_implemented_yet("Waterfall"))
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
        install_proxy(
            app,
            id,
            target_dir,
            "waterfall",
            &version.id,
            custom_url,
            accept_eula,
        )
        .await
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  BungeeCord
// ═══════════════════════════════════════════════════════════════════════════

pub struct BungeeCordLoader;

impl BungeeCordLoader {
    /// Fetch the most recent BungeeCord build numbers from Jenkins.
    /// Used by the UI to populate the version dropdown.
    pub async fn list_recent_builds() -> Result<Vec<u32>, String> {
        fetch_bungeecord_recent_builds().await
    }
}

#[async_trait]
impl LoaderStrategy for BungeeCordLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::BungeeCord
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Proxies
    }
    fn display_name(&self) -> &'static str {
        "BungeeCord"
    }
    fn min_java(&self) -> u8 {
        8
    }
    fn capabilities(&self) -> LoaderCapabilities {
        proxy_capabilities()
    }
    fn stop_command(&self) -> &'static str {
        "end"
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        // BungeeCord has no MC version dimension — `mc_version` is
        // intentionally ignored. Build numbers come from Jenkins.
        let builds = Self::list_recent_builds().await?;
        Ok(builds
            .into_iter()
            .map(|n| VersionMeta {
                id: n.to_string(),
                build: Some(n.to_string()),
                url: Some(bungeecord_jar_url(n)),
                display_name: format!("build #{}", n),
                requires_mc_version: None,
            })
            .collect())
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
        let build_number: u32 = version
            .id
            .parse()
            .map_err(|_| format!("BungeeCord version id must be a build number, got '{}'", version.id))?;
        install_bungeecord(app, id, target_dir, build_number, custom_url, accept_eula).await
    }
}
