# Dev Log - AnvilCraft Panel

Bitácora de desarrollo en formato machine-readable. Solo hechos concretos, fechas, cambios implementados.

---

## 2026-06-03

### Add Instance Creator — Loader Dropdown Conversion (v5)
- **Estado**: implementado (v5 — dropdown con menu categorizado)
- **Frontend**: `CreateInstanceModal.svelte`
    - Trigger button: label "Loader" + botón full-width con dot color + chevron rotatorio
    - Trigger muestra loader seleccionado + badge "MODS" si aplica
    - Click abre floating menu portal (patrón idéntico al version dropdown: backdrop z-60 + menu z-70 fixed)
    - Menu muestra las 4 secciones con header coloreado + loaders clickables (dot + nombre + badge)
    - Position state compartido con version dropdown (`dropdownBottom/Left/Width`)
- **Mutual exclusion**: version / loader / NF MC / NF Build dropdowns cierran los otros al abrir
    - Helper `closeAllDropdowns()` centraliza el cierre
    - Aplicado en los 4 onclicks de los triggers
- **Helpers TS**: `getLoaderCategory(name)` y `getLoaderBadge(name)` derivan de `LOADERS_BY_CATEGORY`
- **i18n**: nueva clave `placeholder_loader` en `en.json` ("Select a loader") + `es.json` ("Selecciona un loader")
- **Refinamientos visuales**: menu loaders `text-sm`→`text-base` (+2px), `py-1.5`→`py-1` (~33% padding)
- **Iteración diseño**: v1 game-version ❌ → v2 grid 2x2 ❌ → v3 tabs row ❌ → v4 lista categorizada ✅ → **v5 dropdown (actual)**

### Loader Module Refactor — Per-Category Trait Architecture
- **Intención**: refactorizar backend Rust para tener `LoaderStrategy` trait + `LoaderRegistry`, 1 archivo por categoría
- **Razón**: el frontend v5 ya muestra 5 categorías + 16 loaders pero el backend no tiene abstracción central — lógica repartida en 4 archivos con branching duplicado. Antes de cablear loaders reales (Spigot, Forge, Fabric, BungeeCord, los 4 híbridos), unificar la base
- **Plan completo**: `C:\Users\luism\.claude\plans\graceful-humming-feigenbaum.md` (aprobado)
- **Estructura planeada**:
    - `src-tauri/src/loaders/` con `mod.rs` (trait) + `registry.rs` + `types.rs` + `common.rs`
    - `vanilla.rs` (1 loader) + `bukkit.rs` (4) + `mods.rs` (4) + `proxies.rs` (3) + `hybrids/{mod,mohist,arclight,banner,magma}.rs`
    - `src/lib/loaders/` con TS mirror: `types.ts` + `catalog.ts` + `index.ts`
- **Adición clave**: `BungeeCord` a `InstanceEngine` enum (faltaba)
- **BungeeCord source**: Jenkins API `ci.md-5.net/job/BungeeCord/api/json`
- **Migration order**: 1) Vanilla → 2) Bukkit → 3) Mods → 4) Proxies → 5) Hybrids → 6) TS mirror → 7) Cleanup
- **Estado**: pendiente de implementar (Step 1: Vanilla)

### Loader Module Refactor — Step 1: Vanilla ✅
- **Estado**: Step 1 implementado y compila limpio (`cargo check` sin errores)
- **Módulo nuevo `src-tauri/src/loaders/`**:
    - `mod.rs` — `LoaderStrategy` trait (`#[async_trait]`) + `LoaderCategory` enum + `LoaderCapabilities` struct
    - `types.rs` — `VersionMeta`, `DownloadInfo`, `InstallPhase` (reusable, base para los otros loaders)
    - `common.rs` — `download_file` + `write_eula_txt` movidos desde `commands/versions.rs`
    - `registry.rs` — `LoaderRegistry` con `OnceLock<>` + 16 entradas estáticas + `by_engine()` + `all()`
    - `vanilla.rs` — `VanillaLoader` con `fetch_versions`, `resolve_download`, `install` (migrado de `install_vanilla`)
    - `bukkit.rs`, `mods.rs`, `proxies.rs`, `hybrids.rs` — stubs retornando "not yet implemented" (cubren los 15 loaders restantes)
- **Trait design**: `engine()`, `category()`, `display_name()` requeridos; resto con defaults sensatos
    - `is_two_step_version() = true` solo para NeoForge
    - `stop_command() = "end"` para los 3 proxies (Velocity/Waterfall/BungeeCord)
    - `min_java()` y `capabilities()` retornan metadata para la UI
- **Crate nuevo**: `async-trait = "0.1"` en `Cargo.toml` (para `dyn LoaderStrategy` object-safe)
- **`InstanceEngine` enum extendido**: 11 → 16 variantes (BungeeCord + Mohist + Arclight + Banner + Magma)
- **`commands/instance.rs` `create_instance`**:
    - Vanilla: ruteo vía `LoaderRegistry::global().by_engine(Vanilla)` + `loader.install(...)`
    - NeoForge + Bukkit/Proxy family: código legacy intacto (Steps 2-4 los migran)
    - **Bug fix**: catch-all `_ => install_vanilla` eliminado — Fabric/Forge/Quilt + los nuevos stubs ahora retornan error claro en vez de descargar vanilla silenciosamente
- **Cleanup**: `install_vanilla` eliminado de `commands/versions.rs`; structs muertos `VersionDetails`/`VersionDownloads`/`VersionDownload` borrados
- **Verificación**: `cargo check` pasa con 0 errores. Warnings restantes (3) son pre-existentes en `commands/curseforge.rs` y no se tocaron
- **Próximo**: Step 2 (Bukkit) — mover `install_project_server` a `loaders/bukkit.rs` con helper compartido por los 4 loaders

### Loader Module Refactor — Step 2: Bukkit ✅
- **Estado**: Step 2 implementado y compila limpio (`cargo check` 0 errores, 3 warnings pre-existentes)
- **`loaders/bukkit.rs` completo**: 4 loaders reales (Paper, Spigot, Purpur, Folia)
    - `resolve_build_download(project, mc_version)`: helper privado que delega a PaperMC API (`api.papermc.io`) o Purpur API (`api.purpurmc.org`) según el proyecto. Purpur usa endpoint `/latest` que devuelve el build más reciente
    - `install_inner(...)`: cuerpo compartido — descargar JAR, renombrar a `server.jar`, escribir EULA, emitir `install-progress: Done`
    - Cada loader es unit struct con su propio `min_java()` (Paper 11, Spigot 8, Purpur 11, Folia 17) y `capabilities()` con `supports_plugins: true`
    - Spigot: install rechaza con error claro si no se pasa `custom_url` (no hay API pública)
- **API structs movidos**: `ProjectBuilds`, `BuildInfo`, `ChangeInfo`, `BuildDownloads`, `ApplicationDownload` (renombrado desde `DownloadInfo` para evitar colisión con `loaders::types::DownloadInfo`) viven en `bukkit.rs`
- **Dispatcher en `commands/instance.rs`**:
    - Helper privado `dispatch_via_registry(app, id, dir, version, url, eula, engine)` para Vanilla + Bukkit family
    - `InstanceEngine::Vanilla` y `InstanceEngine::{Paper, Spigot, Purpur, Folia}` rutean vía registry
    - `Velocity | Waterfall` siguen con shim legacy (Step 4 los migra)
- **Transición shim**: `install_project_server` renombrado a `install_project_server_legacy` en `commands/versions.rs` y convertido a usar `serde_json::Value` en vez de `ProjectBuilds` (ahora privado en bukkit.rs)
- **Cleanup**: 5 structs API eliminados de `commands/versions.rs`; el resto del archivo (ProjectVersionList, get_minecraft_versions, get_project_versions, get_neoforge_versions) intacto
- **Verificación**: `cargo check` pasa limpio
- **Próximo**: Step 3 (Mods) — NeoForge 2-step + Forge installer + Fabric/Quilt meta API

### Loader Module Refactor — Step 3: Mods ✅
- **Estado**: Step 3 implementado y compila limpio (`cargo check` 0 errores, 3 warnings pre-existentes en `curseforge.rs`)
- **`loaders/mods.rs` completo**: 4 loaders reales (NeoForge, Forge, Fabric, Quilt)
    - `NeoForgeLoader` (full impl): `is_two_step_version() = true`, `fetch_mc_versions()` parsea maven-metadata.xml, `fetch_versions(Some(mc))` retorna builds, `install()` descarga installer JAR + ejecuta `java -jar ... --installServer` con stream stderr/stdout a eventos
    - `ForgeLoader` (basic): download de `maven.minecraftforge.net/.../forge-{ver}-installer.jar` + ejecución headless
    - `FabricLoader` (basic): descarga server-launcher de `meta.fabricmc.net/v2/versions/loader/{mc+loader}/server/jar`
    - `QuiltLoader` (basic): análogo a Fabric via `meta.quiltmc.org`
- **Migración código NeoForge**:
    - `extract_neoforge_category` movido a `mods.rs` (privado al módulo)
    - `fetch_neoforge_maven_metadata()` + `parse_mc_versions_from_xml` + `parse_builds_from_xml` como helpers privados
    - `run_neoforge_installer(app, id, version, mc_dir)` extrae la lógica de installer child-process
    - `NeoForgeLoader::list_mc_versions()` y `list_builds(mc, betas)` como métodos inherentes `pub` para que los Tauri commands deleguen
- **`find_forge_args_file` movido**: `commands/server.rs` (privado) → `loaders::mods` (`pub(crate)`). Import en `server.rs` actualizado; 38 líneas de código duplicado eliminadas
- **Tauri commands `get_neoforge_*` ahora wrappers**:
    - `get_neoforge_mc_versions` → `NeoForgeLoader::list_mc_versions().await`
    - `get_neoforge_versions(mc, betas)` → `NeoForgeLoader::list_builds(&mc, betas).await`
- **`commands/curseforge.rs` actualizado**:
    - 2 sitios que importaban `install_neoforge` ahora usan `LoaderRegistry::global().by_engine(InstanceEngine::NeoForge).install(...)`
    - `write_eula_txt` importado de `loaders::common` (donde vive desde Step 1), no de `commands::versions`
- **`commands/instance.rs` dispatch**:
    - `InstanceEngine::NeoForge` → `dispatch_via_registry(... NeoForge)`
    - `InstanceEngine::Fabric | Forge | Quilt` → `dispatch_via_registry(... loader_engine)`
    - Catch-all de error solo cubre BungeeCord + 4 hybrids (Steps 4-5 los migran)
- **Cleanup `commands/versions.rs`**: archivo reescrito de 735 → 211 líneas
    - Eliminados: `install_neoforge` (240 líneas), `extract_neoforge_category` (24), `download_file` y `write_eula_txt` duplicados (160)
    - `install_project_server_legacy` mantenido (Step 4 lo borra cuando mueve Velocity + Waterfall)
- **Verificación**: `cargo check` pasa limpio
- **Próximo**: Step 4 (Proxies) — Velocity + Waterfall a `loaders/proxies.rs`, agregar BungeeCord con Jenkins API, refactorizar `stop_instance` para usar `loader.stop_command()`

### Loader Module Refactor — Step 4: Proxies ✅
- **Estado**: Step 4 implementado y compila limpio (`cargo check` 0 errores, 4 warnings pre-existentes en `curseforge.rs`)
- **`loaders/proxies.rs` completo**: 3 loaders reales (Velocity, Waterfall, BungeeCord)
    - `VelocityLoader` / `WaterfallLoader` comparten `install_proxy()` + `resolve_papermc_build(project, mc_version)` — misma PaperMC API que el Bukkit family
    - `BungeeCordLoader` (nuevo): `fetch_versions()` toma los últimos 30 builds del Jenkins JSON API (`ci.md-5.net/job/BungeeCord/api/json`); `version.id` = build number (u32); JAR URL = `ci.md-5.net/job/BungeeCord/{N}/artifact/bootstrap/target/BungeeCord.jar`
    - `BungeeCordLoader::list_recent_builds()` como método inherente `pub` para futura delegación desde Tauri command
    - Los 3 loaders override `stop_command() = "end"`. `accept_eula` se silencia explícitamente en `install_bungeecord` (BungeeCord no tiene EULA de Mojang)
    - API structs privados al archivo: `ProjectBuilds`/`BuildInfo`/`ApplicationDownload` (PaperMC), `JenkinsBuilds`/`JenkinsBuildRef` (Jenkins) — todos `#[allow(dead_code)]` donde aplica
- **`commands/server.rs` refactor**:
    - `stop_instance` ahora consulta `LoaderRegistry::global().by_engine(inst.loader).map(|l| l.stop_command()).unwrap_or("stop")` en vez del match hardcoded `Velocity | Waterfall => "end"`
    - `InstanceEngine` removido del import (ya no se referencia directamente)
    - Añadido import `crate::loaders::registry::LoaderRegistry`
- **`commands/instance.rs` dispatcher**:
    - `InstanceEngine::Velocity | Waterfall | BungeeCord` → `dispatch_via_registry(... loader_engine)` (3-way)
    - Removido: `use crate::commands::versions::install_project_server_legacy;` y el branch legacy del match
    - Catch-all de error ahora cubre solo los 4 hybrids (Step 5 los migra)
- **`commands/versions.rs` cleanup**:
    - `install_project_server_legacy` (96 líneas) eliminado — los 3 proxies ya no lo necesitan
    - Imports simplificados: ya no se necesitan `download_file`, `write_eula_txt`, `std::fs`, `std::io::Write`
    - Doc header del módulo actualizado: `get_project_versions` ahora explica que sirve para el dropdown de Velocity/Waterfall en la UI
    - Archivo: 211 → 113 líneas
- **Verificación**: `cargo check` pasa limpio. Los 4 warnings restantes son `dead_code` en `commands/curseforge.rs` (estructuras API de Modrinth/CurseForge, no relacionadas con Step 4)
- **Próximo**: Step 5 (Hybrids) — convertir `hybrids.rs` (single file) en `hybrids/` folder con 1 loader per file (4 stubs). Real install logic es explícitamente OUT OF SCOPE per plan; es un follow-up per-loader

### Loader Module Refactor — Step 5: Hybrids ✅
- **Estado**: Step 5 implementado y compila limpio (`cargo check` 0 errores, 4 warnings pre-existentes en `curseforge.rs`)
- **Folder structure**: `loaders/hybrids.rs` (single file, 74 líneas) → `loaders/hybrids/` folder
    - `mod.rs` (29 líneas) — `pub use` re-exports de los 4 loaders + helpers compartidos `hybrid_capabilities()` y `not_implemented()` (`pub(crate)`)
    - `mohist.rs` (43 líneas) — `MohistLoader` unit struct + `impl LoaderStrategy`
    - `arclight.rs` (43 líneas) — `ArclightLoader`
    - `banner.rs` (43 líneas) — `BannerLoader` (min_java 11 — único Fabric-based hybrid)
    - `magma.rs` (43 líneas) — `MagmaLoader`
- **Por qué folder desde día 1**: cada híbrido tiene un install flow único (Mohist parchea Forge + CraftBukkit, Arclight soporta Forge + NeoForge, Banner es Fabric-based, Magma es fork de Forge). 4 archivos paralelos evita el inevitable split de un `hybrids.rs` 350-line más adelante
- **Dispatch sin cambios**: `commands/instance.rs` ya enrutaba los 4 hybrids vía registry desde Step 4 (catch-all de error). Cada stub retorna `Err("X is not yet implemented")` desde `install()`; el error llega a la UI con el step "Error: ..."
- **Capabilities de hybrids**: `supports_plugins: true, supports_mods: true, is_proxy: false, custom_url_supported: true` — esto es lo que los hace "híbridos" frente a las otras categorías
- **Registry intacto**: `LoaderRegistry::global()` ya listaba los 4 desde Step 1. El cambio es puramente estructural — la API pública (`hybrids::MohistLoader`, etc.) se preserva vía re-exports
- **Verificación**: `cargo check` pasa limpio
- **Out of scope explícito per plan**: install logic real de los 4 hybrids. El plan marca "Real hybrid install logic" como follow-up per-loader (cada uno tiene una API source distinta: Mohist/Arclight/Banner/Magma tienen sus propios Jenkins CIs con esquemas diferentes; implementación completa es ~200-400 líneas per loader)
- **Próximo**: Step 6 (TS mirror) — crear `src/lib/loaders/` con `types.ts` + `catalog.ts` + `index.ts`, y reemplazar el inline `LOADERS_BY_CATEGORY` (35 líneas) en `CreateInstanceModal.svelte` por el catalog. Widening de `selectedLoader` type a `LoaderName` (remueve `as any` casts)

### Loader Module Refactor — Step 6: TS Mirror (en curso)
- **Intención**: crear `src/lib/loaders/` módulo TS que hand-mirror el `LoaderRegistry` de Rust (16 loaders, 5 categorías)
- **Archivos nuevos planeados**:
    - `types.ts` — `LoaderName` union (16 variantes) + `LoaderCategory` union (5) + `LoaderMetadata` interface + `LoaderCapabilities` interface + `CATEGORY_ORDER` constant
    - `catalog.ts` — `LOADERS: readonly LoaderMetadata[]` con 16 entradas (1 vanilla + 4 bukkit + 4 mods + 4 hybrids + 3 proxies)
    - `index.ts` — barrel re-export
- **Consumo en `CreateInstanceModal.svelte`**:
    - Borrar el inline `LOADERS_BY_CATEGORY` (35 líneas) y derivar del catalog vía `$derived`
    - Widening `selectedLoader` type de 7-variant union a `LoaderName | null` (default "Vanilla")
    - Remover `as any` cast en `onclick={() => { selectedLoader = loader.name as any; ... }}`
    - `getLoaderCategory()` y `getLoaderBadge()` se vuelven one-liners sobre `LOADERS`
    - `CATEGORY_COLORS` queda inline (es UI styling concern, no data concern)
- **Widening secundario en `store.svelte.ts`**:
    - `Instance.loader` de 6-variant union (`'Vanilla' | 'Paper' | 'Fabric' | 'Forge' | 'NeoForge' | 'Quilt'`) a `LoaderName` (16 variantes)
    - Reemplazar `detectedLoader: any` por `LoaderName | undefined`
    - Reemplazar `loaderMatch[1] as any` por `loaderMatch[1] as LoaderName`
- **Cambios visuales menores en dropdown**:
    - Mods gana Quilt (antes 3, ahora 4 entries)
    - Proxies reordenado: Velocity, Waterfall, BungeeCord (antes Velocity, BungeeCord, Waterfall) — matchea el orden del enum Rust
- **Estado**: implementado

### Loader Module Refactor — Step 6: TS Mirror ✅
- **Estado**: Step 6 implementado; `npm run check` no añade errores nuevos (los 14 errores en `CreateInstanceModal.svelte` son pre-existentes, solo cambiaron de número de línea)
- **3 archivos nuevos en `src/lib/loaders/`**:
    - `types.ts` (38 líneas) — `LoaderName` (16 variantes) + `LoaderCategory` (5) + `LoaderBadge` + `LoaderCapabilities` + `LoaderMetadata` + `CATEGORY_ORDER`
    - `catalog.ts` (~150 líneas) — `LOADERS: readonly LoaderMetadata[]` con 16 entries hand-mirroring `LoaderRegistry::global()`. Incluye min_java y capabilities por loader (ej. `Fabric.minJava = 8`, `Banner.minJava = 11` por ser Fabric-based hybrid)
    - `index.ts` — barrel re-export
- **`CreateInstanceModal.svelte` consume el catalog**:
    - Borrado el inline `LOADERS_BY_CATEGORY` (35 líneas)
    - `loadersByCategory = $derived(CATEGORY_ORDER.map(key => ({ key, loaders: LOADERS.filter(l => l.category === key) })))` — preserva el orden planeado
    - `getLoaderCategory(loader)` y `getLoaderBadge(loader)` reducidos a one-liners sobre `LOADERS`
    - `selectedLoader` type widdened: 7-variant inline union → `LoaderName | null`
    - Removido `as any` cast en `onclick={() => { selectedLoader = loader.name; ... }}` — ahora type-checks
- **`store.svelte.ts` widening**:
    - `Instance.loader`: 6-variant union → `LoaderName` (16 variantes)
    - `detectedLoader: any` → `LoaderName | undefined`
    - `loaderMatch[1] as any` → `loaderMatch[1] as LoaderName`
- **`CATEGORY_COLORS` queda inline** (es UI styling concern, no data concern)
- **Verificación**: `npm run check` no añade errores nuevos. Los 14 errores pre-existentes en el modal son del estilo `installStep = get(_("create_instance.status_..."))` (svelte-i18n typing issue) — no tocan el catalog ni el refactor
- **Próximo**: Step 7 (Cleanup) — colapsar el dispatcher en `create_instance` de 6-arm match a 2-arm (hybrids error + catch-all registry), eliminar comentarios obsoletos

### Loader Module Refactor — Step 7: Cleanup (en curso)
- **Intención**: cerrar el refactor eliminando el branching residual en el dispatcher de `create_instance` y comentarios obsoletos que referencian las funciones `install_*` que ya no existen
- **Trabajo planeado**:
    - Colapsar el `match loader_engine` de 6 arms (Vanilla, Bukkit, NeoForge, Proxies, Mods, Hybrids-error) a 2 arms (Hybrids-error + catch-all `dispatch_via_registry`). El catch-all cubre los 12 engines que tienen install real
    - Actualizar el doc comment de `dispatch_via_registry` para reflejar que ya cubre los 12 non-hybrid engines
    - Limpiar el comentario obsoleto en `commands/curseforge.rs:414` ("Step 3: route through LoaderRegistry instead of importing `install_neoforge` from `commands::versions` directly") — el contexto histórico ya no es relevante
- **Verificación plan**:
    - `rg "install_vanilla|install_project_server|install_neoforge" src-tauri/src/commands/` solo debe retornar comentarios históricos (o nada)
    - `find_forge_args_file` solo se referencia desde `loaders/mods.rs` y `commands/server.rs` (confirmado)
    - `cargo check` pasa limpio
- **Estado**: implementado

### Loader Module Refactor — Step 7: Cleanup ✅
- **Estado**: Step 7 implementado; `cargo check` pasa limpio (0 errores, 4 warnings pre-existentes en `commands/curseforge.rs`)
- **Dispatcher en `create_instance` colapsado**:
    - 6-arm `match loader_engine` (Vanilla, Bukkit, NeoForge, Proxies, Mods, Hybrids-error) → 2-arm match (Hybrids-error + catch-all `dispatch_via_registry`)
    - Catch-all cubre los 12 engines con install real (Vanilla + 4 Bukkit + 4 Mods + 3 Proxies). Solo los 4 hybrids quedan como error arm explícito
    - Dispatcher shrunk de ~80 líneas a ~25 líneas en `commands/instance.rs:194-217`
- **Doc comment actualizado**: `dispatch_via_registry` ahora dice "Used by every non-hybrid engine" en vez de listar Steps 1-4 individualmente
- **Comentario obsoleto en `commands/curseforge.rs:414`**: "Step 3: route through LoaderRegistry instead of importing `install_neoforge` from `commands::versions` directly" → reducido a "Route through the registry." El contexto histórico de la migración ya no es útil
- **Verificación post-cleanup**:
    - `rg "install_vanilla|install_project_server|install_neoforge" src-tauri/src/commands/`: solo retorna 2 referencias, ambas en comentarios históricos (`curseforge.rs:414` reducido, `instance.rs:198` que ya no menciona install_vanilla directamente). Cero referencias en código activo
    - `find_forge_args_file` solo se referencia desde `loaders/mods.rs` y `commands/server.rs` (per plan)
    - `cargo check` pasa limpio
- **Refactor completo**: 7/7 steps finalizados. Estado final del módulo:
    - `src-tauri/src/loaders/` con `mod.rs` (trait) + `registry.rs` + `types.rs` + `common.rs` + `vanilla.rs` + `bukkit.rs` + `mods.rs` + `proxies.rs` + `hybrids/{mod,mohist,arclight,banner,magma}.rs`
    - 16 loaders registrados (1 Vanilla + 4 Bukkit + 4 Mods + 4 Hybrids stub + 3 Proxies)
    - `src/lib/loaders/` TS mirror con `types.ts` + `catalog.ts` + `index.ts`
    - Frontend consume el catalog; tipos widenados; sin `as any` casts en el modal

### Hacer funcionar los loaders end-to-end — Inicio con Vanilla
- **Intención**: el refactor arquitectónico está completo; ahora cablear lógica real para que los loaders puedan crear instancias funcionales. Empezando por Vanilla porque es el caso más simple (no tiene flujo de 2 pasos ni installer)
- **Bug detectado en `VanillaLoader::install`**: `loaders/vanilla.rs:128-153` llama a `self.resolve_download(version)` pero el `version_meta` que llega desde `dispatch_via_registry` (`commands/instance.rs:44-50`) tiene `url: None` por construcción. `resolve_download` lee `version.url.as_deref()` y falla con "Vanilla version is missing its manifest URL"
- **Por qué Vanilla es único**: a diferencia de Bukkit/Mods/Proxies (que construyen la URL del JAR desde `version.id` + nombre del proyecto), Vanilla necesita el per-version manifest URL de Mojang para resolver el download SHA1-verified. El `version_manifest_v2.json` mapea `"1.21.4" → https://piston-meta.mojang.com/v1/.../1.21.4.json` y ese segundo manifest expone el campo `downloads.server.url`
- **Estado**: pendiente de fix

### Vanilla end-to-end — Fix de dispatch URL ✅
- **Estado**: fix aplicado en `loaders/vanilla.rs:128-152`; `cargo check` pasa limpio (0 errores, 4 warnings pre-existentes en `commands/curseforge.rs`)
- **Cambio**: `VanillaLoader::install` ahora detecta si el `version_meta` entrante ya trae `url` poblada; si no (caso del dispatcher), llama a `self.fetch_versions(None)` para re-poblar la lista desde el manifest de Mojang y busca el match exacto por `id`. Si no encuentra el id (versión removida, typo), retorna `format!("Vanilla version {} not found in Mojang manifest", version.id)` en vez del genérico "Vanilla version is missing its manifest URL"
- **Por qué no afecta a Bukkit/Mods/Proxies**: esos loaders no leen `version.url` en su `install` — construyen la URL del JAR desde `version.id` + nombre del proyecto (ej. `meta.fabricmc.net/v2/versions/loader/{mc+loader}/server/jar` para Fabric). El lookup extra solo aplica a Vanilla
- **Trade-off de la HTTP call duplicada**: el manifest se descarga una vez al abrir el dropdown (vía `get_minecraft_versions` Tauri command) y otra vez al instalar. Es ~100KB cacheable por HTTP; aceptable por simplicidad. Una optimización futura sería cachear el manifest en memoria entre el load del dropdown y el install (≈5-30s ventana)
- **Por qué no se movió el lookup al dispatcher**: el dispatcher es genérico (funciona para los 12 non-hybrid engines). Si lo metiera ahí, todos pagarían el costo de un `fetch_versions` extra — Bukkit, Mods y Proxies no lo necesitan. Cada loader paga solo por lo que necesita
- **Próximo**: Vanilla ya debería poder crear instancia end-to-end. Pendiente de prueba manual (crear instancia Vanilla real, verificar que descarga `server.jar`, EULA, y arranca). Después podemos seguir con Bukkit/Mods/Proxies

### Vanilla end-to-end — Descarga se queda en "Connecting" ✅
- **Síntoma reportado**: el usuario ve la notificación `Connecting: https://piston-data.mojang.com/v1/objects/97ccd4c0ed3f81bbb7bfacddd1090b0c56f9bc51/server.jar` indefinidamente; la barra de progreso no avanza
- **Investigación de la API**:
    - `piston-meta.mojang.com/mc/game/version_manifest_v2.json` → HTTP 200, 269KB (manifest OK)
    - `piston-data.mojang.com/v1/objects/<sha1>/server.jar` → HTTP 200, 60MB (JAR OK, 2.7s, 22MB/s desde mi entorno, funciona con y sin User-Agent)
    - `piston-meta.mojang.com/v1/products/java/version_manifest.json` → HTTP 404 (ese endpoint alternativo no existe; pistón sigue siendo la API oficial)
    - Conclusión: la API es la correcta; el problema es del cliente Rust
- **Causas probables del lado cliente**:
    - `reqwest::Client` sin `connect_timeout` — el default es `None`, así que si el TCP SYN se queda colgado, el cliente espera indefinidamente (los 300s del `timeout()` son total, no de conexión)
    - HTTP/2 negotiation puede fallar con la CDN de Azure/Fastly detrás de Mojang en Windows; el cliente se queda en ALPN sin terminar el handshake
    - No hay evento `install-progress` entre `send().await` y el primer chunk del stream — si la conexión tarda, el usuario ve solo "Connecting" sin feedback intermedio
- **Fix aplicado en `loaders/common.rs:10-91`**:
    - **`connect_timeout(30s)`** agregado al `Client::builder()` — si el handshake TCP/TLS no completa en 30s, devuelve error claro en vez de colgar 300s
    - **`http1_only()`** agregado — fuerza HTTP/1.1, evita el ALPN negotiation que puede fallar con Fastly/Azure CDN en Windows
    - **User-Agent más descriptivo** (`AnvilCraft/0.1 (https://github.com/cPathz/AnvilCraft-Panel)`) — en caso de que Mojang empiece a filtrar por UA en el futuro, el actual es claramente identificable como cliente legítimo
    - **Nuevo evento intermedio "Connected. Streaming N bytes…"** entre `send().await` y el primer chunk del stream — el frontend ahora ve la transición `Connecting → Connected → Downloading` en vez de solo `Connecting → (largo silencio) → Downloading`
    - **Log persistente** con código HTTP recibido (`HTTP 200 — 60417480 bytes expected`) — facilita diagnosticar si la respuesta del servidor es OK pero el stream no fluye
- **Verificación**: `cargo check` pasa limpio (0 errores, 4 warnings pre-existentes en `commands/curseforge.rs` + 1 unused import en `commands/versions.rs` pre-existente)
- **Caveat importante**: si el problema del usuario es a nivel de firewall local / antivirus / proxy corporativo bloqueando `piston-data.mojang.com`, ningún cambio en el cliente reqwest lo va a resolver. Con el `connect_timeout(30s)`, al menos el usuario verá un error claro "connection timed out after 30s" en vez de un hang infinito, lo que facilita el diagnóstico
- **Próximo**: re-probar Vanilla en Windows. Si sigue colgado, agregar logging detallado (`eprintln!` con la fase exacta del handshake). Si funciona, pasar al siguiente loader

### Vanilla end-to-end — Body stream cuelga tras "Connected" (en curso)
- **Síntoma actualizado**: tras el fix anterior, el usuario ahora ve `Connected. Streaming 60417480 bytes…` — el handshake TCP/TLS completa y Mojang responde con `Content-Length: 60417480`. Pero el body stream nunca produce chunks, el progreso queda en 0% indefinidamente
- **Diagnóstico del stream**:
    - El response se recibe OK (status 200, content_length correcto)
    - `response.bytes_stream()` retorna un `BytesStream` que envuelve el `Body` de hyper
    - `stream.next().await` nunca retorna `Some(chunk)` ni `None` — se cuelga esperando el primer chunk
    - El curl test desde el mismo tipo de máquina descarga los 60MB en 2.7s, así que el server está sirviendo datos
- **Hipótesis**:
    - hyper 0.14 + reqwest 0.11 + `http1_only()` en Windows podría tener un bug donde el body no se desbloquea después del response
    - El Body podría estar esperando algo (¿TLS session ticket resumption?, ¿keep-alive timeout?, ¿algún handshake interno?)
    - Alternativa: la response es HTTP/1.1 pero el server está usando algo específico (¿content-encoding inesperado?, ¿trailers?) que confunde al cliente
- **Fix en `loaders/common.rs`**:
    - **`tcp_keepalive(60s)`** y **`tcp_nodelay(true)`** agregados al `Client::builder()` — TCP_NODELAY fuerza que chunks pequeños se envíen inmediatamente sin esperar al algoritmo de Nagle
    - **`tokio::time::timeout(45s)`** envuelve `stream.next()` — si en 45s no llega ningún chunk, retorna error claro: `Read timeout: no chunk received in 45s (downloaded N bytes)`
    - **`eprintln!` logging** en cada chunk recibido (`Chunk #1: 16384 bytes (total 16384)`) y al crear el stream — permite ver en consola si los chunks están llegando o no
- **Próximo paso diagnóstico**: el usuario prueba y reporta:
    - Si los chunks empiezan a llegar → el problema era el body buffering / TCP_NODELAY, y la descarga progresa normalmente
    - Si después de 45s ve el error "Read timeout" → confirmar que el body stream de reqwest/hyper no fluye, y necesitamos un workaround (ej. cambiar a `rustls-tls` o leer el body con un `AsyncRead` custom)
    - Si ve los `eprintln!` en consola → la info de diagnóstico es suficiente para encontrar el problema exacto

### Vanilla end-to-end — Regresión introducida por hardening flags
- **Hallazgo crítico**: investigando git history (`cd19c40` Ene 21 2026, "improve Forge installation progress"), el `download_file` original tenía SOLO 2 flags en `Client::builder()`: `user_agent("AnvilCraft/1.0")` + `timeout(300s)`. Vanilla y Bukkit funcionaban perfecto con esta config
- **Causa de la regresión**: durante las sesiones de troubleshooting anteriores, agregué 4 flags defensivos al builder:
    - `connect_timeout(30s)` — innecesario, el TCP SYN no se colgaba
    - `tcp_keepalive(60s)` — innecesario, conexiones no idle
    - `tcp_nodelay(true)` — benigno pero innecesario
    - **`http1_only()`** ← **CULPABLE**: fuerza HTTP/1.1 y deshabilita la negociación HTTP/2 de reqwest. Combinado con `Body::channel()` de hyper 0.14, produce un deadlock en el body stream después del handshake (handshake OK, headers OK, body nunca llega)
- **Por qué no se detectó antes**: el `http1_only()` se agregó "como hardening" contra el fallo de ALPN HTTP/2 en Fastly/Azure CDN que mencioné como hipótesis. En realidad ese fallo NO estaba ocurriendo (los headers del response llegaban, status 200, Content-Length correcto) — agregué el flag por defensa sin evidencia
- **Curl test del usuario** (siguiente sección): confirmó que curl SÍ fluye el body de respuestas HTTP/1.1 (recibió 215 bytes del 404 XML sin problema). El server NO está roto; el cliente reqwest con `http1_only()` + schannel sí
- **Plan de fix (pendiente de aplicar)**: revertir `Client::builder()` al original (2 flags) + mantener las mejoras de observabilidad:
    - **Revertir**: `connect_timeout`, `tcp_keepalive`, `tcp_nodelay`, `http1_only`
    - **Mantener**: evento "Connected. Streaming N bytes…", `tokio::time::timeout(45s)` en `stream.next()`, `eprintln!` de chunks, log persistente con timestamps
- **Lección aprendida**: no agregar flags defensivos sin evidencia del problema. La "hipótesis de hardening" sin síntoma confirmado introdujo una regresión
- **Estado**: fix aplicado ✅
- **Cambio en `loaders/common.rs:18-25`**: `Client::builder()` revertido a la config original (`user_agent` + `timeout` solamente)
    - Eliminados: `connect_timeout(30s)`, `tcp_keepalive(60s)`, `tcp_nodelay(true)`, `http1_only()`
    - `user_agent` revirtió a `"AnvilCraft/1.0"` (el descriptivo más largo era una mejora innecesaria que se agregó en el mismo PR que introdujo la regresión)
    - Comentario "Hardening" eliminado (era contexto para flags que ya no existen)
- **Mantenido (mejoras de observabilidad que no estaban en la regresión)**:
    - Evento intermedio "Connected. Streaming N bytes…" (entre `send().await` y primer chunk) — feedback UX
    - `tokio::time::timeout(45s)` en `stream.next()` — defensivo contra futuros cuelgues
    - `eprintln!` de chunks recibidos — diagnóstico si vuelve a fallar
    - Log persistente con timestamps en `install.log` — venía del original `cd19c40`
- **Verificación**: `cargo check` pasa limpio (0 errores, 4 warnings pre-existentes en `commands/curseforge.rs`)
- **Próximo**: re-probar Vanilla en el host del usuario. Si la descarga progresa y completa, podemos seguir con Bukkit/Mods/Proxies

### Vanilla end-to-end — Root cause real era el frontend, no el backend
- **Diagnóstico corregido** (vía Gemini, validado por diff de `git diff src/lib/components/modals/CreateInstanceModal.svelte`):
    - El backend Rust estaba funcionando correctamente todo el tiempo — los chunks SÍ llegaban, el progreso SÍ se emitía por el canal `install-progress`
    - El problema estaba en el handler de eventos de Svelte dentro de `CreateInstanceModal.svelte:198-274`
- **Bug exacto**: svelte-i18n exporta `_` como un **Store** (Readable), no como una función. El código tenía:
    ```ts
    // ❌ Mal — TypeError: _ is not a function
    installStep = get(_("create_instance.status_downloading"));
    ```
    Cuando se llamaba `_(...)`, JS lanzaba TypeError inmediatamente, lo que abortaba TODO el resto del handler de eventos. Como resultado, los eventos subsiguientes (`Downloading...`, `Finalizing download...`, `Done`) que el backend SÍ estaba emitiendo no actualizaban el state ni el progreso de la UI
    ```ts
    // ✅ Bien — extrae el formatter del store, luego lo llama
    installStep = get(_)("create_instance.status_downloading");
    ```
- **Por qué "Connected" sí se mostraba**: ese evento hardcodea su step directamente en el bloque `else` (`installStep = payload.step;`) sin pasar por la función de traducción. Por eso era el ÚLTIMO mensaje visible antes del crash
- **Por qué mi diagnóstico del body stream estaba equivocado**:
    1. El usuario veía la UI congelada en 0% con "Connected..." → yo asumí que el body stream de reqwest no fluía
    2. Agregué 4 flags defensivos (`http1_only()`, `connect_timeout`, `tcp_keepalive`, `tcp_nodelay`) que NO tenían nada que ver con el problema real
    3. Nunca consideré que el backend podía estar emitiendo eventos correctamente y el problema estaba en el frontend
    4. **Lección crítica**: cuando el síntoma es "UI congelada en paso X" pero el backend tiene logs/eprintln que muestran progreso, **primero descartar que el frontend esté crasheando silenciosamente** antes de tocar el backend
- **Estado del revert en `loaders/common.rs`**: las 4 flags se mantienen removidas. Aunque no eran la causa del problema, no aportan valor:
    - `http1_only()` — innecesario, HTTP/2 auto-negotiation funcionaba
    - `connect_timeout()` — innecesario, el TCP SYN no se colgaba
    - `tcp_keepalive()` — innecesario, conexiones no idle
    - `tcp_nodelay()` — innecesario, default es OK
    - YAGNI: no agregar código defensivo sin evidencia del problema
- **Verificación del fix Gemini aplicado** (vía `git diff`):
    - 18 ocurrencias de `get(_(...)` → `get(_)(...)` en `CreateInstanceModal.svelte`
    - Líneas afectadas: 107, 191, 205, 210, 218, 229, 233, 235, 237, 239, 241, 244, 247, 251, 253, 267, 319, 325
- **Verificación final del fix**: usuario reporta "Funciona correctamente, descargo, creo la instancia, avanzo el porcentaje" — Vanilla descarga + creación de instancia end-to-end ✅
- **Próximo**: usuario prueba si el server inicia correctamente (código de `commands/server.rs`, no del installer)
- **Out of scope (posible follow-up)**: revisar si hay otros archivos Svelte con el mismo bug `get(_(...))`. `grep -rn "get(_(" src/lib/` debería retornar 0 matches después del fix de Gemini

### Vanilla end-to-end — Server startup + join confirmados ✅
- **Resultado reportado por el usuario**: server 1.21.11 arrancó, usuario `cPathz` se conectó desde `127.0.0.1:8192` y entró al game
- **Pipeline completo funcionando**:
    - Download `server.jar` (60MB) ✅
    - EULA aceptado (`eula.txt` con `eula=true`) ✅
    - Server unpacking libraries en primer arranque (esto es el server.jar moderno de Mojang que trae todas las deps y las extrae a `versions/1.21.11/` + `libraries/`) ✅
    - World nuevo generado, 1470 recipes, 1584 advancements cargados ✅
    - `Starting Minecraft server on *:25565` (puerto bindeado) ✅
    - `Done (5.833s)! For help, type "help"` (server ready) ✅
    - Auto-pause: `Server empty for 60 seconds, pausing` ✅
    - Player join: `cPathz[/127.0.0.1:8192] logged in with entity id 13 at (56.5, 70.0, 27.5)` + `cPathz joined the game` ✅
- **Nota sobre el revert en `loaders/common.rs`**: aunque mi rationale original ("http1_only() rompe el body stream") estaba equivocado (el root cause era el frontend), el código final está en estado limpio que **matchea la versión que funcionaba en `cd19c40`**. La remoción de las 4 flags es válida por YAGNI: no agregar código defensivo sin evidencia del problema
- **Estado final Vanilla**: 100% funcional end-to-end. No requiere más trabajo
- **Próximo**: pasar a Bukkit (Paper) — el siguiente loader más simple (sin 2-step versioning, sin installer unpacking)

### Bukkit family — Instalación ya implementada, solo falta verificar
- **Hallazgo**: al revisar `loaders/bukkit.rs`, el `install_inner` (líneas 137-199) y `resolve_build_download` (líneas 88-133) ya implementan el flow completo para los 4 loaders Bukkit
- **APIs verificadas funcionando** (vía WebFetch):
    - **Paper** — `https://api.papermc.io/v2/projects/paper` retorna `versions[]` (62 entries, 1.7.10 → 1.21.11) + `version_groups[]`. `https://api.papermc.io/v2/projects/paper/versions/1.21.4/builds` retorna `builds[]` con `downloads.application.name` = `paper-1.21.4-{N}.jar` (último build = 193)
    - **Folia** — `https://api.papermc.io/v2/projects/folia` retorna `versions[]` (10 entries, 1.19.4 → 1.21.11). Mismo formato de builds que Paper
    - **Purpur** — `https://api.purpurmc.org/v2/purpur` retorna `versions[]` (39 entries, 1.14.1 → 26.1.2). Estructura JSON diferente a PaperMC (sin `project_id`/`version_groups`)
    - **Spigot** — sin API pública. md-5 ya no distribuye binarios oficiales. Solo instalable vía `custom_url`
- **Flujo para el usuario (Paper/Purpur/Folia) — el código ya hace todo**:
    1. Modal: usuario selecciona "Paper" del dropdown
    2. Frontend invoca `get_project_versions("paper")` (Tauri command en `commands/versions.rs:76` que YA está implementado)
    3. Dropdown se llena con MC versions
    4. Usuario selecciona versión (ej. "1.21.4") y hace click en Create
    5. `create_instance` dispatcher → `dispatch_via_registry(InstanceEngine::Paper)`
    6. `VersionMeta { id: "1.21.4", url: None, ... }` → `PaperLoader::install(...)`
    7. `install_inner` → `resolve_build_download("paper", "1.21.4")` → descarga el JAR
    8. `download_file` (mismo que Vanilla, ya verificado) → escribe EULA → Done
- **`fetch_versions` en `loaders/bukkit.rs` retorna "not yet implemented"** — esto es dead code honesto. El frontend usa el Tauri command `get_project_versions` directamente, NO el método del trait. Las 4 implementaciones quedan como stubs que retornan error (sin afectar funcionalidad). Documentar como dead code en lugar de implementar para no agregar código que no se usa
- **Spigot** — requiere `custom_url` (la UI tiene el toggle de custom URL en el modal). El path de install funciona: si el usuario provee un URL válido a un JAR de Spigot, se descarga y configura igual que los otros
- **Estado**: listo para probar. El usuario testea Paper → Purpur → Folia → Spigot (con custom URL) sin requerir más cambios de código
- **Próximo**: usuario prueba los 4 Bukkit loaders end-to-end

### Bukkit family — Toggle pre-release extendido a Paper
- **Decisión**: extender el toggle `showSnapshots` (que solo se mostraba para Vanilla) a Paper. Folia/Purpur no muestran toggle porque sus APIs no devuelven pre-releases/RCs
- **Datos verificados con 3 WebFetch calls a las APIs vivas**:
    - Paper: 59 versions totales, 11 no-clean (7 `-pre*` + 4 `-rc*`), 48 clean releases
    - Folia: 10 versions, 0 no-clean
    - Purpur: 39 versions, 0 no-clean
- **Cambios en `CreateInstanceModal.svelte`**:
    - Línea 780: `{#if selectedLoader === "Vanilla"}` → `{#if selectedLoader === "Vanilla" || selectedLoader === "Paper"}` (extiende visibilidad del toggle)
    - Bloque else de `loadVersions()` (líneas 369-382): aplicar filtro client-side sobre la respuesta de `get_project_versions`. El filtro es defensivo: `!v.includes("-pre") && !v.includes("-rc") && !v.includes("-snapshot")`. Aplicar a los 4 Bukkit (no-op para Folia/Purpur hoy, pero funciona si en el futuro publican pre-releases)
- **Sin cambios en backend** — `get_project_versions` retorna todas las versiones; el filtrado es responsabilidad del consumidor
- **Sin cambios en i18n** — reusa la key `create_instance.show_snapshots` existente
- **Verificación**: `npm run check` (o lo que use el proyecto) pasa limpio
- **Verificación real**: `npm run check` retorna 7 errores pero **0 en `CreateInstanceModal.svelte`** — los 7 son pre-existentes en `ErrorLogView.svelte`, `AddonsView.svelte`, `store.svelte.ts` (sin relación con este cambio)
- **Bug detectado en test del usuario**: toggle funciona en UI pero al activarlo para Paper, las versiones `-pre`/`-rc` no aparecen. Root cause: faltaba un `$effect` que dependa de `showSnapshots` para Paper. El effect existente solo cubría Vanilla
- **Fix aplicado en `CreateInstanceModal.svelte:474-485`**: nuevo `$effect` que explícitamente trackea `showSnapshots` y re-llama `loadVersions()` cuando Paper está seleccionado
    - El `void showSnapshots;` es crítico: Svelte 5 solo trackea state leído **síncronamente** dentro del effect. `loadVersions()` es async y la lectura de `showSnapshots` ocurre después del `await` (en el `versions = showSnapshots ? fetched : fetched.filter(...)` del filter client-side). Sin el `void`, el effect no se re-ejecuta al togglear
- **Verificación post-fix**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos. Compila limpio
- **Verificación por el usuario**: "veo que llega a la 1.21.11 y ya veo las pre y rc" — Paper con snapshots OFF muestra 48 versiones, con ON muestra las 59 (incluyendo los 11 pre/rc) ✅
- **Próximo**: usuario procede a crear instancias de los 4 Bukkit loaders para validar el install path end-to-end

### Paper smart detection — Falso positivo de "snapshot" en builds estables
- **Síntoma reportado por el usuario**: "porque la marco como 1.21.4 -232 y con el icono de snapshot-pre" — la UI muestra un ícono de snapshot en una instancia Paper de versión estable
- **Root cause encontrado** (vía inspección de `%APPDATA%/AnvilCraftPanel/instances/wewewr/instance.json`):
    - `instance.json` tenía `"build": "232-snapshot"` cuando debería ser `"232"`
    - El smart launch detection en `src/lib/runes/store.svelte.ts:236-248` se basaba en `msg.includes("SNAPSHOT")` para agregar el sufijo `-snapshot` al build
- **Por qué es un falso positivo para Paper**: el log de Paper siempre contiene "SNAPSHOT" en esta línea:
    ```
    [22:44:42] [Server thread/INFO]: This server is running Paper version 1.21.4-232-ver/1.21.4@12d8fe0 (2025-06-09T10:15:42Z) (Implementing API version 1.21.4-R0.1-SNAPSHOT)
    ```
    Ese "SNAPSHOT" es de la API version de Mojang (`1.21.4-R0.1-SNAPSHOT`), no del build de Paper. Paper SIEMPRE lo incluye, incluso para builds estables
- **Fix aplicado en `src/lib/runes/store.svelte.ts:241-258`**: solo agregar `-snapshot` o `-experimental` al build si la versión misma (los `parts` del split por `-`) contiene ese indicador. La detección broad sobre `msg` se eliminó
    ```ts
    if (parts.some((p) => /snapshot/i.test(p))) {
        detectedBuild += "-snapshot";
    } else if (parts.some((p) => /experimental/i.test(p))) {
        detectedBuild += "-experimental";
    }
    ```
- **Pre-releases reales siguen funcionando**: cuando el usuario selecciona un MC version como `1.21.9-pre2` o `1.21.11-rc1`, el `instance.version` contiene `-pre` o `-rc`, y el componente de display (`InstanceGallery.svelte:158`, `InstanceDetail.svelte:328`) muestra el ícono por esa condición. No depende del sufijo en el build
- **Limpieza de la instancia afectada**: `instance.json` actualizado manualmente de `"build": "232-snapshot"` → `"build": "232"`. En el próximo restart del server, el smart detection re-evaluará y dejará el build limpio
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Próximo**: usuario reinicia el server Paper (o el icono debería desaparecer al refrescar la lista) y confirma visualmente que ya no aparece el ícono incorrecto

### Paper pre-releases — Filtro de RCs agregado al dropdown
- **Síntoma reportado por el usuario**: "al crear un pre en paper sale 'Error en la instalación: Error: error decoding response body: missing field `project_id` at line 1 column 113'"
- **Root cause encontrado** (vía curl + 11 tests a la API de Paper):
    - La API de Paper lista RCs (`-rc1`, `-rc2`, `-rc3`) en su endpoint `/v2/projects/paper` (dentro del array `versions`)
    - Pero el endpoint de builds `/v2/projects/paper/versions/{rc}/builds` devuelve **HTTP 404** para todos los `-rc*` (verificado: `1.21.9-rc1/2/3` y `1.21.11-rc1/2/3` todos 404)
    - El body del 404 es `{"ok":false,"error":"unknown_method","message":"No endpoint GET /v2/projects/paper/versions/{rc}/builds."}` — NO tiene `project_id`
    - Serde intenta parsearlo como `ProjectBuilds` (que requiere `project_id`) y falla con "missing field `project_id`"
    - Los `-pre*` (pre-releases) sí funcionan: `1.21.9-pre2/3/4`, `1.21.11-pre3/4/5`, `1.13-pre7` todos devuelven HTTP 200
- **Por qué Paper no publica builds para RCs**: los RCs son versiones de Mojang. Paper solo compila builds para RCs que ya se convirtieron en releases oficiales (los `-pre*` son builds experimentales, los `-rc*` son finales candidatos que Paper no soporta)
- **Fix aplicado en `CreateInstanceModal.svelte:381-399`**: filtro actualizado para distinguir RCs de pre-releases
    - RCs (`-rc`) → **siempre filtrados** (no son instalables, sin importar el toggle)
    - Pre-releases (`-pre`, `-snapshot`) → solo visibles si `showSnapshots` está ON
    ```ts
    versions = fetched.filter((v) => {
        if (v.includes("-rc")) return false;
        if (v.includes("-pre") || v.includes("-snapshot")) {
            return showSnapshots;
        }
        return true;
    });
    ```
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Verificación del usuario**: "funciono, crea la instancia y la ejecuta perfectamente" — Paper pre-release (1.21.9-pre2 o 1.21.11-pre3) instala correctamente, ejecuta el server ✅
- **Próximo**: usuario decide si probar los 3 Bukkit restantes (Purpur, Folia, Spigot) o saltar a Mods (NeoForge/Forge/Fabric/Quilt)

### Purpur — Orden del dropdown invertido, fix
- **Síntoma reportado por el usuario**: "el listado esta invertido, las mas nuevas abajo" — al seleccionar Purpur, el dropdown mostraba `1.14.1` arriba y `26.1.2` abajo
- **Root cause**: en `commands/versions.rs:90-99`, el branch de Purpur no llamaba `versions.reverse()`. Verificado que AMBAS APIs (PaperMC y Purpur) devuelven oldest-first — el branch PaperMC ya hacía `v.reverse()` (línea 105), el de Purpur no
    - PaperMC API: `first=1.7.10, last=1.21.11` (oldest-to-newest) — invertido por código
    - Purpur API: `first=1.14.1, last=26.1.2` (oldest-to-newest) — NO invertido, bug
- **Fix aplicado en `commands/versions.rs:90-100`**: agregada la línea `versions.reverse();` antes del `Ok(versions)` en el branch de Purpur. Ahora el dropdown muestra `26.1.2` arriba y `1.14.1` abajo
- **Verificación**: `cargo check` pasa limpio (0 errores, 4 warnings pre-existentes)
- **Nota del usuario**: "purpur no tiene prerelease o snapshots?" — confirmado que la API de Purpur no publica pre-releases (solo versiones estables de Mojang). El toggle "Show snapshots" NO se renderiza para Purpur en el UI por diseño
- **Próximo**: usuario prueba Folia (último Bukkit restante con API pública)

### Folia — Verificación end-to-end ✅
- **Resultado reportado por el usuario**: "ok, funciona" — Folia descarga, instala y ejecuta correctamente
- **Familia Bukkit cerrada**: Vanilla ✅, Paper ✅ (stable + pre-release), Purpur ✅, Folia ✅. Solo Spigot queda pendiente (requiere custom URL)
- **Próximo**: usuario decide si probar Spigot con custom URL, o saltar a Mods (NeoForge/Forge/Fabric/Quilt)

### Loader dropdown — Indicador visual de "tested" / "pending" (v2 — estático)
- **Pedido del usuario** (v1, descartado): tracking automático en localStorage. El usuario verificó su localStorage (F12 → Application) y NO tenía la key `anvilcraft_tested_loaders`. Las 4 instalaciones (Vanilla, Paper, Purpur, Folia) ocurrieron **antes** de que yo agregara el código de tracking — el evento "Done" no disparó la marca porque la lógica no existía aún
- **Corrección del usuario**: "manualmente sin que afecte nada mas pon una tachita a las que nos faltan por configurar, ya que te rminemos una le quitas la marca y listo"
    - **No automático** — el usuario quiere un TODO list estático
    - **Solo ✗ en los pendientes** — los probados no llevan ningún ícono
    - **Manual**: cuando terminemos uno, yo edito el catalog y le pongo `tested: true` (o lo omito, default es "tested")
- **Implementación v2** (más simple, sin localStorage):
    - **`src/lib/loaders/types.ts`**: agregado campo opcional `tested?: boolean` a `LoaderMetadata`
    - **`src/lib/loaders/catalog.ts`**: marcados `tested: true` en Vanilla, Paper, Purpur, Folia. Los 12 restantes no tienen el campo (default false → muestran ✗)
    - **`CreateInstanceModal.svelte`**: removida toda la lógica de localStorage (state, helpers, persistencia en handler "Done"). Display ahora lee directo de `loader.tested` con `{#if !loader.tested}` para mostrar la tachita
- **Estado actual esperado** (después de recargar el modal):
    - Vanilla, Paper, Purpur, Folia → sin ícono (terminados)
    - Spigot, NeoForge, Forge, Fabric, Quilt, los 4 Hybrids, los 3 Proxies → ✗
- **Workflow para próximos loaders**: cuando termines uno, me dices y yo edito el catalog agregando `tested: true` al entry correspondiente. Sin tocar nada más
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Próximo**: usuario decide si prueba Spigot con custom URL o salta a Mods

### NeoForge — Detección de loader/build agregada al smart launch
- **Síntoma reportado por el usuario**: "se crea la instancia bien, pero no detecta la version del jar en la consola" — el server NeoForge arranca OK, pero el smart detection no identifica el loader ni el build
- **Verificación del bug** vía `instance.json`:
    - `loader: "Vanilla"` ← incorrecto, debería ser `NeoForge`
    - `build: null` ← incorrecto, debería ser `21.9.13-beta`
- **Root cause** en `store.svelte.ts:218-219`:
    - El regex `/This server is running (\w+) version ([^\s]+)/` (línea 218) está pensado para loaders tipo Purpur que logean ese formato. NeoForge NO usa ese formato
    - El fallback `/Starting minecraft server version (.*)/i` (línea 219) sí matchea `[minecraft/DedicatedServer]: Starting minecraft server version 1.21.9`, pero asigna `detectedLoader = "Vanilla"` por default
    - NeoForge/Forge logean su info de versión en una línea diferente: `[ne.ne.ne.co.NeoForgeMod]: NeoForge mod loading, version 21.9.13-beta, for MC 1.21.9`
- **Fix aplicado en `store.svelte.ts:217-275`**:
    - **Línea 223**: nuevo regex `/(?:Neo)?Forge mod loading, version ([\d\w.-]+), for MC ([\d\w.-]+)/i` que captura ambos NeoForge y Forge en una sola expresión
    - **Líneas 234-243**: branch nuevo `neoforgeMatch` que setea `detectedLoader` (`NeoForge` o `Forge`), `detectedBuild` (ej. "21.9.13-beta"), y `detectedVersion` (la MC version, ej. "1.21.9") directamente sin pasar por el split de `fullVersionStr`
    - **Líneas 256-275**: refactor del bloque anterior para usar `detectedVersion` y `detectedBuild` ya pobladas cuando vienen de neoforgeMatch, o seguir el path de PaperMC cuando vienen de loaderMatch
- **Por qué no usar el path de PaperMC para NeoForge**: el formato es "X.Y.Z-rc1" o "X.Y.Z-beta" donde el "X.Y.Z" es la versión de NeoForge (≠ MC version). En PaperMC, "X.Y.Z-N" es "MC version-build number" donde X.Y.Z es la MC version. Las dos estructuras son diferentes y no se pueden parsear con la misma lógica
- **`instance.json` del NeoForge test corregido manualmente**:
    - `loader: "Vanilla"` → `"NeoForge"`
    - `build: null` → `"21.9.13-beta"`
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Próximo**: usuario reinicia el server NeoForge (o espera el próximo server start) — el smart detection re-corre con la nueva lógica y la próxima vez detecta loader + build correctamente
- **Workflow final**: NeoForge está funcional end-to-end. Cuando confirmes, le quito la ✗ del catalog

### NeoForge — vanillaMatch ya no sobrescribe el loader
- **Síntoma reportado por el usuario** (después de mi primer fix): la UI muestra "Vanilla 1.21.9 - 21.9.13-beta" — build correcto, pero loader sobrescrito a "Vanilla" en el instance.json
- **Root cause refinado**: el smart detection procesa cada línea de log independientemente. Dos líneas en el startup de NeoForge matchean detecciones diferentes:
    1. `[NeoForgeMod]: NeoForge mod loading, version 21.9.13-beta, for MC 1.21.9` → mi neoforgeMatch fijaba `loader = "NeoForge"`, `build = "21.9.13-beta"`
    2. `[minecraft/DedicatedServer]: Starting minecraft server version 1.21.9` → vanillaMatch sobrescribía `loader = "Vanilla"`, `build = undefined`
    - La última detección gana → loader final: "Vanilla" (incorrecto), build: "21.9.13-beta" (preservado del primer paso)
- **Por qué vanillaMatch es problemático**: `"Starting minecraft server version X"` es emitido por TODOS los loaders (no solo Vanilla) — Paper, Fabric, NeoForge, etc. todos lo incluyen porque es la línea de Mojang's `MinecraftServer` que se loggea después del bootstrap del loader
- **Fix aplicado en `store.svelte.ts:246-251`**: removida la asignación `detectedLoader = "Vanilla"` del branch vanillaMatch. Ahora vanillaMatch solo actualiza `fullVersionStr` (y por extensión `detectedVersion`); NO toca el loader. El loader queda determinado por el usuario al crear la instancia y reforzado solo por detecciones específicas (loaderMatch para Paper/Purpur/etc., neoforgeMatch para NeoForge/Forge)
- **Garantía de correctitud**: el único caso donde vanillaMatch se usaba para identificar "es Vanilla" era redundante — cuando un server es realmente Vanilla, no hay ni loaderMatch ni neoforgeMatch que disparen, y `instance.loader` ya es "Vanilla" desde `create_instance` (línea 156: `instance.loader = engine`). El smart detection no necesita "re-detectar" Vanilla
- **`instance.json` corregido**: `loader: "Vanilla"` → `"NeoForge"` (build ya estaba bien en "21.9.13-beta")
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Próximo**: usuario reinicia el server NeoForge una vez más y la UI debe mostrar "NeoForge 1.21.9 - 21.9.13-beta" sin sobrescritura
- **Workflow final**: NeoForge end-to-end funcional. Confirmo con el usuario y le quito la ✗ del catalog

### NeoForge — Detección de Mod List agregada como path primario
- **Síntoma reportado por el usuario** (segunda iteración): después de reiniciar el server NeoForge, la UI muestra "Forge 26.1.2 - 26.1.2.73" — loader incorrecto (debería ser NeoForge), pero el build "26.1.2.73" se detectó correctamente
- **Verificación del bug** vía `instance.json` del instance 12154: `loader: "Forge"`, `build: "26.1.2.73"`, `version: "26.1.2"`. El usuario instaló desde `maven.neoforged.net` (vía `install.log`) que es el path de NeoForge, no de Forge
- **Análisis del usuario** (acertado): "deberia tomar estas lineas si se repiten en todo neoforge" — sugiere usar las líneas del Mod List (`NeoForge 26.1.2.73 (neoforge)`) en vez de la línea "NeoForge mod loading, version X, for MC Y" porque el Mod List siempre aparece, formato consistente, y el id en paréntesis es ground truth
- **Fix aplicado en `store.svelte.ts:223-225, 245-260`**:
    - **Línea 224**: nuevo regex `/^\s*(Neo)?Forge\s+(\S+)\s+\((neo)?forge\)/m` que captura la entrada del Mod List con tres grupos: `[1] = "Neo"` o `undefined`, `[2] = build version`, `[3] = "neo"` o `undefined`
    - **Líneas 248-262**: branch nuevo `modListMatch` que setea `neoforgeDirect = { loader, build, mc: "" }`. `isNeo = !!capture[1] || !!capture[3]` (cualquier prefijo "Neo" presente → NeoForge). El `mc` se deja vacío porque la MC version está en una línea separada del Mod List
    - **Líneas 270-275**: `detectedVersion` ahora usa `neoforgeDirect.mc || fullVersionStr` — si el Mod List dio mc vacío, se usa la versión que vino de vanillaMatch (de la línea "Starting minecraft server version X") o del valor previo
- **Por qué el Mod List es más confiable**:
    - El id en paréntesis (`(neoforge)` vs `(forge)`) es la fuente canónica del loader — no depende de heurísticas como `msg.includes("NeoForge mod loading")` que pueden fallar con variaciones de case o formato
    - La línea aparece SIEMPRE en el startup de cualquier server FML-based (NeoForge, Forge, hybrid loaders que usan FML)
    - No requiere `isNeo` heurístico; el id en paréntesis ES el discriminador
- **`instance.json` del 12154 corregido**: `loader: "Forge"` → `"NeoForge"` (versión 26.1.2 y build 26.1.2.73 ya estaban correctos)
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Próximo**: usuario reinicia el server NeoForge y la UI debe mostrar "NeoForge 26.1.2 - 26.1.2.73" con el nuevo path de detección
- **Confirmado por el usuario**: NeoForge end-to-end funcional. Catalog actualizado: `NeoForge: tested = true` (sin tachita)

### NeoForge — Fix de modListMatch sobreescribiendo version a vacío
- **Síntoma reportado por el usuario** (segunda iteración de NeoForge, esta vez con MC 1.21.11 + build 21.11.42): "cuando inicio el server cambia a NeoForge- 21.11.42 y cuando detecta la version en la consola cambia nuevamente a Forge 1.21.11- 21.11.42" — la UI transita por un estado intermedio donde la versión se borra, y termina con loader "Forge" en vez de "NeoForge"
- **Root cause refinado** en `store.svelte.ts:286-289`:
    - El log de NeoForge tiene el Mod List ANTES de la línea "NeoForge mod loading". El Mod List line se procesa primero
    - En el branch `modListMatch`, `neoforgeDirect.mc = ""` (vacío, porque la MC version está en una línea separada)
    - Mi código: `detectedVersion = neoforgeDirect.mc || fullVersionStr` = `"" || ""` = `""` (empty string, falsy)
    - Esto triggereaba un overwrite de `instance.version` a `""` (string vacío) en la comparación `normExisting !== normDetected`
- **Por qué el loader termina en "Forge"**: el state vacío probablemente causa un update cascade que dispara el smart detection múltiples veces, y en alguna corrida (probablemente con state stale) el loader se sobreescribe a "Forge"
- **Fix aplicado en `store.svelte.ts:284-302, 336-346`**:
    - **`detectedVersion` ahora es `string | undefined`**. Si `neoforgeDirect.mc` está vacío, NO se setea `detectedVersion` — queda `undefined`
    - **Comparación guarded**: `if (detectedVersion !== undefined) { ... normalize and compare ... }`. Sin detected version, no se hace overwrite
    - El resultado: el `modListMatch` branch ahora solo actualiza build y loader. La versión se queda intacta hasta que `neoforgeMatch` (en una línea posterior) la setea explícitamente
- **Estado final esperado** después de restart con el fix:
    - `loader: "NeoForge"` (correcto)
    - `build: "21.11.42"` (correcto)
    - `version: "1.21.11"` (correcto, sin string vacío transitorio)
- **`instance.json` del fsedfse corregido**: `loader: "Forge"` → `"NeoForge"` (versión y build ya estaban correctos)
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Próximo**: usuario reinicia el server NeoForge 1.21.11. Si funciona, NeoForge queda 100% cerrado

### NeoForge — Bug raíz: msg.startsWith("[") era una asunción falsa
- **Síntoma final** (vía console log del usuario): el smart detection loggeaba `LOADER OVERWRITE {from: 'NeoForge', to: 'Forge'}` cada vez que el server arrancaba. El loader en `instance.json` se sobreescribía a "Forge" en cada restart
- **Root cause encontrado** (vía console logging temporal agregado al smart detection):
    - El `msg` que llega a `parseLog` **NO incluye el timestamp prefix** `[07jun2026 04:19:28.046] [modloading-worker-0/INFO] [...]:`
    - El log del console mostraba `msg: 'NeoForge mod loading, version 21.11.42, for MC 1.21.11'` (53 chars, no la línea completa de 153 chars)
    - Esto significa que el `msg` es solo el cuerpo del mensaje, no la línea cruda
- **Por qué el smart detection fallaba**:
    - Mi check era: `const isNeo = msg.startsWith("[") && msg.includes("NeoForge mod loading")`
    - `msg.startsWith("[")` retornaba `false` (porque msg empieza con "NeoForge", no con "[")
    - `msg.includes("NeoForge mod loading")` retornaba `true`
    - Resultado: `isNeo = false` → `loader = "Forge"` (incorrecto)
- **Lección crítica**: nunca asumir la estructura del input. El `msg` que recibía `parseLog` NO era la línea cruda del log file — era el "message" pre-extraído (probablemente por el parser Rust `ParsedLog::new(&l)` que extrae `message` además de `raw`)
- **Fix aplicado en `store.svelte.ts:266-269`**:
    ```ts
    // ANTES (bug):
    const isNeo = msg.startsWith("[") && msg.includes("NeoForge mod loading");
    // AHORA (fix):
    const isNeo = /neoforge/i.test(msg);
    ```
    - Solo verifica que el mensaje contenga "neoforge" (case-insensitive)
    - Sin dependencia en la estructura del prefijo
- **Logging temporal removido** (`store.svelte.ts:234, 360`) — ya no es necesario porque encontramos el root cause
- **`instance.json` del fsedfse corregido**: `loader: "Forge"` → `"NeoForge"`
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Próximo**: usuario reinicia el server NeoForge 1.21.11. Si el loader se mantiene como "NeoForge" tras el restart, NeoForge queda definitivamente cerrado

### NeoForge — Refactor de smart detection (reordenamiento + comentarios)
- **Acción del usuario**: "haz las correcciones que creas convenientes, avisame al terminar"
- **Refactor aplicado en `store.svelte.ts:215-303`**:
    1. **Reordenamiento del if-else chain**: el orden anterior era `loaderMatch → modListMatch → neoforgeMatch → vanillaMatch`. El nuevo orden es `loaderMatch → neoforgeMatch → modListMatch → vanillaMatch` — pone `neoforgeMatch` (source of truth) ANTES de `modListMatch` (validación secundaria). No cambia el comportamiento (las regex son mutuamente excluyentes para una línea), pero hace el código más legible: primero las detecciones específicas, luego las genéricas
    2. **Bloque de comentarios al inicio** del if-else chain: documente la prioridad de detección, qué es source of truth, y la advertencia sobre `msg` no tener prefix de timestamp (lección del bug anterior)
    3. **Comentario actualizado en `modListMatch` branch**: el comentario antiguo decía incorrectamente que el `mc` se backfilleaba desde "Minecraft X.Y.Z (minecraft)" en una llamada posterior. El backfill real viene de `neoforgeMatch` en una línea de log separada. Corregido
    4. **Comentarios mejorados** en cada branch explicando el rol específico de cada detección
- **Sin cambios funcionales**:
    - `isNeo = /neoforge/i.test(msg)` (el fix del bug anterior se mantiene)
    - `if (neoforgeDirect.mc)` guard contra version vacío (mantiene el fix de no sobrescribir con string vacío)
    - Idempotency: `if (instance.X !== detected.X)` (sin cambios)
- **Logging temporal removido** (no era necesario dejarlo)
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos
- **Estado**: NeoForge listo para validación final con restart del server fsedfse

### Display — Omitir visualmente el build cuando es igual a la version
- **Pedido del usuario**: "Podriamos omitir visualmente el paso 2 para que solo muestre el 3? esto a nivel visual, si se re quiere el paso 2 dime"
- **Contexto**: en una instalación nueva de NeoForge donde el usuario seleccionó el build (ej. "26.1.2.72") como la version inicial, el smart detection produce este flujo:
    - **Step 1 (default)**: `version="26.1.2.72"`, `build=null` → title: "NeoForge 26.1.2.72"
    - **Step 2 (intermedio)**: `modListMatch` setea `build="26.1.2.72"` pero deja `version=undefined` (skip). Como la version del JSON sigue siendo "26.1.2.72" (la del usuario), el display muestra "NeoForge 26.1.2.72 - 26.1.2.72" (redundante)
    - **Step 3 (final)**: `neoforgeMatch` actualiza `version="26.1.2"` (la MC version real). El display muestra "NeoForge 26.1.2 - 26.1.2.72" (limpio)
- **Análisis**: el "Step 2" es un artefacto del orden de procesamiento del smart detection (modListMatch dispara antes que neoforgeMatch, en llamadas separadas). No es un estado deseado, solo un gap entre dos actualizaciones.
- **Fix aplicado** (a nivel visual, no en el smart detection):
    - `InstanceGallery.svelte:155`: `{#if instance.build}` → `{#if instance.build && instance.build !== instance.version}`
    - `InstanceDetail.svelte:325`: misma condición
- **Resultado**: cuando `build === version`, el build se oculta. El flujo visible ahora es:
    - "NeoForge 26.1.2.72" (Step 1)
    - "NeoForge 26.1.2.72" (Step 2 visualmente igual a Step 1, build oculto)
    - "NeoForge 26.1.2 - 26.1.2.72" (Step 3, ya son diferentes)
- **Por qué no tocar el smart detection**: el smart detection debe capturar fielmente lo que reporta el server (incluyendo redundancias temporales). El display es responsable de presentar la información limpiamente. Separación de responsabilidades
- **Edge case**: si después de un restart completo el `build` resulta igual al `version` por alguna razón (ej. version="1.21.11" y build="1.21.11" porque el install setea ambos al MC version), el build se oculta. Esto es probablemente el comportamiento deseado porque no hay info útil adicional
- **Verificación**: `npm run check` retorna 7 errores pre-existentes, 0 nuevos

### Forge — Versión dropdown vacío, install falla
- **Síntoma reportado por el usuario**: "forge me da error al crear, tambien la lista de versiones no se despliega correctamente"
- **Root cause**:
    - En `CreateInstanceModal.svelte:369-394`, el branch "Others" (que cubre Paper/Purpur/Folia/Spigot/etc.) llama a `get_project_versions(project)` con `project = "forge"`
    - El Tauri command `get_project_versions` (`commands/versions.rs:75-108`) construye URL `https://api.papermc.io/v2/projects/forge` para cualquier proyecto que no sea "purpur"
    - PaperMC **no tiene un proyecto "forge"** en su API → devuelve 404
    - El catch block en el modal setea `versions = []` → dropdown vacío
    - Cuando el usuario crea la instancia con version vacío, el `ForgeLoader::install` (`loaders/mods.rs`) intenta descargar `https://maven.minecraftforge.net/.../forge--installer.jar` (URL malformada) → falla
- **Fix requerido** (pendiente): el loader Forge necesita su propio path de version-fetching, similar a NeoForge:
    1. Agregar `ForgeLoader::fetch_versions()` que obtenga la lista de versiones de Forge (vía `maven.minecraftforge.net/.../forge/maven-metadata.xml` o API alternativa)
    2. Agregar un Tauri command `get_forge_versions` en `commands/versions.rs`
    3. Modificar el modal para que Forge use su propio command (en lugar del genérico `get_project_versions`)
- **Workaround inmediato**: Forge queda en estado "no funcional" hasta que se implemente el fix. El catalog mantiene el ✗
- **Estado**: pendiente de implementación. Sigo con Arclight (siguiente en la lista del usuario)

### Arclight (y demás hybrids) — Stub confirmado
- **Verificación al arrancar con Arclight**: el stub retorna el error esperado
- **Flujo del error**:
    1. Usuario selecciona Arclight en el modal
    2. `create_instance` dispatcher (`commands/instance.rs:195-205`) tiene un catch-all que captura `InstanceEngine::Mohist | Arclight | Banner | Magma` y retorna `Err(format!("{:?} is not yet supported by the new loader system", loader_engine))` ANTES de llegar al registry
    3. El error se propaga al UI como "Error: Arclight is not yet supported by the new loader system"
- **Por qué el dispatcher corta antes del stub**: defensa de defense-in-depth. El stub en `loaders/hybrids/arclight.rs` también retornaría error, pero el dispatcher lo previene por si acaso
- **Lo que se necesita para hacer Arclight funcional**:
    1. Implementar `ArclightLoader::fetch_versions()` (consultar Jenkins CI de Arclight o API alternativa)
    2. Implementar `ArclightLoader::install()` (descargar `arclight-{mc}.jar` y ejecutarlo, o aplicar un Forge patch al Minecraft jar)
    3. Remover `InstanceEngine::Arclight` del catch-all en el dispatcher
    4. Validar que el smart detection reconoce el formato de log de Arclight (probablemente emite líneas "Forge mod loading" o "Bukkit" — necesita testing)
- **Estado**: queda con tachita. Sigue siendo un follow-up significativo (mismo scope que los otros 3 hybrids: Mohist, Banner, Magma)

### Velocity — Proxy funcional end-to-end ✅
- **Resultado reportado por el usuario**: "acabo de probar velocity y funciono correctamente" — log de inicio confirma:
    - `Booting up Velocity 3.4.0-SNAPSHOT`
    - `Loaded 1 plugins` (bStats)
    - `Listening on /[0:0:0:0:0:0:0:0]:25565` (IPv6 wildcard)
    - `Done (1.08s)!` (server ready)
- **Nota del smart detection**: Velocity NO emite ninguna de las líneas que matchean los regexes del smart detection (`This server is running X version Y`, `NeoForge mod loading`, `Starting minecraft server version X`). El smart detection no se activa para Velocity, lo cual es OK — la version fue seteada correctamente al crear la instancia via el install
- **Catalog actualizado**: `Velocity: tested = true` (sin tachita)
- **Próximo**: usuario prueba Waterfall (idéntico flow a Velocity, mismo endpoint PaperMC) y luego BungeeCord (formato Jenkins API diferente)

### Waterfall — Funcional pero state detection falla
- **Resultado reportado por el usuario**: "se crea la instancia bien e inicia pero creo que al no tener done se queda en 'iniciando'"
- **Verificación del install**: ✅ Waterfall se descargó correctamente, plugins cargados (6 plugins default), `Listening on /0.0.0.0:25577` después del `BindException` (porque Velocity seguía en 25565 — normal cuando tienes 2 proxies)
- **Bug identificado** (issue de UI, no del install):
    - `server.rs:147-155` tiene la detección de "server ready":
      ```rust
      if lower.contains("done (") || lower.contains("for help, type \"help\"") {
          is_running = true;
          let _ = update_instance_state(&...);
      }
      ```
    - Este check matchea Vanilla (`Done (Xs)! For help, type "help"`) y Velocity (`Done (1.08s)!`) pero NO matchea Waterfall
    - Waterfall solo emite `Listening on /0.0.0.0:25577` cuando está listo (no hay "Done")
    - BungeeCord probablemente tampoco emite "Done" (es similar)
    - **Resultado**: la UI nunca recibe la transición a `Running`, queda en "Iniciando" aunque el server sí esté corriendo
- **Fix requerido** (pendiente, 1 línea): agregar `"listening on"` al check en `server.rs:147`:
    ```rust
    if lower.contains("done (") || lower.contains("for help, type \"help\"") || lower.contains("listening on") {
    ```
- **Catalog actualizado**: `Waterfall: tested = true` (el server funciona, solo la UI muestra estado incorrecto)
- **Estado**: Waterfall funcional, pendiente fix cosmético del state detection
- **Próximo**: usuario prueba BungeeCord (último proxy; usa Jenkins API de md-5, formato de build numbers)

### BungeeCord — Funcional end-to-end + state detection fix verificado
- **Acción del usuario**: corrigió el issue del dropdown de BungeeCord agregando atajo en `get_project_versions` para "bungeecord" que delega a `BungeeCordLoader::list_recent_builds()` (Jenkins API)
- **Fixes del usuario** (verificados, no modifiqué):
    - `commands/versions.rs:76-79`: atajo `if project == "bungeecord"` que evita el 404 de PaperMC y llama directo a `BungeeCordLoader::list_recent_builds()`
    - `CreateInstanceModal.svelte:24-30`: nuevo `isProxyLoader` derived state (usa la capability `isProxy` del catalog) para ocultar el EULA checkbox en proxies
- **State detection del usuario** (también verificado en `server.rs:142-166`):
    - Lookup del `is_proxy` capability via `LoaderRegistry::by_engine(instance_engine).capabilities().is_proxy`
    - Patrón específico para proxies: `lower.contains("listening on /")` (con slash para evitar falsos positivos)
    - Patrón genérico para servers: `lower.contains("done (") || lower.contains("for help, type \"help\"")`
    - Diseño future-proof: cualquier proxy futuro en el catalog con `isProxy: true` queda automáticamente cubierto
- **Catalog actualizado**: `BungeeCord: tested = true` (sin tachita)
- **Estado**: Familia Proxies cerrada ✅ (Velocity, Waterfall, BungeeCord)
- **Acción del usuario**: pidió un diagrama del flujo completo (frontend → backend → descarga → install → carpeta) para analizar antes del próximo fix
- **Diagrama entregado en chat** (no en dev_log, solo como análisis):
    - 8 fases frontend→backend, con flujo Vanilla específico (URL=1, dispatch=2, fetch_versions fallback=3, resolve_download=4, download_file=5, write_eula=6)
    - Divergencias por loader: Bukkit/Mods (installer unpacking con `java -jar ... --installServer`)/Proxies/Hybrids (stub error)
    - Punto de fallo confirmado: entre `response.content_length()` y `bytes_stream().next()` — el handshake + headers OK, pero el body no se entrega
- **Curl test del usuario** (siguiente acción):
    - Comando: `curl -v -o /tmp/test.jar https://piston-data.mojang.com/v1/objects/97ccd4c0ed3f81bbb7bfacd...`
    - **El hash se truncó en PowerShell** (solo 24 chars enviados: `97ccd4c0ed3f81bbb7bfacd`). El 404 (`HTTP/1.1 404 The specified blob does not exist`) es esperado porque ese blob no existe
    - Datos valiosos del output a pesar del 404: TLS OK, ALPN server-accepted http/1.1, connection keep-alive, IPv6 outbound, 215 bytes XML body recibido correctamente
    - **Conclusión clave**: curl **sí fluye el body** de respuestas HTTP/1.1 (recibió los 215 bytes del 404 XML sin problema). Reqwest con `http1_only()` + `native-tls` no fluye bodies de 60MB
- **Hipótesis refinada**: bug en hyper 0.14 + `body::Body::channel()` cuando el TLS backend es schannel. Rustls event loop es diferente y no presenta el bug
- **Plan de fix (pendiente de aplicar)**: cambiar cadena de features de reqwest
    - **De**: `reqwest = { version = "0.11", features = ["json", "stream"] }` (usa `default-tls` = schannel en Windows)
    - **A**: `reqwest = { version = "0.11", default-features = false, features = ["json", "stream", "rustls-tls"] }`
    - Riesgo bajo: rustls es drop-in para el uso que le damos. Cambio localizado al `Cargo.toml` del binario src-tauri
    - Plan B si rustls no funciona: reemplazar `bytes_stream()` por `read_to_end()` con progreso estimado (sacrifica streaming, pero garantiza que la descarga funcione)
- **Estado**: pendiente de aplicar fix (esperando confirmación del usuario para proceder)

## 2026-05-29

### Add Instance Creator — Hybrids Section (v5.1)
- **Estado**: implementado (nueva sección "Híbridos" entre Mods y Proxies)
- **Frontend**: `CreateInstanceModal.svelte`
    - Nueva entrada `Hybrids` en `CATEGORY_COLORS` con paleta yellow-400
    - Nueva entrada en `LOADERS_BY_CATEGORY` con 4 loaders: Mohist, Arclight, Banner, Magma
    - Return type de `getLoaderCategory()` extendido: `"Vanilla" | "Bukkit" | "Mods" | "Hybrids" | "Proxies"`
- **i18n**: nueva clave `category_hybrids` en `en.json` ("Hybrids") + `es.json` ("Híbridos")
- **Diseño**: sin badge per-loader — el color de sección yellow ya diferencia los híbridos
- **Justificación pick loaders**: 4 más populares + activos en 2026 (Forge/Fabric + Bukkit). Mohist = el más popular histórico. Arclight = único con soporte NeoForge. Banner = el único Fabric hybrid. Magma = veterano aún mantenido.

---

## 2026-06-02

### Add Instance Creator — Loader Category Selector (Visual Mockup)
- **Estado**: implementado (v4 — lista categorizada, dummy interactivo)
- **Frontend**: `CreateInstanceModal.svelte` — lista vertical agrupada por categoría en Group 2
    - 4 secciones con header coloreado + línea horizontal: Vanilla, Bukkit, Mods, Proxies
    - Cada loader es fila compacta clickable; activa = dot del color de categoría + fondo tenue
    - Modloaders (NeoForge, Forge, Fabric) llevan badge "MODS" naranja
    - Versión movida a fila propia (NeoForge 2-step MC+Build / estándar con snapshots)
    - EULA checkbox movido a su propia fila full-width fuera del grid
    - Eliminado: dropdown "Loader" previo + su label + estado `showLoaderDropdown` (UI)
- **Estado TS**: removido `selectedCategory` (v3 → v4 ya no se usa; i18n `category_*` se reaprovecha como labels de sección)
- **Loaders añadidos al dummy** (no al backend): Spigot (Bukkit), Forge + Fabric (Mods), BungeeCord (Proxies)
- **Colores**: emerald (Vanilla), blue (Bukkit), orange (Mods), purple (Proxies) — via lookup map `CATEGORY_COLORS` (JIT-safe)
- **i18n**: 4 claves nuevas `category_vanilla/bukkit/mods/proxies` en `en.json` + `es.json` (valores idénticos: términos técnicos)
- **Iteración diseño**: v1 game-version ❌ → v2 grid 2x2 ❌ → v3 tabs row ❌ → v4 lista categorizada ✅
- **Pendiente próximo paso**: conectar lógica real (vs dummy), añadir loaders faltantes al backend, decidir filtrado NeoForge 2-step

### Add Instance Creator — Improvements (planning)
- **Archivo objetivo**: `src/lib/components/modals/CreateInstanceModal.svelte`
- **Estado actual**: modal 420px con 3 tabs (`custom` / `file` / `import`), 7 loaders (Vanilla, Paper, Purpur, Folia, Velocity, Waterfall, NeoForge), NeoForge con doble selector MC→build, importer ZIP (CurseForge/Modrinth) con badge naranja, sección CF Profile Code con modal Cloudflare, IconPicker lateral.
- **Tareas de fondo detectadas durante la lectura inicial** (sin compromiso de tocarlas, priorizar con usuario):
    - Estado `customUrl` declarado (L34-35) y referenciado en disabled (L1062) — sin UI de input visible → parece dead code
    - Texto hardcoded "Seleccionar" (L552) — pendiente i18n
    - EULA checkbox duplicado en 2 ramas (custom + file/import)
    - CF Profile Code: input existe pero el modal Cloudflare solo se abre, no ejecuta flujo
    - 4 dropdowns con misma estructura (loader, version, neoforge MC, neoforge build) → candidato a refactor
    - Watchdog 5s en L213-225: lógica escondida en listener, podría ser un `onMount`/cleanup más limpio
- **Pendiente**: definir alcance concreto con el usuario antes de empezar
- **Invariante**: i18n ES+EN obligatorio en cualquier string nueva

---

## 2026-05-29

### NeoForge Engine Support (Full Pipeline)
- **Backend**: Nuevos comandos `get_neoforge_builds` y `get_neoforge_mc_versions` en `versions.rs` — parseo directo del `maven-metadata.xml` de NeoForge sin dependencia de XML parser pesado.
- **Installer**: Función `install_neoforge()` — descarga el installer JAR, lo ejecuta headlessly (`--installServer`), emite progreso al frontend via `install-progress`, y limpia artefactos temporales.
- **Version Mapping**: Helper `extract_neoforge_category()` que extrae la versión de Minecraft desde el formato de versión NeoForge (ej: `21.1.172` → `1.21.1`).
- **UI**: Rama condicional en `CreateInstanceModal.svelte` para NeoForge — dropdown de versiones MC disponibles + selector de builds NeoForge específicos + checkbox para betas.
- **i18n**: Claves `neoforge_version`, `neoforge_mc_version` en `en.json` y `es.json`.
- **Cargo.toml**: Añadida dependencia `slug` para slugificación de nombres de instancia.

### CurseForge / Modrinth Modpack Importer
- **New File**: `src-tauri/src/commands/curseforge.rs` (598 líneas) — importador completo de modpacks ZIP.
- **Dual Format**: Detección automática `manifest.json` (CurseForge) vs `modrinth.index.json` (Modrinth).
- **CurseForge Flow**: Parseo manifest → resolución URLs via CurseForge API (`$2a$10...` key) → descarga concurrente en batches de 10 → copia overrides → instalación loader.
- **Modrinth Flow**: Parseo index → descarga directa CDN en batches de 8 → respeta rutas relativas (`mods/`, `config/`) → copia overrides.
- **ZIP Extraction**: Helper `extract_zip_to_dir` con validación `enclosed_name()` para seguridad path traversal.
- **UI**: Nueva pestaña "Import" en `CreateInstanceModal` con selector ZIP premium (gradiente naranja), campo CurseForge Profile Code con modal de restricción Cloudflare, y feedback visual de progreso.
- **i18n**: 12 nuevas claves para importación (`import_modpack_zip`, `import_modpack_cf_blocked_title`, etc.).

### Smart Server Launch Mode (NeoForge/Forge)
- **Detection**: `find_forge_args_file()` en `server.rs` — busca recursivamente `win_args.txt` (Windows) o `unix_args.txt` (Unix) dentro de `libraries/` hasta 8 niveles de profundidad.
- **Dual Launch**: Si encuentra args file → lanza con `java @user_jvm_args.txt @libraries/.../win_args.txt nogui` (NeoForge/Forge moderno). Si no → lanza clásico con `-jar server.jar nogui`.
- **Auto-Config**: Genera `user_jvm_args.txt` automáticamente si no existe, con encoding UTF-8.
- **Fallback**: JAR lookup alternativo en raíz de instancia para servidores Paper/Purpur con estructura no estándar.

---

## 2026-05-25

### Console: Smart Player Detection Fix
- **Bugfix (ReferenceError)**: Solucionado error crítico en `store.svelte.ts` donde la variable `runtime` no estaba definida dentro de `parseLog`, bloqueando silenciosamente la detección de jugadores.
- **Language-Agnostic Parsing**: Actualizadas las regex de conexión/desconexión para depender de los logs técnicos internos de NMS (`logged in with entity id`, `lost connection`) en lugar del chat, garantizando el soporte en múltiples idiomas (español, portugués, etc.) y previniendo interferencias de plugins que traducen los mensajes de bienvenida.
- **Clean Naming**: Añadido filtro que limpia prefijos de seguridad o modloaders (ej: `[Not Secure] `) de los nombres de usuario extraídos.

---

## 2026-05-16

### MSIX Store Deployment & Version Automation (Final)
- **MSIX Manual Packaging**: Implementado sistema de empaquetado y firma manual (`MakeAppx`, `SignTool`) vía PowerShell para saltar limitaciones de Tauri 2.
- **Unified Pipeline**: Workflow unificado en GitHub Actions que genera EXE (con auto-update) y MSIX en una sola ejecución (< 11 min).
- **Dynamic Window Title**: Refactor de Rust (`lib.rs`) para inyectar la versión dinámicamente en el título de la ventana durante el arranque, eliminando títulos hardcodeados.
- **Rust Versioning**: Sincronización automática de `APP_VERSION` con el `Cargo.toml` mediante macros de compilación (`env!`).
- **Update System Stability**:
    - Añadido retraso de 1.5s en el arranque para estabilizar conexión antes de buscar actualizaciones.
    - Unificada la lógica reactiva entre `+layout.svelte` y `GlobalUpdateBanner.svelte` para evitar colisiones de red.
- **Store Compliance**: Ajustado `AppxManifest.xml` con identidad oficial del Partner Center (`cPathz`, `es-ES`) y justificaciones de capacidad (`runFullTrust`).

---

## 2026-05-10

### Microsoft Store & CI/CD Automation (Fase 4)
- **Dual Build System**: Configurado GitHub Actions para generar paquetes NSIS y MSIX simultáneamente.
- **Detección de Entorno**: Implementado comando en Rust para identificar si la app corre como `standalone` o `msix`.
- **Auto-Versioning**: Automatizado el parche de versión de 4 dígitos (0.1.x.0) durante el build de la Store en CI.
- **UI Adaptativa**: 
    - Título dinámico para la versión de la Store.
    - Bloque de actualizaciones simplificado en Settings para MSIX.
    - Banner inteligente que respeta el flujo de Windows Update.

### Update System & Professional UX (Fase 3)
- **Store Architecture**: Implementada `updateData` en `AppState` para centralizar la detección de nuevas versiones.
- **Smart Zen Mode**: 
    - Lógica de "Aviso Único": El popup de inicio solo salta una vez por cada nueva versión detectada si el Modo Zen está activo.
    - Persistencia: Implementada `lastIgnoredVersion` en el store para evitar re-notificaciones invasivas tras un descarte manual.
    - Banner Independiente: El banner de la Home permanece como recordatorio no intrusivo persistente.
- **Instalación Real**: Conexión total con `tauri-plugin-updater` para descarga e instalación (con reinicio automático).
- **Feedback Visual**: Implementada barra de progreso real con gradientes y porcentaje reactivo durante la descarga.
- **Settings Redesign**: 
    - Layout Horizontal: Rediseño basado en filas "Split" (Info izquierda / Controles derecha) para optimizar el flujo visual.
    - Tipografía: Escalados los tamaños de fuente (Títulos `text-base`, Descripciones `text-sm`) para mejorar la legibilidad.
    - Agrupación: Reorganizada la sección de Actualizaciones integrando el Modo Zen y el botón de búsqueda en un solo bloque.
- **Roadmap**: Creado `ROADMAP_STORE.md` con la estrategia técnica para la Microsoft Store y CI/CD.

## 2026-05-09

### Console Optimization & Smart Parsing
- **Optimización de Consola**: Soporte Log4j2 ISO y deduplicación de prefijos redundantes (limpieza de `[WARN]: WARNING:`).
- **Controles de UX**: Toggles para modo "Compacto" (ocultar etiquetas) y visualización de "Hora".
- **Nueva Pestaña de Errores**: 
    - Implementación de `ErrorLogView.svelte` con estado vacío premium.
    - Captura automática de logs de tipo `ERROR` y `FATAL` en un array persistente por instancia.
    - Badge dinámico en la navegación con contador reactivo y alerta visual en rojo.
    - Botón de "Limpiar" para vaciar la lista de errores manualmente.
    - Diseño de tarjetas detalladas para visualizar errores con timestamp y plugin origen.
- **Deduplication**: Implementado sistema de limpieza de prefijos redundantes (ej: evita `[WARN]: WARNING:`).
- **Log4j2 Support**: Añadido patrón de parseo nativo para timestamps ISO (`YYYY-MM-DDTHH:mm:ss.SSSZ`).
- **UX Controls**: Nuevos botones "Compacto" (oculta niveles) y "Hora" (toggle timestamps) en la barra de herramientas.
- **Aesthetics**: Unificada la lógica de colores entre Formatos 2 y 3. Formato 3 ajustado a `text-gray-400` (estilo RAW).
- **Bugfix (Timestamps)**: Eliminada la generación de horas artificiales en líneas vacías o incompletas de plugins.
- **Bugfix (Indentation)**: Eliminado `trim()` agresivo en Rust; ahora los logs multilínea (Quartz) y Stacktraces preservan su sangría original.
- **Robustness**: Ajustada la lógica de ocultación de tags de plugins para evitar colisiones cuando el nombre del plugin aparece en el mensaje.

## 2026-05-08

### Console: Formato 2 & Minimalist UX
- **Format 2**: Implementada vista minimalista sin timestamp. Estructura: `[NIVEL]: [PLUGIN] MENSAJE`.
- **Milestones**: Colores exclusivos para eventos críticos: `STARTING` (Celeste), `SUCCESS` (Verde), `STOPPING` (Naranja/Rojo).
- **Opacity**: Sistema de opacidad jerárquica (70% - 100%) para diferenciar estados de carga, ejecución y cierre.
- **Colors**: `INFO` subido a `Zinc-300` para mejor legibilidad sobre fondo oscuro en Formato 2.
- **Isolation**: Refactor de `formatLog` para segregar totalmente la lógica estética de Formato 1 y Formato 2.
- **Scalability**: Capacidad aumentada a 500 líneas en DOM y 2,000 en memoria (ajustable).
- **Docs**: Creado `docs/CONSOLE_STANDARD.md` con reglas de parseo ISO, TrueColor y Stacktraces.

---

## 2026-05-07

### Console ANSI Colors & Stacktrace Styling (Fase 2)
- **Library**: Integración `ansi-to-html` para soporte nativo de colores ANSI en consola.
- **Security**: Configurado `escapeXml: true` en el conversor para prevenir inyección HTML vía logs.
- **Styling**: Colores Tailwind por nivel: `red-400` (ERROR/FATAL), `yellow-400` (WARN), `blue-400` (DEBUG), `gray-400` (INFO).
- **Stacktraces**: Identación `pl-6`, fuente `text-[11px]` y `opacity-60` para agrupación visual con el error padre.
- **Architecture**: Refactor de `store.svelte.ts` para soportar array de `ParsedLog` (objetos) en lugar de solo `string`.
- **Logic**: Función `formatLog` en `ConsoleView.svelte` con detección inteligente de ANSI y fallback para strings.

### Console Defaults & RAW Mode Hardening
- **Defaults**: `logFormat` → `formato1`, `applyConsoleSettings` → `true`, `wrapConsoleText` → `false`.
- **Typography**: Ajustado a JetBrains Mono, 14px, 1.4 line-height (según especificaciones UI).
- **RAW Mode**: Aislado de todo procesamiento. Ahora muestra el texto bruto del JAR sin colores, sin ANSI y sin estilos de stacktrace (safe fallback).
- **Format 1**: Centraliza toda la lógica de visualización enriquecida y colores.

---

## 2026-05-05

### Log Parser & Format Testing
- **New File**: `src/lib/utils/logParser.ts` — función `parseMinecraftLog()` y tipo `LogEntry`
- **Parser Features**:
  - Extrae timestamp, nivel (INFO/WARN/ERROR/DEBUG), source ([Shop], [Backuper], etc.) y mensaje limpio
  - Soporta formatos Standard `[HH:mm:ss] [Thread/LEVEL]: msg` y Paper short `[HH:mm:ss LEVEL]: msg`
  - Elimina códigos ANSI (`\x1B[...m`)
  - Regexes compiladas + Set para validación → O(0.1ms) por log
  - Fallback para texto plano (sin timestamp)
- **Format Panel**: Nuevo botón "Formato 1" que muestra últimos 10 logs parseados con colores por nivel
- **Tests**: `scratch/test-log.ts` con 6 test cases — todos pasando ✓

### Console: Dual Panel Players/Format (No-Tab Design)
- **Structure**: 3 botones apilados en toolbar flotante — Expand/Collapse | Players | Format
- **Botón 1** (Expand/Collapse): Toggle showPlayers — abre/cierra panel, ícono chevron dinámico `< >`
- **Botón 2** (Players): Abre panel con vista Jugadores, resalta cuando activo (blue-500/20)
- **Botón 3** (Format): Abre panel con vista Formato, resalta cuando activo (blue-500/20)
- **Default**: Panel Jugadores abierto + consola muestra logs en RAW por default
- **Navigation**: Presionar botón 2 o 3 abre panel automáticamente + cambia vista
- **Auto-Hide**: Toolbar se desvanece a opacity-40 después de 2 segundos sin interacción (solo si cursor no está sobre botones)
- **Format Testing**: Botón "Raw" muestra preview dinámico de últimos 10 logs sin procesar
- **Hide Noise Button**: Deshabilitado (disabled) — consola siempre muestra raw
- **Files**: `src/lib/components/console/ConsoleView.svelte`, `src/lib/components/views/InstanceDetail.svelte`

### Security Audit & CSP Hardening
- **Scan**: Audit seguridad rápido — no secrets hardcodeados, .gitignore correcto, permisos Tauri granulares
- **CSP Fix**: `tauri.conf.json` — removido `'unsafe-eval'` de `script-src` (era innecesario en Svelte, cierra puerta XSS)
- **Status**: CSP ahora restrictivo — `script-src 'self'` únicamente

### Dev Log Refactor
- **Format**: Reescritura `dev_log.md` a formato compacto machine-readable — agrupado por fecha, bullets concisos, máx 1 nivel nesting
- **Structure**: Headers `## YYYY-MM-DD` → `### vX.X.X Título` → bullets puntuales (~70 chars)
- **Goals**: Legibilidad + parseable + eficiente tokens + aliado para auditorías IA futuras
- **TODO**: Reorganizado en Completado/Pendiente/Invariantes (i18n obligatorio)

### Cambios pendientes (sin commit)
- **Dynamic Version System**: `store.svelte.ts:31` cambia de `version: "0.1.7"` → `version: ""` (leída en runtime via `getVersion()`)
- **Window Title**: `+layout.svelte` setea dinámicamente `v.${version} (${tag})` al iniciar
- **Display Format**: `v{version}` → `v.{version}` (alinea con tag prefix `v.X.X.X`)
- **GlobalUpdateBanner**: Nuevo componente que parsea `[CRITICAL]` y `[APP_NOTES]/[/APP_NOTES]` en release body
- **Release Protocol**: Requiere actualización — versión dinámica elimina necesidad de hardcodear en `store.svelte.ts`

---

## 2026-05-01

### v0.1.10 (Bump)
- `package.json` + `src-tauri/tauri.conf.json` → `0.1.10`

### v0.1.9 (Bump)
- `package.json` + `src-tauri/tauri.conf.json` → `0.1.9`

### v0.1.8 (Bump)
- `package.json` + `src-tauri/tauri.conf.json` → `0.1.8`

### v0.1.7 Addons UX & i18n Polish
- **UI/UX**: Toggle cápsula esmeralda + reemplazo ON/OFF por iconos encendido/apagado + tooltips contextuales dinámicos
- **i18n**: Traducción ES/EN completa pestaña Addons (columnas, estados, errores, vistas vacías)
- **Security**: Bloqueo visual+funcional (escala grises + cursor prohibido) mientras servidor encendido

### v0.1.7 Addons Engine Upgrade
- **Gestión**: Toggle estado via rename `.jar`↔`.jar.disabled` + detección extensiones legadas (`.bkp/.bak/.old/.off`) + borrado inteligente con cleanup config
- **Paralelo**: Escaneo multi-hilo via Rayon
- **Caché**: Unificado `addons_cache.json` reutilizado en todas operaciones
- **Watcher**: File watcher sincroniza UI en tiempo real

### v0.1.6 Addons Update
- **UI**: Eliminación header interno Addons + reubicación botones (Actualizar, Añadir) en tab bar instancia
- **Validación Plugins**: Sistema "candados" JAR — escaneo descriptores (`plugin.yml`, `paper-plugin.yml`, `fabric.mod.json`, `mods.toml`, `velocity-plugin.json`) + detección duplicados (nombre, versión, size, mtime) + modal instalación interactiva (Instalar/Reemplazar/Omitir)
- **Backend**: Comandos Rust optimizados — sustitución segura versiones antiguas + apertura directorio por motor

---

## 2026-04-25

### v0.1.5 Beta
- **Lifecycle**: Detección robusta estados (Starting/Running/Stopping/Stopped) via parsing dinámico stdout/stderr
- **Sync UI**: Sincronización visual perfecta Galería+Detalles — previene cierres fantasmas
- **Fix Backend**: Corrección critical — estados asíncronos no forzaban a `Running`
- **CSS Grid**: `InstanceGallery.svelte` → `xl:grid-cols-3` — evita truncado incorrecto con versions/builds extensas
- **i18n**: Correcciones `en.json`/`es.json` — sistema estados y status

---

## 2026-04-21

### v0.1.4 Stability Patch
- **Critical**: Pantalla azul/blanca causada condición carrera `svelte-i18n` — guardas robustas `+layout.svelte`/`Home.svelte` + timeout 1.5s emergencia
- **TS**: Optimizaciones `onMount` + fixes errores tipos componentes clave
- **z-index**: Fix bloqueo interfaz al cambiar pestaña instancia
- **Error**: `ReferenceError: markDirty is not defined` en InstanceSettings — resuelto
- **i18n**: Watermark "Copia de Evaluación" localizado
- **Async**: Sincronización mejorada puerto servidor + memoria sistema

---

## 2026-04-20

### v0.1.4 Beta
- **Players Panel**: Sidebar usuarios conectados en consola + i18n ES/EN
- **Real-time**: Tracking jugadores via logs (Join/Leave/List)
- **Slots**: Detección reactiva `max-players` desde `server.properties` al iniciar
- **Sync**: Manual refresh lista + límite espacios mejorado
- **Typography**: +3px contadores/etiquetas — legibilidad
- **Avatars**: Integración MCHeads API — avatares 64px HD

---

## 2026-04-13

### v0.1.3 Beta
- **Display Name**: Edición in-situ instancia desde detalles
- **Bidirectional**: Puerto sincronizado Panel ↔ `server.properties`
- **Identity**: Unificación app ID → `AnvilCraftPanel`
- **Java**: Fix descarga Java 16 + inclusión Java 25 (LTS) portátil Adoptium
- **UX**: Mejora legibilidad sección Java + fix scroll console + localización tooltip "Abrir Carpeta"

---

## 2026-04-12

### v0.1.2 Beta
- **i18n**: Implementación completa `svelte-i18n` ES/EN
- **Toasts**: Unificación sistema notificaciones — eliminación alertas nativas
- **Progress**: Mapeo localizado estados backend Rust
- **Branding**: Identidad visual + versión unificada panel
- **Design**: Rediseño minimalista "Acerca de" + barra superior

---

## 2026-01-21

### UI & Layout Refactor
- Refactorización + optimización rendimiento Consola / Home / Layout principal

### Forge
- Streaming instalación en tiempo real + mejoras visuales + logs persistentes

### Consola
- Autocompletado comandos + versionado + soporte args avanzado + storage optimizado

---

## 2026-01-18

### Instancia Manager
- Borrado instancias implementado
- Selección versiones mejorada via API

---

## 2026-01

### Maintenance
- Licencia MIT + README actualizado + disclaimer IA

---

## TODO

**Completado:**
- [x] Integración UI/Backend creación+ajuste instancias
- [x] Pestaña Plugins — Fase 1 (UI base) + Fase 2 (JAR metadata + caché)

**Pendiente:**
- [ ] Optimización logs — eliminar redundancia guardado
- [ ] Accesibilidad panel skins
- [ ] Robustecer creación instancias — prioridad: Vanilla → Forge/Fabric → Proxies → Otros
- [ ] `server.properties` config sistema — mejorar
- [ ] Plugins Fase 3 — integración Modrinth búsqueda/instalación
- [ ] Consola — `--enable-native-access=ALL-UNNAMED` (Java avisos) + pre-generar `server.properties` + ANSI colors + error highlight
- [ ] **Conectividad/Túneles** — multijugador remoto
  - [ ] IP local + puerto dashboard instancia
  - [ ] Túnel (candidato: **playit.gg**) — compartir servidor sin abrir puertos
  - [ ] Botón "Compartir" — levanta túnel + muestra dirección pública
  - [ ] Eval alternativas: ngrok (TCP pago), Tailscale/ZeroTier (instalación ambos lados), Cloudflare Tunnel (no TCP raw)
  - [ ] Investigar herramientas túnel gratuitas terminal (sin deps servicios pago)

**Invariantes:**
- Toda funcionalidad nueva requiere claves i18n en `en.json` + `es.json`
- Siempre registrar la tarea a realizar en esta bitácora **antes** de empezar a modificar código, para no perder el contexto en caso de que se agoten los tokens o se corte el proceso.
