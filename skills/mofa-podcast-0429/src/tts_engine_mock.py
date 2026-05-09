#!/usr/bin/env python3
"""Mock VoxCPM batch engine for testing.
Reads batch JSON from stdin, generates silent WAV files, outputs report JSON.
"""

import json
import os
import struct
import sys


def make_silent_wav(path, duration_ms=500, sample_rate=24000):
    """Write a silent mono 16-bit WAV."""
    num_samples = int(sample_rate * duration_ms / 1000)
    pcm = b"\x00\x00" * num_samples
    data_len = len(pcm)
    file_len = 36 + data_len

    header = (
        b"RIFF"
        + struct.pack("<I", file_len)
        + b"WAVE"
        + b"fmt "
        + struct.pack("<I", 16)
        + struct.pack("<H", 1)       # PCM
        + struct.pack("<H", 1)       # mono
        + struct.pack("<I", sample_rate)
        + struct.pack("<I", sample_rate * 2)
        + struct.pack("<H", 2)       # block align
        + struct.pack("<H", 16)      # bits per sample
        + b"data"
        + struct.pack("<I", data_len)
    )

    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "wb") as f:
        f.write(header + pcm)


def main():
    task = json.load(sys.stdin)
    segments = task.get("segments", [])

    for seg in segments:
        make_silent_wav(seg["output_path"], duration_ms=500)

    print(
        json.dumps(
            {
                "success": True,
                "generated": len(segments),
                "failed": [],
                "model_sample_rate": 24000,
            }
        )
    )


if __name__ == "__main__":
    main()
