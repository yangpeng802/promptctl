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
    pub constraints: PartialConstraints,
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
            constraints: PartialConstraints::default(),
            custom_presets: Vec::new(),
        }
    }
}

/// Config-file form of constraints: every field is optional so "unset" stays
/// distinct from an explicit true/false. Unset fields inherit the active
/// preset's defaults; explicit values win. The TOML shape is unchanged, so
/// existing `config.toml` files keep working as-is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PartialConstraints {
    pub no_unrelated_changes: Option<bool>,
    pub no_unnecessary_refactor: Option<bool>,
    pub preserve_public_interfaces: Option<bool>,
    pub preserve_coding_style: Option<bool>,
    pub analyze_before_modifying: Option<bool>,
    pub build_after_modifying: Option<bool>,
    pub run_tests: Option<bool>,
    pub no_new_files: Option<bool>,
    pub no_dependency_changes: Option<bool>,
    pub no_unrelated_formatting: Option<bool>,
    pub explain_root_cause: Option<bool>,
    pub explain_modifications: Option<bool>,
    pub list_remaining_risks: Option<bool>,
}

impl PartialConstraints {
    /// Fill unset fields from `base`, keeping explicit user choices.
    pub fn resolve(&self, base: Constraints) -> Constraints {
        let p = self;
        Constraints {
            no_unrelated_changes: p.no_unrelated_changes.unwrap_or(base.no_unrelated_changes),
            no_unnecessary_refactor: p
                .no_unnecessary_refactor
                .unwrap_or(base.no_unnecessary_refactor),
            preserve_public_interfaces: p
                .preserve_public_interfaces
                .unwrap_or(base.preserve_public_interfaces),
            preserve_coding_style: p
                .preserve_coding_style
                .unwrap_or(base.preserve_coding_style),
            analyze_before_modifying: p
                .analyze_before_modifying
                .unwrap_or(base.analyze_before_modifying),
            build_after_modifying: p
                .build_after_modifying
                .unwrap_or(base.build_after_modifying),
            run_tests: p.run_tests.unwrap_or(base.run_tests),
            no_new_files: p.no_new_files.unwrap_or(base.no_new_files),
            no_dependency_changes: p
                .no_dependency_changes
                .unwrap_or(base.no_dependency_changes),
            no_unrelated_formatting: p
                .no_unrelated_formatting
                .unwrap_or(base.no_unrelated_formatting),
            explain_root_cause: p.explain_root_cause.unwrap_or(base.explain_root_cause),
            explain_modifications: p
                .explain_modifications
                .unwrap_or(base.explain_modifications),
            list_remaining_risks: p.list_remaining_risks.unwrap_or(base.list_remaining_risks),
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
        let resolved = c.constraints.resolve(Constraints::default());
        assert!(resolved.no_unrelated_changes);
        assert!(!resolved.run_tests);
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
        let resolved = config.constraints.resolve(Constraints::default());
        assert!(resolved.run_tests);
        // fields absent from [constraints] keep their default
        assert!(resolved.no_unrelated_changes);
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

    #[test]
    fn partial_constraints_distinguish_unset_from_explicit() {
        // Unset fields inherit the given base (here: refactor relaxations).
        let empty = PartialConstraints::default();
        let out = empty.resolve(Constraints::for_preset(Preset::Refactor));
        assert!(!out.no_new_files);
        assert!(out.run_tests);
        // The same empty patch over the FIX baseline keeps strict values.
        let out = empty.resolve(Constraints::default());
        assert!(out.no_new_files);
        assert!(!out.run_tests);

        // An explicit false wins even where the base says true ...
        let patch = PartialConstraints {
            no_new_files: Some(false),
            ..PartialConstraints::default()
        };
        assert!(!patch.resolve(Constraints::default()).no_new_files);
        // ... and an explicit true wins where the base says false.
        let patch = PartialConstraints {
            no_new_files: Some(true),
            ..PartialConstraints::default()
        };
        assert!(
            patch
                .resolve(Constraints::for_preset(Preset::Refactor))
                .no_new_files
        );
    }
}
