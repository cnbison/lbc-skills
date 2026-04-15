// SPDX-License-Identifier: Apache-2.0

use eyre::{bail, Result};
use image::RgbaImage;
use std::path::Path;

/// Open an image, guessing format from content (not extension).
fn open_image(p: &Path) -> Result<image::DynamicImage> {
    let reader = image::ImageReader::open(p)
        .map_err(|e| eyre::eyre!("opening {}: {e}", p.display()))?
        .with_guessed_format()
        .map_err(|e| eyre::eyre!("guessing format {}: {e}", p.display()))?;
    reader
        .decode()
        .map_err(|e| eyre::eyre!("decoding {}: {e}", p.display()))
}

/// Stitch images horizontally with a gutter between them.
pub fn stitch_horizontal(paths: &[&Path], gutter: u32, out_file: &Path) -> Result<()> {
    if paths.is_empty() {
        bail!("No images to stitch");
    }

    let images: Vec<image::DynamicImage> = paths
        .iter()
        .map(|p| open_image(p))
        .collect::<Result<Vec<_>>>()?;

    // Target height = max height among all images
    let max_h = images.iter().map(|img| img.height()).max().unwrap();

    // Total width = sum of widths + gutters
    let total_w: u32 =
        images.iter().map(|img| img.width()).sum::<u32>() + gutter * (images.len() as u32 - 1);

    let mut canvas = RgbaImage::new(total_w, max_h);

    let mut x_off = 0u32;
    for img in &images {
        // Scale to match max height
        let scaled = if img.height() != max_h {
            let scale = max_h as f64 / img.height() as f64;
            let new_w = (img.width() as f64 * scale) as u32;
            img.resize_exact(new_w, max_h, image::imageops::FilterType::Lanczos3)
        } else {
            img.clone()
        };

        image::imageops::overlay(&mut canvas, &scaled.to_rgba8(), x_off as i64, 0);
        x_off += scaled.width() + gutter;
    }

    canvas.save(out_file)?;
    let size = std::fs::metadata(out_file)?.len();
    eprintln!("Stitched: {} ({}KB)", out_file.display(), size / 1024);
    Ok(())
}

/// Stitch images vertically with a gutter between them.
pub fn stitch_vertical(paths: &[&Path], gutter: u32, out_file: &Path) -> Result<()> {
    if paths.is_empty() {
        bail!("No images to stitch");
    }

    let images: Vec<image::DynamicImage> = paths
        .iter()
        .map(|p| open_image(p))
        .collect::<Result<Vec<_>>>()?;

    // Target width = max width among all images
    let max_w = images.iter().map(|img| img.width()).max().unwrap();

    // Total height = sum of heights + gutters
    let total_h: u32 =
        images.iter().map(|img| img.height()).sum::<u32>() + gutter * (images.len() as u32 - 1);

    let mut canvas = RgbaImage::new(max_w, total_h);

    let mut y_off = 0u32;
    for img in &images {
        // Scale to match max width
        let scaled = if img.width() != max_w {
            let scale = max_w as f64 / img.width() as f64;
            let new_h = (img.height() as f64 * scale) as u32;
            img.resize_exact(max_w, new_h, image::imageops::FilterType::Lanczos3)
        } else {
            img.clone()
        };

        image::imageops::overlay(&mut canvas, &scaled.to_rgba8(), 0, y_off as i64);
        y_off += scaled.height() + gutter;
    }

    canvas.save(out_file)?;
    let size = std::fs::metadata(out_file)?.len();
    eprintln!("Stitched: {} ({}KB)", out_file.display(), size / 1024);
    Ok(())
}

/// Stitch images in a grid layout with a gutter between them.
pub fn stitch_grid(paths: &[&Path], gutter: u32, out_file: &Path) -> Result<()> {
    if paths.is_empty() {
        bail!("No images to stitch");
    }

    let cols = (paths.len() as f64).sqrt().ceil() as usize;
    let rows = paths.len().div_ceil(cols);

    let images: Vec<image::DynamicImage> = paths
        .iter()
        .map(|p| open_image(p))
        .collect::<Result<Vec<_>>>()?;

    // Find max cell dimensions
    let cell_w = images.iter().map(|img| img.width()).max().unwrap();
    let cell_h = images.iter().map(|img| img.height()).max().unwrap();

    let total_w = cell_w * cols as u32 + gutter * (cols as u32 - 1);
    let total_h = cell_h * rows as u32 + gutter * (rows as u32 - 1);

    let mut canvas = RgbaImage::new(total_w, total_h);

    for (i, img) in images.iter().enumerate() {
        let row = i / cols;
        let col = i % cols;
        let x = col as u32 * (cell_w + gutter);
        let y = row as u32 * (cell_h + gutter);

        let scaled = img.resize_exact(cell_w, cell_h, image::imageops::FilterType::Lanczos3);
        image::imageops::overlay(&mut canvas, &scaled.to_rgba8(), x as i64, y as i64);
    }

    canvas.save(out_file)?;
    let size = std::fs::metadata(out_file)?.len();
    eprintln!("Stitched grid: {} ({}KB)", out_file.display(), size / 1024);
    Ok(())
}
