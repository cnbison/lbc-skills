// SPDX-License-Identifier: Apache-2.0

use crate::config::MofaConfig;
use crate::gemini::{BatchImageRequest, GeminiClient};
use crate::style::Style;
use crate::veo::VeoClient;
use eyre::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Input card definition for video pipeline (from JSON).
#[derive(Deserialize, Debug)]
pub struct VideoCardInput {
    pub name: String,
    pub prompt: String,
    pub style: Option<String>,
    pub anim_style: Option<String>,
    pub anim_desc: Option<String>,
}

/// Animate a single card image into a video with BGM.
#[allow(clippy::too_many_arguments)]
fn animate_card(
    veo: &VeoClient,
    image_path: &Path,
    out_path: &Path,
    anim_prompt: &str,
    bgm_path: Option<&Path>,
    still_duration: f64,
    crossfade_dur: f64,
    fade_out_dur: f64,
    music_volume: f64,
    music_fade_in: f64,
    label: &str,
) -> Result<PathBuf> {
    let dir = out_path.parent().unwrap();
    let base_name = image_path.file_stem().unwrap().to_string_lossy();

    // Step 1: Generate video with Veo
    let raw_video = dir.join(format!("{base_name}-raw.mp4"));
    veo.generate_video(image_path, anim_prompt, &raw_video, None)?;

    // Get raw video duration & dimensions via ffprobe
    let dur_output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(&raw_video)
        .output()?;
    let raw_dur: f64 = String::from_utf8_lossy(&dur_output.stdout)
        .trim()
        .parse()
        .unwrap_or(5.0);

    let dims_output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0",
        ])
        .arg(&raw_video)
        .output()?;
    let dims_str = String::from_utf8_lossy(&dims_output.stdout);
    let dims: Vec<&str> = dims_str.trim().split(',').collect();
    let width = dims.first().unwrap_or(&"1280");
    let height = dims.get(1).unwrap_or(&"720");

    let total_dur = still_duration + raw_dur;
    let fade_out_start = total_dur - fade_out_dur;

    eprintln!("  [{label}] Compositing ({raw_dur:.1}s anim + {still_duration}s still)...");

    // Step 2a: Still image clip
    let still_clip = dir.join(format!("{base_name}-still.mp4"));
    Command::new("ffmpeg")
        .args(["-y", "-loop", "1", "-i"])
        .arg(image_path)
        .args(["-t", &still_duration.to_string()])
        .args([
            "-vf",
            &format!("scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2,setsar=1"),
        ])
        .args(["-c:v", "libx264", "-preset", "medium", "-crf", "20", "-r", "24", "-pix_fmt", "yuv420p", "-an"])
        .arg(&still_clip)
        .output()?;

    // Step 2b: Re-encode animation
    let anim_clip = dir.join(format!("{base_name}-anim.mp4"));
    Command::new("ffmpeg")
        .args(["-y", "-i"])
        .arg(&raw_video)
        .args([
            "-c:v", "libx264", "-preset", "medium", "-crf", "20", "-r", "24", "-pix_fmt",
            "yuv420p", "-an",
        ])
        .arg(&anim_clip)
        .output()?;

    // Step 2c: Crossfade still → animation + fade out
    let no_audio = dir.join(format!("{base_name}-noaudio.mp4"));
    let xfade_offset = still_duration - crossfade_dur;
    Command::new("ffmpeg")
        .args(["-y", "-i"])
        .arg(&still_clip)
        .args(["-i"])
        .arg(&anim_clip)
        .args([
            "-filter_complex",
            &format!(
                "[0:v][1:v]xfade=transition=fade:duration={crossfade_dur}:offset={xfade_offset},fade=t=out:st={fade_out_start}:d={fade_out_dur}[v]"
            ),
            "-map", "[v]",
            "-c:v", "libx264", "-preset", "medium", "-crf", "20", "-movflags", "+faststart", "-an",
        ])
        .arg(&no_audio)
        .output()?;

    // Step 2d: Add music
    let has_music = bgm_path.map(|p| p.exists()).unwrap_or(false);
    if has_music {
        let bgm = bgm_path.unwrap();
        Command::new("ffmpeg")
            .args(["-y", "-i"])
            .arg(&no_audio)
            .args(["-i"])
            .arg(bgm)
            .args([
                "-filter_complex",
                &format!(
                    "[1:a]afade=t=in:d={music_fade_in},afade=t=out:st={fade_out_start}:d={fade_out_dur},volume={music_volume}[a]"
                ),
                "-map", "0:v", "-map", "[a]",
                "-c:v", "copy", "-c:a", "aac", "-b:a", "128k", "-shortest", "-movflags", "+faststart",
            ])
            .arg(out_path)
            .output()?;

        // Cleanup temp files
        for f in [&still_clip, &anim_clip, &no_audio] {
            std::fs::remove_file(f).ok();
        }
    } else {
        std::fs::rename(&no_audio, out_path)?;
        for f in [&still_clip, &anim_clip] {
            std::fs::remove_file(f).ok();
        }
    }

    let size = std::fs::metadata(out_path)?.len();
    eprintln!(
        "  [{label}] Done: {} ({:.1}MB)",
        out_path.display(),
        size as f64 / 1024.0 / 1024.0
    );
    Ok(out_path.to_path_buf())
}

#[allow(clippy::too_many_arguments)]
fn gen_video_images_sync(
    gemini: &GeminiClient,
    card_dir: &Path,
    cards: &[VideoCardInput],
    style: &Style,
    total: usize,
    model: &str,
    ar: &str,
    size: Option<&str>,
    concurrency: usize,
) -> Vec<Option<PathBuf>> {
    let img_paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .unwrap();

    pool.scope(|s| {
        for (idx, card) in cards.iter().enumerate() {
            let img_paths = Arc::clone(&img_paths);

            s.spawn(move |_| {
                let variant = card.style.as_deref().unwrap_or("front");
                let prefix = style.get_prompt(variant);
                let full_prompt = format!("{prefix}\n\n{}", card.prompt);
                let out_path = card_dir.join(format!("card-{}.png", card.name));

                if let Ok(Some(p)) = gemini.gen_image(
                    &full_prompt,
                    &out_path,
                    size,
                    Some(ar),
                    &[],
                    Some(model),
                    Some(&card.name),
                ) {
                    img_paths.lock().unwrap()[idx] = Some(p);
                }
            });
        }
    });

    let result = img_paths.lock().unwrap().clone();
    result
}

/// Video card pipeline: generate PNG cards → animate each → MP4 with BGM.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn run(
    card_dir: &Path,
    cards: &[VideoCardInput],
    style: &Style,
    anim_style: &Style,
    cfg: &MofaConfig,
    concurrency: usize,
    aspect_ratio: Option<&str>,
    image_size: Option<&str>,
    bgm_path: Option<&Path>,
    still_duration: f64,
    crossfade_dur: f64,
    fade_out_dur: f64,
    music_volume: f64,
    music_fade_in: f64,
    batch: bool,
) -> Result<(Vec<Option<PathBuf>>, Vec<Option<PathBuf>>)> {
    let gemini_key = cfg
        .gemini_key()
        .ok_or_else(|| eyre::eyre!("Gemini API key required"))?;
    let gemini = GeminiClient::new(gemini_key.clone());
    let veo = VeoClient::new(gemini_key);

    std::fs::create_dir_all(card_dir)?;
    let total = cards.len();
    let ar = aspect_ratio.unwrap_or("9:16");
    let size = image_size.or(cfg
        .defaults
        .cards
        .as_ref()
        .and_then(|c| c.image_size.as_deref()));
    let model = cfg.gen_model();

    // Phase 1: Generate all card images
    eprintln!("\n=== Phase 1: Generating {total} card images ===");

    let img_paths_vec = if batch {
        let requests: Vec<BatchImageRequest> = cards
            .iter()
            .map(|card| {
                let variant = card.style.as_deref().unwrap_or("front");
                let prefix = style.get_prompt(variant);
                BatchImageRequest {
                    key: card.name.clone(),
                    prompt: format!("{prefix}\n\n{}", card.prompt),
                    out_file: card_dir.join(format!("card-{}.png", card.name)),
                    image_size: size.map(String::from),
                    aspect_ratio: Some(ar.to_string()),
                    ref_images: vec![],
                    model: model.to_string(),
                }
            })
            .collect();
        match gemini.batch_gen_images(requests) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Batch failed ({e}), falling back to parallel sync...");
                gen_video_images_sync(
                    &gemini,
                    card_dir,
                    cards,
                    style,
                    total,
                    model,
                    ar,
                    size,
                    concurrency,
                )
            }
        }
    } else {
        gen_video_images_sync(
            &gemini,
            card_dir,
            cards,
            style,
            total,
            model,
            ar,
            size,
            concurrency,
        )
    };

    // Phase 2: Animate each card (sequential — Veo has rate limits)
    eprintln!("\n=== Phase 2: Animating {total} cards ===");
    let mut video_paths = Vec::new();

    for (i, card) in cards.iter().enumerate() {
        if let Some(ref img_path) = img_paths_vec[i] {
            let anim_variant = card.anim_style.as_deref().unwrap_or("shuimo");
            let anim_prompt = anim_style.get_prompt(anim_variant);
            let full_anim_prompt = if let Some(ref desc) = card.anim_desc {
                format!("{anim_prompt}\n\n{desc}")
            } else {
                anim_prompt.to_string()
            };

            let out_path = card_dir.join(format!("card-{}-animated.mp4", card.name));
            match animate_card(
                &veo,
                img_path,
                &out_path,
                &full_anim_prompt,
                bgm_path,
                still_duration,
                crossfade_dur,
                fade_out_dur,
                music_volume,
                music_fade_in,
                &card.name,
            ) {
                Ok(p) => video_paths.push(Some(p)),
                Err(e) => {
                    eprintln!("  [{}] Animation failed: {e}", card.name);
                    video_paths.push(None);
                }
            }
        } else {
            eprintln!("  [{}] Skipped (no image)", card.name);
            video_paths.push(None);
        }
    }

    let ok_img = img_paths_vec.iter().filter(|p| p.is_some()).count();
    let ok_vid = video_paths.iter().filter(|p| p.is_some()).count();
    eprintln!(
        "\nDone: {ok_img}/{total} images, {ok_vid}/{total} videos in {}/",
        card_dir.display()
    );
    Ok((img_paths_vec, video_paths))
}
