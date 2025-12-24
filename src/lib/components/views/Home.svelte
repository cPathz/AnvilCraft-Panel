<script lang="ts">
    import es from "$lib/i18n/es.json";
</script>

<div
    class="flex flex-col items-center justify-center h-full text-center space-y-6"
>
    <h1 class="text-4xl font-bold text-white">{es.dashboard.welcome}</h1>
    <p class="text-xl text-zinc-400">{es.home.subtitle}</p>

    <button
        class="bg-blue-600 hover:bg-blue-500 text-white px-6 py-3 rounded-lg font-semibold shadow-lg transition-all hover:scale-105 active:scale-95"
    >
        {es.home.create}
    </button>
</div>
