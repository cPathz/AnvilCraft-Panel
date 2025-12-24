<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";
    import es from "$lib/i18n/es.json";

    let selectedName = $derived(
        appState.selectedInstance?.name || es.dashboard.welcome,
    );
    let status = $derived(appState.selectedInstance?.state || "Offline");
</script>

<header
    class="flex items-center justify-between w-full h-16 pl-4 pr-8 bg-[#202020] bg-opacity-90 backdrop-blur-md border-b border-white/5 flex-shrink-0"
>
    <div class="flex items-center gap-3">
        {#if appState.selectedInstance}
            <h2 class="text-lg font-semibold text-white m-0">{selectedName}</h2>
            <span
                class="text-xs px-2 py-0.5 rounded bg-[#333] text-[#aaa]"
                class:bg-green-500-20={status === "Running"}
                class:text-green-500={status === "Running"}
                class:bg-red-500-20={status === "Stopped"}
                class:text-red-500={status === "Stopped"}
                style:background-color={status === "Running"
                    ? "rgba(35, 165, 89, 0.2)"
                    : status === "Stopped"
                      ? "rgba(235, 64, 52, 0.2)"
                      : "#333"}
                style:color={status === "Running"
                    ? "#23a559"
                    : status === "Stopped"
                      ? "#eb4034"
                      : "#aaa"}
            >
                {status}
            </span>
        {/if}
    </div>

    {#if appState.selectedInstance}
        <div class="flex gap-3">
            <!-- Start Button -->
            <button
                class="px-4 py-2 rounded-md font-medium transition-colors flex-shrink-0
                {status !== 'Running'
                    ? 'bg-green-600 hover:bg-green-500 text-white shadow-lg shadow-green-900/20'
                    : 'bg-zinc-700 text-zinc-500 cursor-not-allowed opacity-50'}"
                disabled={status === "Running"}
            >
                Iniciar
            </button>
            <!-- Stop Button -->
            <button
                class="px-4 py-2 rounded-md font-medium transition-colors flex-shrink-0
                {status === 'Running'
                    ? 'bg-red-600 hover:bg-red-500 text-white shadow-lg shadow-red-900/20'
                    : 'bg-zinc-700 text-zinc-500 cursor-not-allowed opacity-50'}"
                disabled={status !== "Running"}
            >
                Detener
            </button>
        </div>
    {/if}
</header>
