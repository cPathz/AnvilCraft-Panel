#!/usr/bin/env node
/**
 * validate-locales.mjs — Verifica que todos los archivos en src/lib/locales/
 * tengan la misma estructura de claves que en.json, y que el JSON sea válido.
 *
 * Uso:
 *   npm run translate:validate
 *
 * Salida:
 *   - Lista archivos OK y archivos con problemas
 *   - Para cada problema: claves faltantes y claves sobrantes
 *   - Exit code 0 si todo OK, 1 si hay problemas
 *
 * Por qué es importante:
 *   svelte-i18n hace fallback al locale 'en' cuando una clave no existe.
 *   Eso significa que si traduces "save" en fr.json pero olvidas "cancel",
 *   el usuario francés verá "Save" en lugar de "Annuler". Este script
 *   previene ese problema en CI / antes de commit.
 */

import { readdirSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const LOCALES_DIR = join(__dirname, '..', 'src', 'lib', 'locales');

const REFERENCE = 'en.json';
const refPath = join(LOCALES_DIR, REFERENCE);

let refData;
try {
  refData = JSON.parse(readFileSync(refPath, 'utf-8'));
} catch (e) {
  console.error(`❌ No se pudo leer ${REFERENCE}: ${e.message}`);
  process.exit(1);
}

const refKeys = flattenKeys(refData);
console.log(`📂 ${REFERENCE} tiene ${refKeys.length} claves\n`);

const files = readdirSync(LOCALES_DIR).filter((f) => f.endsWith('.json'));

let hasErrors = false;

for (const file of files) {
  const path = join(LOCALES_DIR, file);
  let data;
  try {
    data = JSON.parse(readFileSync(path, 'utf-8'));
  } catch (e) {
    console.error(`❌ ${file}: JSON inválido — ${e.message}`);
    hasErrors = true;
    continue;
  }

  const keys = flattenKeys(data);
  const missing = refKeys.filter((k) => !keys.includes(k));
  const extra = keys.filter((k) => !refKeys.includes(k));

  if (file === REFERENCE) {
    console.log(`✓ ${file} (referencia, ${keys.length} claves)`);
    continue;
  }

  // Detectar traducciones vacías o que son solo el key
  const emptyValues = findEmptyValues(data);

  if (missing.length === 0 && extra.length === 0 && emptyValues.length === 0) {
    console.log(`✓ ${file} (${keys.length} claves, completo)`);
  } else {
    hasErrors = true;
    console.log(`⚠️  ${file}:`);
    if (missing.length) {
      console.log(`   Faltan ${missing.length} claves:`);
      missing.slice(0, 5).forEach((k) => console.log(`     - ${k}`));
      if (missing.length > 5) console.log(`     ... y ${missing.length - 5} más`);
    }
    if (extra.length) {
      console.log(`   Sobran ${extra.length} claves:`);
      extra.slice(0, 5).forEach((k) => console.log(`     + ${k}`));
      if (extra.length > 5) console.log(`     ... y ${extra.length - 5} más`);
    }
    if (emptyValues.length) {
      console.log(`   Valores vacíos o sin traducir (${emptyValues.length}):`);
      emptyValues.slice(0, 5).forEach((k) => console.log(`     = ${k}`));
      if (emptyValues.length > 5) console.log(`     ... y ${emptyValues.length - 5} más`);
    }
  }
}

console.log();
if (hasErrors) {
  console.log('❌ Hay problemas. Arregla los archivos antes de hacer commit.');
  process.exit(1);
} else {
  console.log('✅ Todos los locales están completos y consistentes.');
  process.exit(0);
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

function findEmptyValues(obj, prefix = '', out = []) {
  if (obj && typeof obj === 'object' && !Array.isArray(obj)) {
    for (const [k, v] of Object.entries(obj)) {
      const path = prefix ? `${prefix}.${k}` : k;
      if (typeof v === 'string') {
        if (v.trim() === '') {
          out.push(`${path} (vacío)`);
        } else if (v === k) {
          out.push(`${path} (valor = clave, no traducido)`);
        }
      } else if (v && typeof v === 'object' && !Array.isArray(v)) {
        findEmptyValues(v, path, out);
      }
    }
  }
  return out;
}
