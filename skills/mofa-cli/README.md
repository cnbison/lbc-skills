# mofa-cli

Pure Rust CLI for AI-powered content generation. Single binary, zero Node.js dependency.

Generates PPTX presentations, PNG cards, comic strips, infographics, and animated videos using Gemini and Dashscope APIs.

## Install

```bash
cd mofa-cli
cargo build --release
# Binary at target/release/mofa
```

## Quick Start

```bash
export GEMINI_API_KEY="your-gemini-key"

# Image slides (text baked into images)
echo '[{"prompt":"TITLE: AI Trends 2026\n3 key predictions with icons"}]' | \
  mofa slides --style nb-pro --out deck.pptx --slide-dir /tmp/slides

# Editable PPTX (text as editable text boxes)
export DASHSCOPE_API_KEY="your-dashscope-key"
echo '[{"prompt":"TITLE: AI Trends 2026\n3 key predictions with icons"}]' | \
  mofa slides --style nb-pro --auto-layout --out deck.pptx --slide-dir /tmp/slides
```

## Pipelines

### slides — PPTX presentations

```bash
mofa slides [OPTIONS] --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

**Two modes:**

| Mode | Flag | Text editable? | Requires |
|------|------|---------------|----------|
| **Image** (default) | *(none)* | No — text baked into AI image | GEMINI_API_KEY |
| **Editable** | `--auto-layout` | Yes — text as PowerPoint text boxes | GEMINI_API_KEY + DASHSCOPE_API_KEY |

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--style` | `nb-pro` | Style template (see [Styles](#styles)) |
| `-o`, `--out` | required | Output `.pptx` path |
| `--slide-dir` | required | Directory for intermediate PNGs |
| `--auto-layout` | off | Enable editable text mode |
| `--concurrency` | `5` | Parallel generation workers (1-20) |
| `--image-size` | — | `1K`, `2K`, or `4K` |
| `--gen-model` | config | Gemini model for image generation |
| `--vision-model` | config | Vision model for text extraction |
| `--ref-image-size` | — | Lower-res size for reference images |
| `--refine` | off | Use Qwen-Edit for additional text cleanup |
| `-i`, `--input` | stdin | Input JSON file |
| `--root` | auto | Path to mofa-skills root directory |

**Input JSON:**

```json
[
  { "prompt": "TITLE: \"核心发现\"\n3 cards showing key metrics", "style": "normal" },
  { "prompt": "TITLE: \"数据对比\"\nComparison table", "style": "data" },
  { "prompt": "Cover slide with dramatic background", "style": "cover" }
]
```

| Field | Type | Description |
|-------|------|-------------|
| `prompt` | string | Required. Slide content description for AI |
| `style` | string | Style variant: `"normal"`, `"cover"`, `"data"`, `"warm"` |
| `auto_layout` | bool | Per-slide override for editable mode |
| `images` | string[] | Reference image paths for style guidance |
| `source_image` | string | Existing image to use as-is (skip generation) |
| `gen_model` | string | Per-slide model override |

#### PDF-to-PPTX conversion

Convert a PDF into an editable PPTX:

```bash
# 1. Export PDF pages as PNGs
pdftoppm -png -r 300 input.pdf /tmp/pages/page

# 2. Create input JSON with source_image per page
cat > /tmp/input.json << 'EOF'
[
  { "prompt": "Slide content description", "source_image": "/tmp/pages/page-1.png" },
  { "prompt": "Slide content description", "source_image": "/tmp/pages/page-2.png" }
]
EOF

# 3. Run with --auto-layout
mofa slides --auto-layout --style nb-pro --out output.pptx --slide-dir /tmp/slides -i /tmp/input.json
```

The pipeline OCRs text from the source image, removes text with programmatic fill, and overlays editable text boxes.

#### How autoLayout works

1. **Reference image** — Generate image with text (or copy `source_image`)
2. **Text extraction** — Gemini VQA extracts text positions, fonts, colors, alignment
3. **Text removal** — OCR detects word bounding boxes, fills each with sampled background color
4. **PPTX build** — Clean background image + editable text boxes

### cards — PNG greeting cards

```bash
mofa cards --style cny-guochao --card-dir /tmp/cards -i cards.json
```

| Flag | Default | Description |
|------|---------|-------------|
| `--style` | `cny-guochao` | Style template |
| `--card-dir` | required | Output directory for PNGs |
| `--aspect` | — | Aspect ratio (e.g. `1:1`, `9:16`) |
| `--concurrency` | `5` | Parallel workers |
| `--image-size` | — | `1K`, `2K`, or `4K` |

Input: `[{ "prompt": "...", "style": "normal" }]`

### comic — Multi-panel comic strips

```bash
mofa comic --style xkcd --out comic.png -i panels.json --layout grid --gutter 20
```

| Flag | Default | Description |
|------|---------|-------------|
| `--style` | `xkcd` | Style template |
| `-o`, `--out` | required | Output PNG path |
| `--layout` | `horizontal` | `horizontal`, `vertical`, or `grid` |
| `--gutter` | `20` | Gap between panels in pixels |
| `--concurrency` | `3` | Parallel workers |
| `--refine` | off | Refine panels with Qwen-Edit |

Input: `[{ "prompt": "Panel 1: ...", "style": "normal" }]`

### infographic — Multi-section infographics

```bash
mofa infographic --style cyberpunk-neon --out infographic.png -i sections.json
```

| Flag | Default | Description |
|------|---------|-------------|
| `--style` | `cyberpunk-neon` | Style template |
| `-o`, `--out` | required | Output PNG path |
| `--aspect` | — | Aspect ratio per section |
| `--gutter` | `0` | Gap between sections |
| `--concurrency` | `3` | Parallel workers |
| `--refine` | off | Refine sections with Qwen-Edit |

Input: `[{ "prompt": "...", "style": "header" }]`

Sections generated in parallel, stitched vertically.

### video — Animated video cards (Veo)

```bash
mofa video --style video-card --anim-style shuimo --card-dir /tmp/video -i cards.json --bgm music.mp3
```

| Flag | Default | Description |
|------|---------|-------------|
| `--style` | `video-card` | Image style template |
| `--anim-style` | `shuimo` | Animation style template |
| `--card-dir` | required | Working directory |
| `--bgm` | — | Background music file |
| `--aspect` | `9:16` | Image aspect ratio |
| `--still-duration` | `2.0` | Still image duration (seconds) |
| `--crossfade-dur` | `1.0` | Crossfade duration (seconds) |
| `--fade-out-dur` | `1.5` | Fade out duration (seconds) |
| `--music-volume` | `0.3` | BGM volume (0.0-1.0) |

Requires `ffmpeg` and `ffprobe` in PATH.

Phase 1: Generate still images. Phase 2: Animate with Gemini Veo. Phase 3: Composite with ffmpeg.

## Styles

16 built-in styles in `mofa-slides/styles/*.toml`:

| Style | Theme | Best for |
|-------|-------|----------|
| `nb-pro` | Professional purple | Business presentations |
| `agentic-enterprise-red` | Red wireframe 4K | Enterprise AI (Huawei-style) |
| `agentic-enterprise` | Purple wireframe 4K | Enterprise AI consulting |
| `nordic-minimal` | Pure white, red accent | Minimalist, MUJI/IKEA |
| `nb-br` | Blade Runner dark | Sci-fi, cinematic |
| `cc-research` | Golden hour, warm amber | Research, warm cinematic |
| `dark-community` | Corporate blue, AI orbs | Open source community |
| `what-is-life` | Science wireframes | Academic, study notes |
| `opensource` | Lavender, cartoon whale | Open source, cute |
| `tectonic` | Lavender gradient | Consulting, strategy |
| `vlinka-dji` | Dark cinematic, cyan | Product launches |
| `multi-brand` | Multi-company | Tech company comparisons |
| `relevant` | Ultra-minimal figures | Greeting cards |
| `openclaw-red` | Red/black claw motifs | Open source corporate |
| `fengzikai` | Ink brush, xuan paper | Chinese art, 丰子恺风格 |
| `lingnan` | Watercolor, flowers/birds | Chinese art, 岭南画派 |

Each style has variants (`normal`, `cover`, `data`, `warm`) — specify per-slide via the `style` field in input JSON.

### Custom styles

Create a TOML file:

```toml
[meta]
name = "my-style"
display_name = "My Style"
description = "Description"

[variants]
default = "normal"

[variants.normal]
prompt = """
Create a presentation slide image. 1920x1080 pixels, 16:9 format.
Your design system prompt here...
"""

[variants.cover]
prompt = """
Cover slide variant...
"""
```

Place in `mofa-slides/styles/` and use with `--style my-style`.

## Configuration

Reads `mofa/config.json` from the mofa root directory:

```json
{
  "api_keys": {
    "gemini": "env:GEMINI_API_KEY",
    "dashscope": "env:DASHSCOPE_API_KEY"
  },
  "gen_model": "gemini-3.1-flash-image-preview",
  "vision_model": "gemini-2.5-flash",
  "edit_model": "qwen-image-edit-max-2026-01-16",
  "deepseek_ocr_url": "http://localhost:8080/v1/ocr",
  "defaults": {
    "slides": { "style": "nb-pro", "image_size": "2K", "concurrency": 5 }
  }
}
```

### API keys

| Key | Required for | Set via |
|-----|-------------|---------|
| `GEMINI_API_KEY` | All pipelines | `export GEMINI_API_KEY="..."` |
| `DASHSCOPE_API_KEY` | `--auto-layout` (editable mode), `--refine` | `export DASHSCOPE_API_KEY="..."` |
| `DEEPSEEK_OCR_URL` | Optional: local OCR for `--auto-layout` | `export DEEPSEEK_OCR_URL="http://localhost:8080/v1/ocr"` |

Use `"env:VAR_NAME"` in config.json to reference environment variables. Never commit literal keys.

### Models

| Config key | Default | Used for |
|-----------|---------|----------|
| `gen_model` | `gemini-3.1-flash-image-preview` | Image generation |
| `vision_model` | `gemini-2.5-flash` | VQA text extraction (autoLayout) |
| `edit_model` | `qwen-image-edit-max-2026-01-16` | Qwen-Edit refinement |
| `deepseek_ocr_url` | *(none)* | Local DeepSeek-OCR-2 for precise text positions |

#### autoLayout text extraction priority

When `--auto-layout` is enabled, text positions are extracted using the first available method:

1. **DeepSeek-OCR-2** (local, `DEEPSEEK_OCR_URL`) — pixel-accurate grounding boxes, semantic block grouping, no API key needed
2. **Dashscope OCR** (`DASHSCOPE_API_KEY`) — word-level OCR boxes, programmatic grouping
3. **Gemini VQA** (fallback) — vision model guesses positions, least accurate

DeepSeek-OCR-2 provides the best results for dense slides with mixed Chinese/English text. Run locally with [deepseek-ai/DeepSeek-OCR-2](https://huggingface.co/deepseek-ai/DeepSeek-OCR-2).

## Architecture

```
src/
├── main.rs              # CLI entry point (clap subcommands)
├── config.rs            # config.json loading, env:VAR resolution
├── style.rs             # TOML style loading, variant lookup
├── gemini.rs            # Gemini image gen + vision QA
├── dashscope.rs         # Dashscope OCR + Qwen-Edit + text removal
├── deepseek_ocr.rs      # DeepSeek-OCR-2 local inference client
├── layout.rs            # Text extraction (DeepSeek/Dashscope/VQA), bbox calibration
├── pptx.rs              # PPTX builder (Office Open XML / DrawingML)
├── image_util.rs        # Image stitching (horizontal/vertical/grid)
├── veo.rs               # Gemini Veo video generation
└── pipeline/
    ├── slides.rs         # Slide generation + autoLayout pipeline
    ├── cards.rs          # Parallel card generation
    ├── comic.rs          # Comic panel gen + stitch
    ├── infographic.rs    # Infographic section gen + vertical stitch
    └── video.rs          # Veo animation + ffmpeg compositing
```
