
export interface Instance {
    uuid: string;
    name: string;
    motor: 'Vanilla' | 'Paper' | 'Fabric' | 'Forge' | 'Spigot';
    version: string;
    path: string;
    icon?: string;
    ram_min: number;
    ram_max: number;
    port: number;
    state: 'Stopped' | 'Starting' | 'Running' | 'Error';
}

class AppState {
    instances = $state<Instance[]>([]);
    selectedInstance = $state<Instance | null>(null);
    refreshing = $state<boolean>(false);
}

export const appState = new AppState();
