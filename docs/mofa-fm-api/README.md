# mofa-fm-api · 开发总览

## 定位

MoFA FM 播客平台 API 客户端。浏览节目/单集、搜索播客、管理创作者内容、获取 56 个数据源的热搜。含 Python 客户端工具 fm_client.py。

## 当前状态

见 [STATUS.md](./STATUS.md)

## 核心文件索引

| 文件 | 说明 |
|------|------|
| `../../skills/mofa-fm-api/SKILL.md` | Skill 定义入口（API 参考文档） |
| `../../skills/mofa-fm-api/manifest.json` | 元数据与触发词 |
| `../../skills/mofa-fm-api/tools/fm_client.py` | Python API 客户端 + CLI |
| `../../skills/mofa-fm-api/test_api.py` | 测试脚本 |
| `../../skills/mofa-fm-api/TEST_REPORT.md` | 测试报告（30/30 通过） |
| `../../skills/mofa-fm-api/examples/` | 基础使用示例 + 内容创作工作流 |
| `../../skills/mofa-fm-api/docs-site/` | Vercel 静态文档站点 |

## 版本历史

见 [CHANGELOG.md](./CHANGELOG.md)

## 分析报告

见 [analysis.md](./analysis.md) — skill-creator 框架审查报告
