use std::fmt;
use std::str::FromStr;

/// How deeply the agent is expected to analyze before acting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Depth {
    Quick,
    Normal,
    Deep,
}

impl Depth {
    pub const ALL: [Depth; 3] = [Depth::Quick, Depth::Normal, Depth::Deep];

    pub fn key(self) -> &'static str {
        match self {
            Depth::Quick => "quick",
            Depth::Normal => "normal",
            Depth::Deep => "deep",
        }
    }

    pub fn cycle(self, step: isize) -> Self {
        let len = Self::ALL.len() as isize;
        let idx = Self::ALL
            .iter()
            .position(|d| *d == self)
            .map(|i| i as isize)
            .unwrap_or(0);
        Self::ALL[(idx + step).rem_euclid(len) as usize]
    }
}

impl fmt::Display for Depth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Depth::Quick => "QUICK",
            Depth::Normal => "NORMAL",
            Depth::Deep => "DEEP",
        };
        f.write_str(name)
    }
}

impl FromStr for Depth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "quick" | "q" => Ok(Depth::Quick),
            "normal" | "default" | "n" => Ok(Depth::Normal),
            "deep" | "d" => Ok(Depth::Deep),
            _ => Err(format!("invalid depth '{s}' (expected quick|normal|deep)")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_cycles() {
        assert_eq!("quick".parse(), Ok(Depth::Quick));
        assert_eq!("DEEP".parse(), Ok(Depth::Deep));
        assert_eq!("normal".parse(), Ok(Depth::Normal));
        assert!("xx".parse::<Depth>().is_err());
        assert_eq!(Depth::Quick.cycle(-1), Depth::Deep);
        assert_eq!(Depth::Deep.cycle(1), Depth::Quick);
        assert_eq!(Depth::Normal.to_string(), "NORMAL");
    }
}
