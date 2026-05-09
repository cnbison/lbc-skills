"""Data models for OpenClaw Daily News system."""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional, List, Dict, Any
from enum import Enum
import hashlib


class SourceType(Enum):
    """RSS source categories."""
    AI_MEDIA = "ai_media"
    AI_SPECIALIZED = "ai_specialized"
    LLM_AGENT = "llm_agent"
    GITHUB = "github"
    SECURITY = "security"
    RESEARCH = "research"
    COMMUNITY = "community"
    CHINESE_AI = "chinese_ai"
    GOOGLE_NEWS = "google_news"


@dataclass
class Article:
    """Represents a news article."""
    title: str
    link: str
    published: Optional[datetime] = None
    author: Optional[str] = None
    summary: Optional[str] = None
    content: Optional[str] = None
    source: Optional[str] = None
    source_category: Optional[SourceType] = None

    def __post_init__(self):
        """Generate unique ID after initialization."""
        self.id = self._generate_id()

    def _generate_id(self) -> str:
        """Generate unique ID based on link and title."""
        content = f"{self.link}|{self.title}"
        return hashlib.md5(content.encode()).hexdigest()[:12]

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'id': self.id,
            'title': self.title,
            'link': self.link,
            'published': self.published.isoformat() if self.published else None,
            'author': self.author,
            'summary': self.summary,
            'content': self.content,
            'source': self.source,
            'source_category': self.source_category.value if self.source_category else None
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Article':
        """Create from dictionary."""
        published = data.get('published')
        if published and isinstance(published, str):
            published = datetime.fromisoformat(published)

        source_category = data.get('source_category')
        if source_category and isinstance(source_category, str):
            source_category = SourceType(source_category)

        return cls(
            title=data['title'],
            link=data['link'],
            published=published,
            author=data.get('author'),
            summary=data.get('summary'),
            content=data.get('content'),
            source=data.get('source'),
            source_category=source_category
        )


@dataclass
class FilteredArticle:
    """Represents a filtered and summarized article."""
    article: Article
    relevance_score: float = 0.0
    matched_keywords: List[str] = field(default_factory=list)
    ai_summary: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'article': self.article.to_dict(),
            'relevance_score': self.relevance_score,
            'matched_keywords': self.matched_keywords,
            'ai_summary': self.ai_summary
        }


@dataclass
class DailyReport:
    """Represents a daily news report."""
    date: str
    articles: List[FilteredArticle] = field(default_factory=list)
    generated_at: datetime = field(default_factory=datetime.now)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'date': self.date,
            'articles': [a.to_dict() for a in self.articles],
            'generated_at': self.generated_at.isoformat(),
            'article_count': len(self.articles)
        }


@dataclass
class GeneratedContent:
    """Represents generated content (article and podcast)."""
    date: str
    article_markdown: str
    podcast_script: str
    podcast_audio_path: Optional[str] = None
    generated_at: datetime = field(default_factory=datetime.now)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'date': self.date,
            'article_markdown': self.article_markdown,
            'podcast_script': self.podcast_script,
            'podcast_audio_path': self.podcast_audio_path,
            'generated_at': self.generated_at.isoformat()
        }


