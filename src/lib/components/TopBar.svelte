<script lang="ts">
    import { appState as store } from "$lib/runes/store.svelte";
    import es from "$lib/i18n/es.json"; // Asegúrate de importar esto si usas textos
</script>

<div
    class="flex items-center justify-between w-full h-16 pl-4 pr-8 bg-zinc-900/50 backdrop-blur-md border-b border-white/5"
>
    <div class="flex items-center gap-4">
        <h1 class="text-xl font-bold text-white tracking-wide">AnvilCRAFT</h1>

        {#if store.selectedInstance}
            <div class="h-6 w-px bg-zinc-700"></div>
            <div class="flex items-center gap-2">
                <span class="text-zinc-100 font-medium"
                    >{store.selectedInstance.name}</span
                >
                <span
                    class="text-xs px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700"
                >
                    {store.selectedInstance.state || "Desconocido"}
                </span>
            </div>
        {/if}
    </div>

    <div class="flex items-center gap-3">
        {#if store.selectedInstance}
            <button
                class="flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-zinc-800 text-zinc-400 transition-colors border border-transparent hover:border-zinc-700"
            >
                <span>⚙️</span>
                <span class="text-sm font-medium">Configuración</span>
            </button>

            <div
                class="flex items-center bg-zinc-800 rounded-md p-1 border border-zinc-700"
            >
                <button
                    class="px-4 py-1.5 rounded text-sm font-semibold transition-all flex items-center gap-2
                    {store.selectedInstance.state === 'Stopped'
                        ? 'bg-blue-600 text-white hover:bg-blue-500 shadow-lg shadow-blue-900/20'
                        : 'text-zinc-500 cursor-not-allowed'}"
                >
                    ▶ Iniciar
                </button>
            </div>
        {/if}
    </div>
</div>
