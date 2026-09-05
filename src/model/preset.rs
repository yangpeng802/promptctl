use std::fmt;
use std::str::FromStr;

use super::constraints::Constraints;
use super::depth::Depth;
use super::permission::PermissionLevel;
use super::scope::Scope;

/// Task modes. Each preset carries default permission/depth/scope/constraints
/// and drives a distinct prompt template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Preset {
    Analyze,
    Fix,
    Debug,
    Trace,
    Plan,
    Arch,
    Refactor,
    Yolo,
}

impl Preset {
    /// Order used both by the TUI preset list and CLI docs.
    pub const ALL: [Preset; 8] = [
        Preset::Analyze,
        Preset::Fix,
        Preset::Debug,
        Preset::Trace,
        Preset::Plan,
        Preset::Arch,
        Preset::Refactor,
        Preset::Yolo,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Preset::Analyze => "ANALYZE",
            Preset::Fix => "FIX",
            Preset::Debug => "DEBUG",
            Preset::Trace => "TRACE",
            Preset::Plan => "PLAN",
            Preset::Arch => "ARCH",
            Preset::Refactor => "REFACTOR",
            Preset::Yolo => "YOLO",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Preset::Analyze => "analyze",
            Preset::Fix => "fix",
            Preset::Debug => "debug",
            Preset::Trace => "trace",
            Preset::Plan => "plan",
            Preset::Arch => "arch",
            Preset::Refactor => "refactor",
            Preset::Yolo => "yolo",
        }
    }

    pub fn default_permission(self) -> PermissionLevel {
        match self {
            Preset::Analyze | Preset::Plan | Preset::Trace | Preset::Arch => {
                PermissionLevel::ReadOnly
            }
            Preset::Fix | Preset::Debug => PermissionLevel::Minimal,
            Preset::Refactor => PermissionLevel::Refactor,
            Preset::Yolo => PermissionLevel::Yolo,
        }
    }

    pub fn default_depth(self) -> Depth {
        match self {
            Preset::Debug | Preset::Trace | Preset::Plan | Preset::Arch | Preset::Refactor => {
                Depth::Deep
            }
            Preset::Analyze | Preset::Fix | Preset::Yolo => Depth::Normal,
        }
    }

    pub fn default_scope(self) -> Scope {
        match self {
            Preset::Trace | Preset::Arch => Scope::WholeRepo,
            Preset::Refactor => Scope::CurrentModule,
            Preset::Analyze | Preset::Fix | Preset::Debug | Preset::Plan | Preset::Yolo => {
                Scope::Auto
            }
        }
    }

    pub fn default_constraints(self) -> Constraints {
        Constraints::for_preset(self)
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Preset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "analyze" | "analysis" | "analyse" => Ok(Preset::Analyze),
            "fix" | "repair" => Ok(Preset::Fix),
            "debug" => Ok(Preset::Debug),
            "trace" => Ok(Preset::Trace),
            "plan" => Ok(Preset::Plan),
            "arch" | "architecture" => Ok(Preset::Arch),
            "refactor" => Ok(Preset::Refactor),
            "yolo" => Ok(Preset::Yolo),
            _ => Err(format!(
                "invalid preset '{s}' (expected analyze|fix|debug|trace|plan|arch|refactor|yolo)"
            )),
        }
    }
}

/// Resolve the conflict between a pure-analysis preset and an aggressive
/// permission level: ANALYZE / PLAN / TRACE always win and stay read-only,
/// everything else respects the user's choice.
pub fn effective_permission(preset: Preset, permission: PermissionLevel) -> PermissionLevel {
    match preset {
        Preset::Analyze | Preset::Plan | Preset::Trace => PermissionLevel::ReadOnly,
        _ => permission,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_permission_caps_analysis_presets() {
        assert_eq!(
            effective_permission(Preset::Analyze, PermissionLevel::Yolo),
            PermissionLevel::ReadOnly
        );
        assert_eq!(
            effective_permission(Preset::Plan, PermissionLevel::Refactor),
            PermissionLevel::ReadOnly
        );
        assert_eq!(
            effective_permission(Preset::Trace, PermissionLevel::Scoped),
            PermissionLevel::ReadOnly
        );
    }

    #[test]
    fn effective_permission_respects_modification_presets() {
        assert_eq!(
            effective_permission(Preset::Fix, PermissionLevel::Minimal),
            PermissionLevel::Minimal
        );
        assert_eq!(
            effective_permission(Preset::Refactor, PermissionLevel::Refactor),
            PermissionLevel::Refactor
        );
        assert_eq!(
            effective_permission(Preset::Yolo, PermissionLevel::Yolo),
            PermissionLevel::Yolo
        );
        assert_eq!(
            effective_permission(Preset::Debug, PermissionLevel::Minimal),
            PermissionLevel::Minimal
        );
    }

    #[test]
    fn parses_and_defaults() {
        assert_eq!("fix".parse(), Ok(Preset::Fix));
        assert_eq!("ARCH".parse(), Ok(Preset::Arch));
        assert!("nope".parse::<Preset>().is_err());
        assert_eq!(Preset::Debug.default_depth(), Depth::Deep);
        assert_eq!(Preset::Trace.default_scope(), Scope::WholeRepo);
        assert_eq!(
            Preset::Refactor.default_permission(),
            PermissionLevel::Refactor
        );
        assert_eq!(Preset::Fix.to_string(), "FIX");
    }
}
