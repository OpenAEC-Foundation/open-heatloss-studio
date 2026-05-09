//! Cover, colofon, TOC and backcover rendering.
//!
//! These produce flowable streams that flow into the same DocTemplate
//! as the main content. PageBreak is used between sections so each
//! special page lands on its own physical page.

use openaec_layout::{Flowable, PageBreak, Paragraph, ParagraphStyle, Pt, Spacer};

use super::brand::OhsBrand;
use super::schema::{BackcoverConfig, Colofon, Cover, Section, TocConfig};

pub fn render_cover(
    project: &str,
    date: &str,
    cover: &Cover,
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();
    out.push(Box::new(Spacer::from_mm(60.0)));

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

    out.push(Box::new(Spacer::from_mm(40.0)));

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

pub fn render_backcover(bc: &BackcoverConfig, _brand: &OhsBrand) -> Vec<Box<dyn Flowable>> {
    if !bc.enabled {
        return Vec::new();
    }
    let mut style = ParagraphStyle::default();
    style.bold = true;
    style.font_size = Pt(14.0);
    vec![
        Box::new(PageBreak),
        Box::new(Spacer::from_mm(120.0)),
        Box::new(Paragraph::new(
            "Open Heatloss Studio".to_string(),
            style,
        )),
    ]
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

    #[test]
    fn backcover_disabled_returns_empty() {
        let bc = BackcoverConfig { enabled: false };
        let f = render_backcover(&bc, &OhsBrand::default());
        assert!(f.is_empty());
    }

    #[test]
    fn backcover_enabled_returns_at_least_one_flowable() {
        let bc = BackcoverConfig { enabled: true };
        let f = render_backcover(&bc, &OhsBrand::default());
        assert!(!f.is_empty());
    }
}
