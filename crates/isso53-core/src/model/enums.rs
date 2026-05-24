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
    EnkellaagsKop,
    EnkellaagsVrijstaand,
    MeerlaagsGeheel,
    MeerlaagsTop,
    MeerlaagsTussen,
    MeerlaagsOnder,
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

/// Gebouwtype voor de winddrukverdelingsfactor f_type (ISSO 53 tabel 4.6).
///
/// Let op: dit is een **andere** indeling dan [`BuildingShape`] (tabel 4.9) en
/// [`GebouwTypePositie`] (tabel 4.8). Tabel 4.6 keyt op de geveltype-/
/// huidgevelconfiguratie van het gebouw.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum GebouwTypeWinddruk {
    /// Eénlaags gebouw met kap.
    EenlaagsMetKap,
    /// Eénlaags gebouw met plat dak.
    EenlaagsMetPlatDak,
    /// Meerlaags gebouw, standaard geveltype.
    MeerlaagsStandaard,
    /// Meerlaags gebouw, volgevel binnengalerij aan één zijde.
    MeerlaagsVolgevelBinnengalerij,
    /// Meerlaags gebouw, dubbele huidgevel met onderbroken tussenruimte.
    MeerlaagsDubbeleHuidOnderbroken,
    /// Meerlaags gebouw, dubbele huidgevel met doorlopende tussenruimte.
    MeerlaagsDubbeleHuidDoorlopend,
}

/// Type onverwarmde aangrenzende ruimte voor de correctiefactor f_k
/// (ISSO 53 tabel 4.2).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum OnverwarmdeRuimte {
    /// Vertrek/groep met 1 externe scheidingsconstructie/buitenwand.
    VertrekEenExtern,
    /// Vertrek/groep met 2 externe scheidingsconstructies zonder buitendeur.
    VertrekTweeExternZonderDeur,
    /// Vertrek/groep met 2 externe scheidingsconstructies met buitendeur.
    VertrekTweeExternMetDeur,
    /// Vertrek/groep met 3 of meer externe scheidingsconstructies.
    VertrekDrieOfMeerExtern,
    /// Kelder zonder ramen/deuren in externe scheidingsconstructie.
    KelderZonderRamenDeuren,
    /// Kelder met ramen/deuren in externe scheidingsconstructie.
    KelderMetRamenDeuren,
    /// Ruimte onder dak met hoog infiltratievoud (pannendak zonder folielaag).
    RuimteOnderDakHoogInfiltratie,
    /// Ruimte onder overig niet-geïsoleerd dak.
    RuimteOnderDakNietGeisoleerd,
    /// Ruimte onder geïsoleerd dak.
    RuimteOnderDakGeisoleerd,
    /// Interne gemeenschappelijke verkeersruimte zonder buitenwanden,
    /// ventilatievoud < 0,5.
    VerkeersruimteInternLaagVentilatie,
    /// Gemeenschappelijke verkeersruimte, vrij geventileerd (A/V > 0,005).
    VerkeersruimteVrijGeventileerd,
    /// Gemeenschappelijke verkeersruimte, overige gevallen.
    VerkeersruimteOverig,
    /// Vloer boven zwak geventileerde kruipruimte
    /// (openingen ≤ 1000 mm²/m²).
    VloerBovenKruipruimteZwak,
    /// Vloer boven matig geventileerde kruipruimte
    /// (1000 < openingen ≤ 1500 mm²/m²).
    VloerBovenKruipruimteMatig,
    /// Vloer boven sterk geventileerde kruipruimte
    /// (openingen > 1500 mm²/m²).
    VloerBovenKruipruimteSterk,
}

/// Bouwfase voor de minimale ventilatie-eisen volgens het Bouwbesluit
/// (ISSO 53 tabel 4.10).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum VentilatieBouwfase {
    /// Nieuwbouw — strengere eisen (dm³/s per persoon).
    Nieuwbouw,
    /// Bestaande bouw — soepelere eisen (dm³/s per persoon).
    Bestaand,
}

/// Verwarmingssysteem voor temperatuur-gelaagdheid Δθ_2 volgens ISSO 53 tabel 2.3.
/// Gebruikt voor formule 4.23 vloer-f_ig berekening.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "camelCase")]
pub enum HeatingSystem {
    /// Lokale verwarming — Δθ_2 = −1 K
    LokaleVerwarming,
    /// Radiatoren/convectoren hoge temperatuur en luchtverwarming — Δθ_2 = −1 K
    #[default]
    RadiatorenConvHtEnLuchtverwarming,
    /// Radiatoren/convectoren lage temperatuur — Δθ_2 = −1 K
    RadiatorenConvLt,
    /// Plafondverwarming — Δθ_2 = 0 K
    Plafondverwarming,
    /// Wandverwarming — Δθ_2 = −1 K
    Wandverwarming,
    /// Plintverwarming — Δθ_2 = −1 K
    Plintverwarming,
    /// Vloerverwarming + HT radiatoren — Δθ_2 = 0 K
    VloerverwarmingPlusHtRadi,
    /// Vloerverwarming + LT radiatoren — Δθ_2 = 0 K
    VloerverwarmingPlusLtRadi,
    /// Vloerverwarming — Δθ_2 = 0 K
    Vloerverwarming,
    /// Vloerverwarming + wandverwarming — Δθ_2 = 0 K
    VloerverwarmingPlusWandverwarming,
    /// Betonkernactivering — Δθ_2 = 0 K
    Betonkernactivering,
    /// Ventilatorgedreven convectorradiatoren — Δθ_2 = 0 K
    VentilatorgedrevenConvRadi,
}
