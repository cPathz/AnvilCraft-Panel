<script lang="ts">
    import NavigationRail from "$lib/components/NavigationRail.svelte";
    import TopBar from "$lib/components/TopBar.svelte";
    import "../app.css";

    let { children } = $props();
</script>

<div class="flex h-screen w-screen overflow-hidden bg-zinc-900 text-white">
    <NavigationRail />

    <div class="flex-1 flex flex-col h-full min-w-0 overflow-hidden relative">
        <div class="flex-none h-16 z-20">
            <TopBar />
        </div>

        <div
            class="flex-1 relative w-full h-full overflow-y-auto z-10 flex flex-col"
        >
            {@render children()}
        </div>
    </div>
</div>
