# Nuwa-Skill 深度分析报告

> 评估框架：Skill-Creator 五维评估体系（Anatomy / Progressive Disclosure / Triggering / Writing Quality / Testability）
> 评估对象：`skills/nuwa-skill/SKILL.md` 及其附属文件
> 评估日期：2026-05-20
> 评估版本：nuwa-skill v0.1.0（基于 SKILL.md 当前状态）

---

## 一、概述：Nuwa-Skill 是什么

Nuwa-Skill 是一个**元 Skill（Meta-Skill）**——它不直接解决用户问题，而是**生成其他 Skill**。其核心能力是将一个真实人物或模糊需求，通过多 Agent 并行调研、框架提炼、质量验证，最终输出一个可运行的 "人物视角 Skill"（如 `feynman-perspective`）。

---

## 二、Anatomy 结构分析

### 2.1 目录结构

```
skills/nuwa-skill/
├── SKILL.md                              # 主指令文件（645行）
├── references/
│   ├── skill-template.md                 # 人物 Skill 模板
│   └── extraction-framework.md           # 心智模型提炼方法论
├── scripts/
│   ├── download_subtitles.sh             # YouTube 字幕下载
│   ├── srt_to_transcript.py              # 字幕清洗为纯文本
│   ├── merge_research.py                 # 调研结果合并摘要
│   └── quality_check.py                  # 质量自检脚本
└── examples/                             # 14 个已生成的人物 Skill
    ├── feynman-perspective/
    ├── munger-perspective/
    ├── zhangxuefeng-perspective/
    └── ... (共14个)
```

**评估**：

| 维度 | 评分 | 说明 |
|------|------|------|
| 文件组织 | ★★★★★ | 结构清晰，reference/script/example 分离合理 |
| 模板完整性 | ★★★★★ | `skill-template.md` 和 `extraction-framework.md` 提供了完整的构建方法论 |
| 示例丰富度 | ★★★★★ | 14 个示例覆盖中西、古今、不同领域，极具参考价值 |
| 脚本工具 | ★★★★☆ | 4 个脚本覆盖了字幕下载、清洗、合并、质检，但缺少 Agent 调用的自动化封装 |

### 2.2 SKILL.md 结构拆解

| Section | 行数 | 内容 |
|---------|------|------|
| Frontmatter | 1-8 | name, description, 触发词 |
| 核心理念 | 14-26 | "女娲不是复制人，是提炼思维框架" |
| Phase 0: 入口分流 | 29-138 | 直接路径(0A) + 诊断路径(0B) + 目录创建(0.5) |
| Phase 1: 多源采集 | 173-311 | 6 个并行 Agent 的分配、prompt 模板、信息源策略 |
| Phase 1.5: 调研 Review | 314-338 | 检查点表格，要求用户确认 |
| Phase 2: 框架提炼 | 341-409 | 心智模型(3-7个)、决策启发式(5-10条)、表达 DNA、价值观、智识谱系、诚实边界 |
| Phase 2.5: 提炼确认 | 393-409 | 摘要展示，用户确认 |
| Phase 3: Skill 构建 | 413-501 | 读取模板 → 填充内容 → Agentic Protocol → 质量自检 → 输出 |
| Phase 4: 质量验证 | 505-538 | 已知测试 + 边缘测试 + 风格测试 + 通过标准表格 |
| Phase 5: 双 Agent 精炼 | 545-563 | auto-skill-optimizer + skill-creator 视角并行评审 |
| 更新机制 | 566-578 | 已有 Skill 的增量更新流程 |
| 品味守则 | 581-595 | 长文>金句、争议>共识、变化>固定 |
| 特殊场景 | 598-637 | 活人/历史、主题/人物、中国/西方、冷门人物、自我蒸馏 |

**关键发现**：SKILL.md 共 **645 行**，远超 skill-creator 建议的 **<500 行** 上限。虽然使用了 reference 文件分离部分重量级内容（模板 116 行、方法论 152 行），但主文件仍然过长。

---

## 三、Progressive Disclosure 渐进式披露评估

Skill-Creator 推荐的加载层级：

```
Level 1: Metadata (name + description)    ~100 words, always loaded
Level 2: SKILL.md body                    <500 lines, loaded on trigger
Level 3: Bundled resources                unlimited, loaded on demand
```

### 3.1 Nuwa-Skill 的实际加载分布

| 层级 | 内容 | 大小 | 问题 |
|------|------|------|------|
| L1 Metadata | description | ~80 词 | 合格，触发词覆盖较全 |
| L2 Body | 645 行 SKILL.md | 过长 | **超标 29%** |
| L3 Resources | skill-template.md (116行) | 按需加载 | 合格 |
| L3 Resources | extraction-framework.md (152行) | 按需加载 | 合格 |
| L3 Scripts | 4 个脚本 | 按需执行 | 合格 |

### 3.2 超标根因分析

645 行的 SKILL.md 中，以下部分可以进一步下放到 L3：

- **Phase 1.5 和 Phase 2.5 的检查点模板**（约 60 行）：纯展示格式，可移到 `references/checkpoint-templates.md`
- **6 个 Agent 的详细 prompt 模板**（约 80 行）：可移到 `references/agent-prompts.md`，主文件只保留分配表
- **特殊场景章节**（约 40 行）：中国人物/西方人物/冷门人物/自我蒸馏的处理差异，可精简为速查表
- **示例对话**（Phase 0B 约 25 行）：可精简或移到 reference

**预计精简后**：主文件可降至 ~440 行，符合 <500 行标准。

### 3.3 读取指引清晰度

SKILL.md 中明确标注了何时读取 reference 文件：
- "Phase 3 构建时，读取 `references/skill-template.md` 获取标准结构"
- "Phase 2 中标注「信息不足」，在诚实边界中说明"
- "读取 `references/extraction-framework.md` 获取心智模型的三重验证方法论"

**评估**：指引清晰，但缺少**目录索引**（table of contents）。645 行文件没有 TOC，定位特定 Phase 需要手动滚动。

---

## 四、Triggering 触发机制评估

### 4.1 Frontmatter Description

```yaml
description: |
  女娲造人：输入人名/主题/甚至只是模糊需求，自动深度调研→思维框架提炼→生成可运行的人物Skill。
  两种入口：(1)明确人名→直接蒸馏 (2)模糊需求→诊断推荐→再蒸馏。
  触发词：「造skill」「蒸馏XX」「女娲」「造人」「XX的思维方式」「做个XX视角」「更新XX的skill」。
  模糊需求也触发：「我想提升决策质量」「有没有一种思维方式能帮我...」「我需要一个思维顾问」。
```

**Skill-Creator 评估标准**：

| 标准 | 表现 | 评分 |
|------|------|------|
| 包含 "when to use" | 是，列出了明确触发词 | ★★★★★ |
| 包含 "what it does" | 是，"深度调研→思维框架提炼→生成可运行的人物Skill" | ★★★★★ |
| Pushy enough（防 undertrigger） | 中等，触发词列表较长但缺少边缘场景覆盖 | ★★★☆☆ |
| 模糊需求触发 | 是，但可能导致误触发 | ⚠️ 见下方 |

### 4.2 触发精度风险

**高风险误触发场景**：

1. **"我想提升决策质量"** → 这是一个非常常见的泛化需求，用户可能只是想得到一般性建议，而非生成一个 Skill。但 description 明确将其列为触发词，可能导致 overtrigger。

2. **"我需要一个思维顾问"** → 同样过于宽泛。用户可能只是想让我用现有能力给建议，而非走完整的多 Agent 蒸馏流程。

3. **与现有 Perspective Skills 的竞争**：当用户说 "用费曼的视角" 时，系统中已有 `feynman-perspective` Skill。Nuwa 的触发词中未明确排除 "直接使用已有 Skill" 的场景，可能导致两个 Skill 竞争。

### 4.3 改进建议

```yaml
# 建议优化后的 description
description: |
  女娲造人：从零创建一个人物/主题视角 Skill。输入人名、主题或模糊需求，
  自动执行多 Agent 深度调研→心智模型提炼→可运行 Skill 生成。
  当用户想要**新建**或**更新**某个人物的思维视角 Skill 时触发。
  触发词：「造skill」「蒸馏XX」「做个XX视角」「生成XX的skill」「更新XX的skill」。
  如果系统中已有该人物的 Skill（如 feynman-perspective），优先使用现有 Skill，不触发女娲。
  模糊需求仅在用户明确说「想要一个思维顾问/决策框架」且未指定具体人物时触发。
```

---

## 五、Writing Quality 写作质量评估

### 5.1 指令清晰度

| 维度 | 表现 | 评分 |
|------|------|------|
| 步骤编号 | Phase 0→5 清晰编号，子步骤用字母/数字细分 | ★★★★★ |
| 表格使用 | 大量表格（入口分流表、Agent 分配表、信息源优先级表、通过标准表），结构性强 | ★★★★★ |
| 示例丰富 | Phase 0B 有完整对话示例，Phase 3 有 Agentic Protocol 生成示例 | ★★★★★ |
| 反模式标注 | "绝不做的事"、"错误的处理方式" 明确列出 | ★★★★★ |
| 质量门控 | Phase 1.5、2.5、4 三个检查点强制暂停 | ★★★★★ |

### 5.2 写作风格

Skill-Creator 推荐的原则：
- 用 "why" 替代 "MUST/ALWAYS"
- 解释 reasoning，让模型理解
- 避免过度僵化的结构

Nuwa-Skill 的表现：
- ✅ **解释充分**：每个 Phase 开头都有 "为什么需要这个步骤" 的说明
- ✅ **柔性约束**：用 "建议"、"推荐" 而非全大写 MUST
- ⚠️ **部分 rigid**：Phase 4 的通过标准表格使用了明确的数字约束（"3-7个心智模型"、"5-10条启发式"），这是合理的——这些是可量化的质量标准
- ✅ **Theory of Mind**：充分考虑到 Claude 在使用 Skill 时的决策过程（如 "Claude 有 tendency to undertrigger skills"）

### 5.3 可操作性（Actionability）

这是 Nuwa-Skill 的核心优势，也是其复杂度来源。

| 流程步骤 | 可操作性 | 依赖 |
|---------|---------|------|
| Phase 0: 入口分流 | ★★★★★ | 纯 prompt 逻辑，无外部依赖 |
| Phase 0.5: 目录创建 | ★★★★★ | `mkdir` + 文件操作，完全可控 |
| Phase 1: 6 Agent 并行调研 | ★★★☆☆ | **重度依赖 subagents**，Claude.ai 不可用 |
| Phase 1.5: 调研 Review | ★★★★★ | 纯展示，用户确认 |
| Phase 2: 框架提炼 | ★★★★☆ | 依赖 Agent 输出质量，但有方法论支撑 |
| Phase 3: Skill 构建 | ★★★★☆ | 模板驱动，但需要大量内容填充 |
| Phase 4: 质量验证 | ★★★☆☆ | 需要 spawn 子 Agent 运行测试，Claude.ai 不可用 |
| Phase 5: 双 Agent 精炼 | ★★☆☆☆ | **重度依赖 subagents**，Claude.ai 不可用 |

**核心矛盾**：Nuwa-Skill 设计为一个**重度 Agentic 的复杂工作流**，但在 Claude.ai（无 subagents）环境下，Phase 1、4、5 无法执行。这是一个**平台兼容性**问题，不是 Skill 本身的质量问题。

---

## 六、Testability 可测试性评估

### 6.1 内置测试机制

Nuwa-Skill 内置了完善的质量验证体系：

| 测试类型 | 存在 | 自动化程度 |
|---------|------|-----------|
| 已知测试（Sanity Check） | ✅ Phase 4.1 | 需手动 spawn 子 Agent |
| 边缘测试（Edge Case） | ✅ Phase 4.2 | 需手动 spawn 子 Agent |
| 风格测试（Voice Check） | ✅ Phase 4.3 | 需手动 spawn 子 Agent |
| 量化通过标准 | ✅ Phase 4.4（6项标准表格） | 半自动（`quality_check.py` 辅助） |
| 触发准确性测试 | ❌ | 无内置 |
| 回归测试（更新场景） | ⚠️ Phase 4 + 更新机制 | 需手动执行 |

### 6.2 与 Skill-Creator 测试框架的对比

Skill-Creator 推荐的测试流程：
1. 创建 `evals/evals.json` 定义测试 prompt
2. Spawn 并行子 Agent（with-skill vs baseline）
3. 运行量化 assertions
4. 生成 benchmark.json
5. 启动 eval-viewer 人工 review
6. 迭代改进

Nuwa-Skill 的测试流程：
1. Phase 4.1-4.3：定性验证（3 个测试场景）
2. Phase 4.4：量化通过标准（6 项检查）
3. Phase 5：双 Agent 精炼评审

**差距**：
- Nuwa 缺少**系统化的 eval 用例集合**（evals.json）
- Nuwa 缺少**基准对比**（with-skill vs without-skill 或 old vs new）
- Nuwa 缺少**自动化 grading 和 benchmark 生成**
- Nuwa 的 `quality_check.py` 是一个轻量脚本，未达到 Skill-Creator 的 `generate_review.py` + `aggregate_benchmark` 的完整度

### 6.3 改进建议

如果希望将 Nuwa-Skill 的测试体系与 Skill-Creator 对齐：

1. **添加 evals.json 模板**：在 `references/eval-templates/` 中预置人物 Skill 的标准测试用例
2. **扩展 quality_check.py**：加入自动化 grading（PASS/FAIL + evidence）
3. **添加 benchmark 生成**：支持新旧版本 Skill 的盲测对比

---

## 七、综合评分

| 维度 | 评分 | 核心结论 |
|------|------|---------|
| 结构完整性 (Anatomy) | 9/10 | 文件组织优秀，模板/方法论/示例/脚本齐全 |
| 渐进式披露 (Progressive Disclosure) | 6/10 | 主文件 645 行超标，缺少 TOC，部分模板内容可下放 |
| 触发机制 (Triggering) | 6/10 | 触发词覆盖全但精度不足，有 overtrigger 风险 |
| 写作质量 (Writing Quality) | 9/10 | 步骤清晰、示例丰富、方法论成熟、解释充分 |
| 可测试性 (Testability) | 7/10 | 内置 3 层质量验证，但缺少系统化 eval 框架 |
| **综合评分** | **7.4/10** | 方法论极其成熟，但在工程化和平台兼容性上有改进空间 |

---

## 八、关键问题清单

### 8.1 高优先级

1. **SKILL.md 过长（645 行）**
   - 影响：超出建议上限 29%，增加 token 消耗，降低加载效率
   - 修复：将 Agent prompt 模板、检查点模板、特殊场景速查表下放到 references/

2. **触发精度不足**
   - 影响：泛化需求（"提升决策质量"）可能导致不必要的 Skill 生成流程
   - 修复：收紧模糊需求触发条件，明确排除 "直接使用已有 Skill" 的场景

3. **平台兼容性（Claude.ai）**
   - 影响：Phase 1、4、5 重度依赖 subagents，在 Claude.ai 上完全不可用
   - 修复：添加 Claude.ai 适配说明（见 SKILL.md 末尾已有简要说明，但不够突出）

### 8.2 中优先级

4. **缺少系统化 Eval 框架**
   - 修复：参考 Skill-Creator 的 evals.json + benchmark 流程，为生成的 Skill 提供标准测试模板

5. **脚本工具不完整**
   - 缺失：Agent 调用自动化封装、跨平台兼容性处理
   - 修复：添加 `run_agent.sh` 统一封装 subagent 调用

6. **更新机制未验证**
   - Phase 4 的 "更新已有 Skill" 流程描述简洁，缺少具体操作示例
   - 修复：添加一个 "更新费曼 Skill" 的完整示例

### 8.3 低优先级

7. **示例 Skill 缺少版本标注**
   - 14 个 examples 未标注生成时间、Nuwa 版本、验证状态
   - 修复：在 example 目录添加 `metadata.json`

8. **国际化支持有限**
   - README 有多语言版本（JA/KO/ES/EN），但 SKILL.md 仅中文
   - 非核心问题，Skill 的触发和执行都在中文语境下

---

## 九、与 Skill-Creator 最佳实践的对比总结

| 最佳实践 | Nuwa-Skill 现状 | 差距 |
|---------|----------------|------|
| SKILL.md < 500 行 | 645 行 | ❌ 超标 29% |
| Reference > 300 行 需 TOC | extraction-framework.md 152 行，无 TOC | ⚠️ 建议添加 |
| Metadata ~100 词 | ~80 词 | ✅ 合格 |
| Description 需 pushy | 中等 pushy | ⚠️ 可加强 |
| 定义 output format | Phase 3 有详细填充表 | ✅ 优秀 |
| 包含示例 | 大量示例（对话、流程、格式） | ✅ 优秀 |
| 测试用例 + assertions | Phase 4 有定性测试，缺 evals.json | ⚠️ 可加强 |
| 迭代循环（eval → improve → repeat） | Phase 4→5 有验证+精炼，缺系统化 benchmark | ⚠️ 可加强 |
| 解释 why 而非 MUST | 整体优秀 | ✅ 优秀 |

---

## 十、结论

Nuwa-Skill 是一个**方法论极其成熟、设计理念先进的元 Skill**。它在以下方面达到或超过了行业最佳实践：

- ✅ 多 Agent 并行调研的方法论设计
- ✅ 心智模型三重验证的提炼框架
- ✅ 表达 DNA 的量化分析
- ✅ 质量门控的三检查点设计
- ✅ 特殊场景的全面覆盖

但在**工程化落地**方面存在可改进空间：

- ⚠️ 文件长度控制
- ⚠️ 触发精度调优
- ⚠️ 与 Skill-Creator 测试框架的对齐
- ⚠️ Claude.ai 平台的兼容性说明

**总体评价**：这是一个 **A- 级 Skill**——理念 S 级，执行 A 级，工程化 B+ 级。对于大多数 Claude Code 用户（有 subagents），它能可靠运行；对于 Claude.ai 用户，需要手动降级执行。
