---
name: ace-music
description: >-
  通过 ACE-Step 1.5 免费 API 生成 AI 音乐 / 歌曲 / 配乐 / 翻唱。
  Generate AI music using ACE Music's free hosted API (ACE-Step 1.5).
  当用户想做歌、写曲、配乐、生成器乐、做 lo-fi / 流行 / 电子音乐时激活本法。
  支持自定义歌词、风格 prompt、cover 翻唱、repaint 局部重绘。

  Triggers: ace-music, generate music, create song, compose, make a song,
  作曲, 配乐, 生成音乐, 写首歌, 帮我做首歌, 给这段歌词配曲, 生成器乐,
  AI 音乐, lo-fi, suno alternative, music generation
version: 1.0.0
author: fspecii
requires_bins: curl, python3
requires_env: ACE_MUSIC_API_KEY
---

# ACE Music - AI Music Generation

通过 ACE Music 的免费托管 API 生成音乐（底层模型 ACE-Step 1.5）。

> Source: synced from [openclaw/skills@c0a5107](https://github.com/openclaw/skills/commit/c0a510771e56fb26aa12cecc444ad3d6f5b963a1) by @fspecii.

## Usage

<example>
User: 帮我做一首关于夏天的流行歌
Assistant: 好的！我用 ace-music 给你生成。先确认几个选项：
1. 时长（默认 AI 决定，建议 30–120 秒）
2. 语言（en/zh/ja…，默认让 AI 自动检测）
3. 是否需要歌词，还是 AI 自动写？
4. 风格细化？（如：复古迪斯科、Bossa Nova、电子流行）

[确认后调用：./skills/ace-music/scripts/generate.sh "upbeat summer pop ..." --duration 60 --output output/ace-music/summer.mp3]
</example>

<example>
User: 给这段歌词配个民谣女声
Assistant: 收到。请贴歌词，我生成一首民谣女声的版本。
[调用：./skills/ace-music/scripts/generate.sh "gentle acoustic folk, female vocal" --lyrics "..." --duration 90 --output output/ace-music/folk.mp3]
</example>

<example>
User: 做个 90 秒无歌词的 lo-fi 背景音
Assistant: [调用：./skills/ace-music/scripts/generate.sh "lo-fi hip hop beats, chill, rainy day" --instrumental --duration 90 --output output/ace-music/lofi.mp3]
</example>

## Instructions

1. **检查 API Key**：确认环境变量 `ACE_MUSIC_API_KEY` 已设置；未设置时引导用户访问 <https://acemusic.ai/playground/api-key> 获取，然后 `export ACE_MUSIC_API_KEY=<key>`。
2. **收集参数**：与用户确认 prompt（风格描述）、是否需要 lyrics、duration、language、instrumental。
3. **生成命令**：调用 `./skills/ace-music/scripts/generate.sh`，输出到 `output/ace-music/{date}--{topic}.mp3`。
4. **多版本**：当 `--batch >1` 时把所有变体路径都报告给用户。
5. **失败兜底**：若 API 返回非 2xx，stderr 已打印响应体；告知用户 ACE Music 平台状态可能异常，可先用 `./skills/ace-music/scripts/health.sh` 检查连通性。

## Quick Generation

```bash
# 简单 prompt（AI 决定一切）
./skills/ace-music/scripts/generate.sh "upbeat pop song about summer" --duration 30 --output output/ace-music/summer.mp3

# 带歌词
./skills/ace-music/scripts/generate.sh "gentle acoustic ballad, female vocal" \
  --lyrics "[Verse 1]\nSunlight through the window\n\n[Chorus]\nWe are the dreamers" \
  --duration 60 --output output/ace-music/ballad.mp3

# 纯器乐
./skills/ace-music/scripts/generate.sh "lo-fi hip hop beats, chill, rainy day" --instrumental --duration 120 --output output/ace-music/lofi.mp3

# 自然语言（AI 包办全部）
./skills/ace-music/scripts/generate.sh "write me a jazz song about coffee" --sample-mode --output output/ace-music/jazz.mp3

# 指定参数
./skills/ace-music/scripts/generate.sh "rock anthem" --bpm 140 --key "E minor" --seed 42 --output output/ace-music/rock.mp3

# 多版本
./skills/ace-music/scripts/generate.sh "electronic dance track" --batch 3 --output output/ace-music/edm.mp3
```

脚本会把文件路径输出到 stdout。生成完成后把文件路径告诉用户。

## Advanced Usage (curl/direct API)

翻唱（cover）、局部重绘（repaint）或音频输入 —— 详见 `references/api-docs.md` 的完整 API 规格。

主要任务类型：
- `text2music`（默认）— 从文本/歌词生成
- `cover` — 翻唱已有歌曲（需音频输入）
- `repaint` — 修改已有音频的某一段

## Parameters Guide

| 需求 | 用法 |
|------|-----|
| 指定风格 | 在 prompt 中描述："jazz, saxophone solo, smoky bar" |
| 自定义歌词 | `--lyrics "[Verse]...[Chorus]..."` |
| AI 包办全部 | `--sample-mode` |
| 无人声 | `--instrumental` |
| 更长歌曲 | `--duration 120`（秒） |
| 指定节拍 | `--bpm 120` |
| 指定调性 | `--key "C major"` |
| 多版本输出 | `--batch 3` |
| 可复现 | `--seed 42` |
| 非英文人声 | `--language ja`（zh, en, ja, ko 等） |

## Notes

- API **永久免费**（经 ACE Music 团队确认）
- Base URL: `https://api.acemusic.ai`
- 音频以 base64 MP3 返回，脚本自动解码
- Duration：若省略，AI 根据内容自动决定
- 最佳效果建议用 tagged mode（prompt 和歌词分开传入）
