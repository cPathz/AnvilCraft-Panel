// Test file para validar parseMinecraftLog
// Importaría: import { parseMinecraftLog } from '../src/lib/utils/logParser';

// Simulación de la función para pruebas
interface LogEntry {
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
  source: string | null;
  message: string;
}

const ANSI_REGEX = /\x1B\[[0-9;]*m/g;
const LOG_PATTERN_STANDARD = /^\[([^\]]+)\]\s*\[([^\]\/]+)\/([A-Z]+)\]:\s*(.*)$/;
const LOG_PATTERN_SHORT = /^\[([^\]]+)\s+([A-Z]+)\]:\s*(.*)$/;
const SOURCE_PATTERN = /^\s*\[([^\]]+)\]\s*(.*)/;

const VALID_LEVELS = new Set(['INFO', 'WARN', 'ERROR', 'DEBUG']);

function parseMinecraftLog(rawLog: string): LogEntry {
  const cleanLog = rawLog.replace(ANSI_REGEX, '');

  let match = cleanLog.match(LOG_PATTERN_STANDARD);
  if (match) {
    const [, timestamp, , rawLevel, message] = match;
    const level = (VALID_LEVELS.has(rawLevel) ? rawLevel : 'INFO') as 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
    const { source, cleanMessage } = extractSource(message);
    return { timestamp, level, source, message: cleanMessage };
  }

  match = cleanLog.match(LOG_PATTERN_SHORT);
  if (match) {
    const [, timestamp, rawLevel, message] = match;
    const level = (VALID_LEVELS.has(rawLevel) ? rawLevel : 'INFO') as 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
    const { source, cleanMessage } = extractSource(message);
    return { timestamp, level, source, message: cleanMessage };
  }

  const { source, cleanMessage } = extractSource(cleanLog);
  return {
    timestamp: new Date().toLocaleTimeString('es-ES'),
    level: 'INFO',
    source,
    message: cleanMessage.trim()
  };
}

function extractSource(message: string): { source: string | null; cleanMessage: string } {
  const match = message.match(SOURCE_PATTERN);
  if (match) {
    return { source: match[1], cleanMessage: match[2] };
  }
  return { source: null, cleanMessage: message };
}

// ===== PRUEBAS =====
console.log('=== Test parseMinecraftLog ===\n');

// Test 1: Log con [Shop]
const test1 = '[14:32:50] [Server thread/INFO]: [Shop] Player compró diamante';
const result1 = parseMinecraftLog(test1);
console.log('Test 1 - [Shop]:');
console.log('Input:', test1);
console.log('Output:', result1);
console.log('✓ PASS:', result1.source === 'Shop' && result1.level === 'INFO');
console.log();

// Test 2: Log con [Backuper] - WARN
const test2 = '[14:32:51] [Server thread/WARN]: [Backuper] Iniciando backup del servidor';
const result2 = parseMinecraftLog(test2);
console.log('Test 2 - [Backuper] WARN:');
console.log('Input:', test2);
console.log('Output:', result2);
console.log('✓ PASS:', result2.source === 'Backuper' && result2.level === 'WARN');
console.log();

// Test 3: Log sin emisor
const test3 = '[14:32:52] [Server thread/INFO]: Server started';
const result3 = parseMinecraftLog(test3);
console.log('Test 3 - Sin emisor:');
console.log('Input:', test3);
console.log('Output:', result3);
console.log('✓ PASS:', result3.source === null && result3.level === 'INFO');
console.log();

// Test 4: Log con ANSI (simulado)
const test4 = '\x1B[97m[14:32:53]\x1B[0m [Server thread/ERROR]: [PurpurExtras] Error en configuración';
const result4 = parseMinecraftLog(test4);
console.log('Test 4 - Con ANSI + [PurpurExtras]:');
console.log('Input:', test4);
console.log('Output:', result4);
console.log('✓ PASS:', result4.source === 'PurpurExtras' && result4.level === 'ERROR');
console.log();

// Test 5: Paper variante (short format)
const test5 = '[14:32:54 DEBUG]: [Chat] Debug message';
const result5 = parseMinecraftLog(test5);
console.log('Test 5 - Paper short format [Chat]:');
console.log('Input:', test5);
console.log('Output:', result5);
console.log('✓ PASS:', result5.source === 'Chat' && result5.level === 'DEBUG');
console.log();

// Test 6: Línea plana
const test6 = 'Loading server properties...';
const result6 = parseMinecraftLog(test6);
console.log('Test 6 - Texto plano:');
console.log('Input:', test6);
console.log('Output:', result6);
console.log('✓ PASS:', result6.source === null && result6.level === 'INFO');
console.log();

console.log('=== All tests completed ===');
