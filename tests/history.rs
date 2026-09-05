use pm::history::{History, HistoryItem, MAX_ITEMS};
use pm::model::Constraints;

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
fn keeps_last_20_newest_first() {
    let mut history = History::default();
    for i in 0..(MAX_ITEMS + 5) {
        history.push(item(&format!("任务 {i}")));
    }
    assert_eq!(history.items.len(), MAX_ITEMS);
    assert_eq!(history.items[0].task, format!("任务 {}", MAX_ITEMS + 4));
}

#[test]
fn skips_consecutive_duplicates() {
    let mut history = History::default();
    history.push(item("a"));
    history.push(item("a"));
    history.push(item("b"));
    history.push(item("a"));
    assert_eq!(history.items.len(), 3);
    assert_eq!(history.items[0].task, "a");
}

#[test]
fn serializes_to_history_json_shape() {
    let mut history = History::default();
    history.push(HistoryItem {
        task: "分析 getUserByName 调用链".to_string(),
        preset: "trace".to_string(),
        permission: "readonly".to_string(),
        depth: "deep".to_string(),
        scope: "repo".to_string(),
        extra_rules: vec!["不要使用 unsafe".to_string()],
        constraints: Constraints::default(),
        selected_files: vec!["src/main.rs".to_string()],
    });
    let json = serde_json::to_string(&history.items).unwrap();
    let parsed: Vec<HistoryItem> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, history.items);
    assert_eq!(parsed[0].selected_files, vec!["src/main.rs"]);
    // Fields required by the spec are present in the JSON object.
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let first = &value[0];
    for key in [
        "task",
        "preset",
        "permission",
        "depth",
        "scope",
        "extra_rules",
        "constraints",
        "selected_files",
    ] {
        assert!(first.get(key).is_some(), "missing field {key}");
    }
}

#[test]
fn old_records_without_new_fields_load_with_defaults() {
    // history.json written before constraints/selected_files existed.
    let json = r#"[{
        "task": "旧任务",
        "preset": "fix",
        "permission": "minimal",
        "depth": "normal",
        "scope": "auto",
        "extra_rules": []
    }]"#;
    let items: Vec<HistoryItem> = serde_json::from_str(json).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].constraints, Constraints::default());
    assert!(items[0].selected_files.is_empty());
}
