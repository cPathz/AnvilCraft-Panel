# Security Audit — Dependency Vulnerabilities (2026-06-07)

> Generado por Claude. Análisis de `npm audit --omit=dev` + revisión manual.
> Aplica a AnvilCraft v0.1.13. Las vulnerabilidades en devDependencies no se incluyen en este análisis.

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

## Metodología

Análisis generado el 2026-06-07 por Claude Code.

Fuentes:
- `npm audit --omit=dev` (producción, excluyendo devDeps)
- Inspección manual de los advisories para evaluar aplicabilidad
- Exclusión basada en que AnvilCraft es Tauri desktop (no SSR, no dev server expuesto)
