pub mod builder;
pub mod templates;

pub use builder::{PromptBuilder, PromptRequest};

/// Prompt template language. `zh` is fully supported; `en` provides a
/// complete but plainer equivalent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Zh,
    En,
}

impl Lang {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Lang::En,
            _ => Lang::Zh,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_language() {
        assert_eq!(Lang::parse("en"), Lang::En);
        assert_eq!(Lang::parse("EN "), Lang::En);
        assert_eq!(Lang::parse("zh"), Lang::Zh);
        assert_eq!(Lang::parse("fr"), Lang::Zh);
        assert_eq!(Lang::parse("english"), Lang::En);
    }
}
