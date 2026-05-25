//! Per-room heat loss orchestration for ISSO 53 (§4.1).
//!
//! Combines transmission, ventilation, infiltration, heating-up and gain
//! per room into the total Φ_HL,i (formule 4.1).

use crate::error::Result;
use crate::model::{Building, DesignConditions, HeatingUpConfig, Room, VentilationConfig, BoundaryType};
use crate::result::RoomResult;
use crate::tables::temperature::design_indoor_temperature;
use crate::calc::{transmission, infiltration, ventilation, heating_up};

/// Calculate the complete heat loss result for a single room.
/// ISSO 53 formule 4.1, 4.2.
pub fn calculate_room(
    room: &Room,
    all_rooms: &[Room],
    building: &Building,
    climate: &DesignConditions,
    ventilation: &VentilationConfig,
    infiltration_method: &infiltration::InfiltrationMethod,
    heating_up_config: &HeatingUpConfig,
) -> Result<RoomResult> {
    // Determine design indoor temperature θ_i
    let theta_i = room.custom_temperature
        .unwrap_or_else(|| design_indoor_temperature(room.gebruiks_functie, room.ruimte_type));

    // Calculate transmission heat loss (with adjacent room resolution)
    let transmission_result = calculate_transmission_with_adjacent_rooms(
        room, all_rooms, building, climate, theta_i
    )?;

    // Calculate infiltration heat loss
    let phi_i = infiltration::calculate_phi_i(
        room, building, climate, infiltration_method
    )?;
    let h_i = infiltration::calculate_h_i(
        room, building, infiltration_method
    )?;

    // Calculate ventilation heat loss
    let ventilation_result = ventilation::calculate_ventilation(
        room, ventilation, theta_i, climate.theta_e
    )?;

    // Calculate heating-up supplement
    let phi_hu = heating_up::calculate_heating_up(room, heating_up_config)?;

    // Internal heat gains (for now = 0, as per MVP scope)
    let phi_gain = 0.0; // TODO: implement internal gains in future batch

    // Total heat loss: Φ_HL,i = Φ_T + Φ_V + Φ_I + Φ_hu − Φ_gain (formule 4.1)
    let total_heat_loss = transmission_result.phi_t + ventilation_result.phi_vent
        + phi_i + phi_hu - phi_gain;

    Ok(RoomResult {
        room_id: room.id.clone(),
        room_name: room.name.clone(),
        theta_i,
        phi_t: transmission_result.phi_t,
        phi_v: ventilation_result.phi_vent,
        phi_i,
        phi_hu,
        phi_system: 0.0, // TODO: system losses in future scope
        phi_gain,
        total_heat_loss,
        h_t_exterior: transmission_result.h_t_exterior,
        h_t_adjacent_rooms: transmission_result.h_t_adjacent_rooms,
        h_t_unheated: transmission_result.h_t_unheated,
        h_t_adjacent_buildings: transmission_result.h_t_adjacent_buildings,
        h_t_ground: transmission_result.h_t_ground,
        h_v: ventilation_result.h_v,
        h_i,
    })
}

/// Calculate transmission with adjacent room resolution (formule 4.9).
///
/// This extends the basic transmission calculation by resolving adjacent room
/// references and calculating the H_T,ia component according to formule 4.9-4.12.
fn calculate_transmission_with_adjacent_rooms(
    room: &Room,
    all_rooms: &[Room],
    building: &Building,
    climate: &DesignConditions,
    theta_i: f64,
) -> Result<transmission::TransmissionResult> {
    // Start with basic transmission calculation
    let mut result = transmission::calculate_transmission(room, all_rooms, building, climate)?;

    // Calculate H_T,ia for adjacent rooms (formule 4.9)
    let mut h_t_ia = 0.0;

    for element in &room.constructions {
        if element.boundary_type == BoundaryType::AdjacentRoom {
            if let Some(adjacent_room_id) = &element.adjacent_room_id {
                // Resolve the adjacent room
                let adjacent_room = all_rooms
                    .iter()
                    .find(|r| &r.id == adjacent_room_id)
                    .ok_or_else(|| {
                        crate::error::Isso53Error::InvalidInput(format!(
                            "Adjacent room '{}' not found for element '{}'",
                            adjacent_room_id, element.id
                        ))
                    })?;

                // Calculate θ_b (design temperature of adjacent room)
                let theta_b = adjacent_room.custom_temperature
                    .unwrap_or_else(|| design_indoor_temperature(
                        adjacent_room.gebruiks_functie,
                        adjacent_room.ruimte_type
                    ));

                // Calculate f_ia,k according to formule 4.10-4.12
                let f_ia_k = if (theta_i - climate.theta_e).abs() < 0.001 {
                    0.0 // Avoid division by zero
                } else {
                    (theta_i - theta_b) / (theta_i - climate.theta_e)
                };

                // Add to H_T,ia: A × U × f_ia,k
                h_t_ia += element.area * element.u_value * f_ia_k;
            } else {
                return Err(crate::error::Isso53Error::InvalidInput(format!(
                    "Element '{}' has boundary_type=AdjacentRoom but no adjacent_room_id",
                    element.id
                )));
            }
        }
    }

    // Update the result with calculated H_T,ia
    result.h_t_adjacent_rooms = h_t_ia;

    // Add adjacent room contribution to existing phi_t (avoid θ_i inconsistency)
    let phi_t_adjacent = h_t_ia * (theta_i - climate.theta_e);
    result.phi_t += phi_t_adjacent;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    #[test]
    fn test_calculate_room_smoke() {
        let room = create_minimal_room();
        let all_rooms = vec![room.clone()];
        let building = create_test_building();
        let climate = DesignConditions::default();
        let ventilation = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };
        let infiltration_method = infiltration::InfiltrationMethod::Known {
            qv10_kar_class: crate::tables::infiltration::Qv10Class::From040To060,
        };
        let heating_up_config = HeatingUpConfig::default();

        let result = calculate_room(
            &room, &all_rooms, &building, &climate, &ventilation,
            &infiltration_method, &heating_up_config
        );

        assert!(result.is_ok(), "Room calculation should succeed: {:?}", result);
        let room_result = result.unwrap();
        assert_eq!(room_result.room_id, "test_room");
        assert!(room_result.theta_i > 0.0, "Should have design temperature");
    }

    #[test]
    fn test_adjacent_room_resolution() {
        // Create room with adjacent room element
        let mut room1 = create_minimal_room();
        room1.id = "room1".to_string();
        room1.constructions.push(ConstructionElement {
            id: "wall_to_room2".to_string(),
            description: "Wall to room 2".to_string(),
            area: 10.0,
            u_value: 0.5,
            boundary_type: BoundaryType::AdjacentRoom,
            adjacent_room_id: Some("room2".to_string()),
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_temperature: Some(18.0),
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        });

        let room2 = Room {
            id: "room2".to_string(),
            name: "Room 2".to_string(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Verblijfsruimte,
            floor_area: 20.0,
            height: 3.0,
            custom_temperature: Some(18.0), // Different temperature
            constructions: vec![],
            bezetting: Bezetting {
                personen: None,
                personen_per_m2_default: None,
            },
            infiltration_reduction_z: 1.0,
        };

        let all_rooms = vec![room1.clone(), room2];
        let climate = DesignConditions::default();

        let building = crate::model::Building {
            building_shape: crate::model::enums::BuildingShape::Meerlaags,
            construction_year: 2020,
            building_position: crate::model::enums::GebouwTypePositie::MeerlaagsGeheel,
            ventilation_system: crate::model::enums::VentilationSystemType::SystemD,
            thermal_mass: crate::model::enums::ThermalMass::Gemiddeld,
            wind_pressure_type: crate::model::enums::GebouwTypeWinddruk::MeerlaagsStandaard,
            building_height: Some(3.0),
            building_length: Some(20.0),
            building_width: Some(15.0),
            heating_system: crate::model::enums::HeatingSystem::default(),
            source_zone_config: crate::tables::SourceZoneConfig::default(),
        };

        let result = calculate_transmission_with_adjacent_rooms(
            &room1, &all_rooms, &building, &climate, 20.0
        );

        assert!(result.is_ok(), "Adjacent room resolution should work: {:?}", result);
        let transmission = result.unwrap();
        assert!(transmission.h_t_adjacent_rooms > 0.0, "Should have adjacent room transmission");
    }

    fn create_minimal_room() -> Room {
        Room {
            id: "test_room".to_string(),
            name: "Test Room".to_string(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Verblijfsruimte,
            floor_area: 25.0,
            height: 3.0,
            custom_temperature: None,
            constructions: vec![],
            bezetting: Bezetting {
                personen: None,
                personen_per_m2_default: None,
            },
            infiltration_reduction_z: 1.0,
        }
    }

    fn create_test_building() -> Building {
        Building {
            building_shape: BuildingShape::Meerlaags,
            construction_year: 2020,
            building_position: GebouwTypePositie::MeerlaagsTussen,
            ventilation_system: VentilationSystemType::SystemB,
            thermal_mass: ThermalMass::Gemiddeld,
            wind_pressure_type: crate::model::enums::GebouwTypeWinddruk::MeerlaagsStandaard,
            building_height: None,
            building_length: None,
            building_width: None,
            heating_system: Default::default(),
            source_zone_config: Default::default(),
        }
    }
}
