<script lang="ts">
    import { type Instance } from "$lib/runes/store.svelte";

    let { instance }: { instance: Instance } = $props();
</script>

<div class="flex flex-col items-center justify-center h-full text-white">
    <h2 class="text-2xl font-bold mb-4">{instance.name}</h2>
    <p class="text-zinc-400">Consola y detalles de la instancia irán aquí.</p>
</div>
