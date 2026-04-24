//! Niet-transparante openingen (deuren, vaste panelen).
//!
//! Afwijkend van [`super::Window`]: geen g-waarde, geen kozijnfractie —
//! opaque element met alleen een U-waarde, oppervlakte, oriëntatie en helling.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::location::{Orientation, Tilt};
use crate::units::{Area, ThermalTransmittance};

/// Niet-transparante gevelopening (bv. buitendeur, vast paneel).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Opening {
    /// Unieke identificatie binnen het project.
    pub id: String,

    /// Bruto oppervlakte in m².
    pub area: Area,

    /// U-waarde in W/(m²·K).
    pub u_value: ThermalTransmittance,

    /// Oriëntatie van het vlak.
    pub orientation: Orientation,

    /// Helling van het vlak.
    pub tilt: Tilt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_serde_round_trip() {
        let op = Opening {
            id: "o1".into(),
            area: 2.1,
            u_value: 1.4,
            orientation: Orientation::Noord,
            tilt: Tilt::VERTICAL,
        };
        let json = serde_json::to_string(&op).unwrap();
        let back: Opening = serde_json::from_str(&json).unwrap();
        assert_eq!(op, back);
    }

    #[test]
    fn opening_construct_sets_fields() {
        let op = Opening {
            id: "deur-achter".into(),
            area: 1.8,
            u_value: 1.6,
            orientation: Orientation::West,
            tilt: Tilt::VERTICAL,
        };
        assert_eq!(op.orientation, Orientation::West);
        assert!((op.u_value - 1.6).abs() < 1e-9);
    }
}
