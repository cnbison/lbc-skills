// SPDX-License-Identifier: Apache-2.0

use crate::config::MofaConfig;
use crate::gemini::{BatchImageRequest, GeminiClient};
use crate::style::Style;
use eyre::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Input card definition (from JSON).
#[derive(Deserialize, Debug)]
pub struct CardInput {
    pub name: String,
    pub prompt: String,
    pub style: Option<String>,
}

/// Card pipeline: generate PNG greeting cards in parallel.
#[allow(clippy::too_many_arguments)]
pub fn run(
    card_dir: &Path,
    cards: &[CardInput],
    style: &Style,
    cfg: &MofaConfig,
    concurrency: usize,
    aspect_ratio: Option<&str>,
    image_size: Option<&str>,
    gen_model: Option<&str>,
    batch: bool,
) -> Result<Vec<Option<PathBuf>>> {
    let gemini_key = cfg
        .gemini_key()
        .ok_or_else(|| eyre::eyre!("Gemini API key required"))?;
    let gemini = GeminiClient::new(gemini_key);

    std::fs::create_dir_all(card_dir)?;
    let total = cards.len();
    let ar = aspect_ratio.unwrap_or(
        cfg.defaults
            .cards
            .as_ref()
            .and_then(|c| c.aspect_ratio.as_deref())
            .unwrap_or("9:16"),
    );
    let size = image_size.or(cfg
        .defaults
        .cards
        .as_ref()
        .and_then(|c| c.image_size.as_deref()));
    let model = gen_model.unwrap_or(cfg.gen_model());

    let mode_str = if batch {
        "batch, ".to_string()
    } else {
        format!("{concurrency} parallel, ")
    };
    eprintln!("Generating {total} cards ({mode_str}{ar})...");

    let result = if batch {
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
                gen_cards_sync(
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
        gen_cards_sync(
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

    let ok = result.iter().filter(|p| p.is_some()).count();
    eprintln!("\nDone: {ok}/{total} cards in {}/", card_dir.display());
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn gen_cards_sync(
    gemini: &GeminiClient,
    card_dir: &Path,
    cards: &[CardInput],
    style: &Style,
    total: usize,
    model: &str,
    ar: &str,
    size: Option<&str>,
    concurrency: usize,
) -> Vec<Option<PathBuf>> {
    let paths: Arc<Mutex<Vec<Option<PathBuf>>>> = Arc::new(Mutex::new(vec![None; total]));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .unwrap();

    pool.scope(|s| {
        for (idx, card) in cards.iter().enumerate() {
            let paths = Arc::clone(&paths);

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
                    paths.lock().unwrap()[idx] = Some(p);
                }
            });
        }
    });

    let result = paths.lock().unwrap().clone();
    result
}
