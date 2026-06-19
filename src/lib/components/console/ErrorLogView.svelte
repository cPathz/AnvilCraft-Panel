<script lang="ts">
    import { _ } from "svelte-i18n";
    import { fade } from "svelte/transition";
    import { appState } from "$lib/runes/store.svelte";

    let instance = $derived(appState.selectedInstance);
    let runtime = $derived(instance ? appState.getRuntime(instance.id) : null);
    let issues = $derived(runtime?.issues || []);

    function getLevelColor(level: string) {
        switch (level) {
            case 'ERROR':
            case 'FATAL':
                return 'text-red-400 bg-red-400/10 border-red-500/20';
            case 'WARN':
            case 'WARNING':
                return 'text-yellow-400 bg-yellow-400/10 border-yellow-500/20';
            default:
                return 'text-zinc-400 bg-zinc-400/10 border-zinc-500/20';
        }
    }
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
        </div>
    {:else}
        <!-- Sticky Header with Clear Button -->
        <div class="sticky top-0 z-20 bg-[#1e293b]/80 backdrop-blur-md px-8 py-4 border-b border-white/5 flex items-center justify-between shadow-sm">
            <div class="flex flex-col gap-0.5">
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
                    <div class="flex items-center gap-2">
                        <h2 class="text-xl font-bold text-white tracking-tight">
                            {$_("instance_detail.tab_errors")}
                        </h2>
                        <span class="px-2 py-1 rounded text-[11px] font-black bg-amber-500/10 text-amber-500 border border-amber-500/20 uppercase tracking-widest">
                            {$_("common.experimental")}
                        </span>
                    </div>
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-red-500/10 text-red-400 font-bold border border-red-500/20">
                        {issues.length}
                    </span>
                </div>
                <p class="text-[11px] text-zinc-500 italic">
                    {$_('errors.table_intro')}
                </p>
            </div>

            <button 
                onclick={() => { if (instance) appState.clearIssues(instance.id) }}
                class="group flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold text-zinc-400 hover:text-red-400 hover:bg-red-400/10 transition-all border border-transparent hover:border-red-400/20 active:scale-95"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="group-hover:rotate-12 transition-transform"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
                {$_('instance_detail.btn_clear_errors')}
            </button>
        </div>

        <div class="flex-1 overflow-x-auto overflow-y-auto custom-scrollbar relative">
            <table class="w-full text-left border-collapse min-w-[800px]">
                <thead class="sticky top-0 z-10 bg-[#0f172a] shadow-sm">
                    <tr>
                        <th class="px-6 py-3 text-[10px] font-bold text-zinc-500 uppercase tracking-wider border-b border-white/5 w-16">{$_('errors.col_id')}</th>
                        <th class="px-6 py-3 text-[10px] font-bold text-zinc-500 uppercase tracking-wider border-b border-white/5 w-24">{$_('errors.col_type')}</th>
                        <th class="px-6 py-3 text-[10px] font-bold text-zinc-500 uppercase tracking-wider border-b border-white/5 w-32">{$_('errors.col_time')}</th>
                        <th class="px-6 py-3 text-[10px] font-bold text-zinc-500 uppercase tracking-wider border-b border-white/5 w-40">{$_('errors.col_origin')}</th>
                        <th class="px-6 py-3 text-[10px] font-bold text-zinc-500 uppercase tracking-wider border-b border-white/5">{$_('errors.col_message')}</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-white/5 bg-[#1e293b]/20">
                    {#each issues as log, i (i)}
                        <tr class="hover:bg-white/5 transition-colors group">
                            <td class="px-6 py-4 text-xs font-mono text-zinc-600 group-hover:text-zinc-400">
                                {String(i + 1).padStart(3, '0')}
                            </td>
                            <td class="px-6 py-4">
                                <span class="text-[10px] px-2 py-0.5 rounded-full font-bold border {getLevelColor(log.level)}">
                                    {log.level}
                                </span>
                            </td>
                            <td class="px-6 py-4 text-[11px] font-mono text-zinc-500 group-hover:text-zinc-300">
                                {log.timestamp || '--:--:--'}
                            </td>
                            <td class="px-6 py-4">
                                {#if log.plugin}
                                    <span class="text-[10px] px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 font-bold border border-white/5 group-hover:border-blue-500/30 transition-colors">
                                        {log.plugin}
                                    </span>
                                {:else}
                                    <span class="text-[10px] px-2 py-0.5 rounded bg-zinc-900/50 text-zinc-600 font-medium border border-transparent">
                                        {$_('errors.origin_system')}
                                    </span>
                                {/if}
                            </td>
                            <td class="px-6 py-4">
                                <p class="text-xs font-mono text-zinc-300 leading-relaxed break-all max-w-2xl">
                                    {log.message}
                                </p>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
        height: 6px;
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
    
    table {
        border-spacing: 0;
    }
</style>
