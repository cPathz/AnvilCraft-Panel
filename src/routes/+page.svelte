<script lang="ts">
  import CreateInstanceModal from "$lib/components/modals/CreateInstanceModal.svelte";
  import Home from "$lib/components/views/Home.svelte";
  import InstanceGallery from "$lib/components/views/InstanceGallery.svelte";
  import InstanceDetail from "$lib/components/views/InstanceDetail.svelte";
  import SettingsView from "$lib/components/views/SettingsView.svelte";
  import DevDashboard from "$lib/components/views/DevDashboard.svelte";
  import { appState } from "$lib/runes/store.svelte";
</script>

<!-- Main Content - Background is handled in +layout.svelte -->
<div class="relative z-10 w-full h-full">
  {#if appState.creatingInstance}
    <CreateInstanceModal />
  {/if}

  {#if appState.selectedInstance}
    {#key appState.selectedInstance.id}
      <InstanceDetail />
    {/key}
  {:else if appState.view === "instances"}
    <InstanceGallery />
  {:else if appState.view === "settings"}
    <SettingsView />
  {:else if appState.view === "dev"}
    <DevDashboard />
  {:else}
    <Home />
  {/if}
</div>
