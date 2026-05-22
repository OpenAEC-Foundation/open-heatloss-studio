//! Ventilatiesysteem-correctiefactor f_inf voor infiltratie.
//!
//! Bron: ISSO 53 (2016) tabel 4.7, PDF p.46-47.
//!
//! f_inf corrigeert de invloed van de ventilatievoorziening op de
//! infiltratie. Gebruikt in formule 4.31:
//! `q_is = f_wind · f_type · f_inf · (0,23 · q_i,spec)`.

use crate::model::enums::VentilationSystemType;

/// Correctiefactor f_inf voor de invloed van het ventilatiesysteem op de
/// infiltratie. ISSO 53 tabel 4.7 (PDF p.46-47), dimensieloos.
///
/// - A — natuurlijke toe- en afvoer → 0,80;
/// - B — mechanische toevoer + natuurlijke afvoer → 0,85;
/// - C — natuurlijke toevoer + mechanische afvoer → 1,0;
/// - D — gebalanceerde mechanische toe- en afvoer → 1,15;
/// - E — zones met natuurlijke toevoer + mechanische afvoer en zones met
///   lokale WTW (CO₂-sturing op afvoer) → 1,08.
pub fn f_inf(system: VentilationSystemType) -> f64 {
    match system {
        VentilationSystemType::SystemA => 0.80,
        VentilationSystemType::SystemB => 0.85,
        VentilationSystemType::SystemC => 1.0,
        VentilationSystemType::SystemD => 1.15,
        VentilationSystemType::SystemE => 1.08,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f_inf_values() {
        assert_eq!(f_inf(VentilationSystemType::SystemA), 0.80);
        assert_eq!(f_inf(VentilationSystemType::SystemB), 0.85);
        assert_eq!(f_inf(VentilationSystemType::SystemC), 1.0);
        assert_eq!(f_inf(VentilationSystemType::SystemD), 1.15);
        assert_eq!(f_inf(VentilationSystemType::SystemE), 1.08);
    }
}
