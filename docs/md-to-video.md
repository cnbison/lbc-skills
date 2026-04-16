# md-to-video Skill 深度分析

> 分析日期: 2026-04-13
> 分析对象: skills/md-to-video/

---

## 一、结构概览

| 文件 | 用途 | 问题评级 |
|------|------|----------|
| `SKILL.md` | 核心技能定义（YAML frontmatter + 执行指令） | ⚠️ 中 |
| `README.md` | 用户文档（中文） | ✅ 正常 |
| `PIPELINE.md` | 技术参考（Agent 执行手册） | ⚠️ 中 |
| `templates/slide-prompts-guide.md` | 幻灯片 prompt 编写指南 | ✅ 正常 |
| `templates/video-script-template.md` | Markdown 脚本模板 | ✅ 正常 |

---

## 二、发现的错误

### 2.1 SKILL.md 版本声明缺失

**位置**: `SKILL.md:1-8`

**问题**: 缺少 `version` 和 `author` 字段，与其他 skill 不一致。

```yaml
# 当前
---
name: md-to-video
description: Convert a Markdown script...
always: false
---

# 建议
---
name: md-to-video
description: Convert a Markdown script...
version: 0.1.0
author: octos
always: false
---
```

### 2.2 路径处理不一致

**位置**: `SKILL.md:97` vs `PIPELINE.md:48`

**问题**: 输出路径写法不一致

- SKILL.md: `skill-output/md-to-video-<timestamp>/`
- PIPELINE.md: `skill-output/md-to-video-<timestamp>/`

**但**: README.md 写的是 `skill-output/md-to-video-<timestamp>/`

三者一致，无问题。

### 2.3 幻灯片分批数量矛盾

**位置**: `SKILL.md:93` vs `templates/slide-prompts-guide.md:51`

**问题**: 分批策略不一致

| 文件 | 说明 |
|------|------|
| `SKILL.md:93` | "最多 15 张/批" |
| `slide-prompts-guide.md:51` | "超过 15 页时分批" |
| `slide-prompts-guide.md:53-66` | 示例分批为 6页/批、6页/批、剩余 |

**矛盾**: 示例中第一批 6 页（封面+5内容），但叙述说最多 15 页。

**建议**: 统一为 "最多 15 页/批"，示例修正为合理分批。

### 2.4 PPTX 转 PNG 方法描述不清

**位置**: `SKILL.md:124-138`

**问题**: 三种方法的描述存在技术问题

```markdown
# SKILL.md 原文
方法1: mofa pptx-unpack 解压...
方法2: soffice 转换
方法3: Python python-pptx
```

**实际问题**:
1. `mofa pptx-unpack` 不存在，正确应为 `mofa-slides` 工具自带解压或手动解压
2. `soffice` 命令示例不完整，缺少 PDF 转 PNG 的具体命令
3. 方法3没有给出具体代码

**建议**: 给出确定可用的方案，例如:

```bash
# 方案1: LibreOffice (推荐)
soffice --headless --convert-to png --outdir slides/ordered/ deck.pptx

# 方案2: 直接复制 mofa_slides 生成的 PNG（如果工具支持）
```

### 2.5 ffmpeg 命令潜在问题

**位置**: `SKILL.md:155-171`

**问题1**: `-shortest` 和 `-t "$duration"` 同时使用是冗余的

```bash
# 当前（冗余）
ffmpeg -y -loop 1 -i slide-02.png -i page02.mp3 \
  -c:v libx264 -tune stillimage -c:a aac -b:a 192k \
  -pix_fmt yuv420p -shortest \
  -vf "..." \
  -t "$duration" segments/seg-02.mp4
```

`-shortest` 会在任意输入结束时停止，`-t` 指定时长，两者冲突时行为不确定。

**建议**: 只使用 `-t "$duration"`，去掉 `-shortest`。

**问题2**: 视频 scale 滤镜硬编码 1920x1080

```bash
-vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=black"
```

但配置选项中支持 `1K/2K/4K` 分辨率，硬编码导致配置无效。

---

## 三、优化建议

### 3.1 增加错误恢复机制

**当前**: 错误处理分散在各步骤描述中

**建议**: 增加统一的错误处理策略

```markdown
## 错误恢复检查点

- [ ] 步骤2失败（幻灯片生成）→ 重试或分批更小
- [ ] 步骤3失败（TTS）→ 跳过该页音频，使用静态时长
- [ ] 步骤5失败（片段合成）→ 单独重试失败片段
- [ ] 步骤6失败（合并）→ 检查缺失片段，重新生成
```

### 3.2 增加进度报告

**建议**: 长流程（5-15分钟）需要进度反馈

```markdown
每完成一个主要步骤，向用户报告：
- "✅ 幻灯片生成完成（14页）"
- "✅ 旁白生成完成（12段音频）"
- "🔄 正在合成视频片段（3/14）..."
```

### 3.3 分辨率配置实际生效

**当前**: `resolution` 选项在 SKILL.md:236 声明，但 ffmpeg 命令硬编码 1920x1080

**建议**: 根据配置动态计算 scale 参数

```bash
# 1K = 1280x720, 2K = 1920x1080, 4K = 3840x2160
case $resolution in
  1K) scale="1280:720" ;;
  2K) scale="1920:1080" ;;
  4K) scale="3840:2160" ;;
esac
```

### 3.4 音频格式统一

**位置**: `SKILL.md:118` vs `PIPELINE.md:59`

- SKILL.md: `page02.mp3` (MP3)
- PIPELINE.md: `segment_000.wav` (WAV)

**建议**: 统一使用 MP3（压缩率高，TTS 常用输出）或 AAC（ffmpeg 编码更高效）

### 3.5 增加字幕生成功能

**扩展建议**: 当前未支持字幕，可考虑增加

```markdown
### Step X: 生成字幕（可选）

使用 TTS 的文本对齐功能生成 SRT:
- 每个 content page 对应一个字幕块
- 时间戳从音频时长计算
```

### 3.6 背景音乐（BGM）支持

**位置**: `README.md:156` 标记为 TODO

**实现建议**:

```bash
# 在片段合成时混合BGM
ffmpeg -i slide.png -i narration.mp3 -i bgm.mp3 \
  -filter_complex "[1:a][2:a]amix=inputs=2:duration=first:dropout_transition=3[a]" \
  -map 0:v -map "[a]" -c:v libx264 -c:a aac -shortest output.mp4
```

### 3.7 转场效果

**位置**: `README.md:157` 标记为 TODO

**实现建议**: 使用 ffmpeg xfade 滤镜

```bash
# 片段之间添加淡入淡出
ffmpeg -i seg1.mp4 -i seg2.mp4 \
  -filter_complex "xfade=transition=fade:duration=1:offset=END" \
  -c:v libx264 -c:a aac output.mp4
```

---

## 四、文档一致性问题

### 4.1 触发词列表

**SKILL.md:4**:
```
Triggers: md to video, markdown to video, markdown视频...
```

**README.md:50-53**: 示例写法与 SKILL.md 不完全匹配

**建议**: 统一触发词列表，放入 SKILL.md 的 description 中作为规范来源。

### 4.2 风格名称

**SKILL.md:36**: 默认风格 `fengzikai`（中文内容）或 `nb-pro`（英文）

**slide-prompts-guide.md:38-47**: 列出 8 种风格映射

**问题**: `fengzikai` 不在风格列表中？

检查表格: `fengzikai` 是 `水墨风` 的 key，但表格第一列是"风格"，第二列是"关键词"。

**建议**: 明确 key 和显示名称的对应关系。

### 4.3 声音名称

**SKILL.md:37**: 默认 `vivian`（中文）/ `ryan`（英文）

**README.md:101**: 提到 `douwentao`（窦文涛）

**问题**: 声音列表未在文档中完整列出，用户不知道有哪些可用选项。

**建议**: 增加 `fm_voice_list` 工具调用的说明，或列出常用声音。

---

## 五、代码/逻辑问题

### 5.1 concat 文件生成命令

**位置**: `SKILL.md:188-191`

```bash
for f in segments/seg-*.mp4; do
  echo "file '$PWD/$f'" >> concat.txt
done
```

**问题**:
1. 使用 `$PWD` 可能导致路径含空格时出错
2. 未排序，seg-10.mp4 可能在 seg-2.mp4 之前
3. 每次追加到 concat.txt，如果文件已存在会累积

**建议**:

```bash
# 清空并重新生成
> concat.txt
for f in $(ls -v segments/seg-*.mp4); do
  printf "file '%s'\n" "$f" >> concat.txt
done
```

### 5.2 封面/结尾时长配置未生效检查

**位置**: `SKILL.md:232-239`

配置了 `cover_duration` 和 `end_duration`，但需确保 ffmpeg 命令使用这些变量而非硬编码。

### 5.3 音频缺失回退逻辑不完整

**位置**: `SKILL.md:248`

```markdown
Missing audio file: Generate a silent segment with extended duration
(use slide text length to estimate)
```

**问题**: "slide text length" 未定义如何计算时长。

**建议**: 明确算法，例如:

```bash
# 估算：中文字符数 / 3 = 秒数（假设每秒3个字）
# 或固定 5 秒静态展示
```

---

## 六、安全与健壮性

### 6.1 路径注入风险

**位置**: 多处使用 `$f` 或用户输入路径

**问题**: 如果文件名包含特殊字符（如 `'; rm -rf /; '`），直接嵌入命令有风险。

**建议**: 所有路径变量使用引号包裹 `"$variable"`

### 6.2 目录创建检查

**建议**: 明确创建输出目录的步骤

```bash
mkdir -p skill-output/md-to-video-${timestamp}/{slides,audio,segments}
```

### 6.3 磁盘空间检查

**建议**: 视频生成占用空间大，开始前检查可用空间

```bash
df -h . | awk 'NR==2 {if($4<1024) print "警告：磁盘空间不足"}'
```

---

## 七、优先级修复清单

| 优先级 | 问题 | 文件 | 修复难度 |
|--------|------|------|----------|
| 🔴 高 | ffmpeg `-shortest` 与 `-t` 冗余冲突 | SKILL.md:167 | 简单 |
| 🔴 高 | 分辨率配置硬编码不生效 | SKILL.md:157-170 | 中等 |
| 🔴 高 | concat 文件生成未排序 | SKILL.md:188-191 | 简单 |
| 🟡 中 | 版本/作者字段缺失 | SKILL.md:1-8 | 简单 |
| 🟡 中 | PPTX 转 PNG 方法描述不清 | SKILL.md:124-138 | 中等 |
| 🟡 中 | 分批数量示例与说明矛盾 | slide-prompts-guide.md:53-66 | 简单 |
| 🟢 低 | 增加进度报告 | 新增 | 中等 |
| 🟢 低 | 字幕生成支持 | README.md TODO | 困难 |
| 🟢 低 | BGM 支持 | README.md TODO | 中等 |

---

## 八、附录：修复后的关键代码片段

### 8.1 修复后的 ffmpeg 内容片段命令

```bash
# 获取分辨率配置
RESOLUTION="${resolution:-2K}"
case "$RESOLUTION" in
  1K) SCALE="1280:720" ;;
  2K) SCALE="1920:1080" ;;
  4K) SCALE="3840:2160" ;;
  *)  SCALE="1920:1080" ;;
esac

# 生成片段（内容页）
for i in $(seq -f "%02g" 2 $((total_pages-1))); do
  slide="slides/ordered/slide-${i}.png"
  audio="audio/page${i}.mp3"
  output="segments/seg-${i}.mp4"

  if [[ -f "$audio" ]]; then
    duration=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$audio")
    ffmpeg -y -loop 1 -i "$slide" -i "$audio" \
      -c:v libx264 -tune stillimage -c:a aac -b:a 192k \
      -pix_fmt yuv420p \
      -vf "scale=${SCALE}:force_original_aspect_ratio=decrease,pad=${SCALE}:(ow-iw)/2:(oh-ih)/2:color=black" \
      -t "$duration" "$output"
  else
    # 音频缺失，使用 5 秒静态
    ffmpeg -y -loop 1 -i "$slide" \
      -c:v libx264 -tune stillimage -pix_fmt yuv420p \
      -vf "scale=${SCALE}:force_original_aspect_ratio=decrease,pad=${SCALE}:(ow-iw)/2:(oh-ih)/2:color=black" \
      -t 5 -an "$output"
  fi
done
```

### 8.2 修复后的 concat 文件生成

```bash
# 生成排序后的 concat 列表
> concat.txt
for f in $(ls -v segments/seg-*.mp4 2>/dev/null); do
  printf "file '%s'\n" "$f" >> concat.txt
done

# 验证文件存在且非空
if [[ ! -s concat.txt ]]; then
  echo "错误：没有可合并的视频片段"
  exit 1
fi

# 合并
ffmpeg -y -f concat -safe 0 -i concat.txt \
  -c:v libx264 -preset medium -crf 23 \
  -c:a aac -b:a 192k \
  -movflags +faststart \
  final_video.mp4
```

---

## 九、总结

**总体评价**: md-to-video skill 设计合理，流程清晰，文档较完整。但存在以下需要关注的问题:

1. **功能性问题**: 分辨率配置不生效、concat 未排序
2. **技术错误**: ffmpeg 命令参数冗余
3. **文档不一致**: 分批策略示例矛盾、PPTX 转换方法不清
4. **健壮性**: 缺少错误恢复检查点和进度报告

建议按优先级清单逐步修复，特别是标记为 🔴 高的三项，它们直接影响视频生成的正确性。
