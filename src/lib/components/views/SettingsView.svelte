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
    <div class="flex-1 overflow-y-auto w-full p-2">
        <!-- Panel de Idioma -->
        <div class="bg-black/20 border border-white/5 rounded-xl overflow-hidden m-4">
            <div class="px-6 py-4 border-b border-white/5 flex items-center justify-between">
                <div>
                    <h3 class="text-base font-semibold text-white">{$_('settings.language_title')}</h3>
                    <p class="text-sm text-zinc-400">{$_('settings.language_desc')}</p>
                </div>
            </div>
            <div class="p-6">
                <select 
                    bind:value={$locale} 
                    class="bg-[#141b29] border border-white/10 rounded-lg px-4 py-2 text-white outline-none focus:border-blue-500/50 w-full max-w-xs transition-colors"
                >
                    <option value="en">🇺🇸 English</option>
                    <option value="es">🇪🇸 Español</option>
                </select>
            </div>
        </div>

        <!-- Update Section -->
        <div class="bg-black/20 border border-white/5 rounded-xl overflow-hidden m-4">
            <div class="px-6 py-4 border-b border-white/5 flex items-center justify-between">
                <div>
                    <h3 class="text-base font-semibold text-white">{$_('settings.updates_title')}</h3>
                    <p class="text-sm text-zinc-400">{$_('settings.updates_desc')}</p>
                </div>
            </div>
            <div class="p-6 flex items-center gap-4">
                <button 
                    onclick={checkUpdates}
                    disabled={checking}
                    class="bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-800 disabled:text-zinc-500 text-white px-6 py-2 rounded-lg font-bold transition-all flex items-center gap-2 active:scale-95 shadow-lg shadow-blue-900/20"
                >
                    {#if checking}
                        <div class="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"></div>
                        {$_('settings.checking_updates')}
                    {:else}
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                            <polyline points="17 8 12 3 7 8"></polyline>
                            <line x1="12" y1="3" x2="12" y2="15"></line>
                        </svg>
                        {$_('settings.check_updates')}
                    {/if}
                </button>
                
                {#if lastCheck && !pendingUpdate && !checking}
                    <span class="text-xs text-zinc-500 italic">
                        {$_('settings.no_updates')}
                    </span>
                {/if}
            </div>
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
                        AnvilCraft <span class="text-blue-500 font-mono text-base ml-1">v{appState.appInfo.version} ({appState.appInfo.tag})</span>
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
