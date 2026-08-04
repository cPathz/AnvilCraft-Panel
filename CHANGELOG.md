# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Azul Zulu section in Portable Java settings** (`src/lib/components/settings/InstanceSettings.svelte`): nueva sección UI entre Adoptium y "Java Args" que descarga Java 12, 13 y 14 desde `api.azul.com/metadata/v1/zulu/packages`. Cubre los huecos que Adoptium no distribuye (versiones no-LTS pre-17). **Crítico para modpacks que piden Java 14** (ej. SkyFactory One 1.0.7). Estructura en disco: `%APPDATA%/AnvilCraftPanel/runtimes/java/zulu-{12,13,14}/`.
- **Adoptium version expansion** (`src-tauri/src/commands/java.rs:20`): lista de versiones portables alineada con el catálogo oficial. De 6 versiones (`8, 11, 16, 17, 21, 25`) a 13 (`8, 11, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26`). Java 16 sigue pineado a `jdk-16.0.2+7` (special case preservado).
- **Forge version fetching** (`src-tauri/src/commands/versions.rs`): atajo para "forge" en `get_project_versions` que consulta `https://files.minecraftforge.net/maven/net/minecraftforge/forge/maven-metadata.xml` y parsea con regex. Antes el modal 404eaba contra PaperMC API (que no tiene proyecto "forge").

### Fixed
- **Forge installer "program not found"** (`src-tauri/src/loaders/mods.rs:496` + `run_neoforge_installer:220`): el `Command::new("java")` requería `java` en el PATH de Windows. Cuando Java se instala desde el panel (Adoptium/Zulu), el ejecutable vive en `%APPDATA%/AnvilCraftPanel/runtimes/java/{ver}/bin/java.exe` y **no** se agrega al PATH. Ahora `find_any_java_executable()` busca en el directorio del panel antes de hacer fallback al PATH. Si no encuentra nada, error claro: *"Java not found. Install a Java runtime from Settings → Portable Java, or set the JAVA_HOME environment variable."* NeoForge se benefició del mismo fix preventivo.
- **Forge installer hangs at "Finalizing download... 100%"** (`src-tauri/src/loaders/mods.rs:471-668`): pipe buffer deadlock. `Stdio::piped()` + `.wait()` sin drain bloqueaba al installer a ~136MB cuando el buffer del OS se llenaba. Ahora se hace drain de stdout/stderr en tasks tokio paralelas (mismo patrón que `run_neoforge_installer`), con tee a `install.log` y progress events a la UI. Bonus: la UI ahora muestra "Forge: Patching 142/..." en vez de mentir con "Finalizing download... 100%".

### Notes
- **Migración desde RESPALDO**: estos features vienen del backup local `P:/Proyectos/Anvilcraft RESPALDO/AnvilCraft/dev_log.md` (entries 2026-07-01 y 2026-06-18) que no se habían pusheado a git. Ahora sincronizados.
- **Pendiente**: hay un cambio sin commitear en `src-tauri/Cargo.toml` (bump `0.1.14 → 0.1.16`) que el sub-agente dejó pendiente. Decidir si commitear o descartar.

## [0.1.16] - 2026-08-02

### Changed
- MSIX build step ahora es opt-in via variable de repo `BUILD_MSIX`. El workflow ya no falla cuando MSIX no se quiere construir. Se setea en GitHub repo → Settings → Variables → Actions.

### Fixed
- `MSIX_CERT_PASSWORD` se pasa correctamente al step de PowerShell (antes faltaba en el bloque `env:` del workflow, requería re-tag para que funcionara).

## [0.1.15] - 2026-08-02

### Security
- **Moved hardcoded secrets to environment variables**:
  - `CURSEFORGE_API_KEY`: now loaded from `CURSEFORGE_API_KEY` env var (was hardcoded in `src-tauri/src/commands/curseforge.rs`).
  - `MSIX_CERT_PASSWORD`: now loaded from `MSIX_CERT_PASSWORD` env var (was hardcoded in `scripts/build-msix.ps1`).
- **Rotated exposed CurseForge API key** (was visible in git history before this commit).
- **Cleaned git history with `git-filter-repo`**: secret redactions applied to all 164 commits, then force-pushed.
- **Added `.env.example`** as a public template for required environment variables.
- **Updated `.gitignore`** to ignore `.env*` (allowing `.env.example` to be committed).
- **Enhanced `docs/SECURITY_AUDIT.md`** with full security analysis (XSS, command injection, path traversal, capabilities, secret remediation).

### Added
- **`.env.example`** template documenting `CURSEFORGE_API_KEY` and `MSIX_CERT_PASSWORD` variables.
- **Security section in `docs/SECURITY_AUDIT.md`** covering secret remediation, capabilities analysis, and threat model.

## [0.1.14] - 2026-08-02

### Changed
- **Version sync**: Sincronizada la versión a 0.1.14 en `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/msix/AppxManifest.xml` y badge de `README.md`. (Antes: 0.1.13 en Cargo, v0.1.2 en badge.)

### Removed
- **Dead code cleanup**: `src-tauri/src/lib/data/` (161 JSONs huérfanos de definiciones de Minecraft, ~11 MB). Verificado: 0 matches de `include_str!`/`include_bytes!`/`include_dir!`. La data real vive en `src/lib/data/` (lado Svelte).
- **Dead code cleanup**: `src/lib/i18n/` (carpeta muerta de traducciones, no se usaba — `i18n.ts` solo importa de `src/lib/locales/`).

### Added
- **i18n tooling**:
  - `scripts/translate.mjs` — genera traducciones base desde `en.json` con `@parvineyvazov/json-translator` (Google/Bing/Libre/DeepL/GPT, gratis, sin API key).
  - `scripts/validate-locales.mjs` — valida que todos los locales tengan la misma estructura de claves que `en.json` y reporta claves faltantes, sobrantes o vacías.
  - `npm run translate -- <código>` para generar un idioma.
  - `npm run translate:validate` para validar.
- **Documentación**:
  - `CONTRIBUTING.md` — guía de contribuidores (i18n, loaders, dev setup).
  - `docs/ARCHITECTURE.md` — mapa del proyecto, dónde cambiar cada cosa.
  - `docs/I18N.md` — guía completa de internacionalización.
  - `docs/TROUBLESHOOTING.md` — log de problemas conocidos y soluciones.
  - `.editorconfig` — formato consistente entre editores.
  - `.github/ISSUE_TEMPLATE/{bug_report,feature_request}.md`.
  - `.github/PULL_REQUEST_TEMPLATE.md`.

## [0.1.4] - 2026-04-20

### Added
- **Console Player Panel**: 
    - Real-time player tracking by parsing server logs (Join/Leave/List events).
    - **Reactive Sync**: Automatically detects and refreshes `max-players` limit from `server.properties` on server start and manual refresh.
    - **Full i18n Support**: All labels (Users, Slots, Roles) localized in English and Spanish.
    - Robust log parser supporting ANSI escape code stripping and multiple log formats.
    - High-resolution player avatars (64px) powered by MCHeads API.
    - UI Refinements: Increased typography sizes (+3px) for improved dashboard readability.
    - Compact and refined side panel design (`w-52`) with "Pixel Perfect" alignment.

## [0.1.3] - 2026-04-13

### Added
- **Java Environments**: Added support for **Java 25 (LTS)** in portable environments.
- **Port Management**: Bidirectional synchronization between the UI and `server.properties`.

### Changed
- **Instance Management**: 
    - In-place renaming of display names in the instance detail view.
    - Professional unification of app identifier (`AnvilCraftPanel`).
- **Java Support**: Fixed Java 16 downloads via archived JDK binaries.
- **Console**: Fixed overflow issues and scroll synchronization.

## [0.1.2] - 2026-04-12

### Added
- **Internationalization (i18n)**: 
    - Full support for English and Spanish using `svelte-i18n`.
    - Automatic language detection and manual toggle in settings.
- **Notification System**:
    - Replaced all native `alert()` calls with a custom, localized Toast notification system.
    - Improved UX with success/error/warning states.
- **Progress Tracking**:
    - Localized progress states for server installation and environment setup.

### Changed
- **UI/UX Refinement**:
    - Unified versioning and branding across the entire panel (`v0.1.2 Beta`).
    - Redesigned "About" section in settings for a more modern, compact look.
    - Simplified TopBar with integrated brand and version info.

## [Unreleased]

### Added
- **Feature Flags**: Centralized configuration in `src/lib/config/features.ts` to manage experimental features.
- **Console**: 
    - Implemented robust command autocomplete with `command_tree.json`.
    - Added argument parsers for `block`, `item`, `entity`.
    - Added custom scrollbar and "Pixel Perfect" styling to autocomplete menu.
- **Backend**:
    - Added `update_instance_icon` Tauri command.
    - Integrated Paper/Purpur/Velocity/Waterfall version fetching.

### Changed
- **Console UI**:
    - Increased font size to `text-base` (16px).
    - Refined selection color to Yellow (Minecraft style).
    - Improved navigation responsiveness (removed smooth scroll lag).
- **Core**:
    - Updated `InstanceDetail` to use global feature flags.
- **Refactorización de Backend (Rust)**:
  - Se dividió `lib.rs` en módulos (`commands/`, `models.rs`) para mejorar la mantenibilidad.
  - Comandos organizados en categorías: `instance`, `server`, `system`, `versions`.
- **Refactorización de Frontend**:
  - `InstanceDetail.svelte` descompuesto en `ConsoleView.svelte` y `InstanceSettings.svelte`.

### Fixed
- **Console**:
    - Fixed "Text Drift" by preserving exact whitespace in syntax highlighting.
    - Fixed "Unexpected token" syntax error in console logic.
