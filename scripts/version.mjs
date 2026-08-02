#!/usr/bin/env node
/**
 * version.mjs — Single source of truth para la versión del proyecto.
 *
 * Antes: había que actualizar 4+ archivos a mano (package.json, Cargo.toml,
 * tauri.conf.json, AppxManifest.xml, README.md badge, SECURITY_AUDIT.md) y
 * olvidar uno causaba bugs (versión visible ≠ versión real).
 *
 * Ahora: package.json es la única fuente de verdad. Este script propaga
 * cambios a todos los demás archivos automáticamente.
 *
 * Uso:
 *   npm run version                  # Muestra status de todos los archivos
 *   npm run version -- 0.1.15        # Cambia TODO a 0.1.15 (package.json + sync)
 *   npm run version -- patch         # Bump patch (0.1.14 → 0.1.15)
 *   npm run version -- minor         # Bump minor (0.1.14 → 0.2.0)
 *   npm run version -- major         # Bump major (0.1.14 → 1.0.0)
 *   npm run version:check            # Solo verifica sincronización (exit 1 si falla)
 *   npm run version:sync             # Sincroniza desde package.json (sin cambiar versión)
 *
 * Salida:
 *   - Muestra cada archivo y su versión actual
 *   - Marca con ⚠️ los que no coinciden con package.json
 *   - Si pasa una versión, actualiza todos y muestra diff
 */

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = join(__dirname, '..');

// ── Configuración ────────────────────────────────────────────────────────
const FILES = [
  {
    path: 'package.json',
    type: 'json',
    pattern: (v) => `"version": "${v}"`,
  },
  {
    path: 'src-tauri/Cargo.toml',
    type: 'toml',
    pattern: (v) => `version = "${v}"`,
  },
  {
    path: 'src-tauri/tauri.conf.json',
    type: 'json',
    pattern: (v) => `"version": "${v}"`,
  },
  {
    path: 'src-tauri/msix/AppxManifest.xml',
    type: 'xml',
    // MSIX usa formato 4-segmentos: 0.1.14 → 0.1.14.0
    pattern: (v) => `Version="${v}.0"`,
  },
  {
    path: 'README.md',
    type: 'badge',
    pattern: (v) => `Version-v${v}-blue`,
  },
  {
    path: 'docs/SECURITY_AUDIT.md',
    type: 'text',
    pattern: (v) => `Aplica a AnvilCraft v${v}`,
  },
];

// ── Utilidades ───────────────────────────────────────────────────────────

function readVersion(file) {
  const fullPath = join(PROJECT_ROOT, file.path);
  if (!existsSync(fullPath)) return null;

  const content = readFileSync(fullPath, 'utf-8');

  switch (file.type) {
    case 'json': {
      const data = JSON.parse(content);
      return data.version ?? null;
    }
    case 'toml': {
      const match = content.match(/^version\s*=\s*"([^"]+)"/m);
      return match ? match[1] : null;
    }
    case 'xml': {
      // MSIX usa formato 4-segmentos (0.1.14.0). Normalizamos a 3-segmentos
      // para comparar con la fuente de verdad.
      const match = content.match(/<Identity[^>]*Version="([^"]+)"/);
      if (!match) return null;
      const parts = match[1].split('.');
      return parts.slice(0, 3).join('.');
    }
    case 'badge': {
      const match = content.match(/Version-v(\d+\.\d+\.\d+)-/);
      return match ? match[1] : null;
    }
    case 'text': {
      const match = content.match(/Aplica a AnvilCraft (v?\d+\.\d+\.\d+)/);
      return match ? match[1].replace(/^v/, '') : null;
    }
  }
  return null;
}

function writeVersion(file, newVersion) {
  const fullPath = join(PROJECT_ROOT, file.path);
  let content = readFileSync(fullPath, 'utf-8');

  if (file.type === 'json') {
    // Usar regex para no romper formato (algunos JSONs tienen espacios o comentarios)
    content = content.replace(
      /"version"\s*:\s*"[^"]+"/,
      `"version": "${newVersion}"`
    );
  } else if (file.type === 'toml') {
    content = content.replace(
      /^version\s*=\s*"[^"]+"/m,
      `version = "${newVersion}"`
    );
  } else if (file.type === 'xml') {
    content = content.replace(
      /(<Identity[^>]*Version=")[^"]+(")/,
      `$1${newVersion}.0$2`
    );
  } else if (file.type === 'badge') {
    content = content.replace(
      /Version-v\d+\.\d+\.\d+-/,
      `Version-v${newVersion}-`
    );
  } else if (file.type === 'text') {
    content = content.replace(
      /Aplica a AnvilCraft v\d+\.\d+\.\d+/,
      `Aplica a AnvilCraft v${newVersion}`
    );
  }

  writeFileSync(fullPath, content, 'utf-8');
}

function isValidSemver(v) {
  return /^\d+\.\d+\.\d+(-[\w.]+)?$/.test(v);
}

function bumpVersion(current, type) {
  const [major, minor, patch] = current.split('.').map(Number);
  switch (type) {
    case 'major':
      return `${major + 1}.0.0`;
    case 'minor':
      return `${major}.${minor + 1}.0`;
    case 'patch':
      return `${major}.${minor}.${patch + 1}`;
    default:
      return null;
  }
}

function compareVersions(a, b) {
  return a === b;
}

function colorize(text, color) {
  // Códigos ANSI básicos
  const colors = {
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
    gray: '\x1b[90m',
    bold: '\x1b[1m',
    reset: '\x1b[0m',
  };
  return `${colors[color] ?? ''}${text}${colors.reset}`;
}

// ── Comandos ─────────────────────────────────────────────────────────────

function status() {
  console.log(colorize('\n📋 Estado de versión en todos los archivos:\n', 'bold'));

  const sourceVersion = readVersion(FILES[0]); // package.json
  let allMatch = true;
  const table = [];

  for (const file of FILES) {
    const current = readVersion(file);
    const isSource = file.path === 'package.json';
    const matches = current && compareVersions(current, sourceVersion);

    if (!isSource && !matches) allMatch = false;

    table.push({
      path: file.path,
      current,
      isSource,
      matches,
    });
  }

  for (const row of table) {
    const status = row.isSource
      ? colorize('SOURCE', 'cyan')
      : row.matches
        ? colorize('   ✓   ', 'green')
        : colorize('   ⚠️   ', 'yellow');
    const versionStr = row.current
      ? colorize(row.current.padEnd(10), row.matches || row.isSource ? 'green' : 'yellow')
      : colorize('NOT FOUND', 'red');
    const typeStr = colorize(`[${FILES.find((f) => f.path === row.path).type}]`, 'gray');
    console.log(`  ${status}  ${versionStr}  ${typeStr}  ${row.path}`);
  }

  console.log();

  if (allMatch) {
    console.log(colorize('✅ Todos los archivos están sincronizados.', 'green'));
  } else {
    console.log(colorize('⚠️  Algunos archivos no coinciden con package.json.', 'yellow'));
    console.log(colorize(`   Ejecuta: npm run version:sync`, 'gray'));
  }
  console.log();
}

function check() {
  const sourceVersion = readVersion(FILES[0]);
  let hasError = false;

  for (const file of FILES) {
    if (file.path === 'package.json') continue;
    const current = readVersion(file);
    if (!current) {
      console.error(`❌ No se pudo leer versión de ${file.path}`);
      hasError = true;
      continue;
    }
    if (!compareVersions(current, sourceVersion)) {
      console.error(`❌ ${file.path}: ${current} (debería ser ${sourceVersion})`);
      hasError = true;
    }
  }

  if (hasError) {
    console.error('\n❌ Las versiones no están sincronizadas.');
    console.error('   Ejecuta: npm run version:sync\n');
    process.exit(1);
  } else {
    console.log(`✅ Todas las versiones coinciden con package.json (${sourceVersion}).`);
    process.exit(0);
  }
}

function sync() {
  const sourceVersion = readVersion(FILES[0]);
  console.log(colorize(`\n🔄 Sincronizando todo a v${sourceVersion}...\n`, 'cyan'));

  for (const file of FILES) {
    if (file.path === 'package.json') continue;
    const current = readVersion(file);
    if (current && compareVersions(current, sourceVersion)) {
      console.log(`  ${colorize('✓', 'green')} ${file.path} (ya en ${current})`);
    } else {
      writeVersion(file, sourceVersion);
      const newVal = readVersion(file);
      console.log(`  ${colorize('✎', 'blue')} ${file.path}: ${current ?? '?'} → ${newVal}`);
    }
  }

  console.log(colorize('\n✅ Sincronización completa.\n', 'green'));
  console.log(colorize('Próximos pasos:', 'bold'));
  console.log(`  1. Revisa los cambios: ${colorize('git diff', 'cyan')}`);
  console.log(`  2. Commit: ${colorize(`git commit -am "chore: bump version to ${sourceVersion}"`, 'cyan')}`);
  console.log(`  3. (Opcional) Tag: ${colorize(`git tag v${sourceVersion}`, 'cyan')}\n`);
}

function set(newVersion) {
  // Acepta 'patch', 'minor', 'major' o semver explícito
  let targetVersion = newVersion;
  if (['patch', 'minor', 'major'].includes(newVersion)) {
    const current = readVersion(FILES[0]);
    targetVersion = bumpVersion(current, newVersion);
    console.log(colorize(`\n🔼 Bump ${newVersion}: ${current} → ${targetVersion}\n`, 'cyan'));
  } else if (!isValidSemver(newVersion)) {
    console.error(`❌ Versión inválida: "${newVersion}"`);
    console.error('   Usa formato semver (ej. 0.1.15) o patch/minor/major');
    process.exit(1);
  }

  // 1. Actualizar package.json (fuente de verdad)
  const pkgPath = join(PROJECT_ROOT, 'package.json');
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf-8'));
  const oldVersion = pkg.version;
  pkg.version = targetVersion;

  // Mantener formato (2 espacios indentación como está ahora)
  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf-8');
  console.log(`  ${colorize('✎', 'blue')} package.json: ${oldVersion} → ${targetVersion}`);

  // 2. Sincronizar el resto
  for (const file of FILES) {
    if (file.path === 'package.json') continue;
    const current = readVersion(file);
    if (current) {
      writeVersion(file, targetVersion);
      const newVal = readVersion(file);
      console.log(`  ${colorize('✎', 'blue')} ${file.path}: ${current} → ${newVal}`);
    }
  }

  console.log(colorize(`\n✅ Versión actualizada a ${targetVersion} en todos los archivos.\n`, 'green'));
  console.log(colorize('Próximos pasos:', 'bold'));
  console.log(`  1. Actualiza CHANGELOG.md con los cambios de esta versión`);
  console.log(`  2. Revisa: ${colorize('git diff', 'cyan')}`);
  console.log(`  3. Commit: ${colorize(`git commit -am "chore: bump version to ${targetVersion}"`, 'cyan')}`);
  console.log(`  4. (Opcional) Tag: ${colorize(`git tag v${targetVersion}`, 'cyan')}\n`);
}

// ── Entry point ──────────────────────────────────────────────────────────

const args = process.argv.slice(2);

if (args.length === 0) {
  status();
} else if (args[0] === '--help' || args[0] === '-h') {
  console.log(`
${colorize('version.mjs', 'bold')} — Single source of truth para la versión

${colorize('Uso:', 'bold')}
  npm run version                  Muestra status actual
  npm run version -- <semver>       Cambia versión (ej. 0.1.15)
  npm run version -- patch          Bump patch (0.1.14 → 0.1.15)
  npm run version -- minor          Bump minor (0.1.14 → 0.2.0)
  npm run version -- major          Bump major (0.1.14 → 1.0.0)
  npm run version:check             Verifica sincronización (CI)
  npm run version:sync              Sincroniza desde package.json

${colorize('Fuente de verdad:', 'bold')} package.json
${colorize('Archivos sincronizados:', 'bold')}
  - src-tauri/Cargo.toml
  - src-tauri/tauri.conf.json
  - src-tauri/msix/AppxManifest.xml (formato 4-segmentos)
  - README.md (badge)
  - docs/SECURITY_AUDIT.md
`);
} else if (args[0] === '--check') {
  check();
} else if (args[0] === '--sync') {
  sync();
} else {
  set(args[0]);
}

// Exportar para uso programático (check, sync, status)
export { check, sync, status, set };
