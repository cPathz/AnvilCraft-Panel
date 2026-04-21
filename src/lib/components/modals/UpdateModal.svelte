<script lang="ts">
    import { _ } from "svelte-i18n";
    import { check } from "@tauri-apps/plugin-updater";
    import { relaunch } from "@tauri-apps/plugin-process";
    import { toast } from "$lib/runes/toast.svelte";

    let { update = $bindable(), onDone } = $props<{
        update: any;
        onDone: () => void;
    }>();

    let downloading = $state(false);
    let downloadProgress = $state(0);
    let contentLength = $state<number | undefined>(0);

    async function handleInstall() {
        if (!update) return;
        
        try {
            downloading = true;
            let downloaded = 0;
            
            await update.downloadAndInstall((event) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength;
                        console.log(`started downloading ${event.data.contentLength} bytes`);
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        if (contentLength) {
                            downloadProgress = Math.round((downloaded / contentLength) * 100);
                        }
                        break;
                    case 'Finished':
                        console.log('download finished');
                        break;
                }
            });

            toast.success($_('settings.install_update'));
            await relaunch();
        } catch (e) {
            console.error(e);
            toast.error($_('settings.update_error') + ": " + e);
            downloading = false;
        }
    }
</script>

<div class="fixed inset-0 z-[1000] flex items-center justify-center p-4 bg-black/60 backdrop-blur-md">
    <div class="bg-[#141b29] border border-white/10 rounded-2xl w-full max-w-md overflow-hidden shadow-2xl animate-in fade-in zoom-in duration-300">
        <!-- Header -->
        <div class="px-6 py-5 border-b border-white/5 bg-gradient-to-br from-blue-500/10 to-transparent">
            <h3 class="text-xl font-bold text-white flex items-center gap-3">
                <div class="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                </div>
                {$_('settings.update_available')}
            </h3>
        </div>

        <!-- Body -->
        <div class="p-6 space-y-6">
            <div class="space-y-2">
                <div class="flex items-center justify-between">
                    <span class="text-sm font-medium text-zinc-400">Version {update?.version}</span>
                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-blue-500/20 text-blue-400 uppercase tracking-wider">NEW</span>
                </div>
                <p class="text-zinc-300 text-sm leading-relaxed">
                    {$_('settings.update_available_desc')}
                </p>
            </div>

            {#if downloading}
                <div class="space-y-3">
                    <div class="flex justify-between text-xs font-medium">
                        <span class="text-blue-400">{$_('settings.downloading_update')}</span>
                        <span class="text-zinc-500">{downloadProgress}%</span>
                    </div>
                    <div class="w-full h-2 bg-white/5 rounded-full overflow-hidden border border-white/5">
                        <div 
                            class="h-full bg-gradient-to-r from-blue-600 to-blue-400 transition-all duration-300 ease-out"
                            style="width: {downloadProgress}%"
                        ></div>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Footer -->
        <div class="px-6 py-4 bg-black/20 border-t border-white/5 flex items-center justify-end gap-3">
            {#if !downloading}
                <button 
                    onclick={onDone}
                    class="px-4 py-2 text-sm font-medium text-zinc-400 hover:text-white transition-colors"
                >
                    {$_('instance_detail.btn_cancel')}
                </button>
                <button 
                    onclick={handleInstall}
                    class="px-5 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-bold rounded-lg transition-all shadow-lg shadow-blue-900/20 active:scale-95"
                >
                    {$_('settings.install_update')}
                </button>
            {:else}
                <div class="flex items-center gap-2 text-zinc-500 text-xs font-medium py-2">
                    <div class="w-4 h-4 border-2 border-zinc-500/20 border-t-zinc-500 rounded-full animate-spin"></div>
                    Procesando...
                </div>
            {/if}
        </div>
    </div>
</div>
