<script lang="ts">
    import NavigationRail from "$lib/components/NavigationRail.svelte";
    import TopBar from "$lib/components/TopBar.svelte";
    import "../app.css";

    let { children } = $props();
</script>

<div class="app-container">
    <NavigationRail />

    <div class="main-content">
        <TopBar />

        <div class="content-body">
            {@render children()}
        </div>
    </div>
</div>

<style>
    :global(body) {
        margin: 0;
        padding: 0;
        background-color: #313338; /* Discord dark background */
        color: #dbdee1;
        font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
        overflow: hidden; /* Prevent body scroll, handle internally */
    }

    .app-container {
        display: flex;
        width: 100vw;
        height: 100vh;
        overflow: hidden;
    }

    .main-content {
        flex: 1;
        display: flex;
        flex-direction: column;
        height: 100%;
        min-width: 0; /* Prevent flex overflow */
    }

    .content-body {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        position: relative;
        padding: 16px;
    }
</style>
