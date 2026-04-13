# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This is an **Octos skills repository** containing AI-native skills for Claude Code. Skills are declarative prompt engineering artifacts that define specialized AI capabilities. The repository is Chinese-first, with skills designed for Chinese-speaking users.

## Skill Structure

All skills follow a standardized format:

```yaml
---
name: skill-name
description: >-
  Detailed description including trigger words/phrases.
  Triggers: word1, word2, /command.
version: x.y.z  # optional
author: name    # optional
always: false   # optional
---

## Usage

<example>
User: trigger phrase
Assistant: [Expected behavior]
</example>

## Instructions

Step-by-step execution instructions...
```

### Key Conventions

1. **YAML Frontmatter**: Required. The `description` field must include trigger words that activate the skill
2. **Usage Examples**: Include `<example>` blocks showing user input and expected assistant behavior
3. **Execution Instructions**: Numbered steps that Claude follows when executing the skill
4. **File Output**: Skills that generate files use timestamped naming: `{timestamp}--{topic}__{type}.{ext}`

## Skill Categories

### Content Generation Skills

- **`mofa-concept`**: 8-dimensional concept deconstruction (history, dialectics, phenomenology, linguistics, formalization, existentialism, aesthetics, meta-philosophy). Outputs markdown reports and podcast scripts.

- **`ljg-roundtable`**: Structured multi-perspective debate with historical figures. Uses action tags (陈述/质疑/补充/反驳/修正/综合) and ASCII framework diagrams. Outputs org-mode files.

### Pipeline Skills

- **`md-to-video`**: Markdown → AI slides → TTS → ffmpeg → MP4. Requires external tools (`mofa-slides`, `mofa-fm`, `ffmpeg`, `GEMINI_API_KEY`).

## External Dependencies

Some skills require external tools (not managed by package managers):

| Tool | Purpose | Skills Using It |
|------|---------|-----------------|
| `mofa-slides` | AI slide generation | md-to-video |
| `mofa-fm` | TTS and voice cloning | md-to-video |
| `ffmpeg` | Video/audio processing | md-to-video |
| `soffice` | PPTX conversion (optional) | md-to-video |
| `GEMINI_API_KEY` | Image generation API | md-to-video |

## Output Locations

Skills write generated content to:

- `~/Documents/notes/` — Markdown and org-mode reports
- `~/Documents/scripts/` — Podcast scripts
- `./skill-output/<skill-name>-<timestamp>/` — Pipeline artifacts (slides, audio, video segments)

## Creating New Skills

1. Create a new directory under `skills/` or a `.md` file in the root
2. Follow the YAML frontmatter + markdown body format
3. Include clear trigger words in the description
4. Document execution steps with file I/O commands where applicable
5. Use timestamped filenames for generated content

## Language Conventions

- **Primary language**: Chinese (Simplified)
- **Trigger words**: Include both Chinese and English variants where applicable
- **File output**: Use Chinese titles with English type suffixes (e.g., `概念解剖-道__concept.md`)

## Version Control Workflow

**所有代码修改完成后必须推送到远程仓库：**

1. 检查变更：`git status`
2. 添加文件：`git add <files>`
3. 提交更改：`git commit -m "描述信息"`
4. 推送至远程：`git push origin main`

**禁止**在本地保留未推送的提交。每次任务完成时主动执行 push。
