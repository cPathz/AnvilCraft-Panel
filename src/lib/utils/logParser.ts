export interface LogEntry {
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
  source: string | null;
  message: string;
}

// Compilar regexes fuera de la función (reutilizables)
const ANSI_REGEX = /\x1B\[[0-9;]*m/g;
const LOG_PATTERN_STANDARD = /^\[([^\]]+)\]\s*\[([^\]\/]+)\/([A-Z]+)\]:\s*(.*)$/;
const LOG_PATTERN_SHORT = /^\[([^\]]+)\s+([A-Z]+)\]:\s*(.*)$/;
const SOURCE_PATTERN = /^\[([A-Za-z][A-Za-z0-9_\-]*)\]\s+(.*)/;

const VALID_LEVELS = new Set(['INFO', 'WARN', 'ERROR', 'DEBUG']);
const RESERVED_WORDS = new Set(['INFO', 'WARN', 'ERROR', 'DEBUG', 'FATAL', 'TRACE']);

export function parseMinecraftLog(rawLog: string): LogEntry {
  // Eliminar ANSI (una sola pasada)
  const cleanLog = rawLog.replace(ANSI_REGEX, '');

  // Patrón 1: [HH:mm:ss] [Server thread/INFO]: message
  let match = cleanLog.match(LOG_PATTERN_STANDARD);
  if (match) {
    const [, timestamp, , rawLevel, message] = match;
    const level = (VALID_LEVELS.has(rawLevel) ? rawLevel : 'INFO') as 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
    const { source, cleanMessage } = extractSource(message);

    return { timestamp, level, source, message: cleanMessage };
  }

  // Patrón 2: [HH:mm:ss INFO]: message (Paper variante)
  match = cleanLog.match(LOG_PATTERN_SHORT);
  if (match) {
    const [, timestamp, rawLevel, message] = match;
    const level = (VALID_LEVELS.has(rawLevel) ? rawLevel : 'INFO') as 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
    const { source, cleanMessage } = extractSource(message);

    return { timestamp, level, source, message: cleanMessage };
  }

  // Fallback: texto plano (logs de inicio, etc.)
  const { source, cleanMessage } = extractSource(cleanLog);
  return {
    timestamp: '', // No inventar timestamp si no viene en el log
    level: 'INFO',
    source,
    message: cleanMessage.trim()
  };
}

function extractSource(message: string): { source: string | null; cleanMessage: string } {
  const match = message.match(SOURCE_PATTERN);
  if (match) {
    const potentialSource = match[1];
    // Solo acepta como source si NO es una palabra reservada (INFO, WARN, etc)
    if (!RESERVED_WORDS.has(potentialSource.toUpperCase())) {
      return { source: potentialSource, cleanMessage: match[2] };
    }
  }
  return { source: null, cleanMessage: message };
}
