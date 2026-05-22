//! Construction element model for ISSO 51 heat loss calculations.
//!
//! A construction element represents a single boundary surface of a room
//! (wall, floor, ceiling, window, door) with its thermal properties.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enums::{BoundaryType, MaterialType, VerticalPosition};

/// A single construction element forming part of a room boundary.
/// ISSO 51 §2.5 — each element contributes to the room's heat loss.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionElement {
    /// Unique identifier for this element.
    pub id: String,

    /// Human-readable description (e.g., "buitenwand noord", "raam woonkamer").
    pub description: String,

    /// Area of the element in m².
    pub area: f64,

    /// U-value (thermal transmittance) in W/(m²·K).
    pub u_value: f64,

    /// Type of boundary this element faces.
    pub boundary_type: BoundaryType,

    /// Material type: masonry or non-masonry.
    /// Affects thermal bridge correction in the forfaitaire method.
    pub material_type: MaterialType,

    /// Temperature correction factor f_k (dimensionless).
    /// For exterior elements: typically 1.0.
    /// For unheated spaces: from ISSO 51 Table 4.1.
    /// For adjacent rooms: calculated from temperature difference.
    /// For neighboring dwellings: calculated with Δθ corrections.
    /// Set to `None` to have it auto-calculated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_factor: Option<f64>,

    /// ID of the adjacent room (for `BoundaryType::AdjacentRoom`).
    ///
    /// The transmission calculation resolves the adjacent room's design
    /// temperature at runtime by looking the room up in the `Project` room
    /// list. This is the canonical source — `adjacent_temperature` below is
    /// a legacy fallback only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjacent_room_id: Option<String>,

    /// Legacy: hardcoded design temperature of the adjacent space in °C.
    ///
    /// **Deprecated** — the calculation now derives the adjacent-room
    /// temperature from `adjacent_room_id` via a live lookup in
    /// `Project.rooms`. This field is retained for backward compatibility
    /// with older saved projects and is only consulted as a fallback when
    /// `adjacent_room_id` cannot be resolved. New code should leave this
    /// field at `None` and rely on the live room lookup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjacent_temperature: Option<f64>,

    /// Vertical position: floor, ceiling, or wall.
    /// Used for fb calculation to neighboring dwellings.
    #[serde(default = "default_vertical_position")]
    pub vertical_position: VerticalPosition,

    /// Whether to use the forfaitaire thermal bridge correction (ΔU_TB = 0.1).
    /// Only applies to exterior boundary elements (BoundaryType::Exterior).
    #[serde(default = "default_true")]
    pub use_forfaitaire_thermal_bridge: bool,

    /// Custom ΔU_TB value in W/(m²·K) if not using the forfaitaire method.
    /// Overrides the default 0.1 W/(m²·K) correction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_delta_u_tb: Option<f64>,

    /// Ground parameters, only for BoundaryType::Ground elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_params: Option<GroundParameters>,

    /// Whether this element has floor/wall/ceiling heating behind it.
    /// Relevant for system loss calculations (§2.9).
    #[serde(default)]
    pub has_embedded_heating: bool,

    /// Reference to a `CatalogEntry.id` produced by the thermal import.
    /// `None` for openings, manually-added elements and projects from
    /// before the catalog refactor (legacy projects keep working).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_ref: Option<String>,

    /// Onderbouwing van de samengestelde raam-U-waarde (U_w).
    ///
    /// Optioneel, alleen aanwezig op kozijn-/vullings-elementen waarvoor de
    /// U_w-calculator is gebruikt. De rekenkern negeert dit veld volledig —
    /// uitsluitend `u_value` is de rekeningang. `uw_breakdown` dient als
    /// persistente onderbouwing zodat de calculator herladen kan worden en
    /// de opbouw in het rapport terechtkomt. `None` voor alle niet-kozijn
    /// elementen en voor projecten van vóór de U_w-calculator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uw_breakdown: Option<UwBreakdown>,
}

/// Type randafstandhouder voor de Ψ_g-waarde van de beglazingsrand.
///
/// Mirror van `nta8800_tables::glazing_edge::SpacerKind` — lokaal
/// gedefinieerd om geen crate-dependency op `nta8800-tables` te introduceren
/// in `isso51-core`. De 4 varianten en hun snake_case serialisatie moeten
/// gelijk blijven aan de bron-enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Spacer {
    /// Aluminium spacer (standaard, koudebruggend)
    Aluminium,
    /// RVS spacer (verbeterde prestatie)
    Stainless,
    /// Warm-edge polymeer spacer
    WarmEdgePolymer,
    /// Warm-edge schuim spacer (beste prestatie)
    WarmEdgeFoam,
}

/// Onderbouwing van de samengestelde raam-U-waarde U_w.
///
/// Volgens NEN-EN-ISO 10077-1:
/// `U_w = (ΣA_g·U_g + ΣA_f·U_f + Σl_g·Ψ_g) / (ΣA_g + ΣA_f)`.
///
/// Standaard-detailniveau: uniform kozijn — één U_g voor alle ruiten, één
/// U_f, uniforme profielbreedte. Afgeleide waarden (`a_g_m2`, `a_f_m2`,
/// `l_g_m`, `u_w`) worden gecachet maar zijn herberekenbaar uit de invoer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UwBreakdown {
    /// Raambreedte buitenwerks in mm.
    pub width_mm: f64,

    /// Raamhoogte buitenwerks in mm.
    pub height_mm: f64,

    /// Uniforme profielbreedte (buitenkozijn + tussenprofielen) in mm.
    pub frame_width_mm: f64,

    /// Aantal ruit-kolommen (ruit-indeling), standaard 1.
    pub pane_columns: u32,

    /// Aantal ruit-rijen (ruit-indeling), standaard 1.
    pub pane_rows: u32,

    /// Glas-U-waarde U_g in W/(m²·K) — handmatige invoer van de glasleverancier.
    pub u_g: f64,

    /// Herkomst van U_g — vrije-tekst label van de gekozen glasopbouw
    /// (bv. `"Triple glas — U_g 0.60"`). `None` bij handmatige invoer.
    /// Vrije tekst, géén catalogus-id — robuust tegen catalogus-wijzigingen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub u_g_source: Option<String>,

    /// Profiel-U-waarde U_f in W/(m²·K) — handmatige invoer van de profielfabrikant.
    pub u_f: f64,

    /// Herkomst van U_f — vrije-tekst label van het gekozen profielsysteem
    /// (bv. `"Reynaers — MasterLine 8"`). `None` bij handmatige invoer.
    /// Vrije tekst, géén catalogus-id — robuust tegen catalogus-wijzigingen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub u_f_source: Option<String>,

    /// Type randafstandhouder voor de Ψ_g-tabelwaarde.
    /// `None` = volledig handmatige Ψ_g-invoer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacer: Option<Spacer>,

    /// Effectieve lineaire warmtedoorgangscoëfficiënt Ψ_g van de
    /// beglazingsrand in W/(m·K).
    pub psi_g: f64,

    /// `true` wanneer `psi_g` een handmatige override is op de
    /// spacer-tabelwaarde.
    pub psi_g_is_manual: bool,

    /// Afgeleid: totale glasoppervlakte ΣA_g in m². Gecachet, herberekenbaar.
    pub a_g_m2: f64,

    /// Afgeleid: totale profieloppervlakte ΣA_f in m². Gecachet, herberekenbaar.
    pub a_f_m2: f64,

    /// Afgeleid: totale zichtbare glasrand-omtrek Σl_g in m.
    /// Gecachet, herberekenbaar.
    pub l_g_m: f64,

    /// Resultaat: samengestelde raam-U-waarde U_w in W/(m²·K).
    pub u_w: f64,
}

/// Parameters for ground heat loss calculation.
/// ISSO 51 §2.5.5, formule (4.18): H_T,ig = 1.45 × G_w × Σ(A_k × f_g2 × U_e,k)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GroundParameters {
    /// Equivalent U-value for the ground element U_e,k in W/(m²·K).
    pub u_equivalent: f64,

    /// Ground water correction factor G_w (dimensionless).
    /// Typically 1.0 for normal conditions, higher for high water table.
    #[serde(default = "default_gw")]
    pub ground_water_factor: f64,

    /// Temperature correction factor f_g2 (dimensionless).
    /// Accounts for seasonal ground temperature variation.
    #[serde(default = "default_fg2")]
    pub fg2: f64,
}

fn default_vertical_position() -> VerticalPosition {
    VerticalPosition::Wall
}

fn default_true() -> bool {
    true
}

fn default_gw() -> f64 {
    1.0
}

fn default_fg2() -> f64 {
    1.0
}

/// A library entry for a reusable construction type.
/// Can be referenced by multiple ConstructionElements.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionType {
    /// Unique identifier.
    pub id: String,

    /// Human-readable name (e.g., "Spouwmuur Rc=4.5").
    pub name: String,

    /// U-value in W/(m²·K).
    pub u_value: f64,

    /// Material type.
    pub material_type: MaterialType,

    /// Layers making up this construction (optional detail).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<ConstructionLayer>,
}

/// A single layer in a construction assembly.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionLayer {
    /// Material name.
    pub material: String,

    /// Thickness in mm.
    pub thickness: f64,

    /// Thermal conductivity λ in W/(m·K).
    pub lambda: f64,
}

impl ConstructionLayer {
    /// Calculate the thermal resistance R of this layer in m²·K/W.
    pub fn thermal_resistance(&self) -> f64 {
        (self.thickness / 1000.0) / self.lambda
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::enums::{BoundaryType, MaterialType};

    #[test]
    fn test_layer_thermal_resistance() {
        let layer = ConstructionLayer {
            material: "insulation".to_string(),
            thickness: 100.0,
            lambda: 0.035,
        };
        let r = layer.thermal_resistance();
        assert!((r - 2.857).abs() < 0.01);
    }

    /// Build a minimal ConstructionElement for serde tests.
    fn sample_element() -> ConstructionElement {
        ConstructionElement {
            id: "ce1".to_string(),
            description: "raam woonkamer".to_string(),
            area: 2.4,
            u_value: 1.4,
            boundary_type: BoundaryType::Exterior,
            material_type: MaterialType::NonMasonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
            uw_breakdown: None,
        }
    }

    #[test]
    fn construction_element_round_trips_without_uw_breakdown() {
        let element = sample_element();
        let json = serde_json::to_string(&element).expect("serialize");
        // Optioneel veld met skip_serializing_if mag niet in de JSON staan.
        assert!(
            !json.contains("uw_breakdown"),
            "afwezig uw_breakdown moet wegvallen uit de JSON: {json}"
        );
        let parsed: ConstructionElement = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, element);
        assert!(parsed.uw_breakdown.is_none());
    }

    #[test]
    fn construction_element_round_trips_with_uw_breakdown() {
        let mut element = sample_element();
        element.uw_breakdown = Some(UwBreakdown {
            width_mm: 1200.0,
            height_mm: 1500.0,
            frame_width_mm: 80.0,
            pane_columns: 1,
            pane_rows: 1,
            u_g: 1.1,
            u_g_source: Some("HR++ glas — U_g 1.10".to_string()),
            u_f: 1.3,
            u_f_source: Some("Reynaers — MasterLine 8".to_string()),
            spacer: Some(Spacer::WarmEdgePolymer),
            psi_g: 0.04,
            psi_g_is_manual: false,
            a_g_m2: 1.5,
            a_f_m2: 0.3,
            l_g_m: 5.0,
            u_w: 1.18,
        });
        let json = serde_json::to_string(&element).expect("serialize");
        assert!(json.contains("uw_breakdown"));
        // Spacer moet snake_case serialiseren.
        assert!(
            json.contains("warm_edge_polymer"),
            "spacer moet als snake_case string serialiseren: {json}"
        );
        let parsed: ConstructionElement = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, element);
    }

    #[test]
    fn construction_element_without_field_deserializes_to_none() {
        // Een project-JSON van vóór de U_w-calculator heeft geen
        // `uw_breakdown` veld — moet foutloos naar None deserialiseren.
        let json = r#"{
            "id": "ce1",
            "description": "buitenwand noord",
            "area": 12.0,
            "u_value": 0.3,
            "boundary_type": "exterior",
            "material_type": "masonry"
        }"#;
        let parsed: ConstructionElement = serde_json::from_str(json).expect("should parse");
        assert!(parsed.uw_breakdown.is_none());
    }

    #[test]
    fn uw_breakdown_round_trips_with_manual_psi_g() {
        // spacer = None betekent volledig handmatige Ψ_g-invoer.
        let breakdown = UwBreakdown {
            width_mm: 1000.0,
            height_mm: 1000.0,
            frame_width_mm: 70.0,
            pane_columns: 2,
            pane_rows: 1,
            u_g: 1.0,
            u_g_source: None,
            u_f: 1.5,
            u_f_source: None,
            spacer: None,
            psi_g: 0.05,
            psi_g_is_manual: true,
            a_g_m2: 0.7,
            a_f_m2: 0.3,
            l_g_m: 3.6,
            u_w: 1.2,
        };
        let json = serde_json::to_string(&breakdown).expect("serialize");
        assert!(
            !json.contains("spacer"),
            "afwezige spacer moet wegvallen uit de JSON: {json}"
        );
        let parsed: UwBreakdown = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, breakdown);
        assert!(parsed.spacer.is_none());
    }
}
