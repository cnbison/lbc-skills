// SPDX-License-Identifier: Apache-2.0

use crate::dashscope::DashscopeClient;
use crate::deepseek_ocr::DeepSeekOcrClient;
use crate::gemini::GeminiClient;
use crate::pptx::TextOverlay;
use eyre::Result;
use serde_json::Value;
use std::path::Path;

/// Slide dimensions in inches (widescreen 16:9).
pub const SW: f64 = 13.333;
pub const SH: f64 = 7.5;

/// Extract text layout from a slide image using Gemini vision QA.
pub fn extract_text_layout(
    gemini: &GeminiClient,
    image_path: &Path,
    sw: f64,
    sh: f64,
    vision_model: Option<&str>,
    style_hint: Option<&str>,
) -> Result<Vec<TextOverlay>> {
    let (img_w, img_h) = get_image_dimensions(image_path);
    let iw = img_w as u32;
    let ih = img_h as u32;

    let style_context = style_hint
        .map(|h| format!("\n\nSTYLE CONTEXT:\n{h}"))
        .unwrap_or_default();

    // Grid anchors to help the model with spatial reasoning
    let q1_y = ih / 4;
    let mid_y = ih / 2;
    let q3_y = ih * 3 / 4;
    let q1_x = iw / 4;
    let _mid_x = iw / 2;
    let q3_x = iw * 3 / 4;

    let prompt = format!(
        r#"You are a precise layout analyst. Analyze this slide image and extract EVERY text element.

IMAGE: {iw}×{ih} pixels. Origin = top-left (0,0).

SPATIAL GRID for reference:
- Top quarter: y = 0 to {q1_y}
- Upper-mid: y = {q1_y} to {mid_y}
- Lower-mid: y = {mid_y} to {q3_y}
- Bottom quarter: y = {q3_y} to {ih}
- Left quarter: x = 0 to {q1_x} | Center: x = {q1_x} to {q3_x} | Right: x = {q3_x} to {iw}

STEP 1: Mentally locate each text element within the grid zones above.
STEP 2: Report precise percentage-based coordinates.

For EVERY visible text element, return a JSON object with:
- "text": exact text content (use \n for multi-line blocks in same visual group)
- "xPct": left edge as percentage of image width (0.0–100.0)
- "yPct": TOP edge as percentage of image height (0.0–100.0) — topmost pixel touching the tallest letter
- "wPct": bounding box width as percentage of image width — use CONTAINER width (card, column, or full slide), NOT tight text width
- "hPct": bounding box height as percentage of image height
- "fontSize": font size in POINTS (1pt ≈ 1.333px). Match the VISUAL size precisely.
- "color": hex RGB without # (e.g. "2E7D32" for green)
- "bold": true if clearly bold/heavy weight
- "fontFace": best font match (e.g. "Helvetica", "Arial", "Inter")
- "align": "ctr" if centered in container, "l" for left, "r" for right

RULES:
1. yPct must be the TOP edge of the text — the topmost pixel row touching ascenders (h, l, A). Common error: reporting positions too far down. A title in the upper third MUST have yPct < 33.
2. wPct must use the CONTAINER width. Centered title text → nearly 100% of slide width. Text inside a card → the card's inner width.
3. Group multi-line text in same visual block as ONE entry with \n
4. Skip page numbers like "1 / 6"
{style_context}
Return ONLY a JSON array."#
    );

    let result = gemini.vision_qa(image_path, &prompt, vision_model)?;
    let raw: Vec<Value> = serde_json::from_value(result)?;

    let overlays: Vec<TextOverlay> = raw
        .into_iter()
        .map(|v| {
            let text = v.get("text").and_then(|t| t.as_str()).map(String::from);

            // Support both percentage-based (new) and pixel-based (fallback) coordinates
            let (x, y, w, h) = if v.get("xPct").is_some() {
                let xp = v.get("xPct").and_then(|n| n.as_f64()).unwrap_or(0.0);
                let yp = v.get("yPct").and_then(|n| n.as_f64()).unwrap_or(0.0);
                let wp = v.get("wPct").and_then(|n| n.as_f64()).unwrap_or(30.0);
                let hp = v.get("hPct").and_then(|n| n.as_f64()).unwrap_or(5.0);
                (
                    xp * sw / 100.0,
                    yp * sh / 100.0,
                    wp * sw / 100.0,
                    hp * sh / 100.0,
                )
            } else {
                let px = v.get("px").and_then(|n| n.as_f64()).unwrap_or(0.0);
                let py = v.get("py").and_then(|n| n.as_f64()).unwrap_or(0.0);
                let pw = v.get("pw").and_then(|n| n.as_f64()).unwrap_or(400.0);
                let ph = v.get("ph").and_then(|n| n.as_f64()).unwrap_or(50.0);
                (
                    px * sw / img_w,
                    py * sh / img_h,
                    pw * sw / img_w,
                    ph * sh / img_h,
                )
            };

            let font_size = v.get("fontSize").and_then(|n| n.as_f64());
            let align = v
                .get("align")
                .and_then(|a| a.as_str())
                .unwrap_or("l")
                .to_string();

            TextOverlay {
                text,
                x,
                y,
                w,
                h,
                font_size,
                color: v
                    .get("color")
                    .and_then(|c| c.as_str())
                    .unwrap_or("333333")
                    .to_string(),
                bold: v.get("bold").and_then(|b| b.as_bool()).unwrap_or(false),
                italic: false,
                font_face: v.get("fontFace").and_then(|f| f.as_str()).map(String::from),
                align,
                valign: String::new(),
                rotate: None,
                runs: None,
                shadow: None,
                line_spacing: None,
                margin: None,
                transparency: None,
                fill: None,
            }
        })
        .collect();

    let mut overlays = overlays;
    fix_bbox_from_font_size(&mut overlays);
    normalize_font_face(&mut overlays);
    rescale_x_positions(&mut overlays, sw);
    align_columns(&mut overlays, sw, sh);

    // Find topmost and bottommost y to detect title/footer by relative position
    let min_y = overlays.iter().map(|ov| ov.y).fold(f64::INFINITY, f64::min);
    let max_y = overlays
        .iter()
        .map(|ov| ov.y)
        .fold(f64::NEG_INFINITY, f64::max);

    for ov in &mut overlays {
        // Title/footer (within 0.3" of top/bottom) or centered wide elements → full width
        if ov.y - min_y < 0.3 || max_y - ov.y < 0.3 || (ov.align == "ctr" && ov.w > sw * 0.4) {
            ov.x = 0.3;
            ov.w = sw - 0.6;
        }
        // Clamp to slide bounds
        if ov.x < 0.0 {
            ov.x = 0.0;
        }
        if ov.x + ov.w > sw {
            ov.w = sw - ov.x;
        }
    }

    Ok(overlays)
}

/// Refine text layout by drawing bounding boxes on the reference image
/// and asking the vision model to correct any misaligned positions.
/// This is a feedback loop: extract → annotate → correct → finalize.
pub fn refine_text_layout(
    gemini: &GeminiClient,
    image_path: &Path,
    overlays: &[TextOverlay],
    sw: f64,
    sh: f64,
    vision_model: Option<&str>,
) -> Result<Vec<TextOverlay>> {
    let (img_w, img_h) = get_image_dimensions(image_path);
    let iw = img_w as u32;
    let ih = img_h as u32;

    // Draw colored bounding boxes on the image
    let mut img = image::ImageReader::open(image_path)?
        .with_guessed_format()?
        .decode()?
        .to_rgba8();
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

    // Build description of current boxes for the prompt
    let mut box_desc = String::new();
    for (idx, ov) in overlays.iter().enumerate() {
        let px = (ov.x / sw * img_w) as i32;
        let py = (ov.y / sh * img_h) as i32;
        let pw = (ov.w / sw * img_w) as i32;
        let ph = (ov.h / sh * img_h) as i32;
        let (r, g, b) = colors[idx % colors.len()];
        let color = image::Rgba([r, g, b, 255]);

        // Draw rectangle border (3px thick)
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

        let text_short = ov.text.as_deref().unwrap_or("").replace('\n', "\\n");
        let text_short = if text_short.chars().count() > 40 {
            format!("{}...", text_short.chars().take(40).collect::<String>())
        } else {
            text_short
        };
        box_desc.push_str(&format!(
            "[{idx}] \"{text_short}\" → xPct={:.1}, yPct={:.1}, wPct={:.1}, hPct={:.1}, fontSize={}, bold={}, color={}, align={}\n",
            ov.x / sw * 100.0, ov.y / sh * 100.0,
            ov.w / sw * 100.0, ov.h / sh * 100.0,
            ov.font_size.map(|f| format!("{f}")).unwrap_or("?".into()),
            ov.bold, ov.color, ov.align,
        ));
    }

    // Save annotated image to temp file
    let annotated_path = image_path.with_extension("annotated.png");
    img.save(&annotated_path)?;

    let prompt = format!(
        r#"I extracted text elements from this slide and drew colored bounding boxes (rectangles) showing where I think each text element is located.

IMAGE: {iw}×{ih} pixels.

CURRENT EXTRACTION (colored boxes on the image):
{box_desc}
TASK: Look at each colored box and check if it correctly covers the actual text in the image. For EVERY element, return the CORRECTED values.

Common errors to fix:
- Box is BELOW the actual text (yPct too high) — move it up
- Box is too narrow or too wide — adjust wPct
- Box extends outside the text's container (card, column) — constrain it
- Font size is wrong — measure the actual visual height
- Color is wrong — sample the actual text color (use dark colors for readability, even if the original text was light on dark background — the text will be overlaid on a clean version of this image where light text may not be readable)

Return a JSON array with ALL elements, each having:
"idx", "xPct", "yPct", "wPct", "hPct", "fontSize", "color" (hex without #, prefer dark readable colors), "bold", "align"

Return ONLY the JSON array."#
    );

    let result = gemini.vision_qa(&annotated_path, &prompt, vision_model)?;

    // Clean up temp file
    std::fs::remove_file(&annotated_path).ok();

    let corrections: Vec<Value> = serde_json::from_value(result)?;
    let mut refined = overlays.to_vec();

    for corr in &corrections {
        let idx = corr.get("idx").and_then(|i| i.as_u64()).unwrap_or(999) as usize;
        if idx >= refined.len() {
            continue;
        }

        // Apply corrections
        if let Some(xp) = corr.get("xPct").and_then(|n| n.as_f64()) {
            refined[idx].x = xp * sw / 100.0;
        }
        if let Some(yp) = corr.get("yPct").and_then(|n| n.as_f64()) {
            refined[idx].y = yp * sh / 100.0;
        }
        if let Some(wp) = corr.get("wPct").and_then(|n| n.as_f64()) {
            refined[idx].w = wp * sw / 100.0;
        }
        if let Some(hp) = corr.get("hPct").and_then(|n| n.as_f64()) {
            refined[idx].h = hp * sh / 100.0;
        }
        if let Some(fs) = corr.get("fontSize").and_then(|n| n.as_f64()) {
            refined[idx].font_size = Some(fs);
        }
        if let Some(c) = corr.get("color").and_then(|c| c.as_str()) {
            refined[idx].color = c.to_string();
        }
        if let Some(b) = corr.get("bold").and_then(|b| b.as_bool()) {
            refined[idx].bold = b;
        }
        if let Some(a) = corr.get("align").and_then(|a| a.as_str()) {
            refined[idx].align = a.to_string();
        }
    }

    fix_bbox_from_font_size(&mut refined);
    normalize_font_face(&mut refined);

    for ov in &mut refined {
        // Full-width for large centered elements
        if ov.align == "ctr" && ov.w > sw * 0.4 {
            ov.x = 0.3;
            ov.w = sw - 0.6;
        }
        // Clamp to slide bounds
        if ov.x + ov.w > sw {
            ov.w = sw - ov.x;
        }
    }

    Ok(refined)
}

fn get_image_dimensions(image_path: &Path) -> (f64, f64) {
    if let Ok(reader) = image::ImageReader::open(image_path) {
        if let Ok(reader) = reader.with_guessed_format() {
            if let Ok((w, h)) = reader.into_dimensions() {
                return (w as f64, h as f64);
            }
        }
    }
    (1920.0, 1080.0)
}

/// Fix bounding box heights to match VQA font sizes.
///
/// VQA font size guesses are reasonably accurate, but VQA bounding box heights
/// are systematically too small (~2.9x underestimate). We trust font sizes and
/// expand heights to match.
///
/// Also caps font sizes at 40pt — VQA occasionally overestimates large CJK
/// title fonts (e.g. reports 56pt when actual is ~30pt). Users can always
/// adjust font sizes in the PPTX.
fn fix_bbox_from_font_size(overlays: &mut [TextOverlay]) {
    const LINE_SPACING: f64 = 1.3;
    const MAX_FONT_SIZE: f64 = 40.0;

    for ov in overlays.iter_mut() {
        let text = ov.text.as_deref().unwrap_or("");
        if text.is_empty() {
            continue;
        }

        let mut fs = ov.font_size.unwrap_or(18.0);

        // Cap overly large font sizes
        if fs > MAX_FONT_SIZE {
            eprintln!(
                "  bbox fix: fs={:.0}pt → {:.0}pt (capped) for {:?}",
                fs,
                MAX_FONT_SIZE,
                &text.chars().take(30).collect::<String>()
            );
            fs = MAX_FONT_SIZE;
            ov.font_size = Some(fs);
        }

        let num_lines = text.split('\n').count() as f64;
        let expected_h = (fs / 72.0) * num_lines * LINE_SPACING;

        if expected_h > ov.h {
            eprintln!(
                "  bbox fix: h={:.3}\" → {:.3}\" (fs={:.0}pt, {}lines) for {:?}",
                ov.h,
                expected_h,
                fs,
                num_lines as u32,
                &text.chars().take(30).collect::<String>()
            );
            ov.h = expected_h;
        }
    }
}

/// Normalize VQA font face guesses to fonts commonly available in PowerPoint.
fn normalize_font_face(overlays: &mut [TextOverlay]) {
    for ov in overlays.iter_mut() {
        if let Some(ref face) = ov.font_face {
            let normalized = match face.to_lowercase().as_str() {
                // Sans-serif family
                "helvetica" | "helvetica neue" | "sf pro" | "sf pro display" | "sf pro text"
                | "system-ui" | "segoe ui" => "Arial",
                "inter" | "roboto" | "open sans" | "source sans pro" | "noto sans" | "lato"
                | "poppins" | "montserrat" => "Calibri",
                // Serif family
                "times" | "times new roman" | "noto serif" | "source serif pro" | "georgia"
                | "garamond" | "palatino" => "Times New Roman",
                // Monospace
                "courier" | "courier new" | "sf mono" | "fira code" | "jetbrains mono"
                | "source code pro" | "menlo" | "consolas" | "monaco" => "Courier New",
                // Already good PPT fonts — pass through
                "arial"
                | "calibri"
                | "cambria"
                | "tahoma"
                | "verdana"
                | "trebuchet ms"
                | "century gothic"
                | "gill sans mt"
                | "franklin gothic medium"
                | "impact" => face.as_str(),
                // CJK fonts — keep as-is or map to common ones
                _ if face.contains("黑体") || face.contains("Hei") => "Microsoft YaHei",
                _ if face.contains("宋体") || face.contains("Song") => "SimSun",
                // Unknown → default to Calibri (modern, clean)
                _ => "Calibri",
            };
            if normalized != face.as_str() {
                ov.font_face = Some(normalized.to_string());
            }
        }
    }
}

/// Global coordinate transform: rescale x positions to fill the slide content area.
///
/// VQA positions have good relative accuracy but poor absolute accuracy — positions
/// are systematically compressed (elements closer together than reality). This maps
/// the VQA content bounding box to the expected slide content area, stretching
/// positions proportionally. Local alignment is perfectly preserved.
fn rescale_x_positions(overlays: &mut [TextOverlay], sw: f64) {
    if overlays.len() < 4 {
        return;
    }

    // Find VQA content extent (x dimension)
    let vqa_min = overlays.iter().map(|ov| ov.x).fold(f64::INFINITY, f64::min);
    let vqa_max = overlays
        .iter()
        .map(|ov| ov.x + ov.w)
        .fold(f64::NEG_INFINITY, f64::max);
    let vqa_extent = vqa_max - vqa_min;

    if vqa_extent < 1.0 {
        return;
    }

    // Expected content area
    let margin = 0.3;
    let expected_min = margin;
    let expected_max = sw - margin;
    let expected_extent = expected_max - expected_min;

    // Only rescale if VQA extent is significantly smaller than expected
    // (> 5% compression). Otherwise VQA extent is plausible.
    let ratio = expected_extent / vqa_extent;
    if ratio < 1.05 {
        return;
    }

    eprintln!(
        "  x-rescale: VQA extent {vqa_min:.2}\"..{vqa_max:.2}\" ({vqa_extent:.2}\") → \
         {expected_min:.2}\"..{expected_max:.2}\" ({expected_extent:.2}\"), scale={ratio:.3}"
    );

    // Apply affine transform: new_x = (old_x - vqa_min) / vqa_extent * expected_extent + expected_min
    for ov in overlays.iter_mut() {
        ov.x = (ov.x - vqa_min) / vqa_extent * expected_extent + expected_min;
        ov.w *= ratio;
    }
}

/// Detect columns of text and align them.
///
/// VQA returns positions with systematic bias and too-narrow widths.
/// This detects groups of overlays at similar x positions (columns), snaps each
/// group to the leftmost x, and expands widths to fill the column gap.
///
/// Uses gap-based clustering: sort x positions, find the largest gaps between
/// groups, and split at those gaps. This correctly handles cases where VQA
/// places elements within the same column at different x offsets.
fn align_columns(overlays: &mut [TextOverlay], sw: f64, _sh: f64) {
    if overlays.len() < 6 {
        return;
    }

    // Collect (index, x) for non-title elements
    let mut items: Vec<(usize, f64)> = overlays
        .iter()
        .enumerate()
        .filter(|(_, ov)| {
            // Skip full-width centered elements (titles, subtitles)
            !(ov.align == "ctr" && ov.w > sw * 0.3)
        })
        .map(|(i, ov)| (i, ov.x))
        .collect();

    if items.len() < 6 {
        return;
    }

    items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Step 1: Initial fine-grained grouping (0.5" tolerance)
    let col_tol = 0.5;
    let mut groups: Vec<(f64, Vec<usize>)> = Vec::new(); // (median_x, indices)
    for &(idx, x) in &items {
        let added = groups.iter_mut().any(|(gx, members)| {
            if (x - *gx).abs() < col_tol {
                members.push(idx);
                true
            } else {
                false
            }
        });
        if !added {
            groups.push((x, vec![idx]));
        }
    }

    // Sort groups by x and compute median x for each
    groups.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut group_medians: Vec<f64> = Vec::new();
    for (_, members) in &groups {
        let mut xs: Vec<f64> = members.iter().map(|&i| overlays[i].x).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        group_medians.push(xs[xs.len() / 2]);
    }

    // Step 2: Merge groups using gap analysis
    // Find gaps between consecutive groups and merge if gap is small relative to the largest gap
    if groups.len() >= 2 {
        let mut merged = true;
        while merged {
            merged = false;
            if groups.len() < 2 {
                break;
            }

            // Find all gaps
            let mut gaps: Vec<(usize, f64)> = (0..groups.len() - 1)
                .map(|i| (i, group_medians[i + 1] - group_medians[i]))
                .collect();
            gaps.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            // Find the largest gap
            let max_gap = gaps.last().map(|g| g.1).unwrap_or(0.0);

            // Merge the pair with the smallest gap if it's < 75% of the largest gap
            if let Some(&(merge_idx, smallest_gap)) = gaps.first() {
                if smallest_gap < max_gap * 0.75 && groups.len() > 2 {
                    // Merge group merge_idx and merge_idx+1
                    let right_members = groups[merge_idx + 1].1.clone();
                    groups[merge_idx].1.extend(right_members);
                    groups.remove(merge_idx + 1);

                    // Recompute median for merged group
                    let mut xs: Vec<f64> =
                        groups[merge_idx].1.iter().map(|&i| overlays[i].x).collect();
                    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    groups[merge_idx].0 = xs[xs.len() / 2];
                    group_medians.remove(merge_idx + 1);
                    group_medians[merge_idx] = xs[xs.len() / 2];

                    merged = true;
                }
            }
        }
    }

    // Step 3: Filter to columns with 3+ members
    let mut columns: Vec<Vec<usize>> = groups
        .into_iter()
        .map(|(_, members)| members)
        .filter(|c| c.len() >= 3)
        .collect();

    if columns.is_empty() {
        return;
    }

    // Sort columns by position
    columns.sort_by(|a, b| overlays[a[0]].x.partial_cmp(&overlays[b[0]].x).unwrap());

    // Step 4: Distribute columns evenly across the slide content area.
    // VQA relative column positions are unreliable — evenly distributing
    // is more accurate for structured presentation slides.
    let col_count = columns.len();
    let margin = 0.3;
    let gap = 0.15;
    let content_w = sw - 2.0 * margin;
    let col_width = (content_w - (col_count as f64 - 1.0) * gap) / col_count as f64;

    for (ci, col) in columns.iter().enumerate() {
        let col_x = margin + ci as f64 * (col_width + gap);

        eprintln!(
            "  col-align [{ci}]: {} members, x={col_x:.2}\" w={col_width:.2}\"",
            col.len(),
        );

        for &idx in col {
            overlays[idx].x = col_x;
            overlays[idx].w = col_width;
        }
    }
}

/// Extract text layout — currently delegates to VQA extraction.
/// Kept as a separate entry point for pipeline compatibility (OCR path).
#[allow(dead_code)]
pub fn extract_text_layout_ocr(
    _dashscope: &DashscopeClient,
    gemini: &GeminiClient,
    image_path: &Path,
    sw: f64,
    sh: f64,
    vision_model: Option<&str>,
    style_hint: Option<&str>,
) -> Result<Vec<TextOverlay>> {
    extract_text_layout(gemini, image_path, sw, sh, vision_model, style_hint)
}

/// Extract text layout using DeepSeek-OCR-2 for positions + Gemini VQA for text content & styles.
///
/// DeepSeek-OCR-2 provides pixel-accurate bounding boxes via its grounding mode.
/// Gemini VQA reads the actual text content and visual styles (color, bold, font, alignment)
/// for each detected region. This hybrid gives accurate positions with correct text.
pub fn extract_text_layout_deepseek(
    deepseek: &DeepSeekOcrClient,
    gemini: &GeminiClient,
    image_path: &Path,
    sw: f64,
    sh: f64,
    vision_model: Option<&str>,
) -> Result<Vec<TextOverlay>> {
    let (_img_w, img_h) = get_image_dimensions(image_path);

    // Step 1: Get text block bounding boxes from DeepSeek-OCR-2
    let mut blocks = deepseek.ocr_with_grounding(image_path)?;
    if blocks.is_empty() {
        return Ok(Vec::new());
    }

    // De-duplicate overlapping blocks (DeepSeek sometimes emits near-identical bboxes)
    blocks = dedup_ocr_blocks(blocks);
    eprintln!("  DeepSeek: {} bounding boxes after dedup", blocks.len());

    // Step 2: Build a position map for Gemini — tell it WHERE each block is,
    // ask it to read the ACTUAL text and style at each location.
    let mut block_desc = String::new();
    for (i, block) in blocks.iter().enumerate() {
        let xp = block.x1 / 10.0;
        let yp = block.y1 / 10.0;
        let wp = (block.x2 - block.x1) / 10.0;
        let hp = (block.y2 - block.y1) / 10.0;
        block_desc.push_str(&format!(
            "[{i}] region at xPct={xp:.1}, yPct={yp:.1}, wPct={wp:.1}, hPct={hp:.1}\n"
        ));
    }

    let iw = _img_w as u32;
    let ih = img_h as u32;

    let prompt = format!(
        r#"I detected {count} text regions in this {iw}x{ih} slide image using OCR. For each region below, read the EXACT text visible at that location and determine its visual style.

REGIONS (positions as percentage of image dimensions):
{block_desc}
For EVERY region, return a JSON object with:
- "idx": region index
- "text": the EXACT text visible at this location (use \n for multi-line). Read carefully — do NOT guess or paraphrase. If a region contains NO readable text (just icons/graphics), set text to "".
- "fontSize": font size in points (1pt ≈ 1.333px)
- "color": hex RGB without # (e.g. "FFFFFF" for white, "C8102E" for red)
- "bold": true if bold/heavy weight
- "fontFace": best font match (e.g. "Arial", "Microsoft YaHei")
- "align": "ctr" if centered, "l" for left, "r" for right

ALSO: if you see any significant text in the image that is NOT covered by the regions above, add extra entries with "idx": -1 and include "xPct", "yPct", "wPct", "hPct" (percentage of image dimensions) for the position.

Return ONLY a JSON array."#,
        count = blocks.len(),
    );

    let vqa_result = gemini.vision_qa(image_path, &prompt, vision_model)?;
    let entries: Vec<Value> = serde_json::from_value(vqa_result)?;

    // Step 3: Merge DeepSeek positions + VQA text/styles
    let mut overlays: Vec<TextOverlay> = Vec::new();

    for entry in &entries {
        let idx = entry.get("idx").and_then(|i| i.as_i64()).unwrap_or(999);

        let text = entry
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        if text.is_empty() {
            continue;
        }

        let (x, y, w, h) = if idx >= 0 && (idx as usize) < blocks.len() {
            // Position from DeepSeek grounding (accurate)
            let block = &blocks[idx as usize];
            (
                block.x1 / 1000.0 * sw,
                block.y1 / 1000.0 * sh,
                (block.x2 - block.x1) / 1000.0 * sw,
                (block.y2 - block.y1) / 1000.0 * sh,
            )
        } else if idx == -1 {
            // Extra block from VQA — position from VQA percentages
            let xp = entry.get("xPct").and_then(|n| n.as_f64()).unwrap_or(0.0);
            let yp = entry.get("yPct").and_then(|n| n.as_f64()).unwrap_or(0.0);
            let wp = entry.get("wPct").and_then(|n| n.as_f64()).unwrap_or(10.0);
            let hp = entry.get("hPct").and_then(|n| n.as_f64()).unwrap_or(5.0);
            let preview: String = text.chars().take(20).collect();
            eprintln!("  VQA extra block: \"{preview}\" at ({xp:.1}%, {yp:.1}%)");
            (
                xp / 100.0 * sw,
                yp / 100.0 * sh,
                wp / 100.0 * sw,
                hp / 100.0 * sh,
            )
        } else {
            continue;
        };

        // Text + style from VQA (accurate content)
        let font_size = entry.get("fontSize").and_then(|n| n.as_f64());
        let color = entry
            .get("color")
            .and_then(|c| c.as_str())
            .unwrap_or("333333")
            .to_string();
        let bold = entry.get("bold").and_then(|b| b.as_bool()).unwrap_or(false);
        let font_face = entry
            .get("fontFace")
            .and_then(|f| f.as_str())
            .map(String::from);
        let align = entry
            .get("align")
            .and_then(|a| a.as_str())
            .unwrap_or("l")
            .to_string();

        overlays.push(TextOverlay {
            text: Some(text),
            x,
            y,
            w,
            h,
            font_size,
            color,
            bold,
            italic: false,
            font_face,
            align,
            valign: String::new(),
            rotate: None,
            runs: None,
            shadow: None,
            line_spacing: None,
            margin: None,
            transparency: None,
            fill: None,
        });
    }

    eprintln!(
        "  DeepSeek+VQA: {} blocks (positions: DeepSeek, text+style: VQA)",
        overlays.len()
    );

    fix_bbox_from_font_size(&mut overlays);
    normalize_font_face(&mut overlays);

    for ov in &mut overlays {
        if ov.align == "ctr" && ov.w > sw * 0.4 {
            ov.x = 0.3;
            ov.w = sw - 0.6;
        }
        if ov.x + ov.w > sw {
            ov.w = sw - ov.x;
        }
    }

    Ok(overlays)
}

/// De-duplicate OCR blocks with high spatial overlap.
/// DeepSeek-OCR-2 sometimes emits near-identical bounding boxes for the same region.
fn dedup_ocr_blocks(
    blocks: Vec<crate::deepseek_ocr::OcrBlock>,
) -> Vec<crate::deepseek_ocr::OcrBlock> {
    use crate::deepseek_ocr::OcrBlock;

    let mut kept: Vec<OcrBlock> = Vec::new();
    for block in blocks {
        let dominated = kept.iter().any(|existing| {
            // Check if bboxes overlap significantly (IoU-like check)
            let ix1 = block.x1.max(existing.x1);
            let iy1 = block.y1.max(existing.y1);
            let ix2 = block.x2.min(existing.x2);
            let iy2 = block.y2.min(existing.y2);

            if ix1 >= ix2 || iy1 >= iy2 {
                return false; // No overlap
            }

            let inter = (ix2 - ix1) * (iy2 - iy1);
            let area_new = (block.x2 - block.x1) * (block.y2 - block.y1);
            let area_existing = (existing.x2 - existing.x1) * (existing.y2 - existing.y1);
            let smaller_area = area_new.min(area_existing);

            // If intersection covers >70% of the smaller block, it's a duplicate
            smaller_area > 0.0 && inter / smaller_area > 0.7
        });

        if !dominated {
            kept.push(block);
        }
    }
    kept
}

/// The "no text" instruction appended to prompts for clean image regeneration.
pub const NO_TEXT_INSTRUCTION: &str = "\n\nCRITICAL: DO NOT render any text, words, labels, \
    numbers, or letters anywhere on the image. The image must be purely visual with no readable \
    content whatsoever. Leave clean space where text would normally appear.";

/// Anti-leak rules appended to ALL image generation prompts.
/// Prevents Gemini from rendering formatting hints (font sizes, hex colors, CSS notation)
/// as literal text in the generated image.
pub const ANTI_LEAK_RULES: &str = r#"

ANTI-LEAK RULES (CRITICAL — follow exactly):
1. NO parenthetical formatting near content text — never write "(28pt, bold, #C7000B)" next to text content
2. Content text goes in quotes, layout/formatting instructions go OUTSIDE quotes
3. Describe formatting in natural language: "large bold white title" NOT "title (24pt, bold, #FFFFFF)"
4. NO hex color codes, font sizes in pt/px, or CSS-like notation adjacent to content text
5. DO NOT include any page numbers or slide numbers in the image"#;
