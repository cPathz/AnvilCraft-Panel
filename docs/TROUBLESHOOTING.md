# Troubleshooting — Problemas conocidos y soluciones

> **Log de problemas** que han surgido en el proyecto. Cuando algo se arregla, se documenta aquí para no repetir el error.

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

**Causa raíz:** La versión se define en 3 lugares (package.json, Cargo.toml, tauri.conf.json) y se hardcodea en el README. Al bumpear, se actualiza uno y se olvidan los otros.

**Fix:** Pendiente. Estrategia: usar un script o GitHub Action que sincronice las versiones automáticamente. Mientras tanto, al bumpear, actualizar los 3 archivos en el mismo commit.

**Archivos a tocar:**
- `package.json` → `"version": "X.Y.Z"`
- `src-tauri/Cargo.toml` → `version = "X.Y.Z"`
- `src-tauri/tauri.conf.json` → `"version": "X.Y.Z"`
- `README.md` → badge `![Version-vX.Y.Z]`

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
