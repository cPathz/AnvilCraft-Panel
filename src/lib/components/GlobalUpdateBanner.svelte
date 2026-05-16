<script lang="ts">
    import { onMount } from "svelte";
    import { check } from "@tauri-apps/plugin-updater";
    import { relaunch } from "@tauri-apps/plugin-process";
    import { toast } from "$lib/runes/toast.svelte";
    import { _ } from "svelte-i18n";
    import { appState } from "$lib/runes/store.svelte";

    let update = $state<any>(null);
    let isForced = $state(false);
    let releaseNotes = $state("");
    let show = $state(false);
    
    let downloading = $state(false);
    let downloadProgress = $state(0);
    let contentLength = $state<number | undefined>(0);

    // Reactive effect to sync with global appState
    $effect(() => {
        if (appState.updateData) {
            const data = appState.updateData;
            update = data.rawUpdate;
            isForced = data.isCritical;
            
            let body = data.body;
            // Extract fun in-app notes if APP_NOTES tags are present
            const appNotesMatch = body.match(/\[APP_NOTES\]([\s\S]*?)\[\/APP_NOTES\]/);
            if (appNotesMatch && appNotesMatch[1]) {
                body = appNotesMatch[1].trim();
            }
            releaseNotes = body;

            // Distribution logic
            const isStore = appState.appInfo.distChannel === 'msix';
            if (isStore && !isForced) {
                show = false;
                return;
            }

            // Zen Mode Logic
            if (appState.settings.manualUpdate && !isForced) {
                if (data.version !== appState.settings.lastIgnoredVersion) {
                    show = true;
                }
            } else {
                show = true;
            }
        } else {
            show = false;
        }
    });

    async function handleInstall() {
        if (!update) return;
        
        try {
            downloading = true;
            let downloaded = 0;
            
            await update.downloadAndInstall((event: any) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength;
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        if (contentLength) {
                            downloadProgress = Math.round((downloaded / contentLength) * 100);
                        }
                        break;
                    case 'Finished':
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

    function dismiss() {
        if (!isForced) {
            // Save this version as ignored so it doesn't pop up again in Zen Mode
            appState.settings.lastIgnoredVersion = update.version;
            show = false;
        }
    }
</script>

{#if show}
    <!-- Background overlay if forced (blocks UI), otherwise pointer-events-none -->
    {#if isForced}
        <div class="fixed inset-0 z-[9998] bg-black/50 backdrop-blur-sm"></div>
    {/if}

    <!-- Floating Banner/Card (Top Right) -->
    <div 
        class="fixed top-6 right-6 z-[9999] w-[400px] bg-[#141b29] border border-white/10 rounded-xl shadow-2xl overflow-hidden flex flex-col animate-in slide-in-from-top-8 fade-in duration-500 ease-out"
        style={!isForced ? "" : "pointer-events: auto; box-shadow: 0 25px 50px -12px rgba(239, 68, 68, 0.25); border-color: rgba(239, 68, 68, 0.3);"}
    >
        <!-- Header -->
        <div class="px-5 py-4 border-b border-white/5 {isForced ? 'bg-gradient-to-r from-red-500/20 to-transparent' : 'bg-gradient-to-r from-blue-500/10 to-transparent'} flex items-center justify-between">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-full {isForced ? 'bg-red-500' : 'bg-blue-500'} flex items-center justify-center shadow-lg shadow-black/50">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                </div>
                <div>
                    <h3 class="text-white font-bold tracking-tight">
                        {isForced ? 'Actualización Crítica' : '¡Actualización Disponible!'}
                    </h3>
                    <div class="text-xs text-zinc-400 font-medium">
                        Versión v.{update.version}
                    </div>
                </div>
            </div>
            
            {#if isForced}
                <span class="px-2 py-0.5 rounded text-[10px] font-black bg-red-500/20 text-red-400 uppercase tracking-wider animate-pulse">REQUIRED</span>
            {:else}
                <span class="px-2 py-0.5 rounded text-[10px] font-black bg-blue-500/20 text-blue-400 uppercase tracking-wider">NEW</span>
            {/if}
        </div>

        <!-- Body (Changelog) -->
        <div class="p-5 flex flex-col gap-3">
            <p class="text-sm text-zinc-300">
                {#if appState.appInfo.distChannel === 'msix'}
                    Esta es una actualización crítica requerida por seguridad. Por favor, abre la <b>Microsoft Store</b> para descargar la última versión.
                {:else}
                    {isForced ? 'Esta es una actualización de seguridad obligatoria. Por favor, instala la nueva versión para continuar.' : 'Hemos mejorado AnvilCraft. Esto es lo nuevo en esta versión:'}
                {/if}
            </p>
            
            <div class="bg-black/30 rounded-lg border border-white/5 p-3 max-h-[120px] overflow-y-auto custom-scrollbar">
                <pre class="text-xs text-zinc-400 font-mono whitespace-pre-wrap leading-relaxed font-medium">{releaseNotes}</pre>
            </div>

            {#if downloading}
                <div class="mt-2 space-y-2">
                    <div class="flex justify-between text-[11px] font-medium uppercase tracking-wider">
                        <span class="text-blue-400">Descargando...</span>
                        <span class="text-zinc-500">{downloadProgress}%</span>
                    </div>
                    <div class="w-full h-1.5 bg-black/40 rounded-full overflow-hidden border border-white/5">
                        <div 
                            class="h-full {isForced ? 'bg-red-500' : 'bg-blue-500'} transition-all duration-300 ease-out"
                            style="width: {downloadProgress}%"
                        ></div>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Footer / Actions -->
        <div class="px-5 py-3 bg-black/40 border-t border-white/5 flex flex-col gap-2">
            <div class="flex items-center gap-2 w-full">
                {#if !isForced && !downloading}
                    <button 
                        onclick={dismiss}
                        class="flex-1 py-2 text-xs font-medium text-zinc-400 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
                    >
                        Quizás después
                    </button>
                {/if}
                
                {#if appState.appInfo.distChannel === 'msix'}
                    <div class="flex-1 py-2 bg-zinc-800 text-zinc-400 text-xs font-bold rounded-lg text-center border border-white/5">
                        Esperando actualización de la Store...
                    </div>
                {:else}
                    <button 
                        onclick={handleInstall}
                        disabled={downloading}
                        class="flex-1 py-2 {isForced ? 'bg-red-600 hover:bg-red-500 shadow-red-900/20 w-full' : 'bg-blue-600 hover:bg-blue-500 shadow-blue-900/20'} text-white text-xs font-bold rounded-lg transition-all shadow-lg active:scale-95 disabled:opacity-50 disabled:pointer-events-none"
                    >
                        {#if downloading}
                            Procesando...
                        {:else}
                            Instalar y Reiniciar
                        {/if}
                    </button>
                {/if}
            </div>
            
            {#if !isForced && !downloading}
                <p class="text-[10px] text-zinc-500 text-center italic mt-1">
                    Puedes actualizar después desde la configuración global.
                </p>
            {/if}
        </div>
    </div>
{/if}

<style>
    /* Custom thin scrollbar for the changelog area */
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
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
