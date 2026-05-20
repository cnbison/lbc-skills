#!/usr/bin/env bash
# ACE-Step Music Generation via API
# Usage: generate.sh <prompt> [options]
# Options: --lyrics "..." --duration 30 --language en --instrumental --output file.mp3
#          --bpm 120 --key "C major" --seed 42 --sample-mode --batch 2

set -euo pipefail

# Config
API_KEY="${ACE_MUSIC_API_KEY:-}"
BASE_URL="${ACE_MUSIC_BASE_URL:-https://api.acemusic.ai}"
OUTPUT_DIR="${ACE_MUSIC_OUTPUT_DIR:-./output/ace-music}"

# Defaults
DURATION=""
LANGUAGE=""
INSTRUMENTAL="null"
BPM=""
KEY_SCALE=""
SEED=""
SAMPLE_MODE="false"
BATCH_SIZE=1
LYRICS=""
PROMPT=""
FORMAT="mp3"

# Validate numeric helpers
validate_int() {
  local val="$1" name="$2"
  if [[ -n "$val" && ! "$val" =~ ^[0-9]+$ ]]; then
    echo "ERROR: $name must be an integer, got '$val'" >&2
    exit 1
  fi
}
validate_number() {
  local val="$1" name="$2"
  if [[ -n "$val" && ! "$val" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
    echo "ERROR: $name must be a number, got '$val'" >&2
    exit 1
  fi
}

# Parse args
while [[ $# -gt 0 ]]; do
  case "$1" in
    --lyrics) LYRICS="$2"; shift 2 ;;
    --duration) DURATION="$2"; shift 2 ;;
    --language) LANGUAGE="$2"; shift 2 ;;
    --instrumental) INSTRUMENTAL="true"; shift ;;
    --output|-o) OUTPUT="$2"; shift 2 ;;
    --bpm) BPM="$2"; shift 2 ;;
    --key) KEY_SCALE="$2"; shift 2 ;;
    --seed) SEED="$2"; shift 2 ;;
    --sample-mode) SAMPLE_MODE="true"; shift ;;
    --batch) BATCH_SIZE="$2"; shift 2 ;;
    --format) FORMAT="$2"; shift ;;
    *) PROMPT="$1"; shift ;;
  esac
done

# Validate numeric inputs
validate_number "$DURATION" "duration"
validate_number "$BPM" "bpm"
validate_int "$SEED" "seed"
validate_int "$BATCH_SIZE" "batch_size"

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"
OUTPUT="${OUTPUT:-${OUTPUT_DIR}/$(date +%Y%m%d-%H%M%S).mp3}"

if [[ -z "$API_KEY" ]]; then
  echo "ERROR: ACE_MUSIC_API_KEY not set." >&2
  echo "Get your free API key at: https://acemusic.ai/playground/api-key" >&2
  exit 1
fi

if [[ -z "$PROMPT" && "$SAMPLE_MODE" == "false" ]]; then
  echo "Usage: generate.sh <prompt> [--lyrics '...'] [--duration 30] [--language en] [--instrumental] [--output file.mp3]" >&2
  exit 1
fi

# Build audio_config
AUDIO_CONFIG="{\"format\":\"$FORMAT\""
[[ -n "$LANGUAGE" ]] && AUDIO_CONFIG="$AUDIO_CONFIG,\"vocal_language\":\"$LANGUAGE\""
[[ -n "$DURATION" ]] && AUDIO_CONFIG="$AUDIO_CONFIG,\"duration\":$DURATION"
[[ -n "$BPM" ]] && AUDIO_CONFIG="$AUDIO_CONFIG,\"bpm\":$BPM"
[[ "$INSTRUMENTAL" != "null" ]] && AUDIO_CONFIG="$AUDIO_CONFIG,\"instrumental\":$INSTRUMENTAL"
[[ -n "$KEY_SCALE" ]] && AUDIO_CONFIG="$AUDIO_CONFIG,\"key_scale\":\"$KEY_SCALE\""
AUDIO_CONFIG="$AUDIO_CONFIG}"

# Build message content
if [[ -n "$LYRICS" && -n "$PROMPT" ]]; then
  # Tagged mode — real newline, not literal \n
  CONTENT="<prompt>${PROMPT}</prompt>
<lyrics>${LYRICS}</lyrics>"
elif [[ -n "$LYRICS" ]]; then
  CONTENT="$LYRICS"
else
  CONTENT="$PROMPT"
fi

# Escape for JSON
CONTENT_ESCAPED=$(echo -n "$CONTENT" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))" | sed 's/^"//;s/"$//')

# Build request body
BODY="{\"messages\":[{\"role\":\"user\",\"content\":\"$CONTENT_ESCAPED\"}],\"audio_config\":$AUDIO_CONFIG,\"stream\":false"
[[ "$SAMPLE_MODE" == "true" ]] && BODY="$BODY,\"sample_mode\":true"
[[ -n "$SEED" ]] && BODY="$BODY,\"seed\":$SEED"
[[ "$BATCH_SIZE" -gt 1 ]] && BODY="$BODY,\"batch_size\":$BATCH_SIZE"
BODY="$BODY}"

echo "🎵 Generating music..." >&2
echo "   Prompt: ${PROMPT:-[lyrics/sample mode]}" >&2
[[ -n "$DURATION" ]] && echo "   Duration: ${DURATION}s" >&2
[[ -n "$LANGUAGE" ]] && echo "   Language: $LANGUAGE" >&2

# API call
RESPONSE=$(curl -s -X POST "${BASE_URL}/v1/chat/completions" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d "$BODY")

# Single-pass Python via stdin (safe from quote injection)
echo "$RESPONSE" | python3 -c "
import sys, json, base64

d = json.load(sys.stdin)

# Check for errors / choices
if 'choices' not in d:
    print('ERROR: API request failed', file=sys.stderr)
    print(json.dumps(d, indent=2), file=sys.stderr)
    sys.exit(1)

msg = d['choices'][0]['message']
audios = msg.get('audio', [])

if not audios:
    print('ERROR: No audio in response', file=sys.stderr)
    print(json.dumps(d, indent=2), file=sys.stderr)
    sys.exit(1)

# Extract metadata
metadata = msg.get('content', '')

# Decode and save all audio files
output = '$OUTPUT'
for i, a in enumerate(audios):
    url = a['audio_url']['url']
    b64 = url.split(',', 1)[1]
    if len(audios) == 1:
        fname = output
    else:
        stem, ext = output.rsplit('.', 1)
        fname = f'{stem}_{i+1}.{ext}'
    with open(fname, 'wb') as f:
        f.write(base64.b64decode(b64))
    print(f'Saved: {fname}', file=sys.stderr)
    print(fname)

# Print metadata
if metadata:
    print('', file=sys.stderr)
    print('📋 Metadata:', file=sys.stderr)
    print(metadata, file=sys.stderr)
"
