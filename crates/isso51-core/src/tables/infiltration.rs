//! Infiltration rate lookup tables.
//!
//! Bevat zowel **legacy** lookups (ISSO 51:2017 Tabel 4.3 op basis van `qv,10`)
//! als **nieuwe** norm-conforme tabellen:
//!
//! - ISSO 51:2023 Tabel 2.8 — `q_i,spec` per gebouwtype (NEN 8088-1 afgeleid)
//! - NTA 8800:2024 Tabel 11.13 — bouwjaarcorrectie `f_y`
//! - NTA 8800:2024 Tabel 11.14 — uitvoeringsvariant-correctie `f_type`
//! - NEN 8088-1 Tabel 10 — ventilatiesysteem-correctie `f_inf`
//! - NTA 8800 Tabel 11.2 — power-law exponent `n_lea`
//!
//! De legacy-functies `qi_spec_per_exterior_area` en `qi_spec_per_floor_area`
//! blijven in deze run ongewijzigd; deprecation gebeurt in Dispatch 2 nadat
//! `calc/infiltration.rs` op de nieuwe keten is omgezet.

use crate::model::enums::{ConstructionVariant, DwellingClass, VentilationSystemType};

/// Specific infiltration air flow rate q_i,spec per m² of exterior
/// construction area.
/// ISSO 51 Table 4.3.
///
/// # Arguments
/// * `qv10` - Air tightness qv,10 of the building in dm³/s
///
/// # Returns
/// q_i,spec in dm³/s per m² of exterior construction area.
pub fn qi_spec_per_exterior_area(qv10: f64) -> f64 {
    // ISSO 51 Table 4.3: qi,spec in dm³/s per m² exterior construction
    if qv10 <= 50.0 {
        0.08
    } else if qv10 <= 100.0 {
        0.16
    } else if qv10 <= 150.0 {
        0.24
    } else {
        0.32
    }
}

/// Specific infiltration air flow rate q_i,spec per m² of usable floor area.
/// ISSO 51 Table 2.8 (building-level calculation).
///
/// # Arguments
/// * `qv10` - Air tightness qv,10 of the building in dm³/s
///
/// # Returns
/// q_i,spec in dm³/s per m² of usable floor area (gebruiksoppervlak).
pub fn qi_spec_per_floor_area(qv10: f64) -> f64 {
    // ISSO 51 Table 2.8
    if qv10 <= 50.0 {
        0.04
    } else if qv10 <= 100.0 {
        0.08
    } else if qv10 <= 150.0 {
        0.12
    } else {
        0.16
    }
}

/// ISSO 51:2023 Tabel 2.8 — `q_i,spec` per gebouwtype (woningclassificatie).
///
/// Tabel 2.8 koppelt de specifieke infiltratie-luchtvolumestroom direct aan
/// de woningvorm + dakvorm (afgeleid uit NEN 8088-1). Dit is de
/// **norm-conforme** keying-tabel die de legacy [`qi_spec_per_floor_area`]
/// vervangt.
///
/// # Returns
/// `q_i,spec` in dm³/(s·m² Ag) — gebruiksoppervlak-genormaliseerd.
///
/// Bron: ISSO 51:2023 p.41 Tabel 2.8.
pub fn qi_spec_table_2_8(dwelling_class: DwellingClass) -> f64 {
    match dwelling_class {
        DwellingClass::EengezinswoningMetKap => 1.0,
        DwellingClass::EengezinswoningPlatdak => 0.7,
        DwellingClass::EtageFlatOfPortiek => 0.5,
    }
}

/// NTA 8800:2024 Tabel 11.14 — uitvoeringsvariant correctiefactor `f_type`.
///
/// Past de luchtdoorlatendheid op gebouwniveau aan op basis van de positie
/// van de woning binnen het gebouw (tussen / kop / vrijstaand).
///
/// # Returns
/// `f_type` dimensieloos: 1.0 (tussen) / 1.2 (kop) / 1.4 (vrijstaand).
///
/// Bron: NTA 8800:2024 p.487–488 Tabel 11.14.
pub fn f_type_table_11_14(variant: ConstructionVariant) -> f64 {
    match variant {
        ConstructionVariant::Tussen => 1.0,
        ConstructionVariant::Kop => 1.2,
        ConstructionVariant::Vrijstaand => 1.4,
    }
}

/// NTA 8800:2024 Tabel 11.13 — bouwjaarcorrectiefactor `f_y`.
///
/// Vermenigvuldigingsfactor op de specifieke luchtdoorlatendheid op basis van
/// het bouwjaar. Recent gebouwd → lagere lekkage. Bij onbekend bouwjaar
/// (`None`) wordt 1.0 teruggegeven (neutraal — geen op- of afslag).
///
/// # Returns
/// `f_y` dimensieloos:
/// - ≥ 2010 → 0.7
/// - 1990–2009 → 1.0
/// - 1980–1989 → 1.5
/// - 1970–1979 → 2.0
/// - < 1970 → 3.0
/// - onbekend → 1.0
///
/// Bron: NTA 8800:2024 p.486 Tabel 11.13.
pub fn f_y_table_11_13(construction_year: Option<u16>) -> f64 {
    match construction_year {
        Some(y) if y >= 2010 => 0.7,
        Some(y) if y >= 1990 => 1.0,
        Some(y) if y >= 1980 => 1.5,
        Some(y) if y >= 1970 => 2.0,
        Some(_) => 3.0,
        None => 1.0,
    }
}

/// NEN 8088-1 Tabel 10 — ventilatiesysteem-correctiefactor `f_inf`.
///
/// Correctiefactor op de infiltratie afhankelijk van het type ventilatie-
/// systeem (balanced D heeft een afwijkende drukverhouding).
///
/// **Provisorische waardes** — alleen System D is geverifieerd via de Vabi
/// DR-fixture (1.10). Voor overige systemen wordt voorlopig 1.0 teruggegeven
/// (neutrale fallback) totdat NEN 8088-1 Tabel 10 in een opvolg-iteratie
/// volledig in code is gezet. Zie
/// `docs/2026-05-12-nen8088-design-dp-verificatie.md` voor de bron.
///
/// # Returns
/// `f_inf` dimensieloos. System D = 1.10, overige systemen = 1.0 (TODO).
pub fn f_inf_table_nen8088(system: VentilationSystemType) -> f64 {
    match system {
        // Vabi DR-fixture (System D) bevestigd: f_inf = 1.10.
        VentilationSystemType::SystemD => 1.10,
        // TODO: NEN 8088-1 Tabel 10 volledige waardes voor A/B/C/E invullen
        // zodra brondocument verwerkt is. Tijdelijk safe-default 1.0.
        VentilationSystemType::SystemA
        | VentilationSystemType::SystemB
        | VentilationSystemType::SystemC
        | VentilationSystemType::SystemE => 1.0,
    }
}

/// NTA 8800:2024 Tabel 11.2 — power-law exponent `n_lea` voor de
/// drukverhouding-conversie tussen referentie-Δp (10 Pa) en design-Δp.
///
/// Standaardwaarde voor woningbouw: 0.67. Formule:
/// `q_design = q_v10;lea;ref × (Δp_design / 10)^n_lea`.
///
/// Bron: NTA 8800:2024 §11.2 Tabel 11.2.
pub const N_LEA_DEFAULT: f64 = 0.67;

/// Vabi-fit design-drukverschil voor statische warmteverlies-berekening.
///
/// **Geen norm-waarde** — empirisch bepaald via reverse-engineering van Vabi
/// DR-output (zie `docs/2026-05-12-vabi-infiltratie-keten-reproductie.md`).
/// Wordt uitsluitend gebruikt binnen `InfiltrationMethod::VabiCompat`.
/// Voor strikte NTA 8800-berekeningen geldt 4 Pa (niet hier).
///
/// _Toevallige numerieke gelijkenis met π is louter coïncidentie — het is een
/// Vabi-fit drukverschil in Pa, geen wiskundige constante._
#[allow(clippy::approx_constant)]
pub const DESIGN_DP_VABI_PA: f64 = 3.14;

/// NTA 8800-strict design-drukverschil voor de power-law conversie.
///
/// Gelijk aan de referentie-drukverschil van 10 Pa: bij Δp_design = 10 Pa
/// reduceert de power-law term `(Δp_design / 10)^n_lea` tot 1.0, zodat de
/// gemeten `qv,10`-equivalente lekkage **zonder reductie** als ontwerpwaarde
/// wordt gebruikt. Dit is de conservatieve (norm-pure) variant voor
/// `InfiltrationMethod::Nta8800Strict` — geen Vabi-empirie.
pub const DESIGN_DP_NTA8800_PA: f64 = 10.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qi_spec_qv10_100() {
        // ISSO 51 example: qv10=100 → 16 × 10⁻⁵ m³/s per m² = 0.16 dm³/s per m²
        let qi = qi_spec_per_exterior_area(100.0);
        assert!((qi - 0.16).abs() < 0.001);
    }

    #[test]
    fn test_qi_spec_qv10_50() {
        let qi = qi_spec_per_exterior_area(50.0);
        assert!((qi - 0.08).abs() < 0.001);
    }

    #[test]
    fn test_qi_spec_table_2_8_kap() {
        assert!((qi_spec_table_2_8(DwellingClass::EengezinswoningMetKap) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_qi_spec_table_2_8_platdak() {
        assert!((qi_spec_table_2_8(DwellingClass::EengezinswoningPlatdak) - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_qi_spec_table_2_8_etage_flat() {
        assert!((qi_spec_table_2_8(DwellingClass::EtageFlatOfPortiek) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_f_type_table_11_14_tussen() {
        assert!((f_type_table_11_14(ConstructionVariant::Tussen) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_f_type_table_11_14_kop() {
        assert!((f_type_table_11_14(ConstructionVariant::Kop) - 1.2).abs() < 1e-9);
    }

    #[test]
    fn test_f_type_table_11_14_vrijstaand() {
        assert!((f_type_table_11_14(ConstructionVariant::Vrijstaand) - 1.4).abs() < 1e-9);
    }

    #[test]
    fn test_f_y_table_11_13_brackets() {
        assert!((f_y_table_11_13(Some(2020)) - 0.7).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(2010)) - 0.7).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(2009)) - 1.0).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(1990)) - 1.0).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(1989)) - 1.5).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(1980)) - 1.5).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(1979)) - 2.0).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(1970)) - 2.0).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(1969)) - 3.0).abs() < 1e-9);
        assert!((f_y_table_11_13(Some(1900)) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_f_y_table_11_13_unknown_year() {
        assert!((f_y_table_11_13(None) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_f_inf_nen8088_system_d() {
        // Vabi DR-fixture bevestigde 1.10 voor balanced ventilation.
        assert!((f_inf_table_nen8088(VentilationSystemType::SystemD) - 1.10).abs() < 1e-9);
    }

    #[test]
    fn test_f_inf_nen8088_other_systems_fallback() {
        // Voorlopige safe-default 1.0 — placeholder tot Tabel 10 volledig is.
        assert!((f_inf_table_nen8088(VentilationSystemType::SystemA) - 1.0).abs() < 1e-9);
        assert!((f_inf_table_nen8088(VentilationSystemType::SystemB) - 1.0).abs() < 1e-9);
        assert!((f_inf_table_nen8088(VentilationSystemType::SystemC) - 1.0).abs() < 1e-9);
        assert!((f_inf_table_nen8088(VentilationSystemType::SystemE) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_n_lea_default_constant() {
        // NTA 8800 Tabel 11.2 — power-law exponent.
        assert!((N_LEA_DEFAULT - 0.67).abs() < 1e-9);
    }

    #[test]
    fn test_design_dp_vabi_constant() {
        // Vabi-fit 3.14 Pa (geen norm-bron, empirisch).
        assert!((DESIGN_DP_VABI_PA - 3.14).abs() < 1e-9);
    }

    #[test]
    fn test_design_dp_nta8800_constant() {
        // NTA 8800-strict: 10 Pa = referentie-drukverschil → geen reductie.
        assert!((DESIGN_DP_NTA8800_PA - 10.0).abs() < 1e-9);
    }
}
