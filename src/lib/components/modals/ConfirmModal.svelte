<script lang="ts">
    import { _ } from "svelte-i18n";
    import { onMount } from "svelte";

    interface Props {
        title: string;
        message: string;
        confirmText?: string;
        cancelText?: string;
        type?: 'danger' | 'warning' | 'info';
        showCheckbox?: boolean;
        checkboxLabel?: string;
        onConfirm: (checkboxValue: boolean) => void;
        onCancel: () => void;
    }

    let { 
        title, 
        message, 
        confirmText = "Confirmar", 
        cancelText = "Cancelar", 
        type = 'danger',
        showCheckbox = false,
        checkboxLabel = "",
        onConfirm, 
        onCancel 
    }: Props = $props();

    let isChecked = $state(false);
    let visible = $state(false);

    onMount(() => {
        setTimeout(() => visible = true, 10);
    });

    function handleConfirm() {
        visible = false;
        setTimeout(() => onConfirm(isChecked), 300);
    }

    function handleCancel() {
        visible = false;
        setTimeout(() => onCancel(), 300);
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div 
    class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm transition-opacity duration-300 {visible ? 'opacity-100' : 'opacity-0'}"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={handleCancel}
>
    <div 
        class="w-full max-w-md bg-zinc-900/90 border border-white/10 rounded-3xl p-8 shadow-2xl transition-all duration-300 {visible ? 'scale-100 translate-y-0 opacity-100' : 'scale-95 translate-y-4 opacity-0'}"
        onclick={(e) => e.stopPropagation()}
    >
        <!-- Icon -->
        <div class="mb-6 flex justify-center">
            <div class={`w-16 h-16 rounded-full flex items-center justify-center ${type === 'danger' ? 'bg-red-500/20 text-red-500' : 'bg-amber-500/20 text-amber-500'}`}>
                {#if type === 'danger'}
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2M10 11v6M14 11v6"/></svg>
                {:else}
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0zM12 9v4M12 17h.01"/></svg>
                {/if}
            </div>
        </div>

        <!-- Text -->
        <div class="text-center mb-8">
            <h3 class="text-xl font-bold text-white mb-2">{title}</h3>
            <p class="text-zinc-400 text-sm leading-relaxed">{message}</p>
        </div>

        <!-- Checkbox -->
        {#if showCheckbox}
            <button 
                class="flex items-center gap-3 w-full p-4 mb-8 bg-white/5 border border-white/5 rounded-2xl hover:bg-white/10 transition-colors group text-left"
                onclick={() => isChecked = !isChecked}
            >
                <div class={`w-5 h-5 rounded-md border-2 transition-all flex items-center justify-center ${isChecked ? 'bg-red-500 border-red-500' : 'border-zinc-700 group-hover:border-zinc-500'}`}>
                    {#if isChecked}
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="4"><path d="M20 6L9 17l-5-5"/></svg>
                    {/if}
                </div>
                <span class="text-sm font-medium text-zinc-300 group-hover:text-white transition-colors">{checkboxLabel}</span>
            </button>
        {/if}

        <!-- Actions -->
        <div class="grid grid-cols-2 gap-4">
            <button 
                onclick={handleCancel}
                class="px-6 py-3.5 rounded-2xl bg-zinc-800 text-zinc-400 font-bold hover:bg-zinc-700 hover:text-white transition-all active:scale-95"
            >
                {cancelText}
            </button>
            <button 
                onclick={handleConfirm}
                class={`px-6 py-3.5 rounded-2xl font-bold text-white transition-all active:scale-95 shadow-lg shadow-red-500/10 ${type === 'danger' ? 'bg-red-600 hover:bg-red-500 shadow-red-600/20' : 'bg-amber-600 hover:bg-amber-500 shadow-amber-600/20'}`}
            >
                {confirmText}
            </button>
        </div>
    </div>
</div>
