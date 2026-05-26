# Dev Log - AnvilCraft Panel

Bitácora de desarrollo en formato machine-readable. Solo hechos concretos, fechas, cambios implementados.

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
