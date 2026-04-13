# 🎯 任务目标

将一份关于 "OpenClaw (小龙虾AI)" 的 Markdown 文档，转化为带 AI 旁白克隆声音的演示视频。

---

## 📋 完整工作流程

### 第一阶段：阅读原始文档 & 规划

- **工具**：`read_file`
- 读取你提供的 Markdown 源文件，理解内容结构和要点
- 规划 PPT 页数、每页内容摘要、旁白脚本

### 第二阶段：生成演示幻灯片（PPT）

- **工具**：`mofa_slides`
- **技能**：`mofa-slides`（AI 全屏幻灯片生成）
- 根据文档内容编写每页 slide 的 prompt
- 使用 nb-pro 商务风格，Mode 1（图片模式，文字烘焙在 AI 图片中）
- 每页包含标题、关键要点、配图描述
- 输出 `slides.pptx` + 中间图片 `imgs/` 目录

### 第三阶段：克隆声音（如需）

- **工具**：`fm_voice_save` / `fm_voice_list`
- **技能**：`mofa-fm`（TTS 和声音克隆）
- 从你提供的音频样本中保存自定义声音 `douwentao`

### 第四阶段：生成旁白音频

- **工具**：`fm_tts`
- **技能**：`mofa-fm`
- 将每页 PPT 对应的旁白文字合成为语音
- 使用克隆声音 `douwentao`
- 输出多个音频片段

### 第五阶段：合成视频

- **工具**：`shell`（调用 ffmpeg）
- 将幻灯片图片转为视频帧
- 每张幻灯片搭配对应的旁白音频
- 自动计算每张幻灯片的持续时间 = 对应旁白时长
- 交叉淡入淡出转场效果
- 拼接所有片段为完整视频
- 最终合成 `final-video.mp4` / `final-video-v2.mp4`

---

## 🧰 调用的技能 & 工具汇总

| 阶段 | 技能 (Skill) | 工具 (Tool) |
|------|-------------|-------------|
| 读取文档 | — | `read_file` |
| 生成 PPT | `mofa-slides` | `mofa_slides` |
| 声音克隆 | `mofa-fm` | `fm_voice_list`, `fm_voice_save` |
| 旁白合成 | `mofa-fm` | `fm_tts` |
| 视频合成 | — | `shell`（ffmpeg） |
| 发送文件 | — | `send_file` |

---

## 📂 产出文件结构

```
openclaw-presentation-20260412/
├── imgs/              # AI 生成的幻灯片图片
├── slides.pptx        # 可编辑的 PPT 文件
├── video-part1.mp4    # 视频前半段（中间产物）
├── video-part2.mp4    # 视频后半段（中间产物）
├── final-video.mp4    # 最终视频 v1
└── final-video-v2.mp4 # 最终视频 v2（完整旁白版）
```

> **说明**：整个流程是文档 → PPT → 音频 → 视频的线性管线，核心依赖 `mofa-slides` 生成视觉内容，`mofa-fm` 生成声音，`ffmpeg` 完成最终合成。

## 例子
把我给你的md文档，划分为两段，生成两张slides图片，两段文档内容用窦文涛音色生成两段旁白音频，然后基于两张图片和旁白语音合成演说视频。
