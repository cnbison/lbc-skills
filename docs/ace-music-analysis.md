# ace-music Skill 分析报告

> 基于 skill-creator 框架审查、CLAUDE.md 规范、及代码质量评估。
> 分析时间：2026-05-19
> 分析范围：SKILL.md + _meta.json + scripts/generate.sh + references/api-docs.md + 目录结构 + 加载路径检查

---

## 一、执行摘要

`ace-music` 是一个**轻量级 bash + curl 类 skill**，封装了 ACE Music（ACE-Step 1.5）的免费托管 API，用一行命令即可从文本提示词生成 MP3 音乐。脚本设计简洁、API 文档完整，但**作为本仓库的 skill 存在多个结构性问题**：最严重的是**软链未建立 → Claude Code 无法在会话中触发它**；其次 SKILL.md 严重违反 CLAUDE.md 的 frontmatter / Body 规范（无 Triggers、无 `<example>`、无 `## Instructions`、英文单语），且 generate.sh 内部存在若干健壮性 bug。

| 严重程度 | 数量 | 类别 |
|---------|:---:|------|
| 🔴 严重 | 2 | 加载路径缺失、SKILL.md frontmatter 不合规 |
| 🟡 高 | 5 | Body 缺结构、依赖未声明、输出路径违规、语言单语、第三方来源未标注 |
| 🟢 中 | 4 | bash 转义 bug、JSON 注入风险、Python 调用冗余、错误处理 |
| ⚪ 低 | 3 | 优化建议 |

**一句话结论**：API 封装本身写得不错，但作为 lbc-skills 仓库里的"可用 skill"，它处于**未激活、未规范化**状态 —— 用户即使说"做一首歌"，Claude 也不会触发它。

---

## 二、产品定位与现状

### 2.1 它是什么

- **核心功能**：调用 `https://api.acemusic.ai/v1/chat/completions`，将自然语言提示词（可选歌词）转为 MP3 音频。
- **底层模型**：ACE-Step 1.5（开源 Suno 替代品）。
- **来源**：`_meta.json` 显示 owner=`fspecii`，源自 [openclaw/skills@c0a5107](https://github.com/openclaw/skills/commit/c0a510771e56fb26aa12cecc444ad3d6f5b963a1)，**第三方同步过来的 skill**。
- **形态**：CLAUDE.md §3 中的"Python 流水线 skill"的轻量变体 —— 实际是 `bash + curl + python3 oneliner`。

### 2.2 目录结构

```
skills/ace-music/
├── SKILL.md           # 75 行，prompt skill 主体
├── _meta.json         # 11 行，第三方来源元数据
├── references/
│   └── api-docs.md    # 103 行，ACE API 完整规格
└── scripts/
    └── generate.sh    # 140 行，核心生成脚本
```

整体非常精简。脚本类 skill 的"两层职责分离"（脚本做确定性数据搬运、SKILL.md 指导 LLM）在这里基本不存在 —— 因为本 skill 本来就没什么需要 LLM 推理的环节（提示词由用户给，API 直接产出音频）。这是合理的设计选择。

---

## 三、严重问题（🔴）

### 问题 1：`.claude/skills/ace-music` 软链不存在 → skill 永远不会被触发

```bash
$ ls -la .claude/skills/ace-music
ls: .claude/skills/ace-music: No such file or directory
```

CLAUDE.md §2 明确说明：

> Claude Code 只从 `.claude/skills/`（项目级）或 `~/.claude/skills/`（用户级）加载 skill。**放在 `skills/` 下的源文件对运行时不可见**。

而 §6 的 SOP 第 2 步要求新增 skill 时必须 `ln -s ../../skills/<name> .claude/skills/<name>`。**当前 ace-music 跳过了这一步**，导致：

- 用户在会话中说"做一首歌"/"生成音乐"，Claude 不会激活 ace-music。
- README.md 把它列为"🚧 进行中"是诚实的状态标识，但 SKILL.md 里却写得像可用产品。

**修复**：

```bash
cd /Users/loubicheng/project/lbc-skills
ln -s ../../skills/ace-music .claude/skills/ace-music
git add .claude/skills/ace-music
git commit -m "feat(ace-music): expose skill to .claude/skills load path"
```

---

### 问题 2：SKILL.md frontmatter 严重不合规

当前 frontmatter：

```yaml
---
name: ace-music
description: Generate AI music using ACE-Step 1.5 via ACE Music's free API. Use when the user asks to create, generate, or compose music, songs, beats, instrumentals, or audio tracks. Supports lyrics, style prompts, covers, and repainting. Free API, no cost.
---
```

对比 CLAUDE.md §4.1 的规范，**至少 4 项偏离**：

| 规范项 | 要求 | 现状 |
|------|------|------|
| `description` 含显式 `Triggers:` 段落 | 必填 | ❌ 无 |
| 触发词中英双语 | 必填（§7） | ❌ 纯英文 |
| `version` | 可选但建议 | ❌ 无 |
| `requires_bins` | 有外部依赖时必填 | ❌ 无（实际依赖 curl, python3, base64） |
| `requires_env` | 有环境变量依赖时必填 | ❌ 无（实际依赖 `ACE_MUSIC_API_KEY`） |

**后果**：

- 没有 `Triggers:`：Claude 的 skill 路由器只能靠 description 中的英文关键词（"create music", "generate", "compose"）做模糊匹配，命中率显著低于显式触发词。
- 单语：中文用户说"作曲""配乐""生成音乐"时，触发概率进一步降低。
- 未声明 `requires_env`：用户不知道要先配 `ACE_MUSIC_API_KEY`，会在第一次运行时报错（脚本里有兜底，但用户体验差）。

**修复样板**：

```yaml
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
```

---

## 四、高优先级问题（🟡）

### 问题 3：SKILL.md Body 缺少 `<example>` 和 `## Instructions`

CLAUDE.md §4.2 要求：

```markdown
## Usage

<example>
User: trigger phrase
Assistant: [Expected behavior]
</example>

## Instructions

1. 第一步...
2. 第二步...
```

当前 ace-music 的 Body 只有 `## Setup` / `## Quick Generation` / `## Advanced Usage` / `## Parameters Guide` / `## Notes`，**完全没有 `<example>` 块和编号步骤的 Instructions**。这违反了仓库内其他 skill 的统一约定（参考 `mofa-podcast` / `daily-news` / `follow-builders`）。

**为什么这条很重要**：`<example>` 块是给 Claude 看的"模板对话"，让模型在被触发时知道该用什么口吻回应、需要先问什么、再调用哪个工具。没有 example，Claude 容易跳过提示，直接 raw 调用脚本。

**修复样板**：

````markdown
## Usage

<example>
User: 帮我做一首关于夏天的流行歌
Assistant: 好的！我用 ace-music 给你生成。先确认几个选项：
1. 时长（默认 AI 决定）
2. 语言（en/zh/ja…）
3. 是否需要歌词，还是 AI 自动写？
4. 实例风格？（如：复古迪斯科、Bossa Nova、电子流行）

[确认后调用：scripts/generate.sh "upbeat summer pop ..." --duration 60 --output output/ace-music/summer.mp3]
</example>

<example>
User: 给这段歌词配个民谣女声
Assistant: 收到。请贴歌词，我生成一首民谣女声的版本。
[调用：scripts/generate.sh "gentle acoustic folk, female vocal" --lyrics "..." --duration 90 --output output/ace-music/folk.mp3]
</example>

## Instructions

1. **检查 API Key**：确认环境变量 `ACE_MUSIC_API_KEY` 已设置；未设置时引导用户访问 https://acemusic.ai/playground/api-key 获取。
2. **收集参数**：与用户确认 prompt（风格描述）、是否需要 lyrics、duration、language、instrumental。
3. **生成命令**：调用 `scripts/generate.sh`，输出到 `output/ace-music/{date}--{topic}.mp3`。
4. **多版本**：当 `--batch >1` 时把所有变体路径都报告给用户。
5. **失败兜底**：若 API 返回非 2xx，stderr 已打印响应体；告知用户 ACE Music 平台状态可能异常。
````

---

### 问题 4：scripts/generate.sh 的输出路径违反仓库约定

```bash
OUTPUT="output_$(date +%s).mp3"
```

CLAUDE.md §4.3 明确规定多文件流水线产物应输出到 `./output/<skill-name>/`，例如 `./output/ace-music/`。当前脚本直接落在**调用者的当前工作目录**，会污染用户的工作区，且不便集中清理。

**修复**：

```bash
OUTPUT_DIR="${ACE_MUSIC_OUTPUT_DIR:-./output/ace-music}"
mkdir -p "$OUTPUT_DIR"
OUTPUT="${OUTPUT_DIR}/$(date +%Y%m%d-%H%M%S).mp3"
```

并支持 `--output` 覆盖。

---

### 问题 5：依赖未声明，运行时静默失败

generate.sh 实际依赖三个二进制 + 一个环境变量：

| 依赖 | 用途 | 当前是否声明 |
|---|---|---|
| `curl` | 调用 API | ❌ |
| `python3` | JSON 解析 + base64 解码 | ❌ |
| `base64`（隐含通过 python） | — | OK |
| `ACE_MUSIC_API_KEY` | 鉴权 | ⚠️ 仅在脚本内 `[[ -z $API_KEY ]]` 检查 |

CLAUDE.md §4.1 推荐用 frontmatter `requires_bins` / `requires_env` 显式声明（见 mofa-podcast 的 manifest.json 做法）。当前 ace-music 既没有 manifest.json，也没在 SKILL.md frontmatter 声明依赖。

---

### 问题 6：触发词单语 + 关键中文表达缺失

CLAUDE.md §7：

> **触发词**：中英文都要列。

当前 description 全英文，且关键中文场景词缺失（作曲、写首歌、配乐、生成音乐、AI 音乐、给歌词配曲）。在已经 README.md 列出 🚧 状态的情况下，这条偏离让中文用户几乎不可能触发它。

修复方案见问题 2 的样板。

---

### 问题 7：第三方来源在 SKILL.md 中未标注

`_meta.json` 显示这是从 `github.com/openclaw/skills` 同步过来的，作者是 fspecii。但 SKILL.md 里没有任何 "Origin / Source / Upstream" 信息。

CLAUDE.md 虽然没硬性要求，但参考仓库中其他第三方 skill 的处理（例如 `skills/last30days` 在 README 中标注"第三方"），ace-music 应至少在 frontmatter 加 `author: fspecii`，或在 Body 末尾加一行：

```markdown
> Source: synced from [openclaw/skills@c0a5107](https://github.com/openclaw/skills/commit/c0a510771e56fb26aa12cecc444ad3d6f5b963a1) by @fspecii.
```

这样下游用户做调试时能追溯到原仓库。

---

## 五、中等问题（🟢）

### 问题 8：tagged-mode 里的 `\n` 不是真换行

```bash
CONTENT="<prompt>${PROMPT}</prompt>\n<lyrics>${LYRICS}</lyrics>"
```

bash 双引号字符串里 `\n` **不会被解释为换行**，它就是字面两个字符 `\` + `n`。这串字符接着被 `python json.dumps`，结果是 `"<prompt>...</prompt>\\n<lyrics>..."`（即在 JSON 里也只是字面 `\n`，不是换行）。

最终 API 收到的 user content 是单行字符串：`<prompt>...</prompt>\n<lyrics>...</lyrics>`。这未必影响模型理解（标签结构仍能解析），但与 references/api-docs.md Mode 1 示例里展示的"两行结构"不一致。

**修复**：

```bash
CONTENT="<prompt>${PROMPT}</prompt>
<lyrics>${LYRICS}</lyrics>"
```

或者用 printf：

```bash
CONTENT=$(printf '<prompt>%s</prompt>\n<lyrics>%s</lyrics>' "$PROMPT" "$LYRICS")
```

---

### 问题 9：数字字段无校验，存在 JSON 注入风险

```bash
[[ -n "$DURATION" ]] && AUDIO_CONFIG="$AUDIO_CONFIG,\"duration\":$DURATION"
[[ -n "$BPM" ]] && AUDIO_CONFIG="$AUDIO_CONFIG,\"bpm\":$BPM"
[[ -n "$SEED" ]] && BODY="$BODY,\"seed\":$SEED"
[[ "$BATCH_SIZE" -gt 1 ]] && BODY="$BODY,\"batch_size\":$BATCH_SIZE"
```

`DURATION` / `BPM` / `SEED` 直接以裸值拼入 JSON。如果用户传 `--bpm "120, \"evil\":true"`，会产出畸形 JSON，curl 拿到 4xx 错误（不会被 RCE，因为不在 eval/shell 执行链上，但脚本静默失败，调试不友好）。

**修复**：用 `[[ "$DURATION" =~ ^[0-9]+(\.[0-9]+)?$ ]]` 校验，否则提前 exit 并提示。

---

### 问题 10：4 次 fork python3 处理同一个 RESPONSE

```bash
echo "$RESPONSE" | python3 -c "...exit(0 if 'choices' in d else 1)"  # 第 1 次
METADATA=$(echo "$RESPONSE" | python3 -c "...")                       # 第 2 次
COUNT=$(echo "$RESPONSE" | python3 -c "...")                          # 第 3 次
echo "$RESPONSE" | python3 -c "...(decode and save)"                  # 第 4 次
```

每次都重新启动 python 解释器、重新 `json.load`。对小响应无所谓，但响应包含 base64 MP3（几 MB）时，4 次 `echo "$RESPONSE"`（每次都 stringify 全文）+ 4 次 JSON 解析，开销可观。

**修复**：把响应写到临时文件，一次 python 调用完成全部工作。

---

### 问题 11：`--language en` 强制默认覆盖 CoT 自动检测

```bash
LANGUAGE="en"  # 默认
AUDIO_CONFIG="{\"vocal_language\":\"$LANGUAGE\",\"format\":\"$FORMAT\""
```

api-docs.md 显示 `use_cot_language` 默认 `true`，会让 LLM 自动检测语言。但当前脚本**总是**带上 `vocal_language: en`，等于强制覆盖了 CoT。中文用户即使 prompt 是"做一首中文流行歌"，结果可能是英文人声。

**修复**：默认 `LANGUAGE=""`，只有用户传 `--language` 才写入 audio_config，让 API 端的 CoT 自动决定。

---

## 六、低优先级建议（⚪）

### 建议 12：Quick Generation 段落里的命令应该用绝对 / 相对仓库路径

当前文档：

```bash
scripts/generate.sh "upbeat pop song about summer" --duration 30 --output summer.mp3
```

用户在仓库根目录运行时是 `skills/ace-music/scripts/generate.sh`，从 `.claude/skills/ace-music/` 也不能直接 `scripts/...`（依赖 CWD）。建议在 SKILL.md 中明确：

```bash
# 从仓库根目录
./skills/ace-music/scripts/generate.sh ...

# 或从 .claude/skills/ace-music 软链
cd .claude/skills/ace-music && ./scripts/generate.sh ...
```

或者引入一个 wrapper（仿照 mofa-podcast 的做法）让路径无关。

---

### 建议 13：缺少 health-check / 烟雾测试

api-docs.md 第 100 行提到 `GET /health` 返回 `{"status":"ok"}`，但 scripts/ 下没有任何最小化的连通性检查脚本。新建 `scripts/health.sh`：

```bash
#!/usr/bin/env bash
curl -sf "${ACE_MUSIC_BASE_URL:-https://api.acemusic.ai}/health" \
  -H "Authorization: Bearer $ACE_MUSIC_API_KEY" \
  || { echo "ACE Music API unreachable" >&2; exit 1; }
```

让用户在配置完 API key 后能用 5 秒内验证可用性。

---

### 建议 14：README 里的 🚧 状态标识需要落地

README.md 第 93 行标了 🚧（进行中），但 SKILL.md / scripts 里看不出"未完成"。建议要么完成本报告列出的修复后改为 ✅，要么在 SKILL.md 顶部加个 banner：

```markdown
> **状态：🚧 进行中。** API 调用流程已就绪，但 skill 尚未注册到加载路径，正在按 CLAUDE.md 规范进行整改。
```

---

## 七、对比基线 —— 仓库内同类 skill 的差距

| 维度 | ace-music（现状） | follow-builders | mofa-podcast |
|------|------|------|------|
| 软链已建 | ❌ | ✅ | ✅ |
| frontmatter Triggers 中英双语 | ❌ | ✅ | ✅ |
| `<example>` 块 | ❌ | ✅ | ✅ (3 个) |
| `## Instructions` 编号步骤 | ❌ | ✅ | ✅ (4 步) |
| 依赖声明（`requires_bins`/`requires_env`） | ❌ | ✅ | ✅ |
| 输出落到 `output/<skill>/` | ❌ | ✅ | ✅ (skill-output/) |
| 调用约定 / wrapper | N/A | N/A | ✅ |

ace-music 在结构化程度上**显著落后**于本仓库其他 skill。

---

## 八、整改优先级（建议执行顺序）

1. **🔴 立即**：建立软链 `.claude/skills/ace-music → ../../skills/ace-music`，否则一切下游修复都看不到效果。
2. **🔴 立即**：按问题 2 重写 SKILL.md frontmatter，补齐 Triggers / 双语 / requires_*。
3. **🟡 一并**：补齐 SKILL.md 的 `<example>` 和 `## Instructions`，加 author/source 标注。
4. **🟡 一并**：把脚本输出目录改到 `output/ace-music/`。
5. **🟢 跟进**：修复 bash `\n` 转义、数字字段校验、合并 python 调用。
6. **🟢 跟进**：移除 `LANGUAGE` 默认 `en`，尊重 CoT 自动检测。
7. **⚪ 长期**：加 health.sh、wrapper、烟雾测试、把 README 状态推进到 ✅。

完成 1–3 后，这个 skill 就能在新会话中正常激活；完成 1–5 后，它在质量和健壮性上能跟上 follow-builders / mofa-podcast 的水平。

---

## 九、亮点（值得保留）

不全是问题。这个 skill 也有几处做得好：

1. **API 文档完整**：`references/api-docs.md` 把 ACE-Step 的所有字段、4 种输入模式、cover/repaint 用法都列清楚了 —— 这是同类 skill 里少见的高质量参考文档。
2. **CLI 设计简洁**：`generate.sh` 的参数命名（`--lyrics` / `--duration` / `--instrumental` / `--batch`）直观、一致，符合 Unix CLI 习惯。
3. **base64 → MP3 自动落地**：脚本端处理掉了 base64 解码与文件分配（batch 时自动加序号后缀），用户无需手动 base64。
4. **零成本卖点清晰**：在 description 和 Notes 里反复强调"free API, no cost"，对用户决策有帮助。

这些都是后续整改时**应该保留**的好特性。
