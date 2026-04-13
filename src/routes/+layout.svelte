<script lang="ts">
    import NavigationRail from "$lib/components/NavigationRail.svelte";
    import TopBar from "$lib/components/TopBar.svelte";
    import ToastContainer from "$lib/components/ToastContainer.svelte";
    import BetaWatermark from "$lib/components/BetaWatermark.svelte";
    import "../app.css";

    import { appState } from "$lib/runes/store.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { Instance } from "$lib/runes/store.svelte";

    import { listen } from "@tauri-apps/api/event";
    import { getCurrentWindow } from "@tauri-apps/api/window";

    import { setupI18n } from "$lib/i18n";
    import { isLoading, _ } from "svelte-i18n";
    import { get } from "svelte/store";
    import { toast } from "$lib/runes/toast.svelte";

    setupI18n();

    // Global UI state for close warning and force‑close handling
    let showCloseWarning = $state(false);
    let isForceClosing = $state(false);

    let { children } = $props();

    onMount(() => {
        let unlisten: () => void;

        // Close prevention logic
        // (inner duplicate state removed, using outer definitions)
        async function setupCloseHandler() {
            const un = await listen("app-close-forbidden", () => {
                showCloseWarning = true;
            });
            return un;
        }

        let unlistenClose: any;
        setupCloseHandler().then((u) => (unlistenClose = u));

        const init = async () => {
            // Global Log Listener
            try {
                unlisten = await listen<[string, string]>(
                    "server-log",
                    (event) => {
                        const [id, line] = event.payload;
                        appState.ensureRuntime(id);
                        const runtime = appState.getRuntime(id);
                        if (runtime) {
                            runtime.logs.push(line);
                            if (runtime.logs.length > 1000) {
                                runtime.logs = runtime.logs.slice(-1000);
                            }
                        }
                    },
                );

                const unlistenUpdate = await listen(
                    "instance-update",
                    async () => {
                        await refreshInstances();
                    },
                );
                // Combine cleanup
                const oldUnlisten = unlisten;
                unlisten = () => {
                    oldUnlisten();
                    unlistenUpdate();
                };
            } catch (e) {
                console.error("Failed to setup listeners:", e);
            }

            await refreshInstances();
        };

        async function refreshInstances() {
            try {
                appState.refreshing = true;
                const instances = await invoke<Instance[]>("read_instances");
                appState.instances = instances;

                // Sync selectedInstance if active
                if (appState.selectedInstance) {
                    const updated = instances.find(
                        (i) => i.id === appState.selectedInstance!.id,
                    );
                    if (updated) appState.selectedInstance = updated;
                }
            } catch (error) {
                console.error("Failed to load instances:", error);
            } finally {
                appState.refreshing = false;
            }
        }

        init();

        return () => {
            if (unlisten) unlisten();
            if (unlistenClose) unlistenClose();
        };
    });

    async function forceCloseAll() {
        isForceClosing = true;
        try {
            // 1. Kill all known running instances
            const running = appState.instances.filter(
                (i) =>
                    i.state === "Running" ||
                    i.state === "Stopping" ||
                    i.state === "Starting",
            );
            for (const instance of running) {
                await invoke("kill_instance", { id: instance.id });
            }
            // 2. Wait a tiny bit for backend to clear map
            await new Promise((r) => setTimeout(r, 500));
            // 3. Retry close
            await getCurrentWindow().close();
        } catch (e) {
            console.error("Force close failed:", e);
            toast.error(get(_)("settings.toast_force_close_error") + e);
            isForceClosing = false;
        }
    }
</script>

<!-- Close Warning Modal -->
{#if showCloseWarning}
    <div
        class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/80 backdrop-blur-sm p-4"
    >
        <div
            class="bg-[#1e293b] border border-red-500/30 rounded-2xl shadow-2xl max-w-md w-full overflow-hidden flex flex-col"
        >
            <div class="p-6">
                <div class="flex items-center gap-3 mb-4 text-red-400">
                    <svg
                        width="32"
                        height="32"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path
                            d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
                        />
                        <line x1="12" y1="9" x2="12" y2="13" />
                        <line x1="12" y1="17" x2="12.01" y2="17" />
                    </svg>
                    <h2 class="text-xl font-bold">¡Servidores Activos!</h2>
                </div>
                <p class="text-zinc-300 leading-relaxed mb-6">
                    No puedes cerrar AnvilCraft mientras hay servidores en
                    ejecución para evitar corrupción de datos.
                </p>
                <div
                    class="bg-red-500/10 border border-red-500/20 rounded-lg p-3 mb-6"
                >
                    <p class="text-xs text-red-300 font-mono">
                        Servers: {appState.instances
                            .filter(
                                (i) =>
                                    i.state !== "Stopped" &&
                                    i.state !== "Error",
                            )
                            .map((i) => i.name)
                            .join(", ")}
                    </p>
                </div>

                <div class="flex justify-end gap-3">
                    <button
                        onclick={() => (showCloseWarning = false)}
                        class="px-4 py-2 rounded-lg font-medium text-zinc-400 hover:text-white hover:bg-white/5 transition-colors"
                    >
                        Cancelar
                    </button>
                    <button
                        onclick={forceCloseAll}
                        disabled={isForceClosing}
                        class="px-4 py-2 rounded-lg font-bold bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/20 transition-all flex items-center gap-2"
                    >
                        {#if isForceClosing}
                            <svg
                                class="animate-spin h-4 w-4 text-white"
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
                            Cerrando...
                        {:else}
                            Forzar Cierre Total
                        {/if}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}

<div class="relative flex h-screen w-screen text-white overflow-hidden">
    <!-- Global User Requested Background (Darkened +30%) -->
    <div class="absolute inset-0 z-0 bg-[#223049]">
        <!-- Gradient from #223049 (Bg) to #192232 (Bars) -->
        <div
            class="absolute inset-0 bg-gradient-to-br from-[#223049] to-[#192232]"
        ></div>
    </div>

    <!-- Layout Content (Z-Index 10 to sit above background) -->
    <div class="relative z-10 flex w-full h-full">
        <NavigationRail />

        <div class="flex-1 flex flex-col min-w-0 relative">
            <div
                class="flex-1 w-full relative overflow-y-auto z-10 flex flex-col"
            >
                {#if !$isLoading}
                    {@render children()}
                {/if}
            </div>
        </div>
    </div>

    <ToastContainer />
    <BetaWatermark />
</div>
