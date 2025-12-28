<script module>
    import devIcon from "$lib/assets/dev_icon.png";
</script>

<script lang="ts">
    import { open } from "@tauri-apps/plugin-dialog";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";

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
    let unlisten: () => void;

    onMount(async () => {
        loadStats();
        unlisten = await listen<ImportProgress>("import-progress", (event) => {
            progress = event.payload;
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
        progress = { message: "Starting...", step: 0, total_steps: 100 };

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
    class="dev-dashboard w-full h-full p-8 overflow-y-auto bg-[#09090b] text-zinc-100 font-sans"
>
    <!-- ... (header) ... -->
    <div class="max-w-4xl mx-auto space-y-8">
        <!-- Header -->
        <div class="flex items-center space-x-4 border-b border-zinc-800 pb-6">
            <!-- ... existing header content ... -->
            <div
                class="w-16 h-16 rounded-xl overflow-hidden shadow-lg shadow-blue-500/20"
            >
                <img
                    src={devIcon}
                    alt="Dev"
                    class="w-full h-full object-cover"
                />
            </div>
            <div>
                <h1
                    class="text-3xl font-bold bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent"
                >
                    Developer Dashboard
                </h1>
                <p class="text-zinc-400 mt-1">
                    Herramientas internas de desarrollo e ingesta de datos.
                </p>
            </div>
        </div>

        <!-- Stats Section -->
        {#if stats}
            <section in:fade class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div
                    class="bg-[#18181b] border border-zinc-800 rounded-xl p-4 flex flex-col items-center justify-center text-center"
                >
                    <span class="text-zinc-400 text-sm font-medium mb-1"
                        >Versiones Cargadas</span
                    >
                    <span class="text-3xl font-bold text-white"
                        >{stats.version_count}</span
                    >
                </div>
                <div
                    class="bg-[#18181b] border border-zinc-800 rounded-xl p-4 flex flex-col items-center justify-center text-center"
                >
                    <span class="text-zinc-400 text-sm font-medium mb-1"
                        >Última Versión</span
                    >
                    <span class="text-3xl font-bold text-green-400"
                        >{stats.latest_version}</span
                    >
                </div>
                <div
                    class="bg-[#18181b] border border-zinc-800 rounded-xl p-4 flex flex-col items-center justify-center text-center"
                >
                    <span class="text-zinc-400 text-sm font-medium mb-1"
                        >Estado de Datos</span
                    >
                    <span
                        class="px-2 py-1 rounded-full text-xs font-bold bg-blue-900/30 text-blue-300 border border-blue-800/50"
                    >
                        READY
                    </span>
                </div>
            </section>
        {/if}

        <!-- Data Ingestion Section -->
        <section class="space-y-4">
            <!-- ... -->

            <!-- Data Ingestion Section -->
            <section class="space-y-4">
                <h2
                    class="text-xl font-semibold text-zinc-200 flex items-center"
                >
                    <span class="mr-2">📂</span> Ingesta de Datos Minecraft
                </h2>

                <div
                    class="bg-[#18181b] border border-zinc-800 rounded-xl p-6 space-y-6"
                >
                    <p class="text-zinc-400 text-sm">
                        Selecciona la carpeta <code>OUTPUT_DATA</code> generada por
                        tu script de PowerShell. El sistema importará automáticamente
                        las versiones nuevas y actualizará el manifiesto.
                    </p>

                    <!-- Folder Selection -->
                    <div class="flex items-center space-x-4">
                        <button
                            onclick={selectFolder}
                            class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded-lg border border-zinc-700 transition-colors flex items-center font-medium"
                        >
                            Select Folder...
                        </button>

                        <div
                            class="flex-1 font-mono text-sm bg-black/30 p-2.5 rounded border border-zinc-800/50 text-zinc-400 truncate"
                        >
                            {selectedPath || "No folder selected"}
                        </div>
                    </div>

                    <!-- Progress Bar -->
                    {#if loading && progress}
                        <div class="space-y-2 animate-fade-in">
                            <div
                                class="flex justify-between text-xs text-zinc-400 uppercase font-bold tracking-wider"
                            >
                                <span>{progress.message}</span>
                                <span
                                    >{Math.round(
                                        (progress.step / progress.total_steps) *
                                            100,
                                    )}%</span
                                >
                            </div>
                            <div
                                class="w-full bg-zinc-800 rounded-full h-2 overflow-hidden"
                            >
                                <div
                                    class="bg-blue-500 h-full transition-all duration-300 ease-out relative overflow-hidden"
                                    style="width: {(progress.step /
                                        progress.total_steps) *
                                        100}%"
                                >
                                    <div
                                        class="absolute inset-0 bg-white/20 animate-pulse"
                                    ></div>
                                </div>
                            </div>
                        </div>
                    {/if}

                    <!-- Action Button -->
                    <div class="flex justify-end pt-2">
                        <button
                            onclick={runImport}
                            disabled={!selectedPath || loading}
                            class="px-6 py-2.5 bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-800 disabled:text-zinc-500 disabled:cursor-not-allowed text-white rounded-lg font-medium shadow-lg shadow-blue-900/20 transition-all flex items-center"
                        >
                            {#if loading}
                                <svg
                                    class="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
                                    xmlns="http://www.w3.org/2000/svg"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                >
                                    <circle
                                        class="opacity-25"
                                        cx="12"
                                        cy="12"
                                        r="10"
                                        stroke="currentColor"
                                        stroke-width="4"
                                    ></circle>
                                    <path
                                        class="opacity-75"
                                        fill="currentColor"
                                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                    ></path>
                                </svg>
                                Processing...
                            {:else}
                                Run Import Process
                            {/if}
                        </button>
                    </div>
                </div>
            </section>

            <!-- Report Area -->
            {#if importReport}
                <section in:fade class="space-y-4">
                    <h3 class="text-lg font-medium text-zinc-300">
                        Import Report
                    </h3>

                    {#if importReport.error}
                        <div
                            class="p-4 bg-red-900/20 border border-red-800/50 rounded-lg text-red-200"
                        >
                            <strong>Error:</strong>
                            {importReport.error}
                        </div>
                    {:else}
                        <div class="grid grid-cols-2 gap-4">
                            <!-- Added -->
                            <div
                                class="bg-green-900/10 border border-green-800/30 rounded-lg p-4"
                            >
                                <h4
                                    class="text-green-400 font-bold mb-3 flex items-center"
                                >
                                    Added Versions <span
                                        class="ml-2 text-xs bg-green-900/50 px-2 py-0.5 rounded-full"
                                        >{importReport.added.length}</span
                                    >
                                </h4>
                                {#if importReport.added.length === 0}
                                    <p class="text-zinc-500 italic text-sm">
                                        No new versions found.
                                    </p>
                                {:else}
                                    <ul
                                        class="space-y-1 text-sm text-green-200/80 font-mono max-h-40 overflow-y-auto"
                                    >
                                        {#each importReport.added as ver}
                                            <li>+ {ver}</li>
                                        {/each}
                                    </ul>
                                {/if}
                            </div>

                            <!-- Skipped -->
                            <div
                                class="bg-zinc-800/30 border border-zinc-700/30 rounded-lg p-4"
                            >
                                <h4
                                    class="text-zinc-400 font-bold mb-3 flex items-center"
                                >
                                    Skipped (Existing) <span
                                        class="ml-2 text-xs bg-zinc-800 px-2 py-0.5 rounded-full"
                                        >{importReport.skipped.length}</span
                                    >
                                </h4>
                                {#if importReport.skipped.length === 0}
                                    <p class="text-zinc-600 italic text-sm">
                                        None skipped.
                                    </p>
                                {:else}
                                    <ul
                                        class="space-y-1 text-sm text-zinc-500 font-mono max-h-40 overflow-y-auto"
                                    >
                                        {#each importReport.skipped as ver}
                                            <li>= {ver}</li>
                                        {/each}
                                    </ul>
                                {/if}
                            </div>
                        </div>
                    {/if}
                </section>
            {/if}
        </section>
    </div>
</div>

<style>
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
