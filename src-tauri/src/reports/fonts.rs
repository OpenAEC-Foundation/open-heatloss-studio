//! Embedded fonts. SIL OFL 1.1 — see resources/fonts/LICENSE.txt.
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

/// Register Liberation Sans variants in the given registry.
///
/// We register both "LiberationSans" and "LiberationSans-Regular" as aliases so
/// callers can use either form (the openaec-layout `Paragraph` style uses bare
/// `LiberationSans` but the table style appends `-Bold`).
pub fn register(registry: &mut FontRegistry) -> Result<Fonts, String> {
    let regular = registry
        .register_ttf_bytes("LiberationSans-Regular", REGULAR.to_vec())
        .map_err(|e| e.to_string())?;
    // Alias so plain "LiberationSans" lookups resolve to Regular
    registry.register_alias("LiberationSans", "LiberationSans-Regular");

    let bold = registry
        .register_ttf_bytes("LiberationSans-Bold", BOLD.to_vec())
        .map_err(|e| e.to_string())?;
    let italic = registry
        .register_ttf_bytes("LiberationSans-Italic", ITALIC.to_vec())
        .map_err(|e| e.to_string())?;
    let bold_italic = registry
        .register_ttf_bytes("LiberationSans-BoldItalic", BOLD_ITALIC.to_vec())
        .map_err(|e| e.to_string())?;

    Ok(Fonts {
        regular,
        bold,
        italic,
        bold_italic,
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
        assert_ne!(fonts.regular, fonts.italic);
    }

    #[test]
    fn alias_resolves_to_regular() {
        let mut reg = FontRegistry::default();
        let fonts = register(&mut reg).unwrap();
        let by_alias = reg.get("LiberationSans").expect("alias should resolve");
        assert_eq!(by_alias, fonts.regular);
    }
}
