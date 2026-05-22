# CHANGELOG

## [未发布]

## [v1.5.0] - 2026-05-19

### Fixed
- 修复分析报告中的全部遗留问题（fix(daily-news): 修复分析报告中的全部遗留问题）
  - 修复 logger 在定义前使用（Bug 1）
  - generate_podcast.py: 字数 1200→600，旧文件名格式修复
  - manifest.json: 新增 6 个触发词（只收集/只过滤/跳过TTS/指定日期/新闻摘要/播客脚本）
  - 新增口语化触发词（今天有啥新闻/给我讲讲今天的tech新闻/播客做好了没）
  - 日志路径从 cwd 改为固定到 skill-dir/logs/
  - 抽离文章/播客模板到 references/，精简 SKILL.md 30+ 行
  - 优化 example 块，增加 per-stage re-run 示例
  - 涉及文件：`SKILL.md`、`manifest.json`、`references/article-template.md`、`references/podcast-template.md`、`run_pipeline.py`、`scripts/generate_podcast.py`

### Changed
- 修复代码 bug、统一触发词、优化命名规范（refactor(daily-news): 修复代码 bug、统一触发词、优化命名规范）
  - run_pipeline.py: 修复 logger 定义前使用、补全 timedelta 导入、修正 Step 编号
  - SKILL.md: 添加 version: 1.5.0，description 增加主动触发场景，播客字数修正
  - 命名规范：输出文件改为仓库约定格式 `{date}--claw-daily__{type}.{ext}`
  - 新增 docs/daily-news-analysis.md 完整分析报告
  - 涉及文件：`README.md`、`SKILL.md`、`config/sources.yaml`、`run_pipeline.py`、`scripts/config.py`

## [v1.0.0] - 2026-05-10 ~ 2026-05-13

### Added
- 初始创建：daily-news Python 流水线（Organize skills: symlink .claude paths...）
  - 21 个文件，含 SKILL.md、manifest.json、run_pipeline.py、setup.sh
  - scripts/: collect_news.py、filter_news.py、generate_article.py、generate_podcast.py、 summarize_news.py、tts_generate.py、config.py、models.py、mofa_client.py、mofa_publish.py
  - config/sources.yaml: RSS 源配置
  - tools/daily_news.py: CLI 工具

### Fixed
- 修复 pipeline：retry loop、keywords、sources、history dedup（Fix daily-news pipeline）
  - collect_news.py: retry loop 添加 break，防止 3x 数据膨胀
  - sources.yaml: 移除低质量 CNET feed，移除失效源，添加高质量源（OpenAI blog、Google AI blog、Import AI、LessWrong）
  - 扩展关键词到主流 AI/LLM/agent/security 术语
  - time_window_hours: 24 → 48
  - filter_news.py: 添加历史去重（前 2 天输出）
  - 过滤后输出从 ~2 篇提升至 ~50 篇

- 添加垃圾过滤和修复 source_category mapping（Add spam filtering）
  - collect_news.py: category_to_source_type 映射修复 null source_category
  - config.py: get_spam_patterns() 加载正则模式
  - filter_news.py: filter_by_spam() 标题正则匹配，集成到 pipeline
  - sources.yaml: 添加 spam_patterns（中文推广、API 代理广告、低质量 Reddit 帖子）
  - 测试运行过滤掉 8 篇推广/低质量文章

- 修复 .gitignore（Fix .gitignore to only ignore root daily-news/）
  - 只忽略根目录 `daily-news/`，不忽略 `skills/daily-news/`
  - SKILL.md 文档更新 data/output 目录位置

### Changed
- 统一输出目录到 output/（refactor(output): unify skill output under output/）
  - 将 daily-news/ 和 follow-builders/ 输出移到 `output/<skill-name>/`
  - 更新 .gitignore、Python 脚本、config、SKILL.md、README.md、CLAUDE.md
