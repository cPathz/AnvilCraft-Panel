<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { tick, onMount } from "svelte";
    import { slide } from "svelte/transition";
    import { _ } from "svelte-i18n";
    import { FEATURES } from "$lib/config/features";
    import commandTree from "$lib/data/command_tree.json";
    import argumentData from "$lib/data/arguments.json";
    import { parseMinecraftLog } from "$lib/utils/logParser";

    let { instanceId } = $props();

    // Derived State
    let runtime = $derived(
        appState.getRuntime(instanceId) || { logs: [], activeTab: "console" },
    );
    let logs = $derived(runtime.logs);
    let instance = $derived(
        appState.instances.find((i) => i.id === instanceId),
    );
    let currentVersion = $derived(instance?.version);
    let consoleSettings = $derived(appState.settings.console);

    // Local State
    let commandInput = $state("");
    let consoleContainer = $state<HTMLDivElement>();
    let historyIndex = $state(-1);

    // Autocomplete State
    let showAutocomplete = $state(false);
    let autocompleteSuggestions = $state<string[]>([]);
    let autocompleteIndex = $state(0);

    let showConsoleToolbar = $state(false);

    let inputElement = $state<HTMLInputElement>();
    let highlightElement = $state<HTMLDivElement>();

    // Players Panel State
    let showPlayers = $state(true);
    let panelTab = $state<'players' | 'format'>('players');

    // Toolbar visibility
    let toolbarVisible = $state(true);
    let toolbarTimeout: ReturnType<typeof setTimeout> | null = null;

    function hideToolbarAfterDelay() {
        if (toolbarTimeout) clearTimeout(toolbarTimeout);
        toolbarTimeout = setTimeout(() => {
            toolbarVisible = false;
        }, 2000);
    }

    function showToolbar() {
        toolbarVisible = true;
        if (toolbarTimeout) clearTimeout(toolbarTimeout);
    }
    let players = $derived(runtime?.players || []);
    let maxPlayers = $state(20);

    async function fetchMaxPlayers() {
        try {
            const max = await invoke("get_instance_max_players", { id: instanceId });
            maxPlayers = max as number;
        } catch (e) {
            console.error("Failed to get max players:", e);
        }
    }

    onMount(fetchMaxPlayers);

    // Auto-refresh stats when server starts
    $effect(() => {
        if (instance?.state === 'Running') {
            fetchMaxPlayers();
        }
    });

    async function refreshPlayers() {
        fetchMaxPlayers(); // Also refresh limit
        try {
            await invoke("send_command", {
                id: instanceId,
                command: "list",
            });
        } catch (e) {
            console.error("Failed to send list command:", e);
        }
    }

    function syncScroll() {
        if (inputElement && highlightElement) {
            highlightElement.scrollLeft = inputElement.scrollLeft;
        }
    }

    // --- Logic ---

    function escapeHtml(unsafe: string) {
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }

    // Auto-scroll when logs change
    $effect(() => {
        if (logs.length && consoleContainer) {
            scrollToBottom();
        }
    });

    function scrollToBottom() {
        tick().then(() => {
            if (consoleContainer) {
                consoleContainer.scrollTop = consoleContainer.scrollHeight;
            }
        });
    }

    export function clearLogs() {
        const r = appState.getRuntime(instanceId);
        if (r) r.logs = [];
    }

    function getLogLevel(log: string): string {
        if (log.includes("ERROR") || log.includes("stderr")) return "ERROR";
        if (log.includes("WARN")) return "WARN";
        return "INFO";
    }

    function formatLog(log: string): { text: string; level: string } {
        return { text: log, level: getLogLevel(log) };
    }


    async function sendCommand() {
        if (!commandInput.trim()) return;

        // Add to persistent history via global store
        const r = appState.getRuntime(instanceId);
        if (r) {
            if (
                r.commandHistory.length === 0 ||
                r.commandHistory[r.commandHistory.length - 1] !== commandInput
            ) {
                r.commandHistory.push(commandInput);
                if (r.commandHistory.length > 50) r.commandHistory.slice(-50);
            }
        }

        historyIndex = -1;
        try {
            await invoke("send_command", {
                id: instanceId,
                command: commandInput,
            });
            commandInput = "";
            showAutocomplete = false;
        } catch (e) {
            console.error(e);
        }
    }

    function applyAutocomplete(
        suggestion: string,
        trigger: "Tab" | "Enter" | "Click" | "Space",
    ) {
        if (suggestion.startsWith("<")) return;

        let newCommandInput = "";
        if (!commandInput.includes(" ")) {
            newCommandInput = "/" + suggestion;
        } else {
            const parts = commandInput.substring(1).split(" ");
            parts[parts.length - 1] = suggestion;
            newCommandInput = "/" + parts.join(" ");
        }

        if (trigger === "Space") newCommandInput += " ";

        commandInput = newCommandInput;
        showAutocomplete = false;

        // Refocus input
        const input = document.querySelector('input[type="text"]');
        if (input instanceof HTMLElement) input.focus();
    }

    function handleConsoleKeydown(e: KeyboardEvent) {
        if (e.key === "Enter") {
            sendCommand();
            showAutocomplete = false;
            return;
        }

        if (FEATURES.CONSOLE.AUTOCOMPLETE) {
            if (e.key === "Tab") {
                e.preventDefault();
                if (showAutocomplete && autocompleteSuggestions.length > 0) {
                    applyAutocomplete(
                        autocompleteSuggestions[autocompleteIndex],
                        "Tab",
                    );
                }
                return;
            }
            if (e.key === " ") {
                if (showAutocomplete && autocompleteSuggestions.length > 0) {
                    e.preventDefault();
                    applyAutocomplete(
                        autocompleteSuggestions[autocompleteIndex],
                        "Space",
                    );
                    return;
                }
            }
            if (e.key === "Escape") {
                showAutocomplete = false;
                return;
            }
        }

        if (e.key === "ArrowUp") {
            e.preventDefault();
            if (showAutocomplete) {
                autocompleteIndex =
                    autocompleteIndex > 0
                        ? autocompleteIndex - 1
                        : autocompleteSuggestions.length - 1;
                return;
            }
            const r = appState.getRuntime(instanceId);
            if (r && r.commandHistory.length > 0) {
                if (historyIndex === -1)
                    historyIndex = r.commandHistory.length - 1;
                else if (historyIndex > 0) historyIndex--;
                commandInput = r.commandHistory[historyIndex];
            }
        } else if (e.key === "ArrowDown") {
            e.preventDefault();
            if (showAutocomplete) {
                autocompleteIndex =
                    autocompleteIndex < autocompleteSuggestions.length - 1
                        ? autocompleteIndex + 1
                        : 0;
                return;
            }
            const r = appState.getRuntime(instanceId);
            if (r && historyIndex !== -1) {
                if (historyIndex < r.commandHistory.length - 1) {
                    historyIndex++;
                    commandInput = r.commandHistory[historyIndex];
                } else {
                    historyIndex = -1;
                    commandInput = "";
                }
            }
        }
    }

    // Autocomplete Effect
    $effect(() => {
        if (!FEATURES.CONSOLE.AUTOCOMPLETE) {
            showAutocomplete = false;
            return;
        }
        if (!commandInput.startsWith("/")) {
            showAutocomplete = false;
            return;
        }

        const raw = commandInput.substring(1);
        const parts = raw.split(" ");
        let currentNode: any = commandTree;

        for (let i = 0; i < parts.length - 1; i++) {
            const part = parts[i];
            if (!currentNode.children) {
                currentNode = null;
                break;
            }
            if (currentNode.children[part]) {
                currentNode = currentNode.children[part];
            } else {
                const argChildKey = Object.keys(currentNode.children).find(
                    (k) => currentNode.children[k].type === "argument",
                );
                if (argChildKey)
                    currentNode = currentNode.children[argChildKey];
                else {
                    currentNode = null;
                    break;
                }
            }
        }

        if (currentNode && currentNode.children) {
            const currentTyped = parts[parts.length - 1].toLowerCase();
            let possible: string[] = [];

            possible.push(
                ...Object.keys(currentNode.children).filter((k) => {
                    const child = currentNode.children[k];
                    if (child.type !== "literal") return false;

                    // Version Filtering
                    if (
                        currentVersion &&
                        child.versions &&
                        child.versions.length > 0
                    ) {
                        const match =
                            child.versions.includes(currentVersion) ||
                            child.versions.includes("all");
                        // console.log(`Checking ${k}: ${match} (versions: ${child.versions.length})`);
                        return match;
                    }
                    return true;
                }),
            );

            const argKeys = Object.keys(currentNode.children).filter(
                (k) => currentNode.children[k].type === "argument",
            );
            for (const k of argKeys) {
                const parser = currentNode.children[k].parser;

                // Fallback for missing arguments in arguments.json
                const MISSING_ARGUMENTS: Record<
                    string,
                    { id: string; versions: string[] }[]
                > = {
                    "minecraft:gamemode": [
                        { id: "survival", versions: ["all"] },
                        { id: "creative", versions: ["all"] },
                        { id: "adventure", versions: ["all"] },
                        { id: "spectator", versions: ["all"] },
                    ],
                };

                let rawData = (argumentData as any)[parser];
                if (!rawData && MISSING_ARGUMENTS[parser]) {
                    rawData = MISSING_ARGUMENTS[parser];
                }

                if (rawData && Array.isArray(rawData)) {
                    // Check if simple string array or versioned object array
                    const isVersioned =
                        rawData.length > 0 && typeof rawData[0] === "object";

                    if (isVersioned) {
                        const data = rawData as {
                            id: string;
                            versions: string[];
                        }[];
                        possible.push(
                            ...data
                                .filter((item) => {
                                    if (
                                        !item.id.includes(
                                            currentTyped.toLowerCase(),
                                        )
                                    )
                                        return false;
                                    // If we don't know the version, show everything
                                    if (!currentVersion) return true;
                                    // "all" wildcard or specific version match
                                    return (
                                        item.versions.includes("all") ||
                                        item.versions.includes(currentVersion)
                                    );
                                })
                                .map((item) => item.id),
                        );
                    } else {
                        // Legacy/Simple support
                        const data = rawData as string[];
                        possible.push(
                            ...data.filter((item) =>
                                item.includes(currentTyped.toLowerCase()),
                            ),
                        );
                    }
                } else {
                    possible.push(`<${k}>`);
                }
            }

            const matches = possible.filter(
                (p) => p.includes(currentTyped) || p.startsWith("<"),
            );
            if (matches.length > 0) {
                console.log(
                    "Autocomplete triggering with:",
                    matches.length,
                    "matches",
                );
                autocompleteSuggestions = matches
                    .sort((a, b) => {
                        const aIsArg = a.startsWith("<");
                        const bIsArg = b.startsWith("<");
                        if (aIsArg && !bIsArg) return 1;
                        if (!aIsArg && bIsArg) return -1;
                        return a.toLowerCase().localeCompare(b.toLowerCase());
                    })
                    .slice(0, 500);
                showAutocomplete = true;
                autocompleteIndex = 0;
            } else {
                console.log("No matches found for autocomplete");
                showAutocomplete = false;
            }
        } else {
            // console.log("Not triggering autocomplete (not starting with /)");
            showAutocomplete = false;
        }
    });

    // Auto-scroll Effect
    $effect(() => {
        if (showAutocomplete && autocompleteIndex >= 0) {
            const el = document.getElementById(
                `autocomplete-item-${autocompleteIndex}`,
            );
            if (el) el.scrollIntoView({ block: "nearest", behavior: "auto" });
        }
    });

    // Syntax Highlighting
    let coloredHtml = $derived.by(() => {
        if (!commandInput) return "";
        try {
            const tokens = commandInput.split(/(\s+)/);
            let htmlParts: string[] = [];
            let nodeTraversalCursor: any = commandTree;
            let isCommandContext = false;
            let isPathValid = true;

            tokens.forEach((token, index) => {
                if (token.length === 0) return;
                if (/^\s+$/.test(token)) {
                    htmlParts.push(`<span>${escapeHtml(token)}</span>`);
                    return;
                }

                if (!isCommandContext && index === 0) {
                    if (token.startsWith("/")) {
                        isCommandContext = true;
                        const cmd = token.substring(1);
                        if (
                            nodeTraversalCursor.children &&
                            nodeTraversalCursor.children[cmd]
                        ) {
                            nodeTraversalCursor =
                                nodeTraversalCursor.children[cmd];
                            htmlParts.push(
                                `<span class="text-gray-500">/</span><span class="text-zinc-300">${escapeHtml(cmd)}</span>`,
                            );
                        } else {
                            isPathValid = false;
                            htmlParts.push(
                                `<span class="text-gray-500">/</span><span class="text-red-500">${escapeHtml(cmd)}</span>`,
                            );
                        }
                    } else {
                        // Not a command, possibly chat
                        htmlParts.push(
                            `<span class="text-zinc-300">${escapeHtml(token)}</span>`,
                        );
                    }
                    return;
                }

                if (!isCommandContext) {
                    htmlParts.push(
                        `<span class="text-zinc-300">${escapeHtml(token)}</span>`,
                    );
                    return;
                }

                if (!isPathValid) {
                    htmlParts.push(
                        `<span class="text-red-500">${escapeHtml(token)}</span>`,
                    );
                    return;
                }

                if (nodeTraversalCursor && nodeTraversalCursor.children) {
                    if (nodeTraversalCursor.children[token]) {
                        nodeTraversalCursor =
                            nodeTraversalCursor.children[token];
                        htmlParts.push(
                            `<span class="text-zinc-300">${escapeHtml(token)}</span>`,
                        );
                    } else {
                        const argChildKey = Object.keys(
                            nodeTraversalCursor.children,
                        ).find(
                            (k) =>
                                nodeTraversalCursor.children[k].type ===
                                "argument",
                        );
                        if (argChildKey) {
                            const argNode =
                                nodeTraversalCursor.children[argChildKey];
                            nodeTraversalCursor = argNode;
                            let argColor = "text-yellow-200";
                            if (argNode.parser === "brigadier:integer")
                                argColor = "text-green-400";
                            if (argNode.parser === "minecraft:entity")
                                argColor = "text-yellow-400";
                            htmlParts.push(
                                `<span class="${argColor}">${escapeHtml(token)}</span>`,
                            );
                        } else {
                            isPathValid = false;
                            htmlParts.push(
                                `<span class="text-red-500">${escapeHtml(token)}</span>`,
                            );
                        }
                    }
                } else {
                    isPathValid = false;
                    htmlParts.push(
                        `<span class="text-red-500">${escapeHtml(token)}</span>`,
                    );
                }
            });
            return htmlParts.join("");
        } catch (e) {
            console.error("Syntax highlight error:", e);
            return `<span class="text-zinc-300">${escapeHtml(commandInput)}</span>`;
        }
    });
</script>

<div class="flex-1 min-h-0 flex flex-col relative group">
    <div class="flex-1 min-h-0 flex flex-row gap-4 mt-4">
        <!-- Logs Area -->
        <div 
            class="flex-1 relative flex flex-col bg-[#1e293b]/95 rounded-xl border border-white/10 overflow-hidden"
            onmouseenter={() => (showConsoleToolbar = true)}
            onmouseleave={() => (showConsoleToolbar = false)}
        >
            <!-- Floating Toolbar (Fixed) -->
            <div
                class="absolute top-3 right-4 z-30 flex flex-col gap-2 transition-opacity duration-500 {toolbarVisible ? 'opacity-100' : 'opacity-40'}"
                onmouseenter={showToolbar}
                onmouseleave={hideToolbarAfterDelay}
            >
                <!-- Expand/Collapse Panel Button -->
                <button
                    onclick={() => showPlayers = !showPlayers}
                    class="p-2 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 text-zinc-400 hover:text-white transition-all active:scale-95 shadow-lg backdrop-blur-md"
                    title={showPlayers ? "Ocultar panel" : "Mostrar panel"}
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                        {#if showPlayers}
                            <path d="M9 6l6 6l-6 6"/>
                        {:else}
                            <path d="M15 6l-6 6l6 6"/>
                        {/if}
                    </svg>
                </button>

                <!-- Players Button -->
                <button
                    onclick={() => { showPlayers = true; panelTab = 'players'; }}
                    class="p-2 rounded-lg {panelTab === 'players' && showPlayers ? 'bg-blue-500/20 border-blue-500/50 text-white' : 'bg-white/5 border-white/10 text-zinc-400 hover:text-white hover:bg-white/10'} border transition-all active:scale-95 shadow-lg backdrop-blur-md"
                    title="Mostrar jugadores conectados"
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                    </svg>
                </button>

                <!-- Format Console Button -->
                <button
                    onclick={() => { showPlayers = true; panelTab = 'format'; }}
                    class="p-2 rounded-lg {panelTab === 'format' && showPlayers ? 'bg-blue-500/20 border-blue-500/50 text-white' : 'bg-white/5 border-white/10 text-zinc-400 hover:text-white hover:bg-white/10'} border transition-all active:scale-95 shadow-lg backdrop-blur-md"
                    title="Opciones de formato de consola"
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/><path d="M8 9h.01"/><path d="M12 9h.01"/><path d="M16 9h.01"/>
                    </svg>
                </button>

            </div>

            <!-- Scrollable Logs -->
            <div
                class="flex-1 overflow-y-auto p-2 custom-scrollbar flex flex-col justify-start relative"
                style:font-family={appState.applyConsoleSettings ? consoleSettings.fontFamily : undefined}
                style:font-size={appState.applyConsoleSettings ? `${consoleSettings.fontSize}px` : undefined}
                style:line-height={appState.applyConsoleSettings ? consoleSettings.lineHeight : '1.5'}
                style:letter-spacing={appState.applyConsoleSettings ? `${consoleSettings.letterSpacing}px` : undefined}
                style:font-weight={appState.applyConsoleSettings ? consoleSettings.fontWeight : undefined}
                bind:this={consoleContainer}
                role="group"
            >
                {#each logs.slice(-200) as log}
                    {@const formatted = formatLog(log)}
                    <div class="px-2 text-gray-400 hover:bg-white/5 {appState.wrapConsoleText ? 'break-words whitespace-pre-wrap' : 'whitespace-pre'}">
                        {formatted.text}
                    </div>
                {/each}

                {#if logs.length === 0}
                    <div
                        class="absolute inset-0 flex flex-col items-center justify-center text-zinc-700 pointer-events-none select-none"
                    >
                        <div class="bg-white/5 p-4 rounded-3xl mb-4">
                            <svg
                                width="48"
                                height="48"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class="opacity-50"
                                ><rect width="18" height="18" x="2" y="4" rx="2" /><path
                                    d="m10 10-6 6"
                                /><path d="m6 10 6 6" /><path d="m14 10 2 2 2-2" /><path
                                    d="m14 14 2-2 2 2"
                                /></svg
                            >
                        </div>
                        <span class="text-sm font-medium opacity-50"
                            >{$_("console.waiting")}</span
                        >
                    </div>
                {/if}
            </div>
        </div>

    <!-- Players/Format Panel -->
    {#if showPlayers}
        <div
            transition:slide={{ axis: 'x', duration: 300 }}
            class="w-52 bg-[#1e293b]/95 rounded-xl border border-white/10 flex flex-col overflow-hidden shadow-2xl relative"
        >
            {#if panelTab === 'players'}
                <!-- Players Panel -->
                <!-- Header -->
                <div class="px-4 py-3 border-b border-white/5 bg-white/5 flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <span class="text-xs font-bold uppercase tracking-wider text-zinc-500">{$_("console.users_panel_title")}</span>
                        <span class="px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 text-[10px] font-bold">{players.length}</span>
                    </div>
                    <button
                        onclick={refreshPlayers}
                        class="p-1.5 rounded-md text-zinc-500 hover:text-white hover:bg-white/5 transition-all active:rotate-180 duration-500"
                        title={$_("gallery.refresh")}
                    >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>
                        </svg>
                    </button>
                </div>

                <!-- Players List -->
                <div class="flex-1 overflow-y-auto custom-scrollbar p-1.5 space-y-1">
                    {#each players as player}
                        <div class="flex items-center gap-3 p-2.5 rounded-xl hover:bg-white/10 transition-all group/player cursor-default relative">
                            <!-- Avatar -->
                            <div class="w-10 h-10 rounded-xl bg-zinc-800 flex items-center justify-center overflow-hidden shadow-lg border border-white/5 group-hover/player:scale-105 transition-transform shrink-0">
                                <img
                                    src="https://mc-heads.net/avatar/{player}/64"
                                    alt={player}
                                    class="w-full h-full object-contain"
                                    onerror={(e) => {
                                        const target = e.currentTarget as HTMLImageElement;
                                        target.src = "https://mc-heads.net/avatar/MHF_Steve/64";
                                    }}
                                />
                            </div>

                            <div class="flex flex-col min-w-0">
                                <span class="text-[15px] font-bold text-white truncate leading-tight">{player}</span>
                                <div class="flex items-center gap-1.5">
                                    <span class="text-[10px] text-zinc-500 font-black uppercase tracking-widest">{$_("console.player_role")}</span>
                                </div>
                            </div>

                            <!-- Status Indicator -->
                            <div class="absolute right-3 top-1/2 -translate-y-1/2 w-2 h-2 rounded-full bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.6)]"></div>
                        </div>
                    {:else}
                        <div class="flex flex-col items-center justify-center py-8 opacity-20 select-none">
                            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                            </svg>
                            <span class="text-[13px] font-bold uppercase tracking-widest mt-2 text-center px-4">{$_("console.users_no_active")}</span>
                        </div>
                    {/each}
                </div>

                <!-- Footer Stats -->
                <div class="p-3 border-t border-white/5 bg-black/20">
                    <div class="flex items-center justify-between text-[13px] text-zinc-500 font-bold uppercase tracking-widest">
                        <span>{$_("console.users_slots")}</span>
                        <span class="text-zinc-400 font-mono text-[13px]">{players.length}/{maxPlayers}</span>
                    </div>
                </div>
            {:else if panelTab === 'format'}
                <!-- Format Console Panel -->
                <div class="px-4 py-3 border-b border-white/5 bg-white/5">
                    <span class="text-xs font-bold uppercase tracking-wider text-zinc-500">Formato Consola</span>
                </div>

                <!-- Format Options -->
                <div class="flex-1 overflow-y-auto custom-scrollbar p-4 space-y-3">
                    <div class="text-xs text-zinc-400 mb-4">
                        Las opciones de formato se aplicarán próximamente en esta sección.
                    </div>
                </div>
            {/if}
        </div>
    {/if}
</div>

    <!-- Command Input Area (Discord Style) -->
    <div class="pl-0 pr-0 pt-4 pb-3 bg-transparent relative z-20">
        <div
            class="relative bg-[#1e293b]/95 rounded-xl border border-white/10 flex items-center shadow-lg transition-colors focus-within:border-blue-500/50 focus-within:bg-[#1e293b] focus-within:ring-1 focus-within:ring-blue-500/50"
        >
            <!-- Styled Console Input Area -->
            <div
                class="relative w-full flex items-center px-4 py-1.5 min-h-[32px]"
            >
                <!-- Autocomplete Popup (Moved here for better relative positioning context) -->
                {#if FEATURES.CONSOLE.AUTOCOMPLETE && showAutocomplete && autocompleteSuggestions.length > 0}
                    <div
                        class="absolute bottom-full left-4 right-4 mb-2 bg-[#1e293b] border border-white/10 rounded-xl shadow-2xl overflow-hidden z-[100] max-h-[300px] overflow-y-auto custom-scrollbar p-1 flex flex-col-reverse"
                    >
                        {#each autocompleteSuggestions as cmd, i}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <div
                                id="autocomplete-item-{i}"
                                class="px-3 py-2 text-sm cursor-pointer rounded-lg transition-colors flex items-center justify-between {i ===
                                autocompleteIndex
                                    ? 'bg-blue-600 text-white'
                                    : 'text-zinc-400 hover:bg-white/5'}"
                                style:font-family={consoleSettings.fontFamily}
                                role="button"
                                tabindex="0"
                                onclick={() => applyAutocomplete(cmd, "Click")}
                            >
                                <span class="font-medium"
                                    >{cmd.replace("minecraft:", "")}</span
                                >
                                {#if cmd.startsWith("<")}
                                    <span
                                        class="text-[10px] opacity-70 uppercase tracking-wider font-bold"
                                        >Arg</span
                                    >
                                {/if}
                            </div>
                        {/each}
                        <div
                            class="px-3 py-1.5 text-[10px] text-zinc-500 font-bold uppercase tracking-wider border-b border-white/10 bg-[#1e293b]/50 backdrop-blur pb-1 mb-1"
                        >
                            {$_("console.suggestions")}
                        </div>
                    </div>
                {/if}
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="text-zinc-500 mr-2 select-none pointer-events-none opacity-80"
                >
                    <path d="m9 18 6-6-6-6" />
                </svg>
                <div class="relative flex-1 h-7 flex items-center overflow-hidden">
                    <!-- Highlight Layer -->
                    <div
                        bind:this={highlightElement}
                        class="absolute inset-0 w-full h-full text-[15px] leading-normal whitespace-pre flex items-center pointer-events-none z-10 overflow-x-hidden"
                        style:font-family={consoleSettings.fontFamily}
                        aria-hidden="true"
                    >
                        {@html coloredHtml}
                    </div>
                    <!-- Transparent Input Layer -->
                    <input
                        bind:this={inputElement}
                        type="text"
                        bind:value={commandInput}
                        onkeydown={handleConsoleKeydown}
                        oninput={syncScroll}
                        onscroll={syncScroll}
                        placeholder={coloredHtml ? "" : $_("console.placeholder")}
                        class="absolute inset-0 w-full h-full bg-transparent text-transparent caret-blue-400 text-[15px] leading-normal outline-none z-20 border-none ring-0 p-0 m-0 focus:ring-0 placeholder:text-zinc-600"
                        style:font-family={consoleSettings.fontFamily}
                        autocomplete="off"
                        spellcheck="false"
                        autocorrect="off"
                    />
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    /* Scoped but global for this component's hierarchy */
    :global(.custom-scrollbar::-webkit-scrollbar) {
        width: 6px;
    }
    :global(.custom-scrollbar::-webkit-scrollbar-track) {
        background: transparent;
    }
    :global(.custom-scrollbar::-webkit-scrollbar-thumb) {
        background-color: rgba(255, 255, 255, 0.1);
        border-radius: 9999px;
    }
    :global(.custom-scrollbar::-webkit-scrollbar-thumb:hover) {
        background-color: rgba(255, 255, 255, 0.2);
    }
</style>
