// Hand-mirrored types for the Rust `LoaderStrategy` trait. Keep in sync
// with `src-tauri/src/loaders/mod.rs` and `registry.rs`.

export type LoaderName =
    | "Vanilla"
    | "Paper"
    | "Spigot"
    | "Purpur"
    | "Folia"
    | "NeoForge"
    | "Forge"
    | "Fabric"
    | "Quilt"
    | "Mohist"
    | "Arclight"
    | "Banner"
    | "Magma"
    | "Velocity"
    | "Waterfall"
    | "BungeeCord";

export type LoaderCategory =
    | "Vanilla"
    | "Bukkit"
    | "Mods"
    | "Hybrids"
    | "Proxies";

export type LoaderBadge = "MODS";

export interface LoaderCapabilities {
    supportsPlugins: boolean;
    supportsMods: boolean;
    isProxy: boolean;
    customUrlSupported: boolean;
}

export interface LoaderMetadata {
    name: LoaderName;
    category: LoaderCategory;
    badge?: LoaderBadge;
    minJava: number;
    capabilities: LoaderCapabilities;
    // Dev-only TODO marker. `true` = this loader has been verified
    // end-to-end and the developer can flip it to remove the ✗ in the
    // catalog dropdown. Manually maintained — flip to `true` (or omit)
    // when a loader's install path is confirmed working.
    tested?: boolean;
}

// Render order in the UI dropdown. Mirrors the order in
// `src-tauri/src/loaders/registry.rs`.
export const CATEGORY_ORDER: readonly LoaderCategory[] = [
    "Vanilla",
    "Bukkit",
    "Mods",
    "Hybrids",
    "Proxies",
] as const;
