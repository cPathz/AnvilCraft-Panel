# Versionado — Single source of truth

> **Una sola fuente de verdad, todos los archivos sincronizados.** Bumpear la versión es ahora un comando.

## 🎯 El problema (resuelto)

Antes había que actualizar la versión en **6 lugares** distintos:

1. `package.json`
2. `src-tauri/Cargo.toml`
3. `src-tauri/tauri.conf.json`
4. `src-tauri/msix/AppxManifest.xml` (formato 4 segmentos: `0.1.14.0`)
5. `README.md` (badge de shields.io)
6. `docs/SECURITY_AUDIT.md`

Olvidar uno causaba bugs visibles (binario decía v0.1.13 pero el README decía v0.1.2, etc.). **Ya no.**

## 📐 Arquitectura

```
package.json (FUENTE DE VERDAD)
        │
        │  npm run version:sync
        ▼
   ┌────────────────────────────────────────────┐
   │  scripts/version.mjs                       │
   │  Lee package.json, escribe en todos:       │
   │  - src-tauri/Cargo.toml                    │
   │  - src-tauri/tauri.conf.json               │
   │  - src-tauri/msix/AppxManifest.xml         │
   │  - README.md (badge)                       │
   │  - docs/SECURITY_AUDIT.md                  │
   └────────────────────────────────────────────┘
```

**¿Por qué `package.json` es la fuente?** Porque:
- Es el estándar de npm (todos los tools lo leen primero)
- Es lo que ven los contribuidores al abrir el repo
- Tauri también puede leerlo vía la config del build
- Es el único JSON fácil de parsear (vs TOML o XML)

## 🛠️ Comandos

### Ver el estado actual

```bash
npm run version
```

Salida:
```
📋 Estado de versión en todos los archivos:

  SOURCE  0.1.14     [json]    package.json
     ✓    0.1.14     [toml]    src-tauri/Cargo.toml
     ✓    0.1.14     [json]    src-tauri/tauri.conf.json
     ✓    0.1.14.0   [xml]     src-tauri/msix/AppxManifest.xml
     ✓    0.1.14     [badge]   README.md
     ✓    0.1.14     [text]    docs/SECURITY_AUDIT.md

✅ Todos los archivos están sincronizados.
```

### Cambiar la versión (recomendado)

```bash
# Bumpear patch (0.1.14 → 0.1.15)
npm run version -- patch

# Bumpear minor (0.1.14 → 0.2.0)
npm run version -- minor

# Bumpear major (0.1.14 → 1.0.0)
npm run version -- major

# O versión específica
npm run version -- 0.2.0
```

Esto:
1. Actualiza `package.json` con la nueva versión
2. Propaga automáticamente a los otros 5 archivos
3. Muestra diff línea por línea
4. Te dice qué hacer después (commit, tag, changelog)

### Sincronizar sin cambiar (cuando editaste algo a mano)

```bash
npm run version:sync
```

Lee la versión actual de `package.json` y la escribe en todos los demás. Útil si:
- Editaste un archivo a mano por error
- Otro colaborador olvidó correr el script
- Estás en una rama con versiones divergentes

### Verificar sin modificar (CI / pre-commit)

```bash
npm run version:check
```

Sale con código 0 si todo está sincronizado, código 1 si no. Útil para:
- GitHub Actions (bloquea PRs que rompan la sincronización)
- Pre-commit hooks
- Scripts de release

## 🔄 Flujo de release completo

```bash
# 1. Bump de versión (hace todo automático)
npm run version -- patch    # o minor, major, o "0.1.15"

# 2. Actualizar CHANGELOG.md con los cambios
# (esto todavía es manual — ninguna herramienta lo hace bien)

# 3. Verificar que todo compile
npm run check
cargo check --manifest-path src-tauri/Cargo.toml

# 4. Stage + commit
git add -A
git commit -m "chore: bump version to 0.1.15"

# 5. Crear tag (esto dispara el release workflow)
git tag v0.1.15
git push origin main --follow-tags
```

## 🤖 Integración con CI (recomendado)

Agregar un GitHub Action que falle si las versiones no están sincronizadas:

```yaml
# .github/workflows/version-check.yml
name: Version sync check
on:
  pull_request:
    paths:
      - 'package.json'
      - 'src-tauri/Cargo.toml'
      - 'src-tauri/tauri.conf.json'
      - 'src-tauri/msix/AppxManifest.xml'
      - 'README.md'
      - 'docs/SECURITY_AUDIT.md'
jobs:
  check:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm install
      - run: npm run version:check
```

Esto evita que un PR rompa la sincronización.

## 🧩 Detalles técnicos

### Formatos especiales

- **`msix/AppxManifest.xml`** usa formato de 4 segmentos: `0.1.14` → `0.1.14.0`. El script agrega automáticamente el `.0` final.
- **`README.md` badge** usa formato shields.io: `Version-v0.1.14-blue`. El script reemplaza solo el número.
- **`Cargo.toml`** y **`tauri.conf.json`** usan 3 segmentos (semver estándar).

### ¿Qué pasa con el `Cargo.lock`?

**No se toca.** El `Cargo.lock` se regenera automáticamente con `cargo build` o `cargo update`. La versión del proyecto en sí está solo en `Cargo.toml`; el `Cargo.lock` también incluye versiones de TODAS las dependencias, así que tiene su propia lógica.

### ¿Y el CHANGELOG.md?

**No se automatiza.** Aunque técnicamente se podría, ninguna herramienta genera un changelog decente. La convención es:
1. Bump de versión con `npm run version`
2. Editar `CHANGELOG.md` a mano con los cambios
3. Commitear ambos en el mismo PR

### ¿Por qué no usar `npm version` directamente?

`npm version patch` solo actualiza `package.json` y crea un tag de git. No propaga a `Cargo.toml` ni a los otros archivos. Por eso existe este script — es un wrapper que hace TODO.

## ❓ Troubleshooting

### "El script dice que no puede leer un archivo"

Probablemente el archivo no existe o tiene un formato inesperado. Verifica:
```bash
ls -la src-tauri/Cargo.toml
cat src-tauri/Cargo.toml | head -5
```

### "Cambié la versión pero el binario sigue mostrando la vieja"

El binario se reconstruye con `cargo tauri build`. La versión del binario viene de `tauri.conf.json`, que ya queda actualizado. Solo necesitas re-compilar.

### "Olvidé correr el script y commiteé con versiones desincronizadas"

No pasa nada. Solo corre:
```bash
npm run version:sync
git add -A
git commit -m "chore: sync version across files"
```

## 📚 Referencias

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Tauri: app version](https://v2.tauri.app/reference/config/#appconfig)
- [MSIX Package Versioning](https://learn.microsoft.com/en-us/windows/msix/desktop/managing-your-package-identity)
- [shields.io Version Badge](https://shields.io/badges/version)
