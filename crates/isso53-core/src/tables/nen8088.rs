//! NEN 8088-1 en NTA 8800 tabellen voor Vabi-compatibele infiltratie
//!
//! Bronnen:
//! - NEN 8088-1 Tabel 9 (f_type winddrukcoëfficiënt)
//! - NEN 8088-1 Tabel 10 (f_inf ventilatiesysteem)
//! - NTA 8800 Tabel 11.13 (f_jaar bouwjaar)
//!
//! Zie: docs/2026-05-12-nta8800-infiltratie-verificatie.md

use crate::model::{GebouwTypeWinddruk, VentilationSystemType};

/// f_type volgens NEN 8088-1 Tabel 9 - Winddrukcoëfficiënt gebouwtype
/// Vabi gebruikt typisch 0,90 voor standaard gebouwen
pub fn f_type_nen8088(building_type: GebouwTypeWinddruk) -> f64 {
    match building_type {
        GebouwTypeWinddruk::EenlaagsMetKap => 0.30,
        GebouwTypeWinddruk::EenlaagsMetPlatDak => 0.90,
        GebouwTypeWinddruk::MeerlaagsStandaard => 0.90,
        GebouwTypeWinddruk::MeerlaagsVolgevelBinnengalerij => 0.94, // NEN 8088-1 Tabel 9 — wijkt af van ISSO 53 Tabel 4.6 (0.48); Vabi gebruikt NEN 8088-1
        GebouwTypeWinddruk::MeerlaagsDubbeleHuidOnderbroken => 0.94,
        GebouwTypeWinddruk::MeerlaagsDubbeleHuidDoorlopend => 1.00,
    }
}

/// f_inf voor het **Vabi-compat-pad** (`InfiltrationMethod::UnknownVabiCompat`).
///
/// ⚠️ **Geen norm-waarden.** Deze waardes zijn empirisch afgeleid uit
/// Vabi-output (reverse-engineering, zie doc-link in de module-header) en
/// wijken af van BEIDE norm-tabellen (PM-verificatie 2026-06-10 tegen de
/// bron-PDF's):
///
/// | Systeem | hier (Vabi-fit) | NEN 8088-1+C2 Tabel 10 | ISSO 53 Tabel 4.7 |
/// |---------|-----------------|------------------------|--------------------|
/// | A       | 1,10            | 0,80                   | 0,80               |
/// | B       | 1,05            | 0,85                   | 0,85               |
/// | C       | 1,05            | 1,0                    | 1,0                |
/// | D       | 1,00            | 1,10                   | 1,15               |
/// | E       | 1,00            | 1,05 (E.1)             | 1,08               |
///
/// De waardes blijven bewust staan: het Vabi-compat-pad reproduceert de
/// Vabi DR-kantoorwest golden-fixture (System D) en is expliciet als
/// niet-norm gemarkeerd (audit 02, item T3). Norm-conforme f_inf:
/// - ISSO 53 §4.2-keten → [`crate::tables::ventilation_system::f_inf`];
/// - NEN 8088-1+C2 → isso51-core `f_inf_table_nen8088`.
pub fn f_inf_nen8088(ventilation_type: VentilationSystemType) -> f64 {
    match ventilation_type {
        VentilationSystemType::SystemA => 1.10, // Vabi-fit — geen norm-waarde (zie tabel hierboven)
        VentilationSystemType::SystemB => 1.05,
        VentilationSystemType::SystemC => 1.05,
        VentilationSystemType::SystemD => 1.00, // Vabi-fit; geverifieerd via DR-kantoorwest golden
        VentilationSystemType::SystemE => 1.00, // Zone-mix met WTW
    }
}

/// f_jaar volgens NTA 8800 Tabel 11.13 - Bouwjaar correctiefactor
/// Discreet model: j ≥ 2010 → 0,7, overige jaren → zie tabel
pub fn f_jaar_nta8800(construction_year: u16) -> f64 {
    match construction_year {
        0..=1959 => 1.0,
        1960..=1975 => 0.8,
        1976..=1983 => 0.75,
        1984..=1994 => 0.72,
        1995..=2005 => 0.71,
        2006..=2009 => 0.70,
        _ => 0.70, // j ≥ 2010; NTA 8800 discreet vs ISSO 53 exponentieel (0.632 voor 2021)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f_type_nen8088() {
        assert_eq!(f_type_nen8088(GebouwTypeWinddruk::MeerlaagsStandaard), 0.90);
        assert_eq!(f_type_nen8088(GebouwTypeWinddruk::MeerlaagsDubbeleHuidDoorlopend), 1.00);
    }

    #[test]
    fn test_f_inf_nen8088() {
        assert_eq!(f_inf_nen8088(VentilationSystemType::SystemA), 1.10);
        assert_eq!(f_inf_nen8088(VentilationSystemType::SystemD), 1.00);
    }

    #[test]
    fn test_f_jaar_nta8800() {
        assert_eq!(f_jaar_nta8800(1950), 1.0);
        assert_eq!(f_jaar_nta8800(1970), 0.8);
        assert_eq!(f_jaar_nta8800(2015), 0.70);
        assert_eq!(f_jaar_nta8800(2025), 0.70);
    }
}