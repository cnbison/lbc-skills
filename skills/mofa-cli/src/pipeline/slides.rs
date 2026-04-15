// SPDX-License-Identifier: Apache-2.0

use crate::config::MofaConfig;
use crate::dashscope::DashscopeClient;
use crate::deepseek_ocr::DeepSeekOcrClient;
use crate::gemini::{BatchImageRequest, GeminiClient};
use crate::layout::{
    extract_text_layout, extract_text_layout_deepseek, refine_text_layout, ANTI_LEAK_RULES,
    NO_TEXT_INSTRUCTION, SH, SW,
};
use crate::pptx::{self, ImageOverlay, SlideData, TextOverlay};
use crate::style::Style;
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

fn cache_stamp_path(out_file: &Path) -> PathBuf {
    let filename = out_file
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());
    out_file.with_file_name(format!(".{filename}.mofa-cache.json"))
}

fn cacheable_file_exists(out_file: &Path) -> bool {
    out_file.exists()
        && out_file
            .metadata()
            .map(|m| m.len() > 10_000)
            .unwrap_or(false)
}

fn update_path_fingerprint(hasher: &mut Sha256, path: &Path) {
    hasher.update(path.to_string_lossy().as_bytes());
    match path.metadata() {
        Ok(meta) => {
            hasher.update(meta.len().to_le_bytes());
            if let Ok(modified) = meta.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    hasher.update(duration.as_secs().to_le_bytes());
                    hasher.update(duration.subsec_nanos().to_le_bytes());
                }
            }
        }
        Err(_) => hasher.update(b"missing"),
    }
}

fn hash_to_hex(hasher: Sha256) -> String {
    let bytes = hasher.finalize();
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn generation_fingerprint(
    prompt: &str,
    image_size: Option<&str>,
    model: &str,
    ref_images: &[&Path],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"kind=generation\n");
    hasher.update(prompt.as_bytes());
    hasher.update(b"\nmodel=");
    hasher.update(model.as_bytes());
    hasher.update(b"\nimage_size=");
    hasher.update(image_size.unwrap_or("").as_bytes());
    for ref_image in ref_images {
        hasher.update(b"\nref=");
        update_path_fingerprint(&mut hasher, ref_image);
    }
    hash_to_hex(hasher)
}

fn edit_fingerprint(source_image: &Path, prompt: &str, model: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"kind=edit\n");
    update_path_fingerprint(&mut hasher, source_image);
    hasher.update(b"\nprompt=");
    hasher.update(prompt.as_bytes());
    hasher.update(b"\nmodel=");
    hasher.update(model.as_bytes());
    hash_to_hex(hasher)
}

fn read_cache_stamp(out_file: &Path) -> Option<String> {
    let stamp_path = cache_stamp_path(out_file);
    let body = std::fs::read_to_string(stamp_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    json.get("fingerprint")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn write_cache_stamp(out_file: &Path, fingerprint: &str) -> Result<()> {
    let stamp_path = cache_stamp_path(out_file);
    let body = serde_json::json!({ "fingerprint": fingerprint });
    let bytes = serde_json::to_vec_pretty(&body)?;
    std::fs::write(stamp_path, bytes)?;
    Ok(())
}

fn invalidate_stale_cache(out_file: &Path, fingerprint: &str) {
    if !out_file.exists() {
        return;
    }
    let stamp_matches = read_cache_stamp(out_file)
        .map(|existing| existing == fingerprint)
        .unwrap_or(false);
    if cacheable_file_exists(out_file) && stamp_matches {
        return;
    }
    let _ = std::fs::remove_file(out_file);
    let _ = std::fs::remove_file(cache_stamp_path(out_file));
}

fn has_valid_cache(out_file: &Path, fingerprint: &str) -> bool {
    cacheable_file_exists(out_file)
        && read_cache_stamp(out_file)
            .map(|existing| existing == fingerprint)
            .unwrap_or(false)
}

/// Route image generation to Gemini or Dashscope based on model name.
/// Models starting with "qwen-image" go to Dashscope, everything else to Gemini.
#[allow(clippy::too_many_arguments)]
fn generate_image(
    gemini: &GeminiClient,
    dashscope: &Option<DashscopeClient>,
    prompt: &str,
    out_file: &Path,
    image_size: Option<&str>,
    ref_images: &[&Path],
    model: &str,
    label: &str,
) -> Option<PathBuf> {
    let fingerprint = generation_fingerprint(prompt, image_size, model, ref_images);
    invalidate_stale_cache(out_file, &fingerprint);
    if has_valid_cache(out_file, &fingerprint) {
        eprintln!("Cached: {label}");
        return Some(out_file.to_path_buf());
    }

    if model.starts_with("qwen-image") {
        if let Some(ref ds) = dashscope {
            match ds.gen_image(prompt, out_file, Some(model), image_size) {
                Ok(p) => {
                    let _ = write_cache_stamp(&p, &fingerprint);
                    return Some(p);
                }
                Err(e) => eprintln!("{label}: Dashscope gen failed ({e}), falling back to Gemini"),
            }
        }
    }

    // Gemini path (default or fallback)
    gemini
        .gen_image(
            prompt,
            out_file,
            image_size,
            Some("16:9"),
            ref_images,
            Some(model),
            Some(label),
        )
        .ok()
        .flatten()
        .inspect(|path| {
            let _ = write_cache_stamp(path, &fingerprint);
        })
}

/// Input slide definition (from JSON).
#[derive(Deserialize, Debug)]
pub struct SlideInput {
    pub prompt: String,
    pub style: Option<String>,
    pub texts: Option<Vec<TextOverlay>>,
    #[serde(default)]
    pub auto_layout: bool,
    pub images: Option<Vec<String>>,
    pub gen_model: Option<String>,
    /// Path to an existing image to use as-is (skip generation).
    /// VQA text extraction + Qwen-Edit text removal still apply when auto_layout=true.
    /// This enables PDF-to-PPTX conversion: provide original page images, extract text,
    /// remove text, overlay editable text.
    pub source_image: Option<String>,
    /// Images to overlay on the slide at specific positions (e.g. logos).
    pub overlay_images: Option<Vec<ImageOverlay>>,
}

#[derive(Serialize)]
struct SlidesRunManifest {
    version: u32,
    generated_at: String,
    slide_dir: String,
    out_file: String,
    slide_count: usize,
    slides: Vec<SlidesRunManifestSlide>,
}

#[derive(Serialize)]
struct SlidesRunManifestSlide {
    index: usize,
    filename: String,
    path: String,
}

fn write_run_manifest(
    slide_dir: &Path,
    out_file: &Path,
    final_paths: &[Option<PathBuf>],
) -> Result<()> {
    let manifest = SlidesRunManifest {
        version: 1,
        generated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        slide_dir: slide_dir.to_string_lossy().to_string(),
        out_file: out_file.to_string_lossy().to_string(),
        slide_count: final_paths.iter().filter(|path| path.is_some()).count(),
        slides: final_paths
            .iter()
            .enumerate()
            .filter_map(|(index, path)| {
                let path = path.as_ref()?;
                Some(SlidesRunManifestSlide {
                    index,
                    filename: path
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| format!("slide-{:02}.png", index + 1)),
                    path: path.to_string_lossy().to_string(),
                })
            })
            .collect(),
    };

    let manifest_path = slide_dir.join("manifest.json");
    let temp_path = slide_dir.join("manifest.json.tmp");
    let mut body = serde_json::to_vec_pretty(&manifest)?;
    body.push(b'\n');
    std::fs::write(&temp_path, body)?;
    std::fs::rename(&temp_path, &manifest_path)?;
    Ok(())
}

/// Sync fallback for slides pipeline (used when batch fails).
#[allow(clippy::too_many_arguments)]
fn run_slides_sync(
    slide_dir: &Path,
    out_file: &Path,
    slides: &[SlideInput],
    style: &Style,
    cfg: &MofaConfig,
    gemini: &GeminiClient,
    ocr_client: &Option<DeepSeekOcrClient>,
    dashscope: &Option<DashscopeClient>,
    total: usize,
    concurrency: usize,
    image_size: Option<&str>,
    gen_model: Option<&str>,
    ref_image_size: Option<&str>,
    vision_model: Option<&str>,
) -> Result<()> {
    let ref_paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));
    let extracted_texts: Arc<Mutex<Vec<Option<Vec<TextOverlay>>>>> =
        Arc::new(Mutex::new(vec![None; total]));
    let direct_paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()?;

    pool.scope(|s| {
        #[allow(clippy::redundant_locals)]
        for (idx, slide) in slides.iter().enumerate() {
            let gemini = gemini;
            let dashscope = dashscope;
            let ocr_client = ocr_client;
            let ref_paths = Arc::clone(&ref_paths);
            let extracted_texts = Arc::clone(&extracted_texts);
            let direct_paths = Arc::clone(&direct_paths);

            s.spawn(move |_| {
                let variant = slide.style.as_deref().unwrap_or("normal");
                let prefix = style.get_prompt(variant);
                let padded = format!("{:02}", idx + 1);
                let model = slide
                    .gen_model
                    .as_deref()
                    .or(gen_model)
                    .unwrap_or(cfg.gen_model());
                let ref_images: Vec<&Path> = slide
                    .images
                    .as_ref()
                    .map(|imgs| imgs.iter().map(|p| Path::new(p.as_str())).collect())
                    .unwrap_or_default();

                // VQA-first: if texts are provided OR auto_layout is on,
                // generate WITH text, extract layout, remove text, overlay.
                let use_vqa = slide.auto_layout;

                if use_vqa {
                    let ref_file = slide_dir.join(format!("slide-{padded}-ref.png"));
                    let ref_ready = if let Some(ref src) = slide.source_image {
                        let src_path = Path::new(src);
                        if src_path.exists() {
                            std::fs::copy(src_path, &ref_file).is_ok()
                        } else {
                            false
                        }
                    } else {
                        let full_prompt = format!("{prefix}\n\n{}{ANTI_LEAK_RULES}", slide.prompt);
                        let ref_size = ref_image_size.or(image_size);
                        generate_image(
                            gemini,
                            dashscope,
                            &full_prompt,
                            &ref_file,
                            ref_size,
                            &ref_images,
                            model,
                            &format!("Slide {} (ref)", idx + 1),
                        )
                        .is_some()
                    };

                    if ref_ready {
                        let extraction_result = if let Some(ref ocr) = ocr_client {
                            match extract_text_layout_deepseek(
                                ocr,
                                gemini,
                                &ref_file,
                                SW,
                                SH,
                                vision_model,
                            ) {
                                Ok(texts) if !texts.is_empty() => Ok((texts, true)),
                                _ => extract_text_layout(
                                    gemini,
                                    &ref_file,
                                    SW,
                                    SH,
                                    vision_model,
                                    Some(prefix),
                                )
                                .map(|t| (t, false)),
                            }
                        } else {
                            extract_text_layout(
                                gemini,
                                &ref_file,
                                SW,
                                SH,
                                vision_model,
                                Some(prefix),
                            )
                            .map(|t| (t, false))
                        };

                        match extraction_result {
                            Ok((texts, used_ocr)) => {
                                let texts = if !used_ocr {
                                    refine_text_layout(
                                        gemini,
                                        &ref_file,
                                        &texts,
                                        SW,
                                        SH,
                                        vision_model,
                                    )
                                    .unwrap_or(texts)
                                } else {
                                    texts
                                };
                                extracted_texts.lock().unwrap()[idx] = Some(texts);
                            }
                            Err(e) => eprintln!("Slide {}: text extraction failed — {e}", idx + 1),
                        }
                        ref_paths.lock().unwrap()[idx] = Some(ref_file);
                    }
                } else {
                    // Non-VQA mode: image-only OR clean background + manual text overlays
                    let mut full_prompt = format!("{prefix}\n\n{}{ANTI_LEAK_RULES}", slide.prompt);
                    if slide.texts.is_some() {
                        // Clean background mode (like cc-ppt): generate without text,
                        // user-provided texts will be overlaid as native PowerPoint boxes
                        full_prompt.push_str(NO_TEXT_INSTRUCTION);
                    }
                    let out_path = slide_dir.join(format!("slide-{padded}.png"));
                    if let Some(p) = generate_image(
                        gemini,
                        dashscope,
                        &full_prompt,
                        &out_path,
                        image_size,
                        &ref_images,
                        model,
                        &format!("Slide {}", idx + 1),
                    ) {
                        direct_paths.lock().unwrap()[idx] = Some(p);
                    }
                }
            });
        }
    });

    let ref_paths = ref_paths.lock().unwrap().clone();
    let direct_paths = direct_paths.lock().unwrap().clone();
    let mut final_paths: Vec<Option<PathBuf>> = vec![None; total];

    // Phase 3: Remove text from ref images (VQA-first slides need clean backgrounds)
    #[allow(clippy::needless_range_loop)]
    for idx in 0..total {
        let use_vqa = slides[idx].auto_layout;
        if !use_vqa {
            final_paths[idx] = direct_paths[idx].clone();
            continue;
        }
        let Some(ref ref_path) = ref_paths[idx] else {
            continue;
        };
        let padded = format!("{:02}", idx + 1);
        let out_path = slide_dir.join(format!("slide-{padded}.png"));

        if let Some(ref ds) = dashscope {
            match ds.refine_image(ref_path, "Remove all readable text, numbers, and punctuation from this image. Replace removed text with surrounding background. Keep all non-text elements.", &out_path, Some(cfg.edit_model())) {
                Ok(p) => final_paths[idx] = Some(p),
                Err(e) => eprintln!("Slide {}: Qwen-Edit failed ({e})", idx + 1),
            }
        }
    }

    let extracted = extracted_texts.lock().unwrap();
    let slide_data: Vec<SlideData> = (0..total)
        .map(|i| {
            let image_path = final_paths[i]
                .as_ref()
                .map(|p| p.to_string_lossy().to_string());
            // VQA-first: user-provided texts take priority, fall back to VQA-extracted
            let texts = if slides[i].texts.is_some() {
                // User provided explicit text overlays — use those
                slides[i].texts.clone().unwrap_or_default()
            } else if slides[i].auto_layout {
                // Auto-layout — use VQA-extracted texts
                extracted[i].clone().unwrap_or_default()
            } else {
                Vec::new()
            };
            let images = slides[i].overlay_images.clone().unwrap_or_default();
            SlideData {
                image_path,
                texts,
                images,
            }
        })
        .collect();

    write_run_manifest(slide_dir, out_file, &final_paths)?;
    pptx::build_pptx(&slide_data, out_file, SW, SH)?;
    let ok = final_paths.iter().filter(|p| p.is_some()).count();
    eprintln!(
        "\nDone: {out_file} ({ok}/{total} slides)",
        out_file = out_file.display()
    );
    Ok(())
}

/// Full slides pipeline: generate images + build multi-slide PPTX.
#[allow(clippy::too_many_arguments)]
pub fn run(
    slide_dir: &Path,
    out_file: &Path,
    slides: &[SlideInput],
    style: &Style,
    cfg: &MofaConfig,
    concurrency: usize,
    image_size: Option<&str>,
    gen_model: Option<&str>,
    ref_image_size: Option<&str>,
    vision_model: Option<&str>,
    _refine_with_qwen: bool,
    batch: bool,
) -> Result<()> {
    let gemini_key = cfg
        .gemini_key()
        .ok_or_else(|| eyre::eyre!("Gemini API key required"))?;
    let gemini = GeminiClient::new(gemini_key);

    // Build OCR client for grounded text extraction (precise bounding boxes).
    // When available: OCR+VQA mode. When absent: VQA-only mode.
    let ocr_client = match cfg.ocr_url() {
        Some(url) => {
            eprintln!("OCR enabled (OCR+VQA mode): {url}");
            Some(DeepSeekOcrClient::new(url))
        }
        None => {
            eprintln!("OCR not configured — using VQA-only mode");
            None
        }
    };

    // Build Dashscope client for Qwen-Edit text removal
    let dashscope = match cfg.dashscope_key() {
        Some(key) => {
            eprintln!("Dashscope enabled (Qwen-Edit)");
            Some(DashscopeClient::new(key))
        }
        None => {
            eprintln!("Warning: DASHSCOPE_API_KEY not set (needed for Qwen-Edit text removal)");
            None
        }
    };

    std::fs::create_dir_all(slide_dir)?;
    let total = slides.len();
    eprintln!(
        "Generating {total} slides ({}{concurrency} parallel)...",
        if batch { "batch + " } else { "" }
    );

    // Batch mode: pre-generate all images via Batch API, then extract text sequentially
    if batch {
        let mut batch_requests: Vec<(usize, BatchImageRequest, bool, String)> = Vec::new(); // (idx, req, is_auto_layout, fingerprint)

        for (idx, slide) in slides.iter().enumerate() {
            let variant = slide.style.as_deref().unwrap_or("normal");
            let prefix = style.get_prompt(variant);
            let padded = format!("{:02}", idx + 1);
            let model_name = slide
                .gen_model
                .as_deref()
                .or(gen_model)
                .unwrap_or(cfg.gen_model())
                .to_string();

            let ref_images: Vec<PathBuf> = slide
                .images
                .as_ref()
                .map(|imgs| imgs.iter().map(PathBuf::from).collect())
                .unwrap_or_default();

            // VQA-first: texts provided or auto_layout → generate WITH text
            let use_vqa = slide.auto_layout;

            if use_vqa {
                if slide.source_image.is_some() {
                    continue; // source_image slides don't need generation
                }
                let full_prompt = format!("{prefix}\n\n{}{ANTI_LEAK_RULES}", slide.prompt);
                let ref_size = ref_image_size.or(image_size);
                let out_file = slide_dir.join(format!("slide-{padded}-ref.png"));
                let fingerprint = generation_fingerprint(
                    &full_prompt,
                    ref_size,
                    &model_name,
                    &ref_images.iter().map(PathBuf::as_path).collect::<Vec<_>>(),
                );
                invalidate_stale_cache(&out_file, &fingerprint);
                batch_requests.push((
                    idx,
                    BatchImageRequest {
                        key: format!("slide-{padded}-ref"),
                        prompt: full_prompt,
                        out_file,
                        image_size: ref_size.map(String::from),
                        aspect_ratio: Some("16:9".to_string()),
                        ref_images,
                        model: model_name,
                    },
                    true,
                    fingerprint,
                ));
            } else {
                let mut full_prompt = format!("{prefix}\n\n{}{ANTI_LEAK_RULES}", slide.prompt);
                if slide.texts.is_some() {
                    full_prompt.push_str(NO_TEXT_INSTRUCTION);
                }
                let out_file = slide_dir.join(format!("slide-{padded}.png"));
                let fingerprint = generation_fingerprint(
                    &full_prompt,
                    image_size,
                    &model_name,
                    &ref_images.iter().map(PathBuf::as_path).collect::<Vec<_>>(),
                );
                invalidate_stale_cache(&out_file, &fingerprint);
                batch_requests.push((
                    idx,
                    BatchImageRequest {
                        key: format!("slide-{padded}"),
                        prompt: full_prompt,
                        out_file,
                        image_size: image_size.map(String::from),
                        aspect_ratio: Some("16:9".to_string()),
                        ref_images,
                        model: model_name,
                    },
                    false,
                    fingerprint,
                ));
            }
        }

        // Collect indices and submit batch
        let indices: Vec<(usize, bool, String)> = batch_requests
            .iter()
            .map(|(i, _, al, fingerprint)| (*i, *al, fingerprint.clone()))
            .collect();
        let requests: Vec<BatchImageRequest> =
            batch_requests.into_iter().map(|(_, r, _, _)| r).collect();

        let batch_results = if requests.is_empty() {
            vec![]
        } else {
            match gemini.batch_gen_images(requests) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Batch failed ({e}), falling back to parallel sync...");
                    // Fall through to sync path below
                    return run_slides_sync(
                        slide_dir,
                        out_file,
                        slides,
                        style,
                        cfg,
                        &gemini,
                        &ocr_client,
                        &dashscope,
                        total,
                        concurrency,
                        image_size,
                        gen_model,
                        ref_image_size,
                        vision_model,
                    );
                }
            }
        };

        // Map batch results back
        let mut ref_paths_vec: Vec<Option<PathBuf>> = vec![None; total];
        let mut direct_paths_vec: Vec<Option<PathBuf>> = vec![None; total];

        for (result_idx, (slide_idx, is_auto, fingerprint)) in indices.iter().enumerate() {
            if let Some(path) = batch_results.get(result_idx).and_then(|r| r.as_ref()) {
                let _ = write_cache_stamp(path, fingerprint);
                if *is_auto {
                    ref_paths_vec[*slide_idx] = Some(path.to_path_buf());
                } else {
                    direct_paths_vec[*slide_idx] = Some(path.to_path_buf());
                }
            }
        }

        // Handle source_image slides
        for (idx, slide) in slides.iter().enumerate() {
            if slide.auto_layout {
                if let Some(ref src) = slide.source_image {
                    let padded = format!("{:02}", idx + 1);
                    let ref_file = slide_dir.join(format!("slide-{padded}-ref.png"));
                    let src_path = Path::new(src);
                    if src_path.exists() {
                        if let Ok(()) = std::fs::copy(src_path, &ref_file).map(|_| ()) {
                            eprintln!("Slide {} (source): {}", idx + 1, src);
                            ref_paths_vec[idx] = Some(ref_file);
                        }
                    }
                }
            }
        }

        // Phase 2: Extract text from ref images (sequential, VQA-first)
        let mut extracted_texts_vec: Vec<Option<Vec<TextOverlay>>> = vec![None; total];
        for idx in 0..total {
            let use_vqa = slides[idx].auto_layout;
            if !use_vqa {
                continue;
            }
            let Some(ref ref_path) = ref_paths_vec[idx] else {
                continue;
            };
            let variant = slides[idx].style.as_deref().unwrap_or("normal");
            let prefix = style.get_prompt(variant);

            let extraction_result = if let Some(ref ocr) = ocr_client {
                match extract_text_layout_deepseek(ocr, &gemini, ref_path, SW, SH, vision_model) {
                    Ok(texts) if !texts.is_empty() => {
                        eprintln!(
                            "Slide {}: OCR extracted {} text blocks",
                            idx + 1,
                            texts.len()
                        );
                        Ok((texts, true))
                    }
                    Ok(_) | Err(_) => {
                        extract_text_layout(&gemini, ref_path, SW, SH, vision_model, Some(prefix))
                            .map(|t| (t, false))
                    }
                }
            } else {
                extract_text_layout(&gemini, ref_path, SW, SH, vision_model, Some(prefix))
                    .map(|t| (t, false))
            };

            match extraction_result {
                Ok((texts, used_ocr)) => {
                    let texts = if !used_ocr {
                        refine_text_layout(&gemini, ref_path, &texts, SW, SH, vision_model)
                            .unwrap_or(texts)
                    } else {
                        texts
                    };
                    eprintln!("Slide {}: {} text elements", idx + 1, texts.len());
                    extracted_texts_vec[idx] = Some(texts);
                }
                Err(e) => eprintln!("Slide {}: text extraction failed — {e}", idx + 1),
            }
        }

        // Phase 3: Remove text with Qwen-Edit (VQA-first slides)
        let mut final_paths: Vec<Option<PathBuf>> = vec![None; total];
        for idx in 0..total {
            let use_vqa = slides[idx].auto_layout;
            if !use_vqa {
                final_paths[idx] = direct_paths_vec[idx].clone();
                continue;
            }
            let Some(ref ref_path) = ref_paths_vec[idx] else {
                continue;
            };
            let padded = format!("{:02}", idx + 1);
            let out_path = slide_dir.join(format!("slide-{padded}.png"));
            let edit_model = cfg.edit_model();
            let cache_fingerprint = edit_fingerprint(
                ref_path,
                "Remove all readable text, numbers, and punctuation from this image. \
                     Replace removed text with surrounding background. Keep all non-text elements.",
                edit_model,
            );
            invalidate_stale_cache(&out_path, &cache_fingerprint);
            if has_valid_cache(&out_path, &cache_fingerprint) {
                eprintln!("Cached: Slide {}", idx + 1);
                final_paths[idx] = Some(out_path);
                continue;
            }

            if let Some(ref ds) = dashscope {
                eprintln!("Slide {}: removing text with Qwen-Edit...", idx + 1);
                match ds.refine_image(
                    ref_path,
                    "Remove all readable text, numbers, and punctuation from this image. \
                     Replace removed text with surrounding background. Keep all non-text elements.",
                    &out_path,
                    Some(cfg.edit_model()),
                ) {
                    Ok(p) => {
                        let _ = write_cache_stamp(&p, &cache_fingerprint);
                        final_paths[idx] = Some(p);
                    }
                    Err(e) => eprintln!("Slide {}: Qwen-Edit failed ({e})", idx + 1),
                }
            }
        }

        // Build PPTX
        let slide_data: Vec<SlideData> = (0..total)
            .map(|i| {
                let image_path = final_paths[i]
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string());
                // VQA-first: user-provided texts take priority, fall back to VQA-extracted
                let texts = if slides[i].texts.is_some() {
                    slides[i].texts.clone().unwrap_or_default()
                } else if slides[i].auto_layout {
                    extracted_texts_vec[i].clone().unwrap_or_default()
                } else {
                    Vec::new()
                };
                let images = slides[i].overlay_images.clone().unwrap_or_default();
                SlideData {
                    image_path,
                    texts,
                    images,
                }
            })
            .collect();

        write_run_manifest(slide_dir, out_file, &final_paths)?;
        pptx::build_pptx(&slide_data, out_file, SW, SH)?;
        let ok = final_paths.iter().filter(|p| p.is_some()).count();
        eprintln!(
            "\nDone: {out_file} ({ok}/{total} slides)",
            out_file = out_file.display()
        );
        return Ok(());
    }

    // Sync path: Phase 1+2: Generate ref images and extract text (parallel)
    let ref_paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));
    let extracted_texts: Arc<Mutex<Vec<Option<Vec<TextOverlay>>>>> =
        Arc::new(Mutex::new(vec![None; total]));
    // For non-autoLayout slides, store final paths directly
    let direct_paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()?;

    pool.scope(|s| {
        for (idx, slide) in slides.iter().enumerate() {
            let gemini = &gemini;
            let dashscope = &dashscope;
            let ocr_client = &ocr_client;
            let ref_paths = Arc::clone(&ref_paths);
            let extracted_texts = Arc::clone(&extracted_texts);
            let direct_paths = Arc::clone(&direct_paths);

            s.spawn(move |_| {
                let variant = slide.style.as_deref().unwrap_or("normal");
                let prefix = style.get_prompt(variant);
                let padded = format!("{:02}", idx + 1);
                let model = slide
                    .gen_model
                    .as_deref()
                    .or(gen_model)
                    .unwrap_or(cfg.gen_model());

                let ref_images: Vec<&Path> = slide
                    .images
                    .as_ref()
                    .map(|imgs| imgs.iter().map(|p| Path::new(p.as_str())).collect())
                    .unwrap_or_default();

                // VQA-first: if texts are provided OR auto_layout is on,
                // generate WITH text, extract layout, remove text, overlay.
                let use_vqa = slide.auto_layout;

                if use_vqa {
                    // Phase 1: Get reference image (generate or use source_image)
                    let ref_file = slide_dir.join(format!("slide-{padded}-ref.png"));

                    let ref_ready = if let Some(ref src) = slide.source_image {
                        let src_path = Path::new(src);
                        if src_path.exists() {
                            if let Err(e) = std::fs::copy(src_path, &ref_file) {
                                eprintln!("Slide {}: failed to copy source image — {e}", idx + 1);
                                false
                            } else {
                                eprintln!("Slide {} (source): {}", idx + 1, src);
                                true
                            }
                        } else {
                            eprintln!("Slide {}: source_image not found: {src}", idx + 1);
                            false
                        }
                    } else {
                        // Generate WITH text (reference image) + anti-leak rules
                        let full_prompt = format!("{prefix}\n\n{}{ANTI_LEAK_RULES}", slide.prompt);
                        let ref_size = ref_image_size.or(image_size);
                        generate_image(
                            gemini,
                            dashscope,
                            &full_prompt,
                            &ref_file,
                            ref_size,
                            &ref_images,
                            model,
                            &format!("Slide {} (ref)", idx + 1),
                        )
                        .is_some()
                    };

                    if ref_ready {
                        // Phase 2: Extract text positions + styling
                        let extraction_result = if let Some(ref ocr) = ocr_client {
                            match extract_text_layout_deepseek(
                                ocr,
                                gemini,
                                &ref_file,
                                SW,
                                SH,
                                vision_model,
                            ) {
                                Ok(texts) if !texts.is_empty() => {
                                    eprintln!(
                                        "Slide {}: OCR extracted {} text blocks",
                                        idx + 1,
                                        texts.len()
                                    );
                                    Ok((texts, true))
                                }
                                Ok(_) => {
                                    eprintln!(
                                        "Slide {}: OCR returned empty, falling back to VQA",
                                        idx + 1
                                    );
                                    extract_text_layout(
                                        gemini,
                                        &ref_file,
                                        SW,
                                        SH,
                                        vision_model,
                                        Some(prefix),
                                    )
                                    .map(|t| (t, false))
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Slide {}: OCR failed ({e}), falling back to VQA",
                                        idx + 1
                                    );
                                    extract_text_layout(
                                        gemini,
                                        &ref_file,
                                        SW,
                                        SH,
                                        vision_model,
                                        Some(prefix),
                                    )
                                    .map(|t| (t, false))
                                }
                            }
                        } else {
                            extract_text_layout(
                                gemini,
                                &ref_file,
                                SW,
                                SH,
                                vision_model,
                                Some(prefix),
                            )
                            .map(|t| (t, false))
                        };

                        match extraction_result {
                            Ok((texts, used_ocr)) => {
                                eprintln!(
                                    "Slide {}: extracted {} text elements ({})",
                                    idx + 1,
                                    texts.len(),
                                    if used_ocr { "OCR" } else { "VQA" }
                                );
                                let texts = if !used_ocr {
                                    match refine_text_layout(
                                        gemini,
                                        &ref_file,
                                        &texts,
                                        SW,
                                        SH,
                                        vision_model,
                                    ) {
                                        Ok(refined) => {
                                            eprintln!(
                                                "Slide {}: refined {} text elements",
                                                idx + 1,
                                                refined.len()
                                            );
                                            refined
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Slide {}: refinement failed ({e}), using initial",
                                                idx + 1
                                            );
                                            texts
                                        }
                                    }
                                } else {
                                    texts
                                };
                                extracted_texts.lock().unwrap()[idx] = Some(texts);
                            }
                            Err(e) => {
                                eprintln!("Slide {}: text extraction failed — {e}", idx + 1);
                            }
                        }
                        ref_paths.lock().unwrap()[idx] = Some(ref_file);
                    }
                } else {
                    // Pure image mode — no text overlays
                    let full_prompt = format!("{prefix}\n\n{}{ANTI_LEAK_RULES}", slide.prompt);
                    let out_path = slide_dir.join(format!("slide-{padded}.png"));
                    if let Some(p) = generate_image(
                        gemini,
                        dashscope,
                        &full_prompt,
                        &out_path,
                        image_size,
                        &ref_images,
                        model,
                        &format!("Slide {}", idx + 1),
                    ) {
                        direct_paths.lock().unwrap()[idx] = Some(p);
                    }
                }
            });
        }
    });

    // Phase 3: Generate clean images (sequential for Qwen-Edit, parallel for Gemini)
    let ref_paths = ref_paths.lock().unwrap().clone();
    let direct_paths = direct_paths.lock().unwrap().clone();
    let mut final_paths: Vec<Option<PathBuf>> = vec![None; total];

    // Phase 3: Remove text from VQA-first slides
    #[allow(clippy::needless_range_loop)]
    for idx in 0..total {
        let use_vqa = slides[idx].auto_layout;
        if !use_vqa {
            final_paths[idx] = direct_paths[idx].clone();
            continue;
        }
        let Some(ref ref_path) = ref_paths[idx] else {
            continue;
        };

        let padded = format!("{:02}", idx + 1);
        let out_path = slide_dir.join(format!("slide-{padded}.png"));

        let text_removal_prompt = "Remove all readable text from this image. Replace text areas with the surrounding background. Keep all non-text visual elements exactly as they are — preserve all illustrations, wireframes, charts, icons, shapes, lines, and graphical elements. Only remove text.";
        let edit_model = if dashscope.is_some() {
            cfg.edit_model().to_string()
        } else {
            slides[idx]
                .gen_model
                .as_deref()
                .or(gen_model)
                .unwrap_or(cfg.gen_model())
                .to_string()
        };
        let cache_fingerprint = edit_fingerprint(ref_path, text_removal_prompt, &edit_model);
        invalidate_stale_cache(&out_path, &cache_fingerprint);
        if has_valid_cache(&out_path, &cache_fingerprint) {
            eprintln!("Cached: Slide {}", idx + 1);
            final_paths[idx] = Some(out_path);
            continue;
        }

        let removed = if let Some(ref ds) = dashscope {
            eprintln!("Slide {}: removing text with Qwen-Edit...", idx + 1);
            ds.refine_image(
                ref_path,
                text_removal_prompt,
                &out_path,
                Some(cfg.edit_model()),
            )
            .ok()
        } else {
            None
        };
        let removed = removed.or_else(|| {
            eprintln!("Slide {}: removing text with Gemini edit...", idx + 1);
            let model = slides[idx]
                .gen_model
                .as_deref()
                .or(gen_model)
                .unwrap_or(cfg.gen_model());
            gemini
                .edit_image(
                    ref_path,
                    text_removal_prompt,
                    &out_path,
                    Some(model),
                    Some(&format!("Slide {} (text-rm)", idx + 1)),
                )
                .ok()
                .flatten()
        });
        if let Some(p) = removed {
            let _ = write_cache_stamp(&p, &cache_fingerprint);
            final_paths[idx] = Some(p);
        }
    }

    // Build slide data
    let extracted = extracted_texts.lock().unwrap();

    let slide_data: Vec<SlideData> = (0..total)
        .map(|i| {
            let image_path = final_paths[i]
                .as_ref()
                .map(|p| p.to_string_lossy().to_string());
            // VQA-first: user-provided texts take priority, fall back to VQA-extracted
            let texts = if slides[i].texts.is_some() {
                slides[i].texts.clone().unwrap_or_default()
            } else if slides[i].auto_layout {
                extracted[i].clone().unwrap_or_default()
            } else {
                Vec::new()
            };
            let images = slides[i].overlay_images.clone().unwrap_or_default();
            SlideData {
                image_path,
                texts,
                images,
            }
        })
        .collect();

    write_run_manifest(slide_dir, out_file, &final_paths)?;
    pptx::build_pptx(&slide_data, out_file, SW, SH)?;
    let ok = final_paths.iter().filter(|p| p.is_some()).count();
    eprintln!(
        "\nDone: {out_file} ({ok}/{total} slides)",
        out_file = out_file.display()
    );
    Ok(())
}
