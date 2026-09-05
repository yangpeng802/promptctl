use pm::model::{Constraints, Depth, PermissionLevel, Preset, Scope};
use pm::prompt::{Lang, PromptBuilder, PromptRequest};

fn req(preset: Preset, permission: PermissionLevel) -> PromptRequest {
    PromptRequest {
        task: "修复 getUserById 的 data race".to_string(),
        preset,
        permission,
        depth: Depth::Normal,
        scope: Scope::Auto,
        selected_files: Vec::new(),
        constraints: Constraints::for_preset(preset),
        extra_rules: Vec::new(),
        language: Lang::Zh,
    }
}

fn build(preset: Preset, permission: PermissionLevel) -> String {
    PromptBuilder::build(&req(preset, permission))
}

#[test]
fn fix_minimal_contains_core_rules() {
    let prompt = build(Preset::Fix, PermissionLevel::Minimal);
    assert!(prompt.contains("# 任务"), "missing 任务 section");
    assert!(prompt.contains("getUserById"), "task text missing");
    assert!(prompt.contains("最小修改"), "minimal-change rule missing");
    assert!(prompt.contains("顺手重构"), "no-refactor rule missing");
    assert!(prompt.contains("# 工作模式"));
    assert!(prompt.contains("# 分析要求"));
    assert!(prompt.contains("# 修改权限"));
    assert!(prompt.contains("# 工作范围"));
    assert!(prompt.contains("# 约束"));
    assert!(prompt.contains("# 验证要求"));
    assert!(prompt.contains("# 最终输出"));
    // FIX + MINIMAL protects the legacy codebase.
    assert!(prompt.contains("旧代码"), "respect-old-code rule missing");
    // Not read-only.
    assert!(!prompt.contains("你只能读取和分析代码"));
    // Extra findings rule (non-YOLO).
    assert!(prompt.contains("不要顺手修改"));
}

#[test]
fn analyze_stays_read_only_even_with_yolo() {
    let prompt = build(Preset::Analyze, PermissionLevel::Yolo);
    assert!(prompt.contains("只允许分析"));
    assert!(prompt.contains("不要修改任何文件"));
    assert!(prompt.contains("不要生成 patch"));
    // The effective permission (read-only block) wins over YOLO.
    assert!(prompt.contains("你只能读取和分析代码"));
    assert!(!prompt.contains("自由修改"));
}

#[test]
fn plan_stays_read_only_even_with_refactor() {
    let prompt = build(Preset::Plan, PermissionLevel::Refactor);
    assert!(prompt.contains("不要修改任何文件"));
    assert!(prompt.contains("方案"));
    assert!(prompt.contains("验证方式"));
    assert!(!prompt.contains("允许重构"), "L3 block must not appear");
}

#[test]
fn trace_requires_call_graph() {
    let prompt = build(Preset::Trace, PermissionLevel::ReadOnly);
    assert!(prompt.contains("调用链"));
    assert!(prompt.contains("调用图"));
    assert!(prompt.contains("最终落点"));
    assert!(prompt.contains("不修改任何代码"));
    assert!(prompt.contains("按调用模式归纳"));
}

#[test]
fn debug_builds_evidence_chain() {
    let prompt = build(Preset::Debug, PermissionLevel::Minimal);
    assert!(prompt.contains("证据链"));
    assert!(prompt.contains("Observed behavior"));
    assert!(prompt.contains("Root cause"));
    assert!(prompt.contains("根因"));
    assert!(prompt.contains("证据不足"));
    assert!(prompt.contains("不要看到第一个可疑位置就修改"));
}

#[test]
fn yolo_allows_modification_without_conflicting_text() {
    let prompt = build(Preset::Yolo, PermissionLevel::Yolo);
    assert!(prompt.contains("自主完成任务"));
    assert!(prompt.contains("自由修改"));
    // Must NOT contain read-only / forbid-all-modification text.
    assert!(!prompt.contains("你只能读取和分析代码"));
    assert!(!prompt.contains("只读。"));
    assert!(!prompt.contains("不要修改任何文件"));
}

#[test]
fn extra_rules_get_their_own_section() {
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.extra_rules = vec![
        "必须兼容 GCC 4.8".to_string(),
        "不要使用 shared_mutex".to_string(),
    ];
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("# 附加约束"));
    assert!(prompt.contains("- 必须兼容 GCC 4.8"));
    assert!(prompt.contains("- 不要使用 shared_mutex"));
}

#[test]
fn selected_files_are_listed() {
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.scope = Scope::SelectedFiles;
    request.selected_files = vec!["src/cache.cpp".to_string(), "include/cache.hpp".to_string()];
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("- src/cache.cpp"));
    assert!(prompt.contains("- include/cache.hpp"));
    assert!(prompt.contains("只针对以上文件"));
}

#[test]
fn selected_files_never_silently_dropped() {
    // --file with a non-files scope must still list the files.
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.scope = Scope::Auto;
    request.selected_files = vec!["src/a.rs".to_string()];
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("- src/a.rs"), "files must not be dropped");
}

#[test]
fn files_scope_without_files_falls_back_to_auto() {
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.scope = Scope::SelectedFiles;
    request.selected_files = Vec::new();
    let prompt = PromptBuilder::build(&request);
    assert!(
        !prompt.contains("指定的文件中"),
        "vague files text must not appear without files"
    );
    assert!(prompt.contains("最直接相关"), "should fall back to auto");
}

#[test]
fn debug_read_only_keeps_evidence_chain_without_modification() {
    let prompt = build(Preset::Debug, PermissionLevel::ReadOnly);
    assert!(prompt.contains("证据链"));
    assert!(prompt.contains("Observed behavior"));
    assert!(prompt.contains("不要修改任何文件"));
    assert!(prompt.contains("置信度"));
}

#[test]
fn output_has_stop_condition_and_confidence() {
    let prompt = build(Preset::Fix, PermissionLevel::Minimal);
    assert!(prompt.contains("即停止"));
    assert!(prompt.contains("置信度"));
    assert!(prompt.contains("未修改"));
}

#[test]
fn verification_names_real_commands_and_tool_safety() {
    let prompt = build(Preset::Fix, PermissionLevel::Minimal);
    assert!(prompt.contains("真实存在"));
    assert!(prompt.contains("不要编造"));
    assert!(prompt.contains("force-push"));
}

#[test]
fn verification_follows_build_and_test_flags() {
    // FIX default: build yes, tests no.
    let prompt = build(Preset::Fix, PermissionLevel::Minimal);
    assert!(prompt.contains("编译相关目标"));
    assert!(
        !prompt.contains("运行相关测试"),
        "run_tests is off by default"
    );

    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.constraints.run_tests = true;
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("运行相关测试"));

    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.constraints.build_after_modifying = false;
    request.constraints.run_tests = false;
    let prompt = PromptBuilder::build(&request);
    assert!(!prompt.contains("编译相关目标"));
    assert!(prompt.contains("说明应该如何验证"));
}

#[test]
fn english_template() {
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.language = Lang::En;
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("# Task"));
    assert!(prompt.contains("# Working mode"));
    assert!(prompt.contains("# Modification permission"));
    assert!(!prompt.contains("# Additional constraints"));
    assert!(prompt.contains("# Verification"));
    assert!(prompt.contains("# Expected output"));
}

#[test]
fn deep_depth_adds_call_chain_checks() {
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.depth = Depth::Deep;
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("分析深度：深入"));
    assert!(prompt.contains("沿调用链检查关键路径"));
    assert!(prompt.contains("深入分析不等于大范围修改"));
}

#[test]
fn whole_repo_scope_notes_permission_still_applies() {
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.scope = Scope::WholeRepo;
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("搜索整个仓库"));
    assert!(prompt.contains("修改仍然受"));
}

#[test]
fn empty_task_uses_placeholder() {
    let mut request = req(Preset::Fix, PermissionLevel::Minimal);
    request.task = "   ".to_string();
    let prompt = PromptBuilder::build(&request);
    assert!(prompt.contains("未提供任务描述"));
}
