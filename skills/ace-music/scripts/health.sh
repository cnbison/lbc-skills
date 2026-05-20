#!/usr/bin/env bash
# ACE Music API health check
# Usage: ./health.sh
# Returns 0 if API is reachable, 1 otherwise.

set -euo pipefail

API_KEY="${ACE_MUSIC_API_KEY:-}"
BASE_URL="${ACE_MUSIC_BASE_URL:-https://api.acemusic.ai}"

if [[ -z "$API_KEY" ]]; then
  echo "ERROR: ACE_MUSIC_API_KEY not set." >&2
  echo "Get your free API key at: https://acemusic.ai/playground/api-key" >&2
  exit 1
fi

echo "Checking ACE Music API at $BASE_URL ..."

RESPONSE=$(curl -s -w "\n%{http_code}" "${BASE_URL}/health" \
  -H "Authorization: Bearer $API_KEY" \
  || true)

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [[ "$HTTP_CODE" == "200" ]]; then
  echo "✅ ACE Music API is healthy"
  echo "$BODY"
  exit 0
else
  echo "❌ ACE Music API unreachable (HTTP $HTTP_CODE)" >&2
  echo "$BODY" >&2
  exit 1
fi
