<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";

    import IconPicker from "./IconPicker.svelte";

    let instanceName = $state("");
    let activeTab = $state<"custom" | "file" | "import">("custom");
    let selectedLoader = $state<
        "Vanilla" | "Fabric" | "Forge" | "NeoForge" | "Quilt"
    >("Vanilla");
    let hoveredIcon = $state(false);

    // Icon Selection State
    let showIconPicker = $state(false);
    let selectedIcon = $state("/Transparent-Images/grass_block.png");

    function close() {
        appState.creatingInstance = false;
    }

    function handleIconSelect(icon: string) {
        selectedIcon = icon;
        showIconPicker = false;
    }

    async function handleCreate() {
        if (!instanceName.trim()) return;

        try {
            const id = await invoke("create_instance", {
                name: instanceName,
                loader: selectedLoader,
                version: "1.20.1", // TODO: Make dynamic
                icon: selectedIcon,
            });
            console.log("Instance created:", id);

            // Refresh list
            const instances = await invoke<any[]>("read_instances");
            appState.instances = instances;

            close();
        } catch (error) {
            console.error("Failed to create instance:", error);
            alert("Error al crear la instancia: " + error);
        }
    }
</script>

<!-- Backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 animate-fade-in"
    onclick={(e) => {
        if (e.target === e.currentTarget) close();
    }}
>
    <!-- Modal Container -->
    <div
        class="bg-[#18181b] w-full max-w-xl rounded-xl border border-zinc-800 shadow-2xl overflow-hidden flex flex-col max-h-[90vh] animate-scale-in"
    >
        <!-- Header -->
        <div
            class="flex items-center justify-between px-6 py-4 border-b border-zinc-800"
        >
            <h2 class="text-lg font-bold text-white">Crear nueva instancia</h2>
            <button
                onclick={close}
                class="text-zinc-400 hover:text-white transition-colors"
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
        <div class="p-6 overflow-y-auto space-y-6">
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

            <!-- Content -->
            <div class="flex gap-6">
                <!-- Icon Picker -->
                <div class="flex flex-col gap-3">
                    <button
                        type="button"
                        class="w-24 h-24 rounded-2xl bg-zinc-800 border-2 border-dashed border-zinc-700 flex items-center justify-center relative overflow-hidden group cursor-pointer hover:border-zinc-500 transition-colors"
                        onmouseenter={() => (hoveredIcon = true)}
                        onmouseleave={() => (hoveredIcon = false)}
                        onclick={() => (showIconPicker = true)}
                    >
                        <!-- Local asset icon placeholder -->
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
                </div>

                <!-- Form Fields -->
                <div class="flex-1 space-y-5">
                    <!-- Name Input -->
                    <div class="space-y-1.5">
                        <label class="text-sm font-bold text-zinc-300"
                            >Nombre</label
                        >
                        <input
                            type="text"
                            bind:value={instanceName}
                            placeholder="Mi Nuevo Servidor"
                            class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-white placeholder-zinc-600 focus:outline-none focus:border-green-500/50 focus:ring-1 focus:ring-green-500/50 transition-all font-medium"
                        />
                    </div>

                    <!-- Loader Selection -->
                    <div class="space-y-2">
                        <label class="text-sm font-bold text-zinc-300"
                            >Loader</label
                        >
                        <div class="flex flex-wrap gap-2">
                            {#each ["Vanilla", "Fabric", "Forge", "NeoForge", "Quilt"] as loader}
                                <button
                                    class="px-3 py-1.5 rounded-lg text-sm font-medium border transition-all {selectedLoader ===
                                    loader
                                        ? 'bg-green-500/10 border-green-500 text-green-400'
                                        : 'bg-zinc-900 border-zinc-800 text-zinc-400 hover:border-zinc-700'}"
                                    onclick={() =>
                                        (selectedLoader = loader as any)}
                                >
                                    {loader}
                                </button>
                            {/each}
                        </div>
                    </div>

                    <!-- Version Selection -->
                    <div class="space-y-1.5">
                        <label class="text-sm font-bold text-zinc-300"
                            >Versión del Juego</label
                        >
                        <div class="relative">
                            <select
                                class="w-full bg-black/20 border border-zinc-700 rounded-lg px-3 py-2.5 text-white appearance-none focus:outline-none focus:border-green-500/50 transition-all font-medium cursor-pointer"
                            >
                                <option>1.20.4</option>
                                <option>1.20.1</option>
                                <option>1.19.4</option>
                                <option>1.18.2</option>
                            </select>
                            <div
                                class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-zinc-500"
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
                                    ><path d="m6 9 6 6 6-6" /></svg
                                >
                            </div>
                        </div>
                        <div class="flex items-center gap-2 mt-2">
                            <input
                                type="checkbox"
                                id="snapshots"
                                class="rounded border-zinc-700 bg-zinc-900 text-green-500 focus:ring-0 focus:ring-offset-0"
                            />
                            <label
                                for="snapshots"
                                class="text-xs text-zinc-500 select-none cursor-pointer"
                                >Mostrar Snapshots</label
                            >
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Footer -->
        <div class="p-6 pt-2 flex justify-end gap-3">
            <button
                onclick={close}
                class="px-4 py-2 rounded-lg text-sm font-bold text-zinc-400 hover:text-white hover:bg-zinc-800/50 transition-colors"
            >
                Cancelar
            </button>
            <button
                onclick={handleCreate}
                class="px-6 py-2 rounded-lg text-sm font-bold bg-green-600 hover:bg-green-500 text-white shadow-lg shadow-green-900/20 transition-all active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
                disabled={!instanceName.trim()}
            >
                Crear Instancia
            </button>
        </div>
    </div>
</div>

{#if showIconPicker}
    <IconPicker
        onselect={handleIconSelect}
        onclose={() => (showIconPicker = false)}
    />
{/if}

<style>
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
