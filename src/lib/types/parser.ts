export type LogLevel = 'INFO' | 'WARN' | 'ERROR' | 'FATAL' | 'DEBUG' | 'UNKNOWN';

export interface ParsedLog {
  raw: string;
  timestamp?: string;
  level: LogLevel;
  plugin?: string;
  message: string;
  is_stacktrace_line: boolean;
  has_ansi_codes: boolean;
}
