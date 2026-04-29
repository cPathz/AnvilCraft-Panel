<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";
    import { _ } from "svelte-i18n";
    import { invoke } from "@tauri-apps/api/core";

    interface Props {
        instance: Instance;
    }

    let { instance }: Props = $props();
    let runtime = $derived(appState.getRuntime(instance.id));
    let type = $derived(runtime?.addonsType || 'none');

    // UI state
    let loading = $state(true);
    let addons = $state<any[]>([]);

    async function loadAddons(force = false) {
        loading = true;
        try {
            addons = await invoke("get_instance_addons", { id: instance.id, forceScan: force });
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }

    $effect(() => {
        if (type !== 'none') {
            loadAddons();
        }
    });

    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }
</script>

<div class="flex flex-col h-full animate-fade-in">
    <!-- Header with Stats -->
    <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-4">
            <div class="p-3 rounded-2xl bg-blue-500/10 border border-blue-500/20 text-blue-400">
                {#if type === 'mods'}
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
                {:else}
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
                {/if}
            </div>
            <div>
                <h2 class="text-2xl font-bold text-white capitalize">{type}</h2>
                <p class="text-zinc-400 text-sm">Gestiona los complementos de tu servidor</p>
            </div>
        </div>

        <div class="flex gap-2">
            <button 
                onclick={() => loadAddons(true)}
                class="px-4 py-2 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-sm font-medium text-white transition-all flex items-center gap-2"
                disabled={loading}
            >
                {#if loading}
                    <div class="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"></div>
                {/if}
                Actualizar
            </button>
            <button class="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-xl text-sm font-bold text-white shadow-lg shadow-blue-900/20 transition-all active:scale-95">
                + Añadir {type === 'mods' ? 'Mod' : 'Plugin'}
            </button>
        </div>
    </div>

    <!-- Content Area: List View -->
    <div class="flex-grow overflow-hidden flex flex-col bg-white/[0.02] border border-white/5 rounded-3xl">
        <!-- Table Header -->
        <div class="grid grid-cols-[1fr_120px_150px_100px_100px] gap-4 px-6 py-4 border-b border-white/10 text-[11px] font-bold text-zinc-500 uppercase tracking-wider">
            <div>Nombre del {type === 'mods' ? 'Mod' : 'Plugin'}</div>
            <div>Versión</div>
            <div>Autor</div>
            <div>Tamaño</div>
            <div class="text-right">Estado</div>
        </div>

        <div class="flex-grow overflow-y-auto custom-scrollbar">
            {#if loading && addons.length === 0}
                <div class="divide-y divide-white/[0.03]">
                    {#each Array(8) as _}
                        <div class="grid grid-cols-[1fr_120px_150px_100px_100px] gap-4 px-6 py-4 animate-pulse items-center">
                            <div class="flex items-center gap-3">
                                <div class="w-8 h-8 bg-white/10 rounded-lg"></div>
                                <div class="h-4 bg-white/10 rounded w-48"></div>
                            </div>
                            <div class="h-3 bg-white/10 rounded w-16"></div>
                            <div class="h-3 bg-white/10 rounded w-20"></div>
                            <div class="h-3 bg-white/10 rounded w-12"></div>
                            <div class="justify-self-end w-12 h-6 bg-white/10 rounded-full"></div>
                        </div>
                    {/each}
                </div>
            {:else if addons.length > 0}
                <div class="divide-y divide-white/[0.03]">
                    {#each addons as addon}
                        <div class="group grid grid-cols-[1fr_120px_150px_100px_100px] gap-4 px-6 py-3 hover:bg-white/[0.03] items-center transition-colors">
                            <!-- Name & Icon -->
                            <div class="flex items-center gap-3 min-w-0">
                                <div class="w-8 h-8 shrink-0 bg-zinc-900 border border-white/5 rounded-lg flex items-center justify-center text-xs font-bold text-zinc-400">
                                    {addon.name.charAt(0).toUpperCase()}
                                </div>
                                <div class="min-w-0">
                                    <h4 class="font-medium text-sm text-zinc-200 truncate" title={addon.name}>{addon.name}</h4>
                                    <p class="text-[10px] text-zinc-600 truncate">{addon.file_name}</p>
                                </div>
                            </div>

                            <!-- Version -->
                            <div class="text-xs text-zinc-400 font-mono">
                                {addon.version}
                            </div>

                            <!-- Author -->
                            <div class="text-xs text-zinc-500 truncate">
                                {addon.author || '—'}
                            </div>

                            <!-- Size -->
                            <div class="text-xs text-zinc-600">
                                {formatSize(addon.size)}
                            </div>

                            <!-- Status & Actions -->
                            <div class="flex items-center justify-end gap-3">
                                <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <button class="p-1.5 rounded-md hover:bg-red-500/10 text-zinc-600 hover:text-red-400 transition-colors" title="Eliminar">
                                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                                    </button>
                                </div>
                                
                                <div class={`w-10 h-5 rounded-full relative transition-colors ${addon.enabled ? 'bg-green-500/20' : 'bg-zinc-800'}`}>
                                    <div class={`absolute top-1 w-3 h-3 rounded-full transition-all ${addon.enabled ? 'right-1 bg-green-500' : 'left-1 bg-zinc-600'}`}></div>
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="flex flex-col items-center justify-center h-64 text-center">
                    <div class="w-16 h-16 bg-white/5 rounded-full flex items-center justify-center mb-4 border border-white/5">
                        <span class="text-2xl opacity-30">📦</span>
                    </div>
                    <h3 class="text-lg font-bold text-white mb-1">No hay {type}</h3>
                    <p class="text-zinc-500 text-sm max-w-xs">
                        La carpeta está vacía. Añade archivos JAR para empezar.
                    </p>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .animate-fade-in {
        animation: fadeIn 0.4s ease-out;
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: rgba(255, 255, 255, 0.2);
    }
</style>
