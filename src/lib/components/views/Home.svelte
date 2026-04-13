<script lang="ts">
    import { _ } from "svelte-i18n";
    import { appState } from "$lib/runes/store.svelte";

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

    <!-- Content -->
    <div class="relative z-10 flex flex-col items-center">
        <!-- Welcome Text -->
        <h1
            class="text-4xl md:text-5xl font-extrabold text-[#FFFFFF] text-center tracking-tight drop-shadow-md animate-enter delay-100 select-none pb-2"
        >
            {$_("home.welcome")}
        </h1>

        <!-- Subtitle -->
        <p
            class="text-xl text-[#D0D0D0] font-medium text-center max-w-lg leading-relaxed animate-enter delay-200 select-none mt-6 drop-shadow-sm"
        >
            {$_("home.subtitle_1")}
            <span class="block mt-1 text-[#D0D0D0]/80"
                >{$_("home.subtitle_2")}</span
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
                {$_("home.create")}
            </span>
        </button>
    </div>
</div>
