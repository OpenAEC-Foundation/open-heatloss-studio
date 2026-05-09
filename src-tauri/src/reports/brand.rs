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
        cb.on_page(&mut dl, 1, 5, size);
        // Should have produced multiple draw operations
        assert!(!dl.ops.is_empty());
    }
}
