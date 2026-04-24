//! [`VentilationSystem`] — de 5 systeemvarianten uit NTA 8800 bijlage S.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Ventilatiesysteem-type conform NTA 8800:2025+C1:2026 §11.1 + bijlage S.
///
/// Let op: de norm benoemt **B als mechanische toevoer** en **C als mechanische
/// afvoer** (symbolen `SUPPLY_OP` resp. `EXTRACT_OP`). Dit is omgekeerd aan
/// sommige oudere bronnen (NEN 1087) — wij volgen NTA 8800 strikt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VentilationSystem {
    /// **Systeem A** — Natuurlijke ventilatie (geen mechanisch systeem).
    ///
    /// NTA 8800 symbool: `NATURAL_OP`. Luchtstromen via roosters, kieren en
    /// thermische trek.
    A,

    /// **Systeem B** — Mechanische toevoer, natuurlijke afvoer.
    ///
    /// NTA 8800 symbool: `SUPPLY_OP`. Ventilator blaast in, afvoer via
    /// natuurlijke openingen.
    B,

    /// **Systeem C** — Mechanische afvoer, natuurlijke toevoer.
    ///
    /// NTA 8800 symbool: `EXTRACT_OP`. Meest voorkomend in Nederlandse
    /// woningbouw vóór 2000. Ventilator zuigt af via keuken/badkamer.
    C,

    /// **Systeem D** — Balansventilatie (mechanische toe- én afvoer),
    /// optioneel met warmteterugwinning (WTW).
    ///
    /// NTA 8800 symbool: `BALANCED_OP`. Subvarianten D.1 t/m D.5b in
    /// bijlage S §2.4 verschillen in centrale/decentrale opstelling en
    /// sturingstype; dit model codeert alleen de WTW-aanwezigheid als knop.
    D {
        /// Of er WTW aanwezig is (subvariant D.2+ in bijlage S).
        with_wtw: bool,
    },

    /// **Systeem E** — Lokale balansventilatie met toevoer- en afvoerstromen.
    ///
    /// NTA 8800 symbool: `BALANCED-DEC_OP`. Decentrale WTW-units per ruimte.
    /// V1: volledig symmetrisch gemodelleerd als D met WTW.
    E,
}

impl VentilationSystem {
    /// Geeft aan of het systeem een mechanische toevoerventilator heeft.
    #[must_use]
    pub const fn has_mechanical_supply(&self) -> bool {
        matches!(self, Self::B | Self::D { .. } | Self::E)
    }

    /// Geeft aan of het systeem een mechanische afvoerventilator heeft.
    #[must_use]
    pub const fn has_mechanical_exhaust(&self) -> bool {
        matches!(self, Self::C | Self::D { .. } | Self::E)
    }

    /// Geeft aan of het systeem balansventilatie heeft (toevoer én afvoer).
    ///
    /// Alleen systeem D en E voldoen — voor deze systemen is WTW mogelijk.
    #[must_use]
    pub const fn is_balanced(&self) -> bool {
        matches!(self, Self::D { .. } | Self::E)
    }

    /// Factor `f_systype` uit NTA 8800 §11.4.3.3 (tabel onder formule 11.142):
    /// - A → 0 (geen mechanisch systeem)
    /// - B, C → 1 (één ventilator actief)
    /// - D, E → 2 (twee ventilatoren actief)
    #[must_use]
    pub const fn f_systype(&self) -> f64 {
        match self {
            Self::A => 0.0,
            Self::B | Self::C => 1.0,
            Self::D { .. } | Self::E => 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_a_has_no_mechanical_parts() {
        let sys = VentilationSystem::A;
        assert!(!sys.has_mechanical_supply());
        assert!(!sys.has_mechanical_exhaust());
        assert!(!sys.is_balanced());
        assert!((sys.f_systype() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn system_b_supply_only() {
        let sys = VentilationSystem::B;
        assert!(sys.has_mechanical_supply());
        assert!(!sys.has_mechanical_exhaust());
        assert!(!sys.is_balanced());
        assert!((sys.f_systype() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn system_c_exhaust_only() {
        let sys = VentilationSystem::C;
        assert!(!sys.has_mechanical_supply());
        assert!(sys.has_mechanical_exhaust());
        assert!(!sys.is_balanced());
        assert!((sys.f_systype() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn system_d_with_wtw_is_balanced() {
        let sys = VentilationSystem::D { with_wtw: true };
        assert!(sys.has_mechanical_supply());
        assert!(sys.has_mechanical_exhaust());
        assert!(sys.is_balanced());
        assert!((sys.f_systype() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn system_d_without_wtw_still_balanced() {
        let sys = VentilationSystem::D { with_wtw: false };
        assert!(sys.is_balanced());
    }

    #[test]
    fn system_e_like_d_with_wtw() {
        let sys = VentilationSystem::E;
        assert!(sys.is_balanced());
        assert!((sys.f_systype() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn serde_round_trip_system_a() {
        let sys = VentilationSystem::A;
        let json = serde_json::to_string(&sys).unwrap();
        let back: VentilationSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(sys, back);
    }

    #[test]
    fn serde_round_trip_system_d_with_wtw() {
        let sys = VentilationSystem::D { with_wtw: true };
        let json = serde_json::to_string(&sys).unwrap();
        let back: VentilationSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(sys, back);
    }
}
