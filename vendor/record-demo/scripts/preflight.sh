#!/usr/bin/env bash
# Preflight for record-demo camera takes. Exit 0 only when the vendored
# agent-browser + Chrome + ffmpeg setup is ready for recording.
set -euo pipefail

MIN_MAJOR=0
MIN_MINOR=43

die() {
  printf 'record-demo preflight: %s\n' "$*" >&2
  exit 1
}

if ! command -v agent-browser >/dev/null 2>&1; then
  die "agent-browser not on PATH (expected skills vendor/agent-browser-cli/node_modules/.bin first)"
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
  die "ffmpeg not on PATH (required for record start/stop)"
fi

version="$(agent-browser --version 2>/dev/null | head -1 | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+' || true)"
[[ -n "$version" ]] || die "could not parse agent-browser --version"

IFS=. read -r major minor _patch <<<"$version"
if (( major < MIN_MAJOR )) || { (( major == MIN_MAJOR )) && (( minor < MIN_MINOR )); }; then
  die "agent-browser $version is too old; need >= ${MIN_MAJOR}.${MIN_MINOR}.0 (record start must attach in-place, not create a fresh context)"
fi

# record help must describe attach-in-place behavior (guards accidental downgrade)
record_help="$(agent-browser record --help 2>&1 || true)"
if ! grep -qiE 'no new context|as-is' <<<"$record_help"; then
  die "agent-browser record help does not describe attach-in-place recording; refuse to shoot with this CLI"
fi
if ! grep -q -- '--fps' <<<"$record_help" || ! grep -q -- '--cursor' <<<"$record_help"; then
  die "agent-browser record is missing --fps/--cursor (template requires them); upgrade the vendored CLI"
fi

[[ -n "${AGENT_BROWSER_EXECUTABLE_PATH:-}" ]] || die "AGENT_BROWSER_EXECUTABLE_PATH is unset (point it at vendor/browsers/chrome-linux64/chrome)"
[[ -x "$AGENT_BROWSER_EXECUTABLE_PATH" ]] || die "Chrome not executable: $AGENT_BROWSER_EXECUTABLE_PATH"

args="${AGENT_BROWSER_ARGS:-}"
if [[ "$args" != *"--no-sandbox"* ]]; then
  die "AGENT_BROWSER_ARGS must include --no-sandbox (current: ${args:-<empty>})"
fi

# Stale daemons from an older CLI break sessions after upgrades.
agent-browser close --all >/dev/null 2>&1 || true

printf 'record-demo preflight: ok (agent-browser %s, chrome=%s)\n' "$version" "$AGENT_BROWSER_EXECUTABLE_PATH"
