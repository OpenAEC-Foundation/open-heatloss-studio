//! Maandelijkse Q_W;use berekening.
//!
//! Triviale helper, maar geïsoleerd voor toekomstbestendige uitbreiding
//! (bv. maand-afhankelijke SCOP_W, warme/koude bron-temperatuur variant,
//! bijlage T tappatroon-detail).

use nta8800_model::units::Energy;

/// Maandelijkse eindenergiegebruik `Q_W;use;mi` [MJ].
///
/// `q_w_nd_net` is de netto vraag na DWTW-aftrek, in MJ. `total_eta` is het
/// keten-product η_W;em × η_W;dis × η_W;gen. Voor tapwater-warmtepompen kan
/// dit > 1 zijn.
///
/// # Panics
///
/// Deze functie doet geen validatie; de caller moet zorgen dat
/// `total_eta > 0` en eindig is. [`crate::calculate_dhw`] doet die validatie
/// centraal.
#[must_use]
pub fn monthly_q_w_use(q_w_nd_net: Energy, total_eta: f64) -> Energy {
    q_w_nd_net / total_eta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_division() {
        assert!((monthly_q_w_use(1_000.0, 0.8) - 1_250.0).abs() < 1e-6);
    }

    #[test]
    fn heat_pump_scop_greater_one_lowers_use() {
        // SCOP_W = 2,5 → Q_W;use < Q_W;nd
        assert!(monthly_q_w_use(1_000.0, 2.5) < 1_000.0);
    }

    #[test]
    fn zero_nd_gives_zero_use() {
        assert!((monthly_q_w_use(0.0, 0.85) - 0.0).abs() < 1e-12);
    }
}
