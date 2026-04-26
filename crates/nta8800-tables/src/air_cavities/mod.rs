//! NTA 8800 bijlage F — equivalente warmtegeleidingscoëfficiënt van luchtlagen.
//!
//! V1 defaults — representatieve NL-praktijkwaarden, NIET de volledige NTA 8800 tabel.
//! Volledige tabel-overname volgt in V2. Validatie tegen de norm-tekst is open punt.
//!
//! Deze module bevat default λ-equivalent waarden voor verschillende luchtspouwtypes
//! volgens NTA 8800:2025+C1:2026 bijlage F. Luchtspouwen hebben een equivalente
//! warmtegeleidingscoëfficiënt die afhangt van:
//! - Dikte van de luchtspouw
//! - Mate van ventilatie (stil, zwak verlucht, sterk verlucht)
//!
//! # Structuur
//!
//! - [`AirCavityKind`] — type luchtspouw (dikte + ventilatie)
//! - [`AirCavityDefault`] — eigenschappen van één luchtspouwtype
//! - [`get_air_cavity`] — lookup functie op luchtspouwtype
//!
//! Referentie: [`NTA_8800_2025_BIJLAGE_F`](crate::references::NTA_8800_2025_BIJLAGE_F).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type luchtspouw voor λ-equivalent lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum AirCavityKind {
    /// Ongeventileerde spouw 5mm (dun glas-frame tussenruimte)
    Unventilated5mm,
    /// Ongeventileerde spouw 25mm (standaard spouw)
    Unventilated25mm,
    /// Zwak geventileerde spouw (enkele openingen)
    WeaklyVentilated,
    /// Sterk geventileerde spouw (continue ventilatie)
    StronglyVentilated,
}

/// Default eigenschappen van een luchtspouw volgens NTA 8800 bijlage F.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct AirCavityDefault {
    /// Type luchtspouw
    pub kind: AirCavityKind,
    /// Equivalente warmtegeleidingscoëfficiënt λ in W/(m·K)
    pub lambda_eq: f64,
}

/// Default luchtspouw eigenschappen uit NTA 8800 bijlage F.
///
/// V1-scope: 4 representatieve types voor Nederlandse bouwpraktijk.
const AIR_CAVITY_DEFAULTS: &[AirCavityDefault] = &[
    AirCavityDefault {
        kind: AirCavityKind::Unventilated5mm,
        lambda_eq: 0.025,
    },
    AirCavityDefault {
        kind: AirCavityKind::Unventilated25mm,
        lambda_eq: 0.15,
    },
    AirCavityDefault {
        kind: AirCavityKind::WeaklyVentilated,
        lambda_eq: 0.30,
    },
    AirCavityDefault {
        kind: AirCavityKind::StronglyVentilated,
        lambda_eq: 0.50,
    },
];

/// Zoek luchtspouw eigenschappen op type.
///
/// # Parameters
/// - `kind`: Type luchtspouw
///
/// # Returns
/// Eigenschappen van de luchtspouw, of `None` als het type niet gevonden wordt.
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::air_cavities::{get_air_cavity, AirCavityKind};
///
/// let cavity = get_air_cavity(AirCavityKind::Unventilated25mm)
///     .expect("Standaard spouw moet bestaan");
/// assert_eq!(cavity.lambda_eq, 0.15);
/// ```
#[must_use]
pub fn get_air_cavity(kind: AirCavityKind) -> Option<&'static AirCavityDefault> {
    AIR_CAVITY_DEFAULTS.iter().find(|c| c.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_cavity_types_have_data() {
        assert!(get_air_cavity(AirCavityKind::Unventilated5mm).is_some());
        assert!(get_air_cavity(AirCavityKind::Unventilated25mm).is_some());
        assert!(get_air_cavity(AirCavityKind::WeaklyVentilated).is_some());
        assert!(get_air_cavity(AirCavityKind::StronglyVentilated).is_some());
    }

    #[test]
    fn lambda_values_are_plausible() {
        for default in AIR_CAVITY_DEFAULTS {
            assert!(
                default.lambda_eq > 0.0 && default.lambda_eq < 1.0,
                "λ-equivalent {} voor {:?} valt buiten plausibel bereik 0-1 W/(m·K)",
                default.lambda_eq,
                default.kind
            );
        }
    }

    #[test]
    fn ventilated_cavities_have_higher_lambda() {
        let unventilated = get_air_cavity(AirCavityKind::Unventilated25mm).unwrap();
        let weak = get_air_cavity(AirCavityKind::WeaklyVentilated).unwrap();
        let strong = get_air_cavity(AirCavityKind::StronglyVentilated).unwrap();

        assert!(
            unventilated.lambda_eq < weak.lambda_eq,
            "Zwak geventileerd moet hoger zijn dan ongeventileerd"
        );
        assert!(
            weak.lambda_eq < strong.lambda_eq,
            "Sterk geventileerd moet hoger zijn dan zwak geventileerd"
        );
    }
}
