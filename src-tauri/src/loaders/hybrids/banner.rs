//! Banner — the only Fabric-based hybrid (Bukkit + Fabric mod support).
//!
//! Real install flow is a follow-up. Banner publishes via its own Jenkins
//! CI; the install path downloads `banner-{mc}.jar` and runs it. For now the
//! stub returns a clear error so the UI surfaces the loader as "registered
//! but not yet installable".

use async_trait::async_trait;
use std::path::Path;

use crate::loaders::hybrids::{hybrid_capabilities, not_implemented};
use crate::loaders::{LoaderCapabilities, LoaderCategory, LoaderStrategy};
use crate::loaders::types::VersionMeta;
use crate::models::InstanceEngine;

pub struct BannerLoader;

#[async_trait]
impl LoaderStrategy for BannerLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Banner
    }
    fn category(&self) -> LoaderCategory {
        LoaderCategory::Hybrids
    }
    fn display_name(&self) -> &'static str {
        "Banner"
    }
    fn min_java(&self) -> u8 {
        11
    }
    fn capabilities(&self) -> LoaderCapabilities {
        hybrid_capabilities()
    }
    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        Err(not_implemented("Banner"))
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
        Err(not_implemented("Banner"))
    }
}
