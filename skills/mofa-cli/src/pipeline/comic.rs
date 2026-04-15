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

/// Input panel definition (from JSON).
#[derive(Deserialize, Debug)]
pub struct PanelInput {
    pub prompt: String,
    pub refine_prompt: Option<String>,
}

/// Generate panels using synchronous parallel calls.
#[allow(clippy::too_many_arguments)]
fn gen_panels_sync(
    gemini: &GeminiClient,
    out_dir: &Path,
    panels: &[PanelInput],
    style: &Style,
    total: usize,
    model: &str,
    panel_aspect: &str,
    image_size: Option<&str>,
    concurrency: usize,
) -> Vec<Option<PathBuf>> {
    let panel_paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .unwrap();

    pool.scope(|s| {
        for (idx, panel) in panels.iter().enumerate() {
            let panel_paths = Arc::clone(&panel_paths);

            s.spawn(move |_| {
                let prefix = style.get_prompt("panel");
                let prefix = if prefix.is_empty() {
                    style.get_prompt("normal")
                } else {
                    prefix
                };
                let full_prompt = format!(
                    "{prefix}\n\nPanel {} of {total}:\n{}",
                    idx + 1,
                    panel.prompt
                );
                let padded = format!("{:02}", idx + 1);
                let out_path = out_dir.join(format!("panel-{padded}.png"));

                if let Ok(Some(p)) = gemini.gen_image(
                    &full_prompt,
                    &out_path,
                    image_size,
                    Some(panel_aspect),
                    &[],
                    Some(model),
                    Some(&format!("Panel {}", idx + 1)),
                ) {
                    panel_paths.lock().unwrap()[idx] = Some(p);
                }
            });
        }
    });

    let result = panel_paths.lock().unwrap().clone();
    result
}

/// Comic pipeline: generate panels, optional Qwen refinement, stitch.
#[allow(clippy::too_many_arguments)]
pub fn run(
    out_dir: &Path,
    out_file: &Path,
    panels: &[PanelInput],
    style: &Style,
    cfg: &MofaConfig,
    layout: &str,
    concurrency: usize,
    image_size: Option<&str>,
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
    let total = panels.len();
    let model = gen_model.unwrap_or(cfg.gen_model());
    let panel_aspect = if layout == "vertical" { "16:9" } else { "1:1" };

    eprintln!("Generating {total}-panel comic ({layout})...");

    // Phase 1: Generate panels
    let mut panel_paths_vec: Vec<Option<PathBuf>> = if batch {
        // Batch API path
        let requests: Vec<BatchImageRequest> = panels
            .iter()
            .enumerate()
            .map(|(idx, panel)| {
                let prefix = style.get_prompt("panel");
                let prefix = if prefix.is_empty() {
                    style.get_prompt("normal")
                } else {
                    prefix
                };
                let padded = format!("{:02}", idx + 1);
                BatchImageRequest {
                    key: format!("panel-{padded}"),
                    prompt: format!(
                        "{prefix}\n\nPanel {} of {total}:\n{}",
                        idx + 1,
                        panel.prompt
                    ),
                    out_file: out_dir.join(format!("panel-{padded}.png")),
                    image_size: image_size.map(String::from),
                    aspect_ratio: Some(panel_aspect.to_string()),
                    ref_images: vec![],
                    model: model.to_string(),
                }
            })
            .collect();
        match gemini.batch_gen_images(requests) {
            Ok(results) => results,
            Err(e) => {
                eprintln!("Batch failed ({e}), falling back to parallel sync...");
                gen_panels_sync(
                    &gemini,
                    out_dir,
                    panels,
                    style,
                    total,
                    model,
                    panel_aspect,
                    image_size,
                    concurrency,
                )
            }
        }
    } else {
        gen_panels_sync(
            &gemini,
            out_dir,
            panels,
            style,
            total,
            model,
            panel_aspect,
            image_size,
            concurrency,
        )
    };

    // Phase 2: Optional Qwen-Edit refinement (sequential)
    if refine_with_qwen {
        if let Some(ds_key) = cfg.dashscope_key() {
            let dashscope = DashscopeClient::new(ds_key);
            eprintln!("Refining panels with Qwen-Edit...");
            for i in 0..total {
                if panel_paths_vec[i].is_none() {
                    continue;
                }
                if let Some(ref refine_prompt) = panels[i].refine_prompt {
                    let src = panel_paths_vec[i].as_ref().unwrap();
                    let refined = src.with_extension("refined.png");
                    match dashscope.refine_image(src, refine_prompt, &refined, None) {
                        Ok(p) => panel_paths_vec[i] = Some(p),
                        Err(e) => {
                            eprintln!("Panel {} refinement failed: {e}", i + 1);
                        }
                    }
                }
            }
        }
    }

    // Phase 3: Stitch panels
    let valid: Vec<&Path> = panel_paths_vec
        .iter()
        .filter_map(|p| p.as_deref())
        .collect();

    if valid.is_empty() {
        eprintln!("No panels generated, skipping stitch.");
        return Ok(None);
    }

    eprintln!("Stitching {} panels ({layout})...", valid.len());
    match layout {
        "horizontal" => image_util::stitch_horizontal(&valid, gutter, out_file)?,
        "vertical" => image_util::stitch_vertical(&valid, gutter, out_file)?,
        "grid" => image_util::stitch_grid(&valid, gutter, out_file)?,
        _ => image_util::stitch_horizontal(&valid, gutter, out_file)?,
    }

    Ok(Some(out_file.to_path_buf()))
}
