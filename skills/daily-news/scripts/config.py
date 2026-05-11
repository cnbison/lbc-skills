"""Configuration loader for OpenClaw Daily News system."""

import os
from pathlib import Path
from typing import Dict, List, Any
import yaml


def _load_env_file(env_path: Path = None) -> Dict[str, str]:
    """
    Load .env file and return environment variables.

    Args:
        env_path: Path to .env file (default: project root /.env)

    Returns:
        Dictionary of environment variables
    """
    if env_path is None:
        script_dir = Path(__file__).parent
        env_path = script_dir.parent / ".env"

    env_vars = {}
    if env_path.exists():
        with open(env_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    env_vars[key.strip()] = value.strip()
                    # Also set in os.environ
                    os.environ[key.strip()] = value.strip()

    return env_vars


class Config:
    """Configuration management for the OpenClaw Daily News system."""

    def __init__(self, config_path: str = None):
        """
        Initialize configuration.

        Args:
            config_path: Path to sources.yaml config file
        """
        if config_path is None:
            # Default to config/sources.yaml relative to script location
            script_dir = Path(__file__).parent
            config_path = script_dir.parent / "config" / "sources.yaml"

        self.config_path = Path(config_path)
        self.config = self._load_config()

    def _load_config(self) -> Dict[str, Any]:
        """Load configuration from YAML file."""
        # Load .env file first
        _load_env_file()

        if not self.config_path.exists():
            raise FileNotFoundError(f"Config file not found: {self.config_path}")

        with open(self.config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)

        # Expand environment variables in MoFA FM config
        if 'mofa_fm' in config:
            if 'auth' in config['mofa_fm']:
                # Expand token
                if 'token' in config['mofa_fm']['auth']:
                    token = config['mofa_fm']['auth']['token']
                    if isinstance(token, str) and token.startswith('${') and token.endswith('}'):
                        env_var = token[2:-1]
                        config['mofa_fm']['auth']['token'] = os.getenv(env_var, '')

                # Expand username/password if present
                for field in ['username', 'password']:
                    if field in config['mofa_fm']['auth']:
                        value = config['mofa_fm']['auth'][field]
                        if isinstance(value, str) and value.startswith('${') and value.endswith('}'):
                            env_var = value[2:-1]
                            config['mofa_fm']['auth'][field] = os.getenv(env_var, '')

        # Expand environment variables in TTS config
        if 'tts' in config and 'doubao' in config['tts']:
            for field in ['token', 'appid']:
                if field in config['tts']['doubao']:
                    value = config['tts']['doubao'][field]
                    if isinstance(value, str) and value.startswith('${') and value.endswith('}'):
                        env_var = value[2:-1]
                        config['tts']['doubao'][field] = os.getenv(env_var, '')

        return config

    def get_all_sources(self) -> List[str]:
        """Get all RSS feed URLs from all categories."""
        sources = []
        if 'sources' in self.config:
            for category, feeds in self.config['sources'].items():
                if isinstance(feeds, list):
                    sources.extend(feeds)
        return sources

    def get_sources_by_category(self, category: str) -> List[str]:
        """Get RSS feeds for a specific category."""
        if 'sources' in self.config and category in self.config['sources']:
            return self.config['sources'][category]
        return []

    def get_filter_keywords(self) -> List[str]:
        """Get list of filter keywords."""
        if 'filter' in self.config and 'keywords' in self.config['filter']:
            return self.config['filter']['keywords']
        return []

    def get_time_window_hours(self) -> int:
        """Get time window for filtering in hours."""
        if 'filter' in self.config and 'time_window_hours' in self.config['filter']:
            return self.config['filter']['time_window_hours']
        return 24

    def get_max_articles(self) -> int:
        """Get maximum number of articles to process."""
        if 'filter' in self.config and 'max_articles' in self.config['filter']:
            return self.config['filter']['max_articles']
        return 50

    def get_exclude_keywords(self) -> List[str]:
        """Get list of exclude keywords."""
        if 'filter' in self.config and 'exclude_keywords' in self.config['filter']:
            return [k.lower() for k in self.config['filter']['exclude_keywords']]
        return []

    def get_spam_patterns(self) -> List[str]:
        """Get list of spam regex patterns for title filtering."""
        if 'filter' in self.config and 'spam_patterns' in self.config['filter']:
            return self.config['filter']['spam_patterns']
        return []

    def get_min_article_length(self) -> int:
        """Get minimum article length for filtering."""
        if 'filter' in self.config and 'min_article_length' in self.config['filter']:
            return self.config['filter']['min_article_length']
        return 100

    def get_ai_config(self) -> Dict[str, Any]:
        """Get AI configuration."""
        return self.config.get('ai', {})

    def get_output_dir(self) -> Path:
        """Get output directory path (cwd/daily-news/output)."""
        default_dir = Path.cwd() / "daily-news" / "output"
        if 'output' in self.config and 'directory' in self.config['output']:
            configured = Path(self.config['output']['directory'])
            if configured.is_absolute():
                return configured
            return Path.cwd() / configured
        return default_dir

    def get_data_dir(self) -> Path:
        """Get data directory path (cwd/daily-news/data)."""
        return Path.cwd() / "daily-news" / "data"

    def get_article_filename(self, date: str) -> str:
        """Generate article filename for given date."""
        if 'output' in self.config and 'article_template' in self.config['output']:
            return self.config['output']['article_template'].format(date=date)
        return f"openclaw_daily_{date}.md"

    def get_podcast_script_filename(self, date: str) -> str:
        """Generate podcast script filename for given date."""
        if 'output' in self.config and 'podcast_script_template' in self.config['output']:
            return self.config['output']['podcast_script_template'].format(date=date)
        return f"openclaw_podcast_{date}.txt"

    def get_podcast_audio_filename(self, date: str) -> str:
        """Generate podcast audio filename for given date."""
        if 'output' in self.config and 'podcast_audio_template' in self.config['output']:
            return self.config['output']['podcast_audio_template'].format(date=date)
        return f"openclaw_daily_{date}.mp3"

    def get_tts_config(self) -> Dict[str, Any]:
        """Get TTS configuration."""
        return self.config.get('tts', {})


# Global config instance
_config = None


def get_config(config_path: str = None) -> Config:
    """Get or create global config instance."""
    global _config
    if _config is None:
        _config = Config(config_path)
    return _config
