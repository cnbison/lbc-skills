# CHANGELOG

## [未发布]

### Added
- skill-creator 深度分析报告：`docs/mofa-fm-api/analysis.md`
  - 识别 3 个 🔴 严重问题（无 Instructions、无 example、缺 version）
  - 识别 4 个 🟡 高优先级问题（认证未测试、test_report.json 入 git、非标准字段等）
  - 综合评级：🟡 staging，修复 P0 后可升级 active
  - 涉及文件：`docs/mofa-fm-api/{README.md, CHANGELOG.md, STATUS.md, analysis.md}`

## [v1.0.0] - 2026-04-15

### Added
- 初始创建：mofa-fm-api MoFA 类 skill（Add mofa skills: cli, fm-api, fm, podcast, slides）
  - SKILL.md：完整 API 参考文档（认证、播客、搜索、互动、脚本管理 5 大类端点）
  - manifest.json：元数据、触发词、工具声明
  - tools/fm_client.py：Python API 客户端 + CLI（24KB）
  - test_api.py：自动化测试脚本
  - TEST_REPORT.md：测试报告（30/30 公开 API 通过，56 个热搜源验证）
  - examples/：基础使用示例 + 内容创作工作流
  - docs-site/：Vercel 静态文档站点（Swagger UI 风格）
  - architecture.dot：架构图
