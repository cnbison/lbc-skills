---
status: staging
since: 2026-04-15
---

## 验证检查清单

| 检查项 | 状态 | 日期 | 备注 |
|--------|------|------|------|
| SKILL.md 格式正确（frontmatter + Usage + Instructions） | 🚧 | — | 缺少 `<example>` 块和 `## Instructions` |
| 触发词在新会话 system-reminder 中出现 | ✅ | 2026-04-15 | `mofa.fm`/`播客`/`podcast`/`fm api`/`热搜` |
| 至少一次端到端测试通过 | ✅ | 2026-03-18 | test_api.py 30/30 通过（公开 API） |
| docs/ 开发文档已创建 | ✅ | 2026-05-22 | README + CHANGELOG + STATUS + analysis.md |
| skill-creator 深度分析 | ✅ | 2026-05-22 | 报告见 analysis.md，识别 3 个 🔴 严重问题 |
| 无已知阻塞问题 | 🚧 | — | 认证 API 未测试，SKILL.md 结构性问题待修复 |

## 转为 active 的条件

- [ ] 修复 SKILL.md 结构性问题（`<example>` + `## Instructions` + version 字段）
- [ ] 补充认证 API 测试
- [ ] 用户明确确认可实用
