<script lang="ts">
    import { _ } from "svelte-i18n";
    import { fade, scale } from "svelte/transition";

    export interface AddonAnalysis {
        source_path: string;
        name: string;
        version: string;
        status: string; // "valid", "duplicate", "update", "invalid"
        existing_filename: string | null;
        old_version: string | null;
        size: number;
        last_modified: number;
    }

    interface Props {
        analysis: AddonAnalysis[];
        onConfirm: (items: any[]) => void;
        onCancel: () => void;
    }

    let { analysis, onConfirm, onCancel }: Props = $props();

    // Local state for actions
    let items = $state(analysis.map(a => ({
        ...a,
        action: a.status === 'valid' ? 'install' : 
                a.status === 'update' ? 'replace' : 'skip'
    })));

    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    function confirm() {
        onConfirm(items.map(i => ({
            source_path: i.source_path,
            action: i.action,
            existing_filename: i.existing_filename
        })));
    }
</script>

<!-- Backdrop -->
<div class="fixed inset-0 z-[70] bg-black/80 backdrop-blur-md flex items-center justify-center p-6" transition:fade>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div 
        class="bg-[#18181b] w-full max-w-2xl rounded-3xl border border-white/10 shadow-2xl flex flex-col overflow-hidden" 
        transition:scale={{ duration: 200, start: 0.95 }}
        onclick={(e) => e.stopPropagation()}
        role="dialog"
    >
        <!-- Header -->
        <div class="p-6 border-b border-white/5">
            <h2 class="text-xl font-bold text-white">Instalar Complementos</h2>
            <p class="text-zinc-400 text-sm mt-1">Revisa los archivos antes de proceder con la instalación.</p>
        </div>

        <!-- List -->
        <div class="flex-1 overflow-y-auto max-h-[60vh] p-4 space-y-2 custom-scrollbar">
            {#each items as item}
                <div class={`p-4 rounded-2xl border transition-all ${
                    item.status === 'invalid' ? 'bg-red-500/5 border-red-500/10' :
                    item.status === 'duplicate' ? 'bg-orange-500/5 border-orange-500/10' :
                    item.status === 'duplicate_selection' ? 'bg-amber-500/10 border-amber-500/20' :
                    item.status === 'update' ? 'bg-blue-500/5 border-blue-500/10' :
                    item.status === 'update_selection' ? 'bg-indigo-500/10 border-indigo-500/20' :
                    'bg-white/[0.02] border-white/5'
                }`}>
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2">
                                <span class="font-bold text-white truncate text-sm">{item.name}</span>
                                {#if item.status !== 'invalid'}
                                    <span class="px-2 py-0.5 rounded-full bg-white/5 text-[10px] text-zinc-400 font-mono">v{item.version}</span>
                                    <span class="px-2 py-0.5 rounded-full bg-blue-500/10 text-[10px] text-blue-400 font-bold uppercase tracking-wider">{item.platform}</span>
                                {/if}
                            </div>
                            
                            <div class="mt-1 flex items-center gap-3 text-[10px]">
                                {#if item.size > 0}
                                    <span class="text-zinc-500">{formatSize(item.size)}</span>
                                {/if}
                                
                                {#if item.status === 'invalid'}
                                    <span class="text-red-400 font-bold uppercase tracking-tight">No es un complemento válido</span>
                                {:else if item.status === 'duplicate'}
                                    <span class="text-orange-400 font-bold uppercase tracking-tight">Ya existe en el servidor</span>
                                {:else if item.status === 'duplicate_selection'}
                                    <span class="text-amber-400 font-bold uppercase tracking-tight">Carga duplicada (Ya seleccionado)</span>
                                {:else if item.status === 'update'}
                                    <div class="flex items-center gap-1 text-blue-400">
                                        <span class="font-bold uppercase tracking-tight">Actualización disponible</span>
                                        <span class="text-zinc-500 font-mono">(v{item.old_version} &rarr; v{item.version})</span>
                                    </div>
                                {:else if item.status === 'update_selection'}
                                    <div class="flex items-center gap-1 text-indigo-400">
                                        <span class="font-bold uppercase tracking-tight">Versión diferente en selección</span>
                                        <span class="text-zinc-500 font-mono">(v{item.old_version} &rarr; v{item.version})</span>
                                    </div>
                                {/if}
                            </div>
                        </div>

                        <!-- Actions -->
                        <div class="flex gap-2 shrink-0">
                            {#if item.status === 'invalid'}
                                <span class="px-3 py-1.5 text-xs text-red-500/50 font-bold">Omitido</span>
                            {:else if item.status === 'duplicate' || item.status === 'duplicate_selection'}
                                <div class="flex bg-white/5 rounded-xl p-1">
                                    <button 
                                        onclick={() => item.action = 'skip'}
                                        class={`px-3 py-1 rounded-lg text-[11px] font-bold transition-all ${item.action === 'skip' ? 'bg-orange-500 text-white shadow-lg' : 'text-zinc-500 hover:text-zinc-300'}`}
                                    >Omitir</button>
                                    <button 
                                        onclick={() => item.action = 'replace'}
                                        class={`px-3 py-1 rounded-lg text-[11px] font-bold transition-all ${item.action === 'replace' ? 'bg-white/10 text-white' : 'text-zinc-500 hover:text-zinc-300'}`}
                                    >Sobreescribir</button>
                                </div>
                            {:else if item.status === 'update' || item.status === 'update_selection'}
                                <div class="flex bg-white/5 rounded-xl p-1">
                                    <button 
                                        onclick={() => item.action = 'replace'}
                                        class={`px-3 py-1 rounded-lg text-[11px] font-bold transition-all ${item.action === 'replace' ? 'bg-blue-500 text-white shadow-lg' : 'text-zinc-500 hover:text-zinc-300'}`}
                                    >Actualizar</button>
                                    <button 
                                        onclick={() => item.action = 'skip'}
                                        class={`px-3 py-1 rounded-lg text-[11px] font-bold transition-all ${item.action === 'skip' ? 'bg-white/10 text-white' : 'text-zinc-500 hover:text-zinc-300'}`}
                                    >Ignorar</button>
                                </div>
                            {:else}
                                <span class="px-3 py-1.5 text-xs text-green-400 font-bold uppercase tracking-tight">Nuevo</span>
                            {/if}
                        </div>
                    </div>
                </div>
            {/each}
        </div>

        <!-- Footer -->
        <div class="p-6 bg-[#121214] border-t border-white/5 flex justify-between items-center">
            <div class="text-xs text-zinc-500 font-medium">
                {items.filter(i => i.action !== 'skip' && i.status !== 'invalid').length} archivos por instalar
            </div>
            <div class="flex gap-3">
                <button 
                    onclick={onCancel}
                    class="px-5 py-2.5 rounded-2xl font-bold text-zinc-400 hover:text-white hover:bg-white/5 transition-all text-sm"
                >
                    Cancelar
                </button>
                <button 
                    onclick={confirm}
                    class="px-8 py-2.5 rounded-2xl font-bold bg-blue-500 hover:bg-blue-600 text-white shadow-xl shadow-blue-500/20 transition-all active:scale-95 text-sm"
                >
                    Confirmar
                </button>
            </div>
        </div>
    </div>
</div>

<style>
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
