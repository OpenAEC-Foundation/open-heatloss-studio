//! Transmission heat loss calculation for ISSO 53.

use crate::error::Result;
use crate::model::{BoundaryType, ConstructionElement, DesignConditions, Room, Building};
use crate::model::enums::{HeatingSystem, VerticalPosition};
use crate::tables::thermal_bridge::DELTA_U_TB_DEFAULT;
use crate::tables::adjacent_unheated::f_k;
use crate::tables::temperature::{design_indoor_temperature, resolve_theta_i};
use crate::tables::temperature_stratification::delta_theta_1_corrected;
use super::ground::calculate_h_t_ground;

/// Of een element een *horizontaal boven-element* is waarop de
/// gelaagdheidscorrectie Δθ₁ (ISSO 53 tabel 2.3) van toepassing is.
///
/// Δθ₁ geldt voor vloeren-boven-buitenlucht, platte daken en plafonds
/// (form. 4.5/4.6, 4.11/4.12, 4.15/4.16, 4.19/4.20). Verticale wanden krijgen
/// **geen** Δθ₁ (form. 4.3/4.14).
fn is_horizontal_above(element: &ConstructionElement) -> bool {
    matches!(
        element.vertical_position,
        VerticalPosition::Floor | VerticalPosition::Ceiling
    )
}

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

    // Stratificatie-context (ISSO 53 tabel 2.3): Δθ₁ geldt op horizontale
    // boven-elementen, met vide-correctie ×(h/4) op basis van room.height.
    let heating_system = building.heating_system;
    let room_height = room.height;

    // Calculate individual H_T components
    let h_t_exterior =
        calculate_h_t_exterior(&exterior_elements, theta_i, climate.theta_e, heating_system, room_height)?;
    let h_t_unheated = calculate_h_t_unheated(&unheated_elements)?;
    let h_t_adjacent_buildings =
        calculate_h_t_adjacent_buildings(&adjacent_building_elements, theta_i, climate.theta_e)?;
    let h_t_adjacent_rooms =
        calculate_h_t_adjacent_rooms(&adjacent_room_elements, all_rooms, theta_i, climate.theta_e)?;
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
///
/// - **Verticale wanden** (form. 4.3): f_k = 1,0 (geen gelaagdheidscorrectie).
/// - **Horizontale boven-elementen** — vloer boven buitenlucht / plat dak /
///   plafond (form. 4.5/4.6): f_k = (θ_i + Δθ₁ − θ_e) / (θ_i − θ_e), met Δθ₁
///   uit tabel 2.3 (vide-gecorrigeerd ×(h/4) bij h > 4 m, voetnoot 2).
pub fn calculate_h_t_exterior(
    elements: &[&ConstructionElement],
    theta_i: f64,
    theta_e: f64,
    heating_system: HeatingSystem,
    room_height: f64,
) -> Result<f64> {
    let mut h_t_ie = 0.0;
    let delta_t = theta_i - theta_e;

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

        // f_k: 1,0 voor wanden; (θ_i + Δθ₁ − θ_e)/(θ_i − θ_e) voor horizontale
        // boven-elementen (form. 4.5/4.6). Bij Δθ=0 valt dit terug op 1,0.
        let f_k = if is_horizontal_above(element) && delta_t.abs() > 0.001 {
            let delta_theta_1 = delta_theta_1_corrected(heating_system, room_height);
            (theta_i + delta_theta_1 - theta_e) / delta_t
        } else {
            1.0
        };

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
///
// TODO A5-vervolg: tweezijdige Δθ₁/Δθ_a1 + Δθ₂/Δθ_a2 (form. 4.11/4.12) vereist
// per-element buur-heating_system in het model — geparkeerd. Eenzijdig θ_i+Δθ₁
// overschat structureel (de Δθ_a1 van de buurruimte compenseert deels), dus tot
// dat model-veld er is laten we deze tak zónder Δθ — dichter bij correct dan half.
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
///
// TODO A5-vervolg: tweezijdige Δθ₁/Δθ_a1 + Δθ₂/Δθ_a2 (form. 4.19/4.20) vereist
// per-element buur-heating_system in het model — geparkeerd. Eenzijdig θ_i+Δθ₁
// overschat structureel; tot dat model-veld er is laten we deze tak zónder Δθ.
pub fn calculate_h_t_adjacent_buildings(
    elements: &[&ConstructionElement],
    theta_i: f64,
    theta_e: f64,
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

        // Wall (verticaal) → geen Δθ₁, f_k = 1,0.
        let result = calculate_h_t_exterior(
            &[&element],
            21.0,
            -10.0,
            HeatingSystem::RadiatorenConvHtEnLuchtverwarming,
            3.0,
        );
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

    /// Helper: horizontaal dak-element (Ceiling) naar buiten.
    fn roof_element(area: f64, u: f64) -> ConstructionElement {
        ConstructionElement {
            id: "roof".to_string(),
            description: "Plat dak".to_string(),
            area,
            u_value: u,
            boundary_type: BoundaryType::Exterior,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Ceiling,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        }
    }

    /// A5: plat dak (horizontaal) krijgt Δθ₁ → f_k = (θ_i + Δθ₁ − θ_e)/(θ_i − θ_e).
    /// radi-ht Δθ₁=3, θ_i=20, θ_e=−10 → f = 33/30 = 1,1 (+10%). A=10, U=0,3.
    #[test]
    fn test_h_t_exterior_horizontal_gets_delta_theta_1() {
        let roof = roof_element(10.0, 0.3);
        let h = calculate_h_t_exterior(
            &[&roof],
            20.0,
            -10.0,
            HeatingSystem::RadiatorenConvHtEnLuchtverwarming,
            3.0,
        )
        .unwrap();
        let expected = 10.0 * 0.3 * (33.0 / 30.0);
        assert!((h - expected).abs() < 1e-9, "Expected {expected}, got {h}");
        // +10% boven de stratificatie-loze waarde (10×0.3×1.0 = 3.0).
        assert!((h - 3.3).abs() < 1e-9);
    }

    /// Δθ₁=0 systeem (vloerverwarming-hoofd / betonkern) → geen verschuiving.
    #[test]
    fn test_h_t_exterior_horizontal_zero_delta() {
        let roof = roof_element(10.0, 0.3);
        let h = calculate_h_t_exterior(&[&roof], 20.0, -10.0, HeatingSystem::Vloerverwarming, 3.0)
            .unwrap();
        assert!((h - 3.0).abs() < 1e-9, "Δθ₁=0 → f_k=1,0; got {h}");
    }

    /// Vide-correctie: h=8 m → Δθ₁ ×2. radi-ht Δθ₁=3 → 6; f = 36/30 = 1,2.
    #[test]
    fn test_h_t_exterior_vide_correction() {
        let roof = roof_element(10.0, 0.3);
        let h = calculate_h_t_exterior(
            &[&roof],
            20.0,
            -10.0,
            HeatingSystem::RadiatorenConvHtEnLuchtverwarming,
            8.0,
        )
        .unwrap();
        let expected = 10.0 * 0.3 * (36.0 / 30.0); // 3.6
        assert!((h - expected).abs() < 1e-9, "Expected {expected}, got {h}");
    }

    /// Verticale buitenwand krijgt GEEN Δθ₁ (form. 4.3), ook bij ht-systeem.
    #[test]
    fn test_h_t_exterior_wall_no_delta() {
        let mut wall = roof_element(10.0, 0.3);
        wall.vertical_position = VerticalPosition::Wall;
        let h = calculate_h_t_exterior(
            &[&wall],
            20.0,
            -10.0,
            HeatingSystem::LokaleVerwarming, // Δθ₁=4 — mag NIET toegepast worden
            3.0,
        )
        .unwrap();
        assert!((h - 3.0).abs() < 1e-9, "Wand mag geen Δθ₁ krijgen; got {h}");
    }

    /// Horizontaal plafond naar aangrenzend vertrek krijgt Δθ₁ in de teller
    /// (form. 4.11/4.12). θ_i=21, Δθ₁=3, θ_adj=18, θ_e=−10 → f=6/31.
    /// PM-besluit A5: aangrenzend-vertrek krijgt (nog) GEEN Δθ₁ — eenzijdige
    /// toepassing overschat zonder de Δθ_a1-compensatie van de buur. Ook een
    /// horizontaal plafond blijft op f_ia,k = (θ_i − θ_adj)/(θ_i − θ_e).
    #[test]
    fn test_h_t_adjacent_rooms_horizontal_no_delta() {
        let mut ceiling = roof_element(20.0, 0.5);
        ceiling.boundary_type = BoundaryType::AdjacentRoom;
        ceiling.adjacent_temperature = Some(18.0);
        let rooms: Vec<Room> = vec![];
        let h = calculate_h_t_adjacent_rooms(&[&ceiling], &rooms, 21.0, -10.0).unwrap();
        let expected = 20.0 * 0.5 * ((21.0 - 18.0) / 31.0); // 3/31, GEEN Δθ₁
        assert!((h - expected).abs() < 1e-9, "Expected {expected}, got {h}");
    }

    /// PM-besluit A5: aangrenzend-gebouw krijgt (nog) GEEN Δθ₁, ook horizontaal.
    /// θ_i=20, θ_b=15, θ_e=−10 → f=5/30.
    #[test]
    fn test_h_t_adjacent_buildings_horizontal_no_delta() {
        let mut floor = roof_element(12.0, 0.4);
        floor.boundary_type = BoundaryType::AdjacentBuilding;
        floor.vertical_position = VerticalPosition::Floor;
        let h = calculate_h_t_adjacent_buildings(&[&floor], 20.0, -10.0).unwrap();
        let expected = 12.0 * 0.4 * ((20.0 - 15.0) / 30.0); // 5/30, GEEN Δθ₁
        assert!((h - expected).abs() < 1e-9, "Expected {expected}, got {h}");
    }
}