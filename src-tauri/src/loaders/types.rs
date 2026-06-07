use serde::{Deserialize, Serialize};

/// Lightweight metadata for a single installable build/version of a loader.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionMeta {
    pub id: String,
    pub build: Option<String>,
    pub url: Option<String>,
    pub display_name: String,
    pub requires_mc_version: Option<String>,
}

/// A concrete download target: URL + filename + expected size.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadInfo {
    pub url: String,
    pub filename: String,
    pub size: Option<u64>,
}

/// Coarse install phase for progress reporting.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    Starting,
    Downloading,
    Executing,
    Finalizing,
    Done,
    Error,
}
