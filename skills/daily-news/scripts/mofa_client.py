"""MoFA FM API client for podcast platform integration."""

import logging
import requests
from typing import List, Dict, Optional, Any
from dataclasses import dataclass
from pathlib import Path
import time

logger = logging.getLogger(__name__)


@dataclass
class TrendingItem:
    """Trending news item from MoFA FM."""
    source: str
    title: str
    hot_score: int
    url: str
    desc: str = ""
    cover: str = ""

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'source': self.source,
            'title': self.title,
            'hot_score': self.hot_score,
            'url': self.url,
            'desc': self.desc,
            'cover': self.cover
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TrendingItem':
        """Create from dictionary."""
        return cls(
            source=data.get('source', ''),
            title=data.get('title', ''),
            hot_score=data.get('hot_score', 0),
            url=data.get('url', ''),
            desc=data.get('desc', ''),
            cover=data.get('cover', '')
        )


class MoFAFMClient:
    """Client for MoFA FM Podcast Platform API."""

    def __init__(
        self,
        base_url: str = "https://mofa.fm/api",
        token: Optional[str] = None,
        username: Optional[str] = None,
        password: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3,
        auto_relogin: bool = True,
        env_file: str = ".env"
    ):
        """
        Initialize MoFA FM client.

        Args:
            base_url: API base URL
            token: JWT token (recommended)
            username: Username for authentication
            password: Password for authentication
            timeout: Request timeout in seconds
            max_retries: Maximum retry attempts
            auto_relogin: Automatically re-login on 401 errors (default: True)
            env_file: Path to .env file to save updated tokens
        """
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self.max_retries = max_retries
        self.auto_relogin = auto_relogin
        self.env_file = env_file

        # Authentication
        self.token = token
        self.username = username
        self.password = password

        # Session for connection pooling
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json'
        })

        # Set token if provided
        if self.token:
            self._set_auth_header()

    def _set_auth_header(self):
        """Set authorization header with token."""
        if self.token:
            self.session.headers.update({
                'Authorization': f'Bearer {self.token}'
            })

    def _upload_audio_with_relogin(
        self,
        audio_path: Path,
        data: Dict[str, Any],
        _retried_auth: bool = False,
        _retry_count: int = 0
    ) -> Dict[str, Any]:
        """
        Upload audio file with automatic relogin on 401 errors and retry on network errors.

        Args:
            audio_path: Path to audio file
            data: Form data to send with the upload
            _retried_auth: Internal flag to prevent infinite auth retry loops
            _retry_count: Internal counter for network retry attempts

        Returns:
            Response JSON data

        Raises:
            requests.RequestException: On failure after all retries
        """
        if not audio_path.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")

        # Max retries for network errors (not including auth retries)
        MAX_NETWORK_RETRIES = 3
        BASE_RETRY_DELAY = 2  # seconds

        url = f"{self.base_url}/podcasts/episodes/create/"

        # Remove Content-Type header for file upload
        original_headers = self.session.headers.copy()
        if 'Content-Type' in self.session.headers:
            del self.session.headers['Content-Type']

        try:
            file_size = audio_path.stat().st_size
            logger.info(f"Uploading audio file: {audio_path.name} ({file_size / 1024 / 1024:.2f} MB)")

            with open(audio_path, 'rb') as f:
                files = {'audio_file': (audio_path.name, f, 'audio/mpeg')}

                # Calculate timeout based on file size (min 60s, max 300s)
                timeout = min(max(60, file_size // 1024), 300)
                logger.debug(f"Upload timeout: {timeout}s")

                response = self.session.post(
                    url,
                    headers={'Authorization': f'Bearer {self.token}'},
                    data=data,
                    files=files,
                    timeout=timeout
                )

                # Check for authentication errors
                if response.status_code == 401:
                    if self.auto_relogin and not _retried_auth and self.username and self.password:
                        logger.warning("Authentication failed (401) during audio upload. Attempting auto-relogin...")
                        try:
                            self.login()
                            logger.info("✓ Auto-relogin successful. Retrying audio upload...")
                            # Update .env file with new token
                            if self.token:
                                self._update_env_token(self.token)
                            # Close the file and retry
                            f.close()
                            return self._upload_audio_with_relogin(audio_path, data, _retried_auth=True)
                        except Exception as relogin_error:
                            logger.error(f"Auto-relogin failed: {relogin_error}")
                            raise requests.RequestException(f"Authentication failed during audio upload and auto-relogin also failed: {relogin_error}")
                    else:
                        logger.error("Authentication failed (401) during audio upload. Please check your credentials or token.")
                        raise requests.RequestException("Authentication failed (401)")

                response.raise_for_status()
                result = response.json()
                logger.info(f"✓ Upload successful: {audio_path.name}")
                return result

        except (ConnectionResetError, ConnectionError, requests.exceptions.ConnectionError) as e:
            # Retry on network errors
            if _retry_count < MAX_NETWORK_RETRIES:
                retry_delay = BASE_RETRY_DELAY * (2 ** _retry_count)  # Exponential backoff
                logger.warning(f"Network error during upload (attempt {_retry_count + 1}/{MAX_NETWORK_RETRIES}): {e}")
                logger.info(f"Retrying in {retry_delay} seconds...")
                time.sleep(retry_delay)
                return self._upload_audio_with_relogin(audio_path, data, _retried_auth, _retry_count + 1)
            else:
                logger.error(f"✗ Network error: Failed after {MAX_NETWORK_RETRIES} retries: {e}")
                raise requests.RequestException(f"Network error during audio upload after {MAX_NETWORK_RETRIES} retries: {e}")

        except requests.exceptions.Timeout as e:
            # Retry on timeout
            if _retry_count < MAX_NETWORK_RETRIES:
                retry_delay = BASE_RETRY_DELAY * (2 ** _retry_count)  # Exponential backoff
                logger.warning(f"Timeout during upload (attempt {_retry_count + 1}/{MAX_NETWORK_RETRIES}): {e}")
                logger.info(f"Retrying in {retry_delay} seconds...")
                time.sleep(retry_delay)
                return self._upload_audio_with_relogin(audio_path, data, _retried_auth, _retry_count + 1)
            else:
                logger.error(f"✗ Timeout: Failed after {MAX_NETWORK_RETRIES} retries: {e}")
                raise requests.RequestException(f"Timeout during audio upload after {MAX_NETWORK_RETRIES} retries: {e}")

        except requests.exceptions.HTTPError as e:
            # Don't retry on 4xx errors (except 401 which is handled above)
            if e.response.status_code >= 400 and e.response.status_code < 500:
                logger.error(f"✗ Client error {e.response.status_code}: {e}")
                raise
            # Retry on 5xx server errors
            elif e.response.status_code >= 500:
                if _retry_count < MAX_NETWORK_RETRIES:
                    retry_delay = BASE_RETRY_DELAY * (2 ** _retry_count)
                    logger.warning(f"Server error {e.response.status_code} (attempt {_retry_count + 1}/{MAX_NETWORK_RETRIES}): {e}")
                    logger.info(f"Retrying in {retry_delay} seconds...")
                    time.sleep(retry_delay)
                    return self._upload_audio_with_relogin(audio_path, data, _retried_auth, _retry_count + 1)
                else:
                    logger.error(f"✗ Server error {e.response.status_code}: Failed after {MAX_NETWORK_RETRIES} retries: {e}")
                    raise
            else:
                raise

        finally:
            # Restore original headers
            self.session.headers = original_headers

    def _update_env_token(self, new_token: str):
        """
        Update token in .env file.

        Args:
            new_token: New access token to save
        """
        if not self.env_file:
            return

        env_path = Path(self.env_file)
        if not env_path.exists():
            logger.warning(f".env file not found: {self.env_file}")
            return

        try:
            # Read current content
            with open(env_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()

            # Update MOFA_FM_TOKEN line
            updated = False
            for i, line in enumerate(lines):
                if line.strip().startswith('MOFA_FM_TOKEN='):
                    lines[i] = f'MOFA_FM_TOKEN={new_token}\n'
                    updated = True
                    break

            # If not found, add it
            if not updated:
                lines.append(f'\nMOFA_FM_TOKEN={new_token}\n')

            # Write back
            with open(env_path, 'w', encoding='utf-8') as f:
                f.writelines(lines)

            logger.info(f"✓ Updated token in {self.env_file}")

        except Exception as e:
            logger.warning(f"Failed to update .env file: {e}")

    def _request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict] = None,
        files: Optional[Dict] = None,
        require_auth: bool = False,
        retry: int = 0,
        _retried_auth: bool = False
    ) -> Dict[str, Any]:
        """
        Make HTTP request with retry logic and auto-relogin.

        Args:
            method: HTTP method
            endpoint: API endpoint
            data: Request body data
            files: Files to upload
            require_auth: Whether authentication is required
            retry: Current retry attempt
            _retried_auth: Internal flag to prevent infinite auth retry loops

        Returns:
            Response JSON data

        Raises:
            requests.RequestException: On failure after retries
        """
        url = f"{self.base_url}/{endpoint.lstrip('/')}"

        # For public APIs, temporarily remove Authorization header
        auth_backup = None
        if not require_auth and 'Authorization' in self.session.headers:
            auth_backup = self.session.headers.pop('Authorization')

        try:
            if method.upper() == 'GET':
                response = self.session.get(
                    url,
                    params=data,
                    timeout=self.timeout
                )
            elif method.upper() == 'POST':
                # When uploading files, use data= instead of json= to avoid conflict
                if files:
                    response = self.session.post(
                        url,
                        data=data,
                        files=files,
                        timeout=self.timeout
                    )
                else:
                    response = self.session.post(
                        url,
                        json=data,
                        timeout=self.timeout
                    )
            elif method.upper() == 'PUT':
                response = self.session.put(
                    url,
                    json=data,
                    timeout=self.timeout
                )
            elif method.upper() == 'PATCH':
                response = self.session.patch(
                    url,
                    json=data,
                    timeout=self.timeout
                )
            elif method.upper() == 'DELETE':
                response = self.session.delete(
                    url,
                    timeout=self.timeout
                )
            else:
                raise ValueError(f"Unsupported method: {method}")

            # Check for authentication errors
            if response.status_code == 401 and require_auth:
                # Try auto-relogin if enabled and not already retried
                if self.auto_relogin and not _retried_auth and self.username and self.password:
                    logger.warning("Authentication failed (401). Attempting auto-relogin...")
                    try:
                        self.login()
                        logger.info("✓ Auto-relogin successful. Retrying request...")
                        # Update .env file with new token
                        if self.token:
                            self._update_env_token(self.token)
                        # Retry the request with new token
                        return self._request(method, endpoint, data, files, require_auth, retry, _retried_auth=True)
                    except Exception as relogin_error:
                        logger.error(f"Auto-relogin failed: {relogin_error}")
                        raise requests.RequestException(f"Authentication failed and auto-relogin also failed: {relogin_error}")
                else:
                    logger.error("Authentication failed (401). Please check your credentials or token.")
                    raise requests.RequestException("Authentication failed (401)")

            # Check for other errors
            response.raise_for_status()

            return response.json()

        except requests.RequestException as e:
            # Don't retry on 401 if we already handled it above
            if "401" in str(e) or "Authentication" in str(e):
                raise

            if retry < self.max_retries:
                wait_time = 2 ** retry  # Exponential backoff
                logger.warning(
                    f"Request failed (attempt {retry + 1}/{self.max_retries}): {e}. "
                    f"Retrying in {wait_time}s..."
                )
                time.sleep(wait_time)
                return self._request(method, endpoint, data, files, require_auth, retry + 1, _retried_auth)
            else:
                logger.error(f"Request failed after {self.max_retries} retries: {e}")
                raise
        finally:
            # Restore Authorization header if it was removed
            if auth_backup is not None:
                self.session.headers['Authorization'] = auth_backup

    # ==================== Authentication ====================

    def login(self, username: str = None, password: str = None) -> Dict[str, Any]:
        """
        Login and get access token.

        Args:
            username: Username (uses instance username if not provided)
            password: Password (uses instance password if not provided)

        Returns:
            Dict with 'access' and 'refresh' tokens (or full response)
        """
        username = username or self.username
        password = password or self.password

        if not username or not password:
            raise ValueError("Username and password are required")

        logger.info(f"Logging in as {username}...")

        data = {
            'username': username,
            'password': password
        }

        result = self._request('POST', '/auth/login/', data, require_auth=False)

        # Handle different response formats
        # Format 1: {'access': ..., 'refresh': ...}
        # Format 2: {'user': ..., 'tokens': {'access': ..., 'refresh': ...}}
        if 'tokens' in result:
            tokens = result['tokens']
            self.token = tokens.get('access', '')
        elif 'access' in result:
            self.token = result.get('access', '')
        elif 'token' in result:
            self.token = result.get('token', '')
        else:
            logger.warning(f"Unexpected login response format. Keys: {list(result.keys())}")
            # Try to find token in nested structure
            self.token = ''

        if self.token:
            self._set_auth_header()
            logger.info("Login successful")
        else:
            logger.warning("Login completed but no token found in response")

        return result

    def refresh_token(self, refresh_token: str) -> Dict[str, Any]:
        """
        Refresh access token.

        Args:
            refresh_token: Refresh token from login

        Returns:
            Dict with new 'access' and 'refresh' tokens
        """
        logger.info("Refreshing token...")

        data = {'refresh': refresh_token}
        result = self._request('POST', '/auth/token/refresh/', data, require_auth=False)

        # Update token
        self.token = result.get('access', '')
        self._set_auth_header()

        logger.info("Token refreshed")
        return result

    def me(self) -> Dict[str, Any]:
        """
        Get current user information.

        Returns:
            User information dict
        """
        return self._request('GET', '/auth/me/', require_auth=True)

    # ==================== Trending (热搜) ====================

    def get_trending_sources(self) -> List[str]:
        """
        Get list of available trending sources.

        Returns:
            List of source names
        """
        result = self._request('GET', '/podcasts/trending/sources/', require_auth=False)
        return result.get('sources', [])

    def get_trending(self, source: str) -> List[TrendingItem]:
        """
        Get trending topics from a specific source.

        Args:
            source: Source name (e.g., 'bilibili', 'zhihu', 'github')

        Returns:
            List of TrendingItem objects
        """
        logger.info(f"Fetching trending from {source}...")

        result = self._request(
            'GET',
            f'/podcasts/trending/{source}/',
            require_auth=False
        )

        # Handle different response formats
        # GitHub returns: {"data": [...], "total": 13}
        # Other sources might return: {"results": [...]} or {"trending": [...]}
        if isinstance(result, dict):
            result_list = result.get('data', result.get('results', result.get('trending', [])))
        else:
            result_list = result

        items = []
        for item in result_list:
            if isinstance(item, dict):
                trending_item = TrendingItem(
                    source=source,
                    title=item.get('title', ''),
                    hot_score=int(str(item.get('hot', 0)).replace(',', '')),  # Handle "18,774" format
                    url=item.get('url', ''),
                    desc=item.get('desc', item.get('description', '')),
                    cover=item.get('cover', '')
                )
                items.append(trending_item)

        logger.info(f"Fetched {len(items)} trending items from {source}")
        return items

    def get_all_trending(self, sources: List[str]) -> Dict[str, List[TrendingItem]]:
        """
        Get trending from multiple sources.

        Args:
            sources: List of source names

        Returns:
            Dict mapping source name to list of TrendingItem
        """
        results = {}
        for source in sources:
            try:
                items = self.get_trending(source)
                results[source] = items
            except Exception as e:
                logger.error(f"Failed to fetch trending from {source}: {e}")
                results[source] = []

        return results

    # ==================== Shows (节目) ====================

    def list_shows(self) -> List[Dict[str, Any]]:
        """
        List all public shows.

        Returns:
            List of show dictionaries
        """
        result = self._request('GET', '/podcasts/shows/', require_auth=False)
        return result.get('results', result)

    def get_show(self, slug: str) -> Dict[str, Any]:
        """
        Get show details by slug.

        Args:
            slug: Show slug

        Returns:
            Show details dict
        """
        return self._request('GET', f'/podcasts/shows/{slug}/', require_auth=False)

    def create_show(
        self,
        slug: str,
        title: str,
        description: str = "",
        content_type: str = "podcast",
        visibility: str = "public"
    ) -> Dict[str, Any]:
        """
        Create a new show.

        Args:
            slug: URL-friendly identifier
            title: Show title
            description: Show description
            content_type: Content type (podcast, etc.)
            visibility: Visibility (public, private)

        Returns:
            Created show dict
        """
        logger.info(f"Creating show: {title} ({slug})...")

        data = {
            'slug': slug,
            'title': title,
            'description': description,
            'content_type': content_type,
            'visibility': visibility
        }

        result = self._request('POST', '/podcasts/shows/create/', data, require_auth=True)
        logger.info(f"Show created: {result.get('id')}")
        return result

    def update_show(
        self,
        slug: str,
        title: str = None,
        description: str = None
    ) -> Dict[str, Any]:
        """
        Update show details.

        Args:
            slug: Show slug
            title: New title
            description: New description

        Returns:
            Updated show dict
        """
        logger.info(f"Updating show: {slug}...")

        data = {}
        if title:
            data['title'] = title
        if description:
            data['description'] = description

        result = self._request('PUT', f'/podcasts/shows/{slug}/update/', data, require_auth=True)
        logger.info("Show updated")
        return result

    def delete_show(self, slug: str) -> bool:
        """
        Delete a show.

        Args:
            slug: Show slug

        Returns:
            True if successful
        """
        logger.info(f"Deleting show: {slug}...")

        self._request('DELETE', f'/podcasts/shows/{slug}/delete/', require_auth=True)
        logger.info("Show deleted")
        return True

    # ==================== Episodes (单集) ====================

    def list_episodes(self, show_slug: str = None, limit: int = None) -> List[Dict[str, Any]]:
        """
        List episodes with pagination support.

        Args:
            show_slug: Filter by show slug (optional)
            limit: Maximum number of episodes to fetch (None = all)

        Returns:
            List of episode dicts
        """
        all_episodes = []
        page = 1
        has_more = True

        # Fetch all pages
        while has_more:
            result = self._request('GET', f'/podcasts/episodes/?page={page}', require_auth=False)
            episodes = result.get('results', [])
            all_episodes.extend(episodes)

            # Check if there's a next page
            next_url = result.get('next')
            has_more = next_url is not None
            page += 1

            # Apply limit if specified
            if limit and len(all_episodes) >= limit:
                all_episodes = all_episodes[:limit]
                break

        logger.info(f"Fetched {len(all_episodes)} episodes (total in system: {result.get('count', 0)})")

        # Filter by show if specified
        if show_slug:
            filtered = []
            for ep in all_episodes:
                ep_show = ep.get('show', {})
                if isinstance(ep_show, dict):
                    show_slug_match = ep_show.get('slug') == show_slug
                    show_id_match = str(ep_show.get('id', '')) == str(show_slug)
                    if show_slug_match or show_id_match:
                        filtered.append(ep)
            logger.info(f"Filtered to {len(filtered)} episodes for show '{show_slug}'")
            return filtered

        return all_episodes

    def get_episode(self, episode_id: int) -> Dict[str, Any]:
        """
        Get episode details.

        Args:
            episode_id: Episode ID

        Returns:
            Episode dict
        """
        return self._request('GET', f'/podcasts/episodes/{episode_id}/', require_auth=True)

    def create_episode(
        self,
        show_id: int = None,
        show_slug: str = None,
        title: str = "",
        description: str = "",
        script: str = "",
        audio_url: str = None,
        audio_file_path: str = None
    ) -> Dict[str, Any]:
        """
        Create a new episode with optional audio file upload.

        Args:
            show_id: Show ID (integer, required)
            show_slug: Show slug (will be converted to show_id)
            title: Episode title
            description: Episode description
            script: Episode script content
            audio_url: Audio file URL
            audio_file_path: Local audio file path to upload

        Returns:
            Created episode dict
        """
        # Convert show_slug to show_id if needed
        if show_id is None and show_slug:
            logger.info(f"Looking up show_id for slug: {show_slug}")
            try:
                show = self.get_show(show_slug)
                show_id = show.get('id')
                logger.info(f"Found show_id: {show_id}")
            except Exception as e:
                logger.error(f"Failed to get show_id for {show_slug}: {e}")
                raise ValueError(f"Show '{show_slug}' not found")

        if show_id is None:
            raise ValueError("Either show_id or show_slug must be provided")

        logger.info(f"Creating episode: {title}...")

        data = {
            'show_id': show_id,
            'title': title,
            'description': description,
            'script': script,
            'mode': 'podcast'  # Required for script display in web UI
        }

        if audio_url:
            data['audio_url'] = audio_url

        # If uploading audio file, use multipart/form-data
        if audio_file_path:
            result = self._upload_audio_with_relogin(audio_path=Path(audio_file_path), data=data)
        else:
            result = self._request('POST', '/podcasts/episodes/create/', data, require_auth=True)

        # API doesn't return episode_id directly, need to fetch from show's episodes list
        # Retry with backoff to handle API caching/replication lag
        import time
        max_retries = 3
        retry_delay = 2  # seconds

        episode_id = None
        for attempt in range(max_retries):
            if attempt > 0:
                logger.info(f"Retry {attempt + 1}/{max_retries} after {retry_delay}s delay...")
                time.sleep(retry_delay)
                retry_delay *= 2  # Exponential backoff

            logger.info("Fetching episode ID from show's episodes list...")
            episodes = self.list_episodes(show_slug=show_slug, limit=50)

            # Sort episodes by creation time (most recent first)
            def get_created_time(ep):
                created_str = ep.get('created_at', '')
                if created_str:
                    from datetime import datetime
                    try:
                        return datetime.fromisoformat(created_str.replace('Z', '+00:00'))
                    except:
                        pass
                return datetime.min

            episodes_sorted = sorted(episodes, key=get_created_time, reverse=True)

            # Find the episode with matching title (most recent first)
            for ep in episodes_sorted:
                if ep.get('title') == title:
                    episode_id = ep.get('id')
                    logger.info(f"✅ Found matching episode ID: {episode_id} (created: {ep.get('created_at')})")
                    result['id'] = episode_id
                    result['episode_id'] = episode_id
                    break

            if episode_id:
                break
            else:
                logger.warning(f"Attempt {attempt + 1}: Could not find episode ID for title: {title}")
                logger.warning(f"Recent episodes titles: {[ep.get('title') for ep in episodes_sorted[:5]]}")

        if not episode_id:
            logger.error(f"Failed to find episode ID after {max_retries} attempts")
            raise ValueError(f"Could not find newly created episode '{title}'. The API may have failed to create it.")

        return result

    def update_episode_script(self, episode_id: int, script: str) -> Dict[str, Any]:
        """
        Update episode script.

        Args:
            episode_id: Episode ID
            script: Script content (supports Markdown with 【角色】 tags)

        Returns:
            Updated episode dict
        """
        logger.info(f"Updating script for episode {episode_id}...")

        data = {'script': script}
        result = self._request(
            'PATCH',
            f'/podcasts/episodes/{episode_id}/update-script/',
            data,
            require_auth=True
        )

        logger.info("Script updated")
        return result

    def update_episode_audio(
        self,
        episode_id: int,
        audio_path: Path,
        _retried_auth: bool = False
    ) -> Dict[str, Any]:
        """
        Update episode audio file.

        Args:
            episode_id: Episode ID
            audio_path: Path to audio file
            _retried_auth: Internal flag to prevent infinite retry loops

        Returns:
            Updated episode dict
        """
        logger.info(f"Updating audio for episode {episode_id}...")

        if not audio_path.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")

        url = f"{self.base_url}/podcasts/episodes/{episode_id}/update/"

        # Remove Content-Type header for file upload
        original_headers = self.session.headers.copy()
        if 'Content-Type' in self.session.headers:
            del self.session.headers['Content-Type']

        try:
            with open(audio_path, 'rb') as f:
                files = {'audio_file': (audio_path.name, f, 'audio/mpeg')}
                response = self.session.patch(
                    url,
                    headers={'Authorization': f'Bearer {self.token}'},
                    files=files,
                    timeout=180
                )

                # Check for authentication errors
                if response.status_code == 401:
                    if self.auto_relogin and not _retried_auth and self.username and self.password:
                        logger.warning("Authentication failed (401) during audio update. Attempting auto-relogin...")
                        try:
                            self.login()
                            logger.info("✓ Auto-relogin successful. Retrying audio update...")
                            # Update .env file with new token
                            if self.token:
                                self._update_env_token(self.token)
                            # Close the file and retry
                            f.close()
                            return self.update_episode_audio(episode_id, audio_path, _retried_auth=True)
                        except Exception as relogin_error:
                            logger.error(f"Auto-relogin failed: {relogin_error}")
                            raise requests.RequestException(f"Authentication failed during audio update and auto-relogin also failed: {relogin_error}")
                    else:
                        logger.error("Authentication failed (401) during audio update. Please check your credentials or token.")
                        raise requests.RequestException("Authentication failed (401)")

                response.raise_for_status()
                result = response.json()
                logger.info("Audio updated")
                return result

        finally:
            # Restore original headers
            self.session.headers = original_headers

    # ==================== Script Sessions (脚本会话) ====================

    def create_script_session(
        self,
        title: str,
        description: str = "",
        content_type: str = "podcast"
    ) -> Dict[str, Any]:
        """
        Create a script generation session.

        Args:
            title: Session title
            description: Session description
            content_type: Content type

        Returns:
            Session dict with 'id'
        """
        logger.info(f"Creating script session: {title}...")

        data = {
            'title': title,
            'description': description,
            'content_type': content_type
        }

        result = self._request('POST', '/podcasts/script-sessions/', data, require_auth=True)
        logger.info(f"Script session created: {result.get('id')}")
        return result

    def chat_script_session(
        self,
        session_id: int,
        message: str
    ) -> Dict[str, Any]:
        """
        Chat with AI to generate/refine script.

        Args:
            session_id: Session ID
            message: User message/prompt

        Returns:
            Response dict with updated 'current_script'
        """
        logger.debug(f"Chatting with script session {session_id}...")

        data = {'message': message}
        result = self._request(
            'POST',
            f'/podcasts/script-sessions/{session_id}/chat/',
            data,
            require_auth=True
        )

        return result

    def upload_script_reference(
        self,
        session_id: int,
        file_path: str,
        description: str = ""
    ) -> Dict[str, Any]:
        """
        Upload reference file to script session.

        Args:
            session_id: Session ID
            file_path: Path to file
            description: File description

        Returns:
            Upload result dict
        """
        logger.info(f"Uploading reference file to session {session_id}...")

        file_path = Path(file_path)
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        # Determine content type based on file extension
        content_type = 'application/octet-stream'
        if file_path.suffix == '.mp3':
            content_type = 'audio/mpeg'
        elif file_path.suffix in ['.md', '.txt']:
            content_type = 'text/markdown'
        elif file_path.suffix == '.pdf':
            content_type = 'application/pdf'

        # Remove Content-Type header to allow requests to set multipart/form-data
        original_headers = self.session.headers.copy()
        if 'Content-Type' in self.session.headers:
            del self.session.headers['Content-Type']

        try:
            with open(file_path, 'rb') as f:
                files = {'file': (file_path.name, f, content_type)}
                data = {'description': description}

                result = self._request(
                    'POST',
                    f'/podcasts/script-sessions/{session_id}/upload/',
                    data,
                    files=files,
                    require_auth=True
                )

            logger.info("Reference file uploaded")
            return result
        finally:
            # Restore original headers
            self.session.headers = original_headers

    def finalize_script_session(
        self,
        session_id: int,
        show_slug: str,
        title: str
    ) -> Dict[str, Any]:
        """
        Convert script session to published episode.

        Args:
            session_id: Session ID
            show_slug: Target show slug
            title: Episode title

        Returns:
            Created episode dict (contains episode_id, not id)
        """
        logger.info(f"Finalizing script session {session_id}...")

        data = {
            'show_slug': show_slug,
            'title': title
        }

        result = self._request(
            'POST',
            f'/podcasts/script-sessions/{session_id}/finalize/',
            data,
            require_auth=True
        )

        # API returns episode_id, not id
        episode_id = result.get('episode_id')
        if episode_id:
            logger.info(f"Script session finalized to episode {episode_id}")
        else:
            logger.warning(f"Finalize completed but no episode_id returned")

        return result

    def list_script_sessions(self) -> List[Dict[str, Any]]:
        """
        List all script sessions.

        Returns:
            List of session dicts
        """
        result = self._request('GET', '/podcasts/script-sessions/', require_auth=True)
        return result.get('results', result)

    # ==================== Search (搜索) ====================

    def search(self, query: str, quick: bool = False) -> List[Dict[str, Any]]:
        """
        Search for content.

        Args:
            query: Search query
            quick: Use quick search endpoint

        Returns:
            List of search results
        """
        logger.info(f"Searching for: {query}...")

        endpoint = '/search/quick/' if quick else '/search/'
        result = self._request('GET', endpoint, {'q': query}, require_auth=False)

        return result.get('results', result)

    def get_search_suggestions(self, prefix: str) -> List[str]:
        """
        Get search suggestions.

        Args:
            prefix: Query prefix

        Returns:
            List of suggestion strings
        """
        result = self._request('GET', '/search/suggestions/', {'q': prefix}, require_auth=False)
        return result.get('suggestions', [])

    def get_popular_searches(self) -> List[str]:
        """
        Get popular search terms.

        Returns:
            List of popular search terms
        """
        result = self._request('GET', '/search/popular/', require_auth=False)
        return result.get('popular', [])


def main():
    """CLI for testing MoFA FM client."""
    import sys

    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    if len(sys.argv) < 2:
        print("Usage: python mofa_client.py <command> [args]")
        print("\nCommands:")
        print("  health                    - Check API health")
        print("  trending <source>         - Get trending from source")
        print("  search <query>            - Search content")
        print("  login <username> <pass>   - Login and get token")
        print("\nExamples:")
        print("  python mofa_client.py health")
        print("  python mofa_client.py trending bilibili")
        print("  python mofa_client.py search AI")
        sys.exit(1)

    command = sys.argv[1]
    client = MoFAFMClient()

    try:
        if command == "health":
            # Simple health check
            result = client._request('GET', '/health/', require_auth=False)
            print("✓ API is healthy")
            print(f"Response: {result}")

        elif command == "trending":
            if len(sys.argv) < 3:
                print("Usage: python mofa_client.py trending <source>")
                sys.exit(1)

            source = sys.argv[2]
            items = client.get_trending(source)

            print(f"\n📈 Trending from {source} (top 10):")
            print("-" * 60)
            for i, item in enumerate(items[:10], 1):
                print(f"{i}. {item.title}")
                print(f"   🔥 Hot: {item.hot_score}")
                print(f"   🔗 {item.url}")
                if item.desc:
                    print(f"   📝 {item.desc[:100]}...")
                print()

        elif command == "search":
            if len(sys.argv) < 3:
                print("Usage: python mofa_client.py search <query>")
                sys.exit(1)

            query = sys.argv[2]
            results = client.search(query)

            print(f"\n🔍 Search results for '{query}':")
            print("-" * 60)
            for i, result in enumerate(results[:10], 1):
                print(f"{i}. {result.get('title', 'N/A')}")
                print(f"   {result.get('url', 'N/A')}")

        elif command == "login":
            if len(sys.argv) < 4:
                print("Usage: python mofa_client.py login <username> <password>")
                sys.exit(1)

            username = sys.argv[2]
            password = sys.argv[3]
            client.username = username
            client.password = password

            result = client.login()
            print(f"\n✓ Login successful!")

            # Display tokens safely (handle different response formats)
            access_token = result.get('access', result.get('token', None))
            refresh_token = result.get('refresh', None)

            # Check for nested format
            if 'tokens' in result:
                tokens = result['tokens']
                access_token = tokens.get('access', access_token)
                refresh_token = tokens.get('refresh', refresh_token)

            if access_token:
                print(f"Access Token: {access_token[:30]}...{access_token[-10:]}")
                print(f"Full token for .env:")
                print(f"  MOFA_FM_TOKEN={access_token}")
            else:
                print(f"Access Token: (not found in response)")

            if refresh_token:
                print(f"Refresh Token: {refresh_token[:30]}...")

            # Test getting user info
            try:
                me = client.me()
                print(f"\nUser info:")
                print(f"  Username: {me.get('username', 'N/A')}")
                print(f"  Email: {me.get('email', 'N/A')}")
                print(f"\n✓ You can now use the token with:")
                print(f"  1. Add to .env: MOFA_FM_TOKEN={access_token}")
                print(f"  2. Or use: python run_pipeline.py --mofa-publish")
            except Exception as e:
                print(f"\n⚠ Could not fetch user info: {e}")
                print(f"Token saved to client. You can now use --mofa-publish")

        else:
            print(f"Unknown command: {command}")
            sys.exit(1)

    except Exception as e:
        logger.error(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
