//! NTA 8800 bijlage E — warmtegeleidingscoëfficiënten λ van bouwmaterialen.
//!
//! Deze module bevat default-waarden voor warmtegeleidingscoëfficiënten van
//! bouwmaterialen conform NTA 8800:2025+C1:2026 bijlage E. De tabel geeft voor
//! elk materiaal:
//!
//! - λ [W/(m·K)] — warmtegeleidingscoëfficiënt
//! - ρ [kg/m³] — dichtheid (optioneel)
//! - `c_p` [J/(kg·K)] — specifieke warmtecapaciteit (optioneel)
//! - μ [-] — waterdampdiffusieweerstand (optioneel)
//!
//! # Structuur
//!
//! - [`MaterialCategory`] — categorisering van materialen volgens de norm
//! - [`MaterialProperties`] — eigenschappen van één materiaal
//! - [`list_materials`] — alle beschikbare materialen
//! - [`material_by_name`] — zoek materiaal op naam
//! - [`materials_by_category`] — filter op categorie
//!
//! # Conventies
//!
//! Namen zijn Nederlandse omschrijvingen zoals in de norm, niet-gevoelig
//! voor hoofdletters in lookup-functies. Categorieën volgen de indeling
//! van bijlage E.
//!
//! Referentie: [`NTA_8800_2025_BIJLAGE_E`](crate::references::NTA_8800_2025_BIJLAGE_E).

use std::sync::LazyLock;

/// Categorieën bouwmaterialen volgens NTA 8800 bijlage E.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialCategory {
    /// Metselwerk (baksteen, kalkzandsteen, etc.)
    Metselwerk,
    /// Beton en betonproducten
    Beton,
    /// Hout en houtproducten
    Hout,
    /// Isolatiematerialen
    Isolatie,
    /// Metalen (staal, aluminium, etc.)
    Metaal,
    /// Dakbedekking en foliën
    Dakbedekking,
    /// Natuursteen
    Natuursteen,
    /// Gips en gipsproducten
    Gips,
    /// Kunststoffen
    Kunststof,
    /// Glasproducten
    Glas,
    /// Luchtspleten en spouwvulling
    Lucht,
}

/// Eigenschappen van een bouwmateriaal uit NTA 8800 bijlage E.
#[derive(Debug, Clone)]
pub struct MaterialProperties {
    /// Nederlandse naam van het materiaal
    pub name: &'static str,
    /// Categorie waartoe het materiaal behoort
    pub category: MaterialCategory,
    /// Warmtegeleidingscoëfficiënt λ in W/(m·K)
    pub lambda_w_per_mk: f64,
    /// Dichtheid ρ in kg/m³ (optioneel)
    pub rho_kg_per_m3: Option<f64>,
    /// Specifieke warmtecapaciteit `c_p` in J/(kg·K) (optioneel)
    pub c_p_j_per_kgk: Option<f64>,
    /// Waterdampdiffusieweerstand μ (dimensieloos, optioneel)
    pub mu: Option<f64>,
    /// Bronverwijzing naar normdocument
    pub source_ref: &'static str,
}

/// Statische lijst van alle materialen uit NTA 8800 bijlage E.
///
/// Deze data is gebaseerd op de default-waarden in de norm voor Nederlandse
/// bouwpraktijk. Waarden zijn conservatief gekozen binnen de bandbreedte
/// die de norm aangeeft.
const MATERIALS_RAW: &[MaterialProperties] = &[
    // Metselwerk
    MaterialProperties {
        name: "Baksteen (normale dichtheid)",
        category: MaterialCategory::Metselwerk,
        lambda_w_per_mk: 0.70,
        rho_kg_per_m3: Some(1800.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(16.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Baksteen (lichte dichtheid)",
        category: MaterialCategory::Metselwerk,
        lambda_w_per_mk: 0.45,
        rho_kg_per_m3: Some(1400.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(10.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Kalkzandsteen",
        category: MaterialCategory::Metselwerk,
        lambda_w_per_mk: 1.00,
        rho_kg_per_m3: Some(1900.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(20.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Cellenbeton",
        category: MaterialCategory::Metselwerk,
        lambda_w_per_mk: 0.14,
        rho_kg_per_m3: Some(500.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(5.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Beton
    MaterialProperties {
        name: "Gewapend beton",
        category: MaterialCategory::Beton,
        lambda_w_per_mk: 2.50,
        rho_kg_per_m3: Some(2400.0),
        c_p_j_per_kgk: Some(1000.0),
        mu: Some(130.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Ongewapend beton",
        category: MaterialCategory::Beton,
        lambda_w_per_mk: 1.75,
        rho_kg_per_m3: Some(2200.0),
        c_p_j_per_kgk: Some(1000.0),
        mu: Some(100.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Lichtbeton",
        category: MaterialCategory::Beton,
        lambda_w_per_mk: 0.65,
        rho_kg_per_m3: Some(1200.0),
        c_p_j_per_kgk: Some(1000.0),
        mu: Some(8.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Hout
    MaterialProperties {
        name: "Naaldhout",
        category: MaterialCategory::Hout,
        lambda_w_per_mk: 0.13,
        rho_kg_per_m3: Some(500.0),
        c_p_j_per_kgk: Some(1600.0),
        mu: Some(20.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Loofhout",
        category: MaterialCategory::Hout,
        lambda_w_per_mk: 0.18,
        rho_kg_per_m3: Some(700.0),
        c_p_j_per_kgk: Some(1600.0),
        mu: Some(50.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Spaanplaat",
        category: MaterialCategory::Hout,
        lambda_w_per_mk: 0.15,
        rho_kg_per_m3: Some(650.0),
        c_p_j_per_kgk: Some(1600.0),
        mu: Some(50.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "OSB",
        category: MaterialCategory::Hout,
        lambda_w_per_mk: 0.13,
        rho_kg_per_m3: Some(650.0),
        c_p_j_per_kgk: Some(1600.0),
        mu: Some(50.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Multiplex",
        category: MaterialCategory::Hout,
        lambda_w_per_mk: 0.16,
        rho_kg_per_m3: Some(600.0),
        c_p_j_per_kgk: Some(1600.0),
        mu: Some(250.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Isolatie
    MaterialProperties {
        name: "Steenwol",
        category: MaterialCategory::Isolatie,
        lambda_w_per_mk: 0.040,
        rho_kg_per_m3: Some(100.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(1.3),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Glaswol",
        category: MaterialCategory::Isolatie,
        lambda_w_per_mk: 0.040,
        rho_kg_per_m3: Some(50.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(1.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "EPS (geëxpandeerd polystyreen)",
        category: MaterialCategory::Isolatie,
        lambda_w_per_mk: 0.038,
        rho_kg_per_m3: Some(25.0),
        c_p_j_per_kgk: Some(1450.0),
        mu: Some(40.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "XPS (geëxtrudeerd polystyreen)",
        category: MaterialCategory::Isolatie,
        lambda_w_per_mk: 0.030,
        rho_kg_per_m3: Some(35.0),
        c_p_j_per_kgk: Some(1450.0),
        mu: Some(150.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "PUR/PIR schuim",
        category: MaterialCategory::Isolatie,
        lambda_w_per_mk: 0.025,
        rho_kg_per_m3: Some(40.0),
        c_p_j_per_kgk: Some(1400.0),
        mu: Some(60.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Perliet",
        category: MaterialCategory::Isolatie,
        lambda_w_per_mk: 0.045,
        rho_kg_per_m3: Some(90.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(5.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Vermiculiet",
        category: MaterialCategory::Isolatie,
        lambda_w_per_mk: 0.065,
        rho_kg_per_m3: Some(130.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(3.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Metaal
    MaterialProperties {
        name: "Staal",
        category: MaterialCategory::Metaal,
        lambda_w_per_mk: 50.0,
        rho_kg_per_m3: Some(7800.0),
        c_p_j_per_kgk: Some(460.0),
        mu: None, // Metaal is dampvrij
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Aluminium",
        category: MaterialCategory::Metaal,
        lambda_w_per_mk: 230.0,
        rho_kg_per_m3: Some(2700.0),
        c_p_j_per_kgk: Some(880.0),
        mu: None, // Metaal is dampvrij
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Koper",
        category: MaterialCategory::Metaal,
        lambda_w_per_mk: 380.0,
        rho_kg_per_m3: Some(8900.0),
        c_p_j_per_kgk: Some(380.0),
        mu: None, // Metaal is dampvrij
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Dakbedekking
    MaterialProperties {
        name: "Dakpannen (gebakken)",
        category: MaterialCategory::Dakbedekking,
        lambda_w_per_mk: 1.00,
        rho_kg_per_m3: Some(2000.0),
        c_p_j_per_kgk: Some(800.0),
        mu: Some(40.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Bitumen dakbedekking",
        category: MaterialCategory::Dakbedekking,
        lambda_w_per_mk: 0.17,
        rho_kg_per_m3: Some(1100.0),
        c_p_j_per_kgk: Some(1000.0),
        mu: Some(50_000.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "EPDM folie",
        category: MaterialCategory::Dakbedekking,
        lambda_w_per_mk: 0.25,
        rho_kg_per_m3: Some(1500.0),
        c_p_j_per_kgk: Some(1000.0),
        mu: Some(10_000.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Natuursteen
    MaterialProperties {
        name: "Graniet",
        category: MaterialCategory::Natuursteen,
        lambda_w_per_mk: 3.50,
        rho_kg_per_m3: Some(2700.0),
        c_p_j_per_kgk: Some(900.0),
        mu: Some(10_000.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Kalksteen",
        category: MaterialCategory::Natuursteen,
        lambda_w_per_mk: 2.30,
        rho_kg_per_m3: Some(2600.0),
        c_p_j_per_kgk: Some(900.0),
        mu: Some(250.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Zandsteen",
        category: MaterialCategory::Natuursteen,
        lambda_w_per_mk: 1.60,
        rho_kg_per_m3: Some(2200.0),
        c_p_j_per_kgk: Some(900.0),
        mu: Some(40.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Gips
    MaterialProperties {
        name: "Gipsplaat",
        category: MaterialCategory::Gips,
        lambda_w_per_mk: 0.21,
        rho_kg_per_m3: Some(750.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(4.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Gipsvezel plaat",
        category: MaterialCategory::Gips,
        lambda_w_per_mk: 0.32,
        rho_kg_per_m3: Some(1150.0),
        c_p_j_per_kgk: Some(840.0),
        mu: Some(12.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Kunststof
    MaterialProperties {
        name: "PVC",
        category: MaterialCategory::Kunststof,
        lambda_w_per_mk: 0.17,
        rho_kg_per_m3: Some(1400.0),
        c_p_j_per_kgk: Some(900.0),
        mu: Some(50_000.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "PE-folie",
        category: MaterialCategory::Kunststof,
        lambda_w_per_mk: 0.33,
        rho_kg_per_m3: Some(920.0),
        c_p_j_per_kgk: Some(2300.0),
        mu: Some(100_000.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Glas
    MaterialProperties {
        name: "Glas (vlakglas)",
        category: MaterialCategory::Glas,
        lambda_w_per_mk: 1.00,
        rho_kg_per_m3: Some(2500.0),
        c_p_j_per_kgk: Some(750.0),
        mu: None, // Glas is dampvrij
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    // Lucht
    MaterialProperties {
        name: "Luchtspouw (stil)",
        category: MaterialCategory::Lucht,
        lambda_w_per_mk: 0.025,
        rho_kg_per_m3: Some(1.2),
        c_p_j_per_kgk: Some(1000.0),
        mu: Some(1.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
    MaterialProperties {
        name: "Luchtspouw (geventileerd)",
        category: MaterialCategory::Lucht,
        lambda_w_per_mk: 0.15,
        rho_kg_per_m3: Some(1.2),
        c_p_j_per_kgk: Some(1000.0),
        mu: Some(1.0),
        source_ref: crate::references::NTA_8800_2025_BIJLAGE_E,
    },
];

/// Lazy-initialized mapping voor snelle lookup van alle materialen.
static MATERIALS: LazyLock<&'static [MaterialProperties]> = LazyLock::new(|| MATERIALS_RAW);

/// Geef alle beschikbare materialen uit NTA 8800 bijlage E.
///
/// De lijst bevat representatieve Nederlandse bouwmaterialen met hun
/// warmtegeleidingscoëfficiënten en andere thermofysische eigenschappen.
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::materials::list_materials;
///
/// let materials = list_materials();
/// assert!(materials.len() >= 30);
///
/// // Zoek steenwol
/// let steenwol = materials.iter()
///     .find(|m| m.name.to_lowercase().contains("steenwol"))
///     .expect("Steenwol moet aanwezig zijn");
/// assert!(steenwol.lambda_w_per_mk < 0.050);
/// ```
#[must_use]
pub fn list_materials() -> &'static [MaterialProperties] {
    &MATERIALS
}

/// Zoek materiaal op naam (niet hoofdlettergevoelig).
///
/// Geeft het eerste materiaal waarvan de naam de zoekterm bevat.
/// Voor exacte matches gebruik `==` vergelijking op de resultaat-naam.
///
/// # Parameters
/// - `name`: Zoekterm, niet hoofdlettergevoelig
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::materials::material_by_name;
///
/// // Case-insensitive zoeken
/// let steenwol = material_by_name("steenwol").expect("Moet gevonden worden");
/// assert_eq!(steenwol.name, "Steenwol");
///
/// // Gedeeltelijke match
/// let baksteen = material_by_name("baksteen").expect("Moet gevonden worden");
/// assert!(baksteen.name.contains("Baksteen"));
///
/// // Niet gevonden
/// assert!(material_by_name("onbestaandmateriaal").is_none());
/// ```
#[must_use]
pub fn material_by_name(name: &str) -> Option<&'static MaterialProperties> {
    let name_lower = name.to_lowercase();
    MATERIALS
        .iter()
        .find(|m| m.name.to_lowercase().contains(&name_lower))
}

/// Filter materialen op categorie.
///
/// # Parameters
/// - `category`: Gewenste materialencategorie
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::materials::{materials_by_category, MaterialCategory};
///
/// let isolatie = materials_by_category(MaterialCategory::Isolatie);
/// assert!(isolatie.len() >= 5);
///
/// // Alle isolatie heeft λ < 0.1 W/(m·K)
/// for materiaal in isolatie {
///     assert!(materiaal.lambda_w_per_mk < 0.1);
/// }
///
/// let hout = materials_by_category(MaterialCategory::Hout);
/// assert!(hout.iter().any(|m| m.name.contains("Naaldhout")));
/// ```
#[must_use]
pub fn materials_by_category(category: MaterialCategory) -> Vec<&'static MaterialProperties> {
    MATERIALS
        .iter()
        .filter(|m| m.category == category)
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn materials_list_has_minimum_count() {
        let materials = list_materials();
        assert!(
            materials.len() >= 30,
            "Verwacht minimaal 30 materialen, maar vond {}",
            materials.len()
        );
    }

    #[test]
    fn all_materials_have_valid_lambda() {
        let materials = list_materials();
        for material in materials {
            assert!(
                material.lambda_w_per_mk.is_finite() && material.lambda_w_per_mk > 0.0,
                "Materiaal '{}' heeft ongeldige λ-waarde: {}",
                material.name,
                material.lambda_w_per_mk
            );
        }
    }

    #[test]
    fn material_by_name_case_insensitive() {
        // Exact match (case insensitive)
        let steenwol = material_by_name("steenwol").expect("Steenwol moet gevonden worden");
        assert_eq!(steenwol.name, "Steenwol");
        assert_eq!(steenwol.category, MaterialCategory::Isolatie);

        // Uppercase zoekterm
        let steenwol_upper = material_by_name("STEENWOL").expect("Uppercase moet werken");
        assert_eq!(steenwol_upper.name, steenwol.name);

        // Gedeeltelijke match
        let baksteen = material_by_name("baksteen").expect("Gedeeltelijke match moet werken");
        assert!(baksteen.name.contains("Baksteen"));
    }

    #[test]
    fn material_by_name_not_found() {
        let result = material_by_name("onbestaandmateriaal123");
        assert!(
            result.is_none(),
            "Niet-bestaand materiaal mag niet gevonden worden"
        );
    }

    #[test]
    fn materials_by_category_isolatie() {
        let isolatie = materials_by_category(MaterialCategory::Isolatie);
        assert!(
            isolatie.len() >= 5,
            "Verwacht minimaal 5 isolatiematerialen"
        );

        // Alle isolatie heeft lage λ-waarde
        for material in &isolatie {
            assert!(
                material.lambda_w_per_mk < 0.1,
                "Isolatiemateriaal '{}' heeft te hoge λ-waarde: {}",
                material.name,
                material.lambda_w_per_mk
            );
        }

        // Steenwol moet aanwezig zijn
        assert!(
            isolatie.iter().any(|m| m.name.contains("Steenwol")),
            "Steenwol moet in isolatie-categorie zitten"
        );
    }

    #[test]
    fn materials_by_category_hout() {
        let hout = materials_by_category(MaterialCategory::Hout);
        assert!(hout.len() >= 3, "Verwacht minimaal 3 houtsoorten");

        // Naaldhout moet aanwezig zijn
        assert!(
            hout.iter().any(|m| m.name.contains("Naaldhout")),
            "Naaldhout moet in hout-categorie zitten"
        );

        // Hout heeft typische λ-waarden
        for material in &hout {
            assert!(
                (0.10..=0.20).contains(&material.lambda_w_per_mk),
                "Hout '{}' heeft atypische λ-waarde: {}",
                material.name,
                material.lambda_w_per_mk
            );
        }
    }

    #[test]
    fn materials_by_category_metaal() {
        let metaal = materials_by_category(MaterialCategory::Metaal);
        assert!(metaal.len() >= 2, "Verwacht minimaal 2 metalen");

        // Metalen hebben hoge λ-waarden
        for material in &metaal {
            assert!(
                material.lambda_w_per_mk > 10.0,
                "Metaal '{}' heeft te lage λ-waarde: {}",
                material.name,
                material.lambda_w_per_mk
            );
            // Metaal is dampvrij (μ = None)
            assert!(
                material.mu.is_none(),
                "Metaal '{}' moet dampvrij zijn (μ = None)",
                material.name
            );
        }
    }

    #[test]
    fn specific_material_values_correct() {
        // Test specifieke norm-waarden voor representatieve materialen

        // Steenwol
        let steenwol = material_by_name("steenwol").expect("Steenwol moet bestaan");
        assert_abs_diff_eq!(steenwol.lambda_w_per_mk, 0.040, epsilon = 1e-6);
        assert_eq!(steenwol.category, MaterialCategory::Isolatie);

        // Gewapend beton
        let beton = material_by_name("gewapend beton").expect("Gewapend beton moet bestaan");
        assert_abs_diff_eq!(beton.lambda_w_per_mk, 2.50, epsilon = 1e-6);
        assert_eq!(beton.category, MaterialCategory::Beton);
        assert_eq!(beton.rho_kg_per_m3, Some(2400.0));

        // Naaldhout
        let naaldhout = material_by_name("naaldhout").expect("Naaldhout moet bestaan");
        assert_abs_diff_eq!(naaldhout.lambda_w_per_mk, 0.13, epsilon = 1e-6);
        assert_eq!(naaldhout.category, MaterialCategory::Hout);

        // Staal
        let staal = material_by_name("staal").expect("Staal moet bestaan");
        assert_abs_diff_eq!(staal.lambda_w_per_mk, 50.0, epsilon = 1e-6);
        assert_eq!(staal.category, MaterialCategory::Metaal);
        assert!(staal.mu.is_none()); // Dampvrij
    }

    #[test]
    fn all_categories_represented() {
        let materials = list_materials();
        let mut found_categories = std::collections::HashSet::new();

        for material in materials {
            found_categories.insert(material.category);
        }

        // Check dat belangrijkste categorieën aanwezig zijn
        let expected_categories = [
            MaterialCategory::Metselwerk,
            MaterialCategory::Beton,
            MaterialCategory::Hout,
            MaterialCategory::Isolatie,
            MaterialCategory::Metaal,
        ];

        for expected in &expected_categories {
            assert!(
                found_categories.contains(expected),
                "Categorie {expected:?} moet vertegenwoordigd zijn"
            );
        }
    }

    #[test]
    fn source_references_consistent() {
        let materials = list_materials();
        for material in materials {
            assert_eq!(
                material.source_ref,
                crate::references::NTA_8800_2025_BIJLAGE_E,
                "Materiaal '{}' heeft inconsistente bronverwijzing",
                material.name
            );
        }
    }

    #[test]
    fn optional_properties_sensible_when_present() {
        let materials = list_materials();
        for material in materials {
            // Dichtheid: als aanwezig, moet realistisch zijn
            if let Some(rho) = material.rho_kg_per_m3 {
                assert!(
                    rho > 0.0 && rho < 20_000.0,
                    "Materiaal '{}' heeft onrealistische dichtheid: {}",
                    material.name,
                    rho
                );
            }

            // Specifieke warmtecapaciteit: als aanwezig, moet realistisch zijn
            if let Some(cp) = material.c_p_j_per_kgk {
                assert!(
                    cp > 100.0 && cp < 5000.0,
                    "Materiaal '{}' heeft onrealistische c_p: {}",
                    material.name,
                    cp
                );
            }

            // Waterdampdiffusieweerstand: als aanwezig, moet positief zijn
            if let Some(mu) = material.mu {
                assert!(
                    mu > 0.0,
                    "Materiaal '{}' heeft negatieve μ-waarde: {}",
                    material.name,
                    mu
                );
            }
        }
    }

    #[test]
    fn isolatie_materials_have_low_lambda() {
        let isolatie = materials_by_category(MaterialCategory::Isolatie);
        for material in isolatie {
            assert!(
                material.lambda_w_per_mk <= 0.070,
                "Isolatiemateriaal '{}' heeft te hoge λ: {}",
                material.name,
                material.lambda_w_per_mk
            );
        }
    }

    #[test]
    fn beton_materials_have_high_density() {
        let beton = materials_by_category(MaterialCategory::Beton);
        for material in beton {
            if let Some(rho) = material.rho_kg_per_m3 {
                assert!(
                    rho >= 1000.0,
                    "Beton '{}' heeft te lage dichtheid: {}",
                    material.name,
                    rho
                );
            }
        }
    }

    #[test]
    fn lucht_materials_special_properties() {
        let lucht = materials_by_category(MaterialCategory::Lucht);
        if !lucht.is_empty() {
            for material in lucht {
                // Lucht heeft zeer lage dichtheid
                if let Some(rho) = material.rho_kg_per_m3 {
                    assert!(
                        rho < 10.0,
                        "Luchtmateriaal '{}' heeft te hoge dichtheid: {}",
                        material.name,
                        rho
                    );
                }
                // Lucht heeft lage μ-waarde
                if let Some(mu) = material.mu {
                    assert!(
                        mu <= 1.0,
                        "Luchtmateriaal '{}' heeft te hoge μ: {}",
                        material.name,
                        mu
                    );
                }
            }
        }
    }

    #[test]
    fn material_names_unique() {
        let materials = list_materials();
        let mut names = std::collections::HashSet::new();

        for material in materials {
            assert!(
                names.insert(material.name),
                "Dubbele materiaalnaam gevonden: '{}'",
                material.name
            );
        }
    }
}
