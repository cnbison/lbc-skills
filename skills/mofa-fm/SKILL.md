---
name: mofa-fm
description: TTS and voice cloning. Triggers: voice, TTS, text to speech, 语音, 播报, read aloud.
version: 0.4.3
author: hagency
always: false
---

# MoFA FM — Text-to-Speech

## How to use

1. Call `fm_tts` directly with the full text. It runs in background and delivers the audio automatically.
2. Do NOT use spawn, shell scripts, or manual text splitting.
3. Call `fm_voice_list` before TTS to check available voices (preset + custom).

## Voices

Preset: vivian (default), serena, ryan, aiden, eric, dylan, uncle_fu, ono_anna, sohee

Custom voices are saved via `fm_voice_save` and used by name in `fm_tts`.
`fm_voice_save` accepts a short reference clip in WAV directly, or MP3/M4A/OGG/FLAC which will be converted to WAV before saving.

## Style prompt

Leave `prompt` empty for natural content-aware tone. Set it to override:
- News broadcast: `用专业新闻播音员的语气朗读`
- Storytelling: `用讲故事的语气，声音温暖`
- Emotion: `用兴奋激动的语气说话` or `Speak with excitement`
