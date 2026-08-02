# Contributing to AnvilCraft Panel

¡Gracias por tu interés en mejorar AnvilCraft! Este documento te guía para contribuir efectivamente.

## 🤝 Maneras de contribuir

- 🐛 **Reportar bugs** — usa la [issue template de bug](../../.github/ISSUE_TEMPLATE/bug_report.md)
- 💡 **Proponer features** — usa la [issue template de feature](../../.github/ISSUE_TEMPLATE/feature_request.md)
- 🌐 **Agregar un idioma** — ver [ abajo](#-agregar-un-idioma)
- 🧩 **Agregar un mod loader** — ver [abajo](#-agregar-un-mod-loader)
- 🔧 **Mejorar código existente** — abre un PR
- 📚 **Mejorar documentación** — abre un PR

## 🌐 Agregar un idioma

AnvilCraft usa [`svelte-i18n`](https://github.com/svelte-i18n/svelte-i18n) con archivos JSON en `src/lib/locales/`.

### Paso a paso

1. **Fork** el repo y clónalo localmente.
2. Copia `src/lib/locales/en.json` a `src/lib/locales/<tu-código-de-idioma>.json`
   - Usa códigos ISO 639-1: `en`, `es`, `fr`, `de`, `pt`, `ja`, etc.
3. **Traduce solo los valores**, nunca las claves:
   ```json
   // ❌ Mal
   { "common": { "save": "Guardar" } }

   // ✅ Bien
   { "common": { "save": "Save" } }
   ```
4. Verifica que el JSON sea válido (sin comas trailing, comillas balanceadas).
5. Abre un PR con título `i18n: add <idioma> translation`.

### Cómo probar tu traducción localmente

```bash
npm install
npm run dev
```

El idioma se selecciona automáticamente del navegador, o puedes forzarlo desde Settings → Appearance.

## 🧩 Agregar un mod loader

AnvilCraft usa un patrón de **strategy** con `LoaderRegistry`. Para agregar un loader:

1. **Backend (Rust)** — `src-tauri/src/loaders/<categoría>/<nombre>.rs`:
   ```rust
   use crate::loaders::{LoaderStrategy, LoaderCategory};
   use crate::models::InstanceEngine;

   pub struct MiLoader;
   #[async_trait]
   impl LoaderStrategy for MiLoader {
       fn engine(&self) -> InstanceEngine { InstanceEngine::MiLoader }
       fn category(&self) -> LoaderCategory { LoaderCategory::Bukkit }
       fn display_name(&self) -> &'static str { "MiLoader" }
       // implementa fetch_versions, resolve_download, install
   }
   ```

2. Registra el loader en `src-tauri/src/loaders/registry.rs`.

3. Agrega la variante a `InstanceEngine` enum en `src-tauri/src/models.rs`.

4. **Frontend (TypeScript)** — agrega el loader a `src/lib/loaders/catalog.ts` para que aparezca en la UI.

5. Si tu loader soporta modpacks/plugins, configura las capabilities en el `impl LoaderStrategy`.

## 🛠️ Setup de desarrollo

```bash
# Prerrequisitos
# - Node.js 20+
# - Rust 1.70+ (instala con rustup)
# - MS C++ Build Tools (Windows) o build-essential (Linux)
# - Git

# Clonar e instalar
git clone https://github.com/cPathz/AnvilCraft-Panel.git
cd AnvilCraft-Panel
npm install
cargo install tauri-cli --version "^2.0"

# Correr en modo dev
cargo tauri dev

# Compilar para producción
cargo tauri build
```

## 📝 Estilo de código

- **TypeScript:** sigue la config del proyecto (`tsconfig.json`).
- **Svelte 5:** usa Runes (`$state`, `$derived`, `$effect`, `$props`). Evita `let` reactivo viejo.
- **Rust:** usa `cargo fmt` y `cargo clippy` antes de commitear.
- **Commits:** mensajes en presente, ej. `feat: add Folia loader` o `fix: console crash on disconnect`.

## ✅ Antes de abrir un PR

- [ ] El código compila: `cargo check --manifest-path src-tauri/Cargo.toml` y `npm run check`
- [ ] Si agregaste strings de UI, agregaste las traducciones a `locales/en.json` (otros pueden traducir después)
- [ ] Si agregaste archivos, NO agregaste dev debris (logs, scratch/, etc.)
- [ ] El commit es específico y tiene mensaje claro

## 💬 ¿Dudas?

Abre una Discussion en GitHub o contacta al mantenedor (ver README).
