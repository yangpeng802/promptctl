use std::fmt;
use std::str::FromStr;

/// Where the agent is allowed to read and modify code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    Auto,
    CurrentFile,
    CurrentModule,
    SelectedFiles,
    WholeRepo,
}

impl Scope {
    pub const ALL: [Scope; 5] = [
        Scope::Auto,
        Scope::CurrentFile,
        Scope::CurrentModule,
        Scope::SelectedFiles,
        Scope::WholeRepo,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Scope::Auto => "auto",
            Scope::CurrentFile => "file",
            Scope::CurrentModule => "module",
            Scope::SelectedFiles => "files",
            Scope::WholeRepo => "repo",
        }
    }

    /// Short label used by the TUI radio row.
    pub fn label(self) -> &'static str {
        match self {
            Scope::Auto => "Auto",
            Scope::CurrentFile => "File",
            Scope::CurrentModule => "Module",
            Scope::SelectedFiles => "Files",
            Scope::WholeRepo => "Repo",
        }
    }

    pub fn cycle(self, step: isize) -> Self {
        let len = Self::ALL.len() as isize;
        let idx = Self::ALL
            .iter()
            .position(|s| *s == self)
            .map(|i| i as isize)
            .unwrap_or(0);
        Self::ALL[(idx + step).rem_euclid(len) as usize]
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

impl FromStr for Scope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" | "a" => Ok(Scope::Auto),
            "file" | "current_file" | "current-file" | "currentfile" => Ok(Scope::CurrentFile),
            "module" | "current_module" | "current-module" | "currentmodule" | "mod" => {
                Ok(Scope::CurrentModule)
            }
            "files" | "selected_files" | "selected-files" | "selected" => Ok(Scope::SelectedFiles),
            "repo" | "whole_repo" | "whole-repo" | "wholerepo" | "all" => Ok(Scope::WholeRepo),
            _ => Err(format!(
                "invalid scope '{s}' (expected auto|file|module|files|repo)"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_cycles() {
        assert_eq!("auto".parse(), Ok(Scope::Auto));
        assert_eq!("current_module".parse(), Ok(Scope::CurrentModule));
        assert_eq!("REPO".parse(), Ok(Scope::WholeRepo));
        assert!("zz".parse::<Scope>().is_err());
        assert_eq!(Scope::WholeRepo.cycle(1), Scope::Auto);
        assert_eq!(Scope::Auto.cycle(-1), Scope::WholeRepo);
        assert_eq!(Scope::SelectedFiles.to_string(), "files");
        assert_eq!(Scope::CurrentModule.label(), "Module");
    }
}
