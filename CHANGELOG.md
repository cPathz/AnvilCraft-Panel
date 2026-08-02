# Changelog

All notable changes to this project will be documented in this file.

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
