use std::fs;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
use crate::model::Constraints;

/// History keeps the last 20 generations (task + parameters only, never code).
pub const MAX_ITEMS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryItem {
    pub task: String,
    pub preset: String,
    pub permission: String,
    pub depth: String,
    pub scope: String,
    #[serde(default)]
    pub extra_rules: Vec<String>,
    /// Constraints in effect when the prompt was generated. Defaults to the
    /// FIX baseline for records written before this field existed.
    #[serde(default)]
    pub constraints: Constraints,
    /// Files the generation was limited to. Empty for older records.
    #[serde(default)]
    pub selected_files: Vec<String>,
}

#[derive(Debug, Default)]
pub struct History {
    pub items: Vec<HistoryItem>,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("cannot create history directory: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot serialize history: {0}")]
    Json(#[from] serde_json::Error),
}

impl History {
    /// Best-effort load; any problem yields an empty history.
    pub fn load() -> History {
        let Some(path) = Config::history_path() else {
            return History::default();
        };
        let Ok(text) = fs::read_to_string(path) else {
            return History::default();
        };
        match serde_json::from_str::<Vec<HistoryItem>>(&text) {
            Ok(items) => History { items },
            Err(_) => History::default(),
        }
    }

    /// Newest first, capped at [`MAX_ITEMS`], consecutive duplicates skipped.
    pub fn push(&mut self, item: HistoryItem) {
        if self.items.first().is_some_and(|first| *first == item) {
            return;
        }
        self.items.insert(0, item);
        self.items.truncate(MAX_ITEMS);
    }

    /// Best-effort save; callers decide whether to surface the error.
    pub fn save(&self) -> Result<(), HistoryError> {
        let Some(path) = Config::history_path() else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, serde_json::to_string_pretty(&self.items)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(task: &str) -> HistoryItem {
        HistoryItem {
            task: task.to_string(),
            preset: "fix".to_string(),
            permission: "minimal".to_string(),
            depth: "normal".to_string(),
            scope: "auto".to_string(),
            extra_rules: vec![],
            constraints: Constraints::default(),
            selected_files: vec![],
        }
    }

    #[test]
    fn caps_at_max_items_newest_first() {
        let mut h = History::default();
        for i in 0..(MAX_ITEMS + 10) {
            h.push(item(&format!("task-{i}")));
        }
        assert_eq!(h.items.len(), MAX_ITEMS);
        assert_eq!(h.items[0].task, format!("task-{}", MAX_ITEMS + 9));
        assert_eq!(h.items[MAX_ITEMS - 1].task, "task-10");
    }

    #[test]
    fn skips_consecutive_duplicates() {
        let mut h = History::default();
        h.push(item("a"));
        h.push(item("a"));
        h.push(item("b"));
        h.push(item("a"));
        assert_eq!(h.items.len(), 3);
        assert_eq!(h.items[0].task, "a");
    }

    #[test]
    fn roundtrips_through_json() {
        let mut h = History::default();
        h.push(HistoryItem {
            task: "修复 data race".to_string(),
            preset: "debug".to_string(),
            permission: "readonly".to_string(),
            depth: "deep".to_string(),
            scope: "repo".to_string(),
            extra_rules: vec!["必须兼容现有构建环境".to_string()],
            constraints: Constraints::default(),
            selected_files: vec!["src/main.rs".to_string()],
        });
        let json = serde_json::to_string_pretty(&h.items).unwrap();
        let items: Vec<HistoryItem> = serde_json::from_str(&json).unwrap();
        assert_eq!(items, h.items);
    }
}
