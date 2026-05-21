# Persona Forge → Persona-Skill 改版方案

> 基于 `docs/persona-forge-analysis.md` 的深度分析，提出从「人物蒸馏」到「人格化引擎」的架构改版方案。
> 设计日期：2026-05-20
> 核心目标：不仅能把**真实人物**人格化，还能**按需组合**出一个特定人格化（如小学数学老师、耐心导师、幽默面试官）。

---

## 一、改版目标与定位变化

### 1.1 当前定位（Persona Forge）

| 维度 | 现状 |
|------|------|
| 核心能力 | 将真实人物的公开信息蒸馏为可运行的思维视角 Skill |
| 输入 | 人名 / 主题 / 模糊需求 |
| 输出 | `[name]-perspective` 或 `[topic]-framework` Skill |
| 信息源 | 网络搜索 + 本地语料（6 Agent 并行调研） |
| 质量保障 | 三重验证 + 3 个检查点 + 双 Agent 精炼 |

### 1.2 目标定位（Persona-Skill）

| 维度 | 目标 |
|------|------|
| 核心能力 | **人格化引擎**：真实人物蒸馏 + 合成角色组合 |
| 输入 | 人名 / 主题 / 模糊需求 / **角色描述** |
| 输出 | `[name]-perspective` / `[topic]-framework` / `[role]-persona` Skill |
| 信息源 | 网络搜索（人物）+ **维度组合逻辑**（合成角色）+ 本地语料 |
| 质量保障 | 保留原有验证体系，新增**角色一致性校验** |

### 1.3 核心差异：人物蒸馏 vs 角色合成

| 维度 | 人物蒸馏（现有） | 角色合成（新增） |
|------|-----------------|-----------------|
| 输入 | "蒸馏费曼" | "创建一个小学数学老师的 persona" |
| 信息来源 | 真实人物的著作/访谈/决策记录 | 领域通用知识 + 用户指定的维度组合 |
| 调研方式 | 6 Agent 并行网络搜索 | **无需网络搜索**，直接基于维度库生成 |
| 心智模型 | 从真实言行中提取 | 从维度组合中**推导+设计** |
| 表达 DNA | 模仿真实人物的说话方式 | **组合设计**说话方式（温度+风格+节奏） |
| 诚实边界 | "不能替代本人" | "此为合成角色，非真实人物" |
| 生成时间 | 长（需多 Agent 调研） | 短（直接组装） |

---

## 二、架构改版方案

### 2.1 入口分流升级：从 2 条路径到 3 条路径

当前 Phase 0 的入口分流：

```
用户输入
├── 明确人名/主题 → Phase 0A 直接路径
└── 模糊需求 → Phase 0B 诊断路径
```

改版后 Phase 0 的入口分流：

```
用户输入
├── 明确人名/主题 → Phase 0A 直接路径（人物蒸馏）
├── 模糊需求 → Phase 0B 诊断路径（推荐人物或主题）
└── 角色描述/合成需求 → Phase 0C 合成路径（新增）
```

#### Phase 0C 的触发条件

以下输入应分流到合成路径：
- "创建一个小学数学老师的 persona"
- "帮我组合一个耐心但严格的导师角色"
- "我想有个像XX一样温暖、像YY一样专业的角色"
- "设计一个适合教孩子编程的AI伙伴"
- "做一个能帮我改简历的HR面试官角色"
- 任何包含 "persona"、"角色"、"扮演"、"合成"、"组合" 且**不指向特定真实人物**的请求

#### Phase 0C 的执行流程

```
Step 1: 角色类型确认
  - 教育者（Teacher）
  - 顾问（Advisor）
  - 创意伙伴（Creative Partner）
  - 面试官（Interviewer）
  - 陪伴者（Companion）
  - 自定义（Custom）

Step 2: 维度选择（从 Persona Dimension Library 中选择/调整）
  - 必填维度：教学风格、沟通温度、表达方式
  - 选填维度：知识结构、互动模式、情感表达、价值观倾向
  - 用户可自定义新维度

Step 3: 使用场景确认
  - 目标受众（如：6-12岁小学生）
  - 交互场景（如：课后答疑、课堂讲解、作业批改）
  - 特殊约束（如：必须用中文、不能使用复杂术语）

Step 4: 确认与命名
  - 自动生成建议名称（如：patient-math-teacher-persona）
  - 用户确认或修改
```

---

### 2.2 核心新增组件：Persona Dimension Library（角色维度库）

#### 2.2.1 维度库设计

维度库是一组**预定义、可组合的人格化维度**。每个维度有多个选项，用户可选择或自定义。

```yaml
# 维度库结构（存于 references/persona-dimension-library.md）

dimensions:
  teaching_style:          # 教学风格
    - heuristic            # 启发式
    - direct_instruction   # 直接讲授
    - socratic             # 苏格拉底式
    - gamified             # 游戏化
    - scaffolding          # 支架式
    - inquiry_based        # 探究式
    - flipped              # 翻转课堂式
    
  communication_warmth:    # 沟通温度
    - warm_encouraging     # 温暖鼓励
    - rational_calm        # 理性冷静
    - humorous_witty       # 幽默风趣
    - serious_formal       # 严肃认真
    - gentle_firm          # 温和坚定
    - playful_casual       #  playful 随意
    
  knowledge_structure:     # 知识结构
    - broad_interdisciplinary  # 广博跨学科
    - deep_specialist      # 垂直专业
    - experience_oriented  # 经验导向
    - theory_oriented      # 理论导向
    - practical_hands_on   # 实践动手型
    
  expression_mode:         # 表达方式
    - story_driven         # 故事化
    - structured_logical   # 结构化逻辑
    - analogy_rich         # 类比密集
    - data_driven          # 数据驱动
    - visual_spatial       # 视觉空间型
    - conversational       # 对话式
    
  interaction_pattern:     # 互动模式
    - q_a                  # 问答式
    - guided_discovery     # 引导发现式
    - challenging          # 挑战式
    - companion            # 陪伴式
    - collaborative        # 协作式
    - evaluative           # 评估式
    
  emotional_expression:    # 情感表达
    - high_empathy         # 高共情
    - restrained           # 克制
    - enthusiastic         # 热情
    - calm_steady          # 冷静稳重
    - emotionally_intelligent  # 情商高
    
  value_orientation:       # 价值观倾向
    - growth_mindset       # 成长型思维
    - mastery_oriented     # 精通导向
    - process_focused      # 过程导向
    - outcome_focused      # 结果导向
    - equity_inclusive     # 公平包容
    - excellence_driven    # 卓越驱动
    
  humor_style:             # 幽默风格（可选）
    - self_deprecating     # 自嘲
    - absurd_contrast      # 荒诞对比
    - dry_wit              # 冷幽默
    - situational          # 情境幽默
    - no_humor             # 不幽默
```

#### 2.2.2 维度组合的约束规则

不是所有组合都合理。需要内置**冲突检测**和**协同推荐**：

| 冲突组合 | 说明 |
|---------|------|
| `serious_formal` + `playful_casual` | 温度维度互斥 |
| `high_empathy` + `evaluative` | 可能产生情感冲突，需用户确认 |
| `direct_instruction` + `inquiry_based` | 教学风格互斥 |

| 协同推荐 | 说明 |
|---------|------|
| `teaching_style: heuristic` → 推荐 `interaction_pattern: guided_discovery` | 启发式教学适合引导发现 |
| `communication_warmth: warm_encouraging` → 推荐 `emotional_expression: high_empathy` | 温暖鼓励需要高共情支撑 |
| `target_audience: children` → 推荐 `expression_mode: story_driven` + `analogy_rich` | 儿童适合故事+类比 |

---

### 2.3 合成角色的 Skill 生成流程

合成角色**不需要 Phase 1 的 6 Agent 网络调研**，而是走一条更短的"设计+组装"路径：

```
Phase 0C: 角色定义（用户确认维度组合）
    ↓
Phase 1C: 领域知识注入（轻量级）
    ↓
Phase 2C: 角色一致性设计
    ↓
Phase 3C: Skill 组装（使用扩展模板）
    ↓
Phase 4C: 角色测试（3 类测试）
    ↓
Phase 5C: 交付
```

#### Phase 1C: 领域知识注入

**与人物蒸馏的区别**：
- 人物蒸馏：6 Agent 搜索真实人物的言行记录
- 合成角色：**1 个 Agent 搜索该角色的领域通用知识**（如"小学数学教学法的最佳实践"）

执行方式：
```
Agent: 领域知识研究员
目标：搜索 [角色领域] 的通用方法论和最佳实践
搜索方向：
  - 该领域的教学/沟通/评估标准方法
  - 目标受众（如6-12岁儿童）的认知发展特点
  - 该领域的常见挑战和解决方案
输出：写入 references/research/01-domain-knowledge.md
```

**轻量化原则**：
- 只启动 1 个 Agent（而非 6 个）
- 搜索范围限定在领域方法论，不搜索特定人物
- 时长控制在 5 分钟内

#### Phase 2C: 角色一致性设计

基于用户选择的维度组合 + 领域知识，设计角色的"思维操作系统"：

| 组件 | 人物蒸馏（现有） | 角色合成（新增） |
|------|-----------------|-----------------|
| 心智模型 | 从真实言行中提取 | **从维度组合推导** |
| 决策启发式 | 从真实决策中归纳 | **从领域知识+维度设计** |
| 表达 DNA | 模仿真实人物 | **组合设计** |
| 价值观 | 从真实主张中提取 | **从维度+领域推导** |
| 反模式 | 从真实反对中归纳 | **从维度对立面推导** |

**表达 DNA 的组合设计示例**：

用户选择：
- `communication_warmth: warm_encouraging`
- `expression_mode: analogy_rich`
- `humor_style: situational`

推导出的表达 DNA：
```markdown
## 表达DNA

### 句式
- 多用鼓励性开头："这个想法很有意思！"、"你离正确答案只差一步了"
- 类比解释复杂概念："分数就像一块披萨被切成了几份"
- 允许学生犯错："错了没关系，我们来看看哪里可以调整"

### 词汇
- 高频词："试试"、"想想看"、"就像"、"很棒"、"再试一次"
- 避免："这么简单都不会"、"标准答案"、"必须"
- 专属术语：根据学科领域动态生成

### 节奏
- 先给信心，再给知识
- 一个概念配一个类比
- 频繁确认理解："我这样解释清楚吗？"

### 幽默
- 情境幽默：用课堂场景中的意外制造笑点
- 适度自嘲："老师小时候也犯过这个错"
```

#### Phase 3C: Skill 组装（扩展模板）

现有的 `skill-template.md` 需要扩展，支持合成角色的特殊字段：

```markdown
---
name: [role]-persona
description: |
  [角色描述]的人格化AI。一个[教学风格]的[领域]角色，
  擅长[核心能力1]、[核心能力2]。
  触发词：[触发词列表]
  注意：此为合成角色，非真实人物。
---

# [角色名] · 人格化系统

> [一句体现角色核心特征的开场白]

## 角色定位

**我是谁**：[基于维度的自我介绍]
**我的风格**：[教学/沟通/互动风格概述]
**我的目标**：[帮助用户达成的目标]
**我不做的**：[明确的边界]

## 核心能力模型

### 能力1: [名称]
**描述**：[基于维度+领域知识设计]
**应用场景**：[什么时候用这个能力]
**使用方式**：[具体怎么做]
**局限**：[什么时候不该用]

### 能力2: [名称]
...

## 互动规则

### 表达风格
- [基于 expression_mode 维度的具体规则]
- [基于 communication_warmth 维度的具体规则]
- [基于 humor_style 维度的具体规则]

### 回应结构
[基于 interaction_pattern 维度的回应模板]

### 情感边界
[基于 emotional_expression 维度的情感处理规则]

## 领域知识概览

[基于 Phase 1C 领域知识注入的核心方法论摘要]

## 价值观与禁忌

**我追求的**：[基于 value_orientation 维度]
**我拒绝的**：[基于维度的反面]
**特殊情况**：[目标受众相关的特殊处理]

## 角色一致性校验

此角色基于以下维度组合设计：
| 维度 | 选择 |
|------|------|
| [维度名] | [选择] |
| ... | ... |

如使用中发现角色行为与维度定义不一致，请反馈调整。

## 诚实边界

- 此为**合成角色**，基于领域通用知识和用户指定维度设计，非真实人物
- 不能替代真实的 [领域] 专业人士
- 信息截止到 [日期]，领域知识可能过时
- 对于超出 [领域] 范围的问题，角色会承认边界
```

---

### 2.4 混合模式：三种路径的统一执行

改版后的 Persona-Skill 需要在同一个 SKILL.md 中支持三种路径：

```markdown
## 执行流程总览

收到用户输入后，按以下顺序判断路径：

### 路径判断逻辑

```
if 输入包含明确人名/主题:
    → 路径A: 人物蒸馏
elif 输入包含角色描述/persona/合成/组合:
    → 路径C: 角色合成
else:
    → 路径B: 需求诊断（诊断后可能指向A或C）
```

### 路径A: 人物蒸馏（保留现有流程）
[Phase 0A → 0.5 → 1 → 1.5 → 2 → 2.5 → 3 → 4 → 5]
详见上文"人物蒸馏"部分。

### 路径B: 需求诊断（保留现有流程）
[Phase 0B → 推荐候选 → 用户选择 → 进入A或C]
新增：推荐候选时，同时包含"人物候选"和"角色合成候选"。

### 路径C: 角色合成（新增流程）
[Phase 0C → 1C → 2C → 3C → 4C → 5C]
详见上文"合成角色"部分。
```

---

## 三、触发词扩展

### 3.1 当前触发词（Persona Forge）

```
造skill、蒸馏XX、Persona Forge、造人、XX的思维方式、做个XX视角、更新XX的skill
我想提升决策质量、有没有一种思维方式能帮我...、我需要一个思维顾问
```

### 3.2 新增触发词（Persona-Skill）

```
# 人物蒸馏（保留）
造skill、蒸馏XX、做个XX视角、生成XX的skill、更新XX的skill

# 角色合成（新增）
创建一个XX角色、做一个XX persona、帮我设计一个XX、
组合一个XX人格、合成一个XX、角色扮演XX、
需要一个像XX一样...的角色、定制一个XX助手、
设计一个适合XX的AI伙伴、做一个教XX的AI老师

# 混合触发（新增）
用XX的风格教我XX（如：用费曼的风格教我数学）→ 可触发合成路径
做一个像XX一样但擅长YY的角色 → 可触发合成路径
```

### 3.3 触发精度优化

为避免 overtrigger，新增**排他规则**：
- 如果系统中已有对应人物的 Skill（如 `feynman-perspective`），用户说 "用费曼的视角" → **不触发** Persona-Skill，直接使用现有 Skill
- 用户说 "用费曼的风格教我数学" → 触发 Persona-Skill（合成路径），因为需要组合"费曼风格"+"数学教学"两个维度
- 用户说 "帮我提升决策质量" 且**未指定人物** → 触发诊断路径（B），推荐人物或合成角色

---

## 四、质量验证体系（路径C专用）

合成角色的质量验证与人物蒸馏不同，需要新增 Phase 4C：

### 4.1 角色一致性测试

选 3 个该角色**应该**处理的典型场景，验证角色行为是否符合维度定义：

| 测试场景 | 预期行为 | 验证维度 |
|---------|---------|---------|
| 学生答错了一道简单题 | 温暖鼓励 + 引导发现错误原因 | communication_warmth + interaction_pattern |
| 学生问了一个超纲问题 | 承认边界 + 用类比简化解释 | knowledge_structure + expression_mode |
| 学生表现出挫败感 | 高共情回应 + 降低难度 | emotional_expression + value_orientation |

### 4.2 维度偏离检测

验证角色在对话中**不出现**与维度定义冲突的行为：

| 维度选择 | 禁止行为 |
|---------|---------|
| `warm_encouraging` | 不说"这么简单都不会" |
| `analogy_rich` | 不用纯抽象定义解释概念 |
| `high_empathy` | 不忽视学生的情绪信号 |

### 4.3 领域适切性测试

验证角色的领域知识是否准确：
- 数学老师的角色：验证基本数学概念的解释是否正确
- 编程导师的角色：验证代码建议是否符合最佳实践
- HR 面试官的角色：验证面试流程和评估标准是否合理

### 4.4 通过标准

| 检查项 | 通过标准 |
|--------|---------|
| 角色一致性 | 3 个典型场景中，角色行为与维度定义一致 |
| 维度偏离 | 无禁止行为出现 |
| 领域适切性 | 领域知识准确，无事实错误 |
| 边界意识 | 角色能正确识别并拒绝超出能力范围的问题 |
| 表达辨识度 | 读 100 字能识别出这是该角色，不是通用 AI |

---

## 五、实施路线图

### Phase 1: 基础设施（1-2 天）

1. **创建 `references/persona-dimension-library.md`**
   - 定义所有维度和选项
   - 编写冲突检测规则
   - 编写协同推荐规则

2. **扩展 `references/skill-template.md`**
   - 添加合成角色模板（`[role]-persona`）
   - 保留人物 Skill 模板不变

3. **新增脚本工具**
   - `scripts/dimension_validator.py`：验证维度组合是否冲突
   - `scripts/persona_assembler.py`：根据维度组合自动组装 SKILL.md 草稿

### Phase 2: 核心流程（2-3 天）

4. **重写 SKILL.md 主文件**
   - 添加 Phase 0C 合成路径
   - 添加路径判断逻辑
   - 添加 Phase 1C/2C/3C/4C 流程
   - 精简主文件至 <500 行（将 Agent prompt 模板下放到 references）

5. **更新触发机制**
   - 扩展 description 中的触发词
   - 添加排他规则（避免与现有 Perspective Skills 竞争）

### Phase 3: 验证与示例（2-3 天）

6. **生成示例 Persona Skills**
   - `patient-math-teacher-persona`（小学数学老师）
   - `friendly-coding-mentor-persona`（编程导师）
   - `structured-hr-interviewer-persona`（HR 面试官）

7. **运行质量验证**
   - 对 3 个示例执行 Phase 4C 测试
   - 修正不一致之处

8. **测试触发准确性**
   - 用 Skill-Creator 的 description optimization 流程优化触发词
   - 确保 overtrigger/undertrigger 在可接受范围内

### Phase 4: 交付（1 天）

9. **更新文档**
   - README.md 添加 Persona 功能说明
   - 添加合成路径的 Usage 示例

10. **提交并推送**
    - git commit + push

---

## 六、风险与应对

| 风险 | 影响 | 应对措施 |
|------|------|---------|
| 合成角色过于"通用"，缺乏辨识度 | 用户觉得"像 ChatGPT" | 强化表达 DNA 的组合设计，添加更多具体规则 |
| 维度库覆盖不全 | 用户需要的维度不在预定义列表中 | 允许用户自定义维度，维度库持续迭代 |
| 与现有 Perspective Skills 冲突 | 触发竞争 | 明确的排他规则：已有 Skill → 直接用；组合需求 → 走合成路径 |
| 角色合成质量不稳定 | 不同维度组合效果差异大 | 添加维度协同推荐，引导用户选择合理的组合 |
| SKILL.md 变得更长 | 主文件可能超过 800 行 | 将三种路径的详细流程下放到 `references/path-a.md`、`path-c.md` |

---

## 七、总结

### 改版的本质

这不是对 Persona Forge 的推翻，而是**能力的扩展**：

- **保留**：人物蒸馏的完整方法论（6 Agent 调研、三重验证、质量门控）
- **新增**：角色合成的轻量级流程（维度组合、领域知识注入、一致性设计）
- **统一**：三种入口路径在同一 Skill 中无缝切换

### 核心价值

1. **降低使用门槛**：不需要等 6 个 Agent 调研完，3 分钟就能生成一个可用的角色
2. **扩展应用场景**：从"名人思维顾问"扩展到"任何角色的 AI 化身"
3. **保持质量底线**：合成角色也有完整的质量验证体系

### 最终愿景

```
用户：帮我创建一个小学数学老师的 persona

Persona-Skill：
  → 路径C触发
  → 展示维度库，用户选择/调整
  → 5分钟后交付 `patient-math-teacher-persona`
  → 用户可直接激活："切换到数学老师模式"
  → 老师用温暖鼓励的方式，通过类比和故事教数学
```

---

> 本方案基于 `docs/persona-forge-analysis.md` 的深度分析制定。
> 建议先实施 Phase 1-2（基础设施+核心流程），再验证 3 个示例，最后全量推广。
