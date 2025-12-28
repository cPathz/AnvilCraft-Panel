<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { tick } from "svelte";
    import { fade } from "svelte/transition";
    import IconPicker from "$lib/components/modals/IconPicker.svelte";

    // Decomposed Components
    import ConsoleView from "$lib/components/console/ConsoleView.svelte";
    import InstanceSettings from "$lib/components/settings/InstanceSettings.svelte";

    let instance = $derived(appState.selectedInstance!);

    // Ensure runtime exists
    $effect(() => {
        if (instance) appState.ensureRuntime(instance.id);
    });

    // Derived from runtime store for persistence
    let runtime = $derived(
        appState.getRuntime(instance.id) || { logs: [], activeTab: "console" },
    );

    // Tab Management
    let activeTab = $derived(runtime.activeTab);
    function setActiveTab(tab: "console" | "settings") {
        appState.ensureRuntime(instance.id);
        const r = appState.getRuntime(instance.id);
        if (r) r.activeTab = tab;
    }

    // State derived from Instance
    let isServerRunning = $derived(
        instance.state === "Running" || instance.state === "Starting",
    );

    // Settings Dirty State (Bound from InstanceSettings)
    let settingsIsDirty = $state(false);
    let showConfirmModal = $state(false);
    let pendingTab = $state<"console" | "settings" | null>(null);

    // --- Tab Interception Logic ---
    function handleTabChange(tab: "console" | "settings") {
        if (activeTab === "settings" && tab !== "settings" && settingsIsDirty) {
            pendingTab = tab;
            showConfirmModal = true;
        } else {
            setActiveTab(tab);
        }
    }

    function confirmDiscard() {
        // We can't force discard in child easily without reference?
        // Actually, if we switch tab, the component unmounts and state is lost?
        // Yes, if we switch tab, InstanceSettings unmounts and its local state (formSettings) is lost/reset next mount.
        // So "Confirm Discard" just means "Allow Switch".
        showConfirmModal = false;
        if (pendingTab) {
            setActiveTab(pendingTab);
            pendingTab = null;
        }
    }

    function cancelDiscard() {
        showConfirmModal = false;
        pendingTab = null;
    }

    // --- Instance Actions ---
    async function toggleServer() {
        if (instance.state === "Stopped" || instance.state === "Error") {
            try {
                await invoke("start_instance", { id: instance.id });
            } catch (e) {
                console.error(e);
                alert("Error starting server: " + e);
            }
        } else {
            try {
                await invoke("stop_instance", { id: instance.id });
            } catch (e) {
                console.error(e);
            }
        }
    }

    async function forceKill() {
        if (
            !confirm(
                "¿Estás seguro de que quieres forzar el cierre? Esto podría corromper datos.",
            )
        )
            return;
        try {
            await invoke("kill_instance", { id: instance.id });
        } catch (e) {
            console.error(e);
            alert("Error killing server: " + e);
        }
    }

    let showDeleteModal = $state(false);
    async function deleteInstance() {
        if (!instance) return;
        try {
            await invoke("delete_instance", { id: instance.id });
            const instances = await invoke("read_instances");
            appState.instances = instances as Instance[];
            appState.selectedInstance = null;
            appState.view = "instances";
        } catch (e) {
            console.error("Failed to delete instance:", e);
            alert("Error al eliminar la instancia: " + e);
        } finally {
            showDeleteModal = false;
        }
    }

    function openFolder() {
        invoke("open_instances_folder", { slug: instance.path });
    }

    // --- Icon Picker Logic ---
    let showIconPicker = $state(false);
    async function handleIconSelected(newIcon: string) {
        showIconPicker = false;
        try {
            await invoke("update_instance_icon", {
                id: instance.id,
                icon: newIcon,
            });
            if (appState.selectedInstance)
                appState.selectedInstance.icon = newIcon;

            // Refetch to sync list
            const idx = appState.instances.findIndex(
                (i) => i.id === instance.id,
            );
            if (idx !== -1) appState.instances[idx].icon = newIcon;
        } catch (e) {
            console.error(e);
        }
    }
</script>

<div
    class="h-full flex flex-col bg-[#0f172a] text-zinc-100 relative overflow-hidden"
>
    <!-- Ambient Background -->
    {#if instance.icon}
        <div class="absolute inset-0 z-0">
            <img
                src={instance.icon}
                alt=""
                class="w-full h-full object-cover opacity-10 blur-3xl scale-150 transition-all duration-700"
            />
            <div
                class="absolute inset-0 bg-gradient-to-b from-[#0f172a]/90 to-[#0f172a]/95"
            ></div>
        </div>
    {/if}
    <!-- Header -->
    <div
        class="h-20 border-b border-[#1e293b] flex items-center justify-between px-6 shrink-0 bg-[#0f172a]/50 backdrop-blur-md z-20"
    >
        <div class="flex items-center gap-5">
            <!-- Icon -->
            <div class="relative group shrink-0">
                <button
                    onclick={() => (showIconPicker = true)}
                    class="w-12 h-12 rounded-2xl bg-[#1e293b] flex items-center justify-center overflow-hidden border border-white/10 shadow-lg transition-transform active:scale-95 group-hover:border-blue-500/50"
                >
                    {#if instance.icon}
                        <img
                            src={instance.icon}
                            alt="Icon"
                            class="w-full h-full object-cover"
                        />
                    {:else}
                        <span class="text-zinc-500 text-sm font-bold"
                            >{instance.name.substring(0, 2).toUpperCase()}</span
                        >
                    {/if}

                    <!-- Edit Overlay -->
                    <div
                        class="absolute inset-0 bg-black/60 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                        <svg
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="white"
                            stroke-width="2"
                            ><path
                                d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                            /><path
                                d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
                            /></svg
                        >
                    </div>
                </button>
            </div>

            <!-- Info -->
            <div class="flex flex-col justify-center">
                <div class="flex items-center gap-3">
                    <h1
                        class="text-xl font-bold text-white leading-none tracking-tight shadow-black drop-shadow-sm"
                    >
                        {instance.name}
                    </h1>
                    <span
                        class="px-2 py-0.5 rounded-md text-[10px] font-bold uppercase tracking-wider bg-white/5 text-zinc-400 border border-white/5 backdrop-blur-sm"
                    >
                        {instance.loader || "Vanilla"}
                        {instance.version}
                    </span>
                </div>

                <div class="flex items-center gap-3 mt-1.5">
                    <!-- Status -->
                    <span
                        class="flex items-center gap-1.5 text-xs font-medium backdrop-blur-sm px-2 py-0.5 rounded-full bg-black/20 border border-white/5 {instance.state ===
                        'Running'
                            ? 'text-green-400'
                            : instance.state === 'Starting'
                              ? 'text-yellow-400'
                              : instance.state === 'Error'
                                ? 'text-red-400'
                                : 'text-zinc-500'}"
                    >
                        <span class="relative flex h-2 w-2">
                            {#if instance.state === "Running" || instance.state === "Starting"}
                                <span
                                    class="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 {instance.state ===
                                    'Running'
                                        ? 'bg-green-400'
                                        : 'bg-yellow-400'}"
                                ></span>
                            {/if}
                            <span
                                class="relative inline-flex rounded-full h-2 w-2 {instance.state ===
                                'Running'
                                    ? 'bg-green-500'
                                    : instance.state === 'Starting'
                                      ? 'bg-yellow-500'
                                      : instance.state === 'Error'
                                        ? 'bg-red-500'
                                        : 'bg-zinc-600'}"
                            ></span>
                        </span>
                        {instance.state === "Running"
                            ? "En línea"
                            : instance.state === "Starting"
                              ? "Iniciando..."
                              : instance.state === "Error"
                                ? "Error"
                                : "Detenido"}
                    </span>

                    <!-- Separator -->
                    <span class="w-1 h-1 rounded-full bg-zinc-700"></span>

                    <!-- Open Folder Action -->
                    <button
                        onclick={openFolder}
                        class="text-xs text-zinc-500 hover:text-blue-400 transition-colors flex items-center gap-1.5 group/folder"
                        title="Abrir carpeta de la instancia"
                    >
                        <svg
                            class="w-3.5 h-3.5 group-hover/folder:stroke-blue-400 transition-colors"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path
                                d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"
                            />
                        </svg>
                        Carpeta
                    </button>
                </div>
            </div>
        </div>

        <div class="flex items-center gap-3">
            {#if instance.state === "Running" || instance.state === "Starting"}
                <button
                    onclick={forceKill}
                    class="p-2.5 rounded-xl bg-red-500/10 text-red-500 hover:bg-red-500/20 transition-all border border-red-500/20 active:scale-95"
                    title="Forzar Detención"
                >
                    <svg
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path><line
                            x1="12"
                            y1="2"
                            x2="12"
                            y2="12"
                        ></line></svg
                    >
                </button>
            {/if}

            <button
                onclick={toggleServer}
                class="h-10 px-6 rounded-xl font-bold text-sm shadow-xl transition-all flex items-center gap-2 active:scale-95 {instance.state ===
                    'Running' || instance.state === 'Starting'
                    ? 'bg-gradient-to-br from-red-500 to-red-600 hover:from-red-400 hover:to-red-500 text-white shadow-red-500/20'
                    : 'bg-gradient-to-br from-green-500 to-green-600 hover:from-green-400 hover:to-green-500 text-white shadow-green-500/20'}"
            >
                {#if instance.state === "Running" || instance.state === "Starting"}
                    <svg
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        stroke="none"
                        ><rect x="6" y="4" width="4" height="16" rx="1" /><rect
                            x="14"
                            y="4"
                            width="4"
                            height="16"
                            rx="1"
                        /></svg
                    >
                    Detener
                {:else}
                    <svg
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        stroke="none"><path d="M5 3l14 9-14 9V3z" /></svg
                    >
                    Iniciar
                {/if}
            </button>
        </div>
    </div>

    <!-- Navigation Tabs -->
    <div
        class="px-6 pt-4 flex items-center justify-between border-b border-white/5 relative z-10"
    >
        <div class="flex gap-6">
            <button
                onclick={() => handleTabChange("console")}
                class="pb-3 text-sm font-bold relative transition-colors {activeTab ===
                'console'
                    ? 'text-white'
                    : 'text-zinc-500 hover:text-zinc-300'}"
            >
                Consola
                {#if activeTab === "console"}
                    <div
                        class="absolute bottom-0 left-0 w-full h-0.5 bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]"
                        transition:fade
                    ></div>
                {/if}
            </button>
            <button
                onclick={() => handleTabChange("settings")}
                class="pb-3 text-sm font-bold relative transition-colors {activeTab ===
                'settings'
                    ? 'text-white'
                    : 'text-zinc-500 hover:text-zinc-300'}"
            >
                Ajustes
                {#if activeTab === "settings"}
                    <div
                        class="absolute bottom-0 left-0 w-full h-0.5 bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]"
                        transition:fade
                    ></div>
                {/if}
            </button>
        </div>

        {#if !isServerRunning}
            <button
                onclick={() => (showDeleteModal = true)}
                class="mb-2 text-xs font-bold text-red-500/50 hover:text-red-500 hover:bg-red-500/10 px-3 py-1.5 rounded-lg transition-all flex items-center gap-2"
            >
                <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    ><path d="M3 6h18" /><path
                        d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"
                    /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" /></svg
                >
                Eliminar
            </button>
        {/if}
    </div>

    <!-- Main Content Area -->
    <div class="flex-1 min-h-0 flex flex-col relative overflow-hidden z-10">
        {#if activeTab === "console"}
            <div
                class="absolute inset-0 p-6 flex flex-col"
                transition:fade={{ duration: 150 }}
            >
                <ConsoleView instanceId={instance.id} />
            </div>
        {:else if activeTab === "settings"}
            <div
                class="absolute inset-0 flex flex-col"
                transition:fade={{ duration: 150 }}
            >
                <InstanceSettings
                    {instance}
                    {isServerRunning}
                    bind:isDirty={settingsIsDirty}
                />
            </div>
        {/if}
    </div>
</div>

<!-- Modals -->
{#if showDeleteModal}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
        transition:fade
    >
        <div
            class="bg-[#1e293b] border border-white/10 rounded-2xl w-full max-w-md p-6 shadow-2xl relative overflow-hidden"
        >
            <h2 class="text-xl font-bold text-white mb-2">
                Eliminar Instancia
            </h2>
            <p class="text-zinc-400 text-sm mb-6">
                ¿Estás seguro de eliminar <span class="text-white font-bold"
                    >{instance.name}</span
                >? Esta acción no se puede deshacer y borrará todos los archivos
                del servidor.
            </p>
            <div class="flex justify-end gap-3">
                <button
                    onclick={() => (showDeleteModal = false)}
                    class="px-4 py-2 rounded-xl text-sm font-bold text-zinc-400 hover:text-white hover:bg-white/5 transition-colors"
                    >Cancelar</button
                >
                <button
                    onclick={deleteInstance}
                    class="px-4 py-2 rounded-xl text-sm font-bold bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/20 transition-all"
                    >Eliminar Definitivamente</button
                >
            </div>
        </div>
    </div>
{/if}

{#if showConfirmModal}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
        transition:fade
    >
        <div
            class="bg-[#1e293b] w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl"
        >
            <h3 class="text-xl font-bold text-white mb-2">
                Cambios sin guardar
            </h3>
            <p class="text-zinc-400 text-sm mb-6">
                Tienes cambios pendientes en la configuración. ¿Quieres
                descartarlos y salir?
            </p>
            <div class="flex justify-end gap-3">
                <button
                    onclick={cancelDiscard}
                    class="px-4 py-2 rounded-lg text-zinc-400 hover:bg-white/10 transition-colors text-sm font-bold"
                    >Cancelar</button
                >
                <button
                    onclick={confirmDiscard}
                    class="px-4 py-2 rounded-lg bg-red-500 hover:bg-red-600 text-white transition-colors text-sm font-bold shadow-lg shadow-red-500/20"
                    >Descartar y Salir</button
                >
            </div>
        </div>
    </div>
{/if}

{#if showIconPicker}
    <IconPicker
        onclose={() => (showIconPicker = false)}
        onselect={handleIconSelected}
    />
{/if}
