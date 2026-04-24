//! [`WtwSpecification`] — warmteterugwin-unit parameters.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// WTW-specificatie voor een gebalanceerd ventilatiesysteem (D of E).
///
/// Conform NTA 8800 §11.3.2.2 — η_hr wordt ofwel uit een
/// BCRG-kwaliteitsverklaring (NEN-EN 13141-7, 13141-8, 13142, 13053) of uit
/// tabel 11.18 genomen. In V1 accepteren we de fabrikantwaarde zonder
/// correctie voor praktijkprestatie (f_prac;hr) — dat is een expliciete
/// vereenvoudiging.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WtwSpecification {
    /// Effectief warmterendement η_hr, dimensieloos, in `[0, 1]`.
    ///
    /// Fabrikantwaarde (of forfaitair uit tabel 11.18). V1 past **geen**
    /// praktijkprestatiefactor `f_prac;hr` toe — dat is V2-scope.
    pub efficiency: f64,

    /// Specifiek ventilator-vermogen f_SFP in **W/(m³/h)** (NTA 8800 eenheid,
    /// tabel 11.23).
    ///
    /// Let op: sommige fabrikanten geven SFP in W/(m³/s) — deel in dat geval
    /// door 3600 voordat je de waarde hier invult. Typische moderne units
    /// 2026: ~0,125 W/(m³/h) = 0,45 W/(m³/s) (tabel 11.23, y > 2006, DC).
    pub fan_sfp: f64,

    /// Of een 100%-bypass actief is bij hoge buitentemperatuur.
    ///
    /// V1: louter documentair — de bypass-logica (η_hr → 0 bij
    /// T_buiten > 18 °C) is expliciet V2-scope. Tijdens een warmtebehoefte
    /// doet bypass er sowieso niet toe.
    pub bypass_enabled: bool,
}

impl WtwSpecification {
    /// Bouw een `WtwSpecification` zonder validatie — validatie gebeurt in
    /// [`crate::calculate_ventilation`].
    #[must_use]
    pub const fn new(efficiency: f64, fan_sfp: f64, bypass_enabled: bool) -> Self {
        Self {
            efficiency,
            fan_sfp,
            bypass_enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let wtw = WtwSpecification::new(0.80, 0.45 / 3.6, true);
        let json = serde_json::to_string(&wtw).unwrap();
        let back: WtwSpecification = serde_json::from_str(&json).unwrap();
        assert_eq!(wtw, back);
    }

    #[test]
    fn typical_modern_dc_unit_2026() {
        // Tabel 11.23: y > 2006, DC-ventilatoren → f_SFP = 0,45 / 3,6 W/(m³/h)
        let wtw = WtwSpecification::new(0.85, 0.45 / 3.6, true);
        assert!((wtw.fan_sfp - 0.125).abs() < 1e-3);
    }
}
