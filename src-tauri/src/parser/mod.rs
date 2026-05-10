use regex::Regex;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct ParsedLog {
    pub raw: String,
    pub timestamp: Option<String>,
    pub level: LogLevel,
    pub plugin: Option<String>,
    pub message: String,
    pub is_stacktrace_line: bool,
    pub has_ansi_codes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Fatal,
    Debug,
    Unknown,
}

impl ParsedLog {
    pub fn new(raw: &str) -> Self {
        Self::parse(raw)
    }

    fn parse(raw: &str) -> Self {
        // Detectar si es línea de stacktrace BEFORE trimming
        if Self::is_stacktrace_line(raw) {
            return Self::parse_stacktrace(raw);
        }

        let raw_trimmed = raw.trim();

        // Detectar si tiene códigos ANSI
        let has_ansi = raw_trimmed.contains("\x1b[");

        // Intentar patrones en orden de especificidad
        // Patrón 1: [HH:MM:SS LEVEL]: [Plugin] Message (completo)
        if let Some(parsed) = Self::try_pattern_full(raw_trimmed) {
            return parsed;
        }

        // Patrón 2: [HH:MM:SS] [Plugin] Message
        if let Some(parsed) = Self::try_pattern_timestamp_plugin(raw_trimmed) {
            return parsed;
        }

        // Patrón 3: [Plugin] Message (solo plugin)
        if let Some(parsed) = Self::try_pattern_plugin_only(raw_trimmed) {
            return parsed;
        }

        // Patrón 4: Log4j2 ISO Timestamp (2024-05-10T...)
        if let Some(parsed) = Self::try_pattern_iso(raw_trimmed) {
            return parsed;
        }

        // Patrón 5: Message con LEVEL detectado por palabra clave
        if let Some(parsed) = Self::try_pattern_level_keyword(raw_trimmed) {
            return parsed;
        }

        // Patrón 5: [LEVEL]: (sin timestamp, común en algunos logs de inicio)
        if let Some(parsed) = Self::try_pattern_only_level(raw_trimmed) {
            return parsed;
        }

        // Fallback: Retornar UNKNOWN pero sin romper, preservando el espaciado original
        Self {
            raw: raw.to_string(),
            timestamp: None,
            level: LogLevel::Unknown,
            plugin: None,
            message: raw.to_string(),
            is_stacktrace_line: false,
            has_ansi_codes: has_ansi,
        }
    }

    // Patrón: [23:23:10 INFO]: [LuckPerms] Message o [23:23:10 Server thread/INFO]: Message
    fn try_pattern_full(raw: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            // Soporta [HH:mm:ss LEVEL] y [HH:mm:ss Thread/LEVEL]
            Regex::new(r#"^\[(\d{2}:\d{2}:\d{2})\s+(?:[^/\]]*/)?(\w+)\]:\s*(?:\[([^\]]+)\])?\s*(.*)$"#)
                .expect("Invalid regex pattern_full")
        });

        PATTERN.captures(raw).map(|caps| {
            let timestamp = caps.get(1).map(|m| m.as_str().to_string());
            let level = Self::parse_level(caps.get(2).map(|m| m.as_str()).unwrap_or("INFO"));
            let plugin = caps.get(3).map(|m| m.as_str().to_string());
            let message = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();

            Self {
                raw: raw.to_string(),
                timestamp,
                level,
                plugin,
                message,
                is_stacktrace_line: false,
                has_ansi_codes: raw.contains("\x1b["),
            }
        })
    }

    // Patrón: [23:23:10] [Server thread/INFO]: Message
    fn try_pattern_timestamp_plugin(raw: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            // Detecta timestamp y luego un segundo corchete que puede ser [Plugin] o [Thread/LEVEL]
            Regex::new(r#"^\[(\d{2}:\d{2}:\d{2})\]\s+\[(?:([^/\]]+)/)?([^\]]+)\]:\s*(.*)$"#)
                .expect("Invalid regex pattern_timestamp_plugin")
        });

        PATTERN.captures(raw).map(|caps| {
            let timestamp = caps.get(1).map(|m| m.as_str().to_string());
            let thread_or_plugin = caps.get(2).map(|m| m.as_str().to_string());
            let potential_level = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
            let message = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();

            // Determinar si el contenido del segundo corchete es un nivel de log
            let level = Self::parse_level(&potential_level);
            
            let (final_level, final_plugin) = if level != LogLevel::Unknown {
                (level, thread_or_plugin) // Si es un nivel, el prefijo era el hilo/plugin
            } else {
                // Si no es un nivel, tratamos todo el segundo corchete como el plugin (comportamiento original)
                let full_plugin = if let Some(t) = thread_or_plugin {
                    format!("{}/{}", t, potential_level)
                } else {
                    potential_level
                };
                (LogLevel::Unknown, Some(full_plugin))
            };

            Self {
                raw: raw.to_string(),
                timestamp,
                level: final_level,
                plugin: final_plugin,
                message,
                is_stacktrace_line: false,
                has_ansi_codes: raw.contains("\x1b["),
            }
        })
    }

    // Patrón: [LuckPerms] Message
    fn try_pattern_plugin_only(raw: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^\[([^\]]+)\]\s*(.*)$"#)
                .expect("Invalid regex pattern_plugin_only")
        });

        PATTERN.captures(raw).map(|caps| {
            let potential_plugin = caps.get(1).unwrap().as_str();
            let message = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();

            // Validar que sea un nombre de plugin válido (no es [23:23:10])
            if potential_plugin.contains(":") {
                return None;
            }

            Some(Self {
                raw: raw.to_string(),
                timestamp: None,
                level: LogLevel::Unknown,
                plugin: Some(potential_plugin.to_string()),
                message,
                is_stacktrace_line: false,
                has_ansi_codes: raw.contains("\x1b["),
            })
        })
        .flatten()
    }

    // Patrón: Detectar LEVEL como palabra clave en el mensaje
    fn try_pattern_level_keyword(raw: &str) -> Option<Self> {
        let lowercase = raw.to_lowercase();

        for (keyword, level) in &[
            ("warning", LogLevel::Warn),
            ("severe", LogLevel::Fatal),
            ("error", LogLevel::Error),
            ("fatal", LogLevel::Fatal),
            ("warn", LogLevel::Warn),
            ("debug", LogLevel::Debug),
            ("info", LogLevel::Info),
        ] {
            if lowercase.starts_with(keyword) || lowercase.contains(&format!(" {} ", keyword)) {
                let mut message = raw.to_string();
                
                // Limpiar ANSI para detectar el prefijo correctamente
                let clean_raw = Regex::new(r#"\x1b\[[0-9;]*[a-zA-Z]"#).unwrap().replace_all(raw, "");
                let clean_lower = clean_raw.to_lowercase();
                
                let prefix = format!("{}:", keyword);
                if clean_lower.starts_with(&prefix) {
                    // Encontrar dónde termina el prefijo en la cadena original (con ANSI)
                    if let Some(colon_pos) = raw.find(':') {
                        message = raw[colon_pos + 1..].trim().to_string();
                    }
                }

                return Some(Self {
                    raw: raw.to_string(),
                    timestamp: None,
                    level: level.clone(),
                    plugin: None,
                    message,
                    is_stacktrace_line: false,
                    has_ansi_codes: raw.contains("\x1b["),
                });
            }
        }

        None
    }

    // Detectar líneas de stacktrace
    fn is_stacktrace_line(raw: &str) -> bool {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^\s+(at\s+|Caused by:|Exception in thread|Suppressed:)"#)
                .expect("Invalid regex stacktrace")
        });

        PATTERN.is_match(raw)
    }

    fn parse_stacktrace(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
            timestamp: None,
            level: LogLevel::Unknown,
            plugin: None,
            message: raw.to_string(), // Preservamos los espacios/tabs originales
            is_stacktrace_line: true,
            has_ansi_codes: raw.contains("\x1b["),
        }
    }

    fn parse_level(level_str: &str) -> LogLevel {
        match level_str.to_uppercase().as_str() {
            "INFO" => LogLevel::Info,
            "WARN" | "WARNING" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            "FATAL" | "SEVERE" => LogLevel::Fatal,
            "DEBUG" => LogLevel::Debug,
            _ => LogLevel::Unknown,
        }
    }

    fn try_pattern_iso(raw: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            // Soporta 2024-05-10T01:00:41.983Z [Thread] LEVEL Message
            Regex::new(r#"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?)\s+([^\s]+)\s+(\w+)\s+(.*)$"#)
                .expect("Invalid regex pattern_iso")
        });

        PATTERN.captures(raw).map(|caps| {
            let timestamp_full = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            // Extraer solo la parte de la hora HH:mm:ss si es muy largo
            let timestamp = if timestamp_full.contains('T') {
                timestamp_full.split('T').collect::<Vec<&str>>().get(1)
                    .map(|s| s.split('.').collect::<Vec<&str>>()[0].to_string())
            } else {
                Some(timestamp_full.to_string())
            };

            let source = caps.get(2).map(|m| m.as_str().to_string());
            let level = Self::parse_level(caps.get(3).map(|m| m.as_str()).unwrap_or("INFO"));
            let message = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();

            Self {
                raw: raw.to_string(),
                timestamp,
                level,
                plugin: source,
                message,
                is_stacktrace_line: false,
                has_ansi_codes: raw.contains("\x1b["),
            }
        })
    }

    fn try_pattern_only_level(raw: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^\[(\w+)\]:\s*(.*)$"#).expect("Invalid regex pattern_only_level")
        });

        PATTERN.captures(raw).map(|caps| {
            let level_str = caps.get(1).map(|m| m.as_str()).unwrap_or("INFO");
            let level = Self::parse_level(level_str);
            
            // Si no es un nivel válido, no es este patrón
            if level == LogLevel::Unknown {
                return None;
            }

            let message = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();

            Some(Self {
                raw: raw.to_string(),
                timestamp: None,
                level,
                plugin: None,
                message,
                is_stacktrace_line: false,
                has_ansi_codes: raw.contains("\x1b["),
            })
        }).flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_pattern() {
        let raw = "[23:23:10 INFO]: [LuckPerms] Loading configuration...";
        let parsed = ParsedLog::new(raw);
        assert_eq!(parsed.timestamp, Some("23:23:10".to_string()));
        assert_eq!(parsed.level, LogLevel::Info);
        assert_eq!(parsed.plugin, Some("LuckPerms".to_string()));
        assert!(parsed.message.contains("Loading configuration"));
    }

    #[test]
    fn test_parse_timestamp_plugin_pattern() {
        let raw = "[23:23:10] [LuckPerms] Some message";
        let parsed = ParsedLog::new(raw);
        assert_eq!(parsed.timestamp, Some("23:23:10".to_string()));
        assert_eq!(parsed.plugin, Some("LuckPerms".to_string()));
        assert_eq!(parsed.message, "Some message");
    }

    #[test]
    fn test_parse_plugin_only_pattern() {
        let raw = "[MyPlugin] Message here";
        let parsed = ParsedLog::new(raw);
        assert_eq!(parsed.plugin, Some("MyPlugin".to_string()));
        assert_eq!(parsed.message, "Message here");
    }

    #[test]
    fn test_parse_stacktrace() {
        let raw = "    at com.example.Plugin.onEnable(Plugin.java:45)";
        let parsed = ParsedLog::new(raw);
        assert!(parsed.is_stacktrace_line);
        assert_eq!(parsed.level, LogLevel::Unknown);
    }

    #[test]
    fn test_fallback_unknown() {
        let raw = "Some random garbage line without structure";
        let parsed = ParsedLog::new(raw);
        assert_eq!(parsed.level, LogLevel::Unknown);
        assert_eq!(parsed.message, raw);
    }

    #[test]
    fn test_ansi_detection() {
        let raw = "[\x1b[38;5;14m23:23:10 INFO\x1b[0m]: Message";
        let parsed = ParsedLog::new(raw);
        assert!(parsed.has_ansi_codes);
    }

    #[test]
    fn test_level_keyword_detection() {
        let raw = "ERROR: Something went wrong";
        let parsed = ParsedLog::new(raw);
        assert_eq!(parsed.level, LogLevel::Error);
    }

    #[test]
    fn test_multiple_bracket_fallback() {
        let raw = "[23:23:10] some text";
        let parsed = ParsedLog::new(raw);
        // Should not match as plugin (contains colon in potential plugin)
        assert_eq!(parsed.plugin, None);
    }
}
