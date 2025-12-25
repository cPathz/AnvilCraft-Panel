<script lang="ts">
    import { appState, type Instance } from "$lib/runes/store.svelte";

    let instances = $derived(appState.instances);
    let selectedId = $derived(appState.selectedInstance?.id);

    let hoveredLabel = $state<string | null>(null);
    let tooltipTop = $state(0);

    function handleMouseEnter(e: MouseEvent, label: string) {
        const target = e.currentTarget as HTMLElement;
        const rect = target.getBoundingClientRect();
        hoveredLabel = label;
        // Calculate center of the item relative to the viewport,
        // but since we render tooltip fixed/absolute, let's use clientY or rect.top
        // We will position the tooltip fixed to avoid any containment issues.
        tooltipTop = rect.top + rect.height / 2;
    }

    function handleMouseLeave() {
        hoveredLabel = null;
    }
</script>

<nav class="rail">
    <!-- 8 & 7: Contenedor Superior (Fijo) -->
    <div class="fixed-container top">
        <!-- 8: Inicio -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="icon-wrapper"
            aria-label="Inicio"
            class:selected={appState.view === "home" &&
                appState.selectedInstance === null}
            onclick={() => {
                appState.view = "home";
                appState.selectedInstance = null;
            }}
            onmouseenter={(e) => handleMouseEnter(e, "Inicio")}
            onmouseleave={handleMouseLeave}
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
            <!-- REMOVED CSS TOOLTIP -->
        </div>
        <!-- 7: Aplicaciones -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="icon-wrapper"
            aria-label="Aplicaciones"
            class:selected={appState.view === "instances" &&
                appState.selectedInstance === null}
            onclick={() => {
                appState.view = "instances";
                appState.selectedInstance = null;
            }}
            onmouseenter={(e) => handleMouseEnter(e, "Aplicaciones")}
            onmouseleave={handleMouseLeave}
        >
            <div class="pill"></div>
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
            <!-- REMOVED CSS TOOLTIP -->
        </div>
    </div>

    <!-- 6: Separador Superior (Fijo) -->
    <div class="separator"></div>

    <!-- 4: Área de Instancias (Flexible y Desplazable) -->
    <div class="scroller instance-area">
        {#each instances as instance (instance.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="icon-wrapper instance-item"
                class:selected={selectedId === instance.id}
                class:running={instance.state === "Running"}
                onclick={() => (appState.selectedInstance = instance)}
                onmouseenter={(e) => handleMouseEnter(e, instance.name)}
                onmouseleave={handleMouseLeave}
            >
                <!-- 5: Barrita indicador -->
                <div class="pill"></div>

                <img
                    src={instance.icon ||
                        `https://ui-avatars.com/api/?name=${instance.name}&background=random`}
                    alt={instance.name}
                    class="icon instance-icon"
                />
            </div>
        {/each}
    </div>

    <!-- 3: Separador Inferior (Fijo) -->
    <div class="separator"></div>

    <!-- 2 & 1: Contenedor Inferior (Fijo) -->
    <div class="fixed-container bottom">
        <!-- 2: Botón de Agregar (+) -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="icon-wrapper add-new"
            aria-label="Agregar"
            onclick={() => (appState.creatingInstance = true)}
            onmouseenter={(e) => handleMouseEnter(e, "Nueva Instancia")}
            onmouseleave={handleMouseLeave}
        >
            <div class="icon add-icon">+</div>
        </div>
        <!-- 1: Engrane de Ajustes -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="icon-wrapper settings"
            class:selected={appState.view === "settings"}
            aria-label="Configuración"
            onclick={() => {
                appState.view = "settings";
                appState.selectedInstance = null;
            }}
            onmouseenter={(e) => handleMouseEnter(e, "Configuración")}
            onmouseleave={handleMouseLeave}
        >
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
        </div>
    </div>

    <!-- Global Floating Tooltip -->
    {#if hoveredLabel}
        <div class="floating-tooltip" style="top: {tooltipTop}px">
            {hoveredLabel}
        </div>
    {/if}
</nav>

<style>
    .rail {
        width: 80px;
        /* User Darkened Gradient: #192232 to #0f1520 */
        background: linear-gradient(180deg, #192232 0%, #0f1520 100%);
        border-right: 1px solid rgba(255, 255, 255, 0.05);
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
        flex-grow: 1;
        overflow-y: auto;
        scrollbar-width: none;
        padding: 8px 0;
    }

    .instance-area::-webkit-scrollbar {
        display: none;
    }

    .separator {
        width: 32px;
        height: 1px;
        background-color: rgba(255, 255, 255, 0.08);
        margin: 8px 0;
        flex-shrink: 0;
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
        transition: background-color 0.2s;
    }

    .icon-wrapper:hover {
        background-color: rgba(59, 130, 246, 0.05); /* Ion Blue Hover Tint */
    }

    .icon {
        width: 44px;
        height: 44px;
        border-radius: 10px;
        background-color: transparent;
        transition: all 0.2s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #71717a; /* Zinc-500 (Dimmed) */
    }

    /* Hover State */
    .icon-wrapper:hover .icon {
        color: #d4d4d8; /* Zinc-300 */
        transform: scale(1.05);
        text-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
    }

    /* Active/Selected State - Ion Glow Blue */
    .icon-wrapper.selected .icon {
        background-color: rgba(59, 130, 246, 0.15);
        color: #60a5fa; /* Blue-400 */
        box-shadow: 0 0 20px rgba(59, 130, 246, 0.15);
        border: 1px solid rgba(59, 130, 246, 0.1);
    }

    /* Running State Glow - Golden/Orange Gradient */
    .icon-wrapper.running .icon {
        box-shadow:
            0 0 15px rgba(245, 158, 11, 0.5),
            inset 0 0 10px rgba(245, 158, 11, 0.2);
        border: 1px solid rgba(245, 158, 11, 0.4);
        animation: pulse-gold 2s infinite;
    }

    @keyframes pulse-gold {
        0% {
            box-shadow:
                0 0 15px rgba(245, 158, 11, 0.5),
                inset 0 0 10px rgba(245, 158, 11, 0.2);
        }
        50% {
            box-shadow:
                0 0 25px rgba(245, 158, 11, 0.7),
                inset 0 0 15px rgba(245, 158, 11, 0.3);
        }
        100% {
            box-shadow:
                0 0 15px rgba(245, 158, 11, 0.5),
                inset 0 0 10px rgba(245, 158, 11, 0.2);
        }
    }

    .add-new .icon {
        color: #71717a;
        background-color: rgba(255, 255, 255, 0.03);
    }

    .add-new:hover .icon {
        background-color: rgba(59, 130, 246, 0.1);
        color: #60a5fa;
    }

    /* Pill Indicator (Barrita) */
    .pill {
        position: absolute;
        left: 0;
        top: 50%;
        transform: translateY(-50%);
        width: 3px;
        height: 0px;
        background-color: #3b82f6; /* Ion Blue */
        border-top-right-radius: 4px;
        border-bottom-right-radius: 4px;
        opacity: 0;
        transition: all 0.2s ease;
        box-shadow: 0 0 10px rgba(59, 130, 246, 0.5);
    }

    .icon-wrapper.selected .pill {
        height: 24px;
        opacity: 1;
    }

    /* Floating Tooltip */
    .floating-tooltip {
        position: fixed; /* Fixed relative to viewport to avoid scroll clipping */
        left: 80px;
        background: #09090b; /* Zinc-950 */
        color: #e4e4e7;
        padding: 6px 12px;
        border-radius: 4px;
        font-size: 14px;
        font-weight: 600;
        white-space: nowrap;
        pointer-events: none;
        z-index: 9999;
        border: 1px solid rgba(255, 255, 255, 0.05);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
        transform: translateY(-50%); /* Center vertically on the coordinate */
        animation: tooltip-fade 0.1s ease-out;
    }

    @keyframes tooltip-fade {
        from {
            opacity: 0;
            transform: translateY(-50%) scale(0.95);
        }
        to {
            opacity: 1;
            transform: translateY(-50%) scale(1);
        }
    }

    .floating-tooltip::before {
        content: "";
        position: absolute;
        left: -4px;
        top: 50%;
        transform: translateY(-50%);
        border-top: 4px solid transparent;
        border-bottom: 4px solid transparent;
        border-right: 4px solid #09090b;
    }
</style>
