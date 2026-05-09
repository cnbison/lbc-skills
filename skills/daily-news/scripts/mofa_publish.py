"""MoFA FM auto-publish module for podcast distribution."""

import logging
import json
from pathlib import Path
from datetime import datetime
from typing import Optional, Dict, Any

from .mofa_client import MoFAFMClient
from .config import get_config

logger = logging.getLogger(__name__)


class MoFAPublisher:
    """Publisher for automatically uploading podcasts to MoFA FM."""

    def __init__(self, config=None):
        """
        Initialize MoFA publisher.

        Args:
            config: Configuration object (uses default if not provided)
        """
        self.config = config or get_config()
        self.mofa_config = self._get_mofa_config()

        # Initialize client
        self.client = MoFAFMClient(
            base_url=self.mofa_config.get('base_url', 'https://mofa.fm/api'),
            token=self.mofa_config.get('auth', {}).get('token'),
            username=self.mofa_config.get('auth', {}).get('username'),
            password=self.mofa_config.get('auth', {}).get('password'),
            timeout=self.mofa_config.get('api', {}).get('timeout', 30),
            max_retries=self.mofa_config.get('api', {}).get('max_retries', 3),
            auto_relogin=self.mofa_config.get('auto_relogin', True),
            env_file='.env'
        )

        # Show configuration
        self.show_config = self.mofa_config.get('show', {})
        self.show_slug = self.show_config.get('slug', 'claw-daily')

    def _get_mofa_config(self) -> Dict[str, Any]:
        """Get MoFA FM configuration."""
        # Try to get from config.yaml
        if hasattr(self.config, 'config'):
            return self.config.config.get('mofa_fm', {})

        # Fallback to empty dict
        return {}

    def _ensure_authenticated(self):
        """Ensure client is authenticated (login every time for security)."""
        if self.client.username and self.client.password:
            logger.info("Authenticating with username/password...")
            self.client.login()
        else:
            raise ValueError(
                "No authentication configured. "
                "Set MOFA_FM_USERNAME and MOFA_FM_PASSWORD in config"
            )

    def _ensure_show_exists(self) -> Dict[str, Any]:
        """
        Ensure the configured show exists, create if not.

        Returns:
            Show dict
        """
        try:
            # Try to get show
            show = self.client.get_show(self.show_slug)
            logger.info(f"Show '{self.show_slug}' exists")
            return show
        except Exception as e:
            # Show doesn't exist, create it
            if self.show_config.get('auto_create', True):
                logger.info(f"Show '{self.show_slug}' not found, creating...")

                show = self.client.create_show(
                    slug=self.show_slug,
                    title=self.show_config.get('title', 'Claw日报'),
                    description=self.show_config.get(
                        'description',
                        '每日AI和自动化技术新闻播客'
                    ),
                    content_type=self.show_config.get('content_type', 'podcast'),
                    visibility=self.show_config.get('visibility', 'public')
                )

                logger.info(f"✓ Show created: {show['title']} ({show['slug']})")
                return show
            else:
                raise ValueError(
                    f"Show '{self.show_slug}' does not exist and auto_create is disabled"
                )

    def _find_episode_by_title(self, title: str) -> Optional[Dict[str, Any]]:
        """
        Find an episode by title in the current show.

        Args:
            title: Episode title to search for

        Returns:
            Episode dict if found, None otherwise
        """
        try:
            # Get all episodes for this show
            episodes = self.client.list_episodes(show_slug=self.show_slug, limit=50)

            # Find matching episode
            for ep in episodes:
                if ep.get('title') == title:
                    logger.info(f"Found existing episode with title '{title}': ID {ep.get('id')}")
                    return ep

            logger.info(f"No existing episode found with title '{title}'")
            return None

        except Exception as e:
            logger.warning(f"Error searching for existing episode: {e}")
            return None

    def publish(
        self,
        script_path: str,
        audio_path: str,
        date: str = None,
        title: str = None,
        description: str = None
    ) -> Dict[str, Any]:
        """
        Publish podcast to MoFA FM.

        Args:
            script_path: Path to podcast script file
            audio_path: Path to audio file (MP3)
            date: Episode date (YYYY-MM-DD format)
            title: Episode title (auto-generated if not provided)
            description: Episode description (auto-generated if not provided)

        Returns:
            Published episode dict
        """
        if date is None:
            date = datetime.now().strftime("%Y-%m-%d")

        logger.info(f"Publishing podcast for {date} to MoFA FM...")

        # Ensure authenticated
        self._ensure_authenticated()

        # Ensure show exists
        show = self._ensure_show_exists()

        # Read script content
        script_path = Path(script_path)
        if not script_path.exists():
            raise FileNotFoundError(f"Script file not found: {script_path}")

        with open(script_path, 'r', encoding='utf-8') as f:
            script_content = f.read()

        # Check audio file exists
        audio_path = Path(audio_path)
        if not audio_path.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")

        # Generate title and description if not provided
        if title is None:
            title = f"Claw日报 - {date}"

        if description is None:
            description = f"AI和自动化技术新闻播客 - {date}"

        # Check if episode with this title already exists
        logger.info(f"Checking if episode '{title}' already exists...")
        existing_episode = self._find_episode_by_title(title)

        episode = None
        is_new_episode = False

        if existing_episode:
            logger.info(f"✓ Found existing episode: {existing_episode.get('id')}")
            logger.info(f"  Updating existing episode instead of creating new one")
            episode = existing_episode
            episode_id = existing_episode.get('id')

            # Update script
            logger.info("Updating episode script...")
            self.client.update_episode_script(episode_id, script_content)
            logger.info("✓ Script updated")

            # Update audio file if provided
            if audio_path and audio_path.exists():
                logger.info("Updating episode audio...")
                try:
                    self.client.update_episode_audio(episode_id, audio_path)
                    logger.info("✓ Audio updated")
                except Exception as e:
                    logger.warning(f"⚠️ Failed to update audio: {e}")
                    logger.info("Continuing with existing audio...")
        else:
            # Create new episode
            logger.info(f"Creating new episode: {title}...")
            is_new_episode = True

            # Try to upload audio if file exists
            audio_file_to_upload = None
            if audio_path and audio_path.exists():
                audio_file_to_upload = str(audio_path)
                logger.info(f"  Will upload audio: {audio_file_to_upload}")

            try:
                episode = self.client.create_episode(
                    show_slug=self.show_slug,
                    title=title,
                    description=description,
                    script=script_content,
                    audio_file_path=audio_file_to_upload
                )
            except Exception as e:
                # Check if episode was created despite the error (e.g., timeout during upload)
                logger.warning(f"Error during episode creation: {e}")
                logger.info("Checking if episode was created despite the error...")

                # Try to find the newly created episode
                found_episode = self._find_episode_by_title(title)
                if found_episode:
                    logger.info(f"✓ Episode was created: {found_episode.get('id')}")
                    episode = found_episode
                    # Check if audio was uploaded
                    if not found_episode.get('audio_file'):
                        logger.warning("⚠️ Audio file may not have been uploaded due to timeout")
                else:
                    logger.error("✗ Episode was not created. Please check your connection and try again.")
                    raise

            episode_id = episode.get('id')
            logger.info(f"✓ Episode created: {episode_id}")

            # Update script separately (in case it's too long for create_episode)
            logger.info("Updating episode script...")
            self.client.update_episode_script(episode_id, script_content)
            logger.info("✓ Script updated")

        # Verify script was saved successfully
        logger.info("Verifying script was saved...")
        try:
            episode_check = self.client.get_episode(episode_id)
            script_check = episode_check.get('script', '')
            script_length = len(script_check) if script_check else 0

            if script_length > 0:
                logger.info(f"✓ Script verification successful! Length: {script_length} characters")
            else:
                logger.warning(f"⚠️ Script appears to be empty (length: {script_length})")
        except Exception as e:
            logger.warning(f"⚠️ Could not verify script: {e}")

        # Prepare result
        result = {
            'episode_id': episode_id,
            'show_slug': self.show_slug,
            'title': title,
            'date': date,
            'url': f"https://mofa.fm/shows/{self.show_slug}/",
            'script_path': str(script_path),
            'audio_path': str(audio_path)
        }

        logger.info(f"✓ Podcast published successfully!")
        logger.info(f"  Episode ID: {episode_id}")
        logger.info(f"  Show: {self.show_slug}")
        logger.info(f"  URL: {result['url']}")

        return result

    def publish_with_audio_upload(
        self,
        script_path: str,
        audio_path: str,
        date: str = None,
        title: str = None,
        description: str = None
    ) -> Dict[str, Any]:
        """
        Publish podcast with audio file upload.

        Note: This requires the MoFA FM API to support direct file upload.
        If not supported, you'll need to upload audio to a hosting service
        (like OSS, S3, etc.) and provide the URL.

        Args:
            script_path: Path to podcast script file
            audio_path: Path to audio file
            date: Episode date
            title: Episode title
            description: Episode description

        Returns:
            Published episode dict
        """
        # For now, use the regular publish method
        # TODO: Implement audio file upload if API supports it
        logger.warning(
            "Direct audio upload not yet implemented. "
            "Please upload audio to a hosting service and provide the URL."
        )

        return self.publish(script_path, audio_path, date, title, description)


def main():
    """CLI for testing MoFA publisher."""
    import sys

    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    # Parse arguments
    import argparse
    parser = argparse.ArgumentParser(description='Publish podcast to MoFA FM')
    parser.add_argument('--date', type=str, help='Episode date (YYYY-MM-DD)')
    parser.add_argument('--dry-run', action='store_true', help='Simulate without publishing')
    parser.add_argument('--script', type=str, help='Path to script file')
    parser.add_argument('--audio', type=str, help='Path to audio file')

    args = parser.parse_args()

    # Default date
    date = args.date or datetime.now().strftime("%Y-%m-%d")

    # Get config
    config = get_config()
    output_dir = config.get_output_dir()

    # Default paths
    if not args.script:
        script_filename = config.get_podcast_script_filename(date)
        args.script = str(output_dir / script_filename)

    if not args.audio:
        audio_filename = config.get_podcast_audio_filename(date)
        args.audio = str(output_dir / audio_filename)

    # Check files exist
    script_path = Path(args.script)
    audio_path = Path(args.audio)

    if not script_path.exists():
        logger.error(f"Script file not found: {script_path}")
        sys.exit(1)

    if not audio_path.exists():
        logger.error(f"Audio file not found: {audio_path}")
        sys.exit(1)

    # Dry run mode
    if args.dry_run:
        logger.info("[DRY RUN] Would publish the following:")
        logger.info(f"  Date: {date}")
        logger.info(f"  Script: {script_path}")
        logger.info(f"  Audio: {audio_path}")

        # Read and display script preview
        with open(script_path, 'r', encoding='utf-8') as f:
            script_content = f.read()

        logger.info(f"  Script length: {len(script_content)} characters")
        logger.info(f"  Audio size: {audio_path.stat().st_size / 1024 / 1024:.2f} MB")

        logger.info("\n[DRY RUN] Skipped actual publishing")
        sys.exit(0)

    # Actual publishing
    try:
        publisher = MoFAPublisher(config)
        result = publisher.publish(
            script_path=str(script_path),
            audio_path=str(audio_path),
            date=date
        )

        # Save publish result
        result_path = output_dir / f"mofa_publish_{date}.json"
        with open(result_path, 'w', encoding='utf-8') as f:
            json.dump(result, f, indent=2, ensure_ascii=False)

        logger.info(f"\n✓ Publish result saved to: {result_path}")

        # Display result
        print("\n" + "=" * 60)
        print("PUBLISH RESULT")
        print("=" * 60)
        print(f"Episode ID: {result['episode_id']}")
        print(f"Show: {result['show_slug']}")
        print(f"Title: {result['title']}")
        print(f"Date: {result['date']}")
        print(f"URL: {result['url']}")
        print("=" * 60)

    except Exception as e:
        logger.error(f"Failed to publish: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
