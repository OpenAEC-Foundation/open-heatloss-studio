//! Top-level orchestrator: ReportData -> PDF bytes.
//!
//! Builds a flowable stream from cover/colofon/toc/sections/backcover and
//! hands it to openaec-layout's DocTemplate which produces the actual PDF.

use openaec_layout::{
    A3, A4, DocTemplate, Flowable, Frame, Mm, PageBreak, PageTemplate, Paragraph, ParagraphStyle,
    Pt, Rect, Spacer, shared_font_registry,
};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use image::GenericImageView;
use openaec_layout::Color;

use super::blocks::render_block;
use super::brand::{FooterImageData, HeaderImageData, OhsBrand};
use super::fonts;
use super::schema::{Block, Orientation, PaperFormat, ReportData, Section, Style};
use super::special_pages::{
    BackcoverContext, render_backcover, render_colofon, render_cover, render_toc,
};

pub fn generate_pdf(data: &ReportData) -> Result<Vec<u8>, String> {
    let registry = shared_font_registry();
    {
        let mut guard = registry.lock().map_err(|e| format!("font lock: {e}"))?;
        fonts::register(&mut *guard).map_err(|e| format!("font registration: {e}"))?;
    }

    // Brand: start with defaults, then apply optional per-project style
    // overrides (accent_color_hex). Invalid hex values fall back silently
    // to the default so the report still generates.
    let mut brand = OhsBrand::default();
    if let Some(style) = &data.style {
        if let Some(hex) = &style.accent_color_hex {
            if let Some(c) = parse_hex_color(hex) {
                brand.primary = c;
                brand.table_header_bg = c;
            }
        }
    }
    let report_title = data
        .cover
        .as_ref()
        .and_then(|c| c.subtitle.clone())
        .unwrap_or_else(|| "Warmteverliesberekening".into());

    let mut doc = DocTemplate::new(&data.project, registry);

    let page_size = match (data.format, data.orientation) {
        (PaperFormat::A4, Orientation::Portrait) => A4,
        (PaperFormat::A4, Orientation::Landscape) => A4.landscape(),
        (PaperFormat::A3, Orientation::Portrait) => A3,
        (PaperFormat::A3, Orientation::Landscape) => A3.landscape(),
    };

    // Frame: 15mm horizontal margins, 20mm top, 28mm bottom (total 48mm
    // vertical — leaves 249mm content height on A4). The brand callback
    // draws its footer line at page_h − 12mm, so the 28mm bottom gives a
    // ~16mm safety zone for any flowable that overshoots its declared
    // wrap height.
    //
    // IMPORTANT: `Mm(x).0` returns the raw millimeter value as f32 — NOT
    // points. Subtracting that from `page_size.X.0` (which IS points)
    // silently produces nonsense: e.g. `842pt − 48` = 794pt instead of
    // 706pt, leaving the frame ~30mm too tall and content spilling onto
    // the running footer. Convert via `.into()` first.
    // Margins (mm) — defaults match the original layout (15 / 20 / 28).
    // Per-project style override clamps to a sensible range so a typo
    // can't produce a frame that's negative-sized or runs off-page.
    let style_default = Style::default();
    let style = data.style.as_ref().unwrap_or(&style_default);
    let top_mm = style.margin_top_mm.unwrap_or(20.0).clamp(5.0, 80.0);
    let bottom_mm = style.margin_bottom_mm.unwrap_or(28.0).clamp(5.0, 80.0);
    let h_mm = style.margin_horizontal_mm.unwrap_or(15.0).clamp(5.0, 80.0);

    let frame_x: Pt = Mm(h_mm).into();
    let frame_y: Pt = Mm(top_mm).into();
    let h_margin: Pt = Mm(h_mm * 2.0).into();
    let v_margin: Pt = Mm(top_mm + bottom_mm).into();
    let frame_w = Pt(page_size.width.0 - h_margin.0);
    let frame_h = Pt(page_size.height.0 - v_margin.0);
    let frame = Frame::new(Rect::new(frame_x, frame_y, frame_w, frame_h));

    let backcover_present = data.backcover.as_ref().map(|b| b.enabled).unwrap_or(false);

    // Decode optional footer + header images once so each page-callback
    // invocation can re-use the same bytes + intrinsic pixel dimensions
    // for aspect-correct scaling. Failure (invalid base64 or unsupported
    // format) is logged as a warning and the report is still generated
    // without that particular image.
    let footer_image = data.footer.as_ref().and_then(|f| f.image.as_ref()).and_then(
        |img| match decode_footer_image(&img.data) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("[reports] footer image decode failed: {e} — rendering without footer image");
                None
            }
        },
    );
    let header_image = data.header.as_ref().and_then(|h| h.image.as_ref()).and_then(
        |img| match decode_footer_image(&img.data) {
            Ok(d) => Some(HeaderImageData {
                bytes: d.bytes,
                width_px: d.width_px,
                height_px: d.height_px,
            }),
            Err(e) => {
                eprintln!("[reports] header image decode failed: {e} — rendering without header image");
                None
            }
        },
    );

    let tmpl = PageTemplate::new("content", page_size, frame).with_callback(Box::new(
        brand.page_callback(
            &data.project,
            &report_title,
            backcover_present,
            footer_image,
            header_image,
        ),
    ));
    doc.add_page_template(tmpl);

    // Build flowables
    let mut flowables: Vec<Box<dyn Flowable>> = Vec::new();

    if let Some(cover) = &data.cover {
        flowables.extend(render_cover(
            &data.project,
            data.date.as_deref().unwrap_or(""),
            cover,
            &brand,
        ));
    }

    if let Some(c) = &data.colofon {
        if c.enabled {
            flowables.extend(render_colofon(&data.project, c, &brand));
        }
    }

    if let Some(t) = &data.toc {
        // Compute approximate page numbers per section so the TOC can show
        // real pagina-nummers met dot-leaders. Heuristiek-gebaseerde
        // estimator op basis van bloc-types — voor een typisch
        // warmteverlies-rapport (10-25 secties) is dit doorgaans accuraat
        // binnen ±1 pagina per sectie. Wordt op de mid-call gebouwd op
        // basis van dezelfde frame-hoogte als de echte render.
        let pre_content_pages =
            pre_content_page_count(data.cover.is_some(), data.colofon.is_some(), t.enabled, t, &data.sections);
        let section_pages = estimate_section_pages(
            &data.sections,
            frame_h,
            pre_content_pages,
        );
        flowables.extend(render_toc(t, &data.sections, Some(&section_pages), &brand));
    }

    // Each level-1 section starts on its own page (except the first one, which
    // already follows the TOC/colofon page-break). Level-2 sections (per-room
    // details) keep flowing so multiple rooms fit on one page when there's room.
    let mut prev_level: Option<u32> = None;
    for section in &data.sections {
        if section.level == 1 && prev_level.is_some() {
            flowables.push(Box::new(PageBreak));
        }
        flowables.push(Box::new(make_section_heading(&section.title, section.level)));
        flowables.push(Box::new(Spacer::from_mm(2.0)));
        for block in &section.content {
            flowables.extend(render_block(block, &brand));
        }
        flowables.push(Box::new(Spacer::from_mm(6.0)));
        prev_level = Some(section.level);
    }

    if let Some(bc) = &data.backcover {
        let ctx = BackcoverContext {
            project: &data.project,
            subtitle: data.cover.as_ref().and_then(|c| c.subtitle.as_deref()),
            client: data
                .client
                .as_deref()
                .or_else(|| data.colofon.as_ref().and_then(|c| c.opdrachtgever_naam.as_deref())),
            adviseur: data
                .colofon
                .as_ref()
                .and_then(|c| c.adviseur_bedrijf.as_deref()),
            author: Some(data.author.as_str()),
            date: data.date.as_deref(),
            kenmerk: data
                .project_number
                .as_deref()
                .or_else(|| data.colofon.as_ref().and_then(|c| c.kenmerk.as_deref())),
            version: Some(data.version.as_str()),
        };
        flowables.extend(render_backcover(bc, &ctx, &brand));
    }

    doc.build_to_bytes(flowables)
        .map_err(|e| format!("PDF build failed: {e}"))
}

/// Parse a hex color string ("0F766E" or "#0F766E") into an openaec-layout
/// Color. Returns None for any non-6-hex-digit input so callers can fall
/// back to the default accent color silently.
fn parse_hex_color(hex: &str) -> Option<Color> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some(Color::rgb(r, g, b))
}

/// Decode a base64-encoded PNG/JPEG into raw bytes + intrinsic pixel
/// dimensions, ready for per-page rendering by the brand callback.
fn decode_footer_image(b64: &str) -> Result<FooterImageData, String> {
    let bytes = BASE64_STANDARD
        .decode(b64)
        .map_err(|e| format!("base64 decode: {e}"))?;
    let img =
        image::load_from_memory(&bytes).map_err(|e| format!("image decode: {e}"))?;
    let (w, h) = img.dimensions();
    Ok(FooterImageData {
        bytes,
        width_px: w,
        height_px: h,
    })
}

/// Estimate how many pages are consumed by special pages BEFORE the content
/// sections start (cover + colofon + TOC itself). Each special page ends
/// with a PageBreak so they reliably take one page each. The TOC may span
/// multiple pages when there are many entries.
fn pre_content_page_count(
    has_cover: bool,
    has_colofon: bool,
    toc_enabled: bool,
    toc: &super::schema::TocConfig,
    sections: &[Section],
) -> usize {
    let mut pages = 0;
    if has_cover {
        pages += 1;
    }
    if has_colofon {
        pages += 1;
    }
    if toc_enabled {
        // TOC entries shown at this depth — each entry is 1 line at 13pt
        // leading. A4 inner height ~720pt → ~50 entries per page. Header
        // takes ~30pt, spacer ~17pt → reduce capacity by 1 entry.
        let n = sections.iter().filter(|s| s.level <= toc.max_depth).count();
        let entries_per_page = 48usize;
        pages += ((n + entries_per_page - 1) / entries_per_page).max(1);
    }
    pages
}

/// Per-section estimated page-number based on block-type heights. Each
/// level-1 section forces a new page (matches the layout-loop in
/// generate_pdf). Level-2 sub-chapters flow inside the parent's page
/// stream. The estimate is a best-effort approximation — sufficient for
/// a TOC even when off by a page here and there for large rapports.
fn estimate_section_pages(
    sections: &[Section],
    frame_height_pt: Pt,
    pre_content_pages: usize,
) -> Vec<usize> {
    let frame_h = frame_height_pt.0;
    let mut pages = Vec::with_capacity(sections.len());
    let mut current_page = pre_content_pages + 1; // first content page
    let mut cursor_y = 0.0_f32;

    for (i, section) in sections.iter().enumerate() {
        // Level-1 sections start on a new page (forced page break in
        // generate_pdf line ~94-96), except the very first content section.
        if section.level == 1 && i > 0 {
            current_page += 1;
            cursor_y = 0.0;
        }
        pages.push(current_page);

        // Section heading height (level-1 ~26pt, level-2 ~18pt) + 6mm
        // trailing spacer (~17pt) defined in the section loop.
        let heading_h = if section.level == 1 { 26.0 } else { 18.0 };
        cursor_y += heading_h + 6.0;

        // Sum content block heights.
        for block in &section.content {
            cursor_y += estimate_block_height(block);
            // Advance page when the cursor overshoots the frame. Use a
            // while-loop because tall blocks (large images, long tables)
            // can span multiple pages.
            while cursor_y > frame_h {
                current_page += 1;
                cursor_y -= frame_h;
            }
        }

        // Section-trailing spacer (~6mm = 17pt) before next section.
        cursor_y += 17.0;
    }

    pages
}

/// Rough vertical height (pt) for a single Block. Used by the TOC page-
/// number estimator. Numbers tuned against real warmteverlies rapporten:
/// table-row ≈ 16pt, paragraph-line ≈ 13pt, image takes its declared
/// width × heuristic aspect (most charts in this rapport are ~1.6:1).
fn estimate_block_height(block: &Block) -> f32 {
    match block {
        Block::Paragraph { text } => {
            // ~85 chars per line at 10pt body, 13pt leading. Long text
            // wraps to multiple lines.
            let chars = text.chars().count() as f32;
            let lines = (chars / 85.0).ceil().max(1.0);
            lines * 13.0
        }
        Block::Spacer { height_mm } => *height_mm as f32 * 2.83465,
        Block::Table { title, headers, rows, .. } => {
            // Title (~16pt) + header row (~18pt) + body rows (~16pt each)
            let title_h = if title.is_some() { 18.0 } else { 0.0 };
            let header_h = if !headers.is_empty() { 18.0 } else { 0.0 };
            title_h + header_h + rows.len() as f32 * 16.0
        }
        Block::Image { width_mm, caption, .. } => {
            // Heuristic: most charts are 1.6:1 wide:tall. Add ~14pt for
            // caption if present.
            let img_h = *width_mm as f32 / 1.6 * 2.83465;
            let cap_h = if caption.is_some() { 14.0 } else { 0.0 };
            img_h + cap_h
        }
        Block::Calculation { .. } => 50.0,
    }
}

fn make_section_heading(title: &str, level: u32) -> Paragraph {
    let mut style = ParagraphStyle::default();
    style.bold = true;
    let (size, leading) = match level {
        1 => (16.0, 20.0),
        2 => (13.0, 16.0),
        _ => (11.0, 14.0),
    };
    style.font_size = Pt(size);
    style.leading = Pt(leading);
    style.space_before = Pt(4.0);
    style.space_after = Pt(2.0);
    Paragraph::new(title.to_string(), style)
}
