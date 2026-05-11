# Rust report-engine integratie — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vervang het remote-service pad voor PDF-rapportgeneratie door een lokaal in-process Rust-pad gebaseerd op `openaec-layout` (zoals Open Calc Studio), zodat OHS desktop-mode rapporten kan maken zonder netwerktoegang.

**Architecture:** Submodule `libs/openaec-reports` levert de Rust crate `openaec-layout` als `path = "..."` dependency aan `src-tauri`. Een nieuwe `src-tauri/src/reports/` module spiegelt het OCS-patroon: serde-types voor `report.schema.json`, Tauri-commands `generate_report_pdf` en `generate_report_pdf_bytes`, en een renderer die `Vec<Box<dyn Flowable>>` opbouwt. Frontend `reportClient.ts` switcht tussen Tauri `invoke()` en de bestaande HTTP fetch op basis van runtime-detectie. TS-bouwers (`reportBuilder.ts`, `rcReportBuilder.ts`) blijven ongewijzigd.

**Tech Stack:** Rust 2021/2024, `openaec-layout` (printpdf/ttf-parser/lopdf), Tauri 2, serde_json, image, Liberation Sans TTF (open-source, embedded), TypeScript 5.

**Spec reference:** [docs/superpowers/specs/2026-05-09-rust-report-integration-design.md](../specs/2026-05-09-rust-report-integration-design.md)

**Niet in deze PR:** finale 3BM-branding, web-mode lokaal renderen, tenant-config (yaml-overrides), Typst-templates voor pixel-perfect output, decoratieve hoekvormen, stationery-PDF achtergrondlaag.

---

## File Structure

| Bestand | Status | Verantwoordelijkheid |
|---|---|---|
| `.gitmodules` | nieuw | Submodule-definitie voor `libs/openaec-reports` |
| `libs/openaec-reports/` | nieuw (submodule, gepind op SHA) | Bron van `openaec-layout` (Rust subdir van Python repo) |
| `.github/workflows/build-installer.yml` | wijzigen | `actions/checkout@v4` met `submodules: recursive` |
| `src-tauri/Cargo.toml` | wijzigen | `openaec-layout` path-dep + `image`, `serde_json` deps |
| `src-tauri/src/reports/mod.rs` | nieuw | Module-root, re-exports, `#[tauri::command]` entrypoints |
| `src-tauri/src/reports/schema.rs` | nieuw | Serde-types: `ReportData`, `Section`, `Block`, `Cover`, `Colofon`, `TocConfig`, `BackcoverConfig` |
| `src-tauri/src/reports/blocks.rs` | nieuw | Per-block-type rendering: `paragraph`, `table`, `spacer`, `image`, `calculation` |
| `src-tauri/src/reports/special_pages.rs` | nieuw | Cover, colofon, toc, backcover rendering |
| `src-tauri/src/reports/brand.rs` | nieuw | OHS brand-kleuren + `PageCallback` impl voor header/footer/page-numbers |
| `src-tauri/src/reports/fonts.rs` | nieuw | Bundle Liberation Sans als `&'static [u8]` via `include_bytes!` |
| `src-tauri/src/reports/generator.rs` | nieuw | `generate_pdf(data: &ReportData) -> Result<Vec<u8>>` orchestrator |
| `src-tauri/resources/fonts/LiberationSans-Regular.ttf` | nieuw (binary) | Body font |
| `src-tauri/resources/fonts/LiberationSans-Bold.ttf` | nieuw (binary) | Bold font |
| `src-tauri/resources/fonts/LiberationSans-Italic.ttf` | nieuw (binary) | Italic font |
| `src-tauri/resources/fonts/LiberationSans-BoldItalic.ttf` | nieuw (binary) | BoldItalic |
| `src-tauri/resources/brand/ohs/logo.png` | nieuw (binary) | Placeholder logo (300×80) |
| `src-tauri/src/commands.rs` | wijzigen | Module-level `pub use crate::reports::*;` voor command-export |
| `src-tauri/src/lib.rs` | wijzigen | `mod reports;` + `.invoke_handler(tauri::generate_handler![...])` toevoegen aan command-list |
| `src-tauri/capabilities/default.json` | wijzigen | Permission entry voor nieuwe commands |
| `src-tauri/tests/reports_smoke.rs` | nieuw | Smoke-test: valid PDF bytes, > 1KB, page count |
| `frontend/src/lib/reportClient.tauri.ts` | nieuw | Tauri-only client: `invoke('generate_report_pdf_bytes', ...)` → `Blob` |
| `frontend/src/lib/reportClient.ts` | wijzigen | Switch o.b.v. `'__TAURI_INTERNALS__' in window` |

---

## Task 1: Submodule toevoegen

**Files:**
- Create: `.gitmodules`
- Create: `libs/openaec-reports/` (via submodule)

- [ ] **Step 1.1: Add submodule**

```powershell
git submodule add https://github.com/OpenAEC-Foundation/openaec-reports.git libs/openaec-reports
git submodule update --init --recursive
```

- [ ] **Step 1.2: Pin op specifieke commit-SHA**

We pinnen op de current `main` HEAD om reproducibility te garanderen. Lees de huidige SHA:

```powershell
cd libs/openaec-reports; git rev-parse HEAD
```

Schrijf de SHA op (bijv. `abc1234...`). Het is automatisch vastgelegd in de outer commit doordat de submodule-pointer een commit-SHA is.

- [ ] **Step 1.3: Verify path-deps bestaan**

Run: `Test-Path libs/openaec-reports/rust/crates/openaec-layout/Cargo.toml`
Expected: `True`

Run: `Get-Content libs/openaec-reports/rust/crates/openaec-layout/Cargo.toml | Select-String "name = "`
Expected: `name = "openaec-layout"`

- [ ] **Step 1.4: Commit submodule-add**

```powershell
git add .gitmodules libs/openaec-reports
git commit -m "feat(reports): vendor openaec-reports as submodule"
```

---

## Task 2: Cargo dependency wiring

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 2.1: Voeg path-dep toe**

In `src-tauri/Cargo.toml`, onder `[dependencies]`, na de bestaande `isso51-core = { path = "../crates/isso51-core" }` regel:

```toml
openaec-layout = { path = "../libs/openaec-reports/rust/crates/openaec-layout" }
image = "0.25"
```

- [ ] **Step 2.2: Run cargo check**

Run: `cd src-tauri; cargo check 2>&1`
Expected: compilatie succeeds. Tuck error: als edition mismatch faalt, zie de "Open vraag #1" in de spec — switch onze workspace naar edition 2024.

- [ ] **Step 2.3: Verify openaec-layout types beschikbaar**

Maak een tijdelijk test-bestand `src-tauri/src/reports_probe.rs`:

```rust
#[allow(dead_code)]
fn _probe() {
    use openaec_layout::{Pt, Mm, A4, Color, Paragraph, Spacer};
    let _: Pt = Mm(10.0).into();
    let _ = A4;
    let _ = Color::rgb(0, 0, 0);
    let _ = Paragraph::plain("hello");
    let _ = Spacer::from_mm(5.0);
}
```

Voeg `mod reports_probe;` toe aan `src-tauri/src/lib.rs` (tijdelijk).
Run: `cd src-tauri; cargo check`
Expected: succeeds.

- [ ] **Step 2.4: Verwijder probe en commit**

```powershell
git rm src-tauri/src/reports_probe.rs
# verwijder ook de mod line in lib.rs
git add src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat(reports): add openaec-layout path dependency"
```

---

## Task 3: Schema-types voor `report.schema.json` subset

**Files:**
- Create: `src-tauri/src/reports/mod.rs`
- Create: `src-tauri/src/reports/schema.rs`
- Test: `src-tauri/src/reports/schema.rs` (inline `#[cfg(test)]`)

- [ ] **Step 3.1: Test eerst — parse minimal report.json**

Maak `src-tauri/src/reports/schema.rs` met allereerst de testen:

```rust
//! Serde types for the `report.schema.json` subset OHS produces.
//!
//! This is a deliberate subset — only the fields the TS builders in
//! `frontend/src/lib/reportBuilder.ts` and `rcReportBuilder.ts` actually emit.

use serde::Deserialize;

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
            Block::Table { title, headers, rows } => {
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
            Block::Calculation { title, result, unit, reference } => {
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
```

- [ ] **Step 3.2: Schrijf de types — minimaal voor tests**

In hetzelfde bestand, vóór de tests:

```rust
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum PaperFormat { A4, A3 }
fn default_format() -> PaperFormat { PaperFormat::A4 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    #[default] Portrait,
    Landscape,
}

fn default_author() -> String { "Onbekend".into() }
fn default_version() -> String { "1.0".into() }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReportStatus {
    #[default] CONCEPT,
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
fn default_toc_title() -> String { "Inhoudsopgave".into() }
fn default_toc_depth() -> u32 { 2 }

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
fn default_level() -> u32 { 1 }

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

fn default_spacer() -> f64 { 4.0 }
fn default_image_width() -> f64 { 150.0 }

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
    #[default] Center,
    Right,
}
```

Schrijf in `src-tauri/src/reports/mod.rs`:

```rust
//! PDF report generation for warmteverlies reports.
//!
//! Implementation pattern mirrors Open Calc Studio's `src-tauri/src/reports/`.
pub mod schema;
```

- [ ] **Step 3.3: Wire reports module in lib.rs**

In `src-tauri/src/lib.rs`, voeg toe (na bestaande `mod commands;`):

```rust
mod reports;
```

- [ ] **Step 3.4: Run tests**

Run: `cd src-tauri; cargo test --lib reports::schema::tests 2>&1`
Expected: 6 tests passed (parses_minimal_report, parses_table_block, parses_paragraph_with_html_inline, parses_image_with_inline_data, parses_calculation_block, parses_spacer)

- [ ] **Step 3.5: Commit**

```powershell
git add src-tauri/src/reports/mod.rs src-tauri/src/reports/schema.rs src-tauri/src/lib.rs
git commit -m "feat(reports): add ReportData serde schema with tests"
```

---

## Task 4: Bundle Liberation Sans fonts

**Files:**
- Create: `src-tauri/resources/fonts/LiberationSans-Regular.ttf`
- Create: `src-tauri/resources/fonts/LiberationSans-Bold.ttf`
- Create: `src-tauri/resources/fonts/LiberationSans-Italic.ttf`
- Create: `src-tauri/resources/fonts/LiberationSans-BoldItalic.ttf`
- Create: `src-tauri/src/reports/fonts.rs`

- [ ] **Step 4.1: Download Liberation Sans v2.1.5**

Liberation Sans is SIL Open Font License (OFL), redistributable.

```powershell
$url = "https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz"
$tmp = "$env:TEMP\liberation.tar.gz"
Invoke-WebRequest -Uri $url -OutFile $tmp
$dst = "$env:TEMP\liberation-extract"
New-Item -ItemType Directory -Force -Path $dst | Out-Null
tar -xzf $tmp -C $dst
$src = Get-ChildItem -Recurse -Path $dst -Filter "LiberationSans-*.ttf" | Where-Object { $_.Name -in @("LiberationSans-Regular.ttf","LiberationSans-Bold.ttf","LiberationSans-Italic.ttf","LiberationSans-BoldItalic.ttf") }
$src | Copy-Item -Destination "src-tauri/resources/fonts/" -Force
```

Verify: `Get-ChildItem src-tauri/resources/fonts/`
Expected: 4 TTF files, each ~150-200KB.

- [ ] **Step 4.2: Schrijf fonts module**

```rust
//! Embedded fonts. SIL OFL 1.1 — see resources/fonts/LICENSE.
use openaec_layout::{FontId, FontRegistry};

const REGULAR: &[u8] = include_bytes!("../../resources/fonts/LiberationSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../../resources/fonts/LiberationSans-Bold.ttf");
const ITALIC: &[u8] = include_bytes!("../../resources/fonts/LiberationSans-Italic.ttf");
const BOLD_ITALIC: &[u8] = include_bytes!("../../resources/fonts/LiberationSans-BoldItalic.ttf");

pub struct Fonts {
    pub regular: FontId,
    pub bold: FontId,
    pub italic: FontId,
    pub bold_italic: FontId,
}

pub fn register(registry: &mut FontRegistry) -> Result<Fonts, String> {
    Ok(Fonts {
        regular:     registry.register("LiberationSans",            REGULAR.to_vec()).map_err(|e| e.to_string())?,
        bold:        registry.register("LiberationSans-Bold",       BOLD.to_vec()).map_err(|e| e.to_string())?,
        italic:      registry.register("LiberationSans-Italic",     ITALIC.to_vec()).map_err(|e| e.to_string())?,
        bold_italic: registry.register("LiberationSans-BoldItalic", BOLD_ITALIC.to_vec()).map_err(|e| e.to_string())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use openaec_layout::FontRegistry;

    #[test]
    fn registers_all_four_fonts() {
        let mut reg = FontRegistry::default();
        let fonts = register(&mut reg).unwrap();
        // FontId is opaque; just check we got distinct ids
        assert_ne!(fonts.regular, fonts.bold);
        assert_ne!(fonts.italic, fonts.bold_italic);
    }
}
```

NOTE: De exacte API van `FontRegistry::default()` / `register(name, bytes)` moet matchen wat `openaec-layout` exposeert. Check tijdens implementatie via `cargo doc -p openaec-layout --open` of door `libs/openaec-reports/rust/crates/openaec-layout/src/fonts.rs` te lezen. Pas zo nodig aan.

- [ ] **Step 4.3: Add LICENSE-FONTS.txt**

Schrijf naar `src-tauri/resources/fonts/LICENSE.txt`:

```
Liberation Fonts version 2.1.5
SIL Open Font License 1.1 — https://scripts.sil.org/OFL
Copyright (c) 2010 Red Hat, Inc., 2012 Google Corporation. All rights reserved.

For the full license text, see:
https://github.com/liberationfonts/liberation-fonts/blob/main/LICENSE
```

- [ ] **Step 4.4: Wire fonts module**

In `src-tauri/src/reports/mod.rs`, voeg toe na `pub mod schema;`:

```rust
pub mod fonts;
```

- [ ] **Step 4.5: Run tests**

Run: `cd src-tauri; cargo test --lib reports::fonts::tests 2>&1`
Expected: 1 test passed (registers_all_four_fonts).

- [ ] **Step 4.6: Commit**

```powershell
git add src-tauri/resources/fonts src-tauri/src/reports/fonts.rs src-tauri/src/reports/mod.rs
git commit -m "feat(reports): embed Liberation Sans fonts (OFL 1.1)"
```

---

## Task 5: Brand-tokens + page callback

**Files:**
- Create: `src-tauri/src/reports/brand.rs`

- [ ] **Step 5.1: Schrijf de test eerst**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_color_is_teal() {
        let p = OhsBrand::default().primary;
        assert_eq!(p.r, 15);
        assert_eq!(p.g, 118);
        assert_eq!(p.b, 110);
    }

    #[test]
    fn callback_does_not_panic_on_first_page() {
        let brand = OhsBrand::default();
        let cb = brand.page_callback("Project X", "Warmteverliesberekening");
        let mut dl = openaec_layout::DrawList::new();
        let size = openaec_layout::Size {
            width: openaec_layout::Mm(210.0).into(),
            height: openaec_layout::Mm(297.0).into(),
        };
        // PageCallback::on_page must not panic
        use openaec_layout::PageCallback;
        cb.on_page(&mut dl, 1, 5, size);
    }
}
```

- [ ] **Step 5.2: Implementeer**

```rust
//! Brand tokens + page chrome for OHS reports.
//!
//! Phase-1 placeholder branding — neutral OHS palette, plain text logo,
//! thin accent line at top, page-numbers + project-name in footer.
//! Real 3BM-styling (decorative shapes, real logo, stationery PDF) follows
//! in a later PR via tenant configuration.

use openaec_layout::{Color, DrawList, Mm, PageCallback, Pt, Size};

#[derive(Debug, Clone, Copy)]
pub struct OhsBrand {
    pub primary: Color,
    pub secondary: Color,
    pub text: Color,
    pub text_light: Color,
    pub border: Color,
    pub table_header_bg: Color,
    pub table_header_text: Color,
}

impl Default for OhsBrand {
    fn default() -> Self {
        Self {
            primary:           Color::rgb(15, 118, 110),   // teal-700
            secondary:         Color::rgb(55, 65, 81),     // gray-700
            text:              Color::rgb(17, 24, 39),     // gray-900
            text_light:        Color::rgb(107, 114, 128),  // gray-500
            border:            Color::rgb(209, 213, 219),  // gray-300
            table_header_bg:   Color::rgb(15, 118, 110),
            table_header_text: Color::rgb(255, 255, 255),
        }
    }
}

impl OhsBrand {
    /// Build a `PageCallback` with the given header context.
    pub fn page_callback(&self, project_name: &str, report_title: &str) -> OhsPageCallback {
        OhsPageCallback {
            brand: *self,
            project_name: project_name.to_string(),
            report_title: report_title.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OhsPageCallback {
    pub brand: OhsBrand,
    pub project_name: String,
    pub report_title: String,
}

impl PageCallback for OhsPageCallback {
    fn on_page(
        &self,
        draw: &mut DrawList,
        page_num: usize,
        total_pages: usize,
        size: Size,
    ) {
        let margin: Pt = Mm(12.0).into();
        let right_edge = Pt(size.width.0 - margin.0);

        // Top: 1.5pt teal accent line
        let header_y: Pt = Mm(8.0).into();
        draw.set_stroke_color(self.brand.primary);
        draw.set_line_width(Pt(1.5));
        draw.draw_line(margin, header_y, right_edge, header_y);

        // Top-left: project name (small, gray)
        draw.set_font("LiberationSans-Bold", Pt(9.0));
        draw.set_fill_color(self.brand.secondary);
        draw.draw_text(margin, Mm(5.0).into(), &self.project_name);

        // Top-right: report title
        draw.set_font("LiberationSans", Pt(8.0));
        draw.set_fill_color(self.brand.text_light);
        draw.draw_text_right(right_edge, Mm(5.0).into(), &self.report_title);

        // Bottom: thin border
        let footer_y = Pt(size.height.0 - Mm(12.0).0);
        draw.set_stroke_color(self.brand.border);
        draw.set_line_width(Pt(0.5));
        draw.draw_line(margin, footer_y, right_edge, footer_y);

        // Bottom-left: branding
        let txt_y = Pt(size.height.0 - Mm(10.0).0);
        draw.set_font("LiberationSans", Pt(7.0));
        draw.set_fill_color(self.brand.text_light);
        draw.draw_text(margin, txt_y, "Open Heatloss Studio");

        // Bottom-right: page x of N
        let page_str = format!("{} / {}", page_num, total_pages);
        draw.draw_text_right(right_edge, txt_y, &page_str);
    }
}
```

- [ ] **Step 5.3: Wire brand module**

In `src-tauri/src/reports/mod.rs` voeg toe:
```rust
pub mod brand;
```

- [ ] **Step 5.4: Run tests**

Run: `cd src-tauri; cargo test --lib reports::brand::tests 2>&1`
Expected: 2 tests passed.

NOTE: `Color { r, g, b }` accessors moeten matchen met openaec-layout's actual struct. Als het `Color::rgb(...)` is en `r/g/b` private zijn, gebruik in test `Color::rgb(15, 118, 110) == Color::rgb(...)` PartialEq.

- [ ] **Step 5.5: Commit**

```powershell
git add src-tauri/src/reports/brand.rs src-tauri/src/reports/mod.rs
git commit -m "feat(reports): add OHS brand tokens and page callback"
```

---

## Task 6: Block renderers — paragraph + spacer

**Files:**
- Create: `src-tauri/src/reports/blocks.rs`

- [ ] **Step 6.1: Test eerst**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::schema::*;

    #[test]
    fn paragraph_strips_html_inline() {
        let b = Block::Paragraph { text: "<b>Hi</b>".into() };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 1, "exactly one flowable per paragraph");
    }

    #[test]
    fn spacer_produces_one_flowable() {
        let b = Block::Spacer { height_mm: 10.0 };
        let f = render_block(&b, &test_brand());
        assert_eq!(f.len(), 1);
    }

    fn test_brand() -> crate::reports::brand::OhsBrand {
        crate::reports::brand::OhsBrand::default()
    }
}
```

- [ ] **Step 6.2: Implementeer paragraph + spacer**

```rust
//! Render `Block` variants into `Box<dyn Flowable>`.
use openaec_layout::{Flowable, Mm, Paragraph, Spacer};

use super::brand::OhsBrand;
use super::schema::Block;

/// Convert a `Block` to a list of flowables (some blocks emit caption + image).
pub fn render_block(block: &Block, brand: &OhsBrand) -> Vec<Box<dyn Flowable>> {
    match block {
        Block::Paragraph { text } => vec![Box::new(render_paragraph(text, brand))],
        Block::Spacer { height_mm } => vec![Box::new(Spacer::from_mm(*height_mm as f32))],
        // Other variants implemented in later tasks
        _ => vec![Box::new(Spacer::from_mm(2.0))],
    }
}

fn render_paragraph(text: &str, _brand: &OhsBrand) -> Paragraph {
    // The TS builders use limited inline HTML: <b>, <i>, <br>.
    // openaec-layout's Paragraph supports these via parse_inline_html OR plain.
    // For phase 1 we strip tags pragmatically — the visual difference is small
    // and avoids depending on rich-text features.
    let stripped = strip_html_tags(text);
    Paragraph::plain(stripped)
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}
```

NOTE: If `openaec-layout::Paragraph` has `parse_inline_html` or rich-text method, use that instead of `strip_html_tags`. Check during implementation.

- [ ] **Step 6.3: Wire blocks module**

In `src-tauri/src/reports/mod.rs`:
```rust
pub mod blocks;
```

- [ ] **Step 6.4: Run tests**

Run: `cd src-tauri; cargo test --lib reports::blocks::tests 2>&1`
Expected: 2 tests passed.

- [ ] **Step 6.5: Commit**

```powershell
git add src-tauri/src/reports/blocks.rs src-tauri/src/reports/mod.rs
git commit -m "feat(reports): render paragraph and spacer blocks"
```

---

## Task 7: Block renderers — table

**Files:**
- Modify: `src-tauri/src/reports/blocks.rs`

- [ ] **Step 7.1: Test eerst**

Voeg toe aan `mod tests`:

```rust
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
```

- [ ] **Step 7.2: Implementeer table-rendering**

Vervang in `render_block` de `Block::Table` branch:

```rust
Block::Table { title, headers, rows } => {
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();
    if let Some(t) = title {
        out.push(Box::new(make_table_title(t)));
    }
    out.push(Box::new(render_table(headers, rows, brand)));
    out
}
```

Voeg toe onder de file:

```rust
use openaec_layout::{CellContent, Color, Mm, Table, TableStyleConfig};

fn make_table_title(text: &str) -> Paragraph {
    // Simple bold paragraph. Use ParagraphStyle if openaec-layout exposes it.
    Paragraph::plain(text)
}

fn render_table(headers: &[String], rows: &[Vec<String>], brand: &OhsBrand) -> Table {
    let cols = headers.len();
    let n_rows = rows.len() + 1;

    let mut data: Vec<Vec<CellContent>> = Vec::with_capacity(n_rows);
    data.push(headers.iter().map(|h| CellContent::Text(h.clone())).collect());
    for row in rows {
        let mut cells: Vec<CellContent> = row
            .iter()
            .map(|c| CellContent::Text(c.clone()))
            .collect();
        // Pad / truncate to header width
        cells.resize(cols, CellContent::Text(String::new()));
        data.push(cells);
    }

    let style = TableStyleConfig {
        header_bg: brand.table_header_bg,
        header_text: brand.table_header_text,
        body_text: brand.text,
        grid: brand.border,
        // Other fields use defaults (font sizes, padding) — tweak in fase 2 styling pass.
        ..Default::default()
    };

    Table::new(data, style)
}
```

NOTE: `TableStyleConfig` field names in `openaec-layout` MUST be verified against `libs/openaec-reports/rust/crates/openaec-layout/src/table.rs` during implementation. The fields above (`header_bg`, `header_text`, `body_text`, `grid`) are best-effort guesses based on the OCS callback colors. Adjust as needed.

- [ ] **Step 7.3: Run tests**

Run: `cd src-tauri; cargo test --lib reports::blocks::tests 2>&1`
Expected: 4 tests passed.

- [ ] **Step 7.4: Commit**

```powershell
git add src-tauri/src/reports/blocks.rs
git commit -m "feat(reports): render table blocks"
```

---

## Task 8: Block renderers — image + calculation

**Files:**
- Modify: `src-tauri/src/reports/blocks.rs`

- [ ] **Step 8.1: Test eerst — calculation**

```rust
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
```

- [ ] **Step 8.2: Test eerst — image (1×1 px PNG inline)**

```rust
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
    // Expect at least 1 flowable (image only, no caption)
    assert!(f.len() >= 1);
}

#[test]
fn image_with_caption_renders_two_flowables() {
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
    assert_eq!(f.len(), 2);
}
```

- [ ] **Step 8.3: Implementeer calculation**

Vervang de `Block::Calculation` branch (was wildcard fallback):

```rust
Block::Calculation { title, result, unit, reference } => {
    use openaec_layout::Spacer;
    let head = Paragraph::plain(format!("**{}**", title));
    let value = match unit {
        Some(u) => format!("{} {}", result, u),
        None => result.clone(),
    };
    let val = Paragraph::plain(value);
    let ref_text = reference
        .as_deref()
        .map(|r| format!("Ref: {}", r))
        .unwrap_or_default();
    let foot = Paragraph::plain(ref_text);
    vec![Box::new(head), Box::new(val), Box::new(foot)]
}
```

- [ ] **Step 8.4: Implementeer image**

Vervang de wildcard fallback en voeg toe:

```rust
Block::Image { src, caption, width_mm, alignment: _ } => {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use openaec_layout::{ImageData, ImageFlowable};
    let bytes = STANDARD.decode(&src.data).unwrap_or_default();
    if bytes.is_empty() {
        return vec![Box::new(Spacer::from_mm(2.0))];
    }
    let img_data = match ImageData::from_bytes(&bytes) {
        Ok(d) => d,
        Err(_) => return vec![Box::new(Spacer::from_mm(2.0))],
    };
    let mut out: Vec<Box<dyn Flowable>> = vec![
        Box::new(ImageFlowable::new(img_data, Mm(*width_mm as f32))),
    ];
    if let Some(c) = caption {
        out.push(Box::new(Paragraph::plain(c.clone())));
    }
    out
}
```

Voeg `base64` dep toe in `src-tauri/Cargo.toml`:
```toml
base64 = "0.22"
```

- [ ] **Step 8.5: Run tests**

Run: `cd src-tauri; cargo test --lib reports::blocks::tests 2>&1`
Expected: 7 tests passed.

NOTE: `ImageFlowable::new` and `ImageData::from_bytes` signatures must match openaec-layout's actual API. Verify during implementation.

- [ ] **Step 8.6: Commit**

```powershell
git add src-tauri/src/reports/blocks.rs src-tauri/Cargo.toml
git commit -m "feat(reports): render image and calculation blocks"
```

---

## Task 9: Special pages — cover + colofon + toc + backcover

**Files:**
- Create: `src-tauri/src/reports/special_pages.rs`

- [ ] **Step 9.1: Test eerst**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::brand::OhsBrand;
    use crate::reports::schema::{Cover, Colofon, RevisionEntry, BackcoverConfig};

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
                version: "1.0".into(), date: "2026-05-09".into(),
                author: "Y".into(), description: "Eerste opzet".into(),
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
```

- [ ] **Step 9.2: Implementeer**

```rust
//! Cover, colofon, TOC and backcover rendering.
use openaec_layout::{Flowable, Mm, PageBreak, Paragraph, Spacer};

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
    out.push(Box::new(Paragraph::plain(project.to_string())));
    if let Some(sub) = &cover.subtitle {
        out.push(Box::new(Spacer::from_mm(4.0)));
        out.push(Box::new(Paragraph::plain(sub.clone())));
    }
    out.push(Box::new(Spacer::from_mm(40.0)));
    out.push(Box::new(Paragraph::plain(date.to_string())));
    out.push(Box::new(PageBreak::new()));
    out
}

pub fn render_colofon(
    project: &str,
    c: &Colofon,
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    let mut out: Vec<Box<dyn Flowable>> = Vec::new();
    out.push(Box::new(Paragraph::plain(project.to_string())));
    out.push(Box::new(Spacer::from_mm(20.0)));

    push_kv(&mut out, "Project", project);
    if let Some(v) = &c.opdrachtgever_naam { push_kv(&mut out, "In opdracht van", v); }
    if let Some(v) = &c.adviseur_bedrijf { push_kv(&mut out, "Adviseur", v); }
    if let Some(v) = &c.adviseur_naam { push_kv(&mut out, "Naam adviseur", v); }
    if let Some(v) = &c.normen { push_kv(&mut out, "Toegepaste Normen", v); }
    if let Some(v) = &c.datum { push_kv(&mut out, "Datum rapport", v); }
    if let Some(v) = &c.fase { push_kv(&mut out, "Fase in bouwproces", v); }
    if let Some(v) = &c.status_colofon { push_kv(&mut out, "Rapportstatus", v); }
    if let Some(v) = &c.kenmerk { push_kv(&mut out, "Documentkenmerk", v); }

    if !c.revision_history.is_empty() {
        out.push(Box::new(Spacer::from_mm(8.0)));
        out.push(Box::new(Paragraph::plain("Revisiehistorie".into())));
        for r in &c.revision_history {
            out.push(Box::new(Paragraph::plain(format!(
                "{} | {} | {} | {}",
                r.version, r.date, r.author, r.description
            ))));
        }
    }

    out.push(Box::new(PageBreak::new()));
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
    out.push(Box::new(Paragraph::plain(toc.title.clone())));
    out.push(Box::new(Spacer::from_mm(6.0)));
    for (i, s) in sections.iter().enumerate() {
        if s.level <= toc.max_depth {
            out.push(Box::new(Paragraph::plain(format!(
                "{} {}",
                i + 1,
                s.title
            ))));
        }
    }
    out.push(Box::new(PageBreak::new()));
    out
}

pub fn render_backcover(
    bc: &BackcoverConfig,
    _brand: &OhsBrand,
) -> Vec<Box<dyn Flowable>> {
    if !bc.enabled {
        return Vec::new();
    }
    vec![
        Box::new(PageBreak::new()),
        Box::new(Spacer::from_mm(120.0)),
        Box::new(Paragraph::plain("Open Heatloss Studio".into())),
    ]
}
```

- [ ] **Step 9.3: Wire module**

In `src-tauri/src/reports/mod.rs`:
```rust
pub mod special_pages;
```

- [ ] **Step 9.4: Run tests**

Run: `cd src-tauri; cargo test --lib reports::special_pages::tests 2>&1`
Expected: 4 tests passed.

- [ ] **Step 9.5: Commit**

```powershell
git add src-tauri/src/reports/special_pages.rs src-tauri/src/reports/mod.rs
git commit -m "feat(reports): render cover, colofon, toc, backcover pages"
```

---

## Task 10: Generator + Tauri commands + smoke test

**Files:**
- Create: `src-tauri/src/reports/generator.rs`
- Modify: `src-tauri/src/reports/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`
- Create: `src-tauri/tests/reports_smoke.rs`

- [ ] **Step 10.1: Implementeer generator**

```rust
//! Top-level orchestrator: ReportData → PDF bytes.
use openaec_layout::{
    A3, A4, DocTemplate, Flowable, Frame, Mm, PageTemplate, Pt, Rect, Size,
    shared_font_registry,
};

use super::blocks::render_block;
use super::brand::OhsBrand;
use super::fonts;
use super::schema::{Orientation, PaperFormat, ReportData};
use super::special_pages::{render_backcover, render_colofon, render_cover, render_toc};

pub fn generate_pdf(data: &ReportData) -> Result<Vec<u8>, String> {
    let mut registry = shared_font_registry();
    fonts::register(&mut registry).map_err(|e| format!("font registration: {e}"))?;

    let brand = OhsBrand::default();
    let report_title = data
        .cover
        .as_ref()
        .and_then(|c| c.subtitle.clone())
        .unwrap_or_else(|| "Warmteverliesberekening".into());

    let mut doc = DocTemplate::new(&data.project, registry.clone());

    let page_size = match (data.format, data.orientation) {
        (PaperFormat::A4, Orientation::Portrait) => A4,
        (PaperFormat::A4, Orientation::Landscape) => Size {
            width: Mm(297.0).into(),
            height: Mm(210.0).into(),
        },
        (PaperFormat::A3, Orientation::Portrait) => A3,
        (PaperFormat::A3, Orientation::Landscape) => Size {
            width: Mm(420.0).into(),
            height: Mm(297.0).into(),
        },
    };

    let frame_rect = Rect::new(
        Mm(15.0).into(),
        Mm(20.0).into(),
        Pt(page_size.width.0 - Mm(30.0).0),
        Pt(page_size.height.0 - Mm(40.0).0),
    );
    let frame = Frame::new(frame_rect);

    let mut tmpl = PageTemplate::new("content", page_size, frame);
    tmpl.set_callback(brand.page_callback(&data.project, &report_title));
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
        flowables.push(Box::new(openaec_layout::Paragraph::plain(format!(
            "{} {}",
            section_prefix(section.level),
            section.title
        ))));
        flowables.push(Box::new(openaec_layout::Spacer::from_mm(4.0)));
        for block in &section.content {
            flowables.extend(render_block(block, &brand));
        }
        flowables.push(Box::new(openaec_layout::Spacer::from_mm(8.0)));
    }

    if let Some(bc) = &data.backcover {
        flowables.extend(render_backcover(bc, &brand));
    }

    doc.build_to_bytes(flowables)
        .map_err(|e| format!("PDF build failed: {e}"))
}

fn section_prefix(level: u32) -> String {
    match level {
        1 => "#".into(),
        2 => "##".into(),
        _ => "###".into(),
    }
}
```

NOTE: `DocTemplate::new`, `add_page_template`, `set_callback`, `build_to_bytes` signatures verifiëren tegen `openaec-layout/src/doc_template.rs` en `page_template.rs`.

- [ ] **Step 10.2: Implementeer commands**

In `src-tauri/src/reports/mod.rs`, voeg toe na `pub mod generator;`:

```rust
pub mod generator;

use serde_json::Value;

#[tauri::command]
pub fn generate_report_pdf(report: Value, output_path: String) -> Result<(), String> {
    let data: schema::ReportData = serde_json::from_value(report)
        .map_err(|e| format!("invalid report data: {e}"))?;
    let bytes = generator::generate_pdf(&data)?;
    std::fs::write(&output_path, &bytes)
        .map_err(|e| format!("write to {output_path}: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn generate_report_pdf_bytes(report: Value) -> Result<Vec<u8>, String> {
    let data: schema::ReportData = serde_json::from_value(report)
        .map_err(|e| format!("invalid report data: {e}"))?;
    generator::generate_pdf(&data)
}
```

- [ ] **Step 10.3: Register commands in lib.rs**

In `src-tauri/src/lib.rs`, vind de `tauri::Builder::default()` chain en voeg de twee commands toe aan `tauri::generate_handler![...]`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::calculate,
    commands::get_schema,
    commands::import_ifc,
    // ... bestaande commands ...
    reports::generate_report_pdf,
    reports::generate_report_pdf_bytes,
])
```

- [ ] **Step 10.4: Update capabilities**

In `src-tauri/capabilities/default.json` (waarschijnlijk een `permissions` array onder `core:default`):

Voeg toe:
```json
"core:webview:default",
{ "identifier": "core:default" },
"reports:generate-report-pdf",
"reports:generate-report-pdf-bytes"
```

NOTE: De exacte identifier-format (met/zonder `core:` prefix) volgt Tauri 2 conventie. Check de bestaande entries in dat bestand voor het exacte formaat.

- [ ] **Step 10.5: Schrijf smoke test**

```rust
//! End-to-end smoke test: minimal ReportData → valid PDF bytes.
use isso51_desktop_lib::reports::{generator, schema::*};

#[test]
fn generates_valid_pdf_for_minimal_input() {
    let data = ReportData {
        template: "standaard_rapport".into(),
        format: PaperFormat::A4,
        orientation: Orientation::Portrait,
        project: "Test Project".into(),
        project_number: None,
        client: None,
        author: "tester".into(),
        date: Some("2026-05-09".into()),
        version: "1.0".into(),
        status: ReportStatus::CONCEPT,
        cover: Some(Cover {
            subtitle: Some("Warmteverliesberekening".into()),
            image: None,
        }),
        colofon: None,
        toc: None,
        sections: vec![Section {
            title: "Uitgangspunten".into(),
            level: 1,
            content: vec![
                Block::Paragraph { text: "Hello".into() },
                Block::Spacer { height_mm: 4.0 },
                Block::Table {
                    title: Some("Klimaat".into()),
                    headers: vec!["Param".into(), "Waarde".into()],
                    rows: vec![
                        vec!["theta_e".into(), "-10".into()],
                        vec!["theta_b".into(), "17".into()],
                    ],
                },
            ],
        }],
        backcover: Some(BackcoverConfig { enabled: true }),
    };

    let bytes = generator::generate_pdf(&data).expect("generate succeeds");

    assert!(bytes.len() > 1000, "PDF should be at least 1KB, got {}", bytes.len());
    assert_eq!(&bytes[0..4], b"%PDF", "missing PDF magic header");

    // Try parse with lopdf to confirm structural validity
    let doc = lopdf::Document::load_mem(&bytes).expect("lopdf parses output");
    let pages = doc.get_pages();
    assert!(!pages.is_empty(), "expected at least one page");
}
```

Voeg dev-dep toe in `src-tauri/Cargo.toml` onder `[dev-dependencies]`:
```toml
[dev-dependencies]
lopdf = "0.39"
```

Make sure `mod reports;` in `lib.rs` is `pub mod reports;` zodat de integratie-test 'm kan importeren via `isso51_desktop_lib::reports::*`.

- [ ] **Step 10.6: Run smoke test**

Run: `cd src-tauri; cargo test --test reports_smoke 2>&1`
Expected: passes — PDF is valid, has at least one page.

Run: `cd src-tauri; cargo test 2>&1`
Expected: all tests pass (units + smoke).

- [ ] **Step 10.7: Commit**

```powershell
git add src-tauri/src/reports/generator.rs src-tauri/src/reports/mod.rs src-tauri/src/lib.rs src-tauri/capabilities/default.json src-tauri/tests/reports_smoke.rs src-tauri/Cargo.toml
git commit -m "feat(reports): add PDF generator + Tauri commands + smoke test"
```

---

## Task 11: Frontend wiring — switch op Tauri runtime

**Files:**
- Create: `frontend/src/lib/reportClient.tauri.ts`
- Modify: `frontend/src/lib/reportClient.ts`

- [ ] **Step 11.1: Schrijf Tauri-pad**

```ts
/**
 * Tauri-only report client: generate PDF locally via Rust.
 *
 * Used in desktop builds. Browser builds use reportClient.ts's HTTP fallback.
 */
import { invoke } from "@tauri-apps/api/core";

export async function generateReportTauri(
  reportData: Record<string, unknown>,
): Promise<Blob> {
  // Tauri returns Vec<u8> as number[] over IPC; convert to Uint8Array.
  const result = await invoke<number[]>("generate_report_pdf_bytes", {
    report: reportData,
  });
  const bytes = new Uint8Array(result);
  return new Blob([bytes], { type: "application/pdf" });
}
```

- [ ] **Step 11.2: Update reportClient.ts om te switchen**

In `frontend/src/lib/reportClient.ts`, vervang de bestaande `generateReportDirect` met:

```ts
/**
 * API-client voor OpenAEC Reports.
 *
 * Desktop (Tauri): genereert lokaal via Rust openaec-layout engine.
 * Browser: POST naar warmteverlies-backend `/api/v1/report/generate` proxy.
 */
import { getBearerToken } from "./authHeader";
import { generateReportTauri } from "./reportClient.tauri";

const REPORTS_URL = "/api/v1/report/generate";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function generateReportDirect(
  reportData: Record<string, unknown>,
): Promise<Blob> {
  if (isTauriRuntime()) {
    return generateReportTauri(reportData);
  }
  return generateReportHttp(reportData);
}

async function generateReportHttp(reportData: Record<string, unknown>): Promise<Blob> {
  const token = await getBearerToken();

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  if (import.meta.env.DEV) {
    console.log("[report] POST", REPORTS_URL, token ? "(met token)" : "(zonder token)");
  }

  const res = await fetch(REPORTS_URL, {
    method: "POST",
    headers,
    body: JSON.stringify(reportData),
  });

  if (!res.ok) {
    const errorBody = await res.text().catch(() => "");
    let detail: string;
    try {
      const json = JSON.parse(errorBody) as { detail?: string };
      detail = json.detail ?? `Rapport generatie mislukt (${res.status})`;
    } catch {
      detail = errorBody || res.statusText || `HTTP ${res.status}`;
    }
    console.error("[report] Fout response:", res.status, detail);
    throw new Error(detail);
  }

  const contentType = res.headers.get("content-type") || "";
  if (!contentType.includes("application/pdf")) {
    throw new Error("Server retourneerde geen PDF — controleer de backend configuratie.");
  }

  return res.blob();
}
```

- [ ] **Step 11.3: TypeScript-check**

Run: `cd frontend; npx tsc --noEmit 2>&1`
Expected: geen errors.

- [ ] **Step 11.4: Manueel testen — desktop**

```powershell
npm run tauri dev
```

Open de app, vul een minimaal project in, navigeer naar Resultaten, klik "Genereer rapport". Verifieer:
- Geen netwerk-call zichtbaar in dev-tools (alleen `tauri://localhost/...`)
- PDF opent / wordt opgeslagen

- [ ] **Step 11.5: Manueel testen — browser**

```powershell
cd frontend; npm run dev
```

Open localhost:5173. Klik dezelfde rapport-knop. Verifieer:
- Wel een POST naar `/api/v1/report/generate` zichtbaar
- Werkt zoals voorheen (mits backend draait)

- [ ] **Step 11.6: Commit**

```powershell
git add frontend/src/lib/reportClient.ts frontend/src/lib/reportClient.tauri.ts
git commit -m "feat(reports): switch to Tauri PDF engine in desktop, keep HTTP for browser"
```

---

## Task 12: CI submodule support + visual verification

**Files:**
- Modify: `.github/workflows/build-installer.yml` (en eventuele andere workflows die `actions/checkout` gebruiken)

- [ ] **Step 12.1: Voeg submodules-checkout toe**

In `.github/workflows/build-installer.yml`, vind elke `uses: actions/checkout@v4` (of `@v5`) en voeg `with: submodules: recursive` toe:

```yaml
- uses: actions/checkout@v4
  with:
    submodules: recursive
```

- [ ] **Step 12.2: Local CI-equivalent test**

```powershell
# Simulate fresh clone + build
$tmp = "$env:TEMP\ohs-fresh-clone"
Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
git clone --recurse-submodules . $tmp
cd $tmp; cd src-tauri; cargo build
```

Expected: succeeds. Het bouwt zowel onze workspace als de submodule's path-deps.

- [ ] **Step 12.3: Visual verification**

Genereer een test-PDF en open hem:

```powershell
cd src-tauri; cargo test --test reports_smoke -- --nocapture
```

Pas de test aan om de bytes naar disk te schrijven (`std::fs::write("../target/test_output.pdf", &bytes)`) — alleen tijdelijk. Open `target/test_output.pdf`.

Vergelijk visueel met `C:\3BM\50_projecten\7_3BM_bouwkunde\3017 Memeleiland Kavel 4\72_bouwfysica_regelgeving\Memeleiland Kavel 4.pdf`. Documenteer gaps in een commit-message of TODO-comment voor fase-2 styling. **Pixel-perfect match is GEEN acceptance-criterium**; doel is leesbaar + professioneel + alle content aanwezig.

- [ ] **Step 12.4: Commit + PR-pushen**

```powershell
git add .github/workflows/build-installer.yml
git commit -m "ci: enable recursive submodules for openaec-reports"
git push -u origin claude/laughing-kirch-752da4
```

Open een PR met de design-spec en het plan-document als referenties in de body.

---

## Self-review checklist

(Run dit als laatste task voor mergen, *niet* als steps in een eerdere task.)

- [ ] **Spec coverage:** spec sectie 5 noemt 18 bestanden. Alle in dit plan? Verify.
- [ ] **Type consistency:** `generate_report_pdf`, `generate_report_pdf_bytes` consistent in lib.rs, mod.rs, capabilities, en frontend client?
- [ ] **Open vragen:** zijn alle 10 open vragen uit spec sectie 8 of beantwoord (in commits / PR description) of expliciet uitgesteld?
- [ ] **Smoke-test runs in CI:** check `cargo test --test reports_smoke` is onderdeel van een CI-job (in build-installer.yml of een aparte test-workflow).
- [ ] **Cargo build-tijd budget:** meet cold-clean build-tijd voor en na deze PR; documenteer in PR-description als delta > 1 min.
- [ ] **License compliance:** `LICENSE.txt` voor Liberation Sans aanwezig; OFL-attributie in de OHS license-info sectie van de README of een NOTICES bestand.
