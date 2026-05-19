#!/usr/bin/env bash
# Wrapper for the mofa-podcast `main` binary.
#
# Why this exists:
#   - The binary reads its JSON payload from STDIN, not argv.
#   - On macOS the binary may carry com.apple.quarantine and get killed
#     by Gatekeeper with exit code 137. We strip it on first run.
#
# Usage:
#   scripts/run.sh <tool> '<json>'
#   scripts/run.sh <tool> < payload.json
#   echo '<json>' | scripts/run.sh <tool>
#
# <tool> ∈ { podcast_voices | podcast_generate | podcast_voice_save }
# Empty JSON is allowed for podcast_voices.

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${SKILL_DIR}/main"

if [[ ! -x "$BIN" ]]; then
  echo "error: binary not found or not executable: $BIN" >&2
  exit 2
fi

# Idempotent quarantine removal on macOS — silent if attribute absent.
if [[ "$(uname -s)" == "Darwin" ]]; then
  xattr -d com.apple.quarantine "$BIN" 2>/dev/null || true
fi

if [[ $# -lt 1 ]]; then
  echo "usage: $(basename "$0") <tool> ['<json>']" >&2
  echo "  tools: podcast_voices | podcast_generate | podcast_voice_save" >&2
  exit 2
fi

TOOL="$1"
shift

if [[ $# -gt 0 ]]; then
  printf '%s' "$1" | "$BIN" "$TOOL"
else
  "$BIN" "$TOOL"
fi
