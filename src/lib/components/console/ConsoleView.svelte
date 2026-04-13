<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { tick } from "svelte";
    import { _ } from "svelte-i18n";
    import { FEATURES } from "$lib/config/features";
    import commandTree from "$lib/data/command_tree.json";
    import argumentData from "$lib/data/arguments.json";

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

    let hideNoise = $state(true);
    let showConsoleToolbar = $state(false);

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

    export function toggleNoise() {
        hideNoise = !hideNoise;
    }

    export function getHideNoise() {
        return hideNoise;
    }

    function getLogLevel(log: string): string {
        if (log.includes("ERROR") || log.includes("stderr")) return "ERROR";
        if (log.includes("WARN")) return "WARN";
        return "INFO";
    }

    function formatLog(log: string): { text: string; level: string } {
        if (!hideNoise) return { text: log, level: "RAW" };
        const vanillaRegex = /^\[\d{2}:\d{2}:\d{2}\] \[.*?\/(\w+)\]: (.*)/;
        const match = log.match(vanillaRegex);
        if (match) return { text: match[2], level: match[1] };
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
    <!-- Logs Area -->
    <!-- Logs Area -->
    <div
        class="flex-1 overflow-y-auto p-2 space-y-0.5 custom-scrollbar relative flex flex-col justify-start bg-[#1e293b]/95 rounded-xl border border-white/10 ml-0 mr-0 mt-4"
        style:font-family={consoleSettings.fontFamily}
        style:font-size="{consoleSettings.fontSize}px"
        style:line-height={consoleSettings.lineHeight}
        style:letter-spacing="{consoleSettings.letterSpacing}px"
        style:font-weight={consoleSettings.fontWeight}
        bind:this={consoleContainer}
        role="group"
        onmouseenter={() => (showConsoleToolbar = true)}
        onmouseleave={() => (showConsoleToolbar = false)}
    >
        <!-- Floating Toolbar (Inside Scroll Area but fixed? No, sticky or absolute top right of container) -->
        <div
            class="absolute top-2 right-4 z-30 flex justify-end pointer-events-none"
        >
            <div
                class="pointer-events-auto transition-all duration-200 transform translate-x-2 -translate-y-2"
                class:opacity-0={!showConsoleToolbar && hideNoise}
                class:opacity-100={showConsoleToolbar || !hideNoise}
            ></div>
        </div>

        {#each logs.slice(-200) as log}
            {@const formatted = formatLog(log)}
            <div
                class="break-words leading-tight hover:bg-white/5 px-2 rounded -mx-2 group/log relative"
            >
                {#if formatted.level !== "RAW"}
                    <span
                        class="text-[#565f89] text-xs mr-2 select-none font-bold"
                        >[{formatted.level}]</span
                    >
                {/if}
                <span
                    class={formatted.level === "ERROR"
                        ? "text-red-400"
                        : formatted.level === "WARN"
                          ? "text-yellow-400"
                          : formatted.level === "INFO"
                            ? "text-zinc-300"
                            : "text-zinc-400"}>{formatted.text}</span
                >
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
                <div class="relative flex-1 h-7 flex items-center">
                    <!-- Highlight Layer -->
                    <div
                        class="absolute inset-0 w-full h-full text-[15px] leading-normal whitespace-pre flex items-center pointer-events-none z-10"
                        style:font-family={consoleSettings.fontFamily}
                        aria-hidden="true"
                    >
                        {@html coloredHtml}
                    </div>
                    <!-- Transparent Input Layer -->
                    <input
                        type="text"
                        bind:value={commandInput}
                        onkeydown={handleConsoleKeydown}
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
