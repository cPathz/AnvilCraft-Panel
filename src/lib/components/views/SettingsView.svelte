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
                        <span class="text-[11px] text-zinc-300 font-bold">cPathz</span>
                        <a
                            href="https://discord.com/users/cpathz"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-[#5865F2] hover:opacity-80 transition-opacity"
                        >
                            <svg viewBox="0 0 24 24" class="w-3 h-3" fill="currentColor">
                                <path d="M19.27 5.33C17.94 4.71 16.5 4.26 15 4a.09.09 0 0 0-.07.03c-.18.33-.39.76-.53 1.09a16.09 16.09 0 0 0-4.8 0c-.14-.34-.35-.76-.54-1.09c-.01-.02-.04-.03-.07-.03c-1.5.26-2.93.71-4.27 1.33c-.01 0-.02.01-.03.02c-2.72 4.07-3.47 8.03-3.1 11.95c0 .02.01.04.03.05c1.8 1.32 3.53 2.12 5.2 2.65c.03.01.06 0 .07-.02c.4-.55.76-1.13 1.07-1.74c.02-.04 0-.08-.04-.09c-.57-.22-1.11-.48-1.64-.78c-.04-.02-.04-.08.01-.11c.11-.08.22-.17.33-.25c.02-.02.05-.02.07-.01c3.44 1.57 7.15 1.57 10.55 0c.02-.01.05-.01.07.01c.11.09.22.17.33.26c.04.03.04.09 0 .11c-.52.31-1.07.56-1.64.78c-.04.02-.05.06-.03.09c.31.61.66 1.19 1.07 1.74c.02.02.05.02.07.02c1.67-.53 3.4-1.33 5.2-2.65c.02-.01.03-.03.03-.05c.44-4.53-.73-9.22-3.55-11.94a.06.06 0 0 0-.03-.02zM8.52 14.91c-1.03 0-1.89-.95-1.89-2.12s.84-2.12 1.89-2.12c1.06 0 1.9.96 1.89 2.12c0 1.17-.84 2.12-1.89 2.12zm6.97 0c-1.03 0-1.89-.95-1.89-2.12s.84-2.12 1.89-2.12c1.06 0 1.9.96 1.89 2.12c0 1.17-.85 2.12-1.89 2.12z" />
                            </svg>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
