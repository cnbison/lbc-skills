//! MoFA Podcast: Multi-speaker podcast generation with TTS and audio assembly.
//!
//! Protocol: `./mofa-podcast <tool_name>` with JSON on stdin, JSON on stdout.
//! Uses local VoxCPM TTS engine (Python API) via batch generation.

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

// ── Preset speakers (reference-audio based via voxcpm clone) ───────

const PRESET_VOICE_NAMES: &[&str] = &["douwt", "yangmi"];

// ── Emotion → TTS prompt mapping ───────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TtsLanguage {
    Chinese,
    English,
}

impl TtsLanguage {
    #[allow(dead_code)]
    fn api_value(self) -> &'static str {
        match self {
            Self::Chinese => "chinese",
            Self::English => "english",
        }
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0x2CEB0..=0x2EBEF
    )
}

fn infer_tts_language(text: &str) -> Option<TtsLanguage> {
    let mut cjk_count = 0usize;
    let mut latin_count = 0usize;

    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            latin_count += 1;
        } else if is_cjk(ch) {
            cjk_count += 1;
        }
    }

    if cjk_count == 0 && latin_count == 0 {
        None
    } else if cjk_count >= latin_count {
        Some(TtsLanguage::Chinese)
    } else {
        Some(TtsLanguage::English)
    }
}

fn emotion_to_control(emotion: &str) -> Option<&'static str> {
    match emotion.trim().to_lowercase().as_str() {
        "calm" => None,
        "excited" => Some("excited, energetic, enthusiastic"),
        "serious" => Some("serious, composed, formal, measured delivery"),
        "warm" => Some("warm, friendly, gentle, inviting"),
        "angry" => Some("angry, forceful, intense"),
        "sad" => Some("sad, low, reflective, somber"),
        "cheerful" => Some("cheerful, upbeat, positive, smiling voice"),
        "dramatic" => Some("dramatic, theatrical, tense"),
        "curious" => Some("curious, inquisitive, wondering"),
        "thoughtful" => Some("thoughtful, contemplative, steady pacing"),
        _ => None,
    }
}

fn preset_voice_reference(name: &str) -> Option<PathBuf> {
    // Try skill-relative voice_profiles/ (installed binary sidecar)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidates = [
                // Installed binary sidecar (e.g., ~/.octos/skills/mofa-podcast/)
                exe_dir.join("voice_profiles").join(format!("preset_{name}.wav")),
                exe_dir.join("voice_profiles").join(format!("{name}.wav")),
                // Cargo target/ parent (e.g., target/debug/../../ -> project root)
                exe_dir.join("../..").join("voice_profiles").join(format!("preset_{name}.wav")),
                exe_dir.join("../..").join("voice_profiles").join(format!("{name}.wav")),
                // Legacy: target/ sibling
                exe_dir.join("..").join("voice_profiles").join(format!("preset_{name}.wav")),
                exe_dir.join("..").join("voice_profiles").join(format!("{name}.wav")),
            ];
            for c in &candidates {
                if c.exists() {
                    return Some(c.clone());
                }
            }
        }
    }
    // Fallback to data_dir
    let data_dir = data_dir();
    let candidates = [
        data_dir.join("voice_profiles").join(format!("preset_{name}.wav")),
        data_dir.join("voice_profiles").join(format!("{name}.wav")),
    ];
    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }
    None
}

fn terminal_char_for_punctuation_check(text: &str) -> Option<char> {
    for ch in text.trim_end().chars().rev() {
        if ch.is_whitespace() {
            continue;
        }
        if matches!(
            ch,
            '"' | '\'' | '”' | '’' | ')' | '）' | ']' | '】' | '}' | '」' | '』'
        ) {
            continue;
        }
        return Some(ch);
    }
    None
}

fn has_terminal_punctuation(text: &str) -> bool {
    terminal_char_for_punctuation_check(text).is_some_and(|ch| {
        matches!(
            ch,
            '.' | '!' | '?' | ',' | ';' | ':' | '。' | '！' | '？' | '，' | '；' | '：' | '…'
        )
    })
}

fn normalize_tts_text(text: &str, language: Option<TtsLanguage>) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() || has_terminal_punctuation(trimmed) {
        return trimmed.to_string();
    }

    let punctuation = match language {
        Some(TtsLanguage::Chinese) => '。',
        _ => '.',
    };
    format!("{trimmed}{punctuation}")
}

// ── Voice registry (shared with mofa-fm) ───────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct VoiceRegistry {
    #[serde(default)]
    default_voice: Option<String>,
    #[serde(default)]
    voices: BTreeMap<String, VoiceEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
struct VoiceEntry {
    file: String,
    #[serde(default)]
    created_at: u64,
}

fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("OCTOS_DATA_DIR") {
        PathBuf::from(d)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".octos")
    } else {
        PathBuf::from("/tmp")
    }
}

fn work_dir() -> Option<PathBuf> {
    std::env::var("OCTOS_WORK_DIR").ok().map(PathBuf::from)
}

fn resolve_workspace_relative_path(path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        return candidate;
    }
    if let Some(work_dir) = work_dir() {
        let workspace_path = work_dir.join(&candidate);
        if workspace_path.exists() {
            return workspace_path;
        }
    }
    candidate
}

fn resolve_output_dir(output_dir: Option<String>) -> PathBuf {
    match output_dir {
        Some(dir) => {
            let dir_path = PathBuf::from(&dir);
            if dir_path.is_absolute() {
                dir_path
            } else if let Some(work_dir) = work_dir() {
                work_dir.join(dir_path)
            } else {
                dir_path
            }
        }
        None => {
            if let Some(work_dir) = work_dir() {
                work_dir.join("skill-output/mofa-podcast")
            } else {
                PathBuf::from("skill-output/mofa-podcast")
            }
        }
    }
}

fn sanitize_filename_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut last_was_underscore = false;

    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
            last_was_underscore = false;
        } else if !last_was_underscore {
            out.push('_');
            last_was_underscore = true;
        }
    }

    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        "voice".to_string()
    } else {
        out
    }
}

fn segment_file_path(seg_dir: &Path, voice: &str, seg_id: u32) -> PathBuf {
    let safe_voice = sanitize_filename_component(voice);
    seg_dir.join(format!("seg_{seg_id:03}_{safe_voice}.wav"))
}

fn placeholder_file_path(seg_dir: &Path, prefix: &str, line_index: usize) -> PathBuf {
    seg_dir.join(format!("{prefix}_{line_index:03}.wav"))
}

fn load_registry() -> VoiceRegistry {
    let path = data_dir().join("voices.json");
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => VoiceRegistry::default(),
    }
}

fn resolve_custom_voice(name: &str) -> Option<PathBuf> {
    // 1. Prefer skill-relative paths (where cloned voices are stored alongside the binary)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // Skill-relative voices.json
            let voices_json = exe_dir.join("voices.json");
            if voices_json.exists() {
                if let Ok(content) = std::fs::read_to_string(&voices_json) {
                    if let Ok(reg) = serde_json::from_str::<VoiceRegistry>(&content) {
                        if let Some(entry) = reg.voices.get(name) {
                            let path = exe_dir.join(&entry.file);
                            if path.exists() {
                                return Some(path);
                            }
                        }
                    }
                }
            }
            // Skill-relative voice_profiles/
            let profiles = exe_dir.join("voice_profiles");
            for suffix in ["", "_ref"] {
                let wav = profiles.join(format!("{name}{suffix}.wav"));
                if wav.exists() {
                    return Some(wav);
                }
            }
        }
    }

    // 2. Fallback to data_dir for backward compatibility
    let reg = load_registry();
    if let Some(entry) = reg.voices.get(name) {
        let p = PathBuf::from(&entry.file);
        if p.exists() {
            return Some(p);
        }
    }
    let dir = data_dir().join("voice_profiles");
    let wav = dir.join(format!("{name}.wav"));
    if wav.exists() {
        Some(wav)
    } else {
        None
    }
}

// ── VoxCPM batch TTS engine ───────────────────────────────────────

const TTS_ENGINE_PY: &str = include_str!("tts_engine.py");

#[derive(Serialize)]
struct TtsSegmentTask {
    seg_id: u32,
    text: String,
    output_path: String,
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    control: Option<String>,
}

#[derive(Serialize)]
struct TtsBatchTask {
    model_id: String,
    device: String,
    cfg_value: f64,
    inference_timesteps: u32,
    normalize: bool,
    segments: Vec<TtsSegmentTask>,
}

#[derive(Deserialize, Debug)]
struct TtsBatchFailedItem {
    seg_id: u32,
    reason: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TtsBatchReport {
    success: bool,
    generated: usize,
    #[serde(default)]
    failed: Vec<TtsBatchFailedItem>,
}

fn resolve_python3_path() -> String {
    // Prefer miniconda3 python where voxcpm is installed
    let candidates = [
        "/Users/loubicheng/miniconda3/bin/python3",
        "/opt/homebrew/bin/python3",
        "/usr/local/bin/python3",
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return c.to_string();
        }
    }
    "python3".to_string()
}

fn run_tts_batch_engine(task: &TtsBatchTask) -> Result<TtsBatchReport, String> {
    let task_json =
        serde_json::to_string(task).map_err(|e| format!("Serialize batch task: {e}"))?;

    // Write embedded Python script to a temp file so the binary is self-contained.
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("mofa_podcast_tts_engine_{}.py", timestamp()));
    std::fs::write(&script_path, TTS_ENGINE_PY)
        .map_err(|e| format!("Failed to write temp tts_engine.py: {e}"))?;

    let python = resolve_python3_path();
    let mut child = Command::new(&python)
        .arg("-u")
        .arg(&script_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn voxcpm batch engine ({python}): {e}"))?;

    {
        use std::io::Write;
        let mut stdin = child.stdin.take().ok_or("Failed to open engine stdin")?;
        stdin
            .write_all(task_json.as_bytes())
            .map_err(|e| format!("Failed to write batch task to engine stdin: {e}"))?;
        // Explicitly close stdin so child knows EOF
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("voxcpm batch engine failed: {e}"))?;

    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "voxcpm batch engine exited with code {} (python: {}). stderr: {}. stdout: {}",
            output.status.code().unwrap_or(-1),
            python,
            if stderr.is_empty() { "(empty)" } else { &stderr[..stderr.len().min(500)] },
            if stdout.is_empty() { "(empty)" } else { &stdout[..stdout.len().min(200)] },
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: TtsBatchReport = serde_json::from_str(&stdout).map_err(|e| {
        format!(
            "Failed to parse batch report: {e}. Raw stdout: {}",
            &stdout[..stdout.len().min(500)]
        )
    })?;

    Ok(report)
}

// ── Audio helpers ──────────────────────────────────────────────────

struct WavMetadata<'a> {
    audio_format: u16,
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
    data: &'a [u8],
}

fn parse_wav_metadata(bytes: &[u8]) -> Result<WavMetadata<'_>, String> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("not a RIFF/WAVE file".to_string());
    }

    let mut offset = 12usize;
    let mut fmt: Option<(u16, u16, u32, u16)> = None;
    let mut data: Option<&[u8]> = None;

    while offset + 8 <= bytes.len() {
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        let chunk_start = offset + 8;
        let chunk_end = chunk_start
            .checked_add(chunk_size)
            .ok_or_else(|| "invalid WAV chunk size".to_string())?;
        if chunk_end > bytes.len() {
            return Err("truncated WAV chunk".to_string());
        }

        match chunk_id {
            b"fmt " => {
                if chunk_size < 16 {
                    return Err("WAV fmt chunk too short".to_string());
                }
                fmt = Some((
                    u16::from_le_bytes([bytes[chunk_start], bytes[chunk_start + 1]]),
                    u16::from_le_bytes([bytes[chunk_start + 2], bytes[chunk_start + 3]]),
                    u32::from_le_bytes([
                        bytes[chunk_start + 4],
                        bytes[chunk_start + 5],
                        bytes[chunk_start + 6],
                        bytes[chunk_start + 7],
                    ]),
                    u16::from_le_bytes([bytes[chunk_start + 14], bytes[chunk_start + 15]]),
                ));
            }
            b"data" => {
                data = Some(&bytes[chunk_start..chunk_end]);
            }
            _ => {}
        }

        offset = chunk_end + (chunk_size % 2);
    }

    let (audio_format, channels, sample_rate, bits_per_sample) =
        fmt.ok_or_else(|| "WAV fmt chunk missing".to_string())?;
    let data = data.ok_or_else(|| "WAV data chunk missing".to_string())?;
    Ok(WavMetadata {
        audio_format,
        channels,
        sample_rate,
        bits_per_sample,
        data,
    })
}

fn pcm_to_wav_with_format(pcm: &[u8], sample_rate: u32, channels: u16, bits: u16) -> Vec<u8> {
    let byte_rate = sample_rate * u32::from(channels) * u32::from(bits) / 8;
    let block_align = channels * bits / 8;
    let data_len = pcm.len() as u32;
    let file_len = 36 + data_len;

    let mut buf = Vec::with_capacity(44 + pcm.len());
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_len.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    buf.extend_from_slice(pcm);
    buf
}

fn pcm_to_wav(pcm: &[u8], sample_rate: u32) -> Vec<u8> {
    pcm_to_wav_with_format(pcm, sample_rate, 1, 16)
}

fn generate_silence_wav(duration_ms: u32) -> Vec<u8> {
    let sample_rate: u32 = 24000;
    let num_samples = sample_rate * duration_ms / 1000;
    let pcm = vec![0u8; (num_samples * 2) as usize]; // 16-bit silence
    pcm_to_wav(&pcm, sample_rate)
}

fn append_trailing_silence_to_wav(bytes: &[u8], duration_ms: u32) -> Vec<u8> {
    if duration_ms == 0 {
        return bytes.to_vec();
    }

    let Ok(wav) = parse_wav_metadata(bytes) else {
        return bytes.to_vec();
    };
    if wav.audio_format != 1 || wav.bits_per_sample == 0 || wav.bits_per_sample % 8 != 0 {
        return bytes.to_vec();
    }

    let bytes_per_frame =
        usize::from(wav.channels).saturating_mul(usize::from(wav.bits_per_sample / 8));
    if bytes_per_frame == 0 {
        return bytes.to_vec();
    }

    let pad_frames = (u64::from(wav.sample_rate) * u64::from(duration_ms)) / 1000;
    let pad_bytes = pad_frames.saturating_mul(bytes_per_frame as u64) as usize;

    let mut pcm = Vec::with_capacity(wav.data.len().saturating_add(pad_bytes));
    pcm.extend_from_slice(wav.data);
    pcm.resize(pcm.len().saturating_add(pad_bytes), 0);
    pcm_to_wav_with_format(&pcm, wav.sample_rate, wav.channels, wav.bits_per_sample)
}

fn audio_duration_ms(bytes: &[u8], sample_rate: u32) -> u32 {
    if let Ok(wav) = parse_wav_metadata(bytes) {
        let bytes_per_frame =
            usize::from(wav.channels).saturating_mul(usize::from(wav.bits_per_sample / 8));
        if bytes_per_frame == 0 || wav.sample_rate == 0 {
            return 0;
        }
        return ((wav.data.len() / bytes_per_frame) as u32)
            .saturating_mul(1000)
            .saturating_div(wav.sample_rate);
    }

    ((bytes.len() / 2) as u32)
        .saturating_mul(1000)
        .saturating_div(sample_rate)
}

fn has_meaningful_tts_audio(bytes: &[u8]) -> bool {
    if audio_duration_ms(bytes, 24_000) < 150 {
        return false;
    }

    let pcm = parse_wav_metadata(bytes)
        .map(|wav| wav.data)
        .unwrap_or(bytes);

    let mut non_silent_samples = 0usize;
    for chunk in pcm.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        if sample.unsigned_abs() >= 16 {
            non_silent_samples += 1;
            if non_silent_samples >= 32 {
                return true;
            }
        }
    }
    false
}

/// Resolve ffmpeg binary path — checks PATH first, then common install locations.
fn ffmpeg_bin() -> &'static str {
    static FFMPEG: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FFMPEG.get_or_init(|| {
        // Check PATH first
        if Command::new("ffmpeg")
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
        {
            return "ffmpeg".to_string();
        }
        // Common install locations (macOS homebrew, linux)
        for path in [
            "/opt/homebrew/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
            "/usr/bin/ffmpeg",
        ] {
            if Path::new(path).exists() {
                return path.to_string();
            }
        }
        "ffmpeg".to_string() // fallback, will fail gracefully
    })
}

struct FinalAudioOutput {
    path: String,
    format: &'static str,
    warning: Option<String>,
}

fn finalize_audio_output(wav_path: &str) -> FinalAudioOutput {
    let mp3_path = wav_path.replace(".wav", ".mp3");
    let result = Command::new(ffmpeg_bin())
        .args([
            "-y",
            "-i",
            wav_path,
            "-codec:a",
            "libmp3lame",
            "-b:a",
            "192k",
            "-q:a",
            "2",
            &mp3_path,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match result {
        Ok(s) if s.success() => {
            let _ = std::fs::remove_file(wav_path);
            FinalAudioOutput {
                path: mp3_path,
                format: "mp3",
                warning: None,
            }
        }
        Ok(_) | Err(_) => {
            let _ = std::fs::remove_file(&mp3_path);
            FinalAudioOutput {
                path: wav_path.to_string(),
                format: "wav",
                warning: Some(
                    "ffmpeg conversion unavailable; returning WAV output instead of MP3"
                        .to_string(),
                ),
            }
        }
    }
}

fn write_file_bytes(path: &Path, bytes: &[u8], label: &str) -> Result<(), String> {
    std::fs::write(path, bytes)
        .map_err(|e| format!("Failed to write {label} '{}': {e}", path.display()))
}

fn extract_pcm_for_concat<'a>(bytes: &'a [u8], path: &str) -> Result<&'a [u8], String> {
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        let wav = parse_wav_metadata(bytes).map_err(|e| format!("Invalid WAV '{}': {e}", path))?;
        if wav.audio_format != 1 {
            return Err(format!(
                "Unsupported WAV format in '{}': expected PCM but got format {}",
                path, wav.audio_format
            ));
        }
        if wav.channels != 1 || wav.bits_per_sample != 16 || wav.sample_rate != 24_000 {
            return Err(format!(
                "Unsupported WAV format in '{}': expected 24kHz mono 16-bit PCM, got {}Hz {}ch {}-bit",
                path, wav.sample_rate, wav.channels, wav.bits_per_sample
            ));
        }
        Ok(wav.data)
    } else {
        Ok(bytes)
    }
}

/// Concatenate multiple WAV files into one.
/// Tries ffmpeg first for best quality; falls back to raw PCM concatenation.
fn concatenate_wavs(wav_paths: &[String], output_path: &str) -> Result<(), String> {
    if wav_paths.is_empty() {
        return Err("No WAV files to concatenate".into());
    }

    // Try ffmpeg first
    if concatenate_wavs_ffmpeg(wav_paths, output_path).is_ok() {
        return Ok(());
    }
    eprintln!("[podcast] ffmpeg not available, using raw WAV concatenation");

    // Fallback: raw PCM concatenation (all WAVs are 24kHz 16-bit mono)
    let mut all_pcm: Vec<u8> = Vec::new();
    for path in wav_paths {
        let data = std::fs::read(path).map_err(|e| format!("Failed to read {path}: {e}"))?;
        all_pcm.extend_from_slice(extract_pcm_for_concat(&data, path)?);
    }
    let wav = pcm_to_wav(&all_pcm, 24000);
    std::fs::write(output_path, &wav).map_err(|e| format!("Failed to write {output_path}: {e}"))?;
    Ok(())
}

fn concatenate_wavs_ffmpeg(wav_paths: &[String], output_path: &str) -> Result<(), String> {
    let mut filter_inputs = String::new();
    let mut args: Vec<String> = vec!["-y".into()];

    for (i, path) in wav_paths.iter().enumerate() {
        args.push("-i".into());
        args.push(path.clone());
        filter_inputs.push_str(&format!("[{i}:a]"));
    }
    let filter_concat = format!("{}concat=n={}:v=0:a=1[out]", filter_inputs, wav_paths.len());

    args.push("-filter_complex".into());
    args.push(filter_concat);
    args.push("-map".into());
    args.push("[out]".into());
    args.push("-ar".into());
    args.push("24000".into());
    args.push("-ac".into());
    args.push("1".into());
    args.push(output_path.into());

    let result = Command::new(ffmpeg_bin())
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output();

    match result {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!(
                "ffmpeg concat failed: {}",
                &stderr[..stderr.len().min(300)]
            ))
        }
        Err(e) => Err(format!("ffmpeg not available: {e}")),
    }
}

// ── Script parser ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum ScriptLine {
    Dialogue {
        seg_id: u32,
        character: String,
        voice: String,
        is_clone: bool,
        emotion: String,
        text: String,
    },
    Bgm {
        #[allow(dead_code)]
        description: String,
        #[allow(dead_code)]
        fade: String,
        duration_s: u32,
    },
    Pause {
        duration_s: u32,
    },
}

#[derive(Debug, Default)]
struct ScriptParseReport {
    lines: Vec<ScriptLine>,
    invalid_lines: Vec<String>,
    repair_summary: ScriptRepairSummary,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ScriptRepairSummary {
    normalized_markdown_wrappers: usize,
    normalized_header_punctuation: usize,
    collapsed_multiline_dialogues: usize,
    repaired_known_speaker_voices: usize,
}

impl ScriptRepairSummary {
    fn has_repairs(self) -> bool {
        self.normalized_markdown_wrappers > 0
            || self.normalized_header_punctuation > 0
            || self.collapsed_multiline_dialogues > 0
            || self.repaired_known_speaker_voices > 0
    }

    fn messages(self) -> Vec<String> {
        let mut messages = Vec::new();
        if self.normalized_markdown_wrappers > 0 {
            let count = self.normalized_markdown_wrappers;
            messages.push(format!(
                "normalized markdown wrappers on {count} dialogue line{}",
                if count == 1 { "" } else { "s" }
            ));
        }
        if self.normalized_header_punctuation > 0 {
            let count = self.normalized_header_punctuation;
            messages.push(format!(
                "normalized bracket/header punctuation on {count} line{}",
                if count == 1 { "" } else { "s" }
            ));
        }
        if self.collapsed_multiline_dialogues > 0 {
            let count = self.collapsed_multiline_dialogues;
            messages.push(format!(
                "collapsed {count} multiline dialogue block{} into canonical one-line format",
                if count == 1 { "" } else { "s" }
            ));
        }
        if self.repaired_known_speaker_voices > 0 {
            let count = self.repaired_known_speaker_voices;
            messages.push(format!(
                "repaired known speaker voice alias on {count} dialogue line{}",
                if count == 1 { "" } else { "s" }
            ));
        }
        messages
    }
}

fn normalized_character_alias_key(character: &str) -> String {
    character
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-' && *ch != '_')
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn preferred_clone_voice_for_character(character: &str) -> Option<&'static str> {
    match normalized_character_alias_key(character).as_str() {
        "杨幂" | "yangmi" => Some("yangmi"),
        "窦文涛" | "douwentao" => Some("douwt"),
        _ => None,
    }
}

fn repair_known_speaker_voice_alias(
    character: &str,
    voice: String,
    is_clone: bool,
) -> (String, bool, bool) {
    let Some(preferred_voice) = preferred_clone_voice_for_character(character) else {
        return (voice, is_clone, false);
    };
    if is_clone && voice == preferred_voice {
        return (voice, is_clone, false);
    }
    (preferred_voice.to_string(), true, true)
}

struct NormalizedScriptLine {
    text: String,
    normalized_markdown_wrapper: bool,
    normalized_header_punctuation: bool,
}

fn normalize_script_line(line: &str) -> NormalizedScriptLine {
    let mut text = line.trim().to_string();
    let mut normalized_markdown_wrapper = false;
    let mut normalized_header_punctuation = false;

    let bracket_normalized = text
        .replace('【', "[")
        .replace('】', "]")
        .replace('［', "[")
        .replace('］', "]");
    if bracket_normalized != text {
        text = bracket_normalized;
        normalized_header_punctuation = true;
    }

    if text.starts_with("**[") {
        if let Some(end) = text.find("]**") {
            text = format!("{}{}", &text[2..end + 1], &text[end + 3..]);
            normalized_markdown_wrapper = true;
        }
    }

    if text.starts_with('[') {
        if let Some(close) = text.find(']') {
            let (header, rest) = text.split_at(close + 1);
            let normalized_header = header
                .replace('—', " - ")
                .replace('–', " - ")
                .replace('－', " - ")
                .replace('：', ":")
                .replace('，', ",");
            if normalized_header != header {
                text = format!("{normalized_header}{rest}");
                normalized_header_punctuation = true;
            }
        }
    }

    NormalizedScriptLine {
        text,
        normalized_markdown_wrapper,
        normalized_header_punctuation,
    }
}

fn is_skippable_script_metadata(line: &str) -> bool {
    line.starts_with('#')
        || line.starts_with('|')
        || line.starts_with("**")
        || line.starts_with('-')
        || line.starts_with('>')
        || line.starts_with("```")
        || line == "---"
}

fn parse_script_report(script: &str) -> ScriptParseReport {
    let dialogue_re = Regex::new(r"^\[([^\]\-]+)\s*-\s*([^\],]+),\s*([^\]]+)\]\s*(.*)$").unwrap();
    let bgm_re = Regex::new(r"^\[BGM:\s*([^—\-]+)[—\-]\s*([^,]+),\s*(\d+)s?\]").unwrap();
    let pause_re = Regex::new(r"^\[PAUSE:\s*(\d+)s?\]").unwrap();

    let mut lines = Vec::new();
    let mut invalid_lines = Vec::new();
    let mut seg_counter: u32 = 0;
    let mut repair_summary = ScriptRepairSummary::default();

    let raw_lines: Vec<&str> = script.lines().collect();
    let mut i = 0usize;
    while i < raw_lines.len() {
        let line = raw_lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }

        let normalized_line = normalize_script_line(line);
        if normalized_line.normalized_markdown_wrapper {
            repair_summary.normalized_markdown_wrappers += 1;
        }
        if normalized_line.normalized_header_punctuation {
            repair_summary.normalized_header_punctuation += 1;
        }

        if let Some(caps) = bgm_re.captures(&normalized_line.text) {
            lines.push(ScriptLine::Bgm {
                description: caps[1].trim().to_string(),
                fade: caps[2].trim().to_string(),
                duration_s: caps[3].parse().unwrap_or(3),
            });
        } else if let Some(caps) = pause_re.captures(&normalized_line.text) {
            lines.push(ScriptLine::Pause {
                duration_s: caps[1].parse().unwrap_or(2),
            });
        } else if let Some(caps) = dialogue_re.captures(&normalized_line.text) {
            let character = caps[1].trim().to_string();
            let voice_raw = caps[2].trim().to_string();
            let emotion = caps[3].trim().to_string();
            let inline_text = caps[4].trim().to_string();

            let text = if !inline_text.is_empty() {
                inline_text
            } else {
                let mut continuation = Vec::new();
                let mut j = i + 1;
                while j < raw_lines.len() {
                    let next = raw_lines[j].trim();
                    if next.is_empty() {
                        if continuation.is_empty() {
                            j += 1;
                            continue;
                        }
                        break;
                    }

                    let normalized_next = normalize_script_line(next);
                    if dialogue_re.is_match(&normalized_next.text)
                        || bgm_re.is_match(&normalized_next.text)
                        || pause_re.is_match(&normalized_next.text)
                        || is_skippable_script_metadata(next)
                    {
                        break;
                    }

                    continuation.push(next.to_string());
                    j += 1;
                }

                if continuation.is_empty() {
                    invalid_lines.push(line.to_string());
                    i += 1;
                    continue;
                }

                repair_summary.collapsed_multiline_dialogues += 1;
                i = j - 1;
                continuation.join(" ")
            };

            let (voice, is_clone) = if let Some(cloned) = voice_raw.strip_prefix("clone:") {
                (cloned.to_string(), true)
            } else {
                (voice_raw.clone(), false)
            };
            let (voice, is_clone, repaired_voice_alias) =
                repair_known_speaker_voice_alias(&character, voice, is_clone);
            if repaired_voice_alias {
                repair_summary.repaired_known_speaker_voices += 1;
            }

            seg_counter += 1;
            lines.push(ScriptLine::Dialogue {
                seg_id: seg_counter,
                character,
                voice,
                is_clone,
                emotion,
                text,
            });
        } else if is_skippable_script_metadata(line) {
            i += 1;
            continue;
        } else {
            invalid_lines.push(line.to_string());
        }
        i += 1;
    }
    ScriptParseReport {
        lines,
        invalid_lines,
        repair_summary,
    }
}

#[cfg(test)]
fn parse_script(script: &str) -> Vec<ScriptLine> {
    parse_script_report(script).lines
}

fn format_invalid_script_lines(invalid_lines: &[String]) -> String {
    let preview = invalid_lines
        .iter()
        .take(5)
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let suffix = if invalid_lines.len() > 5 {
        format!("\n...and {} more malformed lines", invalid_lines.len() - 5)
    } else {
        String::new()
    };
    format!(
        "Script contains {} malformed non-metadata lines. Expected dialogue lines like [Character - voice, emotion] text, [BGM: ...], or [PAUSE: ...].\n{}{}",
        invalid_lines.len(),
        preview,
        suffix
    )
}

fn render_canonical_script(lines: &[ScriptLine]) -> String {
    let mut rendered = Vec::with_capacity(lines.len());
    for line in lines {
        match line {
            ScriptLine::Dialogue {
                character,
                voice,
                is_clone,
                emotion,
                text,
                ..
            } => {
                let voice_label = if *is_clone {
                    format!("clone:{voice}")
                } else {
                    voice.clone()
                };
                rendered.push(format!("[{character} - {voice_label}, {emotion}] {text}"));
            }
            ScriptLine::Bgm {
                description,
                fade,
                duration_s,
            } => {
                rendered.push(format!("[BGM: {description} - {fade}, {duration_s}s]"));
            }
            ScriptLine::Pause { duration_s } => {
                rendered.push(format!("[PAUSE: {duration_s}s]"));
            }
        }
    }
    rendered.join("\n\n")
}

struct PreparedPodcastScript {
    lines: Vec<ScriptLine>,
    repair_messages: Vec<String>,
    repaired_markdown: Option<String>,
}

fn validate_and_fix_script_markdown(script: &str) -> Result<PreparedPodcastScript, String> {
    let report = parse_script_report(script);
    if report.lines.is_empty() {
        return Err(
            "No dialogue lines found in script. Ensure format: [Character - voice, emotion] text"
                .to_string(),
        );
    }
    if !report.invalid_lines.is_empty() {
        return Err(format_invalid_script_lines(&report.invalid_lines));
    }

    let repair_messages = report.repair_summary.messages();
    let repaired_markdown = report
        .repair_summary
        .has_repairs()
        .then(|| render_canonical_script(&report.lines));

    Ok(PreparedPodcastScript {
        lines: report.lines,
        repair_messages,
        repaired_markdown,
    })
}

fn write_repaired_script_markdown(output_dir: &Path, markdown: &str) -> Result<PathBuf, String> {
    let repaired_path = output_dir.join(format!("podcast_script_normalized_{}.md", timestamp()));
    std::fs::write(&repaired_path, markdown).map_err(|error| {
        format!(
            "Failed to write normalized podcast script '{}': {error}",
            repaired_path.display()
        )
    })?;
    Ok(repaired_path)
}

fn attach_script_fix_context(
    message: String,
    repaired_script_path: Option<&Path>,
    repair_messages: &[String],
) -> String {
    if repair_messages.is_empty() && repaired_script_path.is_none() {
        return message;
    }

    let mut enriched = message;
    if !repair_messages.is_empty() {
        enriched.push_str("\nScript validator repairs: ");
        enriched.push_str(&repair_messages.join("; "));
    }
    if let Some(path) = repaired_script_path {
        enriched.push_str(&format!("\nNormalized script: {}", path.display()));
    }
    enriched
}

// ── Tool handlers ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct GenerateInput {
    #[serde(default)]
    script: Option<String>,
    #[serde(default)]
    script_path: Option<String>,
    #[serde(default)]
    output_dir: Option<String>,
}

struct SegmentDirCleanup {
    path: PathBuf,
}

impl SegmentDirCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for SegmentDirCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn list_custom_voices_from_profiles(presets: &[&str]) -> Vec<String> {
    let mut voices = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let profiles = exe_dir.join("voice_profiles");
            if let Ok(entries) = std::fs::read_dir(&profiles) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("wav") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let name = if let Some(base) = stem.strip_suffix("_ref") {
                                base.to_string()
                            } else {
                                stem.to_string()
                            };
                            if !presets.contains(&name.as_str()) && !voices.contains(&name) {
                                voices.push(name);
                            }
                        }
                    }
                }
            }
        }
    }
    voices
}

// ── Voice save handler ─────────────────────────────────────────────

#[derive(Deserialize)]
struct SaveVoiceInput {
    name: String,
    audio_path: String,
    #[serde(default)]
    force: bool,
}

fn handle_save_voice(input_json: &str) {
    let input: SaveVoiceInput = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => fail(&format!("Invalid input: {e}")),
    };

    let name = sanitize_filename_component(&input.name);
    if name.is_empty() {
        fail("Voice name cannot be empty");
    }
    if name.len() > 64 {
        fail("Voice name too long (max 64 chars)");
    }

    // Resolve source audio path
    let src = resolve_workspace_relative_path(&input.audio_path);
    if !src.exists() {
        fail(&format!("Audio file not found: {}", src.display()));
    }
    if !src.is_file() {
        fail(&format!("Not a file: {}", src.display()));
    }

    // Validate WAV header (first 4 bytes should be "RIFF")
    match std::fs::read(&src) {
        Ok(bytes) if bytes.len() >= 4 && &bytes[..4] == b"RIFF" => {}
        Ok(_) => fail("Audio file is not a valid WAV (expected RIFF header)"),
        Err(e) => fail(&format!("Failed to read audio file: {e}")),
    }

    // Determine destination: skill/voice_profiles/{name}_ref.wav
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    let profiles_dir = exe_dir.join("voice_profiles");
    if let Err(e) = std::fs::create_dir_all(&profiles_dir) {
        fail(&format!("Failed to create voice_profiles dir: {e}"));
    }
    let dest = profiles_dir.join(format!("{name}_ref.wav"));

    if dest.exists() && !input.force {
        fail(&format!(
            "Voice '{name}' already exists. Use force=true to overwrite."
        ));
    }

    match std::fs::copy(&src, &dest) {
        Ok(_) => {}
        Err(e) => fail(&format!("Failed to save voice file: {e}")),
    }

    // Return success with current voice list
    let presets = PRESET_VOICE_NAMES.to_vec();
    let custom = list_custom_voices_from_profiles(&presets);
    let mut out = format!("Voice '{name}' saved successfully to {}\n\n", dest.display());
    out.push_str("## Available Voices\n\n");
    out.push_str("### Preset (built-in)\n");
    for v in &presets {
        out.push_str(&format!("- `{v}`\n"));
    }
    if !custom.is_empty() {
        out.push_str("\n### Custom (cloned)\n");
        for v in &custom {
            out.push_str(&format!("- `{v}` (use as `clone:{v}` in script)\n"));
        }
    }
    succeed(&out);
}

fn handle_voices(_input_json: &str) {
    let presets: Vec<&str> = PRESET_VOICE_NAMES.to_vec();
    let mut custom = list_custom_voices_from_profiles(&presets);

    // Also include voices from registry (backward compat)
    let reg = load_registry();
    for name in reg.voices.keys() {
        if !presets.contains(&name.as_str()) && !custom.contains(name) {
            custom.push(name.clone());
        }
    }

    let mut out = String::new();
    out.push_str("## Available Voices\n\n");
    out.push_str("### Preset (built-in)\n");
    for v in &presets {
        out.push_str(&format!("- `{v}`\n"));
    }

    if !custom.is_empty() {
        out.push_str("\n### Custom (cloned)\n");
        for v in &custom {
            out.push_str(&format!("- `{v}` (use as `clone:{v}` in script)\n"));
        }
    } else {
        out.push_str("\n### Custom (cloned)\n");
        out.push_str(
            "_No custom voices found. Place `.wav` reference audio in the skill's `voice_profiles/` directory._\n",
        );
    }

    succeed(&out);
}

fn handle_generate(input_json: &str) {
    let input: GenerateInput = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => fail(&format!("Invalid input: {e}")),
    };

    match generate_podcast(input) {
        Ok(out) => {
            println!("{out}");
            std::process::exit(0);
        }
        Err(err) => fail(&err),
    }
}

fn generate_podcast(input: GenerateInput) -> Result<serde_json::Value, String> {
    // Read script content
    let script = if let Some(s) = input.script {
        s
    } else if let Some(ref path) = input.script_path {
        let resolved = resolve_workspace_relative_path(path);
        match std::fs::read_to_string(&resolved) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!(
                    "Failed to read script file '{}': {e}",
                    resolved.display()
                ))
            }
        }
    } else {
        return Err("Either 'script' or 'script_path' must be provided".to_string());
    };

    // Setup output directory
    let output_dir = resolve_output_dir(input.output_dir);
    let seg_dir = output_dir.join("segments");
    std::fs::create_dir_all(&seg_dir).map_err(|e| {
        format!(
            "Failed to create segment directory '{}': {e}",
            seg_dir.display()
        )
    })?;
    let _seg_dir_cleanup = SegmentDirCleanup::new(seg_dir.clone());

    // Validate and normalize script markdown before generation.
    let prepared_script = validate_and_fix_script_markdown(&script)?;
    let repaired_script_path = if let Some(markdown) = prepared_script.repaired_markdown.as_deref()
    {
        Some(write_repaired_script_markdown(&output_dir, markdown)?)
    } else {
        None
    };
    if let Some(path) = &repaired_script_path {
        eprintln!(
            "[podcast] Script validator wrote normalized markdown to {}",
            path.display()
        );
    }
    let repair_messages = prepared_script.repair_messages;
    let lines = prepared_script.lines;

    let dialogue_count = lines
        .iter()
        .filter(|l| matches!(l, ScriptLine::Dialogue { .. }))
        .count();
    eprintln!(
        "[podcast] Parsed {} script lines ({} dialogue segments)",
        lines.len(),
        dialogue_count
    );

    // Resolve reference audio for all dialogue segments.
    // With voxcpm all voices are clone-based (preset voices ship built-in reference audio).
    let mut segments: Vec<(u32, String, String, String, String, Option<PathBuf>)> = Vec::new();
    let mut configuration_errors: Vec<String> = Vec::new();

    for line in &lines {
        if let ScriptLine::Dialogue {
            seg_id,
            voice,
            is_clone,
            emotion,
            text,
            character,
            ..
        } = line
        {
            let ref_path = if *is_clone {
                resolve_custom_voice(voice)
            } else if PRESET_VOICE_NAMES.contains(&voice.as_str()) {
                preset_voice_reference(voice)
            } else {
                // Auto-detect: not a preset but exists in clone registry
                resolve_custom_voice(voice)
            };

            if let Some(path) = ref_path {
                segments.push((
                    *seg_id,
                    voice.clone(),
                    emotion.clone(),
                    text.clone(),
                    character.clone(),
                    Some(path),
                ));
            } else {
                configuration_errors.push(format!(
                    "seg_{seg_id:03} ({character}): voice '{voice}' not found. Preset voices require built-in reference audio; custom voices must be saved first."
                ));
            }
        }
    }

    if !configuration_errors.is_empty() {
        return Err(format!(
            "{}",
            attach_script_fix_context(
                format!(
                    "Invalid podcast voice configuration:\n{}",
                    configuration_errors.join("\n")
                ),
                repaired_script_path.as_deref(),
                &repair_messages,
            )
        ));
    }

    // Sort by voice name to minimize model context switching
    segments.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    // Build batch task
    let mut batch_tasks: Vec<TtsSegmentTask> = Vec::new();
    for (seg_id, voice, emotion, text, _character, ref_path) in &segments {
        let seg_path = segment_file_path(&seg_dir, voice, *seg_id);
        let control = emotion_to_control(emotion).map(|c| c.to_string());
        batch_tasks.push(TtsSegmentTask {
            seg_id: *seg_id,
            text: text.clone(),
            output_path: seg_path.to_string_lossy().to_string(),
            mode: "clone".to_string(),
            reference_audio: ref_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            control,
        });
    }

    let mut errors: Vec<String> = Vec::new();

    if !batch_tasks.is_empty() {
        eprintln!(
            "[podcast] Generating {} voice segments via voxcpm batch engine...",
            batch_tasks.len()
        );
        let task = TtsBatchTask {
            model_id: "openbmb/VoxCPM2".to_string(),
            device: "auto".to_string(),
            cfg_value: 2.0,
            inference_timesteps: 10,
            normalize: true,
            segments: batch_tasks,
        };
        match run_tts_batch_engine(&task) {
            Ok(report) => {
                for f in &report.failed {
                    eprintln!("[podcast] ERROR seg_{:03}: {}", f.seg_id, f.reason);
                    if let Some((_, _, _, _, character, _)) =
                        segments.iter().find(|(id, _, _, _, _, _)| *id == f.seg_id)
                    {
                        errors.push(format!("seg_{:03} ({character}): {}", f.seg_id, f.reason));
                    } else {
                        errors.push(format!("seg_{:03}: {}", f.seg_id, f.reason));
                    }
                }
            }
            Err(e) => {
                return Err(attach_script_fix_context(
                    format!("VoxCPM batch engine failed: {e}"),
                    repaired_script_path.as_deref(),
                    &repair_messages,
                ));
            }
        }
    }

    // Phase 3: Assemble timeline
    eprintln!("[podcast] Phase 3: Assembling timeline...");
    let mut timeline_wavs: Vec<String> = Vec::new();
    let mut assembled_dialogue_segments = 0usize;

    for (line_index, line) in lines.iter().enumerate() {
        match line {
            ScriptLine::Dialogue { seg_id, voice, .. } => {
                let seg_path = segment_file_path(&seg_dir, voice, *seg_id);
                if seg_path.exists() {
                    timeline_wavs.push(seg_path.to_string_lossy().to_string());
                    assembled_dialogue_segments += 1;
                    // Insert inter-speaker pause (400ms)
                    let pause_path = seg_dir.join(format!("pause_after_{seg_id:03}.wav"));
                    let silence = generate_silence_wav(400);
                    write_file_bytes(&pause_path, &silence, "inter-speaker pause")?;
                    timeline_wavs.push(pause_path.to_string_lossy().to_string());
                } else {
                    errors.push(format!("seg_{seg_id:03}: missing generated dialogue audio"));
                }
            }
            ScriptLine::Pause { duration_s } => {
                let pause_path = placeholder_file_path(&seg_dir, "pause_line", line_index);
                let silence = generate_silence_wav(duration_s * 1000);
                write_file_bytes(&pause_path, &silence, "pause placeholder")?;
                timeline_wavs.push(pause_path.to_string_lossy().to_string());
            }
            ScriptLine::Bgm { duration_s, .. } => {
                // BGM placeholder: insert silence for now (music mixed in post-production)
                let bgm_path = placeholder_file_path(&seg_dir, "bgm_placeholder_line", line_index);
                let silence = generate_silence_wav(duration_s * 1000);
                write_file_bytes(&bgm_path, &silence, "BGM placeholder")?;
                timeline_wavs.push(bgm_path.to_string_lossy().to_string());
            }
        }
    }

    if timeline_wavs.is_empty() {
        return Err(attach_script_fix_context(
            "No audio segments were generated successfully".to_string(),
            repaired_script_path.as_deref(),
            &repair_messages,
        ));
    }

    if assembled_dialogue_segments != dialogue_count {
        return Err(attach_script_fix_context(
            format!(
                "Podcast generation incomplete: expected {dialogue_count} dialogue segments, but only assembled {assembled_dialogue_segments}. Failed segments:\n{}",
                errors.join("\n")
            ),
            repaired_script_path.as_deref(),
            &repair_messages,
        ));
    }

    // Concatenate all WAVs
    let concat_wav = output_dir.join(format!("podcast_full_{}.wav", timestamp()));
    if let Err(e) = concatenate_wavs(&timeline_wavs, &concat_wav.to_string_lossy()) {
        return Err(attach_script_fix_context(
            format!("Concatenation failed: {e}"),
            repaired_script_path.as_deref(),
            &repair_messages,
        ));
    }

    // Convert to MP3
    let final_audio = finalize_audio_output(&concat_wav.to_string_lossy());

    // Ensure absolute path for files_to_send (crew needs absolute paths for auto-delivery)
    let final_path = std::fs::canonicalize(&final_audio.path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(final_audio.path);

    // Report
    let file_size = std::fs::metadata(&final_path).map(|m| m.len()).unwrap_or(0);
    if file_size == 0 {
        return Err(attach_script_fix_context(
            format!(
                "Final {} output file was empty: {}",
                final_audio.format, final_path
            ),
            repaired_script_path.as_deref(),
            &repair_messages,
        ));
    }
    let size_mb = file_size as f64 / 1_048_576.0;

    let mut output_msg = format!(
        "Podcast generated successfully!\n\
         - Segments: {dialogue_count} dialogue + {} BGM/pause\n\
         - Output: {final_path} ({size_mb:.1} MB, {})",
        lines.len() - dialogue_count,
        final_audio.format.to_uppercase()
    );
    if let Some(warning) = &final_audio.warning {
        output_msg.push_str(&format!("\n- Note: {warning}"));
    }
    if !repair_messages.is_empty() {
        output_msg.push_str(&format!(
            "\n- Script validator repairs: {}",
            repair_messages.join("; ")
        ));
    }
    let normalized_script_path = repaired_script_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    if let Some(path) = &normalized_script_path {
        output_msg.push_str(&format!("\n- Normalized script: {path}"));
    }

    Ok(json!({
        "output": output_msg,
        "success": true,
        "files_to_send": [&final_path],
        "script_repair": {
            "applied": !repair_messages.is_empty(),
            "messages": repair_messages,
            "normalized_script_path": normalized_script_path,
        }
    }))
}

// ── Utility ────────────────────────────────────────────────────────

fn fail(msg: &str) -> ! {
    let out = json!({ "error": msg, "success": false });
    println!("{out}");
    std::process::exit(1);
}

fn succeed(msg: &str) -> ! {
    let out = json!({ "output": msg, "success": true });
    println!("{out}");
    std::process::exit(0);
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Script parser tests ────────────────────────────────────────

    #[test]
    fn parse_basic_dialogue() {
        let script = "[Host - douwt, cheerful] Hello world!";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Dialogue {
                seg_id,
                character,
                voice,
                is_clone,
                emotion,
                text,
            } => {
                assert_eq!(*seg_id, 1);
                assert_eq!(character, "Host");
                assert_eq!(voice, "douwt");
                assert!(!is_clone);
                assert_eq!(emotion, "cheerful");
                assert_eq!(text, "Hello world!");
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_clone_voice() {
        let script = "[Expert - clone:sarah, serious] This is important data.";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Dialogue {
                voice, is_clone, ..
            } => {
                assert_eq!(voice, "sarah");
                assert!(*is_clone);
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_bgm_cue() {
        let script = "[BGM: Upbeat intro music — fade-in, 5s]";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Bgm {
                description,
                fade,
                duration_s,
            } => {
                assert_eq!(description, "Upbeat intro music");
                assert_eq!(fade, "fade-in");
                assert_eq!(*duration_s, 5);
            }
            _ => panic!("Expected Bgm"),
        }
    }

    #[test]
    fn parse_bgm_with_ascii_dash() {
        let script = "[BGM: Outro music - fade-out, 3s]";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Bgm { duration_s, .. } => {
                assert_eq!(*duration_s, 3);
            }
            _ => panic!("Expected Bgm"),
        }
    }

    #[test]
    fn parse_pause() {
        let script = "[PAUSE: 2s]";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Pause { duration_s } => assert_eq!(*duration_s, 2),
            _ => panic!("Expected Pause"),
        }
    }

    #[test]
    fn parse_pause_without_s_suffix() {
        let script = "[PAUSE: 3]";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Pause { duration_s } => assert_eq!(*duration_s, 3),
            _ => panic!("Expected Pause"),
        }
    }

    #[test]
    fn parse_full_script() {
        let script = r#"# My Podcast

**Genre**: talk-show | **Duration**: ~5 min | **Speakers**: 2

| Character | Voice | Type |
|-----------|-------|------|
| Host | douwt | built-in |
| Guest | yangmi | built-in |

---

[BGM: Intro music — fade-in, 3s]

[Host - douwt, cheerful] Welcome to the show!

[Guest - yangmi, excited] Thanks for having me!

[PAUSE: 2s]

[Host - douwt, curious] What are you working on?

[Guest - yangmi, thoughtful] I'm researching AI voice synthesis.

[BGM: Outro — fade-out, 3s]
"#;
        let lines = parse_script(script);
        // Should have: 1 BGM + 4 dialogue + 1 pause + 1 BGM = 7
        assert_eq!(lines.len(), 7);

        // Check types in order
        assert!(matches!(&lines[0], ScriptLine::Bgm { .. }));
        assert!(matches!(&lines[1], ScriptLine::Dialogue { character, .. } if character == "Host"));
        assert!(
            matches!(&lines[2], ScriptLine::Dialogue { character, .. } if character == "Guest")
        );
        assert!(matches!(&lines[3], ScriptLine::Pause { duration_s: 2 }));
        assert!(matches!(&lines[4], ScriptLine::Dialogue { character, .. } if character == "Host"));
        assert!(
            matches!(&lines[5], ScriptLine::Dialogue { character, .. } if character == "Guest")
        );
        assert!(matches!(&lines[6], ScriptLine::Bgm { .. }));
    }

    #[test]
    fn parse_sequential_seg_ids() {
        let script = "[A - douwt, calm] Line one.\n[B - yangmi, calm] Line two.\n[C - douwt, calm] Line three.";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 3);
        if let ScriptLine::Dialogue { seg_id, .. } = &lines[0] {
            assert_eq!(*seg_id, 1);
        }
        if let ScriptLine::Dialogue { seg_id, .. } = &lines[1] {
            assert_eq!(*seg_id, 2);
        }
        if let ScriptLine::Dialogue { seg_id, .. } = &lines[2] {
            assert_eq!(*seg_id, 3);
        }
    }

    #[test]
    fn parse_skips_markdown_headers() {
        let script = "# Title\n## Subtitle\n**Bold text**\n---\n| table | row |\n[Host - douwt, calm] Actual dialogue.";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        assert!(matches!(&lines[0], ScriptLine::Dialogue { .. }));
    }

    #[test]
    fn parse_chinese_script() {
        let script = "[主持人 - yangmi, cheerful] 大家好，欢迎收听今天的节目！";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Dialogue {
                character, text, ..
            } => {
                assert_eq!(character, "主持人");
                assert_eq!(text, "大家好，欢迎收听今天的节目！");
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_empty_script() {
        let lines = parse_script("");
        assert!(lines.is_empty());
    }

    #[test]
    fn parse_report_collects_invalid_lines() {
        let script = "[Host - douwt, calm] Valid line.\nthis is not valid\n[PAUSE: 2s]";
        let report = parse_script_report(script);
        assert_eq!(report.lines.len(), 2);
        assert_eq!(report.invalid_lines, vec!["this is not valid".to_string()]);
    }

    #[test]
    fn parse_only_metadata() {
        let script = "# Title\n\n**Genre**: drama\n\n---\n";
        let lines = parse_script(script);
        assert!(lines.is_empty());
    }

    #[test]
    fn parse_mixed_clone_and_preset() {
        let script =
            "[A - douwt, calm] Preset voice.\n[B - clone:custom_voice, excited] Cloned voice.";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 2);
        match &lines[0] {
            ScriptLine::Dialogue {
                is_clone, voice, ..
            } => {
                assert!(!is_clone);
                assert_eq!(voice, "douwt");
            }
            _ => panic!("Expected Dialogue"),
        }
        match &lines[1] {
            ScriptLine::Dialogue {
                is_clone, voice, ..
            } => {
                assert!(*is_clone);
                assert_eq!(voice, "custom_voice");
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_dialogue_text_on_following_line() {
        let script = "[Host - douwt, cheerful]\nWelcome to the show!";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Dialogue {
                character, text, ..
            } => {
                assert_eq!(character, "Host");
                assert_eq!(text, "Welcome to the show!");
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_bold_wrapped_dialogue_markup() {
        let script = "**[杨幂 - clone:yangmi, 热情专业]** 大家好，欢迎收听节目。";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Dialogue {
                character,
                voice,
                is_clone,
                text,
                ..
            } => {
                assert_eq!(character, "杨幂");
                assert_eq!(voice, "yangmi");
                assert!(*is_clone);
                assert_eq!(text, "大家好，欢迎收听节目。");
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_known_speaker_repairs_generic_voice_aliases() {
        let script = "[杨幂 - nova, calm] 大家好。\n[窦文涛 - alloy, calm] 今天我们聊新闻。";
        let report = parse_script_report(script);
        assert!(report.invalid_lines.is_empty(), "{:?}", report.invalid_lines);
        assert_eq!(report.repair_summary.repaired_known_speaker_voices, 2);
        match &report.lines[0] {
            ScriptLine::Dialogue { voice, is_clone, .. } => {
                assert_eq!(voice, "yangmi");
                assert!(*is_clone);
            }
            _ => panic!("Expected Dialogue"),
        }
        match &report.lines[1] {
            ScriptLine::Dialogue { voice, is_clone, .. } => {
                assert_eq!(voice, "douwt");
                assert!(*is_clone);
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_dialogue_with_fullwidth_header_punctuation() {
        let script = "【Host — douwt， cheerful】 Hello world!";
        let lines = parse_script(script);
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            ScriptLine::Dialogue {
                character,
                voice,
                emotion,
                text,
                ..
            } => {
                assert_eq!(character, "Host");
                assert_eq!(voice, "douwt");
                assert_eq!(emotion, "cheerful");
                assert_eq!(text, "Hello world!");
            }
            _ => panic!("Expected Dialogue"),
        }
    }

    #[test]
    fn parse_generated_markdown_script_formats_used_in_production() {
        let script = r#"# 今日北京新闻播客 — 2026年4月15日

## 主持人
- 窦文涛（clone:douwentao）— 资深新闻评论员
- 杨幂（clone:yangmi）— 新闻主播

---

[窦文涛 - clone:douwentao, professional, enthusiastic]
听众朋友们大家好！这里是今日新闻播客。

**[杨幂 - clone:yangmi, 热情专业]** 大家好，欢迎收听节目。

> *本节目由AI辅助生成。*
"#;
        let report = parse_script_report(script);
        assert!(
            report.invalid_lines.is_empty(),
            "{:?}",
            report.invalid_lines
        );
        assert_eq!(report.lines.len(), 2);
        assert!(
            matches!(&report.lines[0], ScriptLine::Dialogue { character, .. } if character == "窦文涛")
        );
        assert!(
            matches!(&report.lines[1], ScriptLine::Dialogue { character, .. } if character == "杨幂")
        );
    }

    #[test]
    fn validate_and_fix_script_markdown_rewrites_common_llm_drift() {
        let script = "**【Host — douwt， cheerful】**\nHello world!";
        let prepared = validate_and_fix_script_markdown(script).unwrap();
        assert!(!prepared.repair_messages.is_empty());
        assert_eq!(
            prepared.repaired_markdown.as_deref(),
            Some("[Host - douwt, cheerful] Hello world!")
        );
        assert_eq!(prepared.lines.len(), 1);
    }

    #[test]
    fn validate_and_fix_script_markdown_rewrites_known_speaker_voice_aliases() {
        let script = "[杨幂 - nova, calm] 大家好。";
        let prepared = validate_and_fix_script_markdown(script).unwrap();
        assert!(
            prepared
                .repair_messages
                .iter()
                .any(|message| message.contains("known speaker voice alias"))
        );
        assert_eq!(
            prepared.repaired_markdown.as_deref(),
            Some("[杨幂 - clone:yangmi, calm] 大家好。")
        );
    }

    #[test]
    fn generate_podcast_accepts_multiline_dialogue_before_voice_validation() {
        let input = GenerateInput {
            script: Some("[Host - not_a_real_voice, calm]\nhello".to_string()),
            script_path: None,
            output_dir: Some(format!("/tmp/mofa-podcast-test-{}", timestamp())),
        };
        let err = generate_podcast(input).unwrap_err();
        assert!(err.contains("voice"), "{err}");
    }

    #[test]
    fn generate_podcast_persists_normalized_script_before_voice_validation() {
        let output_dir = PathBuf::from(format!("/tmp/mofa-podcast-test-{}", timestamp()));
        let input = GenerateInput {
            script: Some("**【Host — not_a_real_voice， calm】**\nhello".to_string()),
            script_path: None,
            output_dir: Some(output_dir.to_string_lossy().to_string()),
        };
        let err = generate_podcast(input).unwrap_err();
        assert!(err.contains("not found"), "{err}");
        assert!(err.contains("Normalized script:"));

        let repaired_script = std::fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("podcast_script_normalized_"))
                    .unwrap_or(false)
            })
            .expect("normalized script should exist");
        let content = std::fs::read_to_string(&repaired_script).unwrap();
        assert_eq!(content.trim(), "[Host - not_a_real_voice, calm] hello");

        let _ = std::fs::remove_dir_all(&output_dir);
    }

    // ── Emotion mapping tests ──────────────────────────────────────

    #[test]
    fn emotion_calm_returns_none() {
        assert!(emotion_to_control("calm").is_none());
    }

    #[test]
    fn emotion_excited_returns_control() {
        let p = emotion_to_control("excited");
        assert!(p.is_some());
        assert!(p.unwrap().contains("excited"));
    }

    #[test]
    fn emotion_case_insensitive() {
        assert!(emotion_to_control("EXCITED").is_some());
        assert!(emotion_to_control("Cheerful").is_some());
        assert!(emotion_to_control("  warm  ").is_some());
    }

    #[test]
    fn emotion_unknown_returns_none() {
        assert!(emotion_to_control("confused").is_none());
        assert!(emotion_to_control("").is_none());
    }

    #[test]
    fn all_documented_emotions_have_controls() {
        let emotions = [
            "excited",
            "serious",
            "warm",
            "angry",
            "sad",
            "cheerful",
            "dramatic",
            "curious",
            "thoughtful",
        ];
        for e in emotions {
            assert!(
                emotion_to_control(e).is_some(),
                "Missing control for '{e}'"
            );
        }
    }

    // ── Audio helper tests ─────────────────────────────────────────

    #[test]
    fn pcm_to_wav_header() {
        let pcm = vec![0u8; 100];
        let wav = pcm_to_wav(&pcm, 24000);
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(wav.len(), 44 + 100);
    }

    #[test]
    fn pcm_to_wav_sample_rate() {
        let wav = pcm_to_wav(&[], 24000);
        let sr = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(sr, 24000);
    }

    #[test]
    fn generate_silence_correct_length() {
        let wav = generate_silence_wav(1000); // 1 second
                                              // 24000 samples/sec * 2 bytes/sample = 48000 bytes PCM + 44 header
        assert_eq!(wav.len(), 48000 + 44);
    }

    #[test]
    fn generate_silence_400ms() {
        let wav = generate_silence_wav(400);
        // 24000 * 0.4 = 9600 samples * 2 bytes = 19200 + 44 header
        assert_eq!(wav.len(), 19200 + 44);
    }

    #[test]
    fn generate_silence_is_zeros() {
        let wav = generate_silence_wav(100);
        // All PCM data after header should be zeros
        for &b in &wav[44..] {
            assert_eq!(b, 0);
        }
    }

    #[test]
    fn silence_is_not_meaningful_tts_audio() {
        let wav = generate_silence_wav(500);
        assert!(!has_meaningful_tts_audio(&wav));
    }

    #[test]
    fn append_trailing_silence_extends_wav_duration() {
        let wav = pcm_to_wav(&vec![1u8; 4800], 24000); // 100ms mono PCM
        let padded = append_trailing_silence_to_wav(&wav, 250);
        assert_eq!(audio_duration_ms(&wav, 24000), 100);
        assert_eq!(audio_duration_ms(&padded, 24000), 350);
        let original = parse_wav_metadata(&wav).unwrap();
        let padded_meta = parse_wav_metadata(&padded).unwrap();
        assert_eq!(&padded_meta.data[..original.data.len()], original.data);
    }

    #[test]
    fn infer_tts_language_detects_chinese_and_english() {
        assert_eq!(
            infer_tts_language("大家好，欢迎收听节目"),
            Some(TtsLanguage::Chinese)
        );
        assert_eq!(
            infer_tts_language("Hello and welcome to the show"),
            Some(TtsLanguage::English)
        );
    }

    #[test]
    fn normalize_tts_text_adds_terminal_punctuation() {
        assert_eq!(
            normalize_tts_text("大家好，欢迎收听节目", Some(TtsLanguage::Chinese)),
            "大家好，欢迎收听节目。"
        );
        assert_eq!(
            normalize_tts_text("Hello and welcome", Some(TtsLanguage::English)),
            "Hello and welcome."
        );
    }

    #[test]
    fn normalize_tts_text_preserves_existing_terminal_punctuation() {
        assert_eq!(
            normalize_tts_text("大家好，欢迎收听节目。", Some(TtsLanguage::Chinese)),
            "大家好，欢迎收听节目。"
        );
        assert_eq!(
            normalize_tts_text("Hello and welcome!", Some(TtsLanguage::English)),
            "Hello and welcome!"
        );
        assert_eq!(
            normalize_tts_text("“欢迎收听节目。”", Some(TtsLanguage::Chinese)),
            "“欢迎收听节目。”"
        );
    }

    #[test]
    fn sanitize_filename_component_strips_path_characters() {
        assert_eq!(
            sanitize_filename_component("../../yangmi:demo"),
            "yangmi_demo"
        );
        assert_eq!(sanitize_filename_component("voice/name"), "voice_name");
    }

    #[test]
    fn segment_file_path_is_kept_under_segments_dir() {
        let seg_dir = PathBuf::from("/tmp/mofa-podcast-test-segments");
        let path = segment_file_path(&seg_dir, "../../escape", 7);
        assert_eq!(path, seg_dir.join("seg_007_escape.wav"));
    }

    #[test]
    fn placeholder_paths_are_unique_per_line() {
        let seg_dir = PathBuf::from("/tmp/mofa-podcast-test-segments");
        let a = placeholder_file_path(&seg_dir, "pause_line", 1);
        let b = placeholder_file_path(&seg_dir, "pause_line", 2);
        assert_ne!(a, b);
    }

    #[test]
    fn concat_fallback_rejects_wrong_wav_format() {
        let wav = pcm_to_wav(&vec![0u8; 200], 22050);
        let err = extract_pcm_for_concat(&wav, "bad.wav").unwrap_err();
        assert!(err.contains("24kHz mono 16-bit PCM"));
    }

    #[test]
    fn generate_podcast_rejects_unknown_voice_before_network_work() {
        let input = GenerateInput {
            script: Some("[Host - not_a_real_voice, calm] hello".to_string()),
            script_path: None,
            output_dir: Some(format!("/tmp/mofa-podcast-test-{}", timestamp())),
        };
        let err = generate_podcast(input).unwrap_err();
        assert!(err.contains("not found"), "{err}");
    }

    #[test]
    fn generate_podcast_rejects_malformed_script_lines() {
        let input = GenerateInput {
            script: Some("[Host - douwt, calm] hello\nnot valid".to_string()),
            script_path: None,
            output_dir: Some(format!("/tmp/mofa-podcast-test-{}", timestamp())),
        };
        let err = generate_podcast(input).unwrap_err();
        assert!(err.contains("malformed non-metadata lines"));
    }

    // ── Voice grouping / ordering tests ────────────────────────────

    #[test]
    fn builtin_and_clone_separation() {
        let script = r#"[A - douwt, calm] Line 1.
[B - clone:custom, excited] Line 2.
[C - yangmi, serious] Line 3.
[D - clone:custom, warm] Line 4."#;
        let lines = parse_script(script);

        let mut builtin = Vec::new();
        let mut cloned = Vec::new();
        for line in &lines {
            if let ScriptLine::Dialogue {
                is_clone,
                voice,
                seg_id,
                ..
            } = line
            {
                if *is_clone {
                    cloned.push((*seg_id, voice.clone()));
                } else {
                    builtin.push((*seg_id, voice.clone()));
                }
            }
        }
        assert_eq!(builtin.len(), 2);
        assert_eq!(cloned.len(), 2);
        assert_eq!(builtin[0].1, "douwt");
        assert_eq!(builtin[1].1, "yangmi");
        assert_eq!(cloned[0].1, "custom");
        assert_eq!(cloned[1].1, "custom");
    }

    #[test]
    fn voice_grouping_sort_order() {
        // Simulate the sorting logic from handle_generate
        let mut segments = vec![
            (3u32, "douwt".to_string()),
            (1, "yangmi".to_string()),
            (5, "douwt".to_string()),
            (2, "yangmi".to_string()),
        ];
        segments.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        // "douwt" < "yangmi", so douwts first, then yangmis
        assert_eq!(segments[0], (3, "douwt".to_string()));
        assert_eq!(segments[1], (5, "douwt".to_string()));
        assert_eq!(segments[2], (1, "yangmi".to_string()));
        assert_eq!(segments[3], (2, "yangmi".to_string()));
    }
}

// ── Main ───────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        fail("Usage: mofa-podcast <tool_name>  (podcast_voices | podcast_generate)");
    }

    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        input = "{}".to_string();
    }

    match args[1].as_str() {
        "podcast_voices" => handle_voices(&input),
        "podcast_generate" => handle_generate(&input),
        "podcast_voice_save" => handle_save_voice(&input),
        other => fail(&format!(
            "Unknown tool: {other}. Available: podcast_voices, podcast_generate, podcast_voice_save"
        )),
    }
}
