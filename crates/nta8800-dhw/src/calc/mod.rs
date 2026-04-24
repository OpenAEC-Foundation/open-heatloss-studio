//! Warm-tapwater keten-orkestrator.
//!
//! Combineert de drie keten-componenten (afgifte, distributie, opwekking) +
//! optionele DWTW tot een maandelijks en jaarlijks eindenergiegebruik Q_W;use
//! per energiedrager.
//!
//! ## Formule
//!
//! ```text
//! Q_W;rcd;mi  = η_rcd × C_sh × Q_W;nd;mi           [MJ]     (V1 vereenvoudigd
//!                                                            uit formule 13.51)
//! η_total     = η_W;em × η_W;dis × η_W;gen
//! Q_W;use;mi  = (Q_W;nd;mi − Q_W;rcd;mi) / η_total  [MJ]
//! Q_W;use;an  = Σ Q_W;use;mi                        [MJ]
//! ```
//!
//! Voor warmtepomp (SCOP_W > 1) is η_total > 1 mogelijk, wat betekent dat
//! Q_W;use (elektrische input) lager is dan Q_W;nd (warmte-output).

pub mod monthly_use;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::zoning::Rekenzone;

use crate::errors::{DhwCalcResult, DhwError};
use crate::model::{
    DhwDemand, DhwDistribution, DhwEmission, DhwGenerationSystem, DouchewtwRecovery,
};
use crate::result::{DhwBreakdown, DhwResult};

/// Bereken maandelijks en jaarlijks `Q_W;use` voor warm tapwater conform
/// NTA 8800:2025+C1:2026 H.13 (V1 vereenvoudigd keten-model).
///
/// # Argumenten
///
/// - `zone` — rekenzone (V1 gebruikt `zone` alleen voor audit-metadata;
///   de netto warmtebehoefte komt uit `demand`). Zie
///   [`nta8800_model::Rekenzone`].
/// - `demand` — [`DhwDemand`] met `Q_W;nd` maandprofiel (MJ).
/// - `emission` — afgifte-systeem met η_W;em ∈ (0, 1].
/// - `distribution` — distributie-systeem met η_W;dis ∈ (0, 1].
/// - `generation` — opwekkings-systeem (HR-combi, elektr. boiler,
///   warmtepomp, of stadsverwarming).
/// - `recovery` — optioneel DWTW-systeem.
///
/// # Returns
///
/// [`DhwResult`] met `energy_carrier`, maandprofiel + jaartotaal Q_W;use
/// en breakdown (alle η-waarden + Q_W;nd + Q_W;rcd).
///
/// # Errors
///
/// - [`DhwError::InvalidEfficiency`] bij η_W;em, η_W;dis of DWTW-waarden buiten (0, 1]
/// - [`DhwError::InvalidScop`] bij warmtepomp SCOP_W ≤ 0 of niet-eindig
/// - [`DhwError::InvalidDistrictHeatingFactor`] bij stadswarmte-factor buiten (0, 1]
#[allow(clippy::needless_pass_by_value)]
pub fn calculate_dhw(
    _zone: &Rekenzone,
    demand: &DhwDemand,
    emission: &DhwEmission,
    distribution: &DhwDistribution,
    generation: &DhwGenerationSystem,
    recovery: Option<&DouchewtwRecovery>,
) -> DhwCalcResult<DhwResult> {
    // ---- Validatie en η-extractie ----
    let eta_em = emission.validated_efficiency()?;
    let eta_dist = distribution.validated()?;
    let eta_gen = generation.efficiency()?;

    let total_eta = eta_em * eta_dist * eta_gen;
    if !total_eta.is_finite() || total_eta <= 0.0 {
        return Err(DhwError::InvalidEfficiency {
            name: "η_total (dhw)",
            value: total_eta,
            upper: f64::INFINITY,
        });
    }

    let (eta_rcd, aandeel_rcd) = match recovery {
        Some(r) => r.validated()?,
        None => (0.0, 0.0),
    };

    // ---- Maandlus: Q_W;rcd + Q_W;use ----
    let mut out_q_rcd = [0.0_f64; 12];
    let mut out_q_use = [0.0_f64; 12];
    for month in Month::all() {
        let q_nd: Energy = demand.monthly_demand[month];
        // Formule 13.51 vereenvoudigd: Q_rcd = η × C_sh × Q_W;nd
        // Clamp: Q_rcd mag niet groter worden dan Q_W;nd zelf
        // (theoretisch mogelijk bij η=1 × C_sh=1).
        let q_rcd_raw = eta_rcd * aandeel_rcd * q_nd;
        let q_rcd = q_rcd_raw.min(q_nd).max(0.0);
        out_q_rcd[month.index()] = q_rcd;

        let q_nd_net = q_nd - q_rcd;
        out_q_use[month.index()] = monthly_use::monthly_q_w_use(q_nd_net, total_eta);
    }
    let monthly_q_w_use = MonthlyProfile::new(out_q_use);
    let monthly_q_w_rcd = MonthlyProfile::new(out_q_rcd);
    let annual_q_w_use: Energy = out_q_use.iter().sum();

    Ok(DhwResult {
        energy_carrier: generation.energy_carrier(),
        monthly_q_w_use,
        annual_q_w_use,
        breakdown: DhwBreakdown {
            emission_efficiency: eta_em,
            distribution_efficiency: eta_dist,
            generation_efficiency: eta_gen,
            total_efficiency: total_eta,
            recovery_efficiency: eta_rcd,
            recovery_aandeel: aandeel_rcd,
            monthly_q_w_nd: demand.monthly_demand.clone(),
            monthly_q_w_rcd,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nta8800_model::time::Month;

    use crate::model::EnergyCarrier;

    fn sample_zone() -> Rekenzone {
        Rekenzone {
            id: "rz1".into(),
            name: "Woonzone".into(),
            gebouw_id: "g1".into(),
            floor_area: 100.0,
            volume: 250.0,
            efr_ids: vec!["efr1".into()],
            constructions: vec![],
            windows: vec![],
            openings: vec![],
            thermal_bridges_linear: vec![],
            thermal_bridges_point: vec![],
        }
    }

    #[test]
    fn woning_hr_combi_plausible_annual() {
        // 100 m² woning, N_P ≈ 2,28, Q_W;nd ≈ 1.952 kWh = 7.026 MJ/jaar
        // HR-combi (0,80) × Woning default (0,8834) × ind. distributie (1,0)
        //   ⇒ η_total ≈ 0,7067
        // Q_W;use ≈ 7.026 / 0,7067 ≈ 9.941 MJ/jaar gas
        let zone = sample_zone();
        let demand = DhwDemand::forfaitair_woningbouw(100.0).unwrap();
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            None,
        )
        .unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::Gas);
        // 7-12 GJ range conform norm voor 100 m² woning gasverbruik DHW
        assert!(
            r.annual_q_w_use > 7_000.0 && r.annual_q_w_use < 12_000.0,
            "annual Q_W;use = {} MJ",
            r.annual_q_w_use
        );
    }

    #[test]
    fn heat_pump_scop_3_lower_electrical_than_nd() {
        // SCOP_W = 3 ⇒ elektra verbruik ≈ Q_W;nd / (3 × eta_em × eta_dist)
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(12_000.0); // 12 GJ/jaar
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HeatPumpDhw { scop_dhw: 3.0 },
            None,
        )
        .unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::Electricity);
        // Elektra-input moet fors lager zijn dan 12 GJ (SCOP dominant)
        assert!(
            r.annual_q_w_use < 5_500.0,
            "HP elektra = {} (verwacht < 5.5 GJ)",
            r.annual_q_w_use
        );
    }

    #[test]
    fn electric_boiler_near_identity() {
        // Elektr. boiler (0,90) × ind. distributie (1,0) × kort util (1,0)
        //  ⇒ η_total = 0,90 — elektra verbruik ≈ Q_W;nd / 0,90
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(10_000.0);
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::UtiliteitKort,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::electric_boiler_default(),
            None,
        )
        .unwrap();
        let expected = 10_000.0 / 0.90;
        assert_relative_eq!(r.annual_q_w_use, expected, max_relative = 1e-9);
    }

    #[test]
    fn dwtw_eta_half_halveert_douche_bijdrage() {
        // Zonder DWTW: Q_W;use;0 = Q_nd / η_total
        // Met DWTW η=0,5 × C_sh=0,4: Q_rcd = 0,2 × Q_nd
        //   ⇒ Q_W;use = 0,8 × Q_nd / η_total
        //   ⇒ Q_W;use / Q_W;use;0 = 0,8
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(10_000.0);
        let r_no = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            None,
        )
        .unwrap();
        let r_wtw = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            Some(&DouchewtwRecovery::new(0.5)), // default aandeel = 0,4
        )
        .unwrap();
        let ratio = r_wtw.annual_q_w_use / r_no.annual_q_w_use;
        assert_relative_eq!(ratio, 0.8, max_relative = 1e-9);
    }

    #[test]
    fn dwtw_eta_zero_no_effect() {
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(5_000.0);
        let r_no = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::UtiliteitKort,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            None,
        )
        .unwrap();
        let r_zero = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::UtiliteitKort,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            Some(&DouchewtwRecovery::new(0.0)),
        )
        .unwrap();
        assert_relative_eq!(
            r_no.annual_q_w_use,
            r_zero.annual_q_w_use,
            max_relative = 1e-12
        );
    }

    #[test]
    fn dwtw_eta_one_max_recovery_clamped() {
        // η_rcd = 1,0, C_sh = 1,0 ⇒ Q_rcd = Q_W;nd volledig ⇒ Q_W;use = 0
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(8_000.0);
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::UtiliteitKort,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            Some(&DouchewtwRecovery::with_aandeel(1.0, 1.0)),
        )
        .unwrap();
        assert!(
            r.annual_q_w_use.abs() < 1e-9,
            "max recovery zou Q_W;use=0 moeten geven, kreeg {}",
            r.annual_q_w_use
        );
    }

    #[test]
    fn annual_is_sum_of_monthly() {
        let zone = sample_zone();
        let demand = DhwDemand::forfaitair_woningbouw(85.0).unwrap();
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_circulatie(),
            &DhwGenerationSystem::electric_boiler_default(),
            Some(&DouchewtwRecovery::new(0.3)),
        )
        .unwrap();
        let sum_monthly: f64 = Month::all().iter().map(|m| r.monthly_q_w_use[*m]).sum();
        assert_relative_eq!(r.annual_q_w_use, sum_monthly, max_relative = 1e-9);
    }

    #[test]
    fn zero_demand_gives_zero_use() {
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(0.0);
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            None,
        )
        .unwrap();
        assert!(r.annual_q_w_use.abs() < 1e-12);
        for m in Month::all() {
            assert!(r.monthly_q_w_use[m].abs() < 1e-12);
        }
    }

    #[test]
    fn invalid_scop_propagates() {
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(1_000.0);
        let err = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HeatPumpDhw { scop_dhw: -1.0 },
            None,
        )
        .unwrap_err();
        assert!(matches!(err, DhwError::InvalidScop { .. }));
    }

    #[test]
    fn invalid_distribution_propagates() {
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(1_000.0);
        let err = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::custom(0.0),
            &DhwGenerationSystem::HRCombiBoiler,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn district_heating_carrier() {
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(3_000.0);
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::UtiliteitKort,
            &DhwDistribution::default_circulatie(),
            &DhwGenerationSystem::DistrictHeating { factor: 0.95 },
            None,
        )
        .unwrap();
        assert_eq!(r.energy_carrier, EnergyCarrier::DistrictHeat);
    }

    #[test]
    fn breakdown_includes_q_w_nd_and_rcd() {
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(6_000.0);
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            Some(&DouchewtwRecovery::new(0.4)),
        )
        .unwrap();
        let sum_rcd: f64 = Month::all()
            .iter()
            .map(|m| r.breakdown.monthly_q_w_rcd[*m])
            .sum();
        // η_rcd × C_sh × annual = 0,4 × 0,4 × 6000 = 960
        assert_relative_eq!(sum_rcd, 0.4 * 0.4 * 6_000.0, max_relative = 1e-9);
        for m in Month::all() {
            assert_relative_eq!(
                r.breakdown.monthly_q_w_nd[m],
                demand.monthly_demand[m],
                max_relative = 1e-12
            );
        }
    }

    #[test]
    fn serde_round_trip_result() {
        let zone = sample_zone();
        let demand = DhwDemand::forfaitair_woningbouw(110.0).unwrap();
        let r = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_circulatie(),
            &DhwGenerationSystem::HeatPumpDhw { scop_dhw: 2.8 },
            Some(&DouchewtwRecovery::new(0.45)),
        )
        .unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let back: DhwResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r.energy_carrier, back.energy_carrier);
        for m in Month::all() {
            let a = r.monthly_q_w_use[m];
            let b = back.monthly_q_w_use[m];
            assert!((a - b).abs() <= 1e-9 * a.abs().max(1.0));
        }
        assert_relative_eq!(r.annual_q_w_use, back.annual_q_w_use, max_relative = 1e-9);
    }

    #[test]
    fn heat_pump_beats_gas_energetisch() {
        // Warmtepomp moet minder primaire energie-input tonen dan HR-ketel
        // voor dezelfde Q_W;nd (Q_W;use in MJ, dus vergelijking tussen
        // elektra-input en gas-input op dezelfde schaal).
        let zone = sample_zone();
        let demand = DhwDemand::from_annual_even(10_000.0);
        let r_gas = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HRCombiBoiler,
            None,
        )
        .unwrap();
        let r_hp = calculate_dhw(
            &zone,
            &demand,
            &DhwEmission::WoningDefault,
            &DhwDistribution::default_individueel(),
            &DhwGenerationSystem::HeatPumpDhw { scop_dhw: 2.5 },
            None,
        )
        .unwrap();
        assert!(
            r_hp.annual_q_w_use < r_gas.annual_q_w_use,
            "HP {} ≥ gas {} ?",
            r_hp.annual_q_w_use,
            r_gas.annual_q_w_use
        );
    }
}
