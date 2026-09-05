use serde::{Deserialize, Serialize};

use super::preset::Preset;

/// The 13 builtin toggles. `Default` is the default FIX configuration from
/// the requirements; partial config files inherit these values per field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Constraints {
    pub no_unrelated_changes: bool,
    pub no_unnecessary_refactor: bool,
    pub preserve_public_interfaces: bool,
    pub preserve_coding_style: bool,
    pub analyze_before_modifying: bool,
    pub build_after_modifying: bool,
    pub run_tests: bool,
    pub no_new_files: bool,
    pub no_dependency_changes: bool,
    pub no_unrelated_formatting: bool,
    pub explain_root_cause: bool,
    pub explain_modifications: bool,
    pub list_remaining_risks: bool,
}

impl Constraints {
    /// Stable display order used by the TUI checklist.
    pub const LABELS: [&'static str; 13] = [
        "No unrelated changes",
        "No unnecessary refactor",
        "Preserve public interfaces",
        "Preserve coding style",
        "Analyze before modifying",
        "Build after modifying",
        "Run tests",
        "No new files",
        "No dependency changes",
        "No formatting unrelated code",
        "Explain root cause",
        "Explain modifications",
        "List remaining risks",
    ];

    pub fn get(&self, index: usize) -> bool {
        match index {
            0 => self.no_unrelated_changes,
            1 => self.no_unnecessary_refactor,
            2 => self.preserve_public_interfaces,
            3 => self.preserve_coding_style,
            4 => self.analyze_before_modifying,
            5 => self.build_after_modifying,
            6 => self.run_tests,
            7 => self.no_new_files,
            8 => self.no_dependency_changes,
            9 => self.no_unrelated_formatting,
            10 => self.explain_root_cause,
            11 => self.explain_modifications,
            12 => self.list_remaining_risks,
            _ => false,
        }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        match index {
            0 => self.no_unrelated_changes = value,
            1 => self.no_unnecessary_refactor = value,
            2 => self.preserve_public_interfaces = value,
            3 => self.preserve_coding_style = value,
            4 => self.analyze_before_modifying = value,
            5 => self.build_after_modifying = value,
            6 => self.run_tests = value,
            7 => self.no_new_files = value,
            8 => self.no_dependency_changes = value,
            9 => self.no_unrelated_formatting = value,
            10 => self.explain_root_cause = value,
            11 => self.explain_modifications = value,
            12 => self.list_remaining_risks = value,
            _ => {}
        }
    }

    /// Sensible constraint defaults per preset. FIX equals `Default`.
    pub fn for_preset(preset: Preset) -> Constraints {
        match preset {
            Preset::Fix | Preset::Debug => Constraints::default(),
            Preset::Analyze | Preset::Plan | Preset::Trace | Preset::Arch => Constraints {
                build_after_modifying: false,
                run_tests: false,
                explain_modifications: false,
                ..Constraints::default()
            },
            Preset::Refactor => Constraints {
                no_unnecessary_refactor: false,
                build_after_modifying: true,
                run_tests: true,
                no_new_files: false,
                explain_root_cause: false,
                ..Constraints::default()
            },
            Preset::Yolo => Constraints {
                no_unrelated_changes: true,
                no_unnecessary_refactor: false,
                preserve_public_interfaces: false,
                preserve_coding_style: false,
                analyze_before_modifying: false,
                build_after_modifying: true,
                run_tests: true,
                no_new_files: false,
                no_dependency_changes: false,
                no_unrelated_formatting: false,
                explain_root_cause: false,
                explain_modifications: true,
                list_remaining_risks: false,
            },
        }
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Constraints {
            no_unrelated_changes: true,
            no_unnecessary_refactor: true,
            preserve_public_interfaces: true,
            preserve_coding_style: true,
            analyze_before_modifying: true,
            build_after_modifying: true,
            run_tests: false,
            no_new_files: true,
            no_dependency_changes: true,
            no_unrelated_formatting: true,
            explain_root_cause: true,
            explain_modifications: true,
            list_remaining_risks: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_fix_config() {
        let c = Constraints::default();
        assert!(c.no_unrelated_changes);
        assert!(c.no_new_files);
        assert!(c.explain_root_cause);
        assert!(!c.run_tests);
        assert_eq!(c, Constraints::for_preset(Preset::Fix));
        assert_eq!(c, Constraints::for_preset(Preset::Debug));
    }

    #[test]
    fn get_set_roundtrip() {
        let mut c = Constraints::default();
        assert!(c.get(0));
        c.set(6, true);
        assert!(c.run_tests);
        assert!(c.get(6));
        c.set(0, false);
        assert!(!c.get(0));
        assert!(!c.get(99));
    }

    #[test]
    fn yolo_relaxes_constraints() {
        let c = Constraints::for_preset(Preset::Yolo);
        assert!(c.run_tests);
        assert!(!c.no_new_files);
        assert!(!c.no_unnecessary_refactor);
    }
}
