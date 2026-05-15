#!/usr/bin/env bash
# Integration tests for mofa-podcast
# Usage: ./scripts/test-integration.sh [binary_path] [ominix_url]
#
# Requires: ominix-api running, ffmpeg installed
# Without ominix-api, only offline tests run (parser, voices, error handling)

set -euo pipefail

BINARY="${1:-./target/release/mofa-podcast}"
OMINIX_URL="${2:-http://localhost:8080}"
TMPDIR="$(mktemp -d)"
PASS=0
FAIL=0
SKIP=0

cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

pass() { PASS=$((PASS + 1)); echo "  PASS: $1"; }
fail() { FAIL=$((FAIL + 1)); echo "  FAIL: $1 — $2"; }
skip() { SKIP=$((SKIP + 1)); echo "  SKIP: $1 — $2"; }

check_json_field() {
    local json="$1" field="$2" expected="$3"
    local actual
    actual=$(echo "$json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('$field',''))" 2>/dev/null || echo "PARSE_ERROR")
    if [[ "$actual" == *"$expected"* ]]; then
        return 0
    else
        return 1
    fi
}

echo "=== mofa-podcast integration tests ==="
echo "Binary: $BINARY"
echo "Temp dir: $TMPDIR"
echo ""

# ── 1. Binary basics ───────────────────────────────────────────────

echo "--- 1. Binary basics ---"

# 1.1 No args → error
OUT=$(echo '{}' | "$BINARY" 2>&1 || true)
if echo "$OUT" | grep -q "Usage:"; then
    pass "1.1 No tool name shows usage"
else
    fail "1.1 No tool name shows usage" "got: $OUT"
fi

# 1.2 Unknown tool → error
OUT=$(echo '{}' | "$BINARY" unknown_tool 2>&1 || true)
if echo "$OUT" | grep -q "Unknown tool"; then
    pass "1.2 Unknown tool returns error"
else
    fail "1.2 Unknown tool returns error" "got: $OUT"
fi

# ── 2. podcast_voices ─────────────────────────────────────────────

echo ""
echo "--- 2. podcast_voices ---"

OUT=$(echo '{}' | "$BINARY" podcast_voices 2>/dev/null)

# 2.1 Returns success
if check_json_field "$OUT" "success" "True"; then
    pass "2.1 podcast_voices returns success=true"
else
    fail "2.1 podcast_voices returns success=true" "$OUT"
fi

# 2.2 Lists preset voices
if echo "$OUT" | grep -q "vivian"; then
    pass "2.2 Lists vivian preset"
else
    fail "2.2 Lists vivian preset" "$OUT"
fi

# 2.3 Lists all 9 presets
for v in vivian serena ryan aiden eric dylan uncle_fu ono_anna sohee; do
    if echo "$OUT" | grep -q "$v"; then
        pass "2.3 Lists preset: $v"
    else
        fail "2.3 Lists preset: $v" "not found"
    fi
done

# ── 3. podcast_generate — error handling ───────────────────────────

echo ""
echo "--- 3. podcast_generate error handling ---"

# 3.1 No script provided
OUT=$(echo '{}' | "$BINARY" podcast_generate 2>/dev/null || true)
if echo "$OUT" | grep -q "script.*script_path"; then
    pass "3.1 Missing script returns helpful error"
else
    fail "3.1 Missing script returns helpful error" "$OUT"
fi

# 3.2 Empty script
OUT=$(echo '{"script": "# Just a title\n---\n"}' | "$BINARY" podcast_generate 2>/dev/null || true)
if echo "$OUT" | grep -q "No dialogue lines"; then
    pass "3.2 Script with no dialogue returns error"
else
    fail "3.2 Script with no dialogue returns error" "$OUT"
fi

# 3.3 Nonexistent script_path
OUT=$(echo '{"script_path": "/nonexistent/path.md"}' | "$BINARY" podcast_generate 2>/dev/null || true)
if echo "$OUT" | grep -q "Failed to read"; then
    pass "3.3 Nonexistent script_path returns error"
else
    fail "3.3 Nonexistent script_path returns error" "$OUT"
fi

# 3.4 Invalid JSON
OUT=$(echo 'not json' | "$BINARY" podcast_generate 2>/dev/null || true)
if echo "$OUT" | grep -q "Invalid input"; then
    pass "3.4 Invalid JSON returns error"
else
    fail "3.4 Invalid JSON returns error" "$OUT"
fi

# 3.5 Malformed non-metadata line
OUT=$(echo '{"script": "[Host - vivian, calm] ok\nthis is malformed"}' | "$BINARY" podcast_generate 2>/dev/null || true)
if echo "$OUT" | grep -q "malformed non-metadata lines"; then
    pass "3.5 Malformed script line returns error"
else
    fail "3.5 Malformed script line returns error" "$OUT"
fi

# 3.6 Unknown voice fails before generation
OUT=$(echo '{"script": "[Host - not_a_real_voice, calm] hello"}' | "$BINARY" podcast_generate 2>/dev/null || true)
if echo "$OUT" | grep -q "unknown voice"; then
    pass "3.6 Unknown voice returns helpful error"
else
    fail "3.6 Unknown voice returns helpful error" "$OUT"
fi

# ── 4. Script file input ──────────────────────────────────────────

echo ""
echo "--- 4. Script file input ---"

cat > "$TMPDIR/test_script.md" << 'SCRIPT'
# Test Podcast

**Genre**: talk-show | **Duration**: ~1 min | **Speakers**: 2

| Character | Voice | Type |
|-----------|-------|------|
| Host | vivian | built-in |
| Guest | ryan | built-in |

---

[BGM: Intro — fade-in, 3s]

[Host - vivian, cheerful] 大家好，欢迎收听今天的播客。

[Guest - ryan, excited] 谢谢邀请，非常高兴来到节目。

[PAUSE: 2s]

[Host - vivian, curious] 今天聊什么话题？

[Guest - ryan, thoughtful] 我想聊聊人工智能的未来。

[Host - vivian, warm] 非常精彩！感谢收听，下次再见。

[BGM: Outro — fade-out, 3s]
SCRIPT

# ── 5. End-to-end TTS test (requires ominix-api) ──────────────────

echo ""
echo "--- 5. End-to-end TTS generation ---"

# Check if ominix-api is reachable
if curl -sf "$OMINIX_URL/health" >/dev/null 2>&1 || curl -sf "$OMINIX_URL/v1/health" >/dev/null 2>&1; then
    OMINIX_AVAILABLE=true
    echo "  ominix-api available at $OMINIX_URL"
else
    OMINIX_AVAILABLE=false
    echo "  ominix-api not available — skipping TTS tests"
fi

if $OMINIX_AVAILABLE; then
    # 5.1 Generate from script_path
    OUT=$(echo "{\"script_path\": \"$TMPDIR/test_script.md\", \"output_dir\": \"$TMPDIR/output1\"}" \
        | OMINIX_API_URL="$OMINIX_URL" "$BINARY" podcast_generate 2>"$TMPDIR/stderr1.log")

    if check_json_field "$OUT" "success" "True"; then
        pass "5.1 podcast_generate from script_path succeeds"
    else
        fail "5.1 podcast_generate from script_path succeeds" "$OUT"
    fi

    # 5.2 Output MP3 exists
    MP3_PATH=$(echo "$OUT" | python3 -c "import json,sys; print(json.load(sys.stdin).get('files_to_send',[''])[0])" 2>/dev/null || echo "")
    if [[ -n "$MP3_PATH" && -f "$MP3_PATH" ]]; then
        pass "5.2 Output MP3 file exists"
    else
        fail "5.2 Output MP3 file exists" "path=$MP3_PATH"
    fi

    # 5.3 MP3 file is non-empty
    if [[ -f "$MP3_PATH" ]] && [[ $(stat -f%z "$MP3_PATH" 2>/dev/null || stat -c%s "$MP3_PATH" 2>/dev/null) -gt 1000 ]]; then
        pass "5.3 MP3 file is non-trivial (>1KB)"
    else
        fail "5.3 MP3 file is non-trivial (>1KB)" "file too small or missing"
    fi

    # 5.4 Verify audio with ffprobe
    if command -v ffprobe >/dev/null 2>&1 && [[ -f "$MP3_PATH" ]]; then
        DURATION=$(ffprobe -v quiet -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "$MP3_PATH" 2>/dev/null || echo "0")
        if (( $(echo "$DURATION > 5" | bc -l 2>/dev/null || echo 0) )); then
            pass "5.4 Audio duration > 5s (got ${DURATION}s)"
        else
            fail "5.4 Audio duration > 5s" "got ${DURATION}s"
        fi
    else
        skip "5.4 Audio duration check" "ffprobe not available"
    fi

    # 5.5 Segments cleanup
    if [[ ! -d "$TMPDIR/output1/segments" ]]; then
        pass "5.5 Segment files cleaned up after concat"
    else
        fail "5.5 Segment files cleaned up after concat" "segments dir still exists"
    fi

    # 5.6 Stderr shows correct phase ordering
    STDERR=$(cat "$TMPDIR/stderr1.log")
    if echo "$STDERR" | grep -q "Phase 1:.*built-in"; then
        pass "5.6 Phase 1 (built-in) logged"
    else
        fail "5.6 Phase 1 (built-in) logged" "not found in stderr"
    fi

    if echo "$STDERR" | grep -q "Phase 3:.*timeline"; then
        pass "5.7 Phase 3 (timeline assembly) logged"
    else
        fail "5.7 Phase 3 (timeline assembly) logged" "not found in stderr"
    fi

    # 5.8 Voice grouping: ryan segments before vivian in generation order
    FIRST_VOICE=$(echo "$STDERR" | grep '^\[podcast\] \[1/' | sed -n 's/.*(\([a-z_]*\),.*/\1/p' | head -1)
    if [[ "$FIRST_VOICE" == "ryan" ]]; then
        pass "5.8 Voices grouped: ryan generated before vivian"
    else
        fail "5.8 Voices grouped: ryan generated before vivian" "first voice was $FIRST_VOICE"
    fi

    # 5.9 Generate from inline script (single speaker, minimal)
    OUT=$(echo '{"script": "[Solo - vivian, calm] This is a single speaker test.", "output_dir": "'"$TMPDIR/output2"'"}' \
        | OMINIX_API_URL="$OMINIX_URL" "$BINARY" podcast_generate 2>/dev/null)
    if check_json_field "$OUT" "success" "True"; then
        pass "5.9 Single-speaker inline script succeeds"
    else
        fail "5.9 Single-speaker inline script succeeds" "$OUT"
    fi

    # 5.10 Generate with emotion variations
    cat > "$TMPDIR/emotion_test.md" << 'EMO'
[A - vivian, excited] Wow, this is amazing!
[B - ryan, serious] Let me explain the situation carefully.
[C - vivian, sad] That's really unfortunate news.
EMO
    OUT=$(echo "{\"script_path\": \"$TMPDIR/emotion_test.md\", \"output_dir\": \"$TMPDIR/output3\"}" \
        | OMINIX_API_URL="$OMINIX_URL" "$BINARY" podcast_generate 2>/dev/null)
    if check_json_field "$OUT" "success" "True"; then
        pass "5.10 Multiple emotions generate successfully"
    else
        fail "5.10 Multiple emotions generate successfully" "$OUT"
    fi

else
    for i in $(seq 1 10); do
        skip "5.$i TTS test" "ominix-api not available"
    done
fi

# ── Summary ───────────────────────────────────────────────────────

echo ""
echo "=== Results ==="
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
echo "  SKIP: $SKIP"
echo "  TOTAL: $((PASS + FAIL + SKIP))"

if [[ $FAIL -gt 0 ]]; then
    exit 1
fi
exit 0
