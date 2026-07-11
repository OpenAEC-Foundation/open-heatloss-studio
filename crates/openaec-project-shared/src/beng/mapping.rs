//! Mapping-laag: `energy`-invoer-DTO ([`crate::energy`]) → runtime-structs van
//! de `nta8800-*` service-crates.
//!
//! De [`crate::energy::EnergyInput`]-DTO is een stabiel invoermodel; de
//! service-crates (`nta8800-heating`/`-dhw`/`-ventilation`/`-cooling`/`-pv`/
//! `-automation`) hebben elk hun eigen runtime-typen. Deze module is de enige
//! plek waar die twee werelden elkaar raken. **Alle defaults-bij-`None` horen
//! hier** (HR-107 als HR-klasse afwezig, η_dist 0,95, f_reg 0,97, SFP-forfait
//! tabel 11.23 = 0,125 W/(m³/h), DWTW-douche-aandeel 0,4, warmtepomp-SCOP-
//! forfaits): zo blijft [`crate::beng::compute_beng`] vrij van
//! magische getallen en staat elke aanname op één traceerbare plek.
//!
//! De keten-volgorde en carrier-mapping-logica volgen de referentie-
//! orchestrator van Maarten Vroegindeweij
//! (`origin/claude/nta8800-core:crates/nta8800-core/src/orchestrator.rs`);
//! zijn invoermodel is bewust **niet** overgenomen (dat concurreert met
//! [`crate::ProjectV2`]).

use nta8800_automation::{AutomationConfig, BacsClass};
use nta8800_cooling::CoolingSystem;
use nta8800_dhw::model::{DhwGenerationSystem, DouchewtwRecovery};
use nta8800_ep::EnergyCarrier as EpCarrier;
use nta8800_heating::model::{
    ControlFactor, DistributionSystem, EmissionSystem, GenerationSystem, HRClass,
};
use nta8800_model::zoning::UsageFunction;
use nta8800_pv::{PvError, PvSystem};
use nta8800_ventilation::model::{AirFlow, VentilationSystem, WtwSpecification};

use crate::energy::{
    AutomationInput, BacsClassInput, CoolingGeneratorType, CoolingInput, DhwGeneratorType, DhwInput,
    HeatEmissionType, HeatGeneratorType, HeatingInput, HrBoilerClass, PvInput, VentilationInput,
    VentilationSystemType,
};

// ---------------------------------------------------------------------------
// Defaults-bij-`None` (norm-forfaits + gedocumenteerde engineering-waarden)
// ---------------------------------------------------------------------------

/// Distributierendement η_H;dist bij afwezige invoer (geïsoleerde leidingen
/// binnen de thermische schil). Zelfde crate-default als
/// [`DistributionSystem::default_insulated`].
pub const DEFAULT_DISTRIBUTION_EFFICIENCY: f64 = 0.95;

/// Regelfactor f_reg bij afwezige invoer (modulerende regeling). Consistent met
/// de [`crate::energy::HeatingInput::control_factor`]-documentatie.
pub const DEFAULT_CONTROL_FACTOR: f64 = 0.97;

/// Forfaitair specifiek ventilatorvermogen f_SFP in W/(m³/h) — NTA 8800
/// tabel 11.23, moderne DC-unit. Zelfde waarde als de TO-juli-keten.
pub const DEFAULT_SFP_W_PER_M3H: f64 = 0.125;

/// Aandeel douche in Q_W;nd (C_W;nd;sh) bij afwezige invoer — NTA 8800 §13.5.3
/// woningbouw-forfait.
pub const DEFAULT_DOUCHE_AANDEEL: f64 = 0.4;

/// Forfaitaire grensvlak-factor voor stadsverwarming bij afwezige invoer.
/// `< 1` reflecteert het verlies tussen gebouwgrens en afgifte; de primaire-
/// energiefactor van het net zelf zit in H.5.
pub const DEFAULT_DISTRICT_FACTOR: f64 = 0.9;

/// **F3-stub** — forfaitaire SCOP voor een lucht/water-warmtepomp verwarming
/// wanneer geen COP is opgegeven. NTA 8800 kent geen blanco-SCOP; deze waarde
/// is een gedocumenteerde engineering-aanname en moet in F3 vervangen worden
/// door bijlage Q of een kwaliteitsverklaring.
pub const DEFAULT_HEAT_PUMP_SCOP_AIR: f64 = 3.5;

/// **F3-stub** — forfaitaire SCOP voor een bodem/water-warmtepomp verwarming
/// (hoger dan lucht door stabielere brontemperatuur). Zie
/// [`DEFAULT_HEAT_PUMP_SCOP_AIR`].
pub const DEFAULT_HEAT_PUMP_SCOP_GROUND: f64 = 4.5;

/// **F3-stub** — forfaitaire SCOP_W voor een tapwater-warmtepomp wanneer geen
/// waarde is opgegeven (tapwater op 60-65 °C → lager dan CV). Zie bijlage Q/W.
pub const DEFAULT_DHW_HEAT_PUMP_SCOP: f64 = 2.5;

/// Opslagverlies-correctiefactor voor een elektrische boiler bij afwezige
/// invoer (NTA 8800 §13.8 V1-forfait).
pub const DEFAULT_ELECTRIC_BOILER_STORAGE_FACTOR: f64 = 0.90;

/// **F3-stub** — forfaitaire SEER voor compressiekoeling wanneer geen waarde
/// is opgegeven.
pub const DEFAULT_COOLING_SEER: f64 = 3.5;

/// **F3-stub** — forfaitaire COP voor absorptiekoeling.
pub const DEFAULT_ABSORPTION_COP: f64 = 0.7;

/// **F3-stub** — forfaitaire benuttingsfractie voor vrije koeling.
pub const DEFAULT_FREE_COOLING_FRACTION: f64 = 0.3;

/// Systeem-efficiëntie van een PV-veld bij afwezige invoer.
pub const DEFAULT_PV_SYSTEM_EFFICIENCY: f64 = 0.85;

/// Inverter-efficiëntie van een PV-veld bij afwezige invoer.
pub const DEFAULT_PV_INVERTER_EFFICIENCY: f64 = 0.96;

// ---------------------------------------------------------------------------
// Verwarming — [`HeatingInput`] → `nta8800-heating`
// ---------------------------------------------------------------------------

/// De vier keten-componenten die [`nta8800_heating::calculate_heating`] verwacht.
#[derive(Debug, Clone)]
pub struct HeatingMapping {
    /// Opwekker (HR-ketel / warmtepomp / weerstand / stadsverwarming).
    pub generation: GenerationSystem,
    /// Afgiftesysteem (bepaalt η_em).
    pub emission: EmissionSystem,
    /// Distributiesysteem (η_dist).
    pub distribution: DistributionSystem,
    /// Regelfactor f_reg.
    pub control: ControlFactor,
}

/// Vertaal een [`HeatingInput`] naar de `nta8800-heating`-runtime-structs.
///
/// Defaults-bij-`None`: HR-klasse → HR-107; η_dist → 0,95; f_reg → 0,97;
/// afgifte → radiator HT; warmtepomp-SCOP → [`DEFAULT_HEAT_PUMP_SCOP_AIR`]/
/// [`DEFAULT_HEAT_PUMP_SCOP_GROUND`]; stadswarmte-factor →
/// [`DEFAULT_DISTRICT_FACTOR`].
///
/// De `η_dist`/`f_reg`-velden worden als struct-literal gezet (geen validatie
/// hier); [`nta8800_heating::calculate_heating`] valideert ze en geeft een
/// [`nta8800_heating::HeatingError`] bij een waarde buiten (0, 1].
#[must_use]
pub fn map_heating(input: &HeatingInput) -> HeatingMapping {
    let generation = match input.generator {
        HeatGeneratorType::HrBoiler => GenerationSystem::HRBoiler {
            class: map_hr_class(input.hr_class),
        },
        HeatGeneratorType::HeatPumpAir => GenerationSystem::HeatPump {
            scop: input.cop.unwrap_or(DEFAULT_HEAT_PUMP_SCOP_AIR),
        },
        HeatGeneratorType::HeatPumpGround => GenerationSystem::HeatPump {
            scop: input.cop.unwrap_or(DEFAULT_HEAT_PUMP_SCOP_GROUND),
        },
        HeatGeneratorType::ElectricResistance => GenerationSystem::ElectricResistance,
        HeatGeneratorType::DistrictHeating => GenerationSystem::DistrictHeating {
            factor: input.district_factor.unwrap_or(DEFAULT_DISTRICT_FACTOR),
        },
    };

    let emission = match input.emission {
        Some(HeatEmissionType::RadiatorHighTemp) | None => EmissionSystem::RadiatorHighTemp,
        Some(HeatEmissionType::RadiatorLowTemp) => EmissionSystem::RadiatorLowTemp,
        Some(HeatEmissionType::FloorHeating) => EmissionSystem::FloorHeating,
        Some(HeatEmissionType::AirHeating) => EmissionSystem::AirHeating,
        Some(HeatEmissionType::RadiantPanel) => EmissionSystem::RadiantPanel,
    };

    HeatingMapping {
        generation,
        emission,
        distribution: DistributionSystem {
            efficiency: input
                .distribution_efficiency
                .unwrap_or(DEFAULT_DISTRIBUTION_EFFICIENCY),
        },
        control: ControlFactor {
            factor: input.control_factor.unwrap_or(DEFAULT_CONTROL_FACTOR),
        },
    }
}

/// HR-klasse-mapping, HR-107 bij afwezigheid (meest voorkomend in nieuwbouw).
#[must_use]
fn map_hr_class(class: Option<HrBoilerClass>) -> HRClass {
    match class {
        Some(HrBoilerClass::Hr100) => HRClass::HR100,
        Some(HrBoilerClass::Hr104) => HRClass::HR104,
        Some(HrBoilerClass::Hr107) | None => HRClass::HR107,
    }
}

// ---------------------------------------------------------------------------
// Tapwater — [`DhwInput`] → `nta8800-dhw`
// ---------------------------------------------------------------------------

/// Vertaal een [`DhwInput`] naar het `nta8800-dhw`-opwekkersysteem.
///
/// Defaults-bij-`None`: elektrische boiler opslagfactor →
/// [`DEFAULT_ELECTRIC_BOILER_STORAGE_FACTOR`]; tapwater-warmtepomp-SCOP →
/// [`DEFAULT_DHW_HEAT_PUMP_SCOP`]; stadswarmte-factor →
/// [`DEFAULT_DISTRICT_FACTOR`]; HR-combi-η → crate-forfait (0,80).
///
/// Emissie/distributie/vraag (usage-afhankelijk) worden **niet** hier bepaald
/// maar in [`crate::beng::compute_beng`], omdat ze de gebruiksfunctie nodig
/// hebben.
#[must_use]
pub fn map_dhw_generation(input: &DhwInput) -> DhwGenerationSystem {
    match input.generator {
        DhwGeneratorType::HrCombiBoiler => DhwGenerationSystem::HRCombiBoiler,
        DhwGeneratorType::ElectricBoiler => DhwGenerationSystem::ElectricBoiler {
            storage_loss_factor: input
                .efficiency
                .unwrap_or(DEFAULT_ELECTRIC_BOILER_STORAGE_FACTOR),
        },
        DhwGeneratorType::HeatPump => DhwGenerationSystem::HeatPumpDhw {
            scop_dhw: input.efficiency.unwrap_or(DEFAULT_DHW_HEAT_PUMP_SCOP),
        },
        DhwGeneratorType::DistrictHeating => DhwGenerationSystem::DistrictHeating {
            factor: input.efficiency.unwrap_or(DEFAULT_DISTRICT_FACTOR),
        },
    }
}

/// Vertaal het optionele DWTW-blok naar [`DouchewtwRecovery`]. `None` = geen
/// unit. Douche-aandeel → [`DEFAULT_DOUCHE_AANDEEL`] bij afwezigheid.
#[must_use]
pub fn map_dwtw(input: &DhwInput) -> Option<DouchewtwRecovery> {
    input.dwtw.map(|d| {
        DouchewtwRecovery::with_aandeel(
            d.efficiency,
            d.douche_aandeel.unwrap_or(DEFAULT_DOUCHE_AANDEEL),
        )
    })
}

// ---------------------------------------------------------------------------
// Ventilatie — [`VentilationInput`] → `nta8800-ventilation`
// ---------------------------------------------------------------------------

/// De ventilatie-runtime-invoer voor [`nta8800_ventilation::calculate_ventilation`].
#[derive(Debug, Clone)]
pub struct VentilationMapping {
    /// Systeemtype A-E.
    pub system: VentilationSystem,
    /// Luchtstromen (m³/h), met §11.2.2-forfait ingevuld waar debieten ontbreken.
    pub flow: AirFlow,
    /// WTW-specificatie (alleen D/E met rendement).
    pub wtw: Option<WtwSpecification>,
}

/// Vertaal een [`VentilationInput`] naar de `nta8800-ventilation`-runtime-typen.
///
/// Ontbrekende luchtdebieten worden ingevuld met het NTA 8800 §11.2.2-forfait
/// `q_V;ODA;req` ([`q_v_oda_req_m3_per_h`]) — dezelfde beslislogica als de
/// TO-juli-keten: systeem A → natuurlijke toevoer; B/D/E → mechanisch
/// toevoerdebiet; C → mechanisch afvoerdebiet. Zo levert de ventilator-
/// hulpenergie-berekening ook bij forfaitaire invoer een non-zero waarde.
///
/// SFP → [`DEFAULT_SFP_W_PER_M3H`] bij afwezigheid; WTW alleen actief voor
/// D/E met een opgegeven rendement.
#[must_use]
pub fn map_ventilation(
    input: &VentilationInput,
    usage: UsageFunction,
    a_g_m2: f64,
) -> VentilationMapping {
    let with_wtw = input.wtw_efficiency.is_some();
    let system = match input.system {
        VentilationSystemType::A => VentilationSystem::A,
        VentilationSystemType::B => VentilationSystem::B,
        VentilationSystemType::C => VentilationSystem::C,
        VentilationSystemType::D => VentilationSystem::D { with_wtw },
        VentilationSystemType::E => VentilationSystem::E,
    };

    let wtw = match input.system {
        VentilationSystemType::D | VentilationSystemType::E => {
            input.wtw_efficiency.map(|eff| WtwSpecification {
                efficiency: eff,
                fan_sfp: input.sfp_w_per_m3h.unwrap_or(DEFAULT_SFP_W_PER_M3H),
                bypass_enabled: input.bypass_enabled,
            })
        }
        _ => None,
    };

    let mut flow = AirFlow {
        mechanical_supply: input.mechanical_supply_m3_per_h.unwrap_or(0.0),
        mechanical_exhaust: input.mechanical_exhaust_m3_per_h.unwrap_or(0.0),
        infiltration: input.infiltration_m3_per_h.unwrap_or(0.0),
    };

    let q_oda_req = q_v_oda_req_m3_per_h(usage, a_g_m2);
    match input.system {
        VentilationSystemType::A => {
            if input.infiltration_m3_per_h.is_none() {
                flow.infiltration = q_oda_req;
            }
        }
        VentilationSystemType::B | VentilationSystemType::D | VentilationSystemType::E => {
            if input.mechanical_supply_m3_per_h.is_none() {
                flow.mechanical_supply = q_oda_req;
            }
        }
        VentilationSystemType::C => {
            if input.mechanical_exhaust_m3_per_h.is_none() {
                flow.mechanical_exhaust = q_oda_req;
            }
        }
    }

    VentilationMapping { system, flow, wtw }
}

// ---------------------------------------------------------------------------
// Koeling — [`CoolingInput`] → `nta8800-cooling`
// ---------------------------------------------------------------------------

/// Vertaal een [`CoolingInput`] naar een [`CoolingSystem`]. Aanwezigheid van
/// het blok = actieve koeling. Forfaits bij afwezige kentallen: zie
/// [`DEFAULT_COOLING_SEER`]/[`DEFAULT_ABSORPTION_COP`]/
/// [`DEFAULT_FREE_COOLING_FRACTION`].
#[must_use]
pub fn map_cooling(input: &CoolingInput) -> CoolingSystem {
    match input.generator {
        CoolingGeneratorType::Compression => CoolingSystem::CompressionCooling {
            scop_cooling: input.seer.unwrap_or(DEFAULT_COOLING_SEER),
        },
        CoolingGeneratorType::Absorption => CoolingSystem::AbsorptionCooling {
            cop: input.cop.unwrap_or(DEFAULT_ABSORPTION_COP),
        },
        CoolingGeneratorType::FreeCooling => CoolingSystem::FreeCooling {
            factor: input
                .free_cooling_fraction
                .unwrap_or(DEFAULT_FREE_COOLING_FRACTION),
        },
    }
}

// ---------------------------------------------------------------------------
// PV — [`PvInput`] → `nta8800-pv`
// ---------------------------------------------------------------------------

/// Vertaal de PV-velden naar [`PvSystem`]'s.
///
/// **Azimut-conversie:** de DTO gebruikt 0-360° (0 = noord, kloksgewijs), maar
/// `nta8800-pv` valideert `-180..=180`. West (270°) → −90°. Efficiënties →
/// [`DEFAULT_PV_SYSTEM_EFFICIENCY`]/[`DEFAULT_PV_INVERTER_EFFICIENCY`];
/// schaduw → 1,0 (geen schaduw) bij afwezigheid.
///
/// # Errors
///
/// [`PvError`] als een veld (piekvermogen, tilt, geconverteerd azimut,
/// efficiëntie) buiten het `nta8800-pv`-geldige bereik ligt.
pub fn map_pv(pv: &[PvInput]) -> Result<Vec<PvSystem>, PvError> {
    pv.iter()
        .map(|p| {
            let azimuth = normalize_azimuth_to_pm180(p.azimuth_degrees);
            PvSystem::with_shadow(
                p.peak_power_kwp,
                p.tilt_degrees,
                azimuth,
                p.system_efficiency.unwrap_or(DEFAULT_PV_SYSTEM_EFFICIENCY),
                p.inverter_efficiency
                    .unwrap_or(DEFAULT_PV_INVERTER_EFFICIENCY),
                p.shadow_factor.unwrap_or(1.0),
            )
        })
        .collect()
}

/// Converteer een 0-360°-azimut (0 = noord) naar de `nta8800-pv`-conventie
/// `-180..=180` (0 = noord, negatief = west-kant). 270° (west) → −90°.
#[must_use]
fn normalize_azimuth_to_pm180(azimuth_0_360: f64) -> f64 {
    let a = azimuth_0_360.rem_euclid(360.0);
    if a > 180.0 {
        a - 360.0
    } else {
        a
    }
}

// ---------------------------------------------------------------------------
// Automatisering — [`AutomationInput`] → `nta8800-automation`
// ---------------------------------------------------------------------------

/// Vertaal een [`AutomationInput`] naar een uniforme [`AutomationConfig`]
/// (dezelfde BACS-klasse voor alle diensten — de DTO differentieert niet per
/// dienst; dat is V2).
#[must_use]
pub fn map_automation(input: &AutomationInput) -> AutomationConfig {
    AutomationConfig::uniform(map_bacs_class(input.bacs_class))
}

/// BACS-klasse-mapping.
#[must_use]
fn map_bacs_class(class: BacsClassInput) -> BacsClass {
    match class {
        BacsClassInput::A => BacsClass::A,
        BacsClassInput::B => BacsClass::B,
        BacsClassInput::C => BacsClass::C,
        BacsClassInput::D => BacsClass::D,
    }
}

// ---------------------------------------------------------------------------
// Energiedrager-mappers (service-carrier → EP-carrier)
// ---------------------------------------------------------------------------
//
// De service-crates hebben elk hun eigen `EnergyCarrier`-enum; de EP-crate
// (bijlage Z/AB) heeft een bredere set. Deze mappers projecteren op de
// EP-dragers. Logica overgenomen van Maarten Vroegindeweij's orchestrator
// (`origin/claude/nta8800-core`), inclusief de stadskoude→stadswarmte-keuze
// (EP kent geen aparte stadskoude-drager).

/// Verwarmings-carrier → EP-carrier.
#[must_use]
pub fn heating_carrier_to_ep(c: nta8800_heating::model::EnergyCarrier) -> EpCarrier {
    use nta8800_heating::model::EnergyCarrier as H;
    match c {
        H::Gas => EpCarrier::Aardgas,
        H::Electricity => EpCarrier::Elektriciteit,
        H::DistrictHeat => EpCarrier::Stadswarmte,
    }
}

/// Tapwater-carrier → EP-carrier.
#[must_use]
pub fn dhw_carrier_to_ep(c: nta8800_dhw::model::EnergyCarrier) -> EpCarrier {
    use nta8800_dhw::model::EnergyCarrier as D;
    match c {
        D::Gas => EpCarrier::Aardgas,
        D::Electricity => EpCarrier::Elektriciteit,
        D::DistrictHeat => EpCarrier::Stadswarmte,
    }
}

/// Koel-carrier → EP-carrier. Stadskoude mapt op stadswarmte (EP kent geen
/// aparte koudenet-drager — dichtstbijzijnde primaire-energie-classificatie).
#[must_use]
pub fn cooling_carrier_to_ep(c: nta8800_cooling::EnergyCarrier) -> EpCarrier {
    use nta8800_cooling::EnergyCarrier as C;
    match c {
        C::Electricity => EpCarrier::Elektriciteit,
        C::Gas => EpCarrier::Aardgas,
        C::DistrictCold => EpCarrier::Stadswarmte,
    }
}

// ---------------------------------------------------------------------------
// NTA 8800 §11.2.2 — q_V;ODA;req forfait
// ---------------------------------------------------------------------------
//
// OPMERKING (tech-debt): dit forfait staat nu op drie plekken in de workspace
// (`tojuli.rs`, Maartens orchestrator, hier). Consolidatie naar één publieke
// helper in `nta8800-ventilation` is F3-opruimwerk; hier bewust her-
// geïmplementeerd met volledige norm-referenties om de mapping-laag zelf-
// standig te houden.

/// NTA 8800 §11.2.2.1.1, formule (11.22): praktijkprestatiefactor `f_prac;req`.
const NTA_F_PRAC_REQ: f64 = 0.95;
/// NTA 8800 tabel 11.9: `f_lea;du` voor luchtdichtheidsklasse "Onbekend".
const NTA_F_LEA_DU_UNKNOWN: f64 = 1.10;
/// §11.2.2.4.1: omrekenfactor dm³/s → m³/h.
const NTA_DM3S_TO_M3H: f64 = 3.6;
/// §11.2.2.5.1, formule (11.63): woning-ondergrens `(q_usi;spec·A_g) ≥ 35 dm³/s`.
const NTA_WONING_MIN_CAPACITY_DM3S: f64 = 35.0;

/// Tabel 11.8 — `q_usi;spec` (dm³/(s·m²)) en `f_τ` per gebruiksfunctie.
///
/// Voor de woonfunctie is `f_τ = min[(0,38 + A_g·0,006); 0,8]`. Functies zonder
/// eigen kolomwaarde (industrie, overige) krijgen de kantoor-rij als
/// traceerbare conservatieve default — zelfde keuze als de TO-juli-keten.
#[allow(clippy::match_same_arms)]
fn usi_spec_and_f_tau(usage: UsageFunction, a_g: f64) -> (f64, f64) {
    use UsageFunction as UF;
    match usage {
        UF::Woonfunctie => {
            let f_tau = (0.38 + a_g * 0.006).min(0.8);
            (0.50, f_tau)
        }
        UF::Bijeenkomstfunctie => (1.71, 0.15),
        UF::Celfunctie => (0.84, 0.80),
        UF::Gezondheidszorgfunctie => (1.11, 0.30),
        UF::Industriefunctie | UF::Kantoorfunctie | UF::OverigeGebruiksfunctie => (1.11, 0.30),
        UF::Logiesfunctie => (0.84, 0.40),
        UF::Onderwijsfunctie => (3.64, 0.30),
        UF::Sportfunctie => (0.46, 0.30),
        UF::Winkelfunctie => (0.28, 0.40),
    }
}

/// Norm-forfait `q_V;ODA;req` in m³/h (NTA 8800 §11.2.2, formules
/// 11.22/11.56/11.57/11.63 + tabel 11.8) — de benodigde luchtvolumestroom van
/// buitenlucht wanneer geen luchtdebieten zijn ingevoerd.
#[must_use]
pub fn q_v_oda_req_m3_per_h(usage: UsageFunction, a_g_m2: f64) -> f64 {
    let a_g = a_g_m2.max(0.0);
    let (q_usi_spec, f_tau) = usi_spec_and_f_tau(usage, a_g);

    let mut capacity_dm3s = q_usi_spec * a_g;
    if matches!(usage, UsageFunction::Woonfunctie) {
        capacity_dm3s = capacity_dm3s.max(NTA_WONING_MIN_CAPACITY_DM3S);
    }

    // (11.56)/(11.57): geïnstalleerde capaciteit onbekend → reken-waarde;
    // (11.22): f_ctrl = f_sys = 1 (forfait-tak), ε_V = 1.
    NTA_F_LEA_DU_UNKNOWN * f_tau * capacity_dm3s * NTA_DM3S_TO_M3H / NTA_F_PRAC_REQ
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::{DwtwInput, PvInput};

    #[test]
    fn heating_hr_boiler_defaults_to_hr107() {
        let input = HeatingInput {
            generator: HeatGeneratorType::HrBoiler,
            cop: None,
            hr_class: None,
            district_factor: None,
            emission: None,
            distribution_efficiency: None,
            control_factor: None,
            coverage_fraction: 1.0,
        };
        let m = map_heating(&input);
        assert_eq!(m.generation, GenerationSystem::HRBoiler { class: HRClass::HR107 });
        assert_eq!(m.emission, EmissionSystem::RadiatorHighTemp);
        assert!((m.distribution.efficiency - 0.95).abs() < 1e-12);
        assert!((m.control.factor - 0.97).abs() < 1e-12);
    }

    #[test]
    fn heating_ground_heat_pump_uses_ground_scop_default() {
        let input = HeatingInput {
            generator: HeatGeneratorType::HeatPumpGround,
            cop: None,
            hr_class: None,
            district_factor: None,
            emission: Some(HeatEmissionType::FloorHeating),
            distribution_efficiency: None,
            control_factor: None,
            coverage_fraction: 1.0,
        };
        let m = map_heating(&input);
        assert_eq!(
            m.generation,
            GenerationSystem::HeatPump { scop: DEFAULT_HEAT_PUMP_SCOP_GROUND }
        );
        assert_eq!(m.emission, EmissionSystem::FloorHeating);
    }

    #[test]
    fn heating_heat_pump_honours_explicit_cop() {
        let input = HeatingInput {
            generator: HeatGeneratorType::HeatPumpAir,
            cop: Some(3.9),
            hr_class: None,
            district_factor: None,
            emission: None,
            distribution_efficiency: Some(0.90),
            control_factor: Some(1.0),
            coverage_fraction: 1.0,
        };
        let m = map_heating(&input);
        assert_eq!(m.generation, GenerationSystem::HeatPump { scop: 3.9 });
        assert!((m.distribution.efficiency - 0.90).abs() < 1e-12);
        assert!((m.control.factor - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dhw_electric_boiler_uses_storage_default() {
        let input = DhwInput {
            generator: DhwGeneratorType::ElectricBoiler,
            efficiency: None,
            dwtw: None,
            has_solar_boiler: false,
            solar_boiler_fraction: None,
        };
        assert_eq!(
            map_dhw_generation(&input),
            DhwGenerationSystem::ElectricBoiler { storage_loss_factor: 0.90 }
        );
        assert!(map_dwtw(&input).is_none());
    }

    #[test]
    fn dhw_heat_pump_default_scop_and_dwtw_default_aandeel() {
        let input = DhwInput {
            generator: DhwGeneratorType::HeatPump,
            efficiency: None,
            dwtw: Some(DwtwInput { efficiency: 0.45, douche_aandeel: None }),
            has_solar_boiler: false,
            solar_boiler_fraction: None,
        };
        assert_eq!(
            map_dhw_generation(&input),
            DhwGenerationSystem::HeatPumpDhw { scop_dhw: DEFAULT_DHW_HEAT_PUMP_SCOP }
        );
        let dwtw = map_dwtw(&input).unwrap();
        assert!((dwtw.efficiency - 0.45).abs() < 1e-12);
        assert!((dwtw.douche_aandeel - 0.4).abs() < 1e-12);
    }

    #[test]
    fn ventilation_system_c_forfait_fills_exhaust() {
        // Systeem C zonder mechanisch afvoerdebiet → q_V;ODA;req op de afvoer.
        let input = VentilationInput {
            system: VentilationSystemType::C,
            wtw_efficiency: None,
            sfp_w_per_m3h: None,
            bypass_enabled: false,
            mechanical_supply_m3_per_h: None,
            mechanical_exhaust_m3_per_h: None,
            infiltration_m3_per_h: None,
        };
        let m = map_ventilation(&input, UsageFunction::Woonfunctie, 120.0);
        let expected = q_v_oda_req_m3_per_h(UsageFunction::Woonfunctie, 120.0);
        assert!((m.flow.mechanical_exhaust - expected).abs() < 1e-9);
        assert!(m.wtw.is_none());
        assert!(matches!(m.system, VentilationSystem::C));
    }

    #[test]
    fn ventilation_system_d_with_wtw_and_forfait_supply() {
        let input = VentilationInput {
            system: VentilationSystemType::D,
            wtw_efficiency: Some(0.95),
            sfp_w_per_m3h: None,
            bypass_enabled: true,
            mechanical_supply_m3_per_h: None,
            mechanical_exhaust_m3_per_h: None,
            infiltration_m3_per_h: None,
        };
        let m = map_ventilation(&input, UsageFunction::Woonfunctie, 100.0);
        assert!(matches!(m.system, VentilationSystem::D { with_wtw: true }));
        let wtw = m.wtw.unwrap();
        assert!((wtw.efficiency - 0.95).abs() < 1e-12);
        assert!((wtw.fan_sfp - DEFAULT_SFP_W_PER_M3H).abs() < 1e-12);
        assert!(wtw.bypass_enabled);
        let expected = q_v_oda_req_m3_per_h(UsageFunction::Woonfunctie, 100.0);
        assert!((m.flow.mechanical_supply - expected).abs() < 1e-9);
    }

    #[test]
    fn ventilation_explicit_debieten_are_preserved() {
        let input = VentilationInput {
            system: VentilationSystemType::D,
            wtw_efficiency: Some(0.85),
            sfp_w_per_m3h: Some(0.2),
            bypass_enabled: false,
            mechanical_supply_m3_per_h: Some(150.0),
            mechanical_exhaust_m3_per_h: Some(150.0),
            infiltration_m3_per_h: Some(20.0),
        };
        let m = map_ventilation(&input, UsageFunction::Woonfunctie, 100.0);
        assert!((m.flow.mechanical_supply - 150.0).abs() < 1e-12);
        assert!((m.flow.mechanical_exhaust - 150.0).abs() < 1e-12);
        assert!((m.flow.infiltration - 20.0).abs() < 1e-12);
        assert!((m.wtw.unwrap().fan_sfp - 0.2).abs() < 1e-12);
    }

    #[test]
    fn cooling_free_cooling_default_fraction() {
        let input = CoolingInput {
            generator: CoolingGeneratorType::FreeCooling,
            seer: None,
            cop: None,
            free_cooling_fraction: None,
        };
        assert_eq!(
            map_cooling(&input),
            CoolingSystem::FreeCooling { factor: DEFAULT_FREE_COOLING_FRACTION }
        );
    }

    #[test]
    fn pv_azimuth_west_converts_to_negative_ninety() {
        let pv = vec![PvInput {
            id: None,
            name: None,
            peak_power_kwp: 5.0,
            azimuth_degrees: 270.0,
            tilt_degrees: 35.0,
            system_efficiency: None,
            inverter_efficiency: None,
            shadow_factor: None,
        }];
        let systems = map_pv(&pv).unwrap();
        assert_eq!(systems.len(), 1);
        assert!((systems[0].azimuth_degrees - (-90.0)).abs() < 1e-12);
        assert!((systems[0].system_efficiency - DEFAULT_PV_SYSTEM_EFFICIENCY).abs() < 1e-12);
    }

    #[test]
    fn pv_south_stays_180_but_is_rejected_by_range() {
        // 180° (zuid) valt precies op de bovengrens en blijft geldig.
        let pv = vec![PvInput {
            id: None,
            name: None,
            peak_power_kwp: 3.0,
            azimuth_degrees: 180.0,
            tilt_degrees: 30.0,
            system_efficiency: Some(0.8),
            inverter_efficiency: Some(0.95),
            shadow_factor: Some(0.9),
        }];
        let systems = map_pv(&pv).unwrap();
        assert!((systems[0].azimuth_degrees - 180.0).abs() < 1e-12);
        assert!((systems[0].shadow_factor - 0.9).abs() < 1e-12);
    }

    #[test]
    fn automation_maps_uniform_class() {
        let cfg = map_automation(&AutomationInput { bacs_class: BacsClassInput::A });
        assert_eq!(cfg.heating, BacsClass::A);
        assert_eq!(cfg.ventilation, BacsClass::A);
    }

    #[test]
    fn q_v_oda_req_woning_120m2_matches_norm_chain() {
        // f_τ = min(0,38 + 120·0,006; 0,8) = 0,8; cap = max(60; 35) = 60
        // q = 1,10·0,8·60·3,6/0,95 = 200,084…
        let q = q_v_oda_req_m3_per_h(UsageFunction::Woonfunctie, 120.0);
        assert!((q - 200.084_210_526_315).abs() < 1e-6);
    }

    #[test]
    fn carrier_mappers_project_onto_ep_carriers() {
        assert_eq!(
            heating_carrier_to_ep(nta8800_heating::model::EnergyCarrier::Gas),
            EpCarrier::Aardgas
        );
        assert_eq!(
            dhw_carrier_to_ep(nta8800_dhw::model::EnergyCarrier::Electricity),
            EpCarrier::Elektriciteit
        );
        assert_eq!(
            cooling_carrier_to_ep(nta8800_cooling::EnergyCarrier::DistrictCold),
            EpCarrier::Stadswarmte
        );
    }
}
