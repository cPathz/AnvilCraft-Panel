
export interface Instance {
    id: string;
    name: string;
    loader: 'Vanilla' | 'Paper' | 'Fabric' | 'Forge' | 'NeoForge' | 'Quilt';
    version: string;
    path: string;
    icon: string;
    date_created: string;
    last_played: string | null;
    state: 'Stopped' | 'Starting' | 'Running' | 'Stopping' | 'Error';
    settings?: {
        min_ram: number;
        max_ram: number;
        port: number;
        args: string;
        jar_file: string;
    };
    build?: string;
}

class AppState {
    instances = $state<Instance[]>([]);
    selectedInstance = $state<Instance | null>(null);
    view = $state<'home' | 'instances' | 'settings' | 'dev'>('home');
    refreshing = $state<boolean>(false);
    creatingInstance = $state<boolean>(false);

    // App Identity
    appInfo = $state({
        version: "0.1.4",
        tag: "Beta",
        isEvalCopy: true
    });

    // Runtime state (Logs, active tabs, etc)
    instanceRuntime = $state<Record<string, { 
        logs: string[], 
        activeTab: "console" | "settings", 
        commandHistory: string[],
        players: string[] 
    }>>({});

    // Global Settings
    settings = $state({
        console: {
            fontFamily: "JetBrains Mono",
            fontSize: 13,
            lineHeight: 1.1,
            letterSpacing: 0,
            fontWeight: "400", // Normal
            theme: "Campbell"
        }
    });

    ensureRuntime(id: string) {
        if (!this.instanceRuntime[id]) {
            this.instanceRuntime[id] = { 
                logs: [], 
                activeTab: "console", 
                commandHistory: [],
                players: []
            };
        }
    }

    getRuntime(id: string) {
        return this.instanceRuntime[id];
    }

    parseLog(id: string, line: string) {
        const runtime = this.instanceRuntime[id];
        if (!runtime) return;

        // Strip ANSI escape codes first
        const cleanLine = line.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, '').trim();
        
        // Flexible extraction: Handle [HH:mm:ss] [Level]: Msg OR [Level]: Msg OR just Msg
        // We look for the message after the last ": " or after the last "]: "
        let msg = cleanLine;
        const headerMatch = cleanLine.match(/^(?:\[.*?\]\s*)+(?::\s*)?(.*)/);
        if (headerMatch && headerMatch[1]) {
            msg = headerMatch[1].trim();
        }

        // Join detection (Vanilla, Forge, ES)
        const joinMatch = msg.match(/^(.*) joined the game$/) || 
                          msg.match(/^(.*)\[\/.*\] logged in with entity id/) ||
                          msg.match(/^(.*) se ha unido al juego$/);
        if (joinMatch) {
            const name = joinMatch[1].trim();
            if (!runtime.players.includes(name)) {
                runtime.players.push(name);
            }
        }

        // Leave detection (EN, ES)
        const leaveMatch = msg.match(/^(.*) left the game$/) ||
                           msg.match(/^(.*) ha abandonado el juego$/);
        if (leaveMatch) {
            const name = leaveMatch[1].trim();
            runtime.players = runtime.players.filter(p => p !== name);
        }

        // /list command detection
        const listMatch = msg.match(/^There are \d+ (?:of a max of \d+ )?players online: (.*)$/);
        if (listMatch) {
            const namesPart = listMatch[1].trim();
            if (namesPart) {
                // Handle ", " or " " or "," separators
                const names = namesPart.split(/,\s*|\s+/).map(n => n.trim()).filter(n => n.length > 0);
                runtime.players = [...new Set(names)]; // Unique players
            } else {
                runtime.players = [];
            }
        }
    }
}

export const appState = new AppState();
