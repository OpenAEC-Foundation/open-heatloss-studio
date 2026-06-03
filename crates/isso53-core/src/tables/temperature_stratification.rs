//! Temperatuur-gelaagdheid (Δθ₁, Δθ₂, Δθ_v) per verwarmingssysteem
//! volgens ISSO 53 tabel 2.3 (PDF p.21-22).
//!
//! - **Δθ₁ / Δθ_a1** — gelaagdheidscorrectie voor *horizontale* boven-elementen
//!   (vloer boven buitenlucht, plat dak, plafond) in de transmissie-formules
//!   4.5/4.6, 4.11/4.12, 4.15/4.16, 4.19/4.20.
//! - **Δθ₂ / Δθ_a2** — gelaagdheidscorrectie voor *vloer*-elementen (form. 4.23
//!   grond, 4.11/4.12, 4.19/4.20). Reeds in gebruik in `ground.rs`.
//! - **Δθ_v** — gelaagdheidscorrectie ventilatielucht (form. 4.30/4.38/4.39).
//!   Kolom afhankelijk van het oppervlakte-gewogen gemiddelde R_c van de
//!   uitwendige scheidingsconstructies (< 3,5 vs ≥ 3,5; voetnoot 4).
//!
//! **Vide-correctie (voetnoot 2):** bij ruimtehoogte h > 4 m worden Δθ₁ resp.
//! Δθ_a1 vermenigvuldigd met (h / 4). Zie [`vide_factor`].

use crate::model::enums::HeatingSystem;

/// Referentiehoogte voor de gelaagdheidscorrectie (ISSO 53 tabel 2.3 geldt tot
/// max. 4 m hoogte; daarboven de vide-correctie ×(h/4), voetnoot 2).
pub const STRATIFICATION_REFERENCE_HEIGHT_M: f64 = 4.0;

/// Temperatuur-correctie Δθ₁ (resp. Δθ_a1) volgens ISSO 53 tabel 2.3.
///
/// Wordt toegepast op horizontale boven-elementen (plafonds/daken/vloeren boven
/// buitenlucht) in de transmissie-formules 4.5/4.6, 4.11/4.12, 4.15/4.16,
/// 4.19/4.20. Vorm: `f_k = (θ_i + Δθ₁ − θ_a) / (θ_i − θ_e)`.
///
/// Eenheid: K.
pub fn delta_theta_1(system: HeatingSystem) -> f64 {
    match system {
        HeatingSystem::LokaleVerwarming => 4.0,
        HeatingSystem::RadiatorenConvHtEnLuchtverwarming => 3.0,
        HeatingSystem::RadiatorenConvLt => 2.0,
        HeatingSystem::Plafondverwarming => 3.0,
        HeatingSystem::Wandverwarming => 2.0,
        HeatingSystem::Plintverwarming => 1.0,
        HeatingSystem::VloerverwarmingPlusHtRadi => 3.0,
        HeatingSystem::VloerverwarmingPlusLtRadi => 2.0,
        HeatingSystem::Vloerverwarming => 0.0,
        HeatingSystem::VloerverwarmingPlusWandverwarming => 1.0,
        HeatingSystem::Betonkernactivering => 0.0,
        HeatingSystem::VentilatorgedrevenConvRadi => 0.5,
    }
}

/// Temperatuur-correctie Δθ_v voor ventilatielucht volgens ISSO 53 tabel 2.3
/// (form. 4.30/4.38/4.39). Kolom afhankelijk van het oppervlakte-gewogen
/// gemiddelde R_c van de uitwendige scheidingsconstructies (voetnoot 4):
/// - `rc_high = false` → R_c < 3,5 m²K/W
/// - `rc_high = true`  → R_c ≥ 3,5 m²K/W
///
/// Eenheid: K. Δθ_v = 0 voor alle systemen met een toevoertemperatuur hoger
/// dan θ_i (bv. luchtverwarming) en voor onbekende systemen.
///
/// Deze functie levert de data; toepassing in `ventilation.rs`/`infiltration.rs`
/// volgt in een latere ronde (A7).
pub fn delta_theta_v(system: HeatingSystem, rc_high: bool) -> f64 {
    // Δθ_v ≠ 0 alleen voor de systemen met lage stralingstemperatuur in vloer/
    // wand/kern. R_c < 3,5 → −1 K; R_c ≥ 3,5 → −0,5 K.
    let (low_rc, high_rc) = match system {
        HeatingSystem::Wandverwarming
        | HeatingSystem::VloerverwarmingPlusLtRadi
        | HeatingSystem::Vloerverwarming
        | HeatingSystem::VloerverwarmingPlusWandverwarming
        | HeatingSystem::Betonkernactivering => (-1.0, -0.5),

        HeatingSystem::LokaleVerwarming
        | HeatingSystem::RadiatorenConvHtEnLuchtverwarming
        | HeatingSystem::RadiatorenConvLt
        | HeatingSystem::Plafondverwarming
        | HeatingSystem::Plintverwarming
        | HeatingSystem::VloerverwarmingPlusHtRadi
        | HeatingSystem::VentilatorgedrevenConvRadi => (0.0, 0.0),
    };

    if rc_high {
        high_rc
    } else {
        low_rc
    }
}

/// Vide-correctiefactor voor Δθ₁/Δθ_a1 volgens ISSO 53 tabel 2.3 voetnoot 2.
///
/// Tabel 2.3 geldt tot een hoogte van 4 m. Bij vides/atria/hallen met een
/// grotere hoogte wordt Δθ₁ (resp. Δθ_a1) vermenigvuldigd met (h / 4).
/// Voor h ≤ 4 m levert deze functie 1,0 (geen correctie).
///
/// `height_m` = totale vertrekhoogte h [m].
pub fn vide_factor(height_m: f64) -> f64 {
    if height_m > STRATIFICATION_REFERENCE_HEIGHT_M {
        height_m / STRATIFICATION_REFERENCE_HEIGHT_M
    } else {
        1.0
    }
}

/// Vide-gecorrigeerde Δθ₁ voor een gegeven systeem en vertrekhoogte.
/// Combineert [`delta_theta_1`] met [`vide_factor`] (voetnoot 2).
pub fn delta_theta_1_corrected(system: HeatingSystem, height_m: f64) -> f64 {
    delta_theta_1(system) * vide_factor(height_m)
}

/// Temperatuur-correctie voor temperatuurgelaagdheid volgens ISSO 53 tabel 2.3.
/// Gebruikt in formule 4.23 voor vloer-f_ig berekening.
///
/// Formule 4.23: f_ig,k = ((θ_i + Δθ_2) − θ_me) / (θ_i − θ_e)
pub fn delta_theta_2(system: HeatingSystem) -> f64 {
    match system {
        HeatingSystem::LokaleVerwarming => -1.0,
        HeatingSystem::RadiatorenConvHtEnLuchtverwarming => -1.0,
        HeatingSystem::RadiatorenConvLt => -1.0,
        HeatingSystem::Plafondverwarming => 0.0,
        HeatingSystem::Wandverwarming => -1.0,
        HeatingSystem::Plintverwarming => -1.0,
        HeatingSystem::VloerverwarmingPlusHtRadi => 0.0,
        HeatingSystem::VloerverwarmingPlusLtRadi => 0.0,
        HeatingSystem::Vloerverwarming => 0.0,
        HeatingSystem::VloerverwarmingPlusWandverwarming => 0.0,
        HeatingSystem::Betonkernactivering => 0.0,
        HeatingSystem::VentilatorgedrevenConvRadi => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Δθ₁ voor alle 12 systemen — volledige dekking tabel 2.3 kolom 2.
    #[test]
    fn test_delta_theta_1_all_systems() {
        assert_eq!(delta_theta_1(HeatingSystem::LokaleVerwarming), 4.0);
        assert_eq!(delta_theta_1(HeatingSystem::RadiatorenConvHtEnLuchtverwarming), 3.0);
        assert_eq!(delta_theta_1(HeatingSystem::RadiatorenConvLt), 2.0);
        assert_eq!(delta_theta_1(HeatingSystem::Plafondverwarming), 3.0);
        assert_eq!(delta_theta_1(HeatingSystem::Wandverwarming), 2.0);
        assert_eq!(delta_theta_1(HeatingSystem::Plintverwarming), 1.0);
        assert_eq!(delta_theta_1(HeatingSystem::VloerverwarmingPlusHtRadi), 3.0);
        assert_eq!(delta_theta_1(HeatingSystem::VloerverwarmingPlusLtRadi), 2.0);
        assert_eq!(delta_theta_1(HeatingSystem::Vloerverwarming), 0.0);
        assert_eq!(delta_theta_1(HeatingSystem::VloerverwarmingPlusWandverwarming), 1.0);
        assert_eq!(delta_theta_1(HeatingSystem::Betonkernactivering), 0.0);
        assert_eq!(delta_theta_1(HeatingSystem::VentilatorgedrevenConvRadi), 0.5);
        // Default systeem (radi-ht).
        assert_eq!(delta_theta_1(HeatingSystem::default()), 3.0);
    }

    /// Δθ_v voor alle 12 systemen × beide R_c-kolommen — volledige dekking.
    #[test]
    fn test_delta_theta_v_all_systems_both_columns() {
        // Systemen met Δθ_v ≠ 0 (lage stralingstemperatuur).
        for sys in [
            HeatingSystem::Wandverwarming,
            HeatingSystem::VloerverwarmingPlusLtRadi,
            HeatingSystem::Vloerverwarming,
            HeatingSystem::VloerverwarmingPlusWandverwarming,
            HeatingSystem::Betonkernactivering,
        ] {
            assert_eq!(delta_theta_v(sys, false), -1.0, "{sys:?} R_c<3,5");
            assert_eq!(delta_theta_v(sys, true), -0.5, "{sys:?} R_c≥3,5");
        }

        // Systemen met Δθ_v = 0 in beide kolommen.
        for sys in [
            HeatingSystem::LokaleVerwarming,
            HeatingSystem::RadiatorenConvHtEnLuchtverwarming,
            HeatingSystem::RadiatorenConvLt,
            HeatingSystem::Plafondverwarming,
            HeatingSystem::Plintverwarming,
            HeatingSystem::VloerverwarmingPlusHtRadi,
            HeatingSystem::VentilatorgedrevenConvRadi,
        ] {
            assert_eq!(delta_theta_v(sys, false), 0.0, "{sys:?} R_c<3,5");
            assert_eq!(delta_theta_v(sys, true), 0.0, "{sys:?} R_c≥3,5");
        }
    }

    /// Vide-correctie ×(h/4) (voetnoot 2).
    #[test]
    fn test_vide_factor() {
        // h ≤ 4 m → geen correctie.
        assert_eq!(vide_factor(2.6), 1.0);
        assert_eq!(vide_factor(4.0), 1.0);
        // h > 4 m → h/4.
        assert!((vide_factor(8.0) - 2.0).abs() < 1e-12);
        assert!((vide_factor(6.0) - 1.5).abs() < 1e-12);
    }

    /// Δθ₁ gecombineerd met vide-correctie.
    #[test]
    fn test_delta_theta_1_corrected() {
        // radi-ht Δθ₁=3, h=8 → 3 × 2 = 6.
        assert!((delta_theta_1_corrected(HeatingSystem::RadiatorenConvHtEnLuchtverwarming, 8.0) - 6.0).abs() < 1e-12);
        // h ≤ 4 → onveranderd.
        assert_eq!(
            delta_theta_1_corrected(HeatingSystem::RadiatorenConvHtEnLuchtverwarming, 3.0),
            3.0
        );
        // Δθ₁=0 blijft 0 ook bij grote hoogte.
        assert_eq!(delta_theta_1_corrected(HeatingSystem::Betonkernactivering, 12.0), 0.0);
    }

    #[test]
    fn test_all_heating_systems() {
        // Systemen met Δθ_2 = -1 K
        assert_eq!(delta_theta_2(HeatingSystem::LokaleVerwarming), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::RadiatorenConvHtEnLuchtverwarming), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::RadiatorenConvLt), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::Wandverwarming), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::Plintverwarming), -1.0);

        // Systemen met Δθ_2 = 0 K
        assert_eq!(delta_theta_2(HeatingSystem::Plafondverwarming), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VloerverwarmingPlusHtRadi), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VloerverwarmingPlusLtRadi), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::Vloerverwarming), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VloerverwarmingPlusWandverwarming), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::Betonkernactivering), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VentilatorgedrevenConvRadi), 0.0);

        // Default systeem
        assert_eq!(delta_theta_2(HeatingSystem::default()), -1.0);
    }
}
