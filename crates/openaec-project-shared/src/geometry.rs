//! Gedeelde geometrie — spaces, constructions, openings.
//!
//! Canoniek geometrie-model dat door alle calcs gebruikt wordt. Per-calc
//! view-mappers in [`crate::view`] (en in toekomst in `nta8800-*` crates)
//! transformeren dit naar calc-specifieke structs (ISSO 51 `Room` /
//! NTA 8800 `Rekenzone` etc.).
//!
//! **Schaal-keuze:** afmetingen in mm, oppervlakten in m², U-waardes in
//! W/(m²·K), conform de rest van het project (ISSO 51 convention).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Container voor alle geometrie van het project.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SharedGeometry {
    /// Alle verblijfsruimten / kamers in het project.
    #[serde(default)]
    pub spaces: Vec<Space>,
}

/// Een verblijfsruimte (ISSO 51 kamer / NTA 8800 EFR-element).
///
/// `id` moet uniek zijn binnen het project; calc-mappers gebruiken dit als
/// stabiele referentie.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Space {
    /// Unieke identifier binnen het project (bv. "210A.04").
    pub id: String,
    /// Mens-leesbare naam (bv. "Woonkamer").
    pub name: String,
    /// Functie/gebruikstype (vrij-veld, mapt naar calc-specifieke enums).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    /// Vloeroppervlak in m².
    pub floor_area_m2: f64,
    /// Hoogte in m (binnenwerks plafond).
    pub height_m: f64,
    /// Constructies (wanden/vloeren/daken/ramen/deuren) die dit space begrenzen.
    #[serde(default)]
    pub constructions: Vec<Construction>,
    /// Setpoint binnenluchttemperatuur in °C (winter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theta_i_winter_c: Option<f64>,
    /// Setpoint binnenluchttemperatuur in °C (zomer / cooling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theta_i_summer_c: Option<f64>,
}

/// Een constructie (wand/vloer/dak/raam/deur) die een Space begrenst.
///
/// Een Construction hangt aan exact één Space. Voor gedeelde tussenmuren
/// tussen twee Spaces: model als twee Constructions, één per kant, met
/// `BoundaryKind::AdjacentRoom` en `adjacent_space_id` cross-referentie.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Construction {
    /// Unieke ID binnen Space.
    pub id: String,
    /// Korte omschrijving (bv. "Noordgevel woonkamer").
    pub description: String,
    /// Type vlak (vertical wall / floor / ceiling / etc.).
    pub kind: ConstructionKind,
    /// Aan welk type grensvlak (buitenlucht / grond / onverwarmde ruimte / …).
    pub boundary: BoundaryKind,
    /// Bruto oppervlakte in m² (inclusief ramen en deuren).
    pub area_m2: f64,
    /// U-waarde in W/(m²·K). Voor transparante constructies (kozijnen) is
    /// dit de samengestelde U_window inclusief frame.
    pub u_value: f64,
    /// Optionele oriëntatie azimut in graden (0 = N, 90 = O, 180 = Z, 270 = W).
    /// Verplicht voor TO-juli (zoninstraling per oriëntatie).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation_deg: Option<f64>,
    /// Hellingshoek in graden (0 = horizontaal vloer, 90 = verticale wand,
    /// 180 = plat dak naar boven).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slope_deg: Option<f64>,
    /// Ramen + deuren in deze constructie. Hun oppervlakten worden afgetrokken
    /// van het opake deel bij calc.
    #[serde(default)]
    pub openings: Vec<Opening>,
    /// Optionele lagenopbouw (mm + λ) — bron-of-truth voor U-waarde als
    /// `u_value` niet pre-computed is.
    #[serde(default)]
    pub layers: Vec<ConstructionLayer>,
    /// Cross-ref naar adjacent Space.id als `boundary = AdjacentRoom`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjacent_space_id: Option<String>,
    /// Thermal bridge psi-waarde in W/(m·K) (lineaire bridge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub psi_thermal_bridge: Option<f64>,
}

/// Type constructie qua oriëntatie / functie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionKind {
    /// Verticale wand (binnen of buitengevel).
    Wall,
    /// Begane grond vloer / verdiepingsvloer (richting onder).
    Floor,
    /// Plafond / dak (richting boven).
    Ceiling,
    /// Hellend dakvlak.
    Roof,
}

/// Aan welk type grensvlak de constructie zit (ISSO 51 §2.5 / NTA 8800 H.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryKind {
    /// Direct aan buitenlucht.
    Exterior,
    /// Onverwarmde ruimte (zolder, kruipruimte, garage).
    UnheatedSpace,
    /// Andere verwarmde ruimte binnen dezelfde woning/zone.
    AdjacentRoom,
    /// Naastgelegen woning / aangrenzend gebouw.
    AdjacentBuilding,
    /// Grond (vloer op grond, kelderwand).
    Ground,
    /// Open water (woonboot use case).
    OpenWater,
}

/// Een opening (raam of deur) in een Construction.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Opening {
    /// Unieke ID binnen Construction.
    pub id: String,
    /// Type opening.
    pub kind: OpeningKind,
    /// Oppervlakte in m² (frame + glas).
    pub area_m2: f64,
    /// U-waarde van de opening (frame + glas combinatie).
    pub u_value: f64,
    /// g-waarde (zonenergie-doorlatingsfactor) voor ramen. None voor deuren.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub g_value: Option<f64>,
    /// Frame-aandeel (0..1) voor ramen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_fraction: Option<f64>,
}

/// Soort opening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpeningKind {
    /// Raam (transparant, met g-waarde).
    Window,
    /// Deur (opaak of glas, geen g-waarde standaard).
    Door,
}

/// Een laag in een opake constructie. Bij ontbreken van `lambda_w_per_mk`
/// (luchtspouw) wordt `r_si_se` direct gebruikt.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionLayer {
    /// Materiaal-naam.
    pub material: String,
    /// Dikte in mm.
    pub thickness_mm: f64,
    /// Warmtegeleidingscoëfficiënt λ in W/(m·K). 0 = gebruik `r_m2k_per_w`.
    #[serde(default)]
    pub lambda_w_per_mk: f64,
    /// Pre-computed warmteweerstand in m²·K/W (luchtspouw / gegeven Rc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r_m2k_per_w: Option<f64>,
}
