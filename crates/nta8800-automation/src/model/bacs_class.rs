//! BAC-klassen volgens NEN-EN 15232.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Building Automation and Control (BAC) klasse volgens NEN-EN 15232.
///
/// Definieert het automatiseringsniveau van gebouwinstallaties met impact
/// op energie-efficiency. Klassen lopen van D (niet-energy-efficient) tot
/// A (high performance). Verwijzing: [`crate::references::NEN_EN_15232`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum BacsClass {
    /// Klasse A — High energy performance BACS.
    ///
    /// Geavanceerde automatisering met optimale regeling, voorspellende
    /// algoritmen, en uitgebreide sensorische feedback. Leidt tot
    /// energiebesparing t.o.v. standaard regeling (f_BAC < 1.0).
    A,

    /// Klasse B — Advanced BACS.
    ///
    /// Gevorderde automatisering met room-level regeling, tijdschema's,
    /// en basis-optimalisaties. Matige energiebesparing mogelijk
    /// (f_BAC ≈ 0.9-1.0).
    B,

    /// Klasse C — Standard BACS.
    ///
    /// Standaard regeling zonder geavanceerde optimalisaties. Referentie-
    /// niveau voor correctiefactoren (f_BAC ≈ 1.0). Meeste bestaande
    /// gebouwen vallen in deze categorie.
    C,

    /// Klasse D — Non energy efficient BACS.
    ///
    /// Verouderde of slecht afgestelde regeling die energie verspilt.
    /// Handmatige bediening, geen tijdschema's, slechte zonering.
    /// Leidt tot energieverlies (f_BAC > 1.0).
    D,
}

impl BacsClass {
    /// Geeft alle BAC-klassen terug in volgorde van beste naar slechtste efficiency.
    #[must_use]
    pub const fn all_ordered() -> [Self; 4] {
        [Self::A, Self::B, Self::C, Self::D]
    }

    /// Controleert of deze klasse beter is dan de referentieklasse C.
    #[must_use]
    pub const fn is_energy_saving(self) -> bool {
        matches!(self, Self::A | Self::B)
    }

    /// Controleert of deze klasse slechter is dan de referentieklasse C.
    #[must_use]
    pub const fn is_energy_wasting(self) -> bool {
        matches!(self, Self::D)
    }

    /// Geeft een beschrijvende naam voor de klasse.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::A => "High energy performance BACS",
            Self::B => "Advanced BACS",
            Self::C => "Standard BACS",
            Self::D => "Non energy efficient BACS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_uppercase() {
        let json = serde_json::to_string(&BacsClass::A).unwrap();
        assert_eq!(json, "\"A\"");
    }

    #[test]
    fn serde_round_trip_all_classes() {
        for class in BacsClass::all_ordered() {
            let json = serde_json::to_string(&class).unwrap();
            let back: BacsClass = serde_json::from_str(&json).unwrap();
            assert_eq!(class, back);
        }
    }

    #[test]
    fn energy_efficiency_classification() {
        assert!(BacsClass::A.is_energy_saving());
        assert!(BacsClass::B.is_energy_saving());
        assert!(!BacsClass::C.is_energy_saving());
        assert!(!BacsClass::D.is_energy_saving());

        assert!(!BacsClass::A.is_energy_wasting());
        assert!(!BacsClass::B.is_energy_wasting());
        assert!(!BacsClass::C.is_energy_wasting());
        assert!(BacsClass::D.is_energy_wasting());
    }

    #[test]
    fn ordering_contains_all_variants() {
        let ordered = BacsClass::all_ordered();
        assert_eq!(ordered.len(), 4);
        assert!(ordered.contains(&BacsClass::A));
        assert!(ordered.contains(&BacsClass::B));
        assert!(ordered.contains(&BacsClass::C));
        assert!(ordered.contains(&BacsClass::D));
    }

    #[test]
    fn descriptions_non_empty() {
        for class in BacsClass::all_ordered() {
            assert!(!class.description().is_empty());
        }
    }
}