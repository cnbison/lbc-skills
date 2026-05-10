# `second-brain` Skill 深度解析

> 对 `.claude/skills/second-brain` 的设计、实现与使用边界的剖析。
> 源文件：`skills/second-brain/SKILL.md`、`skills/second-brain/scripts/ensue-api.sh`

## 一、定位

**这不是笔记工具，而是「带检索能力的理解沉淀系统」。**

核心论断（SKILL.md:13-19）：保存的不是事实而是**理解**，写给"未来忘了上下文的自己"看。判断保存与否的唯一指标：

> *Will future-me thank me for this?*

## 二、技术栈

| 层 | 实现 |
|---|---|
| 存储后端 | Ensue Network（`api.ensue-network.ai`，JSON-RPC 2.0） |
| 语义检索 | `discover_memories`（带 `embed` 字段，向量化） |
| 调用方式 | `scripts/ensue-api.sh` 这一个 bash 包装，curl + Bearer Token |
| 鉴权 | `ENSUE_API_KEY` 环境变量 |
| 元数据 | `metadata.clawdbot` 字段声明 emoji、必需 env、homepage（典型 clawdbot 集成约定） |

`scripts/ensue-api.sh:33-37` 揭示了实际协议：所有方法都是 `tools/call` 的 MCP-style 调用，6 个固定 method（`list_keys` / `get_memory` / `create_memory` / `update_memory` / `delete_memory` / `discover_memories`）。

## 三、信息架构（关键设计）

```
public/                  # 可分享、长青
  concepts/[domain]/     # 「为什么 / 如何工作」
  toolbox/[category]/    # 用过的工具，含 _index 主索引
  patterns/[domain]/     # 可复用方案
  references/[topic]/    # 速查
private/
  notes/                 # 草稿
  journal/               # 日记
```

设计巧思：

1. **public/private 二分** — 物理隔离个人内容与可分享内容，便于将来导出/分享
2. **强制 domain 分层** — 避免"一锅炖"，所有 key 都是 `<namespace>/<domain>/<name>` 三段式
3. **`_index` 约定** — toolbox 维护一份人工写的导航文件，弥补 LLM 在"跨条目导航"上的弱点

## 四、四种内容模板（SKILL.md:46-131）

| 模板 | 必填字段 | 设计意图 |
|---|---|---|
| **Concept** | What it is / Why it matters / How it works / **Key insight** / Related | "Key insight" 是关键 — 强迫记录"啊哈时刻"，而非纯描述 |
| **Toolbox** | Category / Cost / What / **Why I use it** / When to reach for it / **Gotchas** | "Why I use it" 强制第一人称体验，"Gotchas" 沉淀踩坑 |
| **Pattern** | Problem / Solution / **Trade-offs** / Example | "Trade-offs" 防止把 pattern 当银弹 |
| **Reference** | 表格/列表/代码片段，"minimal prose, maximum signal" | 速查不需要叙事 |

## 五、交互协议

**关键约束（SKILL.md:135-141, 161-166）：**

- ✅ **保存前必须确认** — 永不 auto-save
- ✅ **先 search，后保存** — 防重复
- ❌ **不保存没用过的工具**
- ❌ **不保存半懂的概念**
- ❌ **不保存任何 secrets / API key / 个人路径**

意图映射表（SKILL.md:234-244）做得很完整，覆盖了 CRUD + 检索的全部入口。

## 六、与其他 skill 的关系

| 维度 | second-brain | nuwa-skill | ljg-read |
|---|---|---|---|
| 定位 | 个人知识库 | 造 skill 的 skill | 伴读 |
| 写入方 | 用户主动 | 生成 skill 文件 | 不写入 |
| 长期价值 | **复利** | 一次性产出 | 一次性陪伴 |

second-brain 在 lbc-skills 里相对独特 — 其他 skill 多是"内容生成"（日报、播客、伴读），它是**唯一一个把 Claude 当作长期知识管家**的 skill。

## 七、潜在问题 / 注意点

1. **依赖外部服务** — Ensue 一旦下线，知识库即不可用。无本地导出机制。
2. **无版本历史** — `update_memory` 是覆盖式，旧内容丢失。
3. **embedding 一致性** — `embed: true` 但未说明模型、维度、能否换 provider。
4. **`_index` 需手动维护** — 容易和实际 keys 漂移，没有"重建索引"的命令。
5. **bash 脚本鲁棒性** — `ensue-api.sh:37` 用 `sed 's/^data: //'` 暗示后端可能是 SSE，但脚本未处理多行 SSE 流，只去了首行 prefix。

## 八、最佳使用场景

适合：

1. 跨项目复用的**架构概念**（CRDT、event sourcing 之类）
2. 评估过、用过、踩过坑的**技术选型**（toolbox 是杀手级 use case）
3. 自己反复重新发明的**模式**（如"如何设计 retry"）
4. **外部 API/CLI 的速查表**（references）

不适合：

- 项目专属知识（应该留在项目 README）
- 时效性强的内容（"2024 年 React 19 changelog"）
- 没真正消化的资料（应该先用 ljg-read 读完）
