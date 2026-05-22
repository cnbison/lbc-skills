# docs/ 开发文档维护指南

> 每个 skill 都是独立开发项目，开发过程信息同步记录到 `docs/<skill-name>/`。
> 核心原则：**记录价值 > 记录完整性**，不搞仪式感。

---

## 一、目录结构

```
docs/
├── _shared/                       # 跨 skill 的通用文档
│   ├── skill-format-comparison.md
│   └── skill-loading-paths.md
│
├── docs-maintenance-guide.md      # 本文件
│
├── persona-forge/                 # 每个 skill 一个独立目录
│   ├── README.md                  # 开发总览
│   ├── CHANGELOG.md               # 版本历史（含过程摘要）
│   └── DECISIONS.md               # 关键决策（按需创建，不是每次都有）
│
├── daily-news/
│   ├── README.md
│   └── CHANGELOG.md
│
└── ...
```

---

## 二、标准文件说明

### 1. README.md — 开发总览

什么时候创建：**新建 skill 时初始化**。

内容：
- Skill 定位（一句话）
- 当前状态（版本、活跃/维护中/稳定）
- 核心文件索引（快速跳转到 SKILL.md、关键 references 等）
- 下一步计划（一句话，指向 CHANGELOG）

什么时候更新：重大架构变更后、状态变化时。

### 2. CHANGELOG.md — 版本历史

什么时候创建：**新建 skill 时初始化**。

内容：按时间倒序，每个版本一段：

```markdown
## [未发布]

### Added
- xxx

## [v2.1] - 2026-05-21

### Fixed
- Path C 角色合成确认流程：新增硬规则禁止跳过用户确认
- 涉及文件：`SKILL.md`, `references/path-c.md`
```

**每次代码变更都必须更新**。条目要包含：
- 变更类型（Added / Changed / Fixed / Removed）
- 一句话描述
- 涉及的关键文件（方便追溯）

这就是过程记录。不需要再写 sessions/。

### 3. DECISIONS.md — 关键决策

什么时候创建：**做了架构/设计层面的决策时**，按需创建，不是每次都有。

内容：ADR（Architecture Decision Record）格式：

```markdown
## ADR-001: 为什么从 nuwa 改名为 Persona Forge

- **日期**：2026-05-20
- **背景**：原品牌有伦理敏感风险
- **决策**：全面品牌重塑
- **理由**：更准确描述定位；英文名利于国际化
- **后果**：所有文档需同步更新
```

---

## 三、更新规则

| 场景 | 必须更新 | 可选更新 |
|------|---------|---------|
| 新建 skill | `README.md` + `CHANGELOG.md`（初始条目） | `DECISIONS.md`（如有早期决策） |
| bug fix | `CHANGELOG.md` | — |
| 功能新增 | `CHANGELOG.md` | `DECISIONS.md`（如有设计决策） |
| 架构/设计决策 | `DECISIONS.md` | — |

**底线**：每次代码变更，CHANGELOG.md 必须同步更新。这是唯一不可跳过的文件。

---

## 四、与代码提交的关系

1. **一次 commit 同时包含**代码变更和对应的 `docs/<skill-name>/` 变更
2. `git add` 时检查：是否遗漏了 `docs/<skill-name>/CHANGELOG.md` 的更新
3. commit message 中如有 docs 变更，在末尾加一句：`文档同步更新：CHANGELOG.md`

---

## 五、现有 skill 的追溯

如某个 skill 的 `docs/<skill-name>/` 目录尚未创建，在**下次修改该 skill 时一并初始化**（创建 README.md 和 CHANGELOG.md，并补写历史条目）。

不需要一次性给所有 skill 补建目录，按需逐步进行。

---

## 六、结构性变更检查清单

**硬规则：每次进行结构性变更（改名、移动、删除文件/目录）后，必须执行一次全仓库引用检查，确认无残留旧引用后方可提交。**

### 6.1 变更前必执行的 grep 检查

```bash
# skill 改名/删除前：检查全仓库是否有旧名称残留
grep -rn "旧名称" . --include="*.md" --include="*.json"

# 文件/目录移动前：检查全仓库是否有旧路径残留
grep -rn "旧路径" . --include="*.md" --include="*.json"

# 删除文件前：检查是否有其他文件引用该路径
grep -rn "被删文件名" . --include="*.md"
```

### 6.2 常见场景检查点

| 场景 | 必须检查的文件 |
|------|--------------|
| skill 改名/品牌重塑 | `README.md`（目录表、触发示例）、`CLAUDE.md`（形态示例、速查引用）、所有 `docs/<name>/` 内部引用 |
| 文件/目录移动到 `_shared/` | `CLAUDE.md`（速查引用）、`README.md`（文档链接）、所有 SKILL.md 内部引用 |
| 删除文件 | 全仓库 grep 该文件名，确认无引用 |
| 修改触发词 | `README.md`（触发示例、目录表）、`SKILL.md`（description） |
| 修改输出路径 | `CLAUDE.md`（输出约定）、`SKILL.md`（Instructions）、所有相关脚本 |

### 6.3 commit 前快速自检命令

```bash
# 在仓库根目录执行，检查常见残留问题
grep -rn "nuwa\|女娲\|TODO\|FIXME\|xxx" skills/ .claude/skills/ docs/ CLAUDE.md README.md
```

**底线**：结构性变更的 commit 中，如果 grep 检查发现了旧引用残留，必须修复后才能提交。宁可多一次 commit，也不留过时引用。
