---
name: mofa-slides
version: 0.4.2
description: "AI-generated visual presentations with full-bleed Gemini images. Triggers: mofa, mofa ppt, mofa deck, slides, 幻灯片, generative slides, build a mofa ppt, 用mofa做PPT, AI deck, 做个PPT, make slides."
always: true
requires_bins: mofa
requires_env: GEMINI_API_KEY
---

# mofa-slides

CLI: `mofa slides` | Styles: `mofa-slides/styles/*.toml` | Config: `mofa/config.json`

## Output Paths

**IMPORTANT**: Always use relative paths under `skill-output/` with a unique per-request subdirectory:

```
skill-output/mofa-slides-<YYYYMMDD-HHMMSS>/slides.pptx
skill-output/mofa-slides-<YYYYMMDD-HHMMSS>/slide-dir/
```

Example: `"out": "skill-output/mofa-slides-20260308-143022/deck.pptx"`, `"slide_dir": "skill-output/mofa-slides-20260308-143022/imgs"`

**Never use absolute paths like `/tmp/slides.pptx`** — use relative paths instead.

## Octos Workspace Status

When `mofa-slides` is used inside an Octos slides workspace:

- `check_background_tasks` tells you what happened in execution
- `check_workspace_contract` tells you what is true about the deliverable

Use both when the user asks whether a deck is done, failed, or ready to send.

Do not answer deck readiness from chat history alone. Treat the workspace contract as authoritative:

- execution truth:
  - generation running, verifying, delivering, completed, failed
- deliverable truth:
  - required source files present
  - PPTX artifact present
  - manifest present
  - preview slide images present

If task state says completed but workspace contract is not ready, report the deck as incomplete and name the missing artifacts or failed checks.

## Interaction Guide

Before generating, gather preferences interactively. On Telegram, use inline keyboard buttons when possible:

1. **Topic/content** — Ask what the presentation is about
2. **Style** — Recommend based on content, show options:
   - Business/corporate → `agentic-enterprise-red` or `nb-pro`
   - Academic/research → `what-is-life`
   - Creative/artsy → `fengzikai` or `lingnan`
   - Tech/startup → `nb-br` or `dark-community`
   - Product launch → `vlinka-dji`
   - Conference/summit → `gobi`
3. **Number of slides** — Suggest 5-8 for a pitch, 10-15 for a full deck
4. **Resolution** — Default 2K; suggest 4K for print or large screens
5. **API keys** — Check if GEMINI_API_KEY is configured. If not, ask the user to provide it. This is required for image generation.

Present a slide plan (titles + variants) for confirmation before generating.

**Telegram inline keyboard example** for style selection:
```json
message(content="Choose a style:", metadata={"inline_keyboard": [
  [{"text": "商务 nb-pro", "callback_data": "style:nb-pro"}, {"text": "科幻 nb-br", "callback_data": "style:nb-br"}],
  [{"text": "学术 what-is-life", "callback_data": "style:what-is-life"}, {"text": "国潮 fengzikai", "callback_data": "style:fengzikai"}]
]})
```
User's button press arrives as `[callback] style:nb-pro`.

## Four Modes

### Mode 1: Image-only (default)
Text baked into AI image. Beautiful, but not editable in PowerPoint.
- User says: "做PPT", "make slides"
- `prompt` describes everything (background + text content)
- No `texts` field, no `auto_layout`

### Mode 2: Clean background + manual text overlay
AI generates a text-free background, you specify text boxes manually with precise positioning. Like cc-ppt's approach — best for pixel-perfect control.
- User says: "可编辑PPT with my text", "editable with exact layout"
- Provide `texts` array per slide (NO `auto_layout`)
- AI prompt gets `NO_TEXT_INSTRUCTION` appended → generates clean background only
- Your `texts` are overlaid as native PowerPoint text boxes
- Supports `runs` for rich text (mixed fonts/colors/sizes in one box), `fill` for card backgrounds, `margin`, `lineSpacing`
- No VQA, no text removal — fast and predictable

### Mode 3: Auto-layout (VQA)
Fully automated editable slides. AI generates with text, VQA extracts layout, text is removed, native text boxes overlaid.
- User says: "可编辑PPT", "editable slides", "auto layout"
- Add `auto_layout: true` per slide or `--auto-layout` flag for all
- Requires `GEMINI_API_KEY`. `DASHSCOPE_API_KEY` recommended for best quality text removal.
- Pipeline: Generate → VQA extract → Remove text → Assemble

### Mode 4: PDF-to-PPTX
Convert existing slide images to editable PowerPoint. Provide `source_image` + `auto_layout: true`.
- Pipeline: Copy image → VQA extract → Remove text → Assemble (skips generation)

### Anti-leak rules
All image generation prompts automatically include anti-leak rules that prevent Gemini from rendering formatting hints (font sizes, hex colors, CSS notation) as literal text. This applies to all modes.

### Editable mode pipeline (Modes 3 & 4, 4 phases):
1. **Generate/Import**: Gemini generates full slide image with text (or use `source_image`)
2. **Extract**: VQA reads the image → extracts every text element (content, position, font size, color, weight, alignment). OCR+VQA hybrid when DeepSeek OCR is available.
3. **Remove text**: `qwen-image-edit-max` removes all text, preserving illustrations/wireframes/charts. Falls back to Gemini image editing if DASHSCOPE_API_KEY is not set.
4. **Assemble**: PPTX built with clean background image + editable text boxes placed on top

## Custom styles (inline)

You are NOT limited to the 17 built-in styles. You can write a full style prompt directly in the slide's `prompt` field. The built-in style prefix still gets prepended, so use `--style nb-pro` (minimal) or any neutral style as a base, and override everything in the prompt.

Example — user asks for "art deco" style (not a built-in):
```json
{
  "prompt": "Create a presentation slide image. 1920x1080, 16:9.\n\nDESIGN SYSTEM:\n- BACKGROUND: Deep navy (#1B1F3B) with gold geometric art deco patterns — sunburst rays, chevrons, fan shapes\n- ACCENT: Warm gold (#D4AF37) for decorative lines and borders\n- TYPOGRAPHY: Elegant serif style, cream white (#FFF8E7) text\n- DECORATIVE: Thin gold geometric borders, symmetrical patterns, 1920s luxury aesthetic\n- ILLUSTRATION: Art deco line art — geometric, angular, sophisticated\n\nLeave 60% clean space for text overlays. Decorative borders and patterns on edges only.",
  "texts": [
    {"text": "Annual Report 2025", "x": 1, "y": 2.5, "w": 11, "h": 1.2, "fontSize": 44, "bold": true, "color": "FFF8E7", "align": "ctr"},
    {"text": "Board of Directors Presentation", "x": 2, "y": 3.8, "w": 9, "h": 0.6, "fontSize": 20, "color": "D4AF37", "align": "ctr"}
  ]
}
```

The style prompt should describe: background, colors, illustration style, decorative elements, and where to leave clean space. Follow the same pattern as built-in styles.

**Quick-reference inline style templates** (copy and adapt for the `prompt` field):

| User says | Style prompt snippet |
|-----------|---------------------|
| Art Deco、复古金色 | `Deep navy (#1B1F3B), gold (#D4AF37) geometric sunburst rays, chevrons, fan shapes. Elegant serif, 1920s luxury. Thin gold borders.` |
| Bauhaus、包豪斯 | `Primary colors (red #E53935, blue #1E88E5, yellow #FDD835) on white. Bold geometric shapes — circles, rectangles, triangles. Grid-based layout, Futura/Helvetica font style. Minimal, functional.` |
| Glassmorphism、毛玻璃 | `Soft gradient (#667eea → #764ba2). Frosted glass cards with backdrop-blur, white/translucent borders. Subtle floating shapes behind glass. Modern, airy.` |
| Cyberpunk、赛博朋克 | `Dark background (#0D0D0D). Neon magenta (#FF00FF) and cyan (#00FFFF) accent lines, glitch effects, circuit patterns. Monospace font style. High contrast, futuristic.` |
| 国潮、Chinese guochao | `Deep red (#8B0000) or navy (#1A237E) base. Gold (#D4AF37) traditional patterns — clouds, waves, dragons, lotus. Mix of classical motifs with modern geometry. Bold, vibrant, cultural pride.` |
| 水墨、Chinese ink wash | `Rice paper texture (#F5F0E8). Black ink wash (#333) flowing strokes, mountains, bamboo, plum blossoms. Red seal stamp accent. Zen minimalism, calligraphic elegance.` |
| 敦煌、Dunhuang | `Warm earth tones — sand (#C9A96E), terracotta (#B7623E), turquoise (#2E8B8B), gold. Flying apsaras, cloud scrolls, flame motifs. Tang dynasty mural aesthetic, rich and ornate.` |
| 青花瓷、Blue and white porcelain | `White (#FAFAFA) background. Cobalt blue (#1A3C6D) delicate floral patterns — peonies, lotus, vine scrolls. Fine line art, elegant and timeless. Ming dynasty ceramic aesthetic.` |
| 故宫红、Forbidden City | `Imperial red (#9B1B30) with gold (#C9A96E) accents. Palace architecture elements — roof ridges, lattice windows, cloud patterns. Regal, authoritative, traditional.` |
| Gradient mesh、渐变 | `Smooth multi-color gradient mesh (purple→pink→orange). Soft organic blob shapes. No hard edges. Dreamy, modern, Apple-keynote aesthetic.` |
| Isometric、等距插画 | `Clean white/light gray background. Colorful isometric 3D illustrations — buildings, devices, people. Flat shading, consistent angle. Tech-friendly, modern.` |
| 手绘、Hand-drawn sketch | `Off-white paper (#FFF9F0). Pencil/pen sketch style illustrations — loose hand-drawn lines, crosshatching. Warm, personal, approachable. Think notebook doodles but polished.` |
| Retro 80s、复古80年代 | `Dark purple/navy gradient. Neon grid perspective, chrome text style, sunset gradients (pink→orange→purple). Synthwave aesthetic, VHS scanlines. Nostalgic, bold.` |
| 日式和风、Japanese wa | `Soft cream (#F5F0E1) with indigo (#2C3E6B) accents. Cherry blossoms, wave patterns (seigaiha), torii gates. Delicate, balanced, wabi-sabi minimalism.` |

## Built-in Styles (17)

| User says | `--style` | Variants |
|-----------|-----------|----------|
| 红色企业、华为风、商务红 | `agentic-enterprise-red` | normal, cover, data |
| 紫色企业、咨询风、McKinsey | `agentic-enterprise` | normal, warm, cover, data |
| 极简、北欧、MUJI、IKEA | `nordic-minimal` | normal, data, cover |
| 专业、商务、正式 | `nb-pro` | normal |
| 科幻、赛博朋克、Blade Runner | `nb-br` | normal |
| 暗色、社区、开源社区 | `dark-community` | normal |
| 学术、科研、论文、study notes | `what-is-life` | cover, physics_dark, biology_light, overview |
| 开源、可爱、卡通鲸鱼 | `opensource` | normal, data, cover |
| 暖色、琥珀、电影感 | `cc-research` | normal |
| 产品发布、DJI、大疆 | `vlinka-dji` | cover, hero, feature, scene, data |
| 多品牌对比、公司对比 | `multi-brand` | amazon_light, amazon_dark, google, microsoft, tesla_light, tesla_dark, nvidia_light, nvidia_dark, spacex, overview, cover |
| 简笔画、小人、greeting | `relevant` | front, greeting, scene, festive |
| 策略、咨询、薰衣草 | `tectonic` | normal, data, cover |
| 开源企业、红黑 | `openclaw-red` | normal, cover, data |
| 丰子恺、水墨、童趣、宣纸 | `fengzikai` | normal, cover, data |
| 岭南、国画、水彩、花鸟 | `lingnan` | normal, cover, data, warm |
| 会议、峰会、conference、GOBI | `gobi` | cover, normal, data, warm |
| "有哪些模板？" / "list styles" | Show all above | |
| *(not specified)* | `nb-pro` | |

Set per-slide variant via JSON `"style"` field (e.g. `"style": "cover"`). Defaults to `"normal"`.

## API Modes

| `--api` | Speed | Cost | How it works |
|---------|-------|------|--------------|
| `rt` (default) | Fast (~2-4 min for 10 slides) | Standard pricing | Parallel sync calls via rayon thread pool |
| `batch` | Slow (5-30 min) | **50% cheaper** | Gemini Batch API, async processing. Falls back to `rt` on timeout. |

Use `--api batch` for large decks (15+ slides) where cost matters more than speed.

## Timing & Timeouts

Each slide takes ~15-30 seconds to generate. Total time depends on slide count and concurrency:

| Slides | Concurrency | Estimated Time |
|--------|-------------|----------------|
| 5 | 5 | ~30-60s |
| 10 | 5 | ~1-2 min |
| 15 | 5 | ~2-3 min |
| 25 | 5 | ~4-6 min |

**Tool timeout is 600 seconds (10 min).** To avoid timeouts:

- **Keep slide count under 15** for a single call
- **Increase concurrency**: `"concurrency": 5` or higher (default: 5)
- **Use smaller images**: `"1K"` or `"2K"` instead of `"4K"`
- **Don't use `--api batch`** in octos tool calls — batch can take 5-30 min
- **`--auto-layout` adds ~10-20s per slide** for VQA extraction + Qwen-Edit text removal

If a generation times out, **cached slides are preserved** — rerun and only missing slides will be regenerated.

## Models

| Role | Default model | Flag / config key | API key |
|------|---------------|-------------------|---------|
| Image generation | `gemini-3.1-flash-image-preview` | `--gen-model` | `GEMINI_API_KEY` |
| Text extraction (VQA) | `gemini-3.1-flash-image-preview` | `--vision-model` | `GEMINI_API_KEY` |
| Text removal (inpainting) | `qwen-image-edit-max` | `edit_model` in config | `DASHSCOPE_API_KEY` |

Per-slide generation model override: `"gen_model": "model-name"` in JSON.

## Resolution

| Flag | Values | Description |
|------|--------|-------------|
| `--image-size` | `1K`, `2K`, `4K` | Image resolution. Higher = sharper but slower. |
| `--ref-image-size` | `1K`, `2K` | Lower-res for auto-layout reference image (faster generation, VQA still accurate) |
| `--concurrency` | 1-20 | Parallel slide generation (default: 5) |

## Input JSON Schema

Top-level: array of slide objects.

### Slide Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `prompt` | string | **yes** | Content description for AI (what to render on the slide) |
| `style` | string | no | Variant within the style: `"cover"`, `"normal"`, `"data"`, `"warm"`, etc. Default: `"normal"` |
| `auto_layout` | bool | no | Per-slide override for editable mode |
| `images` | string[] | no | Reference image paths — Gemini uses these for visual style guidance |
| `source_image` | string | no | Existing image path to use as-is (skip AI generation). For PDF-to-PPTX. |
| `gen_model` | string | no | Per-slide generation model override |
| `texts` | TextOverlay[] | no | Manual text overlays — full control over text positioning and styling |

### TextOverlay (manual text boxes)

When `texts` is provided, these text boxes are placed on top of the slide image. AI generates a text-free background automatically. In auto-layout mode, VQA-extracted text is used instead.

Slide canvas: **13.333" wide x 7.5" tall** (16:9 widescreen). All positions in inches.

| Field | JSON key | Type | Default | Description |
|-------|----------|------|---------|-------------|
| Text content | `text` | string | — | Plain text. Use `\n` for line breaks. |
| Rich text | `runs` | TextRun[] | — | Alternative to `text` — mixed formatting per run (see below) |
| Left | `x` | float | 0.5 | Inches from left edge |
| Top | `y` | float | 0.5 | Inches from top edge |
| Width | `w` | float | 6.0 | Text box width in inches |
| Height | `h` | float | 1.0 | Text box height in inches |
| Font | `fontFace` | string | Arial | Font family (Arial, Calibri, Times New Roman, Courier New, Microsoft YaHei, SimSun, etc.) |
| Size | `fontSize` | float | 18 | Font size in points |
| Color | `color` | string | FFFFFF | Hex RGB without # (e.g. `"333333"`, `"CC0000"`) |
| Bold | `bold` | bool | false | Bold weight |
| Italic | `italic` | bool | false | Italic style |
| H-Align | `align` | string | l | `"l"` left, `"c"` or `"ctr"` center, `"r"` right, `"j"` or `"just"` justify |
| V-Align | `valign` | string | t | `"t"` top, `"m"` or `"ctr"` middle, `"b"` bottom |
| Rotation | `rotate` | float | — | Rotation in degrees (optional) |

### TextRun (rich text within one text box)

Use `runs` instead of `text` when you need mixed formatting (e.g. bold title + normal subtitle in one box, or multi-color text).

| Field | JSON key | Type | Description |
|-------|----------|------|-------------|
| Content | `text` | string | Text for this run |
| Color | `color` | string | Hex RGB override (optional) |
| Bold | `bold` | bool | Bold override (optional) |
| Italic | `italic` | bool | Italic override (optional) |
| Size | `fontSize` | float | Font size override in pt (optional) |
| Font | `fontFace` | string | Font family override (optional) |
| Line break | `breakLine` | bool | Insert line break before this run (optional) |

## Examples

### Mode 1: Image mode (text baked in, not editable)

```json
[
  { "prompt": "Cover slide. Large title in the center: \"AI Strategy Report\". Dramatic dark gradient background with subtle tech grid pattern.", "style": "cover" },
  { "prompt": "Title at top: \"Key Findings\". Three metric cards in a row: Revenue +47%, Users 10M, NPS 72. Each card has a large bold number and small label below.", "style": "normal" }
]
```

### Mode 2: Editable slides (RECOMMENDED for quality)

This is the best mode for editable presentations. AI generates a text-free illustrated background, you specify text boxes as native PowerPoint elements. No VQA, no text removal — fast and pixel-perfect.

**How it works**: provide `texts` array per slide. The tool automatically:
1. Appends "DO NOT render any text" to the image generation prompt
2. Generates a clean illustrated background
3. Overlays your `texts` as native editable PowerPoint text boxes

**Prompt writing rules (CRITICAL for quality):**
- The `prompt` describes the BACKGROUND ILLUSTRATION only — decorations, icons, layout zones, atmosphere
- Say where to leave clean space: "Leave LEFT 55% clean for text" or "80% clean space, decorations in corners only"
- NEVER put formatting hints near content: ~~"title (24pt, bold, #C7000B)"~~ → describe in natural language or omit
- Content text belongs ONLY in the `texts` array, not in the `prompt`

**Slide canvas**: 13.333" wide × 7.5" tall (16:9). All positions in inches from top-left.

**Layout reference** (common positions):
- Title: `x: 0.5, y: 0.3, w: 12, h: 1.0, fontSize: 36, bold: true`
- Subtitle: `x: 0.5, y: 1.2, w: 10, h: 0.6, fontSize: 20`
- Body text area: `x: 0.5, y: 2.0, w: 12, h: 4.5, fontSize: 16`
- 2-column cards: left `x: 0.5, w: 5.8`, right `x: 6.8, w: 5.8`
- 3-column cards: `x: 0.4, w: 3.8` | `x: 4.5, w: 3.8` | `x: 8.6, w: 3.8`
- Footer: `x: 0.5, y: 6.5, w: 12, h: 0.5, fontSize: 12`

**Example — 5-slide business deck:**

```json
[
  {
    "style": "cover",
    "prompt": "Dark gradient stage with dramatic purple-blue lighting. Subtle tech grid pattern. Main illustration cluster on RIGHT 45%. Leave LEFT 55% clean.",
    "texts": [
      { "text": "Q4 Strategy Review", "x": 0.6, "y": 2.0, "w": 6.5, "h": 1.2, "fontSize": 42, "bold": true, "color": "FFFFFF", "fontFace": "Arial" },
      { "text": "Product & Engineering", "x": 0.6, "y": 3.3, "w": 6, "h": 0.7, "fontSize": 22, "color": "90CAF9" },
      { "text": "December 2025", "x": 0.6, "y": 4.2, "w": 4, "h": 0.5, "fontSize": 16, "color": "888888" }
    ]
  },
  {
    "prompt": "Clean light background. Small decorative wireframe accents ONLY in top-right and bottom-left corners (at most 15% of slide). Rest is COMPLETELY CLEAN — no shapes, no icons, no placeholder elements.",
    "texts": [
      { "text": "Executive Summary", "x": 0.5, "y": 0.3, "w": 12, "h": 1.0, "fontSize": 36, "bold": true, "color": "2D1B4E" },
      {
        "runs": [
          { "text": "Revenue exceeded targets by 12%", "fontSize": 18, "bold": true, "color": "2E7D32", "breakLine": true },
          { "text": "", "breakLine": true },
          { "text": "Key highlights:", "fontSize": 16, "bold": true, "color": "333333", "breakLine": true },
          { "text": "• Enterprise ARR reached $42M (+31% YoY)", "fontSize": 15, "color": "444444", "breakLine": true },
          { "text": "• Customer count grew to 380 (+28%)", "fontSize": 15, "color": "444444", "breakLine": true },
          { "text": "• Net retention rate: 127%", "fontSize": 15, "color": "444444" }
        ],
        "x": 0.5, "y": 1.6, "w": 12, "h": 4.5, "fontFace": "Calibri", "lineSpacing": 28
      }
    ]
  },
  {
    "prompt": "Three soft-colored rounded card zones arranged horizontally. Left card area has pale blue tint, center has pale green, right has pale orange. Subtle wireframe icons inside each card zone (graph, users, chart). Clean space above for title. No text anywhere.",
    "texts": [
      { "text": "Key Metrics", "x": 0.5, "y": 0.3, "w": 12, "h": 0.9, "fontSize": 36, "bold": true, "color": "2D1B4E" },
      {
        "runs": [
          { "text": "$42M", "fontSize": 36, "bold": true, "color": "1565C0", "breakLine": true },
          { "text": "Annual Recurring Revenue", "fontSize": 14, "color": "666666" }
        ],
        "x": 0.4, "y": 1.8, "w": 3.8, "h": 2.0, "fill": { "color": "EBF5FB" }, "align": "ctr", "valign": "middle", "margin": [15, 15, 15, 15]
      },
      {
        "runs": [
          { "text": "380", "fontSize": 36, "bold": true, "color": "2E7D32", "breakLine": true },
          { "text": "Enterprise Customers", "fontSize": 14, "color": "666666" }
        ],
        "x": 4.5, "y": 1.8, "w": 3.8, "h": 2.0, "fill": { "color": "E8F5E9" }, "align": "ctr", "valign": "middle", "margin": [15, 15, 15, 15]
      },
      {
        "runs": [
          { "text": "127%", "fontSize": 36, "bold": true, "color": "E65100", "breakLine": true },
          { "text": "Net Revenue Retention", "fontSize": 14, "color": "666666" }
        ],
        "x": 8.6, "y": 1.8, "w": 3.8, "h": 2.0, "fill": { "color": "FFF3E0" }, "align": "ctr", "valign": "middle", "margin": [15, 15, 15, 15]
      }
    ]
  },
  {
    "prompt": "Clean minimal background. Faint horizontal divider line across upper third. Tiny decorative dots in bottom-right corner. 85% clean space.",
    "texts": [
      { "text": "Roadmap — Q1 2026", "x": 0.5, "y": 0.3, "w": 12, "h": 0.9, "fontSize": 36, "bold": true, "color": "2D1B4E" },
      { "text": "Platform", "x": 0.5, "y": 1.8, "w": 3.0, "h": 0.6, "fontSize": 20, "bold": true, "color": "1565C0", "fill": { "color": "E3F2FD" }, "align": "ctr", "valign": "middle" },
      { "text": "API v3 launch, SDK for Python/Go/Rust", "x": 3.8, "y": 1.8, "w": 8.5, "h": 0.6, "fontSize": 16, "color": "444444", "valign": "middle" },
      { "text": "Growth", "x": 0.5, "y": 2.7, "w": 3.0, "h": 0.6, "fontSize": 20, "bold": true, "color": "2E7D32", "fill": { "color": "E8F5E9" }, "align": "ctr", "valign": "middle" },
      { "text": "APAC expansion, 3 new enterprise logos", "x": 3.8, "y": 2.7, "w": 8.5, "h": 0.6, "fontSize": 16, "color": "444444", "valign": "middle" },
      { "text": "Team", "x": 0.5, "y": 3.6, "w": 3.0, "h": 0.6, "fontSize": 20, "bold": true, "color": "E65100", "fill": { "color": "FFF3E0" }, "align": "ctr", "valign": "middle" },
      { "text": "Hire 8 engineers, open London office", "x": 3.8, "y": 3.6, "w": 8.5, "h": 0.6, "fontSize": 16, "color": "444444", "valign": "middle" }
    ]
  },
  {
    "style": "cover",
    "prompt": "Warm gradient background, celebratory mood. Subtle confetti-like particles or light sparkles. Clean center area for closing text.",
    "texts": [
      { "text": "Thank You", "x": 1.5, "y": 2.5, "w": 10, "h": 1.5, "fontSize": 48, "bold": true, "color": "FFFFFF", "align": "ctr" },
      { "text": "Questions? team@company.com", "x": 2.5, "y": 4.2, "w": 8, "h": 0.7, "fontSize": 20, "color": "CCCCCC", "align": "ctr" }
    ]
  }
]
```

### Mode 3: Auto-layout (VQA, fully automated)

Same JSON as Mode 1 — just add `--auto-layout` flag. The tool generates the image WITH text, uses Gemini VQA to extract text positions, then removes text from the image, and overlays editable text boxes automatically. No manual `texts` needed.

```json
[
  { "prompt": "Cover slide with large centered title: \"AI Strategy Report\". Dramatic background.", "style": "cover" },
  { "prompt": "Title: \"Key Findings\". Three metric cards in a row showing Revenue, Users, NPS.", "style": "normal" }
]
```

### Mode 4: PDF-to-PPTX conversion

Provide existing page images as `source_image` + `auto_layout: true`. Skips AI generation, runs VQA + text removal on existing images.

```json
[
  { "prompt": "page 1", "source_image": "skill-output/pdf-pages/page-01.png", "auto_layout": true },
  { "prompt": "page 2", "source_image": "skill-output/pdf-pages/page-02.png", "auto_layout": true }
]
```

### Reference images for visual consistency

```json
[
  {
    "prompt": "TITLE: \"Product Overview\"\nFeature grid with icons",
    "images": ["/path/to/brand-guide.png", "/path/to/example-slide.png"]
  }
]
```

## CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--style` | `nb-pro` | Style name (see Styles table) |
| `-o` / `--out` | *required* | Output PPTX file path |
| `--slide-dir` | *required* | Directory for intermediate PNGs |
| `-i` / `--input` | stdin | Input JSON file path |
| `--auto-layout` | false | Enable editable mode (VQA + qwen-image-edit) for ALL slides |
| `--concurrency` | 5 | Parallel generation (1-20) |
| `--image-size` | config | `"1K"` / `"2K"` / `"4K"` |
| `--gen-model` | gemini-3.1-flash-image-preview | Image generation model |
| `--ref-image-size` | same as image-size | Lower-res for auto-layout reference (faster) |
| `--vision-model` | gemini-3.1-flash-image-preview | VQA model for text extraction in auto-layout |
| `--api` | `rt` | API mode: `rt` (realtime, fast parallel) or `batch` (50% cheaper, async 5-30 min) |
| `--root` | auto-detected | Path to mofa root directory |

## Config

`mofa/config.json`:

```json
{
  "api_keys": {
    "gemini": "env:GEMINI_API_KEY",
    "dashscope": "env:DASHSCOPE_API_KEY"
  },
  "gen_model": "gemini-3.1-flash-image-preview",
  "vision_model": "gemini-3.1-flash-image-preview",
  "edit_model": "qwen-image-edit-max",
  "defaults": {
    "slides": { "style": "nb-pro", "image_size": "2K", "concurrency": 5 }
  }
}
```

- `GEMINI_API_KEY` — required for all modes (image generation + VQA)
- `DASHSCOPE_API_KEY` — required for `--auto-layout` (qwen-image-edit text removal)

## Editing Existing PPTX Files

Beyond AI generation, mofa-slides also includes tools for editing existing presentations.

### Text Extraction
```bash
# Convert PPTX to images for analysis
soffice --headless --convert-to pdf presentation.pptx
pdftoppm -png -r 150 presentation.pdf slide

# Or extract text via pandoc
pandoc presentation.pptx -o content.md
```

### Unpack/Edit/Repack OOXML
```bash
# Unpack PPTX to raw XML
python ooxml/scripts/unpack.py presentation.pptx unpacked/

# Edit XML files in unpacked/ppt/slides/
# Then repack
python ooxml/scripts/pack.py unpacked/ edited.pptx

# Validate
python ooxml/scripts/validate.py edited.pptx
```

### Utility Scripts (in `pptx-scripts/`)

| Script | Usage | Purpose |
|--------|-------|---------|
| `html2pptx.js` | `node pptx-scripts/html2pptx.js input.html output.pptx` | HTML → PPTX conversion |
| `inventory.py` | `python pptx-scripts/inventory.py presentation.pptx` | List all slides with content summary |
| `rearrange.py` | `python pptx-scripts/rearrange.py input.pptx output.pptx "3,1,2,5,4"` | Reorder slides |
| `replace.py` | `python pptx-scripts/replace.py input.pptx output.pptx --find "old" --replace "new"` | Find & replace text across all slides |
| `thumbnail.py` | `python pptx-scripts/thumbnail.py presentation.pptx thumbs/` | Generate slide thumbnails |

### When to use which

| Task | Use |
|------|-----|
| Create new deck from scratch | `mofa slides` (Mode 1 or 2) |
| Create editable deck with AI backgrounds | `mofa slides` with `texts` (Mode 2) |
| Convert PDF to editable PPTX | `mofa slides` with `source_image` (Mode 4) |
| Edit text in existing PPTX | `ooxml/scripts/unpack.py` → edit XML → `pack.py` |
| Replace text across deck | `pptx-scripts/replace.py` |
| Reorder slides | `pptx-scripts/rearrange.py` |
| Extract content for analysis | `pandoc` or `pptx-scripts/inventory.py` |
