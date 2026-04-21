
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
        version: "0.1.2",
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
        
        // Flexible split: Find the first ": " after the timestamp/level headers
        // Format can be [HH:mm:ss INFO]: or [HH:mm:ss] [Server thread/INFO]:
        const match = cleanLine.match(/^\[\d{2}:\d{2}:\d{2}.*?\]:?\s+(.*)/);
        if (!match) return;

        const msg = match[1].trim();

        // Join detection
        const joinMatch = msg.match(/^(.*) joined the game$/);
        if (joinMatch) {
            const name = joinMatch[1].trim();
            if (!runtime.players.includes(name)) {
                runtime.players.push(name);
            }
        }

        // Leave detection
        const leaveMatch = msg.match(/^(.*) left the game$/);
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
