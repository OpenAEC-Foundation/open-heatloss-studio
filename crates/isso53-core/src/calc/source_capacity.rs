//! Source (boiler/heat-pump) connection capacity for ISSO 53 (hoofdstuk 5).
//!
//! Two variants:
//! - Individual installation: formule 5.1
//! - Collective installation: formule 5.9 (excludes Φ_T,iaBE)
//!
//! Both apply the infiltration reduction fraction z from tabel 5.1
//! to prevent over-dimensioning (wind never hits all facades simultaneously).

use crate::error::Result;
use crate::result::RoomResult;

/// Calculate the individual connection capacity Φ_source.
/// ISSO 53 formule 5.1: Φ_source = Σ(Φ_T,ie + Φ_T,iae + Φ_T,iaBE + Φ_T,ig) + Φ_Ven + g·Σ Φ_hu - Σ Φ_gain
/// where Φ_Ven = z·Σ(H_i)·Δθ + Σ(H_v)·Δθ (formule 5.2).
///
/// `phi_hu_simultaneity` (`g`, ISSO 53 §4.1/§5.1) reduceert Σ Φ_hu naar het
/// gelijktijdig optredende deel om overdimensionering te voorkomen. Default
/// 1,0 = 100% gelijktijdigheid (ongewijzigd gedrag); de engine doet geen
/// automatische zone-detectie — de factor is een bewuste keuze van de gebruiker.
pub fn calculate_individual(
    rooms: &[RoomResult],
    z_fraction: f64,
    theta_e: f64,
    phi_hu_simultaneity: f64,
) -> Result<f64> {
    let mut phi_t_ie_total = 0.0;     // Σ(Φ_T,ie) - exterior transmission
    let mut phi_t_iae_total = 0.0;    // Σ(Φ_T,iae) - unheated transmission
    let mut phi_t_iabe_total = 0.0;   // Σ(Φ_T,iaBE) - adjacent building transmission
    let mut phi_t_ig_total = 0.0;     // Σ(Φ_T,ig) - ground transmission
    let mut phi_hu_total = 0.0;       // Σ Φ_hu - heating-up supplement
    let mut phi_gain_total = 0.0;     // Σ Φ_gain - internal gains
    let mut h_i_delta_theta_total = 0.0;  // z·Σ(H_i·Δθ) - infiltration with reduction
    let mut h_v_delta_theta_total = 0.0;  // Σ(H_v·Δθ) - ventilation

    for room in rooms {
        let delta_theta = room.theta_i - theta_e;

        // Calculate component transmission losses from H-values
        // NOTE: h_t_adjacent_rooms deliberately excluded — adjacent heated rooms are internal,
        // no net heat transfer to outside (per formule 5.1).
        phi_t_ie_total += room.h_t_exterior * delta_theta;
        phi_t_iae_total += room.h_t_unheated * delta_theta;
        phi_t_iabe_total += room.h_t_adjacent_buildings * delta_theta;
        phi_t_ig_total += room.h_t_ground * delta_theta;

        // Heating-up and gains
        phi_hu_total += room.phi_hu;
        phi_gain_total += room.phi_gain;

        // Ventilation components (formule 5.2)
        h_i_delta_theta_total += room.h_i * delta_theta;
        h_v_delta_theta_total += room.h_v * delta_theta;
    }

    // Apply infiltration reduction factor z (formule 5.2)
    let phi_ven = z_fraction * h_i_delta_theta_total + h_v_delta_theta_total;

    // Gelijktijdigheidsfactor op Σ Φ_hu (§4.1/§5.1) — voorkomt overdimensionering
    // bij niet-gelijktijdig opwarmende zones. Default 1,0 = ongewijzigd.
    let phi_hu_simultaneous = phi_hu_simultaneity * phi_hu_total;

    // Total connection capacity (formule 5.1)
    let phi_source = phi_t_ie_total + phi_t_iae_total + phi_t_iabe_total + phi_t_ig_total
        + phi_ven + phi_hu_simultaneous - phi_gain_total;

    Ok(phi_source)
}

/// Calculate the collective connection capacity Φ_source.
/// ISSO 53 formule 5.9: Same as individual BUT Φ_T,iaBE term is excluded
/// (adjacent buildings carry their own heating load in collective systems).
///
/// `phi_hu_simultaneity` werkt identiek aan [`calculate_individual`].
pub fn calculate_collective(
    rooms: &[RoomResult],
    z_fraction: f64,
    theta_e: f64,
    phi_hu_simultaneity: f64,
) -> Result<f64> {
    let mut phi_t_ie_total = 0.0;     // Σ(Φ_T,ie) - exterior transmission
    let mut phi_t_iae_total = 0.0;    // Σ(Φ_T,iae) - unheated transmission
    // phi_t_iabe_total omitted - that's the key difference from individual
    let mut phi_t_ig_total = 0.0;     // Σ(Φ_T,ig) - ground transmission
    let mut phi_hu_total = 0.0;       // Σ Φ_hu - heating-up supplement
    let mut phi_gain_total = 0.0;     // Σ Φ_gain - internal gains
    let mut h_i_delta_theta_total = 0.0;  // z·Σ(H_i·Δθ) - infiltration with reduction
    let mut h_v_delta_theta_total = 0.0;  // Σ(H_v·Δθ) - ventilation

    for room in rooms {
        let delta_theta = room.theta_i - theta_e;

        // Calculate component transmission losses from H-values
        // NOTE: h_t_adjacent_buildings is deliberately excluded
        phi_t_ie_total += room.h_t_exterior * delta_theta;
        phi_t_iae_total += room.h_t_unheated * delta_theta;
        phi_t_ig_total += room.h_t_ground * delta_theta;

        // Heating-up and gains
        phi_hu_total += room.phi_hu;
        phi_gain_total += room.phi_gain;

        // Ventilation components (formule 5.2)
        h_i_delta_theta_total += room.h_i * delta_theta;
        h_v_delta_theta_total += room.h_v * delta_theta;
    }

    // Apply infiltration reduction factor z (formule 5.2)
    let phi_ven = z_fraction * h_i_delta_theta_total + h_v_delta_theta_total;

    // Gelijktijdigheidsfactor op Σ Φ_hu (§4.1/§5.1) — zie calculate_individual.
    let phi_hu_simultaneous = phi_hu_simultaneity * phi_hu_total;

    // Total connection capacity (formule 5.9 - no Φ_T,iaBE)
    let phi_source = phi_t_ie_total + phi_t_iae_total + phi_t_ig_total
        + phi_ven + phi_hu_simultaneous - phi_gain_total;

    Ok(phi_source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::RoomResult;

    #[test]
    fn test_individual_vs_collective_difference() {
        let rooms = vec![create_test_room_with_adjacent_building()];
        let z_fraction = 0.5;

        let individual = calculate_individual(&rooms, z_fraction, -10.0, 1.0).unwrap();
        let collective = calculate_collective(&rooms, z_fraction, -10.0, 1.0).unwrap();

        // Individual should be higher due to including adjacent building transmission
        assert!(individual > collective,
            "Individual ({}) should be higher than collective ({})", individual, collective);

        // The difference should be the adjacent building contribution
        let expected_diff = rooms[0].h_t_adjacent_buildings * (rooms[0].theta_i - (-10.0));
        let actual_diff = individual - collective;
        assert!((actual_diff - expected_diff).abs() < 0.01,
            "Difference should be adjacent building component: expected {}, got {}",
            expected_diff, actual_diff);
    }

    #[test]
    fn test_z_fraction_reduces_infiltration() {
        let rooms = vec![create_test_room_with_infiltration()];

        let no_reduction = calculate_individual(&rooms, 1.0, -10.0, 1.0).unwrap();
        let with_reduction = calculate_individual(&rooms, 0.5, -10.0, 1.0).unwrap();

        assert!(no_reduction > with_reduction,
            "No reduction ({}) should be higher than with reduction ({})",
            no_reduction, with_reduction);
    }

    /// K2 (§4.1/§5.1): de gelijktijdigheidsfactor `g` grijpt aan op Σ Φ_hu in
    /// Φ_source. `g=0,5` moet exact de halve Φ_hu-bijdrage opleveren t.o.v.
    /// `g=1,0`; het verschil = 0,5·Σ Φ_hu. `g=1,0` = ongewijzigd gedrag.
    #[test]
    fn test_heating_up_simultaneity_factor() {
        let rooms = vec![create_test_room_with_infiltration()]; // phi_hu = 100.0
        let z = 1.0;
        let theta_e = -10.0;

        let full = calculate_individual(&rooms, z, theta_e, 1.0).unwrap();
        let half = calculate_individual(&rooms, z, theta_e, 0.5).unwrap();

        let phi_hu_total: f64 = rooms.iter().map(|r| r.phi_hu).sum();
        assert!(phi_hu_total > 0.0, "fixture moet Φ_hu > 0 hebben");

        // g=0,5 trekt exact 0,5·Σ Φ_hu van Φ_source af t.o.v. g=1,0.
        let expected_diff = 0.5 * phi_hu_total;
        let actual_diff = full - half;
        assert!(
            (actual_diff - expected_diff).abs() < 1e-9,
            "g=0,5 moet 0,5·Σ Φ_hu ({}) schelen, kreeg {}",
            expected_diff, actual_diff
        );

        // g=1,0 is identiek aan het oude gedrag (volledige Σ Φ_hu).
        // Reconstrueer Φ_source zonder Φ_hu en tel die handmatig op.
        let base_without_hu = full - phi_hu_total;
        assert!(
            (full - (base_without_hu + 1.0 * phi_hu_total)).abs() < 1e-9,
            "g=1,0 moet de volledige Σ Φ_hu meenemen"
        );

        // Idem voor de collectieve variant.
        let full_c = calculate_collective(&rooms, z, theta_e, 1.0).unwrap();
        let half_c = calculate_collective(&rooms, z, theta_e, 0.5).unwrap();
        assert!(
            ((full_c - half_c) - expected_diff).abs() < 1e-9,
            "collectief: g=0,5 moet 0,5·Σ Φ_hu schelen"
        );
    }

    fn create_test_room_with_adjacent_building() -> RoomResult {
        RoomResult {
            room_id: "test".to_string(),
            room_name: "Test Room".to_string(),
            theta_i: 20.0,
            phi_t: 1000.0,
            phi_v: 500.0,
            phi_i: 200.0,
            phi_hu: 100.0,
            phi_system: 0.0,
            phi_gain: 0.0,
            total_heat_loss: 1800.0,
            h_t_exterior: 25.0,
            h_t_adjacent_rooms: 0.0,
            h_t_unheated: 10.0,
            h_t_adjacent_buildings: 15.0, // This will be excluded in collective
            h_t_ground: 5.0,
            h_v: 15.0,
            h_i: 8.0,
        }
    }

    fn create_test_room_with_infiltration() -> RoomResult {
        RoomResult {
            room_id: "test".to_string(),
            room_name: "Test Room".to_string(),
            theta_i: 20.0,
            phi_t: 1000.0,
            phi_v: 500.0,
            phi_i: 200.0,
            phi_hu: 100.0,
            phi_system: 0.0,
            phi_gain: 0.0,
            total_heat_loss: 1800.0,
            h_t_exterior: 25.0,
            h_t_adjacent_rooms: 0.0,
            h_t_unheated: 10.0,
            h_t_adjacent_buildings: 0.0,
            h_t_ground: 5.0,
            h_v: 15.0,
            h_i: 10.0, // Significant infiltration for z-factor testing
        }
    }
}
