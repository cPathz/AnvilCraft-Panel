<script module>
    import devIcon from "$lib/assets/dev_icon.png";
</script>

<script lang="ts">
    import { open } from "@tauri-apps/plugin-dialog";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";
    import { fade } from "svelte/transition";
    import { get } from "svelte/store";
    import { appState } from "$lib/runes/store.svelte";
    import { _ } from "svelte-i18n";

    // "Rising Particles" Logic (Adapted from Home.svelte)
    const particles = Array.from({ length: 100 }).map((_, i) => ({
        id: i,
        size: Math.random() * 5 + 1, // 1px to 6px
        startX: Math.random() * 100, // 0vw to 100vw
        endX: Math.random() * 100, // 0vw to 100vw (random drift)
        duration: Math.random() * 4 + 7, // 7s to 11s
        delay: Math.random() * 11, // 0s to 11s
        opacityDelay: Math.random() * 4, // Random delay for inner circle fade
    }));

    let gameRunning = $derived(
        appState.instances.some(
            (i) => i.state === "Running" || i.state === "Starting",
        ),
    );

    interface ImportReportResult {
        added: string[];
        skipped: string[];
        error?: string;
    }

    interface DataStats {
        version_count: number;
        latest_version: string;
        last_updated: string;
        versions: string[];
    }

    let stats = $state<DataStats | null>(null);

    async function loadStats() {
        try {
            stats = await invoke("get_data_stats");
        } catch (e) {
            console.error("Failed to load stats", e);
        }
    }

    interface ImportProgress {
        message: string;
        step: number;
        total_steps: number;
    }

    let selectedPath = $state<string | null>(null);
    let importReport = $state<ImportReportResult | null>(null);
    let loading = $state(false);
    let progress = $state<ImportProgress | null>(null);
    let activeTab = $state("tab_commands");
    let unlisten: () => void;

    onMount(async () => {
        loadStats();
        unlisten = await listen<ImportProgress>("import-progress", (event) => {
            const payload = event.payload;
            let message = payload.message;

            // Map backend messages to translations
            if (message.startsWith("Processing")) {
                message = get(_)("dev.status_importing");
            } else if (message.startsWith("Starting")) {
                message = get(_)("dev.status_starting");
            }

            progress = { ...payload, message };
            if (event.payload.step === 100) {
                loadStats(); // Refresh stats on completion
            }
        });
    });

    onDestroy(() => {
        if (unlisten) unlisten();
    });

    async function selectFolder() {
        const result = await open({
            directory: true,
            multiple: false,
            title: "Select OUTPUT_DATA Folder",
        });

        if (result) {
            selectedPath = result as string;
            importReport = null; // Reset report
            progress = null;
        }
    }

    async function runImport() {
        if (!selectedPath) return;

        loading = true;
        importReport = null;
        progress = { message: get(_)("dev.status_starting"), step: 0, total_steps: 100 };

        try {
            const report = await invoke("import_minecraft_data", {
                sourcePath: selectedPath,
            });
            importReport = report as any;
        } catch (err) {
            importReport = { added: [], skipped: [], error: String(err) };
        } finally {
            loading = false;
            progress = null;
        }
    }
</script>

<div
    class="dev-dashboard w-full h-full text-zinc-100 font-sans flex flex-col overflow-hidden relative transition-colors duration-1000"
    style="background-image: radial-gradient(#334565, #111621);"
>
    <!-- Particle Container -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
        {#each gameRunning ? particles.slice(0, 20) : particles as p}
            <div
                class="circle-container"
                style="
                    --size: {p.size}px;
                    --start-x: {p.startX}vw;
                    --end-x: {p.endX}vw;
                    --duration: {p.duration}s;
                    --delay: {p.delay}s;
                    --opacity-delay: {p.opacityDelay}s;
                "
            >
                <div class="circle"></div>
            </div>
        {/each}
    </div>

    <!-- Main Content Wrapper (z-10 to stay above particles) -->
    <div class="relative z-10 w-full h-full flex flex-col pointer-events-auto">
        <!-- Top Header (Global for View) -->
        <div
            class="flex-shrink-0 px-8 py-6 border-b border-zinc-800/50 flex items-center space-x-6 z-20 shadow-md backdrop-blur-sm bg-[#09090b]/40"
        >
            <div
                class="w-16 h-16 rounded-2xl overflow-hidden shadow-2xl shadow-blue-500/10 border border-zinc-800"
            >
                <img
                    src={devIcon}
                    alt="Dev"
                    class="w-full h-full object-cover opacity-90"
                />
            </div>
            <div>
                <h1 class="text-3xl font-black tracking-tight text-white mb-1">
                    {$_("dev.title")}
                </h1>
                <p
                    class="text-zinc-400 font-medium flex items-center gap-2 text-sm"
                >
                    <span
                        class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"
                    ></span>
                    {$_("dev.subtitle")}
                </p>
            </div>
        </div>

        <!-- Split View: Sidebar + Content -->
        <div class="flex flex-1 overflow-hidden relative">
            <!-- Sidebar -->
            <div
                class="w-48 flex flex-col py-8 px-4 space-y-2 flex-shrink-0 border-r border-zinc-800 relative z-50 bg-[#09090b]/40 backdrop-blur-sm pointer-events-auto"
            >
                {#each ["tab_commands", "tab_menu2", "tab_menu3", "tab_menu4"] as item}
                    <button
                        onclick={() => (activeTab = item)}
                        class="w-full text-left px-5 py-3.5 rounded-xl font-bold relative overflow-hidden group select-none {activeTab ===
                        item
                            ? 'bg-[#18181b]/80 text-white shadow-lg border border-zinc-700/50'
                            : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900/30'}"
                    >
                        <span
                            class="relative z-10 text-[17px] tracking-wide flex items-center justify-between"
                        >
                            {$_(`dev.${item}`)}
                            {#if activeTab === item}
                                <div
                                    class="w-1.5 h-1.5 rounded-full bg-white shadow-[0_0_8px_white]"
                                ></div>
                            {/if}
                        </span>

                        {#if activeTab === item}
                            <div
                                class="absolute left-0 top-0 bottom-0 w-1 bg-white/50 rounded-l-md"
                            ></div>
                        {/if}
                    </button>
                {/each}

                <div class="mt-auto px-2">
                    <div
                        class="p-3 bg-zinc-900/30 rounded-lg border border-zinc-800/50 text-[11px] text-zinc-500 text-center font-mono"
                    >
                        {$_("dev.build")}v{appState.appInfo.version}-{appState.appInfo.tag.toLowerCase()}
                    </div>
                </div>
            </div>

            <!-- Main Content Scrollable Area -->
            <div class="flex-1 overflow-y-auto p-8 relative">
                <div class="max-w-6xl mr-auto space-y-8">
                    <!-- Tab Content -->
                    {#if activeTab === "tab_commands"}
                        <!-- Stats Section -->
                        {#if stats}
                            <section
                                in:fade
                                class="grid grid-cols-1 md:grid-cols-3 gap-6"
                            >
                                <!-- Stat Card 1 -->
                                <div
                                    class="group bg-[#18181b]/60 backdrop-blur-md border border-zinc-800/50 rounded-2xl p-4 flex flex-col items-center justify-center text-center relative overflow-hidden"
                                >
                                    <div
                                        class="absolute inset-0 bg-blue-500/5 opacity-0 group-hover:opacity-100 transition-opacity"
                                    ></div>
                                    <span
                                        class="text-zinc-500 text-[14px] font-black uppercase tracking-widest mb-2 opacity-60"
                                        >{$_("dev.stat_loaded")}</span
                                    >
                                    <span
                                        class="text-3xl font-black text-white tracking-tighter"
                                        >{stats.version_count}</span
                                    >
                                </div>

                                <!-- Stat Card 2 -->
                                <div
                                    class="group bg-[#18181b]/60 backdrop-blur-md border border-zinc-800/50 rounded-2xl p-4 flex flex-col items-center justify-center text-center relative overflow-hidden"
                                >
                                    <div
                                        class="absolute inset-0 bg-green-500/5 opacity-0 group-hover:opacity-100 transition-opacity"
                                    ></div>
                                    <span
                                        class="text-zinc-500 text-[14px] font-black uppercase tracking-widest mb-2 opacity-60"
                                        >{$_("dev.stat_latest")}</span
                                    >
                                    <span
                                        class="text-3xl font-black text-green-400 tracking-tighter"
                                        >{stats.latest_version}</span
                                    >
                                </div>

                                <!-- Stat Card 3 -->
                                <div
                                    class="group bg-[#18181b]/60 backdrop-blur-md border border-zinc-800/50 rounded-2xl p-4 flex flex-col items-center justify-center text-center relative overflow-hidden"
                                >
                                    <div
                                        class="absolute inset-0 bg-purple-500/5 opacity-0 group-hover:opacity-100 transition-opacity"
                                    ></div>
                                    <span
                                        class="text-zinc-500 text-[14px] font-black uppercase tracking-widest mb-2 opacity-60"
                                        >{$_("dev.stat_status")}</span
                                    >
                                    <span
                                        class="px-3 py-1 rounded-full text-xs font-black bg-blue-500/10 text-blue-400 border border-blue-500/20 shadow-[0_0_20px_rgba(59,130,246,0.15)]"
                                    >
                                        {$_("dev.stat_ready")}
                                    </span>
                                </div>
                            </section>
                        {/if}

                        <section class="space-y-6 pt-4">
                            <div
                                class="flex items-center space-x-3 text-zinc-100 border-b border-zinc-800 pb-4"
                            >
                                <span class="text-2xl">📂</span>
                                <h2 class="text-2xl font-bold">
                                    {$_("dev.export_title")}
                                </h2>
                            </div>

                            <div
                                class="bg-[#18181b]/60 backdrop-blur-md border border-zinc-800/50 rounded-2xl p-8 space-y-8 shadow-xl relative overflow-hidden"
                            >
                                <!-- Decorator -->
                                <div
                                    class="absolute top-0 right-0 w-96 h-96 bg-blue-600/5 rounded-full blur-[100px] pointer-events-none -mr-32 -mt-32"
                                ></div>

                                <div class="space-y-2 relative z-10">
                                    <p
                                        class="text-zinc-400 leading-relaxed max-w-2xl"
                                    >
                                        {$_("dev.export_desc")}
                                    </p>
                                </div>

                                <!-- Folder Selection -->
                                <div
                                    class="flex items-center space-x-4 relative z-10"
                                >
                                    <button
                                        onclick={selectFolder}
                                        class="px-6 py-3 bg-white text-black hover:bg-zinc-200 rounded-xl font-bold transition-transform active:scale-95 shadow-lg shadow-white/5 flex items-center gap-2"
                                    >
                                        <span>{$_("dev.btn_browse")}</span>
                                    </button>

                                    <div
                                        class="flex-1 font-mono text-sm bg-black/40 p-3.5 rounded-xl border border-zinc-800 text-zinc-400 truncate flex items-center"
                                    >
                                        <span class="text-zinc-600 mr-2">$</span
                                        >
                                        {selectedPath || $_("dev.no_folder")}
                                    </div>
                                </div>

                                <!-- Progress Bar (Enhanced) -->
                                {#if loading && progress}
                                    <div
                                        class="space-y-3 bg-zinc-900/50 p-4 rounded-xl border border-zinc-800/50"
                                    >
                                        <div
                                            class="flex justify-between text-xs text-blue-400 uppercase font-black tracking-widest"
                                        >
                                            <span>{progress.message}</span>
                                            <span
                                                >{Math.round(
                                                    (progress.step /
                                                        progress.total_steps) *
                                                        100,
                                                )}%</span
                                            >
                                        </div>
                                        <div
                                            class="w-full bg-zinc-800/50 rounded-full h-3 overflow-hidden border border-zinc-800"
                                        >
                                            <div
                                                class="bg-blue-500 h-full transition-all duration-300 ease-out relative overflow-hidden shadow-[0_0_10px_#3b82f6]"
                                                style="width: {(progress.step /
                                                    progress.total_steps) *
                                                    100}%"
                                            >
                                                <div
                                                    class="absolute inset-0 bg-[linear-gradient(45deg,transparent_25%,rgba(255,255,255,0.3)_50%,transparent_75%)] bg-[length:20px_20px] animate-[shimmer_1s_infinite_linear]"
                                                ></div>
                                            </div>
                                        </div>
                                    </div>
                                {/if}

                                <!-- Action Button -->
                                <div
                                    class="flex justify-end pt-4 border-t border-zinc-800/50 relative z-10"
                                >
                                    <button
                                        onclick={runImport}
                                        disabled={!selectedPath || loading}
                                        class="px-8 py-3 bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-800 disabled:text-zinc-500 disabled:cursor-not-allowed text-white rounded-xl font-bold shadow-lg shadow-blue-500/20 transition-all flex items-center gap-3 active:scale-95 disabled:active:scale-100 disabled:shadow-none"
                                    >
                                        {#if loading}
                                            <div
                                                class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"
                                            ></div>
                                            <span>{$_("dev.processing")}</span>
                                        {:else}
                                            <span>{$_("dev.btn_run")}</span>
                                            <span class="text-white/30">→</span>
                                        {/if}
                                    </button>
                                </div>
                            </div>
                        </section>

                        <!-- Report Area -->
                        {#if importReport}
                            <section in:fade class="space-y-4 pt-4 pb-20">
                                <h3 class="text-xl font-bold text-zinc-200">
                                    {$_("dev.report_title")}
                                </h3>

                                {#if importReport.error}
                                    <div
                                        class="p-6 bg-red-500/10 border border-red-500/20 rounded-2xl text-red-200 flex items-start gap-4"
                                    >
                                        <span class="text-2xl">🚨</span>
                                        <div>
                                            <strong
                                                class="block text-red-400 mb-1"
                                                >{$_("dev.report_failed")}</strong
                                            >
                                            {importReport.error}
                                        </div>
                                    </div>
                                {:else}
                                    <div
                                        class="grid grid-cols-1 md:grid-cols-2 gap-6"
                                    >
                                        <!-- Added -->
                                        <div
                                            class="bg-[#18181b]/70 border border-zinc-800 rounded-2xl p-6 relative overflow-hidden"
                                        >
                                            <div
                                                class="absolute top-0 right-0 w-32 h-32 bg-green-500/5 rounded-full blur-3xl pointer-events-none -mr-10 -mt-10"
                                            ></div>
                                            <h4
                                                class="text-green-400 font-black mb-4 flex items-center text-lg"
                                            >
                                                {$_("dev.added_title")}
                                                <span
                                                    class="ml-auto text-xs bg-green-500/10 text-green-400 px-3 py-1 rounded-full border border-green-500/20"
                                                    >{importReport.added.length}
                                                    {$_("dev.added_new")}</span
                                                >
                                            </h4>
                                            {#if importReport.added.length === 0}
                                                <div
                                                    class="h-32 flex items-center justify-center text-zinc-600 italic border-2 border-dashed border-zinc-800 rounded-xl"
                                                >
                                                    {$_("dev.added_none")}
                                                </div>
                                            {:else}
                                                <ul
                                                    class="space-y-2 text-sm text-zinc-300 font-mono max-h-48 overflow-y-auto custom-scrollbar pr-2"
                                                >
                                                    {#each importReport.added as ver}
                                                        <li
                                                            class="flex items-center gap-3 p-2 bg-black/20 rounded-lg border border-white/5"
                                                        >
                                                            <span
                                                                class="text-green-500 text-xs"
                                                                >●</span
                                                            >
                                                            {ver}
                                                        </li>
                                                    {/each}
                                                </ul>
                                            {/if}
                                        </div>

                                        <!-- Skipped -->
                                        <div
                                            class="bg-[#18181b]/70 border border-zinc-800 rounded-2xl p-6 relative overflow-hidden"
                                        >
                                            <h4
                                                class="text-zinc-400 font-black mb-4 flex items-center text-lg"
                                            >
                                                {$_("dev.skipped_title")}
                                                <span
                                                    class="ml-auto text-xs bg-zinc-800 text-zinc-400 px-3 py-1 rounded-full border border-zinc-700"
                                                    >{importReport.skipped
                                                        .length} {$_("dev.skipped_existing")}</span
                                                >
                                            </h4>
                                            {#if importReport.skipped.length === 0}
                                                <div
                                                    class="h-32 flex items-center justify-center text-zinc-600 italic border-2 border-dashed border-zinc-800 rounded-xl"
                                                >
                                                    {$_("dev.skipped_none")}
                                                </div>
                                            {:else}
                                                <ul
                                                    class="space-y-2 text-sm text-zinc-500 font-mono max-h-48 overflow-y-auto custom-scrollbar pr-2"
                                                >
                                                    {#each importReport.skipped as ver}
                                                        <li
                                                            class="flex items-center gap-3 p-2 bg-black/20 rounded-lg border border-white/5"
                                                        >
                                                            <span
                                                                class="text-zinc-700 text-xs"
                                                                >●</span
                                                            >
                                                            {ver}
                                                        </li>
                                                    {/each}
                                                </ul>
                                            {/if}
                                        </div>
                                    </div>
                                {/if}
                            </section>
                        {/if}
                    {:else}
                        <!-- Placeholder for other tabs -->
                        <div
                            class="flex flex-col items-center justify-center h-[500px] border-2 border-dashed border-zinc-800 rounded-3xl text-zinc-600 bg-zinc-900/10 backdrop-blur-sm"
                        >

                            <span class="text-6xl mb-4 opacity-50">🚧</span>
                            <h3 class="text-2xl font-bold text-zinc-500">
                                {$_("dev.wip_title")}
                            </h3>
                            <p class="text-zinc-700 mt-2">
                                {$_("dev." + activeTab)}{$_("dev.wip_desc")}
                            </p>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    /* Particle styling adapted from Home.svelte */
    .circle-container {
        position: absolute;
        top: 0;
        left: 0;
        width: var(--size);
        height: var(--size);
        animation: floatUp var(--duration) linear infinite;
        animation-delay: var(--delay);
        transform: translate3d(var(--start-x), 110vh, 0);
    }

    .circle {
        width: 100%;
        height: 100%;
        border-radius: 50%;
        mix-blend-mode: screen;
        background-image: radial-gradient(
            hsl(180, 100%, 80%),
            hsl(180, 100%, 80%) 10%,
            hsla(180, 100%, 80%, 0) 56%
        );
        animation:
            fadeFrames 200ms infinite,
            scaleFrames 2s infinite;
        animation-delay: var(--opacity-delay);
    }

    @keyframes floatUp {
        from {
            transform: translate3d(var(--start-x), 110vh, 0);
        }
        to {
            transform: translate3d(var(--end-x), -20vh, 0);
        }
    }

    @keyframes fadeFrames {
        0% {
            opacity: 1;
        }
        50% {
            opacity: 0.7;
        }
        100% {
            opacity: 1;
        }
    }

    @keyframes scaleFrames {
        0% {
            transform: scale3d(0.4, 0.4, 1);
        }
        50% {
            transform: scale3d(2.2, 2.2, 1);
        }
        100% {
            transform: scale3d(0.4, 0.4, 1);
        }
    }

    /* Custom Scrollbar for inner lists */
    ul::-webkit-scrollbar {
        width: 6px;
    }
    ul::-webkit-scrollbar-track {
        background: transparent;
    }
    ul::-webkit-scrollbar-thumb {
        background-color: rgba(255, 255, 255, 0.1);
        border-radius: 3px;
    }
</style>
