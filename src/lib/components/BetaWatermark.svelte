<script>
    import { appState } from "$lib/runes/store.svelte";
    import { _ } from "svelte-i18n";
</script>

<div
    class="fixed bottom-6 right-6 pointer-events-none text-right opacity-30 z-[9999] select-none font-mono text-[15px] text-white mix-blend-difference flex flex-col items-end leading-tight gap-1"
    style="text-shadow: 0px 1px 1px rgba(0,0,0,0.5);"
>
    {#if appState.appInfo.isEvalCopy}
        <span class="font-bold tracking-widest uppercase"
            >{$_("settings.eval_copy_branding")}</span
        >
    {/if}
    <span>v{appState.appInfo.version} ({appState.appInfo.tag})</span>
    <span>Dev: cPathz</span>
</div>
