//! Convert isso51-core `Project` and `ProjectResult` to IFCX documents.
//!
//! Produces IFCX overlays that can be composed with a geometry model.

use isso51_core::model::Project;
use isso51_core::result::ProjectResult;

use crate::document::{ifc_class, ns as ifc_ns, IfcxDataEntry, IfcxDocument};
use crate::namespace::{
    ns, Isso51Building, Isso51CalcReheat, Isso51CalcResult, Isso51CalcTransmission,
    Isso51CalcVentilation, Isso51Conditions, Isso51Construction, Isso51Report, Isso51Room,
    Isso51Ventilation,
};

/// Convert an isso51-core `Project` to an IFCX document with isso51:: namespace attributes.
///
/// Creates the IFC hierarchy (IfcProject → IfcSite → IfcBuilding → IfcSpace)
/// and annotates each entry with the corresponding isso51:: input data.
pub fn project_to_ifcx(project: &Project) -> IfcxDocument {
    let mut doc = IfcxDocument::new("isso51-ifcx");

    // IfcProject
    let project_path = uuid();
    let mut project_entry = classify(&project_path, ifc_class::PROJECT);
    set_ifc_prop(&mut project_entry, "Name", &project.info.name);
    project_entry.set_attr(
        ns::CONDITIONS,
        &Isso51Conditions {
            theta_e: project.climate.theta_e,
            theta_b_residential: Some(project.climate.theta_b_residential),
            wind_class: None,
            location: None,
        },
    );
    project_entry.set_attr(
        ns::VENTILATION,
        &Isso51Ventilation {
            system_type: serde_json::to_value(&project.ventilation.system_type)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "system_c".to_string()),
            has_heat_recovery: project.ventilation.has_heat_recovery,
            heat_recovery_efficiency: project.ventilation.heat_recovery_efficiency,
        },
    );

    // IfcSite
    let site_path = uuid();
    let site_entry = classify(&site_path, ifc_class::SITE);
    project_entry
        .children
        .insert("Site".to_string(), site_path.clone());

    // IfcBuilding
    let building_path = uuid();
    let mut building_entry = classify(&building_path, ifc_class::BUILDING);
    building_entry.set_attr(
        ns::BUILDING,
        &Isso51Building {
            building_type: serde_json::to_value(&project.building.building_type)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "detached".to_string()),
            qv10: project.building.qv10,
            total_floor_area: project.building.total_floor_area,
            security_class: serde_json::to_value(&project.building.security_class)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "b".to_string()),
            has_night_setback: project.building.has_night_setback,
            warmup_time: project.building.warmup_time,
            num_floors: Some(project.building.num_floors),
            infiltration_method: Some(
                serde_json::to_value(&project.building.infiltration_method)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "per_exterior_area".to_string()),
            ),
        },
    );

    let mut site_entry_mut = site_entry;
    site_entry_mut
        .children
        .insert("Building".to_string(), building_path.clone());

    // IfcSpaces
    for room in &project.rooms {
        let space_path = uuid();
        let mut space_entry = classify(&space_path, ifc_class::SPACE);
        set_ifc_prop(&mut space_entry, "Name", &room.name);

        space_entry.set_attr(
            ns::ROOM,
            &Isso51Room {
                function: serde_json::to_value(&room.function)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "living_room".to_string()),
                floor_area: room.floor_area,
                height: room.height,
                custom_temperature: room.custom_temperature,
                ventilation_rate: room.ventilation_rate,
                has_mechanical_exhaust: room.has_mechanical_exhaust,
                has_mechanical_supply: room.has_mechanical_supply,
                fraction_outside_air: room.fraction_outside_air,
                heating_system: Some(
                    serde_json::to_value(&room.heating_system)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| "radiator_lt".to_string()),
                ),
            },
        );

        // Construction elements as children
        for constr in &room.constructions {
            let constr_path = uuid();
            let mut constr_entry = IfcxDataEntry::new(&constr_path);
            // Classify by vertical position
            let class_code = match constr.vertical_position {
                isso51_core::model::enums::VerticalPosition::Floor
                | isso51_core::model::enums::VerticalPosition::Ceiling => ifc_class::SLAB,
                isso51_core::model::enums::VerticalPosition::Wall => ifc_class::WALL,
            };
            set_ifc_class(&mut constr_entry, class_code);

            constr_entry.set_attr(
                ns::CONSTRUCTION,
                &Isso51Construction {
                    description: constr.description.clone(),
                    area: constr.area,
                    u_value: constr.u_value,
                    boundary_type: serde_json::to_value(&constr.boundary_type)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| "exterior".to_string()),
                    material_type: serde_json::to_value(&constr.material_type)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| "masonry".to_string()),
                    vertical_position: Some(
                        serde_json::to_value(&constr.vertical_position)
                            .ok()
                            .and_then(|v| v.as_str().map(String::from))
                            .unwrap_or_else(|| "wall".to_string()),
                    ),
                    temperature_factor: constr.temperature_factor,
                    adjacent_temperature: constr.adjacent_temperature,
                    adjacent_room_path: None,
                    use_forfaitaire_thermal_bridge: constr.use_forfaitaire_thermal_bridge,
                    custom_delta_u_tb: constr.custom_delta_u_tb,
                    has_embedded_heating: constr.has_embedded_heating,
                },
            );

            space_entry
                .children
                .insert(constr.description.clone(), constr_path);
            doc.data.push(constr_entry);
        }

        building_entry
            .children
            .insert(room.name.clone(), space_path);
        doc.data.push(space_entry);
    }

    doc.data.push(project_entry);
    doc.data.push(site_entry_mut);
    doc.data.push(building_entry);

    doc
}

/// Create an IFCX overlay document with calculation results.
///
/// The overlay references existing space paths from the input document.
/// Space paths are matched by order (room index → space entry index).
pub fn result_to_ifcx(
    input_doc: &IfcxDocument,
    project: &Project,
    result: &ProjectResult,
) -> IfcxDocument {
    let mut doc = IfcxDocument::new("isso51-ifcx");

    let space_entries = input_doc.find_by_class(ifc_class::SPACE);
    let theta_e = project.climate.theta_e;

    // Per-space results
    for (room_result, space) in result.rooms.iter().zip(space_entries.iter()) {
        let theta_diff = room_result.theta_i - theta_e;
        let mut entry = IfcxDataEntry::new(&space.path);

        entry.set_attr(
            ns::CALC_RESULT,
            &Isso51CalcResult {
                phi_hl: room_result.total_heat_loss,
                phi_t: room_result.transmission.phi_t,
                phi_v: room_result.ventilation.phi_v,
                phi_rh: room_result.heating_up.phi_hu,
                theta_int: room_result.theta_i,
                phi_basis: room_result.basis_heat_loss,
                phi_extra: room_result.extra_heat_loss,
            },
        );

        entry.set_attr(
            ns::CALC_TRANSMISSION,
            &Isso51CalcTransmission {
                h_t: if theta_diff > 0.0 {
                    room_result.transmission.phi_t / theta_diff
                } else {
                    0.0
                },
                phi_t: room_result.transmission.phi_t,
            },
        );

        entry.set_attr(
            ns::CALC_VENTILATION,
            &Isso51CalcVentilation {
                h_v: room_result.ventilation.h_v,
                phi_v: room_result.ventilation.phi_v,
                qi_spec: None,
            },
        );

        entry.set_attr(
            ns::CALC_REHEAT,
            &Isso51CalcReheat {
                phi_rh: room_result.heating_up.phi_hu,
                f_rh: room_result.heating_up.f_rh,
            },
        );

        doc.data.push(entry);
    }

    // Building-level report
    let building_entries = input_doc.find_by_class(ifc_class::BUILDING);
    if let Some(building) = building_entries.first() {
        let mut entry = IfcxDataEntry::new(&building.path);
        let s = &result.summary;
        entry.set_attr(
            ns::CALC_REPORT,
            &Isso51Report {
                connection_capacity: s.connection_capacity,
                total_envelope_loss: s.total_envelope_loss,
                total_ventilation_loss: s.total_ventilation_loss,
                total_heating_up: s.total_heating_up,
                total_area: project.building.total_floor_area,
                specific_loss: if project.building.total_floor_area > 0.0 {
                    s.connection_capacity / project.building.total_floor_area
                } else {
                    0.0
                },
            },
        );
        doc.data.push(entry);
    }

    doc
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn classify(path: &str, class_code: &str) -> IfcxDataEntry {
    let mut entry = IfcxDataEntry::new(path);
    set_ifc_class(&mut entry, class_code);
    entry
}

fn set_ifc_class(entry: &mut IfcxDataEntry, class_code: &str) {
    entry.attributes.insert(
        ifc_ns::IFC_CLASS.to_string(),
        serde_json::json!({
            "code": class_code,
            "uri": format!("https://identifier.buildingsmart.org/uri/buildingsmart/ifc/4.3/class/{class_code}")
        }),
    );
}

fn set_ifc_prop(entry: &mut IfcxDataEntry, prop: &str, value: &str) {
    entry.attributes.insert(
        format!("{}::{}", ifc_ns::IFC_PROP, prop),
        serde_json::Value::String(value.to_string()),
    );
}
