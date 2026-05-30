<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { open } from "@tauri-apps/plugin-dialog";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { toast } from "$lib/runes/toast.svelte";
    import { _, getLocaleFromNavigator } from "svelte-i18n";
    import { get } from "svelte/store";

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
        | "NeoForge"
        | null
    >("Vanilla");

    let acceptEula = $state(true);
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
    let installStep = $state(get(_)("create_instance.status_starting"));

    // File/Import specific
    let sourcePath = $state("");
    let isDetectingVersion = $state(false);

    // ── Modpack Import (ZIP) ───────────────────────────────────────────────
    let modpackZipPath = $state("");
    let showCfBlockedModal = $state(false);
    let cfProfileCode = $state("");

    // ── NeoForge specific ─────────────────────────────────────────────────
    let neoforgeVersions = $state<string[]>([]);
    let loadingNeoforge = $state(false);
    let showNeoforgeVersionDropdown = $state(false);


    async function selectSource() {
        if (activeTab === "file") {
            const selected = await open({
                multiple: false,
                filters: [{ name: "JAR", extensions: ["jar"] }],
            });
            if (selected && typeof selected === "string") {
                sourcePath = selected;
                detectVersion(selected);
            }
        } else if (activeTab === "import") {
            const selected = await open({
                multiple: false,
                directory: true,
            });
            if (selected && typeof selected === "string") {
                sourcePath = selected;
                detectVersion(selected);
            }
        }
    }

    async function selectModpackZip() {
        const selected = await open({
            multiple: false,
            filters: [{ name: "Modpack ZIP", extensions: ["zip"] }],
        });
        if (selected && typeof selected === "string") {
            modpackZipPath = selected;
            // Auto-fill name from filename if empty
            if (!instanceName.trim()) {
                const fname = selected.split(/[\\/]/).pop() || "";
                instanceName = fname.replace(/\.zip$/i, "").replace(/[_-]/g, " ").trim().slice(0, 30);
            }
        }
    }


    async function detectVersion(path: string) {
        isDetectingVersion = true;
        try {
            const detected = await invoke<string>("detect_minecraft_version", {
                path,
            });
            gameVersion = detected;
        } catch (e) {
            console.warn("Could not detect version automatically:", e);
        } finally {
            isDetectingVersion = false;
        }
    }

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
        installStep = get(_)("create_instance.status_preparing");

        let unlisten: (() => void) | null = null;
        let createdId: string | null = null;

        try {
            unlisten = await listen<any>("install-progress", (event) => {
                const payload = event.payload;
                if (createdId && payload.id !== createdId) return;

                if (payload.step.startsWith("Downloading") || payload.step.startsWith("Descargando")) {
                    installStep = get(_("create_instance.status_downloading"));
                    const formatSize = (bytes: number) =>
                        (bytes / 1024 / 1024).toFixed(1);

                    if (payload.total_size) {
                        installStep = get(_("create_instance.status_download_progress", {
                            values: {
                                downloaded: formatSize(payload.downloaded),
                                total: formatSize(payload.total_size)
                            }
                        }));
                        installProgress = payload.progress;
                    } else {
                        installStep = get(_("create_instance.status_download_simple", {
                            values: {
                                downloaded: formatSize(payload.downloaded)
                            }
                        }));
                        installProgress = payload.progress;
                    }
                } else if (
                    payload.step === "Done" ||
                    payload.progress === 100
                ) {
                    installStep = get(_("create_instance.status_completed"));
                    installProgress = 100;
                    setTimeout(() => finishInstallation(payload.id), 500);
                } else if (payload.step === "Creating files...") {
                    installStep = get(_("create_instance.status_creating_files"));
                } else if (payload.step === "Finalizing download...") {
                    installStep = get(_("create_instance.status_finalizing_download"));
                } else if (payload.step.startsWith("Ejecutando instalador") || payload.step.startsWith("Downloading NeoForge")) {
                    installStep = get(_("create_instance.status_executing_installer"));
                } else if (payload.step.startsWith("Instalación completada") || payload.step.startsWith("Installing NeoForge")) {
                    installStep = get(_("create_instance.status_installing_loader"));
                } else if (payload.step.startsWith("Extracting")) {
                    installStep = get(_("create_instance.status_extracting"));
                    installProgress = payload.progress;
                } else if (payload.step.startsWith("Copying")) {
                    installStep = get(_("create_instance.status_copying_overrides"));
                    installProgress = payload.progress;
                } else if (payload.step.startsWith("Downloaded") || payload.step.startsWith("Downloading")) {
                    installStep = get(_("create_instance.status_downloading_mods"));
                    installProgress = payload.progress;
                } else if (payload.step.startsWith("Error")) {
                    installStep = get(_("create_instance.status_error"));
                    installing = false;
                    toast.error(get(_("create_instance.alert_install_error")) + payload.step);
                    if (unlisten) unlisten();
                } else {
                    installStep = payload.step;
                    if (payload.progress > 0)
                        installProgress = payload.progress;
                }
            });

            if (activeTab === "import" && modpackZipPath) {
                // ── Modpack ZIP import flow ──
                const id = await invoke<string>("import_curseforge_zip", {
                    name: instanceName.trim(),
                    zipPath: modpackZipPath,
                    icon: selectedIcon,
                    acceptEula,
                });
                createdId = id;
            } else if (activeTab === "custom") {
                const id = await invoke<string>("create_instance", {
                    name: instanceName.trim(),
                    loader: selectedLoader,
                    version: gameVersion,
                    icon: selectedIcon,
                    customDownloadUrl:
                        useCustomUrl && customUrl ? customUrl : null,
                    acceptEula,
                });
                createdId = id;
            } else {
                // File or legacy Import (folder)
                const id = await invoke<string>("create_instance_from_path", {
                    name: instanceName.trim(),
                    icon: selectedIcon,
                    sourcePath: sourcePath,
                    isFile: activeTab === "file",
                    version: gameVersion,
                    acceptEula,
                });
                createdId = id;

                const isInstaller =
                    sourcePath.toLowerCase().endsWith(".jar") &&
                    sourcePath.toLowerCase().includes("-installer");

                if (isInstaller) {
                    // Stay in "installing" state and wait for finish via listener
                    installStep = "Iniciando instalación...";
                    installProgress = 10;
                } else {
                    // Direct completion for non-installers
                    installProgress = 100;
                    installStep = get(_)("common.status_completed");
                    setTimeout(() => finishInstallation(id), 500);
                }
            }
        } catch (error) {
            console.error("Failed to create instance:", error);
            toast.error(get(_)("create_instance.alert_create_error") + error);
            installing = false;
            if (unlisten) unlisten();
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
            } else if (selectedLoader === "NeoForge") {
                // NeoForge: first load MC versions, then the user selects MC version
                // which triggers loading NeoForge builds
                // We don't use the generic versions list for NeoForge
                loadingVersions = false;
                return;
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

    async function loadNeoforgeVersions(mcVersion: string) {
        if (!mcVersion) return;
        loadingNeoforge = true;
        neoforgeVersions = [];
        try {
            neoforgeVersions = await invoke<string[]>("get_neoforge_versions", { mcVersion, betas: showSnapshots });
            if (neoforgeVersions.length > 0) {
                gameVersion = neoforgeVersions[0]; // select latest build by default
            } else {
                gameVersion = "";
            }
        } catch (e) {
            console.error("Failed to load NeoForge versions:", e);
        } finally {
            loadingNeoforge = false;
        }
    }

    // Separate state for NeoForge MC version picker
    let nfMcVersion = $state("");
    let nfMcVersions = $state<string[]>([]);
    let loadingNfMcVersions = $state(false);
    let showNfMcDropdown = $state(false);

    async function loadNeoForgeMcVersions() {
        loadingNfMcVersions = true;
        try {
            nfMcVersions = await invoke<string[]>("get_neoforge_mc_versions");
            if (nfMcVersions.length > 0 && !nfMcVersion) {
                nfMcVersion = nfMcVersions[0];
                loadNeoforgeVersions(nfMcVersion);
            }
        } catch (e) {
            console.error("Failed to load NeoForge MC versions:", e);
        } finally {
            loadingNfMcVersions = false;
        }
    }


    $effect(() => {
        // Trigger load options when loader/snapshots changes
        if (selectedLoader) {
            if (selectedLoader === "NeoForge") {
                if (nfMcVersion) {
                    loadNeoforgeVersions(nfMcVersion);
                } else {
                    loadNeoForgeMcVersions();
                }
            } else {
                loadVersions();
            }
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
            <h2 class="text-lg font-bold text-white">{$_("create_instance.title")}</h2>
            <button
                onclick={close}
                class="text-zinc-400 hover:text-white transition-colors"
                aria-label={$_("icon_picker.aria_label_close")}
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
                        {$_(`create_instance.tab_${tab}`)}
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
                    aria-label={$_("icon_picker.aria_label_select")}
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
                                >{$_("create_instance.edit_icon")}</span
                            >
                        </div>
                    {/if}
                </button>

                <!-- Name Input -->
                <div class="flex-1 space-y-1.5 pt-1">
                    <label
                        class="text-xs font-bold text-white tracking-wider"
                        for="instance-name">{$_("create_instance.label_name")}</label
                    >
                    <input
                        id="instance-name"
                        type="text"
                        bind:value={instanceName}
                        maxlength="30"
                        autocomplete="off"
                        oninput={(e) => {
                            if (instanceName.startsWith(" ")) {
                                instanceName = instanceName.trimStart();
                            }
                        }}
                        placeholder={$_("create_instance.placeholder_name")}
                        class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 placeholder-zinc-600 focus:outline-none focus:border-green-500/50 focus:ring-1 focus:ring-green-500/50 transition-all font-bold text-base tracking-wide"
                    />
                </div>
            </div>

            <!-- Group 2: Loader + Version / File Selection -->
            <div class="space-y-4">
                {#if activeTab === "custom"}
                    <div class="grid grid-cols-2 gap-4">
                        <!-- Loader Selection Dropdown -->
                        <div class="space-y-1.5 relative">
                            <span
                                class="text-xs font-bold text-white tracking-wider"
                                >{$_("create_instance.loader")}</span
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
                                    {#each ["Vanilla", "Paper", "Purpur", "Folia", "Velocity", "Waterfall", "NeoForge"] as loader}
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
                                            <span class="flex items-center gap-2">
                                                {loader}
                                                {#if loader === "NeoForge"}
                                                    <span class="text-[9px] font-bold px-1.5 py-0.5 rounded bg-orange-500/20 text-orange-400 border border-orange-500/30">MODS</span>
                                                {/if}
                                            </span>
                                            {#if selectedLoader === loader}
                                                <svg
                                                    width="14"
                                                    height="14"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="2"
                                                    ><polyline
                                                        points="20 6 9 17 4 12"
                                                    ></polyline></svg
                                                >
                                            {/if}
                                        </button>
                                    {/each}
                                </div>
                            {/if}

                            <!-- EULA Checkbox moved here for alignment -->
                            <div class="flex items-center gap-2 mt-2">
                                <input
                                    type="checkbox"
                                    id="accept-eula-custom"
                                    bind:checked={acceptEula}
                                    class="rounded border-zinc-700 bg-zinc-900 text-green-500 focus:ring-0 focus:ring-offset-0 w-3 h-3 cursor-pointer"
                                />
                                <label
                                    for="accept-eula-custom"
                                    class="text-xs text-zinc-500 select-none cursor-pointer flex items-center gap-1.5"
                                >
                                    {$_("create_instance.accept_eula")}
                                    <a
                                        href="https://www.minecraft.net/eula"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="text-zinc-500 hover:text-green-500 transition-colors"
                                        title={$_("create_instance.accept_eula") + " (https://www.minecraft.net/eula)"}
                                        onclick={(e) => {
                                            e.preventDefault();
                                            e.stopPropagation();
                                            openUrl(
                                                "https://www.minecraft.net/eula",
                                            );
                                        }}
                                    >
                                        <svg
                                            width="12"
                                            height="12"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2.5"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            ><path
                                                d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                                            ></path><polyline
                                                points="15 3 21 3 21 9"
                                            ></polyline><line
                                                x1="10"
                                                y1="14"
                                                x2="21"
                                                y2="3"
                                            ></line></svg
                                        >
                                    </a>
                                </label>
                            </div>
                        </div>

                        <!-- Version column: NeoForge = 2 step picker, others = generic -->
                        {#if selectedLoader === "NeoForge"}
                            <!-- NeoForge: MC Version + Build selector (2 rows, spanning full grid) -->
                            <div class="col-span-2 grid grid-cols-2 gap-3 mt-1">
                                <!-- MC Version -->
                                <div class="space-y-1.5 relative">
                                    <span class="text-xs font-bold text-white tracking-wider">{$_("create_instance.neoforge_mc_version")}</span>
                                    <button
                                        type="button"
                                        class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 flex items-center justify-between focus:outline-none focus:border-orange-500/50 transition-all disabled:opacity-50"
                                        disabled={loadingNfMcVersions}
                                        onclick={() => (showNfMcDropdown = !showNfMcDropdown)}
                                        id="neoforge-mc-version"
                                    >
                                        <span class="font-bold text-sm truncate">{loadingNfMcVersions ? "..." : (nfMcVersion || "Select")}</span>
                                        {#if loadingNfMcVersions}
                                            <span class="animate-spin text-orange-400">↻</span>
                                        {:else}
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="transition-transform {showNfMcDropdown ? 'rotate-180' : ''}"><path d="m6 9 6 6 6-6"/></svg>
                                        {/if}
                                    </button>
                                    {#if showNfMcDropdown && nfMcVersions.length > 0}
                                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                                        <div class="fixed inset-0 z-10" onclick={() => (showNfMcDropdown = false)}></div>
                                        <div class="absolute z-20 bottom-full mb-1 left-0 right-0 bg-[#18181b] border border-zinc-700 rounded-lg shadow-xl overflow-y-auto custom-scrollbar animate-fade-in max-h-[200px]">
                                            {#each nfMcVersions as v}
                                                <button
                                                    class="w-full text-left px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-800 hover:text-white transition-colors {nfMcVersion === v ? 'text-orange-400' : ''}"
                                                    onclick={() => { nfMcVersion = v; showNfMcDropdown = false; loadNeoforgeVersions(v); }}
                                                >{v}</button>
                                            {/each}
                                        </div>
                                    {/if}
                                </div>
                                <!-- NeoForge Build -->
                                <div class="space-y-1.5 relative">
                                    <span class="text-xs font-bold text-white tracking-wider">{$_("create_instance.neoforge_version")}</span>
                                    <button
                                        type="button"
                                        class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 flex items-center justify-between focus:outline-none focus:border-orange-500/50 transition-all disabled:opacity-50"
                                        disabled={loadingNeoforge || !nfMcVersion}
                                        onclick={() => (showNeoforgeVersionDropdown = !showNeoforgeVersionDropdown)}
                                        id="neoforge-build-version"
                                    >
                                        <span class="font-bold text-sm truncate">{loadingNeoforge ? "..." : (gameVersion || "Select MC first")}</span>
                                        {#if loadingNeoforge}
                                            <span class="animate-spin text-orange-400">↻</span>
                                        {:else}
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="transition-transform {showNeoforgeVersionDropdown ? 'rotate-180' : ''}"><path d="m6 9 6 6 6-6"/></svg>
                                        {/if}
                                    </button>
                                    {#if showNeoforgeVersionDropdown && neoforgeVersions.length > 0}
                                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                                        <div class="fixed inset-0 z-10" onclick={() => (showNeoforgeVersionDropdown = false)}></div>
                                        <div class="absolute z-20 bottom-full mb-1 left-0 right-0 bg-[#18181b] border border-zinc-700 rounded-lg shadow-xl overflow-y-auto custom-scrollbar animate-fade-in max-h-[200px]">
                                            {#each neoforgeVersions as v}
                                                <button
                                                    class="w-full text-left px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-800 hover:text-white transition-colors {gameVersion === v ? 'text-orange-400' : ''}"
                                                    onclick={() => { gameVersion = v; showNeoforgeVersionDropdown = false; }}
                                                >{v}</button>
                                            {/each}
                                        </div>
                                    {/if}
                                </div>
                                <!-- NeoForge Betas Checkbox -->
                                <div class="col-span-2 flex items-center gap-2 mt-1">
                                    <input
                                        type="checkbox"
                                        id="nf-snapshots"
                                        bind:checked={showSnapshots}
                                        class="rounded border-zinc-700 bg-zinc-900 text-orange-500 focus:ring-0 focus:ring-offset-0 w-3 h-3"
                                    />
                                    <label
                                        for="nf-snapshots"
                                        class="text-xs text-zinc-500 select-none cursor-pointer"
                                        >{$_("create_instance.show_snapshots")}</label
                                    >
                                </div>
                            </div>
                        {:else}
                        <!-- Unified Version Selection Logic -->
                        <div class="space-y-1.5 relative">
                            <span
                                class="text-xs font-bold text-white tracking-wider"
                                >{$_("create_instance.game_version")}</span
                            >

                            <button
                                id="game-version"
                                type="button"
                                class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 flex items-center justify-between focus:outline-none focus:border-green-500/50 transition-all font-medium disabled:opacity-50"
                                disabled={loadingVersions}
                                onclick={(e) => {
                                    const rect =
                                        e.currentTarget.getBoundingClientRect();
                                    dropdownBottom =
                                        window.innerHeight - rect.top + 5;
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
                                        <span class="animate-spin block">↻</span
                                        >
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
                                        >{$_("create_instance.show_snapshots")}</label
                                    >
                                </div>
                            {/if}
                        </div>
                        {/if}
                    </div>
                {:else}
                    <!-- File / Import UI -->
                    <div class="space-y-4">
                        {#if activeTab === "import"}
                            <!-- ── Premium Modpack ZIP Importer ── -->
                            <div class="space-y-3">
                                <!-- ZIP selector card -->
                                <div class="bg-gradient-to-br from-orange-500/10 to-amber-500/5 border border-orange-500/20 rounded-xl p-4 space-y-3">
                                    <div class="flex items-center gap-2">
                                        <div class="w-8 h-8 rounded-lg bg-orange-500/20 flex items-center justify-center text-orange-400">
                                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                                        </div>
                                        <div>
                                            <p class="text-xs font-bold text-orange-300">{$_("create_instance.import_modpack_zip")}</p>
                                            <p class="text-[10px] text-zinc-500">{$_("create_instance.import_modpack_zip_desc")}</p>
                                        </div>
                                    </div>
                                    <div class="flex gap-2">
                                        <button
                                            onclick={selectModpackZip}
                                            class="flex-1 bg-black/30 border border-zinc-700 rounded-lg px-3 py-2 text-xs font-bold text-zinc-300 hover:text-white hover:border-orange-500/50 hover:bg-orange-500/5 transition-all text-left truncate"
                                            id="modpack-zip-selector"
                                        >
                                            {modpackZipPath ? modpackZipPath.split(/[\\/]/).pop() : $_("create_instance.import_modpack_btn")}
                                        </button>
                                        {#if modpackZipPath}
                                            <button
                                                onclick={() => (modpackZipPath = "")}
                                                class="p-2 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 hover:bg-red-500/20 transition-colors"
                                                aria-label="Clear"
                                            >
                                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                                            </button>
                                        {/if}
                                    </div>
                                    {#if modpackZipPath}
                                        <div class="flex items-center gap-1.5 text-[10px] text-green-400">
                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                                            ZIP seleccionado: {modpackZipPath.split(/[\\/]/).pop()}
                                        </div>
                                    {/if}
                                </div>

                                <!-- CurseForge Profile Code section (with Cloudflare warning) -->
                                <div class="border border-zinc-800 rounded-xl p-3 space-y-2">
                                    <p class="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">{$_("create_instance.import_modpack_cf_code_title")}</p>
                                    <div class="flex gap-2">
                                        <input
                                            type="text"
                                            bind:value={cfProfileCode}
                                            placeholder={$_("create_instance.import_modpack_cf_code_placeholder")}
                                            class="flex-1 bg-black/20 border border-zinc-700 rounded-lg px-3 py-2 text-xs text-zinc-300 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 transition-all font-mono"
                                            id="cf-profile-code"
                                        />
                                        <button
                                            onclick={() => { if (cfProfileCode.trim()) showCfBlockedModal = true; }}
                                            disabled={!cfProfileCode.trim()}
                                            class="px-3 py-2 text-xs font-bold bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg transition-all disabled:opacity-40"
                                        >Import</button>
                                    </div>
                                </div>
                            </div>
                        {:else}
                        <!-- File Tab (JAR) -->
                        <div class="space-y-1.5">
                            <span
                                class="text-xs font-bold text-white tracking-wider"
                                >{$_("create_instance.select_jar")}</span
                            >
                            <div class="flex gap-2">
                                <button
                                    onclick={selectSource}
                                    class="flex-1 bg-black/20 border border-zinc-700 rounded-lg px-4 py-2.5 text-sm font-bold text-zinc-300 hover:text-white hover:bg-white/5 transition-all text-left truncate overflow-hidden"
                                >
                                    {sourcePath
                                        ? sourcePath
                                        : $_("create_instance.click_to_browse")}
                                </button>
                                {#if sourcePath}
                                    <button
                                        onclick={() => (sourcePath = "")}
                                        aria-label={$_("icon_picker.aria_label_close")}
                                        class="p-2.5 bg-red-500/10 border border-red-500/20 rounded-lg text-red-500 hover:bg-red-500/20 transition-colors"
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
                                            ><line x1="18" y1="6" x2="6" y2="18"
                                            ></line><line
                                                x1="6"
                                                y1="6"
                                                x2="18"
                                                y2="18"
                                            ></line></svg
                                        >
                                    </button>
                                {/if}
                            </div>
                        </div>

                        <div class="space-y-1.5">
                            <span
                                class="text-xs font-bold text-white tracking-wider"
                                >{$_("create_instance.detected_version")}</span
                            >
                            <div class="relative">
                                <input
                                    type="text"
                                    bind:value={gameVersion}
                                    placeholder="Ej: 1.20.1"
                                    class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-zinc-300 focus:outline-none focus:border-green-500/50 transition-all font-bold text-sm tracking-wide"
                                />
                                {#if isDetectingVersion}
                                    <div
                                        class="absolute right-3 top-1/2 -translate-y-1/2"
                                    >
                                        <span
                                            class="animate-spin block text-zinc-500"
                                            >↻</span
                                        >
                                    </div>
                                {/if}
                            </div>
                            <p class="text-[10px] text-zinc-500 italic">
                                {$_("create_instance.detection_tip")}
                            </p>
                        </div>
                        {/if}

                        <!-- EULA Checkbox for File/Import Tab -->
                        <div class="flex items-center gap-2 mt-2">
                            <input
                                type="checkbox"
                                id="accept-eula-file"
                                bind:checked={acceptEula}
                                class="rounded border-zinc-700 bg-zinc-900 text-green-500 focus:ring-0 focus:ring-offset-0 w-3 h-3 cursor-pointer"
                            />
                            <label
                                for="accept-eula-file"
                                class="text-xs text-zinc-500 select-none cursor-pointer flex items-center gap-1.5"
                            >
                                {$_("create_instance.accept_eula")}
                                <a
                                    href="https://www.minecraft.net/eula"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="text-zinc-500 hover:text-green-500 transition-colors"
                                    title={$_("create_instance.accept_eula") + " (https://www.minecraft.net/eula)"}
                                    onclick={(e) => {
                                        e.preventDefault();
                                        e.stopPropagation();
                                        openUrl(
                                            "https://www.minecraft.net/eula",
                                        );
                                    }}
                                >
                                    <svg
                                        width="12"
                                        height="12"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2.5"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        ><path
                                            d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                                        ></path><polyline
                                            points="15 3 21 3 21 9"
                                        ></polyline><line
                                            x1="10"
                                            y1="14"
                                            x2="21"
                                            y2="3"
                                        ></line></svg
                                    >
                                </a>
                            </label>
                        </div>
                    </div>
                {/if}
            </div>
        </div>

        <!-- Footer -->
        <div class="p-6 pt-2 flex justify-end gap-3 min-h-[60px] shrink-0">
            {#if installing}
                <div class="w-full flex flex-col gap-2 justify-center">
                    <div
                        class="flex justify-between items-end gap-4 text-xs text-zinc-400 font-medium mb-1"
                    >
                        <div class="flex flex-col gap-1 overflow-hidden">
                            <span
                                class="text-[10px] uppercase tracking-wider text-zinc-500 font-bold"
                                >{$_("create_instance.status_label")}</span
                            >
                            <span
                                class="break-all line-clamp-2 min-h-[2.5rem] leading-tight text-zinc-300 transition-all duration-200"
                            >
                                {installStep}
                            </span>
                        </div>
                        <span class="shrink-0 text-lg font-mono text-green-500"
                            >{installProgress}%</span
                        >
                    </div>
                    <div
                        class="w-full h-2 bg-zinc-800 rounded-full overflow-hidden shadow-inner shadow-black/40"
                    >
                        <div
                            class="h-full bg-gradient-to-r from-green-600 to-green-400 transition-all duration-300 ease-out relative"
                            style="width: {installProgress}%"
                        >
                            <div
                                class="absolute inset-0 bg-[linear-gradient(45deg,rgba(255,255,255,0.1)_25%,transparent_25%,transparent_50%,rgba(255,255,255,0.1)_50%,rgba(255,255,255,0.1)_75%,transparent_75%,transparent)] bg-[length:20px_20px] animate-shimmer"
                            ></div>
                        </div>
                    </div>
                </div>
            {:else}
                <button
                    onclick={close}
                    class="px-4 py-2 rounded-lg text-sm font-bold text-zinc-400 hover:text-white hover:bg-zinc-800/50 transition-colors"
                >
                    {$_("create_instance.btn_cancel")}
                </button>
                <button
                    onclick={handleCreate}
                    class="px-6 py-2 rounded-lg text-sm font-bold bg-green-600 hover:bg-green-500 text-white shadow-lg shadow-green-900/20 transition-all active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled={!instanceName.trim() ||
                        (activeTab === "custom" && !selectedLoader) ||
                        (activeTab !== "custom" && !sourcePath) ||
                        (activeTab !== "custom" && !gameVersion) ||
                        (useCustomUrl && !customUrl)}
                >
                    {$_("create_instance.btn_create")}
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

    @keyframes shimmer {
        from {
            background-position: 0 0;
        }
        to {
            background-position: 40px 0;
        }
    }
    .animate-shimmer {
        animation: shimmer 1s linear infinite;
    }
</style>
