<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { onDestroy, onMount, tick } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { toast } from "$lib/runes/toast.svelte";
    import IconPicker from "$lib/components/modals/IconPicker.svelte";
    import commandTree from "$lib/data/command_tree.json";
    import argumentData from "$lib/data/arguments.json";

    let instance = $derived(appState.selectedInstance!);
    let settings = $derived(appState.settings.console);

    // Ensure runtime exists
    $effect(() => {
        appState.ensureRuntime(instance.id);
    });

    // Derived from runtime store for persistence (Safe access)
    let runtime = $derived(
        appState.getRuntime(instance.id) || { logs: [], activeTab: "console" },
    );
    let logs = $derived(runtime.logs);

    // Proxy for activeTab to write back to store
    let activeTab = $derived(runtime.activeTab);
    function setActiveTab(tab: "console" | "settings") {
        appState.ensureRuntime(instance.id);
        const r = appState.getRuntime(instance.id);
        if (r) r.activeTab = tab;
    }

    let commandInput = $state("");
    let consoleContainer = $state<HTMLDivElement>();

    // History Index stays local
    let historyIndex = $state(-1);

    // Autocomplete State
    let showAutocomplete = $state(false);
    let autocompleteSuggestions = $state<string[]>([]);
    let autocompleteIndex = $state(0);

    let consoleProfile = $state<"Vanilla" | "Plugins" | "Mods">("Vanilla");
    let hideNoise = $state(true);
    let showConsoleToolbar = $state(false);
    let showDeleteModal = $state(false);

    async function deleteInstance() {
        if (!instance) return;
        try {
            await invoke("delete_instance", { id: instance.id });
            // Refresh logic similar to refreshInstances in Gallery
            const instances = await invoke("read_instances");
            appState.instances = instances as Instance[];

            appState.selectedInstance = null;
            appState.view = "instances";
        } catch (e) {
            console.error("Failed to delete instance:", e);
            alert("Error al eliminar la instancia: " + e);
        } finally {
            showDeleteModal = false;
        }
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

    async function toggleServer() {
        if (instance.state === "Stopped" || instance.state === "Error") {
            try {
                await invoke("start_instance", { id: instance.id });
                // Note: State updates are handled by global event listeners in +layout.svelte
            } catch (e) {
                console.error(e);
                alert("Error starting server: " + e);
            }
        } else {
            try {
                await invoke("stop_instance", { id: instance.id });
            } catch (e) {
                console.error(e);
            }
        }
    }

    async function forceKill() {
        if (
            !confirm(
                "¿Estás seguro de que quieres forzar el cierre? Esto podría corromper datos si no se ha guardado.",
            )
        )
            return;
        try {
            await invoke("kill_instance", { id: instance.id });
        } catch (e) {
            console.error(e);
            alert("Error killing server: " + e);
        }
    }

    async function sendCommand() {
        if (!commandInput.trim()) return;

        // Add to persistent history
        if (
            runtime.commandHistory.length === 0 ||
            runtime.commandHistory[runtime.commandHistory.length - 1] !==
                commandInput
        ) {
            runtime.commandHistory.push(commandInput);
            // Cap history at 50 items to prevent saturation
            if (runtime.commandHistory.length > 50) {
                runtime.commandHistory = runtime.commandHistory.slice(-50);
            }
        }
        historyIndex = -1;

        try {
            await invoke("send_command", {
                id: instance.id,
                command: commandInput,
            });
            // We do NOT manually push to logs, the server will echo back via stdout
            commandInput = "";
            scrollToBottom();
        } catch (e) {
            console.error(e);
        }
    }

    function applyAutocomplete(selected: string) {
        if (selected.startsWith("<")) return; // Argument hint

        let newCommandInput = "";

        // Special handling for the very first command to preserve "/"
        if (!commandInput.includes(" ")) {
            newCommandInput = "/" + selected;
        } else {
            // Retrieve all parts including empty strings from trailing spaces
            // "time set " -> ["time", "set", ""]
            // "time se" -> ["time", "se"]
            const parts = commandInput.substring(1).split(" ");

            // Replace the last part (which is the one being typed/completed)
            parts[parts.length - 1] = selected;

            // Reconstruct: "/" + joined parts
            newCommandInput = "/" + parts.join(" ");
        }

        // SMART SPACING LOGIC
        // Check if the selected node has children to decide if we append a space
        const raw = newCommandInput.substring(1);
        const parts = raw.split(" ");
        let currentNode: any = commandTree;
        let hasChildren = false;

        for (let i = 0; i < parts.length; i++) {
            const part = parts[i];
            if (!currentNode.children) break;

            if (currentNode.children[part]) {
                currentNode = currentNode.children[part];
            } else {
                const argChildKey = Object.keys(currentNode.children).find(
                    (k) => currentNode.children[k].type === "argument",
                );
                if (argChildKey) {
                    currentNode = currentNode.children[argChildKey];
                } else {
                    currentNode = null;
                    break;
                }
            }
        }

        if (
            currentNode &&
            currentNode.children &&
            Object.keys(currentNode.children).length > 0
        ) {
            hasChildren = true;
        }

        if (hasChildren) {
            newCommandInput += " ";
        }

        commandInput = newCommandInput;

        showAutocomplete = false;

        // Refocus input
        const input = document.querySelector('input[type="text"]');
        if (input instanceof HTMLElement) input.focus();
    }

    function handleConsoleKeydown(e: KeyboardEvent) {
        if (e.key === "Enter") {
            // Enter ONLY sends the command, never autocompletes
            sendCommand();
            showAutocomplete = false;
            return;
        }

        if (e.key === "Tab") {
            e.preventDefault();
            if (showAutocomplete && autocompleteSuggestions.length > 0) {
                applyAutocomplete(autocompleteSuggestions[autocompleteIndex]);
            }
            return;
        }

        if (e.key === " ") {
            if (showAutocomplete && autocompleteSuggestions.length > 0) {
                e.preventDefault();
                applyAutocomplete(autocompleteSuggestions[autocompleteIndex]);
                return;
            }
        }

        if (e.key === "Escape") {
            showAutocomplete = false;
            return;
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

            if (runtime.commandHistory.length === 0) return;

            if (historyIndex === -1) {
                // Start navigating from local latest
                historyIndex = runtime.commandHistory.length - 1;
            } else if (historyIndex > 0) {
                historyIndex--;
            }
            commandInput = runtime.commandHistory[historyIndex];
        } else if (e.key === "ArrowDown") {
            e.preventDefault();

            if (showAutocomplete) {
                autocompleteIndex =
                    autocompleteIndex < autocompleteSuggestions.length - 1
                        ? autocompleteIndex + 1
                        : 0;
                return;
            }

            if (historyIndex === -1) return; // Not navigating history

            if (historyIndex < runtime.commandHistory.length - 1) {
                historyIndex++;
                commandInput = runtime.commandHistory[historyIndex];
            } else {
                // Return to empty/new input
                historyIndex = -1;
                commandInput = "";
            }
        }
    }

    // Autocomplete Logic
    $effect(() => {
        if (!commandInput.startsWith("/")) {
            showAutocomplete = false;
            return;
        }

        // Remove leading / and split
        // We want to preserve trailing empty string if space matches
        const raw = commandInput.substring(1);
        const parts = raw.split(" ");
        // Example: "time set " -> ["time", "set", ""]
        // Example: "time se" -> ["time", "se"]

        let currentNode: any = commandTree;
        // Traverse for all parts except the last one (which is being typed)
        for (let i = 0; i < parts.length - 1; i++) {
            const part = parts[i];
            if (!currentNode.children) {
                currentNode = null;
                break;
            }

            // Find child that matches 'part'
            // Either a literal match specific key
            if (currentNode.children[part]) {
                currentNode = currentNode.children[part];
            } else {
                // OR check if there is an argument node (any node type: argument)
                // Arguments in brigadier are usually keys like 'targets', 'pos', etc.
                // We assume if no literal matches, we successfully "consumed" an argument?
                // We need to find IF there is an argument child.
                // In our tree structure, children keys ARE the node names (literals or arg names).
                // If we typed "10", and there is a child "seconds" (type argument), we go there.

                const argChildKey = Object.keys(currentNode.children).find(
                    (k) => currentNode.children[k].type === "argument",
                );
                if (argChildKey) {
                    currentNode = currentNode.children[argChildKey];
                } else {
                    currentNode = null; // Dead end
                    break;
                }
            }
        }

        if (currentNode && currentNode.children) {
            const currentTyped = parts[parts.length - 1].toLowerCase();

            let possible: string[] = [];

            // 1. Literal suggestions
            const literals = Object.keys(currentNode.children).filter(
                (k) => currentNode.children[k].type === "literal",
            );
            possible.push(...literals);

            // 2. Argument suggestions (Dynamic)
            const argKeys = Object.keys(currentNode.children).filter(
                (k) => currentNode.children[k].type === "argument",
            );
            for (const k of argKeys) {
                const argNode = currentNode.children[k];
                const parser = argNode.parser;

                // Check if we have dynamic data for this parser
                // access argumentData as any index signature
                const data = (argumentData as any)[parser];
                if (data && Array.isArray(data)) {
                    // Filter valid items from the big list
                    // Optimization: If currentTyped is empty, maybe don't show ALL 1000 items?
                    // Or just slice top 50.
                    const matches = data.filter((item: string) =>
                        item.startsWith(currentTyped),
                    );
                    possible.push(...matches);
                } else {
                    // Fallback to static placeholder
                    possible.push(`<${k}>`);
                }
            }

            // Filter and Sort
            const matches = possible.filter(
                (p) => p.startsWith(currentTyped) || p.startsWith("<"),
            );

            if (matches.length > 0) {
                autocompleteSuggestions = matches
                    .sort((a, b) => {
                        // Argument placeholders last
                        const aIsArg = a.startsWith("<");
                        const bIsArg = b.startsWith("<");
                        if (aIsArg && !bIsArg) return 1;
                        if (!aIsArg && bIsArg) return -1;

                        // Prefer exact match (e.g. "time") over "time_set" if applicable (not here)
                        // Alphabetical
                        return a.localeCompare(b);
                    })
                    .slice(0, 50); // Increased limit for arguments
                showAutocomplete = true;
                autocompleteIndex = 0;
            } else {
                showAutocomplete = false;
            }
        } else {
            showAutocomplete = false;
        }
    });

    function clearLogs() {
        const r = appState.getRuntime(instance.id);
        if (r) r.logs = [];
    }

    function formatLog(log: string): { text: string; level: string } {
        if (!hideNoise) return { text: log, level: "RAW" };

        // Simple parsing for Vanilla (Strip timestamp and thread info)
        // Expected format: [HH:MM:SS] [Thread/LEVEL]: Message
        // Regex to capture: ^\[\d{2}:\d{2}:\d{2}\] \[.*?/(\w+)\]: (.*)
        const vanillaRegex = /^\[\d{2}:\d{2}:\d{2}\] \[.*?\/(\w+)\]: (.*)/;
        const match = log.match(vanillaRegex);

        if (match) {
            return { text: match[2], level: match[1] };
        }

        // Fallback for lines that don't match (maybe startup banner or simple text)
        return { text: log, level: getLogLevel(log) };
    }

    function getLogLevel(log: string): string {
        if (log.includes("ERROR") || log.includes("stderr")) return "ERROR";
        if (log.includes("WARN")) return "WARN";
        return "INFO";
    }

    function openFolder() {
        invoke("open_instances_folder", { slug: instance.path });
    }

    // --- Settings Logic ---
    // --- Settings Logic ---
    let systemRam = $state(0); // Total System RAM in Bytes
    let originalSettings = $state<any>(null); // To track changes
    let lastSyncedId = $state<string | null>(null); // Track which instance is loaded

    let linkMemory = $state(true);
    let formSettings = $state({
        min_ram: 1024,
        max_ram: 2048,
        port: 25565,
        args: "",
        jar_file: "server.jar",
    });

    let showConfirmModal = $state(false);
    let pendingTab = $state<"console" | "settings" | null>(null);

    let isDirty = $derived(
        originalSettings &&
            JSON.stringify(formSettings) !== JSON.stringify(originalSettings),
    );

    let isServerRunning = $derived(
        instance.state === "Running" || instance.state === "Starting",
    );

    onMount(async () => {
        try {
            systemRam = await invoke<number>("get_system_memory");
        } catch (e) {
            console.error("Failed to get system memory", e);
        }
    });

    // Sync form with instance when selected instance changes
    $effect(() => {
        if (instance && instance.settings) {
            if (lastSyncedId !== instance.id) {
                formSettings = { ...instance.settings };
                originalSettings = { ...instance.settings };
                lastSyncedId = instance.id;
            }
        }
    });

    // Intercept Tab Change
    function handleTabChange(tab: "console" | "settings") {
        if (activeTab === "settings" && tab !== "settings" && isDirty) {
            pendingTab = tab;
            showConfirmModal = true;
        } else {
            setActiveTab(tab);
        }
    }

    function confirmDiscard() {
        discardChanges();
        showConfirmModal = false;
        if (pendingTab) {
            setActiveTab(pendingTab);
            pendingTab = null;
        }
    }

    function cancelDiscard() {
        showConfirmModal = false;
        pendingTab = null;
    }

    async function saveSettings() {
        try {
            await invoke("save_instance_settings", {
                instanceId: instance.id,
                settings: formSettings,
            });
            // Update global state
            if (appState.selectedInstance) {
                appState.selectedInstance.settings = { ...formSettings };
            }
            originalSettings = { ...formSettings };
            toast.success("¡Configuración guardada correctamente!");
        } catch (e) {
            console.error(e);
            toast.error("Error al guardar: " + e);
        }
    }

    function discardChanges() {
        if (originalSettings) {
            formSettings = { ...originalSettings };
        }
    }

    function resetRamSettings() {
        formSettings.min_ram = 2048;
        formSettings.max_ram = 2048;
    }

    function resetGeneralSettingsForm() {
        formSettings.port = 25565;
        formSettings.jar_file = "server.jar";
    }

    // --- Icon Picker Logic ---
    let showIconPicker = $state(false);

    async function handleIconSelected(newIcon: string) {
        showIconPicker = false;
        try {
            await invoke("update_instance_icon", {
                id: instance.id,
                icon: newIcon,
            });
            // Update local state directly for immediate feedback
            if (appState.selectedInstance) {
                appState.selectedInstance.icon = newIcon;
            }
            // Update instance in list
            const idx = appState.instances.findIndex(
                (i) => i.id === instance.id,
            );
            if (idx !== -1) {
                appState.instances[idx].icon = newIcon;
            }
            toast.success("Icono actualizado correctamente");
        } catch (e) {
            console.error(e);
            toast.error("Error al actualizar icono: " + e);
        }
    }
</script>

{#if showIconPicker}
    <IconPicker
        onselect={handleIconSelected}
        onclose={() => (showIconPicker = false)}
    />
{/if}

<div class="flex flex-col h-full w-full bg-[#192232]">
    <!-- Header / Top Bar -->
    <div
        class="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-[#1e293b]/50"
        data-tauri-drag-region
    >
        <div class="flex items-center gap-4">
            <!-- Icon -->
            <!-- Icon -->
            <div
                class="group w-16 h-16 rounded-2xl bg-[#0f1520] flex items-center justify-center shadow-lg border border-white/10 overflow-hidden relative"
            >
                <img
                    src={instance.icon ||
                        `https://ui-avatars.com/api/?name=${instance.name}&background=random`}
                    alt={instance.name}
                    class="w-full h-full object-cover"
                />

                <!-- Edit Overlay -->
                <button
                    onclick={() => (showIconPicker = true)}
                    class="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center cursor-pointer"
                    title="Cambiar icono"
                >
                    <svg
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="text-white drop-shadow-md"
                    >
                        <path
                            d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"
                        />
                        <path d="m15 5 4 4" />
                    </svg>
                </button>
            </div>

            <!-- Info -->
            <div>
                <h1
                    class="text-2xl font-bold text-white tracking-tight flex items-center gap-3"
                >
                    {instance.name}
                </h1>
                <div class="flex items-center gap-2 mt-1">
                    <span
                        class="px-2 py-0.5 rounded text-xs font-medium bg-white/5 text-zinc-400 border border-white/5"
                    >
                        {instance.loader}
                    </span>
                    <span class="text-zinc-500 text-sm">•</span>
                    <span class="text-zinc-400 text-sm">
                        {instance.version}{instance.build
                            ? ` (Build #${instance.build})`
                            : ""}
                    </span>
                </div>
            </div>
        </div>

        <!-- Actions -->
        <div class="flex items-center gap-3">
            <!-- Delete Button -->
            <button
                class="p-2.5 rounded-xl bg-white/5 hover:bg-red-500/10 text-zinc-400 hover:text-red-400 border border-white/5 transition-colors group disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-white/5 disabled:hover:text-zinc-400"
                title={instance.state === "Running" ||
                instance.state === "Starting"
                    ? "Detén la instancia para eliminarla"
                    : "Eliminar Instancia"}
                onclick={() => (showDeleteModal = true)}
                disabled={instance.state === "Running" ||
                    instance.state === "Starting"}
            >
                <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="group-hover:scale-110 transition-transform"
                    ><polyline points="3 6 5 6 21 6"></polyline><path
                        d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                    ></path><line x1="10" y1="11" x2="10" y2="17"></line><line
                        x1="14"
                        y1="11"
                        x2="14"
                        y2="17"
                    ></line></svg
                >
            </button>

            <!-- Open Folder Button -->
            <button
                class="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-zinc-400 hover:text-white border border-white/5 transition-colors"
                title="Abrir Carpeta"
                onclick={openFolder}
            >
                <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path
                        d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 2H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2z"
                    ></path></svg
                >
            </button>

            <!-- Start/Stop Button -->
            <button
                class={`px-6 py-2.5 rounded-xl font-bold flex items-center gap-2 transition-all shadow-lg text-sm border ${
                    instance.state === "Running"
                        ? "bg-red-500 hover:bg-red-600 text-white border-red-400 shadow-red-900/20"
                        : instance.state === "Starting"
                          ? "bg-yellow-600/50 text-white border-yellow-500/50 cursor-wait"
                          : "bg-green-600 hover:bg-green-500 text-white border-green-400 shadow-green-900/30"
                }`}
                disabled={instance.state === "Starting"}
                onclick={toggleServer}
            >
                {#if instance.state === "Running"}
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        ><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg
                    >
                    Detener
                {:else if instance.state === "Starting"}
                    <div
                        class="animate-spin h-5 w-5 border-2 border-white border-t-transparent rounded-full"
                    ></div>
                    Iniciando...
                {:else}
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="currentColor"><path d="M8 5v14l11-7z" /></svg
                    >
                    Iniciar
                {/if}
            </button>
        </div>
    </div>

    <!-- Stats & Tabs Bar -->
    <div
        class="flex items-center justify-between border-b border-white/5 bg-[#192232] px-6 h-14"
    >
        <div class="flex h-full">
            <button
                class="px-4 text-sm font-medium border-b-2 transition-colors flex items-center {activeTab ===
                'console'
                    ? 'border-blue-500 text-blue-400'
                    : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
                onclick={() => handleTabChange("console")}
            >
                Consola
            </button>
            <button
                class="px-4 text-sm font-medium border-b-2 transition-colors flex items-center {activeTab ===
                'settings'
                    ? 'border-blue-500 text-blue-400'
                    : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
                onclick={() => handleTabChange("settings")}
            >
                Ajustes
            </button>
        </div>

        <!-- Right Actions -->
        <div class="flex items-center gap-4">
            {#if activeTab === "console"}
                <button
                    onclick={() => (showConsoleToolbar = !showConsoleToolbar)}
                    title={showConsoleToolbar
                        ? "Ocultar ajustes"
                        : "Mostrar ajustes"}
                    class="h-8 w-8 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-400 hover:text-white flex items-center justify-center transition-colors border border-white/5"
                >
                    {#if showConsoleToolbar}
                        <!-- Minus Icon -->
                        <svg
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"><path d="M5 12h14" /></svg
                        >
                    {:else}
                        <!-- Plus Icon -->
                        <svg
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><path d="M5 12h14" /><path d="M12 5v14" /></svg
                        >
                    {/if}
                </button>

                <button
                    onclick={clearLogs}
                    title="Limpiar consola"
                    class="px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-400 hover:text-white font-bold text-xs transition-colors border border-white/5 flex items-center gap-2"
                >
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path
                            d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21"
                        ></path>
                        <path d="M22 21H7"></path>
                        <path d="m5 11 9 9"></path>
                    </svg>
                    Clear
                </button>
            {/if}

            {#if activeTab === "settings"}
                {#if isDirty}
                    <button
                        onclick={discardChanges}
                        disabled={isServerRunning}
                        class="px-3 py-1.5 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-500 font-bold text-xs transition-colors border border-red-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        Descartar
                    </button>
                {/if}
                <button
                    onclick={saveSettings}
                    disabled={!isDirty || isServerRunning}
                    class={`px-3 py-1.5 rounded-lg font-bold text-xs transition-colors shadow-lg ${
                        !isDirty || isServerRunning
                            ? "bg-zinc-700 text-zinc-400 cursor-not-allowed opacity-50"
                            : "bg-blue-600 hover:bg-blue-500 text-white"
                    }`}
                >
                    Guardar Cambios
                </button>
            {/if}

            <!-- Status Indicator -->
            <div class="flex items-center gap-2 pl-4 border-l border-white/5">
                <div
                    class={`w-2 h-2 rounded-full ${
                        instance.state === "Running"
                            ? "bg-green-500 animate-pulse"
                            : instance.state === "Starting"
                              ? "bg-yellow-500 animate-pulse"
                              : "bg-red-500"
                    }`}
                ></div>
                <span
                    class={`text-xs font-bold uppercase tracking-wider px-2 py-0.5 rounded border ${
                        instance.state === "Running"
                            ? "bg-green-500/10 text-green-400 border-green-500/20"
                            : "bg-red-500/10 text-red-400 border-red-500/20"
                    }`}
                >
                    {instance.state}
                </span>
            </div>
        </div>
    </div>

    <!-- Content Area -->
    {#if activeTab === "console"}
        <div
            class="flex-1 p-0 bg-[#0f1520] overflow-hidden flex flex-col relative"
            style:font-family={settings.fontFamily}
            style:font-size="{settings.fontSize}px"
            style:line-height={settings.lineHeight}
            style:letter-spacing="{settings.letterSpacing}px"
            style:font-weight={settings.fontWeight}
        >
            <!-- Console Toolbar -->
            {#if showConsoleToolbar}
                <div
                    transition:fade={{ duration: 150 }}
                    class="flex items-center justify-between px-4 py-2 border-b border-white/5 bg-[#141b26] text-xs"
                >
                    <div class="flex items-center gap-4">
                        <label
                            class="flex items-center gap-2 cursor-pointer hover:text-white text-zinc-400 transition-colors"
                        >
                            <input
                                type="checkbox"
                                bind:checked={hideNoise}
                                class="rounded bg-white/10 border-white/10 text-blue-500 focus:ring-0 w-3.5 h-3.5"
                            />
                            Ocultar ruido (Raw)
                        </label>
                        <div class="flex items-center gap-2">
                            <span class="text-zinc-500">Perfil:</span>
                            <select
                                bind:value={consoleProfile}
                                class="bg-transparent border border-white/10 rounded px-2 py-0.5 text-zinc-300 focus:border-blue-500 focus:outline-none focus:text-white"
                            >
                                <option value="Vanilla">Vanilla</option>
                                <option value="Plugins" disabled
                                    >Plugins (Pronto)</option
                                >
                                <option value="Mods" disabled
                                    >Mods (Pronto)</option
                                >
                            </select>
                        </div>
                    </div>
                </div>
            {/if}

            <div
                bind:this={consoleContainer}
                class="flex-1 overflow-y-auto p-4 space-y-1 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent"
            >
                {#each logs as log}
                    {@const formatted = formatLog(log)}
                    <div
                        class="text-zinc-300 break-words hover:bg-white/5 px-1 rounded -mx-1"
                    >
                        {#if formatted.level === "INFO"}
                            <span class="text-blue-400">[INFO]</span>
                            {formatted.text}
                        {:else if formatted.level === "WARN"}
                            <span class="text-yellow-400">[WARN]</span>
                            {formatted.text}
                        {:else if formatted.level === "ERROR"}
                            <span class="text-red-400">[ERROR]</span>
                            {formatted.text}
                        {:else}
                            {formatted.text}
                        {/if}
                    </div>
                {/each}
            </div>

            <!-- Command Input -->
            <div
                class="relative px-3 py-1.5 bg-[#1e293b] border-t border-white/5 flex gap-2 items-center"
            >
                {#if showAutocomplete && autocompleteSuggestions.length > 0}
                    <div
                        class="absolute bottom-full left-0 w-full mb-1 bg-[#1e293b] border border-white/10 rounded-t-lg shadow-xl overflow-hidden z-50"
                    >
                        {#each autocompleteSuggestions as cmd, i}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <div
                                class="px-3 py-2 text-sm font-jetbrains cursor-pointer flex justify-between {i ===
                                autocompleteIndex
                                    ? 'bg-blue-600 text-white'
                                    : 'text-zinc-300 hover:bg-white/5'}"
                                role="button"
                                tabindex="0"
                                onclick={() => {
                                    applyAutocomplete(cmd);
                                }}
                            >
                                <span>{cmd}</span>
                                {#if cmd.startsWith("<")}
                                    <span class="opacity-50 text-xs italic"
                                        >Argumento</span
                                    >
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}
                <span class="text-zinc-500 select-none text-xl">{">"}</span>
                <input
                    type="text"
                    bind:value={commandInput}
                    onkeydown={handleConsoleKeydown}
                    placeholder="Escribe un comando..."
                    class="bg-transparent border-none outline-none flex-1 text-white placeholder-zinc-600 font-jetbrains text-sm font-light"
                />
            </div>
        </div>
    {:else}
        <!-- Instance Settings -->
        <div class="flex-1 flex flex-col min-h-0">
            <!-- Scrollable Content -->
            <div class="flex-1 p-8 overflow-y-auto">
                <div class="max-w-3xl space-y-8">
                    <!-- Warning Banner if Running -->
                    {#if isServerRunning}
                        <div
                            class="mb-6 bg-yellow-500/10 border border-yellow-500/20 rounded-xl p-4 flex items-center gap-3"
                        >
                            <div
                                class="p-2 rounded-lg bg-yellow-500/10 text-yellow-500"
                            >
                                <svg
                                    width="20"
                                    height="20"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    ><circle cx="12" cy="12" r="10"
                                    ></circle><line
                                        x1="12"
                                        y1="8"
                                        x2="12"
                                        y2="12"
                                    ></line><line
                                        x1="12"
                                        y1="16"
                                        x2="12.01"
                                        y2="16"
                                    ></line></svg
                                >
                            </div>
                            <div>
                                <h3 class="font-bold text-yellow-500 text-sm">
                                    Ajustes bloqueados
                                </h3>
                                <p class="text-xs text-yellow-500/80">
                                    No puedes modificar la configuración
                                    mientras el servidor está encendido.
                                </p>
                            </div>
                        </div>
                    {/if}

                    <!-- Main Ops Button -->
                    <div class="flex items-center gap-2">
                        <button
                            onclick={toggleServer}
                            class="px-6 py-2 rounded-lg font-bold shadow-lg transition-all active:scale-95 flex items-center gap-2
                            {instance.state === 'Running' ||
                            instance.state === 'Starting'
                                ? 'bg-red-600 hover:bg-red-500 text-white shadow-red-900/20'
                                : 'bg-green-600 hover:bg-green-500 text-white shadow-green-900/20'}"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class:hidden={instance.state === "Running" ||
                                    instance.state === "Starting"}
                            >
                                <polygon points="5 3 19 12 5 21 5 3"></polygon>
                            </svg>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class:hidden={instance.state !== "Running" &&
                                    instance.state !== "Starting"}
                            >
                                <rect x="6" y="4" width="4" height="16"></rect>
                                <rect x="14" y="4" width="4" height="16"></rect>
                            </svg>
                            {instance.state === "Running" ||
                            instance.state === "Starting"
                                ? "Detener"
                                : "Iniciar"}
                        </button>

                        <!-- Force Kill Button (Small) -->
                        {#if instance.state === "Running" || instance.state === "Starting"}
                            <button
                                onclick={forceKill}
                                class="p-2.5 rounded-lg bg-red-900/30 hover:bg-red-900/50 text-red-400 hover:text-red-200 border border-red-900/50 transition-all flex items-center justify-center"
                                title="Forzar Cierre (Kill)"
                            >
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="16"
                                    height="16"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    class="lucide lucide-skull"
                                    ><path d="m11.1 12.93-1.8 1.8" /><path
                                        d="m14.7 12.93 1.8 1.8"
                                    /><path
                                        d="M15.6 11.6a9 9 0 1 1-7.2 0Z"
                                    /></svg
                                >
                            </button>
                        {/if}
                    </div>
                    <section
                        class="bg-[#1e293b]/50 border border-white/5 rounded-xl p-6 relative transition-opacity {isServerRunning
                            ? 'opacity-50 pointer-events-none'
                            : ''}"
                    >
                        <!-- Reset Button - Absolute Position -->
                        <button
                            onclick={resetRamSettings}
                            disabled={isServerRunning}
                            title="Restablecer valores por defecto"
                            class="absolute top-6 right-6 p-2 rounded-lg hover:bg-white/5 text-zinc-500 hover:text-white transition-colors"
                        >
                            <svg
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                ><path
                                    d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"
                                ></path><path d="M3 3v5h5"></path></svg
                            >
                        </button>

                        <div class="flex items-center gap-3 mb-4">
                            <div
                                class="p-2 rounded-lg bg-blue-500/10 text-blue-400"
                            >
                                <svg
                                    width="20"
                                    height="20"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    ><path d="M22 12h-4l-3 9L9 3l-3 9H2" /></svg
                                >
                            </div>
                            <div>
                                <h3 class="font-medium text-white">
                                    Asignación de Memoria (RAM)
                                </h3>
                                <p class="text-xs text-zinc-400">
                                    Memoria del Sistema detectada: <span
                                        class="text-blue-400 font-bold"
                                        >{(systemRam / 1073741824).toFixed(1)} GB</span
                                    >
                                </p>
                            </div>
                        </div>

                        <div
                            class="grid grid-cols-1 md:grid-cols-2 gap-6 items-center"
                        >
                            <!-- Left: Slider -->
                            <div class="space-y-2">
                                <div class="text-center pb-1 translate-y-2">
                                    <p
                                        class="text-base text-blue-400 font-bold"
                                    >
                                        {(formSettings.max_ram / 1024).toFixed(
                                            1,
                                        )} GB
                                        <span
                                            class="text-zinc-500 font-normal ml-1"
                                            >Seleccionado</span
                                        >
                                    </p>
                                </div>

                                <div class="relative pt-2 pb-2">
                                    <input
                                        type="range"
                                        min="1024"
                                        max={systemRam / 1048576}
                                        step="512"
                                        value={formSettings.max_ram}
                                        oninput={(e) => {
                                            const val = parseInt(
                                                e.currentTarget.value,
                                            );
                                            formSettings.max_ram = val;
                                            if (linkMemory)
                                                formSettings.min_ram = val;
                                        }}
                                        disabled={isServerRunning}
                                        class="w-full h-2 bg-[#0f1520] border border-white/5 rounded-lg appearance-none cursor-pointer accent-blue-500 disabled:opacity-50"
                                    />
                                    <!-- Marks -->
                                    <div
                                        class="flex justify-between text-[10px] text-zinc-500 font-bold mt-2 font-mono uppercase tracking-wider"
                                    >
                                        <span>1 GB</span>
                                        <span>2 GB</span>
                                        <span>4 GB</span>
                                        <span>8 GB</span>
                                        <span
                                            >{(systemRam / 1073741824).toFixed(
                                                0,
                                            )} GB</span
                                        >
                                    </div>
                                </div>
                            </div>

                            <!-- Right: Inputs -->
                            <div class="flex items-end gap-3 -translate-y-2">
                                <div class="flex gap-4">
                                    <!-- Min RAM -->
                                    <div class="space-y-2">
                                        <label
                                            for="min-ram"
                                            class="text-[10px] font-bold text-zinc-400 uppercase tracking-wider"
                                            >Mínima (MB) (-Xms)</label
                                        >
                                        <div class="relative group">
                                            <input
                                                id="min-ram"
                                                type="number"
                                                value={formSettings.min_ram}
                                                oninput={(e) => {
                                                    const val = parseInt(
                                                        e.currentTarget.value,
                                                    );
                                                    formSettings.min_ram = val;
                                                    if (linkMemory)
                                                        formSettings.max_ram =
                                                            val;
                                                }}
                                                disabled={isServerRunning}
                                                class="w-28 h-10 bg-[#0f1520] border border-white/10 rounded-lg px-3 text-white focus:border-blue-500 focus:outline-none transition-colors font-mono text-sm disabled:cursor-not-allowed group-hover:border-white/20"
                                            />
                                        </div>
                                    </div>
                                    <!-- Max RAM -->
                                    <div class="space-y-3">
                                        <label
                                            for="max-ram"
                                            class="text-[10px] font-bold text-zinc-400 uppercase tracking-wider"
                                            >Máxima (MB) (-Xmx)</label
                                        >
                                        <div class="relative group">
                                            <input
                                                id="max-ram"
                                                type="number"
                                                value={formSettings.max_ram}
                                                oninput={(e) => {
                                                    const val = parseInt(
                                                        e.currentTarget.value,
                                                    );
                                                    formSettings.max_ram = val;
                                                    if (linkMemory)
                                                        formSettings.min_ram =
                                                            val;
                                                }}
                                                disabled={isServerRunning}
                                                class="w-28 h-10 bg-[#0f1520] border border-white/10 rounded-lg px-3 text-white focus:border-blue-500 focus:outline-none transition-colors font-mono text-sm disabled:cursor-not-allowed group-hover:border-white/20"
                                            />
                                        </div>
                                    </div>
                                </div>

                                <!-- Link Button -->
                                <button
                                    class="h-8 w-8 mb-1 flex items-center justify-center rounded-lg border transition-all {linkMemory
                                        ? 'bg-blue-500/10 border-blue-500/50 text-blue-400'
                                        : 'bg-[#0f1520] border-white/10 text-zinc-500 hover:text-zinc-300'}"
                                    onclick={() => {
                                        linkMemory = !linkMemory;
                                        if (linkMemory) {
                                            // Sync when enabling
                                            formSettings.min_ram =
                                                formSettings.max_ram;
                                        }
                                    }}
                                    title={linkMemory
                                        ? "Desvincular memoria"
                                        : "Vincular memoria (Igualar Min y Max)"}
                                    disabled={isServerRunning}
                                >
                                    {#if linkMemory}
                                        <svg
                                            width="20"
                                            height="20"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            ><path
                                                d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
                                            ></path><path
                                                d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
                                            ></path></svg
                                        >
                                    {:else}
                                        <svg
                                            width="20"
                                            height="20"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            class="opacity-50"
                                            ><path
                                                d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
                                            ></path><path
                                                d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
                                            ></path><line
                                                x1="4"
                                                y1="4"
                                                x2="20"
                                                y2="20"
                                            /></svg
                                        >
                                    {/if}
                                </button>
                            </div>
                        </div>
                    </section>

                    <!-- General Configuration -->
                    <section
                        class="bg-[#1e293b]/50 border border-white/5 rounded-xl p-6 relative transition-opacity {isServerRunning
                            ? 'opacity-50 pointer-events-none'
                            : ''}"
                    >
                        <div class="flex items-center justify-between mb-6">
                            <div class="flex items-center gap-3">
                                <div
                                    class="p-2 rounded-lg bg-purple-500/10 text-purple-400"
                                >
                                    <svg
                                        width="20"
                                        height="20"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        ><rect
                                            x="2"
                                            y="2"
                                            width="20"
                                            height="8"
                                            rx="2"
                                            ry="2"
                                        ></rect><rect
                                            x="2"
                                            y="14"
                                            width="20"
                                            height="8"
                                            rx="2"
                                            ry="2"
                                        ></rect><line
                                            x1="6"
                                            y1="6"
                                            x2="6.01"
                                            y2="6"
                                        ></line><line
                                            x1="6"
                                            y1="18"
                                            x2="6.01"
                                            y2="18"
                                        ></line></svg
                                    >
                                </div>
                                <div>
                                    <h3 class="font-medium text-white">
                                        Configuración General
                                    </h3>
                                    <p class="text-xs text-zinc-400">
                                        Ajustes básicos del servidor.
                                    </p>
                                </div>
                            </div>
                            <button
                                onclick={resetGeneralSettingsForm}
                                disabled={isServerRunning}
                                title="Restablecer valores por defecto"
                                class="p-2 rounded-lg hover:bg-white/5 text-zinc-500 hover:text-white transition-colors"
                            >
                                <svg
                                    width="18"
                                    height="18"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    ><path
                                        d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"
                                    ></path><path d="M3 3v5h5"></path></svg
                                >
                            </button>
                        </div>

                        <div class="space-y-4">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <!-- Server Port -->
                                <div class="space-y-2">
                                    <label
                                        for="server-port"
                                        class="text-xs font-bold text-zinc-400 uppercase"
                                        >Puerto del Servidor</label
                                    >
                                    <input
                                        id="server-port"
                                        type="number"
                                        bind:value={formSettings.port}
                                        placeholder="25565"
                                        disabled={isServerRunning}
                                        class="w-full bg-[#0f1520] border border-white/10 rounded-lg px-4 py-2.5 text-white focus:border-purple-500 focus:outline-none transition-colors font-mono disabled:cursor-not-allowed"
                                    />
                                </div>
                                <!-- Jar File -->
                                <div class="space-y-2">
                                    <label
                                        for="jar-file"
                                        class="text-xs font-bold text-zinc-400 uppercase"
                                        >Archivo Jar</label
                                    >
                                    <input
                                        id="jar-file"
                                        type="text"
                                        bind:value={formSettings.jar_file}
                                        placeholder="server.jar"
                                        disabled={isServerRunning}
                                        class="w-full bg-[#0f1520] border border-white/10 rounded-lg px-4 py-2.5 text-white focus:border-purple-500 focus:outline-none transition-colors font-mono disabled:cursor-not-allowed"
                                    />
                                </div>
                            </div>
                        </div>
                    </section>

                    <!-- Advanced -->
                    <section
                        class="bg-[#1e293b]/50 border border-white/5 rounded-xl p-6 relative transition-opacity {isServerRunning
                            ? 'opacity-50 pointer-events-none'
                            : ''}"
                    >
                        <div class="flex items-center gap-3 mb-6">
                            <div
                                class="p-2 rounded-lg bg-orange-500/10 text-orange-400"
                            >
                                <svg
                                    width="20"
                                    height="20"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    ><path
                                        d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
                                    ></path></svg
                                >
                            </div>
                            <div>
                                <h3 class="font-medium text-white">
                                    Argumentos JVM (Avanzado)
                                </h3>
                                <p class="text-xs text-zinc-400">
                                    Flags adicionales de arranque (ej. Aikars
                                    flags).
                                </p>
                            </div>
                        </div>

                        <div class="space-y-2">
                            <textarea
                                bind:value={formSettings.args}
                                rows="3"
                                placeholder="-XX:+UseG1GC -XX:MaxGCPauseMillis=200..."
                                disabled={isServerRunning}
                                class="w-full bg-[#0f1520] border border-white/10 rounded-lg px-4 py-3 text-white focus:border-orange-500 focus:outline-none transition-colors font-mono text-sm disabled:cursor-not-allowed"
                            ></textarea>
                        </div>
                    </section>
                </div>
            </div>
        </div>
    {/if}

    <!-- Custom Modal Overlay -->
    {#if showConfirmModal}
        <div
            class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm"
            transition:fade={{ duration: 200 }}
        >
            <div
                class="bg-[#192232] border border-white/10 rounded-2xl shadow-2xl max-w-md w-full overflow-hidden"
                in:fly={{ y: 20, duration: 300 }}
            >
                <div class="p-6">
                    <div class="flex items-center gap-4 mb-4">
                        <div
                            class="p-3 rounded-xl bg-yellow-500/10 text-yellow-500"
                        >
                            <svg
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                ><path
                                    d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
                                ></path><line x1="12" y1="9" x2="12" y2="13"
                                ></line><line x1="12" y1="17" x2="12.01" y2="17"
                                ></line></svg
                            >
                        </div>
                        <div>
                            <h3 class="text-lg font-bold text-white">
                                Cambios sin guardar
                            </h3>
                            <p class="text-zinc-400 text-sm">
                                Tienes modificaciones pendientes en la
                                configuración.
                            </p>
                        </div>
                    </div>
                    <p class="text-zinc-300 text-sm leading-relaxed">
                        Si sales ahora, perderás todos los cambios que has
                        realizado. ¿Estás seguro de que quieres descartarlos?
                    </p>
                </div>
                <div
                    class="bg-[#0f1520]/50 p-4 border-t border-white/5 flex gap-3 justify-end"
                >
                    <button
                        class="px-4 py-2 rounded-lg text-sm font-bold text-zinc-300 hover:text-white hover:bg-white/5 transition-colors"
                        onclick={cancelDiscard}
                    >
                        Cancelar
                    </button>
                    <button
                        class="px-4 py-2 rounded-lg text-sm font-bold bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/20 transition-all"
                        onclick={confirmDiscard}
                    >
                        Descartar y Salir
                    </button>
                </div>
            </div>
        </div>
    {/if}

    <!-- Delete Confirmation Modal -->
    {#if showDeleteModal}
        <div
            class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm"
            transition:fade={{ duration: 200 }}
        >
            <div
                class="bg-[#192232] border border-white/10 rounded-2xl shadow-2xl max-w-md w-full overflow-hidden"
                in:fly={{ y: 20, duration: 300 }}
            >
                <div class="p-6">
                    <div class="flex items-center gap-4 mb-4">
                        <div
                            class="p-3 rounded-xl bg-orange-500/10 text-orange-500"
                        >
                            <svg
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                ><path
                                    d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
                                ></path><line x1="12" y1="9" x2="12" y2="13"
                                ></line><line x1="12" y1="17" x2="12.01" y2="17"
                                ></line></svg
                            >
                        </div>
                        <div>
                            <h3 class="text-lg font-bold text-white">
                                ¿Eliminar Instancia?
                            </h3>
                            <p class="text-zinc-400 text-sm">
                                Esta acción eliminará permanentemente la carpeta
                                y los datos de la instancia "<b
                                    >{instance?.name}</b
                                >".
                            </p>
                        </div>
                    </div>

                    <div class="flex justify-end gap-3 mt-6">
                        <button
                            class="px-4 py-2 rounded-lg text-sm font-bold text-zinc-400 hover:text-white hover:bg-white/5 transition-colors"
                            onclick={() => (showDeleteModal = false)}
                        >
                            Cancelar
                        </button>
                        <button
                            class="px-4 py-2 rounded-lg text-sm font-bold bg-red-500 hover:bg-red-400 text-white shadow-lg shadow-red-900/20 transition-all"
                            onclick={deleteInstance}
                        >
                            Eliminar Permanentemente
                        </button>
                    </div>
                </div>
            </div>
        </div>
    {/if}
</div>
