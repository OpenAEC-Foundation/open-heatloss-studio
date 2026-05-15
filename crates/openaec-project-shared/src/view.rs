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
    /// JSON deserialisatie van de legacy blob faalde.
    #[error("isso51 legacy_v1 JSON kon niet gedeserialiseerd worden: {0}")]
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
}
