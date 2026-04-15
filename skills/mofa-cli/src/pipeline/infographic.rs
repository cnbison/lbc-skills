// SPDX-License-Identifier: Apache-2.0

use crate::config::MofaConfig;
use crate::dashscope::DashscopeClient;
use crate::gemini::{BatchImageRequest, GeminiClient};
use crate::image_util;
use crate::style::Style;
use eyre::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Input section definition (from JSON).
#[derive(Deserialize, Debug)]
pub struct SectionInput {
    pub prompt: String,
    pub refine_prompt: Option<String>,
    pub variant: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn gen_sections_sync(
    gemini: &GeminiClient,
    out_dir: &Path,
    sections: &[SectionInput],
    style: &Style,
    total: usize,
    model: &str,
    ar: &str,
    image_size: Option<&str>,
    concurrency: usize,
) -> Vec<Option<PathBuf>> {
    let paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .unwrap();

    pool.scope(|s| {
        for (idx, section) in sections.iter().enumerate() {
            let paths = Arc::clone(&paths);

            s.spawn(move |_| {
                let variant = section.variant.as_deref().unwrap_or_else(|| {
                    if idx == 0 {
                        "header"
                    } else if idx == total - 1 {
                        "footer"
                    } else {
                        "normal"
                    }
                });
                let prefix = style.get_prompt(variant);
                let full_prompt = format!(
                    "{prefix}\n\nSection {} of {total}:\n{}",
                    idx + 1,
                    section.prompt
                );
                let padded = format!("{:02}", idx + 1);
                let out_path = out_dir.join(format!("section-{padded}.png"));

                if let Ok(Some(p)) = gemini.gen_image(
                    &full_prompt,
                    &out_path,
                    image_size,
                    Some(ar),
                    &[],
                    Some(model),
                    Some(&format!("Section {}", idx + 1)),
                ) {
                    paths.lock().unwrap()[idx] = Some(p);
                }
            });
        }
    });

    let result = paths.lock().unwrap().clone();
    result
}

/// Infographic pipeline: generate sections, optional Qwen refinement, vertical stitch.
#[allow(clippy::too_many_arguments)]
pub fn run(
    out_dir: &Path,
    out_file: &Path,
    sections: &[SectionInput],
    style: &Style,
    cfg: &MofaConfig,
    concurrency: usize,
    image_size: Option<&str>,
    aspect_ratio: Option<&str>,
    refine_with_qwen: bool,
    gutter: u32,
    gen_model: Option<&str>,
    batch: bool,
) -> Result<Option<PathBuf>> {
    let gemini_key = cfg
        .gemini_key()
        .ok_or_else(|| eyre::eyre!("Gemini API key required"))?;
    let gemini = GeminiClient::new(gemini_key);

    std::fs::create_dir_all(out_dir)?;
    let total = sections.len();
    let model = gen_model.unwrap_or(cfg.gen_model());
    let ar = aspect_ratio.unwrap_or("16:9");

    eprintln!("Generating {total}-section infographic...");

    // Phase 1: Generate sections
    let mut section_paths_vec: Vec<Option<PathBuf>> = if batch {
        let requests: Vec<BatchImageRequest> = sections
            .iter()
            .enumerate()
            .map(|(idx, section)| {
                let variant = section.variant.as_deref().unwrap_or_else(|| {
                    if idx == 0 {
                        "header"
                    } else if idx == total - 1 {
                        "footer"
                    } else {
                        "normal"
                    }
                });
                let prefix = style.get_prompt(variant);
                let padded = format!("{:02}", idx + 1);
                BatchImageRequest {
                    key: format!("section-{padded}"),
                    prompt: format!(
                        "{prefix}\n\nSection {} of {total}:\n{}",
                        idx + 1,
                        section.prompt
                    ),
                    out_file: out_dir.join(format!("section-{padded}.png")),
                    image_size: image_size.map(String::from),
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
                gen_sections_sync(
                    &gemini,
                    out_dir,
                    sections,
                    style,
                    total,
                    model,
                    ar,
                    image_size,
                    concurrency,
                )
            }
        }
    } else {
        gen_sections_sync(
            &gemini,
            out_dir,
            sections,
            style,
            total,
            model,
            ar,
            image_size,
            concurrency,
        )
    };

    // Phase 2: Optional Qwen-Edit refinement (sequential)
    if refine_with_qwen {
        if let Some(ds_key) = cfg.dashscope_key() {
            let dashscope = DashscopeClient::new(ds_key);
            eprintln!("Refining sections with Qwen-Edit...");
            for i in 0..total {
                if section_paths_vec[i].is_none() {
                    continue;
                }
                if let Some(ref refine_prompt) = sections[i].refine_prompt {
                    let src = section_paths_vec[i].as_ref().unwrap();
                    let refined = src.with_extension("refined.png");
                    match dashscope.refine_image(src, refine_prompt, &refined, None) {
                        Ok(p) => section_paths_vec[i] = Some(p),
                        Err(e) => {
                            eprintln!("Section {} refinement failed: {e}", i + 1);
                        }
                    }
                }
            }
        }
    }

    // Phase 3: Stitch sections vertically
    let valid: Vec<&Path> = section_paths_vec
        .iter()
        .filter_map(|p| p.as_deref())
        .collect();

    if valid.is_empty() {
        eprintln!("No sections generated, skipping stitch.");
        return Ok(None);
    }

    eprintln!("Stitching {} sections vertically...", valid.len());
    image_util::stitch_vertical(&valid, gutter, out_file)?;
    Ok(Some(out_file.to_path_buf()))
}
