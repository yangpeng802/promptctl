use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::model::{Constraints, Depth, PermissionLevel, Preset, Scope};
use crate::prompt::Lang;

/// Top-level config from `<config dir>/pm/config.toml`. Missing fields fall
/// back to the builtin defaults per field; an unparseable file falls back to
/// the whole default config with a warning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub default_preset: String,
    pub default_permission: String,
    pub default_depth: String,
    pub default_scope: String,
    pub language: String,
    pub constraints: Constraints,
    pub custom_presets: Vec<CustomPresetRaw>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_preset: "fix".to_string(),
            default_permission: "minimal".to_string(),
            default_depth: "normal".to_string(),
            default_scope: "auto".to_string(),
            language: "zh".to_string(),
            constraints: Constraints::default(),
            custom_presets: Vec::new(),
        }
    }
}

/// Custom preset as written in the config file (stringly typed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomPresetRaw {
    pub name: String,
    #[serde(default = "default_base")]
    pub base: String,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub depth: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub extra_rules: Vec<String>,
}

fn default_base() -> String {
    "fix".to_string()
}

/// A custom preset resolved against the builtin presets. Unset or invalid
/// fields inherit the base preset's defaults.
#[derive(Debug, Clone)]
pub struct CustomPreset {
    pub name: String,
    pub base: Preset,
    pub permission: Option<PermissionLevel>,
    pub depth: Option<Depth>,
    pub scope: Option<Scope>,
    pub extra_rules: Vec<String>,
}

impl CustomPresetRaw {
    pub fn resolve(&self) -> CustomPreset {
        CustomPreset {
            name: self.name.clone(),
            base: self.base.parse().unwrap_or(Preset::Fix),
            permission: self.permission.as_deref().and_then(|s| s.parse().ok()),
            depth: self.depth.as_deref().and_then(|s| s.parse().ok()),
            scope: self.scope.as_deref().and_then(|s| s.parse().ok()),
            extra_rules: self
                .extra_rules
                .iter()
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect(),
        }
    }
}

impl Config {
    /// OS standard config dir, e.g. `~/.config/pm/config.toml` on Linux.
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("pm").join("config.toml"))
    }

    /// OS standard data dir for generated history.
    pub fn history_path() -> Option<PathBuf> {
        dirs::data_dir().map(|dir| dir.join("pm").join("history.json"))
    }

    /// Load the config file. Never fails: a missing file uses defaults
    /// silently; a broken file uses defaults with a warning message.
    pub fn load() -> (Config, Option<String>) {
        let Some(path) = Self::config_path() else {
            return (Config::default(), None);
        };
        match fs::read_to_string(path) {
            Ok(text) => Self::from_toml(&text),
            Err(_) => (Config::default(), None),
        }
    }

    pub fn from_toml(text: &str) -> (Config, Option<String>) {
        match toml::from_str::<Config>(text) {
            Ok(config) => (config, None),
            Err(err) => (
                Config::default(),
                Some(format!("config parse failed, using defaults ({err})")),
            ),
        }
    }

    pub fn lang(&self) -> Lang {
        Lang::parse(&self.language)
    }

    pub fn default_preset(&self) -> Preset {
        self.default_preset.parse().unwrap_or(Preset::Fix)
    }

    pub fn default_permission(&self) -> PermissionLevel {
        self.default_permission
            .parse()
            .unwrap_or(PermissionLevel::Minimal)
    }

    pub fn default_depth(&self) -> Depth {
        self.default_depth.parse().unwrap_or(Depth::Normal)
    }

    pub fn default_scope(&self) -> Scope {
        self.default_scope.parse().unwrap_or(Scope::Auto)
    }

    pub fn resolve_custom(&self, name: &str) -> Option<CustomPreset> {
        self.custom_presets
            .iter()
            .find(|c| c.name == name)
            .map(CustomPresetRaw::resolve)
    }

    pub fn resolved_customs(&self) -> Vec<CustomPreset> {
        self.custom_presets
            .iter()
            .map(CustomPresetRaw::resolve)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_fix_like() {
        let c = Config::default();
        assert_eq!(c.default_preset(), Preset::Fix);
        assert_eq!(c.default_permission(), PermissionLevel::Minimal);
        assert_eq!(c.default_depth(), Depth::Normal);
        assert_eq!(c.default_scope(), Scope::Auto);
        assert_eq!(c.lang(), Lang::Zh);
        assert!(c.constraints.no_unrelated_changes);
        assert!(!c.constraints.run_tests);
        assert!(c.custom_presets.is_empty());
    }

    #[test]
    fn parses_full_example() {
        let text = r#"
            default_preset = "fix"
            default_permission = "minimal"
            default_depth = "normal"
            default_scope = "auto"
            language = "zh"

            [constraints]
            run_tests = true

            [[custom_presets]]
            name = "legacy-fix"
            base = "fix"
            permission = "minimal"
            depth = "deep"

            extra_rules = [
                "不要修改公共接口",
                "不要引入新依赖",
                "必须兼容现有构建环境"
            ]
        "#;
        let (config, warning) = Config::from_toml(text);
        assert!(warning.is_none());
        assert!(config.constraints.run_tests);
        // fields absent from [constraints] keep their default
        assert!(config.constraints.no_unrelated_changes);
        let cp = config.resolve_custom("legacy-fix").expect("custom preset");
        assert_eq!(cp.base, Preset::Fix);
        assert_eq!(cp.permission, Some(PermissionLevel::Minimal));
        assert_eq!(cp.depth, Some(Depth::Deep));
        assert_eq!(cp.scope, None);
        assert_eq!(cp.extra_rules.len(), 3);
        assert_eq!(cp.extra_rules[2], "必须兼容现有构建环境");
        assert!(config.resolve_custom("missing").is_none());
    }

    #[test]
    fn invalid_values_fall_back() {
        let text = r#"
            default_preset = "bogus"
            default_permission = "wat"
            default_depth = "sideways"
            default_scope = "galaxy"
            language = "de"

            [[custom_presets]]
            name = "weird"
            base = "nope"
            permission = "l9"
        "#;
        let (config, warning) = Config::from_toml(text);
        assert!(
            warning.is_none(),
            "unknown enum values are per-field fallbacks"
        );
        assert_eq!(config.default_preset(), Preset::Fix);
        assert_eq!(config.default_permission(), PermissionLevel::Minimal);
        assert_eq!(config.default_depth(), Depth::Normal);
        assert_eq!(config.default_scope(), Scope::Auto);
        assert_eq!(config.lang(), Lang::Zh);
        let cp = config.resolve_custom("weird").unwrap();
        assert_eq!(cp.base, Preset::Fix);
        assert_eq!(cp.permission, None);
    }

    #[test]
    fn broken_toml_uses_defaults_with_warning() {
        let (config, warning) = Config::from_toml("this is [ definitely not toml");
        assert_eq!(config, Config::default());
        let warning = warning.expect("warning expected");
        assert!(warning.contains("config parse failed"));
    }
}
