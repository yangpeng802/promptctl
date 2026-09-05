# pm — Prompt Maker

`pm` 是一个本地、离线、启动即用的 CLI/TUI 小工具，用来为 Coding Agent
（Zcode、OpenCode、Codex、Claude Code 等）快速生成结构化、约束明确的任务
Prompt。

它不聊天、不调用 LLM、不执行代码。它只做一件事：把你的任务描述和一组
清晰的约束，组装成一份 Agent 容易遵守的 Prompt。

## 为什么

使用 Coding Agent 时经常要重复输入同样的约束：

- 先分析，不要直接修改
- 不要修改无关代码
- 不要顺手重构
- 只做最小修改
- 先确认根因
- 修改后编译测试并说明原因

`pm` 把这些约束结构化。你只需要输入任务，选择任务模式（Preset）、修改
权限（Permission）、分析深度（Depth）和范围（Scope），其余交给模板：

```text
任务描述 + Preset + Permission + Depth + Scope + Constraints + 验证 + 输出要求
= 最终 Prompt
```

## 安装

需要 Rust 工具链。

```sh
cargo install --path .
```

之后可直接使用 `pm` 命令。开发调试用 `cargo run -- ...`。

## 快速上手（TUI）

```sh
pm
```

输入一句任务，按需调整选项，按 `c` 复制，粘贴给 Agent。整个过程十几秒。

- Preview 实时更新，不需要按 Generate
- 所有配置都在同一屏完成，不搞多步向导

## CLI

```sh
pm fix "修复 getUserById 的 data race"
pm analyze "分析 appCache 生命周期"
pm trace "分析 getUserByName 完整调用链" --depth deep
pm debug "分析这个偶发 crash" -p minimal -d deep -c
pm plan "设计 metadata 重构方案"
pm arch "评估 cache 模块的耦合"
pm refactor "重构 cache 模块"
pm yolo "把当前编译问题解决掉"
```

任务也可以从 stdin 传入：

```sh
echo "分析 getUserById" | pm analyze
cat task.txt | pm fix
```

没有 task 且 stdin 为空时（例如直接运行 `pm` 或 `pm fix`），进入 TUI，
并预选对应的 Preset。

### CLI 参数

| 参数 | 缩写 | 说明 |
| --- | --- | --- |
| `--permission` | `-p` | `readonly` `minimal` `scoped` `refactor` `yolo`（或 `l0`..`l4`） |
| `--depth` | `-d` | `quick` `normal` `deep` |
| `--scope` | `-s` | `auto` `file` `module` `files` `repo` |
| `--file` | `-f` | 限定文件，可重复或逗号分隔（有 `--file` 时自动使用 `files` scope；`--scope files` 无 `--file` 时回退到 `auto` 并告警） |
| `--copy` | `-c` | 同时复制到剪贴板（默认仍输出到 stdout） |
| `--no-copy` | | 明确不碰剪贴板 |
| `--quiet` | `-q` | 不输出 Prompt，只复制（配合 `--copy`） |

```sh
pm fix "修复 data race" --permission minimal --depth deep --scope module --copy
pm fix "xxx" -p minimal -d deep -s auto
pm fix "xxx" -c -q        # 只复制
```

## Preset

| Preset | 默认权限 | 默认深度 | 默认范围 | 用途 |
| --- | --- | --- | --- | --- |
| `ANALYZE` | L0 READ ONLY | NORMAL | AUTO | 只分析，不修改 |
| `FIX` | L1 MINIMAL | NORMAL | AUTO | 分析并最小修复（默认） |
| `DEBUG` | L1 MINIMAL | DEEP | AUTO | crash / data race / deadlock 等调试 |
| `TRACE` | L0 READ ONLY | DEEP | REPO | 调用链追踪，输出文本调用图 |
| `PLAN` | L0 READ ONLY | DEEP | AUTO | 分析并产出实施方案，不修改 |
| `ARCH` | L0 READ ONLY | DEEP | REPO | 架构分析 |
| `REFACTOR` | L3 REFACTOR | DEEP | MODULE | 允许重构，要求行为兼容 |
| `YOLO` | L4 YOLO | NORMAL | AUTO | 高自主权，仍禁止无关改动 |

## Permission Level

| 级别 | 名称 | 语义 |
| --- | --- | --- |
| L0 | READ ONLY | 绝对禁止任何修改 |
| L1 | MINIMAL | 只允许完成任务所必需的最小修改（默认） |
| L2 | SCOPED | 允许在任务相关模块内自由修改 |
| L3 | REFACTOR | 允许重构、调内部结构，要求外部行为兼容 |
| L4 | YOLO | Agent 自主决定实现方式，仍禁止无关改动 |

Preset 与 Permission 是两个维度（`FIX + L3` = 允许为修复做相关重构）。
纯分析的 Preset（`ANALYZE` / `PLAN` / `TRACE`）会强制压到 READ ONLY，
界面显示 `Effective permission`，TUI 中 Permission 框会用 `→` 标注实际
生效级别。

## 生成 Prompt 的结构

```text
# 任务
# 工作模式
# 分析要求
# 修改权限
# 工作范围
# 约束
# 附加约束      （仅当有额外规则/自定义 preset 规则时）
# 验证要求
# 最终输出
```

模板内置了几条贯穿性的规则设计：

- 发现额外问题时单独列出，不顺手修（所有非 YOLO 模式）
- 修改必须建立在证据之上（FIX / DEBUG）
- FIX + MINIMAL 默认加入"尊重旧代码"段落，禁止借现代化之名扩大修改
- 深入分析不等于大范围修改
- 权限已隐含的约束不会重复出现，避免 Prompt 啰嗦
- 最终输出统一带停机条件：报完即停，每条关键结论给依据和置信度（高/中/低），并说明未修改的内容
- 验证要求使用仓库真实命令（不要编造），只在任务范围内执行构建/测试，禁止 force-push、删分支、动生产配置等破坏性操作

直接运行看效果：

```sh
pm fix "修复 getUserById 的 data race" -d deep
```

## TUI 键位

| 按键 | 作用 |
| --- | --- |
| `Tab` / `Shift+Tab` | 在分区之间移动焦点 |
| `↑` `↓` | 移动选择（Preset 列表、Constraints） |
| `←` `→` | 切换选项（Permission / Depth / Scope） |
| `Space` | 勾选/取消约束；在 Preset 上应用该 preset |
| `Enter` | 进入编辑（Task / Files / Extra rules）；应用 Preset |
| `Esc` | 退出编辑（不在编辑状态时退出程序） |
| `c` | 复制 Prompt 到剪贴板 |
| `h` | 打开历史记录（`↑` `↓` 选择，`Enter` 恢复） |
| `r` | 恢复当前 Preset 的默认值 |
| `?` | 帮助弹窗 |
| `PgUp` / `PgDn` 或 `Ctrl+↑` / `Ctrl+↓` | 滚动 Preview |
| `q` | 退出 |

复制成功显示 `✓ Prompt copied`，约 2 秒后自动消失；剪贴板不可用时显示
`⚠ Clipboard unavailable`，程序不会崩溃。

切换 Preset 会刷新未手动改过的字段；你手动改过的选项（权限/深度/范围/
约束）不会被莫名重置，按 `r` 才恢复 Preset 默认值。

## 配置

配置文件位于操作系统标准配置目录，例如：

- Linux: `~/.config/pm/config.toml`
- macOS: `~/Library/Application Support/pm/config.toml`
- Windows: `%APPDATA%\pm\config.toml`

文件不存在时使用内置默认值，不会自动创建；解析失败时使用默认值并在
TUI/CLI 中给出警告。示例：

```toml
default_preset = "fix"
default_permission = "minimal"
default_depth = "normal"
default_scope = "auto"
language = "zh"          # zh | en

[constraints]
no_unrelated_changes = true
no_unnecessary_refactor = true
preserve_public_interfaces = true
preserve_coding_style = true
analyze_before_modifying = true
build_after_modifying = true
run_tests = false
no_new_files = true
no_dependency_changes = true
no_unrelated_formatting = true
explain_root_cause = true
explain_modifications = true
list_remaining_risks = true
```

`[constraints]` 缺省的字段继承默认值；无法识别的枚举值按字段回退到默认。

## 自定义 Preset

```toml
[[custom_presets]]
name = "legacy-fix"
base = "fix"              # 基于哪个内置 preset
permission = "minimal"    # 可选，覆盖 base 的默认值
depth = "deep"            # 可选
scope = "module"          # 可选

extra_rules = [
    "不要修改公共接口",
    "不要引入新依赖",
    "必须兼容现有构建环境"
]
```

TUI 中自定义 preset 出现在内置列表之后（分隔线以下）；CLI 用：

```sh
pm run legacy-fix "修复 cache 模块"
```

`extra_rules` 会生成 `# 附加约束` 段落。TUI 中也可以直接在
`Extra rules` 区手动输入（每行一条）。

## 历史记录

最近 20 条生成记录保存在操作系统标准数据目录
（如 macOS `~/Library/Application Support/pm/history.json`、Linux
`~/.local/share/pm/history.json`），只存任务与参数，不存代码。TUI 中按
`h` 打开，`↑` `↓` 选择，`Enter` 恢复整套配置。

## Prompt 语言

`language = "zh"`（默认）生成中文模板；`language = "en"` 提供基本的英文
模板。

## 平台说明

- TUI：Ratatui + Crossterm，支持 Linux / WSL / Windows Terminal / macOS
- 剪贴板：arboard。macOS 与 Windows 开箱即用；Linux 需要 X11 或 Wayland
  会话，WSL 需要 WSLg（或设置 `DISPLAY`/`WAYLAND_DISPLAY`）。剪贴板不可
  用时程序只提示 `Clipboard unavailable`，不会崩溃，Prompt 仍显示在
  Preview / stdout 中
- 程序退出（正常、`q`、`Ctrl+C`、panic）都会恢复终端状态，不会把终端
  留在 raw mode

## 开发

```sh
cargo build
cargo test
cargo clippy
cargo fmt --check
cargo install --path .
```

## 已知限制

- Preview 的滚动按未折行的行数估算，极长的行在窄终端下 PgDn 到底可能
  略有偏差（内容显示不受影响）
- 英文模板是基本版本，丰富度不及中文模板
- 不支持鼠标操作、主题系统、多份配置（均为有意裁剪）
- 历史记录恢复时不包含约束勾选状态和文件列表（历史只存任务、四元参数与额外规则）
