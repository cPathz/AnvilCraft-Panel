use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::loaders::common::{download_file, write_eula_txt};
use crate::loaders::types::{DownloadInfo, VersionMeta};
use crate::loaders::{LoaderCapabilities, LoaderCategory, LoaderStrategy};
use crate::models::{InstanceEngine, InstanceInstallProgress};

#[derive(Debug, Deserialize)]
struct LatestVersions {
    #[allow(dead_code)]
    release: String,
    #[allow(dead_code)]
    snapshot: String,
}

#[derive(Debug, Deserialize)]
struct VersionManifest {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    sha1: String,
    size: u64,
    url: String,
}

pub struct VanillaLoader;

#[async_trait]
impl LoaderStrategy for VanillaLoader {
    fn engine(&self) -> InstanceEngine {
        InstanceEngine::Vanilla
    }

    fn category(&self) -> LoaderCategory {
        LoaderCategory::Vanilla
    }

    fn display_name(&self) -> &'static str {
        "Vanilla"
    }

    fn min_java(&self) -> u8 {
        8
    }

    fn capabilities(&self) -> LoaderCapabilities {
        LoaderCapabilities {
            supports_plugins: false,
            supports_mods: false,
            is_proxy: false,
            custom_url_supported: false,
        }
    }

    async fn fetch_versions(
        &self,
        _mc_version: Option<&str>,
    ) -> Result<Vec<VersionMeta>, String> {
        let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let manifest: VersionManifest = reqwest::get(url)
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(manifest
            .versions
            .into_iter()
            .map(|v| VersionMeta {
                id: v.id.clone(),
                build: None,
                url: Some(v.url),
                display_name: v.id,
                requires_mc_version: None,
            })
            .collect())
    }

    async fn resolve_download(&self, version: &VersionMeta) -> Result<DownloadInfo, String> {
        let details: VersionDetails = reqwest::get(
            version
                .url
                .as_deref()
                .ok_or("Vanilla version is missing its manifest URL")?,
        )
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

        let server = details.downloads.server.ok_or("No server download for this version")?;
        Ok(DownloadInfo {
            url: server.url,
            filename: "server.jar".to_string(),
            size: Some(server.size),
        })
    }

    async fn install(
        &self,
        app: &tauri::AppHandle,
        id: &str,
        target_dir: &Path,
        version: &VersionMeta,
        _custom_url: Option<&str>,
        accept_eula: bool,
    ) -> Result<(), String> {
        // Vanilla is the only loader that needs the per-version manifest URL
        // (Mojang's two-step resolution: `version_manifest_v2.json` →
        // per-version `<id>.json` → `downloads.server.url`). The dispatcher
        // only carries the version id, so look up the full VersionMeta
        // (with URL) from the manifest if the caller didn't supply it.
        let meta = if version.url.is_some() {
            version.clone()
        } else {
            let versions = self.fetch_versions(None).await?;
            versions
                .into_iter()
                .find(|v| v.id == version.id)
                .ok_or_else(|| {
                    format!("Vanilla version {} not found in Mojang manifest", version.id)
                })?
        };

        let download = self.resolve_download(&meta).await?;

        let jar_path = target_dir.join(".minecraft").join(&download.filename);
        if let Some(parent) = jar_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let log_file = target_dir.join("install.log");
        download_file(
            app,
            &download.url,
            &jar_path,
            id,
            download.size,
            Some(&log_file),
        )
        .await?;

        write_eula_txt(target_dir.join(".minecraft").join("eula.txt"), accept_eula)?;

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
}
