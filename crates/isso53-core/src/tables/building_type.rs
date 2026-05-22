//! Gebouwtype-correctiefactoren voor infiltratie.
//!
//! Bronnen:
//! - ISSO 53 (2016) tabel 4.6, PDF p.46 — f_type (winddrukverdeling);
//! - ISSO 53 (2016) tabel 4.8, PDF p.47 — f_typ (gebouwtype/ligging).
//!
//! Let op: `f_type` (tabel 4.6) en `f_typ` (tabel 4.8) zijn **verschillende**
//! factoren met verwarrend gelijkende namen:
//! - `f_type` corrigeert voor de gebouwafhankelijke winddrukverdeling op
//!   basis van de geveltype-/huidgevelconfiguratie;
//! - `f_typ` corrigeert voor het gebouwtype en de ligging (positie binnen
//!   een rij/blok of meerlaags gebouw).
//!
//! Beide worden gebruikt in de infiltratieketen voor onbekende q_v10,kar
//! (formules 4.31 en 4.33).

use crate::model::enums::{GebouwTypePositie, GebouwTypeWinddruk};

/// Correctiefactor f_type voor de gebouwafhankelijke winddrukverdeling.
/// ISSO 53 tabel 4.6 (PDF p.46), dimensieloos. Gebruikt in formule 4.31.
///
/// Voetnoot tabel 4.6: het onderscheid in f_type naar geveltype geldt
/// uitsluitend indien de tussenruimten per etage luchttechnisch zijn
/// gescheiden; anders geldt de standaardwaarde 0,51 voor alle geveltypen.
pub fn f_type(gebouw: GebouwTypeWinddruk) -> f64 {
    match gebouw {
        GebouwTypeWinddruk::EenlaagsMetKap => 1.0,
        GebouwTypeWinddruk::EenlaagsMetPlatDak => 0.77,
        GebouwTypeWinddruk::MeerlaagsStandaard => 0.51,
        GebouwTypeWinddruk::MeerlaagsVolgevelBinnengalerij => 0.48,
        GebouwTypeWinddruk::MeerlaagsDubbeleHuidOnderbroken => 0.46,
        GebouwTypeWinddruk::MeerlaagsDubbeleHuidDoorlopend => 0.15,
    }
}

/// Invloedfactor f_typ voor gebouwtype/ligging.
/// ISSO 53 tabel 4.8 (PDF p.47), dimensieloos. Gebruikt in formule 4.33.
pub fn f_typ(positie: GebouwTypePositie) -> f64 {
    match positie {
        GebouwTypePositie::EnkellaagsTussen => 1.0,
        GebouwTypePositie::EnkellaagsKop => 1.2,
        GebouwTypePositie::EnkellaagsVrijstaand => 1.4,
        GebouwTypePositie::MeerlaagsGeheel => 1.2,
        GebouwTypePositie::MeerlaagsTop => 1.3,
        GebouwTypePositie::MeerlaagsTussen => 1.2,
        GebouwTypePositie::MeerlaagsOnder => 1.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f_type_values() {
        assert_eq!(f_type(GebouwTypeWinddruk::EenlaagsMetKap), 1.0);
        assert_eq!(f_type(GebouwTypeWinddruk::EenlaagsMetPlatDak), 0.77);
        assert_eq!(f_type(GebouwTypeWinddruk::MeerlaagsStandaard), 0.51);
        assert_eq!(f_type(GebouwTypeWinddruk::MeerlaagsVolgevelBinnengalerij), 0.48);
        assert_eq!(f_type(GebouwTypeWinddruk::MeerlaagsDubbeleHuidOnderbroken), 0.46);
        assert_eq!(f_type(GebouwTypeWinddruk::MeerlaagsDubbeleHuidDoorlopend), 0.15);
    }

    #[test]
    fn test_f_typ_enkellaags() {
        assert_eq!(f_typ(GebouwTypePositie::EnkellaagsTussen), 1.0);
        assert_eq!(f_typ(GebouwTypePositie::EnkellaagsKop), 1.2);
        assert_eq!(f_typ(GebouwTypePositie::EnkellaagsVrijstaand), 1.4);
    }

    #[test]
    fn test_f_typ_meerlaags() {
        assert_eq!(f_typ(GebouwTypePositie::MeerlaagsGeheel), 1.2);
        assert_eq!(f_typ(GebouwTypePositie::MeerlaagsTop), 1.3);
        assert_eq!(f_typ(GebouwTypePositie::MeerlaagsTussen), 1.2);
        assert_eq!(f_typ(GebouwTypePositie::MeerlaagsOnder), 1.1);
    }
}
