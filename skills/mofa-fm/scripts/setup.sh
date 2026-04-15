#!/bin/bash
set -e

echo "=== mofa-fm setup ==="

# 1. Check/install ominix-api
if command -v ominix-api &>/dev/null; then
    echo "[OK] ominix-api found: $(which ominix-api)"
else
    echo "[MISSING] ominix-api not found. Installing from source..."
    echo "  cargo install --git https://github.com/OminiX-ai/OminiX-MLX ominix-api --features tts"
    cargo install --git https://github.com/OminiX-ai/OminiX-MLX ominix-api --features tts
fi

# 2. Check models
MODEL_DIR="${HOME}/.OminiX/models"

if [ -d "${MODEL_DIR}/Qwen3-TTS-12Hz-1.7B-CustomVoice-8bit" ]; then
    echo "[OK] CustomVoice model (preset voices)"
else
    echo "[MISSING] CustomVoice model — downloading..."
    echo "  ominix-api --download Qwen3-TTS-12Hz-1.7B-CustomVoice-8bit"
    ominix-api --download Qwen3-TTS-12Hz-1.7B-CustomVoice-8bit
fi

if [ -d "${MODEL_DIR}/Qwen3-TTS-12Hz-1.7B-Base" ]; then
    echo "[OK] Base model (voice cloning)"
else
    echo "[OPTIONAL] Base model for voice cloning not found."
    echo "  To enable voice cloning: ominix-api --download Qwen3-TTS-12Hz-1.7B-Base"
fi

echo ""
echo "Setup complete. Start the server:"
echo "  ominix-api --tts-port 8082 --clone-port 8083"
