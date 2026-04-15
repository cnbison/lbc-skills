// SPDX-License-Identifier: Apache-2.0

use eyre::{bail, Result};
use serde_json::{json, Value};
use std::path::Path;

const POLL_INTERVAL_SECS: u64 = 10;
const MAX_POLLS: usize = 120; // 20min timeout

/// Default Gemini API base URL (shared with GeminiClient).
const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Gemini Veo video generation client.
pub struct VeoClient {
    api_key: String,
    base_url: String,
    http: reqwest::blocking::Client,
}

impl VeoClient {
    pub fn new(api_key: String) -> Self {
        let base_url = std::env::var("GEMINI_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();
        Self {
            api_key,
            base_url,
            http: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    /// Generate a video from an image using Veo.
    /// Returns path to downloaded MP4.
    pub fn generate_video(
        &self,
        image_path: &Path,
        prompt: &str,
        out_file: &Path,
        model: Option<&str>,
    ) -> Result<std::path::PathBuf> {
        let model = model.unwrap_or("veo-3.1-generate-preview");

        // Cache check
        if out_file.exists() {
            if let Ok(meta) = std::fs::metadata(out_file) {
                if meta.len() > 10_000 {
                    eprintln!("Using cached video: {}", out_file.display());
                    return Ok(out_file.to_path_buf());
                }
            }
        }

        let img_data = std::fs::read(image_path)?;
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &img_data);

        // Submit video generation
        let url = format!(
            "{}/models/{model}:generateVideos?key={}",
            self.base_url, self.api_key
        );

        let body = json!({
            "prompt": prompt,
            "image": {
                "imageBytes": b64,
                "mimeType": "image/png"
            }
        });

        let resp = self.http.post(&url).json(&body).send()?;
        let mut operation: Value = resp.json()?;

        // Poll until done
        let op_name = operation
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| eyre::eyre!("No operation name in Veo response: {operation}"))?
            .to_string();

        eprintln!("Veo: generating video...");
        for i in 0..MAX_POLLS {
            if operation
                .get("done")
                .and_then(|d| d.as_bool())
                .unwrap_or(false)
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS));

            let poll_url = format!("{}/{}?key={}", self.base_url, op_name, self.api_key);
            let poll_resp = self.http.get(&poll_url).send()?;
            operation = poll_resp.json()?;

            eprint!("\rVeo: generating{} ", ".".repeat((i % 4) + 1));
        }
        eprintln!();

        if !operation
            .get("done")
            .and_then(|d| d.as_bool())
            .unwrap_or(false)
        {
            bail!("Veo video generation timed out");
        }

        // Download the video
        let video_uri = operation
            .pointer("/response/generatedVideos/0/video/uri")
            .and_then(|u| u.as_str())
            .ok_or_else(|| eyre::eyre!("No video URI in Veo response"))?;

        // Use the Files API to download
        let download_url = format!("{video_uri}?key={}", self.api_key);
        let dl_resp = self.http.get(&download_url).send()?;
        let bytes = dl_resp.bytes()?;

        if let Some(parent) = out_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(out_file, &bytes)?;
        eprintln!("Veo: video saved ({}MB)", bytes.len() / 1024 / 1024);
        Ok(out_file.to_path_buf())
    }
}
