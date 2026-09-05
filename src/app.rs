use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::clipboard::Clipboard;
use crate::config::{Config, CustomPreset};
use crate::history::{History, HistoryItem};
use crate::model::{effective_permission, Constraints, Depth, PermissionLevel, Preset, Scope};
use crate::prompt::{PromptBuilder, PromptRequest};

const PREVIEW_STEP: u16 = 5;
const STATUS_TTL: Duration = Duration::from_secs(2);

/// Tab order of the main page sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Task,
    Preset,
    Permission,
    Depth,
    Scope,
    Files,
    Constraints,
    ExtraRules,
}

pub const FOCUS_ORDER: [Focus; 8] = [
    Focus::Task,
    Focus::Preset,
    Focus::Permission,
    Focus::Depth,
    Focus::Scope,
    Focus::Files,
    Focus::Constraints,
    Focus::ExtraRules,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popup {
    None,
    Help,
    History,
}

/// Preset the TUI should start with, e.g. when launched as `pm fix` with no task.
#[derive(Debug, Clone)]
pub enum PresetHint {
    None,
    Builtin(Preset),
    Custom(String),
}

pub struct StatusMessage {
    pub text: String,
    pub warn: bool,
    pub until: Instant,
}

/// Minimal multiline text editor (cursor is a char index, CJK safe).
#[derive(Debug, Clone, Default)]
pub struct TextArea {
    pub text: String,
    pub cursor: usize,
}

impl TextArea {
    pub fn from_text(s: &str) -> Self {
        TextArea {
            text: s.to_string(),
            cursor: s.chars().count(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    fn chars(&self) -> Vec<char> {
        self.text.chars().collect()
    }

    pub fn insert(&mut self, c: char) {
        let mut v = self.chars();
        let i = self.cursor.min(v.len());
        v.insert(i, c);
        self.cursor = i + 1;
        self.text = v.into_iter().collect();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let mut v = self.chars();
            let i = (self.cursor - 1).min(v.len().saturating_sub(1));
            v.remove(i);
            self.cursor = i;
            self.text = v.into_iter().collect();
        }
    }

    pub fn delete(&mut self) {
        let mut v = self.chars();
        if self.cursor < v.len() {
            v.remove(self.cursor);
            self.text = v.into_iter().collect();
        }
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.text.chars().count());
    }

    pub fn move_home(&mut self) {
        let (line, _) = self.cursor_pos();
        self.cursor = self.index_at(line, 0);
    }

    pub fn move_end(&mut self) {
        let (line, _) = self.cursor_pos();
        self.cursor = self.index_at(line, self.line_len(line));
    }

    pub fn move_up(&mut self) {
        let (line, col) = self.cursor_pos();
        if line == 0 {
            return;
        }
        let target = line - 1;
        self.cursor = self.index_at(target, col.min(self.line_len(target)));
    }

    pub fn move_down(&mut self) {
        let (line, col) = self.cursor_pos();
        if line + 1 >= self.line_count() {
            return;
        }
        let target = line + 1;
        self.cursor = self.index_at(target, col.min(self.line_len(target)));
    }

    pub fn move_caret_to_end(&mut self) {
        self.cursor = self.text.chars().count();
    }

    /// (line index, column in chars) of the cursor.
    pub fn cursor_pos(&self) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;
        for (i, ch) in self.text.chars().enumerate() {
            if i >= self.cursor {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    pub fn line_len(&self, line: usize) -> usize {
        self.text
            .split('\n')
            .nth(line)
            .map(|s| s.chars().count())
            .unwrap_or(0)
    }

    fn index_at(&self, target_line: usize, target_col: usize) -> usize {
        let mut line = 0usize;
        let mut col = 0usize;
        let mut idx = 0usize;
        for ch in self.text.chars() {
            if line == target_line && col == target_col {
                return idx;
            }
            if ch == '\n' {
                if line > target_line {
                    return idx;
                }
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            idx += 1;
        }
        idx
    }
}

/// Which preset list entry is highlighted.
#[derive(Debug, Clone)]
pub enum PresetEntry {
    Builtin(Preset),
    Custom(usize),
    Separator,
}

impl PresetEntry {
    pub fn display_name(&self, customs: &[CustomPreset]) -> String {
        match self {
            PresetEntry::Builtin(p) => p.name().to_string(),
            PresetEntry::Custom(i) => customs
                .get(*i)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "?".to_string()),
            PresetEntry::Separator => String::new(),
        }
    }
}

/// Fields the user changed by hand; preset switches must not clobber them.
#[derive(Debug, Clone, Copy, Default)]
pub struct Dirty {
    pub permission: bool,
    pub depth: bool,
    pub scope: bool,
    pub constraints: bool,
}

pub struct App {
    pub task: TextArea,
    pub files: TextArea,
    pub extra_rules: TextArea,

    pub preset_entries: Vec<PresetEntry>,
    pub preset_index: usize,
    pub preset_state: ListState,
    pub permission: PermissionLevel,
    pub depth: Depth,
    pub scope: Scope,
    pub constraints: Constraints,
    pub constraint_sel: usize,

    pub focus: Focus,
    pub editing: bool,
    pub popup: Popup,
    pub history_sel: usize,

    pub preview_scroll: u16,
    pub preview_max_scroll: u16,
    pub status: Option<StatusMessage>,

    pub dirty: Dirty,
    pub config: Config,
    pub customs: Vec<CustomPreset>,
    pub history: History,
    pub clipboard: Option<Clipboard>,
    pub should_quit: bool,
    pub prompt: String,
}

impl App {
    pub fn new(config: Config, history: History, hint: PresetHint) -> Self {
        let customs = config.resolved_customs();
        let mut preset_entries: Vec<PresetEntry> = Preset::ALL
            .iter()
            .map(|&p| PresetEntry::Builtin(p))
            .collect();
        preset_entries.push(PresetEntry::Separator);
        preset_entries.extend((0..customs.len()).map(PresetEntry::Custom));

        let mut app = App {
            task: TextArea::default(),
            files: TextArea::default(),
            extra_rules: TextArea::default(),
            preset_index: 0,
            preset_entries,
            preset_state: ListState::default(),
            permission: config.default_permission(),
            depth: config.default_depth(),
            scope: config.default_scope(),
            constraints: config.constraints,
            constraint_sel: 0,
            focus: Focus::Task,
            editing: false,
            popup: Popup::None,
            history_sel: 0,
            preview_scroll: 0,
            preview_max_scroll: 0,
            status: None,
            dirty: Dirty::default(),
            customs,
            config,
            history,
            clipboard: Clipboard::new(),
            should_quit: false,
            prompt: String::new(),
        };

        match &hint {
            PresetHint::None => {
                // Startup values come from the config; just highlight its preset.
                app.preset_index = app.index_of_builtin(app.config.default_preset());
            }
            PresetHint::Builtin(preset) => {
                app.preset_index = app.index_of_builtin(*preset);
                app.apply_preset_defaults(false);
            }
            PresetHint::Custom(name) => {
                if let Some(i) = app.index_of_custom(name) {
                    app.preset_index = i;
                    app.apply_preset_defaults(false);
                }
            }
        }
        app.sync();
        app
    }

    // ------------------------------------------------------------- preset --

    fn index_of_builtin(&self, preset: Preset) -> usize {
        self.preset_entries
            .iter()
            .position(|e| matches!(e, PresetEntry::Builtin(p) if *p == preset))
            .unwrap_or(0)
    }

    fn index_of_custom(&self, name: &str) -> Option<usize> {
        self.preset_entries
            .iter()
            .position(|e| matches!(e, PresetEntry::Custom(i) if self.customs.get(*i).is_some_and(|c| c.name == name)))
    }

    fn find_preset_index(&self, key: &str) -> Option<usize> {
        self.preset_entries.iter().position(|e| match e {
            PresetEntry::Builtin(p) => p.key() == key,
            PresetEntry::Custom(i) => self.customs.get(*i).is_some_and(|c| c.name == key),
            PresetEntry::Separator => false,
        })
    }

    pub fn current_base_preset(&self) -> Preset {
        match self.preset_entries.get(self.preset_index) {
            Some(PresetEntry::Builtin(p)) => *p,
            Some(PresetEntry::Custom(i)) => {
                self.customs.get(*i).map(|c| c.base).unwrap_or(Preset::Fix)
            }
            _ => Preset::Fix,
        }
    }

    /// Builtin key or custom preset name, used for history entries.
    pub fn current_preset_key(&self) -> String {
        match self.preset_entries.get(self.preset_index) {
            Some(PresetEntry::Builtin(p)) => p.key().to_string(),
            Some(PresetEntry::Custom(i)) => self
                .customs
                .get(*i)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "fix".to_string()),
            _ => "fix".to_string(),
        }
    }

    /// Apply the highlighted preset's defaults. With `force = false` the
    /// user's manual edits (dirty flags) are preserved.
    fn apply_preset_defaults(&mut self, force: bool) {
        let Some(entry) = self.preset_entries.get(self.preset_index).cloned() else {
            return;
        };
        let (PresetEntry::Builtin(_) | PresetEntry::Custom(_)) = entry else {
            return;
        };
        let (base, perm, depth, scope, rules) = match &entry {
            PresetEntry::Builtin(p) => (
                *p,
                p.default_permission(),
                p.default_depth(),
                p.default_scope(),
                None,
            ),
            PresetEntry::Custom(i) => {
                let Some(c) = self.customs.get(*i) else {
                    return;
                };
                (
                    c.base,
                    c.permission.unwrap_or_else(|| c.base.default_permission()),
                    c.depth.unwrap_or_else(|| c.base.default_depth()),
                    c.scope.unwrap_or_else(|| c.base.default_scope()),
                    Some(c.extra_rules.clone()),
                )
            }
            PresetEntry::Separator => return,
        };
        if force || !self.dirty.permission {
            self.permission = perm;
        }
        if force || !self.dirty.depth {
            self.depth = depth;
        }
        if force || !self.dirty.scope {
            self.scope = scope;
        }
        if force || !self.dirty.constraints {
            self.constraints = base.default_constraints();
        }
        if let Some(rules) = rules {
            self.extra_rules = TextArea::from_text(&rules.join("\n"));
        }
        self.set_status(
            format!("✓ Loaded preset {}", entry.display_name(&self.customs)),
            false,
        );
    }

    fn reset(&mut self) {
        let name = self
            .preset_entries
            .get(self.preset_index)
            .map(|e| e.display_name(&self.customs))
            .unwrap_or_default();
        self.apply_preset_defaults(true);
        self.dirty = Dirty::default();
        self.set_status(format!("✓ Reset to {name} defaults"), false);
    }

    // ------------------------------------------------------------ prompt --

    pub fn effective_permission(&self) -> PermissionLevel {
        effective_permission(self.current_base_preset(), self.permission)
    }

    /// Shown in the status line when the preset caps the chosen permission.
    pub fn effective_note(&self) -> Option<String> {
        let eff = self.effective_permission();
        (eff != self.permission).then(|| format!("Effective permission: {eff}"))
    }

    pub fn sync(&mut self) {
        let request = PromptRequest {
            task: self.task.text.clone(),
            preset: self.current_base_preset(),
            permission: self.permission,
            depth: self.depth,
            scope: self.scope,
            selected_files: parse_entries(&self.files.text),
            constraints: self.constraints,
            extra_rules: parse_lines(&self.extra_rules.text),
            language: self.config.lang(),
        };
        self.prompt = PromptBuilder::build(&request);
    }

    pub fn copy_prompt(&mut self) {
        self.sync();
        let prompt = self.prompt.clone();
        match self.clipboard.as_mut() {
            Some(clipboard) => match clipboard.set_text(&prompt) {
                Ok(()) => {
                    self.set_status("✓ Prompt copied", false);
                    self.record_history();
                }
                Err(_) => self.set_status("⚠ Clipboard unavailable", true),
            },
            None => self.set_status("⚠ Clipboard unavailable", true),
        }
    }

    fn record_history(&mut self) {
        let item = HistoryItem {
            task: self.task.text.trim().to_string(),
            preset: self.current_preset_key(),
            permission: self.effective_permission().key().to_string(),
            depth: self.depth.key().to_string(),
            scope: self.scope.key().to_string(),
            extra_rules: parse_lines(&self.extra_rules.text),
        };
        self.history.push(item);
        let _ = self.history.save();
    }

    fn restore_history(&mut self) {
        let Some(item) = self.history.items.get(self.history_sel).cloned() else {
            return;
        };
        self.task = TextArea::from_text(&item.task);
        if let Some(i) = self.find_preset_index(&item.preset) {
            self.preset_index = i;
        }
        if let Ok(p) = item.permission.parse::<PermissionLevel>() {
            self.permission = p;
        }
        if let Ok(d) = item.depth.parse::<Depth>() {
            self.depth = d;
        }
        if let Ok(s) = item.scope.parse::<Scope>() {
            self.scope = s;
        }
        self.extra_rules = TextArea::from_text(&item.extra_rules.join("\n"));
        self.preview_scroll = 0;
        self.set_status("✓ History restored", false);
    }

    // ------------------------------------------------------------ status --

    /// Expire the status message. Returns true when visible state changed
    /// (the caller must redraw).
    pub fn tick(&mut self) -> bool {
        if self
            .status
            .as_ref()
            .is_some_and(|s| Instant::now() >= s.until)
        {
            self.status = None;
            return true;
        }
        false
    }

    fn set_status(&mut self, text: impl Into<String>, warn: bool) {
        self.status = Some(StatusMessage {
            text: text.into(),
            warn,
            until: Instant::now() + STATUS_TTL,
        });
    }

    // ------------------------------------------------------------- input --

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.handle_key_inner(key);
        self.sync();
    }

    fn handle_key_inner(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Up => {
                    self.scroll_up();
                    return;
                }
                KeyCode::Down => {
                    self.scroll_down();
                    return;
                }
                _ => {}
            }
        }

        match self.popup {
            Popup::Help => {
                self.popup = Popup::None;
                return;
            }
            Popup::History => {
                self.handle_history_key(key);
                return;
            }
            Popup::None => {}
        }

        if self.editing {
            self.handle_edit_key(key);
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') => self.popup = Popup::Help,
            KeyCode::Char('c') | KeyCode::Char('C') => self.copy_prompt(),
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.history_sel = 0;
                self.popup = Popup::History;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => self.reset(),
            KeyCode::Tab => self.focus_next(),
            KeyCode::BackTab => self.focus_prev(),
            KeyCode::Up => self.nav_up(),
            KeyCode::Down => self.nav_down(),
            KeyCode::Left => self.nav_left(),
            KeyCode::Right => self.nav_right(),
            KeyCode::Char(' ') => self.toggle(),
            KeyCode::Enter => self.activate(),
            KeyCode::PageUp => self.scroll_up(),
            KeyCode::PageDown => self.scroll_down(),
            _ => {}
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        if key
            .modifiers
            .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return;
        }
        match key.code {
            KeyCode::Esc => self.editing = false,
            KeyCode::Tab => {
                self.editing = false;
                self.focus_next();
            }
            KeyCode::Enter => {
                if let Some(ta) = self.active_text_area() {
                    ta.insert('\n');
                }
            }
            KeyCode::Backspace => {
                if let Some(ta) = self.active_text_area() {
                    ta.backspace();
                }
            }
            KeyCode::Delete => {
                if let Some(ta) = self.active_text_area() {
                    ta.delete();
                }
            }
            KeyCode::Left => {
                if let Some(ta) = self.active_text_area() {
                    ta.move_left();
                }
            }
            KeyCode::Right => {
                if let Some(ta) = self.active_text_area() {
                    ta.move_right();
                }
            }
            KeyCode::Up => {
                if let Some(ta) = self.active_text_area() {
                    ta.move_up();
                }
            }
            KeyCode::Down => {
                if let Some(ta) = self.active_text_area() {
                    ta.move_down();
                }
            }
            KeyCode::Home => {
                if let Some(ta) = self.active_text_area() {
                    ta.move_home();
                }
            }
            KeyCode::End => {
                if let Some(ta) = self.active_text_area() {
                    ta.move_end();
                }
            }
            KeyCode::Char(c) => {
                if let Some(ta) = self.active_text_area() {
                    ta.insert(c);
                }
            }
            KeyCode::PageUp => self.scroll_up(),
            KeyCode::PageDown => self.scroll_down(),
            _ => {}
        }
    }

    fn handle_history_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc
            | KeyCode::Char('q')
            | KeyCode::Char('h')
            | KeyCode::Char('H')
            | KeyCode::Char('?') => self.popup = Popup::None,
            KeyCode::Up => self.history_sel = self.history_sel.saturating_sub(1),
            KeyCode::Down => {
                if self.history_sel + 1 < self.history.items.len() {
                    self.history_sel += 1;
                }
            }
            KeyCode::Enter => self.restore_history(),
            _ => {}
        }
    }

    fn active_text_area(&mut self) -> Option<&mut TextArea> {
        match self.focus {
            Focus::Task => Some(&mut self.task),
            Focus::Files => Some(&mut self.files),
            Focus::ExtraRules => Some(&mut self.extra_rules),
            _ => None,
        }
    }

    // ------------------------------------------------------------- focus --

    fn focus_next(&mut self) {
        let i = FOCUS_ORDER
            .iter()
            .position(|f| *f == self.focus)
            .unwrap_or(0);
        self.focus = FOCUS_ORDER[(i + 1) % FOCUS_ORDER.len()];
    }

    fn focus_prev(&mut self) {
        let i = FOCUS_ORDER
            .iter()
            .position(|f| *f == self.focus)
            .unwrap_or(0);
        self.focus = FOCUS_ORDER[(i + FOCUS_ORDER.len() - 1) % FOCUS_ORDER.len()];
    }

    fn nav_up(&mut self) {
        match self.focus {
            Focus::Preset => self.move_preset_selection(-1),
            Focus::Constraints => self.constraint_sel = self.constraint_sel.saturating_sub(1),
            _ => {}
        }
    }

    fn nav_down(&mut self) {
        match self.focus {
            Focus::Preset => self.move_preset_selection(1),
            Focus::Constraints if self.constraint_sel + 1 < Constraints::LABELS.len() => {
                self.constraint_sel += 1;
            }
            _ => {}
        }
    }

    fn move_preset_selection(&mut self, step: isize) {
        let mut idx = self.preset_index as isize;
        let last = self.preset_entries.len() as isize - 1;
        loop {
            idx += step;
            if idx < 0 {
                idx = 0;
                break;
            }
            if idx > last {
                idx = last;
                break;
            }
            if !matches!(self.preset_entries[idx as usize], PresetEntry::Separator) {
                break;
            }
        }
        self.preset_index = idx as usize;
    }

    fn nav_left(&mut self) {
        match self.focus {
            Focus::Permission => {
                self.permission = self.permission.cycle(-1);
                self.dirty.permission = true;
            }
            Focus::Depth => {
                self.depth = self.depth.cycle(-1);
                self.dirty.depth = true;
            }
            Focus::Scope => {
                self.scope = self.scope.cycle(-1);
                self.dirty.scope = true;
            }
            _ => {}
        }
    }

    fn nav_right(&mut self) {
        match self.focus {
            Focus::Permission => {
                self.permission = self.permission.cycle(1);
                self.dirty.permission = true;
            }
            Focus::Depth => {
                self.depth = self.depth.cycle(1);
                self.dirty.depth = true;
            }
            Focus::Scope => {
                self.scope = self.scope.cycle(1);
                self.dirty.scope = true;
            }
            _ => {}
        }
    }

    fn toggle(&mut self) {
        match self.focus {
            Focus::Constraints => {
                let value = self.constraints.get(self.constraint_sel);
                self.constraints.set(self.constraint_sel, !value);
                self.dirty.constraints = true;
            }
            Focus::Preset => self.apply_preset_defaults(false),
            _ => {}
        }
    }

    fn activate(&mut self) {
        match self.focus {
            Focus::Task | Focus::Files | Focus::ExtraRules => self.editing = true,
            Focus::Preset => self.apply_preset_defaults(false),
            Focus::Constraints => self.toggle(),
            _ => {}
        }
    }

    // ------------------------------------------------------------ scroll --

    fn scroll_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(PREVIEW_STEP);
    }

    fn scroll_down(&mut self) {
        self.preview_scroll = (self.preview_scroll + PREVIEW_STEP).min(self.preview_max_scroll);
    }
}

/// Split on newlines and commas (for file lists), trimmed, no empties.
fn parse_entries(text: &str) -> Vec<String> {
    text.split(['\n', ','])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// One extra rule per line.
fn parse_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ta(s: &str) -> TextArea {
        TextArea::from_text(s)
    }

    #[test]
    fn text_area_editing() {
        let mut t = ta("ab");
        t.insert('c');
        assert_eq!(t.text, "abc");
        t.backspace();
        assert_eq!(t.text, "ab");
        t.move_left();
        t.insert('X');
        assert_eq!(t.text, "aXb");
        t.delete();
        assert_eq!(t.text, "aX");
        t.backspace(); // removes 'X' (cursor was after it)
        t.backspace(); // removes 'a'
        t.backspace(); // at start, no-op
        assert_eq!(t.text, "");
        assert_eq!(t.cursor, 0);
    }

    #[test]
    fn text_area_multiline_navigation() {
        let mut t = ta("ab\ncdef\ng");
        t.move_caret_to_end();
        assert_eq!(t.cursor_pos(), (2, 1)); // after 'g'
        t.move_up();
        assert_eq!(t.cursor_pos(), (1, 1)); // column preserved, line "cdef"
        t.move_home();
        assert_eq!(t.cursor_pos(), (1, 0));
        t.move_up();
        assert_eq!(t.cursor_pos(), (0, 0));
        t.move_end();
        assert_eq!(t.cursor_pos(), (0, 2));
        t.move_down();
        assert_eq!(t.cursor_pos(), (1, 2));
        t.insert('\n');
        assert_eq!(t.text, "ab\ncd\nef\ng");
    }

    #[test]
    fn text_area_cjk_cursor() {
        let mut t = ta("分析问题");
        t.move_left();
        t.move_left();
        t.insert('X');
        assert_eq!(t.text, "分析X问题");
        t.backspace();
        assert_eq!(t.text, "分析问题");
    }

    #[test]
    fn parse_helpers() {
        assert_eq!(
            parse_entries("src/a.cpp, src/b.hpp\n\nc"),
            vec!["src/a.cpp", "src/b.hpp", "c"]
        );
        assert_eq!(parse_lines("  a \n\nb\n"), vec!["a", "b"]);
        assert!(parse_entries(" , ,\n").is_empty());
    }

    #[test]
    fn preset_entries_skip_separator() {
        let mut app = App::new(Config::default(), History::default(), PresetHint::None);
        app.preset_index = 0; // ANALYZE
        app.move_preset_selection(1);
        assert!(matches!(
            app.preset_entries[app.preset_index],
            PresetEntry::Builtin(Preset::Fix)
        ));
        let last = app.preset_entries.len() - 1;
        app.preset_index = last;
        app.move_preset_selection(1); // stays on the last entry
        assert_eq!(app.preset_index, last);
        app.move_preset_selection(-1);
        assert!(!matches!(
            app.preset_entries[app.preset_index],
            PresetEntry::Separator
        ));
    }

    #[test]
    fn dirty_flags_survive_preset_switch() {
        let mut app = App::new(Config::default(), History::default(), PresetHint::None);
        // FIX defaults selected at startup (config defaults == FIX).
        assert_eq!(app.permission, PermissionLevel::Minimal);
        assert!(app.constraints.no_new_files);

        // Manual changes mark fields dirty.
        app.focus = Focus::Permission;
        app.nav_right();
        assert_eq!(app.permission, PermissionLevel::Scoped);
        assert!(app.dirty.permission);
        app.focus = Focus::Constraints;
        app.constraint_sel = 6; // Run tests
        app.toggle();
        assert!(app.constraints.run_tests);

        // Switching preset keeps manual edits but refreshes the rest.
        app.focus = Focus::Preset;
        app.preset_index = app.index_of_builtin(Preset::Analyze);
        app.apply_preset_defaults(false);
        assert_eq!(app.permission, PermissionLevel::Scoped, "dirty field kept");
        assert!(app.constraints.run_tests, "dirty constraints kept");
        assert_eq!(app.depth, Depth::Normal); // refreshed from preset default

        // Force reset reapplies everything.
        app.reset();
        assert_eq!(app.permission, PermissionLevel::ReadOnly);
        assert!(!app.constraints.run_tests);
        assert!(!app.dirty.permission);
    }

    #[test]
    fn effective_note_reports_cap() {
        let mut app = App::new(Config::default(), History::default(), PresetHint::None);
        app.preset_index = app.index_of_builtin(Preset::Analyze);
        app.apply_preset_defaults(false);
        app.permission = PermissionLevel::Yolo;
        assert_eq!(app.effective_permission(), PermissionLevel::ReadOnly);
        assert!(app
            .effective_note()
            .is_some_and(|n| n.contains("READ ONLY")));
    }

    #[test]
    fn custom_preset_entry_applies() {
        let mut config = Config::default();
        config.custom_presets.push(crate::config::CustomPresetRaw {
            name: "legacy-fix".to_string(),
            base: "fix".to_string(),
            permission: Some("minimal".to_string()),
            depth: Some("deep".to_string()),
            scope: Some("module".to_string()),
            extra_rules: vec!["必须兼容现有构建环境".to_string()],
        });
        let mut app = App::new(config, History::default(), PresetHint::None);
        let idx = app
            .index_of_custom("legacy-fix")
            .expect("custom preset listed");
        app.preset_index = idx;
        app.apply_preset_defaults(false);
        app.sync();
        assert_eq!(app.depth, Depth::Deep);
        assert_eq!(app.scope, Scope::CurrentModule);
        assert_eq!(app.extra_rules.text, "必须兼容现有构建环境");
        assert!(!app.prompt.contains("legacy-fix"));
        assert!(app.prompt.contains("必须兼容现有构建环境"));
    }
}
