//! Opwekkingsrendement η_gen — lookup of user-supplied.
//!
//! V1 implementeert 4 types; de volledige EN 15316-4-1 / bijlage Q mapping
//! is V2.

use crate::errors::HeatingCalcResult;
use crate::model::GenerationSystem;

/// Gevalideerd opwekkingsrendement η_gen (dimensieloos).
///
/// Voor warmtepomp is dit de SCOP en kan > 1 zijn. Voor alle andere
/// varianten in (0, 1].
///
/// # Errors
///
/// Zie [`GenerationSystem::efficiency`].
pub fn eta_gen(generation: &GenerationSystem) -> HeatingCalcResult<f64> {
    generation.efficiency()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::HRClass;

    #[test]
    fn eta_gen_hr107_is_0_95() {
        let eta = eta_gen(&GenerationSystem::HRBoiler {
            class: HRClass::HR107,
        })
        .unwrap();
        assert!((eta - 0.95).abs() < 1e-12);
    }

    #[test]
    fn eta_gen_electric_is_1_0() {
        let eta = eta_gen(&GenerationSystem::ElectricResistance).unwrap();
        assert!((eta - 1.0).abs() < 1e-12);
    }

    #[test]
    fn eta_gen_heat_pump_can_exceed_one() {
        let eta = eta_gen(&GenerationSystem::HeatPump { scop: 5.2 }).unwrap();
        assert!(eta > 1.0);
    }
}
