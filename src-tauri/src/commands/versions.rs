//! Tauri commands that list available Minecraft / project versions for the
//! Add Instance modal. The actual install logic lives in `loaders::*` after
//! the loader-module refactor. Commands here are thin wrappers that fetch
//! version lists and hand them to the frontend.
//!
//! - `get_minecraft_versions` — Mojang manifest (Vanilla).
//! - `get_project_versions` — PaperMC API + Purpur API. Used by Velocity and
//!   Waterfall (Proxies family) for the version dropdown in the UI.
//! - `get_neoforge_versions` / `get_neoforge_mc_versions` — thin wrappers
//!   over `NeoForgeLoader::list_builds` / `list_mc_versions` from
//!   `loaders::mods`.

use crate::loaders::mods::NeoForgeLoader;
use serde::{Deserialize, Serialize};

// ── Vanilla: Mojang manifest ──────────────────────────────────────────────

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
    #[allow(dead_code)]
    version_type: String,
    url: String,
}

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

// ── PaperMC + Purpur (used by get_project_versions) ───────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectVersionList {
    pub project_id: String,
    pub project_name: String,
    pub version_groups: Vec<String>,
    pub versions: Vec<String>,
}

#[tauri::command]
pub async fn get_project_versions(project: String) -> Result<Vec<String>, String> {
    if project == "bungeecord" {
        let builds = crate::loaders::proxies::BungeeCordLoader::list_recent_builds().await?;
        return Ok(builds.into_iter().map(|n| n.to_string()).collect());
    }

    // Used by the frontend to list MC versions per project.
    // After Step 2, Paper/Spigot/Purpur/Folia don't need this anymore
    // (their fetch_versions lives in `loaders::bukkit`), but Velocity and
    // Waterfall still rely on it until Step 4.
    let url = if project == "purpur" {
        "https://api.purpurmc.org/v2/purpur".to_string()
    } else {
        format!("https://api.papermc.io/v2/projects/{}", project)
    };

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if project == "purpur" {
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let mut versions: Vec<String> = json["versions"]
            .as_array()
            .ok_or("Invalid Purpur API")?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect();
        // Purpur API returns oldest-first; reverse so the dropdown shows
        // newest at the top (matches PaperMC branch above).
        versions.reverse();
        Ok(versions)
    } else {
        let data = resp
            .json::<ProjectVersionList>()
            .await
            .map_err(|e| e.to_string())?;
        let mut v = data.versions;
        v.reverse();
        Ok(v)
    }
}

// ── NeoForge: thin wrappers over NeoForgeLoader ───────────────────────────

/// Returns NeoForge versions for a given Minecraft version. Delegates to
/// `NeoForgeLoader::list_builds` in `loaders::mods`.
#[tauri::command]
pub async fn get_neoforge_versions(mc_version: String, betas: bool) -> Result<Vec<String>, String> {
    NeoForgeLoader::list_builds(&mc_version, betas).await
}

/// Lists all Minecraft versions that have at least one NeoForge release.
/// Delegates to `NeoForgeLoader::list_mc_versions` in `loaders::mods`.
#[tauri::command]
pub async fn get_neoforge_mc_versions() -> Result<Vec<String>, String> {
    NeoForgeLoader::list_mc_versions().await
}
