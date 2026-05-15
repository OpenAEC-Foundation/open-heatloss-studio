//! Serde types for the `report.schema.json` subset OHS produces.
//!
//! This is a deliberate subset — only the fields the TS builders in
//! `frontend/src/lib/reportBuilder.ts` and `rcReportBuilder.ts` actually emit.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ReportData {
    pub template: String,
    #[serde(default = "default_format")]
    pub format: PaperFormat,
    #[serde(default)]
    pub orientation: Orientation,
    pub project: String,
    #[serde(default)]
    pub project_number: Option<String>,
    #[serde(default)]
    pub client: Option<String>,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub status: ReportStatus,
    #[serde(default)]
    pub cover: Option<Cover>,
    #[serde(default)]
    pub colofon: Option<Colofon>,
    #[serde(default)]
    pub toc: Option<TocConfig>,
    #[serde(default)]
    pub sections: Vec<Section>,
    #[serde(default)]
    pub backcover: Option<BackcoverConfig>,
    /// Optional per-page footer image rendered on every content page in the
    /// bottom margin (above the page-number text-footer). Aspect ratio is
    /// preserved; image is scaled to fit the printable width and a fixed
    /// max-height in the bottom margin.
    #[serde(default)]
    pub footer: Option<Footer>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Footer {
    #[serde(default)]
    pub image: Option<ImageRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum PaperFormat {
    A4,
    A3,
}
fn default_format() -> PaperFormat {
    PaperFormat::A4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

fn default_author() -> String {
    "Onbekend".into()
}
fn default_version() -> String {
    "1.0".into()
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReportStatus {
    #[default]
    CONCEPT,
    DEFINITIEF,
    REVISIE,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cover {
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub image: Option<ImageRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Colofon {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub opdrachtgever_naam: Option<String>,
    #[serde(default)]
    pub adviseur_bedrijf: Option<String>,
    #[serde(default)]
    pub adviseur_naam: Option<String>,
    #[serde(default)]
    pub normen: Option<String>,
    #[serde(default)]
    pub datum: Option<String>,
    #[serde(default)]
    pub fase: Option<String>,
    #[serde(default)]
    pub status_colofon: Option<String>,
    #[serde(default)]
    pub kenmerk: Option<String>,
    #[serde(default)]
    pub revision_history: Vec<RevisionEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RevisionEntry {
    pub version: String,
    pub date: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TocConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_toc_title")]
    pub title: String,
    #[serde(default = "default_toc_depth")]
    pub max_depth: u32,
}
fn default_toc_title() -> String {
    "Inhoudsopgave".into()
}
fn default_toc_depth() -> u32 {
    2
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BackcoverConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub title: String,
    #[serde(default = "default_level")]
    pub level: u32,
    #[serde(default)]
    pub content: Vec<Block>,
}
fn default_level() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Block {
    Paragraph {
        text: String,
    },
    Table {
        #[serde(default)]
        title: Option<String>,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Spacer {
        #[serde(default = "default_spacer")]
        height_mm: f64,
    },
    Image {
        src: ImageRef,
        #[serde(default)]
        caption: Option<String>,
        #[serde(default = "default_image_width")]
        width_mm: f64,
        #[serde(default)]
        alignment: ImageAlignment,
    },
    Calculation {
        title: String,
        result: String,
        #[serde(default)]
        unit: Option<String>,
        #[serde(default)]
        reference: Option<String>,
    },
}

fn default_spacer() -> f64 {
    4.0
}
fn default_image_width() -> f64 {
    150.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageRef {
    /// Base64-encoded image bytes
    pub data: String,
    pub media_type: String,
    #[serde(default)]
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImageAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_report() {
        let json = r#"{
            "template": "standaard_rapport",
            "format": "A4",
            "orientation": "portrait",
            "project": "Test",
            "author": "x",
            "date": "2026-05-09",
            "version": "1.0",
            "status": "CONCEPT",
            "sections": []
        }"#;
        let r: ReportData = serde_json::from_str(json).unwrap();
        assert_eq!(r.project, "Test");
        assert_eq!(r.format, PaperFormat::A4);
        assert_eq!(r.orientation, Orientation::Portrait);
        assert!(r.sections.is_empty());
    }

    #[test]
    fn parses_table_block() {
        let json = r#"{ "type": "table", "title": "X",
            "headers": ["a","b"], "rows": [["1","2"],["3","4"]] }"#;
        let b: Block = serde_json::from_str(json).unwrap();
        match b {
            Block::Table {
                title,
                headers,
                rows,
            } => {
                assert_eq!(title, Some("X".into()));
                assert_eq!(headers.len(), 2);
                assert_eq!(rows.len(), 2);
            }
            _ => panic!("expected Table"),
        }
    }

    #[test]
    fn parses_paragraph_with_html_inline() {
        let json = r#"{ "type": "paragraph", "text": "<b>hi</b>" }"#;
        let b: Block = serde_json::from_str(json).unwrap();
        match b {
            Block::Paragraph { text } => assert_eq!(text, "<b>hi</b>"),
            _ => panic!("expected Paragraph"),
        }
    }

    #[test]
    fn parses_image_with_inline_data() {
        let json = r#"{ "type": "image",
            "src": { "data": "iVBOR...", "media_type": "image/png", "filename": "x.png" },
            "caption": "Test", "width_mm": 150.0, "alignment": "center" }"#;
        let b: Block = serde_json::from_str(json).unwrap();
        assert!(matches!(b, Block::Image { .. }));
    }

    #[test]
    fn parses_calculation_block() {
        let json = r#"{ "type": "calculation", "title": "Aansluitvermogen",
            "result": "6527", "unit": "W", "reference": "ISSO 51:2023" }"#;
        let b: Block = serde_json::from_str(json).unwrap();
        match b {
            Block::Calculation {
                title,
                result,
                unit,
                reference,
            } => {
                assert_eq!(title, "Aansluitvermogen");
                assert_eq!(result, "6527");
                assert_eq!(unit, Some("W".into()));
                assert_eq!(reference, Some("ISSO 51:2023".into()));
            }
            _ => panic!("expected Calculation"),
        }
    }

    #[test]
    fn parses_spacer() {
        let json = r#"{ "type": "spacer", "height_mm": 4.0 }"#;
        let b: Block = serde_json::from_str(json).unwrap();
        match b {
            Block::Spacer { height_mm } => assert_eq!(height_mm, 4.0),
            _ => panic!("expected Spacer"),
        }
    }
}
