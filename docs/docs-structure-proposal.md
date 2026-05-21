# docs/ 目录规范化方案（已确认·精简版）

> 状态：**已确认**，按本方案执行。
> 核心原则：记录价值 > 记录完整性，不搞仪式感。

---

## 一、问题

当前 `docs/` 是平铺杂乱的：
- 各种 skill 的文档混合在一起，没有归属
- 开发过程信息散落在聊天记录中，无法回溯
- 新开发者无法快速了解某个 skill 的历史和当前状态

---

## 二、方案：按 Skill 分目录的精简体系

### 2.1 目录结构

```
docs/
├── README.md                      # docs/ 目录本身的使用说明
├── docs-maintenance-guide.md      # 完整的维护规则（独立文档）
│
├── _shared/                       # 跨 skill 的通用文档
│   ├── skill-format-comparison.md
│   └── skill-loading-paths.md
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

### 2.2 为什么取消 sessions/ 和 BACKLOG.md

| 取消的文件 | 理由 | 替代方案 |
|-----------|------|---------|
| `sessions/` | 重复记录。每次开发的完整过程已在 git commit message 和 git diff 中 | `CHANGELOG.md` 的详细条目承载过程摘要 |
| `BACKLOG.md` | 增加维护负担，大多数 skill 不需要独立的需求池 | `CHANGELOG.md` 的 `[未发布]` 区块代替 |

### 2.3 标准文件说明

| 文件 | 用途 | 什么时候更新 |
|------|------|-------------|
| `README.md` | 开发总览：定位、状态、文件索引、下一步 | 重大架构变更后 |
| `CHANGELOG.md` | 版本历史 + 过程摘要 | **每次代码变更都必须更新** |
| `DECISIONS.md` | 关键决策记录（ADR 格式） | 做了架构/设计决策时，按需创建 |

---

## 三、开发工作流（SOP）

```
1. 开发前
   → 明确本次开发目标

2. 开发后
   → 更新 CHANGELOG.md（必须）
   → 如有架构决策，更新 DECISIONS.md（按需）
   → 如有状态变化，更新 README.md（按需）

3. 提交
   → git add 同时添加代码变更和 docs/ 变更
   → 一次 commit 包含：代码 + 文档更新
   → commit message 描述代码变更，末尾加"文档同步更新：CHANGELOG.md"
```

---

## 四、现有文档迁移计划

| 现有文件 | 迁移目标 |
|---------|---------|
| `docs/nuwa-persona-redesign.md` | `docs/persona-forge/DECISIONS.md`（ADR-00X） |
| `docs/nuwa-skill-analysis.md` | `docs/persona-forge/CHANGELOG.md`（补历史条目） |
| `docs/persona-forge.md` | 拆分为 `docs/persona-forge/README.md` + `CHANGELOG.md` + `DECISIONS.md` |
| `docs/skill-format-comparison.md` | `docs/_shared/skill-format-comparison.md` |
| `docs/skill-loading-paths.md` | `docs/_shared/skill-loading-paths.md` |
| `docs/claudemd_zh.md` | `docs/_shared/claudemd_zh.md` |

**原则**：不需要一次性迁移所有文件，在下次修改对应 skill 时逐步进行。

---

## 五、CLAUDE.md 中的引用

在 CLAUDE.md 的"工作时的默认动作"章节末尾增加一句话：

```
6. 每次代码变更需同步更新 docs/<skill-name>/CHANGELOG.md
   详细规则见 docs/docs-maintenance-guide.md
```

---

## 六、好处

1. **每个 skill 的开发上下文自包含**：打开 `docs/persona-forge/` 就能看到历史和决策
2. **开发过程可追溯**：CHANGELOG.md 记录了每次变更的内容和原因
3. **与代码提交同步**：一次 commit 同时包含代码和文档
4. **不增加维护负担**：只保留 2-3 个文件，没有 sessions/ 的重复记录
5. **CLAUDE.md 不膨胀**：详细规则放在独立文档，主文档只保留一句话引用

---

## 七、执行状态

- [x] 方案确认（2026-05-21）
- [ ] 创建 `docs/docs-maintenance-guide.md`
- [ ] 在 `CLAUDE.md` 中增加引用
- [ ] 初始化 `docs/persona-forge/` 目录（README + CHANGELOG + DECISIONS）
- [ ] 迁移现有 persona-forge 相关文档
- [ ] 创建 `docs/_shared/` 并迁移通用文档
