use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub segments: SegmentsConfig,
    /// Per-segment config blocks: `[segment.git]`, `[segment.path]`, etc.
    #[serde(default)]
    pub segment: HashMap<String, SegmentConfig>,
    /// `[weather]` TOML section (optional). All fields individually optional.
    /// Consumed only when the `weather` cargo feature is enabled.
    #[serde(default)]
    #[cfg_attr(not(feature = "weather"), allow(dead_code))]
    pub weather: WeatherConfig,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct SegmentsConfig {
    /// Ordered list of segment names. Empty means use the default order.
    pub order: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct SegmentConfig {
    /// Whether this segment is enabled. `None` means use the default (true).
    pub enabled: Option<bool>,
    /// Foreground color override (256-color).
    pub fg: Option<u8>,
    /// Background color override (256-color).
    pub bg: Option<u8>,
    /// Shell command to run (`custom_command` segment).
    pub command: Option<String>,
    /// Cache TTL in seconds (`custom_command` segment). Default: 30.
    pub cache_secs: Option<u64>,
    /// Command timeout in milliseconds (`custom_command` segment). Default: 500.
    pub timeout_ms: Option<u64>,
}

/// `[weather]` TOML section. All fields optional; CLI flags and env vars
/// override anything set here. CLI > env > TOML > built-in defaults.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct WeatherConfig {
    /// `"openmeteo"` (default) or `"openweather"`.
    pub provider: Option<String>,
    /// API key (required for `openweather`).
    pub api_key: Option<String>,
    /// `"metric"` (default) or `"imperial"`.
    pub units: Option<String>,
    /// Cache TTL in minutes. Default: 15.
    pub cache_ttl: Option<u64>,
    /// Show `"City, CC"` prefix. Default: true.
    pub show_city: Option<bool>,
    /// Show weather icon. Default: true.
    pub show_icon: Option<bool>,
    /// Use Nerd Font glyphs instead of plain Unicode. Default: false.
    pub use_nerd_font: Option<bool>,
    /// Fixed latitude override.
    pub lat: Option<f64>,
    /// Fixed longitude override.
    pub lon: Option<f64>,
    /// Shell command that prints `"lat|lon"` on stdout.
    pub location_cmd: Option<String>,
}

impl Config {
    /// Load config from disk. Returns defaults if the file is missing.
    /// Prints to stderr and returns defaults if the file exists but is invalid.
    #[must_use]
    pub fn load() -> Self {
        let path = config_path();
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match toml::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("plx: invalid config at {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Returns whether a segment is enabled. Segments are enabled by default
    /// unless explicitly set to `enabled = false`.
    #[must_use]
    pub fn segment_enabled(&self, name: &str) -> bool {
        self.segment
            .get(name)
            .and_then(|s| s.enabled)
            .unwrap_or(true)
    }
}

fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("PLX_CONFIG") {
        return PathBuf::from(path);
    }
    let base = std::env::var("XDG_CONFIG_HOME").map_or_else(
        |_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
            PathBuf::from(home).join(".config")
        },
        PathBuf::from,
    );
    base.join("plx").join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_empty_order() {
        let cfg = Config::default();
        assert!(cfg.segments.order.is_empty());
        assert!(cfg.segment.is_empty());
    }

    #[test]
    fn parse_minimal_config() {
        let toml = "";
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(cfg.segments.order.is_empty());
    }

    #[test]
    fn parse_custom_order() {
        let toml = r#"
[segments]
order = ["path", "git", "character"]
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.segments.order, vec!["path", "git", "character"]);
    }

    #[test]
    fn parse_segment_enabled() {
        let toml = r"
[segment.hostname]
enabled = false

[segment.git]
enabled = true
";
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(!cfg.segment_enabled("hostname"));
        assert!(cfg.segment_enabled("git"));
    }

    #[test]
    fn segment_enabled_defaults_to_true() {
        let cfg = Config::default();
        assert!(cfg.segment_enabled("anything"));
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let toml = r"
[segment.path]
enabled = true
max_dir_size = 15
fg = 200
";
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(cfg.segment_enabled("path"));
    }

    #[test]
    fn load_missing_file_returns_default() {
        // Point to a file that does not exist
        unsafe { std::env::set_var("PLX_CONFIG", "/tmp/plx-nonexistent-test.toml") };
        let cfg = Config::load();
        assert!(cfg.segments.order.is_empty());
        unsafe { std::env::remove_var("PLX_CONFIG") };
    }
}
