//! Bepaling van de R_c-kolomkeuze voor Δθ_v (ISSO 53 tabel 2.3, voetnoot 4).
//!
//! Δθ_v (ventilatielucht-gelaagdheid, form. 4.30/4.38/4.39) heeft twee kolommen:
//! R_c < 3,5 m²K/W → −1 K, R_c ≥ 3,5 m²K/W → −0,5 K. De grenswaarde wordt
//! bepaald uit het **oppervlakte-gewogen gemiddelde R_c van de uitwendige
//! scheidingsconstructies** van de ruimte (voetnoot 4).
//!
//! Het model draagt per element alleen een `u_value`, geen R_c. We leiden R_c
//! af uit de bouwfysische relatie `R_c ≈ 1/U − R_si − R_se` met de ISSO 53 /
//! NTA 8800-overgangsweerstanden. Voor een gemengde set wand-/dak-/vloer-
//! oriëntaties gebruiken we de verticale-wand-waarden (R_si=0,13; R_se=0,04) als
//! representatieve default — het R_c=3,5-criterium is een grove drempel en de
//! oriëntatie-spreiding in R_se (0,04 dak/wand vs 0,00 grond) verschuift de
//! drempel-uitkomst zelden.

use crate::model::{BoundaryType, ConstructionElement, Room};

/// Binnen-overgangsweerstand R_si voor opgaande/horizontale scheidings-
/// constructies (ISSO 53 / NTA 8800, m²K/W). Conservatieve wand-waarde.
const R_SI: f64 = 0.13;

/// Buiten-overgangsweerstand R_se voor scheidingsconstructies grenzend aan
/// buitenlucht (ISSO 53 / NTA 8800, m²K/W).
const R_SE: f64 = 0.04;

/// R_c-drempel uit ISSO 53 tabel 2.3 voetnoot 4 (m²K/W).
pub const RC_THRESHOLD: f64 = 3.5;

/// Is het oppervlakte-gewogen gemiddelde R_c van de uitwendige
/// scheidingsconstructies van de ruimte ≥ 3,5 m²K/W?
///
/// "Uitwendige scheidingsconstructies" = elementen die de ruimte van het
/// buitenklimaat of de grond scheiden: [`BoundaryType::Exterior`] en
/// [`BoundaryType::Ground`]. Inwendige scheidingen (aangrenzend vertrek/gebouw,
/// onverwarmd) tellen niet mee voor deze gelaagdheids-kolomkeuze.
///
/// Retourneert `false` (R_c < 3,5 → de "zwaardere" Δθ_v = −1 K) wanneer er geen
/// uitwendige scheidingsconstructies zijn — conservatief.
pub fn room_rc_high(room: &Room) -> bool {
    let (sum_area, sum_area_rc) = room
        .constructions
        .iter()
        .filter(|e| is_external_envelope(e))
        .fold((0.0_f64, 0.0_f64), |(a_sum, ar_sum), e| {
            let rc = element_rc(e);
            (a_sum + e.area, ar_sum + e.area * rc)
        });

    if sum_area <= 0.0 {
        return false; // geen uitwendige gevel → conservatief de −1 K-kolom
    }

    let rc_mean = sum_area_rc / sum_area;
    rc_mean >= RC_THRESHOLD
}

/// Telt dit element mee als uitwendige scheidingsconstructie?
fn is_external_envelope(element: &ConstructionElement) -> bool {
    matches!(
        element.boundary_type,
        BoundaryType::Exterior | BoundaryType::Ground
    )
}

/// Construction-R_c afgeleid uit de U-waarde: `R_c = 1/U − R_si − R_se`,
/// geclampt op ≥ 0. Bij U ≤ 0 (degeneraat) → R_c = 0.
fn element_rc(element: &ConstructionElement) -> f64 {
    if element.u_value <= 0.0 {
        return 0.0;
    }
    (1.0 / element.u_value - R_SI - R_SE).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Bezetting, ConstructionElement, GebruiksFunctie, MaterialType, RuimteType,
        VerticalPosition,
    };

    fn ext_element(area: f64, u: f64, boundary: BoundaryType) -> ConstructionElement {
        ConstructionElement {
            id: "e".into(),
            description: "e".into(),
            area,
            u_value: u,
            boundary_type: boundary,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        }
    }

    fn room_with(elems: Vec<ConstructionElement>) -> Room {
        Room {
            id: "r".into(),
            name: "r".into(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Verblijfsruimte,
            floor_area: 25.0,
            height: 3.0,
            custom_temperature: Some(20.0),
            constructions: elems,
            bezetting: Bezetting { personen: None, personen_per_m2_default: None },
            infiltration_reduction_z: 1.0,
            has_mechanical_supply: None,
            ventilation_q_v_established: None,
        }
    }

    #[test]
    fn well_insulated_envelope_is_rc_high() {
        // U=0,2 → R_c = 1/0,2 − 0,17 = 4,83 ≥ 3,5 → true.
        let room = room_with(vec![ext_element(20.0, 0.2, BoundaryType::Exterior)]);
        assert!(room_rc_high(&room));
    }

    #[test]
    fn poorly_insulated_envelope_is_rc_low() {
        // U=0,4 → R_c = 2,5 − 0,17 = 2,33 < 3,5 → false.
        let room = room_with(vec![ext_element(20.0, 0.4, BoundaryType::Exterior)]);
        assert!(!room_rc_high(&room));
    }

    #[test]
    fn area_weighted_mean_decides() {
        // Groot slecht-geïsoleerd vlak domineert → onder de drempel.
        let room = room_with(vec![
            ext_element(50.0, 0.5, BoundaryType::Exterior), // R_c ≈ 1,83
            ext_element(5.0, 0.15, BoundaryType::Exterior),  // R_c ≈ 6,5
        ]);
        // gewogen: (50·1,83 + 5·6,5)/55 ≈ 2,26 < 3,5 → false
        assert!(!room_rc_high(&room));
    }

    #[test]
    fn ground_counts_as_envelope() {
        let room = room_with(vec![ext_element(20.0, 0.15, BoundaryType::Ground)]);
        assert!(room_rc_high(&room)); // R_c ≈ 6,5 ≥ 3,5
    }

    #[test]
    fn internal_walls_ignored() {
        // Alleen een aangrenzend-vertrek-wand (intern) → geen uitwendige gevel.
        let room = room_with(vec![ext_element(20.0, 0.15, BoundaryType::AdjacentRoom)]);
        assert!(!room_rc_high(&room)); // geen envelope → conservatief false
    }
}
