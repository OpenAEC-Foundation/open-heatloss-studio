//! Distributie-verlies berekening.
//!
//! V1 gebruikt één lineair η_dist. De volledige §9.4 methodiek
//! (leiding-lengte × U × ΔT × tijd) is V2.

use crate::model::DistributionSystem;

/// Extractie van η_dist (0 < η ≤ 1).
#[must_use]
pub fn eta_dist(distribution: &DistributionSystem) -> f64 {
    distribution.efficiency
}

/// Verlies-aandeel 1 − η_dist (dimensieloos).
#[must_use]
pub fn loss_fraction(distribution: &DistributionSystem) -> f64 {
    1.0 - eta_dist(distribution)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_insulated_loss_fraction_0_05() {
        let d = DistributionSystem::default_insulated();
        assert!((loss_fraction(&d) - 0.05).abs() < 1e-12);
    }

    #[test]
    fn uninsulated_loss_fraction_0_20() {
        let d = DistributionSystem::uninsulated();
        assert!((loss_fraction(&d) - 0.20).abs() < 1e-12);
    }
}
