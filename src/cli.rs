use std::io::{IsTerminal, Read};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::app::PresetHint;
use crate::clipboard::Clipboard;
use crate::config::{Config, CustomPreset};
use crate::history::{History, HistoryItem};
use crate::model::{effective_permission, Constraints, Depth, PermissionLevel, Preset, Scope};
use crate::prompt::{PromptBuilder, PromptRequest};

#[derive(Debug, Parser)]
#[command(
    name = "pm",
    version,
    about = "Prompt Maker — build disciplined prompts for coding agents",
    after_help = "Run without arguments to open the TUI. Task may also come from stdin."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze code without modifying anything
    Analyze(GenArgs),
    /// Analyze a problem and apply a minimal fix
    Fix(GenArgs),
    /// Produce an implementation plan without modifying code
    Plan(GenArgs),
    /// Analyze a call chain
    Trace(GenArgs),
    /// Analyze module architecture
    Arch(GenArgs),
    /// Refactor related code while keeping behavior
    Refactor(GenArgs),
    /// Debug crashes, data races, deadlocks, test failures
    Debug(GenArgs),
    /// High-autonomy mode
    Yolo(GenArgs),
    /// Use a custom preset defined in config.toml
    Run {
        name: String,
        #[command(flatten)]
        args: GenArgs,
    },
}

#[derive(Debug, Args)]
pub struct GenArgs {
    /// Task description; omit to read stdin, or to open the TUI
    pub task: Vec<String>,
    /// Permission level: readonly|minimal|scoped|refactor|yolo (or l0..l4)
    #[arg(short, long, value_name = "LEVEL")]
    pub permission: Option<PermissionLevel>,
    /// Analysis depth: quick|normal|deep
    #[arg(short, long, value_name = "DEPTH")]
    pub depth: Option<Depth>,
    /// Scope: auto|file|module|files|repo
    #[arg(short, long, value_name = "SCOPE")]
    pub scope: Option<Scope>,
    /// Limit the task to a file (repeat or comma-separate)
    #[arg(short = 'f', long = "file", value_name = "PATH")]
    pub file: Vec<String>,
    /// Also copy the prompt to the clipboard
    #[arg(short, long)]
    pub copy: bool,
    /// Never touch the clipboard
    #[arg(long)]
    pub no_copy: bool,
    /// Do not print the prompt (only copy)
    #[arg(short, long)]
    pub quiet: bool,
}

pub enum Outcome {
    Done(ExitCode),
    Tui(PresetHint),
}

pub fn execute(cli: Cli) -> Outcome {
    let (config, warning) = Config::load();
    if let Some(warning) = warning {
        eprintln!("pm: warning: {warning}");
    }

    let Some(command) = cli.command else {
        return Outcome::Tui(PresetHint::None);
    };

    let (preset, custom, args) = match command {
        Command::Analyze(a) => (Preset::Analyze, None, a),
        Command::Fix(a) => (Preset::Fix, None, a),
        Command::Plan(a) => (Preset::Plan, None, a),
        Command::Trace(a) => (Preset::Trace, None, a),
        Command::Arch(a) => (Preset::Arch, None, a),
        Command::Refactor(a) => (Preset::Refactor, None, a),
        Command::Debug(a) => (Preset::Debug, None, a),
        Command::Yolo(a) => (Preset::Yolo, None, a),
        Command::Run { name, args } => match config.resolve_custom(&name) {
            Some(custom) => (custom.base, Some(custom), args),
            None => {
                eprintln!("pm: unknown preset '{name}'.");
                let names: Vec<&str> = config
                    .custom_presets
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect();
                if names.is_empty() {
                    let path = Config::config_path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "~/.config/pm/config.toml".to_string());
                    eprintln!("Define custom presets in {path} under [[custom_presets]].");
                } else {
                    eprintln!("Available custom presets: {}", names.join(", "));
                }
                return Outcome::Done(ExitCode::FAILURE);
            }
        },
    };

    generate(&config, preset, custom, args)
}

fn generate(
    config: &Config,
    preset: Preset,
    custom: Option<CustomPreset>,
    args: GenArgs,
) -> Outcome {
    let task = resolve_task(&args.task);
    if task.is_empty() {
        let hint = match &custom {
            Some(c) => PresetHint::Custom(c.name.clone()),
            None => PresetHint::Builtin(preset),
        };
        return Outcome::Tui(hint);
    }

    let cp = custom.as_ref();
    let permission = args
        .permission
        .or(cp.and_then(|c| c.permission))
        .unwrap_or_else(|| preset.default_permission());
    let depth = args
        .depth
        .or(cp.and_then(|c| c.depth))
        .unwrap_or_else(|| preset.default_depth());
    let mut scope = args
        .scope
        .or(cp.and_then(|c| c.scope))
        .unwrap_or_else(|| preset.default_scope());
    let selected_files: Vec<String> = args
        .file
        .iter()
        .flat_map(|f| f.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if scope == Scope::SelectedFiles && selected_files.is_empty() {
        eprintln!("pm: warning: --scope files without --file, falling back to auto");
        scope = Scope::Auto;
    } else if !selected_files.is_empty() && scope != Scope::SelectedFiles {
        eprintln!("pm: warning: --file implies --scope files, using files scope");
        scope = Scope::SelectedFiles;
    }
    let extra_rules = cp.map(|c| c.extra_rules.clone()).unwrap_or_default();

    let request = PromptRequest {
        task: task.clone(),
        preset,
        permission,
        depth,
        scope,
        selected_files,
        constraints: config.constraints.resolve(Constraints::for_preset(preset)),
        extra_rules,
        language: config.lang(),
    };
    let prompt = PromptBuilder::build(&request);

    let mut history = History::load();
    history.push(HistoryItem {
        task,
        preset: cp
            .map(|c| c.name.clone())
            .unwrap_or_else(|| preset.key().to_string()),
        permission: effective_permission(preset, permission).key().to_string(),
        depth: depth.key().to_string(),
        scope: scope.key().to_string(),
        extra_rules: request.extra_rules.clone(),
        constraints: request.constraints,
        selected_files: request.selected_files.clone(),
    });
    let _ = history.save();

    let do_copy = args.copy && !args.no_copy;
    if args.quiet && !do_copy {
        eprintln!("pm: warning: --quiet without --copy produces no output");
    }
    let mut copy_failed = false;
    if do_copy {
        copy_failed = match Clipboard::new() {
            Some(mut clipboard) => clipboard.set_text(&prompt).is_err(),
            None => true,
        };
        if copy_failed {
            eprintln!("pm: warning: clipboard unavailable");
        }
    }

    if !args.quiet {
        println!("{prompt}");
    }

    // Quiet + copy failure means the prompt went nowhere.
    if do_copy && copy_failed && args.quiet {
        return Outcome::Done(ExitCode::FAILURE);
    }
    Outcome::Done(ExitCode::SUCCESS)
}

/// Join CLI words, or fall back to stdin when the task is missing.
fn resolve_task(args: &[String]) -> String {
    let mut task = args.join(" ");
    if task.trim().is_empty() && !std::io::stdin().is_terminal() {
        let mut buf = String::new();
        if std::io::stdin().read_to_string(&mut buf).is_ok() {
            task = buf;
        }
    }
    task.trim().to_string()
}
