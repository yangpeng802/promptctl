use pm::config::Config;
use pm::model::{Constraints, Depth, PermissionLevel, Preset};
use pm::prompt::Lang;

#[test]
fn default_config_matches_fix_defaults() {
    let config = Config::default();
    assert_eq!(config.default_preset(), Preset::Fix);
    assert_eq!(config.default_permission(), PermissionLevel::Minimal);
    assert_eq!(config.default_depth(), Depth::Normal);
    assert_eq!(config.default_scope(), pm::model::Scope::Auto);
    assert_eq!(config.lang(), Lang::Zh);
    let resolved = config.constraints.resolve(Constraints::default());
    assert!(resolved.no_unrelated_changes);
    assert!(resolved.no_new_files);
    assert!(!resolved.run_tests);
    assert!(config.custom_presets.is_empty());
}

#[test]
fn parses_full_spec_example() {
    let text = r#"
default_preset = "fix"
default_permission = "minimal"
default_depth = "normal"
default_scope = "auto"
language = "zh"

[constraints]
no_unrelated_changes = true
no_unnecessary_refactor = true
preserve_public_interfaces = true
preserve_coding_style = true
analyze_before_modifying = true
build_after_modifying = true
run_tests = false
no_new_files = true
no_dependency_changes = true
no_unrelated_formatting = true
explain_root_cause = true
explain_modifications = true
list_remaining_risks = true

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
    assert_eq!(config.default_preset(), Preset::Fix);
    assert_eq!(config.default_permission(), PermissionLevel::Minimal);
    let resolved = config.constraints.resolve(Constraints::default());
    assert!(!resolved.run_tests);
    assert!(resolved.no_unrelated_changes);

    let cp = config.resolve_custom("legacy-fix").expect("custom preset");
    assert_eq!(cp.base, Preset::Fix);
    assert_eq!(cp.permission, Some(PermissionLevel::Minimal));
    assert_eq!(cp.depth, Some(Depth::Deep));
    assert_eq!(cp.scope, None);
    assert_eq!(cp.extra_rules.len(), 3);
    assert_eq!(cp.extra_rules[2], "必须兼容现有构建环境");
    assert!(config.resolve_custom("missing").is_none());
    assert_eq!(config.resolved_customs().len(), 1);
}

#[test]
fn broken_toml_falls_back_to_defaults_with_warning() {
    let (config, warning) = Config::from_toml("default_preset = [ definitely broken");
    assert_eq!(config, Config::default());
    let warning = warning.expect("warning expected");
    assert!(warning.contains("config parse failed"));
}

#[test]
fn unknown_enum_values_fall_back_per_field() {
    let text = r#"
default_preset = "nonsense"
default_permission = "l9"
default_depth = "sideways"
default_scope = "galaxy"
language = "de"

[[custom_presets]]
name = "weird"
base = "also-nonsense"
permission = "l0x"
"#;
    let (config, warning) = Config::from_toml(text);
    assert!(warning.is_none(), "per-field fallback, not a parse error");
    assert_eq!(config.default_preset(), Preset::Fix);
    assert_eq!(config.default_permission(), PermissionLevel::Minimal);
    assert_eq!(config.default_depth(), Depth::Normal);
    assert_eq!(config.default_scope(), pm::model::Scope::Auto);
    assert_eq!(config.lang(), Lang::Zh);
    let cp = config.resolve_custom("weird").unwrap();
    assert_eq!(cp.base, Preset::Fix, "unknown base falls back to fix");
    assert_eq!(
        cp.permission, None,
        "invalid permission inherits base default"
    );
}

#[test]
fn partial_constraints_table_inherits_defaults() {
    let text = "language = \"en\"\n\n[constraints]\nrun_tests = true\n";
    let (config, warning) = Config::from_toml(text);
    assert!(warning.is_none());
    assert_eq!(config.lang(), Lang::En);
    let resolved = config.constraints.resolve(Constraints::default());
    assert!(resolved.run_tests, "explicit value kept");
    assert!(
        resolved.no_unrelated_changes,
        "missing field inherits default"
    );
    assert!(resolved.no_new_files, "missing field inherits default");
}

#[test]
fn empty_file_is_valid_defaults() {
    let (config, warning) = Config::from_toml("");
    assert!(warning.is_none());
    assert_eq!(config, Config::default());
}

#[test]
fn explicit_false_is_kept_apart_from_unset() {
    // Explicit false wins over a strict base ...
    let (config, warning) = Config::from_toml("[constraints]\nno_new_files = false\n");
    assert!(warning.is_none());
    assert!(
        !config
            .constraints
            .resolve(Constraints::default())
            .no_new_files
    );
    // ... while an unset field follows each preset's own default.
    assert!(
        !config
            .constraints
            .resolve(Constraints::for_preset(Preset::Refactor))
            .no_new_files
    );
    assert!(
        config
            .constraints
            .resolve(Constraints::default())
            .no_unrelated_changes
    );

    // Explicit true wins over a relaxed preset.
    let (config, _) = Config::from_toml("[constraints]\nno_new_files = true\n");
    assert!(
        config
            .constraints
            .resolve(Constraints::for_preset(Preset::Refactor))
            .no_new_files
    );
}
