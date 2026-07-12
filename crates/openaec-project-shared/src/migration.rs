//! V1 → V2 migratie.
//!
//! Bestaande projecten uit `isso51-core::model::Project` worden in een
//! `ProjectV2` gewikkeld zodat ze leesbaar blijven in de nieuwe app.
//! Specifieke veld-extractie (qv10, building_type, rooms → geometry)
//! gebeurt iteratief in F6.
//!
//! Voor nu: legacy V1 JSON wordt onder `calcs.isso51.legacy_v1` opgeslagen
//! als opaque blob en de `shared` sectie wordt minimaal gevuld vanuit de
//! `info`-velden. Geometry blijft leeg in deze migratie; de bestaande
//! `to_isso51_view` mapper produceert nog steeds een werkende
//! `isso51_core::model::Project` doordat de raw JSON gepreserveerd is.

use serde_json::Value;

use crate::calcs::{Calcs, Iso51Inputs};
use crate::project::{ProjectV2, SCHEMA_VERSION};
use crate::shared::{HeatRecovery, SharedProject, VentilationSystemKind};

/// Convert legacy V1 ISSO 51 Project JSON naar [`ProjectV2`].
///
/// # Errors
/// Geeft `serde_json::Error` als de input geen geldige JSON is.
pub fn from_legacy_v1(v1_json: &str) -> Result<ProjectV2, serde_json::Error> {
    let value: Value = serde_json::from_str(v1_json)?;
    Ok(from_legacy_v1_value(value))
}

/// Variant op [`from_legacy_v1`] die direct met een [`serde_json::Value`]
/// werkt — handig voor de API-handler die al een Value heeft.
pub fn from_legacy_v1_value(v1_value: Value) -> ProjectV2 {
    let info = v1_value.get("info").cloned().unwrap_or(Value::Null);

    let name = info
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Naamloos project")
        .to_string();

    let mut shared = SharedProject::new(name);
    shared.project_number = info
        .get("project_number")
        .and_then(|v| v.as_str())
        .map(String::from);
    shared.address = info.get("address").and_then(|v| v.as_str()).map(String::from);
    shared.client = info.get("client").and_then(|v| v.as_str()).map(String::from);
    shared.date = info.get("date").and_then(|v| v.as_str()).map(String::from);
    shared.engineer = info
        .get("engineer")
        .and_then(|v| v.as_str())
        .map(String::from);
    shared.notes = info.get("notes").and_then(|v| v.as_str()).map(String::from);

    if let Some(area) = v1_value
        .get("building")
        .and_then(|b| b.get("total_floor_area"))
        .and_then(|v| v.as_f64())
    {
        shared.gross_floor_area_m2 = Some(area);
    }
    if let Some(floors) = v1_value
        .get("building")
        .and_then(|b| b.get("num_floors"))
        .and_then(|v| v.as_u64())
    {
        shared.num_storeys = Some(floors as u32);
    }
    if let Some(year) = v1_value
        .get("building")
        .and_then(|b| b.get("construction_year"))
        .and_then(|v| v.as_u64())
    {
        shared.construction_year = Some(year as u32);
    }

    if let Some(ventilation) = v1_value.get("ventilation") {
        if let Some(system_str) = ventilation.get("system_type").and_then(|v| v.as_str()) {
            shared.ventilation_system = match system_str {
                "system_a" => Some(VentilationSystemKind::Natural),
                "system_b" => Some(VentilationSystemKind::MechSupply),
                "system_c" => Some(VentilationSystemKind::MechExhaust),
                "system_d" | "system_e" => Some(VentilationSystemKind::MechBalanced),
                _ => None,
            };
        }

        if ventilation
            .get("has_heat_recovery")
            .and_then(|v| v.as_bool())
            == Some(true)
        {
            let efficiency = ventilation
                .get("heat_recovery_efficiency")
                .and_then(|v| v.as_f64())
                .or_else(|| {
                    ventilation
                        .get("recovery_efficiency")
                        .and_then(|v| v.as_f64())
                })
                .unwrap_or(0.0);
            shared.heat_recovery = Some(HeatRecovery {
                efficiency,
                frost_protection: false,
                supply_temperature: None,
            });
        }
    }

    if shared.gross_floor_area_m2.is_none() {
        if let Some(rooms) = v1_value.get("rooms").and_then(|v| v.as_array()) {
            let sum: f64 = rooms
                .iter()
                .filter_map(|r| r.get("floor_area_m2").and_then(|v| v.as_f64()))
                .sum();
            if sum > 0.0 {
                shared.gross_floor_area_m2 = Some(sum);
            }
        }
    }

    let calcs = Calcs {
        isso51: Some(Iso51Inputs {
            legacy_v1: v1_value,
        }),
        tojuli: None,
        isso53: None,
    };

    ProjectV2 {
        schema_version: SCHEMA_VERSION,
        shared,
        geometry: Default::default(),
        calcs,
        energy: None,
        beng_geometry: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_minimal_v1_to_v2() {
        let v1 = r#"{
            "info": {"name": "Old House", "project_number": "25.012"},
            "building": {"total_floor_area": 120.0, "num_floors": 2},
            "climate": {},
            "ventilation": {},
            "rooms": []
        }"#;
        let v2 = from_legacy_v1(v1).unwrap();
        assert_eq!(v2.schema_version, SCHEMA_VERSION);
        assert_eq!(v2.shared.name, "Old House");
        assert_eq!(v2.shared.project_number.as_deref(), Some("25.012"));
        assert_eq!(v2.shared.gross_floor_area_m2, Some(120.0));
        assert_eq!(v2.shared.num_storeys, Some(2));
        assert!(v2.calcs.isso51.is_some());
        assert!(v2.calcs.tojuli.is_none());
    }

    #[test]
    fn migrate_handles_missing_fields_gracefully() {
        let v1 = r#"{"info": {}, "building": {}, "rooms": []}"#;
        let v2 = from_legacy_v1(v1).unwrap();
        assert_eq!(v2.shared.name, "Naamloos project");
        assert!(v2.shared.project_number.is_none());
    }

    #[test]
    fn migrate_v1_ventilation_systemd_with_wtw() {
        let v1 = r#"{
            "info": {"name": "Weesp"},
            "building": {},
            "ventilation": {
                "system_type": "system_d",
                "has_heat_recovery": true,
                "heat_recovery_efficiency": 0.85
            },
            "rooms": [
                {"floor_area_m2": 60.0},
                {"floor_area_m2": 40.0}
            ]
        }"#;
        let v2 = from_legacy_v1(v1).unwrap();
        assert!(matches!(
            v2.shared.ventilation_system,
            Some(VentilationSystemKind::MechBalanced)
        ));
        let hr = v2.shared.heat_recovery.as_ref().expect("heat_recovery mapped");
        assert!((hr.efficiency - 0.85).abs() < 1e-9);
        assert_eq!(v2.shared.gross_floor_area_m2, Some(100.0));
    }

    #[test]
    fn migrate_v1_ventilation_systema_no_wtw() {
        let v1 = r#"{
            "info": {"name": "Old"},
            "building": {"total_floor_area": 150.0},
            "ventilation": {"system_type": "system_a", "has_heat_recovery": false},
            "rooms": [{"floor_area_m2": 100.0}]
        }"#;
        let v2 = from_legacy_v1(v1).unwrap();
        assert!(matches!(
            v2.shared.ventilation_system,
            Some(VentilationSystemKind::Natural)
        ));
        assert!(v2.shared.heat_recovery.is_none());
        assert_eq!(v2.shared.gross_floor_area_m2, Some(150.0));
    }

    #[test]
    fn migrate_v1_uses_recovery_efficiency_fallback() {
        let v1 = r#"{
            "info": {},
            "ventilation": {
                "system_type": "system_d",
                "has_heat_recovery": true,
                "recovery_efficiency": 0.89
            }
        }"#;
        let v2 = from_legacy_v1(v1).unwrap();
        let hr = v2.shared.heat_recovery.as_ref().unwrap();
        assert!((hr.efficiency - 0.89).abs() < 1e-9);
    }
}
