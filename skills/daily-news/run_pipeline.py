#!/usr/bin/env python3
"""
OpenClaw Daily News Pipeline
Main orchestrator for the daily news automation system.
"""

import sys
import logging
from pathlib import Path
from datetime import datetime
import argparse
import json

# Add scripts directory to path
scripts_dir = Path(__file__).parent / "scripts"
sys.path.insert(0, str(scripts_dir))

from scripts.config import get_config
from scripts.collect_news import NewsCollector
from scripts.filter_news import NewsFilter
from scripts.tts_generate import TTSGenerator
from scripts.models import GeneratedContent

# MoFA FM publishing (optional)
try:
    from scripts.mofa_publish import MoFAPublisher
    MOFA_AVAILABLE = True
except ImportError:
    MOFA_AVAILABLE = False
    logger.warning("MoFA FM publisher not available. Install to enable auto-publish.")

# Configure logging
log_path = Path.cwd() / 'output' / 'daily-news' / 'pipeline.log'
log_path.parent.mkdir(parents=True, exist_ok=True)
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(log_path),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger(__name__)


class Pipeline:
    """Main pipeline orchestrator."""

    def __init__(self, config_path: str = None, date: str = None):
        """
        Initialize pipeline.

        Args:
            config_path: Path to configuration file
            date: Target date (YYYY-MM-DD format, default: today)
        """
        self.config = get_config(config_path)
        self.collector = NewsCollector(self.config)

        if date:
            # Specified date: window is that calendar day (00:00 ~ 23:59)
            self.date = date
            ref_date = datetime.strptime(date, '%Y-%m-%d') + timedelta(hours=23, minutes=59, seconds=59)
        else:
            # "Today": window is the last 24 hours from now
            self.date = datetime.now().strftime("%Y-%m-%d")
            ref_date = datetime.now()

        self.news_filter = NewsFilter(self.config, reference_date=ref_date)
        self.tts_generator = TTSGenerator(self.config)

        logger.info(f"Pipeline initialized for date: {self.date}, reference: {ref_date}")

    def run(self, skip_tts: bool = False, mofa_publish: bool = False) -> GeneratedContent:
        """
        Run the complete pipeline.

        Args:
            skip_tts: Skip TTS audio generation step
            mofa_publish: Publish to MoFA FM platform

        Returns:
            GeneratedContent object with results
        """
        logger.info("=" * 60)
        logger.info("Starting OpenClaw Daily News Pipeline")
        logger.info("=" * 60)

        try:
            # Step 1: Collect news
            logger.info("\n[Step 1/3] Collecting news from RSS feeds...")
            articles = self.collector.collect_all()
            self.collector.save_raw_news(articles)

            if not articles:
                logger.warning("No articles collected. Pipeline stopped.")
                return None

            logger.info(f"✓ Collected {len(articles)} articles")

            # Step 2: Filter news
            logger.info("\n[Step 2/3] Filtering articles...")
            filtered = self.news_filter.filter(articles)
            self.news_filter.save_filtered_news(filtered)

            if not filtered:
                logger.warning("No articles passed filtering. Pipeline stopped.")
                return None

            logger.info(f"✓ Filtered to {len(filtered)} relevant articles")

            # Step 3: Generate podcast audio (TTS)
            podcast_audio_path = None
            article_markdown = ""
            podcast_script = ""

            output_dir = self.config.get_output_dir()
            article_path = output_dir / self.config.get_article_filename(self.date)
            script_path = output_dir / self.config.get_podcast_script_filename(self.date)

            if article_path.exists():
                with open(article_path, 'r', encoding='utf-8') as f:
                    article_markdown = f.read()
                logger.info(f"✓ Found existing article: {article_path}")

            if script_path.exists():
                with open(script_path, 'r', encoding='utf-8') as f:
                    podcast_script = f.read()
                logger.info(f"✓ Found existing podcast script: {script_path}")

            if not skip_tts and podcast_script:
                logger.info("\n[Step 3/3] Generating podcast audio...")
                audio_path = self.tts_generator.generate(podcast_script, self.date)
                if audio_path:
                    podcast_audio_path = str(audio_path)
                    logger.info("✓ Podcast audio generated and saved")
                else:
                    logger.warning("⚠ Audio generation failed, continuing...")
            elif skip_tts:
                logger.info("\n[Step 3/3] Skipping TTS audio generation")
            else:
                logger.info("\n[Step 3/3] No podcast script found, skipping TTS")
                logger.info("    (Run article and podcast generation via Agent first)")

            # Create result
            result = GeneratedContent(
                date=self.date,
                article_markdown=article_markdown,
                podcast_script=podcast_script,
                podcast_audio_path=podcast_audio_path
            )

            # Save metadata
            self._save_metadata(result)

            # Step 4: Publish to MoFA FM (optional)
            if mofa_publish:
                self._publish_to_mofa(result)

            logger.info("\n" + "=" * 60)
            logger.info("Pipeline completed successfully!")
            logger.info("=" * 60)

            self._print_summary(result)

            return result

        except Exception as e:
            logger.error(f"\n✗ Pipeline failed: {e}", exc_info=True)
            raise

    def _save_metadata(self, content: GeneratedContent):
        """Save pipeline metadata."""
        metadata_path = self.config.get_output_dir() / f"metadata_{self.date}.json"
        with open(metadata_path, 'w', encoding='utf-8') as f:
            json.dump(content.to_dict(), f, ensure_ascii=False, indent=2)
        logger.info(f"Metadata saved to {metadata_path}")

    def _print_summary(self, content: GeneratedContent):
        """Print pipeline summary."""
        output_dir = self.config.get_output_dir()

        print("\n" + "=" * 60)
        print("PIPELINE SUMMARY")
        print("=" * 60)
        print(f"Date: {content.date}")
        print(f"\nGenerated files:")
        print(f"  Article:     {output_dir / self.config.get_article_filename(content.date)}")
        print(f"  Script:      {output_dir / self.config.get_podcast_script_filename(content.date)}")
        if content.podcast_audio_path:
            print(f"  Audio:       {content.podcast_audio_path}")
        print(f"  Metadata:    {output_dir / f'metadata_{content.date}.json'}")
        print("=" * 60)

    def _publish_to_mofa(self, content: GeneratedContent):
        """
        Publish content to MoFA FM platform.

        Args:
            content: GeneratedContent to publish
        """
        if not MOFA_AVAILABLE:
            logger.warning("MoFA FM publisher not available. Skipping publish.")
            return

        mofa_config = self.config.config.get('mofa_fm', {})
        if not mofa_config.get('enabled', False):
            logger.info("MoFA FM integration is disabled in config. Skipping publish.")
            return

        if not mofa_config.get('features', {}).get('auto_publish', False):
            logger.info("MoFA FM auto-publish is disabled in config. Skipping publish.")
            return

        logger.info("\n[Step 7/5] Publishing to MoFA FM...")

        try:
            output_dir = self.config.get_output_dir()
            script_filename = self.config.get_podcast_script_filename(content.date)
            audio_filename = self.config.get_podcast_audio_filename(content.date)

            script_path = output_dir / script_filename
            audio_path = output_dir / audio_filename

            if not script_path.exists():
                logger.warning(f"Script file not found: {script_path}. Skipping MoFA publish.")
                return

            if not audio_path.exists():
                logger.warning(f"Audio file not found: {audio_path}. Skipping MoFA publish.")
                return

            publisher = MoFAPublisher(self.config)
            result = publisher.publish(
                script_path=str(script_path),
                audio_path=str(audio_path),
                date=content.date
            )

            logger.info("✓ Published to MoFA FM successfully")
            logger.info(f"  Episode ID: {result['episode_id']}")
            logger.info(f"  URL: {result['url']}")

        except Exception as e:
            logger.error(f"✗ Failed to publish to MoFA FM: {e}")
            logger.info("Continuing pipeline execution despite publish failure...")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='OpenClaw Daily News Pipeline'
    )
    parser.add_argument(
        '--config', '-c',
        help='Path to configuration file (default: config/sources.yaml)'
    )
    parser.add_argument(
        '--skip-tts',
        action='store_true',
        help='Skip TTS audio generation step'
    )
    parser.add_argument(
        '--date', '-d',
        help='Date for the report (YYYY-MM-DD format, default: today)'
    )
    parser.add_argument(
        '--mofa-publish',
        action='store_true',
        help='Publish to MoFA FM platform after generation'
    )
    parser.add_argument(
        '--skip-mofa',
        action='store_true',
        help='Skip MoFA FM publishing even if enabled in config'
    )

    args = parser.parse_args()

    # Override date if provided
    if args.date:
        try:
            datetime.strptime(args.date, '%Y-%m-%d')
        except ValueError:
            logger.error(f"Invalid date format: {args.date}. Use YYYY-MM-DD.")
            sys.exit(1)

    # Create and run pipeline
    pipeline = Pipeline(config_path=args.config, date=args.date)

    # Determine MoFA publish setting
    mofa_publish = args.mofa_publish and not args.skip_mofa

    result = pipeline.run(
        skip_tts=args.skip_tts,
        mofa_publish=mofa_publish
    )

    if result:
        sys.exit(0)
    else:
        logger.error("Pipeline failed to generate content")
        sys.exit(1)


if __name__ == "__main__":
    main()
