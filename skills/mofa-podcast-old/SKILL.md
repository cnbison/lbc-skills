---
name: mofa-podcast
description: Multi-speaker podcast and dialogue generation with TTS voice cloning. Triggers: podcast, 播客, multi-speaker audio, 多人对话, 多人语音, radio show, talk show, 对话生成, 锵锵三人行, 两人对话, 三人对话, voice dialogue, 用XX的声音, 用XX和XX的风格, 声音模仿, 角色对话, 配音.
version: 0.4.1
author: hagency
always: false
requires_bin: mofa-podcast
pipeline: architecture.dot
---

# MoFA Podcast — Multi-Speaker Podcast Generator

Generate professional multi-speaker podcasts from a topic or text. The pipeline collects speaker preferences, expands content into a scripted dialogue with emotion and music cues, lets you review before generation, then produces a final MP3.

## How to use

1. Tell the agent your podcast topic (or paste source text)
2. The agent will ask you for:
   - **Speakers** (1-5): name + voice for each
   - **Genre**: drama, news, talk-show, interview, storytelling, debate, or custom
   - **Length**: target duration in minutes
3. A full script is generated in markdown with `[Speaker - voice, emotion]` format
4. **Review the script** — approve, request edits, or cancel
5. On approval, the engine generates all voices and assembles the final MP3

## Voices

Preset voices (built-in): vivian (default), serena, ryan, aiden, eric, dylan, uncle_fu, ono_anna, sohee

Custom/cloned voices: use `fm_voice_save` in mofa-fm to save cloned voices first, then reference them by name.

## Script Format

The generated script uses this format:

```markdown
# My Podcast Title

**Genre**: talk-show | **Duration**: ~10 min | **Speakers**: 3

| Character | Voice | Type |
|-----------|-------|------|
| Host | vivian | built-in |
| Guest1 | ryan | built-in |
| Expert | clone:sarah | clone |

---

[BGM: Upbeat intro music — fade-in, 5s]

[Host - vivian, cheerful] Welcome to today's show!

[Guest1 - ryan, excited] Thanks for having me!

[BGM: Soft transition — crossfade, 3s]

[Expert - clone:sarah, serious] Let me share some insights...

[PAUSE: 2s]

[Host - vivian, warm] That's fascinating. Let's dig deeper...

[BGM: Outro music — fade-out, 5s]
```

### Emotion tags

Supported emotions (mapped to TTS style prompts):
- `calm` — natural, composed tone
- `excited` — energetic, enthusiastic
- `serious` — formal, weighty
- `warm` — friendly, inviting
- `angry` — intense, forceful
- `sad` — somber, reflective
- `cheerful` — upbeat, positive
- `dramatic` — theatrical, intense
- `curious` — inquisitive, wondering
- `thoughtful` — contemplative, measured

### BGM cues

Background music placeholders — actual music files are mixed in post-production:
- `[BGM: description — fade-in, Ns]` — music fades in over N seconds
- `[BGM: description — fade-out, Ns]` — music fades out
- `[BGM: description — crossfade, Ns]` — crossfade transition

### Pause cues

- `[PAUSE: Ns]` — insert N seconds of silence (1-3s typical)

## Generation Engine

The `podcast_generate` tool:
1. Parses the approved markdown script
2. Extracts all `[Character - voice, emotion] text` lines
3. Assigns sequential segment IDs (`seg_001`, `seg_002`, ...)
4. **Generates built-in voices first**, then cloned voices (minimizes model switching)
5. Within each voice type, groups by persona (avoids reloading)
6. Saves segments as sanitized `seg_{NNN}_{voice}.wav` files inside the output `segments/` directory
7. Concatenates all segments in timeline order
8. Inserts natural pauses between speakers (~400ms) and at `[PAUSE]` cues
9. Outputs final audio via ffmpeg when available, otherwise returns a WAV fallback

## Output

- Script: `skill-output/mofa-podcast/script.md`
- Segments: `skill-output/mofa-podcast/segments/*.wav`
- Final audio: `skill-output/mofa-podcast/podcast_<timestamp>.mp3` (or `.wav` fallback if MP3 conversion is unavailable)
