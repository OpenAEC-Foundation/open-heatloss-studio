//! Serialiseerbaar invoer-model voor de nta8800-core façade.
//!
//! Volgt het isso51-core patroon: één `Project`-struct die met serde uit
//! JSON komt en met schemars een JSON-schema genereert. Installatie-configs
//! hergebruiken de serde-typen van de sub-crates (`GenerationSystem`,
//! `CoolingSystem`, `PvSystem`, …) zodat er geen dubbele enum-definities
//! ontstaan en het schema automatisch meegroeit met de sub-crates.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_cooling::{CoolingDistribution, CoolingEmission, CoolingSystem};
use nta8800_dhw::model::{DhwEmission, DhwGenerationSystem, DouchewtwRecovery};
use nta8800_heating::model::{EmissionSystem, GenerationSystem};
use nta8800_lighting::model::LightingSystem;
use nta8800_model::zoning::UsageFunction;
use nta8800_pv::PvSystem;

/// Forfaitaire vrije verdiepingshoogte (m) om volume uit A_g af te leiden
/// wanneer `volume_m3` ontbreekt.
pub const DEFAULT_STOREY_HEIGHT_M: f64 = 2.7;

/// Top-level project-invoer voor een volledige NTA 8800 berekening.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Project {
    /// Project-metadata.
    #[serde(default)]
    pub info: ProjectInfo,
    /// Gebouw-eigenschappen (functie, oppervlak, volume, massa).
    pub building: Building,
    /// Thermische schil: opake constructies met eventueel ramen erin.
    pub envelope: Vec<EnvelopeElement>,
    /// Ventilatie-systeem + luchtdebieten.
    #[serde(default)]
    pub ventilation: VentilationInput,
    /// Verwarmings-installatie (verplicht — elk gebouw heeft verwarming).
    pub heating: HeatingInput,
    /// Koelings-installatie. `None` = geen actieve koeling (Q_C;use = 0).
    #[serde(default)]
    pub cooling: Option<CoolingInput>,
    /// Warm-tapwater-installatie (verplicht voor woningen; utiliteit krijgt
    /// forfaitaire behoefte per gebruiksfunctie).
    pub dhw: DhwInput,
    /// Verlichting — alleen relevant voor utiliteitsfuncties. Voor
    /// woonfuncties negeert de keten dit veld (H.14 rekent alleen utiliteit).
    #[serde(default)]
    pub lighting: Option<LightingSystem>,
    /// PV-systemen op/aan het gebouw. Lege lijst of `None` = geen PV.
    #[serde(default)]
    pub pv: Option<Vec<PvSystem>>,
    /// Setpoints + gedrag.
    #[serde(default)]
    pub conditions: Conditions,
}

/// Project-metadata (niet reken-relevant).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ProjectInfo {
    /// Projectnaam voor rapportage.
    #[serde(default)]
    pub name: String,
    /// Vrij omschrijvingsveld.
    #[serde(default)]
    pub description: Option<String>,
}

/// Gebouw-eigenschappen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Building {
    /// NTA 8800 gebruiksfunctie — bepaalt forfaits (interne warmtelast,
    /// tapwaterbehoefte, ventilatie-eis) en de label-indeling.
    pub usage_function: UsageFunction,
    /// Gebruiksoppervlakte A_g in m² (> 0).
    pub floor_area_m2: f64,
    /// Bruto volume in m³. Ontbreekt het, dan `A_g × 2,7`.
    #[serde(default)]
    pub volume_m3: Option<f64>,
    /// Bouwjaar — voedt (toekomstige) infiltratie-forfaits; V1 metadata.
    #[serde(default)]
    pub construction_year: Option<u32>,
    /// Thermische-massa-klasse van de constructie.
    #[serde(default)]
    pub thermal_mass: ThermalMassClass,
}

/// Vereenvoudigde thermische-massa-keuze; mapt op
/// `nta8800_demand::ThermalMassInput` presets (tabel 7.10-7.12).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThermalMassClass {
    /// Licht (HSB/SFB, gesloten plafond) — D_m ≈ 55 kJ/(m²·K).
    #[default]
    Light,
    /// Zwaar massief (beton, open plafond) — D_m ≈ 450 kJ/(m²·K).
    Heavy,
}

/// Grens-conditie van een schil-element.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Boundary {
    /// Grenst aan buitenlucht.
    Exterior,
    /// Grenst aan de grond (begane-grondvloer, kelderwand).
    Ground,
    /// Grenst aan een onverwarmde ruimte (b-factor default 0,5).
    UnheatedSpace {
        /// Identificatie van de aangrenzende onverwarmde ruimte.
        #[serde(default)]
        id: Option<String>,
    },
}

/// Opaak schil-element (gevel, dak, vloer) met eventueel ramen erin.
///
/// Het opake transmissie-oppervlak is `area_m2` minus de som van de
/// venster-oppervlakken; de vensters gaan afzonderlijk de transmissie- en
/// zoninstralings-keten in.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnvelopeElement {
    /// Unieke identificatie.
    pub id: String,
    /// Omschrijving voor rapportage.
    #[serde(default)]
    pub description: String,
    /// Bruto oppervlak in m² (inclusief ramen).
    pub area_m2: f64,
    /// Warmtedoorgangscoëfficiënt U van het opake deel in W/(m²·K).
    pub u_value: f64,
    /// Grens-conditie.
    pub boundary: Boundary,
    /// Oriëntatie in graden (0 = noord, 90 = oost, 180 = zuid, 270 = west).
    /// `None` voor horizontale vlakken (dak/vloer).
    #[serde(default)]
    pub orientation_deg: Option<f64>,
    /// Helling in graden (0 = horizontaal, 90 = verticaal). Default 90.
    #[serde(default)]
    pub tilt_deg: Option<f64>,
    /// Ramen in dit element.
    #[serde(default)]
    pub windows: Vec<WindowElement>,
}

/// Raam in een schil-element.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WindowElement {
    /// Unieke identificatie.
    pub id: String,
    /// Bruto vensteroppervlak (kozijn + glas) in m².
    pub area_m2: f64,
    /// Samengestelde U-waarde (glas + kozijn) in W/(m²·K).
    pub u_value: f64,
    /// Zonnewarmtedoorlatingsfactor g (0..=1). Default 0,6.
    #[serde(default = "default_g_value")]
    pub g_value: f64,
    /// Kozijnfractie (0..=1). Default 0,25.
    #[serde(default = "default_frame_fraction")]
    pub frame_fraction: f64,
}

fn default_g_value() -> f64 {
    0.6
}
fn default_frame_fraction() -> f64 {
    0.25
}

/// Ventilatie-systeemkeuze conform NTA 8800 systeem-indeling.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VentilationSystemInput {
    /// Systeem A — natuurlijke toe- en afvoer.
    Natural,
    /// Systeem B — mechanische toevoer, natuurlijke afvoer.
    MechanicalSupply,
    /// Systeem C — natuurlijke toevoer, mechanische afvoer.
    #[default]
    MechanicalExhaust,
    /// Systeem D — gebalanceerde mechanische ventilatie, optioneel met WTW.
    Balanced {
        /// WTW-rendement (0..1). `None` = geen warmteterugwinning.
        #[serde(default)]
        wtw_efficiency: Option<f64>,
    },
}

/// Ventilatie-invoer: systeemtype + optionele luchtdebieten.
///
/// Ontbrekende debieten vallen terug op het NTA 8800 §11.2.2 norm-forfait
/// `q_V;ODA;req` (functie van gebruiksfunctie + A_g) — een traceerbare
/// norm-bepaling, geen handmatige schatting.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct VentilationInput {
    /// Systeemkeuze. Default: systeem C (NL-praktijk bestaande bouw).
    #[serde(default)]
    pub system: VentilationSystemInput,
    /// Mechanisch toevoerdebiet in m³/h.
    #[serde(default)]
    pub mechanical_supply_m3_per_h: Option<f64>,
    /// Mechanisch afvoerdebiet in m³/h.
    #[serde(default)]
    pub mechanical_exhaust_m3_per_h: Option<f64>,
    /// Infiltratie-/natuurlijk toevoerdebiet in m³/h.
    #[serde(default)]
    pub infiltration_m3_per_h: Option<f64>,
}

/// Verwarmings-installatie.
///
/// Hergebruikt de serde-typen van `nta8800-heating`; distributie-rendement en
/// regelfactor zijn platte getallen met norm-conforme defaults.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeatingInput {
    /// Afgifte-systeem (radiator HT/LT, vloerverwarming, lucht, stralings).
    pub emission: EmissionSystem,
    /// Opwekker (HR-ketel, warmtepomp, elektrisch, stadswarmte).
    pub generation: GenerationSystem,
    /// Distributie-rendement η_H;dist ∈ (0, 1]. Default 0,95 (geïsoleerde
    /// leidingen binnen de thermische schil).
    #[serde(default = "default_heating_distribution")]
    pub distribution_efficiency: f64,
    /// Regelfactor f_reg ∈ (0, 1]. Default 0,97 (modulerende regeling).
    #[serde(default = "default_control_factor")]
    pub control_factor: f64,
}

fn default_heating_distribution() -> f64 {
    0.95
}
fn default_control_factor() -> f64 {
    0.97
}

/// Koelings-installatie (None = geen actieve koeling).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoolingInput {
    /// Koelopwekker (compressie/absorptie/vrije koeling) met COP.
    pub system: CoolingSystem,
    /// Distributie-rendement koeling.
    #[serde(default = "CoolingDistribution::default_insulated")]
    pub distribution: CoolingDistribution,
    /// Afgifte + regelfactor koeling.
    #[serde(default = "default_cooling_emission")]
    pub emission: CoolingEmission,
}

fn default_cooling_emission() -> CoolingEmission {
    CoolingEmission {
        efficiency: 0.95,
        regulation_factor: 0.95,
    }
}

/// Warm-tapwater-installatie.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DhwInput {
    /// Opwekker (HR-combi, elektrische boiler, tapwater-warmtepomp,
    /// stadswarmte).
    pub generation: DhwGenerationSystem,
    /// Afgifte-systeem. Default: woningbouw-forfait.
    #[serde(default = "default_dhw_emission")]
    pub emission: DhwEmission,
    /// Distributie-rendement η_W;dis ∈ (0, 1]. Default 1,0 (individueel
    /// toestel zonder circulatieleiding).
    #[serde(default = "default_dhw_distribution")]
    pub distribution_efficiency: f64,
    /// Optionele douche-warmteterugwinning.
    #[serde(default)]
    pub shower_heat_recovery: Option<DouchewtwRecovery>,
    /// Expliciete netto jaarbehoefte Q_W;nd in kWh — overschrijft het
    /// forfait. VERPLICHT voor gebruiksfuncties zonder tabel-13.1-forfait
    /// (industriefunctie); optioneel elders (bv. gemeten verbruik).
    #[serde(default)]
    pub annual_demand_kwh: Option<f64>,
}

fn default_dhw_emission() -> DhwEmission {
    DhwEmission::WoningDefault
}
fn default_dhw_distribution() -> f64 {
    1.0
}

/// Setpoints en gedrags-parameters.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Conditions {
    /// Verwarmings-setpoint in °C (constant, alle maanden). Default 20.
    #[serde(default = "default_heating_setpoint")]
    pub heating_setpoint_c: f64,
    /// Koel-setpoint in °C (constant, alle maanden). Default 24.
    #[serde(default = "default_cooling_setpoint")]
    pub cooling_setpoint_c: f64,
    /// Schaduw-factor F_sh ∈ [0, 1] voor zoninstraling. 1,0 = geen schaduw.
    #[serde(default = "default_shading_factor")]
    pub shading_factor: f64,
}

impl Default for Conditions {
    fn default() -> Self {
        Self {
            heating_setpoint_c: default_heating_setpoint(),
            cooling_setpoint_c: default_cooling_setpoint(),
            shading_factor: default_shading_factor(),
        }
    }
}

fn default_heating_setpoint() -> f64 {
    20.0
}
fn default_cooling_setpoint() -> f64 {
    24.0
}
fn default_shading_factor() -> f64 {
    1.0
}

impl Project {
    /// Façade-niveau invoer-validatie (vóór de orchestratie).
    ///
    /// # Errors
    ///
    /// [`crate::CoreError::InvalidInput`] met een mens-leesbare omschrijving.
    pub fn validate(&self) -> crate::CoreResult<()> {
        if !self.building.floor_area_m2.is_finite() || self.building.floor_area_m2 <= 0.0 {
            return Err(crate::CoreError::InvalidInput(format!(
                "building.floor_area_m2 moet > 0 zijn (was {})",
                self.building.floor_area_m2
            )));
        }
        if let Some(v) = self.building.volume_m3 {
            if !v.is_finite() || v <= 0.0 {
                return Err(crate::CoreError::InvalidInput(format!(
                    "building.volume_m3 moet > 0 zijn (was {v})"
                )));
            }
        }
        if self.envelope.is_empty() {
            return Err(crate::CoreError::InvalidInput(
                "envelope mag niet leeg zijn — minimaal één schil-element".into(),
            ));
        }
        for el in &self.envelope {
            if !el.area_m2.is_finite() || el.area_m2 <= 0.0 {
                return Err(crate::CoreError::InvalidInput(format!(
                    "envelope[{}].area_m2 moet > 0 zijn (was {})",
                    el.id, el.area_m2
                )));
            }
            if !el.u_value.is_finite() || el.u_value <= 0.0 {
                return Err(crate::CoreError::InvalidInput(format!(
                    "envelope[{}].u_value moet > 0 zijn (was {})",
                    el.id, el.u_value
                )));
            }
            let window_area: f64 = el.windows.iter().map(|w| w.area_m2).sum();
            if window_area > el.area_m2 {
                return Err(crate::CoreError::InvalidInput(format!(
                    "envelope[{}]: venster-oppervlak ({window_area} m²) > element-oppervlak ({} m²)",
                    el.id, el.area_m2
                )));
            }
            for w in &el.windows {
                if !w.area_m2.is_finite() || w.area_m2 <= 0.0 {
                    return Err(crate::CoreError::InvalidInput(format!(
                        "window[{}].area_m2 moet > 0 zijn (was {})",
                        w.id, w.area_m2
                    )));
                }
                if !(0.0..=1.0).contains(&w.g_value) {
                    return Err(crate::CoreError::InvalidInput(format!(
                        "window[{}].g_value moet in [0, 1] liggen (was {})",
                        w.id, w.g_value
                    )));
                }
            }
        }
        let c = &self.conditions;
        if c.cooling_setpoint_c <= c.heating_setpoint_c {
            return Err(crate::CoreError::InvalidInput(format!(
                "cooling_setpoint_c ({}) moet > heating_setpoint_c ({}) zijn",
                c.cooling_setpoint_c, c.heating_setpoint_c
            )));
        }
        if !(0.0..=1.0).contains(&c.shading_factor) {
            return Err(crate::CoreError::InvalidInput(format!(
                "shading_factor moet in [0, 1] liggen (was {})",
                c.shading_factor
            )));
        }
        Ok(())
    }

    /// Effectief bruto volume: expliciet veld of `A_g × 2,7`.
    #[must_use]
    pub fn effective_volume_m3(&self) -> f64 {
        self.building
            .volume_m3
            .unwrap_or(self.building.floor_area_m2 * DEFAULT_STOREY_HEIGHT_M)
    }
}
