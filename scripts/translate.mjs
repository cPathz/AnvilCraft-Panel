#!/usr/bin/env node
/**
 * translate.mjs — Genera traducciones base usando @parvineyvazov/json-translator
 *
 * Uso:
 *   npm run translate -- fr               # genera src/lib/locales/fr.json desde en.json
 *   npm run translate -- fr de ja zh      # genera varios idiomas en una corrida
 *   npm run translate -- --all            # genera idiomas comunes (es, fr, de, it, pt, ru, zh, ja, ko, ar)
 *   npm run translate -- --all --from es  # genera varios idiomas desde es.json (en vez de en.json)
 *   npm run translate -- fr --engine libre # usa LibreTranslate en vez de Google
 *   npm run translate -- fr --force       # sobreescribe si ya existe
 *
 * Notas:
 * - Las traducciones automáticas son del 60-80% de calidad. Un nativo debe revisarlas.
 * - Después de generar, abre el archivo y ajusta los términos técnicos del proyecto
 *   (ej. "instance", "loader", "modpack" — suelen quedarse sin traducir en algunos idiomas).
 */

import { spawn } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = join(__dirname, '..');
const LOCALES_DIR = join(PROJECT_ROOT, 'src', 'lib', 'locales');

const args = process.argv.slice(2);
let sourceFile = 'en.json';
let engines = ['google'];
let force = false;
const langs = [];

for (let i = 0; i < args.length; i++) {
  const a = args[i];
  if (a === '--from' || a === '-f') {
    sourceFile = args[++i];
  } else if (a === '--engine' || a === '-e') {
    engines = [args[++i]];
  } else if (a === '--force') {
    force = true;
  } else if (a === '--all') {
    langs.push('es', 'fr', 'de', 'it', 'pt', 'ru', 'zh-CN', 'ja', 'ko', 'ar', 'hi', 'tr', 'pl', 'nl');
  } else if (a === '--help' || a === '-h') {
    printHelp();
    process.exit(0);
  } else if (a.startsWith('--')) {
    console.error(`❌ Flag desconocida: ${a}`);
    printHelp();
    process.exit(1);
  } else {
    langs.push(a);
  }
}

if (langs.length === 0) {
  console.error('❌ No especificaste idiomas.');
  printHelp();
  process.exit(1);
}

const sourcePath = join(LOCALES_DIR, sourceFile);
if (!existsSync(sourcePath)) {
  console.error(`❌ No se encuentra el archivo fuente: ${sourcePath}`);
  process.exit(1);
}

// Verificar que el JSON fuente es válido y tiene la misma estructura que en.json de referencia
const sourceContent = readFileSync(sourcePath, 'utf-8');
let sourceObj;
try {
  sourceObj = JSON.parse(sourceContent);
} catch (e) {
  console.error(`❌ El archivo fuente no es JSON válido: ${e.message}`);
  process.exit(1);
}

const refPath = join(LOCALES_DIR, 'en.json');
if (existsSync(refPath)) {
  const refKeys = flattenKeys(JSON.parse(readFileSync(refPath, 'utf-8')));
  const sourceKeys = flattenKeys(sourceObj);
  const missing = refKeys.filter((k) => !sourceKeys.includes(k));
  const extra = sourceKeys.filter((k) => !refKeys.includes(k));
  if (missing.length || extra.length) {
    console.warn(`⚠️  ${sourceFile} no coincide con en.json:`);
    if (missing.length) console.warn(`   Faltan ${missing.length} claves. Ej: ${missing.slice(0, 3).join(', ')}`);
    if (extra.length) console.warn(`   Sobran ${extra.length} claves. Ej: ${extra.slice(0, 3).join(', ')}`);
  }
}

console.log(`📂 Fuente: ${sourceFile}`);
console.log(`🌐 Idiomas a generar: ${langs.join(', ')}`);
console.log(`🔧 Motor: ${engines[0]}\n`);

for (const lang of langs) {
  const targetPath = join(LOCALES_DIR, `${lang}.json`);
  if (existsSync(targetPath) && !force) {
    console.log(`⏭️  ${lang}.json ya existe, saltando (usa --force para sobreescribir)`);
    continue;
  }

  console.log(`⏳ Generando ${lang}.json...`);
  const ok = await runTranslator({
    source: sourcePath,
    target: targetPath,
    from: sourceFile.replace('.json', ''),
    to: lang,
    engine: engines[0],
  });

  if (ok) {
    const stat = readFileSync(targetPath, 'utf-8');
    const lines = stat.split('\n').length;
    console.log(`✅ ${lang}.json creado (${lines} líneas, ${stat.length} bytes)`);
  } else {
    console.error(`❌ Falló ${lang}.json`);
  }
}

console.log(`\n🎉 Listo. Próximos pasos:`);
console.log(`   1. Abre cada .json generado y revisa la calidad.`);
console.log(`   2. Ajusta términos técnicos del proyecto (instance, loader, modpack, addon).`);
console.log(`   3. Corre: npm run translate:validate  (para verificar que no falten claves)`);
console.log(`   4. Commit: git add src/lib/locales/ && git commit -m "i18n: add ${langs.join(', ')} translations"`);

function printHelp() {
  console.log(`Uso: npm run translate -- [opciones] <idioma> [idioma...]

Opciones:
  -f, --from <archivo>     Archivo fuente (default: en.json)
  -e, --engine <motor>     Motor de traducción: google, bing, libre, deepl, gpt (default: google)
  --all                    Genera idiomas comunes: es, fr, de, it, pt, ru, zh-CN, ja, ko, ar, hi, tr, pl, nl
  --force                  Sobreescribe si el archivo ya existe
  -h, --help               Muestra esta ayuda

Idiomas: códigos BCP-47 / ISO 639-1 (en, es, fr, de, ja, zh-CN, pt-BR, etc.)

Ejemplos:
  npm run translate -- fr
  npm run translate -- fr de ja zh-CN
  npm run translate -- --all
  npm run translate -- pt-BR --engine libre
  npm run translate -- fr --force
`);
}

function runTranslator({ source, target, from, to, engine }) {
  return new Promise((resolve) => {
    // Usamos la CLI jsontt del paquete @parvineyvazov/json-translator
    const cli = 'npx';
    const cliArgs = ['--yes', 'jsontt', source, '--module', engine, '--from', from, '--to', to];
    if (existsSync(target)) {
      // --name hace que el output se llame igual al target
      cliArgs.push('--name', target.replace('.json', ''));
    }

    const proc = spawn(cli, cliArgs, { cwd: PROJECT_ROOT, stdio: 'inherit', shell: true });
    proc.on('exit', (code) => resolve(code === 0));
    proc.on('error', () => resolve(false));
  });
}

function flattenKeys(obj, prefix = '', out = []) {
  if (obj && typeof obj === 'object' && !Array.isArray(obj)) {
    for (const [k, v] of Object.entries(obj)) {
      const path = prefix ? `${prefix}.${k}` : k;
      if (v && typeof v === 'object' && !Array.isArray(v)) {
        flattenKeys(v, path, out);
      } else {
        out.push(path);
      }
    }
  }
  return out;
}
