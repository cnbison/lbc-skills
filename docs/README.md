# docs/ 目录

> `lbc-skills` 仓库的开发文档集合。

## 结构

```
docs/
├── README.md                      # 本文件
├── docs-maintenance-guide.md      # docs/ 维护规则
├── _shared/                       # 跨 skill 的通用文档
│   ├── skill-format-comparison.md
│   └── skill-loading-paths.md
│
└── <skill-name>/                 # 每个 skill 的独立开发文档
    ├── README.md                  # 开发总览
    ├── CHANGELOG.md               # 版本历史
    └── DECISIONS.md               # 关键决策（按需创建）
```

## 规则

- 每个 skill 的开发文档独立存放在 `docs/<skill-name>/` 下
- 跨 skill 的通用文档放在 `docs/_shared/`
- 详细的维护规则见 [`docs-maintenance-guide.md`](./docs-maintenance-guide.md)

## 现有 Skill 文档

| Skill | 路径 |
|------|------|
| Persona Forge | [`persona-forge/`](./persona-forge/) |
