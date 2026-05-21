# CHANGELOG

## [未发布]

### Added
- docs/ 规范化：创建开发文档体系（README + CHANGELOG + DECISIONS）

## [v2.1] - 2026-05-21

### Fixed
- Path C（角色合成）Phase 0C 确认流程：新增硬规则禁止跳过用户确认
  - Step 2：无论描述多明确，必须展示推荐维度组合并获确认
  - Step 1/3：增加判定分支（信息已含则确认，未含则追问）
  - Step 4：必须获得用户确认后才能创建目录
  - "绝不做的事"新增：禁止跳过 Phase 0C 用户确认
  - 涉及文件：`SKILL.md`、`references/path-c.md`

## [v2.0] - 2026-05-20

### Added
- 三路径改版：Path A（人物蒸馏）+ Path B（需求诊断）+ Path C（角色合成）
- 角色合成能力：从维度库组合出全新人格化角色
- 新增参考文件：`path-c.md`、`persona-dimension-library.md`
- 合成角色 Skill 模板（扩展 `skill-template.md`）
- 角色一致性校验机制

### Changed
- 品牌重塑：从 `nuwa-skill`（女娲造人）更名为 `Persona Forge`
- 入口分流升级：从 2 条路径升级为 3 条路径
- Phase 0 新增 Path B（需求诊断）和 Path C（角色合成）

## [v1.x] - 2026-04 ~ 2026-05

### Added
- `nuwa-skill` 初版：纯人物蒸馏能力
- `examples/` 目录：收录多个 perspective skill 实例

### Changed
- 2026-05-18：删除 JA/KO/ES 多语言 README，精简冗余资源
- 2026-05-20：完成品牌重塑，正式更名为 Persona Forge
