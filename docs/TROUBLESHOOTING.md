# Troubleshooting — Problemas conocidos y soluciones

> **Log de problemas** que han surgido en el proyecto. Cuando algo se arregla, se documenta aquí para no repetir el error.

---

## 🧹 Limpiezas de dead code (histórico)

### [2026-08-02] `src/lib/i18n/` — carpeta muerta eliminada

**Síntoma:** Carpeta duplicada de traducciones `src/lib/i18n/es.json` que nadie usaba.

**Causa raíz:** Diseño viejo donde había dos directorios de locales (`lib/i18n/` y `lib/locales/`). Solo `lib/locales/` quedó activo en `i18n.ts:7-8`, pero la otra carpeta nunca se borró.

**Verificación:** `grep -r 'i18n/' src/` no encontró referencias (solo el import de `i18n.ts` que apunta a `./locales/`).

**Fix:** `Remove-Item src/lib/i18n -Recurse -Force`. Commit: `chore: reorganizar estructura`.

---

### [2026-08-02] `src-tauri/src/lib/data/` — data huérfana eliminada

**Síntoma:** 150+ JSONs de Minecraft (blocks, items, commands, registries por versión) en `src-tauri/src/lib/data/minecraft/` que el binario no incluía y el runtime no leía.

**Causa raíz:** Diseño inicial donde la data de Minecraft se iba a embeber en el binario Rust con `include_str!` / `include_bytes!` / `include_dir!`. El diseño cambió: `commands/dev.rs::import_minecraft_data` ahora escribe a `../src/lib/data/` (lado Svelte) y el frontend lo lee con `import` directo. La carpeta Rust quedó huérfana.

**Verificación:**
- `grep -r 'include_str!\|include_bytes!\|include_dir!\|embed_file' src-tauri/src/` → 0 matches
- Todas las referencias en `commands/dev.rs:137,418,455,474,490,522,556` apuntan a `../src/lib/data/...`
- `src/lib/data/` (Svelte side) sigue siendo el que se usa en runtime: `ConsoleView.svelte:8-9`, `IconPicker.svelte:2`, `CreateInstanceModal.svelte:16`

**Fix:** `Remove-Item src-tauri/src/lib/data -Recurse -Force`. Commit: `chore: remove dead minecraft data embeds`.

**Lección:** Antes de usar macros de embebido (`include_str!` etc.), confirmar que el diseño los necesita. Si el frontend Svelte puede leer JSONs con `import`, no hay razón para duplicar al lado Rust.

---

## 🔴 Críticos (bloquean funcionalidad)

### [2026-06-18] Window title mostraba "v.0.1.12" hardcodeado

**Síntoma:** La ventana mostraba "AnvilCraft Panel v.0.1.12" sin importar la versión real.

**Causa raíz:** En `src/routes/+layout.svelte:41` había `setTitle("AnvilCraft Panel v.0.1.12 (beta)")` que se ejecutaba en `onMount` inmediato, antes de la lógica async que traía la versión real. Si el async fallaba silenciosamente, el título quedaba fijo en v.0.1.12.

**Fix:** Eliminada la línea hardcodeada. El título se construye ahora async con `getVersion()` real + `get_distribution_channel()`.

**Lección:** Nunca hardcodear versiones. Siempre leerlas de la fuente de verdad (Tauri `getVersion()` o `env!("CARGO_PKG_VERSION")` en Rust).

---

## 🟡 Warnings recurrentes (no bloquean pero molestan)

### Versiones desincronizadas entre archivos

**Síntoma:** El binario dice "v.0.1.13" pero `package.json` dice 0.1.14, o el badge de README dice v.1.2.

**Causa raíz:** La versión se define en 6 lugares (package.json, Cargo.toml, tauri.conf.json, AppxManifest.xml, README badge, SECURITY_AUDIT). Al bumpear, se actualiza uno y se olvidan los otros.

**Fix (2026-08-02):** Resuelto. Ahora `package.json` es la **única fuente de verdad** y un script sincroniza el resto.

```bash
# Ver status
npm run version

# Bumpear
npm run version -- patch    # o minor/major/"0.1.15"

# Sincronizar sin cambiar
npm run version:sync

# Verificar en CI
npm run version:check
```

Sincroniza 5 archivos automáticamente:
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/msix/AppxManifest.xml` (formato 4-segmentos, agrega `.0`)
- `README.md` (badge)
- `docs/SECURITY_AUDIT.md`

Doc completa: `docs/VERSIONING.md`.

---

### `dead_code` warnings en `curseforge.rs`

**Síntoma:** Al compilar, aparecen warnings sobre campos no usados:
```
warning: fields `name` and `version` are never read
  --> src/commands/curseforge.rs:40
```

**Causa raíz:** Los structs `CfManifest`, `MrIndex` y `MrFile` tienen campos definidos pero el código no los lee. Probablemente planeados para uso futuro que no se completó.

**Fix:** Tres opciones:
1. `#[allow(dead_code)]` en los structs
2. Borrar los campos no usados
3. Implementar el uso de esos campos

Recomendación: opción 2 (limpiar), pero requiere entender por qué estaban planeados.

---

### a11y warnings en Svelte (`ConsoleView`, `AddonInstallModal`)

**Síntoma:**
```
<div> with a mouseenter or mouseleave handler must have an ARIA role
```

**Causa raíz:** Componentes interactivos sin atributos ARIA para lectores de pantalla.

**Fix:** Agregar `role="..."` y `tabindex="0"` a los `<div>` con handlers de mouse.

---

## 🐛 Bugs de runtime

### [2026-08-02] `session.lock` stuck — Minecraft server no arranca después de cerrar AnvilCraft

**Síntoma:**
```
[ERROR]: Failed to start the minecraft server
java.io.IOException: El proceso no tiene acceso al archivo porque otro proceso tiene bloqueada una parte del archivo
    at net.minecraft.util.DirectoryLock.create(DirectoryLock.java:35)
    at net.minecraft.world.level.storage.LevelStorageSource$LevelStorageAccess.createLock(...)
```

El server no arranca, aunque la app esté cerrada. El log del server muestra el error justo después de `Starting net.minecraft.server.Main`.

**Causa raíz:** Minecraft usa un archivo `session.lock` en `<instancia>/.minecraft/` para evitar que dos servers abran el mismo mundo simultáneamente. Cuando el child process de `java.exe` queda zombie (AnvilCraft crashea, o la app se cierra abruptamente), Windows no suelta el file lock inmediatamente. El nuevo server no puede crear su `session.lock` → IOException.

**Hay dos causas concurrentes:**
1. El child process de `java.exe` no se mata/espera correctamente cuando AnvilCraft termina (o crashea). El `session.lock` queda tomado por el Java zombie.
2. Windows puede tardar en liberar file locks de procesos zombie, incluso después de matarlos.

**Fix inmediato (workaround):**
```powershell
# 1. Ver qué java está corriendo
Get-Process java -ErrorAction SilentlyContinue

# 2. Matar todos los java zombies
Get-Process java -ErrorAction SilentlyContinue | Stop-Process -Force

# 3. Borrar el session.lock de la instancia
Remove-Item "C:\Users\LMacias\AppData\Roaming\AnvilCraftPanel\instances\<nombre>\.minecraft\session.lock" -Force -ErrorAction SilentlyContinue
```

**Fix de raíz (pendiente en `src-tauri/src/commands/server.rs`):**
1. Usar `command.kill_on_drop(true)` en el `tokio::process::Command` o `std::process::Command` que spawnea `java -jar`. Esto garantiza que si el handle de Tauri se cae, el child también.
2. Después de `Stop`, llamar `child.wait()` para confirmar que el proceso murió antes de retornar.
3. Implementar startup orphan detection: cuando AnvilCraft arranca, checkear si hay `java.exe` con `parent_pid == <anvilcraft>` o `session.lock` huérfanos, y limpiarlos/avisar.

**Warnings de Java 22+ observados (no bloqueantes, pero sí ruido):**
```
[WARN]: java.lang.System::load has been called by com.sun.jna.Native ...
[WARN]: Use --enable-native-access=ALL-UNNAMED to avoid a warning
[WARN]: sun.misc.Unsafe::objectFieldOffset has been called by org.joml.MemUtil
```

Son restricciones de acceso nativo de Java 22+. No son la causa del error, pero en una versión futura de Java van a bloquear a `jna` y `joml`. Eventualmente Mojang actualizará esas libs.

**Lección:** Cuando un spawn de proceso es a largo plazo, la limpieza de Tauri/Rust debe ser **explícita y bloqueante** (wait con timeout), no implícita. `kill_on_drop` es el seguro mínimo, no la solución completa.

---

### [2026-08-04] Race condition — Start se habilita antes de que el install termine

**Síntoma:**
```
Missing required library: net/minecraftforge/forge/1.21.11-61.1.14/forge-1.21.11-61.1.14-server.jar
Exception in thread "main" java.lang.IllegalStateException: Missing required libraries! Check log
        at net.minecraftforge.bootstrap.shim.Main.main(Main.java:70)
```

Aparece cuando el usuario hace click en Start en una instancia recién creada, antes de que el loader termine de escribir el server jar. Típico de Forge 1.21.11, que tarda ~50s en instalar (descarga installer, libraries, binary patcher). El `forge-...-server.jar` es el ÚLTIMO archivo que escribe el installer. Si el usuario clickea Start antes, el shim arranca con libraries incompletas.

**Causa raíz:** `create_instance` escribía `state: "Stopped"` en el JSON inicial, y la UI habilitaba el botón Start desde el momento cero. No había gating entre "install corriendo" y "install terminado".

**Fix en 2 capas (commits `ee6a282` + `36bab8e` + `22844c5`):**

1. Nuevo state `InstanceState::Installing` en el enum (`models.rs`). `create_instance` ahora escribe `Installing` en el JSON inicial. El background install task flipea a `Stopped` cuando termina (éxito o error), vía el helper `loaders::common::finalize_install(app, id, target_dir)`.
2. `start_instance` rechaza con error claro si el state persistido es `Installing` (defense-in-depth en backend).
3. Frontend: el botón Start en `InstanceDetail.svelte` se deshabilita con `disabled={state === "Installing"}` y muestra "Installing..." con spinner. El `toggleServer` handler también hace early-return.
4. Cada loader (los 12 con install real) llama a `finalize_install` **antes** de emitir el evento `install-progress` con `step: "Done"`, así el state flip y el "100% del modal" ocurren en el mismo instante — sin gap visible.

**Verificación manual:**
1. Crear una nueva instancia Forge desde la UI.
2. Verificar que el botón Start se deshabilita con spinner durante el install (~50s).
3. Al terminar, el botón vuelve a "Start Server" instantáneamente (sin gap visible entre el modal al 100% y el botón habilitado).

**Lección:** Cuando un install es async y el frontend depende del state del backend para habilitar acciones, el state tiene que ser preciso desde el primer momento. `Installing → Stopped` no es un detalle cosmético — es la única forma de evitar que el usuario dispare acciones sobre una instancia a medio instalar. Además, los eventos de "progress" del frontend (modal al 100%) deben emitirse en el mismo orden que el state real del backend, idealmente desde el mismo lugar.

### [2026-08-04] UI no refrescaba el state después del install

**Síntoma:** El backend flipeaba `instance.json` de `Installing` a `Stopped` cuando el install terminaba, pero la UI seguía mostrando el botón "Installing..." con spinner indefinidamente. Solo se actualizaba después de cerrar y reabrir la app, o navegar a otra vista.

**Causa raíz:** El spawn block de `create_instance` escribía el state en disco pero **nunca emitía el evento `instance-update`**. El listener de `+layout.svelte:95-109` (que llama `read_instances` y re-asigna `appState.selectedInstance`) ya existía, pero nadie lo disparaba al terminar el install.

**Fix (commit `36bab8e`, refinado en `22844c5`):** Cada loader llama a `finalize_install` (en `loaders/common.rs:212`) antes del emit `"Done"`. Ese helper hace:
```rust
update_instance_state(instances_dir, id, InstanceState::Stopped);
app.emit("instance-update", ());
```

El `+layout.svelte` escucha `instance-update`, re-fetchea, re-asigna el `selectedInstance` con la versión fresca, y el `$derived` en `InstanceDetail` re-evalúa → el condicional `{#if state === "Installing"}` se vuelve falso → el botón cambia a "Start Server".

**Lección:** Cada vez que el backend cambia algo que el frontend muestra, tiene que emitir un evento. No basta con escribir a disco y esperar a que el frontend pregunte (polling). El estado del frontend es eventualmente-consistente solo si hay un evento que dispare el refresh.

**Bug latente relacionado (no fixeado todavía):** El listener de `instance-update` se registra dentro de un `init()` async en `+layout.svelte:95-109`. Si un `instance-update` se emite antes de que `listen()` se complete (raro, solo si el install termina en los primeros ~100ms de abrir la app), se pierde. Fix recomendado: mover la suscripción a un `onMount` síncrono.

---

## 🟢 Cosas que pasan y se resuelven rápido

### `cargo tauri dev` falla: "vite no se reconoce"

**Síntoma:** `error: failed to run beforeDevCommand: "vite" no se reconoce...`

**Causa:** No instalaste `node_modules/`.

**Fix:** `npm install` en la raíz del proyecto.

---

### Git: "dubious ownership" en repo clonado

**Síntoma:** `fatal: detected dubious ownership in repository at '...'`

**Causa:** El repo fue clonado por otro usuario (o en otra máquina). Git 2.36+ tiene seguridad contra esto.

**Fix:** `git config --global --add safe.directory <ruta-del-repo>`

---

### Git push rechazado: "Protected branch update failed"

**Síntoma:** `remote: error: GH006: Protected branch update failed for refs/heads/main`

**Causa:** Branch protection rule en GitHub bloqueó el push (probablemente intentabas force-push).

**Fix:** Si necesitas force-push legitimamente, ve a Settings → Branches → desmarca "Do not allow force pushes" temporalmente, push, y vuelve a marcar.

---

### Git: "Author identity unknown"

**Síntoma:** `fatal: unable to auto-detect email address`

**Causa:** Primera vez usando git, sin config de identidad.

**Fix:**
```bash
git config --global user.name "Tu Nombre"
git config --global user.email "tu@email.com"
```

---

## 🔍 Cómo agregar una entrada nueva

Cuando te topes con un problema y lo resuelvas, agrégalo aquí con este formato:

```markdown
### [YYYY-MM-DD] Título corto del problema

**Síntoma:** qué se observa (mensaje de error, comportamiento raro, etc.)
**Causa raíz:** por qué pasa
**Fix:** cómo se resolvió
**Lección:** qué aprendimos para no repetirlo
```

Esto convierte la historia del proyecto en conocimiento reutilizable.
