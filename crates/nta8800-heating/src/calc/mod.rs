//! Verwarming-keten orkestrator.
//!
//! Combineert de vier keten-componenten (afgifte, distributie, opwekking,
//! regeling) tot een maandelijks en jaarlijks eindenergiegebruik Q_H;use.
//!
//! ## Formule
//!
//! ```text
//! η_total = η_em × η_dist × η_gen × f_reg
//! Q_H;use;mi = Q_H;nd;mi / η_total       [MJ]
//! Q_H;use;an = Σ Q_H;use;mi              [MJ]
//! ```
//!
//! Voor warmtepomp (SCOP > 1) is η_total > 1 mogelijk, wat betekent dat
//! Q_H;use (elektrische input) lager is dan Q_H;nd (warmte-output).

pub mod distribution_loss;
pub mod emission_loss;
pub mod generation_efficiency;
pub mod monthly_use;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;

use nta8800_demand::DemandResult;

use crate::errors::{HeatingCalcResult, HeatingError};
use crate::model::{ControlFactor, DistributionSystem, EmissionSystem, GenerationSystem};
use crate::result::{HeatingBreakdown, HeatingResult};

/// Bereken maandelijks en jaarlijks Q_H;use voor verwarming conform NTA 8800
/// H.9 (V1 vereenvoudigd keten-model).
///
/// # Argumenten
///
/// - `demand` — [`DemandResult`] uit `nta8800-demand` (levert Q_H;nd per maand in MJ).
/// - `emission` — afgifte-systeem-type (radiator HT/LT, vloerverwarming, etc.).
/// - `distribution` — distributie-systeem met η_dist ∈ (0, 1].
/// - `generation` — opwekkings-systeem (HR-ketel, warmtepomp, elektr.
///   weerstand of stadsverwarming).
/// - `control` — regel-factor f_reg ∈ (0, 1].
///
/// # Returns
///
/// [`HeatingResult`] met `energy_carrier`, maandprofiel + jaartotaal Q_H;use
/// en breakdown (alle η-waarden + Q_H;nd-input).
///
/// # Errors
///
/// - [`HeatingError::InvalidEfficiency`] bij η_em, η_dist of f_reg buiten (0, 1]
/// - [`HeatingError::InvalidScop`] bij warmtepomp SCOP ≤ 0 of niet-eindig
/// - [`HeatingError::InvalidDistrictHeatingFactor`] bij stadswarmte-factor ≤ 0 of > 1
#[allow(clippy::needless_pass_by_value)]
pub fn calculate_heating(
    demand: &DemandResult,
    emission: EmissionSystem,
    distribution: &DistributionSystem,
    generation: &GenerationSystem,
    control: ControlFactor,
) -> HeatingCalcResult<HeatingResult> {
    // ---- Validatie en η-extractie ----
    let eta_em = emission.validated_efficiency()?;
    let eta_dist = distribution.validated()?;
    let eta_gen = generation.efficiency()?;
    let f_reg = control.validated()?;

    let total_eta = eta_em * eta_dist * eta_gen * f_reg;
    if !total_eta.is_finite() || total_eta <= 0.0 {
        return Err(HeatingError::InvalidEfficiency {
            name: "η_total",
            value: total_eta,
            upper: f64::INFINITY,
        });
    }

    // ---- Maandlus: Q_H;use = Q_H;nd / η_total ----
    let mut out_q_use = [0.0_f64; 12];
    for month in Month::all() {
        let q_nd: Energy = demand.monthly_heating_demand[month];
        out_q_use[month.index()] = monthly_use::monthly_q_h_use(q_nd, total_eta);
    }
    let monthly_q_h_use = MonthlyProfile::new(out_q_use);
    let annual_q_h_use: Energy = out_q_use.iter().sum();

    Ok(HeatingResult {
        energy_carrier: generation.energy_carrier(),
        monthly_q_h_use,
        annual_q_h_use,
        breakdown: HeatingBreakdown {
            emission_efficiency: eta_em,
            distribution_efficiency: eta_dist,
            generation_efficiency: eta_gen,
            control_factor: f_reg,
            total_efficiency: total_eta,
            monthly_q_h_nd: demand.monthly_heating_demand.clone(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EnergyCarrier, HRClass};

    fn sample_demand(monthly_mj: [f64; 12]) -> DemandResult {
        DemandResult {
            monthly_heating_demand: MonthlyProfile::new(monthly_mj),
            monthly_cooling_demand: MonthlyProfile::from_constant(0.0),
            annual_heating_demand: monthly_mj.iter().sum(),
            annual_cooling_demand: 0.0,
            breakdown: nta8800_demand::DemandBreakdown {
                monthly_q_ht: MonthlyProfile::from_constant(0.0),
                monthly_q_gn: MonthlyProfile::from_constant(0.0),
                monthly_q_sol: MonthlyProfile::from_constant(0.0),
                monthly_q_int: MonthlyProfile::from_constant(0.0),
                monthly_utilization_heating: MonthlyProfile::from_constant(0.0),
                monthly_utilization_cooling: MonthlyProfile::from_constant(0.0),
                time_constant_hours: 48.0,
            },
        }
    }

    /// Synthetische winter-dominante vraag: 10 000 MJ januari, aflopend.
    fn winter_dominant_demand() -> DemandResult {
        sample_demand([
            10_000.0, 8_500.0, 6_500.0, 3_500.0, 1_000.0, 100.0, 0.0, 50.0, 800.0, 3_000.0,
            6_000.0, 9_000.0,
        ])
    }

    #[test]
    fn hr107_ht_radiator_realistic() {
        // 10 GJ januari, HR107 (0,95) + radiator HT (0,95) + goed geïsoleerd
        // distributie (0,95) + weersafhankelijk (0,95).
        // η_total = 0.95^4 = 0.81450625
        // Q_H;use jan = 10_000 / 0.81450625 ≈ 12_277 MJ gas
        let demand = winter_dominant_demand();
        let r = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HRBoiler {
                class: HRClass::HR107,
            },
            ControlFactor::weather_compensated(),
        )
        .unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::Gas);
        let jan = r.monthly_q_h_use[Month::Januari];
        assert!(
            (jan - 12_277.0).abs() < 5.0,
            "Q_H;use januari: {jan} (verwacht ~12_277)"
        );
        // η_total = 0.95^4
        let expected_total = 0.95_f64.powi(4);
        assert!((r.breakdown.total_efficiency - expected_total).abs() < 1e-9);
    }

    #[test]
    fn heat_pump_scop_4_approximates_quarter() {
        // Q_H;nd = 12 000 MJ per maand constant.
        // SCOP = 4, FloorHeating η_em = 0.96, η_dist = 0.95, f_reg = 1.0
        // η_total = 0.96 × 0.95 × 4 × 1.0 = 3.648
        // Q_H;use (elek) = 12_000 / 3.648 ≈ 3_289.5 MJ
        let demand = sample_demand([12_000.0; 12]);
        let r = calculate_heating(
            &demand,
            EmissionSystem::FloorHeating,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HeatPump { scop: 4.0 },
            ControlFactor::on_off(),
        )
        .unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::Electricity);
        let expected = 12_000.0 / (0.96 * 0.95 * 4.0);
        let jan = r.monthly_q_h_use[Month::Januari];
        assert!(
            (jan - expected).abs() < 1e-6,
            "Q_H;use januari: {jan} (verwacht {expected})"
        );
        // Sanity: Q_H;use moet ongeveer een kwart van Q_H;nd zijn
        // (SCOP=4 domineert de keten; η_em en η_dist dragen enkele procenten extra).
        assert!(jan > 3_000.0 && jan < 3_500.0);
    }

    #[test]
    fn electric_resistance_plain() {
        // Q_H;nd = 1 000 MJ, η_em = 0.95, η_dist = 0.95, η_gen = 1.0, f_reg = 1.0
        // Q_H;use = 1000 / (0.95 × 0.95 × 1.0) ≈ 1 108 MJ
        let demand = sample_demand([1_000.0; 12]);
        let r = calculate_heating(
            &demand,
            EmissionSystem::RadiatorLowTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::ElectricResistance,
            ControlFactor::on_off(),
        )
        .unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::Electricity);
        let expected = 1_000.0 / (0.95 * 0.95);
        let jan = r.monthly_q_h_use[Month::Januari];
        assert!((jan - expected).abs() < 1e-6);
    }

    #[test]
    fn scop_1_equals_electric_resistance() {
        // Sanity: SCOP = 1 maakt warmtepomp thermisch identiek aan weerstand.
        let demand = sample_demand([5_000.0; 12]);
        let r_hp = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HeatPump { scop: 1.0 },
            ControlFactor::on_off(),
        )
        .unwrap();
        let r_el = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::ElectricResistance,
            ControlFactor::on_off(),
        )
        .unwrap();
        for m in Month::all() {
            assert!((r_hp.monthly_q_h_use[m] - r_el.monthly_q_h_use[m]).abs() < 1e-9);
        }
        assert!(
            (r_hp.annual_q_h_use - r_el.annual_q_h_use).abs()
                <= 1e-9 * r_hp.annual_q_h_use.abs().max(1.0)
        );
    }

    #[test]
    fn district_heating_carrier() {
        let demand = sample_demand([2_000.0; 12]);
        let r = calculate_heating(
            &demand,
            EmissionSystem::FloorHeating,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::DistrictHeating { factor: 0.9 },
            ControlFactor::on_off(),
        )
        .unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::DistrictHeat);
    }

    #[test]
    fn annual_is_sum_of_monthly() {
        let demand = winter_dominant_demand();
        let r = calculate_heating(
            &demand,
            EmissionSystem::RadiatorLowTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HRBoiler {
                class: HRClass::HR104,
            },
            ControlFactor::modulating(),
        )
        .unwrap();
        let sum_monthly: f64 = Month::all().iter().map(|m| r.monthly_q_h_use[*m]).sum();
        assert!(
            (r.annual_q_h_use - sum_monthly).abs() < 1e-9,
            "annual != Σ monthly"
        );
    }

    #[test]
    fn winter_exceeds_summer() {
        let demand = winter_dominant_demand();
        let r = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HRBoiler {
                class: HRClass::HR107,
            },
            ControlFactor::on_off(),
        )
        .unwrap();
        assert!(r.monthly_q_h_use[Month::Januari] > r.monthly_q_h_use[Month::Juli]);
    }

    #[test]
    fn zero_demand_gives_zero_use() {
        let demand = sample_demand([0.0; 12]);
        let r = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HRBoiler {
                class: HRClass::HR100,
            },
            ControlFactor::on_off(),
        )
        .unwrap();
        assert!((r.annual_q_h_use - 0.0).abs() < 1e-12);
        for m in Month::all() {
            assert!((r.monthly_q_h_use[m] - 0.0).abs() < 1e-12);
        }
    }

    #[test]
    fn invalid_scop_propagates() {
        let demand = sample_demand([1.0; 12]);
        let err = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HeatPump { scop: -2.0 },
            ControlFactor::on_off(),
        )
        .unwrap_err();
        assert!(matches!(err, HeatingError::InvalidScop { .. }));
    }

    #[test]
    fn invalid_distribution_propagates() {
        let demand = sample_demand([1.0; 12]);
        let err = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem { efficiency: 0.0 },
            &GenerationSystem::HRBoiler {
                class: HRClass::HR107,
            },
            ControlFactor::on_off(),
        )
        .unwrap_err();
        assert!(matches!(err, HeatingError::InvalidEfficiency { .. }));
    }

    #[test]
    fn breakdown_includes_q_h_nd_input() {
        let demand = winter_dominant_demand();
        let r = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HRBoiler {
                class: HRClass::HR107,
            },
            ControlFactor::on_off(),
        )
        .unwrap();
        for m in Month::all() {
            assert!(
                (r.breakdown.monthly_q_h_nd[m] - demand.monthly_heating_demand[m]).abs() < 1e-12
            );
        }
    }

    #[test]
    fn serde_round_trip_heating_result() {
        let demand = winter_dominant_demand();
        let r = calculate_heating(
            &demand,
            EmissionSystem::FloorHeating,
            &DistributionSystem::moderate(),
            &GenerationSystem::HeatPump { scop: 4.5 },
            ControlFactor::weather_compensated(),
        )
        .unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let back: HeatingResult = serde_json::from_str(&json).unwrap();
        // JSON float round-trip kan laatste ULP verliezen; vergelijk numeriek
        // binnen ULP-tolerantie. Structuur moet exact kloppen.
        assert_eq!(r.energy_carrier, back.energy_carrier);
        for m in Month::all() {
            let a = r.monthly_q_h_use[m];
            let b = back.monthly_q_h_use[m];
            assert!((a - b).abs() <= 1e-9 * a.abs().max(1.0), "Q_H;use {m:?}");
        }
        assert!(
            (r.annual_q_h_use - back.annual_q_h_use).abs()
                <= 1e-9 * r.annual_q_h_use.abs().max(1.0)
        );
        assert!((r.breakdown.total_efficiency - back.breakdown.total_efficiency).abs() < 1e-12);
    }

    #[test]
    fn hr107_beter_dan_hr100() {
        // Bij zelfde afgifte/distributie/regeling moet HR-107 minder Q_H;use
        // vragen dan HR-100 (hogere η_gen = lager gasverbruik).
        let demand = winter_dominant_demand();
        let r100 = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HRBoiler {
                class: HRClass::HR100,
            },
            ControlFactor::on_off(),
        )
        .unwrap();
        let r107 = calculate_heating(
            &demand,
            EmissionSystem::RadiatorHighTemp,
            &DistributionSystem::default_insulated(),
            &GenerationSystem::HRBoiler {
                class: HRClass::HR107,
            },
            ControlFactor::on_off(),
        )
        .unwrap();
        assert!(r107.annual_q_h_use < r100.annual_q_h_use);
    }
}
