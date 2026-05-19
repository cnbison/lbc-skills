---
name: mofa-podcast
description: >-
  多人对话播客与语音合成生成器。当用户想做播客、生成对话音频、给文字配音、
  用特定声音朗读、做电台节目、锵锵三人行风格对话时，激活本法。
  支持 1-5 人对话、情绪标签、BGM cue、声音克隆。
  请务必在用户提到播客、对话生成、配音、声音克隆、多人语音、角色对话、
  电台节目、锵锵三人行、用某人的声音时激活本法，即使用户没有明确说"播客"。

  Triggers: podcast, 播客, multi-speaker audio, 多人对话, 多人语音, radio show,
  talk show, 对话生成, 锵锵三人行, 两人对话, 三人对话, voice dialogue,
  用XX的声音, 用XX和XX的风格, 声音模仿, 角色对话, 配音,
  做个播客, 生成对话音频, 给这段文字配音, 做个电台节目, 播客生成
version: 0.4.5
author: hagency
always: false
requires_bins: mofa-podcast, ffmpeg, python3
pipeline: architecture.dot
---

# MoFA Podcast — Multi-Speaker Podcast Generator

Generate professional multi-speaker podcasts from a topic or text. The pipeline collects speaker preferences, expands content into a scripted dialogue with emotion and music cues, lets you review before generation, then produces a final MP3.

## Usage

<example>
User: 做个三人科技播客，讨论 AI Agent 的发展趋势
Assistant: [进入 gather 阶段]
好的！我们来做一个三人科技播客。请告诉我：
1. 三位 speaker 的名字和想要的声音（可用预设 douwt、yangmi，或克隆声音）
2. 风格：talk-show / news / interview / storytelling / debate
3. 目标时长：如 5 分钟、10 分钟

[收集完毕后生成脚本 → 展示供审核 → 生成音频]
- 脚本：./skill-output/mofa-podcast-{timestamp}/{date}--{topic}__script.md
- 音频：./skill-output/mofa-podcast-{timestamp}/{date}--{topic}__podcast.mp3
</example>

<example>
User: 用窦文涛和罗永浩的风格，给这段文字配个对话
Assistant: [进入 gather 阶段]
好的！用 douwt（窦文涛风格）和 yangmi（或其他克隆声音）。请把文字贴给我，并确认：
1. Speaker 名字
2. 风格：talk-show / 闲聊
3. 时长

[生成脚本 → 审核 → 生成音频]
</example>

<example>
User: 只重新生成音频，不要改脚本
Assistant: 读取已审核的 script.md，直接调用 podcast_generate 生成音频。
</example>

---

## Instructions

### Pipeline 架构

4 步流程详见 `architecture.dot`（Graphviz）：**gather → script → review → produce**

```
gather   → 收集用户需求（topic、speakers、genre、length）
script   → 生成完整 markdown 脚本
review   → 展示给用户审核，支持修改
produce  → 调用 podcast_generate 生成音频
```

### 步骤 1：收集需求（gather）

向用户确认以下信息。如果用户在消息中已经提供了部分信息，确认并只询问缺失项：

1. **Topic / source content** — 播客主题或原文
2. **Speakers（1-5 人）** — 每人名字 + 声音
3. **Genre / style** — drama、news、talk-show、interview、storytelling、debate、或自定义
4. **Target length** — 目标时长（分钟）

 Speaker 声音选择：
 - **默认预设**：`douwt`（窦文涛风格）、`yangmi`（杨幂风格）
 - **克隆声音**：`clone:xxx`（用户自行添加，见下文"声音管理"）

调用 `podcast_voices` 列出当前可用声音供用户选择。

收集完毕后输出结构化摘要：

```config
TOPIC: <topic>
SPEAKERS: <count>
SPEAKER_1: <name> | <voice> | <built-in or clone>
...
GENRE: <genre>
LENGTH: <minutes>min
```

### 步骤 2：生成脚本（script）

将 topic 扩展为完整的 markdown 播客脚本。规则：

- **字数**：~150 词/分钟，10 分钟 ≈ 1500 词对话
- **Speaker 行格式**：`[CharacterName - voice_persona, emotion] 对话文本`
  - emotion: calm、excited、serious、warm、angry、sad、cheerful、dramatic、curious、thoughtful
- **BGM cue**：`[BGM: description — fade_type, Ns]`（intro、transition、outro）
- **Pause cue**：`[PAUSE: Ns]`（1-3 秒）
- **结构**：开场 → 主体（2-4 段）→ 过渡 → 结尾
- **Genre 匹配**：
  - drama: 戏剧性、情绪张力
  - news: 正式、事实播报
  - talk-show: 闲聊、调侃、互动
  - interview: Q&A、追问
  - storytelling: 叙事、沉浸
  - debate: 对立观点、结构化论证
- **语言**：匹配用户 topic 语言

保存脚本到 `./skill-output/mofa-podcast-{timestamp}/{date}--{topic}__script.md`。

### 步骤 3：审核（review）

向用户展示完整脚本，询问：

1. ✅ **Approve** — 进入生成
2. ✏️ **Edit** — 说明修改点，读取 script 修改后重新展示
3. ❌ **Cancel** — 停止

### 步骤 4：生成音频（produce）

用户批准后，调用 `podcast_generate`：

```json
{"script_path": "./skill-output/mofa-podcast-{timestamp}/{date}--{topic}__script.md"}
```

生成完成后向用户报告最终音频路径。

---

## 声音管理

### 预设声音（built-in）

| 声音 | 说明 | 类型 |
|------|------|------|
| `douwt` | 窦文涛风格 | 预设 |
| `yangmi` | 杨幂风格 | 预设 |

### 克隆声音（clone）

用户可自行添加、查询、删除克隆声音。

**查询当前克隆声音**：
调用 `podcast_voices` 列出所有可用声音（预设 + 克隆）。

**添加克隆声音**：
1. 用户提供一段参考音频（5-30 秒，WAV 格式，干净人声）
2. 调用 `podcast_voice_save(name="xxx", audio_path="/path/to/ref.wav")`
3. 保存后在脚本中使用 `clone:xxx`

**删除克隆声音**：
直接删除 `voice_profiles/` 目录下的对应 `.wav` 文件即可。

---

## 脚本格式参考

```markdown
# My Podcast Title

**Genre**: talk-show | **Duration**: ~10 min | **Speakers**: 3

| Character | Voice | Type |
|-----------|-------|------|
| Host | douwt | built-in |
| Guest1 | yangmi | built-in |
| Expert | clone:sarah | clone |

---

[BGM: Upbeat intro music — fade-in, 5s]

[Host - douwt, cheerful] Welcome to today's show!

[Guest1 - yangmi, excited] Thanks for having me!

[BGM: Soft transition — crossfade, 3s]

[Expert - clone:sarah, serious] Let me share some insights...

[PAUSE: 2s]

[Host - douwt, warm] That's fascinating. Let's dig deeper...

[BGM: Outro music — fade-out, 5s]
```

### Emotion 标签

| 标签 | 效果 |
|------|------|
| `calm` | 自然、平和 |
| `excited` | 热情、 energetic |
| `serious` | 正式、沉稳 |
| `warm` | 友好、亲切 |
| `angry` | 愤怒、强烈 |
| `sad` | 低沉、沉思 |
| `cheerful` | 欢快、积极 |
| `dramatic` | 戏剧性、张力 |
| `curious` | 好奇、探究 |
| `thoughtful` | 深思、从容 |

### BGM 与 Pause Cue

- `[BGM: description — fade-in, Ns]` — 淡入
- `[BGM: description — fade-out, Ns]` — 淡出
- `[BGM: description — crossfade, Ns]` — 交叉淡入淡出
- `[PAUSE: Ns]` — 静默 N 秒

---

## 输出规范

所有产物输出到 `./skill-output/mofa-podcast-{timestamp}/` 目录：

| 文件 | 命名 |
|------|------|
| 脚本 | `{date}--{topic}__script.md` |
| 分段 WAV | `segments/seg_{NNN}_{voice}.wav` |
| 最终音频 | `{date}--{topic}__podcast.mp3`（ffmpeg 可用）或 `.wav`（fallback）|

---

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
