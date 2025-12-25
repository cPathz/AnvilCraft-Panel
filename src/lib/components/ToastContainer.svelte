<script lang="ts">
    import { toast } from "$lib/runes/toast.svelte";
    import { fly, fade } from "svelte/transition";
    import { flip } from "svelte/animate";

    // Icons could be imported or inline SVGs
</script>

<div
    class="fixed bottom-6 right-6 z-[100] flex flex-col gap-3 pointer-events-none"
>
    {#each toast.toasts as t (t.id)}
        <div
            class="pointer-events-auto min-w-[300px] max-w-sm rounded-xl shadow-2xl border backdrop-blur-md p-4 flex items-center gap-3 overflow-hidden relative group"
            in:fly={{ x: 20, duration: 300 }}
            out:fade={{ duration: 200 }}
            animate:flip={{ duration: 300 }}
            class:bg-emerald-500-10={t.type === "success"}
            class:border-emerald-500-20={t.type === "success"}
            class:bg-red-500-10={t.type === "error"}
            class:border-red-500-20={t.type === "error"}
            class:bg-blue-500-10={t.type === "info"}
            class:border-blue-500-20={t.type === "info"}
            class:bg-yellow-500-10={t.type === "warning"}
            class:border-yellow-500-20={t.type === "warning"}
            style:background-color={t.type === "success"
                ? "rgba(16, 185, 129, 0.1)"
                : t.type === "error"
                  ? "rgba(239, 68, 68, 0.1)"
                  : t.type === "warning"
                    ? "rgba(234, 179, 8, 0.1)"
                    : "rgba(59, 130, 246, 0.1)"}
            style:border-color={t.type === "success"
                ? "rgba(16, 185, 129, 0.2)"
                : t.type === "error"
                  ? "rgba(239, 68, 68, 0.2)"
                  : t.type === "warning"
                    ? "rgba(234, 179, 8, 0.2)"
                    : "rgba(59, 130, 246, 0.2)"}
        >
            <!-- Icon -->
            <div class="shrink-0">
                {#if t.type === "success"}
                    <div
                        class="h-8 w-8 rounded-full bg-emerald-500/20 flex items-center justify-center text-emerald-400"
                    >
                        <svg
                            width="18"
                            height="18"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><path d="M20 6 9 17l-5-5" /></svg
                        >
                    </div>
                {:else if t.type === "error"}
                    <div
                        class="h-8 w-8 rounded-full bg-red-500/20 flex items-center justify-center text-red-400"
                    >
                        <svg
                            width="18"
                            height="18"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><circle cx="12" cy="12" r="10" /><line
                                x1="12"
                                y1="8"
                                x2="12"
                                y2="12"
                            /><line x1="12" y1="16" x2="12.01" y2="16" /></svg
                        >
                    </div>
                {:else if t.type === "warning"}
                    <div
                        class="h-8 w-8 rounded-full bg-yellow-500/20 flex items-center justify-center text-yellow-400"
                    >
                        <svg
                            width="18"
                            height="18"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><path
                                d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"
                            /><line x1="12" y1="9" x2="12" y2="13" /><line
                                x1="12"
                                y1="17"
                                x2="12.01"
                                y2="17"
                            /></svg
                        >
                    </div>
                {:else}
                    <div
                        class="h-8 w-8 rounded-full bg-blue-500/20 flex items-center justify-center text-blue-400"
                    >
                        <svg
                            width="18"
                            height="18"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><circle cx="12" cy="12" r="10" /><line
                                x1="12"
                                y1="16"
                                x2="12"
                                y2="12"
                            /><line x1="12" y1="8" x2="12.01" y2="8" /></svg
                        >
                    </div>
                {/if}
            </div>

            <!-- Content -->
            <div class="flex-1 min-w-0">
                <p class="font-bold text-sm text-white">
                    {t.type === "success"
                        ? "Éxito"
                        : t.type === "error"
                          ? "Error"
                          : t.type === "warning"
                            ? "Advertencia"
                            : "Información"}
                </p>
                <p class="text-xs text-zinc-300 truncate font-medium mt-0.5">
                    {t.message}
                </p>
            </div>

            <!-- Close Button -->
            <button
                aria-label="Cerrar notificación"
                onclick={() => toast.remove(t.id)}
                class="shrink-0 p-1.5 rounded-lg hover:bg-white/10 text-zinc-400 hover:text-white transition-colors opacity-0 group-hover:opacity-100 focus:opacity-100"
            >
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path d="M18 6 6 18" /><path d="m6 6 12 12" /></svg
                >
            </button>
        </div>
    {/each}
</div>
