"""Podcast script generator data module — Agent-driven podcast generation.

This module no longer calls AI APIs directly. The Agent (Claude Code) reads
the article markdown and writes the podcast script directly.
"""

import logging
from typing import List
from pathlib import Path
from datetime import datetime
import json

from .models import FilteredArticle
from .config import get_config

logger = logging.getLogger(__name__)


class PodcastGenerator:
    """Data helper for podcast scripts. AI generation is done by the Agent."""

    def __init__(self, config=None):
        """
        Initialize podcast generator helper.

        Args:
            config: Configuration object (uses default if not provided)
        """
        self.config = config or get_config()

    def _extract_key_news(self, articles: List[FilteredArticle], max_count: int = 8) -> List[FilteredArticle]:
        """Extract top news for podcast coverage."""
        return sorted(articles, key=lambda x: x.relevance_score, reverse=True)[:max_count]

    def _format_news_items(self, articles: List[FilteredArticle]) -> str:
        """Format news items for prompt."""
        items = []
        for i, article in enumerate(articles, 1):
            art = article.article
            summary = article.ai_summary or art.summary or art.content or ""

            item = f"{i}. 标题: {art.title}\n"
            item += f"   摘要: {summary[:150]}\n"
            items.append(item)

        return "\n".join(items)

    @staticmethod
    def build_podcast_prompt(article_markdown: str, articles: List[FilteredArticle], date: str = None) -> str:
        """
        Build a prompt for the Agent to generate the podcast script.

        Args:
            article_markdown: Generated article markdown
            articles: List of FilteredArticle objects
            date: Script date (YYYY-MM-DD format)

        Returns:
            Prompt string for the Agent
        """
        if date is None:
            date = datetime.now().strftime("%Y-%m-%d")

        generator = PodcastGenerator()

        # Calculate weekday for the date
        date_obj = datetime.strptime(date, "%Y-%m-%d")
        weekdays = ["星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日"]
        weekday = weekdays[date_obj.weekday()]
        date_readable = date_obj.strftime("%Y年%m月%d日")

        # Extract key news
        key_news = generator._extract_key_news(articles)
        news_items = generator._format_news_items(key_news)

        return f"""请将以下Claw每日观察文章改写为播客脚本。

**节目名称：** claw日报
**播出日期：** {date_readable} {weekday}

关键资讯:
{news_items}

原文:
{article_markdown[:5000]}

请生成一个3-5分钟的科技新闻播客脚本，要求:

**风格要求（重要）:**
- 热情、友好的开场白，让听众感到亲切
- 自然流畅的对话，像两个朋友在聊天
- 每个话题之间有自然的过渡，不要生硬切换
- 结尾要带有关键要点的总结，让听众记住核心内容
- 全程采用对话式语气，避免生硬的播报感

**结构:**

[开场] (20-25秒)
- Alex热情报出节目名称："大家好，欢迎收听今天的claw日报"
- 必须播报完整日期："今天是{date_readable} {weekday}"
- Sarah用友好的语气预告今天的主题
- 两人自然互动，营造轻松氛围

[今日头条] (60-90秒)
- 最重要的1-2条新闻深度讨论
- Alex分析技术影响和细节
- Sarah从用户角度讨论价值
- 两人要有问答互动，不要各说各话

[技术速递] (60秒)
- 3-4条技术新闻快速介绍
- 每条新闻30秒左右的简短对话
- 主持人之间有互动和简短点评

[安全焦点] (45秒)
- AI安全相关新闻
- 简明扼要地说明风险
- 给出实用的建议

[GitHub热榜] (30秒)
- 值得关注的开源项目
- 用一两句话说明为什么值得关注

[社区声音] (30秒)
- 开发者社区讨论热点
- 展示不同观点

[总结] (30秒)
- Alex回顾今日3-4个关键要点
- Sarah友好地告别
- 预告明天再见

**格式要求（必须严格遵守）:**
- 使用 [ ] 标记环节名称，如 [开场]、[今日头条]
- 使用 "Alex:" 和 "Sarah:" 标记说话人（这是唯一正确的说话人标记格式）
- 不要使用 "(Alex...)" 或 "(Sarah...)" 这种格式
- 说话人标记后直接接对话内容，不要添加语气描述
- 总字数1200-1800字（不含舞台说明）
- 口语化，易于朗读
- 对话要自然，多用口语词汇和语气词
- **每个部分必须包含2-3条不同的新闻**

**正确格式示例:**
```
[开场]
Alex: 大家好，欢迎收听今天的claw日报。
Sarah: 是的，我是Sarah，今天又是充满科技新闻的一天。
Alex: 今天是2026年03月20日 星期五，让我们一起来看看今天的头条新闻吧！
Sarah: 好的，Alex，我迫不及待想知道今天有什么新鲜事。

[今日头条]
Alex: 首先得说说这个大新闻...
Sarah: 哇，这是真的吗？
```

请严格按照上述格式生成脚本:"""

    def _fix_script_format(self, script: str) -> str:
        """
        Fix common format issues in generated scripts.

        Args:
            script: Generated script

        Returns:
            Fixed script
        """
        import re

        lines = script.split('\n')
        fixed_lines = []

        for line in lines:
            # Skip empty lines but preserve them
            if not line.strip():
                fixed_lines.append(line)
                continue

            # Fix lines like "(Alex...) dialogue" -> "Alex: dialogue"
            # Pattern 1: (Alex...) or (Sarah...) at the start
            match1 = re.match(r'^\((Alex|Sarah)[^)]*\)\s*(.+)$', line.strip())
            if match1:
                speaker = match1.group(1)
                dialogue = match1.group(2)
                fixed_lines.append(f"{speaker}: {dialogue}")
                continue

            # Pattern 2: Lines starting with Alex or Sarah but without colon
            match2 = re.match(r'^(Alex|Sarah)\s+([^\:].+)$', line.strip())
            if match2:
                speaker = match2.group(1)
                dialogue = match2.group(2)
                fixed_lines.append(f"{speaker}: {dialogue}")
                continue

            # Keep original line if no match
            fixed_lines.append(line)

        return '\n'.join(fixed_lines)

    def _generate_fallback_script(self, articles: List[FilteredArticle], date: str) -> str:
        """Generate fallback podcast script without AI."""
        lines = [
            f"Claw 每日播客 - {date}",
            "",
            "[开场]",
            "",
            "Alex: 大家好，欢迎来到OpenClaw每日观察。我是Alex。",
            "",
            "Sarah: 我是Sarah。今天我们来看一下OpenClaw和AI领域的最新动态。",
            "",
            "[今日头条]",
            ""
        ]

        key_news = self._extract_key_news(articles, 3)

        for i, article in enumerate(key_news, 1):
            art = article.article
            summary = article.ai_summary or art.summary or ""

            lines.append(f"Alex: 第{i}条新闻是关于{art.title}的。")
            lines.append("")
            lines.append(f"Sarah: 这条新闻很有意思。{summary[:100]}")
            lines.append("")
            lines.append(f"Alex: 从技术角度来看，这反映了...")
            lines.append("")

        lines.extend([
            "[技术速递]",
            "",
            "Alex: 接下来我们快速看一下其他技术新闻：",
            ""
        ])

        for article in articles[3:6]:
            art = article.article
            lines.append(f"Sarah: {art.title}")
            lines.append(f"Alex: 这个项目值得关注，因为它...")

        lines.extend([
            "",
            "[总结]",
            "",
            "Alex: 以上就是今天的OpenClaw每日观察。",
            "",
            "Sarah: 感谢大家的收听，我们明天再见！",
            "",
            "Alex: 再见！"
        ])

        return "\n".join(lines)

    def save_script(self, script: str, date: str = None, output_path: str = None):
        """
        Save podcast script to file.

        Args:
            script: Generated podcast script
            date: Script date (YYYY-MM-DD format)
            output_path: Output file path
        """
        if output_path is None:
            output_dir = self.config.get_output_dir()
            output_dir.mkdir(parents=True, exist_ok=True)

            if date is None:
                date = datetime.now().strftime("%Y-%m-%d")

            filename = self.config.get_podcast_script_filename(date)
            output_path = output_dir / filename
        else:
            output_path = Path(output_path)

        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(script)

        logger.info(f"Saved podcast script to {output_path}")


def main():
    """Main function for testing — prints the podcast prompt for Agent review."""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    # Load article and data
    config = get_config()
    output_dir = config.get_output_dir()
    article_path = output_dir / f"claw_daily_{datetime.now().strftime('%Y-%m-%d')}.md"
    data_path = config.get_data_dir() / "summarized_news.json"

    if not article_path.exists():
        print(f"No article file found at {article_path}")
        return

    if not data_path.exists():
        print(f"No data file found at {data_path}")
        return

    # Read article
    with open(article_path, 'r', encoding='utf-8') as f:
        article = f.read()

    # Read data
    with open(data_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Reconstruct articles
    from .models import Article
    articles = []
    for a in data['articles']:
        art = Article.from_dict(a['article'])
        filtered = FilteredArticle(
            article=art,
            relevance_score=a.get('relevance_score', 0),
            matched_keywords=a.get('matched_keywords', []),
            ai_summary=a.get('ai_summary')
        )
        articles.append(filtered)

    # Build prompt for Agent
    date = datetime.now().strftime("%Y-%m-%d")
    prompt = PodcastGenerator.build_podcast_prompt(article, articles, date)

    print(f"\n=== Podcast Prompt for Agent ({date}) ===\n")
    print(prompt[:2000])
    print("\n... [truncated] ...")
    print("\nThe Agent should generate the podcast script and call save_script() to persist it.")


if __name__ == "__main__":
    main()
