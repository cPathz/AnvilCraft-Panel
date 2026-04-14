<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { toast } from "$lib/runes/toast.svelte";
    import { onMount, tick } from "svelte";
    import { fade, slide } from "svelte/transition";
    import { listen } from "@tauri-apps/api/event";
    import { _ } from "svelte-i18n";
    import { get } from "svelte/store";

    let {
        instance,
        isServerRunning,
        isDirty = $bindable(false),
    } = $props<{
        instance: Instance;
        isServerRunning: boolean;
        isDirty?: boolean;
    }>();

    // Local State
    let formSettings = $state({
        min_ram: 1024,
        max_ram: 2048,
        port: 25565,
        args: "",
        jar_file: "server.jar",
        java_path: null as string | null,
    });

    const formatRam = (mb: number) => {
        const gb = mb / 1024;
        if (gb % 1 === 0) return `${gb} GB`;
        return `${gb.toFixed(1)} GB`;
    };

    let originalSettings = $state<any>(null);
    let systemRam = $state(0);
    let linkMemory = $state(true);
    let lastSyncedId = $state<string | null>(null);

    // Java Downloader State
    let javaRuntimes = $state<any[]>([]);
    let downloadProgress = $state<
        Record<string, { step: string; progress: number }>
    >({});
    let unlistenProgress: any = null;

    // derived max allowed RAM (75% of system RAM, at least 512 MB)
    let maxAllowed = $derived.by(() => {
        if (!systemRam) return 512;
        const totalMb = Math.floor(systemRam / 1024 / 1024);
        return Math.max(512, Math.floor(totalMb * 0.75));
    });

    // Helper to mark component dirty
    function markDirty() {
        isDirty = true;
    }

    let internalIsDirty = $derived(
        originalSettings &&
            JSON.stringify(formSettings) !== JSON.stringify(originalSettings),
    );

    // Sync internal dirty state to parent prop
    $effect(() => {
        isDirty = internalIsDirty;
    });

    // Initialization
    onMount(async () => {
        try {
            systemRam = await invoke<number>("get_system_memory");
        } catch (e) {
            console.error("Failed to get system memory", e);
        }
        await loadJavaRuntimes();

        // Listen for Java download progress
        unlistenProgress = await listen<any>("install-progress", (event) => {
            const payload = event.payload;
            if (payload.id.startsWith("java-download-")) {
                let step = payload.step;
                
                // Map backend steps to translations
                if (step.startsWith("Downloading")) {
                    step = get(_)("common.status_downloading");
                } else if (step === "Done" || payload.progress === 100) {
                    step = get(_)("common.status_completed");
                } else if (step === "Preparing") {
                    step = get(_)("common.status_preparing");
                } else if (step === "Creating files...") {
                    step = get(_)("create_instance.status_creating_files");
                } else if (step === "Finalizing download...") {
                    step = get(_)("create_instance.status_finalizing_download");
                }

                downloadProgress[payload.id] = {
                    step,
                    progress: payload.progress,
                };
                if (payload.step === "Done") {
                    loadJavaRuntimes();
                    setTimeout(() => {
                        delete downloadProgress[payload.id];
                    }, 3000);
                }
            }
        });

        return () => {
            if (unlistenProgress) unlistenProgress();
        };
    });

    async function loadJavaRuntimes() {
        try {
            javaRuntimes = await invoke("get_available_java_versions");
        } catch (e) {
            console.error("Failed to load java runtimes", e);
        }
    }

    async function handleDownloadJava(version: number) {
        const id = `java-download-${version}`;
        downloadProgress[id] = { step: get(_)("common.status_starting"), progress: 0 };
        try {
            const path = await invoke<string>("download_java_runtime", {
                version,
            });
            formSettings.java_path = path;
            markDirty();
            toast.success(get(_)("instance_settings.toast_java_downloaded", { values: { version } }));
        } catch (e) {
            console.error(e);
            toast.error(get(_)("instance_settings.toast_java_error", { values: { version } }) + e);
            delete downloadProgress[id];
        }
    }

    function useJavaRuntime(path: string) {
        formSettings.java_path = path;
        markDirty();
        toast.success(get(_)("instance_settings.toast_java_path"));
    }

    // Sync form when instance changes
    $effect(() => {
        if (instance && instance.settings) {
            if (lastSyncedId !== instance.id) {
                formSettings = { ...instance.settings };
                originalSettings = { ...instance.settings };
                lastSyncedId = instance.id;
            }
        }
    });

    // RAM input handlers to enforce constraints
    function toggleLink() {
        linkMemory = !linkMemory;
        if (linkMemory) {
            formSettings.min_ram = formSettings.max_ram;
        }
        markDirty();
    }

    function handleMinInput() {
        markDirty();
        if (linkMemory) {
            formSettings.max_ram = formSettings.min_ram;
        } else if (formSettings.min_ram > formSettings.max_ram) {
            formSettings.max_ram = formSettings.min_ram;
        }
    }

    function handleMaxInput() {
        markDirty();
        if (linkMemory) {
            formSettings.min_ram = formSettings.max_ram;
        } else if (formSettings.max_ram < formSettings.min_ram) {
            formSettings.min_ram = formSettings.max_ram;
        }
    }

    // Actions
    async function saveSettings() {
        try {
            await invoke("save_instance_settings", {
                instanceId: instance.id,
                settings: formSettings,
            });

            // Update Global State
            if (
                appState.selectedInstance &&
                appState.selectedInstance.id === instance.id
            ) {
                appState.selectedInstance.settings = { ...formSettings };
            }
            originalSettings = { ...formSettings };
            toast.success(get(_)("instance_settings.toast_save_success"));
        } catch (e) {
            console.error(e);
            toast.error(get(_)("instance_settings.toast_save_error") + e);
        }
    }

    function discardChanges() {
        if (originalSettings) {
            formSettings = { ...originalSettings };
        }
    }

    function resetGeneralSettingsForm() {
        formSettings.port = 25565;
        formSettings.jar_file = "server.jar";
        formSettings.args = "";
        formSettings.java_path = null;

        const totalMb = Math.floor(systemRam / 1024 / 1024);
        // Step to 512MB increments
        if (totalMb >= 8192) {
            formSettings.min_ram = 4096;
            formSettings.max_ram = 4096;
        } else {
            formSettings.min_ram = 2048;
            formSettings.max_ram = 2048;
        }
        markDirty();
    }

    async function selectJavaPath() {
        try {
            const selected = await open({
                multiple: false,
                filters: [
                    {
                        name: "Java Executable",
                        extensions: ["exe", "bin", "sh", "*"],
                    },
                ],
            });
            if (selected && typeof selected === "string") {
                formSettings.java_path = selected;
                markDirty();
            }
        } catch (e) {
            console.error(e);
            toast.error(get(_)("instance_settings.toast_picker_error") + e);
        }
    }

    function formatBytes(bytes: number, decimals = 2) {
        if (bytes === 0) return "0 Bytes";
        const k = 1024;
        const dm = decimals < 0 ? 0 : decimals;
        const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return (
            parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i]
        );
    }
</script>

<div class="flex-1 flex flex-col min-h-0">
    <!-- Action Bar (Save/Discard) -->
    {#if isDirty}
        <div
            class="bg-yellow-500/10 border-b border-yellow-500/20 px-8 py-3 flex items-center justify-between"
            transition:fade
        >
            <span
                class="text-yellow-500 text-sm font-bold flex items-center gap-2"
            >
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    ><path
                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    /></svg
                >
                {$_("instance_settings.unsaved_title")}
            </span>
            <div class="flex gap-2">
                <button
                    onclick={discardChanges}
                    class="px-3 py-1.5 rounded-lg text-xs font-bold text-zinc-400 hover:text-white hover:bg-white/10 transition-colors"
                >
                    {$_("instance_settings.btn_discard")}
                </button>
                <button
                    onclick={saveSettings}
                    class="px-4 py-1.5 rounded-lg text-xs font-bold text-[#0f172a] bg-yellow-400 hover:bg-yellow-300 transition-colors shadow-lg shadow-yellow-400/20"
                >
                    {$_("instance_settings.btn_save")}
                </button>
            </div>
        </div>
    {/if}

    <!-- Scrollable Content -->
    <div class="flex-1 p-8 overflow-y-auto custom-scrollbar">
        <div class="max-w-3xl space-y-8">
            <!-- Warning Banner if Running -->
            {#if isServerRunning}
                <div
                    class="mb-6 bg-yellow-500/10 border border-yellow-500/20 rounded-xl p-4 flex items-center gap-3"
                >
                    <div
                        class="w-10 h-10 rounded-full bg-yellow-500/20 flex items-center justify-center shrink-0"
                    >
                        <svg
                            width="20"
                            height="20"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="#eab308"
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
                        <h3 class="font-bold text-yellow-500">
                            {$_("instance_settings.running_title")}
                        </h3>
                        <p class="text-sm text-yellow-200/70">
                            {$_("instance_settings.running_desc")}
                        </p>
                    </div>
                </div>
            {/if}

            <div class="space-y-8">
                <div class="grid grid-cols-2 gap-16">
                    <!-- Column 1: RAM -->
                    <div class="space-y-4">
                        <div
                            class="grid grid-cols-[1fr,96px] gap-x-4 items-center gap-y-1"
                        >
                            <!-- Header Min -->
                            <label
                                for="min-ram"
                                class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                                >{$_("instance_settings.min_ram")}</label
                            >
                            <div></div>

                            <!-- Row Min -->
                            <input
                                id="min-ram"
                                type="range"
                                min="512"
                                step="512"
                                max={maxAllowed}
                                bind:value={formSettings.min_ram}
                                oninput={handleMinInput}
                                disabled={isServerRunning}
                                class="w-full accent-blue-500 h-1.5 bg-zinc-700/50 rounded-lg appearance-none cursor-pointer"
                            />
                            <div
                                class="bg-[#1e293b] px-3 py-2 rounded-xl text-center text-sm font-jetbrains font-bold text-zinc-400 shadow-inner"
                            >
                                {formatRam(formSettings.min_ram)}
                            </div>

                            <!-- Link Row -->
                            <div></div>
                            <div class="flex justify-center -my-1">
                                <button
                                    onclick={toggleLink}
                                    class="p-1.5 rounded-full hover:bg-white/5 transition-colors group"
                                    title={linkMemory
                                        ? $_("instance_settings.tooltip_unlink")
                                        : $_("instance_settings.tooltip_link")}
                                >
                                    <svg
                                        width="18"
                                        height="18"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2.5"
                                        class={linkMemory
                                            ? "text-blue-500"
                                            : "text-zinc-600 opacity-50"}
                                    >
                                        <path
                                            d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
                                        ></path>
                                        {#if linkMemory}
                                            <path
                                                d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
                                            ></path>
                                        {/if}
                                    </svg>
                                </button>
                            </div>

                            <!-- Header Max -->
                            <label
                                for="max-ram"
                                class="text-xs font-bold text-zinc-400 uppercase tracking-wider mt-2"
                                >{$_("instance_settings.max_ram")}</label
                            >
                            <div></div>

                            <!-- Row Max -->
                            <input
                                id="max-ram"
                                type="range"
                                min="512"
                                step="512"
                                max={maxAllowed}
                                bind:value={formSettings.max_ram}
                                oninput={handleMaxInput}
                                disabled={isServerRunning}
                                class="w-full accent-blue-500 h-1.5 bg-zinc-700/50 rounded-lg appearance-none cursor-pointer"
                            />
                            <div
                                class="bg-[#1e293b] px-3 py-2 rounded-xl text-center text-sm font-jetbrains font-bold text-zinc-300 shadow-inner"
                            >
                                {formatRam(formSettings.max_ram)}
                            </div>

                            <!-- Stats -->
                            <div
                                class="col-span-2 flex justify-between text-xs text-zinc-500 mt-4 pr-1"
                            >
                                <span>{$_("instance_settings.system_ram")}{formatBytes(systemRam)}</span>
                                <span
                                    class={formSettings.max_ram >
                                    (systemRam / 1024 / 1024) * 0.8
                                        ? "text-yellow-500"
                                        : ""}
                                >
                                    {Math.round(
                                        (formSettings.max_ram /
                                            (systemRam / 1024 / 1024)) *
                                            100,
                                    )}{$_("instance_settings.of_total")}
                                </span>
                            </div>
                        </div>
                    </div>
                    <!-- Column 2: JAR & Port -->
                    <div class="space-y-6">
                        <!-- JAR -->
                        <div class="space-y-2">
                            <label
                                for="jar-file"
                                class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                                >{$_("instance_settings.jar_file")}</label
                            >
                            <div class="relative group">
                                <div
                                    class="absolute left-4 top-1/2 -translate-y-1/2 text-zinc-600"
                                >
                                    <svg
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        ><path
                                            d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
                                        ></path><path
                                            d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
                                        ></path></svg
                                    >
                                </div>
                                <input
                                    id="jar-file"
                                    type="text"
                                    bind:value={formSettings.jar_file}
                                    oninput={markDirty}
                                    disabled={isServerRunning}
                                    class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl pl-12 pr-4 py-3 font-jetbrains text-sm focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300 shadow-inner"
                                />
                            </div>
                        </div>

                        <!-- Port -->
                        <div class="space-y-2">
                            <label
                                for="server-port"
                                class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                                >{$_("instance_settings.port")}</label
                            >
                            <div class="relative group">
                                <div
                                    class="absolute left-4 top-1/2 -translate-y-1/2 text-zinc-600"
                                >
                                    <svg
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        ><line x1="4" y1="9" x2="20" y2="9"
                                        ></line><line
                                            x1="4"
                                            y1="15"
                                            x2="20"
                                            y2="15"
                                        ></line><line
                                            x1="10"
                                            y1="3"
                                            x2="8"
                                            y2="21"
                                        ></line><line
                                            x1="16"
                                            y1="3"
                                            x2="14"
                                            y2="21"
                                        ></line></svg
                                    >
                                </div>
                                <input
                                    id="server-port"
                                    type="number"
                                    bind:value={formSettings.port}
                                    oninput={markDirty}
                                    disabled={isServerRunning}
                                    class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl pl-12 pr-4 py-3 font-jetbrains text-sm focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300 shadow-inner"
                                />
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Java Selection -->
                <div class="space-y-3">
                    <label
                        for="java-path"
                        class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                        >{$_("instance_settings.java_path")}</label
                    >
                    <div class="flex gap-2">
                        <div class="relative flex-1 group">
                            <div
                                class="absolute left-4 top-1/2 -translate-y-1/2 text-zinc-600"
                            >
                                <svg
                                    width="16"
                                    height="16"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    ><path
                                        d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
                                    /><polyline
                                        points="3.27 6.96 12 12.01 20.73 6.96"
                                    /><line
                                        x1="12"
                                        y1="22.08"
                                        x2="12"
                                        y2="12"
                                    /></svg
                                >
                            </div>
                            <input
                                id="java-path"
                                type="text"
                                bind:value={formSettings.java_path}
                                oninput={markDirty}
                                disabled={isServerRunning}
                                placeholder={$_("instance_settings.java_default")}
                                class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl pl-12 pr-4 py-3 font-jetbrains text-sm focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300 placeholder:text-zinc-700 shadow-inner"
                            />
                        </div>
                        <button
                            onclick={selectJavaPath}
                            disabled={isServerRunning}
                            class="px-4 bg-zinc-800 border border-zinc-700 rounded-xl text-zinc-300 hover:text-white hover:bg-zinc-700 transition-all disabled:opacity-30 flex items-center gap-2"
                        >
                            <svg
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                ><path
                                    d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                                /></svg
                            >
                            <span
                                class="text-xs font-bold uppercase tracking-widest"
                                >{$_("instance_settings.btn_browse")}</span
                            >
                        </button>
                    </div>
                    {#if instance.version.includes("1.12") || instance.version.includes("1.8")}
                        <p
                            class="text-[10px] text-yellow-500/70 italic flex items-center gap-1.5 px-1"
                        >
                            <svg
                                width="12"
                                height="12"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="3"
                                ><circle cx="12" cy="12" r="10" /><line
                                    x1="12"
                                    y1="8"
                                    x2="12"
                                    y2="12"
                                /><line
                                    x1="12"
                                    y1="16"
                                    x2="12.01"
                                    y2="16"
                                /></svg
                            >
                            {$_("instance_settings.tip_java8")}
                        </p>
                    {:else if instance.version.includes("1.16.5")}
                        <p
                            class="text-[10px] text-yellow-500/70 italic flex items-center gap-1.5 px-1"
                        >
                            <svg
                                width="12"
                                height="12"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="3"
                                ><circle cx="12" cy="12" r="10" /><line
                                    x1="12"
                                    y1="8"
                                    x2="12"
                                    y2="12"
                                /><line
                                    x1="12"
                                    y1="16"
                                    x2="12.01"
                                    y2="16"
                                /></svg
                            >
                            {$_("instance_settings.tip_java11")}
                        </p>
                    {/if}
                </div>

                <!-- Portable Java Downloader -->
                <div
                    class="bg-[#1e293b]/30 border border-[#1e293b] rounded-2xl overflow-hidden"
                >
                    <div
                        class="px-6 py-4 border-b border-[#1e293b] bg-[#1e293b]/20 flex items-center justify-between"
                    >
                        <div class="flex items-center gap-3">
                            <div
                                class="w-8 h-8 rounded-lg bg-blue-500/20 flex items-center justify-center"
                            >
                                <svg
                                    width="18"
                                    height="18"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="#3b82f6"
                                    stroke-width="2"
                                    ><path
                                        d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"
                                    /><polyline
                                        points="7 10 12 15 17 10"
                                    /><line
                                        x1="12"
                                        y1="15"
                                        x2="12"
                                        y2="3"
                                    /></svg
                                >
                            </div>
                            <div class="flex flex-col gap-0.5">
                                <h3 class="text-base font-bold text-zinc-200">
                                    {$_("instance_settings.java_portable")}
                                </h3>
                                <p class="text-xs text-zinc-500 font-medium max-w-lg">
                                    {$_("instance_settings.java_portable_desc")}
                                </p>
                            </div>
                        </div>
                    </div>
                    <div class="p-4 space-y-4">
                        <div
                            class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3"
                        >
                            {#each javaRuntimes as runtime}
                                <div
                                    class="bg-[#0f172a] border border-[#1e293b] rounded-xl p-3 flex flex-col gap-3 relative overflow-hidden group"
                                >
                                    <div
                                        class="flex items-center justify-between"
                                    >
                                        <div class="flex items-center gap-2">
                                            <span
                                                class="text-xs font-black uppercase tracking-tighter transition-colors {runtime.version === 16 ? 'text-yellow-500/60 cursor-help' : 'text-zinc-500'}"
                                                title={runtime.version === 16 ? $_("instance_settings.java_retired_tip") : null}
                                                >Java {runtime.version}</span
                                            >
                                        </div>
                                        {#if runtime.is_downloaded}
                                            <div
                                                class="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"
                                            ></div>
                                        {/if}
                                    </div>

                                    {#if downloadProgress[`java-download-${runtime.version}`]}
                                        <div
                                            class="space-y-2 py-1"
                                            transition:slide
                                        >
                                            <div
                                                class="flex justify-between text-[10px] text-blue-400 font-bold"
                                            >
                                                <span class="truncate pr-2"
                                                    >{downloadProgress[
                                                        `java-download-${runtime.version}`
                                                    ].step}</span
                                                >
                                                <span
                                                    >{downloadProgress[
                                                        `java-download-${runtime.version}`
                                                    ].progress}%</span
                                                >
                                            </div>
                                            <div
                                                class="h-1 bg-zinc-800 rounded-full overflow-hidden"
                                            >
                                                <div
                                                    class="h-full bg-blue-500 transition-all duration-300"
                                                    style="width: {downloadProgress[
                                                        `java-download-${runtime.version}`
                                                    ].progress}%"
                                                ></div>
                                            </div>
                                        </div>
                                    {:else}
                                        <div class="flex flex-col gap-2">
                                            {#if runtime.is_downloaded}
                                                <button
                                                    onclick={() =>
                                                        useJavaRuntime(
                                                            runtime.path,
                                                        )}
                                                    disabled={formSettings.java_path ===
                                                        runtime.path}
                                                    class="w-full py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-wider transition-all
                                                           {formSettings.java_path ===
                                                    runtime.path
                                                        ? 'bg-blue-500/10 text-blue-500 border border-blue-500/20'
                                                        : 'bg-zinc-800 text-zinc-400 hover:text-white hover:bg-zinc-700'}"
                                                >
                                                    {formSettings.java_path ===
                                                    runtime.path
                                                        ? $_("instance_settings.java_in_use")
                                                        : $_("instance_settings.java_use")}
                                                </button>
                                            {:else}
                                                <button
                                                    onclick={() =>
                                                        handleDownloadJava(
                                                            runtime.version,
                                                        )}
                                                    class="w-full py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-[10px] font-bold uppercase tracking-wider transition-all shadow-lg shadow-blue-600/20"
                                                >
                                                    {$_("instance_settings.java_download")}
                                                </button>
                                            {/if}
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                        <p class="text-xs text-zinc-500 px-1 italic">
                            {$_("instance_settings.java_folder_info")}<code
                                class="text-zinc-400 bg-black/20 px-1 rounded"
                                >%APPDATA%/AnvilCraftPanel/runtimes</code
                            >
                        </p>
                    </div>
                </div>

                <!-- Java Args -->
                <div class="space-y-3">
                    <div class="flex justify-between items-center">
                        <label
                            for="java-args"
                            class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                            >{$_("instance_settings.java_args")}</label
                        >
                        <button
                            onclick={resetGeneralSettingsForm}
                            disabled={isServerRunning}
                            class="text-[10px] font-bold text-zinc-500 hover:text-yellow-500 transition-colors uppercase tracking-widest px-2 py-1 rounded hover:bg-yellow-500/5"
                        >
                            {$_("instance_settings.btn_reset")}
                        </button>
                    </div>
                    <input
                        id="java-args"
                        type="text"
                        bind:value={formSettings.args}
                        oninput={markDirty}
                        disabled={isServerRunning}
                        placeholder="-XX:+UseG1GC..."
                        class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl px-4 py-3 font-jetbrains text-sm focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300 placeholder:text-zinc-700 shadow-inner"
                    />
                </div>
            </div>
        </div>
    </div>
</div>

<style>
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
