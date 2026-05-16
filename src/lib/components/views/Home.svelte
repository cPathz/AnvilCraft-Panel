<script lang="ts">
    import { _ } from "svelte-i18n";
    import { appState } from "$lib/runes/store.svelte";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { fade, scale } from "svelte/transition";
    import { relaunch } from "@tauri-apps/plugin-process";
    import { toast } from "$lib/runes/toast.svelte";

    // Interaction State
    let showUpdateConfirm = $state(false);
    let showRedirectConfirm = $state(false);
    
    // Installation State
    let downloading = $state(false);
    let downloadProgress = $state(0);

    const mockUrl = "https://github.com/cPathz/AnvilCraft-Panel/releases/latest";

    async function handleInstall() {
        if (!appState.updateData?.rawUpdate) return;
        
        try {
            downloading = true;
            let downloaded = 0;
            const update = appState.updateData.rawUpdate;
            let contentLength: number | undefined = 0;
            
            await update.downloadAndInstall((event: any) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength;
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        if (contentLength) {
                            downloadProgress = Math.round((downloaded / contentLength) * 100);
                        }
                        break;
                    case 'Finished':
                        break;
                }
            });

            toast.success($_('settings.install_update'));
            await relaunch();
        } catch (e) {
            console.error(e);
            toast.error($_('settings.update_error') + ": " + e);
            downloading = false;
        }
    }

    // "Rising Particles" Logic (Adapted from SCSS loop)
    const particles = Array.from({ length: 20 }).map((_, i) => ({
        id: i,
        size: Math.random() * 5 + 1, // 1px to 6px
        startX: Math.random() * 100, // 0vw to 100vw
        endX: Math.random() * 100, // 0vw to 100vw (random drift)
        duration: Math.random() * 4 + 7, // 7s to 11s (7000ms + random(4000)ms)
        delay: Math.random() * 11, // 0s to 11s
        opacityDelay: Math.random() * 4, // Random delay for the inner circle fade
    }));
    // Check if any game is running or starting
    let gameRunning = $derived(
        appState.instances.some(
            (i) => i.state === "Running" || i.state === "Starting",
        ),
    );
</script>

<!-- Main Container with Radial Gradient Background -->
<div
    class="w-full h-full flex flex-col items-center justify-center p-6 pb-[20vh] z-10 relative overflow-hidden transition-colors duration-1000"
    style="background-image: radial-gradient(#334565, #111621);"
    data-tauri-drag-region
>
    <!-- Particle Container -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
        {#each gameRunning ? particles.slice(0, 20) : particles as p}
            <div
                class="circle-container"
                style="
                    --size: {p.size}px;
                    --start-x: {p.startX}vw;
                    --end-x: {p.endX}vw;
                    --duration: {p.duration}s;
                    --delay: {p.delay}s;
                    --opacity-delay: {p.opacityDelay}s;
                "
            >
                <div class="circle"></div>
            </div>
        {/each}
    </div>

    <style>
        /* Particle styling adapted from snippet */
        .circle-container {
            position: absolute;
            top: 0; /* Positioned relative to container, animation handles movement */
            left: 0;
            width: var(--size);
            height: var(--size);
            animation: floatUp var(--duration) linear infinite;
            animation-delay: var(--delay);
            /* Start below screen */
            transform: translate3d(var(--start-x), 110vh, 0);
            will-change: transform;
        }

        .circle {
            width: 100%;
            height: 100%;
            border-radius: 50%;
            /* mix-blend-mode: screen; Removed for performance */
            background-image: radial-gradient(
                hsl(180, 100%, 80%),
                hsl(180, 100%, 80%) 10%,
                hsla(180, 100%, 80%, 0) 56%
            );

            animation:
                fadeFrames 200ms infinite,
                scaleFrames 2s infinite;
            animation-delay: var(--opacity-delay);
        }

        /* Keyframes */
        @keyframes floatUp {
            from {
                transform: translate3d(var(--start-x), 110vh, 0);
            }
            to {
                transform: translate3d(var(--end-x), -20vh, 0);
            }
        }

        @keyframes fadeFrames {
            0% {
                opacity: 1;
            }
            50% {
                opacity: 0.7;
            }
            100% {
                opacity: 1;
            }
        }

        @keyframes scaleFrames {
            0% {
                transform: scale3d(0.4, 0.4, 1);
            }
            50% {
                transform: scale3d(2.2, 2.2, 1);
            }
            100% {
                transform: scale3d(0.4, 0.4, 1);
            }
        }

        /* Existing Entrance Animations */
        @keyframes fadeInUp {
            from {
                opacity: 0;
                transform: translate3d(0, 20px, 0);
            }
            to {
                opacity: 1;
                transform: translate3d(0, 0, 0);
            }
        }
        .animate-enter {
            animation: fadeInUp 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;
            opacity: 0;
        }
        .delay-100 {
            animation-delay: 0.1s;
        }
        .delay-200 {
            animation-delay: 0.2s;
        }
        .delay-300 {
            animation-delay: 0.3s;
        }

        /* New Button Styles */
        .btn-grad {
            background-image: linear-gradient(
                35deg,
                #a4ff4d 0%,
                #ffd700 51%,
                #a4ff4d 100%
            );
            background-size: 200% auto;
            border: 0;
            border-radius: 1rem; /* rounded-2xl */
            box-shadow: rgba(164, 255, 77, 0.2) 0 15px 30px -5px;
            box-sizing: border-box;
            color: black;
            display: flex;
            padding: 3px;
            text-decoration: none;
            user-select: none;
            cursor: pointer;
            transition: 0.5s;
            touch-action: manipulation;
        }

        .btn-grad:hover {
            background-position: right center; /* change the direction of the change here */
            color: black;
        }

        .btn-grad:active {
            transform: scale(0.95);
        }

        .btn-grad span {
            background-color: transparent;
            padding: 16px 32px;
            border-radius: 0.9rem;
            width: 100%;
            height: 100%;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            font-weight: 700;
        }
    </style>

    <!-- Update Notification Banner (Real Logic) - Absolute Top Center -->
    {#if appState.updateData}
        <div class="absolute top-4 left-1/2 -translate-x-1/2 w-fit max-w-sm z-50 animate-enter" transition:fade>
            <div 
                class="relative group cursor-pointer block text-left"
                role="button"
                tabindex="0"
                onclick={() => showUpdateConfirm = true}
                onkeydown={(e) => e.key === 'Enter' && (showUpdateConfirm = true)}
            >
                <!-- Glow Effect (Amber/Orange) -->
                <div class="absolute -inset-0.5 bg-gradient-to-r from-amber-500 to-orange-500 rounded-2xl blur opacity-20 group-hover:opacity-40 transition duration-1000 group-hover:duration-200"></div>
                
                <!-- Main Card -->
                <div class="relative px-4 py-3 bg-[#0f172a]/80 backdrop-blur-xl border border-amber-500/20 rounded-2xl flex items-center gap-4 shadow-2xl transition-transform active:scale-95">
                    <div class="w-10 h-10 rounded-full bg-amber-500/10 flex items-center justify-center border border-amber-500/30 shadow-[0_0_15px_rgba(245,158,11,0.1)]">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                            <polyline points="7 10 12 15 17 10"></polyline>
                            <line x1="12" y1="15" x2="12" y2="3"></line>
                        </svg>
                    </div>
                    <div class="flex flex-col flex-1">
                        <div class="flex items-center gap-2">
                            <span class="text-white font-bold text-[17px]">
                                {appState.updateData.isCritical ? 'Actualización Crítica' : '¡Actualización Disponible!'}
                            </span>
                            <span class="px-1.5 py-0.5 rounded {appState.updateData.isCritical ? 'bg-red-500 text-white' : 'bg-amber-500 text-black'} text-[11px] font-black uppercase tracking-wider shadow-lg">
                                v{appState.updateData.version}
                            </span>
                        </div>
                        <p class="text-zinc-400 text-[14px] font-medium leading-tight">
                            Revisa los cambios en 
                            <button 
                                class="text-amber-400 font-bold hover:underline"
                                onclick={(e) => { e.stopPropagation(); openUrl('https://github.com/cPathz/AnvilCraft-Panel/releases/latest'); }}
                            >
                                Github.
                            </button>
                        </p>
                    </div>
                </div>
            </div>
        </div>
    {/if}

    <!-- Content -->
    <div class="relative z-10 flex flex-col items-center">
        <!-- Welcome Text -->
        <h1
            class="text-4xl md:text-5xl font-extrabold text-[#FFFFFF] text-center tracking-tight drop-shadow-md animate-enter delay-100 select-none pb-2"
        >
            {$_ ? $_("home.welcome") : "Welcome"}
        </h1>

        <!-- Subtitle -->
        <p
            class="text-xl text-[#D0D0D0] font-medium text-center max-w-lg leading-relaxed animate-enter delay-200 select-none mt-6 drop-shadow-sm"
        >
            {$_ ? $_("home.subtitle_1") : "Managing your instances"}
            <span class="block mt-1 text-[#D0D0D0]/80"
                >{$_ ? $_("home.subtitle_2") : "made simple."}</span
            >
        </p>

        <!-- CTA Button -->
        <button
            class="btn-grad mt-10 animate-enter delay-300"
            onclick={() => (appState.creatingInstance = true)}
        >
            <span class="relative">
                <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><line x1="12" y1="5" x2="12" y2="19"></line><line
                        x1="5"
                        y1="12"
                        x2="19"
                        y2="12"
                    ></line></svg
                >
                {$_ ? $_("home.create") : "Create Instance"}
            </span>
        </button>
    </div>
</div>

<!-- Update Confirmation Dialog -->
{#if showUpdateConfirm}
    <div 
        class="fixed inset-0 z-[100] flex items-center justify-center p-6"
        transition:fade={{ duration: 200 }}
    >
        <!-- Overlay (BLOQUEO TOTAL) -->
        <div class="absolute inset-0 bg-black/80 backdrop-blur-lg"></div>
        
        <!-- Modal -->
        <div 
            class="relative w-full max-w-[360px] bg-[#111827] border border-white/10 rounded-[2rem] shadow-2xl overflow-hidden p-7 flex flex-col items-center text-center gap-5"
            transition:scale={{ duration: 300, start: 0.9, opacity: 0 }}
        >
            <div class="w-14 h-14 rounded-full bg-blue-500/10 flex items-center justify-center border border-blue-500/20 shadow-inner">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                    <polyline points="7 10 12 15 17 10"></polyline>
                    <line x1="12" y1="15" x2="12" y2="3"></line>
                </svg>
            </div>
            
            <div class="space-y-1.5">
                <h2 class="text-xl font-black text-white leading-tight">{$_('settings.update_available')} (v{appState.updateData?.version})</h2>
                <p class="text-zinc-400 text-xs font-medium px-2">{$_('home.update_restart_warning')}</p>
            </div>
            
            <div class="flex flex-col w-full gap-2.5 mt-1">
                {#if downloading}
                    <div class="w-full space-y-2 my-2 animate-pulse">
                        <div class="flex justify-between text-[10px] font-black uppercase tracking-widest text-blue-400">
                            <span>{$_('common.status_downloading')}</span>
                            <span>{downloadProgress}%</span>
                        </div>
                        <div class="w-full h-1.5 bg-white/5 rounded-full overflow-hidden border border-white/5">
                            <div 
                                class="h-full bg-gradient-to-r from-blue-600 to-blue-400 transition-all duration-300 ease-out" 
                                style="width: {downloadProgress}%"
                            ></div>
                        </div>
                        <p class="text-[9px] text-zinc-500 italic">{$_('home.update_preparing')}</p>
                    </div>
                {:else}
                    <button 
                        class="w-full py-2.5 bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold rounded-xl transition-all shadow-lg shadow-blue-900/20 active:scale-[0.98]"
                        onclick={handleInstall}
                    >
                        {$_('common.toast_success')}
                    </button>
                    {#if !appState.updateData?.isCritical}
                        <button 
                            class="w-full py-2.5 bg-white/5 hover:bg-white/10 text-zinc-500 text-xs font-bold rounded-xl transition-all active:scale-[0.98]"
                            onclick={() => showUpdateConfirm = false}
                        >
                            Quizás luego
                        </button>
                    {/if}
                {/if}
            </div>
        </div>
    </div>
{/if}

<!-- Redirect Confirmation Dialog -->
{#if showRedirectConfirm}
    <div 
        class="fixed inset-0 z-[100] flex items-center justify-center p-6"
        transition:fade={{ duration: 200 }}
    >
        <!-- Overlay -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-md" onclick={() => showRedirectConfirm = false}></div>
        
        <!-- Modal -->
        <div 
            class="relative w-full max-w-[360px] bg-[#111827] border border-white/10 rounded-[2rem] shadow-2xl overflow-hidden p-7 flex flex-col items-center text-center gap-5"
            transition:scale={{ duration: 300, start: 0.9, opacity: 0 }}
        >
            <div class="w-14 h-14 rounded-full bg-amber-500/10 flex items-center justify-center border border-amber-500/20 shadow-inner">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                    <polyline points="15 3 21 3 21 9"></polyline>
                    <line x1="10" y1="14" x2="21" y2="3"></line>
                </svg>
            </div>
            
            <div class="space-y-1.5">
                <h2 class="text-xl font-black text-white leading-tight">{$_('home.redirect_title')}</h2>
                <p class="text-zinc-400 text-xs font-medium px-2">
                    {$_('settings.updates_desc')}
                </p>
                <div class="bg-black/40 p-2.5 rounded-xl border border-white/5 mx-2 mt-1">
                    <p class="text-amber-500/80 font-mono text-[10px] break-all">{mockUrl}</p>
                </div>
                <p class="text-zinc-500 text-[10px] mt-1 italic">{$_('home.redirect_confirm')}</p>
            </div>
            
            <div class="flex flex-col w-full gap-2.5 mt-1">
                <button 
                    class="w-full py-2.5 bg-amber-600 hover:bg-amber-500 text-white text-xs font-bold rounded-xl transition-all shadow-lg shadow-amber-900/20 active:scale-[0.98]"
                    onclick={async () => {
                        showRedirectConfirm = false;
                        await openUrl(mockUrl);
                    }}
                >
                    {$_('common.toast_success')}
                </button>
                <button 
                    class="w-full py-2.5 bg-white/5 hover:bg-white/10 text-zinc-500 text-xs font-bold rounded-xl transition-all active:scale-[0.98]"
                    onclick={() => showRedirectConfirm = false}
                >
                    {$_('instance_detail.btn_cancel')}
                </button>
            </div>
        </div>
    </div>
{/if}
