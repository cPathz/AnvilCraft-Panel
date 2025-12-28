# Changelog

All notable changes to this project will be documented in this file.

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
