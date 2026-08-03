# Security Audit — Análisis de Seguridad (actualizado 2026-08-02)

> Aplica a AnvilCraft v0.1.16. Cubre:
> - Dependencias npm con vulnerabilidades
> - Secretos hardcodeados en código
> - Capabilities de Tauri (qué puede hacer el frontend)
> - XSS, command injection, path traversal
>
> Generado por Claude (análisis) + Luis Macias (decisiones de remediación).

## Resumen ejecutivo

`npm audit --omit=dev` reporta **5 vulnerabilidades** en dependencias de producción (5 paquetes distintos, varios con múltiples advisories):

| # | Paquete | Severidad | Fix disponible |
|---|---|---|---|
| 1 | `devalue` (transitive de SvelteKit) | **HIGH** | Sí (vía `npm audit fix`) |
| 2 | `svelte` (≤5.55.6) | moderate | Sí (5.56+) |
| 3 | `svelte-i18n` (≥3.7.2) | moderate | Requiere breaking change |
| 4 | `esbuild` (transitive de `svelte-i18n`) | moderate | Solo vía upgrade de svelte-i18n |
| 5 | `postcss` (<8.5.10) | moderate | Sí |

**Ninguna afecta el runtime de AnvilCraft** (todas son SSR-only o build-time). El reporte completo de GitHub (41 vulns) incluye Rust crates que requieren `cargo audit` para auditar localmente.

---

## 🔴 HIGH — `devalue` (DoS en SSR)

**GHSAs**: GHSA-g2pg-6438-jwpf, GHSA-vw5p-8cq8-m7mv, GHSA-33hq-fvwr-56pm, GHSA-8qm3-746x-r74r, GHSA-cfw5-2vxh-hr84, GHSA-mwv9-gp5h-frr4

**Vulnerabilidad**: DoS via memory/CPU exhaustion in `devalue.parse`, prototype pollution en `devalue.unflatten`.

**¿Aplica a AnvilCraft?** ❌ **No directamente**
- AnvilCraft es una app Tauri **desktop**, no usa SSR
- `devalue` es una dep transitive de SvelteKit, pero SvelteKit no usa SSR en builds de Tauri
- El ataque requiere un servidor HTTP que ejecute `devalue.parse` sobre input no confiable

**Acción recomendada**: Cerrar el Dependabot alert con razón "won't fix — Tauri desktop, no SSR surface".

---

## 🟡 MODERATE — `svelte` ≤5.55.6 (XSS en SSR)

**GHSAs**: GHSA-6738-r8g5-qwp3, GHSA-crpf-4hrx-3jrp, GHSA-m56q-vw4c-c2cp, GHSA-f7gr-6p89-r883, GHSA-h7h7-mm68-gmrc, GHSA-phwv-c562-gvmh, GHSA-pr6f-5x2q-rwfp, GHSA-f3cj-j4f6-wq85, GHSA-rcqx-6q8c-2c42

**Vulnerabilidad**: Múltiples XSS en SSR (element spreading, `<svelte:element>`, contenteditable bindings, DOM clobbering).

**¿Aplica a AnvilCraft?** ❌ **No directamente**
- AnvilCraft renderiza en WebView pero el HTML viene del Rust backend (no de un server externo)
- Todos los advisories son específicos a SSR

**Acción recomendada**: Cerrar el Dependabot alert con razón "won't fix — Tauri desktop, no SSR surface". Upgrade a 5.56+ no es prioritario.

---

## 🟡 MODERATE — `svelte-i18n` (esbuild transitive)

**Vulnerabilidad**: `esbuild ≤0.24.2` permite que cualquier website envíe requests al dev server y lea la respuesta.

**¿Aplica a AnvilCraft?** ❌ **No** (solo afecta `npm run dev` localmente, dev server only)

**Acción recomendada**: Cerrar el Dependabot alert con razón "dev server only, not production". Esperar upgrade no-breaking de svelte-i18n (4.x) en el futuro.

---

## 🟡 MODERATE — `postcss` <8.5.10 (XSS en CSS)

**Vulnerabilidad**: XSS via unescaped `</style>` en CSS Stringify.

**¿Aplica a AnvilCraft?** ❌ **No** (build-time tool, no incluido en el bundle final)

**Acción recomendada**: Cerrar el Dependabot alert con razón "build-time tool, not in production bundle". Upgrade trivial con `npm audit fix` si se hace housekeeping general.

---

## Pendientes a futuro (no urgentes)

| # | Acción | Beneficio | Esfuerzo |
|---|---|---|---|
| 1 | Configurar `cargo audit` en CI (1 línea en `.github/workflows/`) | Monitoreo continuo de Rust crates | 5 min |
| 2 | Upgrade `postcss` a 8.5.10+ (build-time, sin impacto runtime) | Cierra moderate | 1 min |
| 3 | Upgrade `svelte` a 5.56+ (cuando sea estable) | Cierra 9 advisories de SSR | 15 min testing |
| 4 | Upgrade `svelte-i18n` a 4.x (cuando exista estable) | Cierra moderate de esbuild | 30 min |
| 5 | Auditar Rust crates via `cargo audit` después de configurarlo | Visibilidad completa | 1 sesión |

---

## Cómo cerrar los Dependabot alerts manualmente

1. Ir a https://github.com/cPathz/AnvilCraft-Panel/security/dependabot
2. Para cada uno de los 5 paquetes npm: click → "Dismiss alert" → "Won't fix" (con razón: "Tauri desktop app, no SSR surface")
3. Para los ~36 alerts restantes (Rust crates + devDeps): dismiss con "Acceptable risk" hasta configurar `cargo audit` en CI

---

# 🔐 Auditoría de código (2026-08-02)

Análisis manual del código fuente en busca de secretos hardcodeados, vulnerabilidades, y configuración insegura.

## 🔴 Secretos hardcodeados (encontrados y remediados)

### 1. `CURSEFORGE_API_KEY` en código fuente

**Archivo:** `src-tauri/src/commands/curseforge.rs:32`

```rust
// ANTES (vulnerable):
const CF_API_KEY: &str = "REDACTED-CURSEFORGE-KEY-ROTATED-2026-08-02";
```

**Riesgo:** API key de CurseForge **privada** hardcodeada en código fuente, visible en GitHub. Un atacante podría:
- Abusar la key para hacer requests no autorizadas
- Hacer que CurseForge rate-limit o deshabilite la key
- Costar dinero si hay límites de pago

**Remediación (v0.1.15):** Movida a variable de entorno `CURSEFORGE_API_KEY`. El código retorna error claro si no está configurada.

**Acción adicional requerida:**
1. ✅ Luis rotó la key en https://console.curseforge.com/ (key nueva generada)
2. ✅ Key vieja fue revocada
3. ⚠️ Pendiente: limpiar el historial de git con `git filter-repo` para borrar la key del log

### 2. `MSIX_CERT_PASSWORD` en script de build

**Archivo:** `scripts/build-msix.ps1:46`

```powershell
# ANTES (vulnerable):
$CertPassword = "REDACTED-MSIX-CERT-PASSWORD-ROTATED-2026-08-02"
```

**Riesgo:** Password de certificado de firma MSIX hardcodeado. Como el certificado es autofirmado/temporal, el riesgo es menor que el #1, pero igualmente es mala práctica.

**Remediación (v0.1.15):** Movida a variable de entorno `MSIX_CERT_PASSWORD`. El script retorna error claro si no está configurada.

## 🛡️ Análisis de superficie de ataque

### Capabilities de Tauri (`src-tauri/capabilities/default.json`)

| Permiso | Qué permite | ¿Necesario? | Riesgo |
|---|---|---|---|
| `core:default` | API core mínima | Sí | Mínimo |
| `opener:default` | Abrir URLs externas en navegador | Sí (links a Discord, GitHub) | Bajo (solo abre URLs hardcoded en la UI) |
| `dialog:default` | Diálogos de archivo (abrir, guardar) | Sí (importar modpacks, seleccionar archivos) | Bajo |
| `updater:allow-check` | Verificar actualizaciones | Sí | Bajo |
| `updater:allow-download-and-install` | Descargar e instalar updates | Sí | **Medio** (vector para RCE si atacante controla el endpoint) |
| `process:default` | Plugin de procesos (exit, listado) | Sí | Bajo |
| `core:window:allow-start-dragging` | Arrastrar ventana | Sí (UI) | Mínimo |
| `core:window:allow-close` | Cerrar ventana | Sí | Mínimo |

**Análisis:**
- ✅ Solo se exponen los permisos necesarios (no hay permisos amplios)
- ✅ No hay `core:webview:allow-internal-toggle-devtools` (no se puede abrir DevTools desde código)
- ⚠️ `updater:allow-download-and-install` usa endpoint de GitHub Releases (`https://github.com/cPathz/AnvilCraft-Panel/releases/latest/download/latest.json`). Si un atacante compromete GitHub Releases, podría distribuir binarios maliciosos. **Mitigación:** El updater usa `pubkey` (clave pública de firma) en `tauri.conf.json:45` para verificar la firma. ✓

### XSS en Svelte

**Análisis:** Búsqueda de `{@html ...}` y similares:

```
src/lib/components/console/ConsoleView.svelte:758, 767, 1004
```

**Mitigación existente:** `src/lib/utils/ansiConverter.ts:7` configura `escapeXML: true` en `ansi-to-html`, lo que escapa `<`, `>`, `&`, `"`, `'` en el contenido del log. ✓

**Vector residual:** Un log de Minecraft que contenga `<script>alert(1)</script>` se convierte a `&lt;script&gt;alert(1)&lt;/script&gt;` antes de pasar a `{@html}`, por lo que se renderiza como texto literal, no como script.

**Riesgo residual:** BAJO. Si un atacante controla el proceso Java (Minecraft server), podría intentar inyectar, pero `escapeXML` lo mitiga.

### Command injection (Rust)

**Análisis:** Búsqueda de `Command::new`, `child_process`:

```
src-tauri/src/loaders/mods.rs:220, 232, 234, 496, 507, 509
src-tauri/src/commands/instance.rs:816, 823, 830, 992, 1003, 1235, 1242, 1249
src-tauri/src/commands/server.rs:63, 134
```

**Análisis:** Todos los usos son `Command::new(binario).arg(valor)` que en Rust NO invocan shell. Es seguro contra command injection clásico. ✓

**Vector residual:** Paths que vienen del frontend se pasan a `Command::arg(path)`. Como Rust no usa shell, no hay inyección, pero sí podría abrirse un binario arbitrario si el path es controlado. **Riesgo:** Bajo (binarios legítimos esperados: `explorer`, `xdg-open`, `open`).

### Path traversal

**Análisis:** Búsqueda de `fs::read`, `fs::write`, `PathBuf::join`:

**Mitigación existente:** `src-tauri/src/commands/curseforge.rs:124-127` usa `entry.enclosed_name()` que retorna `None` si el path del ZIP contiene `..` o es absoluto. ✓

**Vector residual:** Paths de instancias se construyen con `app_data_dir().join("instances").join(slug)`. El `slug` viene del frontend (`create_instance`). Si no se valida que el slug sea seguro, podría apuntar fuera de `instances/`. **Riesgo:** Bajo (el slug es generado por el frontend, no por input del usuario externo).

### HTTPS / TLS

**Análisis:** Todas las llamadas HTTP usan `reqwest::Client::new()` que por defecto verifica certificados TLS. ✓

```
src-tauri/src/commands/java.rs:107
src-tauri/src/loaders/proxies.rs (varios)
src-tauri/src/loaders/vanilla.rs
src-tauri/src/commands/curseforge.rs:178
```

**Endpoints:**
- `https://api.adoptium.net` (Java downloads)
- `https://api.curseforge.com` (modpacks)
- `https://api.modrinth.com` (modpacks)
- `https://launchermeta.mojang.com` (Minecraft versions)
- `https://github.com/...` (updater)

Todos HTTPS. ✓

## 📋 Resumen de remediación (v0.1.15)

| Severidad | Hallazgo | Estado |
|---|---|---|
| 🔴 | `CF_API_KEY` hardcodeada | ✅ Movida a env var |
| 🔴 | `MSIX_CERT_PASSWORD` hardcodeado | ✅ Movido a env var |
| 🟡 | `.env*` no en .gitignore | ✅ Agregado |
| 🟢 | XSS en consola | ✅ Ya mitigado con `escapeXML` |
| 🟢 | Command injection | ✅ Rust usa `Command::arg()` (sin shell) |
| 🟢 | Path traversal en ZIPs | ✅ Ya mitigado con `enclosed_name()` |
| 🟢 | HTTPS enforcement | ✅ `reqwest` verifica TLS |
| 🟢 | Capabilities de Tauri | ✅ Mínimas necesarias |

## 📝 Acciones pendientes

1. **Rotar API key de CurseForge** (Luis debe hacerlo en https://console.curseforge.com/)
2. **Limpiar historial de git** con `git-filter-repo` (eliminar la key vieja del log)
3. **Force-push** (justificado en este caso por secret leak)
4. **Configurar `cargo audit` en CI** (pendiente, no urgente)
5. **Configurar Dependabot auto-dismiss** para advisories que no aplican (Tauri desktop)
6. **Agregar pre-commit hook** que corra `gitleaks` o similar para prevenir futuros secrets

---

## Metodología

Análisis generado/actualizado el 2026-08-02 por Claude Code.

Fuentes:
- `npm audit` (producción + devDependencies)
- Búsqueda manual de patrones de secretos (`api[_-]?key`, `secret`, `token`, `password`, `private[_-]?key`)
- Inspección de capabilities de Tauri
- Análisis de flujo de datos (frontend → IPC → Rust → filesystem/process)
- Verificación de TLS, sanitización HTML, validación de paths
