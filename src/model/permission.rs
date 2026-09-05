use std::fmt;
use std::str::FromStr;

/// How much the agent is allowed to modify, from L0 (read only) to L4 (yolo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionLevel {
    ReadOnly,
    Minimal,
    Scoped,
    Refactor,
    Yolo,
}

impl PermissionLevel {
    pub const ALL: [PermissionLevel; 5] = [
        PermissionLevel::ReadOnly,
        PermissionLevel::Minimal,
        PermissionLevel::Scoped,
        PermissionLevel::Refactor,
        PermissionLevel::Yolo,
    ];

    pub fn key(self) -> &'static str {
        match self {
            PermissionLevel::ReadOnly => "readonly",
            PermissionLevel::Minimal => "minimal",
            PermissionLevel::Scoped => "scoped",
            PermissionLevel::Refactor => "refactor",
            PermissionLevel::Yolo => "yolo",
        }
    }

    pub fn cycle(self, step: isize) -> Self {
        let len = Self::ALL.len() as isize;
        let idx = Self::ALL
            .iter()
            .position(|p| *p == self)
            .map(|i| i as isize)
            .unwrap_or(0);
        Self::ALL[(idx + step).rem_euclid(len) as usize]
    }
}

impl fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (level, name) = match self {
            PermissionLevel::ReadOnly => (0, "READ ONLY"),
            PermissionLevel::Minimal => (1, "MINIMAL"),
            PermissionLevel::Scoped => (2, "SCOPED"),
            PermissionLevel::Refactor => (3, "REFACTOR"),
            PermissionLevel::Yolo => (4, "YOLO"),
        };
        write!(f, "L{level} {name}")
    }
}

impl FromStr for PermissionLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "readonly" | "read_only" | "read-only" | "ro" | "l0" | "none" => {
                Ok(PermissionLevel::ReadOnly)
            }
            "minimal" | "min" | "l1" => Ok(PermissionLevel::Minimal),
            "scoped" | "scope" | "l2" => Ok(PermissionLevel::Scoped),
            "refactor" | "l3" => Ok(PermissionLevel::Refactor),
            "yolo" | "l4" => Ok(PermissionLevel::Yolo),
            _ => Err(format!(
                "invalid permission '{s}' (expected readonly|minimal|scoped|refactor|yolo)"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aliases() {
        assert_eq!("readonly".parse(), Ok(PermissionLevel::ReadOnly));
        assert_eq!("L1".parse(), Ok(PermissionLevel::Minimal));
        assert_eq!("Scoped".parse(), Ok(PermissionLevel::Scoped));
        assert_eq!(" l3 ".parse(), Ok(PermissionLevel::Refactor));
        assert_eq!("YOLO".parse(), Ok(PermissionLevel::Yolo));
        assert!("nonsense".parse::<PermissionLevel>().is_err());
    }

    #[test]
    fn display_and_cycle() {
        assert_eq!(PermissionLevel::ReadOnly.to_string(), "L0 READ ONLY");
        assert_eq!(PermissionLevel::Yolo.to_string(), "L4 YOLO");
        assert_eq!(PermissionLevel::ReadOnly.cycle(-1), PermissionLevel::Yolo);
        assert_eq!(PermissionLevel::Yolo.cycle(1), PermissionLevel::ReadOnly);
        assert_eq!(PermissionLevel::Minimal.cycle(1), PermissionLevel::Scoped);
    }
}
