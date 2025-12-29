<script lang="ts">
    import InstanceSettings from "./InstanceSettings.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    let appVersion = "";
    onMount(() => {
        invoke<string>("get_app_version").then((v) => (appVersion = v));
    });
</script>

<div class="h-full flex flex-col bg-[#192232]">
    <!-- Header -->
    <div
        class="flex-none bg-[#141b29] border-b border-white/5 px-6 py-4 flex items-center justify-between"
        data-tauri-drag-region
    >
        <div class="flex items-center gap-4 pointer-events-none">
            <div
                class="w-12 h-12 rounded-lg bg-white/5 flex items-center justify-center border border-white/10"
            >
                <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="text-zinc-400"
                    ><circle cx="12" cy="12" r="3"></circle><path
                        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
                    ></path></svg
                >
            </div>
            <div>
                <h2 class="text-lg font-bold text-white tracking-tight">
                    Configuración Global
                </h2>
                <div class="text-xs text-zinc-400">
                    Preferencias de la aplicación
                </div>
            </div>
        </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto w-full">
        <InstanceSettings />

        <!-- About Section (Footer Style) -->
        <div
            class="mt-auto py-12 flex flex-col items-center justify-center opacity-90"
        >
            <h3
                class="text-sm font-bold text-white mb-3 uppercase tracking-widest opacity-40"
            >
                Acerca de
            </h3>

            <div class="flex flex-col items-center gap-2 text-base">
                <span class="text-white font-mono font-bold text-xl"
                    >AnvilCraft v{appVersion}</span
                >

                <span
                    class="text-yellow-500 font-bold text-xs bg-yellow-500/10 px-2.5 py-1 rounded border border-yellow-500/20 uppercase tracking-wide my-1"
                >
                    AnvilCRAFT Panel
                    <span
                        class="text-[10px] font-normal text-yellow-500/80 uppercase tracking-wider bg-yellow-500/10 px-1.5 py-0.5 rounded border border-yellow-500/20"
                    >
                        v{appVersion} Copia de Evaluación
                    </span>
                </span>

                <div class="flex items-center gap-2 mt-1">
                    <span class="text-zinc-500">Desarrollado por</span>
                    <span class="text-zinc-300 font-medium text-lg">cPathz</span
                    >

                    <!-- Discord Button -->
                    <a
                        href="https://discord.com/users/cpathz"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-[#5865F2] hover:text-[#4752C4] transition-colors ml-1 p-1 hover:bg-[#5865F2]/10 rounded"
                        title="Discord Profile"
                    >
                        <svg
                            viewBox="0 0 24 24"
                            class="w-5 h-5"
                            fill="currentColor"
                        >
                            <path
                                d="M19.27 5.33C17.94 4.71 16.5 4.26 15 4a.09.09 0 0 0-.07.03c-.18.33-.39.76-.53 1.09a16.09 16.09 0 0 0-4.8 0c-.14-.34-.35-.76-.54-1.09c-.01-.02-.04-.03-.07-.03c-1.5.26-2.93.71-4.27 1.33c-.01 0-.02.01-.03.02c-2.72 4.07-3.47 8.03-3.1 11.95c0 .02.01.04.03.05c1.8 1.32 3.53 2.12 5.2 2.65c.03.01.06 0 .07-.02c.4-.55.76-1.13 1.07-1.74c.02-.04 0-.08-.04-.09c-.57-.22-1.11-.48-1.64-.78c-.04-.02-.04-.08.01-.11c.11-.08.22-.17.33-.25c.02-.02.05-.02.07-.01c3.44 1.57 7.15 1.57 10.55 0c.02-.01.05-.01.07.01c.11.09.22.17.33.26c.04.03.04.09 0 .11c-.52.31-1.07.56-1.64.78c-.04.02-.05.06-.03.09c.31.61.66 1.19 1.07 1.74c.02.02.05.02.07.02c1.67-.53 3.4-1.33 5.2-2.65c.02-.01.03-.03.03-.05c.44-4.53-.73-9.22-3.55-11.94a.06.06 0 0 0-.03-.02zM8.52 14.91c-1.03 0-1.89-.95-1.89-2.12s.84-2.12 1.89-2.12c1.06 0 1.9.96 1.89 2.12c0 1.17-.84 2.12-1.89 2.12zm6.97 0c-1.03 0-1.89-.95-1.89-2.12s.84-2.12 1.89-2.12c1.06 0 1.9.96 1.89 2.12c0 1.17-.85 2.12-1.89 2.12z"
                            />
                        </svg>
                    </a>
                </div>

                <p
                    class="text-sm text-zinc-400 mt-4 text-center max-w-md font-medium tracking-wide"
                >
                    AnvilCraft Evaluation Copy. Not for public distribution.
                </p>
            </div>
        </div>
    </div>
</div>
