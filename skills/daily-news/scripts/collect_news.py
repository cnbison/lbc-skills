"""News collector module for fetching RSS feeds."""

import feedparser
import requests
from datetime import datetime, timedelta
from typing import List, Dict, Optional
import logging
import time
from pathlib import Path

from .models import Article, SourceType
from .config import get_config

logger = logging.getLogger(__name__)


class NewsCollector:
    """Collect news articles from RSS feeds."""

    def __init__(self, config=None):
        """
        Initialize news collector.

        Args:
            config: Configuration object (uses default if not provided)
        """
        self.config = config or get_config()
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
        })

    def fetch_feed(self, url: str, category: str = None, max_retries: int = 2) -> List[Article]:
        """
        Fetch articles from a single RSS feed.

        Args:
            url: RSS feed URL
            category: Source category for classification
            max_retries: Maximum retry attempts for failed connections

        Returns:
            List of Article objects
        """
        articles = []
        source_type = None

        # Map YAML category names to SourceType enum
        category_to_source_type = {
            'automation_rss': SourceType.COMMUNITY,
            'tech_media': SourceType.AI_MEDIA,
            'dev_media': SourceType.AI_SPECIALIZED,
            'github': SourceType.GITHUB,
            'opensource': SourceType.COMMUNITY,
            'security': SourceType.SECURITY,
            'community': SourceType.COMMUNITY,
            'chinese_media': SourceType.CHINESE_AI,
            'ai_agent_ecosystem': SourceType.LLM_AGENT,
        }
        if category:
            source_type = category_to_source_type.get(category)

        # Retry logic for connection errors
        for attempt in range(max_retries + 1):
            try:
                # Use feedparser with timeout
                feed = feedparser.parse(url, request_headers={'User-Agent': self.session.headers['User-Agent']})

                # Check if feed was parsed successfully
                if hasattr(feed, 'bozo') and feed.bozo and feed.bozo_exception:
                    # Connection reset errors often indicate rate limiting
                    error_msg = str(feed.bozo_exception)
                    if 'Connection reset' in error_msg or 'Errno 54' in error_msg:
                        if attempt < max_retries:
                            logger.warning(f"Connection reset by {url}, retrying ({attempt+1}/{max_retries})...")
                            time.sleep(2 ** attempt)  # Exponential backoff: 1s, 2s
                            continue
                        else:
                            logger.warning(f"Feed parsing warning for {url}: {feed.bozo_exception} (after {max_retries} retries)")
                    else:
                        logger.warning(f"Feed parsing warning for {url}: {feed.bozo_exception}")

                # Extract feed title as source
                source = getattr(feed.feed, 'title', url)

                for entry in feed.entries:
                    try:
                        # Parse publication date
                        published = None
                        if hasattr(entry, 'published_parsed') and entry.published_parsed:
                            published = datetime(*entry.published_parsed[:6])
                        elif hasattr(entry, 'updated_parsed') and entry.updated_parsed:
                            published = datetime(*entry.updated_parsed[:6])

                        # Extract link
                        link = entry.get('link', '')
                        if hasattr(entry, 'links') and len(entry.links) > 0:
                            link = entry.links[0].href

                        # Extract content/summary
                        summary = entry.get('summary', '')
                        content = ''

                        if hasattr(entry, 'content') and len(entry.content) > 0:
                            content = entry.content[0].get('value', '')
                        elif hasattr(entry, 'description'):
                            content = entry.description

                        # Extract author
                        author = entry.get('author', None)

                        article = Article(
                            title=entry.get('title', 'Untitled'),
                            link=link,
                            published=published,
                            author=author,
                            summary=summary[:500] if summary else None,  # Limit summary length
                            content=content[:2000] if content else None,  # Limit content length
                            source=source,
                            source_category=source_type
                        )
                        articles.append(article)

                    except Exception as e:
                        logger.debug(f"Error parsing entry: {e}")
                        continue

                logger.info(f"Fetched {len(articles)} articles from {url}")
                break  # Success, exit retry loop

            except Exception as e:
                logger.error(f"Error fetching feed {url}: {e}")

        return articles

    def collect_all(self) -> List[Article]:
        """
        Collect articles from all configured sources.

        Returns:
            List of all Article objects
        """
        all_articles = []
        sources_config = self.config.config.get('sources', {})

        total_feeds = sum(len(feeds) for feeds in sources_config.values())
        logger.info(f"Starting collection from {total_feeds} RSS feeds")

        for category, feeds in sources_config.items():
            if not isinstance(feeds, list):
                continue

            logger.info(f"Fetching {len(feeds)} feeds from category: {category}")

            for url in feeds:
                articles = self.fetch_feed(url, category)
                all_articles.extend(articles)

                # Adaptive rate limiting
                # Reddit and other strict sources need longer delays
                if 'reddit' in url.lower():
                    time.sleep(2.0)  # 2 seconds for Reddit
                elif 'github' in url.lower():
                    time.sleep(1.0)  # 1 second for GitHub
                else:
                    time.sleep(0.5)  # 0.5 seconds for others

        logger.info(f"Total articles collected: {len(all_articles)}")
        return all_articles

    def save_raw_news(self, articles: List[Article], output_path: str = None):
        """
        Save collected articles to JSON file.

        Args:
            articles: List of Article objects
            output_path: Output file path (default: data/raw_news.json)
        """
        import json

        if output_path is None:
            output_path = self.config.get_data_dir() / "raw_news.json"
        else:
            output_path = Path(output_path)

        output_path.parent.mkdir(parents=True, exist_ok=True)

        data = {
            'collected_at': datetime.now().isoformat(),
            'count': len(articles),
            'articles': [a.to_dict() for a in articles]
        }

        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

        logger.info(f"Saved {len(articles)} articles to {output_path}")


def main():
    """Main function for testing."""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    collector = NewsCollector()
    articles = collector.collect_all()
    collector.save_raw_news(articles)

    print(f"\nCollected {len(articles)} articles")


if __name__ == "__main__":
    main()
