#!/usr/bin/env python3
"""VoxCPM batch TTS engine for mofa-podcast skill.

Protocol: receive batch task JSON on stdin, write report JSON to stdout.
Loads the VoxCPM model once and generates all segments sequentially.
Outputs 24kHz mono 16-bit WAV files regardless of model native sample rate.
"""

import json
import os
import sys


def main() -> None:
    task = json.load(sys.stdin)

    # Lazy import so the script can be syntax-checked without voxcpm installed
    try:
        import soundfile as sf
        from voxcpm import VoxCPM
    except ImportError as e:
        print(json.dumps({"success": False, "error": f"Missing Python dependency: {e}"}))
        sys.exit(1)

    model_id = task.get("model_id", "openbmb/VoxCPM2")
    cfg_value = task.get("cfg_value", 2.0)
    inference_timesteps = task.get("inference_timesteps", 10)
    normalize = task.get("normalize", False)

    try:
        model = VoxCPM.from_pretrained(
            model_id,
            load_denoiser=False,
        )
    except Exception as e:
        print(json.dumps({"success": False, "error": f"Failed to load VoxCPM model: {e}"}))
        sys.exit(1)

    native_sr = getattr(model, "tts_model", None)
    native_sr = getattr(native_sr, "sample_rate", None) if native_sr else None
    if native_sr is None:
        # Fallback: inspect a dummy generation or assume 24000
        native_sr = 24000

    segments = task.get("segments", [])
    failed = []

    for seg in segments:
        seg_id = seg.get("seg_id", "?")
        try:
            text = seg["text"]
            output_path = seg["output_path"]
            mode = seg.get("mode", "design")
            control = seg.get("control")
            reference_audio = seg.get("reference_audio")

            # Build final text with control prefix (same as voxcpm CLI)
            final_text = text
            if control:
                control_str = control.strip()
                if control_str:
                    final_text = f"({control_str}){text}"

            kwargs = {
                "text": final_text,
                "cfg_value": cfg_value,
                "inference_timesteps": inference_timesteps,
            }
            if normalize:
                kwargs["normalize"] = True

            if mode == "clone" and reference_audio:
                if not os.path.isfile(reference_audio):
                    raise FileNotFoundError(
                        f"Reference audio not found: {reference_audio}"
                    )
                kwargs["reference_wav_path"] = reference_audio
            elif mode == "clone" and not reference_audio:
                raise ValueError(
                    "clone mode requires reference_audio but none was provided"
                )

            wav = model.generate(**kwargs)

            # Ensure numpy array shape is (samples,)
            import numpy as np

            if wav is None or (hasattr(wav, "size") and wav.size == 0):
                raise RuntimeError("Model returned empty audio")

            if isinstance(wav, list):
                wav = np.array(wav, dtype=np.float32)
            if wav.ndim > 1:
                # Stereo or more channels -> mix to mono
                wav = np.mean(wav, axis=-1)
            wav = np.squeeze(wav)

            # Resample to 24kHz if needed
            target_sr = 24000
            if native_sr != target_sr:
                try:
                    import librosa

                    wav = librosa.resample(
                        wav, orig_sr=native_sr, target_sr=target_sr
                    )
                except ImportError:
                    # Fallback: simple linear interpolation (not great but avoids hard fail)
                    old_len = len(wav)
                    new_len = int(old_len * target_sr / native_sr)
                    indices = np.linspace(0, old_len - 1, new_len)
                    wav = np.interp(indices, np.arange(old_len), wav)

            os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
            sf.write(output_path, wav, target_sr, subtype="PCM_16")
        except Exception as e:
            failed.append({"seg_id": seg_id, "reason": str(e)})

    print(
        json.dumps(
            {
                "success": len(failed) == 0,
                "generated": len(segments) - len(failed),
                "failed": failed,
                "model_sample_rate": native_sr,
            },
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
