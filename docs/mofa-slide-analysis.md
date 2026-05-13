# mofa-slides 深度技术分析

> 分析日期：2026-04-13  
> 分析对象：mofa-slides v0.4.5（ skill + mofa-cli 核心实现）

---

## 1. 产品定位与核心能力

**mofa-slides** 是一个 AI 原生（AI-Native）的演示文稿生成引擎，其核心价值主张是：

- **全出血（full-bleed）AI 生成图像**：每一页幻灯片的背景都是 Gemini 生成的 1920×1080 16:9 图像。
- **四模式工作流**：从"纯图像不可编辑"到"全自动可编辑 PPTX"，覆盖不同场景对"美观度"与"可编辑性"的权衡。
- **风格即配置（Style-as-Config）**：通过 20+ TOML 风格模板，将设计系统（配色、字体、插画风格）从 prompt 中抽离，实现可复用、可扩展。

它不只是"把 Markdown 转成 PPT"，而是一个**图像优先（image-first）**的生成式演示引擎。

---

## 2. 架构总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         mofa-slides 架构                             │
├─────────────────────────────────────────────────────────────────────┤
│  用户输入 (JSON/JS)  →  风格选择 (TOML)  →  模式路由                 │
│                            ↓                                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │ Mode 1      │  │ Mode 2      │  │ Mode 3/4    │                 │
│  │ Image-only  │  │ Clean BG +  │  │ Auto-layout │                 │
│  │ (默认)       │  │ manual text │  │ (VQA驱动)   │                 │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                 │
│         │                │                │                         │
│         └────────────────┴────────────────┘                         │
│                          ↓                                          │
│              Gemini API (图像生成) / Batch API                       │
│                          ↓                                          │
│         ┌────────────────┴────────────────┐                        │
│         │                                 │                        │
│    直接输出 PNG                    VQA 文本提取                      │
│         │                        (Gemini Vision / DeepSeek OCR)    │
│         │                                 ↓                        │
│         │                         Qwen-Edit 去文字                   │
│         │                                 ↓                        │
│         └────────────────┬────────────────┘                        │
│                          ↓                                          │
│              PPTX 组装 (Rust 手写 OOXML)                             │
│                          ↓                                          │
│              skill-output/xxx/slides.pptx                           │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.1 技术栈分层

| 层级 | 技术 | 说明 |
|------|------|------|
| **CLI 核心** | Rust (`mofa-cli`) | 并发调度、API 调用、PPTX 组装、缓存管理 |
| **图像生成** | Gemini 3.1 Flash Image Preview | 默认模型，支持参考图、batch API |
| **文本移除** | Dashscope `qwen-image-edit-max` | 通过 inpainting 移除文字，保留插图 |
| **OCR 增强** | DeepSeek OCR (可选) | 提供更高精度的文本 bbox |
| **PPTX 工具链** | Python (`python-pptx`) + JS (`pptxgenjs`) | 后处理脚本：替换、重排、缩略图、清单 |
| **OOXML 底层** | Python (`lxml`, `defusedxml`) | 解包/打包/校验 PPTX 的 XML |

---

## 3. 四大生成模式深度解析

### Mode 1: Image-only（默认）
- **机制**：`prompt + style_prefix + ANTI_LEAK_RULES` → Gemini 生成完整 slide 图像 → 直接插入 PPTX。
- **特点**：最快、最美、但文字不可编辑。
- **适用**：终稿、打印、视觉冲击优先。

### Mode 2: Clean Background + Manual Text Overlay（推荐）
- **机制**：
  1. prompt 追加 `NO_TEXT_INSTRUCTION`（"DO NOT render any text"）。
  2. Gemini 生成**无文字的纯净背景图**。
  3. 用户通过 `texts` 数组指定精确的文本框位置、字体、颜色、对齐方式。
  4. Rust 代码直接写 OOXML，将文本框叠加到背景图上。
- **特点**：可编辑、无需 VQA/去文字、速度最快、质量最可控。
- **关键代码**：`mofa-cli/src/pptx.rs` 中 `build_text_shape_xml()` 手写 `<p:sp>` 形状 XML。

### Mode 3: Auto-layout（全自动可编辑）
- **机制**：4 阶段流水线
  1. **Generate**：生成带文字的参考图 (`slide-XX-ref.png`)
  2. **Extract**：VQA（Gemini Vision）读取图像，提取每个文本元素的内容、位置、字号、颜色、粗细。可选 DeepSeek OCR 做混合增强。
  3. **Remove**：Qwen-Edit 将参考图上的所有文字擦除，生成干净背景 (`slide-XX.png`)
  4. **Assemble**：用提取到的文本布局叠加到干净背景上，输出 PPTX。
- **特点**：全自动、最慢、对 VQA 精度敏感。

### Mode 4: PDF-to-PPTX
- **机制**：跳过 AI 生成，直接提供 `source_image` + `auto_layout: true`，走 Mode 3 的 Extract → Remove → Assemble 阶段。
- **特点**：将现有 PDF/图片转成可编辑 PPTX。

---

## 4. 核心代码文件分析

### 4.1 `mofa-cli/src/pipeline/slides.rs` — 主调度器

这是整个 slides 功能的"心脏"，约 800+ 行 Rust 代码，承担以下职责：

#### 4.1.1 缓存系统（精巧的指纹机制）
```rust
fn generation_fingerprint(prompt: &str, image_size: Option<&str>, model: &str, ref_images: &[&Path]) -> String
fn edit_fingerprint(source_image: &Path, prompt: &str, model: &str) -> String
```
- 使用 **SHA-256** 对 prompt、模型、尺寸、参考图路径+修改时间计算指纹。
- 生成结果旁会写一个隐藏的 `.slide-XX.mofa-cache.json` 文件记录指纹。
- **中断恢复能力**：如果某次调用超时，下次 rerun 时只有缺失的 slide 会重新生成。
- 对 Qwen-Edit 的去文字结果也做缓存，避免重复付费。

#### 4.1.2 并发模型
- 使用 **`rayon` ThreadPool** 控制并发度（默认 5 线程）。
- 图像生成阶段是**纯并行**的（I/O 密集型，受 API rate limit 约束）。
- VQA 提取和 Qwen-Edit 阶段在 sync 模式下是**顺序执行**的（依赖外部 API，且互相独立但无法并行化同一种 API 的串行调用）。

#### 4.1.3 Batch API 支持
- 当用户传入 `--api batch` 时，会预先将所有 slide 的生成请求收集为 `Vec<BatchImageRequest>`，调用 `gemini.batch_gen_images()`。
- Batch 模式下成本降低 **50%**，但耗时 5-30 分钟。
- 如果 Batch 失败，有**优雅降级**：自动 fallback 到 `run_slides_sync()`。

### 4.2 `mofa-cli/src/layout.rs` — VQA 布局提取引擎

这是 Auto-layout 模式的核心算法模块。

#### 4.2.1 Gemini VQA Prompt 工程
布局提取的 prompt 设计非常精细：
- 提供**空间网格参考**（Top quarter / Upper-mid / Lower-mid / Bottom quarter）帮助模型做空间推理。
- 要求返回 **百分比坐标** (`xPct`, `yPct`, `wPct`, `hPct`)，天然适配不同分辨率的缩放。
- 强调 `yPct` 必须是**文本顶部**（topmost pixel touching ascenders），这是为了避免模型将 bbox 中心误报为顶部。
- 强调 `wPct` 必须使用**容器宽度**而非文字本身的 tight width。

#### 4.2.2 后处理算法
提取出的原始坐标会经过多层修正：

1. **`fix_bbox_from_font_size()`**：VQA 给出的高度常被严重低估（~2.9x），该函数根据字号反推合理高度。
2. **`normalize_font_face()`**：统一字体名称映射。
3. **`rescale_x_positions()`** / **`align_columns()`**：检测并修正列对齐。
4. **标题/页脚特殊处理**：如果文本靠近顶部或底部（0.3 英寸内），或居中且宽度超过 40%，则强制拉满全宽。

#### 4.2.3 Refine 反馈回路
```rust
pub fn refine_text_layout(...)
```
这是一个**视觉-语言闭环**设计：
1. 在参考图上用彩色矩形画出当前提取的 bbox。
2. 将带标注的图再次发给 Gemini Vision。
3. Prompt 要求模型检查每个框是否准确覆盖真实文本，并返回修正后的坐标。
4. 修正结果回写 overlay 数组。

这个设计非常聪明——它把"布局验证"问题转化为了另一个 VQA 问题。

### 4.3 `mofa-cli/src/pptx.rs` — OOXML 组装器

Rust 代码直接拼接 OOXML（Office Open XML）字符串来构建 PPTX，不依赖任何外部库。

#### 4.3.1 手写 XML 的能力边界
- **文本框**：支持 `text` 纯文本、`runs` 富文本（同一文本框内混排颜色/字号/粗斜体）。
- **样式属性**：`fill`（背景色+透明度）、`shadow`（外阴影）、`margin`（内边距）、`lineSpacing`（行距）、`rotate`（旋转）。
- **图片叠加**：支持 `overlay_images`（如 Logo）。
- **字体回退**：每个文本 run 都会生成 `<a:latin>`, `<a:ea>`, `<a:cs>` 三种字体声明，确保中西文兼容。

#### 4.3.2 关键常量
```rust
const EMU_PER_INCH: f64 = 914_400.0;  // Office 使用的英制单位
const PT_TO_HPTS: f64 = 100.0;        // 字号单位：1pt = 100 半磅
```
- 幻灯片画布固定为 **13.333" × 7.5"**（16:9 宽屏），所有坐标以英寸输入。

### 4.4 `mofa-cli/src/style.rs`（推断）+ `styles/*.toml`

风格系统采用 **TOML 文件**描述，每个风格包含：
- `meta`：名称、显示名、分类、标签
- `variants`：多个变体（cover / normal / data / warm 等）
- 每个 variant 是一个完整的 prompt 模板

**设计优点**：
- 风格与业务代码完全解耦，新增风格只需添加 TOML 文件。
- Prompt 中详细规定了分辨率、配色 HEX 值、字体、留白比例、插画风格，确保 Gemini 输出的一致性。

**代表风格举例**：
| 风格 | 视觉特征 | 适用场景 |
|------|----------|----------|
| `nb-pro` | 淡紫渐变、细线框、 corporate | 商务通用 |
| `fengzikai` | 丰子恺水墨、宣纸纹理、留白 | 人文、艺术 |
| `what-is-life` | 深蓝/薰衣草、青色线框科学图标 | 学术、科研 |
| `vlinka-dji` | 深灰渐变、产品渲染、青色轮廓光 | 产品发布 |
| `multi-brand` | 多公司品牌色（Amazon/Google/NVIDIA 等） | 竞品分析 |

---

## 5. Python/JS 工具链分析

`mofa-slides/pptx-scripts/` 和 `ooxml/scripts/` 提供了一套**后处理/诊断/修复**工具箱：

### 5.1 `inventory.py`
- 提取 PPTX 中所有文本内容、形状位置、段落格式。
- 支持递归解析 `GroupShape`，计算绝对坐标。
- 可检测文本溢出和形状重叠问题。

### 5.2 `replace.py`
- 基于 inventory 的 JSON 输出做批量文本替换。
- 保留原有格式（bullet、alignment、spacing）。

### 5.3 `rearrange.py`
- 按索引重新排序、复制幻灯片。
- 实现了 `duplicate_slide()`，正确处理图片 relationship（`rId` / `blip` 引用更新）。

### 5.4 `thumbnail.py`
- 将 PPTX 转为缩略图网格（PDF 中间态 → PIL 拼接）。

### 5.5 `html2pptx.js`
- 使用 **Playwright + sharp + pptxgenjs** 将 HTML 页面精确转为 PPTX。
- 支持 CSS 渐变、边框、边距的映射。
- 这是**无模板新建 PPT** 的备用工作流（与 AI 生成模式平行）。

### 5.6 `ooxml/scripts/`
- `unpack.py`：将 PPTX 解压为 XML，并 pretty-print。
- `pack.py`：将目录重新打包为 PPTX，自动去除 pretty-print 的空白节点，可选 `soffice` 校验。
- `validate.py`：XSD Schema 校验 + tracked changes 校验（针对 docx）。

---

## 6. 关键设计决策与工程亮点

### 6.1 Anti-leak Rules
所有生成 prompt 会自动追加 `ANTI_LEAK_RULES`，防止 Gemini 将"设计系统说明"（如"font size 24pt, hex #C7000B"）当成字面内容渲染到图上。这是图像生成式 PPT 的**常见故障模式**的一个优雅解决方案。

### 6.2 参考图尺寸分离
```rust
--image-size 2K      // 最终输出分辨率
--ref-image-size 1K  // VQA 参考图分辨率（可更低以加速）
```
- 在 Auto-layout 模式下，VQA 不需要 4K 图也能准确识别文字位置。这一分离显著降低了 API 成本和时间。

### 6.3 模式优先策略
代码中 `texts`（用户手动指定）> `auto_layout`（VQA 提取）：
```rust
let texts = if slides[i].texts.is_some() {
    slides[i].texts.clone().unwrap_or_default()
} else if slides[i].auto_layout {
    extracted[i].clone().unwrap_or_default()
} else {
    Vec::new()
};
```
- 这给用户提供了"半自动"能力：可以启用 auto_layout，但对某一页手动覆盖文本位置。

### 6.4 路由策略：Gemini vs Dashscope
```rust
if model.starts_with("qwen-image") {
    // 走 Dashscope
} else {
    // 走 Gemini
}
```
- 通过模型名前缀动态选择后端，支持未来接入更多图像生成服务商。

---

## 7. 潜在问题与改进空间

### 7.1 VQA 精度瓶颈
- Auto-layout 的质量严重依赖 Gemini Vision 对文字位置、字号、颜色的判断。
- 复杂布局（如重叠文本、艺术字、极小号文字）仍可能提取失败。
- **建议**：增加一个"布局校验层"，在 refine 后计算文本框与背景图的语义分割 IoU，低置信度时告警。

### 7.2 Qwen-Edit 的可用性依赖
- Dashscope API key 是 auto-layout 的强依赖。如果用户只有 Gemini key，则无法完成文字移除。
- **当前 fallback**：只是打印 warning，不会尝试用 Gemini image editing 替代（虽然 SKILL.md 文档提到有这个 fallback，但代码中 sync path 的 text removal 只有 `if let Some(ref ds) = dashscope` 分支）。
- **建议**：增加 Gemini image editing 作为 fallback，保证单 key 可用性。

### 7.3 中文字体支持有限
- `pptx.rs` 中字体回退逻辑虽然写了 `<a:ea>` 标签，但实际字体名还是依赖用户本地安装（如 "Microsoft YaHei"）。
- 如果目标机器没有安装对应字体，PPTX 会 fallback 到系统默认字体，导致排版错位。
- **建议**：在生成阶段将中文字体子集化嵌入到 PPTX 的 `ppt/fonts/` 目录中。

### 7.4 并发与 API 限流
- `rayon` 的线程池大小固定，但没有对 Gemini API 做 rate limit 自适应退避（exponential backoff）。
- 在高并发+大尺寸图片时，可能触发 API 429 错误。
- **建议**：在 API client 层增加带 jitter 的 retry 逻辑。

### 7.5 Batch 模式不支持 source_image
```rust
if slide.source_image.is_some() { continue; }
```
- Batch 模式下，提供 `source_image` 的 slide 会被跳过生成（这是对的），但后续 batch 结果映射时这部分逻辑需要 `ref_paths_vec` 单独回填。目前代码已处理，但流程上略显割裂。

### 7.6 缺少实时进度反馈
- 当前只有 `eprintln!` 在 stderr 输出进度。
- 对于 15+ 页、batch 模式等长耗时任务，上层 UI（如 Telegram）无法获取中间进度。
- **建议**：输出结构化 JSON Lines 日志或支持 SSE/WebSocket 进度推送。

---

## 8. 目录结构与文件索引

```
mofa-slides/
├── SKILL.md                    # AI Agent 调用规范（模式、参数、示例）
├── USER_GUIDE.md               # 面向用户的完整使用指南
├── manifest.json               # Skill 注册清单（tools schema + 二进制下载地址）
├── requirements.txt            # Python 依赖：python-pptx, markitdown, lxml
├── architecture.dot            # Graphviz 架构图源文件
├── docs/
│   └── 00-SKILL-完整制作指南.md
├── styles/                     # 20 个 TOML 风格模板
│   ├── nb-pro.toml
│   ├── what-is-life.toml
│   ├── fengzikai.toml
│   ├── vlinka-dji.toml
│   └── ...
├── pptx-scripts/               # 后处理/诊断/转换脚本
│   ├── html2pptx.js            # HTML → PPTX（Playwright + pptxgenjs）
│   ├── inventory.py            # 文本/布局清单提取
│   ├── replace.py              # 批量文本替换
│   ├── rearrange.py            # 幻灯片重排/复制
│   └── thumbnail.py            # 缩略图网格生成
└── ooxml/scripts/              # OOXML 底层操作
    ├── unpack.py               # PPTX 解压 + XML 格式化
    ├── pack.py                 # 目录打包回 PPTX + soffice 校验
    ├── validate.py             # XSD Schema 校验入口
    └── validation/             # 各文件类型的校验器实现

mofa-cli/src/                   # Rust CLI 核心
├── pipeline/slides.rs          # Slide 生成主流程（调度、缓存、Batch）
├── layout.rs                   # VQA 文本提取 + refine 修正算法
├── pptx.rs                     # OOXML 手写组装器
├── gemini.rs                   # Gemini API 客户端（生成/VQA/Batch）
├── dashscope.rs                # Dashscope API 客户端（Qwen-Edit）
├── deepseek_ocr.rs             # DeepSeek OCR 客户端
├── style.rs                    # TOML 风格加载
├── config.rs                   # 配置文件解析
└── main.rs                     # CLI 入口
```

---

## 9. 总结

mofa-slides 是一个**工程完成度很高**的 AI 生成式演示引擎。其最突出的设计特点是：

1. **多模式架构**精准匹配了不同场景下"美观 vs 可编辑"的 trade-off。
2. **缓存与指纹机制**让中断恢复和重复调用变得经济可靠。
3. **VQA + Refine 闭环**是Auto-layout模式的灵魂，将几何问题巧妙地转化为了视觉语言问题。
4. **Rust 手写 OOXML**虽然看起来"脏"，但避免了引入重型依赖，保证了跨平台部署的简洁性。

如果要在生产环境中大规模使用，最值得投入的方向是：
- **增强 VQA  fallback 和校验**（降低 auto-layout 的坏case率）
- **字体嵌入**（解决中文字体跨设备兼容）
- **API 限流与进度反馈**（提升长任务的用户体验）

---
*分析完成*
