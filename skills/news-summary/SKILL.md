---
name: news-summary
description: This skill should be used when the user asks for news updates, daily briefings, or what's happening in the world. Fetches news from trusted international RSS feeds and can create voice summaries.
---

# News Summary

## Overview

Fetch and summarize news from trusted international sources via RSS feeds.

## RSS Feeds

### BBC (Primary)
```bash
# World news
curl -s "https://feeds.bbci.co.uk/news/world/rss.xml"

# Top stories
curl -s "https://feeds.bbci.co.uk/news/rss.xml"

# Business
curl -s "https://feeds.bbci.co.uk/news/business/rss.xml"

# Technology
curl -s "https://feeds.bbci.co.uk/news/technology/rss.xml"
```

### Reuters
```bash
# World news
curl -s "https://www.reutersagency.com/feed/?best-regions=world&post_type=best"
```

### NPR (US perspective)
```bash
curl -s "https://feeds.npr.org/1001/rss.xml"
```

### Al Jazeera (Global South perspective)
```bash
curl -s "https://www.aljazeera.com/xml/rss/all.xml"
```

## Parse RSS

Use Python with `feedparser` for reliable extraction:

```python
import feedparser

url = "https://feeds.bbci.co.uk/news/technology/rss.xml"
feed = feedparser.parse(url)

for entry in feed.entries[:10]:
    print(f"Title: {entry.title}")
    print(f"Summary: {entry.get('summary', '')[:200]}")
    print(f"Link: {entry.link}")
    print()
```

Install `feedparser` if not available:
```bash
pip install feedparser
```

## Workflow

### Text summary
1. Fetch BBC world headlines
2. Optionally supplement with Reuters/NPR
3. Summarize key stories
4. Group by region or topic
5. Save the full report to a timestamped markdown file in the `news/` subdirectory:
   ```bash
   mkdir -p news
   cat > "news/news_summary_$(date +%Y%m%d_%H%M).md" << 'EOF'
   # News Summary — $(date '+%Y-%m-%d %H:%M')

   [full report content here]
   EOF
   ```

### Voice summary
1. Create text summary in the **same language as the user's request**
2. Detect the user's language and select the matching Edge TTS voice
3. Generate voice with Edge TTS (free, no API key required)
4. Send as audio message

**Language-to-Voice Reference:**

| User Language | Edge TTS Voice |
|---------------|----------------|
| 中文 (Chinese) | `zh-CN-XiaoxiaoNeural` |
| English (US) | `en-US-AriaNeural` |
| English (UK) | `en-GB-SoniaNeural` |
| 日本語 (Japanese) | `ja-JP-NanamiNeural` |
| 한국어 (Korean) | `ko-KR-SunHiNeural` |
| Deutsch (German) | `de-DE-KatjaNeural` |
| Français (French) | `fr-FR-DeniseNeural` |
| Español (Spanish) | `es-ES-ElviraNeural` |

```bash
# Install edge-tts if not already available
# pip install edge-tts

# Example: Chinese request
edge-tts --voice "zh-CN-XiaoxiaoNeural" \
  --text "<news summary text>" \
  --write-media ./voice/news.mp3

# Example: English request
edge-tts --voice "en-US-AriaNeural" \
  --text "<news summary text>" \
  --write-media ./voice/news.mp3
```

Or via Python:
```python
import edge_tts
import asyncio

async def main(text, voice="en-US-AriaNeural"):
    communicate = edge_tts.Communicate(text, voice)
    await communicate.save("./voice/news.mp3")

# Pick voice based on detected user language
# asyncio.run(main(summary_text, "zh-CN-XiaoxiaoNeural"))
```

## Example Output Format

```
📰 News Summary [date]

🌍 WORLD
- [headline 1]
- [headline 2]

💼 BUSINESS
- [headline 1]

💻 TECH
- [headline 1]
```

## Best Practices

- Keep summaries concise (5-8 top stories)
- Prioritize breaking news and major events
- For voice: ~2 minutes max
- Balance perspectives (Western + Global South)
- Cite source if asked