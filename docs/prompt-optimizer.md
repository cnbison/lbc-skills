# prompt-optimizer Skill 深度解析与使用指南

> **Skill 路径**：`skills/prompt-optimizer/SKILL.md`
> **形态**：纯 prompt skill（无脚本、无外部依赖）
> **核心能力**：基于任务场景匹配合适的提示词框架，生成更清晰、更可执行的 prompt

---

## 一、Skill 定位：Prompt 工程的"框架匹配引擎"

`prompt-optimizer` 不是简单的"润色 prompt"工具，而是一个**结构化的问题诊断 + 框架匹配 + 生成引擎**。它的核心假设是：**没有最好的 prompt，只有最适合当前任务的 prompt 框架**。

**与其他文本优化类 skill 的本质区别**：

| 维度 | 普通润色 | prompt-optimizer |
|------|---------|-----------------|
| 方法 | 语感调整、措辞优化 | 框架化重构 |
| 输入 | 一段文字 | 任务描述 / 模糊想法 / 原始 prompt |
| 输出 | 更好的文字 | 结构化、可复用的 prompt 模板 |
| 深度 | 表层 | 诊断 → 匹配 → 追问 → 生成 |
| 可解释性 | 低（改了什么不明确） | 高（说明为什么选这个框架） |

---

## 二、设计模式：Reviewer + Inversion + Generator

该 skill 明确声明采用三种设计模式的组合：

### 2.1 Reviewer（评审者模式）

先判断用户现有 prompt 或任务描述的问题：
- 是结构缺失？还是信息不足？
- 是过度复杂化了简单任务？还是简单任务被写得太随意？
- 目标受众是否明确？输出格式是否清晰？

**关键判断**：不要一上来就套框架，先诊断"病"再开"药"。

### 2.2 Inversion（逆向模式）

当信息不足时，不猜测、不假设，而是**主动追问最小必要信息**：
- Goal Clarity（目标是否清晰）
- Target Audience（受众是谁）
- Context Completeness（背景信息是否充分）
- Format Requirements（输出格式要求）
- Constraints（限制条件）

这是该 skill 区别于"模板生成器"的核心设计——它知道**好的 prompt 来自对需求的精确理解，而非华丽的模板**。

### 2.3 Generator（生成器模式）

在诊断完成、信息补足、框架选定后，才进入生成阶段：
- 按框架组件结构化 prompt
- 融入所有澄清后的信息
- 确保清晰度和 specificity
- 必要时加入示例（如果框架要求 few-shot）

---

## 三、六步工作流详解

```
┌─────────────────────────────────────────────┐
│ Step 1: Analyze User Input                   │
│ （判断输入类型：原始prompt / 任务描述 / 模糊想法）  │
├─────────────────────────────────────────────┤
│ Step 2: Match Scenario and Select Framework  │
│ （按复杂度 + 按领域 + 快速选择表 三重匹配）        │
├─────────────────────────────────────────────┤
│ Step 3: Load Framework Details               │
│ （读取 references/frameworks/ 下的框架详情文件）   │
├─────────────────────────────────────────────┤
│ Step 4: Clarify Ambiguities                  │
│ （追问目标、受众、上下文、格式、约束）             │
├─────────────────────────────────────────────┤
│ Step 5: Generate Optimized Prompt            │
│ （应用框架结构，生成最终 prompt）                │
├─────────────────────────────────────────────┤
│ Step 6: Present and Iterate                  │
│ （解释框架选择理由，提供迭代建议）                │
└─────────────────────────────────────────────┘
```

### 3.1 Step 1：输入分析

该 skill 能处理三种输入形态：

| 输入类型 | 示例 | 处理方式 |
|---------|------|---------|
| **原始 prompt** | "帮我写个营销文案" | 诊断结构问题，判断是否需要框架升级 |
| **任务描述** | "我要给新产品写推广邮件" | 提取场景要素，匹配营销类框架 |
| **模糊想法** | "我想让 AI 帮我做点什么" | 进入追问模式，引导用户明确需求 |

### 3.2 Step 2：框架选择（三重匹配机制）

这是 skill 最核心的设计——**不是给用户一个框架，而是帮用户找到最合适的框架**。

**维度一：按复杂度匹配**

| 复杂度 | 元素数量 | 推荐框架 |
|--------|---------|---------|
| 简单（≤3 个元素） | APE, ERA, TAG, RTF, BAB, PEE, ELI5 |
| 中等（4-5 个元素） | RACE, CIDI, SPEAR, SPAR, FOCUS, SMART, GOPA, ORID, CARE, ROSE, PAUSE, TRACE, GRADE, TRACI, RODES |
| 复杂（6+ 个元素） | RACEF, CRISPE, SCAMPER, Six Thinking Hats, ROSES, PROMPT, RISEN, RASCEF, Atomic Prompting |

**维度二：按领域匹配**

| 领域 | 推荐框架 |
|------|---------|
| 营销内容 | BAB, SPEAR, Challenge-Solution-Benefit, BLOG, PROMPT, RHODES |
| 决策分析 | RICE, Pros and Cons, Six Thinking Hats, Tree of Thought, PAUSE, What If |
| 教育培训 | Bloom's Taxonomy, ELI5, Socratic Method, PEE, Hamburger Model |
| 产品开发 | SCAMPER, HMW, CIDI, RELIC, 3Cs Model |
| AI 对话/助手 | COAST, ROSES, TRACE, RACE, RASCEF |
| 写作创作 | BLOG, 4S Method, Hamburger Model, Few-shot, RHODES, Chain of Destiny |
| 图像生成 | Atomic Prompting |
| 快速简单任务 | Zero-shot, ERA, TAG, APE, RTF |
| 复杂推理 | Chain of Thought, Tree of Thought |

**维度三：快速选择表**

对于不知道用什么框架的用户，提供直觉式映射：

| 用户说 | 推荐框架 |
|--------|---------|
| "我要一个简单的 prompt" | APE, ERA, TAG |
| "我想说服/销售" | BAB, SPEAR, Challenge-Solution-Benefit |
| "我需要分析/决策" | RICE, Pros and Cons, Chain of Thought |
| "我想教/解释" | ELI5, Bloom's Taxonomy, Socratic Method |
| "我需要创意" | SCAMPER, HMW, SPARK, Imagine |
| "我要结构化写作" | BLOG, 4S Method, Hamburger Model |
| "我要逐步推理" | Chain of Thought, Tree of Thought |
| "我在生成图像" | Atomic Prompting |
| "我要详细计划" | RISEN, RASCEF, CRISPE |

### 3.3 Step 3：加载框架详情

框架详情存储在 `references/frameworks/` 目录下，命名规范为 `XX_FrameworkName_Framework.md`。

每个框架文件包含：
- 应用场景
- 框架组件及解释
- 优缺点
- 多个实践示例

**注意**：当前该 skill 的 references 目录可能为空或框架文件未完全导入。实际使用中，Claude 会尝试读取这些文件，如果找不到，会基于训练语料中的框架知识进行生成。

### 3.4 Step 4：澄清追问（最小必要信息）

在生成最终 prompt 前，skill 会验证五个关键维度：

| 维度 | 追问示例 |
|------|---------|
| **Goal Clarity** | "What specific outcome are you hoping to achieve?" |
| **Target Audience** | "Who is the target audience for this content?" |
| **Context Completeness** | "What context should the AI consider?" |
| **Format Requirements** | "Are there any format or length requirements?" |
| **Constraints** | "Are there any limitations or restrictions?" |

**设计意图**：避免"Garbage In, Garbage Out"。再精良的框架，如果输入的需求本身就是模糊的，输出也必然是模糊的。

### 3.5 Step 5：生成优化 Prompt

应用选定框架的结构，生成最终 prompt。关键原则：
- 按框架组件组织信息
- 融入所有澄清后的细节
- 确保清晰度和 specificity
- 必要时加入示例（few-shot）

### 3.6 Step 6：呈现与迭代

输出不仅给结果，还给**理由**：
- 说明为什么选这个框架
- 解释每个框架元素如何应用
- 提供变体建议
- 支持用户反馈后的迭代

---

## 四、核心约束（Gotchas）

Skill 内置了 5 条行为约束，防止"过度设计"和"框架滥用"：

| 约束 | 含义 |
|------|------|
| **不要一上来就套框架** | 先诊断，再匹配 |
| **不要为简单 prompt 过度设计** | 简单任务用简单框架（APE/ERA/TAG） |
| **不要强行输出长模板** | 用户只想快速润色时，给简洁版本 |
| **信息不清时先追问** | 目标、受众、格式、约束不明确时，先补信息 |
| **说明为什么选这个框架** | 解释比堆砌框架名更重要 |

这五条约束体现了该 skill 的**实用主义哲学**：prompt 优化的目的是让 AI 更好地完成任务，而不是让 prompt 看起来专业。

---

## 五、框架速查表

### 5.1 按复杂度分类

**简单（≤3 元素）**
- **APE** (Action, Purpose, Expectation) — 行动、目的、期望
- **ERA** (Expectation, Role, Action) — 期望、角色、行动
- **TAG** (Task, Action, Goal) — 任务、行动、目标
- **RTF** (Role, Task, Format) — 角色、任务、格式
- **BAB** (Before, After, Bridge) — 现状、理想、桥梁
- **PEE** (Point, Evidence, Explanation) — 观点、证据、解释
- **ELI5** (Explain Like I'm 5) — 通俗解释

**中等（4-5 元素）**
- **RACE** (Role, Action, Context, Expectation) — 角色、行动、上下文、期望
- **CIDI** (Context, Intent, Direction, Input) — 上下文、意图、方向、输入
- **SPEAR** (Situation, Problem, Evaluation, Action, Result) — 情境、问题、评估、行动、结果
- **SPAR** (Situation, Problem, Action, Result) — 情境、问题、行动、结果
- **FOCUS** (Foundation, Objectives, Constraints, Understanding, Solution) — 基础、目标、约束、理解、方案
- **SMART** (Specific, Measurable, Achievable, Relevant, Time-bound) — 具体、可衡量、可达成、相关、时限
- **GOPA** (Goal, Objective, Plan, Action) — 目标、目的、计划、行动
- **ORID** (Objective, Reflective, Interpretive, Decisional) — 客观、反映、解释、决策
- **CARE** (Context, Action, Result, Example) — 上下文、行动、结果、示例
- **ROSE** (Role, Objective, Scenario, Expectation) — 角色、目标、场景、期望

**复杂（6+ 元素）**
- **RACEF** (Role, Action, Context, Expectation, Format) — 角色、行动、上下文、期望、格式
- **CRISPE** (Capacity, Insight, Statement, Personality, Experiment) — 能力、洞察、陈述、个性、实验
- **SCAMPER** (Substitute, Combine, Adapt, Modify, Put, Eliminate, Reverse) — 替代、组合、适应、修改、用途、消除、反转
- **Six Thinking Hats** — 六顶思考帽
- **ROSES** (Role, Objective, Scenario, Expected Solution, Steps) — 角色、目标、场景、期望方案、步骤
- **PROMPT** (Problem, Role, Objective, Method, Parameters, Timeline) — 问题、角色、目标、方法、参数、时间线
- **RISEN** (Role, Input, Steps, Expectation, Narrowing) — 角色、输入、步骤、期望、收敛
- **RASCEF** (Role, Action, Steps, Context, Expectation, Format) — 角色、行动、步骤、上下文、期望、格式
- **Atomic Prompting** — 原子级提示（用于图像生成）

### 5.2 按领域分类

| 领域 | 首选框架 |
|------|---------|
| 营销内容 | BAB, SPEAR, Challenge-Solution-Benefit |
| 决策分析 | RICE, Pros and Cons, Six Thinking Hats, Tree of Thought |
| 教育培训 | ELI5, Bloom's Taxonomy, Socratic Method |
| 产品开发 | SCAMPER, HMW, CIDI |
| AI 对话/助手 | COAST, ROSES, TRACE, RACE |
| 写作创作 | BLOG, 4S Method, Hamburger Model |
| 图像生成 | Atomic Prompting |
| 快速任务 | Zero-shot, ERA, TAG, APE |
| 复杂推理 | Chain of Thought, Tree of Thought |

---

## 六、使用场景

| 场景 | 用户输入示例 | 预期行为 |
|------|------------|---------|
| **优化现有 prompt** | "帮我改改这个 prompt" | 诊断问题，选择框架，输出优化版 |
| **从任务描述生成** | "我要写一封客户挽留邮件" | 匹配营销领域框架（如 BAB/SPEAR），追问细节后生成 |
| **选择困难** | "不知道用什么框架好" | 用快速选择表引导，根据用户意图推荐 |
| **复杂任务拆解** | "我要做一个产品决策分析" | 推荐 Six Thinking Hats / Tree of Thought，按框架拆解 |
| **教学场景** | "帮我设计一个 prompt 教学生理解XX" | 匹配教育类框架（ELI5 / Bloom's / Socratic） |
| **图像生成优化** | "帮我优化 Midjourney 的 prompt" | 使用 Atomic Prompting 框架 |

---

## 七、设计亮点

### 7.1 "反过度设计"的实用主义

与其他 prompt 工程工具不同，该 skill 明确反对"为了框架而框架"。Gotchas 中的五条约束确保：
- 简单任务不会被塞进复杂模板
- 用户不会收到一堆看不懂的框架术语
- 信息不足时不会瞎猜，而是追问

### 7.2 三重匹配机制

按复杂度 + 按领域 + 快速选择表的三重设计，覆盖了不同类型的用户：
- **专业用户**：知道任务复杂度，按复杂度选
- **领域用户**：知道自己做什么，按领域选
- **新手用户**：只知道"我想做什么"，按直觉描述选

### 7.3 可解释性优先

Skill 强制要求解释"为什么选这个框架"，这比单纯给结果更有教育价值。用户在多次使用后，会逐渐学会自己判断该用什么框架。

### 7.4 迭代友好

Step 6 明确支持"用户要求修改时迭代"，并维持框架结构。这意味着用户可以在框架的骨架上做微调，而不是每次从零开始。

---

## 八、局限性与注意事项

### 8.1 框架文件依赖

Skill 依赖 `references/frameworks/` 目录下的框架详情文件。如果文件缺失，Claude 会退回到训练语料中的框架知识，但可能不够精确或缺少最新实践。

**建议**：如果使用频繁，应确保框架文件完整导入。

### 8.2 框架数量庞大

skill 内置了 30+ 个框架，对用户来说可能有"选择 overload"的风险。虽然有三重匹配机制，但用户可能需要多次使用才能熟悉各个框架的区别。

### 8.3 语言偏向

当前框架名和说明以英文为主（APE, RACE, CRISPE 等），对中文用户可能不够直观。实际使用时，Claude 会用中文解释框架含义，但框架本身的记忆成本还在。

### 8.4 与 Claude Code Skill 系统的交互

该 skill 生成的优化 prompt 通常是给用户**在其他场景中使用**的（比如直接发给 ChatGPT、Claude Web、Midjourney 等），而不是在 Claude Code 内部使用。这意味着它的价值更多体现在"外部能力扩展"而非"Claude Code 自身增强"。

---

## 九、与其他 Skill 的对比

| Skill | 类型 | 输入 | 输出 | 最佳场景 |
|-------|------|------|------|---------|
| **prompt-optimizer** | Prompt 工程 | 任务描述 / 原始 prompt | 结构化 prompt 模板 | 需要系统化设计 prompt 时 |
| **skill-creator** | Skill 开发 | 人物/任务描述 | 完整的 SKILL.md | 创建可复用的 AI 能力包 |
| **dou-wentao-perspective** | 思维框架 | 分析问题 | 人物视角的分析 | 需要特定思维方式时 |
| **persona-forge** | 人格化引擎 | 人名/主题 | 人物 Skill | 把任意人物蒸馏成 skill |

**关键区分**：
- `prompt-optimizer` 优化的是**单次交互的 prompt**
- `skill-creator` 创建的是**可复用的 skill 文件**
- 两者的关系：当你用 `prompt-optimizer` 反复优化某类任务的 prompt 后，可以用 `skill-creator` 把它固化成一个 skill

---

## 十、文件结构

```
skills/prompt-optimizer/
├── SKILL.md                              # 主定义文件（151 行）
└── references/
    ├── Frameworks_Summary.md             # 框架摘要（场景、复杂度、领域映射）
    └── frameworks/
        ├── 01_RACEF_Framework.md
        ├── 02_CRISPE_Framework.md
        ├── 03_SCAMPER_Framework.md
        └── ...                           # 各框架详情文件
```

当前状态：纯 prompt skill，零外部依赖。

---

## 十一、总结

`prompt-optimizer` 是一个**结构化、可解释、反过度设计的 prompt 工程工具**。它的核心价值不在于"让 prompt 变长"，而在于"让 prompt 变对"——通过诊断 → 匹配 → 追问 → 生成的四步闭环，确保每个 prompt 都建立在正确的框架和清晰的需求之上。

**最值得学习的三点**：

1. **实用主义优先**：明确反对为简单任务套复杂框架，五条 Gotchas 是 prompt 工程的"医德"
2. **可解释性设计**：不仅给结果，还给理由，让用户学会自己判断
3. **最小必要信息原则**：信息不足时不瞎猜，而是追问，这比"看起来智能的猜测"更专业

如果你经常需要为不同任务设计 prompt，这个 skill 是最好的系统性起点。当你对某个框架用得足够熟练后，甚至可以跳过 skill，直接用框架自己写——而这，恰恰是 skill 设计的最高境界：让用户最终不再需要它。
