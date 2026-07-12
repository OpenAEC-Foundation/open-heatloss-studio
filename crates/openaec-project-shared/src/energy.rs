//! Additief `energy`-invoerblok op [`crate::ProjectV2`] — installatie- en
//! opwek-invoer voor de NTA 8800 / BENG-keten (F2).
//!
//! ## Ontwerp
//!
//! Dit blok is een **stabiel invoer-DTO**, géén rekentype. De veldnamen en
//! eenheden spiegelen John Heikens' `open-energy-studio/src/core/energy/
//! types.ts` (de tweede normlezing), terwijl elke type-enum 1-op-1 aansluit
//! op een opwekker-variant van de betreffende `nta8800-*` service-crate. De
//! F2b-orchestrator vertaalt dit DTO naar de runtime-structs van die crates;
//! per variant staat in de doc-comment welk crate-type het doel is. Zo blijft
//! `openaec-project-shared` ontkoppeld van de calc-internals (geen extra
//! dependency op `nta8800-heating`/`-dhw`/`-pv`/`-automation`) en groeit het
//! invoermodel niet mee met interne herzieningen van die crates.
//!
//! ## Additief & backward-compatible
//!
//! Het blok hangt als `Option<EnergyInput>` op `ProjectV2` met
//! `#[serde(default)]`; bestaande project-JSON's (ISSO 51/53, TO-juli) zonder
//! `energy`-veld deserialiseren ongewijzigd naar `None`. Binnen het blok is
//! alles `Option`/`Vec` met defaults, zodat een half-ingevuld formulier geldig
//! blijft.
//!
//! ## Eenheden-conventies
//!
//! - Rendementen/COP/SCOP: dimensieloos.
//! - Luchtdebieten: m³/h (NTA 8800 §11.2 rekent in m³/h, niet m³/s).
//! - SFP: W/(m³/h) — NTA 8800 tabel 11.23 eenheid. Let op: John's `sfp` staat
//!   in W/(dm³/s); deel door 3,6 bij het overnemen (1 m³/h = 1000/3600 dm³/s,
//!   dus 0,45 W/(dm³/s) ÷ 3,6 = 0,125 W/(m³/h) = het tabel-11.23-forfait).
//! - Hoeken: graden. Azimut 0 = noord, 90 = oost, 180 = zuid, 270 = west.
//!
//! ## Bronregistratie van kentallen (F4c)
//!
//! In de NTA 8800-praktijk komt een prestatiewaarde (SCOP, η_WTW, SFP, …) uit
//! één van drie bronnen: het **norm-forfait** (de default van deze keten), een
//! **gecontroleerde kwaliteitsverklaring** (BCRG-databank) of een
//! **gelijkwaardigheidsverklaring**. De adviseur heeft een *dossierplicht*: per
//! toegepaste waarde moet vastliggen wáár die vandaan komt. Daarom draagt elk
//! deelsysteem een optioneel [`ValueSource`]-veld ([`ValueSourceKind`] +
//! vrije referentie naar het brondocument).
//!
//! **Korrel = deelsysteem, niet losse f64.** De bron hangt bewust op
//! deelsysteem-niveau ([`HeatingInput`], [`DhwInput`], [`DwtwInput`],
//! [`VentilationInput`], [`CoolingInput`], per [`PvInput`]) en niet op elke
//! afzonderlijke f64. Een BCRG-attest of gelijkwaardigheidsverklaring wordt in
//! de praktijk per *apparaat/systeem* afgegeven (het attest van een warmtepomp
//! dekt SCOP + bijbehorende kentallen samen; een douche-WTW-attest dekt η + het
//! douche-aandeel). Per-f64-korrel zou het metadata-oppervlak vervelvoudigen,
//! niet aansluiten op hoe verklaringen worden uitgegeven, en de UI met een
//! bronkiezer op elk getal belasten. Het deelsysteem is de natuurlijke
//! documentgrens.
//!
//! **Puur metadata.** Het bronveld beïnvloedt de berekening NOOIT — het reist
//! mee naar het resultaat ([`crate::beng::ValueSourceReport`]) voor rapportage
//! en dossier, maar geen enkele reken-crate leest het.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Herkomst van de prestatiewaarden van één installatie-deelsysteem
/// (NTA 8800-dossierplicht; zie module-doc, "Bronregistratie").
///
/// Puur metadata: raakt de berekening niet, maar legt vast wáár de kentallen
/// van het deelsysteem vandaan komen zodat de rapportage en het dossier de
/// herkomst kunnen tonen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ValueSource {
    /// Soort bron.
    pub kind: ValueSourceKind,

    /// Vrije referentie naar het brondocument: BCRG-attestnummer,
    /// verklaringskenmerk, meetrapport-ID, … Afwezig/leeg voor forfait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

/// Soort herkomst van een prestatiewaarde (NTA 8800-dossierplicht).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueSourceKind {
    /// Norm-forfait — de default van de keten (geen dossierstuk vereist).
    #[default]
    Forfait,
    /// Gecontroleerde kwaliteitsverklaring uit de BCRG-databank.
    Kwaliteitsverklaring,
    /// Gelijkwaardigheidsverklaring (art. gelijkwaardigheid Bbl).
    Gelijkwaardigheidsverklaring,
    /// Meting / meetrapport.
    Meting,
    /// Overige, niet-gecategoriseerde bron.
    Overig,
}

/// Installatie- en hernieuwbaar-invoer voor de BENG-keten.
///
/// Alle deelsystemen zijn optioneel; een ontbrekend systeem betekent dat de
/// orchestrator het forfait van de norm toepast of het aandeel op nul zet
/// (bv. geen `cooling` ⇒ Q_C;use = 0).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EnergyInput {
    /// Verwarmingssysteem (opwekking + afgifte + distributie + regeling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heating: Option<HeatingInput>,

    /// Warm-tapwater-systeem (DHW).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dhw: Option<DhwInput>,

    /// Ventilatiesysteem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ventilation: Option<VentilationInput>,

    /// Koelsysteem. `None` = geen actieve koeling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooling: Option<CoolingInput>,

    /// PV-velden/-strings. Lege lijst = geen PV.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pv: Vec<PvInput>,

    /// Gebouwautomatisering (BACS). `None` = referentieklasse C.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation: Option<AutomationInput>,
}

// ---------------------------------------------------------------------------
// Verwarming — spiegelt `nta8800-heating` (H.9)
// ---------------------------------------------------------------------------

/// Opwekker-type verwarming.
///
/// Mapt op `nta8800_heating::model::GenerationSystem`. John's `hr107`/`hr_combi`
/// vallen beide onder [`HeatGeneratorType::HrBoiler`] (klasse via
/// [`HeatingInput::hr_class`]); `heat_pump_air`/`heat_pump_ground` onder de
/// twee warmtepomp-varianten (identiek voor de crate, gescheiden voor
/// rapportage); `biomass` ontbreekt in V1 van de crate en is bewust weggelaten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HeatGeneratorType {
    /// HR-ketel (gas). → `GenerationSystem::HRBoiler { class }`.
    HrBoiler,
    /// Lucht/water-warmtepomp. → `GenerationSystem::HeatPump { scop }`.
    HeatPumpAir,
    /// Bodem/water-warmtepomp. → `GenerationSystem::HeatPump { scop }`.
    HeatPumpGround,
    /// Elektrische weerstandsverwarming. → `GenerationSystem::ElectricResistance`.
    ElectricResistance,
    /// Stadsverwarming / warmtenet. → `GenerationSystem::DistrictHeating { factor }`.
    DistrictHeating,
}

/// HR-ketelklasse (deellastrendement op onderwaarde). Mapt op
/// `nta8800_heating::model::HRClass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HrBoilerClass {
    /// HR-100 — deellast ≥ 100 %.
    Hr100,
    /// HR-104 — deellast ≥ 104 %.
    Hr104,
    /// HR-107 — deellast ≥ 107 %.
    Hr107,
}

/// Afgiftesysteem verwarming. Mapt op
/// `nta8800_heating::model::EmissionSystem` (bepaalt η_em + impliciet het
/// afgiftetemperatuur-regime HT/LT/VLT).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HeatEmissionType {
    /// Radiator hoge temperatuur (70-90 °C). → `EmissionSystem::RadiatorHighTemp`.
    RadiatorHighTemp,
    /// Radiator lage temperatuur (~55 °C). → `EmissionSystem::RadiatorLowTemp`.
    RadiatorLowTemp,
    /// Vloerverwarming (~35 °C). → `EmissionSystem::FloorHeating`.
    FloorHeating,
    /// Luchtverwarming. → `EmissionSystem::AirHeating`.
    AirHeating,
    /// Stralingspanelen. → `EmissionSystem::RadiantPanel`.
    RadiantPanel,
}

/// Verwarmingssysteem-invoer (NTA 8800 H.9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HeatingInput {
    /// Opwekker-type.
    pub generator: HeatGeneratorType,

    /// Seizoensgemiddelde COP (SCOP) — alleen voor de warmtepomp-varianten.
    /// Dimensieloos, > 1 typisch. Spiegelt John's `cop`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cop: Option<f64>,

    /// HR-ketelklasse — alleen voor [`HeatGeneratorType::HrBoiler`]. Bepaalt
    /// het forfaitair η_gen (crate-default HR-107 als afwezig).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hr_class: Option<HrBoilerClass>,

    /// Grensvlak-factor (0 < f ≤ 1) — alleen voor
    /// [`HeatGeneratorType::DistrictHeating`]. Verlies tussen gebouwgrens en
    /// afgifte; de primaire-energiefactor van het net zelf zit in H.5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub district_factor: Option<f64>,

    /// Afgiftesysteem. Afwezig ⇒ crate-default (radiator HT).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emission: Option<HeatEmissionType>,

    /// Distributierendement η_H;dist ∈ (0, 1]. Afwezig ⇒ crate-default 0,95
    /// (geïsoleerde leidingen binnen de thermische schil).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution_efficiency: Option<f64>,

    /// Regelfactor f_reg ∈ (0, 1]. Afwezig ⇒ crate-default 0,97
    /// (modulerende regeling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_factor: Option<f64>,

    /// Dekkingsgraad van dit systeem (0..=1). Spiegelt John's
    /// `coverageFraction`; 1,0 = enige opwekker. Multi-opwekker/hybride
    /// splitsing is V2.
    #[serde(default = "default_coverage_fraction")]
    pub coverage_fraction: f64,

    /// Herkomst van de verwarmingskentallen (dossierplicht, zie module-doc).
    /// `None`/forfait = norm-forfait. Puur metadata — raakt de berekening niet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ValueSource>,
}

fn default_coverage_fraction() -> f64 {
    1.0
}

// ---------------------------------------------------------------------------
// Tapwater (DHW) — spiegelt `nta8800-dhw` (H.13)
// ---------------------------------------------------------------------------

/// Opwekker-type warm tapwater. Mapt op
/// `nta8800_dhw::model::DhwGenerationSystem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DhwGeneratorType {
    /// HR-combiketel (gas). → `DhwGenerationSystem::HRCombiBoiler`.
    HrCombiBoiler,
    /// Elektrische boiler (voorraadvat/doorstromer). → `ElectricBoiler`.
    ElectricBoiler,
    /// Tapwater-warmtepomp. → `HeatPumpDhw`.
    HeatPump,
    /// Stadsverwarming / warmtenet. → `DistrictHeating`.
    DistrictHeating,
}

/// Warm-tapwater-invoer (NTA 8800 H.13).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DhwInput {
    /// Opwekker-type.
    pub generator: DhwGeneratorType,

    /// η_W;gen (HR-combi/elektrische boiler/stadsverwarming) of SCOP_W
    /// (warmtepomp). Afwezig ⇒ crate-forfait per type (bv. HR-combi 0,80).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub efficiency: Option<f64>,

    /// Douchewater-warmteterugwinning (DWTW). `None` = geen unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dwtw: Option<DwtwInput>,

    /// Zonneboiler aanwezig. Spiegelt John's `hasSolarBoiler`.
    ///
    /// **V2-scope in de crate** (bijlage W); nu louter invoer, nog niet
    /// verrekend. Documentair opgenomen zodat de UI het kan vastleggen.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_solar_boiler: bool,

    /// Zonneboiler-dekkingsfractie (0..=1). Spiegelt John's
    /// `solarBoilerFraction`. V2-scope, zie [`Self::has_solar_boiler`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solar_boiler_fraction: Option<f64>,

    /// Herkomst van de tapwaterkentallen (dossierplicht, zie module-doc).
    /// `None`/forfait = norm-forfait. Puur metadata — raakt de berekening niet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ValueSource>,
}

/// Douchewater-warmteterugwinning (bijlage U, vereenvoudigd). Mapt op
/// `nta8800_dhw::model::DouchewtwRecovery`.
///
/// Niet langer `Copy` sinds het optionele [`ValueSource`] (met `String`) hier
/// hangt; de mapping leest via `as_ref()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DwtwInput {
    /// Netto thermisch rendement η (0..=1), typisch 0,25-0,50.
    pub efficiency: f64,

    /// Aandeel douche in Q_W;nd (C_W;nd;sh, 0..=1). Afwezig ⇒ crate-default 0,4
    /// (woningbouw).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub douche_aandeel: Option<f64>,

    /// Herkomst van het DWTW-rendement (dossierplicht, zie module-doc).
    /// `None`/forfait = norm-forfait. Puur metadata — raakt de berekening niet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ValueSource>,
}

// ---------------------------------------------------------------------------
// Ventilatie — spiegelt `nta8800-ventilation` (H.11)
// ---------------------------------------------------------------------------

/// Ventilatiesysteem-type conform NTA 8800 §11.1 + bijlage S. Mapt op
/// `nta8800_ventilation::model::VentilationSystem`.
///
/// Let op de NTA 8800-conventie: B = mechanische toevoer, C = mechanische
/// afvoer (omgekeerd aan oudere NEN 1087-bronnen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum VentilationSystemType {
    /// A — natuurlijke toe- en afvoer.
    A,
    /// B — mechanische toevoer, natuurlijke afvoer.
    B,
    /// C — mechanische afvoer, natuurlijke toevoer.
    C,
    /// D — gebalanceerd (mech. toe- én afvoer), WTW via [`VentilationInput::wtw_efficiency`].
    D,
    /// E — decentrale balansventilatie met WTW.
    E,
}

/// Ventilatie-invoer (NTA 8800 H.11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VentilationInput {
    /// Systeemtype A-E.
    pub system: VentilationSystemType,

    /// WTW-rendement η_hr (0..=1). Aanwezigheid activeert WTW bij systeem D
    /// (`VentilationSystem::D { with_wtw: true }`). Spiegelt John's
    /// `heatRecoveryEfficiency`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wtw_efficiency: Option<f64>,

    /// Specifiek ventilatorvermogen f_SFP in **W/(m³/h)** (NTA 8800 tabel
    /// 11.23). Afwezig ⇒ forfait per systeemtype. Spiegelt John's `sfp`
    /// (die in W/(dm³/s) staat — deel door 3,6 bij het overnemen, zie
    /// module-doc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sfp_w_per_m3h: Option<f64>,

    /// 100 %-bypass bij hoge buitentemperatuur aanwezig. V1: documentair.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bypass_enabled: bool,

    /// Mechanisch toevoerdebiet in m³/h. Afwezig ⇒ norm-forfait `q_V;ODA;req`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mechanical_supply_m3_per_h: Option<f64>,

    /// Mechanisch afvoerdebiet in m³/h. Afwezig ⇒ norm-forfait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mechanical_exhaust_m3_per_h: Option<f64>,

    /// Infiltratie-luchtstroom q_V;lea in m³/h. Afwezig ⇒ afgeleid uit
    /// luchtdichtheidsklasse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub infiltration_m3_per_h: Option<f64>,

    /// Herkomst van de ventilatiekentallen (η_WTW, SFP; dossierplicht, zie
    /// module-doc). `None`/forfait = norm-forfait. Puur metadata — raakt de
    /// berekening niet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ValueSource>,
}

// ---------------------------------------------------------------------------
// Koeling — spiegelt `nta8800-cooling` (H.10)
// ---------------------------------------------------------------------------

/// Koudeopwekker-type. Mapt op
/// `nta8800_cooling::model::CoolingSystem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoolingGeneratorType {
    /// Compressiekoeling (split/VRV/omkeerbare WP). → `CompressionCooling { scop_cooling }`.
    Compression,
    /// Absorptiekoeling (warmte-gedreven). → `AbsorptionCooling { cop }`.
    Absorption,
    /// Vrije koeling (ventilatief/bodem). → `FreeCooling { factor }`.
    FreeCooling,
}

/// Koel-invoer (NTA 8800 H.10). Aanwezigheid van dit blok = actieve koeling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoolingInput {
    /// Opwekker-type.
    pub generator: CoolingGeneratorType,

    /// SEER / SCOP_cooling voor compressiekoeling. Spiegelt John's `eer`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seer: Option<f64>,

    /// COP voor absorptiekoeling (typisch 0,6-1,3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cop: Option<f64>,

    /// Benuttingsfractie (0..=1) voor vrije koeling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_cooling_fraction: Option<f64>,

    /// Herkomst van de koelkentallen (SEER/COP; dossierplicht, zie module-doc).
    /// `None`/forfait = norm-forfait. Puur metadata — raakt de berekening niet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ValueSource>,
}

// ---------------------------------------------------------------------------
// PV — spiegelt `nta8800-pv` (H.16, §16)
// ---------------------------------------------------------------------------

/// Eén PV-veld/-string. Mapt op `nta8800_pv::PvSystem`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PvInput {
    /// Optionele identificatie voor rapportage/UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Optionele naam.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Piekvermogen in kWp (> 0). Spiegelt John's `peakPower`.
    pub peak_power_kwp: f64,

    /// Azimut γ in graden (0 = noord, 90 = oost, 180 = zuid, 270 = west).
    /// `nta8800-pv` valideert -180..=180; converteer indien nodig.
    pub azimuth_degrees: f64,

    /// Hellingshoek β in graden (0 = horizontaal, 90 = verticaal). Spiegelt
    /// John's `tilt`.
    pub tilt_degrees: f64,

    /// Totale systeem-efficiëntie (0 < η ≤ 1). Afwezig ⇒ crate-default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_efficiency: Option<f64>,

    /// Inverter-efficiëntie (0 < η ≤ 1). Afwezig ⇒ crate-default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverter_efficiency: Option<f64>,

    /// Schaduwfactor (0..=1, 1 = geen schaduw). Afwezig ⇒ 1,0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_factor: Option<f64>,

    /// Herkomst van de PV-kentallen (systeem-/inverterrendement; dossierplicht,
    /// zie module-doc). Per PV-veld apart. `None`/forfait = norm-forfait. Puur
    /// metadata — raakt de berekening niet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ValueSource>,
}

// ---------------------------------------------------------------------------
// Automatisering — spiegelt `nta8800-automation` (H.15 / EN 15232)
// ---------------------------------------------------------------------------

/// BACS-klasse (NEN-EN 15232). Mapt op
/// `nta8800_automation::model::BacsClass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum BacsClassInput {
    /// A — high energy performance BACS.
    A,
    /// B — advanced BACS.
    B,
    /// C — standard BACS (referentie).
    C,
    /// D — non energy efficient BACS.
    D,
}

/// Gebouwautomatisering-invoer (NTA 8800 H.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AutomationInput {
    /// BACS-klasse.
    pub bacs_class: BacsClassInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_energy_round_trip() {
        let e = EnergyInput::default();
        let json = serde_json::to_string(&e).unwrap();
        // Alle velden None/leeg ⇒ compacte `{}`.
        assert_eq!(json, "{}");
        let back: EnergyInput = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    /// Volledig gevulde fixture met een bron op heating (kwaliteitsverklaring),
    /// dwtw (meting) en pv (gelijkwaardigheidsverklaring).
    fn filled_fixture() -> EnergyInput {
        EnergyInput {
            heating: Some(HeatingInput {
                generator: HeatGeneratorType::HeatPumpGround,
                cop: Some(4.2),
                hr_class: None,
                district_factor: None,
                emission: Some(HeatEmissionType::FloorHeating),
                distribution_efficiency: Some(0.95),
                control_factor: Some(0.97),
                coverage_fraction: 1.0,
                source: Some(ValueSource {
                    kind: ValueSourceKind::Kwaliteitsverklaring,
                    reference: Some("BCRG-20231234".into()),
                }),
            }),
            dhw: Some(DhwInput {
                generator: DhwGeneratorType::HeatPump,
                efficiency: Some(2.8),
                dwtw: Some(DwtwInput {
                    efficiency: 0.45,
                    douche_aandeel: Some(0.4),
                    source: Some(ValueSource {
                        kind: ValueSourceKind::Meting,
                        reference: None,
                    }),
                }),
                has_solar_boiler: false,
                solar_boiler_fraction: None,
                source: None,
            }),
            ventilation: Some(VentilationInput {
                system: VentilationSystemType::D,
                wtw_efficiency: Some(0.85),
                sfp_w_per_m3h: Some(0.125),
                bypass_enabled: true,
                mechanical_supply_m3_per_h: Some(150.0),
                mechanical_exhaust_m3_per_h: Some(150.0),
                infiltration_m3_per_h: Some(25.0),
                source: None,
            }),
            cooling: Some(CoolingInput {
                generator: CoolingGeneratorType::Compression,
                seer: Some(4.0),
                cop: None,
                free_cooling_fraction: None,
                source: None,
            }),
            pv: vec![PvInput {
                id: Some("pv-dak-zuid".into()),
                name: Some("Dakveld zuid".into()),
                peak_power_kwp: 5.5,
                azimuth_degrees: 180.0,
                tilt_degrees: 35.0,
                system_efficiency: Some(0.85),
                inverter_efficiency: Some(0.96),
                shadow_factor: None,
                source: Some(ValueSource {
                    kind: ValueSourceKind::Gelijkwaardigheidsverklaring,
                    reference: Some("GWV-2024-07".into()),
                }),
            }],
            automation: Some(AutomationInput {
                bacs_class: BacsClassInput::A,
            }),
        }
    }

    #[test]
    fn filled_energy_round_trip() {
        let e = filled_fixture();
        let json = serde_json::to_string(&e).unwrap();
        let back: EnergyInput = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn coverage_fraction_defaults_to_one() {
        // Heating zonder coverage_fraction in JSON ⇒ default 1,0.
        let json = r#"{"generator":"hr_boiler","hr_class":"hr107"}"#;
        let h: HeatingInput = serde_json::from_str(json).unwrap();
        assert!((h.coverage_fraction - 1.0).abs() < 1e-12);
        assert_eq!(h.generator, HeatGeneratorType::HrBoiler);
        assert_eq!(h.hr_class, Some(HrBoilerClass::Hr107));
    }

    #[test]
    fn ventilation_system_serializes_uppercase() {
        let json = serde_json::to_string(&VentilationSystemType::D).unwrap();
        assert_eq!(json, "\"D\"");
    }

    #[test]
    fn value_source_kind_defaults_to_forfait() {
        assert_eq!(ValueSourceKind::default(), ValueSourceKind::Forfait);
    }

    #[test]
    fn value_source_serializes_snake_case() {
        let s = ValueSource {
            kind: ValueSourceKind::Kwaliteitsverklaring,
            reference: Some("BCRG-1".into()),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#"{"kind":"kwaliteitsverklaring","reference":"BCRG-1"}"#);
        let back: ValueSource = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn source_round_trips_on_every_subsystem() {
        // De `filled_energy_round_trip`-fixture zet al een bron op heating, dwtw
        // en pv; hier verifiëren we dat de round-trip die bronnen bewaart.
        let e = filled_fixture();
        let json = serde_json::to_string(&e).unwrap();
        let back: EnergyInput = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
        assert_eq!(
            back.heating.unwrap().source.unwrap().kind,
            ValueSourceKind::Kwaliteitsverklaring
        );
        assert_eq!(
            back.dhw.unwrap().dwtw.unwrap().source.unwrap().kind,
            ValueSourceKind::Meting
        );
    }

    #[test]
    fn json_without_source_is_backward_compatible() {
        // Bestaande project-JSON zonder `source` deserialiseert ongewijzigd:
        // elk `source`-veld valt terug op `None` (dossier-metadata is optioneel).
        let json = r#"{
            "heating": {"generator":"hr_boiler","hr_class":"hr107"},
            "dhw": {"generator":"hr_combi_boiler","dwtw":{"efficiency":0.4}},
            "ventilation": {"system":"D","wtw_efficiency":0.85},
            "cooling": {"generator":"compression","seer":4.0},
            "pv": [{"peak_power_kwp":3.5,"azimuth_degrees":180.0,"tilt_degrees":35.0}]
        }"#;
        let e: EnergyInput = serde_json::from_str(json).unwrap();
        assert!(e.heating.unwrap().source.is_none());
        let dhw = e.dhw.unwrap();
        assert!(dhw.source.is_none());
        assert!(dhw.dwtw.unwrap().source.is_none());
        assert!(e.ventilation.unwrap().source.is_none());
        assert!(e.cooling.unwrap().source.is_none());
        assert!(e.pv[0].source.is_none());
    }

    #[test]
    fn source_none_is_omitted_from_json() {
        // `skip_serializing_if` houdt de goldens byte-compatibel: een subsysteem
        // zonder bron serialiseert geen `source`-sleutel.
        let h = HeatingInput {
            generator: HeatGeneratorType::HrBoiler,
            cop: None,
            hr_class: Some(HrBoilerClass::Hr107),
            district_factor: None,
            emission: None,
            distribution_efficiency: None,
            control_factor: None,
            coverage_fraction: 1.0,
            source: None,
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(!json.contains("source"), "source zou weggelaten moeten zijn: {json}");
    }

    #[test]
    fn generator_types_serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&HeatGeneratorType::HeatPumpAir).unwrap(),
            "\"heat_pump_air\""
        );
        assert_eq!(
            serde_json::to_string(&DhwGeneratorType::HrCombiBoiler).unwrap(),
            "\"hr_combi_boiler\""
        );
    }
}
