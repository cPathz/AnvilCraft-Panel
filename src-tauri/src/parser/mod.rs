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

        // Patrón 4: Message con LEVEL detectado por palabra clave
        if let Some(parsed) = Self::try_pattern_level_keyword(raw_trimmed) {
            return parsed;
        }

        // Fallback: Retornar UNKNOWN pero sin romper
        Self {
            raw: raw.to_string(),
            timestamp: None,
            level: LogLevel::Unknown,
            plugin: None,
            message: raw_trimmed.to_string(),
            is_stacktrace_line: false,
            has_ansi_codes: has_ansi,
        }
    }

    // Patrón: [23:23:10 INFO]: [LuckPerms] Message
    fn try_pattern_full(raw: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^\[(\d{2}:\d{2}:\d{2})\s+(\w+)\]:\s+(?:\[([^\]]+)\])?\s*(.*)$"#)
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

    // Patrón: [23:23:10] [LuckPerms] Message
    fn try_pattern_timestamp_plugin(raw: &str) -> Option<Self> {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^\[(\d{2}:\d{2}:\d{2})\]\s+\[([^\]]+)\]\s*(.*)$"#)
                .expect("Invalid regex pattern_timestamp_plugin")
        });

        PATTERN.captures(raw).map(|caps| {
            let timestamp = caps.get(1).map(|m| m.as_str().to_string());
            let plugin = caps.get(2).map(|m| m.as_str().to_string());
            let message = caps.get(3).map(|m| m.as_str()).unwrap_or("").to_string();

            Self {
                raw: raw.to_string(),
                timestamp,
                level: LogLevel::Unknown,
                plugin,
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
            ("error", LogLevel::Error),
            ("warn", LogLevel::Warn),
            ("fatal", LogLevel::Fatal),
            ("debug", LogLevel::Debug),
            ("info", LogLevel::Info),
        ] {
            if lowercase.starts_with(keyword) || lowercase.contains(&format!(" {} ", keyword)) {
                return Some(Self {
                    raw: raw.to_string(),
                    timestamp: None,
                    level: level.clone(),
                    plugin: None,
                    message: raw.to_string(),
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
            message: raw.trim().to_string(),
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
