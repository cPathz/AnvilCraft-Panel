<script lang="ts">
  import Home from "$lib/components/views/Home.svelte";
  import { appState } from "$lib/runes/store.svelte";
</script>

{#if appState.selectedInstance}
  <div class="flex-1 w-full h-full p-4">
    <h1>Consola Activa</h1>
  </div>
{:else}
  <Home />
{/if}
