// SPDX-License-Identifier: Apache-2.0

use eyre::Result;
use serde_json::Value;
use std::path::Path;

/// A text block detected by DeepSeek-OCR-2 with grounding.
/// Bounding box coordinates are normalized to [0, 1000) range.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OcrBlock {
    pub text: String,
    pub block_type: String, // "text", "sub_title", "title", etc.
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

#[allow(dead_code)]
impl OcrBlock {
    /// Convert normalized coords to pixel coords given image dimensions.
    pub fn to_pixels(&self, img_w: f64, img_h: f64) -> (f64, f64, f64, f64) {
        (
            self.x1 / 1000.0 * img_w,
            self.y1 / 1000.0 * img_h,
            self.x2 / 1000.0 * img_w,
            self.y2 / 1000.0 * img_h,
        )
    }

    /// Width in normalized coords.
    pub fn width(&self) -> f64 {
        self.x2 - self.x1
    }

    /// Height in normalized coords.
    pub fn height(&self) -> f64 {
        self.y2 - self.y1
    }
}

/// Client for a local DeepSeek-OCR-2 inference endpoint.
pub struct DeepSeekOcrClient {
    endpoint: String,
    http: reqwest::blocking::Client,
}

impl DeepSeekOcrClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            http: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(600))
                .build()
                .unwrap(),
        }
    }

    /// OCR with grounding: returns text blocks with bounding boxes.
    pub fn ocr_with_grounding(&self, image_path: &Path) -> Result<Vec<OcrBlock>> {
        let img_data = std::fs::read(image_path)?;
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &img_data);

        let body = serde_json::json!({
            "image": b64,
            "prompt": "<|grounding|>Convert the document to markdown.",
            "max_tokens": 16384,
            "repetition_penalty": 1.3,
        });

        eprintln!(
            "  DeepSeek-OCR: sending image ({} KB)...",
            img_data.len() / 1024
        );

        let resp = self.http.post(&self.endpoint).json(&body).send()?;

        let data: Value = resp.json()?;

        let content = data
            .pointer("/choices/0/message/content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| eyre::eyre!("DeepSeek-OCR returned no content: {data}"))?;

        let blocks = parse_grounding_output(content);
        eprintln!("  DeepSeek-OCR: detected {} text blocks", blocks.len());
        Ok(blocks)
    }
}

/// Parse DeepSeek-OCR-2 grounding output format.
///
/// Format:
/// ```text
/// type[[x1, y1, x2, y2]]
/// text content
/// possibly multiline
///
/// type[[x1, y1, x2, y2]]
/// next block
/// ```
///
/// Coordinates are in normalized [0, 1000) range.
fn parse_grounding_output(content: &str) -> Vec<OcrBlock> {
    let mut blocks = Vec::new();
    let mut lines = content.lines().peekable();

    // Skip any leading text before the first grounding tag
    while let Some(line) = lines.peek() {
        if line.contains("[[") && line.contains("]]") {
            break;
        }
        lines.next();
    }

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Try to parse a grounding header: type[[x1, y1, x2, y2]]
        if let Some(block_header) = parse_block_header(line) {
            // Collect text lines until next header or empty line
            let mut text_lines = Vec::new();
            while let Some(next_line) = lines.peek() {
                let next = next_line.trim();
                if next.is_empty() {
                    lines.next();
                    break;
                }
                if next.contains("[[") && next.contains("]]") && parse_block_header(next).is_some()
                {
                    break;
                }
                text_lines.push(next.to_string());
                lines.next();
            }

            let text = text_lines.join("\n");
            // Strip markdown heading markers (##, ###) from text
            let text = text
                .lines()
                .map(|l| l.trim_start_matches('#').trim())
                .collect::<Vec<_>>()
                .join("\n");

            // Skip empty blocks and blocks with repetition artifacts
            if text.is_empty() || has_repetition(&text) {
                continue;
            }

            blocks.push(OcrBlock {
                text,
                block_type: block_header.0,
                x1: block_header.1,
                y1: block_header.2,
                x2: block_header.3,
                y2: block_header.4,
            });
        }
    }

    blocks
}

/// Parse a line like `text[[34, 93, 741, 137]]` or `sub_title[[34, 29, 333, 87]]`.
/// Also handles multi-bbox: `text[[564, 234, 623, 269], [681, 156, 843, 192]]`
/// by computing the union bounding box.
fn parse_block_header(line: &str) -> Option<(String, f64, f64, f64, f64)> {
    let bracket_start = line.find("[[")?;
    let block_type = line[..bracket_start].trim().to_string();

    // Extract all [x, y, x2, y2] groups using regex-like manual parsing
    let rest = &line[bracket_start..];
    let mut bboxes: Vec<(f64, f64, f64, f64)> = Vec::new();

    let mut i = 0;
    let chars: Vec<char> = rest.chars().collect();
    while i < chars.len() {
        if chars[i] == '[' && i + 1 < chars.len() && chars[i + 1] != '[' {
            // Found inner [, collect until ]
            let start = i + 1;
            if let Some(end) = rest[start..].find(']') {
                let coords_str = &rest[start..start + end];
                let coords: Vec<f64> = coords_str
                    .split(',')
                    .filter_map(|s| s.trim().parse::<f64>().ok())
                    .collect();
                if coords.len() == 4 {
                    bboxes.push((coords[0], coords[1], coords[2], coords[3]));
                }
                i = start + end + 1;
                continue;
            }
        }
        i += 1;
    }

    if bboxes.is_empty() {
        return None;
    }

    // Compute union bounding box
    let x1 = bboxes.iter().map(|b| b.0).fold(f64::INFINITY, f64::min);
    let y1 = bboxes.iter().map(|b| b.1).fold(f64::INFINITY, f64::min);
    let x2 = bboxes.iter().map(|b| b.2).fold(f64::NEG_INFINITY, f64::max);
    let y2 = bboxes.iter().map(|b| b.3).fold(f64::NEG_INFINITY, f64::max);

    Some((block_type, x1, y1, x2, y2))
}

/// Detect repetition artifacts in model output.
/// DeepSeek-OCR-2 sometimes enters a repetition loop on dense content.
fn has_repetition(text: &str) -> bool {
    // Check for LaTeX-style repetition like \( ^{2} \)  \( ^{2} \)
    if text.contains(r"\( ^{") && text.matches(r"\( ^{").count() > 3 {
        return true;
    }
    // Check for excessive character repetition (same substring repeated 5+ times)
    let chars: Vec<char> = text.chars().collect();
    if chars.len() > 50 {
        // Take first 20 chars as a chunk, check if it repeats in the second half
        let chunk: String = chars[..20.min(chars.len())].iter().collect();
        let mid_char = chars.len() / 2;
        let second_half: String = chars[mid_char..].iter().collect();
        if second_half.matches(&chunk).count() >= 3 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grounding_output() {
        let content = r#" 从高利润软件

sub_title[[34, 29, 333, 87]]
## AI编程重塑全栈竞争力

text[[34, 93, 741, 137]]
价值迁移：从高利润软件

text[[129, 156, 197, 191]]
软件市场
"#;
        let blocks = parse_grounding_output(content);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, "sub_title");
        assert_eq!(blocks[0].text, "AI编程重塑全栈竞争力");
        assert_eq!(blocks[0].x1, 34.0);
        assert_eq!(blocks[0].y1, 29.0);
        assert_eq!(blocks[1].text, "价值迁移：从高利润软件");
        assert_eq!(blocks[2].text, "软件市场");
    }

    #[test]
    fn test_parse_block_header() {
        let h = parse_block_header("text[[34, 93, 741, 137]]").unwrap();
        assert_eq!(h.0, "text");
        assert_eq!(h.1, 34.0);
        assert_eq!(h.4, 137.0);

        let h = parse_block_header("sub_title[[0, 0, 1000, 50]]").unwrap();
        assert_eq!(h.0, "sub_title");

        // Multi-bbox: union bounding box
        let h = parse_block_header("text[[564, 234, 623, 269], [681, 156, 843, 192]]").unwrap();
        assert_eq!(h.0, "text");
        assert_eq!(h.1, 564.0); // min x1
        assert_eq!(h.2, 156.0); // min y1
        assert_eq!(h.3, 843.0); // max x2
        assert_eq!(h.4, 269.0); // max y2
    }

    #[test]
    fn test_parse_skips_empty_text_blocks() {
        let content = "image[[82, 157, 229, 265]]\n\ntext[[100, 200, 300, 250]]\nhello\n";
        let blocks = parse_grounding_output(content);
        assert_eq!(blocks.len(), 1); // image block skipped (empty text)
        assert_eq!(blocks[0].text, "hello");
    }

    #[test]
    fn test_repetition_detection() {
        assert!(has_repetition(
            r"全球唯一 \( ^{2} \)  \( ^{2} \)  \( ^{2} \)  \( ^{2} \)"
        ));
        assert!(!has_repetition("Normal text without repetition"));
    }
}
