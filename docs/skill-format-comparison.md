# Claude Code 与 OpenClaw SKILL.md 格式对比

两者核心架构类似（都是 YAML frontmatter + Markdown body），但在字段设计和使用方式上有几个关键区别。

---

## 1. 触发词定义方式

| | Claude Code | OpenClaw |
|---|---|---|
| **方式** | 嵌入在 `description` 自然语言描述中 | 独立的 `triggers` 数组字段 |
| **示例** | `description: Use when user says '伴读'...` | `triggers: ["伴读", "read with me"]` |

---

## 2. 权限与能力声明

**Claude Code**：没有显式的权限/能力字段，工具使用由对话中动态决定。

**OpenClaw**：有结构化的安全边界声明：

```yaml
capabilities: [web_fetch, file_read]
permissions: [read_local, network]
mcp_tools: [openweathermap]
allowed-tools: ["Bash(pdftotext:*)", "Read"]
```

---

## 3. 参数定义

**Claude Code**：无结构化参数定义，参数提取靠指令中的自然语言描述。

**OpenClaw**：有明确的 `inputs` / `outputs` 定义：

```yaml
inputs:
  - name: city
    type: string
    required: true
    default: "Beijing"
```

---

## 4. 代码入口支持

**Claude Code**：纯指令型，逻辑全部写在 Markdown 中。

**OpenClaw**：支持代码型 skill，可配 `index.js` + `package.json`：

```
my-skill/
├── skill.md
├── index.js      # 逻辑入口
└── package.json  # 依赖
```

---

## 5. 特有字段

| 字段 | Claude Code | OpenClaw |
|---|---|---|
| `user_invocable` | 控制用户是否可在对话中触发 | 无对应 |
| `always` | 控制是否始终激活 | 无对应 |
| `metadata.openclaw` | 无 | OpenClaw 专属元数据 |
| `requires.env` / `requires.bins` | 部分支持 | 支持 |

---

## 6. 文件位置与命名

| | Claude Code | OpenClaw |
|---|---|---|
| **路径** | `.claude/skills/` | `~/.openclaw/skills/` 或项目 `skills/` |
| **文件名** | `SKILL.md`（大写） | `skill.md`（小写，也可大写） |

---

## 总结

OpenClaw 的格式更偏向**工程化/结构化**，强调安全声明、参数类型、MCP 集成；Claude Code 的格式更偏向**纯提示工程**，依赖自然语言描述和 `<example>` 标签来约束行为。如果你的 skill 需要调用外部工具或严格的安全边界，OpenClaw 的声明式字段更有优势；如果是纯对话式角色扮演或内容生成，Claude Code 的格式更简洁。

---

## Sources

- [OpenClaw Skill Anatomy: Understanding SKILL.md](https://www.shopclawmart.com/blog/openclaw-skill-anatomy-skills-md)
- [OpenClaw Skills Developer Guide](https://www.meta-intelligence.tech/en/insight-openclaw-skills)
- [How to build custom OpenClaw skills for your own automations - LumaDock](https://lumadock.com/tutorials/build-custom-openclaw-skills)
- [What are OpenClaw Skills? A 2026 Developer's Guide - DigitalOcean](https://www.digitalocean.com/resources/articles/what-are-openclaw-skills)
