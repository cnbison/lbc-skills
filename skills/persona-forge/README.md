<div align="center">

# Persona Forge.skill

> *「人格化一切——名人思维，或你创造的独特角色」*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-Skill-blueviolet)](https://claude.ai/code)
[![Skills](https://img.shields.io/badge/skills.sh-Compatible-green)](https://skills.sh)

<br>

**Persona Forge 是人格化引擎：既能蒸馏名人的思维方式，也能按你的需求组合出全新角色。**

<br>

[效果示例](#效果示例) · [安装](#安装) · [它能做什么](#它能做什么) · [工作原理](#工作原理)

<br>

**其他语言:**
[English](README_EN.md)


</div>

---

## 效果示例

### 示例1：蒸馏名人思维

```
用户      ❯ 我同时想做自媒体、写书、做独立开发，精力完全不够

Naval     ❯ 你列了三个欲望。每个欲望都是你跟不快乐签的一份合同。
            不是精力不够，是合同太多。

            问你自己：哪一个做起来你会忘记时间？
            那个就是你的特定知识所在。

            不是三选一。是先一，再一，再一。
            串行复利，不是并行内耗。
```

```
用户      ❯ 我们的SaaS产品获客成本太高了

马斯克     ❯ 先别想怎么降。先算物理极限。
            获客的最小必要动作是什么？用户从知道你到付费，
            信息传递的理论最短路径是什么？

            现在你的实际路径是理论路径的几倍？
            如果超过3倍，中间一定有可以砍掉的步骤。
            不是优化漏斗，是质疑漏斗本身该不该存在。
```

### 示例2：合成定制角色

```
用户      ❯ 创建一个小学数学老师的 persona

Persona Forge ❯ 已生成「耐心数学老师」角色。

用户      ❯ 为什么 1/2 加 1/3 不等于 2/5？

耐心数学老师 ❯ 这个想法很有意思！很多小朋友一开始都会这么想。
               我们来做个实验：假设你有一块披萨...

               （角色基于「温暖鼓励 + 类比密集 + 引导发现」维度组合设计）
```

这不是角色扮演。Naval 用的是「欲望即合同」，马斯克用的是「渐近极限法」，小学老师用的是「支架式教学 + 类比启发」。**它们不是在复读模板，是在用特定的认知框架帮你分析。**

---

## 安装

```bash
npx skills add cnbison/lbc-skills@persona-forge
```

然后在 Claude Code 里：

**蒸馏名人：**
```
> 蒸馏一个保罗·格雷厄姆
> 造一个张小龙的视角 Skill
> 帮我做一个段永平的 Skill
```

**合成角色：**
```
> 创建一个小学数学老师的 persona
> 做一个耐心但严格的导师角色
> 设计一个适合教孩子编程的 AI 伙伴
```

造完之后直接调用：
```
> 用芒格的视角帮我分析这个投资决策
> 费曼会怎么解释量子计算？
> 切换到数学老师，给我讲分数
```

---

## 它能做什么

Persona Forge 提取五层认知结构：

| 层次 | 说明 |
|---|---|
| **怎么说话** | 表达 DNA——语气、节奏、用词偏好 |
| **怎么想** | 心智模型、认知框架 |
| **怎么判断** | 决策启发式 |
| **什么不做** | 反模式、价值观底线 |
| **知道局限** | 诚实边界 |

### 两种能力

**1. 人物蒸馏** —— 输入人名，6 个 Agent 并行调研真实人物的著作、访谈、决策记录，提取其独特的认知操作系统。

**2. 角色合成** —— 不依赖特定人物，从「角色维度库」中选择/组合维度（教学风格、沟通温度、表达方式等），5 分钟生成一个定制角色。

### 诚实边界

每个 Skill 都明确标注做不到什么：

- 蒸馏不了直觉——框架能提取，灵感不能
- 捕捉不了突变——截止到调研时间的快照
- 公开表达 ≠ 真实想法——只能基于公开信息
- 合成角色 ≠ 真实人物——明确标注为 AI 生成角色

**一个不告诉你局限在哪的 Skill，不值得信任。**

---

## 已生成 Skill

Persona Forge 已生成 13 位人物 + 1 个主题。每个都是独立的、可直接安装使用的 Skill：

### 人物 Skill（蒸馏）

| 人物 | 领域 | 独立仓库 | 一键安装 |
|------|------|---------|---------|
| 🔥 **Paul Graham** | 创业/写作/产品/人生哲学 | [paul-graham-skill](https://github.com/alchaincyf/paul-graham-skill) | `npx skills add alchaincyf/paul-graham-skill` |
| 🔥 **张一鸣** | 产品/组织/全球化/人才 | [zhang-yiming-skill](https://github.com/alchaincyf/zhang-yiming-skill) | `npx skills add alchaincyf/zhang-yiming-skill` |
| 🔥 **Karpathy** | AI/工程/教育/开源 | [karpathy-skill](https://github.com/alchaincyf/karpathy-skill) | `npx skills add alchaincyf/karpathy-skill` |
| 🔥 **Ilya Sutskever** | AI安全/scaling/研究品味 | [ilya-sutskever-skill](https://github.com/alchaincyf/ilya-sutskever-skill) | `npx skills add alchaincyf/ilya-sutskever-skill` |
| 🔥 **MrBeast** | 内容创造/YouTube方法论 | [mrbeast-skill](https://github.com/alchaincyf/mrbeast-skill) | `npx skills add alchaincyf/mrbeast-skill` |
| 🔥 **特朗普** | 谈判/权力/传播/行为预判 | [trump-skill](https://github.com/alchaincyf/trump-skill) | `npx skills add alchaincyf/trump-skill` |
| ⭐ **乔布斯** | 产品/设计/战略 | [steve-jobs-skill](https://github.com/alchaincyf/steve-jobs-skill) | `npx skills add alchaincyf/steve-jobs-skill` |
| **马斯克** | 工程/成本/第一性原理 | [elon-musk-skill](https://github.com/alchaincyf/elon-musk-skill) | `npx skills add alchaincyf/elon-musk-skill` |
| **芒格** | 投资/多元思维/逆向思考 | [munger-skill](https://github.com/alchaincyf/munger-skill) | `npx skills add alchaincyf/munger-skill` |
| **费曼** | 学习/教学/科学思维 | [feynman-skill](https://github.com/alchaincyf/feynman-skill) | `npx skills add alchaincyf/feynman-skill` |
| **纳瓦尔** | 财富/杠杆/人生哲学 | [naval-skill](https://github.com/alchaincyf/naval-skill) | `npx skills add alchaincyf/naval-skill` |
| **塔勒布** | 风险/反脆弱/不确定性 | [taleb-skill](https://github.com/alchaincyf/taleb-skill) | `npx skills add alchaincyf/taleb-skill` |
| **张雪峰** | 教育/职业规划/阶层流动 | [zhangxuefeng-skill](https://github.com/alchaincyf/zhangxuefeng-skill) | `npx skills add alchaincyf/zhangxuefeng-skill` |

### 主题 Skill

| 主题 | 领域 | 独立仓库 | 一键安装 |
|------|------|---------|---------|
| **X导师** | X/Twitter运营全栈 | [x-mentor-skill](https://github.com/alchaincyf/x-mentor-skill) | `npx skills add alchaincyf/x-mentor-skill` |

人物 Skill 蒸馏一个人的思维方式；主题 Skill 蒸馏一个领域的方法论。每个仓库都包含完整的调研数据和效果示例对话。

想蒸馏不在列表里的人或主题？安装 Persona Forge，说「蒸馏一个 XXX」就行。
想创造一个自定义角色？说「创建一个 XXX 的 persona」就行。

---

## 达尔文.skill：让所有 Skill 持续进化

<div align="center">

<a href="https://github.com/alchaincyf/darwin-skill">
<img src="https://raw.githubusercontent.com/alchaincyf/darwin-skill/master/assets/banner.svg" alt="达尔文.skill" width="600">
</a>

</div>

Persona Forge 造 Skill，**[达尔文](https://github.com/alchaincyf/darwin-skill)** 让 Skill 进化。

受 Karpathy autoresearch 启发，达尔文.skill 用自主实验循环批量优化所有 Skill：8 维度评估、棘轮机制（只保留改进，自动回滚退步）、独立子 agent 评分。Persona Forge 的 Phase 5 双 Agent 精炼就内置了达尔文的评估体系，这也是 Persona Forge 生成的 Skill 质量高的原因之一。

```bash
npx skills add alchaincyf/darwin-skill
```

---

## 工作原理

### 人物蒸馏流程

输入一个名字后，Persona Forge 做四件事：

**1. 六路并行采集**——著作、播客/访谈、社交媒体、批评者视角、决策记录、人生时间线，6 个 Agent 同时跑，各自存档。

**2. 三重验证提炼**——一个观点要被收录为心智模型，必须：跨 2+ 个领域出现过（不是随口一说）、能推断对新问题的立场（有预测力）、不是所有聪明人都会这么想（有排他性）。三个都过才收录。

**3. 构建 Skill**——3-7 个心智模型 + 5-10 条决策启发式 + 表达 DNA + 价值观与反模式 + 诚实边界，写入 SKILL.md。

**4. 质量验证**——拿 3 个此人公开回答过的问题测试，方向一致才通过。再用 1 个他没讨论过的问题测试，Skill 应该表现出适度不确定而非斩钉截铁。

### 角色合成流程

输入角色描述后：

**1. 维度组合**——从角色维度库（教学风格、沟通温度、表达方式、互动模式等）中选择/调整维度。

**2. 领域知识注入**——1 个 Agent 搜索该角色的领域通用方法论（如"小学数学教学法的最佳实践"）。

**3. 角色一致性设计**——基于维度组合 + 领域知识，推导心智模型、决策启发式、表达 DNA。

**4. 组装与验证**——使用扩展模板生成 Skill，执行角色一致性测试 + 维度偏离检测 + 领域适切性测试。

完整方法论在 `references/extraction-framework.md`（人物蒸馏）和 `references/persona-dimension-library.md`（角色合成）。

---

## 仓库结构

```
persona-forge/
├── SKILL.md                      # Persona Forge 本体（三路径入口 + 精简索引）
├── references/
│   ├── extraction-framework.md   # 提炼方法论（三重验证、质量自检）
│   ├── skill-template.md         # 生成 Skill 的模板（人物 + 合成角色）
│   ├── persona-dimension-library.md  # 角色维度库（合成角色用）
│   ├── path-a.md                 # 人物蒸馏详细流程（Phase 1-5）
│   └── path-c.md                 # 角色合成详细流程（Phase 1C-5C）
└── examples/                          # 13 个人物 + 1 个主题
    ├── steve-jobs-perspective/        # 乔布斯
    ├── paul-graham-perspective/       # Paul Graham
    ├── zhang-yiming-perspective/      # 张一鸣
    └── ...
```

调研过程全透明。每个 example 都包含完整的调研文件，你可以看到信息怎么被收集、筛选、变成心智模型。

---

## 关于

Persona Forge 不复制人。它提取认知操作系统。

一个好的人物 Skill，让你用另一个人的眼睛看自己的问题。一个好的人格化角色，让 AI 以你需要的风格和能力与你互动。不是为了模仿他们，而是为了拓展你自己的思维边界和互动体验。

**Persona Forge** —— 人格化引擎。

---

## 许可证

MIT — 随便用，随便改，随便造。

---

<div align="center">

**Persona Forge** —— 人格化一切。

*下一个你想对话的思维，何必只能是名人。*

</div>
