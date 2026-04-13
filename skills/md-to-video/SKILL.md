---
name: md-to-video
description: Convert a Markdown script into a narrated video. Pipeline: Markdown → AI slides → TTS narration → ffmpeg segments → final MP4. Triggers: md to video, markdown to video, markdown视频, 脚本做视频, 文稿做视频, md视频, PPT视频, 演示视频, narrated slides, narrated video, 幻灯片视频, 朗读PPT, 旁白视频, 旁白PPT, 做个讲解视频, 讲解视频.
version: 0.1.0
author: octos
always: false
---

# MD-to-Video — Markdown Script to Narrated Video

Convert a Markdown script into a fully narrated video with AI-generated slides, TTS voice-over, and ffmpeg assembly.

## Pipeline Overview

```
Markdown Script
    │
    ├─→ [1] Parse & Plan — Extract pages, split into cover/content/end
    │
    ├─→ [2] Generate Slides — mofa-slides AI image generation (batch, up to 15 per call)
    │
    ├─→ [3] Generate Audio — fm_tts for each content page narration
    │
    ├─→ [4] Convert Slides to PNG — Extract from PPTX to ordered PNGs
    │
    ├─→ [5] Assemble Segments — ffmpeg: slide image + audio → per-page MP4 segments
    │
    └─→ [6] Concatenate — ffmpeg concat → final MP4
```

## How to Use

1. User provides a **Markdown script** with page-by-page content (one `## Page N` or `## 第N页` heading per slide)
2. The agent gathers preferences:
   - **Visual style**: mofa-slides style (default: `fengzikai` for Chinese content, `nb-pro` for English)
   - **Voice**: TTS voice for narration (default: `vivian` for Chinese, `ryan` for English)
   - **Resolution**: `2K` (default) or `4K`
   - **Cover/end duration**: static seconds (default: 5s cover, 3s end)
3. The agent generates a **slides plan** mapping each markdown page to a slide prompt + variant
4. Execute the pipeline (see steps below)
5. Deliver the final MP4 via `send_file`

## Input Format

The Markdown script should follow this structure:

```markdown
## Page 1: 封面
概念解剖——道
中国哲学核心概念深度解读

## Page 2: 开场
朋友们，今天咱们来聊聊...

## Page 3: 定锚——道的底子
道是宇宙的根儿...

## Page 11: 结尾
好了，关于"道"...
```

Rules:
- Each `## Page N` or `## 第N页` becomes one slide
- Page 1 is always **cover** (no audio, static image)
- Last 1-2 pages can be **end cards** (no audio, static image)
- Pages between cover and end are **content pages** with narration text
- Content pages may also contain slide visual content (bullet points, titles) separate from narration — the agent should distinguish between "what's shown on slide" vs "what's read aloud"

## Step-by-Step Execution

### Step 1: Parse & Plan

Read the markdown file. For each page, extract:
- **Slide content**: Short text for the slide image (title + key points)
- **Narration text**: Full text to be read aloud by TTS

Create a plan table:

| Slide | Type | Slide Prompt (简短) | Narration (旁白全文) |
|-------|------|---------------------|---------------------|
| 01 | cover | "概念解剖——道" | (none) |
| 02 | content | "开场白" | 朋友们，今天咱们来聊聊... |
| ... | ... | ... | ... |
| 14 | end | "感谢收听" | (none) |

Show the plan to the user for confirmation before proceeding.

### Step 2: Generate Slides (mofa-slides)

Use `mofa_slides` to generate AI slides.

**CRITICAL constraints:**
- Maximum **15 slides per call**. If the script has more than 15 pages, split into multiple calls (part1, part2, ...)
- Use image-only mode (Mode 1): prompt describes everything including text
- Set appropriate `style` variant: `cover` for first slide, `normal` for content, `cover` for end slides
- Output to `skill-output/md-to-video-<timestamp>/slides/`

**Slide prompt writing:**
- Each prompt should describe: background atmosphere, layout, title text, bullet points, any decorative elements
- Match the visual style (e.g., ink-wash for `fengzikai`, corporate for `nb-pro`)
- Keep slide text SHORT — key titles and bullet points only, not full narration

Example slide JSON for a Chinese philosophy topic with `fengzikai` style:

```json
[
  { "prompt": "水墨风格封面幻灯片。中央大标题'概念解剖——道'，副标题'中国哲学核心概念深度解读'。淡雅宣纸纹理背景，水墨山水意境，留白充足。", "style": "cover" },
  { "prompt": "水墨风格内容幻灯片。顶部标题'定锚——道的底子'。四条要点：别把道当成神或造物主、别觉得能用话把道说透、别把道归成玄学、别把道当成死规矩。水墨笔触装饰，留白设计。", "style": "normal" }
]
```

### Step 3: Generate Audio (fm_tts)

For each **content page** (not cover/end), call `fm_tts` with the narration text.

**Important:**
- Generate audio sequentially or in small batches (3-5 at a time) to avoid overwhelming the TTS engine
- Save each audio file as `page02.mp3`, `page03.mp3`, etc. (matching slide number)
- Use `output_path` to specify the save location: `skill-output/md-to-video-<timestamp>/audio/`
- Set appropriate `language` (`chinese` or `english`)
- Use `prompt` for style guidance if needed (e.g., `用讲故事的语气，声音温暖`)

### Step 4: Convert Slides to PNG

Extract slide images from the generated PPTX.

**Method 1 (preferred):** Use `mofa pptx-unpack` to unpack the PPTX, then use `soffice` to convert:
```bash
cd skill-output/md-to-video-<timestamp>/slides/
# If soffice is available:
soffice --headless --convert-to pdf deck.pptx
# Then convert PDF to PNG:
# (use Python or other tools)
```

**Method 2:** If the slides tool outputs slide PNGs in a directory (check `slide_dir`), use those directly.

**Method 3 (fallback):** Use Python with python-pptx to extract images, or use the `mofa pptx-unpack` approach.

**Final result:** All slides as ordered PNGs:
```
skill-output/md-to-video-<timestamp>/slides/ordered/
├── slide-01.png
├── slide-02.png
├── ...
└── slide-14.png
```

### Step 5: Assemble Segments (ffmpeg)

For each slide, create a video segment using ffmpeg:

**Cover/end slides** (no audio):
```bash
ffmpeg -y -loop 1 -i slide-01.png \
  -c:v libx264 -tune stillimage -pix_fmt yuv420p \
  -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=black" \
  -t 5 -an segments/seg-01.mp4
```

**Content slides** (with audio):
```bash
# First get audio duration
duration=$(ffprobe -v error -show_entries format=duration -of csv=p=0 page02.mp3)

ffmpeg -y -loop 1 -i slide-02.png -i page02.mp3 \
  -c:v libx264 -tune stillimage -c:a aac -b:a 192k \
  -pix_fmt yuv420p -shortest \
  -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=black" \
  -t "$duration" segments/seg-02.mp4
```

**Rules:**
- Cover segments: default 5 seconds, no audio (`-an`)
- End segments: default 3 seconds, no audio (`-an`)
- Content segments: duration = audio file duration, with audio
- All segments: 1920×1080, h264, yuv420p
- Scale/pad ensures correct aspect ratio with black letterboxing if needed
- Save all segments to `skill-output/md-to-video-<timestamp>/segments/`
- Verify each segment file exists before proceeding to concat

### Step 6: Concatenate (ffmpeg)

Create a concat file and merge all segments:

```bash
# Create concat list
for f in segments/seg-*.mp4; do
  echo "file '$PWD/$f'" >> concat.txt
done

# Merge
ffmpeg -y -f concat -safe 0 -i concat.txt \
  -c:v libx264 -preset medium -crf 23 \
  -c:a aac -b:a 192k \
  -movflags +faststart \
  final_video.mp4
```

**Output:** `skill-output/md-to-video-<timestamp>/final_video.mp4`

### Step 7: Deliver

- Report stats: duration, file size, resolution
- Use `send_file` to deliver the final MP4
- Clean up temporary files (concat.txt, intermediate files if desired)

## Directory Structure

```
skill-output/md-to-video-<timestamp>/
├── script.md              # Original or parsed markdown script
├── slides-plan.md         # Slide generation plan
├── slides/                # Generated slide assets
│   ├── deck.pptx          # PPTX output from mofa-slides
│   └── ordered/           # Extracted PNG slides
│       ├── slide-01.png
│       ├── slide-02.png
│       └── ...
├── audio/                 # TTS narration files
│   ├── page02.mp3
│   ├── page03.mp3
│   └── ...
├── segments/              # Per-slide video segments
│   ├── seg-01.mp4
│   ├── seg-02.mp4
│   └── ...
└── final_video.mp4        # Final assembled video
```

## Configuration Defaults

| Setting | Default | Options |
|---------|---------|---------|
| Visual style | `fengzikai` (Chinese) / `nb-pro` (English) | Any mofa-slides style |
| Voice | `vivian` (Chinese) / `ryan` (English) | Any preset or custom voice |
| Resolution | `2K` | `1K`, `2K`, `4K` |
| Cover duration | 5 seconds | Any integer |
| End duration | 3 seconds | Any integer |
| Video resolution | 1920×1080 | Fixed |
| Video codec | libx264, crf 23 | Configurable |
| Audio codec | AAC 192kbps | Configurable |

## Error Handling

- **Slide generation timeout**: If >15 slides, split into multiple mofa-slides calls
- **TTS failures**: Retry once, then skip and use extended static duration for that slide
- **Missing audio file**: Generate a silent segment with extended duration (use slide text length to estimate)
- **ffmpeg segment failure**: Log the error, skip the segment, and warn the user
- **Path encoding issues**: Always use the REAL filesystem path (not URL-encoded paths). Use `pwd` or `realpath` to resolve. Avoid paths with `%3A` or other URL-encoded characters.

## Dependencies

- **mofa-slides** skill (for AI slide generation)
- **mofa-fm** skill (for TTS voice)
- **ffmpeg** (for video assembly)
- **soffice** (optional, for PPTX → PNG conversion)
- **GEMINI_API_KEY** (required for slide image generation)
