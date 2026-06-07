use crate::models::InstanceEngine;
use crate::loaders::types::{DownloadInfo, VersionMeta};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod common;
pub mod registry;
pub mod types;

pub mod vanilla;
pub mod bukkit;
pub mod mods;
pub mod hybrids;
pub mod proxies;

/// Top-level grouping that mirrors the UI category list.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LoaderCategory {
    Vanilla,
    Bukkit,
    Mods,
    Hybrids,
    Proxies,
}

/// Capability flags exposed to the UI.
#[derive(Default, Debug, Clone, Copy)]
pub struct LoaderCapabilities {
    pub supports_plugins: bool,
    pub supports_mods: bool,
    pub is_proxy: bool,
    pub custom_url_supported: bool,
}

/// Strategy contract for a single loader (Vanilla, Paper, NeoForge, etc.).
///
/// Every loader — including stubs that are not yet implemented — must implement
/// `engine`, `category`, and `display_name`. The rest have sensible defaults that
/// match what most loaders do (stop with "stop", write `server.jar`, no
/// custom URL).
#[async_trait]
pub trait LoaderStrategy: Send + Sync {
    // ── Identity (required) ───────────────────────────────────
    fn engine(&self) -> InstanceEngine;
    fn category(&self) -> LoaderCategory;
    fn display_name(&self) -> &'static str;

    // ── Versioning ────────────────────────────────────────────
    /// True if this loader requires an MC version before yielding builds.
    /// Only NeoForge currently uses this two-step flow.
    fn is_two_step_version(&self) -> bool {
        false
    }

    /// Step 1 for two-step loaders: list MC versions that have builds.
    async fn fetch_mc_versions(&self) -> Result<Vec<String>, String> {
        Ok(vec![])
    }

    /// Step 2 for two-step, or single-step: list installable versions.
    async fn fetch_versions(&self, _mc_version: Option<&str>) -> Result<Vec<VersionMeta>, String> {
        Err(format!(
            "{} version fetching is not yet implemented",
            self.display_name()
        ))
    }

    /// Resolve a chosen VersionMeta to a concrete download target.
    async fn resolve_download(&self, _version: &VersionMeta) -> Result<DownloadInfo, String> {
        Err(format!(
            "{} download resolution is not yet implemented",
            self.display_name()
        ))
    }

    // ── Install ───────────────────────────────────────────────
    async fn install(
        &self,
        _app: &tauri::AppHandle,
        _id: &str,
        _target_dir: &Path,
        _version: &VersionMeta,
        _custom_url: Option<&str>,
        _accept_eula: bool,
    ) -> Result<(), String> {
        Err(format!("{} install is not yet implemented", self.display_name()))
    }

    // ── Launch ────────────────────────────────────────────────
    /// stdin command to gracefully stop the running server.
    /// Most loaders use "stop"; proxies (Velocity, Waterfall, BungeeCord) use "end".
    fn stop_command(&self) -> &'static str {
        "stop"
    }

    // ── Metadata for UI ───────────────────────────────────────
    fn min_java(&self) -> u8 {
        8
    }

    fn capabilities(&self) -> LoaderCapabilities {
        LoaderCapabilities::default()
    }
}
