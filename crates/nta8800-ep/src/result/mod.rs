//! Resultaat-types voor EP-score berekeningen.

use serde::{Deserialize, Serialize};

/// EP-label classificatie volgens NTA 8800:2025.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EpLabel {
    /// A++++ — Zeer hoge energieprestatie.
    #[serde(rename = "A++++")]
    Aplus4,

    /// A+++ — Hoge energieprestatie.
    #[serde(rename = "A+++")]
    Aplus3,

    /// A++ — Goede energieprestatie.
    #[serde(rename = "A++")]
    Aplus2,

    /// A+ — Redelijke energieprestatie.
    #[serde(rename = "A+")]
    Aplus,

    /// A — Standaard energieprestatie.
    A,

    /// B — Matige energieprestatie.
    B,

    /// C — Lage energieprestatie.
    C,

    /// D — Zeer lage energieprestatie.
    D,

    /// E — Slechte energieprestatie.
    E,

    /// F — Zeer slechte energieprestatie.
    F,

    /// G — Slechtst mogelijke energieprestatie.
    G,
}

impl EpLabel {
    /// Geeft de label-string zoals gebruikt in rapportage.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Aplus4 => "A++++",
            Self::Aplus3 => "A+++",
            Self::Aplus2 => "A++",
            Self::Aplus => "A+",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
        }
    }

    /// Geeft alle mogelijke EP-labels in volgorde van best naar slechtst.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Aplus4,
            Self::Aplus3,
            Self::Aplus2,
            Self::Aplus,
            Self::A,
            Self::B,
            Self::C,
            Self::D,
            Self::E,
            Self::F,
            Self::G,
        ]
    }
}

/// Energiegebruik en CO2-uitstoot per service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceBreakdown {
    /// Primair energiegebruik voor deze dienst [MJ].
    pub primary_energy_mj: f64,

    /// CO2-uitstoot voor deze dienst [kg].
    pub co2_kg: f64,

    /// Aandeel hernieuwbare energie voor deze dienst [0.0-1.0].
    pub renewable_fraction: f64,
}

/// Resultaat van EP-score berekening.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpResult {
    /// EP-label (A++++ t/m G) gebaseerd op specifiek primair energiegebruik.
    pub ep_label: EpLabel,

    /// Totaal primair energiegebruik [MJ].
    pub ep_total_mj: f64,

    /// Specifiek primair energiegebruik [MJ/m²] — basis voor EP-label.
    pub ep_total_mj_per_m2: f64,

    /// Hernieuwbaar aandeel totaal [0.0-1.0].
    pub ep_renewable_share: f64,

    /// Totale CO2-uitstoot per m² [kg/m²].
    pub ep_co2_kg_per_m2: f64,

    /// Breakdown per energiedienst.
    pub breakdown: EpBreakdown,
}

/// Gedetailleerde breakdown per energiedienst.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpBreakdown {
    /// Verwarming — space heating.
    pub heating: ServiceBreakdown,

    /// Koeling — space cooling.
    pub cooling: ServiceBreakdown,

    /// Warmtapwater — domestic hot water.
    pub dhw: ServiceBreakdown,

    /// Verlichting — artificial lighting.
    pub lighting: ServiceBreakdown,

    /// Ventilatie hulpenergie — auxiliary energy for ventilation.
    pub ventilation_aux: ServiceBreakdown,

    /// Gebouwautomatisering — building automation and control systems.
    pub automation: ServiceBreakdown,

    /// PV-opbrengst — on-site renewable energy generation.
    ///
    /// **Opmerking:** Negatieve waarden voor primary_energy_mj en co2_kg
    /// geven netto energieproductie aan.
    pub pv: ServiceBreakdown,
}
