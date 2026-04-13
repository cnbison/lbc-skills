# Slide Prompt Generation Guide

## Input
- `slides-plan.md`: 分页后的幻灯片内容（`## 第N页：标题` 格式）
- `style`: 用户选择的幻灯片风格（如 fengzikai, nb-pro 等）

## Output
生成给 `mofa_slides` 工具的 JSON 数组，每个元素是一页幻灯片的配置。

## Prompt 编写规则

### 封面页（第一页）
```json
{
  "prompt": "Cover slide. Title: \"<标题>\". Subtitle: \"<副标题>\". <风格描述>",
  "style": "cover"
}
```

### 内容页（第2~N-1页）
```json
{
  "prompt": "Content slide. Title at top: \"<页标题>\". Body content: <要点列表>. <风格描述>",
  "style": "normal"
}
```

### 结尾页（最后一页）
```json
{
  "prompt": "Closing slide. \"谢谢\" or \"感谢收听\" centered. Warm, elegant. <风格描述>",
  "style": "cover"
}
```

## 风格关键词（追加到 prompt 末尾）

| 风格 | 关键词 |
|------|--------|
| 水墨风 (fengzikai) | Chinese ink wash painting style, rice paper texture, black ink strokes, traditional Chinese aesthetic, 宣纸质感, 水墨晕染 |
| 商务 (nb-pro) | Professional, clean, dark background, subtle gradient, modern corporate |
| 学术 (what-is-life) | Academic, warm tones, research paper aesthetic, scholarly |
| 国潮 (agentic-enterprise-red) | Chinese guochao style, bold red and gold, traditional patterns with modern design |
| 科幻 (nb-br) | Cyberpunk, neon lights, dark background, futuristic |
| 暖色 (cc-research) | Warm amber tones, cinematic feel, cozy atmosphere |
| 可爱 (opensource) | Cute, cartoon whale mascot, colorful, playful |
| 极简 (nordic-minimal) | Minimalist, clean white, Scandinavian design, simple geometry |

## 分批生成策略

当幻灯片超过 15 页时，分批生成：

### 第1批：封面 + 前5页内容
```
slides 1-6 → deck-part1.pptx
```

### 第2批：中间内容页
```
slides 7-12 → deck-part2.pptx
```

### 第3批：剩余内容页 + 结尾
```
slides 13-N → deck-part3.pptx
```

### 合并提示
分批生成后，用以下命令提取所有幻灯片 PNG：
```bash
# 方法1: soffice 转换（如果可用）
soffice --headless --convert-to pdf deck-part1.pptx
pdftoppm -png -r 200 deck-part1.pdf slide

# 方法2: mofa-pptx-unpack 解压 OOXML 提取嵌入图片
python mofa-pptx-unpack deck-part1.pptx unpacked-part1/
# 查找 unpacked-part1/ppt/media/ 中的图片
```

## 注意事项

1. 封面和结尾页使用 `cover` variant，内容页使用 `normal` variant
2. 如果用户要求可编辑 PPT，添加 `"auto_layout": true`
3. prompt 中不要包含格式化提示（如 "24pt bold #FF0000"），只用自然语言描述
4. 每页内容控制在 5-7 个要点以内，避免文字过多
5. 图片分辨率建议使用 2K，平衡质量和速度
