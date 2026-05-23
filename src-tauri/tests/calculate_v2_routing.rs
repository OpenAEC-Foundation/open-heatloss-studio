//! Integration tests voor ProjectV2 dual-pipeline routing.

use isso51_desktop_lib::commands;
use openaec_project_shared::{
    calcs::{ActiveNorm, Calcs, Iso51Inputs, Iso53Inputs},
    ProjectV2,
};

#[test]
fn calculate_v2_routes_isso51() {
    let mut v2 = ProjectV2::new("Test ISSO 51");
    v2.calcs = Calcs {
        isso51: Some(Iso51Inputs {
            legacy_v1: serde_json::json!({
                "info": {"name": "Test ISSO 51"},
                "building": {
                    "building_type": "detached",
                    "qv10": 0.4,
                    "total_floor_area": 100.0,
                    "num_floors": 2,
                    "security_class": "b"
                },
                "climate": {"theta_e": -10.0},
                "ventilation": {"system_type": "system_c"},
                "rooms": [{
                    "id": "wk",
                    "name": "Woonkamer",
                    "function": "living_room",
                    "floor_area": 25.0,
                    "height": 2.7,
                    "theta_i": 20.0,
                    "constructions": []
                }]
            }),
        }),
        tojuli: None,
        isso53: None,
    };

    assert_eq!(v2.calcs.active_norm(), ActiveNorm::Isso51);

    let result = commands::calculate_v2(v2);
    assert!(result.is_ok(), "ISSO 51 calculation should succeed: {:?}", result);

    let json_result = result.unwrap();
    assert!(json_result.is_object(), "Result should be a JSON object");

    // Verify it has the expected structure of an ISSO 51 ProjectResult
    assert!(json_result.get("rooms").is_some(), "Should have rooms array");
    assert!(json_result.get("summary").is_some(), "Should have summary object");
}

#[test]
fn calculate_v2_routes_isso53() {
    let mut v2 = ProjectV2::new("Test ISSO 53");
    v2.calcs = Calcs {
        isso51: None,
        tojuli: None,
        isso53: Some(Iso53Inputs {
            legacy: serde_json::json!({
                "info": {
                    "name": "Test ISSO 53",
                    "projectNumber": null,
                    "address": null,
                    "client": null,
                    "date": null,
                    "engineer": null,
                    "notes": null
                },
                "building": {
                    "buildingShape": "meerlaags",
                    "constructionYear": 2020,
                    "buildingPosition": "meerlaagsTussen",
                    "ventilationSystem": "systemD",
                    "thermalMass": "gemiddeld",
                    "windPressureType": "meerlaagsStandaard"
                },
                "climate": {
                    "thetaE": -10.0,
                    "thetaMe": 9.0
                },
                "ventilation": {
                    "systemType": "systemD",
                    "hasHeatRecovery": true,
                    "heatRecoveryEfficiency": 0.85,
                    "frostProtection": null,
                    "supplyTemperature": null,
                    "hasPreheating": false,
                    "preheatingTemperature": null
                },
                "rooms": [{
                    "id": "K01",
                    "name": "Kantoor 1.01",
                    "gebruiksFunctie": "kantoor",
                    "ruimteType": "verblijfsruimte",
                    "floorArea": 25.0,
                    "height": 2.8,
                    "customTemperature": null,
                    "constructions": [
                        {
                            "id": "wall-n",
                            "description": "Buitenwand noord",
                            "area": 14.0,
                            "uValue": 0.21,
                            "boundaryType": "exterior",
                            "materialType": "masonry",
                            "temperatureFactor": null,
                            "adjacentRoomId": null,
                            "adjacentTemperature": null,
                            "verticalPosition": "wall",
                            "useForfaitaireThermalBridge": true,
                            "customDeltaUTb": null,
                            "groundParams": null,
                            "hasEmbeddedHeating": false,
                            "unheatedSpace": null
                        },
                        {
                            "id": "floor",
                            "description": "Vloer op grond",
                            "area": 25.0,
                            "uValue": 0.22,
                            "boundaryType": "ground",
                            "materialType": "masonry",
                            "temperatureFactor": null,
                            "adjacentRoomId": null,
                            "adjacentTemperature": null,
                            "verticalPosition": "floor",
                            "useForfaitaireThermalBridge": true,
                            "customDeltaUTb": null,
                            "groundParams": {
                                "uEquivalent": 0.18,
                                "groundWaterFactor": 1.0,
                                "fIg": 1.0
                            },
                            "hasEmbeddedHeating": false,
                            "unheatedSpace": null
                        }
                    ],
                    "bezetting": {
                        "personen": 3,
                        "persorenPerM2Default": null
                    },
                    "infiltrationReductionZ": 1.0
                }],
                "infiltrationMethod": {
                    "known": {
                        "qv10_kar_class": "From040To060"
                    }
                },
                "heatingUp": {
                    "setbackActive": false,
                    "pWPerM2": 0.0,
                    "warmupMinutes": 60.0
                }
            }),
        }),
    };

    assert_eq!(v2.calcs.active_norm(), ActiveNorm::Isso53);

    let result = commands::calculate_v2(v2);
    assert!(result.is_ok(), "ISSO 53 calculation should succeed: {:?}", result);

    let json_result = result.unwrap();
    assert!(json_result.is_object(), "Result should be a JSON object");

    // Verify it has the expected structure of an ISSO 53 ProjectResult
    assert!(json_result.get("rooms").is_some(), "Should have rooms array");
    assert!(json_result.get("summary").is_some(), "Should have summary object");
}

#[test]
fn calculate_v2_fails_on_empty_calcs() {
    let v2 = ProjectV2::new("Empty project");

    // Default project has no calc inputs — should route to ISSO 51 but fail
    assert_eq!(v2.calcs.active_norm(), ActiveNorm::Isso51);

    let result = commands::calculate_v2(v2);
    assert!(result.is_err(), "Empty project should fail to calculate");

    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("isso51-sectie"),
        "Error should mention missing ISSO 51 section: {error_msg}"
    );
}