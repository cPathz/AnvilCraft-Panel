//! Hybrid loaders (Mohist, Arclight, Banner, Magma).
//!
//! Stubs in Step 5 of the loader-module refactor. Each loader registers
//! itself in the registry so the UI can render it, but the install path
//! returns "not yet implemented" until the per-loader patching logic lands
//! in a follow-up. The folder structure lands from day 1 so each per-loader
//! install flow gets its own file as the real implementations land.

pub mod arclight;
pub mod banner;
pub mod magma;
pub mod mohist;

pub use arclight::ArclightLoader;
pub use banner::BannerLoader;
pub use magma::MagmaLoader;
pub use mohist::MohistLoader;

use crate::loaders::LoaderCapabilities;

/// Shared capability flags for all hybrids: they support both Bukkit
/// plugins and Forge/Fabric mods, which is what makes them "hybrid".
pub(crate) fn hybrid_capabilities() -> LoaderCapabilities {
    LoaderCapabilities {
        supports_plugins: true,
        supports_mods: true,
        is_proxy: false,
        custom_url_supported: true,
    }
}

/// User-facing error string for stub install/fetch methods.
pub(crate) fn not_implemented(name: &str) -> String {
    format!("{} is not yet implemented", name)
}
