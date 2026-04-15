// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use eyre::{Result, WrapErr};
use std::collections::HashMap;
use std::path::Path;

/// A loaded TOML style with variant-based prompt lookup.
#[derive(Debug)]
pub struct Style {
    pub meta: Option<toml::Value>,
    variants: HashMap<String, String>,
    default_variant: String,
}

impl Style {
    /// Get the prompt for a variant tag, falling back to default.
    pub fn get_prompt(&self, tag: &str) -> &str {
        self.variants
            .get(tag)
            .or_else(|| self.variants.get(&self.default_variant))
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}

/// Load a single TOML style file.
pub fn load_style(path: &Path) -> Result<Style> {
    let content = std::fs::read_to_string(path)
        .wrap_err_with(|| format!("reading style: {}", path.display()))?;
    let parsed: toml::Value = content
        .parse::<toml::Value>()
        .wrap_err_with(|| format!("parsing TOML: {}", path.display()))?;

    let meta = parsed.get("meta").cloned();

    let mut variants = HashMap::new();
    let mut default_variant = "normal".to_string();

    if let Some(v) = parsed.get("variants").and_then(|v| v.as_table()) {
        if let Some(d) = v.get("default").and_then(|d| d.as_str()) {
            default_variant = d.to_string();
        }
        for (key, val) in v {
            if key == "default" {
                continue;
            }
            if let Some(tbl) = val.as_table() {
                if let Some(prompt) = tbl.get("prompt").and_then(|p| p.as_str()) {
                    variants.insert(key.clone(), prompt.to_string());
                }
            }
        }
    }

    Ok(Style {
        meta,
        variants,
        default_variant,
    })
}

/// Load all TOML style files from a directory into a name→Style catalog.
pub fn load_style_dir(dir: &Path) -> Result<HashMap<String, Style>> {
    let mut catalog = HashMap::new();
    if !dir.exists() {
        return Ok(catalog);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "toml").unwrap_or(false) {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            match load_style(&path) {
                Ok(style) => {
                    catalog.insert(name, style);
                }
                Err(e) => {
                    eprintln!("warning: skipping {}: {e}", path.display());
                }
            }
        }
    }
    Ok(catalog)
}
