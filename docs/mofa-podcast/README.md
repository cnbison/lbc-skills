# mofa-podcast · 开发总览

## 定位

多人对话播客与语音合成生成器。支持 1-5 人对话、情绪标签、BGM cue、声音克隆。从话题或文本生成专业级多人播客 MP3。

## 当前状态

见 [STATUS.md](./STATUS.md) — 🟢 active

## 核心文件索引

| 文件 | 说明 |
|------|------|
| `../../skills/mofa-podcast/SKILL.md` | Skill 定义入口 |
| `../../skills/mofa-podcast/manifest.json` | 元数据与依赖 |
| `../../skills/mofa-podcast/architecture.dot` | Pipeline 架构图（4 步：gather→script→review→produce） |
| `../../skills/mofa-podcast/src/main.rs` | Rust 二进制主入口 |
| `../../skills/mofa-podcast/src/tts_engine.py` | TTS 引擎 |
| `../../skills/mofa-podcast/scripts/run.sh` | Wrapper 脚本（处理 stdin protocol + 去除 quarantine） |
| `../../skills/mofa-podcast/scripts/test-integration.sh` | 集成测试 |
| `../../skill-output/mofa-podcast-*/` | 生成产物目录 |

## 版本历史

见 [CHANGELOG.md](./CHANGELOG.md)

## 分析报告

见 [analysis.md](./analysis.md) — skill-creator 框架审查报告
