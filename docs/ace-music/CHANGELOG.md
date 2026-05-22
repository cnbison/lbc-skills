# CHANGELOG

## [未发布]

## [v1.0.0] - 2026-05-20

### Fixed
- 按分析报告修复所有结构性问题（fix(ace-music): 按分析报告修复所有结构性问题）
  - 建立软链 `.claude/skills/ace-music`，skill 可被 Claude 加载
  - 重写 SKILL.md frontmatter：添加中英双语 Triggers、version、author、requires_bins、requires_env
  - 补齐 SKILL.md Body：`<example>` 块 + `## Instructions` 编号步骤
  - 修复 generate.sh：
    - 输出目录改为 `./output/ace-music/`（符合仓库约定）
    - 修复 tagged mode 中 `\n` 不是真换行的问题
    - 添加数字字段校验（duration/bpm/seed/batch）防止 JSON 注入
    - 合并 4 次 Python 调用为 1 次，通过 stdin 安全传递响应
    - 移除 LANGUAGE 默认 `"en"`，尊重 API 的 CoT 自动语言检测
  - 新增 `scripts/health.sh` 用于 API 连通性检查
  - 涉及文件：`.claude/skills/ace-music`、`SKILL.md`、`scripts/generate.sh`、`scripts/health.sh`

## [v0.1.0] - 2026-04-16

### Added
- 初始创建：ace-music skill（Update skills: add ace-music, podcastifier, ffmpeg-editor, nuwa-skill and docs）
  - 引入 ACE-Step 1.5 免费 API 生成 AI 音乐
  - 基础 generate.sh 脚本
  - api-docs.md 参考文档
  - 涉及文件：`SKILL.md`、`scripts/generate.sh`、`references/api-docs.md`
