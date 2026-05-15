//! Enumerations used throughout the ISSO 51 calculation model.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type of boundary for a construction element.
/// Determines which heat loss formula is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryType {
    /// Direct to outside air (ISSO 51 §2.5.1)
    Exterior,
    /// To an unheated space adjacent to the dwelling (ISSO 51 §2.5.2)
    UnheatedSpace,
    /// To another heated room within the same dwelling (ISSO 51 §2.5.3)
    AdjacentRoom,
    /// To a neighboring dwelling/building (ISSO 51 §2.5.4)
    AdjacentBuilding,
    /// To the ground (ISSO 51 §2.5.5)
    Ground,
    /// To open water (canal/river/lake — woonboot use case).
    ///
    /// Not a norm category — this is an engineering choice for constructions
    /// that sit directly against open water. The design temperature comes
    /// from `DesignConditions.theta_water` (default 5 °C, override per project).
    /// Reports must include a footnote when this variant is used.
    Water,
}

/// Room function determines the design indoor temperature (θ_i).
/// Values from ISSO 51 Table 2.11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoomFunction {
    /// Living room / verblijfsruimte (20°C)
    LivingRoom,
    /// Kitchen / keuken (20°C)
    Kitchen,
    /// Bedroom / slaapkamer (20°C)
    Bedroom,
    /// Bathroom / badkamer (22°C)
    Bathroom,
    /// Toilet / toiletruimte (15°C)
    Toilet,
    /// Hallway / entree/gang (15°C)
    Hallway,
    /// Landing / overloop (15°C)
    Landing,
    /// Storage room / bergruimte (5°C if frost protection needed)
    Storage,
    /// Attic / zolder used as living space (20°C)
    Attic,
    /// Custom temperature
    Custom,
}

impl RoomFunction {
    /// Returns the design indoor temperature θ_i in °C.
    /// ISSO 51 Table 2.11.
    pub fn design_temperature(&self) -> f64 {
        match self {
            RoomFunction::LivingRoom => 20.0,
            RoomFunction::Kitchen => 20.0,
            RoomFunction::Bedroom => 20.0,
            RoomFunction::Bathroom => 22.0,
            RoomFunction::Toilet => 15.0,
            RoomFunction::Hallway => 15.0,
            RoomFunction::Landing => 15.0,
            RoomFunction::Storage => 5.0,
            RoomFunction::Attic => 20.0,
            RoomFunction::Custom => 20.0, // default, should be overridden
        }
    }
}

/// Type of heating system installed.
/// Affects Δθ values (Table 2.12) and system losses (§2.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HeatingSystem {
    /// Gas heater, wall-mounted heater
    LocalGasHeater,
    /// IR panels wall-mounted
    IrPanelWall,
    /// IR panels ceiling-mounted
    IrPanelCeiling,
    /// High-temperature radiators/convectors (medium temp > 50°C).
    ///
    /// Default variant — statistisch meest voorkomende installatie in
    /// Nederlandse woningen. Gebruikt wanneer een Room-JSON zonder
    /// `heating_system` veld wordt gedeserialiseerd (third-party clients).
    #[default]
    RadiatorHt,
    /// Low-temperature radiators/convectors (medium temp ≤ 50°C)
    RadiatorLt,
    /// Ceiling heating
    CeilingHeating,
    /// Wall heating
    WallHeating,
    /// Baseboard/plinth heating
    PlinthHeating,
    /// Floor heating + HT radiators
    FloorHeatingWithRadiatorHt,
    /// Floor heating + LT radiators
    FloorHeatingWithRadiatorLt,
    /// Floor heating as main system (floor temp ≥ 27°C)
    FloorHeatingMainHigh,
    /// Floor heating as main system (floor temp < 27°C)
    FloorHeatingMainLow,
    /// Floor + wall heating combined
    FloorAndWallHeating,
    /// Fan-driven convectors/radiators (NEN-EN 16430)
    FanConvector,
}

/// Ventilation system type (A through E).
/// ISSO 51 §2.5.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VentilationSystemType {
    /// System A: Natural supply and natural exhaust
    SystemA,
    /// System B: Mechanical supply and natural exhaust
    SystemB,
    /// System C: Natural supply and mechanical exhaust
    SystemC,
    /// System D: Mechanical supply and mechanical exhaust (balanced)
    SystemD,
    /// System E: Combination of systems within one dwelling
    SystemE,
}

/// Frost protection method for heat recovery units.
/// Determines the supply temperature θ_t (ISSO 51 Table 2.14, erratum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FrostProtectionType {
    /// Unknown type of frost protection
    Unknown,
    /// Reduced fan speed and/or temporary imbalance (central)
    CentralReducedSpeed,
    /// Enthalpy exchanger (central, min 70% thermal efficiency)
    CentralEnthalpy,
    /// Pre-heating (central)
    CentralPreheating,
    /// Reduced fan speed and/or temporary imbalance (decentral)
    DecentralReducedSpeed,
    /// Enthalpy exchanger (decentral, min 70% thermal efficiency)
    DecentralEnthalpy,
    /// Pre-heating (decentral)
    DecentralPreheating,
    /// Electric pre-heating without heat recovery
    ElectricPreheating,
}

/// Security class for heat loss to neighbors.
/// ISSO 51 Table 2.16.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecurityClass {
    /// Class A: no heat loss to neighbors assumed (c_z = 0)
    A,
    /// Class B: moderate risk (c_z = 0.5)
    B,
    /// Class C: high risk, neighbors may not heat (c_z = 1.0)
    C,
}

impl SecurityClass {
    /// Returns the security factor c_z.
    /// ISSO 51 Table 2.16.
    pub fn factor(&self) -> f64 {
        match self {
            SecurityClass::A => 0.0,
            SecurityClass::B => 0.5,
            SecurityClass::C => 1.0,
        }
    }
}

/// Material type for thermal bridge correction.
/// Determines whether ΔU_TB = 0.1 is applied per the forfaitaire method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    /// Masonry/stone-like materials (steenachtig)
    Masonry,
    /// Non-masonry materials like glass, doors (niet-steenachtig)
    NonMasonry,
}

/// Infiltration calculation method.
/// Determines how the specific infiltration rate is applied.
///
/// **Legacy varianten** (`PerExteriorArea`, `PerFloorArea`) blijven beschikbaar
/// voor backward-compatibiliteit met bestaande project-JSONs, maar zijn niet
/// (meer) norm-conform met ISSO 51:2023 / NTA 8800 (Tabel 4.3 is geschrapt in
/// de 2023-publicatie). Voor nieuwe projecten: gebruik `VabiCompat` of
/// `Nta8800Strict`.
///
/// Zie ook `crates/isso51-core/src/tables/infiltration.rs` voor de
/// onderliggende tabel-functies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InfiltrationMethod {
    /// **Legacy** — per m² exterior construction area (ISSO 51:2017 Tabel 4.3).
    /// `q_i = qi_spec_ext × ΣA_exterior`. Default blijft deze variant ivm
    /// backward-compat met bestaande fixtures; nieuwe rekenketen gaat via
    /// `VabiCompat` / `Nta8800Strict`.
    #[default]
    PerExteriorArea,
    /// **Legacy** — per m² floor area (eigen tabel, niet norm-conform).
    /// `q_i = qi_spec × A_floor`
    PerFloorArea,
    /// **Nieuw** — Vabi-compatibele hybride methode: ISSO 51:2023 Tabel 2.8
    /// (`qi_spec` per gebouwtype) gecombineerd met NTA 8800 power-law
    /// (`n_lea = 0.67`) en design-Δp = 3.14 Pa (Vabi-fit). Aanbevolen voor
    /// projecten die met Vabi-DR resultaten moeten matchen.
    VabiCompat,
    /// **Nieuw** — strikt NTA 8800 Tabel 11.14 + 11.13 keten:
    /// `q_v10;lea;ref = f_type × f_y × q_v10;spec;reken`, gevolgd door power-law
    /// (`n_lea = 0.67`). Geen Vabi-fit; design-Δp volgens NTA 8800 (4 Pa-domein).
    Nta8800Strict,
    /// **Nieuw** — formule (11.85) met `building.qv10` als directe input.
    ///
    /// Voor projecten waar de luchtdoorlatendheid daadwerkelijk gemeten is
    /// (blower-door test). Slaat Tabel 2.8 / `f_type` / `f_y` over — de
    /// gemeten waarde verdisconteert deze al. Keten reduceert tot:
    /// `qi = qv10 × (Δp_design / 10)^n_lea × f_inf`. Replicateert
    /// Vabi-DR-keten exact (Δp = 3.14 Pa) op fixtures waar `qv10` gemeten is.
    MeasuredQv10,
}

/// Building type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    /// Detached house
    Detached,
    /// Semi-detached house
    SemiDetached,
    /// Terraced/row house (tussenwoning)
    Terraced,
    /// End-of-terrace (hoekwoning)
    EndOfTerrace,
    /// Apartment in a porch building (portiekwoning)
    Porch,
    /// Gallery apartment (galerijwoning)
    Gallery,
    /// Stacked housing (gestapeld)
    Stacked,
}

/// Position of a construction element relative to adjacent buildings.
/// Used for floor/ceiling fb calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerticalPosition {
    /// Floor (below the room)
    Floor,
    /// Ceiling (above the room)
    Ceiling,
    /// Vertical wall
    Wall,
}

/// Woningtype-classificatie volgens ISSO 51:2023 Tabel 2.8 — bepaalt de
/// `q_i,spec` (specifieke infiltratie per m² gebruiksoppervlak) op basis van
/// woningvorm + dakvorm.
///
/// **Onderscheiden van** de bestaande [`BuildingType`]: `BuildingType` is een
/// fijnmaziger gebouwclassificatie (Detached / SemiDetached / Terraced / etc.)
/// die elders in de code gebruikt wordt voor warmwordingsfactoren (Tabel 2.13)
/// en winddruk. `DwellingClass` is uitsluitend de drie-rijen-keying van
/// Tabel 2.8 (en optioneel als input voor mapping naar `ConstructionVariant`
/// in NTA 8800 Tabel 11.14).
///
/// Bron: ISSO 51:2023 p.41 Tabel 2.8 (waardes via NEN 8088-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DwellingClass {
    /// Eengezinswoning met kap (hellend dak) → `q_i,spec = 1.0 dm³/(s·m² Ag)`
    EengezinswoningMetKap,
    /// Eengezinswoning met plat dak → `q_i,spec = 0.7 dm³/(s·m² Ag)`
    EengezinswoningPlatdak,
    /// Etage / flat / portiekwoning → `q_i,spec = 0.5 dm³/(s·m² Ag)`
    EtageFlatOfPortiek,
}

/// Uitvoeringsvariant volgens NTA 8800 Tabel 11.14 — bepaalt de correctie-
/// factor `f_type` op de luchtdoorlatendheid op gebouwniveau.
///
/// Bron: NTA 8800:2024 p.487–488 Tabel 11.14.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionVariant {
    /// Tussenwoning / tussen-appartement → `f_type = 1.0`
    Tussen,
    /// Hoek-/kopwoning of kop-appartement → `f_type = 1.2`
    Kop,
    /// Vrijstaande woning → `f_type = 1.4`
    Vrijstaand,
}

/// Methode voor aggregatie van transmissieverliezen op gebouwniveau.
///
/// ISSO 51:2023 §3.5.1 zegt letterlijk: `Φ_basis = Φ_T,ie + Φ_T,iae + Φ_T,ig + Φ_i − Φ_gain`,
/// inclusief verlies via onverwarmde ruimtes (`Φ_T,iae`). De markt-tool Vabi
/// telt `Φ_T,iae` op gebouwniveau echter als 0 (rapporteert het wel per kamer,
/// neemt het niet op in `Φ_basis_gebouw`). Voor projecten die met Vabi-uitvoer
/// vergeleken moeten worden geeft strikte normuitvoering ~17% hogere
/// `connection_capacity`.
///
/// Daarom configurable per project, default = `VabiCompat` (markt-conventie).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AggregationMethod {
    /// Φ_T,iae wordt NIET opgenomen in Φ_basis_gebouw (Vabi-conventie, markt-default).
    /// Wijkt af van ISSO 51:2023 §3.5.1 letterlijk, maar geeft Vabi-compatible getallen.
    VabiCompat,
    /// Φ_T,iae WEL in Φ_basis_gebouw conform ISSO 51:2023 §3.5.1 letterlijk.
    /// Geeft ~17% hogere connection_capacity dan VabiCompat. Voor strikte audits.
    NormStrict,
}

impl Default for AggregationMethod {
    fn default() -> Self {
        Self::VabiCompat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_function_temperatures() {
        assert_eq!(RoomFunction::LivingRoom.design_temperature(), 20.0);
        assert_eq!(RoomFunction::Bathroom.design_temperature(), 22.0);
        assert_eq!(RoomFunction::Hallway.design_temperature(), 15.0);
        assert_eq!(RoomFunction::Toilet.design_temperature(), 15.0);
        assert_eq!(RoomFunction::Storage.design_temperature(), 5.0);
    }

    #[test]
    fn test_security_class_factors() {
        assert_eq!(SecurityClass::A.factor(), 0.0);
        assert_eq!(SecurityClass::B.factor(), 0.5);
        assert_eq!(SecurityClass::C.factor(), 1.0);
    }

    #[test]
    fn test_infiltration_method_default_remains_per_exterior_area() {
        // Backward-compat: bestaande fixture-JSONs zonder `infiltration_method`
        // moeten naar PerExteriorArea blijven defaulten. Wijzigen van deze
        // default = breaking change voor alle bestaande projecten.
        assert_eq!(
            InfiltrationMethod::default(),
            InfiltrationMethod::PerExteriorArea
        );
    }

    #[test]
    fn test_infiltration_method_new_variants_exist() {
        // Sanity check dat de twee nieuwe varianten geconstrueerd kunnen worden.
        let m1 = InfiltrationMethod::VabiCompat;
        let m2 = InfiltrationMethod::Nta8800Strict;
        assert_ne!(m1, m2);
        assert_ne!(m1, InfiltrationMethod::PerExteriorArea);
    }

    #[test]
    fn test_dwelling_class_distinct() {
        // Drie afzonderlijke rijen uit Tabel 2.8.
        let a = DwellingClass::EengezinswoningMetKap;
        let b = DwellingClass::EengezinswoningPlatdak;
        let c = DwellingClass::EtageFlatOfPortiek;
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn test_construction_variant_distinct() {
        // Drie afzonderlijke uitvoeringsvarianten uit Tabel 11.14.
        let t = ConstructionVariant::Tussen;
        let k = ConstructionVariant::Kop;
        let v = ConstructionVariant::Vrijstaand;
        assert_ne!(t, k);
        assert_ne!(k, v);
        assert_ne!(t, v);
    }
}
