//! EP-label toewijzing volgens NTA 8800:2025.
//!
//! Implementeert label-classificatie op basis van specifiek primair energiegebruik
//! en gebruiksfunctie conform tabellen 5.1 en 5.2.

use nta8800_model::zoning::UsageFunction;

use crate::{EpError, EpLabel};

/// Wijst EP-label toe op basis van specifiek primair energiegebruik en gebruiksfunctie.
///
/// Implementeert label-drempels conform:
/// - NTA 8800:2025 tabel 5.1 voor woonfuncties
/// - NTA 8800:2025 tabel 5.2 voor utiliteitsgebouwen
///
/// **Label-drempels (2023 waarden):**
///
/// ### Woonfunctie [MJ/m²]:
/// - A++++: ≤ 100
/// - A+++: ≤ 150
/// - A++: ≤ 200
/// - A+: ≤ 250
/// - A: ≤ 300
/// - B: ≤ 400
/// - C: ≤ 500
/// - D: ≤ 650
/// - E: ≤ 800
/// - F: ≤ 1000
/// - G: > 1000
///
/// ### Kantoorfunctie en andere utiliteit [MJ/m²]:
/// - A++++: ≤ 200
/// - A+++: ≤ 300
/// - A++: ≤ 400
/// - A+: ≤ 500
/// - A: ≤ 600
/// - B: ≤ 750
/// - C: ≤ 900
/// - D: ≤ 1100
/// - E: ≤ 1300
/// - F: ≤ 1600
/// - G: > 1600
///
/// **V1 vereenvoudigingen:**
/// - Alle niet-woon functies gebruiken kantoorfunctie-drempels als fallback
/// - Geen subsector-specifieke drempels voor gezondheidszorg, onderwijs, etc.
///
/// # Errors
///
/// Retourneert [`EpError::UnknownEpLabel`] bij extreme waarden buiten bereik
/// (theoretisch niet mogelijk met huidige drempels, maar defensief programmeren).
///
/// # Voorbeeld
///
/// ```
/// # use nta8800_ep::calc::label::assign_label;
/// # use nta8800_model::zoning::UsageFunction;
/// # use nta8800_ep::EpLabel;
/// // Woning met 180 MJ/m²
/// let label = assign_label(180.0, UsageFunction::Woonfunctie)?;
/// assert_eq!(label, EpLabel::Aplus2); // A++
///
/// // Kantoor met 350 MJ/m²
/// let label = assign_label(350.0, UsageFunction::Kantoorfunctie)?;
/// assert_eq!(label, EpLabel::Aplus2); // A++
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn assign_label(ep_score_mj_per_m2: f64, function: UsageFunction) -> Result<EpLabel, EpError> {
    if ep_score_mj_per_m2 < 0.0 {
        return Err(EpError::UnknownEpLabel { ep_score_mj_per_m2 });
    }

    match function {
        UsageFunction::Woonfunctie => assign_label_residential(ep_score_mj_per_m2),
        // V1: alle utiliteit gebruikt kantoorfunctie-drempels
        _ => assign_label_non_residential(ep_score_mj_per_m2),
    }
}

/// Label-toewijzing voor woonfuncties volgens tabel 5.1.
#[allow(clippy::unnecessary_wraps)] // Result-symmetrie met publieke assign_label
fn assign_label_residential(ep_score: f64) -> Result<EpLabel, EpError> {
    let label = if ep_score <= 100.0 {
        EpLabel::Aplus4
    } else if ep_score <= 150.0 {
        EpLabel::Aplus3
    } else if ep_score <= 200.0 {
        EpLabel::Aplus2
    } else if ep_score <= 250.0 {
        EpLabel::Aplus
    } else if ep_score <= 300.0 {
        EpLabel::A
    } else if ep_score <= 400.0 {
        EpLabel::B
    } else if ep_score <= 500.0 {
        EpLabel::C
    } else if ep_score <= 650.0 {
        EpLabel::D
    } else if ep_score <= 800.0 {
        EpLabel::E
    } else if ep_score <= 1000.0 {
        EpLabel::F
    } else {
        EpLabel::G
    };

    Ok(label)
}

/// Label-toewijzing voor utiliteitsgebouwen volgens tabel 5.2 (kantoorfunctie-drempels).
#[allow(clippy::unnecessary_wraps)] // Result-symmetrie met publieke assign_label
fn assign_label_non_residential(ep_score: f64) -> Result<EpLabel, EpError> {
    let label = if ep_score <= 200.0 {
        EpLabel::Aplus4
    } else if ep_score <= 300.0 {
        EpLabel::Aplus3
    } else if ep_score <= 400.0 {
        EpLabel::Aplus2
    } else if ep_score <= 500.0 {
        EpLabel::Aplus
    } else if ep_score <= 600.0 {
        EpLabel::A
    } else if ep_score <= 750.0 {
        EpLabel::B
    } else if ep_score <= 900.0 {
        EpLabel::C
    } else if ep_score <= 1100.0 {
        EpLabel::D
    } else if ep_score <= 1300.0 {
        EpLabel::E
    } else if ep_score <= 1600.0 {
        EpLabel::F
    } else {
        EpLabel::G
    };

    Ok(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residential_label_thresholds() {
        // Test exact drempel waarden voor woonfunctie
        assert_eq!(
            assign_label(100.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::Aplus4
        );
        assert_eq!(
            assign_label(150.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::Aplus3
        );
        assert_eq!(
            assign_label(200.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::Aplus2
        );
        assert_eq!(
            assign_label(250.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::Aplus
        );
        assert_eq!(
            assign_label(300.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::A
        );
        assert_eq!(
            assign_label(400.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::B
        );
        assert_eq!(
            assign_label(500.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::C
        );
        assert_eq!(
            assign_label(650.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::D
        );
        assert_eq!(
            assign_label(800.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::E
        );
        assert_eq!(
            assign_label(1000.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::F
        );
        assert_eq!(
            assign_label(1001.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::G
        );
    }

    #[test]
    fn test_non_residential_label_thresholds() {
        // Test kantoorfunctie drempels
        assert_eq!(
            assign_label(200.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::Aplus4
        );
        assert_eq!(
            assign_label(300.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::Aplus3
        );
        assert_eq!(
            assign_label(400.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::Aplus2
        );
        assert_eq!(
            assign_label(500.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::Aplus
        );
        assert_eq!(
            assign_label(600.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::A
        );
        assert_eq!(
            assign_label(750.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::B
        );
        assert_eq!(
            assign_label(900.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::C
        );
        assert_eq!(
            assign_label(1100.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::D
        );
        assert_eq!(
            assign_label(1300.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::E
        );
        assert_eq!(
            assign_label(1600.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::F
        );
        assert_eq!(
            assign_label(1601.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::G
        );
    }

    #[test]
    fn test_all_non_residential_use_office_thresholds() {
        let non_residential_functions = [
            UsageFunction::Bijeenkomstfunctie,
            UsageFunction::Celfunctie,
            UsageFunction::Gezondheidszorgfunctie,
            UsageFunction::Industriefunctie,
            UsageFunction::Kantoorfunctie,
            UsageFunction::Logiesfunctie,
            UsageFunction::Onderwijsfunctie,
            UsageFunction::Sportfunctie,
            UsageFunction::Winkelfunctie,
            UsageFunction::OverigeGebruiksfunctie,
        ];

        for function in non_residential_functions {
            // Alle utiliteit moet zelfde drempels hebben (kantoorfunctie)
            assert_eq!(
                assign_label(350.0, function).unwrap(),
                EpLabel::Aplus2, // A++ voor 350 MJ/m² utiliteit
                "Functie {function:?} gebruikt niet kantoorfunctie-drempels"
            );
        }
    }

    #[test]
    fn test_intermediate_values() {
        // Test waarden tussen drempels
        assert_eq!(
            assign_label(175.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::Aplus2
        ); // Tussen 150-200
        assert_eq!(
            assign_label(450.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::Aplus
        ); // Tussen 400-500
    }

    #[test]
    fn test_negative_ep_score() {
        assert!(matches!(
            assign_label(-100.0, UsageFunction::Woonfunctie),
            Err(EpError::UnknownEpLabel {
                ep_score_mj_per_m2: -100.0
            })
        ));
    }

    #[test]
    fn test_zero_ep_score() {
        // Nul energiegebruik krijgt beste label
        assert_eq!(
            assign_label(0.0, UsageFunction::Woonfunctie).unwrap(),
            EpLabel::Aplus4
        );
        assert_eq!(
            assign_label(0.0, UsageFunction::Kantoorfunctie).unwrap(),
            EpLabel::Aplus4
        );
    }
}
