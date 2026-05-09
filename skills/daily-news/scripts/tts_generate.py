"""TTS audio generator module for podcast audio."""

import logging
import asyncio
import re
import json
from pathlib import Path
from datetime import datetime
from typing import Optional, List
import edge_tts
import requests

from .config import get_config

logger = logging.getLogger(__name__)


class TTSGenerator:
    """Generate podcast audio using TTS."""

    def __init__(self, config=None):
        """
        Initialize TTS generator.

        Args:
            config: Configuration object (uses default if not provided)
        """
        self.config = config or get_config()
        tts_config = self.config.get_tts_config()
        self.provider = tts_config.get('provider', 'edge')

        if self.provider == 'edge':
            self.voice = tts_config.get('edge_voice', 'zh-CN-YunxiNeural')
            # 确保有男女声音配置
            self.voices = tts_config.get('edge_voices', {
                'male': 'zh-CN-YunxiNeural',      # 男声
                'female': 'zh-CN-XiaoxiaoNeural'   # 女声
            })
        elif self.provider == 'doubao':
            doubao_config = tts_config.get('doubao', {})
            self.token = doubao_config.get('token', '')
            self.appid = doubao_config.get('appid', '')
            self.cluster = doubao_config.get('cluster', 'volcano_tts')
            self.api_url = doubao_config.get('api_url', 'https://openspeech.bytedance.com/api/v1/tts')
            # 音频参数
            self.speed_ratio = doubao_config.get('speed_ratio', 1.0)
            self.volume_ratio = doubao_config.get('volume_ratio', 1.0)
            self.pitch_ratio = doubao_config.get('pitch_ratio', 1.0)
            # 默认音色
            self.voice = 'zh_female_xiaohe_uranus_bigtts'  # 小何 2.0
            # 确保有男女声音配置
            self.voices = tts_config.get('doubao_voices', {
                'male': 'zh_male_yunzhou_uranus_bigtts',      # 云舟 2.0 - 男主持
                'female': 'zh_female_xiaohe_uranus_bigtts'     # 小何 2.0 - 女主持
            })
        elif self.provider == 'openai':
            openai_config = tts_config.get('openai', {})
            self.api_key = openai_config.get('api_key', '')
            self.model = openai_config.get('model', 'tts-1')
            self.voice = openai_config.get('voice', 'alloy')
            self.base_url = 'https://api.openai.com/v1'

    def _clean_text(self, text: str) -> str:
        """
        Clean text by removing bracketed content.

        Args:
            text: Raw text

        Returns:
            Cleaned text with [...] and (...) removed
        """
        # Remove [...]
        text = re.sub(r'\[.*?\]', '', text)
        # Remove (...)
        text = re.sub(r'（.*?）', '', text)
        # Remove (...)
        text = re.sub(r'\(.*?\)', '', text)
        # Clean up extra whitespace
        text = re.sub(r'\s+', ' ', text).strip()
        return text

    def _parse_script_for_dialogue(self, script: str) -> List[dict]:
        """
        Parse podcast script to extract dialogue lines.

        Args:
            script: Podcast script text

        Returns:
            List of dialogue segments with speaker info
        """
        segments = []
        lines = script.split('\n')

        current_speaker = None
        current_text = []

        for line in lines:
            original_line = line
            line = line.strip()

            # Check for speaker markers in brackets like [主持人 Alex ]
            if 'Alex' in line and ('[' in line or '【' in line):
                if current_speaker and current_text:
                    segments.append({
                        'speaker': current_speaker,
                        'text': ' '.join(current_text).strip()
                    })
                current_speaker = 'Alex'
                current_text = []
                logger.debug(f"Found Alex speaker marker: {original_line}")
            elif 'Sarah' in line and ('[' in line or '【' in line):
                if current_speaker and current_text:
                    segments.append({
                        'speaker': current_speaker,
                        'text': ' '.join(current_text).strip()
                    })
                current_speaker = 'Sarah'
                current_text = []
                logger.debug(f"Found Sarah speaker marker: {original_line}")
            # Also support Alex: and Sarah: format
            elif line.startswith('Alex:') or line.startswith('Sarah:'):
                if current_speaker and current_text:
                    segments.append({
                        'speaker': current_speaker,
                        'text': ' '.join(current_text).strip()
                    })
                speaker = line.split(':')[0]
                current_speaker = speaker
                text = line.split(':', 1)[1].strip()
                current_text = [text] if text else []
                logger.debug(f"Found speaker via colon format: {speaker}")
            # Skip bracket-only lines (sections, music cues)
            elif line and not line.startswith('[') and not line.startswith('（') and not line.startswith('【'):
                # Continue current speaker's text
                if current_speaker:
                    current_text.append(line)

        # Add last segment
        if current_speaker and current_text:
            text = ' '.join(current_text).strip()
            if text:
                segments.append({
                    'speaker': current_speaker,
                    'text': text
                })

        logger.info(f"Parsed {len(segments)} dialogue segments")
        for i, seg in enumerate(segments):
            logger.info(f"  Segment {i}: speaker={seg['speaker']}, text_length={len(seg['text'])}")

        # If no dialogue format found, treat as single segment
        if not segments:
            logger.warning("No dialogue format found, treating as single segment")
            segments = [{'speaker': 'narrator', 'text': script}]

        return segments

    def _get_voice_for_speaker(self, speaker: str) -> str:
        """Get TTS voice for speaker."""
        speaker_lower = speaker.lower()

        if speaker_lower == 'alex':
            voice = self.voices.get('male', self.voice)
            logger.info(f"Speaker {speaker} -> Male voice: {voice}")
            return voice
        elif speaker_lower == 'sarah':
            voice = self.voices.get('female', self.voice)
            logger.info(f"Speaker {speaker} -> Female voice: {voice}")
            return voice
        else:
            logger.info(f"Speaker {speaker} -> Default voice: {self.voice}")
            return self.voice

    async def _generate_edge_tts(self, text: str, voice: str, output_path: Path) -> bool:
        """
        Generate audio using Edge TTS.

        Args:
            text: Text to convert
            voice: Voice to use
            output_path: Output file path

        Returns:
            True if successful
        """
        try:
            communicate = edge_tts.Communicate(text, voice)

            await communicate.save(str(output_path))
            logger.info(f"Generated TTS audio: {output_path}")
            return True

        except Exception as e:
            logger.error(f"Edge TTS error: {e}")
            return False

    def _generate_openai_tts(self, text: str, output_path: Path) -> bool:
        """
        Generate audio using OpenAI TTS.

        Args:
            text: Text to convert
            output_path: Output file path

        Returns:
            True if successful
        """
        url = f"{self.base_url}/audio/speech"
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }

        data = {
            "model": self.model,
            "voice": self.voice,
            "input": text
        }

        try:
            response = requests.post(url, headers=headers, json=data, timeout=30)
            response.raise_for_status()

            with open(output_path, 'wb') as f:
                f.write(response.content)

            logger.info(f"Generated OpenAI TTS audio: {output_path}")
            return True

        except Exception as e:
            logger.error(f"OpenAI TTS error: {e}")
            return False

    def _generate_doubao_tts(self, text: str, voice_type: str, output_path: Path) -> bool:
        """
        Generate audio using Doubao TTS (豆包语音合成 Model 2.0).

        Args:
            text: Text to convert
            voice_type: Voice type to use (e.g., 'zh_female_xiaohe_uranus_bigtts')
            output_path: Output file path

        Returns:
            True if successful
        """
        import uuid
        import base64

        headers = {
            "Authorization": f"Bearer;{self.token}",
            "Content-Type": "application/json"
        }

        # 生成唯一的请求ID
        reqid = str(uuid.uuid4())

        payload = {
            "app": {
                "appid": self.appid,
                "token": self.token,
                "cluster": self.cluster
            },
            "user": {
                "uid": "user_001"
            },
            "audio": {
                "voice_type": voice_type,
                "encoding": "mp3",
                "rate": 24000,
                "speed_ratio": self.speed_ratio,
                "volume_ratio": self.volume_ratio,
                "pitch_ratio": self.pitch_ratio
            },
            "request": {
                "reqid": reqid,
                "text": text,
                "text_type": "plain",
                "operation": "query"
            }
        }

        try:
            response = requests.post(self.api_url, headers=headers, json=payload, timeout=60)

            if response.status_code == 200:
                result = response.json()

                # 检查响应码
                if result.get("code") == 3000:  # 3000表示成功
                    # 解码base64音频数据
                    audio_data = base64.b64decode(result.get("data", ""))
                    with open(output_path, 'wb') as f:
                        f.write(audio_data)
                    logger.info(f"Generated Doubao TTS audio: {output_path}")
                    return True
                else:
                    logger.error(f"Doubao TTS error (code {result.get('code')}): {result.get('message')}")
                    return False
            else:
                logger.error(f"Doubao HTTP error {response.status_code}: {response.text}")
                return False

        except Exception as e:
            logger.error(f"Doubao TTS error: {e}")
            return False

    async def _generate_segment(self, text: str, voice: str, temp_path: Path) -> bool:
        """Generate a single audio segment using the configured provider."""
        if self.provider == 'edge':
            return await self._generate_edge_tts(text, voice, temp_path)
        elif self.provider == 'doubao':
            return self._generate_doubao_tts(text, voice, temp_path)
        else:
            return False

    async def _generate_dialogue_audio(self, segments: List[dict], output_path: Path) -> bool:
        """
        Generate audio for dialogue with different voices.

        Args:
            segments: List of dialogue segments
            output_path: Output file path

        Returns:
            True if successful
        """
        import subprocess
        import tempfile
        import time

        temp_files = []

        try:
            for i, segment in enumerate(segments):
                voice = self._get_voice_for_speaker(segment['speaker'])
                text = self._clean_text(segment['text'])

                if not text:
                    continue

                if len(text) > 3000:
                    text = text[:3000]

                temp_file = tempfile.NamedTemporaryFile(
                    suffix=f'_segment_{i}.mp3',
                    delete=False,
                    dir=output_path.parent
                )
                temp_files.append(temp_file.name)

                if self.provider == 'doubao' and i > 0:
                    time.sleep(0.5)

                success = await self._generate_segment(text, voice, Path(temp_file.name))

                if not success:
                    logger.error(f"Failed to generate segment {i}")
                    for temp_file in temp_files:
                        Path(temp_file).unlink(missing_ok=True)
                    return False

            logger.info(f"Generated {len(temp_files)} audio segments")

            if len(temp_files) > 1:
                list_file = output_path.parent / 'file_list.txt'
                with open(list_file, 'w') as f:
                    for temp_file in temp_files:
                        f.write(f"file '{temp_file}'\n")

                cmd = [
                    'ffmpeg',
                    '-f', 'concat',
                    '-safe', '0',
                    '-i', str(list_file),
                    '-c', 'copy',
                    '-y',
                    str(output_path)
                ]
                result = subprocess.run(cmd, capture_output=True, timeout=300)

                for temp_file in temp_files:
                    Path(temp_file).unlink(missing_ok=True)
                list_file.unlink(missing_ok=True)

                if result.returncode == 0:
                    logger.info(f"✓ Merged {len(temp_files)} audio segments: {output_path}")
                    return True
                else:
                    logger.error(f"ffmpeg failed: {result.stderr.decode()}")
                    return False
            else:
                Path(temp_files[0]).rename(output_path)
                logger.info(f"✓ Single segment audio: {output_path}")
                return True

        except Exception as e:
            logger.error(f"Error generating dialogue audio: {e}")
            for temp_file in temp_files:
                Path(temp_file).unlink(missing_ok=True)
            return False

    def generate(self, script: str, date: str = None, output_path: str = None) -> Optional[Path]:
        """
        Generate podcast audio from script.

        Args:
            script: Podcast script text
            date: Audio date (YYYY-MM-DD format)
            output_path: Output file path

        Returns:
            Path to generated audio file, or None if failed
        """
        if date is None:
            date = datetime.now().strftime("%Y-%m-%d")

        logger.info(f"Generating podcast audio for {date}")

        if output_path is None:
            output_dir = self.config.get_output_dir()
            output_dir.mkdir(parents=True, exist_ok=True)
            filename = self.config.get_podcast_audio_filename(date)
            output_path = output_dir / filename
        else:
            output_path = Path(output_path)

        segments = self._parse_script_for_dialogue(script)
        logger.info(f"Script parsed into {len(segments)} segments")

        if self.provider in ('edge', 'doubao'):
            success = asyncio.run(self._generate_dialogue_audio(segments, output_path))
        elif self.provider == 'openai':
            combined_text = ' '.join([s['text'] for s in segments])
            success = self._generate_openai_tts(combined_text, output_path)
        else:
            logger.error(f"Unknown TTS provider: {self.provider}")
            return None

        if success:
            logger.info(f"Podcast audio generated: {output_path}")
            return output_path
        else:
            logger.error("Failed to generate podcast audio")
            return None


def main():
    """Main function for testing."""
    import argparse
    parser = argparse.ArgumentParser(description='Generate TTS audio for podcast script')
    parser.add_argument('--date', help='Target date (YYYY-MM-DD, default: today)')
    args = parser.parse_args()

    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    # Load podcast script
    config = get_config()
    output_dir = config.get_output_dir()
    date = args.date or datetime.now().strftime('%Y-%m-%d')
    script_filename = config.get_podcast_script_filename(date)
    script_path = output_dir / script_filename

    if not script_path.exists():
        print(f"No script file found at {script_path}")
        return

    with open(script_path, 'r', encoding='utf-8') as f:
        script = f.read()

    # Generate audio
    generator = TTSGenerator()
    audio_path = generator.generate(script, date)

    if audio_path:
        print(f"\nAudio generated successfully: {audio_path}")
        print(f"File size: {audio_path.stat().st_size / 1024 / 1024:.2f} MB")
    else:
        print("\nFailed to generate audio")


if __name__ == "__main__":
    main()
