# Skill 加载路径说明：`.claude/skills/` vs `skills/`

> 测试问题：把 skill 放在 `.claude/skills/ljg-read` 和 `skills/ljg-read` 是否都可以？使用时有什么区别？

## 实测结果

在 Claude Code 会话中，system-reminder 列出的可用 skills 为：

- `dou-wentao-perspective`
- `follow-builders`
- `persona-forge`
- `ljg-read`

这 4 个 skill 正好对应 `.claude/skills/` 下的 4 个目录。

而 `skills/` 下的 `mofa-fm`、`daily-news`、`news-summary`、`mofa-podcast-0429` 等目录**全部没有出现在可用 skill 列表里**。

结论：只有 `.claude/skills/` 下的 skill 会被 Claude Code 自动加载。

## 三种 skill 加载路径

| 路径 | 性质 | 是否被 Claude Code 加载 |
|------|------|----------------------|
| `<repo>/.claude/skills/<name>/` | 项目级 skill 安装目录（Claude Code 约定） | ✅ 会自动扫描加载 |
| `<repo>/skills/<name>/` | 仓库源码目录（本仓库自己的开发/分发目录） | ❌ 不会被加载 |
| `~/.claude/skills/<name>/` | 用户级 skill（跨项目全局） | ✅ 会加载（当前机器该目录不存在） |

## 当前 ljg-read 的情况

两个位置的 `skill.md` 内容**完全相同**：

```bash
$ diff -r .claude/skills/ljg-read/ skills/ljg-read/
# 无任何输出，文件一致

$ ls -la .claude/skills/ljg-read/skill.md skills/ljg-read/skill.md
# 两个文件大小都是 10583 字节
```

但即使内容相同，**只有 `.claude/skills/ljg-read/` 那份会被 Claude Code 实际调用**。如果删除 `.claude/skills/ljg-read/`，仅保留 `skills/ljg-read/`，这个 skill 在 Claude Code 中**就不再可用**。

## 为什么要存在两个目录？

- `.claude/skills/` = "已安装"区，Claude Code 直接读取
- `skills/` = "源码 / 分发包"区，类似上架到市场前的工作区，便于版本管理、复制分享、被其他项目引用
- 这就是为什么 `persona-forge` 等造 skill 流程的最终产物需要"安装"到 `.claude/skills/` 后才能被触发使用

## 建议的管理方式

可以选其中一种做法，避免出现"看起来都装了，却调不到"的混乱：

### 方案 A：软链接（推荐用于自研 skill）

让源码留在 `skills/` 下方便维护，通过软链接挂到加载目录：

```bash
cd .claude/skills
ln -s ../../skills/ljg-read ljg-read
```

优点：单一来源（source of truth），改 `skills/` 自动生效。

### 方案 B：只用 `.claude/skills/`

如果不需要"分发源码"的概念，直接删掉 `skills/<name>/` 的重复目录，所有内容都在 `.claude/skills/` 下维护。

### 方案 C：双写同步（不推荐）

两边都保留独立副本，每次改动都手动同步。容易出现版本漂移，仅在分发场景下才考虑。

## 排查清单

当 skill 看似已存在却无法触发时，按以下顺序检查：

1. 是否放在 `.claude/skills/<name>/` 或 `~/.claude/skills/<name>/`？
2. skill 文件名是否为 `skill.md` 或 `SKILL.md`（注意约定）？
3. YAML frontmatter 中 `name` 与 `description` 是否齐全，trigger 词是否清晰？
4. 重启 Claude Code 会话后再观察 system-reminder 列出的 skills 列表。
