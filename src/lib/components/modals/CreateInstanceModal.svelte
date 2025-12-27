<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";

    import IconPicker from "./IconPicker.svelte";

    import iconList from "$lib/data/icons.json";

    let instanceName = $state("");
    let gameVersion = $state("");
    let activeTab = $state<"custom" | "file" | "import">("custom");

    let selectedLoader = $state<
        | "Vanilla"
        | "Paper"
        | "Purpur"
        | "Folia"
        | "Velocity"
        | "Waterfall"
        | null
    >("Vanilla");
    let hoveredIcon = $state(false);

    // Custom URL State
    let useCustomUrl = $state(false);
    let customUrl = $state("");

    // Version State
    let versions = $state<string[]>([]);
    let showSnapshots = $state(false);
    let loadingVersions = $state(false);
    let showVersionDropdown = $state(false);

    // UI Layout State
    let showLoaderDropdown = $state(false);
    let dropdownBottom = $state(0);
    let dropdownLeft = $state(0);
    let dropdownWidth = $state(0);
    let dropdownMaxHeight = 260;

    // Icon Selection State
    let showIconPicker = $state(false);

    // Pick random icon on init
    let selectedIcon = $state(
        `/Transparent-Images/${iconList[Math.floor(Math.random() * iconList.length)]}`,
    );

    let installing = $state(false);
    let installProgress = $state(0);
    let installStep = $state("Iniciando...");

    function close() {
        if (installing) return;
        appState.creatingInstance = false;
    }

    function handleIconSelect(icon: string) {
        selectedIcon = icon;
        showIconPicker = false;
    }

    async function handleCreate() {
        if (!instanceName.trim()) return;

        installing = true;
        installProgress = 0;
        installStep = "Preparando...";

        let unlisten: () => void;
        let createdId: string | null = null;

        try {
            unlisten = await listen<any>("install-progress", (event) => {
                const payload = event.payload;
                if (payload.step.startsWith("Downloading")) {
                    installStep = "Descargando servidor...";
                    // Formatter helper
                    const formatSize = (bytes: number) =>
                        (bytes / 1024 / 1024).toFixed(1);

                    if (payload.total_size) {
                        installStep = `Descargando... ${formatSize(payload.downloaded)} MB de ${formatSize(payload.total_size)} MB`;
                        installProgress = payload.progress;
                    } else {
                        // If we are in "unknown size" mode or heuristic
                        installStep = `Descargando... ${formatSize(payload.downloaded)} MB de NULL`;
                        installProgress = payload.progress;
                    }
                } else if (payload.step === "Done") {
                    finishInstallation(payload.id);
                } else if (payload.step.startsWith("Error")) {
                    installStep = "Error";
                    installing = false;
                    alert("Error en la instalación: " + payload.step);
                    if (unlisten) unlisten();
                } else {
                    // Update any other step (Creating, Resolving, Connecting, etc)
                    installStep = payload.step;
                    if (payload.progress > 0)
                        installProgress = payload.progress;
                }
            });

            const id = await invoke<string>("create_instance", {
                name: instanceName.trim(),
                loader: selectedLoader,
                version: gameVersion,
                icon: selectedIcon,
                customDownloadUrl: useCustomUrl && customUrl ? customUrl : null,
            });
            createdId = id; // Store it for the event listener
            console.log("Instance created, installing:", id);
        } catch (error) {
            console.error("Failed to create instance:", error);
            alert("Error al crear la instancia: " + error);
            installing = false;
            if (unlisten!) unlisten();
        }
    }

    async function finishInstallation(targetId?: string | null) {
        const instances = await invoke<any[]>("read_instances");
        appState.instances = instances;

        if (targetId) {
            const found = instances.find((i) => i.id === targetId);
            if (found) {
                appState.selectedInstance = found;
                appState.view = "instances"; // Ensure we are on the instances view context
            }
        }

        installing = false;
        close();
    }

    async function loadVersions() {
        if (!selectedLoader) return;
        loadingVersions = true;
        versions = []; // Clear previous

        try {
            if (selectedLoader === "Vanilla") {
                versions = await invoke("get_minecraft_versions", {
                    snapshots: showSnapshots,
                });
            } else {
                // Custom API Loaders
                try {
                    versions = await invoke("get_project_versions", {
                        project: selectedLoader.toLowerCase(),
                    });
                } catch (e) {
                    console.error(
                        `Failed to load versions for ${selectedLoader}:`,
                        e,
                    );
                    versions = [];
                }
            }

            // Auto-select first version if current is invalid
            if (
                versions.length > 0 &&
                (!gameVersion || !versions.includes(gameVersion))
            ) {
                gameVersion = versions[0];
            }
        } catch (error) {
            console.error("Failed to fetch versions:", error);
        } finally {
            loadingVersions = false;
        }
    }

    $effect(() => {
        // Trigger load options when loader/snapshots changes
        if (selectedLoader) {
            loadVersions();
        }
    });

    $effect(() => {
        if (selectedLoader === "Vanilla") {
            loadVersions();
        }
    });
</script>

<!-- Window Resize Handler to prevent floating dropdowns -->
<svelte:window
    onresize={() => {
        showLoaderDropdown = false;
        showVersionDropdown = false;
    }}
/>

<!-- Backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-fade-in p-4"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => {
        if (e.target === e.currentTarget) close();
    }}
>
    <!-- Modal Container -->
    <div
        class="bg-[#18181b] w-full max-w-[420px] rounded-xl border border-zinc-800 shadow-2xl flex flex-col max-h-[90vh] animate-scale-in"
        tabindex="-1"
    >
        <!-- Header -->
        <div
            class="flex items-center justify-between px-6 py-4 border-b border-zinc-800/50"
        >
            <h2 class="text-lg font-bold text-white">Crear nueva instancia</h2>
            <button
                onclick={close}
                class="text-zinc-400 hover:text-white transition-colors"
                aria-label="Cerrar"
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
                    ><line x1="18" y1="6" x2="6" y2="18"></line><line
                        x1="6"
                        y1="6"
                        x2="18"
                        y2="18"
                    ></line></svg
                >
            </button>
        </div>

        <!-- Body -->
        <div class="p-6 space-y-6">
            <!-- Tabs -->
            <div class="bg-black/20 p-1 rounded-lg flex gap-1">
                {#each ["custom", "file", "import"] as tab}
                    <button
                        class="flex-1 py-1.5 px-3 rounded-md text-sm font-medium transition-all {activeTab ===
                        tab
                            ? 'bg-zinc-700 text-white shadow-sm'
                            : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/5'}"
                        onclick={() => (activeTab = tab as any)}
                    >
                        {tab === "custom"
                            ? "Custom"
                            : tab === "file"
                              ? "From File"
                              : "Import"}
                    </button>
                {/each}
            </div>

            <!-- Group 1: Icon + Name -->
            <div class="flex gap-4">
                <!-- Icon Picker -->
                <button
                    type="button"
                    class="w-24 h-24 flex-shrink-0 rounded-2xl bg-zinc-800 border-2 border-dashed border-zinc-700 flex items-center justify-center relative overflow-hidden group cursor-pointer hover:border-zinc-500 transition-colors"
                    onmouseenter={() => (hoveredIcon = true)}
                    onmouseleave={() => (hoveredIcon = false)}
                    onclick={() => (showIconPicker = true)}
                    aria-label="Seleccionar icono"
                >
                    <img
                        src={selectedIcon}
                        alt="Icon"
                        class="w-12 h-12 transition-opacity group-hover:opacity-100 object-contain drop-shadow-md"
                    />
                    {#if hoveredIcon}
                        <div
                            class="absolute inset-0 bg-black/60 flex items-center justify-center backdrop-blur-[1px] pointer-events-none"
                        >
                            <span class="text-xs text-white font-medium"
                                >Editar</span
                            >
                        </div>
                    {/if}
                </button>

                <!-- Name Input -->
                <div class="flex-1 space-y-1.5 pt-1">
                    <label
                        class="text-xs font-bold text-white tracking-wider"
                        for="instance-name">Nombre</label
                    >
                    <input
                        id="instance-name"
                        type="text"
                        bind:value={instanceName}
                        maxlength="22"
                        autocomplete="off"
                        oninput={(e) => {
                            if (instanceName.startsWith(" ")) {
                                instanceName = instanceName.trimStart();
                            }
                        }}
                        placeholder="Mi Nuevo Servidor"
                        class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 placeholder-zinc-600 focus:outline-none focus:border-green-500/50 focus:ring-1 focus:ring-green-500/50 transition-all font-bold text-base tracking-wide"
                    />
                </div>
            </div>

            <!-- Group 2: Loader + Version -->
            <div class="grid grid-cols-2 gap-4">
                <!-- Loader Selection Dropdown -->
                <div class="space-y-1.5 relative">
                    <span class="text-xs font-bold text-white tracking-wider"
                        >Loader</span
                    >
                    <button
                        type="button"
                        class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 flex items-center justify-between focus:outline-none focus:border-green-500/50 transition-all font-medium text-left"
                        onclick={() =>
                            (showLoaderDropdown = !showLoaderDropdown)}
                    >
                        <span class="font-bold text-sm"
                            >{selectedLoader || "Seleccionar"}</span
                        >
                        <svg
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="transition-transform duration-200 {showLoaderDropdown
                                ? 'rotate-180'
                                : ''}"
                        >
                            <path d="m6 9 6 6 6-6" />
                        </svg>
                    </button>

                    <!-- Loader Dropdown Menu -->
                    {#if showLoaderDropdown}
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div
                            class="fixed inset-0 z-10"
                            onclick={() => (showLoaderDropdown = false)}
                        ></div>
                        <div
                            class="absolute z-20 bottom-full mb-1 left-0 right-0 bg-[#18181b] border border-zinc-700 rounded-lg shadow-xl overflow-hidden animate-fade-in max-h-[260px] overflow-y-auto custom-scrollbar"
                        >
                            {#each ["Vanilla", "Paper", "Purpur", "Folia", "Velocity", "Waterfall"] as loader}
                                <button
                                    class="w-full text-left px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-800 hover:text-white transition-colors flex items-center justify-between"
                                    class:bg-green-900_20={selectedLoader ===
                                        loader}
                                    class:text-green-400={selectedLoader ===
                                        loader}
                                    onclick={() => {
                                        selectedLoader = loader as any;
                                        showLoaderDropdown = false;
                                    }}
                                >
                                    {loader}
                                    {#if selectedLoader === loader}
                                        <svg
                                            width="14"
                                            height="14"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2"
                                            ><polyline points="20 6 9 17 4 12"
                                            ></polyline></svg
                                        >
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>

                <!-- Unified Version Selection Logic -->
                <div class="space-y-1.5 relative">
                    <span class="text-xs font-bold text-white tracking-wider"
                        >Versión del Juego</span
                    >

                    <button
                        id="game-version"
                        type="button"
                        class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 flex items-center justify-between focus:outline-none focus:border-green-500/50 transition-all font-medium disabled:opacity-50"
                        disabled={loadingVersions}
                        onclick={(e) => {
                            const rect =
                                e.currentTarget.getBoundingClientRect();
                            dropdownBottom = window.innerHeight - rect.top + 5;
                            dropdownLeft = rect.left;
                            dropdownWidth = rect.width;
                            showVersionDropdown = !showVersionDropdown;
                        }}
                    >
                        <span class="truncate font-bold text-sm">
                            {#if loadingVersions && versions.length === 0}
                                ...
                            {:else}
                                {gameVersion}
                            {/if}
                        </span>

                        <div class="text-zinc-500">
                            {#if loadingVersions}
                                <span class="animate-spin block">↻</span>
                            {:else}
                                <svg
                                    width="16"
                                    height="16"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    class="transition-transform duration-200 {showVersionDropdown
                                        ? 'rotate-180'
                                        : ''}"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                            {/if}
                        </div>
                    </button>

                    {#if selectedLoader === "Vanilla"}
                        <div class="flex items-center gap-2 mt-2">
                            <input
                                type="checkbox"
                                id="snapshots"
                                bind:checked={showSnapshots}
                                class="rounded border-zinc-700 bg-zinc-900 text-green-500 focus:ring-0 focus:ring-offset-0 w-3 h-3"
                            />
                            <label
                                for="snapshots"
                                class="text-xs text-zinc-500 select-none cursor-pointer"
                                >Mostrar Snapshots</label
                            >
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        <!-- Footer -->
        <div class="p-6 pt-2 flex justify-end gap-3 min-h-[60px] shrink-0">
            {#if installing}
                <div class="w-full flex flex-col gap-2 justify-center">
                    <div
                        class="flex justify-between text-xs text-zinc-400 font-medium"
                    >
                        <span>{installStep}</span>
                        <span>{installProgress}%</span>
                    </div>
                    <div
                        class="w-full h-2 bg-zinc-800 rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full bg-green-500 transition-all duration-300 ease-out"
                            style="width: {installProgress}%"
                        ></div>
                    </div>
                </div>
            {:else}
                <button
                    onclick={close}
                    class="px-4 py-2 rounded-lg text-sm font-bold text-zinc-400 hover:text-white hover:bg-zinc-800/50 transition-colors"
                >
                    Cancelar
                </button>
                <button
                    onclick={handleCreate}
                    class="px-6 py-2 rounded-lg text-sm font-bold bg-green-600 hover:bg-green-500 text-white shadow-lg shadow-green-900/20 transition-all active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled={!instanceName.trim() ||
                        !selectedLoader ||
                        (useCustomUrl && !customUrl)}
                >
                    Crear Instancia
                </button>
            {/if}
        </div>
    </div>
</div>

{#if showIconPicker}
    <IconPicker
        onselect={handleIconSelect}
        onclose={() => (showIconPicker = false)}
    />
{/if}

{#if showVersionDropdown}
    <!-- Dropdown Portal (Fixed Position) -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="fixed inset-0 z-[60]"
        onclick={() => (showVersionDropdown = false)}
    ></div>

    <div
        class="fixed z-[70] bg-[#18181b] border border-zinc-700 rounded-lg shadow-xl overflow-hidden animate-fade-in flex flex-col"
        style="bottom: {dropdownBottom}px; left: {dropdownLeft}px; width: {dropdownWidth}px; max-height: {dropdownMaxHeight}px;"
    >
        <div class="overflow-y-auto custom-scrollbar flex-1 py-1">
            <!-- Determine list based on loader -->
            {#each versions as v}
                <button
                    class="w-full text-left px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-800 hover:text-white transition-colors flex items-center justify-between"
                    class:bg-green-900_20={gameVersion === v}
                    class:text-green-400={gameVersion === v}
                    onclick={() => {
                        gameVersion = v;
                        showVersionDropdown = false;
                    }}
                >
                    {v}
                    {#if gameVersion === v}
                        <svg
                            width="14"
                            height="14"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><polyline points="20 6 9 17 4 12"></polyline></svg
                        >
                    {/if}
                </button>
            {/each}
        </div>
    </div>
{/if}

<style>
    /* Custom Scrollbar for the dropdown */
    .custom-scrollbar::-webkit-scrollbar {
        width: 8px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: rgba(0, 0, 0, 0.2);
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #3f3f46;
        border-radius: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #52525b;
    }

    @keyframes scaleIn {
        from {
            opacity: 0;
            transform: scale(0.95) translateY(10px);
        }
        to {
            opacity: 1;
            transform: scale(1) translateY(0);
        }
    }
    .animate-scale-in {
        animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }
    .animate-fade-in {
        animation: fadeIn 0.2s ease-out;
    }
    @keyframes fadeIn {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }
</style>
