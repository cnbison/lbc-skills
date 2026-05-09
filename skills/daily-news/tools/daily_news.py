#!/usr/bin/env python3
"""
Daily News Skill CLI Entry

Wrapper to run the Claw Daily News pipeline and individual stages.
This tool auto-detects the virtual environment and falls back to system Python.
"""

import sys
import os
import subprocess
import argparse
from pathlib import Path


def get_python() -> str:
    """Return the path to the preferred Python interpreter."""
    skill_dir = Path(__file__).parent.parent.resolve()
    venv_python = skill_dir / "venv" / "bin" / "python"
    if venv_python.exists():
        return str(venv_python)
    return sys.executable


def run_setup(skill_dir: Path) -> bool:
    """Run setup.sh to create venv and install dependencies."""
    setup_script = skill_dir / "setup.sh"
    if not setup_script.exists():
        print("[daily-news] setup.sh not found. Please install dependencies manually.")
        return False
    print("[daily-news] Running setup.sh ...")
    result = subprocess.run(["bash", str(setup_script)], cwd=str(skill_dir))
    return result.returncode == 0


def _build_env(skill_dir: Path) -> dict:
    """Build environment with skill directory in PYTHONPATH."""
    env = os.environ.copy()
    pp = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = str(skill_dir) + (os.pathsep + pp if pp else "")
    return env


def run_pipeline(skill_dir: Path, date: str = None, skip_tts: bool = False, config: str = None) -> int:
    """Run the full pipeline."""
    python = get_python()
    cmd = [python, str(skill_dir / "run_pipeline.py")]
    if date:
        cmd += ["--date", date]
    if skip_tts:
        cmd.append("--skip-tts")
    if config:
        cmd += ["--config", config]
    print(f"[daily-news] Running: {' '.join(cmd)}")
    return subprocess.run(cmd, env=_build_env(skill_dir)).returncode


def run_stage(skill_dir: Path, stage: str, extra_args: list = None) -> int:
    """Run a specific pipeline stage."""
    python = get_python()
    module_map = {
        "collect": "scripts.collect_news",
        "filter": "scripts.filter_news",
        "tts": "scripts.tts_generate",
    }
    module = module_map.get(stage)
    if not module:
        print(f"[daily-news] Unknown stage: {stage}. Valid: {', '.join(module_map.keys())}")
        return 1
    cmd = [python, "-m", module]
    if extra_args:
        cmd.extend(extra_args)
    print(f"[daily-news] Running: {' '.join(cmd)}")
    return subprocess.run(cmd, env=_build_env(skill_dir)).returncode


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="daily-news",
        description="Claw Daily News Skill — run pipeline or individual stages",
    )
    parser.add_argument(
        "action",
        choices=["run", "setup", "collect", "filter", "tts"],
        help="Action to perform (summarize/article/podcast are done by the Agent)",
    )
    parser.add_argument("--date", help="Target date (YYYY-MM-DD)")
    parser.add_argument("--skip-tts", action="store_true", help="Skip TTS generation")
    parser.add_argument("--config", help="Path to custom config.yaml")
    parser.add_argument("--force-setup", action="store_true", help="Force run setup.sh before action")

    args = parser.parse_args()
    skill_dir = Path(__file__).parent.parent.resolve()

    if args.force_setup or not (skill_dir / "venv").exists():
        if not run_setup(skill_dir):
            return 1

    if args.action == "run":
        return run_pipeline(skill_dir, date=args.date, skip_tts=args.skip_tts, config=args.config)
    elif args.action == "setup":
        return 0 if run_setup(skill_dir) else 1
    else:
        extra = []
        if args.date:
            extra += ["--date", args.date]
        return run_stage(skill_dir, args.action, extra)


if __name__ == "__main__":
    sys.exit(main())
