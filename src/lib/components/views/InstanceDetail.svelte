<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { onDestroy, onMount, tick } from "svelte";
    import { fade, fly } from "svelte/transition";

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

    async function sendCommand() {
        if (!commandInput.trim()) return;
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

    function openFolder() {
        invoke("open_instances_folder", { slug: instance.path });
    }

    // --- Settings Logic ---
    // --- Settings Logic ---
    let systemRam = $state(0); // Total System RAM in Bytes
    let originalSettings = $state<any>(null); // To track changes
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
            // Only update if we switched instances or first load (not while editing)
            if (
                !originalSettings ||
                instance.id !== appState.selectedInstance?.id
            ) {
                /* simple check */
            }

            if (!originalSettings || originalSettings.id !== instance.id) {
                formSettings = { ...instance.settings };
                originalSettings = { ...instance.settings, id: instance.id }; // track ID to avoid loop
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
            originalSettings = { ...formSettings, id: instance.id };
            alert("¡Configuración guardada correctamente!");
        } catch (e) {
            console.error(e);
            alert("Error al guardar: " + e);
        }
    }

    function discardChanges() {
        if (originalSettings) {
            const { id, ...settings } = originalSettings;
            formSettings = { ...settings };
        }
    }

    function resetRamSettings() {
        formSettings.min_ram = 2048;
        formSettings.max_ram = 2048;
    }

    function resetGeneralSettings() {
        formSettings.port = 25565;
        formSettings.jar_file = "server.jar";
    }
</script>

<div class="flex flex-col h-full w-full bg-[#192232]">
    <!-- Header / Top Bar -->
    <div
        class="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-[#1e293b]/50"
        data-tauri-drag-region
    >
        <div class="flex items-center gap-4">
            <!-- Icon -->
            <div
                class="w-16 h-16 rounded-2xl bg-[#0f1520] flex items-center justify-center shadow-lg border border-white/10 overflow-hidden"
            >
                <img
                    src={instance.icon ||
                        `https://ui-avatars.com/api/?name=${instance.name}&background=random`}
                    alt={instance.name}
                    class="w-full h-full object-cover"
                />
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
                    <span class="text-zinc-400 text-sm">{instance.version}</span
                    >
                </div>
            </div>
        </div>

        <!-- Actions -->
        <div class="flex items-center gap-3">
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
        class="flex items-center justify-between border-b border-white/5 bg-[#192232] px-6"
    >
        <div class="flex">
            <button
                class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
                'console'
                    ? 'border-blue-500 text-blue-400'
                    : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
                onclick={() => handleTabChange("console")}
            >
                Consola
            </button>
            <button
                class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
                'settings'
                    ? 'border-blue-500 text-blue-400'
                    : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
                onclick={() => handleTabChange("settings")}
            >
                Ajustes
            </button>
        </div>

        <!-- Status Indicator -->
        <div class="flex items-center">
            <div class="flex items-center gap-2">
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
            <div
                bind:this={consoleContainer}
                class="flex-1 overflow-y-auto p-4 space-y-1 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent"
            >
                {#each logs as log}
                    <div
                        class="text-zinc-300 break-words hover:bg-white/5 px-1 rounded -mx-1"
                    >
                        <!-- Simple highlighting -->
                        {#if log.includes("INFO")}
                            <span class="text-blue-400">[INFO]</span>
                            {log.split("INFO]:")[1] || log.split("INFO]")[1]}
                        {:else if log.includes("WARN")}
                            <span class="text-yellow-400">[WARN]</span>
                            {log.split("WARN]:")[1] || log.split("WARN]")[1]}
                        {:else if log.includes("ERROR")}
                            <span class="text-red-400">[ERROR]</span>
                            {log.split("ERROR]:")[1] || log.split("ERROR]")[1]}
                        {:else}
                            {log}
                        {/if}
                    </div>
                {/each}
            </div>

            <!-- Command Input -->
            <div class="p-3 bg-[#1e293b] border-t border-white/5 flex gap-2">
                <span class="text-zinc-500 select-none">{">"}</span>
                <input
                    type="text"
                    bind:value={commandInput}
                    onkeydown={(e) => e.key === "Enter" && sendCommand()}
                    placeholder="Escribe un comando..."
                    class="bg-transparent border-none outline-none flex-1 text-white placeholder-zinc-600 font-mono"
                />
            </div>
        </div>
    {:else}
        <!-- Instance Settings -->
        <div class="flex-1 flex flex-col min-h-0">
            <!-- Fixed Header -->
            <div class="px-8 py-6 border-b border-white/5 bg-[#192232] z-10">
                <div class="max-w-3xl">
                    <h2
                        class="text-xl font-bold text-white flex justify-between items-center"
                    >
                        <span>Configuración de Instancia</span>
                        <div class="flex items-center gap-3">
                            {#if isDirty}
                                <button
                                    onclick={discardChanges}
                                    disabled={isServerRunning}
                                    class="px-4 py-2 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 font-bold text-sm transition-colors border border-red-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    Descartar
                                </button>
                            {/if}
                            <button
                                onclick={saveSettings}
                                disabled={!isDirty || isServerRunning}
                                class={`px-4 py-2 rounded-lg font-bold text-sm transition-colors shadow-lg ${
                                    !isDirty || isServerRunning
                                        ? "bg-zinc-700 text-zinc-400 cursor-not-allowed opacity-50"
                                        : "bg-blue-600 hover:bg-blue-500 text-white"
                                }`}
                            >
                                Guardar Cambios
                            </button>
                        </div>
                    </h2>
                </div>
            </div>

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

                    <!-- RAM Section -->
                    <section
                        class="bg-[#1e293b]/50 border border-white/5 rounded-xl p-6 relative transition-opacity {isServerRunning
                            ? 'opacity-50 pointer-events-none'
                            : ''}"
                    >
                        <!-- ... content ... -->
                        <div class="flex items-center justify-between mb-4">
                            <div class="flex items-center gap-3">
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
                                        ><path
                                            d="M22 12h-4l-3 9L9 3l-3 9H2"
                                        /></svg
                                    >
                                </div>
                                <div>
                                    <h3 class="font-medium text-white">
                                        Asignación de Memoria (RAM)
                                    </h3>
                                    <p class="text-xs text-zinc-400">
                                        Memoria del Sistema detectada: <span
                                            class="text-blue-400 font-bold"
                                            >{(systemRam / 1073741824).toFixed(
                                                1,
                                            )} GB</span
                                        >
                                    </p>
                                </div>
                            </div>
                            <button
                                onclick={resetRamSettings}
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

                        <div class="space-y-6">
                            <div class="grid grid-cols-2 gap-6">
                                <!-- Min RAM -->
                                <div class="space-y-2">
                                    <label
                                        for="min-ram"
                                        class="text-xs font-bold text-zinc-400 uppercase"
                                        >Mínima (MB) (-Xms)</label
                                    >
                                    <input
                                        id="min-ram"
                                        type="number"
                                        bind:value={formSettings.min_ram}
                                        disabled={isServerRunning}
                                        class="w-full bg-[#0f1520] border border-white/10 rounded-lg px-4 py-2.5 text-white focus:border-blue-500 focus:outline-none transition-colors font-mono disabled:cursor-not-allowed"
                                    />
                                </div>
                                <!-- Max RAM -->
                                <div class="space-y-2">
                                    <label
                                        for="max-ram"
                                        class="text-xs font-bold text-zinc-400 uppercase"
                                        >Máxima (MB) (-Xmx)</label
                                    >
                                    <input
                                        id="max-ram"
                                        type="number"
                                        bind:value={formSettings.max_ram}
                                        disabled={isServerRunning}
                                        class="w-full bg-[#0f1520] border border-white/10 rounded-lg px-4 py-2.5 text-white focus:border-blue-500 focus:outline-none transition-colors font-mono disabled:cursor-not-allowed"
                                    />
                                </div>
                            </div>

                            <!-- Slider for Max RAM -->
                            <div class="space-y-2">
                                <div
                                    class="flex justify-between text-xs text-zinc-500"
                                >
                                    <span>1 GB</span>
                                    <span
                                        >{(
                                            (systemRam / 1073741824) *
                                            0.75
                                        ).toFixed(1)} GB (Recomendado)</span
                                    >
                                    <span
                                        >{(systemRam / 1073741824).toFixed(1)} GB</span
                                    >
                                </div>
                                <input
                                    type="range"
                                    min="1024"
                                    max={systemRam / 1048576}
                                    step="512"
                                    bind:value={formSettings.max_ram}
                                    disabled={isServerRunning}
                                    class="w-full accent-blue-500 h-2 bg-zinc-700 rounded-lg appearance-none cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                                />
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
                                onclick={resetGeneralSettings}
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
</div>
