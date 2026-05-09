# follow-builders 深度分析：实现原理与使用指南

## 一、产品定位与核心理念

**follow-builders** 是一个 AI-native 的信息聚合技能（skill），其设计哲学非常明确：

> **Follow builders with original opinions, not influencers who regurgitate.**
> （追踪有原创见解的建造者，而非只会搬运信息的网红。）

它解决的核心痛点是：AI 领域信息爆炸，顶级 builders（研究员、创始人、工程师）在 X/Twitter、播客、官方博客中持续输出高价值观点，但普通用户没有时间去逐一追踪和消化。follow-builders 通过"中心化抓取 + 本地 LLM 混编"的架构，将这些内容自动整理成可阅读的日报/周报。

---

## 二、整体架构：三层分离设计

follow-builders 采用了非常清晰的**三层架构**：

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 1: Central Feed Generator (GitHub Actions)               │
│  ├─ generate-feed.js  →  每日自动抓取 X/播客/博客                │
│  ├─ feed-x.json       →  推文数据                               │
│  ├─ feed-podcasts.json → 播客逐字稿                             │
│  └─ feed-blogs.json   →  博客全文                               │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2: User-side Preparation (Node.js scripts)               │
│  ├─ prepare-digest.js →  拉取 feed + 读取用户配置 + 加载 prompts │
│  └─ deliver.js        →  Telegram / Email / stdout 投递         │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3: LLM Remix (Claude / OpenClaw Agent)                   │
│     读取 JSON → 按 prompts 混编摘要 → 输出最终 digest            │
└─────────────────────────────────────────────────────────────────┘
```

这个架构的最大优势是：**用户端不需要任何 API key 去访问 X、YouTube 或博客**。所有高成本、高门槛的数据抓取工作都在中央的 GitHub Actions 中完成，用户端只是拉取几个公开的 JSON 文件。

---

## 三、Layer 1：中心化 Feed 生成器深度解析

### 3.1 自动化工作流

文件：`.github/workflows/generate-feed.yml`

- **触发条件**：
  - 定时：`cron: '0 6 * * *'`（每天 UTC 6:00，对应北京时间 14:00）
  - 手动：`workflow_dispatch`，支持 `all` / `tweets-only` / `podcasts-only` / `blogs-only`
- **环境依赖**：`X_BEARER_TOKEN`（X API v2）、`POD2TXT_API_KEY`（播客转录服务）
- **输出产物**：`feed-x.json`、`feed-podcasts.json`、`feed-blogs.json`、`state-feed.json`

### 3.2 generate-feed.js 实现细节

这是整个技能最"重"的代码，负责三大内容源的数据获取与去重。

#### 3.2.1 X/Twitter 抓取（Official API v2）

```javascript
// 关键参数
const TWEET_LOOKBACK_HOURS = 24;
const MAX_TWEETS_PER_USER = 3;
```

**流程**：
1. **批量用户 ID 查询**：`users/by?usernames=...&user.fields=name,description`，每批最多 100 个用户
2. **逐用户拉取推文**：`users/{id}/tweets?max_results=5&exclude=retweets,replies&start_time={cutoff}`
3. **去重过滤**：用 `state.seenTweets` 跳过已推送过的 tweet ID
4. **长推文处理**：通过 `note_tweet.text` 获取超过 280 字符的完整内容
5. **速率限制处理**：如遇 HTTP 429，直接中断剩余账户的抓取

**技术细节**：
- 默认追踪 25 位 builders（`config/default-sources.json`）
- 每个 builder 每天最多 3 条推文
- 只抓取原创推文，**完全排除 retweets 和 replies**

#### 3.2.2 播客抓取（RSS + pod2txt）

```javascript
const PODCAST_LOOKBACK_HOURS = 336; // 14 天
const POD2TXT_BASE = 'https://pod2txt.vercel.app/api';
```

**流程**：
1. **RSS 抓取**：对每个播客的 `rssUrl` 发起请求，用正则解析 `<item>` 块获取 `title`、`guid`、`pubDate`、`link`
2. **候选集筛选**：检查最近 3 期节目，跳过 `state.seenVideos` 中已处理的 GUID
3. **时间窗口过滤**：只保留 14 天内发布的节目
4. **转录获取**：调用 `pod2txt` 的异步转录 API

**pod2txt 轮询机制**：
```javascript
const maxAttempts = 5;
const pollInterval = 30000; // 30 秒
```
- 第一次请求可能返回 `processing`
- 脚本会轮询最多 5 次，总耗时约 2.5 分钟
- 成功返回 `ready` 后，从 `data.url` 下载完整 transcript 文本

**容错设计**：
- 如果某个节目的 transcript 获取失败，会标记为 `seen` 避免明天再试，然后尝试下一个候选节目
- 最终每天最多输出 **1 个播客节目**（取最近一期能成功转录的）

#### 3.2.3 博客抓取（定向 HTML Scraping）

目前支持两个博客源：
- **Anthropic Engineering** (`https://www.anthropic.com/engineering`)
- **Claude Blog** (`https://claude.com/blog`)

**Anthropic Engineering 解析策略**：
1. **首选策略**：提取 Next.js 的 `__NEXT_DATA__` script 标签中的 JSON，导航到 Sanity CMS 的 `pageProps.posts`
2. **降级策略**：正则匹配 `href="/engineering/([a-z0-9-]+)"` 获取文章 slug，再逐个访问文章页面提取内容
3. **文章正文提取**：优先从 `__NEXT_DATA__` 的 `post.body`（Portable Text 格式）提取纯文本；失败则回退到 `<article>` 标签的 HTML strip

**Claude Blog 解析策略**：
1. **索引页**：正则匹配 `href="/blog/([a-z0-9-]+)"`
2. **文章页**：
   - 优先从 JSON-LD (`application/ld+json`) 提取 `headline`、`author`、`datePublished`
   - 正文从 Webflow 的 `u-rich-text-blog` 或 `w-richtext` 容器提取
   - 再失败则全页 HTML strip（去掉 nav/footer/script）

**参数**：
```javascript
const BLOG_LOOKBACK_HOURS = 72;     // 3 天回溯
const MAX_ARTICLES_PER_BLOG = 3;    // 每个博客最多 3 篇
```

### 3.3 去重机制：state-feed.json

```javascript
// 状态结构
{
  "seenTweets": { "tweetId": timestamp, ... },
  "seenVideos": { "episodeGuid": timestamp, ... },
  "seenArticles": { "articleUrl": timestamp, ... }
}
```

- 去重粒度精确到单个 tweet、单个播客 episode、单篇文章
- **自动清理**：保存时删除 7 天前的记录，防止文件无限膨胀
- `state-feed.json` 被提交到 GitHub 仓库，因此跨运行、跨日期的去重是持久化的

---

## 四、Layer 2：用户端准备与投递

### 4.1 prepare-digest.js：数据聚合与 prompt 管理

这是用户端运行的脚本，**不需要任何 API key**。

**核心功能**：

1. **读取用户配置**：`~/.follow-builders/config.json`
   ```json
   {
     "language": "en|zh|bilingual",
     "frequency": "daily|weekly",
     "delivery": { "method": "stdout|telegram|email", ... }
   }
   ```

2. **拉取三个 feed**：
   ```javascript
   const FEED_X_URL = 'https://raw.githubusercontent.com/zarazhangrui/follow-builders/main/feed-x.json';
   const FEED_PODCASTS_URL = '.../feed-podcasts.json';
   const FEED_BLOGS_URL = '.../feed-blogs.json';
   ```

3. **Prompt 加载的三级优先级**：
   - **Priority 1**：`~/.follow-builders/prompts/<file>`（用户自定义，不会被覆盖）
   - **Priority 2**：GitHub 远程最新（随 skill 升级自动获得改进）
   - **Priority 3**：本地 skill 目录中的默认副本（离线fallback）

4. **输出统一 JSON 结构**：
   ```json
   {
     "status": "ok",
     "config": { ... },
     "podcasts": [...],
     "x": [...],
     "blogs": [...],
     "stats": { "podcastEpisodes": 1, "xBuilders": 5, "totalTweets": 8, "blogPosts": 2 },
     "prompts": { "digest_intro": "...", "summarize_podcast": "...", ... },
     "errors": [...]
   }
   ```

这个 JSON 是 LLM 的**唯一输入**。SKILL.md 中反复强调：
> "Your ONLY job is to read this JSON, remix the content, and output the digest text."

### 4.2 deliver.js：多渠道投递

支持三种投递方式：

#### Telegram
```javascript
const MAX_LEN = 4000; // Telegram 单条消息上限 4096，预留余量用 4000
```
- 超长消息自动按换行符智能分片
- 使用 `Markdown` parse_mode，若解析失败自动降级为纯文本
- 分片间有 500ms 延迟避免 rate limit

#### Email（Resend）
- 发件人固定为 `AI Builders Digest <digest@resend.dev>`
- 主题带日期：`AI Builders Digest — Friday, March 14, 2026`
- 纯文本格式

#### stdout（默认）
- 直接打印到终端，由 agent 或 OpenClaw 负责最终展示
- 在 Claude Code 中非持久化场景下，这是最简单的方式

---

## 五、Layer 3：LLM Remix 与 Prompt 工程

follow-builders 的摘要质量很大程度上依赖于其 **5 个纯文本 prompt 文件**。

### 5.1 Prompt 文件体系

| 文件 | 作用 |
|------|------|
| `digest-intro.md` | 整体框架规范：标题格式、章节顺序、链接规则、禁止编造 |
| `summarize-podcast.md` | 播客 remix：200-400 字，必须有 "The Takeaway" 和一句直接引用 |
| `summarize-tweets.md` | 推文汇总：每位 builder 2-4 句，跳过无实质内容 |
| `summarize-blogs.md` | 博客摘要：100-300 字，突出核心公告/发现 |
| `translate.md` | 中英翻译规范：技术术语保留英文，段落级双语交错 |

### 5.2 关键 prompt 约束

**`digest-intro.md` 中的硬性规则**：
- **必须有链接**：每条内容必须附带原始链接，没链接就不包含
- **禁止编造**："NEVER make up quotes, opinions, or content"
- **作者格式**：用全名+职位，不用 @handle（因为 Telegram 中 `@handle` 会误链接到 Telegram 用户）
- **章节顺序**：X/TWITTER → OFFICIAL BLOGS → PODCASTS

**`summarize-podcast.md` 的内容风格要求**：
> "Stands alone as a complete piece — avoids references like 'this interview,' 'this video,' 'in this conversation.' Write as if distilling lessons from a person's philosophy, not summarizing a specific piece of content."

这确保了摘要读起来像"知识萃取"而非"内容梗概"。

**`translate.md` 的语言策略**：
- 技术术语保留英文：`AI`, `LLM`, `GPU`, `API`, `fine-tuning`, `RAG`, `token`, `prompt`, `agent`, `transformer`
- 双语模式要求**段落级交错**，而非先英后中

---

## 六、平台适配：OpenClaw vs Claude Code

follow-builders 的一个设计亮点是**对运行平台的感知与适配**。

### 6.1 平台检测

```bash
which openclaw 2>/dev/null && echo "PLATFORM=openclaw" || echo "PLATFORM=other"
```

### 6.2 OpenClaw 环境

- **持久化 agent**：终端关闭后 agent 仍在运行
- **内置消息渠道**：自动通过用户当前聊天的渠道（Telegram/Discord/飞书/WhatsApp 等）投递
- **定时任务**：使用 `openclaw cron add` 创建 cron job
- **渠道投递关键细节**：
  - 严禁使用 `--channel last`（多渠道配置时会失败）
  - 必须显式指定 `--channel <name>` 和 `--to "<target ID>"`
  - 创建后必须 `openclaw cron run <jobId>` 验证测试投递是否成功

### 6.3 Claude Code / Cursor 环境

- **非持久化 agent**：终端关闭 = agent 停止
- **默认按需使用**：输入 `/ai` 才生成 digest
- **如需自动推送**：必须配置 Telegram 或 Email，然后用系统 `crontab` 定时执行
- **crontab 的局限**：SKILL.md 坦诚地指出，纯 crontab 模式只能跑 `prepare-digest.js | deliver.js`，绕过 LLM，因此投递的是 raw JSON，不是 remix 后的 digest。想要完整的 LLM remix 自动推送，最好迁移到 OpenClaw。

---

## 七、首次使用流程（Onboarding）

首次触发 `/ai` 时，agent 会执行 9 步 onboarding：

1. **自我介绍**：说明追踪 builders 的理念
2. **频率与时间**：每日/每周，选择时区
3. **投递方式**：OpenClaw 自动跳过；Claude Code 需选 Telegram/Email/On-demand
4. **语言偏好**：English / Chinese / Bilingual
5. **API key 设置**（如需要自动推送）：引导用户获取 Telegram bot token 或 Resend API key
6. **展示信息源**：从 `default-sources.json` 读取并展示追踪的 builders 和播客
7. **配置可修改性说明**：告知用户可以随时通过对话修改
8. **设置 Cron 定时任务**：根据平台和投递方式创建定时任务
9. **欢迎摘要**：**立即**生成并投递第一份 digest，让用户看到实际效果并给反馈

 onboarding 完成后，配置写入 `~/.follow-builders/config.json`，`onboardingComplete: true`。

---

## 八、数据安全与隐私设计

- **无云端 API key 共享**：X API key、pod2txt key 只在 GitHub Actions 中使用，用户不可见也用不到
- **本地存储敏感信息**：Telegram token、Resend key 仅存在 `~/.follow-builders/.env`
- **只读公开内容**：所有抓取的都是公开的 blog post、YouTube 视频、X 帖子
- **无用户行为追踪**：没有 analytics、没有远程日志收集

---

## 九、使用指南

### 9.1 快速开始

在 Claude Code 中安装：
```bash
git clone https://github.com/zarazhangrui/follow-builders.git ~/.claude/skills/follow-builders
cd ~/.claude/skills/follow-builders/scripts && npm install
```

然后输入：
```
/ai
```

Agent 会以对话方式引导你完成所有设置，无需手动编辑配置文件。

### 9.2 日常指令

| 用户输入 | 效果 |
|----------|------|
| `/ai` | 立即生成一份最新 digest |
| "改成每周推送" | 更新 frequency 为 weekly |
| "换中文" | 更新 language 为 zh |
| "摘要写短一点" | Agent 复制并修改 summarize prompt |
| "显示我的设置" | 读取并展示 config.json |
| "显示我追踪了谁" | 展示 default-sources.json 中的 builders 和播客 |

### 9.3 自定义摘要风格（高级）

Prompt 文件可以个性化。例如想让播客摘要更短：

```bash
mkdir -p ~/.follow-builders/prompts
cp ~/.claude/skills/follow-builders/prompts/summarize-podcast.md ~/.follow-builders/prompts/summarize-podcast.md
```

编辑 `~/.follow-builders/prompts/summarize-podcast.md`，把 `200-400 words` 改为 `100-150 words`。下次 `/ai` 时就会自动使用你的自定义版本。

要恢复默认，直接删除该文件即可：
```bash
rm ~/.follow-builders/prompts/summarize-podcast.md
```

---

## 十、实现亮点与可借鉴之处

1. **"中心化 feed + 本地 LLM" 的解耦架构**：把高成本、需要 API key 的数据抓取和需要去重状态管理的逻辑放在 GitHub Actions 中；用户端只做轻量聚合和 LLM 调用，极大降低了使用门槛。

2. **Prompt 三级加载机制**：用户自定义 > 远程最新 > 本地默认。这个设计既允许个性化，又能让用户自动获得官方 prompt 改进。

3. **平台感知与适配**：同一套 skill 代码能根据 `openclaw` 是否存在自动切换交付策略，体现了对部署环境的深刻理解。

4. **去重状态持久化**：`state-feed.json` 随仓库提交，7 天自动清理，简单但有效。

5. **定向 Scraping 而非通用 Crawler**：针对 Anthropic Engineering 和 Claude Blog 分别写了专门的解析器（Next.js `__NEXT_DATA__`、JSON-LD、Webflow rich text），比用通用库更稳定、更精准。

6. **强制链接与反编造规则**：在 prompt 层面用 ALL CAPS 反复强调 "No fabrication" 和 "Mandatory links"，这是应对 LLM 幻觉的关键防线。

---

## 十一、局限与未来可能的扩展方向

- **播客每天只输出 1 期**：当前逻辑是遍历候选节目，取第一个能成功转录的。如果一天内多个播客都有新节目，只会拿到最近的一期。
- **博客源有限**：目前只内置了 Anthropic 和 Claude 两个官方博客，扩展新博客源需要写新的 HTML parser。
- **X API 成本**：依赖官方 X API v2，若 API 政策或定价变化可能需要调整。
- **Crontab 自动推送无 LLM remix**：在 Claude Code 中非 OpenClaw 环境下，纯 crontab 自动推送只能发送 raw JSON，体验降级。

---

## 参考文件位置

本分析基于仓库中的以下文件：

- `skills/follow-builders/SKILL.md` — Skill 定义与执行指令
- `skills/follow-builders/scripts/generate-feed.js` — 中央 feed 生成器
- `skills/follow-builders/scripts/prepare-digest.js` — 用户端数据准备
- `skills/follow-builders/scripts/deliver.js` — 消息投递脚本
- `skills/follow-builders/config/default-sources.json` — 默认信息源配置
- `skills/follow-builders/prompts/*.md` — 5 个 LLM prompt 文件
- `skills/follow-builders/.github/workflows/generate-feed.yml` — GitHub Actions 工作流
- `skills/follow-builders/README.zh-CN.md` — 中文使用说明
- `skills/follow-builders/examples/sample-digest.md` — 输出样例
