<script lang="ts">
    import NavigationRail from "$lib/components/NavigationRail.svelte";
    import TopBar from "$lib/components/TopBar.svelte";
    import "../app.css";

    import { appState } from "$lib/runes/store.svelte";

    let { children } = $props();
</script>

<div class="flex h-screen w-screen bg-zinc-900 text-white">
    <NavigationRail />

    <div class="flex-1 flex flex-col min-w-0 relative">
        {#if appState.selectedInstance}
            <div class="flex-none h-16 z-20">
                <TopBar />
            </div>
        {/if}

        <div class="flex