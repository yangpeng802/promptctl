#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Install promptctl's /pm-fix integration for OpenCode.

Usage:
  ./scripts/install-opencode-plugin.sh [--install-pm | --plugin-only]
  ./scripts/install-opencode-plugin.sh --uninstall

Options:
  --install-pm   Install/update the pm binary from this checkout.
  --plugin-only  Install only the plugin; fail if pm is not on PATH.
  --uninstall    Remove the installed promptctl plugin (keeps pm installed).
  -h, --help     Show this help.

Without an option, pm is installed only when it is not already on PATH.
EOF
}

script_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(CDPATH= cd -- "${script_dir}/.." && pwd)"
plugin_source="${repo_dir}/integrations/opencode/promptctl.ts"
config_root="${OPENCODE_CONFIG_DIR:-${XDG_CONFIG_HOME:-${HOME:?HOME is not set}/.config}/opencode}"
plugin_dir="${config_root}/plugins"
plugin_target="${plugin_dir}/promptctl.ts"
mode="auto"

if [[ $# -gt 1 ]]; then
  usage >&2
  exit 2
fi

if [[ $# -eq 1 ]]; then
  case "$1" in
    --install-pm) mode="install-pm" ;;
    --plugin-only) mode="plugin-only" ;;
    --uninstall) mode="uninstall" ;;
    -h|--help) usage; exit 0 ;;
    *)
      printf 'Unknown option: %s\n\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
fi

if [[ "$mode" == "uninstall" ]]; then
  if [[ ! -e "$plugin_target" ]]; then
    printf 'promptctl OpenCode plugin is not installed: %s\n' "$plugin_target"
    exit 0
  fi
  if ! grep -Fq "promptctl OpenCode integration" "$plugin_target"; then
    printf 'Refusing to remove an unrecognized file: %s\n' "$plugin_target" >&2
    exit 1
  fi
  rm -f -- "$plugin_target"
  printf 'Removed %s\n' "$plugin_target"
  printf 'Restart OpenCode to finish uninstalling /pm-fix.\n'
  exit 0
fi

if [[ ! -f "$plugin_source" ]]; then
  printf 'Plugin source not found: %s\n' "$plugin_source" >&2
  exit 1
fi

if [[ "$mode" == "install-pm" ]] || { [[ "$mode" == "auto" ]] && ! command -v pm >/dev/null 2>&1; }; then
  if ! command -v cargo >/dev/null 2>&1; then
    printf 'pm is not on PATH and cargo is unavailable. Install Rust first, or put pm on PATH.\n' >&2
    exit 1
  fi
  printf 'Installing pm from %s ...\n' "$repo_dir"
  cargo install --locked --path "$repo_dir"
fi

if ! command -v pm >/dev/null 2>&1; then
  printf 'pm is not available on PATH. Run again without --plugin-only or use --install-pm.\n' >&2
  exit 1
fi

mkdir -p -- "$plugin_dir"

if [[ -f "$plugin_target" ]] && cmp -s -- "$plugin_source" "$plugin_target"; then
  printf 'OpenCode plugin is already up to date: %s\n' "$plugin_target"
else
  if [[ -e "$plugin_target" ]]; then
    backup="${plugin_target}.bak.$(date +%Y%m%d%H%M%S)"
    cp -p -- "$plugin_target" "$backup"
    printf 'Backed up existing plugin to %s\n' "$backup"
  fi
  install -m 0644 "$plugin_source" "$plugin_target"
  printf 'Installed OpenCode plugin to %s\n' "$plugin_target"
fi

printf '\nRestart OpenCode, then run:\n'
printf '  /pm-fix 修复 WritableCurveHandle 的 data race\n'
