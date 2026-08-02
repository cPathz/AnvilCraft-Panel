# Comandos del proyecto — Cheat sheet

> **Todos los comandos que necesitas para trabajar con AnvilCraft Panel.**
> Guarda este archivo en favoritos (Ctrl+D).

---

## 🚀 Setup inicial (una vez por máquina)

### Prerrequisitos
- **Node.js 20+** → https://nodejs.org/
- **Rust 1.70+** → instalar con [rustup](https://rustup.rs/)
- **MS C++ Build Tools** (solo Windows) → Visual Studio Installer → workload "Desarrollo de escritorio con C++"
- **Git** → https://git-scm.com/

### Clonar e instalar
```powershell
git clone https://github.com/cPathz/AnvilCraft-Panel.git
cd AnvilCraft-Panel
npm install
cargo install tauri-cli --version "^2.0"
```

---

## 🛠️ Desarrollo

| Comando | Qué hace |
|---|---|
| `npm run dev` | Inicia Vite dev server (frontend solo, sin Rust) |
| `cargo tauri dev` | Modo dev completo (frontend + Rust + ventana nativa) |
| `npm run build` | Build de producción del frontend |
| `cargo tauri build` | Build completo → genera `.msi` y `.exe` con auto-update |
| `npm run check` | Type-check con svelte-check |
| `npm run check:watch` | Type-check en watch mode |

---

## 🌐 Internacionalización (i18n)

| Comando | Qué hace |
|---|---|
| `npm run translate -- fr` | Genera `src/lib/locales/fr.json` desde en.json |
| `npm run translate -- fr de ja` | Genera varios idiomas en una corrida |
| `npm run translate -- --all` | Genera 14 idiomas comunes (es, fr, de, it, pt, ru, zh-CN, ja, ko, ar, hi, tr, pl, nl) |
| `npm run translate -- fr --engine libre` | Usa LibreTranslate en vez de Google |
| `npm run translate -- fr --force` | Sobreescribe si ya existe |
| `npm run translate -- --help` | Muestra todas las opciones |
| `npm run translate:validate` | Verifica que todos los locales tengan la misma estructura que en.json |

### Flujo: agregar un idioma nuevo

```powershell
# 1. Generar borrador automático (Google Translate)
npm run translate -- fr

# 2. Editar src/lib/locales/fr.json
#    Ajustar términos técnicos:
#    - "instance" (generalmente queda en inglés)
#    - "loader" (nunca se traduce)
#    - "modpack", "addon", "world" (generalmente en inglés)
#    - Minecraft, Java, JAR (NUNCA se traducen)

# 3. Validar estructura
npm run translate:validate

# 4. Registrar en src/lib/i18n.ts:
#    register('fr', () => import('./locales/fr.json'));
#    const supportedLocales = ['en', 'es', 'fr'];

# 5. Probar localmente
npm run dev
# Settings → Appearance → Idioma: Français

# 6. Commit + PR
git add src/lib/locales/fr.json src/lib/i18n.ts
git commit -m "i18n: add French translation"
git push origin main
```

> **Más detalles:** `docs/I18N.md`

---

## 📦 Versionado (single source of truth)

| Comando | Qué hace |
|---|---|
| `npm run version` | Muestra status de versión en los 6 archivos |
| `npm run version -- patch` | Bump patch (0.1.14 → 0.1.15) + sync |
| `npm run version -- minor` | Bump minor (0.1.14 → 0.2.0) + sync |
| `npm run version -- major` | Bump major (0.1.14 → 1.0.0) + sync |
| `npm run version -- 0.2.0` | Versión específica + sync |
| `npm run version:check` | Solo verifica sincronización (útil en CI) |
| `npm run version:sync` | Sincroniza desde package.json sin cambiar versión |

### Flujo: release de nueva versión

```powershell
# 1. Bump de versión (sincroniza 6 archivos)
npm run version -- patch

# 2. Actualizar CHANGELOG.md a mano con los cambios

# 3. Verificar todo
npm run check
npm run version:check
npm run translate:validate

# 4. Commit
git add -A
git commit -m "chore: bump version to 0.1.15"

# 5. Crear tag + push (esto dispara el release workflow)
git tag v0.1.15
git push origin main --follow-tags
```

> **Más detalles:** `docs/VERSIONING.md`

---

## 🔄 Git workflow

### Push normal
```powershell
git add -A
git commit -m "mensaje descriptivo"
git push origin main
```

### Si rechazan (branch protection, "Protected branch update failed")
```powershell
# El push fue rechazado porque main está protegido.
# Si necesitas forzar (raro), ve a GitHub → Settings → Branches
# y desmarca temporalmente "Do not allow force pushes".
# Después vuélvelo a marcar.

# Si solo necesitas sincronizarte con remote:
git pull --rebase origin main
git push origin main
```

### Limpiar working tree
```powershell
# Ver qué hay
git status

# Descartar cambios locales
git restore <archivo>
git clean -fd  # PELIGROSO: borra untracked files

# Reset completo al último commit (PELIGROSO)
git reset --hard HEAD
```

---

## 🐛 Troubleshooting común

### `cargo tauri dev` falla: "vite no se reconoce"
```powershell
# Falta instalar dependencias
npm install
```

### Git: "dubious ownership"
```powershell
git config --global --add safe.directory 'P:/Proyectos/AnvilCraft'
```

### Git push: "Author identity unknown"
```powershell
git config --global user.name "Tu Nombre"
git config --global user.email "tu@email.com"
```

### `node_modules` corruptos
```powershell
Remove-Item node_modules -Recurse -Force
Remove-Item package-lock.json
npm install
```

### `src-tauri/target` corrupto (Rust)
```powershell
Remove-Item src-tauri/target -Recurse -Force
cargo tauri dev  # recompila (~5-10 min la primera vez)
```

### `cargo` falla por linker en Windows
```
# Falta MS C++ Build Tools.
# Abrir Visual Studio Installer → Modify →
# Workload: "Desarrollo de escritorio con C++"
```

### `svelte-check` muestra error de `@ts-expect-error` obsoleto
```
# vite.config.js:4:1: Unused '@ts-expect-error' directive.
# Causa: actualizacion de TypeScript/types. El @ts-expect-error ya no es necesario.
# Fix: quitar la línea del @ts-expect-error en vite.config.js
```

### Vulnerabilidades npm
```powershell
# Ver
npm audit

# Arreglar automáticas (sin breaking changes)
npm audit fix

# Arreglar todas (puede romper cosas)
npm audit fix --force
```

> **Más problemas documentados:** `docs/TROUBLESHOOTING.md`

---

## 📂 Estructura clave del proyecto

```
AnvilCraft/
├── src/                        # Frontend (Svelte 5 + Runes)
│   ├── lib/locales/            # Traducciones i18n (en.json, es.json, ...)
│   ├── lib/components/         # Componentes UI (views, modals, console, settings)
│   ├── lib/runes/              # Estado global con Svelte 5 Runes
│   └── lib/i18n.ts             # Setup de svelte-i18n
│
├── src-tauri/                  # Backend (Rust)
│   ├── src/loaders/            # 16 mod loaders (1 Vanilla implementado, 15 stubs)
│   ├── src/commands/           # 35+ comandos Tauri expuestos al frontend
│   ├── src/parser/             # Parsing de logs de Minecraft
│   ├── src/lib.rs              # Builder + invoke_handler
│   └── tauri.conf.json         # Config Tauri
│
├── scripts/                    # Scripts de tooling (Node.js ESM)
│   ├── version.mjs             # Single source of truth de versión
│   ├── translate.mjs           # Genera traducciones base
│   └── validate-locales.mjs    # Valida estructura de locales
│
├── docs/                       # Documentación pública
│   ├── ARCHITECTURE.md         # Mapa del proyecto
│   ├── I18N.md                 # Guía de internacionalización
│   ├── VERSIONING.md           # Sistema de versionado
│   ├── TROUBLESHOOTING.md      # Log de problemas conocidos
│   ├── COMMANDS.md             # ← este archivo
│   ├── SECURITY_AUDIT.md       # Auditoría de deps
│   ├── ROADMAP_STORE.md        # Roadmap de Microsoft Store
│   └── CONSOLE_PERFORMANCE.md  # Optimizaciones de consola
│
├── .github/                    # GitHub-specific
│   ├── workflows/              # CI/CD (release.yml, etc.)
│   ├── ISSUE_TEMPLATE/         # Templates de issues
│   └── PULL_REQUEST_TEMPLATE.md
│
├── CONTRIBUTING.md             # Guía de contribuidores
├── CHANGELOG.md                # Historial de versiones
├── README.md                   # Readme principal (GitHub)
└── package.json                # Dependencias + scripts npm
```

> **Detalles:** `docs/ARCHITECTURE.md`

---

## 🎯 Comandos rápidos — TL;DR

```powershell
# Setup
npm install

# Dev
cargo tauri dev

# Type-check
npm run check

# Status
npm run version
npm run translate:validate

# Bump de versión
npm run version -- patch

# Generar idioma
npm run translate -- fr

# Commit
git add -A
git commit -m "..."
git push origin main
```

---

## 🔗 Referencias

- **Repo:** https://github.com/cPathz/AnvilCraft-Panel
- **Tauri docs:** https://v2.tauri.app/
- **SvelteKit docs:** https://kit.svelte.dev/
- **svelte-i18n:** https://github.com/svelte-i18n/svelte-i18n
- **json-translator:** https://github.com/mololab/json-translator
- **Semantic Versioning:** https://semver.org/
- **shields.io (badges):** https://shields.io/
