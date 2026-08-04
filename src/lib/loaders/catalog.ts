// Hand-mirrored loader catalog. Must stay in sync with the 16-entry list in
// `src-tauri/src/loaders/registry.rs`. To make the runtime catalog the
// source of truth instead, expose a `get_loaders` Tauri command that returns
// this struct from the Rust side.

import type { LoaderMetadata } from "./types";

export const LOADERS: readonly LoaderMetadata[] = [
    // Vanilla
    {
        name: "Vanilla",
        category: "Vanilla",
        minJava: 8,
        tested: true,
        capabilities: {
            supportsPlugins: false,
            supportsMods: false,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    // Bukkit family
    {
        name: "Paper",
        category: "Bukkit",
        minJava: 11,
        tested: true,
        capabilities: {
            supportsPlugins: true,
            supportsMods: false,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Spigot",
        category: "Bukkit",
        minJava: 8,
        capabilities: {
            supportsPlugins: true,
            supportsMods: false,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Purpur",
        category: "Bukkit",
        minJava: 11,
        tested: true,
        capabilities: {
            supportsPlugins: true,
            supportsMods: false,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Folia",
        category: "Bukkit",
        minJava: 17,
        tested: true,
        capabilities: {
            supportsPlugins: true,
            supportsMods: false,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    // Mods family
    {
        name: "NeoForge",
        category: "Mods",
        badge: "MODS",
        minJava: 17,
        tested: true,
        capabilities: {
            supportsPlugins: false,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Forge",
        category: "Mods",
        badge: "MODS",
        minJava: 17,
        tested: true,
        capabilities: {
            supportsPlugins: false,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Fabric",
        category: "Mods",
        badge: "MODS",
        minJava: 8,
        capabilities: {
            supportsPlugins: false,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Quilt",
        category: "Mods",
        badge: "MODS",
        minJava: 8,
        capabilities: {
            supportsPlugins: false,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    // Hybrids family
    {
        name: "Mohist",
        category: "Hybrids",
        minJava: 17,
        capabilities: {
            supportsPlugins: true,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Arclight",
        category: "Hybrids",
        minJava: 17,
        capabilities: {
            supportsPlugins: true,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Banner",
        category: "Hybrids",
        minJava: 11,
        capabilities: {
            supportsPlugins: true,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    {
        name: "Magma",
        category: "Hybrids",
        minJava: 17,
        capabilities: {
            supportsPlugins: true,
            supportsMods: true,
            isProxy: false,
            customUrlSupported: true,
        },
    },
    // Proxies family
    {
        name: "Velocity",
        category: "Proxies",
        minJava: 17,
        tested: true,
        capabilities: {
            supportsPlugins: true,
            supportsMods: false,
            isProxy: true,
            customUrlSupported: true,
        },
    },
    {
        name: "Waterfall",
        category: "Proxies",
        minJava: 11,
        tested: true,
        capabilities: {
            supportsPlugins: true,
            supportsMods: false,
            isProxy: true,
            customUrlSupported: true,
        },
    },
    {
        name: "BungeeCord",
        category: "Proxies",
        minJava: 8,
        tested: true,
        capabilities: {
            supportsPlugins: true,
            supportsMods: false,
            isProxy: true,
            customUrlSupported: true,
        },
    },
];
