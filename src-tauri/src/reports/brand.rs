//! Brand tokens + page chrome for OHS reports.
//!
//! Phase-1 placeholder branding — neutral OHS palette, plain text logo,
//! thin accent line at top, page-numbers + project-name in footer.
//! Real 3BM-styling (decorative shapes, real logo, stationery PDF) follows
//! in a later PR via tenant configuration.

use openaec_layout::{Color, DrawList, Mm, PageCallback, Pt, Size};

/// Pre-decoded footer image data: raw bytes (PNG/JPEG) + intrinsic pixel
/// dimensions for aspect-correct scaling on each page.
#[derive(Debug, Clone)]
pub struct FooterImageData {
    pub bytes: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
}

/// Same shape as FooterImageData but rendered in the header zone (top of
/// each content page, above the accent line). Separate type so the
/// PageCallback can keep them apart without parameter confusion.
#[derive(Debug, Clone)]
pub struct HeaderImageData {
    pub bytes: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
}

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
            primary: Color::rgb(15, 118, 110),    // teal-700
            secondary: Color::rgb(55, 65, 81),    // gray-700
            text: Color::rgb(17, 24, 39),         // gray-900
            text_light: Color::rgb(107, 114, 128), // gray-500
            border: Color::rgb(209, 213, 219),    // gray-300
            table_header_bg: Color::rgb(15, 118, 110),
            table_header_text: Color::rgb(255, 255, 255),
        }
    }
}

impl OhsBrand {
    /// Build a `PageCallback` with the given header context.
    ///
    /// `suppress_chrome_on_last` skips header/footer drawing on the final page,
    /// used when a backcover is present and shouldn't carry the running chrome.
    pub fn page_callback(
        &self,
        project_name: &str,
        report_title: &str,
        suppress_chrome_on_last: bool,
        footer_image: Option<FooterImageData>,
        header_image: Option<HeaderImageData>,
    ) -> OhsPageCallback {
        OhsPageCallback {
            brand: *self,
            project_name: project_name.to_string(),
            report_title: report_title.to_string(),
            suppress_chrome_on_last,
            footer_image,
            header_image,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OhsPageCallback {
    pub brand: OhsBrand,
    pub project_name: String,
    pub report_title: String,
    pub suppress_chrome_on_last: bool,
    pub footer_image: Option<FooterImageData>,
    pub header_image: Option<HeaderImageData>,
}

impl PageCallback for OhsPageCallback {
    fn on_page(
        &self,
        draw: &mut DrawList,
        page_num: usize,
        total_pages: usize,
        size: Size,
    ) {
        if self.suppress_chrome_on_last && page_num == total_pages && total_pages > 1 {
            return;
        }

        let margin: Pt = Mm(12.0).into();
        let right_edge = Pt(size.width.0 - margin.0);

        // Optional header-image (logo / beeldmerk). Rendered above the
        // accent line, right-anchored, max 14mm tall. When present the
        // project-name + report-title text moves below the image so they
        // don't overlap.
        let header_image_h: Pt = if let Some(img) = &self.header_image {
            let max_w_pt: Pt = Pt(size.width.0 / 3.0);
            let max_h_pt: Pt = Mm(14.0).into();
            let aspect = img.width_px as f32 / img.height_px.max(1) as f32;
            let max_aspect = max_w_pt.0 / max_h_pt.0;
            let (w_pt, h_pt) = if aspect > max_aspect {
                (max_w_pt, Pt(max_w_pt.0 / aspect))
            } else {
                (Pt(max_h_pt.0 * aspect), max_h_pt)
            };
            let img_x = Pt(right_edge.0 - w_pt.0);
            let img_y: Pt = Mm(3.0).into();
            draw.draw_image(img.bytes.clone(), img_x, img_y, w_pt, h_pt);
            h_pt
        } else {
            Pt(0.0)
        };

        // Top: 1.5pt teal accent line — adjusted down to clear header image
        let header_y: Pt = if header_image_h.0 > 0.0 {
            let img_top: Pt = Mm(3.0).into();
            let gap: Pt = Mm(2.0).into();
            Pt(img_top.0 + header_image_h.0 + gap.0)
        } else {
            Mm(8.0).into()
        };
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

        // Footer-layout splitst zich in 2 paden:
        //
        // (A) GEEN footer-image: bestaande lay-out — thin border line +
        //     "Open Heatloss Studio" links + "page x / N" rechts, op
        //     page_h − 12mm en page_h − 10mm.
        //
        // (B) MET footer-image: image staat helemaal onderin de pagina
        //     (touching the page edge met een minimale 3mm safe-zone) en
        //     vult de volledige content-breedte. Het paginanummer schuift
        //     naar BOVEN de image i.p.v. eronder, zodat hij niet wordt
        //     overlapt. Border-line + branding-text vervallen — de image
        //     IS het visuele footer-element.

        if let Some(img) = &self.footer_image {
            // -- pad B: image-footer --
            let max_w_pt: Pt = Pt(size.width.0 - margin.0 * 2.0);
            let max_h_pt: Pt = Mm(22.0).into();
            let aspect = img.width_px as f32 / img.height_px.max(1) as f32;
            let max_aspect = max_w_pt.0 / max_h_pt.0;
            let (w_pt, h_pt) = if aspect > max_aspect {
                (max_w_pt, Pt(max_w_pt.0 / aspect))
            } else {
                (Pt(max_h_pt.0 * aspect), max_h_pt)
            };
            // Bottom anchor: image-bottom op page_h − 3mm (minimale safe-
            // zone tegen edge-cropping bij printen)
            let bottom_safe: Pt = Mm(3.0).into();
            let img_y = Pt(size.height.0 - bottom_safe.0 - h_pt.0);
            let img_x = Pt((size.width.0 - w_pt.0) / 2.0);
            draw.draw_image(img.bytes.clone(), img_x, img_y, w_pt, h_pt);

            // Paginanummer BOVEN de image (3mm gap), klein lettertype
            // rechts uitgelijnd zodat 'ie niet competeert met het beeldmerk.
            let pn_gap: Pt = Mm(3.0).into();
            let pn_y = Pt(img_y.0 - pn_gap.0);
            draw.set_font("LiberationSans", Pt(7.0));
            draw.set_fill_color(self.brand.text_light);
            let page_str = format!("{} / {}", page_num, total_pages);
            draw.draw_text_right(right_edge, pn_y, &page_str);
        } else {
            // -- pad A: text-footer (geen image) — helemaal onderin de
            //    pagina; tekst-baseline op page_h − 5mm zodat de letter
            //    ongeveer 3mm boven de page-edge eindigt. Border-line
            //    daarboven op page_h − 7mm, knap als visuele separator
            //    tussen content-frame en footer.
            let footer_y = Pt(size.height.0 - Mm(7.0).0);
            draw.set_stroke_color(self.brand.border);
            draw.set_line_width(Pt(0.5));
            draw.draw_line(margin, footer_y, right_edge, footer_y);

            let txt_y = Pt(size.height.0 - Mm(5.0).0);
            draw.set_font("LiberationSans", Pt(7.0));
            draw.set_fill_color(self.brand.text_light);
            draw.draw_text(margin, txt_y, "Open Heatloss Studio");

            let page_str = format!("{} / {}", page_num, total_pages);
            draw.draw_text_right(right_edge, txt_y, &page_str);
        }
    }
}

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
        let cb = brand.page_callback("Project X", "Warmteverliesberekening", false, None, None);
        let mut dl = openaec_layout::DrawList::new();
        let size = openaec_layout::Size {
            width: openaec_layout::Mm(210.0).into(),
            height: openaec_layout::Mm(297.0).into(),
        };
        // PageCallback::on_page must not panic
        cb.on_page(&mut dl, 1, 5, size);
        // Should have produced multiple draw operations
        assert!(!dl.ops.is_empty());
    }

    #[test]
    fn callback_suppresses_chrome_on_last_page_when_flag_set() {
        let brand = OhsBrand::default();
        let cb = brand.page_callback("Project X", "Warmteverliesberekening", true, None, None);
        let mut dl = openaec_layout::DrawList::new();
        let size = openaec_layout::Size {
            width: openaec_layout::Mm(210.0).into(),
            height: openaec_layout::Mm(297.0).into(),
        };
        cb.on_page(&mut dl, 5, 5, size);
        assert!(
            dl.ops.is_empty(),
            "no draw ops should be emitted on suppressed last page"
        );
    }

    #[test]
    fn callback_draws_chrome_on_non_last_page_when_flag_set() {
        let brand = OhsBrand::default();
        let cb = brand.page_callback("Project X", "Warmteverliesberekening", true, None, None);
        let mut dl = openaec_layout::DrawList::new();
        let size = openaec_layout::Size {
            width: openaec_layout::Mm(210.0).into(),
            height: openaec_layout::Mm(297.0).into(),
        };
        cb.on_page(&mut dl, 3, 5, size);
        assert!(!dl.ops.is_empty(), "intermediate pages still get chrome");
    }
}
