//! Afgifte-verlies berekening.
//!
//! V1 vereenvoudigt het volledige ΔT-model van NTA 8800 §9.3 (tabel 9.2) tot
//! een enkel η_em per type. Deze module biedt toekomstbestendige helpers
//! die in V2 de ΔT-correcties kunnen gaan berekenen.

use crate::model::EmissionSystem;

/// Extractie van het afgifterendement η_em (dimensieloos, in (0, 1]).
///
/// Wrapper rond [`EmissionSystem::default_efficiency`]. Bestaat als
/// standalone functie voor symmetrie met `distribution_loss` en
/// `generation_efficiency`.
#[must_use]
pub fn eta_em(emission: EmissionSystem) -> f64 {
    emission.default_efficiency()
}

/// Bereken het verlies-aandeel voor afgifte (1 − η_em), dimensieloos.
///
/// Gebruikt in rapportage om de relatieve bijdrage van afgifte-verliezen aan
/// de totale keten-verliezen te kunnen weergeven.
#[must_use]
pub fn loss_fraction(emission: EmissionSystem) -> f64 {
    1.0 - eta_em(emission)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loss_plus_eta_equals_one() {
        for s in [
            EmissionSystem::RadiatorHighTemp,
            EmissionSystem::RadiatorLowTemp,
            EmissionSystem::FloorHeating,
            EmissionSystem::AirHeating,
            EmissionSystem::RadiantPanel,
        ] {
            assert!((loss_fraction(s) + eta_em(s) - 1.0).abs() < 1e-12);
        }
    }
}
