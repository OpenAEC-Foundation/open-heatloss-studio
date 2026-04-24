//! Maandelijkse Q_H;use berekening.
//!
//! Triviale helper, maar geïsoleerd voor toekomstbestendige uitbreiding
//! (bv. maand-afhankelijke SCOP, modulatie-effecten, etc.).

use nta8800_model::units::Energy;

/// Maandelijkse eindenergiegebruik Q_H;use;mi [MJ].
///
/// `q_h_nd` is in MJ (NTA 8800 convention), `total_eta` is het keten-product
/// η_em × η_dist × η_gen × f_reg. Voor warmtepomp kan dit > 1 zijn.
///
/// # Panics
///
/// Deze functie doet geen validatie; de caller moet zorgen dat
/// `total_eta > 0` en eindig is. [`crate::calculate_heating`] doet die
/// validatie centraal.
#[must_use]
pub fn monthly_q_h_use(q_h_nd: Energy, total_eta: f64) -> Energy {
    q_h_nd / total_eta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_division() {
        assert!((monthly_q_h_use(1_000.0, 0.9) - 1_111.111_111_111_1).abs() < 1e-6);
    }

    #[test]
    fn scop_greater_one_lowers_use() {
        // SCOP = 4 → Q_H;use < Q_H;nd
        assert!(monthly_q_h_use(1_000.0, 4.0) < 1_000.0);
    }

    #[test]
    fn zero_nd_gives_zero_use() {
        assert!((monthly_q_h_use(0.0, 0.85) - 0.0).abs() < 1e-12);
    }
}
