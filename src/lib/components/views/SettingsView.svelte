<script lang="ts">
    import AppearanceSettings from "./AppearanceSettings.svelte";
    import UpdateModal from "../modals/UpdateModal.svelte";
    import { appState } from "$lib/runes/store.svelte";
    import { locale, _ } from "svelte-i18n";
    import { check } from "@tauri-apps/plugin-updater";
    import { toast } from "$lib/runes/toast.svelte";

    let checking = $state(false);
    let pendingUpdate = $state<any>(null);
    let lastCheck = $state(false);

    async function checkUpdates() {
        try {
            checking = true;
            lastCheck = false;
            const update = await check();
            if (update) {
                pendingUpdate = update;
            } else {
                lastCheck = true;
            }
        } catch (e) {
            console.error(e);
            toast.error($_('settings.update_error') + ": " + e);
        } finally {
            checking = false;
        }
    }
</script>

<div class="h-full flex flex-col bg-[#192232]">
    <!-- Header -->
    <div
        class="flex-none bg-[#141b29] border-b border-white/5 px-6 py-4 flex items-center justify-between"
        data-tauri-drag-region
    >
        <div class="flex items-center gap-4 pointer-events-none">
            <div
                class="w-12 h-12 rounded-lg bg-white/5 flex items-center justify-center border border-white/10"
            >
                <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="text-zinc-400"
                    ><circle cx="12" cy="12" r="3"></circle><path
                        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
                    ></path></svg
                >
            </div>
            <div>
                <h2 class="text-lg font-bold text-white tracking-tight">
                    {$_('settings.global_title')}
                </h2>
                <div class="text-xs text-zinc-400">
                    {$_('settings.global_desc')}
                </div>
            </div>
        </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto w-full p-6 space-y-4">
        <!-- Panel de Idioma -->
        <div class="bg-black/20 border border-white/5 rounded-2xl overflow-hidden group hover:border-white/10 transition-all">
            <div class="px-6 py-5 flex items-center justify-between gap-8 bg-white/[0.01]">
                <!-- Info -->
                <div class="flex items-center gap-4 min-w-0">
                    <div class="w-10 h-10 flex-none rounded-xl bg-blue-500/10 flex items-center justify-center border border-blue-500/20">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line>
                            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                        </svg>
                    </div>
                    <div class="min-w-0">
                        <h3 class="text-base font-bold text-white truncate">{$_('settings.language_title')}</h3>
                        <p class="text-sm text-zinc-400 truncate">{$_('settings.language_desc')}</p>
                    </div>
                </div>

                <!-- Control -->
                <div class="relative w-full max-w-[240px] flex-none">
                    <select 
                        bind:value={$locale} 
                        class="appearance-none bg-[#141b29] border border-white/10 rounded-xl px-4 py-3 text-sm text-white outline-none focus:border-blue-500/50 w-full transition-all hover:bg-white/[0.03] cursor-pointer"
                    >
                        <option value="en">🇺🇸 English</option>
                        <option value="es">🇪🇸 Español</option>
                    </select>
                    <div class="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-zinc-500">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
                    </div>
                </div>
            </div>
        </div>

        <!-- Update Section -->
        <div class="bg-black/20 border border-white/5 rounded-2xl overflow-hidden group hover:border-white/10 transition-all">
            <div class="px-6 py-5 flex items-center justify-between gap-8 bg-white/[0.01]">
                <!-- Info -->
                <div class="flex items-center gap-4 min-w-0">
                    <div class="w-10 h-10 flex-none rounded-xl bg-amber-500/10 flex items-center justify-center border border-amber-500/20">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                            <polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line>
                        </svg>
                    </div>
                    <div class="min-w-0">
                        <h3 class="text-base font-bold text-white truncate">{$_('settings.updates_title')}</h3>
                        <p class="text-sm text-zinc-400 truncate">{$_('settings.updates_desc')}</p>
                    </div>
                </div>

                <!-- Controls -->
                <div class="flex items-center gap-4 flex-none">
                    <button 
                        onclick={checkUpdates}
                        disabled={checking}
                        class="bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-800/50 disabled:text-zinc-600 text-white px-6 py-3 rounded-xl font-bold text-sm transition-all flex items-center gap-2 active:scale-95 shadow-lg shadow-blue-900/20"
                    >
                        {#if checking}
                            <div class="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"></div>
                            {$_('settings.checking_updates')}
                        {:else}
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
                            {$_('settings.check_updates')}
                        {/if}
                    </button>

                    <!-- Manual Mode Box (Zen Mode) -->
                    <div class="flex items-center gap-6 p-3 bg-white/5 rounded-2xl border border-white/5 hover:bg-white/[0.08] transition-all max-w-[280px]">
                        <div>
                            <h4 class="text-[13px] font-bold text-white leading-none">Modo Zen</h4>
                            <p class="text-[10px] text-zinc-500 mt-1 leading-tight">Solo avisar una vez por versión al iniciar.</p>
                        </div>
                        <button 
                            class="relative w-10 h-5.5 rounded-full flex-none transition-colors duration-200 outline-none {appState.settings.manualUpdate ? 'bg-amber-500 shadow-[0_0_10px_rgba(245,158,11,0.2)]' : 'bg-zinc-700'}"
                            onclick={() => appState.settings.manualUpdate = !appState.settings.manualUpdate}
                        >
                            <div class="absolute top-1 left-1 w-3.5 h-3.5 bg-white rounded-full shadow-md transition-transform duration-200 {appState.settings.manualUpdate ? 'translate-x-4.5' : ''}"></div>
                        </button>
                    </div>
                </div>
            </div>
            
            {#if lastCheck && !pendingUpdate && !checking}
                <div class="px-6 pb-4 -mt-2">
                    <p class="text-xs text-zinc-500 italic">{$_('settings.no_updates')}</p>
                </div>
            {/if}
        </div>

        {#if pendingUpdate}
            <UpdateModal 
                bind:update={pendingUpdate} 
                onDone={() => pendingUpdate = null} 
            />
        {/if}

        <AppearanceSettings />

        <!-- Ultra-Compact Modern Footer -->
        <div class="mt-12 pb-24 flex flex-col items-center justify-center text-center">
            <div class="space-y-4">
                <!-- App & Version -->
                <div class="flex flex-col gap-1">
                    <h2 class="text-xl font-black text-white/90 tracking-tight">
                        AnvilCraft <span class="text-blue-500 font-mono text-base ml-1">v.{appState.appInfo.version} ({appState.appInfo.tag})</span>
                    </h2>
                </div>

                <!-- Compact Badge & Dev -->
                <div class="flex items-center gap-4 py-2 px-5 rounded-lg bg-white/[0.02] border border-white/[0.05]">
                    <span class="text-[9px] font-black text-yellow-500/80 uppercase tracking-widest leading-none">
                         {$_('settings.eval_copy_branding')}
                    </span>
                    <div class="w-px h-3 bg-white/10"></div>
                    <div class="flex items-center gap-2">
                        <span class="text-[9px] text-zinc-500 uppercase tracking-tighter">{$_('settings.developed_by')}</span>
                        <a
                            href="https://discord.com/users/cpathz"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-zinc-300 hover:text-white transition-colors text-[11px] font-bold"
                        >
                            cPathz
                        </a>
                        
                        <div class="w-px h-3 bg-white/10 ml-1"></div>

                        <a
                            href="https://discord.gg/E4PFVUe8vz"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-zinc-500 hover:text-white transition-colors flex items-center gap-1.5 ml-1"
                            title="Discord Server"
                        >
                            <span class="text-[9px] font-black text-blue-500/80 uppercase tracking-widest leading-none">
                                {$_('addons.community')}
                            </span>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
