use crate::model::{effective_permission, Constraints, Depth, PermissionLevel, Preset, Scope};
use crate::prompt::templates as t;
use crate::prompt::Lang;

/// Everything needed to generate one prompt.
#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub task: String,
    pub preset: Preset,
    pub permission: PermissionLevel,
    pub depth: Depth,
    pub scope: Scope,
    pub selected_files: Vec<String>,
    pub constraints: Constraints,
    pub extra_rules: Vec<String>,
    pub language: Lang,
}

/// Assembles the final prompt from per-preset / per-permission / per-depth
/// / per-scope templates, skipping lines already implied by the permission
/// level so the output stays tight.
pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build(req: &PromptRequest) -> String {
        let lang = req.language;
        let perm = effective_permission(req.preset, req.permission);
        let read_only = perm == PermissionLevel::ReadOnly;

        let mut sections: Vec<String> = Vec::with_capacity(9);
        sections.push(section(t::h_task(lang), task_body(req)));
        sections.push(section(
            t::h_work(lang),
            t::work_mode(lang, req.preset, perm),
        ));
        sections.push(section(
            t::h_analysis(lang),
            t::analysis(lang, req.preset, req.depth),
        ));
        sections.push(section(
            t::h_permission(lang),
            t::permission_text(lang, perm).to_string(),
        ));
        sections.push(section(t::h_scope(lang), scope_body(req)));
        sections.push(section(t::h_constraints(lang), constraints_body(req, perm)));
        if !req.extra_rules.is_empty() {
            sections.push(section(t::h_extra(lang), extra_body(req)));
        }
        sections.push(section(
            t::h_verification(lang),
            t::verification(
                lang,
                read_only,
                req.constraints.build_after_modifying,
                req.constraints.run_tests,
            ),
        ));
        sections.push(section(
            t::h_output(lang),
            t::output(lang, req.preset, read_only, &req.constraints),
        ));
        sections.join("\n\n")
    }
}

fn section(header: &str, body: String) -> String {
    format!("{header}\n\n{body}")
}

fn task_body(req: &PromptRequest) -> String {
    let mut task = req.task.trim().to_string();
    if task.is_empty() {
        return t::task_placeholder(req.language).to_string();
    }
    if !task.ends_with(|c: char| {
        matches!(
            c,
            '。' | '！' | '？' | '；' | '，' | '.' | '!' | '?' | ';' | ','
        )
    }) {
        task.push_str(t::sentence_end(req.language));
    }
    task
}

fn scope_body(req: &PromptRequest) -> String {
    if !req.selected_files.is_empty() {
        let mut s = String::from(t::scope_files_head(req.language));
        for f in &req.selected_files {
            s.push_str(&format!("\n- {f}"));
        }
        s.push_str("\n\n");
        if req.scope != Scope::SelectedFiles {
            // Defensive: never silently drop --file when scope mismatches.
            // Keep the requested reading scope, then pin modifications to files.
            s.push_str(t::scope_text(req.language, req.scope));
            s.push_str("\n\n");
        }
        s.push_str(t::scope_files_tail(req.language));
        return s;
    }
    if req.scope == Scope::SelectedFiles {
        // No files given: fall back to Auto instead of a vague file-less text.
        // The CLI also warns and falls back; this covers TUI/API paths.
        return t::scope_text(req.language, Scope::Auto).to_string();
    }
    t::scope_text(req.language, req.scope).to_string()
}

fn constraints_body(req: &PromptRequest, perm: PermissionLevel) -> String {
    let lang = req.language;
    let c = &req.constraints;
    let mut lines: Vec<&'static str> = Vec::new();

    if perm != PermissionLevel::ReadOnly {
        let minimal = perm == PermissionLevel::Minimal;
        // Lines already implied by the MINIMAL forbidden-list are skipped.
        if c.no_unrelated_changes && !minimal {
            lines.push(t::c_no_unrelated(lang));
        }
        if c.no_unnecessary_refactor && !minimal {
            lines.push(t::c_no_refactor(lang));
        }
        if c.no_unrelated_formatting && !minimal {
            lines.push(t::c_no_formatting(lang));
        }
        if c.preserve_public_interfaces
            && c.preserve_coding_style
            && perm != PermissionLevel::Refactor
        {
            lines.push(t::c_interfaces_style(lang));
        } else {
            if c.preserve_public_interfaces && perm != PermissionLevel::Refactor {
                lines.push(t::c_interfaces(lang));
            }
            if c.preserve_coding_style {
                lines.push(t::c_style(lang));
            }
        }
        if c.analyze_before_modifying
            && !matches!(req.preset, Preset::Fix | Preset::Debug | Preset::Plan)
        {
            lines.push(t::c_analyze_first(lang));
        }
        if c.no_new_files {
            lines.push(t::c_no_new_files(lang));
        }
        if c.no_dependency_changes {
            lines.push(t::c_no_deps(lang));
        }
    }

    let mut body = String::new();
    for line in lines {
        body.push_str("- ");
        body.push_str(line);
        body.push('\n');
    }
    // Every non-YOLO mode reports extra findings instead of fixing them.
    if req.preset != Preset::Yolo {
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str(t::extra_findings(lang, perm == PermissionLevel::ReadOnly));
    }
    body.trim_end().to_string()
}

fn extra_body(req: &PromptRequest) -> String {
    req.extra_rules
        .iter()
        .map(|rule| format!("- {rule}"))
        .collect::<Vec<_>>()
        .join("\n")
}
