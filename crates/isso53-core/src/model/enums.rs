//! Enums for ISSO 53 domain model.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Gebruiksfunctie volgens Bouwbesluit (ISSO 53 tabel 2.2).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum GebruiksFunctie {
    Kantoor,
    Onderwijs,
    Gezondheidszorg,
    Bijeenkomst,
    Logies,
    Sport,
    Winkel,
    Cel,
    Industrie,
}

/// Ruimtetype binnen een gebruiksfunctie (ISSO 53 tabel 2.2).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum RuimteType {
    Verblijfsruimte,
    Verblijfsgebied,
    Badruimte,
    Toiletruimte,
    Verkeersruimte,
    TechnischeRuimte,
    Bergruimte,
    OnbenoemdeRuimte,
    Stallingsruimte,
    Garage,
    // Domeinspecifiek
    Kantoorruimte,
    Receptie,
    Lesruimte,
    Collegezaal,
    Werkplaats,
    Bureauruimte,
    Patientenkamer,
    Operatiekamer,
    Onderzoekruimte,
    Eetruimte,
    Restaurant,
    Kantine,
    Vergaderruimte,
    Hotelkamer,
    Sportzaal,
    Verkoopruimte,
    Supermarkt,
    Warenhuis,
}

/// Thermal boundary type for construction elements.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum BoundaryType {
    /// To exterior (θ_e).
    Exterior,
    /// To adjacent heated room.
    AdjacentRoom,
    /// To adjacent building (neighbor).
    AdjacentBuilding,
    /// To ground.
    Ground,
    /// To unheated space.
    Unheated,
    /// To water.
    Water,
}

/// Vertical position of construction element.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum VerticalPosition {
    Wall,
    Floor,
    Ceiling,
}

/// Material type for thermal bridge calculation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum MaterialType {
    Masonry,
    NonMasonry,
}

/// Gebouwvorm voor infiltratie-berekening (ISSO 53 tabel 4.9).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum BuildingShape {
    EenLaagMetKap,
    EenLaagMetHalfPlatDak,
    EenLaagMetPlatDak,
    Meerlaags,
}

/// Thermische massa van het gebouw (ISSO 53 tabel 2.4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ThermalMass {
    /// c_eff = 15 Wh/(m³·K)
    Licht,
    /// c_eff = 50 Wh/(m³·K)
    Gemiddeld,
    /// c_eff = 75 Wh/(m³·K)
    Zwaar,
}

/// Berekeningsmethode ISSO 53.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum CalculationMethod {
    /// Hoofdstuk 3 — schilmethode (voorontwerp).
    Shell,
    /// Hoofdstuk 4 — per vertrek (definitief ontwerp).
    PerRoom,
    /// Hoofdstuk 5.1 — individueel aansluitvermogen.
    SourceIndividual,
    /// Hoofdstuk 5.2 — collectief aansluitvermogen.
    SourceCollective,
}

/// Ventilatiesysteemtype (ISSO 53 tabel 4.7).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum VentilationSystemType {
    /// Natuurlijke toe- en afvoer.
    SystemA,
    /// Mechanische toevoer + natuurlijke afvoer.
    SystemB,
    /// Natuurlijke toevoer + mechanische afvoer.
    SystemC,
    /// Gebalanceerde mechanische ventilatie.
    SystemD,
    /// Zone-mix met lokale WTW + CO₂-sturing.
    SystemE,
}

/// Gebouwtype-positie voor infiltratie (ISSO 53 tabel 4.8).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum GebouwTypePositie {
    EnkellaagsTussen,
    EnkellagsKop,
    EnkellagsVrijstaand,
    MeerlagsGeheel,
    MeerlagsTop,
    MeerlaagsTussen,
    MeerlagsOnder,
}

/// Infiltratie-input methode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum InfiltrationInput {
    /// q_v10,kar is bekend — gebruik tabel 4.5.
    KnownQv10,
    /// q_v10,kar onbekend — gebruik formule 4.31.
    UnknownQv10,
}