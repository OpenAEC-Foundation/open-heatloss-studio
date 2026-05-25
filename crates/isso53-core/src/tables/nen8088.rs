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

/// f_inf volgens NEN 8088-1 Tabel 10 - Ventilatiesysteem correctiefactor
/// Vabi gebruikt typisch 1,10 voor natuurlijke ventilatie (System A)
pub fn f_inf_nen8088(ventilation_type: VentilationSystemType) -> f64 {
    match ventilation_type {
        VentilationSystemType::SystemA => 1.10, // NEN 8088-1 Tabel 10 — wijkt af van ISSO 53 Tabel 4.7 (0.80); Vabi gebruikt NEN 8088-1
        VentilationSystemType::SystemB => 1.05,
        VentilationSystemType::SystemC => 1.05,
        VentilationSystemType::SystemD => 1.00, // NEN 8088-1 Tabel 10 — wijkt af van ISSO 53 Tabel 4.7 (1.15); Vabi gebruikt NEN 8088-1
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