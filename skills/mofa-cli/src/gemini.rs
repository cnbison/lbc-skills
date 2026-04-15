// SPDX-License-Identifier: Apache-2.0

use eyre::{Result, WrapErr};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const DEFAULT_GEN_MODEL: &str = "gemini-3.1-flash-image-preview";
const CACHE_THRESHOLD: u64 = 10_000; // 10KB

// Batch API constants
const BATCH_POLL_INITIAL_SECS: f64 = 5.0;
const BATCH_POLL_MAX_SECS: f64 = 30.0;
const BATCH_POLL_MULTIPLIER: f64 = 1.5;
const BATCH_MAX_WAIT_SECS: u64 = 1800; // 30 minutes

/// A single image generation request for batch submission.
pub struct BatchImageRequest {
    pub key: String,
    pub prompt: String,
    pub out_file: PathBuf,
    pub image_size: Option<String>,
    pub aspect_ratio: Option<String>,
    pub ref_images: Vec<PathBuf>,
    pub model: String,
}

/// Build content parts array (ref images as base64 + text prompt).
fn build_content_parts(prompt: &str, ref_images: &[PathBuf]) -> Result<Vec<Value>> {
    let mut parts = Vec::new();
    for img_path in ref_images {
        let data = std::fs::read(img_path)
            .wrap_err_with(|| format!("reading ref image: {}", img_path.display()))?;
        let ext = img_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let mime = if ext == "jpg" || ext == "jpeg" {
            "image/jpeg"
        } else {
            "image/png"
        };
        parts.push(json!({
            "inlineData": {
                "mimeType": mime,
                "data": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data),
            }
        }));
    }
    parts.push(json!({ "text": prompt }));
    Ok(parts)
}

/// Build content parts from borrowed Path slices (for gen_image compatibility).
fn build_content_parts_borrowed(prompt: &str, ref_images: &[&Path]) -> Result<Vec<Value>> {
    let owned: Vec<PathBuf> = ref_images.iter().map(|p| p.to_path_buf()).collect();
    build_content_parts(prompt, &owned)
}

/// Build generationConfig JSON.
fn build_generation_config(image_size: Option<&str>, aspect_ratio: Option<&str>) -> Value {
    let mut config = json!({
        "responseModalities": ["IMAGE", "TEXT"]
    });
    if image_size.is_some() || aspect_ratio.is_some() {
        let mut img_config = json!({});
        if let Some(ar) = aspect_ratio {
            img_config["aspectRatio"] = json!(ar);
        }
        if let Some(size) = image_size {
            img_config["imageSize"] = json!(size);
        }
        config["imageConfig"] = img_config;
    }
    config
}

/// Check if a file is cached (exists and >10KB).
fn is_cached(path: &Path) -> bool {
    if path.exists() {
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.len() > CACHE_THRESHOLD;
        }
    }
    false
}

/// Extract the first image from a generateContent response parts array.
fn extract_image_from_parts(parts: &[Value]) -> Option<Vec<u8>> {
    for part in parts {
        if let Some(inline) = part.get("inlineData") {
            if let Some(b64) = inline.get("data").and_then(|d| d.as_str()) {
                if let Ok(bytes) =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
                {
                    return Some(bytes);
                }
            }
        }
    }
    None
}

/// Default Gemini API base URL.
const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Gemini API client for image generation and vision QA.
pub struct GeminiClient {
    api_key: String,
    base_url: String,
    http: reqwest::blocking::Client,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        let base_url = std::env::var("GEMINI_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();
        Self {
            api_key,
            base_url,
            http: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
        }
    }

    /// Sanitize error messages to avoid leaking API key.
    fn sanitize(&self, msg: &str) -> String {
        let safe = msg.replace(&self.api_key, "[REDACTED]");
        safe.chars().take(200).collect()
    }

    /// Generate an image via Gemini `generateContent` with IMAGE response modality.
    ///
    /// Returns the output file path on success, None on failure after retries.
    #[allow(clippy::too_many_arguments)]
    pub fn gen_image(
        &self,
        prompt: &str,
        out_file: &Path,
        image_size: Option<&str>,
        aspect_ratio: Option<&str>,
        ref_images: &[&Path],
        model: Option<&str>,
        label: Option<&str>,
    ) -> Result<Option<std::path::PathBuf>> {
        let tag =
            label.unwrap_or_else(|| out_file.file_stem().unwrap().to_str().unwrap_or("image"));
        let model = model.unwrap_or(DEFAULT_GEN_MODEL);

        if is_cached(out_file) {
            eprintln!("Cached: {tag}");
            return Ok(Some(out_file.to_path_buf()));
        }

        let config = build_generation_config(image_size, aspect_ratio);
        let parts = build_content_parts_borrowed(prompt, ref_images)?;

        let url = format!(
            "{}/models/{model}:generateContent?key={}",
            self.base_url, self.api_key
        );

        let body = json!({
            "contents": [{ "role": "user", "parts": parts }],
            "generationConfig": config,
        });

        for attempt in 1..=3 {
            match self.http.post(&url).json(&body).send() {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<Value>() {
                        if let Some(parts) = data
                            .pointer("/candidates/0/content/parts")
                            .and_then(|p| p.as_array())
                        {
                            if let Some(bytes) = extract_image_from_parts(parts) {
                                if let Some(parent) = out_file.parent() {
                                    std::fs::create_dir_all(parent).ok();
                                }
                                std::fs::write(out_file, &bytes)?;
                                eprintln!("{tag} [{model}]: {}KB", bytes.len() / 1024);
                                return Ok(Some(out_file.to_path_buf()));
                            }
                        }
                        eprintln!("{tag}: no image, attempt {attempt}/3");
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{tag}: error {attempt}/3 — {}",
                        self.sanitize(&format!("{e}"))
                    );
                }
            }
            if attempt < 3 {
                std::thread::sleep(std::time::Duration::from_secs(15));
            }
        }
        eprintln!("{tag}: FAILED after 3 attempts");
        Ok(None)
    }

    /// Edit an existing image: send image + text prompt, get modified image back.
    pub fn edit_image(
        &self,
        image_path: &Path,
        prompt: &str,
        out_file: &Path,
        model: Option<&str>,
        label: Option<&str>,
    ) -> Result<Option<std::path::PathBuf>> {
        let tag = label.unwrap_or("edit");
        let model = model.unwrap_or(DEFAULT_GEN_MODEL);

        if is_cached(out_file) {
            eprintln!("Cached: {tag}");
            return Ok(Some(out_file.to_path_buf()));
        }

        let img_data = std::fs::read(image_path)?;
        let ext = image_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let mime = if ext == "jpg" || ext == "jpeg" {
            "image/jpeg"
        } else {
            "image/png"
        };
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &img_data);

        let parts = vec![
            json!({ "inlineData": { "mimeType": mime, "data": b64 } }),
            json!({ "text": prompt }),
        ];

        let config = json!({ "responseModalities": ["IMAGE", "TEXT"] });
        let url = format!(
            "{}/models/{model}:generateContent?key={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "contents": [{ "role": "user", "parts": parts }],
            "generationConfig": config,
        });

        for attempt in 1..=3 {
            match self.http.post(&url).json(&body).send() {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<Value>() {
                        if let Some(parts) = data
                            .pointer("/candidates/0/content/parts")
                            .and_then(|p| p.as_array())
                        {
                            if let Some(bytes) = extract_image_from_parts(parts) {
                                if let Some(parent) = out_file.parent() {
                                    std::fs::create_dir_all(parent).ok();
                                }
                                std::fs::write(out_file, &bytes)?;
                                eprintln!("{tag} [{model}]: {}KB", bytes.len() / 1024);
                                return Ok(Some(out_file.to_path_buf()));
                            }
                        }
                        eprintln!("{tag}: no image, attempt {attempt}/3");
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{tag}: error {attempt}/3 — {}",
                        self.sanitize(&format!("{e}"))
                    );
                }
            }
            if attempt < 3 {
                std::thread::sleep(std::time::Duration::from_secs(15));
            }
        }
        eprintln!("{tag}: FAILED after 3 attempts");
        Ok(None)
    }

    /// Generate multiple images via the Gemini Batch API.
    ///
    /// Submits all uncached requests as a batch, polls for completion, writes results.
    /// Returns a Vec with the same length as `requests`, with `Some(path)` for successes.
    pub fn batch_gen_images(
        &self,
        requests: Vec<BatchImageRequest>,
    ) -> Result<Vec<Option<PathBuf>>> {
        let total = requests.len();
        let mut results: Vec<Option<PathBuf>> = vec![None; total];

        // Step 1: Cache filter
        let mut uncached: Vec<(usize, &BatchImageRequest)> = Vec::new();
        for (i, req) in requests.iter().enumerate() {
            if is_cached(&req.out_file) {
                eprintln!("Cached: {}", req.key);
                results[i] = Some(req.out_file.clone());
            } else {
                uncached.push((i, req));
            }
        }

        if uncached.is_empty() {
            eprintln!("All {total} items cached, skipping batch.");
            return Ok(results);
        }

        eprintln!("Batch: {}/{total} items need generation", uncached.len());

        // Step 2: Group by model
        let mut by_model: HashMap<String, Vec<(usize, &BatchImageRequest)>> = HashMap::new();
        for (i, req) in &uncached {
            by_model
                .entry(req.model.clone())
                .or_default()
                .push((*i, req));
        }

        // Step 3: Submit + poll for each model group
        for (model, group) in &by_model {
            let batch_requests: Vec<Value> = group
                .iter()
                .map(|(_, req)| {
                    let parts = build_content_parts(&req.prompt, &req.ref_images)?;
                    let config = build_generation_config(
                        req.image_size.as_deref(),
                        req.aspect_ratio.as_deref(),
                    );
                    Ok(json!({
                        "request": {
                            "contents": [{"role": "user", "parts": parts}],
                            "generationConfig": config,
                        },
                        "metadata": {"key": req.key}
                    }))
                })
                .collect::<Result<Vec<_>>>()?;

            let url = format!("{}/models/{model}:batchGenerateContent", self.base_url);

            let body = json!({
                "batch": {
                    "display_name": format!("mofa-{}", std::process::id()),
                    "input_config": {
                        "requests": {
                            "requests": batch_requests
                        }
                    }
                }
            });

            // Submit batch
            let resp = self
                .http
                .post(&url)
                .header("x-goog-api-key", &self.api_key)
                .json(&body)
                .send()
                .map_err(|e| eyre::eyre!("Batch submit: {}", self.sanitize(&format!("{e}"))))?;

            let batch_info: Value = resp.json()?;
            let batch_name = batch_info
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| {
                    eyre::eyre!(
                        "No batch name in response: {}",
                        serde_json::to_string_pretty(&batch_info)
                            .unwrap_or_default()
                            .chars()
                            .take(500)
                            .collect::<String>()
                    )
                })?
                .to_string();

            eprintln!(
                "Batch submitted: {batch_name} ({} requests, model={model})",
                group.len()
            );

            // Step 4: Poll with exponential backoff
            let mut interval = BATCH_POLL_INITIAL_SECS;
            let mut elapsed = 0u64;

            let status = loop {
                std::thread::sleep(std::time::Duration::from_secs(interval as u64));
                elapsed += interval as u64;

                let poll_url = format!("{}/{batch_name}?key={}", self.base_url, self.api_key);
                let poll_resp =
                    self.http.get(&poll_url).send().map_err(|e| {
                        eyre::eyre!("Batch poll: {}", self.sanitize(&format!("{e}")))
                    })?;
                let poll_data: Value = poll_resp.json()?;

                let state = poll_data
                    .pointer("/metadata/state")
                    .and_then(|s| s.as_str())
                    .unwrap_or("UNKNOWN");

                eprint!("\rBatch [{model}]: {state} ({elapsed}s)   ");

                match state {
                    "JOB_STATE_SUCCEEDED" | "BATCH_STATE_SUCCEEDED" => {
                        eprintln!();
                        break poll_data;
                    }
                    "JOB_STATE_FAILED"
                    | "BATCH_STATE_FAILED"
                    | "JOB_STATE_CANCELLED"
                    | "BATCH_STATE_CANCELLED"
                    | "JOB_STATE_EXPIRED"
                    | "BATCH_STATE_EXPIRED" => {
                        eprintln!();
                        return Err(eyre::eyre!("Batch {state}"));
                    }
                    _ => {
                        if elapsed >= BATCH_MAX_WAIT_SECS {
                            eprintln!();
                            return Err(eyre::eyre!("Batch timed out after {elapsed}s"));
                        }
                        interval = (interval * BATCH_POLL_MULTIPLIER).min(BATCH_POLL_MAX_SECS);
                    }
                }
            };

            // Step 5: Extract results
            let key_to_idx: HashMap<&str, usize> = group
                .iter()
                .map(|(i, req)| (req.key.as_str(), *i))
                .collect();

            if let Some(responses) = status
                .pointer("/metadata/output/inlinedResponses/inlinedResponses")
                .and_then(|r| r.as_array())
            {
                for inlined in responses {
                    let key = inlined
                        .pointer("/metadata/key")
                        .and_then(|k| k.as_str())
                        .unwrap_or("");

                    if let Some(&orig_idx) = key_to_idx.get(key) {
                        if let Some(parts) = inlined
                            .pointer("/response/candidates/0/content/parts")
                            .and_then(|p| p.as_array())
                        {
                            if let Some(bytes) = extract_image_from_parts(parts) {
                                let out = &requests[orig_idx].out_file;
                                if let Some(parent) = out.parent() {
                                    std::fs::create_dir_all(parent).ok();
                                }
                                std::fs::write(out, &bytes)?;
                                eprintln!("{key}: {}KB", bytes.len() / 1024);
                                results[orig_idx] = Some(out.clone());
                            }
                        }
                    }
                }
            }
        }

        let ok = results.iter().filter(|r| r.is_some()).count();
        eprintln!("Batch complete: {ok}/{total} images");
        Ok(results)
    }

    /// Vision QA: send image to Gemini vision model and get structured JSON response.
    pub fn vision_qa(&self, image_path: &Path, prompt: &str, model: Option<&str>) -> Result<Value> {
        let model = model.unwrap_or("gemini-3.1-flash-image-preview");
        let img_data = std::fs::read(image_path)?;
        let ext = image_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let mime = if ext == "jpg" || ext == "jpeg" {
            "image/jpeg"
        } else {
            "image/png"
        };

        let url = format!(
            "{}/models/{model}:generateContent?key={}",
            self.base_url, self.api_key
        );

        let body = json!({
            "contents": [{
                "role": "user",
                "parts": [
                    {
                        "inlineData": {
                            "mimeType": mime,
                            "data": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &img_data),
                        }
                    },
                    { "text": prompt }
                ]
            }],
            "generationConfig": {
                "responseMimeType": "application/json"
            }
        });

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| eyre::eyre!("{}", self.sanitize(&format!("{e}"))))?;
        let data: Value = resp.json()?;

        let raw = data
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(|t| t.as_str())
            .ok_or_else(|| eyre::eyre!("Vision QA returned no text"))?;

        let parsed: Value = serde_json::from_str(raw)?;
        Ok(parsed)
    }
}
