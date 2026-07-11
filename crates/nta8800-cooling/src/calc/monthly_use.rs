//! Pad 1 — actieve koeling volgens NTA 8800 H.10.
//!
//! Compressie/absorptie: `Q_C;use;zi;mi = Q_C;nd;zi;mi / (η_em · η_dist · COP · f_reg)`  [MJ]
//! — de koude die de opwekker levert `Q_C;gen;out = Q_C;nd / (η_em · η_dist · f_reg)`
//! gedeeld door de energie-efficiëntie EER/ζ (§10.5.6.1 formules 10.76/10.77, p. 417).
//!
//! Voor [`crate::model::CoolingSystem::FreeCooling`] dekt de vrije opwekker
//! (preferentie 1, tabel 10.15) de energiefractie `factor`; het restant
//! `(1 − factor)` loopt via een backup-compressiekoelmachine (forfait EER 3,0,
//! tabel 10.29). Vrije koeling kost **alleen pompenergie** (§10.5.7.2.1,
//! formule 10.86 p. 423): `W_fc = Q_C;gen;out / EER_fc`. Dus:
//!
//! `Q_C;use = Q_C;gen;out · [ factor / EER_fc + (1 − factor) / EER_backup ]`
//!
//! De vroegere modellering (het niet-vrije deel tegen "nominale COP = 1,0",
//! d.w.z. `Q_C;use ≈ (1 − factor) · Q_C;gen;out`) is weerlegd: dat rekent bijna
//! één-op-één elektriciteit op het backup-deel i.p.v. deling door EER_backup.

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::zoning::Rekenzone;

use nta8800_demand::DemandResult;

use crate::errors::{CoolingCalcResult, CoolingError};
use crate::model::{CoolingDistribution, CoolingEmission, CoolingSystem};
use crate::result::{CoolingBreakdown, CoolingResult};

/// Forfaitair opwekkingsrendement vrije koeling EER_fc — NTA 8800:2025+C1:2026
/// tabel 10.34 (p. 424), rij "Koudeopslag gesloten systeem met
/// bodemwarmtewisselaars" / "Oppervlaktewater". Conservatieve V1-forfait (de
/// laagste rij ≥ 8, passend bij een bodem-WP-bron); het invoermodel selecteert
/// het EER_fc-type nog niet (V2). Alle tabel-10.34-waarden zijn ≥ 8 → de koude
/// telt als omgevingskoude (§5.6.2.2, zie [`EER_RENCOLD_THRESHOLD`]).
pub const EER_FREE_COOLING: f64 = 10.0;

/// Forfaitair EER van de backup-compressiekoelmachine die het niet-vrije deel
/// `(1 − factor)` van de koudevraag dekt — NTA 8800 tabel 10.29 (p. 419),
/// "onbekende koudeopwekker in een collectieve gebouwinstallatie" = 3,00. Onder
/// de rencold-drempel (< 8) → dit deel telt niet als hernieuwbaar.
pub const EER_BACKUP_COMPRESSION: f64 = 3.0;

/// Drempel-EER waarboven (vrije) koeling als hernieuwbare omgevingskoude telt —
/// NTA 8800 §5.6.2.2, formule 5.34 (p. 105): `EER ≥ 8` → `Q_C;gen;out` telt als
/// rencold; `EER < 8` → 0.
pub const EER_RENCOLD_THRESHOLD: f64 = 8.0;

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

    let (monthly_q_c_use, monthly_rencold) = compute_monthly_use(
        &demand.monthly_cooling_demand,
        system,
        eta_em,
        eta_dist,
        f_reg,
        cop,
    );

    let annual: Energy = Month::all().iter().map(|m| monthly_q_c_use[*m]).sum();
    let annual_rencold: Energy = Month::all().iter().map(|m| monthly_rencold[*m]).sum();

    let free_cooling_factor = match system {
        CoolingSystem::FreeCooling { factor } => *factor,
        _ => 0.0,
    };

    Ok(CoolingResult {
        energy_carrier: system.energy_carrier(),
        monthly_q_c_use,
        annual_q_c_use: annual,
        monthly_rencold_mj: monthly_rencold,
        annual_rencold_mj: annual_rencold,
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

/// Bereken per maand het koel-eindgebruik `Q_C;use` én de hernieuwbare
/// omgevingskoude `Q_rencold` (beide MJ).
///
/// - Compressie/absorptie: `Q_C;use = Q_C;gen;out / COP`, rencold = 0 (forfait
///   EER 3,0 / ζ 0,80 < 8, §5.6.2.2).
/// - Vrije koeling: `Q_C;gen;out = Q_C;nd / (η_em · η_dist · f_reg)` wordt voor
///   het aandeel `factor` tegen `EER_fc` (tabel 10.34) en voor `(1 − factor)`
///   tegen een backup-compressie-EER (tabel 10.29) omgezet naar elektriciteit
///   (formule 10.86). De vrij-geleverde koude `factor · Q_C;gen;out` telt als
///   rencold zodra `EER_fc ≥ 8`.
fn compute_monthly_use(
    demand: &MonthlyProfile<Energy>,
    system: &CoolingSystem,
    eta_em: f64,
    eta_dist: f64,
    f_reg: f64,
    cop: f64,
) -> (MonthlyProfile<Energy>, MonthlyProfile<Energy>) {
    let mut use_values = [0.0_f64; 12];
    let mut rencold_values = [0.0_f64; 12];
    let system_losses = eta_em * eta_dist * f_reg;
    for (idx, month) in Month::all().iter().enumerate() {
        let q_nd = demand[*month];
        let (q_use, q_rencold) = match system {
            CoolingSystem::FreeCooling { factor } => {
                // Koude die de opwekker moet leveren (§10.5, Q_C;gen;out).
                let q_gen_out = q_nd / system_losses;
                // EER van de vrije opwekker (V1-forfait tabel 10.34; V2 maakt dit
                // type-afhankelijk variabel). Als lokale variabele zodat de
                // rencold-drempeltest op de *werkelijk gebruikte* EER test.
                let eer_fc = EER_FREE_COOLING;
                // Elektriciteit: vrij deel via EER_fc, restant via backup-EER
                // (formule 10.86 + tabel 10.29). Beide EER's zijn > 0.
                let q_use =
                    q_gen_out * (factor / eer_fc + (1.0 - factor) / EER_BACKUP_COMPRESSION);
                // rencold (§5.6.2.2): vrij geleverde koude telt alleen bij
                // EER ≥ 8. Tabel 10.34 kent types < 8 niet expliciet (laagste =
                // dauwpuntskoeling = 8), maar de drempel blijft betekenisvol
                // zodra V2 een variabele `eer_fc` levert of een lage-EER-opwekker
                // onder de vrije-koeling-tak valt.
                let q_rencold = if eer_fc >= EER_RENCOLD_THRESHOLD {
                    factor * q_gen_out
                } else {
                    0.0
                };
                (q_use, q_rencold)
            }
            _ => (q_nd / (system_losses * cop), 0.0),
        };
        use_values[idx] = q_use.max(0.0);
        rencold_values[idx] = q_rencold.max(0.0);
    }
    (
        MonthlyProfile::new(use_values),
        MonthlyProfile::new(rencold_values),
    )
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
    fn freecooling_factor_nul_loopt_volledig_via_backup_eer() {
        // NTA 8800 tabel 10.15/10.29: factor=0 → geen vrije koeling; het volledige
        // Q_C;gen;out loopt via de backup-compressiekoelmachine (EER 3,0).
        // Q_C;gen;out = 500 (η=1) → Q_C;use = 500 / 3,0. (Weerlegt de oude lezing
        // "COP = 1,0" die 500 gaf.) Geen rencold (backup EER 3 < 8).
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
            500.0 / EER_BACKUP_COMPRESSION,
            epsilon = 1e-9
        );
        assert_abs_diff_eq!(result.annual_rencold_mj, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn freecooling_factor_1_kost_alleen_pompenergie() {
        // NTA 8800 formule 10.86 + tabel 10.34: factor=1 → alle koude vrij, maar
        // W_fc = Q_C;gen;out / EER_fc ≠ 0 (pompenergie). Q_C;gen;out = 1000 (η=1)
        // → Q_C;use = 1000 / EER_fc. rencold = 1 × 1000 (EER_fc ≥ 8, §5.6.2.2).
        // (Weerlegt de oude lezing "vrije koeling = 0 elektriciteit".)
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
        assert_abs_diff_eq!(
            result.annual_q_c_use,
            1000.0 / EER_FREE_COOLING,
            epsilon = 1e-9
        );
        assert_abs_diff_eq!(result.annual_rencold_mj, 1000.0, epsilon = 1e-9);
    }

    #[test]
    fn freecooling_factor_0_3_splitst_vrij_en_backup() {
        // NTA 8800 formule 10.86 + tabellen 10.34/10.29: factor=0,3, Q_C;gen;out =
        // 1000 (η=1) → Q_C;use = 1000·(0,3/EER_fc + 0,7/EER_backup); rencold =
        // 0,3 × 1000. (Weerlegt de oude lezing "(1−factor)·Q bij COP 1,0" = 700.)
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
        let expected = 1000.0 * (0.3 / EER_FREE_COOLING + 0.7 / EER_BACKUP_COMPRESSION);
        assert_abs_diff_eq!(result.monthly_q_c_use[Month::Juli], expected, epsilon = 1e-9);
        assert_abs_diff_eq!(result.monthly_rencold_mj[Month::Juli], 300.0, epsilon = 1e-9);
    }

    #[test]
    fn compressie_levert_geen_rencold() {
        // §5.6.2.2: compressiekoeling (forfait EER 3,0 < 8) telt niet als
        // omgevingskoude.
        let mut monthly = [0.0; 12];
        monthly[Month::Juli.index()] = 1000.0;
        let demand = demand_with_cooling(monthly);
        let system = CoolingSystem::CompressionCooling { scop_cooling: 4.0 };
        let result = calculate_cooling(
            &zone(),
            &demand,
            &system,
            &CoolingDistribution::default(),
            &CoolingEmission::default(),
        )
        .unwrap();
        assert_abs_diff_eq!(result.annual_rencold_mj, 0.0, epsilon = 1e-9);
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

    #[test]
    fn negative_freecooling_factor_returns_error_not_negative_rencold() {
        // Ondergrens-validatie: `validate_system` (aangeroepen in
        // `calculate_cooling` vóór `compute_monthly_use`) weigert factor < 0, dus
        // de twee-termen-formule krijgt nooit een negatieve `factor` → geen
        // negatieve rencold. Dekt de DTO-invoer die via `map_cooling` binnenkomt.
        let demand = demand_with_cooling([0.0; 12]);
        let system = CoolingSystem::FreeCooling { factor: -0.1 };
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
