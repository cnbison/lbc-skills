//! MoFA FM: Voice management and TTS with custom voice cloning.
//!
//! Protocol: `./main <tool_name>` with JSON on stdin, JSON on stdout.
//! Requires OMINIX_API_URL and OCTOS_DATA_DIR environment variables.

use std::collections::BTreeMap;
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;

// ── Preset speakers (cannot be overwritten) ──────────────────────────

const PRESET_VOICES: &[&str] = &[
    "vivian", "serena", "ryan", "aiden", "eric", "dylan", "uncle_fu", "ono_anna", "sohee",
];

// ── Input types ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TtsInput {
    text: String,
    #[serde(default)]
    voice: Option<String>,
    #[serde(default)]
    output_path: Option<String>,
    #[serde(default)]
    language: Option<String>,
    /// Style/emotion prompt (e.g. "用兴奋激动的语气说话，充满热情和活力")
    #[serde(default)]
    prompt: Option<String>,
    /// Speed factor: >1.0 = faster, <1.0 = slower (0.5-2.0)
    #[serde(default)]
    speed: Option<f32>,
}

#[derive(Deserialize)]
struct VoiceSaveInput {
    name: String,
    audio_path: String,
}

#[derive(Deserialize)]
struct VoiceDeleteInput {
    name: String,
}

// ── Voice registry ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
struct VoiceRegistry {
    #[serde(default)]
    default_voice: Option<String>,
    #[serde(default)]
    voices: BTreeMap<String, VoiceEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VoiceEntry {
    file: String,
    #[serde(default)]
    created: Option<String>,
}

fn voices_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OCTOS_VOICE_DIR") {
        return PathBuf::from(dir);
    }
    // Match voice platform skill: $OCTOS_DATA_DIR/voice_profiles
    let data_dir = std::env::var("OCTOS_DATA_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(data_dir).join("voice_profiles")
}

fn registry_path() -> PathBuf {
    voices_dir()
        .parent()
        .unwrap_or(Path::new("/tmp"))
        .join("voices.json")
}

fn load_registry() -> VoiceRegistry {
    let path = registry_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        VoiceRegistry::default()
    }
}

fn save_registry(reg: &VoiceRegistry) {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(data) = serde_json::to_string_pretty(reg) {
        std::fs::write(&path, data).ok();
    }
}

/// Resolve a voice name: returns Some(wav_path) for custom voices, None for presets.
fn resolve_custom_voice(name: &str) -> Option<PathBuf> {
    let reg = load_registry();
    if let Some(entry) = reg.voices.get(name) {
        let path = voices_dir().join(&entry.file);
        if path.exists() {
            return Some(path);
        }
    }
    // Try direct file lookup in voices dir (e.g. <name>.wav without registry entry)
    let direct = voices_dir().join(format!("{name}.wav"));
    if direct.exists() {
        return Some(direct);
    }
    None
}

fn is_preset(name: &str) -> bool {
    PRESET_VOICES.contains(&name.to_lowercase().as_str())
}

fn is_wav_file(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

fn normalize_reference_audio_to_wav(src: &Path, dest: &Path) -> Result<(), String> {
    if is_wav_file(src) {
        std::fs::copy(src, dest)
            .map(|_| ())
            .map_err(|e| format!("Failed to copy WAV audio file: {e}"))?;
        return Ok(());
    }

    let ffmpeg = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &src.to_string_lossy(),
            "-vn",
            "-acodec",
            "pcm_s16le",
            "-ar",
            "24000",
            "-ac",
            "1",
            &dest.to_string_lossy(),
        ])
        .output();

    if let Ok(output) = ffmpeg {
        if output.status.success() && is_wav_file(dest) {
            return Ok(());
        }
        let _ = std::fs::remove_file(dest);
    }

    let afconvert = std::process::Command::new("afconvert")
        .args([
            "-f",
            "WAVE",
            "-d",
            "LEI16@24000",
            &src.to_string_lossy(),
            &dest.to_string_lossy(),
        ])
        .output();

    if let Ok(output) = afconvert {
        if output.status.success() && is_wav_file(dest) {
            return Ok(());
        }
        let _ = std::fs::remove_file(dest);
    }

    Err(
        "Failed to convert audio to WAV. Neither ffmpeg nor afconvert produced a valid WAV file."
            .to_string(),
    )
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Resolve the ominix API base URL. Checks in order:
///   1. OMINIX_API_URL env var
///   2. ~/.ominix/api_url discovery file
///   3. Default http://localhost:9090
fn ominix_base_url() -> String {
    if let Ok(url) = std::env::var("OMINIX_API_URL") {
        return url.trim_end_matches('/').to_string();
    }
    if let Some(home) = std::env::var_os("HOME") {
        let discovery = Path::new(&home).join(".ominix").join("api_url");
        if let Ok(url) = std::fs::read_to_string(&discovery) {
            let url = url.trim();
            if !url.is_empty() {
                return url.trim_end_matches('/').to_string();
            }
        }
    }
    "http://localhost:9090".to_string()
}

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(120))
        // No request timeout and no tcp_keepalive — ominix-api is single-threaded (MLX),
        // so the server may go silent for 10-30s between sentence chunks while synthesizing.
        // tcp_keepalive would kill the connection during these silent gaps.
        .build()
        .expect("failed to build HTTP client")
}

/// Wrap raw PCM bytes (16-bit signed LE, mono) in a WAV header.
fn pcm_to_wav(pcm: &[u8], sample_rate: u32) -> Vec<u8> {
    let data_len = pcm.len() as u32;
    let file_len = 36 + data_len;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}

/// Try to convert WAV to MP3 using ffmpeg for smaller file size.
/// Returns the MP3 path on success, or the original WAV path if conversion is unavailable.
fn try_convert_to_mp3(wav_path: &str, mp3_path: &str) -> String {
    if !wav_path.ends_with(".wav") || wav_path == mp3_path {
        return wav_path.to_string();
    }
    let result = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            wav_path,
            "-codec:a",
            "libmp3lame",
            "-q:a",
            "2",
            mp3_path,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match result {
        Ok(status) if status.success() => {
            let _ = std::fs::remove_file(wav_path);
            mp3_path.to_string()
        }
        _ => wav_path.to_string(),
    }
}

fn default_tts_output_mp3_path(voice_tag: &str, text: &str) -> PathBuf {
    let text_preview: String = text
        .chars()
        .take(20)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_end_matches('_')
        .to_string();
    let filename = format!("{voice_tag}_{text_preview}_{}.mp3", timestamp());
    if let Ok(work_dir) = std::env::var("OCTOS_WORK_DIR") {
        let dir = Path::new(&work_dir);
        let _ = std::fs::create_dir_all(dir);
        return dir.join(filename);
    }
    match std::env::current_dir() {
        Ok(dir) => dir.join(filename),
        Err(_) => PathBuf::from(format!("/tmp/{filename}")),
    }
}

fn resolve_tts_output_paths(
    requested_output: Option<String>,
    voice_tag: &str,
    text: &str,
) -> (String, String) {
    let final_path = requested_output
        .map(PathBuf::from)
        .unwrap_or_else(|| default_tts_output_mp3_path(voice_tag, text));

    let wav_path = if final_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"))
    {
        final_path.clone()
    } else {
        final_path.with_extension("wav")
    };

    (
        wav_path.to_string_lossy().to_string(),
        final_path.to_string_lossy().to_string(),
    )
}

/// Call TTS endpoint, handle both streaming PCM and WAV responses.
fn fetch_tts_wav(
    client: &reqwest::blocking::Client,
    url: &str,
    body: &serde_json::Value,
) -> Result<Vec<u8>, String> {
    let resp = client
        .post(url)
        .timeout(Duration::from_secs(1800))
        .json(body)
        .send()
        .map_err(|e| {
            // Print full error chain for debugging
            let mut msg = format!("TTS request failed: {e}");
            let mut source = e.source();
            while let Some(cause) = source {
                msg.push_str(&format!(" caused by: {cause}"));
                source = cause.source();
            }
            msg
        })?;

    let status = resp.status();
    if !status.is_success() {
        let resp_text = resp.text().unwrap_or_default();
        return Err(format!(
            "TTS error (HTTP {status}): {}",
            truncate(&resp_text, 200)
        ));
    }

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let bytes = resp
        .bytes()
        .map_err(|e| format!("Failed to read TTS response: {e}"))?;

    if bytes.is_empty() {
        return Err("TTS returned empty response".to_string());
    }

    // If already WAV, pass through
    if content_type.contains("wav") || (bytes.len() >= 4 && &bytes[..4] == b"RIFF") {
        return Ok(bytes.to_vec());
    }

    // Raw PCM → wrap in WAV header (24kHz, 16-bit, mono)
    Ok(pcm_to_wav(&bytes, 24000))
}

/// Minimum ominix-api version required for model-specific endpoints.
const MIN_OMINIX_VERSION: &str = "0.1.0";

fn check_health(client: &reqwest::blocking::Client, base_url: &str) -> Result<(), String> {
    // Generous timeout: ominix-api is single-threaded (MLX), so /health may block
    // while a TTS synthesis is in progress. 60s avoids false "not running" errors.
    match client
        .get(format!("{base_url}/health"))
        .timeout(Duration::from_secs(60))
        .send()
    {
        Ok(resp) if resp.status().is_success() => {
            // Check version from health response
            if let Ok(body) = resp.json::<serde_json::Value>() {
                if let Some(version) = body.get("version").and_then(|v| v.as_str()) {
                    if !version_gte(version, MIN_OMINIX_VERSION) {
                        return Err(format!(
                            "ominix-api {version} is too old (need >= {MIN_OMINIX_VERSION}).\n\
                             Upgrade: cargo install --git https://github.com/OminiX-ai/OminiX-MLX ominix-api --features tts --force"
                        ));
                    }
                } else {
                    eprintln!(
                        "Warning: ominix-api at {base_url} does not report version. \
                         Consider upgrading for prompt/speed support."
                    );
                }
            }
            Ok(())
        }
        Ok(resp) => Err(format!(
            "ominix-api returned HTTP {} at {base_url}. Check server logs.",
            resp.status()
        )),
        Err(_) => {
            // Check if the binary is installed at all
            let installed = std::process::Command::new("which")
                .arg("ominix-api")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !installed {
                Err(
                    "ominix-api is not installed. Install it:\n\
                     cargo install --git https://github.com/OminiX-ai/OminiX-MLX ominix-api --features tts\n\
                     Then start: ominix-api --tts-port 8082 --clone-port 8083"
                        .to_string(),
                )
            } else {
                Err(format!(
                    "ominix-api is installed but not running at {base_url}.\n\
                     Start it: ominix-api --tts-port 8082 --clone-port 8083"
                ))
            }
        }
    }
}

/// Simple semver comparison: is `have` >= `need`?
/// Strips build metadata (+hash) before comparing.
fn version_gte(have: &str, need: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        let base = s
            .split('+')
            .next()
            .unwrap_or(s)
            .split('-')
            .next()
            .unwrap_or(s);
        base.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let h = parse(have);
    let n = parse(need);
    for i in 0..n.len().max(h.len()) {
        let a = h.get(i).copied().unwrap_or(0);
        let b = n.get(i).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    true // equal
}

fn fail(msg: &str) -> ! {
    let out = json!({"output": msg, "success": false});
    println!("{out}");
    std::process::exit(1);
}

fn succeed(msg: &str) -> ! {
    let out = json!({"output": msg, "success": true});
    println!("{out}");
    std::process::exit(0);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end: String = s.chars().take(max).collect();
        format!("{end}...")
    }
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn now_iso() -> String {
    // Simple ISO-ish timestamp without chrono dependency
    let secs = timestamp();
    format!("{secs}")
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

// ── fm_tts ───────────────────────────────────────────────────────────

fn handle_tts(input_json: &str) {
    let input: TtsInput = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => fail(&format!("Invalid input: {e}")),
    };

    if input.text.trim().is_empty() {
        fail("'text' must not be empty");
    }

    let client = http_client();
    let base_url = ominix_base_url();
    if let Err(e) = check_health(&client, &base_url) {
        fail(&e);
    }

    let voice_tag = input.voice.as_deref().unwrap_or("default");
    let (wav_output_path, requested_output_path) =
        resolve_tts_output_paths(input.output_path.clone(), voice_tag, &input.text);

    if let Some(parent) = Path::new(&wav_output_path).parent() {
        if !parent.exists() {
            fail(&format!(
                "Output directory does not exist: {}",
                parent.display()
            ));
        }
    }

    let language = input.language.unwrap_or_else(|| "chinese".to_string());

    // Resolve voice: check custom registry first, then fall back to preset
    let voice_name = input.voice.unwrap_or_else(|| {
        let reg = load_registry();
        reg.default_voice.unwrap_or_else(|| "vivian".to_string())
    });

    // Validate: must be a known custom voice or a preset
    if resolve_custom_voice(&voice_name).is_none() && !is_preset(&voice_name) {
        let presets = PRESET_VOICES.join(", ");
        let reg = load_registry();
        let custom: Vec<&str> = reg.voices.keys().map(|s| s.as_str()).collect();
        let custom_list = if custom.is_empty() {
            String::new()
        } else {
            format!("\nCustom voices: {}", custom.join(", "))
        };
        fail(&format!(
            "Unknown voice '{voice_name}'. Available presets: {presets}{custom_list}\n\
             To use a custom voice, first save it with fm_voice_save."
        ));
    }

    let wav_bytes = if let Some(ref_path) = resolve_custom_voice(&voice_name) {
        // Custom voice → /v1/audio/tts/clone (multipart with raw WAV)
        let ref_bytes = match std::fs::read(&ref_path) {
            Ok(b) => b,
            Err(e) => fail(&format!("Failed to read voice '{}': {e}", voice_name)),
        };
        use reqwest::blocking::multipart::{Form, Part};
        let mut form = Form::new()
            .text("input", input.text.clone())
            .text("language", language.clone())
            .part(
                "reference_audio",
                Part::bytes(ref_bytes)
                    .file_name("ref.wav")
                    .mime_str("audio/wav")
                    .unwrap(),
            );
        if let Some(speed) = input.speed {
            form = form.text("speed", speed.to_string());
        }
        if let Some(ref prompt) = input.prompt {
            form = form.text("prompt", prompt.clone());
        }
        let endpoint = format!("{base_url}/v1/audio/tts/clone");
        let resp = match client
            .post(&endpoint)
            .timeout(Duration::from_secs(1800))
            .multipart(form)
            .send()
        {
            Ok(r) => r,
            Err(e) => fail(&format!("Clone request failed: {e}")),
        };
        let status = resp.status();
        if !status.is_success() {
            let t = resp.text().unwrap_or_default();
            fail(&format!(
                "Clone error (HTTP {status}): {}",
                truncate(&t, 200)
            ));
        }
        let bytes = match resp.bytes() {
            Ok(b) => b.to_vec(),
            Err(e) => fail(&format!("Failed to read clone response: {e}")),
        };
        // Wrap raw PCM in WAV header if needed (streaming mode returns PCM, not WAV)
        if bytes.len() >= 4 && &bytes[..4] == b"RIFF" {
            bytes
        } else {
            pcm_to_wav(&bytes, 24000)
        }
    } else {
        // Preset voice → /v1/audio/tts/qwen3 (JSON)
        let mut body = json!({
            "input": input.text,
            "voice": voice_name,
            "language": language
        });
        if let Some(ref prompt) = input.prompt {
            body["prompt"] = json!(prompt);
        }
        if let Some(speed) = input.speed {
            body["speed"] = json!(speed);
        }
        let endpoint = format!("{base_url}/v1/audio/tts/qwen3");
        match fetch_tts_wav(&client, &endpoint, &body) {
            Ok(b) => b,
            Err(e) => fail(&e),
        }
    };

    if let Err(e) = std::fs::write(&wav_output_path, &wav_bytes) {
        fail(&format!("Failed to write {wav_output_path}: {e}"));
    }

    let duration_secs = wav_bytes.len().saturating_sub(44) as f64 / 48000.0;
    let final_path = if requested_output_path.ends_with(".wav") {
        wav_output_path.clone()
    } else {
        try_convert_to_mp3(&wav_output_path, &requested_output_path)
    };
    let voice_label = if resolve_custom_voice(&voice_name).is_some() {
        format!("{voice_name} (custom)")
    } else {
        voice_name
    };

    // Output files_to_send so the agent auto-delivers to the user.
    // Don't include file path in output — prevents LLM from also calling send_file.
    let out = json!({
        "output": format!("Audio generated and sent to user ({duration_secs:.1}s, voice: {voice_label})."),
        "success": true,
        "files_to_send": [&final_path]
    });
    println!("{out}");
    std::process::exit(0);
}

// ── fm_voice_save ────────────────────────────────────────────────────

fn handle_voice_save(input_json: &str) {
    let input: VoiceSaveInput = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => fail(&format!("Invalid input: {e}")),
    };

    let name = input.name.to_lowercase();

    if !is_valid_name(&name) {
        fail("Voice name must be 1-64 characters, alphanumeric/underscore/dash only");
    }

    if is_preset(&name) {
        fail(&format!(
            "Cannot use '{name}' — it's a preset voice name. Choose a different name."
        ));
    }

    let src = Path::new(&input.audio_path);
    if !src.exists() {
        fail(&format!("Audio file not found: {}", input.audio_path));
    }
    if !src.is_file() {
        fail(&format!("Not a file: {}", input.audio_path));
    }
    if let Ok(meta) = std::fs::metadata(src) {
        if meta.len() == 0 {
            fail("Audio file is empty (0 bytes)");
        }
        if meta.len() > 50_000_000 {
            fail("Audio file too large (>50MB). Use a 3-10 second clip.");
        }
    }

    // Create voices directory
    let dir = voices_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        fail(&format!("Failed to create voices directory: {e}"));
    }

    // Normalize reference audio to a real WAV file so uploaded MP3/M4A/OGG
    // samples work reliably with downstream voice cloning.
    let filename = format!("{name}.wav");
    let dest = dir.join(&filename);
    if let Err(e) = normalize_reference_audio_to_wav(src, &dest) {
        fail(&e);
    }

    // Update registry
    let mut reg = load_registry();
    reg.voices.insert(
        name.clone(),
        VoiceEntry {
            file: filename,
            created: Some(now_iso()),
        },
    );
    save_registry(&reg);

    succeed(&format!(
        "Voice '{name}' saved successfully. Use it with fm_tts by setting voice to '{name}'."
    ));
}

// ── fm_voice_list ────────────────────────────────────────────────────

fn handle_voice_list(_input_json: &str) {
    let reg = load_registry();

    let mut output = String::from("**Preset voices:**\n");
    for v in PRESET_VOICES {
        output.push_str(&format!("  - {v}\n"));
    }

    if reg.voices.is_empty() {
        output.push_str("\n**Custom voices:** (none saved)\n");
    } else {
        output.push_str(&format!("\n**Custom voices ({}):**\n", reg.voices.len()));
        for (name, entry) in &reg.voices {
            let path = voices_dir().join(&entry.file);
            let exists = if path.exists() { "" } else { " [file missing]" };
            output.push_str(&format!("  - {name}{exists}\n"));
        }
    }

    // Show wav files in voices dir that aren't in the registry
    let vdir = voices_dir();
    if vdir.is_dir() {
        let on_disk: Vec<String> = std::fs::read_dir(&vdir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.strip_suffix(".wav").map(|n| n.to_string())
            })
            .filter(|n| !reg.voices.contains_key(n))
            .collect();
        if !on_disk.is_empty() {
            output.push_str(&format!("\n**Saved voices ({}):**\n", on_disk.len()));
            for name in &on_disk {
                output.push_str(&format!("  - {name}\n"));
            }
        }
    }

    if let Some(ref default) = reg.default_voice {
        output.push_str(&format!("\n**Default voice:** {default}"));
    }

    succeed(&output);
}

#[cfg(test)]
mod tests {
    use super::{resolve_tts_output_paths, try_convert_to_mp3};

    #[test]
    fn requested_mp3_uses_distinct_temp_wav() {
        let (wav_path, final_path) =
            resolve_tts_output_paths(Some("/tmp/sample.mp3".to_string()), "serena", "hello world");
        assert_eq!(final_path, "/tmp/sample.mp3");
        assert_eq!(wav_path, "/tmp/sample.wav");
    }

    #[test]
    fn requested_wav_keeps_same_output_path() {
        let (wav_path, final_path) =
            resolve_tts_output_paths(Some("/tmp/sample.wav".to_string()), "serena", "hello");
        assert_eq!(final_path, "/tmp/sample.wav");
        assert_eq!(wav_path, "/tmp/sample.wav");
    }

    #[test]
    fn mp3_conversion_refuses_in_place_non_wav_inputs() {
        assert_eq!(
            try_convert_to_mp3("/tmp/sample.mp3", "/tmp/sample.mp3"),
            "/tmp/sample.mp3"
        );
    }
}

// ── fm_voice_delete ──────────────────────────────────────────────────

fn handle_voice_delete(input_json: &str) {
    let input: VoiceDeleteInput = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => fail(&format!("Invalid input: {e}")),
    };

    let name = input.name.to_lowercase();

    if is_preset(&name) {
        fail(&format!("Cannot delete preset voice '{name}'"));
    }

    let mut reg = load_registry();

    if let Some(entry) = reg.voices.remove(&name) {
        // Delete the audio file
        let path = voices_dir().join(&entry.file);
        if path.exists() {
            std::fs::remove_file(&path).ok();
        }

        // Clear default if it was this voice
        if reg.default_voice.as_deref() == Some(&name) {
            reg.default_voice = None;
        }

        save_registry(&reg);
        succeed(&format!("Voice '{name}' deleted."));
    } else {
        fail(&format!(
            "Custom voice '{name}' not found. Use fm_voice_list to see available voices."
        ));
    }
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tool_name = args.get(1).map(|s| s.as_str()).unwrap_or("unknown");

    let mut buf = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
        fail(&format!("Failed to read stdin: {e}"));
    }

    match tool_name {
        "fm_tts" => handle_tts(&buf),
        "fm_voice_save" => handle_voice_save(&buf),
        "fm_voice_list" => handle_voice_list(&buf),
        "fm_voice_delete" => handle_voice_delete(&buf),
        _ => fail(&format!(
            "Unknown tool '{tool_name}'. Expected: fm_tts, fm_voice_save, fm_voice_list, fm_voice_delete"
        )),
    }
}
