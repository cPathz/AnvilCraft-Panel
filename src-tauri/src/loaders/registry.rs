use std::sync::OnceLock;

use crate::loaders::LoaderStrategy;
use crate::loaders::{bukkit, hybrids, mods, proxies, vanilla};
use crate::models::InstanceEngine;

static REGISTRY: OnceLock<LoaderRegistry> = OnceLock::new();

/// Central lookup for all known loader strategies.
pub struct LoaderRegistry {
    loaders: &'static [&'static dyn LoaderStrategy],
}

impl LoaderRegistry {
    /// Initialize-once global registry holding references to every loader.
    /// 16 loaders, 5 categories, O(n) lookup by `InstanceEngine`.
    pub fn global() -> &'static Self {
        REGISTRY.get_or_init(|| Self {
            loaders: &[
                // Vanilla
                &vanilla::VanillaLoader,
                // Bukkit family
                &bukkit::PaperLoader,
                &bukkit::SpigotLoader,
                &bukkit::PurpurLoader,
                &bukkit::FoliaLoader,
                // Mods family
                &mods::NeoForgeLoader,
                &mods::ForgeLoader,
                &mods::FabricLoader,
                &mods::QuiltLoader,
                // Hybrid family (stubs)
                &hybrids::MohistLoader,
                &hybrids::ArclightLoader,
                &hybrids::BannerLoader,
                &hybrids::MagmaLoader,
                // Proxies family
                &proxies::VelocityLoader,
                &proxies::WaterfallLoader,
                &proxies::BungeeCordLoader,
            ],
        })
    }

    /// Look up a loader by its engine variant. Returns None if the engine is
    /// not registered (shouldn't happen for any variant in `InstanceEngine`).
    pub fn by_engine(&self, e: InstanceEngine) -> Option<&'static dyn LoaderStrategy> {
        self.loaders.iter().copied().find(|l| l.engine() == e)
    }

    /// All registered loaders, in registry order.
    pub fn all(&self) -> &[&'static dyn LoaderStrategy] {
        self.loaders
    }
}
