# CHANGELOG

## [未发布]

### Added
- 新增四部经典风格指南，覆盖 guwen-translate 全局缺口
  - 《论语》`references/lunyu.md`：仁、礼、学核心，还原师徒对话现场
  - 《庄子》`references/zhuangzi.md`：逍遥、齐物、解构核心，让寓言吞掉你
  - 《孙子兵法》`references/sunzi.md`：先胜、奇正、虚实、全胜核心，拆解结构
  - 《坛经》`references/tanjing.md`：顿悟、不立文字、自性核心，机锋翻转
  - 全部遵循"形式随气而变"原则，无固定输出模块映射
  - 更新 SKILL.md：description 关键词、4 个新 example、8 类典籍识别、命名格式

- 新增未来待覆盖经典分析文档（`future-classics-analysis.md`）
  - 从使用频率、气质独特性、通用文言盲区三维度诊断 guwen-translate 全局缺口
  - 识别出 T1 优先补充经典：《论语》《庄子》
  - 识别出 T2 值得补充经典：《孙子兵法》《坛经》
  - 识别出 T3 优先级较低：《孟子》《楚辞》《史记》《尚书》等
  - 提出待决策问题：儒家统一框架 vs 独立、兵家范围、禅宗细分

## [v0.1.0] - 2026-05-18

### Added
- 初始创建：古文今解 skill（feat(guwen-translate): 添加古文今解 skill）
  - 支持 4 部典籍定向风格指南：《道德经》《诗经》《易经》《金刚经》
  - 通用文言翻译框架（信达雅三层结构）
  - 5 模块输出结构：原文、今解、意境/结构、生活映照、余响/留白
  - 语气约束：真而不硬，深而不玄，美而不艳，静而不空
  - 素材来源：docs/guwen/（通过 references/guwen 软链接引用）
  - 触发词：古文今解、翻译古文、古译今、今解、古文翻译等
  - 涉及文件：`SKILL.md`、`references/guwen`

### Changed
- 合并道德经双指南为单一指南（refactor(guwen): 合并道德经双指南为单一指南）
  - daodejing.md + daodejing02.md → 统一 daodejing.md
  - 保留详细方法论骨架，融入精简版的凝练表达与"经之余响"
  - 删除冗余文件：`docs/guwen/daodejing02.md`
  - 涉及文件：`docs/guwen/daodejing.md`、`SKILL.md`

- 增加输出文件保存到 output 目录（feat(guwen-translate): 增加输出文件保存到 output 目录）
  - 完成诠释后自动保存为 `./output/guwen-translate/{YYYY-MM-DD}--{topic}__guwen.md`
  - 与仓库其他 skill 的输出约定保持一致
  - 涉及文件：`SKILL.md`

- 按 skill-creator 框架优化 SKILL.md（refactor(guwen-translate): 按 skill-creator 框架优化 SKILL.md）
  - 添加 `version: 0.1.0` 字段
  - description 增加主动触发场景，减少 undertrigger
  - 通用文言流程明确增加 `wenyan.md` 读取指令
  - 新增质量红线小节，明确 7 种禁止输出类型：
    1. 学术腔 / 论文体
    2. 鸡汤 / 成功学
    3. 网络金句堆砌
    4. 假禅意 / 伪玄虚
    5. 错误典籍归属
    6. 过度引申
    7. 封闭总结
  - 涉及文件：`SKILL.md`

- 统一源文件位置，简化 references 层级（refactor(guwen-translate): 统一源文件位置，简化 references 层级）
  - 将 `docs/guwen/*.md` 五本指南迁移至 `skills/guwen-translate/references/`
  - 删除 `docs/guwen/` 目录（不再作为源文件位置）
  - 删除 `references/guwen` 软链接
  - 更新 SKILL.md 中 reference 路径：`references/guwen/` → `references/`
  - 源文件现统一出自 `skills/guwen-translate`，不再依赖 `docs/`
  - 涉及文件：`SKILL.md`、`references/daodejing.md`、`references/jingangjing.md`、`references/shijin.md`、`references/wenyan.md`、`references/yijing.md`
