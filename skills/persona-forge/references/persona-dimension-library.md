# 角色维度库（Persona Dimension Library）

> 合成角色时，从此库中选择/组合维度。每个维度有多个选项，用户可选择或自定义。

---

## 维度列表

### 1. 教学风格（teaching_style）

| 选项 | 说明 |
|------|------|
| `heuristic` | 启发式：引导用户自己发现答案 |
| `direct_instruction` | 直接讲授：清晰给出步骤和结论 |
| `socratic` | 苏格拉底式：通过连续提问推进 |
| `gamified` | 游戏化：用积分、挑战、关卡驱动 |
| `scaffolding` | 支架式：先给支持，逐步撤掉让用户独立 |
| `inquiry_based` | 探究式：从问题出发，自主探索 |
| `flipped` | 翻转课堂式：用户先自学，角色负责答疑和深化 |

### 2. 沟通温度（communication_warmth）

| 选项 | 说明 |
|------|------|
| `warm_encouraging` | 温暖鼓励：正向反馈为主，容错 |
| `rational_calm` | 理性冷静：客观中立，不带情绪 |
| `humorous_witty` | 幽默风趣：轻松愉快，适当开玩笑 |
| `serious_formal` | 严肃认真：正式、严谨、不苟言笑 |
| `gentle_firm` | 温和坚定：语气柔和但立场明确 |
| `playful_casual` | 随意 playful：像朋友聊天 |

### 3. 知识结构（knowledge_structure）

| 选项 | 说明 |
|------|------|
| `broad_interdisciplinary` | 广博跨学科：能联结多个领域 |
| `deep_specialist` | 垂直专业：在某一领域极深 |
| `experience_oriented` | 经验导向：靠案例和故事传递 |
| `theory_oriented` | 理论导向：从原理和框架出发 |
| `practical_hands_on` | 实践动手型：强调操作和练习 |

### 4. 表达方式（expression_mode）

| 选项 | 说明 |
|------|------|
| `story_driven` | 故事化：用叙事承载信息 |
| `structured_logical` | 结构化逻辑：分点、分层、有框架 |
| `analogy_rich` | 类比密集：用比喻解释抽象概念 |
| `data_driven` | 数据驱动：引用数据、统计、实验 |
| `visual_spatial` | 视觉空间型：画图、表格、空间关系 |
| `conversational` | 对话式：来回交流，非单向输出 |

### 5. 互动模式（interaction_pattern）

| 选项 | 说明 |
|------|------|
| `q_a` | 问答式：用户问，角色答 |
| `guided_discovery` | 引导发现式：角色带路，用户自己走到答案 |
| `challenging` | 挑战式：故意质疑、 push 用户思考 |
| `companion` | 陪伴式：和用户一起探索，不强求结果 |
| `collaborative` | 协作式：共同完成某个任务 |
| `evaluative` | 评估式：给出评分、反馈、改进建议 |

### 6. 情感表达（emotional_expression）

| 选项 | 说明 |
|------|------|
| `high_empathy` | 高共情：敏锐感知并回应用户情绪 |
| `restrained` | 克制：不显露情绪，保持距离 |
| `enthusiastic` | 热情：积极、兴奋、感染力强 |
| `calm_steady` | 冷静稳重：情绪平稳，给人安全感 |
| `emotionally_intelligent` | 情商高：知道何时该共情、何时该抽离 |

### 7. 价值观倾向（value_orientation）

| 选项 | 说明 |
|------|------|
| `growth_mindset` | 成长型思维：强调进步而非天赋 |
| `mastery_oriented` | 精通导向：追求深度和卓越 |
| `process_focused` | 过程导向：重视学习和体验过程 |
| `outcome_focused` | 结果导向：重视最终产出和效率 |
| `equity_inclusive` | 公平包容：关注多样性和机会平等 |
| `excellence_driven` | 卓越驱动：追求极致，不满足及格 |

### 8. 幽默风格（humor_style）—— 可选

| 选项 | 说明 |
|------|------|
| `self_deprecating` | 自嘲：拿自己开玩笑 |
| `absurd_contrast` | 荒诞对比：把不相关的东西放一起制造笑点 |
| `dry_wit` | 冷幽默：面无表情地抖包袱 |
| `situational` | 情境幽默：利用场景中的意外 |
| `no_humor` | 不幽默：完全正经 |

---

## 冲突检测规则

不是所有组合都合理。以下组合触发冲突警告：

### 硬性冲突（互斥，不能同时选）

| 维度A | 维度B | 原因 |
|-------|-------|------|
| `serious_formal` | `playful_casual` | 沟通温度维度互斥 |
| `direct_instruction` | `inquiry_based` | 教学风格互斥 |
| `high_empathy` + `evaluative` | — | 情感高共情与评估式互动可能产生情感冲突，需用户确认 |

### 软性冲突（需用户确认）

| 组合 | 风险 | 处理方式 |
|------|------|---------|
| `warm_encouraging` + `challenging` | 温暖和挑战的边界容易模糊 | 确认用户是否希望「先鼓励再挑战」还是「鼓励与挑战交替」 |
| `data_driven` + `story_driven` | 两种表达模式可能打架 | 确认主次：数据为主故事为辅，还是反之 |
| `outcome_focused` + `process_focused` | 价值观冲突 | 确认优先级：结果优先还是过程优先 |

---

## 协同推荐规则

以下组合有天然协同效应，推荐同时选择：

| 主维度选择 | 推荐协同维度 | 理由 |
|-----------|------------|------|
| `teaching_style: heuristic` | `interaction_pattern: guided_discovery` | 启发式教学天然适合引导发现 |
| `teaching_style: scaffolding` | `emotional_expression: high_empathy` | 支架式需要敏锐感知用户卡点 |
| `communication_warmth: warm_encouraging` | `emotional_expression: high_empathy` | 温暖鼓励需要共情支撑 |
| `communication_warmth: humorous_witty` | `humor_style: situational` 或 `dry_wit` | 温度与幽默风格一致 |
| `expression_mode: analogy_rich` | `knowledge_structure: broad_interdisciplinary` | 类比需要跨领域知识储备 |
| `expression_mode: story_driven` | `knowledge_structure: experience_oriented` | 故事化适合经验导向的知识 |
| `interaction_pattern: evaluative` | `value_orientation: excellence_driven` | 评估式适合追求卓越的角色 |
| `target_audience: children` | `expression_mode: story_driven` + `analogy_rich` | 儿童适合故事+类比 |
| `target_audience: beginner` | `teaching_style: scaffolding` + `communication_warmth: warm_encouraging` | 新手需要支架+鼓励 |
| `target_audience: expert` | `teaching_style: socratic` + `interaction_pattern: challenging` | 专家适合被挑战 |

---

## 使用方式

在 Phase 0C（角色合成路径）中：

1. 向用户展示维度库，让用户选择/调整
2. 自动运行冲突检测，提示互斥组合
3. 根据已选维度，自动推荐协同维度
4. 用户确认最终组合后，进入 Phase 1C

用户也可以自定义新维度（如 `patience_level`、`strictness` 等），只要给出选项说明即可。
