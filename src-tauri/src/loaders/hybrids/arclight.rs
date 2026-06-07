//! Arclight — hybrid that supports both Forge and NeoForge as the mod host.
//!
//! Real install flow is a follow-up. Arclight publishes via its own Jenkins
//! CI; the install path downloads `arclight-{mc}.jar` and runs it. For now
//! the stub returns a clear error so the UI surfaces the loader as
//! "registered but not yet installable".

use async_trait::async_trait;
use std::path::Path;

use crate::loaders::hybrids::{hybrid_capabilities, not_implemented};
use crate::loaders::{LoaderCapabilities, LoaderCategory, LoaderStrategy};
use crate::loaders::types::VersionMeta;
use crate::models::InstanceEngine;

pub struct ArclightLoader;

#[async_trait]
impl LoaderStrategy for ArclightLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Arclight
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Hybrids
    }
    fn display_name(&self) -> &'static str {
        "Arclight"
    }
    fn min_java(&self) -> u8 {
        17
    }
    fn capabilities(&self) -> LoaderCapabilities {
        hybrid_capabilities()
    }
    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err(not_implemented("Arclight"))
    }
    async fn install(
        &self,
        _app: &tauri::AppHandle,
        _id: &str,
        _target_dir: &Path,
        _version: &VersionMeta,
        _custom_url: Option<&str>,
        _accept_eula: bool,
    ) -> Result<(), String> {
        Err(not_implemented("Arclight"))
    }
}
