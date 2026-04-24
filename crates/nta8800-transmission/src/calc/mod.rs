//! Rekenmodules voor de NTA 8800 maand-transmissiemethode.
//!
//! De sub-modules dekken elk een deel van de coëfficiënt-bepaling (§8.2, §8.4,
//! §8.3, §8.5, §8.2.3+§8.2.4). De publieke entry-point
//! [`calculate_transmission`] orchestreert deze en past de maand-formule (7.14)
//! toe per maand.

use std::collections::HashMap;
use std::hash::BuildHasher;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::{Energy, Temperature};
use nta8800_model::{ClimateData, Rekenzone, ThermalBridgeLinear, ThermalBridgePoint};

use crate::errors::{TransmissionError, TransmissionResult as CalcResult};
use crate::model::TransmissionElement;
use crate::result::{TransmissionBreakdown, TransmissionResult};

pub mod h_t_adjacent_zone;
pub mod h_t_ground;
pub mod h_t_outdoor;
pub mod h_t_unheated;
pub mod monthly_energy;
pub mod thermal_bridges;

/// Conversiefactor van kWh naar MJ (zie formule (7.14) waarin Q in kWh staat
/// en [`nta8800_model::units::Energy`] MJ representeert).
pub const KWH_TO_MJ: f64 = 3.6;

/// Maandlengtes in uren volgens NTA 8800 §17.2 (standaard-jaar, 8760 h totaal).
///
/// Gebruikt als factor `t_mi` in formules (7.14)/(7.15). Een lookup op
/// `Month::*.index()` geeft direct de juiste waarde. Februari = 28 dagen.
pub const MONTH_HOURS: [f64; 12] = [
    744.0, // januari   (31·24)
    672.0, // februari  (28·24)
    744.0, // maart     (31·24)
    720.0, // april     (30·24)
    744.0, // mei       (31·24)
    720.0, // juni      (30·24)
    744.0, // juli      (31·24)
    744.0, // augustus  (31·24)
    720.0, // september (30·24)
    744.0, // oktober   (31·24)
    720.0, // november  (30·24)
    744.0, // december  (31·24)
];

/// Parameterset voor grondtransmissie binnen één berekening.
///
/// Vereenvoudigde V1-aanpak conform §8.3.1 (bijlage I.2.3-pad): de consumer
/// levert `h_g_an` (W/K voor de gehele zone) aan. De volledige NEN-EN-ISO 13370
/// bepaling (op basis van vloerconstructie, karakteristieke breedte en
/// vloerrand-ψ) komt in V2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundParameters {
    /// Jaargemiddelde buitentemperatuur `θ_e;avg;an` in °C. Wordt in formule
    /// (7.14) apart meegenomen — de grondtransmissie gebruikt `θ_e;avg;an`,
    /// niet `θ_e;avg;mi`.
    pub annual_average_outdoor_temperature: Temperature,
}

impl GroundParameters {
    /// Construct met jaargemiddelde buitentemperatuur.
    #[must_use]
    pub const fn new(annual_average_outdoor_temperature: Temperature) -> Self {
        Self {
            annual_average_outdoor_temperature,
        }
    }

    /// Afgeleide: bereken het jaargemiddelde uit het maandelijks profiel in
    /// `ClimateData`. Dit is de voorkeurs-werkwijze zodat er geen discrepantie
    /// ontstaat met het in de klimaatdata opgenomen profiel.
    #[must_use]
    pub fn from_climate(climate: &ClimateData) -> Self {
        let sum: Temperature = Month::all()
            .into_iter()
            .map(|m| climate.outdoor_temperature[m])
            .sum();
        Self::new(sum / 12.0)
    }
}

/// Bereken maandelijkse transmissiewarmteverliezen voor één rekenzone.
///
/// Implementeert NTA 8800 H.7 maand-methode voor transmissie (formule (7.14))
/// met de deelcoëfficiënten uit H.8 (formules (7.16), (8.1), (8.52)).
///
/// # Argumenten
///
/// - `zone` — de rekenzone (id's van koudebruggen etc. worden niet gelezen; de
///   caller levert expliciet `thermal_bridges_*` en `elements` aan).
/// - `elements` — alle transmissie-elementen die aan de zone zijn toegewezen
///   (gevels, daken, vloeren, ramen, deuren) met hun boundary-classificatie.
/// - `thermal_bridges_linear`, `thermal_bridges_point` — lineaire en
///   puntvormige koudebruggen voor deze zone.
/// - `indoor_temperature` — maandprofiel `θ_int;calc;H;zi;mi` in °C.
/// - `climate` — klimaatdata met `θ_e;avg;mi` (wordt ook gebruikt voor het
///   afleiden van `θ_e;avg;an` via [`GroundParameters::from_climate`] als geen
///   `ground_parameters` is opgegeven).
/// - `h_g_an` — jaargemiddelde warmteoverdrachtcoëfficiënt naar grond in W/K.
///   Consumer berekent deze zelf volgens §8.3 of bijlage I.2.3. Pas 0.0 toe
///   als de zone geen grondcontact heeft.
/// - `unheated_space_b_factors` — map `id → b_U` voor elke
///   [`BoundaryType::UnheatedSpace`]. Waarden moeten in `0..=1` liggen. De
///   **vereenvoudigde volgorde**: consumer bepaalt `b_U` zelf (§8.4.1), typisch
///   0.5 (gedeeltelijk), 0.8 (licht geïsoleerde schil tussen onverwarmde en
///   buiten), of via formule (8.59).
/// - `adjacent_zone_temperatures` — map `id → maandprofiel`. Aanwezigheid van
///   een profile voor een id activeert de opt-in NEN-EN-ISO 13789 berekening
///   via formule (8.60)/(8.61). Een ontbrekend profile voor een aangrenzende
///   zone wordt geïnterpreteerd als `H_A = 0` conform NTA 8800 §8.5.
///
/// # Retourneert
///
/// Een [`TransmissionResult`] met maandelijkse totalen in MJ, een
/// breakdown per boundary-type, en de brutokoëfficiënten H_D, H_U, H_g;an, H_A.
///
/// # Errors
///
/// - [`TransmissionError::MissingUnheatedBFactor`] — als een element naar een
///   onverwarmde ruimte verwijst waarvoor geen b-factor is opgegeven.
/// - [`TransmissionError::BFactorOutOfRange`] — als een b-factor buiten
///   `0..=1` valt.
/// - [`TransmissionError::InvalidArea`] — bij niet-eindige of negatieve
///   oppervlakte.
#[allow(clippy::too_many_arguments)]
pub fn calculate_transmission<S1, S2>(
    zone: &Rekenzone,
    elements: &[TransmissionElement],
    thermal_bridges_linear: &[ThermalBridgeLinear],
    thermal_bridges_point: &[ThermalBridgePoint],
    indoor_temperature: &MonthlyProfile<Temperature>,
    climate: &ClimateData,
    h_g_an: f64,
    unheated_space_b_factors: &HashMap<String, f64, S1>,
    adjacent_zone_temperatures: &HashMap<String, MonthlyProfile<Temperature>, S2>,
) -> CalcResult<TransmissionResult>
where
    S1: BuildHasher,
    S2: BuildHasher,
{
    let _ = zone; // reserveer voor toekomstige per-zone-context (logging/audit)

    // ---- Valideer elementen ----
    for el in elements {
        if !el.area.is_finite() || el.area <= 0.0 {
            return Err(TransmissionError::InvalidArea {
                element_id: el.id.clone(),
                area: el.area,
            });
        }
    }

    // ---- Coëfficiënten in W/K ----
    let h_d_elements = h_t_outdoor::conductance_outdoor_elements(elements);
    let (h_bridges_linear, h_bridges_point) =
        thermal_bridges::bridge_conductances(thermal_bridges_linear, thermal_bridges_point);

    // H_D = ΣAU (outdoor) + ΣψL + Σχ — formule (8.1)
    let h_d = h_d_elements + h_bridges_linear + h_bridges_point;

    // H_U (formule (8.52))
    let h_u = h_t_unheated::conductance_via_unheated(elements, unheated_space_b_factors)?;

    // H_g;an (§8.3 — jaarlijkse coëfficiënt, door consumer aangeleverd)
    let h_g_an_total = h_t_ground::conductance_via_ground(elements, h_g_an);

    // ---- Maandlussen ----
    let mut out_outdoor = [0.0_f64; 12];
    let mut out_unheated = [0.0_f64; 12];
    let mut out_ground = [0.0_f64; 12];
    let mut out_adjacent = [0.0_f64; 12];
    let mut out_bridges = [0.0_f64; 12];

    let annual_avg_outdoor =
        GroundParameters::from_climate(climate).annual_average_outdoor_temperature;

    let h_a_elements_conductance = h_t_adjacent_zone::conductance_per_adjacent_zone(elements);

    let mut h_a_total_max = 0.0_f64;

    for month in Month::all() {
        let idx = month.index();
        let t_mi = MONTH_HOURS[idx];

        let theta_i = indoor_temperature[month];
        let theta_e_mi = climate.outdoor_temperature[month];
        let delta_t_outdoor = theta_i - theta_e_mi;
        let delta_t_ground = theta_i - annual_avg_outdoor;

        // --- Outdoor (element-deel van H_D) ---
        let q_outdoor_kwh = h_d_elements * delta_t_outdoor * 0.001 * t_mi;
        out_outdoor[idx] = q_outdoor_kwh * KWH_TO_MJ;

        // --- Thermal bridges (ψ×L + Σχ) ---
        let q_bridges_kwh = (h_bridges_linear + h_bridges_point) * delta_t_outdoor * 0.001 * t_mi;
        out_bridges[idx] = q_bridges_kwh * KWH_TO_MJ;

        // --- Unheated (H_U) ---
        let q_unheated_kwh = h_u * delta_t_outdoor * 0.001 * t_mi;
        out_unheated[idx] = q_unheated_kwh * KWH_TO_MJ;

        // --- Ground (H_g;an × (θ_i − θ_e;avg;an)) ---
        let q_ground_kwh = h_g_an_total * delta_t_ground * 0.001 * t_mi;
        out_ground[idx] = q_ground_kwh * KWH_TO_MJ;

        // --- Adjacent zone (opt-in formules (8.60)/(8.61)) ---
        let mut h_a_mi = 0.0_f64;
        let mut q_adjacent_kwh = 0.0_f64;
        for (zone_id, h_d_ia) in &h_a_elements_conductance {
            if let Some(profile) = adjacent_zone_temperatures.get(zone_id) {
                let theta_a = profile[month];
                // formule (8.61): b_A;mi = (θ_i − θ_a) / (θ_i − θ_e;mi)
                // gevolgd door (8.60): H_A;mi = H_D;ia · b_A;mi
                // Gecombineerd met Q = H_A × (θ_i − θ_e;mi) × t × 0.001
                // vereenvoudigt dit tot Q = H_D;ia × (θ_i − θ_a) × t × 0.001,
                // wat numeriek stabieler is (geen deling door ~0 als θ_i ≈ θ_e).
                let q_zone = h_d_ia * (theta_i - theta_a) * 0.001 * t_mi;
                q_adjacent_kwh += q_zone;
                let denom = theta_i - theta_e_mi;
                if denom.abs() > f64::EPSILON {
                    h_a_mi += h_d_ia * ((theta_i - theta_a) / denom);
                }
            }
            // ontbrekend profile → H_A = 0 (NTA default, §8.5)
        }
        out_adjacent[idx] = q_adjacent_kwh * KWH_TO_MJ;
        if h_a_mi.abs() > h_a_total_max.abs() {
            h_a_total_max = h_a_mi;
        }
    }

    // ---- Totalen ----
    let monthly_q_t_vals: [Energy; 12] = std::array::from_fn(|i| {
        out_outdoor[i] + out_unheated[i] + out_ground[i] + out_adjacent[i] + out_bridges[i]
    });
    let annual_q_t: Energy = monthly_q_t_vals.iter().sum();

    Ok(TransmissionResult {
        monthly_q_t: MonthlyProfile::new(monthly_q_t_vals),
        annual_q_t,
        breakdown: TransmissionBreakdown {
            outdoor: MonthlyProfile::new(out_outdoor),
            unheated_space: MonthlyProfile::new(out_unheated),
            ground: MonthlyProfile::new(out_ground),
            adjacent_zone: MonthlyProfile::new(out_adjacent),
            thermal_bridges: MonthlyProfile::new(out_bridges),
        },
        h_d,
        h_u,
        h_g_an: h_g_an_total,
        h_a: h_a_total_max,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_hours_sum_to_8760() {
        let sum: f64 = MONTH_HOURS.iter().sum();
        assert!((sum - 8760.0).abs() < 1e-9, "sum = {sum}");
    }

    #[test]
    fn kwh_to_mj_is_three_point_six() {
        assert!((KWH_TO_MJ - 3.6).abs() < f64::EPSILON);
    }

    #[test]
    fn ground_parameters_from_constant_climate_equals_constant() {
        use std::collections::BTreeMap;
        let climate = ClimateData {
            outdoor_temperature: MonthlyProfile::from_constant(10.0),
            solar_irradiation: BTreeMap::new(),
            cooling_reference_temperature: MonthlyProfile::from_constant(None),
            wind_speed: MonthlyProfile::from_constant(3.0),
            wtw_preheat_temperature: MonthlyProfile::from_constant(0.0),
        };
        let gp = GroundParameters::from_climate(&climate);
        assert!((gp.annual_average_outdoor_temperature - 10.0).abs() < 1e-12);
    }
}
