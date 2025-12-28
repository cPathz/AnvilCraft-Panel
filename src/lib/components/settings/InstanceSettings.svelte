<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "$lib/runes/toast.svelte";
    import { onMount, tick } from "svelte";
    import { fade } from "svelte/transition";

    let {
        instance,
        isServerRunning,
        isDirty = $bindable(false),
    } = $props<{
        instance: Instance;
        isServerRunning: boolean;
        isDirty?: boolean;
    }>();

    // Local State
    let formSettings = $state({
        min_ram: 1024,
        max_ram: 2048,
        port: 25565,
        args: "",
        jar_file: "server.jar",
    });

    let originalSettings = $state<any>(null);
    let systemRam = $state(0);
    let linkMemory = $state(true);
    let lastSyncedId = $state<string | null>(null);

    let internalIsDirty = $derived(
        originalSettings &&
            JSON.stringify(formSettings) !== JSON.stringify(originalSettings),
    );

    // Sync internal dirty state to parent prop
    $effect(() => {
        isDirty = internalIsDirty;
    });

    // Initialization
    onMount(async () => {
        try {
            systemRam = await invoke<number>("get_system_memory");
        } catch (e) {
            console.error("Failed to get system memory", e);
        }
    });

    // Sync form when instance changes
    $effect(() => {
        if (instance && instance.settings) {
            if (lastSyncedId !== instance.id) {
                formSettings = { ...instance.settings };
                originalSettings = { ...instance.settings };
                lastSyncedId = instance.id;
            }
        }
    });

    // Actions
    async function saveSettings() {
        try {
            await invoke("save_instance_settings", {
                instanceId: instance.id,
                settings: formSettings,
            });

            // Update Global State
            if (
                appState.selectedInstance &&
                appState.selectedInstance.id === instance.id
            ) {
                appState.selectedInstance.settings = { ...formSettings };
            }
            originalSettings = { ...formSettings };
            toast.success("¡Configuración guardada correctamente!");
        } catch (e) {
            console.error(e);
            toast.error("Error al guardar: " + e);
        }
    }

    function discardChanges() {
        if (originalSettings) {
            formSettings = { ...originalSettings };
        }
    }

    function resetGeneralSettingsForm() {
        formSettings.port = 25565;
        formSettings.jar_file = "server.jar";
        formSettings.min_ram = 1024;
        formSettings.max_ram = 2048;
        formSettings.args = "";
    }

    function formatBytes(bytes: number, decimals = 2) {
        if (bytes === 0) return "0 Bytes";
        const k = 1024;
        const dm = decimals < 0 ? 0 : decimals;
        const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return (
            parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i]
        );
    }
</script>

<div class="flex-1 flex flex-col min-h-0">
    <!-- Action Bar (Save/Discard) -->
    {#if isDirty}
        <div
            class="bg-yellow-500/10 border-b border-yellow-500/20 px-8 py-3 flex items-center justify-between"
            transition:fade
        >
            <span
                class="text-yellow-500 text-sm font-bold flex items-center gap-2"
            >
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    ><path
                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    /></svg
                >
                Cambios sin guardar
            </span>
            <div class="flex gap-2">
                <button
                    onclick={discardChanges}
                    class="px-3 py-1.5 rounded-lg text-xs font-bold text-zinc-400 hover:text-white hover:bg-white/10 transition-colors"
                >
                    Descartar
                </button>
                <button
                    onclick={saveSettings}
                    class="px-4 py-1.5 rounded-lg text-xs font-bold text-[#0f172a] bg-yellow-400 hover:bg-yellow-300 transition-colors shadow-lg shadow-yellow-400/20"
                >
                    Guardar Cambios
                </button>
            </div>
        </div>
    {/if}

    <!-- Scrollable Content -->
    <div class="flex-1 p-8 overflow-y-auto custom-scrollbar">
        <div class="max-w-3xl space-y-8">
            <!-- Warning Banner if Running -->
            {#if isServerRunning}
                <div
                    class="mb-6 bg-yellow-500/10 border border-yellow-500/20 rounded-xl p-4 flex items-center gap-3"
                >
                    <div
                        class="w-10 h-10 rounded-full bg-yellow-500/20 flex items-center justify-center shrink-0"
                    >
                        <svg
                            width="20"
                            height="20"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="#eab308"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><path
                                d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
                            ></path><line x1="12" y1="9" x2="12" y2="13"
                            ></line><line x1="12" y1="17" x2="12.01" y2="17"
                            ></line></svg
                        >
                    </div>
                    <div>
                        <h3 class="font-bold text-yellow-500">
                            Servidor en Ejecución
                        </h3>
                        <p class="text-sm text-yellow-200/70">
                            Detén el servidor para modificar la configuración
                            avanzada.
                        </p>
                    </div>
                </div>
            {/if}

            <div class="space-y-6">
                <div class="grid grid-cols-2 gap-6">
                    <!-- RAM Min -->
                    <div class="space-y-2">
                        <label
                            for="min-ram"
                            class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                            >RAM Mínima (MB)</label
                        >
                        <div class="relative group">
                            <input
                                id="min-ram"
                                type="number"
                                bind:value={formSettings.min_ram}
                                disabled={isServerRunning}
                                class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl px-4 py-3 font-jetbrains focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300"
                            />
                            <div
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-xs font-bold text-zinc-600 bg-[#1e293b] px-2 py-1 rounded"
                            >
                                MIN
                            </div>
                        </div>
                    </div>

                    <!-- RAM Max -->
                    <div class="space-y-2">
                        <label
                            for="max-ram"
                            class="text-xs font-bold text-zinc-400 uppercase tracking-wider flex justify-between"
                        >
                            <span>RAM Máxima (MB)</span>
                            {#if linkMemory}
                                <button
                                    onclick={() => (linkMemory = false)}
                                    class="text-blue-400 hover:text-blue-300 transition-colors"
                                    title="Desvincular"
                                >
                                    <svg
                                        width="14"
                                        height="14"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        ><path
                                            d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
                                        ></path><path
                                            d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
                                        ></path></svg
                                    >
                                </button>
                            {:else}
                                <button
                                    onclick={() => (linkMemory = true)}
                                    class="text-zinc-600 hover:text-blue-400 transition-colors"
                                    title="Vincular"
                                >
                                    <svg
                                        width="14"
                                        height="14"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        class="opacity-50"
                                        ><path
                                            d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
                                        ></path></svg
                                    >
                                </button>
                            {/if}
                        </label>
                        <div class="relative group">
                            <input
                                id="max-ram"
                                type="number"
                                bind:value={formSettings.max_ram}
                                disabled={isServerRunning}
                                class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl px-4 py-3 font-jetbrains focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-white shadow-[0_0_20px_rgba(59,130,246,0.1)] focus:shadow-[0_0_30px_rgba(59,130,246,0.2)]"
                            />
                            <div
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-xs font-bold text-blue-500 bg-blue-500/10 px-2 py-1 rounded"
                            >
                                MAX
                            </div>
                        </div>
                        <div
                            class="flex justify-between text-xs text-zinc-500 px-1"
                        >
                            <span>Sistema: {formatBytes(systemRam)}</span>
                            <span
                                class={formSettings.max_ram >
                                (systemRam / 1024 / 1024) * 0.8
                                    ? "text-yellow-500"
                                    : ""}
                            >
                                {Math.round(
                                    (formSettings.max_ram /
                                        (systemRam / 1024 / 1024)) *
                                        100,
                                )}% del total
                            </span>
                        </div>
                    </div>
                </div>

                <!-- Java Args -->
                <div class="space-y-2">
                    <label
                        for="java-args"
                        class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                        >Argumentos Java</label
                    >
                    <input
                        id="java-args"
                        type="text"
                        bind:value={formSettings.args}
                        disabled={isServerRunning}
                        placeholder="-XX:+UseG1GC..."
                        class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl px-4 py-3 font-jetbrains text-sm focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300 placeholder:text-zinc-700"
                    />
                </div>

                <!-- JAR File & Port -->
                <div class="grid grid-cols-3 gap-6">
                    <div class="col-span-2 space-y-2">
                        <label
                            for="jar-file"
                            class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                            >Archivo JAR</label
                        >
                        <div class="relative group">
                            <div
                                class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600"
                            >
                                <svg
                                    width="16"
                                    height="16"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    ><path
                                        d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
                                    ></path><path
                                        d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
                                    ></path></svg
                                >
                            </div>
                            <input
                                id="jar-file"
                                type="text"
                                bind:value={formSettings.jar_file}
                                disabled={isServerRunning}
                                class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl pl-10 pr-4 py-3 font-jetbrains text-sm focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300"
                            />
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label
                            for="server-port"
                            class="text-xs font-bold text-zinc-400 uppercase tracking-wider"
                            >Puerto</label
                        >
                        <input
                            id="server-port"
                            type="number"
                            bind:value={formSettings.port}
                            disabled={isServerRunning}
                            class="w-full bg-[#0f172a] border-2 border-[#1e293b] rounded-xl px-4 py-3 font-jetbrains text-sm focus:border-blue-500 focus:outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed text-zinc-300 text-center"
                        />
                    </div>
                </div>

                <!-- Reset Buttons -->
                <div class="pt-4 flex justify-end gap-3">
                    <button
                        onclick={resetGeneralSettingsForm}
                        disabled={isServerRunning}
                        class="px-4 py-2 rounded-lg text-xs font-bold text-zinc-500 hover:text-zinc-300 hover:bg-white/5 transition-colors"
                    >
                        Restaurar Valores
                    </button>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    :global(.custom-scrollbar::-webkit-scrollbar) {
        width: 6px;
    }
    :global(.custom-scrollbar::-webkit-scrollbar-track) {
        background: transparent;
    }
    :global(.custom-scrollbar::-webkit-scrollbar-thumb) {
        background-color: rgba(255, 255, 255, 0.1);
        border-radius: 9999px;
    }
    :global(.custom-scrollbar::-webkit-scrollbar-thumb:hover) {
        background-color: rgba(255, 255, 255, 0.2);
    }
</style>
