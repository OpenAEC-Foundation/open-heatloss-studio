//! View-mappers van `ProjectV2` naar calc-specifieke runtime structs.
//!
//! F2 levert alleen de ISSO 51 view-mapper — die haalt de legacy V1 JSON
//! terug uit `calcs.isso51.legacy_v1` en deserialiseert naar
//! [`isso51_core::model::Project`]. F6 zal dit vervangen met een echte
//! mapper die `shared` + `geometry` gebruikt.

use crate::project::ProjectV2;
use isso51_core::model::Project;

/// Errors uit view-mappers.
#[derive(Debug, thiserror::Error)]
pub enum ViewError {
    /// `calcs.isso51` is niet ingevuld op deze ProjectV2.
    #[error("project heeft geen isso51-sectie ingevuld")]
    MissingIsso51,
    /// `calcs.isso53` is niet ingevuld op deze ProjectV2.
    #[error("project heeft geen isso53-sectie ingevuld")]
    MissingIsso53,
    /// JSON deserialisatie van de legacy blob faalde.
    #[error("JSON deserialisatie faalde: {0}")]
    Deserialize(#[from] serde_json::Error),
}

/// Bouw een [`isso51_core::model::Project`] uit een [`ProjectV2`].
///
/// **F2-implementatie:** leest `calcs.isso51.legacy_v1` als JSON-blob
/// terug. Werkt zolang die blob origineel een geldige V1 Project was.
/// F6 vervangt dit met een echte view die `shared` + `geometry` +
/// `Iso51Inputs` combineert.
pub fn to_isso51_project(v2: &ProjectV2) -> Result<Project, ViewError> {
    let isso51 = v2
        .calcs
        .isso51
        .as_ref()
        .ok_or(ViewError::MissingIsso51)?;
    let project: Project = serde_json::from_value(isso51.legacy_v1.clone())?;
    Ok(project)
}

/// Bouw een [`isso53_core::model::Project`] uit een [`ProjectV2`].
///
/// **Transitional implementatie:** leest `calcs.isso53.legacy` als JSON-blob
/// terug, parallel aan het ISSO 51-pattern. Toekomstige versies zullen dit
/// vervangen met een echte view-mapper die `shared` + `geometry` +
/// `Iso53Inputs` combineert.
pub fn to_isso53_project(v2: &ProjectV2) -> Result<isso53_core::model::Project, ViewError> {
    let isso53 = v2
        .calcs
        .isso53
        .as_ref()
        .ok_or(ViewError::MissingIsso53)?;
    let project: isso53_core::model::Project =
        serde_json::from_value(isso53.legacy.clone())?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::from_legacy_v1;

    #[test]
    fn legacy_v1_round_trip_via_v2() {
        let v1 = r#"{
            "info": {"name": "RT Test"},
            "building": {
                "building_type": "detached",
                "qv10": 0.4,
                "total_floor_area": 100.0,
                "num_floors": 2,
                "security_class": "b"
            },
            "climate": {"theta_e": -10.0},
            "ventilation": {"system_type": "system_c"},
            "rooms": []
        }"#;
        let v2 = from_legacy_v1(v1).unwrap();
        let v1_again = to_isso51_project(&v2).unwrap();
        assert_eq!(v1_again.info.name, "RT Test");
        assert!((v1_again.building.qv10 - 0.4).abs() < 1e-9);
        assert!((v1_again.climate.theta_e - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn empty_v2_has_no_isso51() {
        let v2 = ProjectV2::new("Empty");
        let err = to_isso51_project(&v2).unwrap_err();
        assert!(matches!(err, ViewError::MissingIsso51));
    }

    #[test]
    fn empty_v2_has_no_isso53() {
        let v2 = ProjectV2::new("Empty");
        let err = to_isso53_project(&v2).unwrap_err();
        assert!(matches!(err, ViewError::MissingIsso53));
    }

    #[test]
    fn isso53_legacy_round_trip() {
        use crate::calcs::{Calcs, Iso53Inputs};

        let isso53_json = serde_json::json!({
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
            "rooms": [],
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
        });

        let mut v2 = ProjectV2::new("Test ISSO 53");
        v2.calcs = Calcs {
            isso51: None,
            tojuli: None,
            isso53: Some(Iso53Inputs {
                legacy: isso53_json,
            }),
            isso74: None,
        };

        let project = to_isso53_project(&v2).unwrap();
        assert_eq!(project.info.name, "Test ISSO 53");
        assert_eq!(project.climate.theta_e, -10.0);
    }
}
