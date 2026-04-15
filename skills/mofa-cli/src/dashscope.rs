// SPDX-License-Identifier: Apache-2.0

use base64::Engine;
use eyre::Result;
use serde_json::{json, Value};
use std::path::Path;

const DEFAULT_EDIT_MODEL: &str = "qwen-image-edit-max";

/// A word/line detected by OCR with 4-corner bounding box (pixel coordinates).
/// Corners: top-left (x1,y1), top-right (x2,y2), bottom-right (x3,y3), bottom-left (x4,y4).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OcrWord {
    pub text: String,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub x3: f64,
    pub y3: f64,
    pub x4: f64,
    pub y4: f64,
}

#[allow(dead_code)]
impl OcrWord {
    /// Bounding box left edge (min x).
    pub fn left(&self) -> f64 {
        self.x1.min(self.x4)
    }
    /// Bounding box top edge (min y).
    pub fn top(&self) -> f64 {
        self.y1.min(self.y2)
    }
    /// Bounding box width.
    pub fn width(&self) -> f64 {
        self.x2.max(self.x3) - self.left()
    }
    /// Bounding box height.
    pub fn height(&self) -> f64 {
        self.y3.max(self.y4) - self.top()
    }
    /// Estimated font size in points (height_px / 1.333).
    pub fn font_size_pt(&self) -> f64 {
        self.height() / 1.333
    }
    /// Bounding box right edge (max x).
    pub fn right(&self) -> f64 {
        self.x2.max(self.x3)
    }
    /// Bounding box bottom edge (max y).
    pub fn bottom(&self) -> f64 {
        self.y3.max(self.y4)
    }
    /// Vertical center of the bounding box.
    pub fn center_y(&self) -> f64 {
        (self.top() + self.bottom()) / 2.0
    }
    /// Horizontal center of the bounding box.
    pub fn center_x(&self) -> f64 {
        (self.left() + self.right()) / 2.0
    }
}

/// Sample the background color around a bounding box by looking at border pixels.
/// Takes median of border pixel colors to avoid sampling text or edge artifacts.
#[allow(dead_code)]
fn sample_border_color(
    img: &image::RgbImage,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
) -> image::Rgb<u8> {
    let (iw, ih) = (img.width(), img.height());
    let mut samples: Vec<[u8; 3]> = Vec::new();

    // Sample from a border strip 2-4px outside the bbox
    let margin = 3u32;
    let outer_x0 = x0.saturating_sub(margin);
    let outer_y0 = y0.saturating_sub(margin);
    let outer_x1 = (x1 + margin).min(iw - 1);
    let outer_y1 = (y1 + margin).min(ih - 1);

    // Top edge
    if outer_y0 < y0 {
        for x in outer_x0..=outer_x1 {
            let p = img.get_pixel(x, outer_y0);
            samples.push(p.0);
        }
    }
    // Bottom edge
    if outer_y1 > y1 {
        for x in outer_x0..=outer_x1 {
            let p = img.get_pixel(x, outer_y1);
            samples.push(p.0);
        }
    }
    // Left edge
    if outer_x0 < x0 {
        for y in outer_y0..=outer_y1 {
            let p = img.get_pixel(outer_x0, y);
            samples.push(p.0);
        }
    }
    // Right edge
    if outer_x1 > x1 {
        for y in outer_y0..=outer_y1 {
            let p = img.get_pixel(outer_x1, y);
            samples.push(p.0);
        }
    }

    if samples.is_empty() {
        return image::Rgb([255, 255, 255]); // fallback to white
    }

    // Median per channel
    let mut rs: Vec<u8> = samples.iter().map(|s| s[0]).collect();
    let mut gs: Vec<u8> = samples.iter().map(|s| s[1]).collect();
    let mut bs: Vec<u8> = samples.iter().map(|s| s[2]).collect();
    rs.sort();
    gs.sort();
    bs.sort();
    let mid = rs.len() / 2;
    image::Rgb([rs[mid], gs[mid], bs[mid]])
}

/// Default Dashscope API base URL.
const DEFAULT_DASHSCOPE_URL: &str =
    "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation";

/// Dashscope API client for Qwen image editing.
pub struct DashscopeClient {
    api_key: String,
    endpoint: String,
    http: reqwest::blocking::Client,
}

impl DashscopeClient {
    pub fn new(api_key: String) -> Self {
        let endpoint = std::env::var("DASHSCOPE_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_DASHSCOPE_URL.to_string());
        Self {
            api_key,
            endpoint,
            http: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap(),
        }
    }

    /// Refine an image via Qwen-Edit using the multimodal-generation API.
    pub fn refine_image(
        &self,
        image_path: &Path,
        prompt: &str,
        out_file: &Path,
        model: Option<&str>,
    ) -> Result<std::path::PathBuf> {
        let model = model.unwrap_or(DEFAULT_EDIT_MODEL);

        // Resize large images to reduce payload, remember original size for upscale
        let img = image::ImageReader::open(image_path)?
            .with_guessed_format()?
            .decode()?;
        let orig_w = img.width();
        let orig_h = img.height();
        let img = if orig_w > 2048 {
            let scale = 2048.0 / orig_w as f64;
            let new_h = (orig_h as f64 * scale) as u32;
            eprintln!(
                "  Dashscope: resizing {}x{} → 2048x{} for upload",
                orig_w, orig_h, new_h
            );
            img.resize_exact(2048, new_h, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        // Encode as base64 JPEG for inline submission
        let mut jpeg_buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut jpeg_buf, image::ImageFormat::Jpeg)?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(jpeg_buf.get_ref());
        let data_uri = format!("data:image/jpeg;base64,{b64}");
        eprintln!(
            "  Dashscope: encoded image as base64 ({}KB)",
            b64.len() / 1024
        );

        // Use the multimodal-generation endpoint with messages format
        let body = json!({
            "model": model,
            "input": {
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            { "image": data_uri },
                            { "text": prompt }
                        ]
                    }
                ]
            },
            "parameters": {
                "n": 1,
                "watermark": false,
                "prompt_extend": false,
                "negative_prompt": "text, words, letters, watermark"
            }
        });

        // Retry with backoff for rate limits
        let max_retries = 3;
        for attempt in 0..=max_retries {
            eprintln!("  Dashscope: submitting edit task (sync)...");

            let resp = self
                .http
                .post(&self.endpoint)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?;

            let data: Value = resp.json()?;

            // Check for rate limit
            if let Some(code) = data.get("code").and_then(|c| c.as_str()) {
                if (code.contains("RateQuota") || code.contains("Throttling"))
                    && attempt < max_retries
                {
                    let wait = 10 * (attempt + 1) as u64;
                    eprintln!("  Dashscope: rate limited, retrying in {wait}s...");
                    std::thread::sleep(std::time::Duration::from_secs(wait));
                    continue;
                }
            }

            // Extract result image URL from response
            let img_url = data
                .pointer("/output/choices/0/message/content/0/image")
                .and_then(|u| u.as_str())
                .ok_or_else(|| eyre::eyre!("Dashscope edit failed: {data}"))?;

            let path = self.download_result(img_url, out_file)?;

            // Upscale back to original resolution if we downscaled
            if orig_w > 2048 {
                eprintln!("  Dashscope: upscaling back to {}x{}", orig_w, orig_h);
                let result_img = image::ImageReader::open(&path)?
                    .with_guessed_format()?
                    .decode()?;
                let upscaled =
                    result_img.resize_exact(orig_w, orig_h, image::imageops::FilterType::Lanczos3);
                upscaled.save(&path)?;
            }

            return Ok(path);
        }
        Err(eyre::eyre!(
            "Dashscope rate limited after {max_retries} retries"
        ))
    }

    /// OCR an image using qwen-vl-ocr, returning word-level bounding boxes.
    /// Each result has `text` and `location` [x1,y1, x2,y2, x3,y3, x4,y4] in pixels.
    #[allow(dead_code)]
    pub fn ocr_image(&self, image_path: &Path) -> Result<Vec<OcrWord>> {
        let img_data = std::fs::read(image_path)?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&img_data);
        let ext = image_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let mime = if ext == "jpg" || ext == "jpeg" {
            "image/jpeg"
        } else {
            "image/png"
        };
        let data_uri = format!("data:{mime};base64,{b64}");

        let body = json!({
            "model": "qwen-vl-ocr",
            "input": {
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            {
                                "image": data_uri,
                                "min_pixels": 3072,
                                "max_pixels": 8388608
                            },
                            {
                                "type": "text",
                                "text": "Read all text in this image."
                            }
                        ]
                    }
                ]
            },
            "parameters": {
                "ocr_options": {
                    "task": "advanced_recognition"
                }
            }
        });

        let max_retries = 3;
        for attempt in 0..=max_retries {
            let resp = self
                .http
                .post(&self.endpoint)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?;

            let data: Value = resp.json()?;

            if let Some(code) = data.get("code").and_then(|c| c.as_str()) {
                if (code.contains("RateQuota") || code.contains("Throttling"))
                    && attempt < max_retries
                {
                    let wait = 10 * (attempt + 1) as u64;
                    eprintln!("  OCR: rate limited, retrying in {wait}s...");
                    std::thread::sleep(std::time::Duration::from_secs(wait));
                    continue;
                }
            }

            // Parse words_info from response
            let mut words = Vec::new();
            if let Some(choices) = data.pointer("/output/choices").and_then(|c| c.as_array()) {
                for choice in choices {
                    if let Some(content) = choice
                        .pointer("/message/content")
                        .and_then(|c| c.as_array())
                    {
                        for item in content {
                            if let Some(ocr) = item.get("ocr_result") {
                                if let Some(infos) =
                                    ocr.get("words_info").and_then(|w| w.as_array())
                                {
                                    for info in infos {
                                        let text = info
                                            .get("text")
                                            .and_then(|t| t.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let loc: Vec<f64> = info
                                            .get("location")
                                            .and_then(|l| l.as_array())
                                            .map(|arr| {
                                                arr.iter().filter_map(|v| v.as_f64()).collect()
                                            })
                                            .unwrap_or_default();
                                        if loc.len() == 8 && !text.is_empty() {
                                            words.push(OcrWord {
                                                text,
                                                x1: loc[0],
                                                y1: loc[1],
                                                x2: loc[2],
                                                y2: loc[3],
                                                x3: loc[4],
                                                y3: loc[5],
                                                x4: loc[6],
                                                y4: loc[7],
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            eprintln!("  OCR: detected {} text regions", words.len());
            return Ok(words);
        }
        Err(eyre::eyre!("OCR rate limited after {max_retries} retries"))
    }

    /// Remove text from an image using OCR + programmatic background fill.
    ///
    /// For each OCR-detected text region, samples the surrounding background color
    /// and fills the text area with it. This is deterministic, fast, and avoids
    /// AI inpainting artifacts on dense slides.
    #[allow(dead_code)]
    pub fn remove_text(&self, image_path: &Path, out_file: &Path) -> Result<std::path::PathBuf> {
        // Step 1: OCR to find all text regions
        let words = self.ocr_image(image_path)?;
        if words.is_empty() {
            eprintln!("  remove_text: OCR found no text, copying original");
            std::fs::copy(image_path, out_file)?;
            return Ok(out_file.to_path_buf());
        }
        eprintln!(
            "  remove_text: OCR found {} text regions, filling with background...",
            words.len()
        );

        // Step 2: Load image
        let mut img = image::ImageReader::open(image_path)?
            .with_guessed_format()?
            .decode()?
            .to_rgb8();
        let (iw, ih) = (img.width(), img.height());

        // Step 3: For each word, sample background color and fill
        let padding_factor = 0.15;
        for word in &words {
            let pad_x = word.width() * padding_factor;
            let pad_y = word.height() * padding_factor;
            let x0 = (word.left() - pad_x).max(0.0) as u32;
            let y0 = (word.top() - pad_y).max(0.0) as u32;
            let x1 = (word.right() + pad_x).min(iw as f64 - 1.0) as u32;
            let y1 = (word.bottom() + pad_y).min(ih as f64 - 1.0) as u32;

            // Sample background color from border pixels around the bbox
            let bg = sample_border_color(&img, x0, y0, x1, y1);

            // Fill the text region with background color
            for y in y0..=y1 {
                for x in x0..=x1 {
                    img.put_pixel(x, y, bg);
                }
            }
        }

        img.save(out_file)?;
        eprintln!(
            "  remove_text: done — {} (filled {} regions)",
            out_file.file_name().unwrap().to_string_lossy(),
            words.len()
        );
        Ok(out_file.to_path_buf())
    }

    /// Generate an image from a text prompt using Qwen-Image (text-to-image).
    pub fn gen_image(
        &self,
        prompt: &str,
        out_file: &Path,
        model: Option<&str>,
        size: Option<&str>,
    ) -> Result<std::path::PathBuf> {
        let model = model.unwrap_or("qwen-image-2.0-pro");

        // Determine size string (width*height) from mofa size notation
        let size_str = match size {
            Some("4K") => "2048*1152", // 16:9 max for qwen
            Some("2K") => "1344*768",
            Some("1K") => "1024*576",
            _ => "1344*768",
        };

        let body = json!({
            "model": model,
            "input": {
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            { "text": prompt }
                        ]
                    }
                ]
            },
            "parameters": {
                "n": 1,
                "watermark": false,
                "prompt_extend": false,
                "size": size_str
            }
        });

        let max_retries = 3;
        for attempt in 0..=max_retries {
            eprintln!("  Dashscope gen [{model}]: submitting...");

            let resp = self
                .http
                .post(&self.endpoint)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?;

            let data: serde_json::Value = resp.json()?;

            if let Some(code) = data.get("code").and_then(|c| c.as_str()) {
                if (code.contains("RateQuota") || code.contains("Throttling"))
                    && attempt < max_retries
                {
                    let wait = 10 * (attempt + 1) as u64;
                    eprintln!("  Dashscope gen: rate limited, retrying in {wait}s...");
                    std::thread::sleep(std::time::Duration::from_secs(wait));
                    continue;
                }
            }

            let img_url = data
                .pointer("/output/choices/0/message/content/0/image")
                .and_then(|u| u.as_str())
                .ok_or_else(|| eyre::eyre!("Dashscope gen failed: {data}"))?;

            return self.download_result(img_url, out_file);
        }
        Err(eyre::eyre!(
            "Dashscope gen rate limited after {max_retries} retries"
        ))
    }

    fn download_result(&self, img_url: &str, out_file: &Path) -> Result<std::path::PathBuf> {
        let img_resp = self.http.get(img_url).send()?;
        let bytes = img_resp.bytes()?;
        if let Some(parent) = out_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(out_file, &bytes)?;
        eprintln!(
            "  Dashscope: done — {} ({}KB)",
            out_file.file_name().unwrap().to_string_lossy(),
            bytes.len() / 1024
        );
        Ok(out_file.to_path_buf())
    }
}
