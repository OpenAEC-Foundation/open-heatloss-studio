//! Transmission heat loss calculations.
//! ISSO 51 §2.5.1 through §2.5.5.
//!
//! Calculates the specific heat loss coefficient H_T for each boundary type:
//! - H_T,ie: to exterior (outside air)
//! - H_T,ia: to adjacent rooms within the dwelling
//! - H_T,io: to unheated spaces
//! - H_T,ib: to neighboring dwellings/buildings
//! - H_T,ig: to the ground
//! - H_T,iw: to open water (non-norm category — woonboot use case, §11 of
//!   the adjacent room temperature spec)

use crate::model::construction::ConstructionElement;
use crate::model::enums::{BoundaryType, VerticalPosition};
use crate::model::room::Room;
use crate::tables::thermal_bridge;

/// Calculate the specific heat loss H_T,ie to exterior for a single element.
/// [`ISSO_51_2023_FORMULE4_3A`](crate::formulas::ISSO_51_2023_FORMULE4_3A):
/// H_T,ie = Σ(A_k × f_k × (U_k + ΔU_TB))
///
/// # Arguments
/// * `element` - The construction element facing the exterior
///
/// # Returns
/// Contribution to H_T,ie in W/K for this element.
pub fn h_t_exterior_element(element: &ConstructionElement) -> f64 {
    let f_k = element.temperature_factor.unwrap_or(1.0);
    let delta_u = thermal_bridge::delta_u_tb(
        element.use_forfaitaire_thermal_bridge,
        element.custom_delta_u_tb,
    );
    element.area * f_k * (element.u_value + delta_u)
}

/// Calculate the specific heat loss H_T,ia to an adjacent room.
/// [`ISSO_51_2023_FORMULE4_6`](crate::formulas::ISSO_51_2023_FORMULE4_6):
/// H_T,ia = Σ(A_k × U_k × f_ia,k)
///
/// The temperature factor f_ia,k depends on element position.
/// For horizontal constructions between heated rooms, temperature stratification
/// applies on BOTH sides of the tussenvloer:
/// - Wall: f_ia = (θ_i - θ_a) / (θ_i - θ_e)
/// - Ceiling: f_ia = ((θ_i + Δθ₁) - (θ_a + Δθ₂)) / (θ_i - θ_e)
/// - Floor: f_ia = ((θ_i + Δθ₂) - (θ_a + Δθ₁)) / (θ_i - θ_e)
///
/// The ceiling surface (this room) is at θ_i + Δθ₁ (warm air rises),
/// while the floor surface (adjacent room above) is at θ_a + Δθ₂ (floor cooler).
/// Vice versa for floor elements.
///
/// Note: assumes the adjacent heated room has the same heating system (same Δθ values).
///
/// # Arguments
/// * `element` - The construction element facing the adjacent room
/// * `theta_i` - Design indoor temperature of this room in °C
/// * `theta_a` - Design temperature of the adjacent room in °C
/// * `theta_e` - Design outdoor temperature in °C
/// * `delta_1` - Δθ₁ from Table 2.12 (ceiling correction)
/// * `delta_2` - Δθ₂ from Table 2.12 (floor correction)
///
/// # Returns
/// Contribution to H_T,ia in W/K for this element.
pub fn h_t_adjacent_room_element(
    element: &ConstructionElement,
    theta_i: f64,
    theta_a: f64,
    theta_e: f64,
    delta_1: f64,
    delta_2: f64,
) -> f64 {
    let f_ia = if let Some(f) = element.temperature_factor {
        f
    } else {
        match element.vertical_position {
            VerticalPosition::Wall => (theta_i - theta_a) / (theta_i - theta_e),
            VerticalPosition::Ceiling => {
                ((theta_i + delta_1) - (theta_a + delta_2)) / (theta_i - theta_e)
            }
            VerticalPosition::Floor => {
                ((theta_i + delta_2) - (theta_a + delta_1)) / (theta_i - theta_e)
            }
        }
    };
    element.area * element.u_value * f_ia
}

/// Calculate the specific heat loss H_T,io to unheated spaces.
/// [`ISSO_51_2023_FORMULE4_10`](crate::formulas::ISSO_51_2023_FORMULE4_10):
/// H_T,io = Σ(A_k × U_k × f_k)
///
/// # Arguments
/// * `element` - The construction element facing the unheated space
///
/// # Returns
/// Contribution to H_T,io in W/K for this element.
pub fn h_t_unheated_element(element: &ConstructionElement) -> f64 {
    let f_k = element.temperature_factor.unwrap_or(0.5);
    element.area * element.u_value * f_k
}

/// Calculate the specific heat loss H_T,ib to neighboring buildings.
/// [`ISSO_51_2023_FORMULE4_14`](crate::formulas::ISSO_51_2023_FORMULE4_14):
/// H_T,ib = c_z × Σ(A_k × U_k × f_b)
///
/// The temperature factor f_b depends on the element position:
/// - Wall: [`ISSO_51_2023_FORMULE4_15`](crate::formulas::ISSO_51_2023_FORMULE4_15):
///   f_b = (θ_i - θ_b) / (θ_i - θ_e)
/// - Ceiling: [`ISSO_51_2023_FORMULE4_17`](crate::formulas::ISSO_51_2023_FORMULE4_17):
///   f_b = (θ_i + Δθ_1 - θ_b) / (θ_i - θ_e)
/// - Floor: [`ISSO_51_2023_FORMULE4_16`](crate::formulas::ISSO_51_2023_FORMULE4_16):
///   f_b = (θ_i + Δθ_2 - θ_b) / (θ_i - θ_e)
///
/// Note: c_z is applied at the room level, not per element.
///
/// # Arguments
/// * `element` - The construction element facing the neighboring building
/// * `theta_i` - Design indoor temperature of this room in °C
/// * `theta_b` - Temperature of neighboring building in °C
/// * `theta_e` - Design outdoor temperature in °C
/// * `delta_1` - Δθ₁ from Table 2.12 (ceiling correction)
/// * `delta_2` - Δθ₂ from Table 2.12 (floor correction)
///
/// # Returns
/// Contribution to H_T,ib in W/K for this element (before c_z multiplication).
pub fn h_t_adjacent_building_element(
    element: &ConstructionElement,
    theta_i: f64,
    theta_b: f64,
    theta_e: f64,
    delta_1: f64,
    delta_2: f64,
) -> f64 {
    let f_b = if let Some(f) = element.temperature_factor {
        f
    } else {
        match element.vertical_position {
            VerticalPosition::Wall => (theta_i - theta_b) / (theta_i - theta_e),
            VerticalPosition::Ceiling => (theta_i + delta_1 - theta_b) / (theta_i - theta_e),
            VerticalPosition::Floor => (theta_i + delta_2 - theta_b) / (theta_i - theta_e),
        }
    };
    element.area * element.u_value * f_b
}

/// Calculate the specific heat loss H_T,ig to the ground.
/// [`ISSO_51_2023_FORMULE4_18`](crate::formulas::ISSO_51_2023_FORMULE4_18):
/// H_T,ig = 1.45 × G_w × Σ(A_k × f_g2 × U_e,k)
///
/// # Arguments
/// * `element` - The construction element in contact with the ground
///
/// # Returns
/// Contribution to H_T,ig in W/K for this element.
pub fn h_t_ground_element(element: &ConstructionElement) -> f64 {
    if let Some(ref gp) = element.ground_params {
        1.45 * gp.ground_water_factor * element.area * gp.fg2 * gp.u_equivalent
    } else {
        0.0
    }
}

/// Calculate the specific heat loss H_T,iw to open water (non-norm category).
///
/// This is **not** an ISSO 51 / NEN-EN 12831 formula. It models a
/// construction that sits directly against open water (canals, rivers,
/// lakes — the woonboot use case). Water has an effectively unlimited
/// thermal mass that is continuously refreshed, so the water-side surface
/// temperature is clamped to `theta_water` from `DesignConditions` (default
/// 5 °C). The full ΔT between the room and the water counts — no b-factor
/// damping like ground.
///
/// The equivalent temperature factor written into `H_T`-space is
/// f_w = (θ_i - θ_water) / (θ_i - θ_e), so that multiplying the resulting
/// H_T,iw by (θ_i - θ_e) in `phi_transmission` recovers the physical
/// heat flow A·U·(θ_i - θ_water).
///
/// # Arguments
/// * `element` - The construction element facing open water
/// * `theta_i` - Design indoor temperature of this room in °C
/// * `theta_water` - Design temperature of the water in °C (from `DesignConditions`)
/// * `theta_e` - Design outdoor temperature in °C (normalisation reference)
///
/// # Returns
/// Contribution to H_T,iw in W/K for this element.
pub fn h_t_water_element(
    element: &ConstructionElement,
    theta_i: f64,
    theta_water: f64,
    theta_e: f64,
) -> f64 {
    let denom = theta_i - theta_e;
    if denom.abs() < 1e-9 {
        return 0.0;
    }
    let f_w = (theta_i - theta_water) / denom;
    element.area * element.u_value * f_w
}

/// Resolve the design temperature of the adjacent space for an
/// `AdjacentRoom` element, in priority order:
///
/// 1. Live lookup of `adjacent_room_id` in the full project room list.
/// 2. Legacy `adjacent_temperature` field on the element (for backward
///    compat with older saved projects).
/// 3. The current room's own `theta_i` (ΔT = 0 — the safe fallback).
///
/// This eliminates the silent 0-W bug where walls between two heated
/// rooms with different setpoints produced zero heat loss because
/// `adjacent_temperature` was never populated by the importer.
///
/// Emits a warning to stderr when an `adjacent_room_id` is set but the
/// room cannot be found in the project (orphan reference — usually a
/// project corruption or a room deletion that didn't clean up references).
/// The warning is never silently swallowed; it leaves a trail in the
/// CLI/server logs without taking the calculation hostage.
fn resolve_adjacent_temperature(
    element: &ConstructionElement,
    theta_i: f64,
    rooms: &[Room],
) -> f64 {
    if let Some(ref id) = element.adjacent_room_id {
        if let Some(room) = rooms.iter().find(|r| &r.id == id) {
            return room.design_temperature();
        }
        // Orphan reference — log to stderr (the only logging vehicle the
        // pure-Rust core has) and fall through to the legacy field /
        // theta_i fallback.
        eprintln!(
            "warmteverlies: adjacent_room_id '{}' op constructie '{}' verwijst naar een niet-bestaande room — fallback op adjacent_temperature/theta_i",
            id, element.id
        );
    }
    element.adjacent_temperature.unwrap_or(theta_i)
}

/// Aggregated per-boundary-type transmission coefficients for a single room.
///
/// Returned by [`calculate_all_h_t`].
#[derive(Debug, Clone, Copy, Default)]
pub struct HTransmission {
    /// H_T,ie — to exterior air (W/K).
    pub h_t_ie: f64,
    /// H_T,ia — to adjacent heated rooms (W/K).
    pub h_t_ia: f64,
    /// H_T,io — to unheated spaces (W/K).
    pub h_t_io: f64,
    /// H_T,ib — to adjacent buildings (W/K), after c_z multiplication.
    pub h_t_ib: f64,
    /// H_T,ig — to the ground (W/K).
    pub h_t_ig: f64,
    /// H_T,iw — to open water (W/K).
    pub h_t_iw: f64,
}

/// Calculate all specific heat loss coefficients for a set of construction elements.
///
/// # Arguments
/// * `elements` - All construction elements of a room
/// * `rooms` - The full project room list, for `AdjacentRoom` temperature lookup
/// * `theta_i` - Design indoor temperature of this room in °C
/// * `theta_e` - Design outdoor temperature in °C
/// * `theta_b` - Temperature of neighboring buildings in °C
/// * `theta_water` - Design temperature of open water in °C
/// * `c_z` - Security factor for neighbor heat loss
/// * `delta_1` - Δθ₁ from Table 2.12
/// * `delta_2` - Δθ₂ from Table 2.12
//
// The argument list is long because every heat-loss branch needs its own
// reference temperature; bundling them into a struct would obscure the
// call sites in `room_load.rs` without reducing the wiring. Silencing the
// lint locally keeps the public helper ergonomic.
#[allow(clippy::too_many_arguments)]
pub fn calculate_all_h_t(
    elements: &[ConstructionElement],
    rooms: &[Room],
    theta_i: f64,
    theta_e: f64,
    theta_b: f64,
    theta_water: f64,
    c_z: f64,
    delta_1: f64,
    delta_2: f64,
) -> HTransmission {
    let mut h_t_ie = 0.0;
    let mut h_t_ia = 0.0;
    let mut h_t_io = 0.0;
    let mut h_t_ib_sum = 0.0;
    let mut h_t_ig = 0.0;
    let mut h_t_iw = 0.0;

    for element in elements {
        match element.boundary_type {
            BoundaryType::Exterior => {
                h_t_ie += h_t_exterior_element(element);
            }
            BoundaryType::AdjacentRoom => {
                let theta_a = resolve_adjacent_temperature(element, theta_i, rooms);
                h_t_ia += h_t_adjacent_room_element(
                    element, theta_i, theta_a, theta_e, delta_1, delta_2,
                );
            }
            BoundaryType::UnheatedSpace => {
                h_t_io += h_t_unheated_element(element);
            }
            BoundaryType::AdjacentBuilding => {
                h_t_ib_sum += h_t_adjacent_building_element(
                    element, theta_i, theta_b, theta_e, delta_1, delta_2,
                );
            }
            BoundaryType::Ground => {
                h_t_ig += h_t_ground_element(element);
            }
            BoundaryType::Water => {
                h_t_iw += h_t_water_element(element, theta_i, theta_water, theta_e);
            }
        }
    }

    HTransmission {
        h_t_ie,
        h_t_ia,
        h_t_io,
        h_t_ib: c_z * h_t_ib_sum,
        h_t_ig,
        h_t_iw,
    }
}

/// Calculate total transmission heat loss Φ_T for a room.
/// [`ISSO_51_2023_FORMULE4_2`](crate::formulas::ISSO_51_2023_FORMULE4_2):
/// Φ_T = (H_T,ie + H_T,ia + H_T,io + H_T,ib + H_T,ig) × (θ_i - θ_e)
pub fn phi_transmission(h_t_total: f64, theta_i: f64, theta_e: f64) -> f64 {
    h_t_total * (theta_i - theta_e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::construction::ConstructionElement;
    use crate::model::enums::{BoundaryType, MaterialType, VerticalPosition};

    fn make_exterior_element(area: f64, u_value: f64, material: MaterialType) -> ConstructionElement {
        ConstructionElement {
            id: "test".to_string(),
            description: "test".to_string(),
            area,
            u_value,
            boundary_type: BoundaryType::Exterior,
            material_type: material,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        }
    }

    #[test]
    fn test_isso51_example_room1_h_t_ie() {
        // ISSO 51 Example 1, Room 1 (woonkamer):
        // buitenwand: A=7.29, U=0.36, f=1 → 7.29 × 1 × (0.36+0.1) = 3.35
        // raam: A=4.32, U=3.2, f=1 → 4.32 × 1 × (3.2+0.1) = 14.26
        // buitenwand bij deur: A=0.36, U=0.36, f=1 → 0.36 × 1 × (0.36+0.1) = 0.17
        // deur naar balkon: A=2.16, U=2.78, f=1 → 2.16 × 1 × (2.78+0.1) = 6.22
        // Total H_T,ie = 24.00

        let elements = vec![
            make_exterior_element(7.29, 0.36, MaterialType::Masonry),
            make_exterior_element(4.32, 3.2, MaterialType::NonMasonry),
            make_exterior_element(0.36, 0.36, MaterialType::Masonry),
            make_exterior_element(2.16, 2.78, MaterialType::NonMasonry),
        ];

        let h_t_ie: f64 = elements.iter().map(|e| h_t_exterior_element(e)).sum();

        // The example gives 24.00 (rounded)
        assert!((h_t_ie - 24.00).abs() < 0.1, "H_T,ie = {h_t_ie}, expected ~24.00");
    }

    #[test]
    fn test_isso51_example_room1_h_t_ia() {
        // ISSO 51 Example 1, Room 1 (woonkamer, θ_i=20, θ_e=-10):
        // naar keuken (θ_a=20): A=7.36, U=2.17, f=0 → 0
        // naar slaapkamer1 (θ_a=20): A=11.20, U=2.17, f=0 → 0
        // naar entree (θ_a=15): A=2.51, U=2.17, f=(20-15)/(20--10)=0.1667 → 0.91
        // naar toilet (θ_a=15): A=3.12, U=2.17, f=0.1667 → 1.13
        // naar badkamer (θ_a=22): A=3.64, U=2.17, f=(20-22)/(20--10)=-0.0667 → -0.53
        // Total H_T,ia = 1.51

        let theta_i = 20.0;
        let theta_e = -10.0;

        let tests = vec![
            (7.36, 2.17, 20.0, 0.0),     // keuken
            (11.20, 2.17, 20.0, 0.0),     // slaapkamer1
            (2.51, 2.17, 15.0, 0.91),     // entree
            (3.12, 2.17, 15.0, 1.13),     // toilet
            (3.64, 2.17, 22.0, -0.53),    // badkamer
        ];

        let mut total = 0.0;
        for (area, u, theta_a, expected) in &tests {
            let element = ConstructionElement {
                id: "test".to_string(),
                description: "test".to_string(),
                area: *area,
                u_value: *u,
                boundary_type: BoundaryType::AdjacentRoom,
                material_type: MaterialType::Masonry,
                temperature_factor: None,
                adjacent_room_id: None,
                adjacent_temperature: Some(*theta_a),
                vertical_position: VerticalPosition::Wall,
                use_forfaitaire_thermal_bridge: false,
                custom_delta_u_tb: None,
                ground_params: None,
                has_embedded_heating: false,
                catalog_ref: None,
            };
            let h = h_t_adjacent_room_element(&element, theta_i, *theta_a, theta_e, 2.0, -1.0);
            assert!(
                (h - expected).abs() < 0.02,
                "Element with A={area}, θ_a={theta_a}: got {h}, expected {expected}"
            );
            total += h;
        }

        assert!((total - 1.51).abs() < 0.1, "H_T,ia = {total}, expected ~1.51");
    }

    #[test]
    fn test_isso51_example_room1_h_t_ib() {
        // ISSO 51 Example 1, Room 1 (woonkamer):
        // θ_i=20, θ_b=15 (assumed from f_b values), θ_e=-10
        // Δθ_1=2.0 (LT-radiator), Δθ_2=-1.0
        //
        // woningscheidende wand: A=18.09, U=2.08, f_b=(20-15)/30=0.1667 → 6.27
        // plafond: A=28.20, U=2.5, f_b=(20+2-15)/30=0.2333 → 16.45
        // vloer: A=28.20, U=2.5, f_b=(20-1-15)/30=0.1333 → 9.40
        // Sum = 32.12, c_z=0.5 → H_T,ib = 16.06

        let theta_i = 20.0;
        let theta_b = 15.0;
        let theta_e = -10.0;
        let delta_1 = 2.0; // LT radiator
        let delta_2 = -1.0;

        let elements = vec![
            (18.09, 2.08, VerticalPosition::Wall),
            (28.20, 2.5, VerticalPosition::Ceiling),
            (28.20, 2.5, VerticalPosition::Floor),
        ];

        let mut sum = 0.0;
        for (area, u, pos) in &elements {
            let element = ConstructionElement {
                id: "test".to_string(),
                description: "test".to_string(),
                area: *area,
                u_value: *u,
                boundary_type: BoundaryType::AdjacentBuilding,
                material_type: MaterialType::Masonry,
                temperature_factor: None,
                adjacent_room_id: None,
                adjacent_temperature: None,
                vertical_position: *pos,
                use_forfaitaire_thermal_bridge: false,
                custom_delta_u_tb: None,
                ground_params: None,
                has_embedded_heating: false,
                catalog_ref: None,
            };
            sum += h_t_adjacent_building_element(&element, theta_i, theta_b, theta_e, delta_1, delta_2);
        }

        let h_t_ib = 0.5 * sum;
        assert!(
            (h_t_ib - 16.06).abs() < 0.1,
            "H_T,ib = {h_t_ib}, expected ~16.06"
        );
    }

    #[test]
    fn test_isso51_example_room1_total_transmission() {
        // Room 1 total: Φ_T = (24.00 + 1.51 + 0 + 16.06 + 0) × (20 - -10) = 1247 W
        let h_t_total = 24.00 + 1.51 + 0.0 + 16.06 + 0.0;
        let phi_t = phi_transmission(h_t_total, 20.0, -10.0);
        assert!(
            (phi_t - 1247.0).abs() < 5.0,
            "Φ_T = {phi_t}, expected ~1247"
        );
    }

    // ----- 2026-04-10 tests: adjacent-room live lookup + water boundary -----
    //
    // Regression & spec coverage for the two restbugs of §4.2 of
    // `warmteverlies_adjacent_room_temp_spec.md`.

    use crate::model::enums::{HeatingSystem, RoomFunction};
    use crate::model::room::Room;

    /// Build a minimal room whose only purpose is to participate in the
    /// `design_temperature()` lookup from the transmission calculator.
    fn make_lookup_room(
        id: &str,
        function: RoomFunction,
        custom_temperature: Option<f64>,
    ) -> Room {
        Room {
            id: id.to_string(),
            name: id.to_string(),
            function,
            custom_temperature,
            floor_area: 10.0,
            height: 2.6,
            constructions: vec![],
            heating_system: HeatingSystem::RadiatorLt,
            ventilation_rate: Some(0.0),
            has_mechanical_exhaust: false,
            has_mechanical_supply: false,
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
            internal_air_temperature: None,
            clamp_positive: true,
        }
    }

    /// Build a plain adjacent-room wall element with the given area and U,
    /// and an optional `adjacent_room_id`.
    fn make_adjacent_wall(
        id: &str,
        area: f64,
        u: f64,
        adjacent_room_id: Option<&str>,
    ) -> ConstructionElement {
        ConstructionElement {
            id: id.to_string(),
            description: id.to_string(),
            area,
            u_value: u,
            boundary_type: BoundaryType::AdjacentRoom,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: adjacent_room_id.map(String::from),
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        }
    }

    /// Spec test §4.4 #1:
    /// Twee rooms (20 °C en 18 °C), een wand ertussen, verify that
    /// H_T,ia > 0 and exactly equals A × U × (20-18)/(20-(-10)).
    #[test]
    fn test_adjacent_room_temperature_lookup() {
        // room-a is the calculating room at 20 °C; room-b is the adjacent
        // room at 18 °C (custom_temperature override).
        let room_b = make_lookup_room(
            "room-b",
            RoomFunction::LivingRoom,
            Some(18.0),
        );
        let rooms = vec![
            make_lookup_room("room-a", RoomFunction::LivingRoom, Some(20.0)),
            room_b,
        ];

        let area = 10.0;
        let u = 1.5;
        let wall = make_adjacent_wall("w1", area, u, Some("room-b"));
        let elements = vec![wall];

        let h_t = calculate_all_h_t(
            &elements, &rooms, 20.0, -10.0, 17.0, 5.0, 1.0, 2.0, -1.0,
        );

        // Expected: A × U × (20-18)/(20-(-10)) = 10 × 1.5 × 2/30 = 1.0 W/K
        let expected = area * u * (20.0 - 18.0) / (20.0 - (-10.0));
        assert!(
            (h_t.h_t_ia - expected).abs() < 1e-9,
            "H_T,ia = {}, expected {}",
            h_t.h_t_ia,
            expected
        );
        assert!(h_t.h_t_ia > 0.0, "H_T,ia must be > 0");

        // Sanity: a room-to-same-setpoint wall must still be 0.
        let h_t_same = calculate_all_h_t(
            &[make_adjacent_wall("w2", area, u, Some("room-a"))],
            &rooms,
            20.0,
            -10.0,
            17.0,
            5.0,
            1.0,
            2.0,
            -1.0,
        );
        assert!(
            h_t_same.h_t_ia.abs() < 1e-9,
            "H_T,ia to same-temperature room must be 0, got {}",
            h_t_same.h_t_ia
        );
    }

    /// Spec test §4.4 #2:
    /// Buurroom heeft `custom_temperature = Some(15.0)` — the override
    /// must take priority over the room function default.
    #[test]
    fn test_adjacent_room_with_custom_temperature() {
        // room-b has function=LivingRoom (default 20 °C) but a custom override
        // of 15 °C — the override wins.
        let rooms = vec![
            make_lookup_room("room-a", RoomFunction::LivingRoom, Some(20.0)),
            make_lookup_room("room-b", RoomFunction::LivingRoom, Some(15.0)),
        ];

        let area = 8.0;
        let u = 2.0;
        let wall = make_adjacent_wall("w1", area, u, Some("room-b"));

        let h_t = calculate_all_h_t(
            &[wall],
            &rooms,
            20.0,
            -10.0,
            17.0,
            5.0,
            1.0,
            2.0,
            -1.0,
        );

        // Expected: A × U × (20-15)/(20-(-10)) = 8 × 2 × 5/30 ≈ 2.6667 W/K
        let expected = area * u * (20.0 - 15.0) / 30.0;
        assert!(
            (h_t.h_t_ia - expected).abs() < 1e-9,
            "H_T,ia = {}, expected {}",
            h_t.h_t_ia,
            expected
        );
    }

    /// Spec test §4.4 #3:
    /// Buurroom heeft alleen `function = Bathroom`, default 22 °C
    /// must be picked up via `Room::design_temperature()`.
    #[test]
    fn test_adjacent_room_with_function_default() {
        // room-b has Bathroom function (22 °C per ISSO 51 Table 2.11)
        // and NO custom_temperature — the function default must be used.
        let rooms = vec![
            make_lookup_room("room-a", RoomFunction::LivingRoom, None), // 20 °C
            make_lookup_room("room-b", RoomFunction::Bathroom, None),   // 22 °C
        ];

        let area = 5.0;
        let u = 1.8;
        let wall = make_adjacent_wall("w1", area, u, Some("room-b"));

        let h_t = calculate_all_h_t(
            &[wall],
            &rooms,
            20.0,
            -10.0,
            17.0,
            5.0,
            1.0,
            2.0,
            -1.0,
        );

        // Expected: A × U × (20-22)/(20-(-10)) = 5 × 1.8 × (-2)/30 = -0.6 W/K
        // (negative because the bathroom is warmer — heat flows into this room)
        let expected = area * u * (20.0 - 22.0) / 30.0;
        assert!(
            (h_t.h_t_ia - expected).abs() < 1e-9,
            "H_T,ia = {}, expected {}",
            h_t.h_t_ia,
            expected
        );
        assert!(
            h_t.h_t_ia < 0.0,
            "H_T,ia must be negative (bathroom is warmer)"
        );
    }

    /// Spec test §4.4 #4:
    /// `adjacent_room_id = Some("room-99")` that does not exist — the
    /// calculation must fall back gracefully to θ_i (ΔT = 0) without
    /// panicking, and a warning must be emitted to stderr.
    #[test]
    fn test_adjacent_room_orphan_id() {
        let rooms = vec![
            make_lookup_room("room-a", RoomFunction::LivingRoom, Some(20.0)),
            make_lookup_room("room-b", RoomFunction::LivingRoom, Some(18.0)),
        ];

        // orphan: points at a non-existing room
        let wall = make_adjacent_wall("w1", 10.0, 1.5, Some("room-99"));

        let h_t = calculate_all_h_t(
            &[wall],
            &rooms,
            20.0,
            -10.0,
            17.0,
            5.0,
            1.0,
            2.0,
            -1.0,
        );

        // Expected: fallback to theta_i → ΔT = 0 → H_T,ia = 0 (safe).
        assert!(
            h_t.h_t_ia.abs() < 1e-9,
            "Orphan adjacent_room_id must fall back to 0 W/K, got {}",
            h_t.h_t_ia
        );

        // Legacy fallback: if adjacent_temperature *is* set, the orphan
        // branch should still honour it (backward compat for old projects).
        let mut wall_with_legacy = make_adjacent_wall("w2", 10.0, 1.5, Some("room-99"));
        wall_with_legacy.adjacent_temperature = Some(18.0);

        let h_t_legacy = calculate_all_h_t(
            &[wall_with_legacy],
            &rooms,
            20.0,
            -10.0,
            17.0,
            5.0,
            1.0,
            2.0,
            -1.0,
        );
        let expected_legacy = 10.0 * 1.5 * (20.0 - 18.0) / 30.0;
        assert!(
            (h_t_legacy.h_t_ia - expected_legacy).abs() < 1e-9,
            "Orphan id with legacy adjacent_temperature must use the legacy value, got {}",
            h_t_legacy.h_t_ia
        );
    }

    /// Spec test §4.4 #6:
    /// Water boundary with `theta_water = 5.0` and θᵢ = 20 °C must
    /// produce ΔT = 15 K heat flow through the element.
    #[test]
    fn test_water_boundary_uses_theta_water() {
        let area = 6.0;
        let u = 0.8;
        let element = ConstructionElement {
            id: "water-wall".to_string(),
            description: "Beton onderwater".to_string(),
            area,
            u_value: u,
            boundary_type: BoundaryType::Water,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        };

        let theta_i = 20.0;
        let theta_e = -10.0;
        let theta_water = 5.0;

        let h_t_iw = h_t_water_element(&element, theta_i, theta_water, theta_e);

        // The H_T representation: A × U × (θ_i - θ_water) / (θ_i - θ_e)
        // = 6 × 0.8 × 15 / 30 = 2.4 W/K
        let expected_h = area * u * (theta_i - theta_water) / (theta_i - theta_e);
        assert!(
            (h_t_iw - expected_h).abs() < 1e-9,
            "H_T,iw = {}, expected {}",
            h_t_iw,
            expected_h
        );

        // And after multiplication by (θ_i - θ_e) in phi_transmission the
        // physical flow must match the direct A × U × ΔT formula.
        let phi = phi_transmission(h_t_iw, theta_i, theta_e);
        let expected_phi = area * u * (theta_i - theta_water); // = 6 × 0.8 × 15 = 72 W
        assert!(
            (phi - expected_phi).abs() < 1e-9,
            "Φ_T,iw = {}, expected {}",
            phi,
            expected_phi
        );
    }

    /// Spec test §4.4 #7:
    /// `theta_water = 8.0` (project-level override) must flow through
    /// to the water-boundary calculation.
    #[test]
    fn test_water_boundary_override() {
        let area = 10.0;
        let u = 1.0;
        let element = ConstructionElement {
            id: "water-wall".to_string(),
            description: "Beton onderwater".to_string(),
            area,
            u_value: u,
            boundary_type: BoundaryType::Water,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        };

        // Default path
        let h_default = h_t_water_element(&element, 20.0, 5.0, -10.0);
        let expected_default = area * u * (20.0 - 5.0) / 30.0; // 5.0 W/K
        assert!(
            (h_default - expected_default).abs() < 1e-9,
            "default theta_water=5.0 path got {}, expected {}",
            h_default,
            expected_default
        );

        // Override path: warmer water → smaller ΔT → smaller loss
        let h_override = h_t_water_element(&element, 20.0, 8.0, -10.0);
        let expected_override = area * u * (20.0 - 8.0) / 30.0; // 4.0 W/K
        assert!(
            (h_override - expected_override).abs() < 1e-9,
            "override theta_water=8.0 path got {}, expected {}",
            h_override,
            expected_override
        );

        // The override must actually reduce the loss (monotone sanity).
        assert!(
            h_override < h_default,
            "warmer water must reduce H_T,iw"
        );

        // Same check via calculate_all_h_t — verify theta_water is routed
        // through the full calc loop, not just the low-level helper.
        let rooms: Vec<Room> = vec![];
        let h_t = calculate_all_h_t(
            &[element],
            &rooms,
            20.0,
            -10.0,
            17.0,
            8.0, // theta_water override
            1.0,
            2.0,
            -1.0,
        );
        assert!(
            (h_t.h_t_iw - expected_override).abs() < 1e-9,
            "calculate_all_h_t did not propagate theta_water override: got {}, expected {}",
            h_t.h_t_iw,
            expected_override
        );
    }
}
