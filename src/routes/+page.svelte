<script lang="ts">
  import { appState } from "$lib/runes/store.svelte";
  import Home from "$lib/components/views/Home.svelte";
  import InstanceView from "$lib/components/views/InstanceView.svelte";
</script>

<div class="h-full w-full">
  {#if appState.selectedInstance}
    <InstanceView instance={appState.selectedInstance} />
  {:else}
    <Home />
  {/if}
</div>
