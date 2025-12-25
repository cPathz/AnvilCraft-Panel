<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";

    let instances = $derived(appState.instances);

    function selectInstance(instance: Instance) {
        appState.selectedInstance = instance;
    }

    async function openFolder() {
        try {
            await invoke("open_instances_folder");
        } catch (e) {
            console.error(e);
        }
    }

    async function refreshInstances() {
        try {
            const instances = await invoke("read_instances");
            appState.instances = instances as Instance[];
        } catch (e) {
            console.error(e);
        }
    }
</script>

<svelte:window onfocus={refreshInstances} />

<div class="w-full h-full px-8 pb-8 pt-5 overflow-y-auto">
    <div class="max-w-6xl mx-auto space-y-6">
        <!-- Header -->
        <div class="flex items-center justify-between" data-tauri-drag-region>
            <div>
                <h2
                    class="text-2xl font-bold text-white tracking-tight drop-shadow-sm"
                >
                    Mis Instancias
                </h2>
                <p class="text-sm text-slate-300 mt-0.5 font-medium">
                    Gestiona tus servidores de Minecraft
                </p>
            </div>
            <div class="flex items-center gap-2">
                <button
                    class="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white px-3 py-1.5 rounded-lg text-sm font-medium transition-colors flex items-center gap-2 shadow-lg shadow-black/20 border border-zinc-700"
                    onclick={refreshInstances}
                    title="Actualizar lista"
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
                        ><path
                            d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"
                        /><path d="M3 3v5h5" /><path
                            d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"
                        /><path d="M16 16h5v5" /></svg
                    >
                </button>
                <button
                    class="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white px-3 py-1.5 rounded-lg text-sm font-medium transition-colors flex items-center gap-2 shadow-lg shadow-black/20 border border-zinc-700"
                    onclick={openFolder}
                >
                    <svg
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        ><path
                            d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 2H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2z"
                        ></path></svg
                    > Ver Carpeta
                </button>
            </div>
        </div>

        <!-- Widget Grid View -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each instances as instance}
                <!-- Widget Card (Glassmorphism) -->
                <button
                    class="group relative bg-white/5 hover:bg-white/10 backdrop-blur-md border border-white/10 hover:border-white/20 rounded-xl p-3 transition-all flex items-start text-left overflow-hidden shadow-lg hover:shadow-black/30 hover:-translate-y-0.5"
                    onclick={() => selectInstance(instance)}
                >
                    <!-- Neon Left Border (The 'Green Stripe') -->
                    <div
                        class="absolute left-0 top-0 bottom-0 w-1.5"
                        class:bg-green-500={instance.state === "Running"}
                        class:bg-red-500={instance.state === "Stopped" ||
                            instance.state === "Error"}
                        class:bg-yellow-500={instance.state === "Starting"}
                    ></div>

                    <!-- Inner Layout: Icon Left | Details Right -->
                    <div class="flex w-full gap-4 pl-3">
                        <!-- Big Icon (Square-ish) -->
                        <div
                            class="w-16 h-16 rounded-xl bg-gradient-to-b from-[#192232] to-[#0f1520] flex-shrink-0 flex items-center justify-center shadow-inner overflow-hidden"
                        >
                            <img
                                src={instance.icon ||
                                    `https://ui-avatars.com/api/?name=${instance.name}&background=random`}
                                alt={instance.name}
                                class="w-full h-full object-cover"
                            />
                        </div>

                        <!-- Right Content Column -->
                        <div
                            class="flex-grow flex flex-col gap-1.5 min-w-0 justify-center"
                        >
                            <!-- Name -->
                            <h3
                                class="font-bold text-white text-[15px] leading-none truncate group-hover:text-blue-300 transition-colors drop-shadow-md"
                            >
                                {instance.name}
                            </h3>

                            <!-- Details (Motor/Version) -->
                            <div
                                class="flex items-center gap-2 text-xs font-mono text-zinc-400"
                            >
                                <span
                                    class="bg-white/10 border border-white/5 px-1.5 rounded text-zinc-200"
                                    >{instance.loader}</span
                                >
                                <span class="text-zinc-400"
                                    >{instance.version}</span
                                >
                            </div>

                            <!-- Status & Arrow -->
                            <div class="flex items-center justify-between">
                                <span
                                    class="px-1.5 py-0 rounded border text-xs font-bold tracking-wider uppercase transition-colors"
                                    class:border-green-500={instance.state ===
                                        "Running"}
                                    class:text-green-500={instance.state ===
                                        "Running"}
                                    class:bg-green-500-10={instance.state ===
                                        "Running"}
                                    class:border-red-500={instance.state ===
                                        "Stopped"}
                                    class:text-red-500={instance.state ===
                                        "Stopped"}
                                    class:bg-red-500-10={instance.state ===
                                        "Stopped"}
                                >
                                    {instance.state}
                                </span>

                                <span
                                    class="text-zinc-600 group-hover:text-zinc-300 transition-colors text-sm transform group-hover:translate-x-1"
                                >
                                    &rarr;
                                </span>
                            </div>
                        </div>
                    </div>
                </button>
            {/each}

            <!-- Add New Widget -->
            <button
                class="border-2 border-dashed border-zinc-800 hover:border-zinc-700 hover:bg-zinc-900/30 rounded-xl flex flex-col items-center justify-center gap-2 text-zinc-600 hover:text-zinc-400 transition-all min-h-[90px]"
                onclick={() => (appState.creatingInstance = true)}
            >
                <div class="text-4xl text-white font-medium">+</div>
                <span class="text-xs font-medium">Crear Instancia</span>
            </button>
        </div>
    </div>
</div>
