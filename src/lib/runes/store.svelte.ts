import type { ParsedLog } from "$lib/types/parser";
import type { LoaderName } from "$lib/loaders";

export interface Instance {
    id: string;
    name: string;
    loader: LoaderName;
    version: string;
    path: string;
    icon: string;
    date_created: string;
    last_played: string | null;
    state: 'Stopped' | 'Starting' | 'Running' | 'Stopping' | 'Installing' | 'Error';
    settings?: {
        min_ram: number;
        max_ram: number;
        port: number;
        args: string;
        jar_file: string;
    };
    build?: string;
}

class AppState {
    instances = $state<Instance[]>([]);
    selectedInstance = $state<Instance | null>(null);
    view = $state<'home' | 'instances' | 'settings' | 'dev'>('home');
    refreshing = $state<boolean>(false);
    creatingInstance = $state<boolean>(false);

    // Console Settings Toggle (Dev only)
    applyConsoleSettings = $state<boolean>(true);
    wrapConsoleText = $state<boolean>(false);
    hideConsoleLevels = $state<boolean>(false);
    showConsoleTimestamps = $state<boolean>(false);
    logFormat = $state<'raw' | 'formato1' | 'formato2' | 'formato3'>('formato3');

    // App Identity
    appInfo = $state({
        version: "",
        tag: "Beta",
        isEvalCopy: false,
        distChannel: 'standalone' as 'standalone' | 'msix'
    });

    // Update Data
    updateData = $state<{
        version: string,
        body: string,
        date: string,
        isCritical: boolean,
        available: boolean,
        rawUpdate: any
    } | null>(null);

    // Runtime state (Logs, active tabs, etc)
    instanceRuntime = $state<Record<string, { 
        logs: (ParsedLog | string)[], 
        issues: ParsedLog[],
        activeTab: "console" | "settings" | "addons" | "errors", 
        commandHistory: string[],
        players: string[],
        addonsType: 'plugins' | 'mods' | 'none'
    }>>({});

    // Global Settings
    settings = $state({
        manualUpdate: false, // Default: automatic/standard notification
        lastIgnoredVersion: "", // Stores the version string of the last dismissed update
        console: {
            fontFamily: "JetBrains Mono",
            fontSize: 14,
            lineHeight: 1.4,
            letterSpacing: 0,
            fontWeight: "400", // Normal
            theme: "Campbell"
        }
    });

    constructor() {
        if (typeof window !== 'undefined' && window.localStorage) {
            // Load global settings
            const savedSettings = window.localStorage.getItem('anvilcraft_settings');
            if (savedSettings) {
                try {
                    const parsed = JSON.parse(savedSettings);
                    this.settings = {
                        ...this.settings,
                        ...parsed,
                        console: {
                            ...this.settings.console,
                            ...(parsed.console || {})
                        }
                    };
                } catch (e) {
                    console.error("Error loading global settings:", e);
                }
            }

            // Load transient console toggles
            const savedConsoleToggles = window.localStorage.getItem('anvilcraft_console_toggles');
            if (savedConsoleToggles) {
                try {
                    const parsed = JSON.parse(savedConsoleToggles);
                    if (parsed.applyConsoleSettings !== undefined) this.applyConsoleSettings = parsed.applyConsoleSettings;
                    if (parsed.wrapConsoleText !== undefined) this.wrapConsoleText = parsed.wrapConsoleText;
                    if (parsed.hideConsoleLevels !== undefined) this.hideConsoleLevels = parsed.hideConsoleLevels;
                    if (parsed.showConsoleTimestamps !== undefined) this.showConsoleTimestamps = parsed.showConsoleTimestamps;
                    if (parsed.logFormat !== undefined) this.logFormat = parsed.logFormat;
                } catch (e) {
                    console.error("Error loading console toggles:", e);
                }
            }
        }
    }


    ensureRuntime(id: string) {
        if (!this.instanceRuntime[id]) {
            this.instanceRuntime[id] = { 
                logs: [], 
                issues: [],
                activeTab: "console", 
                commandHistory: [],
                players: [],
                addonsType: 'none'
            };
        }
    }

    getRuntime(id: string) {
        return this.instanceRuntime[id];
    }

    addLog(id: string, log: ParsedLog) {
        this.ensureRuntime(id);
        const runtime = this.instanceRuntime[id];
        if (!runtime) return;

        // Añadir a logs principales
        runtime.logs.push(log);
        if (runtime.logs.length > 2000) {
            runtime.logs = runtime.logs.slice(-2000);
        }

        // Si es un error o advertencia, añadir a la pestaña de errores
        if (log.level === 'ERROR' || log.level === 'FATAL' || log.level === 'WARN') {
            runtime.issues.push(log);
            // Limitar a 500 errores para no saturar memoria
            if (runtime.issues.length > 500) {
                runtime.issues = runtime.issues.slice(-500);
            }
        }

        // Procesar detección de jugadores, versiones, etc.
        this.parseLog(id, log.raw);
    }

    clearIssues(id: string) {
        const runtime = this.instanceRuntime[id];
        if (runtime) {
            runtime.issues = [];
        }
    }

    parseLog(id: string, line: string) {
        const runtime = this.instanceRuntime[id];
        if (!runtime) return;

        // Strip ANSI escape codes first
        const cleanLine = line.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, '').trim();
        
        // Flexible extraction: Handle [HH:mm:ss] [Level]: Msg OR [Level]: Msg OR just Msg
        // We look for the message after the last ": " or after the last "]: "
        let msg = cleanLine;
        const headerMatch = cleanLine.match(/^(?:\[.*?\]\s*)+(?::\s*)?(.*)/);
        if (headerMatch && headerMatch[1]) {
            msg = headerMatch[1].trim();
        }

        // Clean up common prefixes that modloaders or vanilla add to system messages
        msg = msg.replace(/^\[Not Secure\]\s*/i, "");

        // Join detection (Rely on NMS internal network logs + chat fallback)
        const joinMatch = msg.match(/^(.*?)\[\/.*?\] logged in with entity id/) ||
                          msg.match(/^(.*?) joined the game$/) || 
                          msg.match(/^(.*?) se ha unido al juego$/);
        if (joinMatch) {
            const name = joinMatch[1].trim();
            if (!runtime.players.includes(name)) {
                runtime.players.push(name);
            }
        }

        // Leave detection (Rely on NMS internal network logs + chat fallback)
        const leaveMatch = msg.match(/^(.*?) lost connection:/) ||
                           msg.match(/^(.*?) left the game$/) ||
                           msg.match(/^(.*?) ha abandonado el juego$/);
        if (leaveMatch) {
            const name = leaveMatch[1].trim();
            runtime.players = runtime.players.filter(p => p !== name);
        }

        // /list command detection
        const listMatch = msg.match(/^(?:There are|Hay) \d+.*?(?:players online|jugadores en l[íi]nea):\s*(.*)$/i);
        if (listMatch) {
            const namesPart = listMatch[1].trim();
            if (namesPart) {
                // Handle ", " or " " or "," separators
                const names = namesPart.split(/,\s*|\s+/).map(n => n.trim()).filter(n => n.length > 0);
                runtime.players = [...new Set(names)]; // Unique players
            } else {
                runtime.players = [];
            }
        }

        // ── Version & Loader detection ───────────────────────────────────
        //
        // The msg passed here is the *message body* of the log line (not
        // the raw line with the `[HH:MM:SS] [logger]` prefix). Anything
        // that assumes a leading "[" is wrong — see the bug history in
        // dev_log for the `msg.startsWith("[")` mistake.
        //
        // Detection priority (most specific match fires first):
        //   1. loaderMatch   — Paper/Purpur "This server is running X version Y"
        //   2. neoforgeMatch — NeoForge/Forge "NeoForge mod loading, version X, for MC Y"
        //                       **Source of truth** for NeoForge: gives both
        //                       build and MC version in a single line.
        //   3. modListMatch  — FML "Mod List" entry "NeoForge X.Y.Z (neoforge)"
        //                       Validates the loader id canonically via the
        //                       parens; redundant with neoforgeMatch for
        //                       build/MC but kept as a cross-check.
        //   4. vanillaMatch  — "Starting minecraft server version X" (universal)
        //                       Only sets version; never overwrites loader
        //                       (this line is emitted by every loader).
        //
        // Idempotency: every comparison below is `if (instance.X !== X)`,
        // so repeated server starts with the same detected values are
        // no-ops. The first start fills in the build + version detected
        // from the console; subsequent starts confirm the stored value.
        const loaderMatch = msg.match(/This server is running (\w+) version ([^\s]+)/);
        const vanillaMatch = msg.match(/Starting minecraft server version (.*)/i);
        // NeoForge/Forge explicit format. The (?:Neo)? prefix matches both
        // "NeoForge mod loading" and "Forge mod loading" without ambiguity.
        const neoforgeMatch = msg.match(/(?:Neo)?Forge mod loading, version ([\d\w.-]+), for MC ([\d\w.-]+)/i);
        // FML "Mod List" entry. Cross-checks the loader id via the
        // canonical parens suffix ("(neoforge)" vs "(forge)").
        const modListMatch = msg.match(/^\s*(Neo)?Forge\s+(\S+)\s+\((neo)?forge\)/m);

        if (loaderMatch || vanillaMatch || neoforgeMatch || modListMatch) {
            let fullVersionStr = "";
            let detectedLoader: LoaderName | undefined = undefined;
            // NeoForge/Forge get version + build assigned directly because
            // their build format (`21.9.13-beta`) doesn't fit the
            // `version-build` shape the PaperMC branch parses.
            let neoforgeDirect: { loader: "NeoForge" | "Forge"; build: string; mc: string } | null = null;

            if (loaderMatch) {
                // PaperMC / Purpur format. Captures the loader name
                // directly from "This server is running X version Y".
                detectedLoader = loaderMatch[1] as LoaderName; // e.g., "Purpur", "Paper"
                fullVersionStr = loaderMatch[2].trim();
            } else if (neoforgeMatch) {
                // ── Source of truth for NeoForge/Forge ──────────────
                // Capture[1] = build version (e.g., "21.11.42")
                // Capture[2] = MC version (e.g., "1.21.11")
                // isNeo: just look for "neoforge" in the message (the
                // message body never has a leading "[", so we don't check
                // for that). Case-insensitive for safety.
                const isNeo = /neoforge/i.test(msg);
                neoforgeDirect = {
                    loader: isNeo ? "NeoForge" : "Forge",
                    build: neoforgeMatch[1],
                    mc: neoforgeMatch[2],
                };
            } else if (modListMatch) {
                // FML Mod List — validacion cruzada del loader id.
                // Capture[1] = "Neo" prefix (NeoForge), Capture[2] =
                // build version, Capture[3] = "neo" prefix in the id
                // (always present for NeoForge, absent for vanilla Forge).
                // No MC version here — that comes from neoforgeMatch on
                // a separate log line, or from the initial install
                // selection. Leaving `mc: ""` so the comparison guard
                // (`if (neoforgeDirect.mc)`) skips the version update
                // and doesn't overwrite instance.version with an empty
                // string.
                const isNeo = !!modListMatch[1] || !!modListMatch[3];
                neoforgeDirect = {
                    loader: isNeo ? "NeoForge" : "Forge",
                    build: modListMatch[2],
                    mc: "",
                };
            } else if (vanillaMatch) {
                // Universal fallback. Does NOT set detectedLoader because
                // every loader (Vanilla, Paper, NeoForge, Forge, etc.)
                // emits this line — using it to set loader="Vanilla"
                // would silently overwrite a more specific detection
                // (NeoForge/Forge/loaderMatch) that ran on an earlier
                // log line. The loader is set correctly at instance
                // creation; this branch only confirms the version.
                fullVersionStr = vanillaMatch[1].trim();
            }

            let detectedVersion: string | undefined;
            let detectedBuild: string | undefined;
            if (neoforgeDirect) {
                detectedLoader = neoforgeDirect.loader;
                detectedBuild = neoforgeDirect.build;
                // Only assign version if this match actually carries MC
                // version info. The modListMatch path doesn't (the
                // "Minecraft X.Y.Z (minecraft)" line is processed in a
                // separate parseLog call, by which time the version has
                // already been set by neoforgeMatch or vanillaMatch).
                // Without this guard, `detectedVersion` would be `""`
                // and overwrite instance.version with an empty string.
                if (neoforgeDirect.mc) {
                    detectedVersion = neoforgeDirect.mc;
                }
            } else {
                detectedVersion = fullVersionStr;
                detectedBuild = undefined;

                if (detectedLoader !== "Vanilla" && fullVersionStr.includes('-')) {
                    const parts = fullVersionStr.split('-');
                    detectedVersion = parts[0];
                    if (parts.length > 1) {
                        detectedBuild = parts[1];
                        // Only append -snapshot/-experimental to the build if the
                        // version string itself contains such an indicator. The
                        // previous logic did a broad `msg.includes("SNAPSHOT")`
                        // which false-positived on Paper builds because Paper
                        // always references Mojang's `*-R0.1-SNAPSHOT` API
                        // version in its log line, even for stable releases.
                        // Real pre-releases still surface via the `-pre`/`-rc`
                        // checks on `instance.version` in the display component.
                        if (parts.some((p) => /snapshot/i.test(p))) {
                            detectedBuild += "-snapshot";
                        } else if (parts.some((p) => /experimental/i.test(p))) {
                            detectedBuild += "-experimental";
                        }
                    }
                }
            }

            const instance = this.instances.find(i => i.id === id);
            if (instance) {
                let needsUpdate = false;
                
                const normalizeVersion = (v: string) => {
                    return v.toLowerCase()
                        .replace(/pre-release|prerelease/g, "pre")
                        .replace(/releasecandidate/g, "rc")
                        .replace(/[^a-z0-9]/g, "");
                };

                const normExisting = normalizeVersion(instance.version);
                // Only compare/overwrite the version when we have a detected
                // value. Some branches (modListMatch) intentionally leave
                // detectedVersion undefined to avoid overwriting with `""`.
                if (detectedVersion !== undefined) {
                    const normDetected = normalizeVersion(detectedVersion);
                    if (normExisting !== normDetected) {
                        instance.version = detectedVersion;
                        needsUpdate = true;
                    }
                }
                if (detectedBuild && instance.build !== detectedBuild) {
                    instance.build = detectedBuild;
                    needsUpdate = true;
                }
                if (detectedLoader && instance.loader !== detectedLoader) {
                    instance.loader = detectedLoader;
                    needsUpdate = true;
                }

                if (needsUpdate) {
                    if (this.selectedInstance?.id === id) {
                        this.selectedInstance.version = instance.version;
                        if (detectedBuild) this.selectedInstance.build = detectedBuild;
                        if (detectedLoader) this.selectedInstance.loader = detectedLoader;
                    }
                    
                    import('@tauri-apps/api/core').then(({ invoke }) => {
                        invoke('update_instance_version', { 
                            id, 
                            version: instance.version, 
                            build: detectedBuild,
                            loader: detectedLoader
                        }).catch(console.error);
                    });
                }
            }
        }
    }
}

export const appState = new AppState();
