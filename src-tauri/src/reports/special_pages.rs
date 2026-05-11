//! Cover, colofon, TOC and backcover rendering.
//!
//! These produce flowable streams that flow into the same DocTemplate
//! as the main content. PageBreak is used between sections so each
//! special page lands on its own physical page.

use openaec_layout::{
    Alignment, Flowable, ImageFlowable, Mm, PageBreak, Paragraph, ParagraphStyle, Pt, Spacer,
};

use super::brand::OhsBrand;
use super::schema::{BackcoverConfig, Colofon, Cover, Section, TocConfig};

/// Context passed into the backcover renderer.
///
/// Pulled together from `ReportData` + `Colofon` so the backcover can show
/// project meta (opdrachtgever, adviseur, datum, versie) and a generator
/// footer line. The fields stay optional so the renderer degrades gracefully
/// when a particular value is missing.
pub struct BackcoverContext<'a> {
    pub project: &'a str,
    pub subtitle: Option<&'a str>,
    pub client: Option<&'a str>,
    pub adviseur: Option<&'a str>,
    pub author: Option<&'a str>,
    pub date: Option<&'a str>,
    pub kenmerk: Option<&'a str>,
    pub version: Option<&'a str>,
}

pub fn render_cover(
    project: &str,
    date: &str,
    cover: &Cover,
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    let mut out: Vec<Box<dyn Flowable>> = Vec::new();

    // Top margin: smaller when an image is present (image takes the visual
    // top half of the page); generous spacing when there's just text.
    let has_image = cover
        .image
        .as_ref()
        .map(|i| !i.data.is_empty())
        .unwrap_or(false);
    out.push(Box::new(Spacer::from_mm(if has_image { 20.0 } else { 60.0 })));

    if let Some(image) = &cover.image {
        let bytes = STANDARD.decode(&image.data).unwrap_or_default();
        if !bytes.is_empty() {
            let width: Pt = Mm(170.0).into();
            if let Ok(img) = ImageFlowable::from_bytes(bytes, width) {
                out.push(Box::new(img.with_alignment(Alignment::Center)));
                out.push(Box::new(Spacer::from_mm(14.0)));
            }
        }
    }

    let mut title_style = ParagraphStyle::default();
    title_style.bold = true;
    title_style.font_size = Pt(28.0);
    title_style.leading = Pt(34.0);
    out.push(Box::new(Paragraph::new(project.to_string(), title_style)));

    if let Some(sub) = &cover.subtitle {
        out.push(Box::new(Spacer::from_mm(4.0)));
        let mut sub_style = ParagraphStyle::default();
        sub_style.font_size = Pt(16.0);
        sub_style.leading = Pt(20.0);
        out.push(Box::new(Paragraph::new(sub.clone(), sub_style)));
    }

    out.push(Box::new(Spacer::from_mm(if has_image { 20.0 } else { 40.0 })));

    let mut date_style = ParagraphStyle::default();
    date_style.font_size = Pt(11.0);
    date_style.leading = Pt(14.0);
    out.push(Box::new(Paragraph::new(date.to_string(), date_style)));

    out.push(Box::new(PageBreak));
    out
}

pub fn render_colofon(
    project: &str,
    c: &Colofon,
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();

    let mut head_style = ParagraphStyle::default();
    head_style.bold = true;
    head_style.font_size = Pt(18.0);
    head_style.leading = Pt(22.0);
    out.push(Box::new(Paragraph::new(
        "Colofon".to_string(),
        head_style,
    )));
    out.push(Box::new(Spacer::from_mm(8.0)));

    push_kv(&mut out, "Project", project);
    if let Some(v) = &c.opdrachtgever_naam {
        push_kv(&mut out, "In opdracht van", v);
    }
    if let Some(v) = &c.adviseur_bedrijf {
        push_kv(&mut out, "Adviseur", v);
    }
    if let Some(v) = &c.adviseur_naam {
        push_kv(&mut out, "Naam adviseur", v);
    }
    if let Some(v) = &c.normen {
        push_kv(&mut out, "Toegepaste Normen", v);
    }
    if let Some(v) = &c.datum {
        push_kv(&mut out, "Datum rapport", v);
    }
    if let Some(v) = &c.fase {
        push_kv(&mut out, "Fase in bouwproces", v);
    }
    if let Some(v) = &c.status_colofon {
        push_kv(&mut out, "Rapportstatus", v);
    }
    if let Some(v) = &c.kenmerk {
        push_kv(&mut out, "Documentkenmerk", v);
    }

    if !c.revision_history.is_empty() {
        out.push(Box::new(Spacer::from_mm(8.0)));
        let mut hist_style = ParagraphStyle::default();
        hist_style.bold = true;
        out.push(Box::new(Paragraph::new(
            "Revisiehistorie".to_string(),
            hist_style,
        )));
        for r in &c.revision_history {
            out.push(Box::new(Paragraph::plain(format!(
                "{} | {} | {} | {}",
                r.version, r.date, r.author, r.description
            ))));
        }
    }

    out.push(Box::new(PageBreak));
    out
}

fn push_kv(out: &mut Vec<Box<dyn Flowable>>, k: &str, v: &str) {
    out.push(Box::new(Paragraph::plain(format!("{}: {}", k, v))));
    out.push(Box::new(Spacer::from_mm(2.0)));
}

pub fn render_toc(
    toc: &TocConfig,
    sections: &[Section],
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    if !toc.enabled {
        return Vec::new();
    }
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();
    let mut head_style = ParagraphStyle::default();
    head_style.bold = true;
    head_style.font_size = Pt(18.0);
    head_style.leading = Pt(22.0);
    out.push(Box::new(Paragraph::new(toc.title.clone(), head_style)));
    out.push(Box::new(Spacer::from_mm(6.0)));
    for (i, s) in sections.iter().enumerate() {
        if s.level <= toc.max_depth {
            out.push(Box::new(Paragraph::plain(format!("{} {}", i + 1, s.title))));
        }
    }
    out.push(Box::new(PageBreak));
    out
}

pub fn render_backcover(
    bc: &BackcoverConfig,
    ctx: &BackcoverContext<'_>,
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    if !bc.enabled {
        return Vec::new();
    }
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();
    out.push(Box::new(PageBreak));
    out.push(Box::new(Spacer::from_mm(70.0)));

    let mut brand_style = ParagraphStyle::default();
    brand_style.bold = true;
    brand_style.font_size = Pt(22.0);
    brand_style.leading = Pt(28.0);
    out.push(Box::new(Paragraph::new(
        "Open Heatloss Studio".to_string(),
        brand_style,
    )));

    let mut tagline_style = ParagraphStyle::default();
    tagline_style.font_size = Pt(11.0);
    tagline_style.leading = Pt(14.0);
    out.push(Box::new(Paragraph::new(
        "Open-source warmteverliesberekening volgens ISSO 51:2023".to_string(),
        tagline_style,
    )));

    out.push(Box::new(Spacer::from_mm(14.0)));

    let mut head_style = ParagraphStyle::default();
    head_style.bold = true;
    head_style.font_size = Pt(10.0);
    head_style.leading = Pt(13.0);
    head_style.space_after = Pt(3.0);
    out.push(Box::new(Paragraph::new("Rapport".to_string(), head_style)));

    push_meta(&mut out, "Project", Some(ctx.project));
    push_meta(&mut out, "Onderwerp", ctx.subtitle);
    push_meta(&mut out, "Opdrachtgever", ctx.client);
    push_meta(&mut out, "Adviseur", ctx.adviseur);
    push_meta(&mut out, "Auteur", ctx.author);
    push_meta(&mut out, "Datum", ctx.date);
    push_meta(&mut out, "Documentkenmerk", ctx.kenmerk);
    push_meta(&mut out, "Versie", ctx.version);

    out.push(Box::new(Spacer::from_mm(20.0)));

    let mut foot_head = ParagraphStyle::default();
    foot_head.bold = true;
    foot_head.font_size = Pt(9.0);
    foot_head.leading = Pt(12.0);
    foot_head.space_after = Pt(2.0);
    out.push(Box::new(Paragraph::new(
        "Gegenereerd met Open Heatloss Studio".to_string(),
        foot_head,
    )));

    let mut foot_style = ParagraphStyle::default();
    foot_style.font_size = Pt(8.0);
    foot_style.leading = Pt(11.0);
    out.push(Box::new(Paragraph::new(
        "Pure Rust ISSO 51:2023 rekenkern · IFCX (.ifcenergy) format".to_string(),
        foot_style.clone(),
    )));
    out.push(Box::new(Paragraph::new(
        "https://github.com/OpenAEC-Foundation/open-heatloss-studio".to_string(),
        foot_style.clone(),
    )));
    out.push(Box::new(Paragraph::new(
        "MIT License · Onderdeel van het OpenAEC Foundation ecosysteem".to_string(),
        foot_style,
    )));

    out
}

fn push_meta(out: &mut Vec<Box<dyn Flowable>>, label: &str, value: Option<&str>) {
    let v = match value {
        Some(s) if !s.trim().is_empty() => s,
        _ => return,
    };
    let mut style = ParagraphStyle::default();
    style.font_size = Pt(9.0);
    style.leading = Pt(12.0);
    out.push(Box::new(Paragraph::new(
        format!("{}: {}", label, v),
        style,
    )));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::brand::OhsBrand;
    use crate::reports::schema::{BackcoverConfig, Colofon, Cover, RevisionEntry};

    #[test]
    fn cover_with_subtitle_emits_at_least_two_flowables() {
        let cover = Cover {
            subtitle: Some("Warmteverliesberekening".into()),
            image: None,
        };
        let f = render_cover("Project X", "2026-05-09", &cover, &OhsBrand::default());
        assert!(f.len() >= 2);
    }

    #[test]
    fn colofon_with_revision_history_renders() {
        let c = Colofon {
            enabled: true,
            opdrachtgever_naam: Some("X".into()),
            adviseur_bedrijf: Some("3BM".into()),
            adviseur_naam: Some("Y".into()),
            normen: Some("ISSO 51".into()),
            datum: Some("2026-05-09".into()),
            fase: None,
            status_colofon: Some("CONCEPT".into()),
            kenmerk: Some("3017".into()),
            revision_history: vec![RevisionEntry {
                version: "1.0".into(),
                date: "2026-05-09".into(),
                author: "Y".into(),
                description: "Eerste opzet".into(),
            }],
        };
        let f = render_colofon("Project X", &c, &OhsBrand::default());
        assert!(!f.is_empty());
    }

    fn test_ctx<'a>() -> BackcoverContext<'a> {
        BackcoverContext {
            project: "Project X",
            subtitle: Some("Warmteverlies"),
            client: Some("Klant"),
            adviseur: Some("3BM"),
            author: Some("Auteur"),
            date: Some("2026-05-11"),
            kenmerk: Some("3017"),
            version: Some("1.0"),
        }
    }

    #[test]
    fn backcover_disabled_returns_empty() {
        let bc = BackcoverConfig { enabled: false };
        let f = render_backcover(&bc, &test_ctx(), &OhsBrand::default());
        assert!(f.is_empty());
    }

    #[test]
    fn backcover_enabled_returns_at_least_one_flowable() {
        let bc = BackcoverConfig { enabled: true };
        let f = render_backcover(&bc, &test_ctx(), &OhsBrand::default());
        assert!(!f.is_empty());
    }

    #[test]
    fn backcover_skips_empty_meta_fields() {
        let bc = BackcoverConfig { enabled: true };
        let ctx_full = test_ctx();
        let mut ctx_sparse = test_ctx();
        ctx_sparse.client = None;
        ctx_sparse.kenmerk = None;
        ctx_sparse.adviseur = None;
        let full = render_backcover(&bc, &ctx_full, &OhsBrand::default());
        let sparse = render_backcover(&bc, &ctx_sparse, &OhsBrand::default());
        // sparse output has fewer flowables because 3 meta rows are skipped
        assert!(sparse.len() < full.len());
    }
}
