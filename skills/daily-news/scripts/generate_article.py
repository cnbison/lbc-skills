"""Article generator data module — Agent-driven article generation.

This module no longer calls AI APIs directly. The Agent (Claude Code) reads
the summarized news data and writes the article markdown directly.
"""

import logging
from typing import List, Dict
from pathlib import Path
from datetime import datetime
import json

from .models import FilteredArticle
from .config import get_config

logger = logging.getLogger(__name__)


class ArticleGenerator:
    """Data helper for daily news articles. AI generation is done by the Agent."""

    def __init__(self, config=None):
        """
        Initialize article generator helper.

        Args:
            config: Configuration object (uses default if not provided)
        """
        self.config = config or get_config()

    def _categorize_for_display(self, articles: List[FilteredArticle]) -> Dict[str, List[FilteredArticle]]:
        """Categorize articles for display in article."""
        categories = {
            '重要新闻': [],
            '技术进展': [],
            '安全动态': [],
            'GitHub动态': [],
            '研究论文': [],
            '产业趋势': [],
            '社区讨论': []
        }

        for article in articles:
            art = article.article
            source_cat = art.source_category

            # Security articles
            if 'security' in art.source or 'hacker' in art.source.lower() or \
               any(k in art.title.lower() for k in ['security', 'vulnerability', 'exploit', '漏洞', '安全']):
                categories['安全动态'].append(article)
            # GitHub releases
            elif 'github.com' in art.link and 'release' in art.link:
                categories['GitHub动态'].append(article)
            # Research papers
            elif 'arxiv' in art.source or source_cat and 'research' in str(source_cat):
                categories['研究论文'].append(article)
            # Community
            elif source_cat and 'community' in str(source_cat):
                categories['社区讨论'].append(article)
            # High relevance articles
            elif article.relevance_score >= 3.0:
                categories['重要新闻'].append(article)
            # Tech progress
            elif any(k in art.title.lower() for k in ['gpt', 'llm', 'agent', 'model', 'release', 'launch']):
                categories['技术进展'].append(article)
            # Industry trends
            else:
                categories['产业趋势'].append(article)

        # Remove empty categories
        return {k: v for k, v in categories.items() if v}

    def _get_best_summary(self, article: FilteredArticle, min_length: int = 300) -> str:
        """
        Get the best summary for an article, ensuring minimum length.

        Args:
            article: FilteredArticle object
            min_length: Minimum required length in characters

        Returns:
            Best available summary (up to 500 characters)
        """
        art = article.article

        # Priority sources for summary
        sources = [
            article.ai_summary,      # AI摘要（最佳）
            art.summary,             # 原始摘要
            art.content[:1000] if art.content else ""  # 原始内容前1000字符
        ]

        # Try each source, return first one that meets minimum length
        for source in sources:
            if source and len(source) >= min_length:
                return source[:500]  # Return up to 500 characters

        # If no source is long enough, try combining
        if article.ai_summary and art.summary:
            combined = f"{article.ai_summary}\n{art.summary}"
            if len(combined) >= min_length:
                return combined[:500]

        # Fallback: return AI summary even if short (best quality)
        if article.ai_summary:
            return article.ai_summary

        # Last resort: return original summary or content
        return art.summary or (art.content[:500] if art.content else "暂无摘要")

    def _format_articles_for_prompt(self, articles: List[FilteredArticle]) -> str:
        """Format articles for Agent prompt."""
        formatted = []

        # Use top 15 articles
        for i, article in enumerate(articles[:15], 1):
            art = article.article

            # Get best summary with minimum 300 characters
            summary = self._get_best_summary(article, min_length=300)

            # Build relevance indicator
            relevance = f" (相关度: {article.relevance_score:.1f})" if article.relevance_score else ""

            # Build keywords string
            keywords_str = ""
            if article.matched_keywords:
                # Show top 3 keywords
                top_keywords = article.matched_keywords[:3]
                keywords_str = f", 关键词: {', '.join(top_keywords)}"

            # Format entry
            entry = f"{i}. **{art.title}**{relevance}\n"
            entry += f"   来源: {art.source or '未知'} | 发布时间: {art.published or '未知'}{keywords_str}\n"
            entry += f"   链接: {art.link}\n"
            entry += f"   内容: {summary}\n"
            entry += f"   {'─' * 60}\n"  # Separator line
            formatted.append(entry)

        return "\n".join(formatted)

    @staticmethod
    def build_article_prompt(articles: List[FilteredArticle], date: str = None) -> str:
        """
        Build a prompt for the Agent to generate the daily article.

        Args:
            articles: List of FilteredArticle objects
            date: Article date (YYYY-MM-DD format)

        Returns:
            Prompt string for the Agent
        """
        if date is None:
            date = datetime.now().strftime("%Y-%m-%d")

        generator = ArticleGenerator()
        categorized = generator._categorize_for_display(articles)
        articles_text = generator._format_articles_for_prompt(articles)
        categories_summary = "\n".join([f"- {cat}: {len(arts)}条" for cat, arts in categorized.items()])

        return f"""请根据以下资讯生成一篇《Claw 每日观察》文章。

日期: {date}

资讯分类统计:
{categories_summary}

今日资讯:
{articles_text}

请生成一篇结构完整、内容丰富的文章，包含以下部分:

# Claw 每日观察 - {date}

## 今日摘要
简要总结今日最重要的3-5条新闻（每条30-50字）

## 重要新闻
详细报道今日最重要的新闻事件（2-3条，每条150-250字）
- **重要**: 为每条新闻添加背景信息、技术细节、行业影响分析
- 如果新闻较少，可以深入分析现有新闻的多个角度

## 技术进展
介绍最新的技术突破和产品发布（2-3条，每条150-250字）
- 可以包括: 技术原理、应用场景、与现有方案的对比

## 安全动态
AI安全和Agent相关的安全资讯
- 如果今日没有相关新闻，可以说:"今日暂无重大安全动态，但提醒用户关注AI安全最佳实践..."
- 或者提供通用的AI安全建议

## GitHub动态
值得关注的开源项目更新
- 重点介绍项目功能、技术亮点、star数变化

## 研究前沿
重要的学术论文和研究进展
- 如果今日没有相关新闻，可以介绍相关的技术趋势或研究方向

## 社区热议
开发者社区的讨论热点和观点
- 可以包括: 不同观点的讨论、社区反应、用户反馈

## 今日总结
对全天资讯的总结和展望（150-200字）
- 包括: 今日主要动态概述、对未来发展的预测、对开发者的建议

**关键要求（必须遵守）**:
1. **避免重复**: 同一条新闻不要在不同部分重复出现
2. **合理分配**: 如果新闻较少，合理分配到各个部分，不要强行填充
3. **内容扩展**: 对于每条新闻，添加以下内容以增加字数和深度:
   - 背景信息（什么是相关技术/产品）
   - 技术细节（关键特性、技术栈）
   - 行业影响（对用户/开发者的影响）
   - 相关链接或类似产品（如果适用）
4. **列表格式**: 使用 "-" 开头的列表格式
5. **原文链接**: 每条新闻后添加 `[详情](链接URL)`
6. **字数要求**: 总字数1500-2500字（即使新闻少也要通过扩展达到要求）
7. **专业表达**: 使用专业但不生硬的语言
8. **突出重点**: 特别关注OpenClaw、AI Agent、大语言模型等内容

如果今日资讯较少，请通过以下方式增加内容深度:
- 添加技术背景解释
- 分析行业趋势和影响
- 提供实用建议和最佳实践
- 介绍类似产品或竞品对比
- 探讨未来发展方向

请生成文章:"""

    def _generate_fallback(self, articles: List[FilteredArticle], date: str, categorized: Dict) -> str:
        """Generate fallback article without AI."""
        lines = [
            f"# Claw 每日观察 - {date}",
            "",
            "## 今日摘要",
            ""
        ]

        # Add top articles
        for i, article in enumerate(articles[:5], 1):
            art = article.article
            summary = article.ai_summary or art.summary or ""
            lines.append(f"{i}. {art.title}")
            lines.append(f"   {summary[:100]}...")
            lines.append("")

        # Add categories
        for category, items in categorized.items():
            lines.append(f"\n## {category}")
            lines.append("")
            for item in items[:3]:
                art = item.article
                summary = item.ai_summary or art.summary or ""
                lines.append(f"### {art.title}")
                lines.append("")
                lines.append(f"{summary[:300]}")
                lines.append("")
                lines.append(f"[阅读原文]({art.link}) | 来源: {art.source}")
                lines.append("")

        return "\n".join(lines)

    def save_article(self, article: str, date: str = None, output_path: str = None):
        """
        Save generated article to file.

        Args:
            article: Generated article markdown
            date: Article date (YYYY-MM-DD format)
            output_path: Output file path
        """
        if output_path is None:
            output_dir = self.config.get_output_dir()
            output_dir.mkdir(parents=True, exist_ok=True)

            if date is None:
                date = datetime.now().strftime("%Y-%m-%d")

            filename = self.config.get_article_filename(date)
            output_path = output_dir / filename
        else:
            output_path = Path(output_path)

        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(article)

        logger.info(f"Saved article to {output_path}")


def main():
    """Main function for testing — prints the article prompt for Agent review."""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    # Load summarized news
    summarized_path = get_config().get_data_dir() / "summarized_news.json"

    if not summarized_path.exists():
        print("No summarized news file found. Run summarize_news.py first.")
        return

    with open(summarized_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Reconstruct articles
    from .models import Article
    articles = []
    for a in data['articles']:
        article = Article.from_dict(a['article'])
        filtered = FilteredArticle(
            article=article,
            relevance_score=a.get('relevance_score', 0),
            matched_keywords=a.get('matched_keywords', []),
            ai_summary=a.get('ai_summary')
        )
        articles.append(filtered)

    # Build prompt for Agent
    date = datetime.now().strftime("%Y-%m-%d")
    prompt = ArticleGenerator.build_article_prompt(articles, date)

    print(f"\n=== Article Prompt for Agent ({date}) ===\n")
    print(prompt[:2000])
    print("\n... [truncated] ...")
    print("\nThe Agent should generate the article markdown and call save_article() to persist it.")


if __name__ == "__main__":
    main()
