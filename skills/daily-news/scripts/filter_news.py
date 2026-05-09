"""News filter module for filtering and deduplicating articles."""

import logging
from datetime import datetime, timedelta
from typing import List, Set, Dict, Optional
from pathlib import Path
import re
import json

from .models import Article, FilteredArticle, SourceType
from .config import get_config

logger = logging.getLogger(__name__)


class NewsFilter:
    """Filter and deduplicate news articles."""

    def __init__(self, config=None, reference_date: datetime = None):
        """
        Initialize news filter.

        Args:
            config: Configuration object (uses default if not provided)
            reference_date: Reference date for time window filtering (default: now)
        """
        self.config = config or get_config()
        self.keywords = [k.lower() for k in self.config.get_filter_keywords()]
        self.exclude_keywords = [k.lower() for k in self.config.get_exclude_keywords()]
        self.min_article_length = self.config.get_min_article_length()
        self.time_window_hours = self.config.get_time_window_hours()
        self.max_articles = self.config.get_max_articles()
        self.reference_date = reference_date or datetime.now()

    def filter_by_length(self, articles: List[Article]) -> List[Article]:
        """
        Filter articles by minimum content length.

        Args:
            articles: List of Article objects

        Returns:
            Articles meeting the minimum length requirement
        """
        if self.min_article_length <= 0:
            return articles

        result = []
        for article in articles:
            total_length = len(article.title or "") + len(article.summary or "") + len(article.content or "")
            if total_length >= self.min_article_length:
                result.append(article)

        logger.info(f"Length-filtered to {len(result)} articles (min: {self.min_article_length})")
        return result

    def filter_by_keywords(self, articles: List[Article]) -> List[FilteredArticle]:
        """
        Filter articles by keywords and calculate relevance scores.

        Enhanced with trending keyword matching and boost scoring.

        Args:
            articles: List of Article objects

        Returns:
            List of FilteredArticle objects with relevance scores
        """
        filtered = []

        for article in articles:
            title_lower = article.title.lower()
            summary_lower = (article.summary or "").lower()
            content_lower = (article.content or "").lower()

            matched_keywords = []
            relevance_score = 0.0
            trending_matches = 0

            # Match against static keywords
            for keyword in self.keywords:
                if keyword in title_lower:
                    matched_keywords.append(keyword)
                    relevance_score += 3.0  # High weight for title matches
                elif keyword in summary_lower:
                    matched_keywords.append(keyword)
                    relevance_score += 1.5  # Medium weight for summary matches
                elif keyword in content_lower:
                    matched_keywords.append(keyword)
                    relevance_score += 0.5  # Low weight for content matches

            if matched_keywords:
                filtered_article = FilteredArticle(
                    article=article,
                    relevance_score=relevance_score,
                    matched_keywords=matched_keywords
                )
                filtered.append(filtered_article)

        logger.info(f"Filtered to {len(filtered)} articles matching keywords")
        return filtered

    def filter_by_exclude_keywords(self, filtered_articles: List[FilteredArticle]) -> List[FilteredArticle]:
        """
        Filter out articles containing exclude keywords.

        Args:
            filtered_articles: List of FilteredArticle objects

        Returns:
            Filtered list with excluded articles removed
        """
        if not self.exclude_keywords:
            return filtered_articles

        result = []
        for filtered in filtered_articles:
            article = filtered.article
            text = f"{article.title} {article.summary or ''} {article.content or ''}".lower()

            excluded = any(kw in text for kw in self.exclude_keywords)
            if not excluded:
                result.append(filtered)

        logger.info(f"Exclude-filtered to {len(result)} articles")
        return result

    def filter_by_time(self, filtered_articles: List[FilteredArticle]) -> List[FilteredArticle]:
        """
        Filter articles by time window relative to reference date.

        Args:
            filtered_articles: List of FilteredArticle objects

        Returns:
            Filtered list within time window
        """
        cutoff_time = self.reference_date - timedelta(hours=self.time_window_hours)
        time_filtered = []

        for filtered in filtered_articles:
            article = filtered.article
            if article.published and article.published >= cutoff_time:
                time_filtered.append(filtered)

        logger.info(f"Time-filtered to {len(time_filtered)} articles within {self.time_window_hours}h window")
        return time_filtered

    def deduplicate(self, filtered_articles: List[FilteredArticle]) -> List[FilteredArticle]:
        """
        Remove duplicate articles based on title similarity and links.

        Args:
            filtered_articles: List of FilteredArticle objects

        Returns:
            Deduplicated list
        """
        seen_ids: Set[str] = set()
        seen_titles: Set[str] = set()
        deduplicated = []

        for filtered in filtered_articles:
            article = filtered.article

            # Skip if we've seen this article ID
            if article.id in seen_ids:
                continue

            # Check for similar titles (simple normalization)
            title_normalized = re.sub(r'[^\w\s]', '', article.title.lower()).strip()

            # Check if this title is very similar to any we've seen
            is_duplicate = False
            for seen_title in seen_titles:
                if self._titles_similar(title_normalized, seen_title):
                    is_duplicate = True
                    break

            if not is_duplicate:
                seen_ids.add(article.id)
                seen_titles.add(title_normalized)
                deduplicated.append(filtered)

        logger.info(f"Deduplicated from {len(filtered_articles)} to {len(deduplicated)} articles")
        return deduplicated

    def _titles_similar(self, title1: str, title2: str, threshold: float = 0.85) -> bool:
        """
        Check if two titles are similar using simple word overlap.

        Args:
            title1: First title
            title2: Second title
            threshold: Similarity threshold (0-1)

        Returns:
            True if titles are similar above threshold
        """
        words1 = set(title1.split())
        words2 = set(title2.split())

        if not words1 or not words2:
            return False

        intersection = words1.intersection(words2)
        union = words1.union(words2)

        similarity = len(intersection) / len(union) if union else 0
        return similarity >= threshold

    def rank_and_limit(self, filtered_articles: List[FilteredArticle]) -> List[FilteredArticle]:
        """
        Rank articles by relevance score and limit to max_articles.

        Args:
            filtered_articles: List of FilteredArticle objects

        Returns:
            Ranked and limited list
        """
        # Sort by relevance score (descending)
        ranked = sorted(filtered_articles, key=lambda x: x.relevance_score, reverse=True)

        # Limit to max_articles
        limited = ranked[:self.max_articles]

        logger.info(f"Ranked and limited to {len(limited)} articles (max: {self.max_articles})")
        return limited

    def categorize_articles(self, filtered_articles: List[FilteredArticle]) -> Dict[str, List[FilteredArticle]]:
        """
        Categorize articles by source type.

        Args:
            filtered_articles: List of FilteredArticle objects

        Returns:
            Dictionary mapping categories to article lists
        """
        categories = {
            'security': [],
            'research': [],
            'github': [],
            'ai_media': [],
            'ai_specialized': [],
            'llm_agent': [],
            'community': [],
            'chinese': [],
            'other': []
        }

        for filtered in filtered_articles:
            article = filtered.article
            category = article.source_category

            if category == SourceType.SECURITY:
                categories['security'].append(filtered)
            elif category == SourceType.RESEARCH:
                categories['research'].append(filtered)
            elif category == SourceType.GITHUB:
                categories['github'].append(filtered)
            elif category == SourceType.AI_MEDIA:
                categories['ai_media'].append(filtered)
            elif category == SourceType.AI_SPECIALIZED:
                categories['ai_specialized'].append(filtered)
            elif category == SourceType.LLM_AGENT:
                categories['llm_agent'].append(filtered)
            elif category == SourceType.COMMUNITY:
                categories['community'].append(filtered)
            elif category == SourceType.CHINESE_AI:
                categories['chinese'].append(filtered)
            else:
                categories['other'].append(filtered)

        return categories

    def filter(self, articles: List[Article]) -> List[FilteredArticle]:
        """
        Run complete filtering pipeline.

        Args:
            articles: List of Article objects

        Returns:
            Filtered, deduplicated, and ranked list
        """
        logger.info(f"Starting filtering pipeline with {len(articles)} articles")

        # Step 0: Filter by minimum content length
        articles = self.filter_by_length(articles)

        # Step 1: Filter by keywords
        filtered = self.filter_by_keywords(articles)

        # Step 2: Filter by exclude keywords
        filtered = self.filter_by_exclude_keywords(filtered)

        # Step 3: Filter by time
        filtered = self.filter_by_time(filtered)

        # Step 4: Deduplicate
        filtered = self.deduplicate(filtered)

        # Step 5: Rank and limit
        filtered = self.rank_and_limit(filtered)

        logger.info(f"Filtering complete: {len(filtered)} articles")
        return filtered

    def save_filtered_news(self, filtered_articles: List[FilteredArticle], output_path: str = None):
        """
        Save filtered articles to JSON file.

        Args:
            filtered_articles: List of FilteredArticle objects
            output_path: Output file path (default: data/filtered_news.json)
        """
        import json

        if output_path is None:
            output_path = self.config.get_data_dir() / "filtered_news.json"
        else:
            output_path = Path(output_path)

        output_path.parent.mkdir(parents=True, exist_ok=True)

        categorized = self.categorize_articles(filtered_articles)

        data = {
            'filtered_at': datetime.now().isoformat(),
            'count': len(filtered_articles),
            'categories': {k: len(v) for k, v in categorized.items()},
            'articles': [a.to_dict() for a in filtered_articles]
        }

        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

        logger.info(f"Saved {len(filtered_articles)} filtered articles to {output_path}")


def main():
    """Main function for testing."""
    import argparse
    parser = argparse.ArgumentParser(description='Filter news articles')
    parser.add_argument('--date', help='Reference date for time window (YYYY-MM-DD, default: today)')
    args = parser.parse_args()

    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    # Load raw news from file
    import json
    raw_path = get_config().get_data_dir() / "raw_news.json"

    if not raw_path.exists():
        print("No raw news file found. Run collect_news.py first.")
        return

    with open(raw_path, 'r', encoding='utf-8') as f:
        raw_data = json.load(f)

    articles = [Article.from_dict(a) for a in raw_data['articles']]

    # Parse reference date
    ref_date = None
    if args.date:
        # Specified date: window is that calendar day (00:00 ~ 23:59)
        ref_date = datetime.strptime(args.date, '%Y-%m-%d') + timedelta(hours=23, minutes=59, seconds=59)

    # Filter
    news_filter = NewsFilter(reference_date=ref_date)
    filtered = news_filter.filter(articles)
    news_filter.save_filtered_news(filtered)

    # Print summary
    print(f"\nFiltered to {len(filtered)} articles")
    categorized = news_filter.categorize_articles(filtered)
    for category, items in categorized.items():
        if items:
            print(f"  {category}: {len(items)}")


if __name__ == "__main__":
    main()
