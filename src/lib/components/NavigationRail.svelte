<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";

    // Mock data for initial visualization if store is empty
    const mockInstances = [
        {
            uuid: "1",
            name: "Survival",
            icon: "https://placehold.co/48x48/5865F2/FFF?text=S",
            state: "Running",
            motor: "Paper",
            version: "1.20.1",
            path: "",
            ram_min: 2048,
            ram_max: 4096,
            port: 25565,
        },
        {
            uuid: "2",
            name: "Creative",
            icon: "https://placehold.co/48x48/eb4034/FFF?text=C",
            state: "Stopped",
            motor: "Vanilla",
            version: "1.20.4",
            path: "",
            ram_min: 1024,
            ram_max: 2048,
            port: 25566,
        },
        {
            uuid: "3",
            name: "Skyblock",
            icon: "https://placehold.co/48x48/f9a825/FFF?text=K",
            state: "Stopped",
            motor: "Spigot",
            version: "1.20.1",
            path: "",
            ram_min: 1024,
            ram_max: 2048,
            port: 25567,
        },
        {
            uuid: "4",
            name: "BedWars",
            icon: "https://placehold.co/48x48/c62828/FFF?text=B",
            state: "Stopped",
            motor: "Fabric",
            version: "1.20.1",
            path: "",
            ram_min: 2048,
            ram_max: 4096,
            port: 25568,
        },
        {
            uuid: "5",
            name: "Factions",
            icon: "https://placehold.co/48x48/2e7d32/FFF?text=F",
            state: "Running",
            motor: "Forge",
            version: "1.20.1",
            path: "",
            ram_min: 4096,
            ram_max: 8192,
            port: 25569,
        },
    ] as Instance[];

    let instances = $derived(
        appState.instances.length > 0 ? appState.instances : mockInstances,
    );
    let selectedId = $derived(appState.selectedInstance?.uuid);
</script>

<nav class="rail">
    <!-- 8 & 7: Contenedor Superior (Fijo) -->
    <div class="fixed-container top">
        <!-- 8: Inicio -->
        <!-- 8: Inicio -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="icon-wrapper"
            aria-label="Inicio"
            class:selected={appState.selectedInstance === null}
            onclick={() => (appState.selectedInstance = null)}
        >
            <!-- 5: Barrita indicador (Also for Home) -->
            <div class="pill"></div>

            <div class="icon home-icon">
                <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
                    ></path><polyline points="9 22 9 12 15 12 15 22"
                    ></polyline></svg
                >
            </div>
            <span class="tooltip">Inicio</span>
        </div>
        <!-- 7: Aplicaciones -->
        <div class="icon-wrapper" aria-label="Aplicaciones">
            <div class="icon apps-icon">
                <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><rect x="3" y="3" width="7" height="7"></rect><rect
                        x="14"
                        y="3"
                        width="7"
                        height="7"
                    ></rect><rect x="14" y="14" width="7" height="7"
                    ></rect><rect x="3" y="14" width="7" height="7"></rect></svg
                >
            </div>
            <span class="tooltip">Aplicaciones</span>
        </div>
    </div>

    <!-- 6: Separador Superior (Fijo) -->
    <div class="separator"></div>

    <!-- 4: Área de Instancias (Flexible y Desplazable) -->
    <div class="scroller instance-area">
        {#each instances as instance (instance.uuid)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="icon-wrapper instance-item"
                class:selected={selectedId === instance.uuid}
                onclick={() => (appState.selectedInstance = instance)}
            >
                <!-- 5: Barrita indicador -->
                <div class="pill"></div>

                <img
                    src={instance.icon ||
                        `https://ui-avatars.com/api/?name=${instance.name}&background=random`}
                    alt={instance.name}
                    class="icon instance-icon"
                />
                <span class="tooltip">{instance.name}</span>
            </div>
        {/each}
    </div>

    <!-- 3: Separador Inferior (Fijo) -->
    <div class="separator"></div>

    <!-- 2 & 1: Contenedor Inferior (Fijo) -->
    <div class="fixed-container bottom">
        <!-- 2: Botón de Agregar (+) -->
        <div class="icon-wrapper add-new" aria-label="Agregar">
            <div class="icon add-icon">+</div>
            <span class="tooltip">Agregar Servidor</span>
        </div>
        <!-- 1: Engrane de Ajustes -->
        <div class="icon-wrapper settings" aria-label="Configuración">
            <div class="icon settings-icon">
                <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><circle cx="12" cy="12" r="3"></circle><path
                        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
                    ></path></svg
                >
            </div>
            <span class="tooltip">Configuración</span>
        </div>
    </div>
</nav>

<style>
    .rail {
        width: 80px; /* Fixed width (User request: 80px) */
        background-color: #1e1f22; /* Discord Dark */
        display: flex;
        flex-direction: column;
        align-items: center;
        height: 100vh;
        flex-shrink: 0;
        padding: 12px 0;
        box-sizing: border-box;
    }

    .fixed-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 100%;
        gap: 8px;
        flex-shrink: 0;
    }

    .instance-area {
        width: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        flex-grow: 1; /* Occupy available space */
        overflow-y: auto; /* Scroll if needed */
        scrollbar-width: none; /* Hide scrollbar Firefox */
        padding: 8px 0;
    }

    .instance-area::-webkit-scrollbar {
        display: none; /* Hide scrollbar Chrome/Safari */
    }

    .separator {
        width: 32px;
        height: 2px;
        background-color: #35363c; /* Pill background color or similar dark grey */
        margin: 8px 0;
        flex-shrink: 0;
        border-radius: 1px;
    }

    /* Icon Styles */
    .icon-wrapper {
        position: relative;
        cursor: pointer;
        width: 80px;
        height: 48px;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 1px;
    }

    .icon {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background-color: #313338;
        transition:
            border-radius 0.2s ease,
            background-color 0.2s ease,
            color 0.2s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #dbdee1; /* Text color */
        overflow: hidden;
    }

    /* Hover State */
    .icon-wrapper:hover .icon,
    .icon-wrapper.selected .icon {
        border-radius: 16px;
        background-color: #5865f2; /* Discord Blurple */
        color: white;
    }

    .add-new .icon {
        color: #23a559;
        font-size: 24px;
        font-weight: 400;
        background-color: #313338;
    }

    .add-new:hover .icon {
        background-color: #23a559;
        color: white;
    }

    /* Pill Indicator (Barrita) */
    .pill {
        position: absolute;
        left: 0;
        top: 50%;
        transform: translateY(-50%);
        width: 4px;
        height: 8px;
        background-color: white;
        border-top-right-radius: 4px;
        border-bottom-right-radius: 4px;
        opacity: 0;
        transition:
            height 0.2s ease,
            opacity 0.2s ease;
    }

    .icon-wrapper:hover .pill {
        height: 20px;
        opacity: 1;
    }

    .icon-wrapper.selected .pill {
        height: 40px;
        opacity: 1;
    }

    /* Simple Tooltip on hover (Optional, simplified) */
    .tooltip {
        position: absolute;
        left: 80px;
        background: #111214;
        color: white;
        padding: 6px 12px;
        border-radius: 4px;
        font-size: 14px;
        font-weight: 600;
        white-space: nowrap;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.1s ease;
        z-index: 100;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    }

    .tooltip::before {
        content: "";
        position: absolute;
        left: -4px;
        top: 50%;
        transform: translateY(-50%);
        border-top: 4px solid transparent;
        border-bottom: 4px solid transparent;
        border-right: 4px solid #111214;
    }

    .icon-wrapper:hover .tooltip {
        opacity: 1;
    }
</style>
