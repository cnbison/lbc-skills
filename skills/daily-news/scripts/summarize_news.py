"""News summarizer data module — Agent-driven summary generation.

This module no longer calls AI APIs directly. The Agent (Claude Code) reads
the filtered news data and generates summaries, then uses save_summaries() to persist them.
"""

import logging
from typing import List, Optional
from pathlib import Path
from datetime import datetime
import json

from .models import FilteredArticle, Article
from .config import get_config

logger = logging.getLogger(__name__)


class AISummarizer:
    """Data helper for news summaries. AI generation is done by the Agent."""

    def __init__(self, config=None):
        """
        Initialize summarizer helper.

        Args:
            config: Configuration object (uses default if not provided)
        """
        self.config = config or get_config()

    @staticmethod
    def load_filtered(filtered_path: Path = None) -> List[FilteredArticle]:
        """
        Load filtered articles from JSON file.

        Args:
            filtered_path: Path to filtered_news.json

        Returns:
            List of FilteredArticle objects
        """
        if filtered_path is None:
            filtered_path = get_config().get_data_dir() / "filtered_news.json"

        if not filtered_path.exists():
            logger.warning(f"No filtered news file found at {filtered_path}")
            return []

        with open(filtered_path, 'r', encoding='utf-8') as f:
            data = json.load(f)

        articles = []
        for a in data.get('articles', []):
            article = Article.from_dict(a['article'])
            filtered = FilteredArticle(
                article=article,
                relevance_score=a.get('relevance_score', 0),
                matched_keywords=a.get('matched_keywords', []),
                ai_summary=a.get('ai_summary')
            )
            articles.append(filtered)

        return articles

    @staticmethod
    def build_summary_prompt(article: FilteredArticle) -> str:
        """
        Build a prompt for the Agent to generate a summary.

        Args:
            article: FilteredArticle to summarize

        Returns:
            Prompt string for the Agent
        """
        art = article.article
        title = art.title or "无标题"
        content = art.summary or art.content or ""

        if len(content) > 2000:
            content = content[:2000] + "..."

        return f"""请用中文总结以下资讯，要求：
1. 简明扼要，不超过150字
2. 突出核心信息
3. 使用客观、专业的语言

标题: {title}
来源: {art.source or '未知'}
链接: {art.link}

内容摘要:
{content}

请提供总结:"""

    def save_summaries(self, articles: List[FilteredArticle], output_path: str = None):
        """
        Save summarized articles to JSON file.

        Args:
            articles: List of FilteredArticle objects
            output_path: Output file path (default: data/summarized_news.json)
        """
        if output_path is None:
            output_path = self.config.get_data_dir() / "summarized_news.json"
        else:
            output_path = Path(output_path)

        output_path.parent.mkdir(parents=True, exist_ok=True)

        data = {
            'summarized_at': datetime.now().isoformat(),
            'count': len(articles),
            'articles': [a.to_dict() for a in articles]
        }

        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

        logger.info(f"Saved {len(articles)} summarized articles to {output_path}")


def main():
    """Main function for testing — loads filtered data and prints prompts for Agent review."""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    articles = AISummarizer.load_filtered()
    if not articles:
        print("No filtered news file found. Run filter_news.py first.")
        return

    print(f"\nLoaded {len(articles)} filtered articles.")
    print("The Agent should read these articles and generate ai_summary for each, then call save_summaries().\n")

    for i, article in enumerate(articles[:3], 1):
        print(f"\n=== Article {i} ===")
        print(f"Title: {article.article.title}")
        print(f"Prompt for Agent:\n{AISummarizer.build_summary_prompt(article)}\n")


if __name__ == "__main__":
    main()
