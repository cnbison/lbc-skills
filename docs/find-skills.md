# find-skills Skill 深度解析与使用指南

> **Skill 路径**：`skills/find-skills/SKILL.md`
> **来源**：Anthropic 官方 skills 仓库（通过 `npx skills add` 引入）
> **形态**：纯 prompt skill（无脚本、无外部依赖）

---

## 一、Skill 定位：Agent 能力的"应用商店入口"

`find-skills` 是一个**辅助发现型 skill**，它不是直接完成某个任务，而是帮助用户找到**能完成某个任务的其他 skill**。换句话说，它是 Claude Code skill 生态的"搜索引擎 + 应用商店导购"。

**核心价值主张**：

当用户说"我想做 X"时，Claude 通常用自己的通用能力直接做。但 `find-skills` 提供了一个更优路径——先检查生态里是否已有专门做 X 的 skill，如果有，引导用户安装使用（更专业、更系统）；如果没有，再回退到通用能力。

---

## 二、触发条件：六类典型场景

| 场景类型 | 示例用户输入 |
|---------|------------|
| **能力询问** | "how do I do X" |
| **主动寻找** | "find a skill for X" / "is there a skill for X" |
| **功能扩展** | "can you do X"（X 是专业领域） |
| **能力表达** | "I wish I had help with..." |
| **工具搜索** | "search for tools/templates/workflows" |
| **领域求助** | 提到具体领域（design, testing, deployment 等） |

**与 `nuwa-skill` 的区别**：

| 维度 | find-skills | nuwa-skill |
|------|-------------|-----------|
| 目标 | 找到**已有的** skill | **创建新的** skill |
| 输入 | "我想做 X" | "我想拥有 XX 的思维方式" |
| 输出 | 安装命令 + 使用说明 | 完整的 SKILL.md |
| 前提 | 生态里已有类似 skill | 生态里没有，需要从零造 |

---

## 三、工作流设计：六步闭环

```
┌─────────────────────────────────────────────────┐
│ Step 1: 理解需求（domain + task + 常见度判断）      │
├─────────────────────────────────────────────────┤
│ Step 2: 查 Leaderboard（skills.sh 热门榜）          │
├─────────────────────────────────────────────────┤
│ Step 3: CLI 搜索（npx skills find [query]）        │
├─────────────────────────────────────────────────┤
│ Step 4: 质量验证（install count / source / stars）  │
├─────────────────────────────────────────────────┤
│ Step 5: 呈现选项（名称 + 安装数 + 命令 + 链接）      │
├─────────────────────────────────────────────────┤
│ Step 6: 协助安装（npx skills add -g -y）           │
└─────────────────────────────────────────────────┘
```

### 3.1 第一步：需求解析

Skill 要求 Claude 从用户问题中提取三个要素：
1. **Domain**（领域）：React、testing、design 等
2. **Task**（具体任务）：writing tests、creating animations 等
3. **常见度判断**：这是不是足够通用的任务，以至于大概率已有 skill

### 3.2 第二步：Leaderboard 优先

**设计意图**：避免一上来就跑 CLI 搜索。如果用户要的是"React 性能优化"，而 skills.sh  leaderboard 上已经有 `vercel-labs/agent-skills` 这个 100K+ 安装量的官方包，直接推荐它比搜索更高效。

### 3.3 第四步：质量验证（关键设计）

这是该 skill 最体现"防御性思维"的部分：

| 指标 | 阈值 | 风险等级 |
|------|------|---------|
| **Install count** | < 100 | 高度怀疑 |
| **Install count** | 1K+ | 推荐 |
| **Source** | 官方（vercel-labs, anthropic, microsoft） | 可信 |
| **Source** | 未知作者 | 需额外审查 |
| **GitHub stars** | < 100 | 高度怀疑 |

**为什么这很重要**：Skill 生态目前处于野蛮生长阶段，任何人都可以发布 skill。如果没有质量 gate，用户可能被引导安装低质量甚至恶意的 skill。

### 3.4 第六步：安装机制

```bash
npx skills add <owner/repo@skill> -g -y
```

- `-g`：全局安装（用户级，所有项目可用）
- `-y`：跳过确认

**注意**：`npx skills` 的默认安装路径是 `~/.agents/skills/`，不是 `.claude/skills/`。这意味着：

1. 安装后 Claude Code **不会自动识别**（它不扫描 `.agents/`）
2. 需要手动软链接或复制到 `.claude/skills/` 才能生效
3. 这一点在 skill 的说明中**没有明确提及**，是本仓库迁移 `skill-creator` 时踩过的坑

---

## 四、技能分类速查表

Skill 内置了 7 个常见分类，作为搜索时的思维框架：

| 分类 | 示例关键词 |
|------|----------|
| Web Development | react, nextjs, typescript, css, tailwind |
| Testing | testing, jest, playwright, e2e |
| DevOps | deploy, docker, kubernetes, ci-cd |
| Documentation | docs, readme, changelog, api-docs |
| Code Quality | review, lint, refactor, best-practices |
| Design | ui, ux, design-system, accessibility |
| Productivity | workflow, automation, git |

---

## 五、搜索技巧

1. **具体优于宽泛**：`react testing` > `testing`
2. **多词尝试**：`deploy` 不行就试 `deployment` 或 `ci-cd`
3. **优先查官方源**：`vercel-labs/agent-skills`、`anthropics/skills`、`ComposioHQ/awesome-claude-skills`

---

## 六、失败处理：找不到 skill 时

当搜索无结果时，skill 要求 Claude 做三件事：
1. **承认未找到**
2. **提供兜底方案**："我可以用通用能力直接帮你做"
3. **引导创建**："如果你经常做这件事，可以创建一个自己的 skill"

```bash
npx skills init my-xyz-skill
```

---

## 七、与 lbc-skills 管理模式的对比

`find-skills` 代表了一种**"去中心化、包管理器驱动"**的 skill 管理模式，而 `lbc-skills` 采用的是**"中心化、源码仓库 + 软链接"**模式。两者各有优劣：

| 维度 | find-skills / npx skills | lbc-skills 模式 |
|------|-------------------------|----------------|
| **发现方式** | skills.sh leaderboard + CLI 搜索 | README 目录表 + GitHub 浏览 |
| **安装方式** | `npx skills add`（包管理器） | `git clone` + `ln -s`（源码管理） |
| **安装路径** | `~/.agents/skills/` | `.claude/skills/`（软链接） |
| **版本控制** | `skills-lock.json`（锁定版本） | `git`（完整版本历史） |
| **定制化** | 依赖官方更新 | 可自由修改源码 |
| **质量审查** | install count + stars（社区投票） | 人工审核 + 分类标注（WIP/active） |
| **跨项目复用** | `-g` 全局安装 | 每个项目单独软链接 |
| **与 Claude Code 集成** | ❌ 需额外软链接步骤 | ✅ 直接识别 |

**本仓库的实践经验**：

我们同时使用了两种模式：
- `skill-creator` 和 `find-skills` 通过 `npx skills add` 引入（去中心化）
- 其他 19 个 skill 通过源码 + 软链接管理（中心化）

踩过的坑：`npx skills` 装到 `.agents/` 后 Claude Code 看不到，必须手动迁移到 `skills/` 并软链接到 `.claude/skills/`。

---

## 八、设计亮点

### 8.1 防御性推荐机制

强制要求验证 install count、source reputation 和 GitHub stars，而不是简单把搜索结果抛给用户。这在 skill 生态早期（质量参差不齐）尤为重要。

### 8.2 Leaderboard 优先策略

先查热门榜再跑搜索，符合"80/20 法则"——大多数常见需求已经被头部 skill 覆盖。

### 8.3 失败兜底设计

找不到 skill 时不让用户"卡住"，而是提供两条出路：
1. 用通用能力直接做（短期解决）
2. 创建一个 skill（长期解决）

---

## 九、局限性与注意事项

### 9.1 安装路径问题（已验证）

`npx skills add` 默认安装到 `~/.agents/skills/` 或 `./.agents/skills/`，Claude Code **不会自动加载**。用户需要：

```bash
# 方式一：复制到本仓库的 skills/ 并软链接
cp -R ~/.agents/skills/some-skill skills/some-skill
ln -s ../../skills/some-skill .claude/skills/some-skill

# 方式二：直接软链接（但不利于版本控制）
ln -s ~/.agents/skills/some-skill .claude/skills/some-skill
```

### 9.2 信息时效性

skills.sh leaderboard 和 install count 是动态变化的，但 skill 的 prompt 中列出的示例数据（如 "100K+ installs"）可能过时。实际使用时应该跑实时 CLI 搜索，而不是依赖 prompt 中的静态示例。

### 9.3 中文生态覆盖有限

skills.sh 目前以英文 skill 为主，`lbc-skills` 这样的中文 skill 集合在 leaderboard 上几乎没有曝光。对于中文用户，本仓库的 README 目录表可能比 `npx skills find` 更实用。

---

## 十、使用场景

| 场景 | 用户输入示例 | 预期行为 |
|------|------------|---------|
| **找现成工具** | "我想给 React 项目加测试" | 推荐 `vercel-labs/agent-skills` 中的 testing skill |
| **确认能力边界** | "你能帮我做代码审查吗？" | 搜索 `review` / `pr review` skill，有则推荐，无则兜底 |
| **扩展能力** | "我希望你能帮我写 changelog" | 搜索 `changelog` skill |
| **不知道有没有** | "有没有处理 PDF 的 skill？" | 搜索 `pdf` / `document` skill |
| **创建新 skill** | "我经常需要做 X，但没有 skill" | 引导 `npx skills init` |

---

## 十一、总结

`find-skills` 是一个**轻量但设计精良的辅助 skill**，它填补了 Claude Code 生态中的一个空白：用户知道"我想做 X"，但不知道"有没有专门做 X 的 skill"。

它的六步工作流（理解需求 → 查排行榜 → 搜索 → 验证质量 → 呈现 → 安装）是一个完整的用户引导闭环，特别是第四步的质量验证，体现了对生态现状的清醒认知。

**但需要注意**：由于 `npx skills` 与 Claude Code 的加载路径不完全兼容，实际使用中可能需要手动调整安装位置。对于 `lbc-skills` 这样的自管型仓库，更推荐直接浏览 README 目录表或搜索本仓库的 `skills/` 目录。
