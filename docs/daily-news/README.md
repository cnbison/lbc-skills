# daily-news · 开发总览

## 定位

Claw Daily News 自动化流水线。RSS 采集 → 过滤去重 → Agent 摘要/文章/播客脚本 → TTS 合成 MP3。Python 脚本负责确定性数据工作，Agent 负责 LLM 生成。

## 当前状态

见 [STATUS.md](./STATUS.md) — 🟢 active

## 核心文件索引

| 文件 | 说明 |
|------|------|
| `../../skills/daily-news/SKILL.md` | Skill 定义入口 |
| `../../skills/daily-news/manifest.json` | 元数据与触发词 |
| `../../skills/daily-news/run_pipeline.py` | 流水线主入口 |
| `../../skills/daily-news/scripts/` | 流水线脚本（采集、过滤、摘要、生成、TTS） |
| `../../skills/daily-news/config/sources.yaml` | RSS 源与关键词配置 |
| `../../skills/daily-news/references/` | 文章模板、播客模板 |
| `../../output/daily-news/` | 生成产物目录 |

## 版本历史

见 [CHANGELOG.md](./CHANGELOG.md)

## 分析报告

见 [analysis.md](./analysis.md) — skill-creator 框架审查报告（2026-05-18）
