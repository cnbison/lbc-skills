# News Summary Skill — Edge TTS Voice Reference

This document provides the complete Edge TTS voice catalog used by the `news-summary` skill for generating voice summaries. Voices are selected automatically based on the language of the user's request.

---

## Multilingual Voice Reference

When a user requests a voice summary, detect their language and select the matching voice from the table below.

| User Language | Locale | Edge TTS Voice | Gender | Style |
|---------------|--------|----------------|--------|-------|
| 中文 (Chinese) | `zh-CN` | `zh-CN-XiaoxiaoNeural` | Female | Warm |
| English (US) | `en-US` | `en-US-AriaNeural` | Female | Neutral |
| English (UK) | `en-GB` | `en-GB-SoniaNeural` | Female | Neutral |
| 日本語 (Japanese) | `ja-JP` | `ja-JP-NanamiNeural` | Female | Friendly |
| 한국어 (Korean) | `ko-KR` | `ko-KR-SunHiNeural` | Female | Bright |
| Deutsch (German) | `de-DE` | `de-DE-KatjaNeural` | Female | Neutral |
| Français (French) | `fr-FR` | `fr-FR-DeniseNeural` | Female | Neutral |
| Español (Spanish) | `es-ES` | `es-ES-ElviraNeural` | Female | Neutral |

> **Rule**: The text summary language must match the user's request language. Do not generate an English voice summary for a Chinese request.

---

## Chinese Voices (Detailed)

All `zh-*` voices available in Edge TTS. Use this list for fine-tuning voice selection in Chinese.

### Mandarin — Mainland China (`zh-CN`)

| Voice | Gender | Style Tags | Recommended Use |
|-------|--------|-----------|-----------------|
| `zh-CN-XiaoxiaoNeural` | Female | Warm | News, novel, general briefing (**default**) |
| `zh-CN-XiaoyiNeural` | Female | Lively | Cartoon, novel, casual content |
| `zh-CN-YunjianNeural` | Male | Passion | Sports, novel, energetic content |
| `zh-CN-YunxiNeural` | Male | Lively, sunshine | Novel, storytelling |
| `zh-CN-YunxiaNeural` | Male | Cute | Cartoon, novel, lighthearted content |
| `zh-CN-YunyangNeural` | Male | Professional, reliable | **News, formal reports** |
| `zh-CN-liaoning-XiaobeiNeural` | Female | Humorous | Liaoning dialect content |
| `zh-CN-shaanxi-XiaoniNeural` | Female | Bright | Shaanxi dialect content |

### Cantonese — Hong Kong (`zh-HK`)

| Voice | Gender | Style Tags | Recommended Use |
|-------|--------|-----------|-----------------|
| `zh-HK-HiuGaaiNeural` | Female | Friendly, positive | General |
| `zh-HK-HiuMaanNeural` | Female | Friendly, positive | General |
| `zh-HK-WanLungNeural` | Male | Friendly, positive | General |

### Mandarin — Taiwan (`zh-TW`)

| Voice | Gender | Style Tags | Recommended Use |
|-------|--------|-----------|-----------------|
| `zh-TW-HsiaoChenNeural` | Female | Friendly, positive | General |
| `zh-TW-HsiaoYuNeural` | Female | Friendly, positive | General |
| `zh-TW-YunJheNeural` | Male | Friendly, positive | General |

---

## Usage Examples

### CLI

```bash
# Chinese news summary
edge-tts --voice "zh-CN-YunyangNeural" \
  --text "今日科技新闻摘要..." \
  --write-media news/news_summary_$(date +%Y%m%d_%H%M).mp3

# English news summary
edge-tts --voice "en-US-AriaNeural" \
  --text "Today's tech news briefing..." \
  --write-media news/news_summary_$(date +%Y%m%d_%H%M).mp3
```

### Python

```python
import edge_tts
import asyncio

VOICE_MAP = {
    "zh": "zh-CN-XiaoxiaoNeural",
    "en": "en-US-AriaNeural",
    "ja": "ja-JP-NanamiNeural",
    "ko": "ko-KR-SunHiNeural",
    "de": "de-DE-KatjaNeural",
    "fr": "fr-FR-DeniseNeural",
    "es": "es-ES-ElviraNeural",
}

async def generate_voice(text: str, lang: str = "en", output_path: str = "./voice/news.mp3"):
    voice = VOICE_MAP.get(lang, "en-US-AriaNeural")
    communicate = edge_tts.Communicate(text, voice)
    await communicate.save(output_path)

# Example
# asyncio.run(generate_voice("今日新闻摘要...", lang="zh", output_path="news/zh_news.mp3"))
```

---

## Quick Command

List all available voices on your system:

```bash
edge-tts --list-voices | grep "zh-"
```

---

*Last updated: 2026-05-02*
