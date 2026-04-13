<script lang="ts">
    import iconList from "$lib/data/icons.json";
    import { _ } from "svelte-i18n";

    let { onselect, onclose } = $props<{
        onselect: (icon: string) => void;
        onclose: () => void;
    }>();

    let searchQuery = $state("");
    let displayLimit = $state(60);

    // Reset limit when search changes
    $effect(() => {
        searchQuery;
        displayLimit = 60;
    });

    // Derived state for filtering
    let filteredIcons = $derived(
        iconList.filter((name) =>
            name.toLowerCase().includes(searchQuery.toLowerCase()),
        ),
    );

    let displayedIcons = $derived(filteredIcons.slice(0, displayLimit));

    function loadMore() {
        displayLimit += 60;
    }
</script>

<!-- Backdrop -->
<div
    class="fixed inset-0 z-[60] bg-black/80 backdrop-blur-md flex items-center justify-center p-6 animate-fade-in"
    role="button"
    tabindex="0"
    onkeydown={(e) => {
        if (e.key === "Escape") onclose();
    }}
    onclick={(e) => {
        if (e.target === e.currentTarget) onclose();
    }}
>
    <!-- Picker Container -->
    <div
        class="bg-[#18181b] w-full max-w-4xl h-[80vh] rounded-2xl border border-zinc-800 shadow-2xl flex flex-col overflow-hidden animate-scale-in"
        role="dialog"
        aria-modal="true"
    >
        <!-- Header -->
        <div
            class="p-4 border-b border-zinc-800 flex items-center gap-4 bg-[#18181b] z-10"
        >
            <div class="relative flex-1">
                <svg
                    class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500"
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    ><circle cx="11" cy="11" r="8"></circle><path
                        d="m21 21-4.3-4.3"
                    ></path></svg
                >
                <input
                    type="text"
                    bind:value={searchQuery}
                    autocomplete="off"
                    placeholder={$_("icon_picker.search_placeholder")}
                    class="w-full bg-zinc-900 border border-zinc-700 rounded-xl pl-10 pr-4 py-3 text-white placeholder-zinc-500 focus:outline-none focus:border-blue-500 transition-all font-medium"
                />
            </div>
            <button
                onclick={onclose}
                aria-label={$_("icon_picker.aria_label_close")}
                class="p-2 text-zinc-400 hover:text-white rounded-lg hover:bg-white/5 transition-colors"
            >
                <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    ><line x1="18" y1="6" x2="6" y2="18"></line><line
                        x1="6"
                        y1="6"
                        x2="18"
                        y2="18"
                    ></line></svg
                >
            </button>
        </div>

        <!-- Grid -->
        <div class="flex-1 overflow-y-auto p-4 custom-scrollbar bg-[#121214]">
            {#if filteredIcons.length === 0}
                <div
                    class="flex flex-col items-center justify-center h-full text-zinc-500"
                >
                    <p>{$_("icon_picker.no_results")}</p>
                </div>
            {:else}
                <div
                    class="grid grid-cols-[repeat(auto-fill,minmax(80px,1fr))] gap-3"
                >
                    {#each displayedIcons as icon}
                        <button
                            class="group relative aspect-square rounded-xl bg-zinc-900 border border-zinc-800 hover:border-blue-500 hover:bg-blue-500/10 transition-all flex flex-col items-center justify-center gap-2 p-2"
                            onclick={() =>
                                onselect(`/Transparent-Images/${icon}`)}
                            title={icon}
                        >
                            <img
                                loading="lazy"
                                src={`/Transparent-Images/${icon}`}
                                alt={icon}
                                class="w-10 h-10 object-contain drop-shadow-md group-hover:scale-110 transition-transform"
                            />
                            <span
                                class="text-[10px] text-zinc-500 group-hover:text-blue-200 truncate w-full text-center px-1 font-mono"
                            >
                                {icon.replace(".png", "").replace(/_/g, " ")}
                            </span>
                        </button>
                    {/each}
                </div>

                {#if displayedIcons.length < filteredIcons.length}
                    <div class="flex justify-center mt-6 pb-2">
                        <button
                            onclick={loadMore}
                            class="px-6 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-full text-sm font-medium transition-colors shadow-lg border border-zinc-700 hover:border-zinc-600"
                        >
                            {$_("icon_picker.btn_load_more", { values: { remaining: filteredIcons.length - displayedIcons.length } })}
                        </button>
                    </div>
                {/if}
            {/if}
        </div>

        <!-- Footer status -->
        <div
            class="p-2 px-4 border-t border-zinc-800 text-xs text-zinc-600 bg-[#18181b] flex justify-between"
        >
            <span>{$_("icon_picker.available_count", { values: { count: iconList.length } })}</span>
            <span
                >{$_("icon_picker.showing_count", { values: { displayed: displayedIcons.length, filtered: filteredIcons.length } })}</span
            >
        </div>
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 8px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: #18181b;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #27272a;
        border-radius: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #3f3f46;
    }

    @keyframes scaleIn {
        from {
            opacity: 0;
            transform: scale(0.95) translateY(10px);
        }
        to {
            opacity: 1;
            transform: scale(1) translateY(0);
        }
    }
    .animate-scale-in {
        animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }
    .animate-fade-in {
        animation: fadeIn 0.2s ease-out;
    }
    @keyframes fadeIn {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }
</style>
