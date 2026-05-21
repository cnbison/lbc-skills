# Frame Method（反路径锁定多框架分析法）深度解析

> 分析对象：`skills/frame-analysis/SKILL.md`
> 分析日期：2026-05-18

---

## 一、定位与核心思想

Frame Method 是一个**高阶思维工具型 skill**，目标不是完成具体任务，而是改变 Claude 面对复杂问题时的默认推理模式。其核心主张可以概括为一句话：**"在探索期延迟收敛，先让问题充分展开。"**

设计者认为，大语言模型在回答开放性问题时存在一种结构性缺陷——过早收敛（premature convergence）。模型倾向于迅速给出一个看起来完整、自洽、聪明的答案，而这个答案往往会锁定后续所有推理路径，使用户看不到问题的其他可能面貌。Frame Method 就是要对抗这种倾向。

这个定位决定了它的独特性：它不是"帮你做事"的 skill，而是"帮你改变思考方式"的 skill。在本仓库现有的 skill 谱系中，它最接近 `dou-wentao-perspective`（思维顾问）和 `persona-forge`（思维框架提炼），但比前者更方法论化，比后者更偏向通用认知工具而非特定人物视角。

---

## 二、文件结构与格式合规性

### 2.1 目录结构

```
skills/frame-analysis/
└── SKILL.md              # 单一文件，纯 prompt skill
```

极简骨架，符合本仓库"能用 prompt 解决的不引入脚本"的原则。

### 2.2 Frontmatter 分析

```yaml
---
name: cg0x-frame-analysis
alwaysApply: true
description: >
  Frame Method (反路径锁定多框架分析法) with on/off toggle.
  ...
---
```

| 字段 | 现状 | 合规性 |
|------|------|--------|
| `name` | `cg0x-frame-analysis` | 与目录名 `frame-analysis` 不完全一致。本仓库惯例是 skill 名与目录名一致（kebab-case），此处加了 `cg0x-` 前缀，可能是作者命名空间标记。|
| `alwaysApply` | `true` | 危险信号。本仓库 skill 通常用 `always: false`（或省略），因为 `always: true` 会导致该 skill 在**每个会话**中常驻加载，增加系统提示负担。但设计者在这里玩了一个技巧——`alwaysApply: true` 只是让 skill 被加载，内部逻辑默认是 OFF（休眠），通过命令切换。这是一种"物理常驻、逻辑按需"的设计。|
| `description` | 详细说明了三种模式 | 包含了触发指令（`/cg0x-frame-analysis on/off/<question>`），但没有中英双语触发词列表，与本仓库"description 必含触发词"的惯例有偏差。不过考虑到这是一个命令式交互 skill，传统的"触发词"概念在这里不太适用。|
| `version` / `author` | 缺失 | 非必填，但建议补充。|
| `requires_bins` / `requires_env` | 缺失 | 本 skill 为纯 prompt，无需声明。|

**格式建议**：
- 建议将 `alwaysApply: true` 改为 `always: false`，因为当前设计已经通过内部状态机实现了按需激活，`alwaysApply` 的常驻加载意义不大，反而增加 token 消耗。
- `description` 中可补充标准触发词，如 `Triggers: /cg0x-frame-analysis, 框架分析, 反路径锁定`。

---

## 三、触发机制设计：命令式状态机

Frame Method 采用了一种在本仓库中**非常罕见**的触发模式：**命令式状态机**。

```
/cg0x-frame-analysis on      → 开启自动判断模式
/cg0x-frame-analysis off     → 关闭（休眠）
/cg0x-frame-analysis <问题>  → 单次触发
```

这种模式的优势：
1. **精确控制**：用户明确知道 skill 是否在工作，不像关键词触发那样容易误触。
2. **状态持久**：开启后可以对后续所有消息进行"静默判断"，实现一种半自动的分析顾问角色。
3. **避免干扰**：对于简单事实类问题，skill 内部有 Use Gate，会主动拒绝进入复杂分析，避免过度思考。

潜在问题：
1. **学习成本**：用户需要记住 `/cg0x-frame-analysis` 这个命令前缀，不够直观。
2. **前缀冲突**：如果未来有其他 skill 也使用 `cg0x-` 前缀，可能产生命令空间碰撞。
3. **Claude Code 的 skill 加载机制**：`alwaysApply: true` 的实际行为取决于 Claude Code 客户端如何解析，并非所有客户端都支持命令式状态切换。

---

## 四、方法论深度：一个完整的认知协议

Frame Method 不是简单的"多角度看问题"，而是一套严格的**认知纪律协议**。其方法论可以拆解为五个层次：

### 4.1 问题适配层（Use Gate）

明确列出了不适合使用该方法的 5 类场景：
- 具体事实查询
- 直接执行步骤
- 已收敛问题
- 要求快速结论
- 范围过小的问题

这个门槛设计非常关键。很多"思维框架"的失败，都是因为被用在了不需要框架的地方——用大炮打蚊子，既浪费又惹人烦。Use Gate 的存在说明设计者很清楚这个方法的边界。

### 4.2 框架生成层（Core Principles）

10 条核心原则构成了方法论的骨架，其中最值得注意的几条：

- **"Default to 3–5 frames"**：不是越多越好，而是强制要求"足够多以展开张力，足够少以保持清晰"。
- **"Before all frames are sufficiently developed, do not synthesize, rank, or signal preference"**：这是对抗 LLM 默认行为的硬性约束。
- **"A unified conclusion is optional"**：明确许可"无结论"状态，这是对传统问答模式的根本性挑战。
- **"Do not force all frames into a higher-order master frame"**：防止用"更深层的统一理论"来消解真正的冲突。

### 4.3 输出结构层（Output Structure）

五段式输出结构非常清晰：

| 部分 | 功能 |
|------|------|
| Minimal Restatement | 最小失真重述问题，防止偷换问题 |
| List the Frames | 3-5 个有区分度的框架命名 |
| Develop Each Frame | 六要素：看什么、假设、能解释什么、忽略什么、冲突点、失败条件 |
| Preserve the Tension | 显式标识不兼容之处 |
| Stop at the Problem Map | 停在问题地图，不强行收敛 |

这个结构强迫 Claude 在执行时"放慢脚步"，每一步都有明确的产出物，不容易跳步。

### 4.4 语言纪律层（Language Requirements）

这是本 skill 中最有野心的部分。它不仅约束思考结构，还约束**语言风格**：

- 禁用"归根到底""本质上""at the end of the day"——这些词是过早收敛的语言标记。
- 禁用"not X but Y"构造——这种修辞会抹杀一条路径来强迫另一条。
- 禁止情绪安抚、姿态性认同、过早选边。

这些约束本质上是在要求 Claude 扮演一个**冷静到近乎冷峻的分析者**角色，而不是一个讨用户喜欢的助手。这种自我定位非常高阶。

### 4.5 自检与防漂移层（Idempotence & Drift Control）

- **幂等性要求**：同一问题多次运行，分析骨架应保持稳定。
- **伪差异检测**：要求框架至少在两个维度上不同（假设/注意力/评价标准/时间尺度/系统边界/风险偏好）。
- **主框架吞噬检测**：如果一个框架倾向于吞掉其他所有框架，视为"master-frame drift"并抑制。

这些机制针对的是 LLM 在执行多框架分析时的典型失败模式：表面上有多个框架，实际上都在说同一件事，或者有一个"更正确的"框架在暗中主导。

---

## 五、与仓库其他 Skill 的对比

| 维度 | Frame Method | dou-wentao-perspective | guwen-translate | daily-news |
|------|-------------|----------------------|-----------------|------------|
| **形态** | 纯 prompt | 纯 prompt | 纯 prompt | Python 流水线 |
| **目标** | 改变推理模式 | 模拟特定人物视角 | 古文翻译与解读 | 信息采集与生成 |
| **触发方式** | 命令式状态机 | 关键词触发 | 关键词触发 | 关键词触发 |
| **alwaysApply** | `true`（逻辑按需） | `false` | `false` | `false` |
| **方法论深度** | 极高（完整认知协议） | 中（13个来源提炼） | 中（四层翻译框架） | 低（流程自动化） |
| **输出约束** | 极强（语言+结构） | 中（风格模仿） | 中（四层结构） | 低（模板填充） |
| **自我检查** | 9项内部自检清单 | 无 | 无 | 无 |

Frame Method 在本仓库中的方法论深度是**独一档**的。它不是关于"做什么"，而是关于"怎么做思考本身"。

---

## 六、优点与亮点

1. **反 LLM 天性设计**：LLM 天生爱总结、爱收敛、爱让用户满意。Frame Method 的每一条规则都是对这些天性的对抗，这种"逆本能"设计非常难得。

2. **边界意识极强**：Use Gate、Failure Conditions、Idempotence 三层防线，说明设计者不仅知道这个方法能做什么，更清楚它不能做什么、会在哪里失败。

3. **语言即思维**：将语言风格约束写入方法论（禁用"本质上"等词），体现了"语言塑造思维"的深刻理解。这不是表层的话术规范，而是深层认知干预。

4. **不追求结论的正当性**：明确允许"无统一结论"状态，这在以"帮助用户解决问题"为默认目标的 AI 助手中非常罕见，也更接近真实的复杂问题分析。

5. **主框架吞噬检测**：这是高阶思维工具才会遇到的问题——最强的解释框架往往会消灭其他框架的存在意义。设计者意识到了这一点并写了显式规则来对抗。

---

## 七、潜在问题与风险

1. **alwaysApply: true 的副作用**：即使内部逻辑是 OFF，skill 文件本身仍然会被加载到每个会话的上下文中。对于 184 行的 SKILL.md，这意味着每个对话都要多承载这些 token，可能对长对话的性能产生轻微影响。

2. **命令前缀的记忆负担**：`/cg0x-frame-analysis` 这个前缀较长且带有命名空间标记，对于中文用户不够友好。可以考虑增加更短的中文别名。

3. **Use Gate 的判断依赖模型自身**：skill 要求 Claude"静默判断"问题是否适合 Frame Method，但这个判断本身可能出错——特别是对于处于灰色地带的问题。

4. **语言约束可能过度**：禁止"本质上""归根到底"等词在中文语境中有时是必要的（比如哲学讨论），一刀切的禁用可能在某些场景下显得僵硬。

5. **缺乏示例（Example）块**：本仓库的验收清单要求"至少一个 `<example>` 块"，而 Frame Method 的 SKILL.md 中没有提供示例。新用户可能难以直观理解输出应该长什么样。

6. **无软链接暴露**：`ls -la .claude/skills/` 中没有看到 `frame-analysis`，说明这个 skill 尚未暴露到 Claude Code 的加载路径。按照仓库 SOP，需要执行：
   ```bash
   ln -s ../../skills/frame-analysis .claude/skills/frame-analysis
   ```

---

## 八、改进建议

### 8.1 格式层面

1. **修正 alwaysApply**：改为 `always: false`，因为内部状态机已经实现了按需激活，无需物理常驻。
2. **补充 version 和 author**：完善 frontmatter。
3. **description 中补充触发词**：`Triggers: /cg0x-frame-analysis, 框架分析, frame method, 反路径锁定`
4. **增加 `<example>` 块**：展示一个完整的 Frame Method 输出样例，降低用户学习成本。
5. **建立软链接**：暴露到 `.claude/skills/` 并 `git add`。

### 8.2 内容层面

1. **增加中文命令别名**：如 `/框架分析 on` 或 `/fm on`（如果与其他 skill 不冲突）。
2. **Use Gate 可视化**：当 skill 判断问题不适合时，可以给出一个"轻量判断"作为备选，而不是简单的拒绝。
3. **框架差异度自检示例**：在 Idempotence 部分增加一个"如何判断两个框架是否真正不同"的具体示例。
4. **考虑增加 `references/`**：Frame Method 的思想来源可以追溯至多种认知科学和批判性思维传统（如查理·芒格的多元思维模型、法学中的 issue spotting、社会科学中的多范式分析）。可以在 `references/` 中放置相关文献或笔记，提升 skill 的可信度和深度。

---

## 九、总体评价

Frame Method 是一个**方法论层面非常成熟**的 skill。它不是"又一个 prompt 模板"，而是一个完整的**认知协议**——从问题准入、框架生成、语言纪律到自检防漂移，每一层都有明确的设计意图和失败防御。

在本仓库的 skill 谱系中，它占据了一个独特的生态位：**通用高阶思维基础设施**。如果其他 skill 是"专用工具"（翻译、播报、人物视角），Frame Method 就是"工具的工具"——它不改变你分析什么，而改变你如何分析。

其最大的价值在于**对抗 LLM 的过早收敛本能**。在 AI 辅助决策越来越深入的今天，一个专门用来"让人慢下来、看清楚分歧、不急于下结论"的思维工具，可能比任何"快速给出答案"的工具都更有长期价值。

**成熟度评分**：8.5/10
- 方法论深度：9.5/10
- 执行可行性：8/10
- 格式合规性：6.5/10（缺少示例、软链接、alwaysApply 待修正）
- 与本仓库生态融合度：7/10

---

## 十、相关资源

- Skill 源码：`skills/frame-analysis/SKILL.md`
- 本仓库 SOP：`CLAUDE.md` 第 6 节"创建新 Skill 的步骤"
- 同类型参考：`skills/dou-wentao-perspective/SKILL.md`（思维顾问类）
