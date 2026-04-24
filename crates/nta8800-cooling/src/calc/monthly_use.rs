//! Pad 1 — actieve koeling volgens NTA 8800 H.10.
//!
//! `Q_C;use;zi;mi = Q_C;nd;zi;mi / (η_em · η_dist · COP · f_reg)`  [MJ]
//!
//! Voor [`crate::model::CoolingSystem::FreeCooling`] geldt dat alleen
//! `(1 − factor)` van de koudebehoefte door het mechanische deel hoeft te
//! worden gedekt; de rest is "gratis" via ventilatie of bodemlus. We
//! modelleren dit als:
//!
//! `Q_C;use = (1 − factor) · Q_C;nd / (η_em · η_dist · f_reg)`
//!
//! met een conservatieve nominale COP=1,0 voor de elektrische hulpenergie
//! die nog wel nodig is (ventilator, pomp).

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::zoning::Rekenzone;

use nta8800_demand::DemandResult;

use crate::errors::{CoolingCalcResult, CoolingError};
use crate::model::{CoolingDistribution, CoolingEmission, CoolingSystem};
use crate::result::{CoolingBreakdown, CoolingResult};

/// Valideer een rendement (0, 1].
fn validate_efficiency(name: &'static str, value: f64) -> CoolingCalcResult<()> {
    if !value.is_finite() || value <= 0.0 || value > 1.0 {
        return Err(CoolingError::InvalidEfficiency { name, value });
    }
    Ok(())
}

/// Valideer de COP/SCOP of vrije-koeling factor.
fn validate_system(system: &CoolingSystem) -> CoolingCalcResult<()> {
    match system {
        CoolingSystem::CompressionCooling { scop_cooling } => {
            if !scop_cooling.is_finite() || *scop_cooling <= 0.0 {
                return Err(CoolingError::NonPositiveCop {
                    value: *scop_cooling,
                });
            }
        }
        CoolingSystem::AbsorptionCooling { cop } => {
            if !cop.is_finite() || *cop <= 0.0 {
                return Err(CoolingError::NonPositiveCop { value: *cop });
            }
        }
        CoolingSystem::FreeCooling { factor } => {
            if !factor.is_finite() || !(0.0..=1.0).contains(factor) {
                return Err(CoolingError::InvalidFreeCoolingFactor { value: *factor });
            }
        }
    }
    Ok(())
}

/// Pad 1 — bereken het maandelijkse en jaarlijkse eindgebruik voor koeling.
///
/// `Q_C;use;zi;mi = Q_C;nd;zi;mi / (η_em · η_dist · COP · f_reg)`
///
/// # Parameters
/// - `_zone` — de rekenzone (momenteel alleen voor traceability; toekomstige
///   versies kunnen zone-specifieke correcties toepassen).
/// - `demand` — resultaat uit `nta8800-demand` met Q_C;nd per maand.
/// - `system` — koudeopwekker-type + efficiency.
/// - `distribution` — η_dist;C.
/// - `emission` — η_em;C + f_reg.
///
/// # Errors
/// Geeft [`CoolingError`] bij negatieve/ongeldige rendementen, COP of factor.
pub fn calculate_cooling(
    _zone: &Rekenzone,
    demand: &DemandResult,
    system: &CoolingSystem,
    distribution: &CoolingDistribution,
    emission: &CoolingEmission,
) -> CoolingCalcResult<CoolingResult> {
    validate_efficiency("η_em", emission.efficiency)?;
    validate_efficiency("η_dist", distribution.efficiency)?;
    validate_efficiency("f_reg", emission.regulation_factor)?;
    validate_system(system)?;

    let eta_em = emission.efficiency;
    let eta_dist = distribution.efficiency;
    let f_reg = emission.regulation_factor;
    let cop = system.nominal_cop();

    let monthly_q_c_use = compute_monthly_use(
        &demand.monthly_cooling_demand,
        system,
        eta_em,
        eta_dist,
        f_reg,
        cop,
    );

    let annual: Energy = Month::all().iter().map(|m| monthly_q_c_use[*m]).sum();

    let free_cooling_factor = match system {
        CoolingSystem::FreeCooling { factor } => *factor,
        _ => 0.0,
    };

    Ok(CoolingResult {
        energy_carrier: system.energy_carrier(),
        monthly_q_c_use,
        annual_q_c_use: annual,
        breakdown: CoolingBreakdown {
            monthly_q_c_nd: demand.monthly_cooling_demand.clone(),
            emission_efficiency: eta_em,
            distribution_efficiency: eta_dist,
            regulation_factor: f_reg,
            effective_cop: cop,
            free_cooling_factor,
        },
    })
}

fn compute_monthly_use(
    demand: &MonthlyProfile<Energy>,
    system: &CoolingSystem,
    eta_em: f64,
    eta_dist: f64,
    f_reg: f64,
    cop: f64,
) -> MonthlyProfile<Energy> {
    let mut values = [0.0_f64; 12];
    for (idx, month) in Month::all().iter().enumerate() {
        let q_nd = demand[*month];
        let q_use = match system {
            CoolingSystem::FreeCooling { factor } => {
                // Alleen (1 − factor) moet via hulp-energie gedekt worden.
                let mechanical_fraction = 1.0 - *factor;
                mechanical_fraction * q_nd / (eta_em * eta_dist * f_reg * cop)
            }
            _ => q_nd / (eta_em * eta_dist * f_reg * cop),
        };
        values[idx] = if q_use > 0.0 { q_use } else { 0.0 };
    }
    MonthlyProfile::new(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use nta8800_demand::DemandBreakdown;

    fn zone() -> Rekenzone {
        Rekenzone {
            id: "rz1".into(),
            name: "Test".into(),
            gebouw_id: "g1".into(),
            floor_area: 120.0,
            volume: 300.0,
            efr_ids: vec![],
            constructions: vec![],
            windows: vec![],
            openings: vec![],
            thermal_bridges_linear: vec![],
            thermal_bridges_point: vec![],
        }
    }

    fn demand_with_cooling(monthly: [f64; 12]) -> DemandResult {
        let annual = monthly.iter().sum::<f64>();
        DemandResult {
            monthly_heating_demand: MonthlyProfile::from_constant(0.0),
            monthly_cooling_demand: MonthlyProfile::new(monthly),
            annual_heating_demand: 0.0,
            annual_cooling_demand: annual,
            breakdown: DemandBreakdown {
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

    #[test]
    fn compressie_scop_4_halveert_vier_keer() {
        // Q_C;nd = 1000 MJ in juli, η_em=η_dist=f_reg=1, SCOP=4
        // → Q_C;use = 1000 / 4 = 250 MJ
        let mut monthly = [0.0; 12];
        monthly[Month::Juli.index()] = 1000.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::CompressionCooling { scop_cooling: 4.0 };
        let emission = CoolingEmission {
            efficiency: 1.0,
            regulation_factor: 1.0,
        };
        let dist = CoolingDistribution { efficiency: 1.0 };
        let result = calculate_cooling(&zone(), &demand, &system, &dist, &emission).unwrap();
        assert_abs_diff_eq!(result.monthly_q_c_use[Month::Juli], 250.0, epsilon = 1e-9);
        assert_abs_diff_eq!(result.annual_q_c_use, 250.0, epsilon = 1e-9);
        assert_eq!(
            result.energy_carrier,
            crate::model::EnergyCarrier::Electricity
        );
    }

    #[test]
    fn freecooling_factor_nul_gelijk_aan_scop_1() {
        // factor=0 → geen vrije koeling, alle koude via hulp-energie COP=1
        let mut monthly = [0.0; 12];
        monthly[Month::Augustus.index()] = 500.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::FreeCooling { factor: 0.0 };
        let emission = CoolingEmission {
            efficiency: 1.0,
            regulation_factor: 1.0,
        };
        let dist = CoolingDistribution { efficiency: 1.0 };
        let result = calculate_cooling(&zone(), &demand, &system, &dist, &emission).unwrap();
        assert_abs_diff_eq!(
            result.monthly_q_c_use[Month::Augustus],
            500.0,
            epsilon = 1e-9
        );
    }

    #[test]
    fn freecooling_factor_1_geeft_nul_energie() {
        // factor=1 → alle koude gratis, Q_C;use = 0
        let mut monthly = [0.0; 12];
        monthly[Month::Juli.index()] = 1000.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::FreeCooling { factor: 1.0 };
        let result = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution { efficiency: 1.0 },
            &CoolingEmission {
                efficiency: 1.0,
                regulation_factor: 1.0,
            },
        )
        .unwrap();
        assert_abs_diff_eq!(result.annual_q_c_use, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn freecooling_factor_0_3_dekt_30_procent() {
        // factor=0,3, Q_C;nd = 1000 MJ, η=1, COP=1 → (1-0,3) × 1000 = 700 MJ
        let mut monthly = [0.0; 12];
        monthly[Month::Juli.index()] = 1000.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::FreeCooling { factor: 0.3 };
        let result = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution { efficiency: 1.0 },
            &CoolingEmission {
                efficiency: 1.0,
                regulation_factor: 1.0,
            },
        )
        .unwrap();
        assert_abs_diff_eq!(result.monthly_q_c_use[Month::Juli], 700.0, epsilon = 1e-9);
    }

    #[test]
    fn absorptie_cop_0_8() {
        // Q_C;nd = 800 MJ, COP=0,8, η=1 → Q_C;use = 1000 MJ
        let mut monthly = [0.0; 12];
        monthly[Month::Juni.index()] = 800.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::AbsorptionCooling { cop: 0.8 };
        let result = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution { efficiency: 1.0 },
            &CoolingEmission {
                efficiency: 1.0,
                regulation_factor: 1.0,
            },
        )
        .unwrap();
        assert_abs_diff_eq!(result.monthly_q_c_use[Month::Juni], 1000.0, epsilon = 1e-9);
        assert_eq!(result.energy_carrier, crate::model::EnergyCarrier::Gas);
    }

    #[test]
    fn full_year_q_c_use_sums_annual() {
        let mut monthly = [0.0; 12];
        monthly[Month::Juni.index()] = 400.0;
        monthly[Month::Juli.index()] = 1000.0;
        monthly[Month::Augustus.index()] = 800.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::CompressionCooling { scop_cooling: 4.0 };
        let emission = CoolingEmission::default();
        let dist = CoolingDistribution::default();
        let result = calculate_cooling(&zone(), &demand, &system, &dist, &emission).unwrap();
        let expected: f64 = [400.0_f64, 1000.0, 800.0]
            .iter()
            .map(|q| q / (emission.efficiency * dist.efficiency * emission.regulation_factor * 4.0))
            .sum();
        assert_abs_diff_eq!(result.annual_q_c_use, expected, epsilon = 1e-6);
    }

    #[test]
    fn cooling_result_serde_roundtrip_via_calc() {
        let mut monthly = [0.0; 12];
        monthly[Month::Juli.index()] = 500.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::CompressionCooling { scop_cooling: 3.5 };
        let result = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution::default(),
            &CoolingEmission::default(),
        )
        .unwrap();
        let json = serde_json::to_string(&result).unwrap();
        let back: CoolingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }

    #[test]
    fn invalid_scop_returns_error() {
        let demand = demand_with_cooling([0.0; 12]);
        let system = CoolingSystem::CompressionCooling { scop_cooling: -1.0 };
        let err = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution::default(),
            &CoolingEmission::default(),
        )
        .unwrap_err();
        assert!(matches!(err, CoolingError::NonPositiveCop { .. }));
    }

    #[test]
    fn invalid_emission_returns_error() {
        let demand = demand_with_cooling([0.0; 12]);
        let system = CoolingSystem::CompressionCooling { scop_cooling: 4.0 };
        let emission = CoolingEmission {
            efficiency: 1.5,
            regulation_factor: 1.0,
        };
        let err = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution::default(),
            &emission,
        )
        .unwrap_err();
        assert!(matches!(err, CoolingError::InvalidEfficiency { .. }));
    }

    #[test]
    fn invalid_freecooling_factor_returns_error() {
        let demand = demand_with_cooling([0.0; 12]);
        let system = CoolingSystem::FreeCooling { factor: 1.5 };
        let err = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution::default(),
            &CoolingEmission::default(),
        )
        .unwrap_err();
        assert!(matches!(err, CoolingError::InvalidFreeCoolingFactor { .. }));
    }
}
