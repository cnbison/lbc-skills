// SPDX-License-Identifier: Apache-2.0

mod config;
mod dashscope;
mod deepseek_ocr;
mod gemini;
mod image_util;
mod layout;
mod pipeline;
mod pptx;
mod style;
mod veo;

use clap::{Parser, Subcommand, ValueEnum};
use eyre::Result;
use std::io::Read;
use std::path::PathBuf;

/// Gemini API mode for image generation.
#[derive(Clone, Debug, ValueEnum)]
enum ApiMode {
    /// Realtime: parallel sync calls, fast (~2-3 min)
    Rt,
    /// Batch API: 50% cheaper, async (may take 5-30 min)
    Batch,
}

#[derive(Parser)]
#[command(name = "mofa", about = "AI-powered content generation CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to mofa root directory (auto-detected if omitted)
    #[arg(long, global = true)]
    root: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a multi-slide PPTX presentation
    Slides {
        /// Style name (from styles/ directory)
        #[arg(long, default_value = "nb-pro")]
        style: String,
        /// Output PPTX file
        #[arg(long, short)]
        out: PathBuf,
        /// Directory for intermediate slide PNGs
        #[arg(long)]
        slide_dir: PathBuf,
        /// Parallel generation limit
        #[arg(long, default_value = "5")]
        concurrency: usize,
        /// Image size: 1K, 2K, 4K
        #[arg(long)]
        image_size: Option<String>,
        /// Gemini model override
        #[arg(long)]
        gen_model: Option<String>,
        /// Lower-res image size for autoLayout Phase 1 reference images
        #[arg(long)]
        ref_image_size: Option<String>,
        /// Vision model for autoLayout text extraction
        #[arg(long)]
        vision_model: Option<String>,
        /// Enable editable text mode: extract text, clean background, overlay text boxes
        #[arg(long)]
        auto_layout: bool,
        /// Use Qwen-Edit to remove text from reference images (cleaner output)
        #[arg(long)]
        refine: bool,
        /// API mode: rt (realtime, default) or batch (50% cheaper, async)
        #[arg(long, value_enum, default_value = "rt")]
        api: ApiMode,
        /// Input JSON file (or stdin if omitted)
        #[arg(long, short)]
        input: Option<PathBuf>,
    },
    /// Generate PNG greeting/holiday cards
    Cards {
        /// Style name
        #[arg(long, default_value = "cny-guochao")]
        style: String,
        /// Directory for card PNGs
        #[arg(long)]
        card_dir: PathBuf,
        /// Aspect ratio
        #[arg(long)]
        aspect: Option<String>,
        /// Parallel generation limit
        #[arg(long, default_value = "5")]
        concurrency: usize,
        /// Image size: 1K, 2K, 4K
        #[arg(long)]
        image_size: Option<String>,
        /// API mode: rt (realtime, default) or batch (50% cheaper, async)
        #[arg(long, value_enum, default_value = "rt")]
        api: ApiMode,
        /// Input JSON file (or stdin)
        #[arg(long, short)]
        input: Option<PathBuf>,
    },
    /// Generate a multi-panel comic strip
    Comic {
        /// Style name
        #[arg(long, default_value = "xkcd")]
        style: String,
        /// Output PNG file
        #[arg(long, short)]
        out: PathBuf,
        /// Working directory for panel PNGs
        #[arg(long)]
        work_dir: Option<PathBuf>,
        /// Layout: horizontal, vertical, grid
        #[arg(long, default_value = "horizontal")]
        layout: String,
        /// Parallel generation limit
        #[arg(long, default_value = "3")]
        concurrency: usize,
        /// Image size
        #[arg(long)]
        image_size: Option<String>,
        /// Refine panels with Qwen-Edit
        #[arg(long)]
        refine: bool,
        /// Gap between panels in pixels
        #[arg(long, default_value = "20")]
        gutter: u32,
        /// API mode: rt (realtime, default) or batch (50% cheaper, async)
        #[arg(long, value_enum, default_value = "rt")]
        api: ApiMode,
        /// Input JSON file (or stdin)
        #[arg(long, short)]
        input: Option<PathBuf>,
    },
    /// Generate a multi-section infographic
    Infographic {
        /// Style name
        #[arg(long, default_value = "cyberpunk-neon")]
        style: String,
        /// Output PNG file
        #[arg(long, short)]
        out: PathBuf,
        /// Working directory for section PNGs
        #[arg(long)]
        work_dir: Option<PathBuf>,
        /// Parallel generation limit
        #[arg(long, default_value = "3")]
        concurrency: usize,
        /// Image size
        #[arg(long)]
        image_size: Option<String>,
        /// Aspect ratio per section
        #[arg(long)]
        aspect: Option<String>,
        /// Refine sections with Qwen-Edit
        #[arg(long)]
        refine: bool,
        /// Gap between sections in pixels
        #[arg(long, default_value = "0")]
        gutter: u32,
        /// API mode: rt (realtime, default) or batch (50% cheaper, async)
        #[arg(long, value_enum, default_value = "rt")]
        api: ApiMode,
        /// Input JSON file (or stdin)
        #[arg(long, short)]
        input: Option<PathBuf>,
    },
    /// Unpack a PPTX/DOCX/XLSX into a directory of XML files
    PptxUnpack {
        /// Input Office file (.pptx, .docx, .xlsx)
        input: PathBuf,
        /// Output directory
        output_dir: PathBuf,
    },
    /// Pack a directory of XML files back into a PPTX/DOCX/XLSX
    PptxPack {
        /// Input directory (unpacked Office document)
        input_dir: PathBuf,
        /// Output Office file (.pptx, .docx, .xlsx)
        output: PathBuf,
    },
    /// Generate animated video cards with Veo
    Video {
        /// Image style name
        #[arg(long, default_value = "video-card")]
        style: String,
        /// Animation style name
        #[arg(long, default_value = "shuimo")]
        anim_style: String,
        /// Directory for PNGs and MP4s
        #[arg(long)]
        card_dir: PathBuf,
        /// Background music file
        #[arg(long)]
        bgm: Option<PathBuf>,
        /// Aspect ratio for images
        #[arg(long, default_value = "9:16")]
        aspect: String,
        /// Image size
        #[arg(long)]
        image_size: Option<String>,
        /// Parallel limit for image gen
        #[arg(long, default_value = "3")]
        concurrency: usize,
        /// Still image duration (seconds)
        #[arg(long, default_value = "2.0")]
        still_duration: f64,
        /// Crossfade duration (seconds)
        #[arg(long, default_value = "1.0")]
        crossfade_dur: f64,
        /// Fade out duration (seconds)
        #[arg(long, default_value = "1.5")]
        fade_out_dur: f64,
        /// Music volume (0.0-1.0)
        #[arg(long, default_value = "0.3")]
        music_volume: f64,
        /// Music fade in duration (seconds)
        #[arg(long, default_value = "2.0")]
        music_fade_in: f64,
        /// API mode: rt (realtime, default) or batch (50% cheaper, async)
        #[arg(long, value_enum, default_value = "rt")]
        api: ApiMode,
        /// Input JSON file (or stdin)
        #[arg(long, short)]
        input: Option<PathBuf>,
    },
}

fn read_input(path: Option<&PathBuf>) -> Result<String> {
    match path {
        Some(p) => {
            // JS input support: if the file ends in .js, evaluate it with Node
            // and capture the JSON output. The JS file should module.exports an array.
            if p.extension().is_some_and(|ext| ext == "js") {
                eprintln!("JS input: evaluating {} with Node...", p.display());
                let abs = std::fs::canonicalize(p)?;
                let script = format!(
                    "const fs=require('fs');\
                     const m={{}};\
                     const module={{exports:m}};\
                     eval(fs.readFileSync({0},'utf8'));\
                     console.log(JSON.stringify(module.exports));",
                    serde_json::to_string(&abs.to_string_lossy().as_ref())?
                );
                let output = std::process::Command::new("node")
                    .arg("-e")
                    .arg(&script)
                    .output()
                    .map_err(|e| eyre::eyre!("failed to run node: {e}. Is Node.js installed?"))?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eyre::bail!("node failed: {stderr}");
                }
                Ok(String::from_utf8(output.stdout)?)
            } else {
                Ok(std::fs::read_to_string(p)?)
            }
        }
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn find_styles_dir(mofa_root: &std::path::Path, skill_name: &str) -> PathBuf {
    // Try mofa-<skill>/styles/ first, then mofa/styles/
    let skill_styles = mofa_root.join(format!("mofa-{skill_name}")).join("styles");
    if skill_styles.exists() {
        return skill_styles;
    }
    mofa_root.join("mofa").join("styles")
}

/// Plugin protocol mode: called as `./main <tool_name>` with JSON on stdin.
/// Returns `{"output": "...", "success": true/false}` on stdout.
fn run_plugin(tool_name: &str) -> Result<()> {
    let mut input_json = String::new();
    std::io::stdin().read_to_string(&mut input_json)?;
    let args: serde_json::Value =
        serde_json::from_str(&input_json).unwrap_or_else(|_| serde_json::json!({}));

    // Resolve mofa root relative to the binary location:
    // binary is at <skills_dir>/<skill>/main, so parent.parent = skills_dir
    // sibling dirs (mofa-comic/styles/, mofa-slides/styles/) are also under skills_dir
    let mofa_root = if let Ok(exe) = std::env::current_exe() {
        let skill_dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let skills_dir = skill_dir.parent().unwrap_or(skill_dir);
        // Use skills_dir if any mofa-* sibling dirs exist (plugin mode)
        if std::fs::read_dir(skills_dir)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_str()
                        .is_some_and(|n| n.starts_with("mofa-"))
                })
            })
            .unwrap_or(false)
        {
            skills_dir.to_path_buf()
        } else {
            config::find_mofa_root()
        }
    } else {
        config::find_mofa_root()
    };
    let cfg = config::MofaConfig::load_default(&mofa_root);

    let result = match tool_name {
        "mofa_slides" => plugin_slides(&args, &mofa_root, &cfg),
        "mofa_cards" => plugin_cards(&args, &mofa_root, &cfg),
        "mofa_comic" => plugin_comic(&args, &mofa_root, &cfg),
        "mofa_infographic" => plugin_infographic(&args, &mofa_root, &cfg),
        "mofa_video" => plugin_video(&args, &mofa_root, &cfg),
        _ => Err(eyre::eyre!("unknown tool: {tool_name}")),
    };

    match result {
        Ok(output) => {
            println!("{}", serde_json::json!({"output": output, "success": true}));
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({"output": format!("{e:#}"), "success": false})
            );
        }
    }
    Ok(())
}

/// Resolve a temp directory under data dir/tmp/ if available, else system temp.
/// Creates the directory if it doesn't exist.
fn resolve_temp_dir(prefix: &str) -> PathBuf {
    let base = std::env::var("OCTOS_DATA_DIR")
        .or_else(|_| std::env::var("CREW_DATA_DIR"))
        .map(|d| PathBuf::from(d).join("tmp"))
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join(format!("{prefix}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// Relocate a path under data dir/tmp/ if set and the path
/// is under the system temp directory. This ensures per-profile isolation.
fn relocate_path(path: &std::path::Path) -> PathBuf {
    if let Ok(data_dir) =
        std::env::var("OCTOS_DATA_DIR").or_else(|_| std::env::var("CREW_DATA_DIR"))
    {
        let sys_tmp = std::env::temp_dir();
        if path.starts_with(&sys_tmp) {
            let relative = path.strip_prefix(&sys_tmp).unwrap_or(path);
            let relocated = PathBuf::from(&data_dir).join("tmp").join(relative);
            if let Some(parent) = relocated.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            return relocated;
        }
    }
    path.to_path_buf()
}

fn plugin_slides(
    args: &serde_json::Value,
    mofa_root: &std::path::Path,
    cfg: &config::MofaConfig,
) -> Result<String> {
    let style_name = args
        .get("style")
        .and_then(|v| v.as_str())
        .unwrap_or("nb-pro");
    let out_str = args
        .get("out")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("missing 'out' (output PPTX path)"))?;
    let out = relocate_path(std::path::Path::new(out_str));
    let slide_dir = args
        .get("slide_dir")
        .and_then(|v| v.as_str())
        .map(|s| relocate_path(std::path::Path::new(s)))
        .unwrap_or_else(|| resolve_temp_dir("mofa-slides"));
    let concurrency = args
        .get("concurrency")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    let image_size = args.get("image_size").and_then(|v| v.as_str());
    let gen_model = args.get("gen_model").and_then(|v| v.as_str());
    let ref_image_size = args.get("ref_image_size").and_then(|v| v.as_str());
    let vision_model = args.get("vision_model").and_then(|v| v.as_str());
    let auto_layout = args
        .get("auto_layout")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let refine = args
        .get("refine")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Support `input` parameter: read slides from a JS/JSON file.
    // JS files are evaluated via Node.js (module.exports = [...]).
    let slides_json = if let Some(input_path) = args.get("input").and_then(|v| v.as_str()) {
        let path = relocate_path(std::path::Path::new(input_path));
        if !path.exists() {
            eyre::bail!("input file not found: {}", path.display());
        }
        let is_js = path
            .extension()
            .is_some_and(|e| e == "js" || e == "mjs" || e == "cjs");
        if is_js {
            // Canonicalize to absolute path — Node require() treats bare
            // relative paths (no ./ prefix) as node_modules lookups.
            let abs_path = std::fs::canonicalize(&path).unwrap_or(path.clone());
            let require_path = serde_json::to_string(abs_path.to_string_lossy().as_ref())?;
            let output = std::process::Command::new("node")
                .arg("-e")
                .arg(format!(
                    "console.log(JSON.stringify(require({require_path})))"
                ))
                .output()
                .map_err(|e| eyre::eyre!("failed to run node: {e}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eyre::bail!("node failed to evaluate {}: {stderr}", path.display());
            }
            serde_json::from_slice(&output.stdout)
                .map_err(|e| eyre::eyre!("failed to parse node output: {e}"))?
        } else {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content)
                .map_err(|e| eyre::eyre!("failed to parse {}: {e}", path.display()))?
        }
    } else {
        args.get("slides")
            .ok_or_else(|| eyre::eyre!("missing 'slides' array (pass 'slides' or 'input')"))?
            .clone()
    };
    let mut slides: Vec<pipeline::slides::SlideInput> = serde_json::from_value(slides_json)?;

    // Relocate embedded file paths (source_image, images, overlay_images) for per-profile isolation
    for slide in &mut slides {
        if let Some(ref src) = slide.source_image {
            slide.source_image = Some(
                relocate_path(std::path::Path::new(src))
                    .to_string_lossy()
                    .to_string(),
            );
        }
        if let Some(ref mut imgs) = slide.images {
            for img in imgs.iter_mut() {
                *img = relocate_path(std::path::Path::new(img))
                    .to_string_lossy()
                    .to_string();
            }
        }
        if let Some(ref mut overlays) = slide.overlay_images {
            for ov in overlays.iter_mut() {
                ov.path = relocate_path(std::path::Path::new(&ov.path))
                    .to_string_lossy()
                    .to_string();
            }
        }
    }

    if auto_layout {
        for slide in &mut slides {
            slide.auto_layout = true;
        }
    }

    // Check workspace styles first (agent-created), then built-in styles.
    // If neither exists, fall back to nb-pro so generation never fails on style.
    let style_filename = format!("{style_name}.toml");
    let builtin_dir = find_styles_dir(mofa_root, "slides");
    let cwd_style = std::env::current_dir()
        .ok()
        .map(|d| d.join("styles").join(&style_filename))
        .filter(|p| p.exists());
    let builtin_style = builtin_dir.join(&style_filename);
    let style_file = if let Some(ws) = cwd_style {
        ws
    } else if builtin_style.exists() {
        builtin_style
    } else {
        eprintln!("style '{}' not found, falling back to nb-pro", style_name);
        builtin_dir.join("nb-pro.toml")
    };
    let loaded_style = style::load_style(&style_file)?;

    std::fs::create_dir_all(&slide_dir).ok();

    let batch = args.get("api").and_then(|v| v.as_str()).unwrap_or("rt") == "batch";
    pipeline::slides::run(
        &slide_dir,
        &out,
        &slides,
        &loaded_style,
        cfg,
        concurrency,
        image_size,
        gen_model,
        ref_image_size,
        vision_model,
        refine,
        batch,
    )?;

    Ok(format!("Generated PPTX: {}", out.display()))
}

fn plugin_cards(
    args: &serde_json::Value,
    mofa_root: &std::path::Path,
    cfg: &config::MofaConfig,
) -> Result<String> {
    let style_name = args
        .get("style")
        .and_then(|v| v.as_str())
        .unwrap_or("cny-guochao");
    let card_dir = args
        .get("card_dir")
        .and_then(|v| v.as_str())
        .map(|s| relocate_path(std::path::Path::new(s)))
        .ok_or_else(|| eyre::eyre!("missing 'card_dir'"))?;
    let aspect = args.get("aspect").and_then(|v| v.as_str());
    let concurrency = args
        .get("concurrency")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    let image_size = args.get("image_size").and_then(|v| v.as_str());

    let cards_json = args
        .get("cards")
        .ok_or_else(|| eyre::eyre!("missing 'cards' array"))?;
    let cards: Vec<pipeline::cards::CardInput> = serde_json::from_value(cards_json.clone())?;

    let styles_dir = find_styles_dir(mofa_root, "cards");
    let style_file = styles_dir.join(format!("{style_name}.toml"));
    let loaded_style = style::load_style(&style_file)?;

    std::fs::create_dir_all(&card_dir).ok();

    let batch = args.get("api").and_then(|v| v.as_str()).unwrap_or("rt") == "batch";
    pipeline::cards::run(
        &card_dir,
        &cards,
        &loaded_style,
        cfg,
        concurrency,
        aspect,
        image_size,
        None,
        batch,
    )?;

    Ok(format!(
        "Generated {} card(s) in {}",
        cards.len(),
        card_dir.display()
    ))
}

fn plugin_comic(
    args: &serde_json::Value,
    mofa_root: &std::path::Path,
    cfg: &config::MofaConfig,
) -> Result<String> {
    let style_name = args.get("style").and_then(|v| v.as_str()).unwrap_or("xkcd");
    let out_str = args
        .get("out")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("missing 'out' (output PNG path)"))?;
    let out = relocate_path(std::path::Path::new(out_str));
    let work_dir = args
        .get("work_dir")
        .and_then(|v| v.as_str())
        .map(|s| relocate_path(std::path::Path::new(s)))
        .unwrap_or_else(|| {
            out.parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        });
    let layout = args
        .get("layout")
        .and_then(|v| v.as_str())
        .unwrap_or("horizontal");
    let concurrency = args
        .get("concurrency")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;
    let image_size = args.get("image_size").and_then(|v| v.as_str());
    let refine = args
        .get("refine")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let gutter = args.get("gutter").and_then(|v| v.as_u64()).unwrap_or(20) as u32;

    let panels_json = args
        .get("panels")
        .ok_or_else(|| eyre::eyre!("missing 'panels' array"))?;
    let panels: Vec<pipeline::comic::PanelInput> = serde_json::from_value(panels_json.clone())?;

    let styles_dir = find_styles_dir(mofa_root, "comic");
    let style_file = styles_dir.join(format!("{style_name}.toml"));
    let loaded_style = style::load_style(&style_file)?;

    std::fs::create_dir_all(&work_dir).ok();

    let batch = args.get("api").and_then(|v| v.as_str()).unwrap_or("rt") == "batch";
    pipeline::comic::run(
        &work_dir,
        &out,
        &panels,
        &loaded_style,
        cfg,
        layout,
        concurrency,
        image_size,
        refine,
        gutter,
        None,
        batch,
    )?;

    Ok(format!("Generated comic: {}", out.display()))
}

fn plugin_infographic(
    args: &serde_json::Value,
    mofa_root: &std::path::Path,
    cfg: &config::MofaConfig,
) -> Result<String> {
    let style_name = args
        .get("style")
        .and_then(|v| v.as_str())
        .unwrap_or("cyberpunk-neon");
    let out_str = args
        .get("out")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("missing 'out' (output PNG path)"))?;
    let out = relocate_path(std::path::Path::new(out_str));
    let work_dir = args
        .get("work_dir")
        .and_then(|v| v.as_str())
        .map(|s| relocate_path(std::path::Path::new(s)))
        .unwrap_or_else(|| {
            out.parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        });
    let concurrency = args
        .get("concurrency")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;
    let image_size = args.get("image_size").and_then(|v| v.as_str());
    let aspect = args.get("aspect").and_then(|v| v.as_str());
    let refine = args
        .get("refine")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let gutter = args.get("gutter").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

    let sections_json = args
        .get("sections")
        .ok_or_else(|| eyre::eyre!("missing 'sections' array"))?;
    let sections: Vec<pipeline::infographic::SectionInput> =
        serde_json::from_value(sections_json.clone())?;

    let styles_dir = find_styles_dir(mofa_root, "infographic");
    let style_file = styles_dir.join(format!("{style_name}.toml"));
    let loaded_style = style::load_style(&style_file)?;

    std::fs::create_dir_all(&work_dir).ok();

    let batch = args.get("api").and_then(|v| v.as_str()).unwrap_or("rt") == "batch";
    pipeline::infographic::run(
        &work_dir,
        &out,
        &sections,
        &loaded_style,
        cfg,
        concurrency,
        image_size,
        aspect,
        refine,
        gutter,
        None,
        batch,
    )?;

    Ok(format!("Generated infographic: {}", out.display()))
}

fn plugin_video(
    args: &serde_json::Value,
    mofa_root: &std::path::Path,
    cfg: &config::MofaConfig,
) -> Result<String> {
    let style_name = args
        .get("style")
        .and_then(|v| v.as_str())
        .unwrap_or("video-card");
    let anim_style_name = args
        .get("anim_style")
        .and_then(|v| v.as_str())
        .unwrap_or("shuimo");
    let card_dir = args
        .get("card_dir")
        .and_then(|v| v.as_str())
        .map(|s| relocate_path(std::path::Path::new(s)))
        .ok_or_else(|| eyre::eyre!("missing 'card_dir'"))?;
    let bgm = args
        .get("bgm")
        .and_then(|v| v.as_str())
        .map(|s| relocate_path(std::path::Path::new(s)));
    let aspect = args
        .get("aspect")
        .and_then(|v| v.as_str())
        .unwrap_or("9:16");
    let image_size = args.get("image_size").and_then(|v| v.as_str());
    let concurrency = args
        .get("concurrency")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;
    let still_duration = args
        .get("still_duration")
        .and_then(|v| v.as_f64())
        .unwrap_or(2.0);
    let crossfade_dur = args
        .get("crossfade_dur")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);
    let fade_out_dur = args
        .get("fade_out_dur")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.5);
    let music_volume = args
        .get("music_volume")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.3);
    let music_fade_in = args
        .get("music_fade_in")
        .and_then(|v| v.as_f64())
        .unwrap_or(2.0);

    let cards_json = args
        .get("cards")
        .ok_or_else(|| eyre::eyre!("missing 'cards' array"))?;
    let cards: Vec<pipeline::video::VideoCardInput> = serde_json::from_value(cards_json.clone())?;

    let styles_dir = find_styles_dir(mofa_root, "video");
    let img_style_file = styles_dir.join(format!("{style_name}.toml"));
    let img_style = style::load_style(&img_style_file)?;
    let anim_style_file = styles_dir.join(format!("{anim_style_name}.toml"));
    let anim_style = if anim_style_file.exists() {
        style::load_style(&anim_style_file)?
    } else {
        style::load_style(&img_style_file)?
    };

    std::fs::create_dir_all(&card_dir).ok();

    let batch = args.get("api").and_then(|v| v.as_str()).unwrap_or("rt") == "batch";
    pipeline::video::run(
        &card_dir,
        &cards,
        &img_style,
        &anim_style,
        cfg,
        concurrency,
        Some(aspect),
        image_size,
        bgm.as_deref(),
        still_duration,
        crossfade_dur,
        fade_out_dur,
        music_volume,
        music_fade_in,
        batch,
    )?;

    Ok(format!(
        "Generated {} video card(s) in {}",
        cards.len(),
        card_dir.display()
    ))
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // Plugin protocol: if argv[1] looks like a tool name (contains '_'), use plugin mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1].starts_with("mofa_") {
        return run_plugin(&args[1]);
    }

    let cli = Cli::parse();

    let mofa_root = cli.root.unwrap_or_else(config::find_mofa_root);
    let cfg = config::MofaConfig::load_default(&mofa_root);

    match cli.command {
        Commands::Slides {
            style: style_name,
            out,
            slide_dir,
            concurrency,
            image_size,
            gen_model,
            ref_image_size,
            vision_model,
            auto_layout,
            refine,
            api,
            input,
        } => {
            let styles_dir = find_styles_dir(&mofa_root, "slides");
            let style_file = styles_dir.join(format!("{style_name}.toml"));
            let loaded_style = style::load_style(&style_file)?;

            let json = read_input(input.as_ref())?;
            let mut slides: Vec<pipeline::slides::SlideInput> = serde_json::from_str(&json)?;

            // --auto-layout flag overrides all slides
            if auto_layout {
                for slide in &mut slides {
                    slide.auto_layout = true;
                }
            }

            pipeline::slides::run(
                &slide_dir,
                &out,
                &slides,
                &loaded_style,
                &cfg,
                concurrency,
                image_size.as_deref(),
                gen_model.as_deref(),
                ref_image_size.as_deref(),
                vision_model.as_deref(),
                refine,
                matches!(api, ApiMode::Batch),
            )?;
        }
        Commands::Cards {
            style: style_name,
            card_dir,
            aspect,
            concurrency,
            image_size,
            api,
            input,
        } => {
            let styles_dir = find_styles_dir(&mofa_root, "cards");
            let style_file = styles_dir.join(format!("{style_name}.toml"));
            let loaded_style = style::load_style(&style_file)?;

            let json = read_input(input.as_ref())?;
            let cards: Vec<pipeline::cards::CardInput> = serde_json::from_str(&json)?;

            pipeline::cards::run(
                &card_dir,
                &cards,
                &loaded_style,
                &cfg,
                concurrency,
                aspect.as_deref(),
                image_size.as_deref(),
                None,
                matches!(api, ApiMode::Batch),
            )?;
        }
        Commands::Comic {
            style: style_name,
            out,
            work_dir,
            layout,
            concurrency,
            image_size,
            refine,
            gutter,
            api,
            input,
        } => {
            let styles_dir = find_styles_dir(&mofa_root, "comic");
            let style_file = styles_dir.join(format!("{style_name}.toml"));
            let loaded_style = style::load_style(&style_file)?;

            let out_dir = work_dir.unwrap_or_else(|| {
                out.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf()
            });

            let json = read_input(input.as_ref())?;
            let panels: Vec<pipeline::comic::PanelInput> = serde_json::from_str(&json)?;

            pipeline::comic::run(
                &out_dir,
                &out,
                &panels,
                &loaded_style,
                &cfg,
                &layout,
                concurrency,
                image_size.as_deref(),
                refine,
                gutter,
                None,
                matches!(api, ApiMode::Batch),
            )?;
        }
        Commands::Infographic {
            style: style_name,
            out,
            work_dir,
            concurrency,
            image_size,
            aspect,
            refine,
            gutter,
            api,
            input,
        } => {
            let styles_dir = find_styles_dir(&mofa_root, "infographic");
            let style_file = styles_dir.join(format!("{style_name}.toml"));
            let loaded_style = style::load_style(&style_file)?;

            let out_dir = work_dir.unwrap_or_else(|| {
                out.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf()
            });

            let json = read_input(input.as_ref())?;
            let sections: Vec<pipeline::infographic::SectionInput> = serde_json::from_str(&json)?;

            pipeline::infographic::run(
                &out_dir,
                &out,
                &sections,
                &loaded_style,
                &cfg,
                concurrency,
                image_size.as_deref(),
                aspect.as_deref(),
                refine,
                gutter,
                None,
                matches!(api, ApiMode::Batch),
            )?;
        }
        Commands::Video {
            style: style_name,
            anim_style: anim_style_name,
            card_dir,
            bgm,
            aspect,
            image_size,
            concurrency,
            still_duration,
            crossfade_dur,
            fade_out_dur,
            music_volume,
            music_fade_in,
            api,
            input,
        } => {
            let styles_dir = find_styles_dir(&mofa_root, "video");
            let img_style_file = styles_dir.join(format!("{style_name}.toml"));
            let img_style = style::load_style(&img_style_file)?;

            let anim_style_file = styles_dir.join(format!("{anim_style_name}.toml"));
            let anim_style = if anim_style_file.exists() {
                style::load_style(&anim_style_file)?
            } else {
                style::load_style(&img_style_file)?
            };

            let json = read_input(input.as_ref())?;
            let cards: Vec<pipeline::video::VideoCardInput> = serde_json::from_str(&json)?;

            pipeline::video::run(
                &card_dir,
                &cards,
                &img_style,
                &anim_style,
                &cfg,
                concurrency,
                Some(&aspect),
                image_size.as_deref(),
                bgm.as_deref(),
                still_duration,
                crossfade_dur,
                fade_out_dur,
                music_volume,
                music_fade_in,
                matches!(api, ApiMode::Batch),
            )?;
        }
        Commands::PptxUnpack { input, output_dir } => {
            use std::io::Read as IoRead;
            eprintln!("Unpacking {} → {}", input.display(), output_dir.display());
            std::fs::create_dir_all(&output_dir)?;
            let file = std::fs::File::open(&input)?;
            let mut archive = zip::ZipArchive::new(file)?;
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let out_path = output_dir.join(entry.name());
                if entry.is_dir() {
                    std::fs::create_dir_all(&out_path)?;
                } else {
                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut buf = Vec::new();
                    entry.read_to_end(&mut buf)?;
                    // Pretty-print XML files for readability
                    if out_path
                        .extension()
                        .is_some_and(|e| e == "xml" || e == "rels")
                    {
                        if let Ok(text) = String::from_utf8(buf.clone()) {
                            if let Ok(formatted) = pretty_print_xml(&text) {
                                std::fs::write(&out_path, formatted)?;
                                continue;
                            }
                        }
                    }
                    std::fs::write(&out_path, buf)?;
                }
            }
            eprintln!("Done: {} files extracted", archive.len());
        }
        Commands::PptxPack { input_dir, output } => {
            use std::io::{Read as IoRead, Write as IoWrite};
            eprintln!("Packing {} → {}", input_dir.display(), output.display());
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = std::fs::File::create(&output)?;
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            let mut count = 0u32;
            for entry in walkdir::WalkDir::new(&input_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let rel = path.strip_prefix(&input_dir)?;
                let name = rel.to_string_lossy().to_string();
                let mut buf = Vec::new();
                std::fs::File::open(path)?.read_to_end(&mut buf)?;
                // Condense XML files (remove pretty-print whitespace)
                if path.extension().is_some_and(|e| e == "xml" || e == "rels") {
                    if let Ok(text) = String::from_utf8(buf.clone()) {
                        if let Ok(condensed) = condense_xml(&text) {
                            zip.start_file(&name, options)?;
                            zip.write_all(condensed.as_bytes())?;
                            count += 1;
                            continue;
                        }
                    }
                }
                zip.start_file(&name, options)?;
                zip.write_all(&buf)?;
                count += 1;
            }
            zip.finish()?;
            eprintln!("Done: {} files packed", count);
        }
    }

    Ok(())
}

/// Pretty-print XML for readability (used by pptx-unpack).
fn pretty_print_xml(xml: &str) -> Result<String> {
    // Simple indent-based pretty printer for XML
    let mut result = String::with_capacity(xml.len() * 2);
    let mut depth: i32 = 0;
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut text_buf = String::new();
    let mut prev_was_close = false;

    for ch in xml.chars() {
        match ch {
            '<' => {
                if !text_buf.trim().is_empty() {
                    result.push_str(text_buf.trim());
                }
                text_buf.clear();
                in_tag = true;
                tag_buf.clear();
                tag_buf.push(ch);
            }
            '>' => {
                tag_buf.push(ch);
                in_tag = false;
                let is_close = tag_buf.starts_with("</");
                let is_self_close = tag_buf.ends_with("/>");
                let is_decl = tag_buf.starts_with("<?");

                if is_close {
                    depth -= 1;
                }
                if !prev_was_close || is_close {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..depth.max(0) {
                        result.push_str("  ");
                    }
                }
                result.push_str(&tag_buf);
                prev_was_close = is_close;

                if !is_close && !is_self_close && !is_decl {
                    depth += 1;
                }
                tag_buf.clear();
            }
            _ => {
                if in_tag {
                    tag_buf.push(ch);
                } else {
                    text_buf.push(ch);
                }
            }
        }
    }
    Ok(result)
}

/// Condense XML by removing pretty-print whitespace (used by pptx-pack).
fn condense_xml(xml: &str) -> Result<String> {
    let mut result = String::with_capacity(xml.len());
    let mut in_tag = false;
    let mut in_preserve = false; // Inside a text element like <w:t> or <a:t>
    let mut prev_was_gt = false;

    for ch in xml.chars() {
        match ch {
            '<' => {
                in_tag = true;
                result.push(ch);
                prev_was_gt = false;
            }
            '>' => {
                in_tag = false;
                result.push(ch);
                prev_was_gt = true;
                // Check if we just entered a text-preserving element
                let last_tag: String = result
                    .chars()
                    .rev()
                    .take_while(|&c| c != '<')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                if last_tag.contains(":t>") || last_tag.contains(":t ") {
                    in_preserve = !last_tag.starts_with('/');
                }
            }
            '\n' | '\r' => {
                if in_tag || in_preserve {
                    result.push(ch);
                }
                // Skip newlines between tags
            }
            ' ' | '\t' => {
                if in_tag || in_preserve {
                    result.push(ch);
                } else if prev_was_gt {
                    // Skip indentation whitespace between tags
                } else {
                    result.push(ch);
                }
            }
            _ => {
                result.push(ch);
                prev_was_gt = false;
            }
        }
    }
    Ok(result)
}
