#!/bin/bash
# OpenClaw Daily News Setup Script

set -e

echo "================================"
echo "OpenClaw Daily News Setup"
echo "================================"
echo ""

# Check Python version
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "Detected Python: $python_version"

# Create virtual environment
if [ ! -d "venv" ]; then
    echo ""
    echo "Creating virtual environment..."
    python3 -m venv venv
else
    echo "Virtual environment already exists"
fi

# Activate virtual environment
echo ""
echo "Activating virtual environment..."
source venv/bin/activate

# Upgrade pip
echo ""
echo "Upgrading pip..."
pip install --upgrade pip

# Install dependencies
echo ""
echo "Installing dependencies..."
pip install -r requirements.txt

# Create necessary directories
echo ""
echo "Creating directories..."
mkdir -p data output

# Make run_pipeline.py executable
chmod +x run_pipeline.py

echo ""
echo "================================"
echo "Setup complete!"
echo "================================"
echo ""
echo "Next steps:"
echo ""
echo "1. (可选) 如需 MoFA FM 发布或 Doubao TTS，配置 .env:"
echo "   - cp .env.example .env"
echo "   - 编辑 .env 填入可选服务的 Key"
echo ""
echo "2. (可选) Install ffmpeg for audio processing:"
echo "   - macOS: brew install ffmpeg"
echo "   - Linux: sudo apt install ffmpeg"
echo ""
echo "3. Run the pipeline via Claude Code Agent:"
echo "   - 对 Claude 说: 生成日报"
echo ""

