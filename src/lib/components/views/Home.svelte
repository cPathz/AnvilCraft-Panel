<script lang="ts">
    import es from "$lib/i18n/es.json";
</script>

<div
    class="absolute inset-0 flex flex-col items-center justify-center space-y-6 p-6"
>
    <h1 class="text-4xl font-bold text-white text-center select-none">
        {es.dashboard?.welcome || "Bienvenido a AnvilCraft"}
    </h1>

    <p class="text-xl text-zinc-400 text-center select-none">
        {es.home?.subtitle ||
            "Administra tus servidores de Minecraft fácilmente"}
    </p>

    <button
        class="bg-blue-600 hover:bg-blue-500 text-white px-8 py-3 rounded-lg font-bold shadow-lg transition-all hover:scale-105 active:scale-95"
    >
        {es.home?.create || "Crear Servidor"}
    </button>
</div>
