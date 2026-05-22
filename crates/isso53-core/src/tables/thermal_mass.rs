//! Specifieke effectieve opslagcapaciteit c_eff per thermische massa.
//!
//! Bron: ISSO 53 (2016) tabel 2.4, PDF p.24.
//!
//! `c_eff` wordt gebruikt om de effectieve opslagcapaciteit van het gebouw
//! te benaderen wanneer niet alle wandgegevens beschikbaar zijn:
//! `C_eff = c_eff · V` (formule 2.12), met V de inhoud op buitenafmetingen.

use crate::model::enums::ThermalMass;

/// Specifieke effectieve opslagcapaciteit c_eff in Wh/(m³·K).
/// ISSO 53 tabel 2.4 (PDF p.24).
///
/// - `Licht` → 15 (lichte daken/wanden, verlaagde plafonds, verhoogde vloeren);
/// - `Gemiddeld` → 50 (steenachtige buitenwanden, betonnen vloeren/plafonds);
/// - `Zwaar` → 75 (wanden ρ ≥ 1500 kg/m³, betonnen vloeren/plafonds).
pub fn c_eff(mass: ThermalMass) -> f64 {
    match mass {
        ThermalMass::Licht => 15.0,
        ThermalMass::Gemiddeld => 50.0,
        ThermalMass::Zwaar => 75.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_eff_values() {
        assert_eq!(c_eff(ThermalMass::Licht), 15.0);
        assert_eq!(c_eff(ThermalMass::Gemiddeld), 50.0);
        assert_eq!(c_eff(ThermalMass::Zwaar), 75.0);
    }
}
