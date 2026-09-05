//! Static prompt text, in Chinese (primary) and English (basic).
//!
//! Keep every block explicit and non-repetitive: say what the agent MAY do,
//! what counts as a necessary change, what must never be done in passing,
//! how to verify, and what to report back.

use crate::model::{Depth, PermissionLevel, Preset};
use crate::prompt::Lang;
use Lang::{En, Zh};

// ---------------------------------------------------------------- headers --

pub fn h_task(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 任务",
        En => "# Task",
    }
}

pub fn h_work(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 工作模式",
        En => "# Working mode",
    }
}

pub fn h_analysis(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 分析要求",
        En => "# Analysis requirements",
    }
}

pub fn h_permission(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 修改权限",
        En => "# Modification permission",
    }
}

pub fn h_scope(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 工作范围",
        En => "# Scope",
    }
}

pub fn h_constraints(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 约束",
        En => "# Constraints",
    }
}

pub fn h_extra(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 附加约束",
        En => "# Additional constraints",
    }
}

pub fn h_verification(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 验证要求",
        En => "# Verification",
    }
}

pub fn h_output(lang: Lang) -> &'static str {
    match lang {
        Zh => "# 最终输出",
        En => "# Expected output",
    }
}

// ------------------------------------------------------------------- task --

pub fn task_placeholder(lang: Lang) -> &'static str {
    match lang {
        Zh => "（未提供任务描述，请先补充。）",
        En => "(No task description provided.)",
    }
}

pub fn sentence_end(lang: Lang) -> &'static str {
    match lang {
        Zh => "。",
        En => ".",
    }
}

// ------------------------------------------------------------- work mode --

pub fn work_mode(lang: Lang, preset: Preset, perm: PermissionLevel) -> String {
    let read_only = perm == PermissionLevel::ReadOnly;
    match preset {
        Preset::Analyze => work_analyze(lang),
        Preset::Plan => work_plan(lang),
        Preset::Trace => work_trace(lang),
        Preset::Arch => work_arch(lang),
        Preset::Debug => {
            if read_only {
                work_debug_read_only(lang)
            } else {
                work_debug(lang)
            }
        }
        Preset::Fix => {
            if read_only {
                work_fix_read_only(lang)
            } else {
                let base = work_fix(lang);
                if perm == PermissionLevel::Minimal {
                    format!("{}\n\n{}", base, respect_old_code(lang))
                } else {
                    base
                }
            }
        }
        Preset::Refactor => {
            if read_only {
                work_refactor_read_only(lang)
            } else {
                work_refactor(lang)
            }
        }
        Preset::Yolo => {
            if read_only {
                work_analyze(lang)
            } else {
                work_yolo(lang)
            }
        }
    }
}

fn work_analyze(lang: Lang) -> String {
    match lang {
        Zh => "本任务只允许分析，不修改任何代码。\
\n\
\n先阅读与任务相关的代码，理解现状，然后给出分析结论。\
\n\
\n不要修改任何文件。\
\n不要生成 patch。\
\n不要因为发现问题而顺手修复。"
            .to_string(),
        En => "This task is analysis-only. Do not modify any code.\
\n\
\nRead the relevant code first, understand how it works, then report your findings.\
\n\
\nDo not modify any files.\
\nDo not generate patches.\
\nDo not fix issues you happen to find along the way."
            .to_string(),
    }
}

fn work_fix(lang: Lang) -> String {
    match lang {
        Zh => "先分析，再修改。\
\n\
\n首先定位相关实现、直接调用关系和共享状态，确认问题的根因；\
确认根因之后，再实施修复。\
\n\
\n不要因为看到表面可疑的代码就立即修改。\
需要区分现象、触发条件和真正的根因。\
\n\
\n修改必须建立在证据之上：不要因为某段代码看起来不安全就直接修改，\
先确认它是否真的参与当前问题。"
            .to_string(),
        En => "Analyze first, then fix.\
\n\
\nLocate the relevant implementation, direct call sites and shared state, and confirm the root cause before changing anything.\
\n\
\nDo not jump to the first suspicious-looking line.\
Distinguish the symptom, the trigger and the actual root cause.\
\n\
\nChanges must be backed by evidence: confirm a piece of code actually participates in the problem before touching it."
            .to_string(),
    }
}

fn work_fix_read_only(lang: Lang) -> String {
    match lang {
        Zh => "本任务只允许分析，不修改代码。\
\n\
\n先定位相关实现、直接调用关系和共享状态，确认问题的根因。\
\n\
\n不要修改任何文件。\
\n不要生成 patch。\
\n不要因为发现问题而顺手修复。"
            .to_string(),
        En => "This task is analysis-only. Do not modify any code.\
\n\
\nLocate the relevant implementation, call relationships and shared state, and identify the root cause.\
\n\
\nDo not modify any files.\
\nDo not generate patches.\
\nDo not fix issues you find along the way."
            .to_string(),
    }
}

fn work_plan(lang: Lang) -> String {
    match lang {
        Zh => "本任务只做分析和方案设计，不修改代码。\
\n\
\n深入阅读相关代码，确认影响范围，然后给出可以直接执行的实施方案。\
\n\
\n不要修改任何文件。\
\n不要生成 patch。"
            .to_string(),
        En => "Analysis and planning only. Do not modify any code.\
\n\
\nRead the relevant code, confirm the blast radius, then produce a concrete implementation plan.\
\n\
\nDo not modify any files.\
\nDo not generate patches."
            .to_string(),
    }
}

fn work_trace(lang: Lang) -> String {
    match lang {
        Zh => "本任务只分析调用链，不修改任何代码。\
\n\
\n先找到目标的定义位置，再沿调用关系向下追踪。"
            .to_string(),
        En => "Call-chain analysis only. Do not modify any code.\
\n\
\nFind the definition of the target, then follow the calls downward."
            .to_string(),
    }
}

fn work_arch(lang: Lang) -> String {
    match lang {
        Zh => "本任务以架构分析为主。\
\n\
\n从模块职责、依赖方向、边界和数据流入手梳理整体结构，\
再针对任务关注点给出评估。"
            .to_string(),
        En => "This task is an architecture analysis.\
\n\
\nMap module responsibilities, dependency directions, boundaries and data flow first, then assess the areas this task cares about."
            .to_string(),
    }
}

fn work_refactor(lang: Lang) -> String {
    match lang {
        Zh => "先理解现有行为，再重构。\
\n\
\n- 重构前先确认现有行为和现有测试覆盖；\
\n- 围绕任务目标重构，控制修改范围；\
\n- 保持外部行为兼容；\
\n- 说明每处重构的理由。"
            .to_string(),
        En => "Understand existing behavior before refactoring.\
\n\
\n- Confirm current behavior and existing test coverage first;\
\n- Keep the refactor focused on the task;\
\n- Preserve external behavior;\
\n- Explain why each refactoring step is needed."
            .to_string(),
    }
}

fn work_refactor_read_only(lang: Lang) -> String {
    match lang {
        Zh => "本任务只允许分析，不修改代码。\
\n\
\n先梳理现有行为和结构，评估重构空间，给出重构建议。\
\n\
\n不要修改任何文件。"
            .to_string(),
        En => "This task is analysis-only. Do not modify any code.\
\n\
\nMap existing behavior and structure, and propose a refactoring approach instead of applying it.\
\n\
\nDo not modify any files."
            .to_string(),
    }
}

fn work_debug(lang: Lang) -> String {
    match lang {
        Zh => "本任务是调试任务。先建立证据链，再下结论。\
\n\
\n不要仅凭函数名、代码风格或第一印象猜测问题。\
\n\
\n区分以下四类信息：\
\n- 观察到的现象（Observed behavior）\
\n- 可能的原因（Possible cause）\
\n- 已确认的证据（Confirmed evidence）\
\n- 真正的根因（Root cause）\
\n\
\n不要看到第一个可疑位置就修改。\
如果无法确认根因，明确说明证据不足，而不是强行修改。"
            .to_string(),
        En => "This is a debugging task. Build an evidence chain before drawing conclusions.\
\n\
\nDo not guess from function names or code style.\
\n\
\nDistinguish:\
\n- Observed behavior\
\n- Possible cause\
\n- Confirmed evidence\
\n- Root cause\
\n\
\nDo not modify the first suspicious spot you see.\
If the root cause cannot be confirmed, say the evidence is insufficient instead of forcing a fix."
            .to_string(),
    }
}

fn work_debug_read_only(lang: Lang) -> String {
    match lang {
        Zh => "本任务只允许分析，不修改代码。以调试方式分析：先建立证据链，再下结论。\
\n\
\n不要仅凭函数名、代码风格或第一印象猜测问题。\
\n\
\n区分以下四类信息：\
\n- 观察到的现象（Observed behavior）\
\n- 可能的原因（Possible cause）\
\n- 已确认的证据（Confirmed evidence）\
\n- 真正的根因（Root cause）\
\n\
\n如果无法确认根因，明确说明证据不足，而不是强行给结论。\
\n\
\n不要修改任何文件。\
\n不要生成 patch。"
            .to_string(),
        En => "This task is analysis-only. Do not modify any code. Debug it by building an evidence chain.\
\n\
\nDo not guess from function names or code style.\
\n\
\nDistinguish:\
\n- Observed behavior\
\n- Possible cause\
\n- Confirmed evidence\
\n- Root cause\
\n\
\nIf the root cause cannot be confirmed, say the evidence is insufficient instead of forcing a conclusion.\
\n\
\nDo not modify any files.\
\nDo not generate patches."
            .to_string(),
    }
}

fn work_yolo(lang: Lang) -> String {
    match lang {
        Zh => "你可以自主完成任务。\
\n\
\n- 自由阅读项目代码；\
\n- 自行决定实现方式；\
\n- 自由修改相关文件，必要时重构或新增辅助代码；\
\n- 自己运行测试，修复过程中发现的问题。\
\n\
\n仍然禁止：\
\n- 无意义的大范围格式化；\
\n- 与目标完全无关的功能修改。"
            .to_string(),
        En => "You have wide autonomy for this task.\
\n\
\n- Read the project freely;\
\n- Decide the implementation yourself;\
\n- Modify related files freely, refactor or add helper code when needed;\
\n- Run tests yourself and fix issues you find.\
\n\
\nStill forbidden:\
\n- Meaningless large-scale reformatting;\
\n- Changes to functionality unrelated to the goal."
            .to_string(),
    }
}

/// FIX + MINIMAL: protect the legacy codebase from drive-by modernization.
fn respect_old_code(lang: Lang) -> String {
    match lang {
        Zh => "这是现有代码库中的局部问题。\
\n不要以现代化代码、统一风格或改善设计为理由扩大修改。\
\n旧代码即使看起来不够优雅，只要与当前问题无关，就保持不动。"
            .to_string(),
        En => "This is a local problem in an existing codebase.\
\nDo not widen the change in the name of modernization, style unification or better design.\
\nOld code stays untouched as long as it is unrelated to the current problem, even if it looks inelegant."
            .to_string(),
    }
}

// --------------------------------------------------------------- analysis --

pub fn analysis(lang: Lang, preset: Preset, depth: Depth) -> String {
    format!(
        "{}\n\n{}",
        analysis_focus(lang, preset),
        depth_text(lang, depth)
    )
}

fn analysis_focus(lang: Lang, preset: Preset) -> String {
    match preset {
        Preset::Analyze => match lang {
            Zh => "分析需要覆盖：\
\n\
\n- 现状：相关代码目前如何工作；\
\n- 调用关系：谁调用它，它又调用了什么；\
\n- 核心逻辑：关键分支和状态变化；\
\n- 潜在问题和风险；\
\n- 结论。"
                .to_string(),
            En => "The analysis must cover:\
\n\
\n- Current behavior: how the related code works today;\
\n- Call relationships: who calls it, what it calls;\
\n- Core logic: key branches and state changes;\
\n- Potential issues and risks;\
\n- Conclusion."
                .to_string(),
        },
        Preset::Fix => match lang {
            Zh => "分析时重点确认：\
\n\
\n- 相关代码的读写位置和依赖关系；\
\n- 对象生命周期和所有权；\
\n- 相关调用链，以及调用方隐含的假设；\
\n- 异常路径和边界条件；\
\n- 现有测试和编译目标能否覆盖这个问题。"
                .to_string(),
            En => "Focus the analysis on:\
\n\
\n- Where the related state is read and written;\
\n- Object lifetimes and ownership;\
\n- The involved call chain and assumptions callers make;\
\n- Error paths and boundary conditions;\
\n- Whether existing tests and build targets cover the problem."
                .to_string(),
        },
        Preset::Debug => match lang {
            Zh => "分析时重点确认：\
\n\
\n- 问题的完整表现和稳定触发条件；\
\n- 失败路径和错误处理逻辑；\
\n- 并发、生命周期、所有权相关代码；\
\n- 边界条件；\
\n- 能否用日志、测试或最小改动复现。"
                .to_string(),
            En => "Focus the analysis on:\
\n\
\n- The full symptom and reliable reproduction steps;\
\n- Failure paths and error handling;\
\n- Concurrency, lifetimes and ownership;\
\n- Boundary conditions;\
\n- Whether logs, tests or a minimal change can reproduce it."
                .to_string(),
        },
        Preset::Plan => match lang {
            Zh => "制定方案前需要确认：\
\n\
\n- 相关实现和调用关系；\
\n- 改动的影响范围；\
\n- 现有测试和验证手段。"
                .to_string(),
            En => "Before writing the plan, confirm:\
\n\
\n- The related implementation and call relationships;\
\n- The blast radius of the change;\
\n- Existing tests and verification tools."
                .to_string(),
        },
        Preset::Trace => match lang {
            Zh => "调用链分析需要覆盖：\
\n\
\n- 入口和调用者；\
\n- 向下的调用链和关键函数；\
\n- 数据输入和输出；\
\n- 状态变化和副作用；\
\n- 关键对象的生命周期；\
\n- 最终落点。\
\n\
\n如果存在多条路径，区分主路径和异常路径。\
如果调用点很多，不必机械列出全部，按调用模式归纳。"
                .to_string(),
            En => "The trace must cover:\
\n\
\n- Entry points and callers;\
\n- The downward call chain and key functions;\
\n- Data inputs and outputs;\
\n- State changes and side effects;\
\n- Lifetimes of key objects;\
\n- The final destination.\
\n\
\nIf multiple paths exist, separate the main path from error paths.\
If there are many call sites, summarize the call patterns instead of listing them all."
                .to_string(),
        },
        Preset::Arch => match lang {
            Zh => "架构分析需要覆盖：\
\n\
\n- 模块职责划分；\
\n- 依赖关系和依赖方向；\
\n- 模块边界和耦合点；\
\n- 数据流和对象生命周期；\
\n- 全局状态；\
\n- 潜在架构问题。"
                .to_string(),
            En => "The analysis must cover:\
\n\
\n- Module responsibilities;\
\n- Dependencies and their direction;\
\n- Boundaries and coupling points;\
\n- Data flow and object lifetimes;\
\n- Global state;\
\n- Potential architectural issues."
                .to_string(),
        },
        Preset::Refactor => match lang {
            Zh => "重构前需要确认：\
\n\
\n- 现有行为，包括边界行为；\
\n- 现有测试覆盖情况；\
\n- 重构后如何保证行为兼容。"
                .to_string(),
            En => "Before refactoring, confirm:\
\n\
\n- Current behavior, including edge cases;\
\n- Existing test coverage;\
\n- How behavior compatibility will be preserved."
                .to_string(),
        },
        Preset::Yolo => match lang {
            Zh => "按任务需要自行决定分析范围和方式，但每个结论都要有代码依据。".to_string(),
            En => "Decide the analysis scope yourself, but back every conclusion with code."
                .to_string(),
        },
    }
}

fn depth_text(lang: Lang, depth: Depth) -> String {
    match depth {
        Depth::Quick => match lang {
            Zh => "分析深度：快速。\
\n\
\n只阅读最相关的代码，快速确认问题，不要扩大分析范围，结论保持简洁。"
                .to_string(),
            En => "Analysis depth: quick.\
\n\
\nRead only the most relevant code, confirm the problem fast, keep the conclusion concise."
                .to_string(),
        },
        Depth::Normal => match lang {
            Zh => "分析深度：正常。\
\n\
\n阅读相关实现，检查直接调用关系和依赖，验证结论后再继续。"
                .to_string(),
            En => "Analysis depth: normal.\
\n\
\nRead the implementation, check direct call relationships and dependencies, and verify conclusions before proceeding."
                .to_string(),
        },
        Depth::Deep => match lang {
            Zh => "分析深度：深入。\
\n\
\n不要只看当前函数：\
\n- 沿调用链检查关键路径；\
\n- 检查生命周期和状态变化；\
\n- 检查异常路径和并发；\
\n- 检查隐式依赖；\
\n- 必要时搜索同类代码或相关历史实现对照。\
\n\
\n深入分析不等于大范围修改，修改范围仍以下文约束为准。"
                .to_string(),
            En => "Analysis depth: deep.\
\n\
\nDo not stop at the current function:\
\n- Follow key paths along the call chain;\
\n- Check lifetimes and state changes;\
\n- Check error paths and concurrency;\
\n- Check implicit dependencies;\
\n- Compare with similar code when useful.\
\n\
\nDeep analysis does not mean large-scale changes; modification scope is still limited by the constraints below."
                .to_string(),
        },
    }
}

// ------------------------------------------------------------- permission --

pub fn permission_text(lang: Lang, perm: PermissionLevel) -> &'static str {
    match lang {
        Zh => match perm {
            PermissionLevel::ReadOnly => {
                "只读。你只能读取和分析代码。\
\n\
\n禁止：\
\n- 修改文件；\
\n- 创建文件；\
\n- 删除文件；\
\n- 生成 patch 并应用；\
\n- 修改配置。"
            }
            PermissionLevel::Minimal => {
                "最小修改。只允许完成当前任务所必需的最小修改。\
\n\
\n禁止：\
\n- 顺手重构；\
\n- 修改无关代码；\
\n- 清理无关 warning；\
\n- 重新格式化整个文件；\
\n- 修改命名；\
\n- 调整不相关的结构。"
            }
            PermissionLevel::Scoped => {
                "范围受限。允许在任务相关的模块内自由修改，但不得扩大到与任务无关的模块。"
            }
            PermissionLevel::Refactor => {
                "允许重构。\
\n\
\n允许：\
\n- 重构任务相关代码；\
\n- 调整内部结构、抽取函数；\
\n- 调整内部 API；\
\n- 新增辅助文件。\
\n\
\n要求：\
\n- 保持外部行为兼容；\
\n- 保持公开接口不变，除非任务明确要求改变。"
            }
            PermissionLevel::Yolo => {
                "自主模式。你可以自行判断实现方式：\
\n\
\n- 修改多个相关模块；\
\n- 新增文件；\
\n- 重构；\
\n- 删除明显废弃的实现；\
\n- 调整内部接口。\
\n\
\n但不要修改与目标无关的功能。"
            }
        },
        En => match perm {
            PermissionLevel::ReadOnly => {
                "Read-only. You may only read and analyze code.\
\n\
\nForbidden:\
\n- Modifying files\
\n- Creating files\
\n- Deleting files\
\n- Generating or applying patches\
\n- Changing configuration"
            }
            PermissionLevel::Minimal => {
                "Minimal changes. Only modify what is strictly necessary for this task.\
\n\
\nDo not:\
\n- Refactor opportunistically\
\n- Touch unrelated code\
\n- Clean up unrelated warnings\
\n- Reformat whole files\
\n- Rename things\
\n- Restructure unrelated code"
            }
            PermissionLevel::Scoped => {
                "Scoped changes. You may freely modify code within the modules related to this task, but do not expand into unrelated modules."
            }
            PermissionLevel::Refactor => {
                "Refactoring allowed.\
\n\
\nAllowed:\
\n- Refactor task-related code\
\n- Adjust internal structure, extract functions\
\n- Adjust internal APIs\
\n- Add helper files\
\n\
\nRequired:\
\n- Preserve external behavior\
\n- Preserve public interfaces unless the task explicitly requires changing them"
            }
            PermissionLevel::Yolo => {
                "Autonomous mode. Decide the implementation yourself:\
\n\
\n- Modify multiple related modules\
\n- Add files\
\n- Refactor\
\n- Remove clearly obsolete implementations\
\n- Adjust internal interfaces\
\n\
\nBut do not change functionality unrelated to the goal."
            }
        },
    }
}

// ------------------------------------------------------------------ scope --

pub fn scope_text(lang: Lang, scope: crate::model::Scope) -> &'static str {
    use crate::model::Scope;
    match lang {
        Zh => match scope {
            Scope::Auto => {
                "从与任务最直接相关的代码开始。\
\n\
\n只有在确有必要时才扩大阅读范围。\
可以阅读其他模块来确认调用关系，但不要因此扩大修改范围。"
            }
            Scope::CurrentFile => {
                "以当前文件为主要范围。\
\n\
\n可以读取依赖代码来理解上下文，但除非用户明确允许，不要修改其他文件。"
            }
            Scope::CurrentModule => {
                "分析范围限定在当前模块及其直接相关文件。\
\n\
\n可以读取模块外的代码来确认调用关系，但修改应限定在任务相关范围内。"
            }
            Scope::SelectedFiles => {
                "任务限定在指定的文件中。\
\n\
\n可以读取这些文件的依赖来理解上下文，但修改只针对这些文件。"
            }
            Scope::WholeRepo => {
                "可以搜索整个仓库来理解依赖和调用关系。\
\n\
\n但修改仍然受「修改权限」一节的约束。"
            }
        },
        En => match scope {
            Scope::Auto => {
                "Start from the code most directly related to the task.\
\n\
\nOnly widen the reading scope when genuinely necessary.\
You may read other modules to confirm call relationships, but do not widen the modification scope because of that."
            }
            Scope::CurrentFile => {
                "Focus on the current file.\
\n\
\nYou may read dependencies for context, but do not modify other files unless explicitly allowed."
            }
            Scope::CurrentModule => {
                "Limit analysis to the current module and directly related files.\
\n\
\nYou may read code outside the module to confirm call relationships, but keep modifications within the task-related scope."
            }
            Scope::SelectedFiles => {
                "The task is limited to the specified files.\
\n\
\nYou may read their dependencies for context, but only modify those files."
            }
            Scope::WholeRepo => {
                "You may search the whole repository to understand dependencies and call relationships.\
\n\
\nModifications are still limited by the modification-permission section above."
            }
        },
    }
}

pub fn scope_files_head(lang: Lang) -> &'static str {
    match lang {
        Zh => "任务限定在以下文件：",
        En => "The task is limited to these files:",
    }
}

pub fn scope_files_tail(lang: Lang) -> &'static str {
    match lang {
        Zh => "可以读取这些文件的依赖来理解上下文，但修改只针对以上文件。",
        En => {
            "You may read their dependencies for context, but only modify the files listed above."
        }
    }
}

// ------------------------------------------------------------ constraints --

pub fn c_no_unrelated(lang: Lang) -> &'static str {
    match lang {
        Zh => "只修改与任务直接相关的代码；",
        En => "Only modify code directly related to the task;",
    }
}

pub fn c_no_refactor(lang: Lang) -> &'static str {
    match lang {
        Zh => "除任务需要外，不做重构；",
        En => "No refactoring beyond what the task requires;",
    }
}

pub fn c_no_formatting(lang: Lang) -> &'static str {
    match lang {
        Zh => "不要格式化与任务无关的代码；",
        En => "Do not reformat code unrelated to the task;",
    }
}

pub fn c_interfaces(lang: Lang) -> &'static str {
    match lang {
        Zh => "保持现有公开接口；",
        En => "Preserve existing public interfaces;",
    }
}

pub fn c_style(lang: Lang) -> &'static str {
    match lang {
        Zh => "保持现有代码风格；",
        En => "Preserve existing coding style;",
    }
}

pub fn c_interfaces_style(lang: Lang) -> &'static str {
    match lang {
        Zh => "保持现有公开接口和代码风格；",
        En => "Preserve existing public interfaces and coding style;",
    }
}

pub fn c_analyze_first(lang: Lang) -> &'static str {
    match lang {
        Zh => "先完成分析、确认原因，再修改；",
        En => "Analyze and confirm the cause before modifying;",
    }
}

pub fn c_no_new_files(lang: Lang) -> &'static str {
    match lang {
        Zh => "不要新增文件；",
        En => "Do not create new files;",
    }
}

pub fn c_no_deps(lang: Lang) -> &'static str {
    match lang {
        Zh => "不要新增或修改依赖；",
        En => "Do not add or change dependencies;",
    }
}

/// Rule: report extra findings instead of fixing them in passing.
/// Applies to every preset except YOLO.
pub fn extra_findings(lang: Lang, read_only: bool) -> &'static str {
    match (lang, read_only) {
        (Zh, false) => {
            "如果分析过程中发现其他潜在问题：\
\n- 不要顺手修改；\
\n- 在最终结果中单独列出；\
\n- 说明它是否可能与当前问题相关。"
        }
        (Zh, true) => {
            "如果分析过程中发现其他潜在问题，在最终结果中单独列出，\
并说明严重程度以及是否可能与主任务相关。"
        }
        (En, false) => {
            "If you notice other potential problems during analysis:\
\n- Do not fix them in passing;\
\n- List them separately in the final report;\
\n- Note whether they might be related to the current problem."
        }
        (En, true) => {
            "If you notice other potential problems, list them separately in the final report with severity and their possible relation to the main task."
        }
    }
}

// ------------------------------------------------------------ verification --

pub fn verification(lang: Lang, read_only: bool, build: bool, tests: bool) -> String {
    if read_only {
        return match lang {
            Zh => "结论必须有代码依据：给出文件、位置和调用关系。不要凭猜测下结论。不要执行写操作或破坏性命令。"
                .to_string(),
            En => "Every conclusion must be backed by code: file, location and call relationship. Do not speculate. Do not run write operations or destructive commands."
                .to_string(),
        };
    }
    let (head, items): (&str, Vec<&str>) = match lang {
        Zh => ("修改完成后：", {
            let mut items = vec!["检查修改逻辑，确认覆盖了所有相关调用点"];
            if build {
                items.push("编译相关目标（使用仓库中真实存在的命令，不要编造）");
            }
            if tests {
                items.push("运行相关测试；如果没有相关测试，明确说明（使用真实存在的测试命令）");
            }
            if !build && !tests {
                items.push("说明应该如何验证这次修改");
            }
            items.push("确认没有引入新的问题");
            items
        }),
        En => ("After modifying:", {
            let mut items =
                vec!["Review the changes and make sure all related call sites are covered"];
            if build {
                items.push("Build the affected targets (use real commands from the repo, do not invent them)");
            }
            if tests {
                items.push(
                    "Run the relevant tests; if there are none, say so explicitly (use real test commands)",
                );
            }
            if !build && !tests {
                items.push("Explain how the change should be verified");
            }
            items.push("Confirm no new problems were introduced");
            items
        }),
    };
    let safety = match lang {
        Zh => "只在任务范围内执行读写和构建/测试命令；不要 force-push、删除分支、修改生产配置或执行破坏性操作，需要时先说明。",
        En => "Only run read/write and build/test commands within the task scope; do not force-push, delete branches, touch production config, or run destructive commands without asking first.",
    };
    format!("{head}\n\n{}\n\n{safety}", numbered(&items))
}

// ----------------------------------------------------------------- output --

pub fn output(
    lang: Lang,
    preset: Preset,
    read_only: bool,
    constraints: &crate::model::Constraints,
) -> String {
    let body = if read_only {
        match preset {
            Preset::Trace => out_trace(lang),
            Preset::Plan => out_plan(lang),
            Preset::Arch => out_arch(lang),
            Preset::Debug => out_debug_read_only(lang),
            _ => out_analyze(lang),
        }
    } else {
        match preset {
            Preset::Fix => out_fix(lang, constraints),
            Preset::Debug => out_debug(lang, constraints),
            Preset::Refactor => out_refactor(lang),
            Preset::Yolo => out_yolo(lang),
            Preset::Arch => out_arch(lang),
            _ => out_analyze(lang),
        }
    };
    format!("{body}\n\n{}", report_footer(lang))
}

/// Shared closing line: stop condition + evidence/confidence + unmodified scope.
/// Kept to two sentences so prompts stay tight.
fn report_footer(lang: Lang) -> &'static str {
    match lang {
        Zh => "满足以上输出后即停止，不要继续扩大范围。每条关键结论给出依据（文件、位置、调用关系或测试输出）和置信度（高/中/低）；证据不足时直说，不要强行结论；同时说明未修改的内容。",
        En => "Stop once the above report is complete; do not keep expanding the scope. Back each key claim with evidence (file, location, call relationship or test output) and confidence (high/medium/low); if evidence is insufficient, say so; also note what was left unmodified.",
    }
}

fn out_debug_read_only(lang: Lang) -> String {
    match lang {
        Zh => "完成后按以下结构报告：\
\n\
\n1. 现象（Observed behavior）；\
\n2. 可能原因（Possible cause）；\
\n3. 已确认的证据（Confirmed evidence，含文件、位置和调用关系）；\
\n4. 根因（Root cause）及置信度（高/中/低）。\
\n\
\n如果证据不足以确认根因，明确说明证据不足，不要强行给结论。\
\n不要修改任何文件。"
            .to_string(),
        En => "Report at the end:\
\n\
\n1. Observed behavior;\
\n2. Possible cause;\
\n3. Confirmed evidence (file, location, call relationship);\
\n4. Root cause with confidence (high/medium/low).\
\n\
\nIf the evidence is insufficient, say so instead of forcing a conclusion.\
\nDo not modify any files."
            .to_string(),
    }
}

fn numbered(items: &[&str]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {s}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn out_analyze(lang: Lang) -> String {
    match lang {
        Zh => "完成后输出分析报告：\
\n\
\n1. 现状和核心逻辑；\
\n2. 潜在问题：按严重程度排序，每条给出代码位置和依据；\
\n3. 风险；\
\n4. 结论和建议。\
\n\
\n不要只给结论。"
            .to_string(),
        En => "Report at the end:\
\n\
\n1. Current behavior and core logic;\
\n2. Potential issues, ordered by severity, each with file location and evidence;\
\n3. Risks;\
\n4. Conclusion and suggestions.\
\n\
\nDo not report conclusions without evidence."
            .to_string(),
    }
}

fn out_trace(lang: Lang) -> String {
    match lang {
        Zh => "完成后输出：\
\n\
\n1. 调用链说明：入口、关键函数、数据流、最终落点；\
\n2. 一个文本调用图，例如：\
\n\
\n   Caller\
\n     ↓\
\n   getUserByName\
\n     ↓\
\n   Registry lookup\
\n     ↓\
\n   Entity factory\
\n\
\n不要机械罗列全部调用点，按调用模式归纳。"
            .to_string(),
        En => "Report at the end:\
\n\
\n1. The call chain: entry points, key functions, data flow, final destination;\
\n2. A text call graph, for example:\
\n\
\n   Caller\
\n     ↓\
\n   target function\
\n     ↓\
\n   registry lookup\
\n     ↓\
\n   entity factory\
\n\
\nSummarize call patterns instead of listing every call site."
            .to_string(),
    }
}

fn out_plan(lang: Lang) -> String {
    match lang {
        Zh => "完成后输出完整实施方案：\
\n\
\n1. 需要修改的文件和位置；\
\n2. 每一步的具体改动；\
\n3. 风险和注意事项；\
\n4. 验证方式。\
\n\
\n方案要具体到文件和函数，可以直接执行。"
            .to_string(),
        En => "Produce a concrete implementation plan:\
\n\
\n1. Files and locations to modify;\
\n2. Concrete steps;\
\n3. Risks and caveats;\
\n4. How to verify.\
\n\
\nThe plan must be specific enough to execute directly."
            .to_string(),
    }
}

fn out_arch(lang: Lang) -> String {
    match lang {
        Zh => "完成后输出架构分析报告：\
\n\
\n1. 模块职责和依赖关系（可以用简单的文本结构图）；\
\n2. 数据流和对象生命周期；\
\n3. 潜在架构问题，按影响排序；\
\n4. 改进建议。"
            .to_string(),
        En => "Report at the end:\
\n\
\n1. Module responsibilities and dependencies (a simple text diagram helps);\
\n2. Data flow and object lifetimes;\
\n3. Architectural issues, ordered by impact;\
\n4. Improvement suggestions."
            .to_string(),
    }
}

fn out_fix(lang: Lang, c: &crate::model::Constraints) -> String {
    if lang == Zh {
        let mut items: Vec<&str> = Vec::new();
        if c.explain_root_cause {
            items.push("根因是什么，证据是什么");
        }
        if c.explain_modifications {
            items.push("修改了哪些文件和位置");
            items.push("为什么采用这种修改");
        }
        items.push("如何验证");
        if c.list_remaining_risks {
            items.push("是否仍存在潜在风险");
        }
        format!("完成后报告：\n\n{}\n\n不要只给结论。", numbered(&items))
    } else {
        let mut items: Vec<&str> = Vec::new();
        if c.explain_root_cause {
            items.push("The root cause and the evidence");
        }
        if c.explain_modifications {
            items.push("Which files and locations were modified");
            items.push("Why this change");
        }
        items.push("How it was verified");
        if c.list_remaining_risks {
            items.push("Whether risks remain");
        }
        format!(
            "When done, report:\n\n{}\n\nDo not report conclusions without evidence.",
            numbered(&items)
        )
    }
}

fn out_debug(lang: Lang, c: &crate::model::Constraints) -> String {
    if lang == Zh {
        let mut items: Vec<&str> = vec![
            "现象（Observed behavior）",
            "可能原因（Possible cause）",
            "已确认的证据（Confirmed evidence）",
            "根因（Root cause）",
        ];
        if c.explain_modifications {
            items.push("修改了哪些文件和位置，为什么");
        }
        items.push("如何验证");
        if c.list_remaining_risks {
            items.push("是否仍存在潜在风险");
        }
        format!(
            "完成后按以下结构报告：\n\n{}\n\n如果证据不足以确认根因，明确说明证据不足，不要强行给结论。",
            numbered(&items)
        )
    } else {
        let mut items: Vec<&str> = vec![
            "Observed behavior",
            "Possible cause",
            "Confirmed evidence",
            "Root cause",
        ];
        if c.explain_modifications {
            items.push("Which files were modified and why");
        }
        items.push("How it was verified");
        if c.list_remaining_risks {
            items.push("Whether risks remain");
        }
        format!(
            "When done, report:\n\n{}\n\nIf the evidence is insufficient to confirm the root cause, say so instead of forcing a conclusion.",
            numbered(&items)
        )
    }
}

fn out_refactor(lang: Lang) -> String {
    match lang {
        Zh => "完成后报告：\
\n\
\n1. 重构了哪些文件和位置；\
\n2. 每处重构的理由；\
\n3. 如何保证行为兼容；\
\n4. 如何验证；\
\n5. 是否仍存在潜在风险。"
            .to_string(),
        En => "When done, report:\
\n\
\n1. What was refactored;\
\n2. Why each change was made;\
\n3. How behavior compatibility is preserved;\
\n4. How it was verified;\
\n5. Whether risks remain."
            .to_string(),
    }
}

fn out_yolo(lang: Lang) -> String {
    match lang {
        Zh => "完成后报告：\
\n\
\n1. 做了什么，为什么；\
\n2. 如何验证；\
\n3. 是否有需要注意的遗留问题。"
            .to_string(),
        En => "When done, report:\
\n\
\n1. What was done and why;\
\n2. How it was verified;\
\n3. Any remaining issues worth attention."
            .to_string(),
    }
}
