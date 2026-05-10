<script lang="ts">
    import { _ } from "svelte-i18n";
    import { fade, slide } from "svelte/transition";
    import { appState } from "$lib/runes/store.svelte";

    let instance = $derived(appState.selectedInstance);
    let runtime = $derived(instance ? appState.getRuntime(instance.id) : null);
    let issues = $derived(runtime?.issues || []);
</script>

<div class="h-full w-full flex flex-col min-h-0" in:fade={{ duration: 200 }}>
    {#if issues.length === 0}
        <div class="flex-1 flex flex-col items-center justify-center p-8 text-center">
            <!-- Icono de escudo con pulso sutil -->
            <div class="relative mb-6">
                <div class="absolute inset-0 bg-emerald-500/20 rounded-full blur-xl animate-pulse"></div>
                <div class="relative bg-zinc-900 border border-emerald-500/30 w-20 h-20 rounded-full flex items-center justify-center shadow-lg shadow-emerald-500/10">
                    <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-emerald-400">
                        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                        <path d="m9 12 2 2 4-4"/>
                    </svg>
                </div>
            </div>

            <!-- Texto Principal -->
            <h3 class="text-xl font-medium text-zinc-100 mb-2">
                {$_('errors.empty_title')}
            </h3>
            
            <!-- Subtítulo -->
            <p class="text-zinc-400 max-w-md text-sm leading-relaxed">
                {$_('errors.empty_subtitle')}
            </p>

            <!-- Decoración de fondo minimalista -->
            <div class="mt-12 flex items-center gap-2 opacity-20">
                <div class="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                <div class="w-32 h-px bg-gradient-to-r from-emerald-500 to-transparent"></div>
            </div>
        </div>
    {:else}
        <div class="flex-1 overflow-y-auto px-8 py-6 space-y-4 custom-scrollbar">
            {#each issues as log, i (i)}
                <div 
                    class="group bg-[#1e293b]/40 border border-red-500/10 hover:border-red-500/30 rounded-xl p-4 transition-all duration-200"
                    transition:slide|local={{ duration: 200 }}
                >
                    <div class="flex items-start gap-4">
                        <!-- Nivel/Icono -->
                        <div class="mt-1 shrink-0 w-8 h-8 rounded-lg bg-red-500/10 flex items-center justify-center text-red-400 border border-red-500/20">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
                        </div>

                        <!-- Contenido -->
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2 mb-1">
                                <span class="text-[10px] font-bold tracking-widest text-red-500 uppercase">
                                    {log.level}
                                </span>
                                <span class="text-[10px] text-zinc-500 font-mono">
                                    [{log.timestamp || 'N/A'}]
                                </span>
                                {#if log.plugin}
                                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 font-medium">
                                        {log.plugin}
                                    </span>
                                {/if}
                            </div>
                            <p class="text-zinc-200 text-sm leading-relaxed font-mono break-words">
                                {log.message}
                            </p>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.05);
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: rgba(255, 255, 255, 0.1);
    }
</style>
