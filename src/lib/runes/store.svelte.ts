
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
}

class AppState {
    instances = $state<Instance[]>([]);
    selectedInstance = $state<Instance | null>(null);
    view = $state<'home' | 'instances' | 'settings'>('home');
    refreshing = $state<boolean>(false);
    creatingInstance = $state<boolean>(false);
}

export const appState = new AppState();
