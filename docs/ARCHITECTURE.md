# AnvilCraft Panel — Arquitectura

> **Este documento es tu mapa para hacer cambios al proyecto.** Antes de modificar algo, búscalo aquí primero.

## 🎯 Qué es AnvilCraft

Aplicación de escritorio (Tauri 2 + SvelteKit) para gestionar servidores de Minecraft en Windows. **Privacidad y todo-local** — los datos nunca salen de tu PC.

## 📐 Vista general de la arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (Svelte 5)              src/                          │
│  - UI, i18n, navegación                                            │
│  - Llama al backend vía invoke()                                  │
├─────────────────────────────────────────────────────────────────┤
│                  IPC (Tauri messages)                              │
├─────────────────────────────────────────────────────────────────┤
│  Backend (Rust)                  src-tauri/src/                 │
│  - 35+ commands accesibles desde JS                              │
│  - Loaders, models, file watcher                                  │
│  - Spawn de procesos Minecraft (Child)                            │
└─────────────────────────────────────────────────────────────────┘
```

## 📁 Estructura de carpetas

```
AnvilCraft/
├── src/                            # ── FRONTEND ──
│   ├── routes/                     # SvelteKit pages (solo +layout, +page)
│   └── lib/
│       ├── components/             # UI organizada por subdir
│       │   ├── views/              # Pantallas principales
│       │   ├── modals/             # Modales
│       │   ├── console/            # Vista de consola
│       │   └── settings/           # Paneles de settings por instancia
│       ├── loaders/                # MIRROR TS de los loaders Rust
│       ├── data/                   # JSONs estáticos (icons, commands)
│       ├── locales/                # ← TRADUCCIONES i18n (en.json, es.json, ...)
│       ├── runes/                  # Estado global (Svelte 5 Runes)
│       ├── utils/                  # Utilidades (logParser, ansiConverter)
│       ├── types/                  # TypeScript types
│       ├── config/                 # Feature flags
│       ├── i18n.ts                 # Setup de svelte-i18n
│       └── assets/                 # Imágenes
│
├── src-tauri/                      # ── BACKEND ──
│   ├── src/
│   │   ├── main.rs                 # Entry point
│   │   ├── lib.rs                  # Builder + invoke_handler (35 commands)
│   │   ├── models.rs               # Structs: Instance, Addon, InstanceEngine
│   │   ├── parser/                 # Parsing de logs
│   │   ├── loaders/                # ← 16 loaders en 5 categorías
│   │   │   ├── mod.rs              # trait LoaderStrategy
│   │   │   ├── registry.rs         # LoaderRegistry (16 entradas)
│   │   │   ├── vanilla.rs          # Único loader implementado
│   │   │   ├── bukkit.rs           # Paper, Spigot, Purpur, Folia (stubs)
│   │   │   ├── mods.rs             # NeoForge, Forge, Fabric, Quilt (stubs)
│   │   │   ├── proxies.rs          # Velocity, Waterfall, BungeeCord
│   │   │   └── hybrids/            # Mohist, Arclight, Banner, Magma
│   │   ├── commands/               # 8 archivos, uno por dominio
│   │   │   ├── instance.rs         # CRUD de instancias
│   │   │   ├── server.rs           # start/stop/kill
│   │   │   ├── versions.rs         # Listas de MC
│   │   │   ├── java.rs             # JREs (Adoptium + próximamente Azul Zulu)
│   │   │   ├── curseforge.rs       # Modpack import
│   │   │   ├── system.rs           # Info del sistema
│   │   │   ├── dev.rs              # Dev tools
│   │   │   └── version.rs          # Versión de la app
│   │   └── lib/data/               # ← Data embebida en el binario
│   │       ├── minecraft/          # JSONs por versión de MC (100+ archivos)
│   │       ├── arguments.json
│   │       └── command_tree.json
│   ├── capabilities/default.json  # Permisos Tauri
│   ├── icons/                      # Íconos multi-formato
│   └── tauri.conf.json             # Config Tauri (ventana, bundle, updater)
│
├── .github/
│   ├── workflows/release.yml       # CI: build + release en tag push
│   ├── ISSUE_TEMPLATE/             # Templates de issues
│   └── PULL_REQUEST_TEMPLATE.md    # Template de PRs
│
├── docs/                           # Documentación pública
│   ├── ARCHITECTURE.md             # ← este archivo
│   ├── SECURITY_AUDIT.md
│   ├── ROADMAP_STORE.md
│   └── CONSOLE_PERFORMANCE.md
│
└── .github/workflows/release.yml
```

## 🗺️ Dónde hacer cambios comunes

### "Quiero cambiar/agregar un idioma"
→ Edita `src/lib/locales/<idioma>.json`
→ Ver `CONTRIBUTING.md` para agregar uno nuevo.

### "Quiero cambiar el texto de un botón/menú"
→ Busca el texto en `src/lib/locales/en.json`
→ Encuentra la clave (ej. `instance.btn_create`)
→ Cámbiala en TODOS los archivos de `locales/`
→ (O solo en `en.json` y deja que un colaborador traduzca después)

### "Quiero agregar un item al menú lateral"
→ Edita `src/lib/components/NavigationRail.svelte`
→ Busca el array de items del menú
→ Agrega tu entrada

### "Quiero cambiar la configuración global"
→ Edita `src/lib/runes/store.svelte.ts` (AppState class)
→ O `src/lib/config/features.ts` (feature flags)

### "Quiero agregar un mod loader"
→ **Backend:** crea `src-tauri/src/loaders/<categoría>/<nombre>.rs`
→ **Backend:** implementa `LoaderStrategy` trait
→ **Backend:** registra en `src-tauri/src/loaders/registry.rs`
→ **Backend:** agrega variante a `InstanceEngine` enum en `src-tauri/src/models.rs`
→ **Frontend:** agrega al array en `src/lib/loaders/catalog.ts`
→ Ver `CONTRIBUTING.md` sección "Agregar un mod loader"

### "Quiero agregar un comando Tauri (expuesto al frontend)"
→ Crea función async en `src-tauri/src/commands/<dominio>.rs`
→ Decora con `#[tauri::command]`
→ Registra en `src-tauri/src/lib.rs` (sección `invoke_handler`)
→ Llámala desde el frontend con `invoke('nombre_comando', { args })`

### "Quiero cambiar los colores/tema"
→ Tailwind config en `tailwind.config.js`
→ Variables CSS en `src/app.css`
→ Theme de consola en `src/lib/runes/store.svelte.ts` (settings.console.theme)

### "Quiero cambiar el ícono de la app"
→ Íconos en `src-tauri/icons/` (varios tamaños)
→ Tauri usa estos automáticamente en el build

## 🔄 Flujo de datos típico (crear instancia)

```
1. Usuario click "Add Instance" en Home.svelte
2. CreateInstanceModal.svelte abre
3. invoke('get_project_versions', { loader: 'Paper' })
   → commands/versions.rs::get_project_versions
   → LoaderRegistry::by_engine(Paper) → PaperLoader
   → PaperLoader::fetch_versions()
4. invoke('create_instance', { ... })
   → commands/instance.rs::create_instance
   → Crea carpeta en %APPDATA%/AnvilCraftPanel/instances/
   → LoaderStrategy::install() (descarga JAR, etc.)
5. Emite evento "instance-update" → frontend refresca
```

## 🎯 Patrones importantes

### Strategy pattern (loaders)
- **Trait:** `LoaderStrategy` en `src-tauri/src/loaders/mod.rs`
- **Registry:** `LoaderRegistry` con `OnceLock` (singleton)
- **16 loaders** registrados, **5 categorías**

### Singleton + IPC
- `app.manage(ChildProcessMap(...))` — procesos vivos
- `app.manage(AddonWatcherState(...))` — file watcher
- `app.emit("event", payload)` — notifica al frontend

### State management (Svelte 5 Runes)
- `appState` es una clase con `$state(...)` properties
- Singleton exportado desde `runes/store.svelte.ts`
- Persistencia automática a `localStorage` con `$effect(...)`

### i18n con svelte-i18n
- Setup en `lib/i18n.ts`
- Idiomas en `lib/locales/`
- Lazy loading con `register('en', () => import('./locales/en.json'))`

## 🛠️ Build y release

- **Dev:** `cargo tauri dev` (inicia Vite + compila Rust + abre ventana)
- **Build:** `cargo tauri build` (genera `.msi` y `.exe` con auto-update)
- **Release:** push de tag `v*` dispara `.github/workflows/release.yml` que firma y publica

## 📚 Más documentación

- `SECURITY_AUDIT.md` — auditoría de seguridad de dependencias
- `ROADMAP_STORE.md` — roadmap de Microsoft Store
- `CONSOLE_PERFORMANCE.md` — optimizaciones de la consola
- `CONTRIBUTING.md` — cómo contribuir
- `docs/ANALYSIS.md` — análisis técnico (privado, ignorado de git)
