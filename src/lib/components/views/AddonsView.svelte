<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";
    import { _ } from "svelte-i18n";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { listen } from "@tauri-apps/api/event";
    import { toast } from "$lib/runes/toast.svelte";
    import AddonInstallModal, { type AddonAnalysis } from "$lib/components/modals/AddonInstallModal.svelte";
    import ConfirmModal from "$lib/components/modals/ConfirmModal.svelte";
    import { onMount, onDestroy } from "svelte";

    interface Props {
        instance: Instance;
        loading?: boolean;
    }

    let { instance, loading = $bindable(true) }: Props = $props();
    let runtime = $derived(appState.getRuntime(instance.id));
    let type = $derived(runtime?.addonsType || 'none');
    let isLocked = $derived(instance.state === 'Running' || instance.state === 'Starting' || runtime?.state === 'Running' || runtime?.state === 'Starting');

    // UI state
    let addons = $state<any[]>([]);
    let analysisResults = $state<AddonAnalysis[]>([]);
    let showInstallModal = $state(false);
    let showConfirmDelete = $state(false);
    let addonToDelete = $state<any>(null);

    // Expose methods for parent
    export function refresh() {
        loadAddons(true);
    }
    export function openFolder() {
        openAddonsFolder();
    }

    export async function openAddDialog() {
        try {
            const selected = await open({
                multiple: true,
                filters: [{
                    name: 'Java Archive',
                    extensions: ['jar']
                }]
            });

            if (selected) {
                loading = true;
                const paths = Array.isArray(selected) ? selected : [selected];
                analysisResults = await invoke("analyze_instance_addons", { id: instance.id, sourcePaths: paths });
                showInstallModal = true;
            }
        } catch (e) {
            console.error(e);
            toast.error("Error al analizar complementos: " + e);
        } finally {
            loading = false;
        }
    }

    async function handleInstallConfirm(items: any[]) {
        showInstallModal = false;
        loading = true;
        try {
            await invoke("install_instance_addons", { id: instance.id, items });
            const installedCount = items.filter(i => i.action !== 'skip').length;
            if (installedCount > 0) {
                toast.success(`Se han instalado ${installedCount} complementos`);
                await loadAddons(true);
            }
        } catch (e) {
            console.error(e);
            toast.error("Error al instalar complementos: " + e);
        } finally {
            loading = false;
        }
    }

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

    async function toggleAddon(addon: any) {
        try {
            const newState = !addon.enabled;
            await invoke("toggle_instance_addon", { 
                id: instance.id, 
                fileName: addon.file_name, 
                enabled: newState 
            });
            // The file watcher will handle the refresh, but we update locally for speed
            addon.enabled = newState;
        } catch (e) {
            toast.error(e.toString());
        }
    }

    function requestDelete(addon: any) {
        addonToDelete = addon;
        showConfirmDelete = true;
    }

    async function handleConfirmDelete(deleteFolder: boolean) {
        if (!addonToDelete) return;
        showConfirmDelete = false;
        
        try {
            loading = true;
            await invoke("delete_instance_addon", { 
                id: instance.id, 
                fileName: addonToDelete.file_name, 
                deleteFolder 
            });
            toast.success($_('addons.toast_deleted'));
            addonToDelete = null;
        } catch (e) {
            toast.error($_('addons.toast_delete_error') + e);
        } finally {
            loading = false;
        }
    }

    function getStatusLabel(addon: any) {
        if (addon.enabled) return $_('addons.status_active');
        const fn = addon.file_name.toLowerCase();
        if (fn.endsWith(".bkp") || fn.endsWith(".bak")) return $_('addons.status_backup');
        if (fn.endsWith(".old") || fn.endsWith(".off")) return $_('addons.status_legacy');
        return $_('addons.status_disabled');
    }

    let unlisten: () => void;
    
    onMount(async () => {
        unlisten = await listen('addons-changed', () => {
            loadAddons();
        });
    });

    onDestroy(() => {
        if (unlisten) unlisten();
    });

    $effect(() => {
        if (type !== 'none') {
            loadAddons();
        }
    });

    async function openAddonsFolder() {
        try {
            await invoke("open_instance_addons_folder", { id: instance.id });
        } catch (e) {
            console.error(e);
        }
    }

    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }
</script>

<div class="flex flex-col h-full animate-fade-in">
    <!-- Content Area: List View -->
    <div class="flex-grow overflow-hidden flex flex-col bg-white/[0.02] border border-white/5 rounded-3xl">
        <!-- Table Header -->
        <div class="grid grid-cols-[1fr_100px_150px_100px_180px] gap-4 px-6 py-4 border-b border-white/10 text-[11px] font-bold text-zinc-500 uppercase tracking-wider">
            <div>{$_(`addons.col_name`)}</div>
            <div>{$_('addons.col_version')}</div>
            <div>{$_('addons.col_author')}</div>
            <div>{$_('addons.col_size')}</div>
            <div class="text-right">{$_('addons.col_status')}</div>
        </div>

        <div class="flex-grow overflow-y-auto custom-scrollbar">
            {#if loading && addons.length === 0}
                <div class="divide-y divide-white/[0.03]">
                    {#each Array(8) as _}
                        <div class="grid grid-cols-[1fr_100px_150px_100px_180px] gap-4 px-6 py-4 animate-pulse items-center">
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
                        <div class="group grid grid-cols-[1fr_100px_150px_100px_180px] gap-4 px-6 py-3 hover:bg-white/[0.03] items-center transition-colors">
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
                            <div class="flex items-center justify-end gap-4">
                                <div class="text-[10px] font-bold uppercase tracking-tighter transition-colors duration-300 {isLocked ? 'text-zinc-700' : addon.enabled ? 'text-emerald-500/50' : 'text-zinc-600'}">
                                    {getStatusLabel(addon)}
                                </div>

                                <div class="flex items-center gap-2 transition-all duration-300 {isLocked ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}">
                                    {#if isLocked}
                                        <div class="p-1.5 text-zinc-700 cursor-not-allowed" title={$_('addons.locked_tooltip')}>
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18.36 6.64a9 9 0 11-12.73 0M12 2v10"/></svg>
                                        </div>
                                    {:else}
                                        <button 
                                            onclick={() => requestDelete(addon)}
                                            class="p-1.5 rounded-md hover:bg-red-500/10 text-zinc-600 hover:text-red-400 transition-colors" 
                                            title={$_('addons.tooltip_delete', { values: { type: type === 'mods' ? 'mod' : 'plugin' } })}
                                        >
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                                        </button>
                                    {/if}
                                </div>
                                
                                <button 
                                    onclick={() => !isLocked && toggleAddon(addon)}
                                    disabled={isLocked}
                                    title={addon.enabled ? $_('addons.tooltip_disable', { values: { type: type === 'mods' ? 'mod' : 'plugin' } }) : $_('addons.tooltip_enable', { values: { type: type === 'mods' ? 'mod' : 'plugin' } })}
                                    class={`relative w-12 h-6 rounded-full transition-all duration-300 flex items-center overflow-hidden border-0 outline-none
                                        ${isLocked ? 'bg-zinc-800 opacity-40 cursor-not-allowed' : 
                                          addon.enabled ? 'bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.15)]' : 'bg-zinc-400'}`}
                                >
                                    <!-- Label Icon -->
                                    <div class={`absolute transition-all duration-300 pointer-events-none select-none flex items-center justify-center
                                        ${addon.enabled ? 'left-2.5 text-white/60' : 'right-2.5 text-white/60'}`}>
                                        {#if addon.enabled}
                                            <svg width="6" height="8" viewBox="0 0 6 10" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><path d="M3 2v6"/></svg>
                                        {:else}
                                            <svg width="8" height="8" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="5" cy="5" r="3.5"/></svg>
                                        {/if}
                                    </div>
                                    
                                    <!-- Slider Thumb -->
                                    <div class={`w-4 h-4 rounded-full bg-white shadow-sm transition-all duration-300 ease-in-out mx-1
                                        ${addon.enabled ? 'translate-x-6' : 'translate-x-0'}`}>
                                    </div>
                                </button>
                            </div>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="flex flex-col items-center justify-center h-64 text-center">
                    <div class="w-16 h-16 bg-white/5 rounded-full flex items-center justify-center mb-4 border border-white/5">
                        <span class="text-2xl opacity-30">📦</span>
                    </div>
                    <h3 class="text-lg font-bold text-white mb-1">{$_('addons.no_addons', { values: { type } })}</h3>
                    <p class="text-zinc-500 text-sm max-w-xs">
                        {$_('addons.no_addons_desc')}
                    </p>
                </div>
            {/if}
        </div>
    </div>
</div>

{#if showInstallModal}
    <AddonInstallModal 
        analysis={analysisResults} 
        onConfirm={handleInstallConfirm}
        onCancel={() => showInstallModal = false}
    />
{/if}

{#if showConfirmDelete && addonToDelete}
    <ConfirmModal 
        title={$_('addons.delete_modal_title')}
        message={$_('addons.delete_modal_msg', { values: { name: addonToDelete.name } })}
        confirmText={$_('addons.delete_modal_confirm')}
        cancelText={$_('instance_detail.btn_cancel')}
        type="danger"
        showCheckbox={true}
        checkboxLabel={$_('addons.delete_modal_checkbox')}
        onConfirm={handleConfirmDelete}
        onCancel={() => {
            showConfirmDelete = false;
            addonToDelete = null;
        }}
    />
{/if}

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
