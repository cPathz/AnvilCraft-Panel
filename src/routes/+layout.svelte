<script lang="ts">
    import NavigationRail from "$lib/components/NavigationRail.svelte";
    import TopBar from "$lib/components/TopBar.svelte";
    import "../app.css";

    import { appState } from "$lib/runes/store.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { Instance } from "$lib/runes/store.svelte";

    import { listen } from "@tauri-apps/api/event";

    let { children } = $props();

    onMount(() => {
        let unlisten: () => void;

        const init = async () => {
            // Global Log Listener
            try {
                unlisten = await listen<[string, string]>(
                    "server-log",
                    (event) => {
                        const [id, line] = event.payload;
                        appState.ensureRuntime(id);
                        const runtime = appState.getRuntime(id);
                        if (runtime) runtime.logs.push(line);
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
        };
    });
</script>

<div class="relative flex h-screen w-screen text-white overflow-hidden">
    <!-- Global User Requested Background (Darkened +30%) -->
    <div class="absolute inset-0 z-0 bg-[#223049]">
        <!-- Gradient from #223049 (Bg) to #192232 (Bars) -->
        <div
            class="absolute inset-0 bg-gradient-to-br from-[#223049] to-[#192232]"
        ></div>

        <!-- Subtle noise texture overlay for professional finish (Optional but recommended) -->
        <div
            class="absolute inset-0 opacity-20 bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPScxMDAlJyBoZWlnaHQ9JzEwMCUnPjxmaWx0ZXIgaWQ9J25vaXNlJz48ZmVUdXJidWxlbmNlIHR5cGU9J2ZyYWN0YWxOb2lzZScgYmFzZUZyZXF1ZW5jeT0nMC42JyBzdGl0Y2hUaWxlcz0nc3RpdGNoJy8+PC9maWx0ZXI+PHJlY3Qgd2lkdGg9JzEwMCUnIGhlaWdodD0nMTAwJScgZmlsdGVyPSd1cmwoI25vaXNlKScgb3BhY2l0eT0nMC41Jy8+PC9zdmc+')] mix-blend-overlay"
        ></div>
    </div>

    <!-- Layout Content (Z-Index 10 to sit above background) -->
    <div class="relative z-10 flex w-full h-full">
        <NavigationRail />

        <div class="flex-1 flex flex-col min-w-0 relative">
            <div
                class="flex-1 w-full relative overflow-y-auto z-10 flex flex-col"
            >
                {@render children()}
            </div>
        </div>
    </div>
</div>
