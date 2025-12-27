<script>
    import { getVersion } from "@tauri-apps/api/app";
    import { onMount } from "svelte";

    let version = $state("Dev");

    onMount(async () => {
        try {
            version = await getVersion();
        } catch (e) {
            console.error(e);
        }
    });
</script>

<div
    class="fixed bottom-6 right-6 pointer-events-none text-right opacity-30 z-[9999] select-none font-mono text-[15px] text-white mix-blend-difference flex flex-col items-end leading-tight gap-1"
    style="text-shadow: 0px 1px 1px rgba(0,0,0,0.5);"
>
    <span class="font-bold tracking-widest uppercase"
        >AnvilCraft Evaluation Copy</span
    >
    <span>Build {version} (Alpha)</span>
    <span>Dev: cPathz</span>
</div>
