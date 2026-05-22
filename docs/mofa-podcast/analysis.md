# mofa-podcast Skill 分析报告

> 基于 skill-creator 框架审查、CLAUDE.md 规范、及代码质量评估。
> 分析时间：2026-05-19
> 分析范围：SKILL.md + manifest.json + Cargo.toml + architecture.dot + src/main.rs + 目录结构

---

## 一、执行摘要

`mofa-podcast` 是一个 **MoFA 类 Rust 二进制 skill**，核心功能是多人对话播客生成（TTS 语音合成 + 音频拼接）。整体架构清晰（4 步 pipeline：gather → script → review → produce），拥有 19 个预设/克隆声音，情绪标签系统完整。但存在 **版本号不一致**、**manifest 依赖声明不完整**、**SKILL.md 缺少规范结构**、**输出路径不符合仓库约定**等问题。

| 严重程度 | 数量 | 类别 |
|---------|:---:|------|
| 🔴 严重 | 2 | 版本不一致、manifest 依赖缺失 |
| 🟡 高 | 4 | 触发词/结构/规范 |
| 🟢 中 | 4 | 格式/输出/测试 |
| ⚪ 低 | 3 | 优化建议 |

---

## 二、严重问题（🔴）

### 问题 1：版本号在三处文件中不一致

| 文件 | 版本 |
|------|------|
| `SKILL.md` frontmatter | `0.4.1` |
| `manifest.json` | `0.4.5` |
| `Cargo.toml` | `0.4.5` |

**后果**：SKILL.md 的版本落后于实际代码，用户无法从文档判断 skill 的真实版本。如果存在版本相关的 bug 修复或行为变更，文档会造成误导。

**修复**：统一三处版本号。建议以 `Cargo.toml` 的 `0.4.5` 为准（代码是事实来源），同步更新 SKILL.md 和 manifest.json。

---

### 问题 2：manifest.json 依赖声明不完整

```json
"requires": {
  "bins": ["python3"],
  "os": ["darwin", "linux"]
}
```

**缺失项**：

1. **缺少 `ffmpeg`**：SKILL.md 第 112 行明确提到 "Outputs final audio via ffmpeg when available"，但 manifest 没有声明 ffmpeg 依赖。用户在没有 ffmpeg 的环境中使用时，会静默回退到 WAV 输出，而非提前被告知。

2. **缺少 `mofa-podcast` 二进制本身**：虽然 `requires_bin: mofa-podcast` 在 SKILL.md 中声明了，但 manifest.json 的 `requires.bins` 中却没有。manifest 和 SKILL.md 的依赖声明不同步。

3. **OS 限制**：`["darwin", "linux"]` 排除了 Windows。如果 Rust 二进制确实不支持 Windows，应在 SKILL.md 中说明；如果支持，应补充 `"windows"`。

4. **网络依赖**：`"requires_network": false` 与实际不符。TTS 生成（VoxCPM）需要调用本地 Python API，如果 VoxCPM 需要下载模型或调用云服务，则网络是必需的。

**修复**：
```json
"requires": {
  "bins": ["python3", "ffmpeg", "mofa-podcast"],
  "os": ["darwin", "linux"]
}
```
并核实 `requires_network` 的设置。

---

## 三、高优先级问题（🟡）

### 问题 3：SKILL.md 缺少 `<example>` 块

当前 Usage 部分使用自然语言描述流程（"1. Tell the agent your podcast topic..."），但没有使用 `<example>` 标签。根据 CLAUDE.md §4.2 和 skill-creator 的指导，`<example>` 是 SKILL.md 的推荐结构，能帮助模型理解预期的输入输出模式。

**修复**：在 `## How to use` 后添加至少 2 个 `<example>` 块：
```markdown
<example>
User: 做个三人科技播客，讨论 AI Agent 的发展趋势
Assistant: [调用 gather → 确认 speakers / genre / length → script → review → produce]
</example>
```

---

### 问题 4：SKILL.md 缺少 `## Instructions` 编号步骤

当前 SKILL.md 的结构是：
```
## How to use
## Voices
## Script Format
## Generation Engine
## Output
```

缺少统一的 `## Instructions` 部分。skill-creator 建议 Instructions 用编号步骤，避免模糊指令。当前 Agent 需要自行推断执行顺序（虽然 architecture.dot 定义了 pipeline，但 SKILL.md 并未引用它）。

**修复**：将 `## How to use` 改为 `## Instructions`，并明确编号步骤：
1. 收集用户需求（topic, speakers, genre, length）
2. 生成脚本 markdown
3. 展示脚本供用户审核
4. 调用 podcast_generate 生成音频

---

### 问题 5：description 不够 "pushy"

当前 description：
```
Multi-speaker podcast and dialogue generation with TTS voice cloning.
Triggers: podcast, 播客, multi-speaker audio, 多人对话, ...
```

skill-creator 指出 Claude 有 "undertrigger" 倾向，description 应该主动推荐自己：

> "Make sure to use this skill whenever the user mentions podcasts, dialogue generation, voice cloning, or wants to create any kind of multi-speaker audio content, even if they don't explicitly ask for a 'podcast.'"

**修复**：在 description 末尾追加 pushy 提示语，明确覆盖边缘场景（如"用XX的声音"、"配音"、"生成对话"）。

---

### 问题 6：触发词缺少口语化表达

现有触发词（18 个）：podcast, 播客, multi-speaker audio, 多人对话, 多人语音, radio show, talk show, 对话生成, 锵锵三人行, 两人对话, 三人对话, voice dialogue, 用XX的声音, 用XX和XX的风格, 声音模仿, 角色对话, 配音

**缺少的口语化表达**：
- "做个播客"
- "生成对话音频"
- "给这段文字配音"
- "用张三和李四的声音读这段"
- "做个电台节目"

**修复**：将口语化触发词追加到 description 的 Triggers 列表中。

---

## 四、中优先级问题（🟢）

### 问题 7：输出路径不符合仓库约定

当前输出路径（SKILL.md 第 117-118 行）：
```
skill-output/mofa-podcast/script.md
skill-output/mofa-podcast/podcast_<timestamp>.mp3
```

CLAUDE.md §4.3 约定：
- `~/Documents/notes/` — 文档类
- `~/Documents/scripts/` — 播客脚本类
- `./skill-output/<skill>-<timestamp>/` — 多文件流水线产物

当前路径是 `skill-output/mofa-podcast/`，缺少时间戳，多次运行会覆盖。且没有使用 `{YYYY-MM-DD}--{topic}__{type}.{ext}` 的命名格式。

**修复建议**：
```
./skill-output/mofa-podcast-{timestamp}/
  ├── {date}--{topic}__script.md
  ├── segments/*.wav
  └── {date}--{topic}__podcast.mp3
```

或者，如果保持 MoFA 的约定不变，至少应在 SKILL.md 中说明"多次运行会覆盖，请自行备份"。

---

### 问题 8：architecture.dot 未被 SKILL.md 引用

`architecture.dot` 定义了完整的 4 步 pipeline（gather → script → review → produce），但 SKILL.md 中没有任何地方提到这个文件。Agent 在运行时不会自动读取它。

**修复**：在 `## Instructions` 中引用 architecture.dot：
```markdown
## Pipeline 架构
详见 `architecture.dot`（Graphviz），4 步流程：gather → script → review → produce。
```

---

### 问题 9：缺少 styles/ 目录

根据 CLAUDE.md §5.1，MoFA 类 skill 的标准骨架包含 `styles/` 目录（TOML 风格定义）。`mofa-podcast` 没有 styles/，原因可能是播客生成不需要视觉风格。但如果未来支持"播客封面图生成"或"播客可视化"，这个目录会需要。

**当前状态**：合理缺失（语音类 skill 不需要 styles），但应在文档中说明。

---

### 问题 10：缺少 evals/ 和测试覆盖

只有一个 `scripts/test-integration.sh`，没有：
- `evals/evals.json`（skill-creator 推荐的标准 eval 结构）
- 单元测试（Rust `#[test]` 或 Python unittest）
- 描述触发准确度测试（trigger eval）

**建议**：至少添加 trigger eval 集合（8-10 个 should-trigger + 8-10 个 should-not-trigger），验证 description 的触发准确度。

---

## 五、低优先级优化（⚪）

### 建议 11：Voice profiles 缺少文档

19 个 voice profiles 存在于 `voice_profiles/` 目录，但除了 `douwt` 和 `yangmi` 在 SKILL.md 中提到外，其他克隆声音没有文档说明。用户不知道 `mabaoguo_ref.wav` 是什么声音。

**建议**：在 `references/voice-guide.md` 中添加声音目录，说明每个预设/克隆声音的来源和特点。

---

### 建议 12：Emotion 映射可以扩展

当前 10 个情绪标签（calm, excited, serious, warm, angry, sad, cheerful, dramatic, curious, thoughtful）覆盖了基础场景。但中文播客中常见的 "吐槽"、"调侃"、"正经"、"撒娇" 等情绪没有对应映射。

**建议**：增加中文语境下的情绪映射，如：
- `teasing` → "playful, mocking, light-hearted banter"
- `sarcastic` → "dry, ironic, deadpan delivery"

---

### 建议 13：TTS 语言推断逻辑有盲区

`infer_tts_language` 通过统计 CJK 和拉丁字母数量来推断语言。对于中日韩混合文本（如中文中夹杂大量英文术语）可能会误判。

```rust
if cjk_count >= latin_count { Chinese } else { English }
```

**建议**：增加阈值缓冲（如 `cjk_count >= latin_count * 0.7`），或支持用户显式指定语言。

---

## 六、跨文件引用关系图

```
SKILL.md
├── 引用 manifest.json（tools: podcast_voices, podcast_voice_save, podcast_generate）
├── 引用 voice_profiles/（预设 + 克隆声音）
├── 引用 architecture.dot（pipeline 定义，但 SKILL.md 未显式引用）
└── 输出到 skill-output/mofa-podcast/

manifest.json
├── 定义 3 个 tool 接口
├── binaries.darwin-aarch64（url/sha256 为空）
└── requires（缺少 ffmpeg, mofa-podcast）

Cargo.toml
└── version 0.4.5（与 SKILL.md 的 0.4.1 不一致）

src/main.rs
├── 调用 tts_engine.py / tts_engine_mock.py（Python TTS 后端）
├── 使用 ffmpeg（未在 manifest 声明）
└── 解析 markdown script，生成 segment WAV，拼接为 MP3
```

---

## 七、修改优先级建议

| 顺序 | 项 | 预计工作量 |
|:---:|---|:---|
| 1 | 统一三处版本号（以 Cargo.toml 0.4.5 为准） | 1 分钟 |
| 2 | 补全 manifest.json 依赖（ffmpeg, mofa-podcast） | 2 分钟 |
| 3 | 修复 requires_network 设置 | 1 分钟 |
| 4 | 添加 `<example>` 块（2 个） | 5 分钟 |
| 5 | 重构 `## How to use` 为 `## Instructions` 编号步骤 | 10 分钟 |
| 6 | 追加口语化触发词 + pushy description | 3 分钟 |
| 7 | 核实 OS 支持范围（darwin/linux/windows） | 2 分钟 |
| 8 | 在 Instructions 中引用 architecture.dot | 2 分钟 |
| 9 | 添加 voice profiles 文档 | 15 分钟 |
| 10 | 设计 evals（trigger eval + integration test） | 30 分钟 |

---

## 八、总体评估

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 架构设计 | ⭐⭐⭐⭐⭐ | 4 步 pipeline 清晰，MoFA 风格完整，情绪/BGM/停顿 cue 系统成熟 |
| 代码质量 | ⭐⭐⭐⭐ | Rust 代码结构合理，语言推断、情绪映射、声音分组逻辑清晰 |
| 声音资产 | ⭐⭐⭐⭐⭐ | 19 个 voice profiles，预设 + 克隆机制完整 |
| SKILL.md 规范 | ⭐⭐⭐ | 缺少 example、Instructions 编号、pushy description |
| 触发准确度 | ⭐⭐⭐ | 触发词覆盖尚可，但缺少口语化表达，description 不够 pushy |
| 输出规范 | ⭐⭐⭐ | 路径缺少时间戳，命名不符合仓库约定 |
| 依赖声明 | ⭐⭐ | manifest.json 不完整，版本号不一致 |
| 可测试性 | ⭐⭐ | 仅有一个 integration test，缺少 eval 框架 |
