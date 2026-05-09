//! Top-level orchestrator: ReportData -> PDF bytes.
//!
//! Builds a flowable stream from cover/colofon/toc/sections/backcover and
//! hands it to openaec-layout's DocTemplate which produces the actual PDF.

use openaec_layout::{
    A3, A4, DocTemplate, Flowable, Frame, Mm, PageTemplate, Paragraph, ParagraphStyle, Pt, Rect,
    Spacer, shared_font_registry,
};

use super::blocks::render_block;
use super::brand::OhsBrand;
use super::fonts;
use super::schema::{Orientation, PaperFormat, ReportData};
use super::special_pages::{render_backcover, render_colofon, render_cover, render_toc};

pub fn generate_pdf(data: &ReportData) -> Result<Vec<u8>, String> {
    let registry = shared_font_registry();
    {
        let mut guard = registry.lock().map_err(|e| format!("font lock: {e}"))?;
        fonts::register(&mut *guard).map_err(|e| format!("font registration: {e}"))?;
    }

    let brand = OhsBrand::default();
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

    // Frame: 15mm horizontal margins, 20mm top, 20mm bottom (so total
    // vertical margin is 40mm — leaves ~257mm content height on A4).
    let frame_x: Pt = Mm(15.0).into();
    let frame_y: Pt = Mm(20.0).into();
    let frame_w = Pt(page_size.width.0 - Mm(30.0).0);
    let frame_h = Pt(page_size.height.0 - Mm(40.0).0);
    let frame = Frame::new(Rect::new(frame_x, frame_y, frame_w, frame_h));

    let tmpl = PageTemplate::new("content", page_size, frame).with_callback(Box::new(
        brand.page_callback(&data.project, &report_title),
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
        flowables.extend(render_toc(t, &data.sections, &brand));
    }

    for section in &data.sections {
        flowables.push(Box::new(make_section_heading(&section.title, section.level)));
        flowables.push(Box::new(Spacer::from_mm(2.0)));
        for block in &section.content {
            flowables.extend(render_block(block, &brand));
        }
        flowables.push(Box::new(Spacer::from_mm(6.0)));
    }

    if let Some(bc) = &data.backcover {
        flowables.extend(render_backcover(bc, &brand));
    }

    doc.build_to_bytes(flowables)
        .map_err(|e| format!("PDF build failed: {e}"))
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
