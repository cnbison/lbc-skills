// SPDX-License-Identifier: Apache-2.0

use eyre::{bail, Result};
use serde::Deserialize;
use std::io::Write;
use std::path::Path;

/// 1 inch = 914400 EMU (English Metric Units)
const EMU_PER_INCH: f64 = 914_400.0;
/// PowerPoint font size unit: 1 pt = 100 half-points
const PT_TO_HPTS: f64 = 100.0;

/// Shadow specification for text overlays.
#[derive(Deserialize, Debug, Clone)]
pub struct TextShadow {
    #[serde(default = "default_shadow_blur")]
    pub blur: f64,
    #[serde(default = "default_shadow_offset")]
    pub offset: f64,
    #[serde(default = "default_shadow_angle")]
    pub angle: f64,
    #[serde(default = "default_shadow_opacity")]
    pub opacity: f64,
    #[serde(default = "default_shadow_color")]
    pub color: String,
}

fn default_shadow_blur() -> f64 {
    4.0
}
fn default_shadow_offset() -> f64 {
    2.0
}
fn default_shadow_angle() -> f64 {
    45.0
}
fn default_shadow_opacity() -> f64 {
    0.6
}
fn default_shadow_color() -> String {
    "000000".into()
}

/// Fill specification for text box background.
#[derive(Deserialize, Debug, Clone)]
pub struct TextFill {
    #[serde(default = "default_fill_color")]
    pub color: String,
    #[serde(default)]
    pub transparency: f64,
}

fn default_fill_color() -> String {
    "000000".into()
}

/// Text overlay specification matching mofa-pptx `texts` API.
#[derive(Deserialize, Debug, Clone)]
pub struct TextOverlay {
    pub text: Option<String>,
    pub runs: Option<Vec<TextRun>>,
    #[serde(default = "default_x")]
    pub x: f64,
    #[serde(default = "default_y")]
    pub y: f64,
    #[serde(default = "default_w")]
    pub w: f64,
    #[serde(default = "default_h")]
    pub h: f64,
    #[serde(rename = "fontFace")]
    pub font_face: Option<String>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<f64>,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default = "default_align")]
    pub align: String,
    #[serde(default = "default_valign")]
    pub valign: String,
    pub rotate: Option<f64>,
    pub shadow: Option<TextShadow>,
    #[serde(rename = "lineSpacing")]
    pub line_spacing: Option<f64>,
    pub margin: Option<f64>,
    #[allow(dead_code)]
    pub transparency: Option<f64>,
    pub fill: Option<TextFill>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextRun {
    pub text: String,
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<f64>,
    #[serde(rename = "fontFace")]
    pub font_face: Option<String>,
    #[serde(rename = "breakLine")]
    pub break_line: Option<bool>,
}

fn default_x() -> f64 {
    0.5
}
fn default_y() -> f64 {
    0.5
}
fn default_w() -> f64 {
    6.0
}
fn default_h() -> f64 {
    1.0
}
fn default_color() -> String {
    "FFFFFF".into()
}
fn default_align() -> String {
    "l".into()
}
fn default_valign() -> String {
    "t".into()
}

fn inches_to_emu(inches: f64) -> i64 {
    (inches * EMU_PER_INCH).round() as i64
}

fn pptx_align(a: &str) -> &str {
    match a {
        "center" | "c" | "ctr" => "ctr",
        "right" | "r" => "r",
        "justify" | "j" | "just" => "just",
        _ => "l",
    }
}

fn pptx_valign(a: &str) -> &str {
    match a {
        "middle" | "m" | "ctr" => "ctr",
        "bottom" | "b" => "b",
        _ => "t",
    }
}

fn build_run_xml(
    text: &str,
    font_face: &str,
    font_size: f64,
    color: &str,
    bold: bool,
    italic: bool,
) -> String {
    let sz = (font_size * PT_TO_HPTS) as i64;
    let b = if bold { r#" b="1""# } else { "" };
    let i = if italic { r#" i="1""# } else { "" };
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<a:r><a:rPr lang="en-US" sz="{sz}"{b}{i} dirty="0"><a:solidFill><a:srgbClr val="{color}"/></a:solidFill><a:latin typeface="{font_face}" pitchFamily="34" charset="0"/><a:ea typeface="{font_face}" pitchFamily="34" charset="-122"/><a:cs typeface="{font_face}" pitchFamily="34" charset="-120"/></a:rPr><a:t>{escaped}</a:t></a:r>"#
    )
}

fn build_text_shape_xml(overlay: &TextOverlay, shape_id: u32) -> String {
    let x = inches_to_emu(overlay.x);
    let y = inches_to_emu(overlay.y);
    let w = inches_to_emu(overlay.w);
    let h = inches_to_emu(overlay.h);
    let align = pptx_align(&overlay.align);
    let anchor = pptx_valign(&overlay.valign);
    let font_face = overlay.font_face.as_deref().unwrap_or("Arial");
    let font_size = overlay.font_size.unwrap_or(18.0);

    let rotation = overlay
        .rotate
        .map(|deg| format!(r#" rot="{}""#, (deg * 60000.0) as i64))
        .unwrap_or_default();

    // Shadow effect on shape
    let shadow_xml = overlay.shadow.as_ref().map(|s| {
        let blur_emu = (s.blur * 12700.0) as i64; // pt to EMU
        let dist_emu = (s.offset * 12700.0) as i64;
        let angle = (s.angle * 60000.0) as i64;
        let alpha = ((s.opacity * 100000.0) as i64).min(100000);
        format!(
            r#"<a:effectLst><a:outerShdw blurRad="{blur_emu}" dist="{dist_emu}" dir="{angle}" algn="bl" rotWithShape="0"><a:srgbClr val="{}"><a:alpha val="{alpha}"/></a:srgbClr></a:outerShdw></a:effectLst>"#,
            s.color
        )
    }).unwrap_or_default();

    // Fill for text box background
    let fill_xml = if let Some(fill) = &overlay.fill {
        let alpha = (((100.0 - fill.transparency) * 1000.0) as i64).clamp(0, 100000);
        format!(
            r#"<a:solidFill><a:srgbClr val="{}"><a:alpha val="{alpha}"/></a:srgbClr></a:solidFill>"#,
            fill.color
        )
    } else {
        "<a:noFill/>".to_string()
    };

    // Inset margin (same all sides, in EMU)
    let inset = overlay
        .margin
        .map(|m| {
            let emu = (m * 12700.0) as i64; // pt to EMU
            format!(r#" lIns="{emu}" tIns="{emu}" rIns="{emu}" bIns="{emu}""#)
        })
        .unwrap_or_else(|| r#" lIns="0" tIns="0" rIns="0" bIns="0""#.to_string());

    // Line spacing in 100ths of a point
    let line_spacing_xml = overlay
        .line_spacing
        .map(|ls| {
            let val = (ls * 100.0) as i64;
            format!(r#"<a:lnSpc><a:spcPts val="{val}"/></a:lnSpc>"#)
        })
        .unwrap_or_default();

    let para_content = if let Some(runs) = &overlay.runs {
        let mut xml = String::new();
        for run in runs {
            let rf = run.font_face.as_deref().unwrap_or(font_face);
            let rs = run.font_size.unwrap_or(font_size);
            let rc = run.color.as_deref().unwrap_or(&overlay.color);
            let rb = run.bold.unwrap_or(overlay.bold);
            let ri = run.italic.unwrap_or(overlay.italic);
            if run.break_line == Some(true) {
                xml.push_str(&format!(r#"</a:p><a:p><a:pPr algn="{align}"/>"#));
            }
            xml.push_str(&build_run_xml(&run.text, rf, rs, rc, rb, ri));
        }
        xml
    } else {
        let text = overlay.text.as_deref().unwrap_or("");
        let lines: Vec<&str> = text.split('\n').collect();
        if lines.len() <= 1 {
            build_run_xml(
                text,
                font_face,
                font_size,
                &overlay.color,
                overlay.bold,
                overlay.italic,
            )
        } else {
            // Multi-line: each line becomes a separate <a:p> paragraph
            let mut xml = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    // Close previous paragraph, open new one
                    xml.push_str(&format!(r#"</a:p><a:p><a:pPr algn="{align}" indent="0" marL="0"><a:buNone/></a:pPr>"#));
                }
                xml.push_str(&build_run_xml(
                    line,
                    font_face,
                    font_size,
                    &overlay.color,
                    overlay.bold,
                    overlay.italic,
                ));
            }
            xml
        }
    };

    let end_sz = (font_size * PT_TO_HPTS) as i64;

    format!(
        r#"<p:sp><p:nvSpPr><p:cNvPr id="{shape_id}" name="Text {shape_id}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm{rotation}><a:off x="{x}" y="{y}"/><a:ext cx="{w}" cy="{h}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom>{fill_xml}<a:ln/>{shadow_xml}</p:spPr><p:txBody><a:bodyPr wrap="square" rtlCol="0" anchor="{anchor}"{inset}/><a:lstStyle/><a:p><a:pPr algn="{align}" indent="0" marL="0"><a:buNone/>{line_spacing_xml}</a:pPr>{para_content}<a:endParaRPr lang="en-US" sz="{end_sz}" dirty="0"/></a:p></p:txBody></p:sp>"#
    )
}

/// An image to overlay on a slide at a specific position.
#[derive(Deserialize, Debug, Clone)]
pub struct ImageOverlay {
    pub path: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default = "default_overlay_w")]
    pub w: f64,
    #[serde(default = "default_overlay_h")]
    pub h: f64,
}

fn default_overlay_w() -> f64 {
    2.0
}
fn default_overlay_h() -> f64 {
    1.5
}

/// Data for a single slide in a multi-slide PPTX.
pub struct SlideData {
    pub image_path: Option<String>,
    pub texts: Vec<TextOverlay>,
    pub images: Vec<ImageOverlay>,
}

/// Build a multi-slide PPTX from slide data.
pub fn build_pptx(slides: &[SlideData], out_file: &Path, slide_w: f64, slide_h: f64) -> Result<()> {
    if slides.is_empty() {
        bail!("No slides to build");
    }

    let sw = inches_to_emu(slide_w);
    let sh = inches_to_emu(slide_h);
    let num_slides = slides.len();

    // Collect media files and build per-slide XML
    let mut slide_xmls = Vec::new();
    let mut slide_rel_xmls = Vec::new();
    let mut media_entries: Vec<(String, Vec<u8>, &str)> = Vec::new(); // (name, data, content_type)

    for (idx, sd) in slides.iter().enumerate() {
        let slide_num = idx + 1;

        let mut shapes_xml = String::new();
        let mut has_image = false;
        let mut media_name = String::new();
        let mut slide_media: Vec<(String, Vec<u8>, &str)> = Vec::new(); // per-slide media

        if let Some(img_path_str) = &sd.image_path {
            let img_path = Path::new(img_path_str);
            if img_path.exists() {
                let img_data = std::fs::read(img_path)?;
                let ext = img_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png")
                    .to_lowercase();
                let (mname, ctype) = match ext.as_str() {
                    "jpg" | "jpeg" => (format!("image{slide_num}.jpeg"), "image/jpeg"),
                    _ => (format!("image{slide_num}.png"), "image/png"),
                };
                media_name = mname.clone();
                slide_media.push((mname, img_data, ctype));
                has_image = true;
            }
        }

        // Load overlay images
        let mut overlay_rids: Vec<String> = Vec::new();
        for (oi, overlay_img) in sd.images.iter().enumerate() {
            let img_path = Path::new(&overlay_img.path);
            if img_path.exists() {
                let img_data = std::fs::read(img_path)?;
                let ext = img_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png")
                    .to_lowercase();
                let (mname, ctype) = match ext.as_str() {
                    "jpg" | "jpeg" => (format!("overlay{slide_num}_{oi}.jpeg"), "image/jpeg"),
                    _ => (format!("overlay{slide_num}_{oi}.png"), "image/png"),
                };
                let rid = format!("rId{}", 3 + oi);
                overlay_rids.push(rid);
                slide_media.push((mname, img_data, ctype));
            }
        }

        // Background image picture shape
        let pic_xml = if has_image {
            format!(
                r#"<p:pic><p:nvPicPr><p:cNvPr id="2" name="Background"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="rId2"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{sw}" cy="{sh}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>"#
            )
        } else {
            String::new()
        };

        // Overlay image shapes
        for (oi, overlay_img) in sd.images.iter().enumerate() {
            if oi < overlay_rids.len() {
                let rid = &overlay_rids[oi];
                let ox = inches_to_emu(overlay_img.x);
                let oy = inches_to_emu(overlay_img.y);
                let ow = inches_to_emu(overlay_img.w);
                let oh = inches_to_emu(overlay_img.h);
                let shape_id = (oi as u32) + 100;
                shapes_xml.push_str(&format!(
                    r#"<p:pic><p:nvPicPr><p:cNvPr id="{shape_id}" name="Overlay {oi}"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="{ox}" y="{oy}"/><a:ext cx="{ow}" cy="{oh}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>"#
                ));
            }
        }

        // Text overlay shapes
        let text_shape_base = (sd.images.len() as u32) + 3;
        for (i, overlay) in sd.texts.iter().enumerate() {
            shapes_xml.push_str(&build_text_shape_xml(overlay, (i as u32) + text_shape_base));
        }

        let slide_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld name="Slide {slide_num}">
<p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
{pic_xml}
{shapes_xml}
</p:spTree>
</p:cSld>
<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#
        );
        slide_xmls.push(slide_xml);

        // Slide relationships
        let mut rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>"#.to_string();
        if has_image {
            rels.push_str(&format!(
                r#"
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/{media_name}"/>"#
            ));
        }
        // Overlay image relationships
        for (oi, rid) in overlay_rids.iter().enumerate() {
            let overlay_media_idx = if has_image { oi + 1 } else { oi };
            let overlay_media_name = &slide_media[overlay_media_idx].0;
            rels.push_str(&format!(
                r#"
<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/{overlay_media_name}"/>"#
            ));
        }
        rels.push_str("\n</Relationships>");
        slide_rel_xmls.push(rels);

        // Add all slide media to global list
        media_entries.extend(slide_media);
    }

    // Build presentation.xml with slide list
    let mut slide_id_list = String::new();
    let mut pres_rels_slides = String::new();
    for i in 0..num_slides {
        let slide_num = i + 1;
        let slide_id = 256 + i as u32;
        let rid = format!("rId{}", i + 2);
        slide_id_list.push_str(&format!(r#"<p:sldId id="{slide_id}" r:id="{rid}"/>"#));
        pres_rels_slides.push_str(&format!(
            r#"
<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{slide_num}.xml"/>"#
        ));
    }

    // Content types — collect unique image extensions
    let mut img_ext_types = std::collections::HashSet::new();
    for (name, _, ctype) in &media_entries {
        let ext = Path::new(name).extension().unwrap().to_str().unwrap();
        img_ext_types.insert((ext.to_string(), ctype.to_string()));
    }

    let mut ext_defaults = String::new();
    for (ext, ctype) in &img_ext_types {
        ext_defaults.push_str(&format!(
            r#"<Default Extension="{ext}" ContentType="{ctype}"/>"#
        ));
    }

    let mut slide_overrides = String::new();
    for i in 1..=num_slides {
        slide_overrides.push_str(&format!(
            r#"<Override PartName="/ppt/slides/slide{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#
        ));
    }

    let content_types = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="xml" ContentType="application/xml"/>
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
{ext_defaults}
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
{slide_overrides}
<Override PartName="/ppt/presProps.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presProps+xml"/>
<Override PartName="/ppt/viewProps.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.viewProps+xml"/>
<Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
<Override PartName="/ppt/tableStyles.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.tableStyles+xml"/>
<Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#
    );

    let presentation = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" saveSubsetFonts="1" autoCompressPictures="0">
<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
<p:sldIdLst>{slide_id_list}</p:sldIdLst>
<p:sldSz cx="{sw}" cy="{sh}"/>
<p:notesSz cx="{sh}" cy="{sw}"/>
<p:defaultTextStyle><a:defPPr><a:defRPr lang="en-US"/></a:defPPr></p:defaultTextStyle>
</p:presentation>"#
    );

    let pres_rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
{pres_rels_slides}
<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/presProps" Target="presProps.xml"/>
<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/viewProps" Target="viewProps.xml"/>
<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>
<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/tableStyles" Target="tableStyles.xml"/>
</Relationships>"#,
        num_slides + 2,
        num_slides + 3,
        num_slides + 4,
        num_slides + 5,
    );

    // Static XML files (same as office.rs)
    let root_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;

    let slide_layout = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>"#;

    let layout_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#;

    let slide_master = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:bg><p:bgPr><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill><a:effectLst/></p:bgPr></p:bg><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle><a:lvl1pPr algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1"><a:lnSpc><a:spcPct val="90000"/></a:lnSpc><a:spcBef><a:spcPct val="0"/></a:spcBef><a:buNone/><a:defRPr sz="4400" kern="1200"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill><a:latin typeface="+mj-lt"/><a:ea typeface="+mj-ea"/><a:cs typeface="+mj-cs"/></a:defRPr></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr marL="228600" indent="-228600" algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1"><a:lnSpc><a:spcPct val="90000"/></a:lnSpc><a:spcBef><a:spcPts val="1000"/></a:spcBef><a:buFont typeface="Arial" panose="020B0604020202020204" pitchFamily="34" charset="0"/><a:buChar char="&#x2022;"/><a:defRPr sz="2800" kern="1200"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill><a:latin typeface="+mn-lt"/><a:ea typeface="+mn-ea"/><a:cs typeface="+mn-cs"/></a:defRPr></a:lvl1pPr></p:bodyStyle><p:otherStyle><a:defPPr><a:defRPr lang="en-US"/></a:defPPr></p:otherStyle></p:txStyles></p:sldMaster>"#;

    let master_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#;

    let theme = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
<a:themeElements>
<a:clrScheme name="Office"><a:dk1><a:srgbClr val="000000"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="1F497D"/></a:dk2><a:lt2><a:srgbClr val="EEECE1"/></a:lt2><a:accent1><a:srgbClr val="4F81BD"/></a:accent1><a:accent2><a:srgbClr val="C0504D"/></a:accent2><a:accent3><a:srgbClr val="9BBB59"/></a:accent3><a:accent4><a:srgbClr val="8064A2"/></a:accent4><a:accent5><a:srgbClr val="4BACC6"/></a:accent5><a:accent6><a:srgbClr val="F79646"/></a:accent6><a:hlink><a:srgbClr val="0000FF"/></a:hlink><a:folHlink><a:srgbClr val="800080"/></a:folHlink></a:clrScheme>
<a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>
<a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme>
</a:themeElements>
<a:objectDefaults/><a:extraClrSchemeLst/>
</a:theme>"#;

    let pres_props = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentationPr xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"/>"#;

    let view_props = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:viewPr xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:normalViewPr horzBarState="maximized"><p:restoredLeft sz="15611"/><p:restoredTop sz="94610"/></p:normalViewPr>
<p:slideViewPr><p:cSldViewPr snapToGrid="0" snapToObjects="1"><p:cViewPr varScale="1"><p:scale><a:sx n="136" d="100"/><a:sy n="136" d="100"/></p:scale><p:origin x="216" y="312"/></p:cViewPr><p:guideLst/></p:cSldViewPr></p:slideViewPr>
<p:notesTextViewPr><p:cViewPr><p:scale><a:sx n="1" d="1"/><a:sy n="1" d="1"/></p:scale><p:origin x="0" y="0"/></p:cViewPr></p:notesTextViewPr>
<p:gridSpacing cx="76200" cy="76200"/>
</p:viewPr>"#;

    let table_styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:tblStyleLst xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" def="{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}"/>"#;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let core_props = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:title>Presentation</dc:title>
<dc:creator>mofa</dc:creator>
<cp:lastModifiedBy>mofa</cp:lastModifiedBy>
<cp:revision>1</cp:revision>
<dcterms:created xsi:type="dcterms:W3CDTF">{now}</dcterms:created>
<dcterms:modified xsi:type="dcterms:W3CDTF">{now}</dcterms:modified>
</cp:coreProperties>"#
    );

    let app_props = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
<TotalTime>0</TotalTime>
<Words>0</Words>
<Application>mofa</Application>
<PresentationFormat>On-screen Show (16:9)</PresentationFormat>
<Paragraphs>0</Paragraphs>
<Slides>{num_slides}</Slides>
<Notes>0</Notes>
<HiddenSlides>0</HiddenSlides>
<MMClips>0</MMClips>
<ScaleCrop>false</ScaleCrop>
<LinksUpToDate>false</LinksUpToDate>
<SharedDoc>false</SharedDoc>
<HyperlinksChanged>false</HyperlinksChanged>
<AppVersion>16.0000</AppVersion>
</Properties>"#
    );

    // Pack into ZIP
    if let Some(parent) = out_file.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let file = std::fs::File::create(out_file)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Static XML files
    let static_files: &[(&str, &str)] = &[
        ("[Content_Types].xml", &content_types),
        ("_rels/.rels", root_rels),
        ("docProps/app.xml", &app_props),
        ("docProps/core.xml", &core_props),
        ("ppt/presentation.xml", &presentation),
        ("ppt/_rels/presentation.xml.rels", &pres_rels),
        ("ppt/presProps.xml", pres_props),
        ("ppt/viewProps.xml", view_props),
        ("ppt/tableStyles.xml", table_styles),
        ("ppt/slideLayouts/slideLayout1.xml", slide_layout),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels", layout_rels),
        ("ppt/slideMasters/slideMaster1.xml", slide_master),
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels", master_rels),
        ("ppt/theme/theme1.xml", theme),
    ];

    for (name, content) in static_files {
        zip.start_file(*name, opts)?;
        zip.write_all(content.as_bytes())?;
    }

    // Per-slide XML
    for (i, (slide_xml, rel_xml)) in slide_xmls.iter().zip(slide_rel_xmls.iter()).enumerate() {
        let n = i + 1;
        zip.start_file(format!("ppt/slides/slide{n}.xml"), opts)?;
        zip.write_all(slide_xml.as_bytes())?;
        zip.start_file(format!("ppt/slides/_rels/slide{n}.xml.rels"), opts)?;
        zip.write_all(rel_xml.as_bytes())?;
    }

    // Media files
    for (name, data, _) in &media_entries {
        zip.start_file(format!("ppt/media/{name}"), opts)?;
        zip.write_all(data)?;
    }

    zip.finish()?;
    eprintln!("{}", out_file.display());
    Ok(())
}
