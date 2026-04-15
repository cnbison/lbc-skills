# mofa-slides User Guide

Complete guide to creating AI-generated presentations with mofa-slides.

## Table of Contents

- [Getting Started](#getting-started)
- [Three Modes](#three-modes)
- [Style Gallery](#style-gallery)
- [Writing Effective Prompts](#writing-effective-prompts)
- [Slide Variants](#slide-variants)
- [Resolution & Quality](#resolution--quality)
- [Manual Text Overlays](#manual-text-overlays)
- [Rich Text Formatting](#rich-text-formatting)
- [Reference Images](#reference-images)
- [PDF-to-PPTX Conversion](#pdf-to-pptx-conversion)
- [Advanced Configuration](#advanced-configuration)
- [Input JSON Reference](#input-json-reference)
- [CLI Reference](#cli-reference)
- [Configuration File](#configuration-file)
- [Tips & Best Practices](#tips--best-practices)
- [Troubleshooting](#troubleshooting)

---

## Getting Started

### Prerequisites

- **GEMINI_API_KEY** — required for all modes
- **DASHSCOPE_API_KEY** — required only for editable mode (`--auto-layout`)
- **mofa** CLI binary (built from `mofa-cli/`)

### Your First Deck

1. Create an input file `slides.json`:

```json
[
  { "prompt": "TITLE: \"AI趋势报告\"\nSubtitle: 2026年行业展望\nCentered, dramatic dark background", "style": "cover" },
  { "prompt": "TITLE: \"三大趋势\"\n3 cards with icons:\n1. On-device AI\n2. Agent frameworks\n3. Open-source models", "style": "normal" },
  { "prompt": "TITLE: \"市场数据\"\nTable: Region, Revenue, Growth\nAsia $12B +35%\nEurope $8B +22%\nAmericas $15B +28%", "style": "data" }
]
```

2. Generate:

```bash
mofa slides --style nb-pro --out my-deck.pptx --slide-dir /tmp/slides -i slides.json
```

3. Open `my-deck.pptx` in PowerPoint / Keynote / Google Slides.

---

## Three Modes

### 1. Image Mode (Default)

Text is baked into the AI-generated image. Fast, beautiful, but text is not editable in PowerPoint.

```bash
mofa slides --style nb-pro --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

**Best for:** Final presentations, printed decks, visual impact.

### 2. Editable Mode

AI generates a reference image → extracts text via OCR/VQA → removes text from image → overlays editable PowerPoint text boxes. Slower but produces editable PPTX.

```bash
mofa slides --style nb-pro --auto-layout --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

**Requires:** `DASHSCOPE_API_KEY` (for text removal). Optional: `DEEPSEEK_OCR_URL` for higher-accuracy OCR.

**Best for:** Drafts that need editing, collaborative decks, iterative refinement.

### 3. PDF-to-PPTX Conversion

Convert existing PDF/image pages into editable PPTX. Provide page images as `source_image`, and mofa extracts text, cleans the background, and overlays editable text boxes.

```bash
# Step 1: Export PDF pages as PNGs (use any PDF tool)
# Step 2: Create input JSON with source_image paths
# Step 3: Run with --auto-layout
mofa slides --auto-layout --style nb-pro --out editable.pptx --slide-dir /tmp/edit -i pages.json
```

```json
[
  { "prompt": "page 1", "source_image": "/tmp/pdf-pages/page-01.png" },
  { "prompt": "page 2", "source_image": "/tmp/pdf-pages/page-02.png" }
]
```

---

## Style Gallery

17 built-in styles covering business, tech, art, science, and product themes.

### Business & Consulting

| Style | `--style` | Vibe | Resolution |
|-------|-----------|------|------------|
| Professional Purple | `nb-pro` | Soft lavender, clean corporate, McKinsey-lite | HD |
| Agentic Enterprise (Purple) | `agentic-enterprise` | Premium wireframe, purple accents, consulting-grade | HD |
| Agentic Enterprise (Red) | `agentic-enterprise-red` | Huawei red, bold tech-forward, 4K wireframes | 4K |
| Nordic Minimal | `nordic-minimal` | Pure white, red accent, MUJI/IKEA inspired | HD |
| Tectonic | `tectonic` | Lavender gradient, strategy consulting | HD |

### Tech & Product

| Style | `--style` | Vibe | Resolution |
|-------|-----------|------|------------|
| Blade Runner | `nb-br` | Dark cinematic, cyan/neon sci-fi | HD |
| DJI Launch | `vlinka-dji` | Dark cinematic, cyan rim lighting, product showcase | HD |
| Multi-Brand | `multi-brand` | Company-specific branding (Amazon, Google, Tesla, NVIDIA, SpaceX, Microsoft) | HD |
| Dark Community | `dark-community` | Corporate blue, open-source community | HD |

### Art & Culture

| Style | `--style` | Vibe | Resolution |
|-------|-----------|------|------------|
| Feng Zikai (丰子恺) | `fengzikai` | Ink brush art, xuan paper, childlike warmth, 40% white space | HD |
| Lingnan (岭南画派) | `lingnan` | Watercolor washes, flowers & birds, bold brushwork | HD |
| Relevant | `relevant` | Ultra-minimal stick figures, greeting-card style | HD |

### Academic & Research

| Style | `--style` | Vibe | Resolution |
|-------|-----------|------|------------|
| What is Life | `what-is-life` | Science wireframes, physics/biology domains | HD |
| Golden Hour | `cc-research` | Warm amber, cinematic research feel | HD |

### Community & Conference

| Style | `--style` | Vibe | Resolution |
|-------|-----------|------|------------|
| GOBI Conference | `gobi` | Azure-gold-aqua, tech conference, 4K | 4K |
| Open Source | `opensource` | Lavender, cute cartoon whale | HD |
| OpenClaw Red | `openclaw-red` | Red/black, open-source enterprise | HD |

### Style Details

#### Agentic Enterprise (Red)

Inspired by Huawei keynote decks. Deep black covers with red wireframe overlays, white content slides with large (35-40% of slide) wireframe illustrations showing network nodes, server racks, circuit patterns. Uses Manrope + Noto Sans SC fonts.

**Variants:** `normal` (content), `cover` (black + red wireframes), `data` (stats + wireframes)

#### Professional Purple (nb-pro)

Clean, sophisticated corporate look. Soft lavender gradient background with subtle wireframe corner decorations. Purple accent (#6B4FA0) for highlights. Slide numbers in bottom-right.

**Variants:** `normal` only

#### Feng Zikai (丰子恺)

Ink brush art inspired by the famous Chinese artist. Warm off-white xuan paper background with simple brush line drawings. Minimum 40% empty space (留白). Sparse watercolor washes in pale pink, green, yellow. Small red seal stamp in corner.

**Variants:** `normal` (content + illustrations), `cover` (large central illustration), `data` (hand-drawn brush-ink table borders)

#### Multi-Brand

Each variant uses a specific company's brand colors and design language:
- **Amazon:** Orange (#FF9900) wireframes on dark navy or white
- **Google:** Four-color wireframes (blue, red, yellow, green)
- **Microsoft:** Blue (#0078D4) + complementary colors, structured and trustworthy
- **NVIDIA:** Green (#76B900) wireframes on black, circuit board aesthetic
- **Tesla:** Red (#CC0000) wireframes, sleek automotive design studio
- **SpaceX:** Ice blue on deep space black, aerospace precision

**Variants:** `amazon_light`, `amazon_dark`, `google`, `microsoft`, `nvidia_light`, `nvidia_dark`, `tesla_light`, `tesla_dark`, `spacex`, `overview` (comparison), `cover`

#### DJI Launch (vlinka-dji)

Cinematic product showcase style. Dark charcoal gradients with dramatic studio lighting. Cyan (#00D4FF) rim lights creating glowing silhouettes. Products appear floating with reflections on glossy dark surfaces.

**Variants:** `cover` (hero shot), `hero` (large product showcase), `feature` (spec grid with cyan icons), `scene` (product in environment), `data` (large cyan stat numbers)

#### GOBI Conference

Modern tech conference aesthetic. Azure blue (#2B74FF), osmanthus gold (#FFC640), and aqua green (#00C896) palette. Subtle connected-nodes geometric patterns. Manrope headings.

**Variants:** `cover` (dark charcoal + lavender overlay), `normal` (ice-blue background), `data` (5-color data palette), `warm` (dark keynote mode with gold accents)

#### What is Life

Schrödinger-inspired science style. Multi-domain color coding: Physics (cyan), Biology (teal), Chemistry (amber), Thermodynamics (rose). Wireframe illustrations of atoms, DNA, crystal lattices.

**Variants:** `cover` (lavender gradient), `physics_dark` (dark navy + cyan), `biology_light` (white + teal), `overview` (multi-domain comparison)

---

## Writing Effective Prompts

The `prompt` field tells Gemini what to render on the slide. Good prompts produce better slides.

### Prompt Structure

```
TITLE: "Main Heading"
SUBTITLE: "Secondary text" (optional)
Content description: what to show, how to lay it out
```

### Layout Patterns

**3-Column Cards:**
```
TITLE: "Our Strategy"
3 cards with wireframe icons:
1. Growth icon — "Scale" — "Expand into 5 new markets"
2. Team icon — "Hire" — "Grow engineering team to 200"
3. Innovation icon — "Innovate" — "Launch 3 new products"
```

**Data Table:**
```
TITLE: "Q3 Results"
Table with columns: Metric, Q2, Q3, Change
Revenue $2.1B $3.2B +52%
Users 8.5M 12.1M +42%
NPS 68 74 +6pts
```

**Metric Cards:**
```
TITLE: "Key Metrics"
4 large stat cards in 2x2 grid:
Top-left: "$3.2B" — "Revenue"
Top-right: "+47%" — "YoY Growth"
Bottom-left: "12M" — "Active Users"
Bottom-right: "98.5%" — "Uptime"
```

**Timeline:**
```
TITLE: "Roadmap 2026"
Horizontal timeline with 4 milestones:
Q1: MVP Launch → Q2: Beta Program → Q3: GA Release → Q4: Enterprise Tier
```

**2x3 Grid:**
```
TITLE: "Product Features"
6 items in 2x3 grid with icons:
1. Cloud icon — "Cloud Native" — "Deploy anywhere"
2. Lock icon — "Secure" — "E2E encryption"
3. Speed icon — "Fast" — "Sub-10ms latency"
4. Scale icon — "Scalable" — "Auto-scaling"
5. API icon — "Extensible" — "Plugin system"
6. Globe icon — "Global" — "Multi-region"
```

**Cover Slide:**
```
TITLE: "Company Name"
SUBTITLE: "Tagline or event name"
Below subtitle: "Date or context"
At bottom: "website.com"
Centered vertically, dramatic background
```

### Bilingual Content

Prompts can mix Chinese and English:

```
TITLE: "AI编程重塑全栈竞争力"
SUBTITLE: "从高利润软件到AI赋能"
3 cards:
1. "软件市场" — "$3万亿" — 传统软件正在被AI重塑
2. "AI赋能" — "10x效率" — 开发效率提升10倍
3. "全栈能力" — "必备技能" — 从前端到部署全覆盖
```

### Tips for Better Prompts

1. **Be specific about layout** — "3 cards in a row" is better than "show some features"
2. **Include numbers** — Stats, metrics, and data make slides more impactful
3. **Describe the mood** — "dramatic dark background", "clean white", "cinematic"
4. **Mention icons** — "wireframe icon of a cloud", "gear icon", "network icon"
5. **Specify hierarchy** — "large heading", "small gray label", "bold number"
6. **Keep it focused** — One main idea per slide, don't overcrowd

---

## Slide Variants

Each style supports multiple variants for different slide types. Set per-slide in JSON:

```json
{ "prompt": "...", "style": "cover" }
{ "prompt": "...", "style": "data" }
{ "prompt": "...", "style": "normal" }
```

### Common Variants

| Variant | Purpose | When to Use |
|---------|---------|-------------|
| `normal` | Standard content slide | Most slides (default if not specified) |
| `cover` | Title/introduction slide | First slide, section dividers |
| `data` | Data-heavy content | Tables, charts, metrics, specifications |
| `warm` | Dark/warm color variant | Keynote-style, dramatic emphasis |

### Style-Specific Variants

| Style | Special Variants | Purpose |
|-------|-----------------|---------|
| `multi-brand` | `amazon_light`, `amazon_dark`, `google`, `microsoft`, `nvidia_light`, `nvidia_dark`, `tesla_light`, `tesla_dark`, `spacex`, `overview` | Company-branded slides |
| `what-is-life` | `physics_dark`, `biology_light`, `overview` | Science domain-specific |
| `relevant` | `front`, `greeting`, `scene`, `festive` | Greeting card variants |
| `vlinka-dji` | `hero`, `feature`, `scene` | Product showcase variants |

### Mixing Variants in One Deck

You can use different variants for different slides:

```json
[
  { "prompt": "Title slide...", "style": "cover" },
  { "prompt": "Mission...", "style": "normal" },
  { "prompt": "Revenue table...", "style": "data" },
  { "prompt": "Vision...", "style": "warm" },
  { "prompt": "Key metrics...", "style": "data" },
  { "prompt": "Thank you...", "style": "cover" }
]
```

---

## Resolution & Quality

### Image Size

Controls the resolution of generated slide images:

| Value | Resolution | Quality | Speed | Cost |
|-------|-----------|---------|-------|------|
| `1K` | ~1024px | Good for drafts | Fast | Low |
| `2K` | ~2048px | Recommended default | Medium | Medium |
| `4K` | ~4096px | Maximum detail | Slow | Higher |

```bash
mofa slides --image-size 4K --style gobi --out hires.pptx --slide-dir /tmp/hires -i slides.json
```

Note: Some styles (agentic-enterprise-red, gobi) are designed for 4K and their prompts specify 3840x2160. Using `--image-size 4K` matches their intended quality.

### Generation Model

Override the AI model used for image generation:

```bash
# Use a specific model for all slides
mofa slides --gen-model gemini-2.5-pro-preview --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

Or per-slide in JSON:
```json
{ "prompt": "...", "gen_model": "gemini-2.5-pro-preview" }
```

Default: `gemini-3.1-flash-image-preview`

### Concurrency

Control how many slides are generated in parallel:

```bash
# Generate 10 slides at a time (faster but more API load)
mofa slides --concurrency 10 --out deck.pptx --slide-dir /tmp/slides -i slides.json

# Generate 1 at a time (slower but less API pressure)
mofa slides --concurrency 1 --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

Default: `5`. Useful range: `1`-`20`.

### Reference Image Size (Editable Mode)

In editable mode (`--auto-layout`), a reference image is generated first, then text is extracted. Use a lower resolution for this reference to save time:

```bash
mofa slides --auto-layout --ref-image-size 1K --image-size 2K --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

### Vision Model (Editable Mode)

Override the model used for text extraction in editable mode:

```bash
mofa slides --auto-layout --vision-model gemini-2.5-flash --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

Default: `gemini-2.5-flash`

---

## Manual Text Overlays

For pixel-perfect control, specify text boxes manually via the `texts` field. The AI generates the background image (without text), and your text boxes are placed on top.

### Slide Coordinate System

```
┌────────────────────────────────────────────────────┐
│  Slide: 13.333" wide × 7.5" tall (16:9)           │
│                                                     │
│  Origin (0,0) is top-left corner                   │
│  x: 0 ──────────────────────────────────→ 13.333"  │
│  y: 0                                              │
│  │                                                  │
│  │                                                  │
│  ↓                                                  │
│  7.5"                                               │
└────────────────────────────────────────────────────┘
```

All positions are in **inches**. Full slide width = 13.333", full height = 7.5".

### Common Positions

| Position | x | y | w | Notes |
|----------|---|---|---|-------|
| Full-width title | 0.5 | 0.5 | 12.333 | Leave 0.5" margin each side |
| Centered title | 0.5 | 3.0 | 12.333 | Vertically centered |
| Left column (of 2) | 0.5 | 1.5 | 5.917 | Half-width with gap |
| Right column (of 2) | 6.917 | 1.5 | 5.917 | |
| Left column (of 3) | 0.5 | 1.5 | 3.778 | Third-width with gaps |
| Center column (of 3) | 4.778 | 1.5 | 3.778 | |
| Right column (of 3) | 9.056 | 1.5 | 3.778 | |
| Footer | 0.5 | 6.8 | 12.333 | Near bottom |

### TextOverlay Fields

| Field | JSON Key | Type | Default | Description |
|-------|----------|------|---------|-------------|
| Text | `text` | string | — | Plain text content. Use `\n` for line breaks. |
| Rich text | `runs` | TextRun[] | — | Mixed formatting (see Rich Text section) |
| Left | `x` | float | 0.5 | Inches from left edge |
| Top | `y` | float | 0.5 | Inches from top edge |
| Width | `w` | float | 6.0 | Text box width (inches) |
| Height | `h` | float | 1.0 | Text box height (inches) |
| Font | `fontFace` | string | Arial | Font family |
| Size | `fontSize` | float | 18 | Font size (points) |
| Color | `color` | string | FFFFFF | Hex RGB without # |
| Bold | `bold` | bool | false | Bold weight |
| Italic | `italic` | bool | false | Italic style |
| H-Align | `align` | string | l | `l` left, `c`/`ctr` center, `r` right, `j`/`just` justify |
| V-Align | `valign` | string | t | `t` top, `m`/`ctr` middle, `b` bottom |
| Rotation | `rotate` | float | — | Degrees (optional) |

### Available Fonts

These fonts are guaranteed to work in PowerPoint:

| Category | Font Names |
|----------|-----------|
| Sans-serif | Arial, Calibri, Tahoma, Verdana, Trebuchet MS, Segoe UI |
| Serif | Times New Roman, Cambria, Georgia, Garamond, Palatino |
| Monospace | Courier New, Consolas |
| Display | Impact |
| CJK (Chinese) | Microsoft YaHei (黑体系), SimSun (宋体系) |

Other fonts specified will be auto-mapped: Helvetica → Arial, Inter/Roboto/Lato → Calibri, etc.

### Example: Cover Slide with Manual Text

```json
{
  "prompt": "Dark gradient background with geometric patterns, no text",
  "texts": [
    {
      "text": "2026 战略规划",
      "x": 0.5, "y": 2.0, "w": 12.333, "h": 1.5,
      "fontSize": 54, "bold": true, "color": "FFFFFF", "align": "c"
    },
    {
      "text": "Building the Future of Open Source",
      "x": 0.5, "y": 3.8, "w": 12.333, "h": 0.8,
      "fontSize": 24, "color": "AAAAAA", "align": "c"
    },
    {
      "text": "March 2026 | Confidential",
      "x": 0.5, "y": 6.5, "w": 12.333, "h": 0.5,
      "fontSize": 12, "color": "666666", "align": "c"
    }
  ]
}
```

### Example: Two-Column Layout

```json
{
  "prompt": "Clean white background with subtle left accent bar, no text",
  "texts": [
    {
      "text": "Our Approach",
      "x": 0.5, "y": 0.5, "w": 12.333, "h": 0.8,
      "fontSize": 36, "bold": true, "color": "2D1B4E", "align": "l"
    },
    {
      "text": "We combine deep technical expertise with strategic consulting to help enterprises adopt open-source solutions at scale.",
      "x": 0.5, "y": 1.8, "w": 5.917, "h": 4.0,
      "fontSize": 16, "color": "333333", "align": "l"
    },
    {
      "text": "Key metrics:\n\n• 15,000+ GitHub stars\n• 200+ enterprise clients\n• 30+ countries\n• 98% client retention",
      "x": 6.917, "y": 1.8, "w": 5.917, "h": 4.0,
      "fontSize": 16, "color": "333333", "align": "l"
    }
  ]
}
```

---

## Rich Text Formatting

Use `runs` instead of `text` for mixed formatting within a single text box — bold titles with normal subtitles, multi-color text, or inline emphasis.

### TextRun Fields

| Field | JSON Key | Type | Description |
|-------|----------|------|-------------|
| Content | `text` | string | Text for this run |
| Color | `color` | string | Hex RGB override |
| Bold | `bold` | bool | Bold override |
| Italic | `italic` | bool | Italic override |
| Size | `fontSize` | float | Font size in pt |
| Font | `fontFace` | string | Font family |
| Line break | `breakLine` | bool | Insert `\n` before this run |

### Example: Title + Subtitle in One Box

```json
{
  "runs": [
    { "text": "Revenue Growth", "bold": true, "fontSize": 32, "color": "2D1B4E" },
    { "text": "  Q3 2026 Results", "fontSize": 18, "color": "888888" }
  ],
  "x": 0.5, "y": 0.5, "w": 12.333, "h": 1.0
}
```

### Example: Large Stat with Label

```json
{
  "runs": [
    { "text": "$3.2B", "bold": true, "fontSize": 72, "color": "00AA44" },
    { "text": "+47% Year-over-Year", "fontSize": 20, "color": "666666", "breakLine": true }
  ],
  "x": 1.0, "y": 2.0, "w": 5.0, "h": 3.0
}
```

### Example: Multi-Color Inline Text

```json
{
  "runs": [
    { "text": "Status: ", "fontSize": 16, "color": "333333" },
    { "text": "APPROVED", "fontSize": 16, "color": "00AA44", "bold": true },
    { "text": " | Priority: ", "fontSize": 16, "color": "333333" },
    { "text": "HIGH", "fontSize": 16, "color": "CC0000", "bold": true }
  ],
  "x": 0.5, "y": 5.0, "w": 12.333, "h": 0.5
}
```

---

## Reference Images

Pass reference images to guide Gemini's visual output. Useful for brand consistency, logo integration, or style matching.

```json
{
  "prompt": "TITLE: \"Product Overview\"\nFeature grid with company logo in upper-left",
  "images": ["/path/to/logo.png", "/path/to/brand-guide.png"]
}
```

### Use Cases

1. **Brand consistency** — Pass your logo or brand guide to keep the visual identity
2. **Style matching** — Pass an existing slide to match its look and feel
3. **Visual elements** — Pass icons, photos, or illustrations to incorporate
4. **Cross-slide consistency** — Pass the same reference to all slides

### Tips

- Reference images are sent to Gemini as context — they influence the generated image but aren't placed literally
- Use high-quality images for better results
- Multiple images can be passed per slide
- Works in both image mode and editable mode

---

## PDF-to-PPTX Conversion

Convert existing PDF pages into editable PowerPoint files.

### Step-by-Step

1. **Export PDF pages as PNGs** (use any tool):

```bash
# Using ImageMagick
magick -density 300 input.pdf -quality 95 /tmp/pdf-pages/page-%02d.png

# Using poppler (pdftoppm)
pdftoppm -png -r 300 input.pdf /tmp/pdf-pages/page
```

2. **Create input JSON:**

```json
[
  { "prompt": "page 1", "source_image": "/tmp/pdf-pages/page-01.png" },
  { "prompt": "page 2", "source_image": "/tmp/pdf-pages/page-02.png" },
  { "prompt": "page 3", "source_image": "/tmp/pdf-pages/page-03.png" }
]
```

3. **Run conversion:**

```bash
mofa slides --auto-layout --style nb-pro \
  --out editable.pptx --slide-dir /tmp/edit -i pages.json
```

### What Happens

1. Each `source_image` is used directly (no AI generation)
2. OCR/VQA extracts text positions, fonts, sizes, and colors
3. Text is removed from the image (via Dashscope inpainting)
4. Clean image becomes the slide background
5. Extracted text is overlaid as editable PowerPoint text boxes

### Per-Slide vs Global Auto-Layout

You can mix auto-layout and image-mode slides in one deck:

```json
[
  { "prompt": "page 1", "source_image": "/tmp/page-01.png", "auto_layout": true },
  { "prompt": "AI-generated summary slide", "auto_layout": false }
]
```

---

## Advanced Configuration

### Per-Slide Model Override

Different slides can use different generation models:

```json
[
  { "prompt": "Simple text slide", "gen_model": "gemini-2.5-flash" },
  { "prompt": "Complex data visualization", "gen_model": "gemini-3.1-flash-image-preview" }
]
```

### Qwen-Edit Refinement

For cleaner text removal in editable mode, add `--refine`:

```bash
mofa slides --auto-layout --refine --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

This uses Qwen-Edit (via Dashscope) for an additional text removal pass. Produces cleaner backgrounds but takes longer.

### DeepSeek-OCR-2 (Local OCR)

For higher-accuracy text extraction, run a local DeepSeek-OCR-2 endpoint and set:

```bash
export DEEPSEEK_OCR_URL="http://localhost:8080/v1/ocr"
```

This provides pixel-accurate bounding boxes (better than Gemini VQA for dense slides).

### Custom Mofa Root

If your config isn't auto-detected:

```bash
mofa slides --root /path/to/mofa-skills --style nb-pro --out deck.pptx --slide-dir /tmp/slides -i slides.json
```

---

## Input JSON Reference

### Complete Slide Object

```json
{
  "prompt": "Content description (required)",
  "style": "variant name (cover, normal, data, warm, etc.)",
  "auto_layout": true,
  "images": ["/path/to/ref1.png", "/path/to/ref2.png"],
  "source_image": "/path/to/existing-image.png",
  "gen_model": "gemini-3.1-flash-image-preview",
  "texts": [
    {
      "text": "Plain text with\nline breaks",
      "x": 0.5, "y": 0.5, "w": 12.333, "h": 1.0,
      "fontFace": "Arial",
      "fontSize": 24,
      "color": "333333",
      "bold": true,
      "italic": false,
      "align": "l",
      "valign": "t",
      "rotate": 0
    },
    {
      "runs": [
        { "text": "Bold part", "bold": true, "color": "CC0000", "fontSize": 28 },
        { "text": " normal part", "color": "333333", "fontSize": 18 },
        { "text": "New line", "breakLine": true, "italic": true }
      ],
      "x": 0.5, "y": 2.0, "w": 12.333, "h": 2.0
    }
  ]
}
```

### Field Summary

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `prompt` | string | yes | — | What AI should generate |
| `style` | string | no | `"normal"` | Variant within the style |
| `auto_layout` | bool | no | CLI flag | Per-slide editable mode override |
| `images` | string[] | no | — | Reference image paths |
| `source_image` | string | no | — | Existing image (skip generation) |
| `gen_model` | string | no | CLI/config | Generation model override |
| `texts` | TextOverlay[] | no | — | Manual text boxes |

---

## CLI Reference

```
mofa slides [OPTIONS] -o <OUT> --slide-dir <DIR>
```

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--style` | | `nb-pro` | Style template name |
| `--out` | `-o` | *required* | Output PPTX path |
| `--slide-dir` | | *required* | Directory for intermediate PNGs |
| `--input` | `-i` | stdin | Input JSON file |
| `--auto-layout` | | `false` | Enable editable text mode |
| `--concurrency` | | `5` | Parallel generation (1-20) |
| `--image-size` | | config | `1K` / `2K` / `4K` |
| `--gen-model` | | `gemini-3.1-flash-image-preview` | Image generation model |
| `--ref-image-size` | | same as image-size | Reference image resolution (editable mode) |
| `--vision-model` | | `gemini-2.5-flash` | Text extraction model (editable mode) |
| `--refine` | | `false` | Use Qwen-Edit for text removal |
| `--root` | | auto-detected | Path to mofa root directory |

---

## Configuration File

`mofa/config.json`:

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
    "slides": {
      "style": "nb-pro",
      "image_size": "2K",
      "concurrency": 5,
      "auto_layout": false
    }
  }
}
```

API keys can use `"env:VAR_NAME"` to read from environment variables, or literal values.

---

## Tips & Best Practices

### Prompt Tips

1. **Start with TITLE** — Always begin prompts with `TITLE: "..."` for consistent heading placement
2. **Be specific about layout** — "3 cards in a row" beats "show features"
3. **Include real data** — Numbers and stats make AI-generated slides more convincing
4. **Describe hierarchy** — Mention font sizes, emphasis, and visual weight
5. **Match variant to content** — Use `cover` for title slides, `data` for tables/metrics

### Performance Tips

1. **Start with `--concurrency 3`** to avoid API rate limits, increase after confirming it works
2. **Use `--image-size 1K`** for quick drafts, then `2K` or `4K` for final output
3. **Use `--ref-image-size 1K`** with `--auto-layout` — reference images don't need full resolution
4. **Batch your slides** — generating 10 slides at once is faster than 10 separate runs

### Quality Tips

1. **Choose the right style** — match style to content and audience
2. **Mix variants** — use `cover` → `normal` → `data` → `normal` → `cover` for good flow
3. **Reference images help** — passing a logo or brand guide improves consistency
4. **Editable mode** for drafts you'll iterate on, image mode for final presentations
5. **Check intermediate PNGs** in `--slide-dir` to debug AI generation quality

### Common Color Codes

| Color | Hex | Use |
|-------|-----|-----|
| White | `FFFFFF` | Text on dark backgrounds |
| Black | `000000` | Text on light backgrounds |
| Dark gray | `333333` | Body text |
| Medium gray | `666666` | Secondary text |
| Light gray | `999999` | Captions, footnotes |
| Red | `CC0000` | Warnings, highlights |
| Green | `00AA44` | Positive metrics |
| Blue | `2B74FF` | Links, accents |
| Purple | `6B4FA0` | nb-pro accent |
| Gold | `FFC640` | gobi accent |

---

## Troubleshooting

### "Gemini API key required"
Set `GEMINI_API_KEY` environment variable or add it to `mofa/config.json`.

### Editable mode produces no text boxes
Ensure `DASHSCOPE_API_KEY` is set. Without it, text extraction falls back to Gemini VQA only (less reliable on dense slides).

### Slides look different from expected
- Check you're using the right `--style` and per-slide `"style"` variant
- Try a different `--gen-model` if image quality is inconsistent
- Add reference images for better style guidance

### Text extraction misses some text
- Set `DEEPSEEK_OCR_URL` for a local DeepSeek-OCR-2 endpoint (most accurate)
- Or ensure `DASHSCOPE_API_KEY` is set for Dashscope OCR fallback
- Dense slides with many small text elements are harder to extract

### PPTX opens with wrong fonts
Fonts are normalized to PowerPoint-compatible families. If you need a specific font, use `fontFace` in manual text overlays and ensure the font is installed on the viewing machine.

### Generation is slow
- Reduce `--concurrency` if hitting API rate limits
- Use `--image-size 1K` for drafts
- Use `--ref-image-size 1K` with `--auto-layout`
