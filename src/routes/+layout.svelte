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
    import { isLoading, _, locale } from "svelte-i18n";
    import { get } from "svelte/store";
    import { toast } from "$lib/runes/toast.svelte";

    setupI18n();

    // Global UI state for close warning and force‑close handling
    let showCloseWarning = $state(false);
    let isForceClosing = $state(false);
    
    // Safety flag to force render after a timeout
    let forceRender = $state(false);
    onMount(() => {
        setTimeout(() => { forceRender = true; }, 1500);
    });

    let { children } = $props();

    onMount(() => {
        let unlisten: () => void;

        // Close prevention logic
        async function setupCloseHandler() {
            try {
                const un = await listen("app-close-forbidden", () => {
                    showCloseWarning = true;
                });
                return un;
            } catch (e) {
                console.error("Failed to setup close handler:", e);
            }
        }

        let unlistenClose: any;
        setupCloseHandler().then((u) => (unlistenClose = u));

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
                            appState.parseLog(id, line);
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
                    if (typeof oldUnlisten === "function") oldUnlisten();
                    if (typeof unlistenUpdate === "function") unlistenUpdate();
                };
            } catch (e) {
                console.error("Failed to setup listeners:", e);
            }

            await refreshInstances();
        };

        init();

        return () => {
            if (typeof unlisten === "function") unlisten();
            if (typeof unlistenClose === "function") unlistenClose();
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
                    <h2 class="text-xl font-bold">{$_("layout.modal_close_active_title")}</h2>
                </div>
                <p class="text-zinc-300 leading-relaxed mb-6">
                    {$_("layout.modal_close_active_desc")}
                </p>
                <div
                    class="bg-red-500/10 border border-red-500/20 rounded-lg p-3 mb-6"
                >
                    <p class="text-xs text-red-300 font-mono">
                        {$_("layout.modal_close_active_list")} {appState.instances
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
                        {$_("instance_detail.btn_cancel")}
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
                            {$_("layout.btn_closing")}
                        {:else}
                            {$_("layout.btn_force_close")}
                        {/if}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}

<div class="h-screen w-screen overflow-hidden bg-[#223049] relative">
    <!-- Main Content Area -->
    <div class="absolute inset-0 flex">
        <NavigationRail />

        <div class="flex-1 flex flex-col min-w-0 relative">
            <div class="flex-1 w-full relative overflow-y-auto flex flex-col">
                {#if ($isLoading || !$locale) && !forceRender}
                    <!-- Fallback while loading -->
                    <div class="flex-1 flex items-center justify-center">
                        <div class="animate-pulse flex flex-col items-center gap-4">
                            <div class="w-12 h-12 rounded-full border-4 border-blue-500/20 border-t-blue-500 animate-spin"></div>
                            <span class="text-zinc-500 font-medium">{$_("layout.loading")}</span>
                        </div>
                    </div>
                {:else}
                    {@render children()}
                {/if}
            </div>
        </div>
    </div>

    <ToastContainer />
    <BetaWatermark />
</div>
