//! Top-level ProjectV2 + version detection.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::calcs::Calcs;
use crate::energy::EnergyInput;
use crate::geometry::SharedGeometry;
use crate::shared::SharedProject;

/// Schema version geserialiseerd op de top-level. V1 = oude ISSO 51-only
/// `isso51_core::model::Project`. V2 = drielagig multi-calc (deze module).
pub const SCHEMA_VERSION: u32 = 2;

/// Multi-calc project root. Zie ADR-002.
///
/// Serialiseert naar JSON met top-level `schema_version` zodat backend
/// readers V1 en V2 kunnen onderscheiden. V1 → V2 conversie in
/// [`crate::migration::from_legacy_v1`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectV2 {
    /// Schema version, altijd [`SCHEMA_VERSION`] voor V2 projecten.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Cross-calc metadata + locatie + gebouwtype.
    pub shared: SharedProject,

    /// Gedeelde geometrie (spaces + constructions + openings).
    #[serde(default)]
    pub geometry: SharedGeometry,

    /// Per-norm specifieke inputs. Map kan leeg zijn voor een vers project.
    #[serde(default)]
    pub calcs: Calcs,

    /// Installatie- en hernieuwbaar-invoer voor de NTA 8800 / BENG-keten (F2).
    /// Additief: afwezig in bestaande project-JSON's ⇒ `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy: Option<EnergyInput>,
}

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

impl ProjectV2 {
    /// Maak een leeg V2 project met een minimum aan velden gevuld.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            shared: SharedProject::new(name),
            geometry: SharedGeometry::default(),
            calcs: Calcs::default(),
            energy: None,
        }
    }

    /// Detecteert versie uit ruwe JSON-bytes. Geeft `2` voor V2, `1` voor
    /// oude ISSO 51 projecten (ontbrekend `schema_version` veld), of een
    /// expliciet versie-nummer als anders geserialiseerd.
    pub fn detect_version(json: &str) -> Result<u32, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        Ok(value
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_project_serde_round_trip() {
        let p = ProjectV2::new("Test");
        let json = serde_json::to_string(&p).unwrap();
        let back: ProjectV2 = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, SCHEMA_VERSION);
        assert_eq!(back.shared.name, "Test");
    }

    #[test]
    fn detect_v1_without_schema_version() {
        let v1_json = r#"{"info": {"name": "Old"}, "building": {}, "rooms": []}"#;
        assert_eq!(ProjectV2::detect_version(v1_json).unwrap(), 1);
    }

    #[test]
    fn detect_v2_with_schema_version() {
        let v2_json = r#"{"schema_version": 2, "shared": {"name": "X"}}"#;
        assert_eq!(ProjectV2::detect_version(v2_json).unwrap(), 2);
    }

    #[test]
    fn v2_json_without_energy_block_deserializes() {
        // Bestaand ProjectV2-JSON van vóór het energy-blok: geen `energy`-veld.
        let json = r#"{
            "schema_version": 2,
            "shared": {"name": "Bestaand project"},
            "geometry": {"spaces": []},
            "calcs": {}
        }"#;
        let p: ProjectV2 = serde_json::from_str(json).unwrap();
        assert_eq!(p.shared.name, "Bestaand project");
        assert!(p.energy.is_none());
    }

    #[test]
    fn energy_absent_is_skipped_in_serialization() {
        let p = ProjectV2::new("Test");
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("energy"));
    }

    #[test]
    fn project_with_energy_round_trips() {
        use crate::energy::{EnergyInput, PvInput};
        let mut p = ProjectV2::new("Met PV");
        p.energy = Some(EnergyInput {
            pv: vec![PvInput {
                id: None,
                name: None,
                peak_power_kwp: 4.0,
                azimuth_degrees: 180.0,
                tilt_degrees: 30.0,
                system_efficiency: None,
                inverter_efficiency: None,
                shadow_factor: None,
            }],
            ..EnergyInput::default()
        });
        let json = serde_json::to_string(&p).unwrap();
        let back: ProjectV2 = serde_json::from_str(&json).unwrap();
        assert_eq!(back.energy.unwrap().pv[0].peak_power_kwp, 4.0);
    }
}
