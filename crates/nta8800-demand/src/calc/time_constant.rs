//! Tijdconstante τ van een rekenzone.
//!
//! NTA 8800 §7.8, formule (7.17):
//!
//! ```text
//! τ = (C_m;int;eff;zi / 3600) / (H_tr + H_ve)   [h]
//! ```
//!
//! Met `C_m;int;eff;zi` in J/K (zie [`nta8800_tables::thermal_capacity::zone_heat_capacity`]),
//! factor 3600 converteert J/K → Wh/K zodat de noemer (W/K) samen met de
//! teller (Wh/K) een resultaat in uren geeft.

use nta8800_tables::thermal_capacity::zone_heat_capacity;

use crate::errors::{DemandCalcResult, DemandError};
use crate::model::ThermalMassInput;

/// Seconden per uur — conversie J → Wh in formule (7.17).
pub const SECONDS_PER_HOUR: f64 = 3600.0;

/// Bereken de tijdconstante τ in uren.
///
/// # Parameters
///
/// - `mass` — thermische-massa classificatie (tabel 7.10/7.11/7.12)
/// - `floor_area_m2` — `A_g;zi` in m²
/// - `h_tr` — totale transmissie-coëfficiënt (H_D + H_U + H_g;an + H_A) in W/K
/// - `h_ve` — ventilatie-coëfficiënt in W/K
///
/// # Errors
///
/// - [`DemandError::InvalidFloorArea`] als `floor_area_m2 ≤ 0` of niet-eindig
/// - [`DemandError::NonPositiveConductance`] als `h_tr + h_ve ≤ 0` of
///   niet-eindig — τ zou dan oneindig of negatief worden.
pub fn time_constant_hours(
    mass: &ThermalMassInput,
    floor_area_m2: f64,
    h_tr: f64,
    h_ve: f64,
) -> DemandCalcResult<f64> {
    if !floor_area_m2.is_finite() || floor_area_m2 <= 0.0 {
        return Err(DemandError::InvalidFloorArea { floor_area_m2 });
    }
    let total = h_tr + h_ve;
    if !total.is_finite() || total <= 0.0 {
        return Err(DemandError::NonPositiveConductance {
            total_conductance: total,
        });
    }
    // C_m in J/K, /3600 → Wh/K, /(W/K) → h
    let c_m_j_per_k = zone_heat_capacity(mass.floor, mass.wall, mass.ceiling, floor_area_m2);
    Ok((c_m_j_per_k / SECONDS_PER_HOUR) / total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lichte_woning_100m2_typische_tau() {
        // D_m = 55 kJ/(m²·K) → C_m = 5.5 MJ/K = 5.5e6 J/K
        // τ = (5.5e6 / 3600) / (150 + 50) = 1527.78 / 200 = 7.64 h
        let mass = ThermalMassInput::light_woning();
        let tau = time_constant_hours(&mass, 100.0, 150.0, 50.0).unwrap();
        assert!((tau - 7.638_888_888_888).abs() < 1e-6);
    }

    #[test]
    fn zware_woning_100m2_hogere_tau() {
        let light = ThermalMassInput::light_woning();
        let heavy = ThermalMassInput::zwaar_massief();
        let tau_light = time_constant_hours(&light, 100.0, 150.0, 50.0).unwrap();
        let tau_heavy = time_constant_hours(&heavy, 100.0, 150.0, 50.0).unwrap();
        assert!(
            tau_heavy > tau_light * 5.0,
            "zware zone moet veel hogere τ hebben: heavy={tau_heavy}, light={tau_light}"
        );
    }

    #[test]
    fn nul_floor_area_geeft_error() {
        let mass = ThermalMassInput::light_woning();
        let err = time_constant_hours(&mass, 0.0, 100.0, 50.0).unwrap_err();
        assert!(matches!(err, DemandError::InvalidFloorArea { .. }));
    }

    #[test]
    fn nul_conductance_geeft_error() {
        let mass = ThermalMassInput::light_woning();
        let err = time_constant_hours(&mass, 100.0, 0.0, 0.0).unwrap_err();
        assert!(matches!(err, DemandError::NonPositiveConductance { .. }));
    }

    #[test]
    fn grotere_zone_geeft_hogere_tau() {
        // A dubbel → C_m dubbel → τ dubbel (bij constante H_tr/H_ve)
        let mass = ThermalMassInput::light_woning();
        let t1 = time_constant_hours(&mass, 100.0, 150.0, 50.0).unwrap();
        let t2 = time_constant_hours(&mass, 200.0, 150.0, 50.0).unwrap();
        assert!((t2 - 2.0 * t1).abs() < 1e-6);
    }

    #[test]
    fn hogere_h_verlaagt_tau() {
        let mass = ThermalMassInput::light_woning();
        let t1 = time_constant_hours(&mass, 100.0, 150.0, 50.0).unwrap();
        let t2 = time_constant_hours(&mass, 100.0, 300.0, 100.0).unwrap();
        // Dubbele totale H → halve τ
        assert!((t2 - 0.5 * t1).abs() < 1e-6);
    }
}
