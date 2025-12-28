
export interface Instance {
    id: string;
    name: string;
    loader: 'Vanilla' | 'Paper' | 'Fabric' | 'Forge' | 'NeoForge' | 'Quilt';
    version: string;
    path: string;
    icon: string;
    date_created: string;
    last_played: string | null;
    state: 'Stopped' | 'Starting' | 'Running' | 'Error';
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

    // Runtime state (Logs, active tabs, etc)
    instanceRuntime = $state<Record<string, { logs: string[], activeTab: "console" | "settings", commandHistory: string[] }>>({});

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
            this.instanceRuntime[id] = { logs: [], activeTab: "console", commandHistory: [] };
        }
    }

    getRuntime(id: string) {
        return this.instanceRuntime[id];
    }
}

export const appState = new AppState();
