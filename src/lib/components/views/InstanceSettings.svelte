<script lang="ts">
    import { appState } from "$lib/runes/store.svelte";

    // Options
    const fonts = [
        "JetBrains Mono",
        "Consolas",
        "Courier New",
        "Lucida Console",
        "monospace",
    ];
    const weights = [
        { label: "Thin", value: "100" },
        { label: "Extra Light", value: "200" },
        { label: "Light", value: "300" },
        { label: "Normal", value: "400" },
        { label: "Medium", value: "500" },
        { label: "Semi Bold", value: "600" },
        { label: "Bold", value: "700" },
        { label: "Extra Bold", value: "800" },
    ];

    let settings = $derived(appState.settings.console);
</script>

<div class="p-6 w-full max-w-full space-y-6 animate-fade-in-up">
    <!-- Header -->
    <div>
        <h2 class="text-2xl font-bold text-white mb-1">Terminal</h2>
        <p class="text-zinc-400 text-sm">
            Personaliza la apariencia de la consola
        </p>
    </div>

    <!-- Live Preview -->
    <div class="space-y-3">
        <span class="text-sm font-medium text-zinc-300 ml-1">Vista Previa</span>
        <div
            class="w-full h-48 bg-[#0f1520] rounded-xl border border-white/10 shadow-xl overflow-hidden p-4 select-none transition-all duration-200"
            style:font-family={settings.fontFamily}
            style:font-size="{settings.fontSize}px"
            style:line-height={settings.lineHeight}
            style:letter-spacing="{settings.letterSpacing}px"
            style:font-weight={settings.fontWeight}
        >
            <div class="text-zinc-300">
                <span class="text-green-400">user@AnvilCraft</span>:<span
                    class="text-blue-400">~/server</span
                >$ java -jar server.jar<br />
                <span class="text-blue-400 font-bold">[INFO]</span> Starting
                minecraft server version 1.21.1<br />
                <span class="text-blue-400 font-bold">[INFO]</span> Loading
                properties<br />
                <span class="text-blue-400 font-bold">[INFO]</span> Default game
                type: SURVIVAL<br />
                <span class="text-yellow-400 font-bold">[WARN]</span> Ambiguity
                between arguments [teleport, destination]<br />
                <span class="text-red-400 font-bold">[ERROR]</span> Failed to
                bind to port 25565<br />
                <span class="text-zinc-500"
                    >> echo "Test with {settings.fontFamily}"</span
                ><br />
                <span class="text-purple-400">diff --git a/win b/win</span><br
                />
                <span class="text-red-500">- Windows Console</span><br />
                <span class="text-green-500">+ Windows Terminal!</span>
            </div>
        </div>
    </div>

    <!-- Settings Controls -->
    <div class="space-y-4">
        <span class="text-sm font-medium text-zinc-300 ml-1">Texto</span>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <!-- Font Family -->
            <div
                class="bg-zinc-800/50 border border-white/5 rounded-xl p-4 flex flex-col gap-3"
                title="La tipografía utilizada en los logs"
            >
                <span class="font-medium text-white text-sm"
                    >Tipo de fuente</span
                >
                <select
                    bind:value={appState.settings.console.fontFamily}
                    class="bg-zinc-900 border border-zinc-700 text-white text-sm rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none appearance-none cursor-pointer transition-all"
                >
                    {#each fonts as font}
                        <option value={font} style:font-family={font}
                            >{font}</option
                        >
                    {/each}
                </select>
            </div>

            <!-- Font Size -->
            <div
                class="bg-zinc-800/50 border border-white/5 rounded-xl p-4 flex flex-col gap-3"
                title="Tamaño en píxeles (px)"
            >
                <span class="font-medium text-white text-sm"
                    >Tamaño de la fuente</span
                >
                <div class="flex items-center gap-3">
                    <input
                        type="range"
                        min="8"
                        max="24"
                        step="1"
                        bind:value={appState.settings.console.fontSize}
                        class="flex-1 accent-blue-500 h-1.5 bg-zinc-700 rounded-lg appearance-none cursor-pointer"
                    />
                    <div
                        class="bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 min-w-[3rem] text-center"
                    >
                        <span class="text-white text-sm"
                            >{appState.settings.console.fontSize}</span
                        >
                    </div>
                </div>
            </div>

            <!-- Line Height -->
            <div
                class="bg-zinc-800/50 border border-white/5 rounded-xl p-4 flex flex-col gap-3"
                title="Espaciado vertical entre líneas (Multiplicador)"
            >
                <span class="font-medium text-white text-sm">Alto de línea</span
                >
                <div class="flex items-center gap-3">
                    <input
                        type="range"
                        min="0.8"
                        max="2.0"
                        step="0.05"
                        bind:value={appState.settings.console.lineHeight}
                        class="flex-1 accent-blue-500 h-1.5 bg-zinc-700 rounded-lg appearance-none cursor-pointer"
                    />
                    <div
                        class="bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 min-w-[3rem] text-center"
                    >
                        <span class="text-white text-sm"
                            >{appState.settings.console.lineHeight.toFixed(
                                1,
                            )}</span
                        >
                    </div>
                </div>
            </div>

            <!-- Letter Spacing -->
            <div
                class="bg-zinc-800/50 border border-white/5 rounded-xl p-4 flex flex-col gap-3"
                title="Separación horizontal entre caracteres (px)"
            >
                <span class="font-medium text-white text-sm"
                    >Espaciado (Letter Spacing)</span
                >
                <div class="flex items-center gap-3">
                    <input
                        type="range"
                        min="-2"
                        max="5"
                        step="0.1"
                        bind:value={appState.settings.console.letterSpacing}
                        class="flex-1 accent-blue-500 h-1.5 bg-zinc-700 rounded-lg appearance-none cursor-pointer"
                    />
                    <div
                        class="bg-zinc-900 border border-zinc-700 rounded-lg px-2 py-1 min-w-[3rem] text-center"
                    >
                        <span class="text-white text-sm"
                            >{appState.settings.console.letterSpacing}</span
                        >
                    </div>
                </div>
            </div>

            <!-- Font Weight (New Row logically, but grid auto-flows) -->
            <div
                class="bg-zinc-800/50 border border-white/5 rounded-xl p-4 flex flex-col gap-3"
                title="Grosor de los caracteres"
            >
                <span class="font-medium text-white text-sm"
                    >Espesor de la fuente</span
                >
                <select
                    bind:value={appState.settings.console.fontWeight}
                    class="bg-zinc-900 border border-zinc-700 text-white text-sm rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none appearance-none cursor-pointer transition-all"
                >
                    {#each weights as weight}
                        <option value={weight.value}>{weight.label}</option>
                    {/each}
                </select>
            </div>
        </div>
    </div>
</div>
