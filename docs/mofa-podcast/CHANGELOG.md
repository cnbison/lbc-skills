# CHANGELOG

## [未发布]

## [v0.4.5] - 2026-05-19

### Fixed
- 添加 wrapper script 解决 stdin protocol 和 macOS quarantine 问题（fix(mofa-podcast): add wrapper script）
  - 主二进制从 stdin 读取 JSON payload（非 argv），macOS 可能携带 com.apple.quarantine 导致 SIGKILL (exit 137)
  - `scripts/run.sh` 包装两者问题，SKILL.md 更新所有工具调用通过 run.sh
  - 涉及文件：`SKILL.md`、`scripts/run.sh`

### Changed
- 按分析报告修复所有问题（refactor(mofa-podcast): 按分析报告修复所有问题）
  - 创建软链接 `.claude/skills/mofa-podcast → skills/mofa-podcast`
  - 统一版本号：SKILL.md 0.4.1 → 0.4.5（与 manifest/Cargo 一致）
  - 重写 description：更 pushy，新增 5 个口语化触发词
  - 重构 SKILL.md：新增 `<example>` 块和 `## Instructions` 编号步骤
  - 引用 `architecture.dot` 的 4 步 pipeline（gather→script→review→produce）
  - 输出路径改为仓库通行规则：`./skill-output/mofa-podcast-{timestamp}/`
  - 命名格式：`{date}--{topic}__script.md` / `__podcast.mp3`
  - 澄清声音管理：默认 douwt/yangmi，其他为克隆，支持查询/添加/删除
  - manifest.json 补全依赖：`python3 → python3 + ffmpeg + mofa-podcast`
  - 涉及文件：`SKILL.md`、`manifest.json`

## [v0.4.0] - 2026-05-15

### Added
- 重写版本上线（Add and adjust skills）
  - 9 个文件，3263 行新增
  - Rust 二进制：`Cargo.toml`、`Cargo.lock`、`src/main.rs`
  - TTS 引擎：`src/tts_engine.py`、`src/tts_engine_mock.py`
  - 架构图：`architecture.dot`
  - 集成测试：`scripts/test-integration.sh`
  - SKILL.md、manifest.json

## [v0.1.0] - 2026-04-15 ~ 2026-05-10

### Added
- 初始创建：mofa-podcast MoFA 类 skill（Add mofa skills: cli, fm-api, fm, podcast, slides）
  - Rust 二进制：`Cargo.toml`、`Cargo.lock`、`src/main.rs`
  - `architecture.dot`、manifest.json、scripts/test-integration.sh、SKILL.md

### Removed
- 旧版本退役（Organize skills: symlink .claude paths, retire mofa-podcast...）
  - 将 `skills/mofa-podcast/` 旧版本移为 `skills/mofa-podcast-old/`
  - 删除旧版本 7 个文件（3605 行）
