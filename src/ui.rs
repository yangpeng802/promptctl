use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Focus, Popup, TextArea};
use crate::model::{Constraints, Scope};

const COL: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;

fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().fg(COL).add_modifier(Modifier::BOLD)
    } else {
        Style::new()
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    if area.width < 46 || area.height < 14 {
        let msg = "Terminal too small for pm (need at least 46x14).";
        f.render_widget(Paragraph::new(msg), area);
        return;
    }
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);
    render_main(f, app, chunks[0]);
    render_status(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);

    match app.popup {
        Popup::None => {}
        Popup::Help => render_help(f),
        Popup::History => render_history(f, app),
    }
}

// ------------------------------------------------------------------ main --

fn render_main(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_task_lines = app.task.line_count().clamp(1, 4) as u16;
    let chunks = Layout::vertical([
        Constraint::Length(visible_task_lines + 2),
        Constraint::Fill(2),
        Constraint::Fill(1),
    ])
    .split(area);
    render_task(f, app, chunks[0]);

    let columns =
        Layout::horizontal([Constraint::Length(24), Constraint::Min(20)]).split(chunks[1]);
    render_preset_list(f, app, columns[0]);
    render_options(f, app, columns[1]);

    render_preview(f, app, chunks[2]);
}

fn render_task(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Task;
    let editing = focused && app.editing;
    let title = if editing {
        " Task — editing (Esc to finish) "
    } else if focused {
        " Task (Enter to edit) "
    } else {
        " Task "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style(focused));

    let max_lines = area.height.saturating_sub(2).max(1) as usize;
    let text = text_area_lines(&app.task, max_lines, editing);
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_preset_list(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Preset;
    let items: Vec<ListItem> = app
        .preset_entries
        .iter()
        .map(|entry| match entry {
            crate::app::PresetEntry::Builtin(p) => ListItem::new(Line::from(p.name())),
            crate::app::PresetEntry::Custom(i) => match app.customs.get(*i) {
                Some(c) => ListItem::new(Line::from(c.name.clone())),
                None => ListItem::new("?"),
            },
            crate::app::PresetEntry::Separator => {
                ListItem::new(Line::from(Span::styled("──────────", Style::new().fg(DIM))))
            }
        })
        .collect();
    app.preset_state.select(Some(app.preset_index));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Preset ")
                .border_style(border_style(focused)),
        )
        .highlight_symbol("> ")
        .highlight_style(Style::new().fg(COL).add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, area, &mut app.preset_state);
}

fn render_options(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(3),
    ])
    .split(area);
    let pair =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(rows[0]);
    render_permission(f, app, pair[0]);
    render_depth(f, app, pair[1]);
    render_scope(f, app, rows[1]);
    render_files(f, app, rows[2]);
    render_extra_rules(f, app, rows[3]);
    render_constraints(f, app, rows[4]);
}

fn render_permission(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Permission;
    let mut spans = vec![Span::raw(app.permission.to_string())];
    let effective = app.effective_permission();
    if effective != app.permission {
        spans.push(Span::styled(
            format!(" → {effective}"),
            Style::new().fg(Color::Yellow),
        ));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Permission ")
        .border_style(border_style(focused));
    f.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn render_depth(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Depth;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Depth ")
        .border_style(border_style(focused));
    f.render_widget(Paragraph::new(app.depth.to_string()).block(block), area);
}

fn render_scope(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Scope;
    let mut spans = vec![Span::styled(" Scope ", Style::new().fg(DIM))];
    for scope in Scope::ALL {
        let selected = scope == app.scope;
        let style = if selected {
            Style::new()
                .fg(if focused { COL } else { Color::White })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(DIM)
        };
        let bullet = if selected { "● " } else { "○ " };
        spans.push(Span::styled(bullet, style));
        spans.push(Span::styled(scope.label(), style));
        spans.push(Span::raw("   "));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_files(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Files;
    let editing = focused && app.editing;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Files (Scope = Files) ")
        .border_style(border_style(focused));
    let text: Vec<Line<'static>> = if app.files.is_empty() && !editing {
        vec![Line::from(Span::styled(
            "(none — one per line)",
            Style::new().fg(DIM),
        ))]
    } else {
        text_area_lines(&app.files, 1, editing)
    };
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_extra_rules(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::ExtraRules;
    let editing = focused && app.editing;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Extra rules (one per line) ")
        .border_style(border_style(focused));
    let text: Vec<Line<'static>> = if app.extra_rules.is_empty() && !editing {
        vec![Line::from(Span::styled("(none)", Style::new().fg(DIM)))]
    } else {
        text_area_lines(&app.extra_rules, 1, editing)
    };
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_constraints(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Constraints;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Constraints (Space to toggle) ")
        .border_style(border_style(focused));

    let per_col = Constraints::LABELS.len().div_ceil(2); // 7
    let mut lines: Vec<Line> = Vec::new();
    for row in 0..per_col {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..2 {
            let idx = col * per_col + row;
            if idx >= Constraints::LABELS.len() {
                continue;
            }
            let label = Constraints::LABELS[idx];
            let on = app.constraints.get(idx);
            let selected = focused && app.constraint_sel == idx;
            let mut style = if on {
                Style::new().fg(Color::Green)
            } else {
                Style::new().fg(DIM)
            };
            if selected {
                style = style
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD);
            }
            let checkbox = if on { "[x] " } else { "[ ] " };
            spans.push(Span::styled(format!("{checkbox}{label}"), style));
            let pad = 36usize.saturating_sub(4 + label.len());
            spans.push(Span::raw(" ".repeat(pad)));
        }
        lines.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_preview(f: &mut Frame, app: &mut App, area: Rect) {
    let inner = area.height.saturating_sub(2) as usize;
    let total = app.prompt.lines().count();
    app.preview_max_scroll = total.saturating_sub(inner) as u16;
    let scroll = app.preview_scroll.min(app.preview_max_scroll);
    let title = format!(
        " Preview — {} chars (PgUp/PgDn to scroll) ",
        app.prompt.chars().count()
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let paragraph = Paragraph::new(app.prompt.as_str())
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .block(block);
    f.render_widget(paragraph, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    if let Some(status) = &app.status {
        let style = if status.warn {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new().fg(Color::Green)
        };
        f.render_widget(
            Paragraph::new(Line::styled(status.text.clone(), style)),
            area,
        );
    } else if let Some(note) = app.effective_note() {
        f.render_widget(
            Paragraph::new(Line::styled(note, Style::new().fg(Color::Yellow))),
            area,
        );
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let _ = app;
    let hints = " Q Quit   ? Help   C Copy   H History   R Reset   Tab Focus   ↑↓ Select   ←→ Change   Space Toggle   Enter Edit";
    f.render_widget(
        Paragraph::new(Line::styled(hints, Style::new().fg(DIM))),
        area,
    );
}

// ---------------------------------------------------------------- popups --

fn render_help(f: &mut Frame) {
    let area = centered_rect(72, 70, f.area());
    f.render_widget(Clear, area);
    let lines = vec![
        Line::from(Span::styled(
            "pm — Prompt Maker",
            Style::new().fg(COL).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        key_line("Tab / Shift+Tab", "Move between sections"),
        key_line("↑ / ↓", "Move selection (preset, constraints)"),
        key_line("← / →", "Change option (permission, depth, scope)"),
        key_line("Space", "Toggle constraint / apply preset"),
        key_line("Enter", "Edit text field / apply preset"),
        key_line("c", "Copy prompt to clipboard"),
        key_line("h", "Open history (↑↓ select, Enter restore)"),
        key_line("r", "Reset to preset defaults"),
        key_line("PgUp / PgDn", "Scroll preview (also Ctrl+↑ / Ctrl+↓)"),
        key_line("Esc", "Exit editing — or quit when not editing"),
        key_line("q", "Quit"),
    ];
    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help (? / Esc to close) ")
                .border_style(border_style(true)),
        ),
        area,
    );
}

fn key_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {key:<16}"), Style::new().fg(COL)),
        Span::raw(desc.to_string()),
    ])
}

fn render_history(f: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 60, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" History (↑↓ select, Enter restore, Esc close) ")
        .border_style(border_style(true));

    if app.history.items.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "No history yet — generate and copy a prompt first.",
                Style::new().fg(DIM),
            )))
            .block(block),
            area,
        );
        return;
    }

    let width = area.width.saturating_sub(4) as usize;
    let items: Vec<ListItem> = app
        .history
        .items
        .iter()
        .map(|item| {
            let first_line = item.task.lines().next().unwrap_or("");
            let mut label = format!("[{}] {}", item.preset, first_line);
            if label.chars().count() > width {
                label = label.chars().take(width).collect::<String>() + "…";
            }
            ListItem::new(label)
        })
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_symbol("> ")
        .highlight_style(Style::new().fg(COL).add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    state.select(Some(
        app.history_sel
            .min(app.history.items.len().saturating_sub(1)),
    ));
    f.render_stateful_widget(list, area, &mut state);
}

// ----------------------------------------------------------------- text --

/// Render a [`TextArea`] as styled lines with a reversed "caret" block, plus
/// the cursor position relative to the visible window.
fn text_area_lines(ta: &TextArea, max_lines: usize, editing: bool) -> Vec<Line<'static>> {
    let (cursor_line, cursor_col) = ta.cursor_pos();
    let lines: Vec<String> = ta.text.split('\n').map(str::to_string).collect();
    if lines.is_empty() {
        return vec![caret_line("", cursor_col)];
    }
    let start = if lines.len() <= max_lines {
        0
    } else {
        (cursor_line + 1)
            .saturating_sub(max_lines)
            .min(lines.len() - max_lines)
    };
    let end = (start + max_lines).min(lines.len());
    lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if editing && start + i == cursor_line {
                caret_line(line, cursor_col)
            } else {
                Line::from(line.clone())
            }
        })
        .collect()
}

fn caret_line(line: &str, col: usize) -> Line<'static> {
    let chars: Vec<char> = line.chars().collect();
    let before: String = chars[..col.min(chars.len())].iter().collect();
    let at: String = chars
        .get(col)
        .map(|c| c.to_string())
        .unwrap_or_else(|| " ".to_string());
    let after: String = chars[(col + 1).min(chars.len())..].iter().collect();
    Line::from(vec![
        Span::raw(before),
        Span::styled(at, Style::new().add_modifier(Modifier::REVERSED)),
        Span::raw(after),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage(100 - percent_y - (100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage(100 - percent_x - (100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
