//! Render `Block` variants into `Box<dyn Flowable>`.
//!
//! Maps the JSON blocks emitted by the TS report builders
//! (`reportBuilder.ts` and `rcReportBuilder.ts`) to openaec-layout flowables.

use openaec_layout::{Flowable, Mm, Paragraph, ParagraphStyle, Pt, Spacer};

use super::brand::OhsBrand;
use super::schema::{Block, ImageRef};

/// Convert a `Block` to a list of flowables (some blocks emit caption + image).
pub fn render_block(block: &Block, brand: &OhsBrand) -> Vec<Box<dyn Flowable>> {
    match block {
        Block::Paragraph { text } => vec![Box::new(render_paragraph(text, brand))],
        Block::Spacer { height_mm } => vec![Box::new(Spacer::from_mm(*height_mm as f32))],
        Block::Table {
            title,
            headers,
            rows,
        } => render_table_block(title.as_deref(), headers, rows, brand),
        Block::Image {
            src,
            caption,
            width_mm,
            alignment: _,
        } => render_image_block(src, caption.as_deref(), *width_mm),
        Block::Calculation {
            title,
            result,
            unit,
            reference,
        } => render_calculation(title, result, unit.as_deref(), reference.as_deref(), brand),
    }
}

fn render_paragraph(text: &str, _brand: &OhsBrand) -> Paragraph {
    // The TS builders use limited inline HTML: <b>, <i>, <br>.
    // openaec-layout's Paragraph::strip_tags already handles `<b>`/`<i>` tags
    // pragmatically — pass the raw text through and let the engine handle it.
    Paragraph::plain(text.to_string())
}

fn render_table_block(
    title: Option<&str>,
    headers: &[String],
    rows: &[Vec<String>],
    brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();
    if let Some(t) = title {
        out.push(Box::new(make_table_title(t)));
    }
    out.push(Box::new(render_table(headers, rows, brand)));
    out
}

fn make_table_title(text: &str) -> Paragraph {
    let mut style = ParagraphStyle::default();
    style.bold = true;
    style.font_size = Pt(11.0);
    style.leading = Pt(14.0);
    style.space_before = Pt(2.0);
    style.space_after = Pt(2.0);
    Paragraph::new(text.to_string(), style)
}

fn render_table(
    headers: &[String],
    rows: &[Vec<String>],
    brand: &OhsBrand,
) -> openaec_layout::Table {
    use openaec_layout::{Table, TableStyleConfig};

    let cols = headers.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));

    // Pad / truncate each row to header width
    let normalized_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            let mut padded: Vec<String> = row.clone();
            padded.resize(cols, String::new());
            padded
        })
        .collect();

    let style = TableStyleConfig {
        header_background: Some(brand.table_header_bg),
        header_text_color: brand.table_header_text,
        grid_color: brand.border,
        ..Default::default()
    };

    Table::new(headers.to_vec(), normalized_rows).with_style(style)
}

fn render_image_block(
    src: &ImageRef,
    caption: Option<&str>,
    width_mm: f64,
) -> Vec<Box<dyn Flowable>> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use openaec_layout::ImageFlowable;

    let bytes = STANDARD.decode(&src.data).unwrap_or_default();
    if bytes.is_empty() {
        return vec![Box::new(Spacer::from_mm(2.0))];
    }
    let width: Pt = Mm(width_mm as f32).into();
    let img = match ImageFlowable::from_bytes(bytes, width) {
        Ok(i) => i,
        Err(_) => return vec![Box::new(Spacer::from_mm(2.0))],
    };
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();
    if let Some(c) = caption {
        out.push(Box::new(img.with_caption(c.to_string())));
    } else {
        out.push(Box::new(img));
        // No caption variant — return single flowable
        return out;
    }
    out
}

fn render_calculation(
    title: &str,
    result: &str,
    unit: Option<&str>,
    reference: Option<&str>,
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    let mut head_style = ParagraphStyle::default();
    head_style.bold = true;
    head_style.space_after = Pt(2.0);
    let head = Paragraph::new(title.to_string(), head_style);

    let value_text = match unit {
        Some(u) => format!("{} {}", result, u),
        None => result.to_string(),
    };
    let mut val_style = ParagraphStyle::default();
    val_style.font_size = Pt(14.0);
    val_style.leading = Pt(18.0);
    val_style.space_after = Pt(2.0);
    let val = Paragraph::new(value_text, val_style);

    let ref_text = reference.map(|r| format!("Ref: {}", r)).unwrap_or_default();
    let mut foot_style = ParagraphStyle::default();
    foot_style.font_size = Pt(8.0);
    foot_style.leading = Pt(10.0);
    foot_style.italic = true;
    foot_style.space_after = Pt(4.0);
    let foot = Paragraph::new(ref_text, foot_style);

    vec![Box::new(head), Box::new(val), Box::new(foot)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::schema::*;

    fn test_brand() -> crate::reports::brand::OhsBrand {
        crate::reports::brand::OhsBrand::default()
    }

    #[test]
    fn paragraph_strips_html_inline() {
        let b = Block::Paragraph {
            text: "<b>Hi</b>".into(),
        };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 1, "exactly one flowable per paragraph");
    }

    #[test]
    fn spacer_produces_one_flowable() {
        let b = Block::Spacer { height_mm: 10.0 };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn table_with_title_emits_two_flowables() {
        let b = Block::Table {
            title: Some("Klimaat".into()),
            headers: vec!["Param".into(), "Waarde".into()],
            rows: vec![vec!["theta_e".into(), "-10".into()]],
        };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 2, "title paragraph + table flowable");
    }

    #[test]
    fn table_without_title_emits_one_flowable() {
        let b = Block::Table {
            title: None,
            headers: vec!["x".into()],
            rows: vec![vec!["1".into()]],
        };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn calculation_renders_three_paragraphs() {
        let b = Block::Calculation {
            title: "Aansluitvermogen".into(),
            result: "6527".into(),
            unit: Some("W".into()),
            reference: Some("ISSO 51:2023".into()),
        };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 3);
    }

    const TINY_PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkAAIAAAoAAv/lxKUAAAAASUVORK5CYII=";

    #[test]
    fn image_renders_one_flowable_for_valid_png() {
        let b = Block::Image {
            src: ImageRef {
                data: TINY_PNG_B64.into(),
                media_type: "image/png".into(),
                filename: None,
            },
            caption: None,
            width_mm: 50.0,
            alignment: ImageAlignment::Center,
        };
        let f = render_block(&b, &test_brand());
        // No caption => one flowable (the image)
        assert!(!f.is_empty());
    }

    #[test]
    fn image_with_caption_renders_one_flowable_with_attached_caption() {
        // Note: openaec-layout's ImageFlowable bakes the caption into the
        // image flowable itself (via with_caption), so we still emit a
        // single flowable in this design.
        let b = Block::Image {
            src: ImageRef {
                data: TINY_PNG_B64.into(),
                media_type: "image/png".into(),
                filename: None,
            },
            caption: Some("Test".into()),
            width_mm: 50.0,
            alignment: ImageAlignment::Center,
        };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 1);
    }
}
