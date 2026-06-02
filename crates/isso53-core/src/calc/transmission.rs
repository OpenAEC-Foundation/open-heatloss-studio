//! Transmission heat loss calculation for ISSO 53.

use crate::error::Result;
use crate::model::{BoundaryType, ConstructionElement, DesignConditions, Room, Building};
use crate::tables::thermal_bridge::DELTA_U_TB_DEFAULT;
use crate::tables::adjacent_unheated::f_k;
use crate::tables::temperature::{design_indoor_temperature, resolve_theta_i};
use super::ground::calculate_h_t_ground;

/// Calculate transmission heat losses for a room.
/// ISSO 53 formules 4.2: Φ_T,i = (H_T,ie + H_T,ia + H_T,iae + H_T,iaBE + H_T,ig) × (θ_i - θ_e)
pub fn calculate_transmission(
    room: &Room,
    all_rooms: &[Room], // For adjacent room resolution
    building: &Building,
    climate: &DesignConditions,
) -> Result<TransmissionResult> {
    // Group elements by boundary type for calculations
    let mut exterior_elements = Vec::new();
    let mut unheated_elements = Vec::new();
    let mut adjacent_building_elements = Vec::new();
    let mut adjacent_room_elements: Vec<&ConstructionElement> = Vec::new();
    let mut ground_elements = Vec::new();

    for element in &room.constructions {
        match element.boundary_type {
            BoundaryType::Exterior => exterior_elements.push(element),
            BoundaryType::Unheated => unheated_elements.push(element),
            BoundaryType::AdjacentBuilding => adjacent_building_elements.push(element),
            BoundaryType::Ground => ground_elements.push(element),
            BoundaryType::AdjacentRoom => adjacent_room_elements.push(element),
            _ => {} // Water, etc. not in scope for this batch
        }
    }

    // Get design temperature θ_i for this room.
    // Vervangt de exterieur-sentinel (garage e.d.) door θ_e, zodat die
    // nooit als f64::MIN in de verlies-berekening lekt.
    let theta_i: f64 = resolve_theta_i(room, climate.theta_e);

    // Calculate individual H_T components
    let h_t_exterior = calculate_h_t_exterior(&exterior_elements)?;
    let h_t_unheated = calculate_h_t_unheated(&unheated_elements)?;
    let h_t_adjacent_buildings = calculate_h_t_adjacent_buildings(&adjacent_building_elements, theta_i, climate.theta_e)?;
    let h_t_adjacent_rooms = calculate_h_t_adjacent_rooms(&adjacent_room_elements, all_rooms, theta_i, climate.theta_e)?;
    let h_t_ground = calculate_h_t_ground(&ground_elements, theta_i, climate, building.heating_system)?;

    // Calculate total transmission heat loss: Φ_T,i = H_T,total × (θ_i - θ_e)
    let h_t_total = h_t_exterior + h_t_adjacent_rooms + h_t_unheated + h_t_adjacent_buildings + h_t_ground;
    let phi_t = h_t_total * (theta_i - climate.theta_e);

    Ok(TransmissionResult {
        phi_t,
        h_t_exterior,
        h_t_adjacent_rooms,
        h_t_unheated,
        h_t_adjacent_buildings,
        h_t_ground,
    })
}

/// Calculate H_T,ie to exterior air.
/// ISSO 53 formule 4.3, PDF p.38: H_T,ie = Σ(A_k × (U_k + ΔU_TB) × f_k)
/// where f_k = 1.0 for exterior boundaries.
pub fn calculate_h_t_exterior(elements: &[&ConstructionElement]) -> Result<f64> {
    let mut h_t_ie = 0.0;

    for element in elements {
        // Get thermal bridge correction ΔU_TB.
        // Voorkeursvolgorde: expliciete custom-waarde > forfaitaire default > 0.
        let delta_u_tb = element.custom_delta_u_tb
            .unwrap_or_else(|| {
                if element.use_forfaitaire_thermal_bridge {
                    DELTA_U_TB_DEFAULT
                } else {
                    0.0
                }
            });

        // f_k = 1.0 for exterior (no correction factor)
        let f_k = 1.0;

        h_t_ie += element.area * (element.u_value + delta_u_tb) * f_k;
    }

    Ok(h_t_ie)
}

/// Calculate H_T,iae to unheated spaces.
/// ISSO 53 formule 4.13, PDF p.40: H_T,iae = Σ(A_k × U_k × f_k)
/// where f_k comes from tabel 4.2 based on unheated space characteristics.
pub fn calculate_h_t_unheated(elements: &[&ConstructionElement]) -> Result<f64> {
    let mut h_t_iae = 0.0;

    for element in elements {
        // Get correction factor f_k from tabel 4.2 for unheated space
        let f_k_value = element.unheated_space
            .map(f_k)
            .or(element.temperature_factor)
            .ok_or_else(|| crate::error::Isso53Error::InvalidInput(format!(
                "element {} (boundary=Unheated): vereist unheated_space of temperature_factor",
                element.id
            )))?;

        h_t_iae += element.area * element.u_value * f_k_value;
    }

    Ok(h_t_iae)
}

/// Calculate H_T,ia to adjacent heated/cooled rooms (within same building).
/// Calculate H_T,ia to adjacent rooms.
/// ISSO 53 formule 4.18, PDF p.43: H_T,ia = Σ(A_k × U_k × f_ia,k)
/// waarbij f_ia,k = (θ_i - θ_adjacent) / (θ_i - θ_e)
///
/// Geen thermische brug correctie (interne elementen — ISSO 53 §4.4).
///
/// Temperature resolution:
/// 1. If element.adjacent_room_id exists → lookup in all_rooms, use Room.custom_temperature
/// 2. Fallback to element.adjacent_temperature
/// 3. Error if both missing
pub fn calculate_h_t_adjacent_rooms(
    elements: &[&ConstructionElement],
    all_rooms: &[crate::model::Room],
    theta_i: f64,
    theta_e: f64,
) -> Result<f64> {
    let mut h_t_ia = 0.0;
    for element in elements {
        let theta_adj = if let Some(adjacent_room_id) = &element.adjacent_room_id {
            // Lookup adjacent room by ID
            let adjacent_room = all_rooms
                .iter()
                .find(|r| &r.id == adjacent_room_id)
                .ok_or_else(|| crate::error::Isso53Error::InvalidInput(format!(
                    "Adjacent room '{}' not found for element '{}'",
                    adjacent_room_id, element.id
                )))?;

            // Use room's custom temperature or calculate default based on usage
            adjacent_room.custom_temperature
                .unwrap_or_else(|| design_indoor_temperature(
                    adjacent_room.gebruiks_functie,
                    adjacent_room.ruimte_type
                ))
        } else if let Some(temp) = element.adjacent_temperature {
            // Fallback to explicit temperature on element
            temp
        } else {
            return Err(crate::error::Isso53Error::InvalidInput(format!(
                "Element '{}' (boundary=AdjacentRoom): vereist adjacent_room_id of adjacent_temperature",
                element.id
            )));
        };

        let f_ia_k = if (theta_i - theta_e).abs() < 0.001 {
            0.0
        } else {
            (theta_i - theta_adj) / (theta_i - theta_e)
        };
        h_t_ia += element.area * element.u_value * f_ia_k;
    }
    Ok(h_t_ia)
}

/// Calculate H_T,iaBE to adjacent buildings.
/// ISSO 53 formule 4.17, PDF p.42: H_T,iaBE = Σ(A_k × U_k × f_ia,k)
/// where f_ia,k is calculated from formules 4.18-4.20 based on neighbor temperature.
pub fn calculate_h_t_adjacent_buildings(
    elements: &[&ConstructionElement],
    theta_i: f64,
    theta_e: f64
) -> Result<f64> {
    let mut h_t_iabe = 0.0;

    for element in elements {
        // Calculate f_ia,k using formule 4.18: f_ia,k = (θ_i - θ_b) / (θ_i - θ_e)
        // where θ_b is adjacent building temperature (ISSO 53 §4.5 default: 15°C)
        let theta_b = element.adjacent_temperature.unwrap_or(15.0);

        let f_ia_k = if (theta_i - theta_e).abs() < 0.001 {
            0.0 // Avoid division by zero
        } else {
            (theta_i - theta_b) / (theta_i - theta_e)
        };

        h_t_iabe += element.area * element.u_value * f_ia_k;
    }

    Ok(h_t_iabe)
}

/// Results from transmission calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct TransmissionResult {
    /// Total transmission heat loss Φ_T,i in W.
    pub phi_t: f64,
    /// Heat loss coefficient to exterior H_T,ie in W/K.
    pub h_t_exterior: f64,
    /// Heat loss coefficient to adjacent heated rooms H_T,ia in W/K.
    pub h_t_adjacent_rooms: f64,
    /// Heat loss coefficient to unheated spaces H_T,iae in W/K.
    pub h_t_unheated: f64,
    /// Heat loss coefficient to adjacent buildings H_T,iaBE in W/K.
    pub h_t_adjacent_buildings: f64,
    /// Heat loss coefficient to ground H_T,ig in W/K.
    pub h_t_ground: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ConstructionElement, MaterialType, VerticalPosition, BoundaryType};

    #[test]
    fn test_h_t_exterior_smoke() {
        // 1 element A=10 m², U=0.3, exterior, use_forfaitaire_thermal_bridge=true
        // Expected: H_T,ie = 10 × (0.3 + 0.1) × 1.0 = 4.0 W/K
        let element = ConstructionElement {
            id: "wall1".to_string(),
            description: "Test wall".to_string(),
            area: 10.0,
            u_value: 0.3,
            boundary_type: BoundaryType::Exterior,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        };

        let result = calculate_h_t_exterior(&[&element]);
        assert!(result.is_ok());
        let h_t_ie = result.unwrap();
        assert!((h_t_ie - 4.0).abs() < 0.001, "Expected 4.0, got {}", h_t_ie);
    }

    #[test]
    fn test_h_t_adjacent_rooms_smoke() {
        // 1 element A=20 m², U=0.5, adjacent_temperature=18°C, θ_i=21°C, θ_e=-10°C
        // f_ia,k = (21 - 18) / (21 - (-10)) = 3 / 31 ≈ 0.0968
        // Expected: H_T,ia = 20 × 0.5 × 0.0968 ≈ 0.968 W/K
        let element = ConstructionElement {
            id: "wall_adj".to_string(),
            description: "Adjacent room wall".to_string(),
            area: 20.0,
            u_value: 0.5,
            boundary_type: BoundaryType::AdjacentRoom,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: Some(18.0),
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        };

        // Empty rooms list - should fallback to adjacent_temperature
        let rooms: Vec<Room> = vec![];

        let theta_i = 21.0;
        let theta_e = -10.0;
        let result = calculate_h_t_adjacent_rooms(&[&element], &rooms, theta_i, theta_e);
        assert!(result.is_ok(), "Error: {:?}", result);
        let h_t_ia = result.unwrap();
        let expected = 20.0 * 0.5 * (3.0 / 31.0); // ≈ 0.968
        assert!((h_t_ia - expected).abs() < 0.001, "Expected {:.3}, got {:.3}", expected, h_t_ia);
    }

    #[test]
    fn test_h_t_adjacent_rooms_missing_temperature() {
        // Element zonder adjacent_temperature en room not found moet error geven
        let element = ConstructionElement {
            id: "wall_adj_bad".to_string(),
            description: "Adjacent room wall without temp".to_string(),
            area: 10.0,
            u_value: 0.3,
            boundary_type: BoundaryType::AdjacentRoom,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: Some("room2".to_string()),
            adjacent_temperature: None, // Missing!
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        };

        // Empty rooms list - should error because room2 not found and no adjacent_temperature
        let rooms: Vec<Room> = vec![];

        let result = calculate_h_t_adjacent_rooms(&[&element], &rooms, 21.0, -10.0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("room2"));
        assert!(err.to_string().contains("not found"));
    }
}