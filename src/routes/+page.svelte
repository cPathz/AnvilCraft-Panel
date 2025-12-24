<script lang="ts">
  import CreateInstanceModal from "$lib/components/modals/CreateInstanceModal.svelte";
  import Home from "$lib/components/views/Home.svelte";
  import InstanceGallery from "$lib/components/views/InstanceGallery.svelte";
  import { appState } from "$lib/runes/store.svelte";
</script>

<!-- Main Content - Background is handled in +layout.svelte -->
<div class="relative z-10 w-full h-full">
  {#if appState.creatingInstance}
    <CreateInstanceModal />
  {/if}

  {#if appState.selectedInstance}
    <div class="flex-1 w-full h-full p-4 text-white">
      <h1>Consola Activa</h1>
    </div>
  {:else if appState.view === "instances"}
    <InstanceGallery />
  {:else}
    <Home />
  {/if}
</div>
