// SPDX-License-Identifier: Apache-2.0
//
// Test the hybrid OCR+VQA text extraction on a slide image.
// Usage: DASHSCOPE_API_KEY=... GEMINI_API_KEY=... cargo run --example test_ocr_layout -- <image_path>

use mofa_lib::dashscope::DashscopeClient;
use mofa_lib::gemini::GeminiClient;
use mofa_lib::layout::{extract_text_layout, extract_text_layout_ocr, SH, SW};
use std::path::Path;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();
    let img_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("/Users/yuechen/home/cc-ppt/slides-huawei-fullstack/slide-01.png");
    let img = Path::new(img_path);

    let ds_key = std::env::var("DASHSCOPE_API_KEY")
        .map_err(|_| eyre::eyre!("Set DASHSCOPE_API_KEY env var"))?;
    let gm_key =
        std::env::var("GEMINI_API_KEY").map_err(|_| eyre::eyre!("Set GEMINI_API_KEY env var"))?;

    let ds = DashscopeClient::new(ds_key);
    let gm = GeminiClient::new(gm_key);

    // Step 1: Raw OCR
    eprintln!("=== Step 1: Raw OCR ===");
    let words = ds.ocr_image(img)?;
    eprintln!("Detected {} words\n", words.len());
    for (i, w) in words.iter().enumerate() {
        eprintln!(
            "  [{i:3}] \"{:<30}\" ({:.0},{:.0})-({:.0},{:.0})  h={:.0}  fs={:.1}pt",
            w.text,
            w.left(),
            w.top(),
            w.right(),
            w.bottom(),
            w.height(),
            w.font_size_pt()
        );
    }

    // Step 2: Hybrid OCR+VQA extraction
    eprintln!("\n=== Step 2: Hybrid OCR+VQA ===");
    let ocr_overlays = extract_text_layout_ocr(&ds, &gm, img, SW, SH, None, None)?;
    eprintln!("\n{} text blocks:\n", ocr_overlays.len());
    for (i, ov) in ocr_overlays.iter().enumerate() {
        let text: String = ov.text.as_deref().unwrap_or("").chars().take(60).collect();
        eprintln!(
            "  [{i:2}] x={:.2}\" y={:.2}\" w={:.2}\" h={:.2}\"  fs={:.1}pt  color={}  bold={}  \"{}\"",
            ov.x,
            ov.y,
            ov.w,
            ov.h,
            ov.font_size.unwrap_or(0.0),
            ov.color,
            ov.bold,
            text.replace('\n', "\\n")
        );
    }

    // Step 3: VQA-only extraction (for comparison)
    eprintln!("\n=== Step 3: VQA-only (comparison) ===");
    let vqa_overlays = extract_text_layout(&gm, img, SW, SH, None, None)?;
    eprintln!("\n{} text elements:\n", vqa_overlays.len());
    for (i, ov) in vqa_overlays.iter().enumerate() {
        let text: String = ov.text.as_deref().unwrap_or("").chars().take(60).collect();
        eprintln!(
            "  [{i:2}] x={:.2}\" y={:.2}\" w={:.2}\" h={:.2}\"  fs={:.1}pt  color={}  bold={}  \"{}\"",
            ov.x,
            ov.y,
            ov.w,
            ov.h,
            ov.font_size.unwrap_or(0.0),
            ov.color,
            ov.bold,
            text.replace('\n', "\\n")
        );
    }

    // Step 4: Draw debug images
    eprintln!("\n=== Step 4: Debug images ===");
    draw_debug_image(img, &ocr_overlays, "ocr")?;
    draw_debug_image(img, &vqa_overlays, "vqa")?;

    Ok(())
}

fn draw_debug_image(
    image_path: &Path,
    overlays: &[mofa_lib::pptx::TextOverlay],
    label: &str,
) -> eyre::Result<()> {
    let mut img = image::ImageReader::open(image_path)?
        .with_guessed_format()?
        .decode()?
        .to_rgba8();

    let iw = img.width();
    let ih = img.height();
    let colors: &[(u8, u8, u8)] = &[
        (255, 0, 0),
        (0, 170, 0),
        (0, 0, 255),
        (255, 136, 0),
        (170, 0, 170),
        (0, 170, 170),
        (255, 68, 68),
        (68, 170, 68),
    ];

    for (idx, ov) in overlays.iter().enumerate() {
        let px = (ov.x / SW * iw as f64) as i32;
        let py = (ov.y / SH * ih as f64) as i32;
        let pw = (ov.w / SW * iw as f64) as i32;
        let ph = (ov.h / SH * ih as f64) as i32;
        let (r, g, b) = colors[idx % colors.len()];
        let color = image::Rgba([r, g, b, 200]);

        for t in 0..3i32 {
            let x0 = (px + t).max(0) as u32;
            let y0 = (py + t).max(0) as u32;
            let x1 = ((px + pw - t).max(0) as u32).min(iw - 1);
            let y1 = ((py + ph - t).max(0) as u32).min(ih - 1);
            for x in x0..=x1 {
                if y0 < ih {
                    img.put_pixel(x.min(iw - 1), y0, color);
                }
                if y1 < ih {
                    img.put_pixel(x.min(iw - 1), y1, color);
                }
            }
            for y in y0..=y1 {
                if x0 < iw {
                    img.put_pixel(x0, y.min(ih - 1), color);
                }
                if x1 < iw {
                    img.put_pixel(x1, y.min(ih - 1), color);
                }
            }
        }
    }

    let stem = image_path.file_stem().unwrap().to_string_lossy();
    let out_path = image_path.with_file_name(format!("{stem}-{label}-debug.png"));
    img.save(&out_path)?;
    eprintln!("Saved: {}", out_path.display());
    Ok(())
}
