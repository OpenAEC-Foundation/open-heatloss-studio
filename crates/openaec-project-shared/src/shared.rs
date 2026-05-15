//! `SharedProject` — cross-calc metadata, locatie, gebouwtype.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Project-metadata + locatie + gebouwtype dat alle calcs delen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SharedProject {
    /// Projectnaam.
    pub name: String,

    /// Projectnummer / referentie.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_number: Option<String>,

    /// Adres van het gebouw (vrij format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Postcode — basis voor KNMI-locatie binding (zie F5 + TODO-tools roadmap).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postcode: Option<String>,

    /// Plaatsnaam / locatie-omschrijving.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Opdrachtgever.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,

    /// Berekeningsdatum (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Verantwoordelijk ingenieur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engineer: Option<String>,

    /// Vrije aantekeningen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Gebouwtype (woning / utiliteit + subtype).
    #[serde(default = "default_building_type")]
    pub building_type: BuildingTypeShared,

    /// Bouwjaar (relevant voor f_iso bij TO-juli en infiltratie bij ISSO 51).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction_year: Option<u32>,

    /// Bruto gebruiksoppervlak A_g in m² (gebouwniveau).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gross_floor_area_m2: Option<f64>,

    /// Aantal bouwlagen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_storeys: Option<u32>,
}

impl SharedProject {
    /// Lege shared sectie met enkel naam ingevuld.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            project_number: None,
            address: None,
            postcode: None,
            location: None,
            client: None,
            date: None,
            engineer: None,
            notes: None,
            building_type: default_building_type(),
            construction_year: None,
            gross_floor_area_m2: None,
            num_storeys: None,
        }
    }
}

fn default_building_type() -> BuildingTypeShared {
    BuildingTypeShared::Woning {
        subtype: ResidentialType::Detached,
    }
}

/// Top-level gebouwcategorie. Bepaalt welke berekeningen logisch zijn en
/// hoe defaults worden gekozen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BuildingTypeShared {
    /// Woongebouw met subtype.
    Woning {
        /// ISSO 51 / NTA 8800 woningtype.
        subtype: ResidentialType,
    },
    /// Utiliteitsgebouw met subtype.
    Utiliteit {
        /// NTA 8800 utiliteitstype.
        subtype: UtilityType,
    },
}

/// Woning-subtypes (parallel aan `isso51_core::model::enums::BuildingType`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResidentialType {
    /// Vrijstaande woning.
    Detached,
    /// 2-onder-1-kap.
    SemiDetached,
    /// Tussenwoning.
    Terraced,
    /// Hoekwoning.
    EndOfTerrace,
    /// Portiekwoning.
    Porch,
    /// Galerijwoning.
    Gallery,
    /// Gestapelde woning.
    Stacked,
}

/// Utiliteitsgebouw-subtypes (subset NTA 8800 §2.2 gebruiksfuncties).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UtilityType {
    /// Kantoorgebouw.
    Office,
    /// Onderwijsgebouw.
    Education,
    /// Bijeenkomstgebouw.
    Assembly,
    /// Gezondheidszorg (met of zonder bedgebied).
    Healthcare,
    /// Logies (hotel/pension).
    Lodging,
    /// Sportgebouw.
    Sport,
    /// Winkelgebouw.
    Retail,
    /// Industriegebouw (bedrijfshal/loods).
    Industrial,
    /// Overig — vereist user-input van NTA 8800 functietype apart.
    Other,
}
