# mofa-fm-api Skill-Creator 深度分析

> 分析日期：2026-05-22
> 分析框架：skill-creator 合规性审查 + 可激活度评估
> 分析范围：`skills/mofa-fm-api/` 全部文件

---

## 一、元数据合规性

### 1.1 Frontmatter（SKILL.md 头部）

```yaml
---
name: mofa-fm-api
description: "MoFA FM Podcast Platform API client — browse shows/episodes, search podcasts, manage creator content, trending topics. Triggers: mofa.fm, 播客, podcast, fm api, 热搜"
requires_bins: python3
octos_only: true
always: false
---
```

| 字段 | 状态 | 说明 |
|------|:--:|------|
| `name` | ✅ | kebab-case，与目录一致 |
| `description` | 🟡 | 触发词已列出（5个，中英双语），但 `Triggers:` 前缀与标准格式略有差异 |
| `requires_bins` | ✅ | 声明 `python3` |
| `version` | 🔴 **缺失** | 必须添加，建议与 manifest.json 一致 `1.0.0` |
| `octos_only` | 🟡 **非标准** | CLAUDE.md §4 未定义此字段，需确认是否为遗留字段或自定义扩展 |
| `requires_env` | 🟡 | manifest.json 中声明 JWT auth，但 frontmatter 未声明对应的环境变量需求（如 `MOFA_FM_API_KEY` 或 `JWT_TOKEN`） |

### 1.2 manifest.json

| 字段 | 状态 | 说明 |
|------|:--:|------|
| `name` / `version` | ✅ | 与目录一致，version 为 `1.0.0` |
| `triggers` | ✅ | 5 个触发词，与 SKILL.md description 一致 |
| `tools` | ✅ | 声明 `fm_client` 入口为 `tools/fm_client.py` |
| `requires` | ✅ | python3 + requests |
| `api` | 🟡 | 自定义字段，非标准 skill-creator 字段，但不影响加载 |

**不一致点**：manifest.json 有 `version: 1.0.0`，但 SKILL.md frontmatter 缺少 `version` 字段。

---

## 二、SKILL.md Body 结构合规性

| 检查项 | 标准 | 实际 | 状态 |
|--------|------|------|:--:|
| `<example>` 块 | 至少 1 个 | **0 个** | 🔴 |
| `## Instructions` | 必须有 | **缺失** | 🔴 |
| `## Usage` | 推荐有 | 用 `## 使用示例` 替代（第 184 行） | 🟡 |
| 编号步骤指令 | 推荐 | 无 | 🟡 |
| API 参考完整性 | — | 5 大类端点完整（认证/播客/搜索/互动/脚本管理） | ✅ |

### 2.1 严重结构缺陷

1. **无 `<example>` 块**：全文件 301 行，0 个 `<example>`。Claude Code 依赖 `<example>` 块学习 skill 的交互模式。缺失导致激活后行为不可预测。
2. **无 `## Instructions`**：skill 被触发后，Claude 不知道应该执行什么操作。当前文件实质是一份**纯 API 参考手册**，而非**可执行的 skill 定义**。
3. **无编号步骤**：即使作为 API 参考，也缺少 "当用户提到触发词时，应该执行哪几步" 的指令。

### 2.2 内容质量

| 维度 | 评估 |
|------|------|
| API 文档完整度 | 优秀。Base URL、认证方式、5 大类端点、数据结构、状态码、限制均有覆盖 |
| 示例代码 | 丰富。Python + cURL 双格式，覆盖主要端点 |
| 中文表达 | 良好，术语准确 |
| 与 Claude Code 集成 | **差**。Claude 读完此文件后，不知道该在什么场景调用哪个 API |

---

## 三、代码与工具审查

### 3.1 tools/fm_client.py（659 行）

| 维度 | 评估 |
|------|------|
| 结构 | Python 类 `FMClient` + CLI 入口。包含 API 调用封装 + 命令行参数解析 |
| 依赖 | `requests`（已在 manifest.json 声明） |
| 硬编码 | 无 API key 硬编码，符合 CLAUDE.md §3 约束 |
| 错误处理 | 基础 HTTP 状态码处理 |
| 代码注释 | 英文注释，符合 CLAUDE.md §7 |

**问题**：
- CLI 功能完整，但作为 skill 的 `tools/` 组件，它是否被 Claude Code 正确调用？需要验证工具注册路径。

### 3.2 test_api.py（501 行）

| 维度 | 评估 |
|------|------|
| 测试函数 | 5 个（`test_*`） |
| 通过率 | 30/30 公开 API 通过（2026-03-18） |
| 认证 API 测试 | **0** — 注册/登录/Token 刷新均未测试 |
| 测试报告 | TEST_REPORT.md + test_report.json（224KB） |

**问题**：
- `test_report.json`（224KB）未被 `.gitignore` 排除，纳入 git 历史会永久膨胀仓库。
- 认证流程是创作者内容管理的前提，缺失认证测试意味着 "管理创作者内容" 这一核心场景未经验证。

### 3.3 docs-site/（40KB）

- Vercel 静态文档站点，含 Swagger UI 风格 `index.html` + `vercel.json`
- 作为 skill 目录的一部分，`docs-site/` 不是 skill 运行必需文件
- 建议：评估是否需要保留在 skill 源码目录内，或迁移至独立仓库

---

## 四、端到端可激活度评估

### 4.1 触发链路

```
用户输入含 "mofa.fm" / "播客" / "podcast" / "fm api" / "热搜"
    ↓
Claude Code 识别触发词 → 加载 SKILL.md
    ↓
❌ 无 Instructions → Claude 不知道下一步该做什么
    ↓
可能行为：打印 API 参考文档（非预期）/ 询问用户意图 / 无响应
```

### 4.2 场景覆盖度

| 场景 | 支持度 | 说明 |
|------|:--:|------|
| "查一下今天的热搜" | 🟡 | 有 API，但无 Instructions 告诉 Claude 调用 `get_trending` |
| "搜索关于 AI 的播客" | 🟡 | 有 API，但无 Instructions |
| "帮我发布一个播客单集" | 🔴 | 需要认证 API（未测试）+ 无 Instructions |
| "mofa.fm 上有什么节目" | 🟡 | 有 API，但无 Instructions |

**结论**：当前 skill 在触发后**无法完成任何有意义的端到端任务**，因为缺少 Instructions 告诉 Claude 如何将用户意图映射到 API 调用。

---

## 五、问题分级

### 🔴 严重（阻塞激活）

| # | 问题 | 影响 | 修复建议 |
|---|------|------|---------|
| 1 | SKILL.md 无 `## Instructions` | Claude 触发后不知道做什么 | 添加 `## Instructions`，用编号步骤描述：识别用户意图 → 选择 API → 调用 fm_client → 格式化输出 |
| 2 | SKILL.md 无 `<example>` 块 | Claude 无法学习交互模式 | 添加至少 2 个 `<example>`：查热搜、搜索播客 |
| 3 | frontmatter 缺少 `version` | 元数据不完整 | 添加 `version: 1.0.0` |

### 🟡 高（建议修复）

| # | 问题 | 影响 | 修复建议 |
|---|------|------|---------|
| 4 | 认证 API 未测试 | 创作者管理功能不可靠 | 补充注册/登录/Token 刷新测试用例 |
| 5 | `test_report.json` 入 git | 仓库膨胀 | 添加 `.gitignore` 排除 `test_report.json` |
| 6 | `octos_only` 非标准字段 | 可能引发解析问题 | 确认必要性，如不需要则移除 |
| 7 | `requires_env` 未声明 | 认证场景缺少依赖提示 | 声明 `MOFA_FM_TOKEN` 或相关环境变量 |

### 🟢 中（可选优化）

| # | 问题 | 影响 | 修复建议 |
|---|------|------|---------|
| 8 | `docs-site/` 在 skill 目录内 | 增加 skill 体积 | 考虑移出至独立 docs 仓库或 `output/` |
| 9 | `## 使用示例` 非标准标题 | 与 CLAUDE.md 约定的 `## Usage` 不一致 | 可保持现状，或改为 `## Usage` |

---

## 六、修复建议（按优先级）

### P0：结构修复（激活前提）

```markdown
---
name: mofa-fm-api
description: >-
  MoFA FM 播客平台 API 客户端。浏览节目/单集、搜索播客、
  管理创作者内容、获取 56 个数据源的热搜。
  Triggers: mofa.fm, 播客, podcast, fm api, 热搜
version: 1.0.0
requires_bins: python3
requires_env: MOFA_FM_API_KEY  # 或 JWT 相关变量
always: false
---

## Usage

<example>
User: 今天的热搜有什么？
Assistant: 我来查询 MoFA FM 的热搜数据...
[调用 fm_client get_trending]
今天的热搜 Top 5：...
</example>

<example>
User: 搜索关于 AI 的播客
Assistant: 正在搜索...
[调用 fm_client search_podcasts query="AI"]
找到 3 个相关节目：...
</example>

## Instructions

1. 识别用户意图，匹配以下场景之一：
   - 查热搜 → 调用 `/trending` 端点
   - 搜索播客 → 调用 `/search` 端点
   - 浏览节目 → 调用 `/shows` 端点
   - 管理内容 → 需认证，检查环境变量
2. 根据场景构造 API 请求参数
3. 调用 `tools/fm_client.py` 中对应方法
4. 将 JSON 响应格式化为用户友好的中文输出
5. 如遇 401/403，提示用户配置认证信息
```

### P1：测试与清理

1. 补充 `test_auth.py` 覆盖注册/登录/Token 刷新
2. 新建 `.gitignore`：
   ```
   test_report.json
   __pycache__/
   *.pyc
   .env
   ```
3. 移除或解释 `octos_only` 字段

### P2：可选优化

1. 将 `docs-site/` 迁移至 `output/mofa-fm-api/docs-site/` 或独立仓库
2. 统一标题为 `## Usage`

---

## 七、总结

| 维度 | 评分 | 说明 |
|------|:--:|------|
| 元数据完整性 | 6/10 | 缺 version、requires_env；有非标准字段 |
| Body 结构合规 | 2/10 | 缺 Instructions、缺 example、缺 Usage |
| API 文档质量 | 9/10 | 完整、准确、示例丰富 |
| 代码质量 | 7/10 | 结构合理，但 CLI 与 skill 集成未验证 |
| 测试覆盖 | 5/10 | 公开 API 全通过，认证 API 零覆盖 |
| 端到端可激活 | 1/10 | 触发后无法执行任何有意义操作 |
| **综合评级** | **🟡 staging** | 框架完整但结构性问题阻塞激活 |

**核心结论**：`mofa-fm-api` 是一份优秀的 API 参考手册，但**不是一份合格的 Claude Code skill**。它缺少 skill 最核心的 `Instructions` 和 `<example>`，导致 Claude 无法将用户意图转化为 API 调用。修复 P0 项后可升级为 active。
