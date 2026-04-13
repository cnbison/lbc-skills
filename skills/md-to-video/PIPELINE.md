# MD-to-Video Pipeline — Agent Reference

## Overview

This pipeline converts a Markdown document into a narrated video with AI-generated slides.

## Pipeline Stages

### Stage 1: Input Analysis

Read the source Markdown and identify:
- Total page count (count `## ` headers)
- Cover page (first `## ` section)
- Content pages (middle sections with narration text)
- End page (last section)

### Stage 2: Slides Plan Generation

Create a slide plan mapping each Markdown section to a slide:
- Cover slide: title + subtitle, use `cover` variant
- Content slides: key bullet points extracted from narration
- End slide: closing message, use `cover` variant

**Slide plan format** (save as `slides-plan.md`):
```markdown
# 幻灯片内容分页（共N页）

## 第1页：封面
[Title]
[Subtitle]

## 第2页：[Section title]
[Bullet points from narration — 3-5 lines max]

## 第N页：结尾
[Closing message]
```

### Stage 3: Generate Slides

Use `mofa_slides` to generate slide images:
- Style: user-chosen (default `fengzikai` for ink-wash)
- Cover/End: use `cover` variant
- Content: use `normal` variant
- Resolution: 2K (default)
- Split into batches of 5 if >15 slides (tool timeout limit)

**Key: save slide PNGs to `slides/ordered/slide-NN.png`**

The `mofa_slides` tool generates a PPTX and PNGs in `slide_dir`. Extract PNGs to a flat ordered directory.

### Stage 4: Generate Narration Audio

For each content page (skip cover/end), use `fm_tts` to generate narration:
- Voice: user-chosen (default from fm_voice_list)
- Language: match source text (chinese/english)
- Style prompt: optional narration style

**File naming**: `audio/pageNN.mp3` (NN = 02, 03, ..., matching slide numbers)

### Stage 5: Assemble Video

For each slide, create an MP4 segment using ffmpeg:

**Cover/End segments** (no audio, static image):
```bash
ffmpeg -y -loop 1 -i "slide-NN.png" \
  -c:v libx264 -tune stillimage -pix_fmt yuv420p \
  -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=black" \
  -t [DURATION] -an "segments/seg-NN.mp4"
```

**Content segments** (slide + narration audio):
```bash
DURATION=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "audio/pageNN.mp3")
ffmpeg -y -loop 1 -i "slide-NN.png" -i "audio/pageNN.mp3" \
  -c:v libx264 -tune stillimage -c:a aac -b:a 192k \
  -pix_fmt yuv420p -shortest \
  -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=black" \
  -t "$DURATION" "segments/seg-NN.mp4"
```

### Stage 6: Concatenate All Segments

```bash
# Create concat list
for f in segments/seg-*.mp4; do echo "file '$f'" >> concat.txt; done

# Merge
ffmpeg -y -f concat -safe 0 -i concat.txt \
  -c:v libx264 -preset medium -crf 23 \
  -c:a aac -b:a 192k \
  -movflags +faststart \
  final_video.mp4
```

### Stage 7: Deliver

Use `send_file` to deliver the final MP4 to the user.

## Directory Structure

```
skill-output/md-to-video-<timestamp>/
├── script.md           # Original/source Markdown
├── slides-plan.md      # Slide content plan
├── slides/
│   ├── deck.pptx       # Generated PPTX
│   └── ordered/        # Extracted PNGs: slide-01.png ~ slide-NN.png
├── audio/              # Narration MP3s: page02.mp3 ~ page(N-1).mp3
├── segments/           # Individual MP4 segments: seg-01.mp4 ~ seg-NN.mp4
└── final_video.mp4     # Final assembled video
```

## Timing Estimates

| Stage | Time | Notes |
|-------|------|-------|
| Slides (5-15) | 1-3 min | Parallel generation, may batch |
| Audio (per page) | 30-60s each | Sequential, background TTS |
| Segments | 1-2 min | ffmpeg encoding |
| Concat | 30s | Fast merge |
| **Total** | **5-15 min** | Depends on slide/page count |

## Error Handling

- If a slide PNG is missing, skip that segment and log a warning
- If audio is missing for a content page, treat as static page (3s)
- Always verify all segments exist before concat
- Use `ls -la segments/` to confirm before final merge
- If path has special chars (`%3A`), use relative paths from the working directory
