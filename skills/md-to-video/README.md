# md-to-video Skill

将 Markdown 文稿自动转换为带旁白的视频。

## 功能

- 📄 **Markdown 解析**：自动识别封面、内容页、结尾页
- 🎨 **AI 幻灯片生成**：使用 mofa-slides 生成水墨风/商务风等风格的幻灯片
- 🔊 **TTS 旁白**：使用 fm_tts 为每页生成语音旁白
- 🎬 **视频合成**：使用 ffmpeg 将幻灯片和音频合成为 MP4 视频

## 安装

```bash
# 通过 skill-store 安装（推荐）
manage_skills(action="install", repo="mofa-org/mofa-skills/md-to-video")

# 或手动安装
# 将整个 skill-output/md-to-video 目录复制到 ~/.octos/profiles/<profile>/data/skills/
```

## 依赖

- `mofa-slides` skill（AI 幻灯片生成）
- `mofa-fm` skill（TTS 语音合成）
- `ffmpeg`（视频合成）
- `GEMINI_API_KEY`（幻灯片图像生成）

## 使用方法

### 1. 准备 Markdown 文稿

```markdown
## Page 1: 封面
概念解剖——道
中国哲学核心概念深度解读

## Page 2: 开场
朋友们，今天咱们来聊聊中国哲学里最核心的一个词——"道"。

## Page 3: 定锚
道是宇宙的根儿，是万事万物运行的规矩。

## Page 10: 结尾
道不远人，人自远之。
```

### 2. 触发技能

用户消息示例：
- "帮我把这个 markdown 做成视频"
- "用这个脚本生成一个讲解视频"
- "md to video with 水墨风格 and 窦文涛声音"

### 3. 技能执行流程

1. **解析文稿**：识别页数、封面、内容、结尾
2. **生成幻灯片计划**：展示给用户确认
3. **生成幻灯片**：调用 `mofa_slides`（分批，最多 15 页/批）
4. **生成旁白音频**：调用 `fm_tts` 为每页生成语音
5. **提取幻灯片 PNG**：从 PPTX 提取为有序 PNG
6. **合成视频片段**：ffmpeg 将每页幻灯片 + 音频合成为 MP4 片段
7. **合并所有片段**：ffmpeg concat 生成最终视频
8. **发送视频**：使用 `send_file` 交付给用户

### 4. 输出

```
skill-output/md-to-video-<timestamp>/
├── script.md           # 原始 Markdown
├── slides-plan.md      # 幻灯片计划
├── slides/
│   ├── deck-part1.pptx # 生成的 PPTX（可能分批）
│   └── ordered/        # 提取的 PNG 幻灯片
├── audio/              # 旁白 MP3 文件
├── segments/           # 视频片段
└── final_video.mp4     # 最终视频
```

## 配置选项

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `style` | `fengzikai` | 幻灯片风格（水墨风） |
| `voice` | `vivian` | TTS 旁白声音 |
| `resolution` | `2K` | 幻灯片分辨率（1K/2K/4K） |
| `cover_duration` | `5` | 封面静态时长（秒） |
| `end_duration` | `3` | 结尾静态时长（秒） |
| `video_resolution` | `1920x1080` | 输出视频分辨率 |

## 示例

### 示例 1：中文哲学讲解（水墨风）

```
用户：帮我把这个《道德经》讲解做成视频，用水墨风格，声音要像窦文涛
```

执行：
- 风格：`fengzikai`（水墨风）
- 声音：`douwentao`（需先克隆）
- 输出：7 分钟水墨风格讲解视频

### 示例 2：英文技术分享（商务风）

```
用户：Convert this tech talk markdown to a video, use professional style
```

执行：
- 风格：`nb-pro`（商务风）
- 声音：`ryan`（英文男声）
- 输出：10 分钟技术分享视频

## 故障排除

### 问题：幻灯片生成超时

**原因**：超过 15 页，单次调用超时

**解决**：分批生成（每批最多 15 页），然后合并

### 问题：ffmpeg 找不到文件

**原因**：路径包含 URL 编码字符（如 `%3A`）

**解决**：使用相对路径或 `realpath` 解析真实路径

### 问题：音频文件缺失

**原因**：TTS 生成失败

**解决**：重试 `fm_tts`，或跳过该页使用静态时长

### 问题：视频无法播放

**原因**：concat 文件包含不存在的片段

**解决**：检查所有 `segments/seg-*.mp4` 是否存在，重新生成缺失的片段

## 开发笔记

### 添加新风格

在 `templates/slide-prompts-guide.md` 中添加风格关键词映射。

### 自定义旁白声音

用户可先用 `fm_voice_save` 克隆声音，然后在技能中引用：
```
voice: "clone:douwentao"
```

### 扩展功能

- [ ] 支持背景音乐（BGM）
- [ ] 支持转场动画
- [ ] 支持字幕生成
- [ ] 支持多语言混合

## 许可证

MIT License
