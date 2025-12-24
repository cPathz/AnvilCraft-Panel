<script lang="ts">
    import es from "$lib/i18n/es.json";
    import { appState } from "$lib/runes/store.svelte";
</script>

<div class="w-full h-full bg-transparent">
    <style>
        /* Custom fadeInUp implementation */
        @keyframes fadeInUp {
            from {
                opacity: 0;
                transform: translate3d(0, 20px, 0);
            }
            to {
                opacity: 1;
                transform: translate3d(0, 0, 0);
            }
        }

        .animate-fade-in-up {
            animation: fadeInUp 0.8s ease-out forwards;
        }

        /* Stagger delays */
        .delay-100 {
            animation-delay: 0.1s;
            opacity: 0; /* Star invisible */
        }
        .delay-200 {
            animation-delay: 0.2s;
            opacity: 0;
        }
        .delay-300 {
            animation-delay: 0.3s;
            opacity: 0;
        }
    </style>

    <!-- Content Container -->
    <div
        class="relative flex flex-col items-center justify-center w-full h-full space-y-5 p-6 pb-[20vh] z-10"
    >
        <h1
            class="text-5xl font-extrabold text-white text-center tracking-tight drop-shadow-lg animate-fade-in-up delay-100 select-none hover:animate-pulse"
        >
            {es.dashboard?.welcome || "Bienvenido a AnvilCraft"}
        </h1>

        <p
            class="text-xl text-zinc-300 font-medium text-center max-w-lg leading-relaxed drop-shadow-md animate-fade-in-up delay-200 select-none"
        >
            {es.home?.subtitle ||
                "Administra tus servidores de Minecraft fácilmente"}
        </p>

        <button
            class="mt-4 bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-400 text-white px-8 py-3.5 rounded-xl font-bold shadow-lg shadow-black/40 transition-all active:scale-95 border border-white/10 animate-fade-in-up delay-300 cursor-pointer"
            onclick={() => (appState.creatingInstance = true)}
        >
            {es.home?.create || "Nueva Instancia"}
        </button>
    </div>
</div>
