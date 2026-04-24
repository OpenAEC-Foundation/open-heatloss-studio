//! `EnergiefunctieRuimte` (EFR) — NTA 8800 §6.3.
//!
//! Een EFR beschrijft een cluster ruimten binnen een rekenzone met
//! éénduidig binnenklimaat-regime (gebruiksprofiel, temperatuur-setpoint,
//! gebruiksfunctie). Meerdere EFR's kunnen onder één rekenzone vallen.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::units::Area;
use crate::zoning::usage_function::UsageFunction;

/// Energiefunctieruimte volgens NTA 8800 §6.3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EnergiefunctieRuimte {
    /// Unieke identificatie.
    pub id: String,

    /// Menselijk leesbare naam (bv. `"Woonkamer + keuken"`).
    pub name: String,

    /// Id van de [`super::Rekenzone`] waartoe deze EFR behoort.
    pub rekenzone_id: String,

    /// Gebruiksoppervlak `A_g` van de EFR in m².
    pub floor_area: Area,

    /// Gebruiksfunctie van de EFR (kan afwijken van de primaire
    /// gebouwfunctie bij gemengde gebouwen).
    pub usage_function: UsageFunction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn efr_serde_round_trip() {
        let efr = EnergiefunctieRuimte {
            id: "efr1".into(),
            name: "Woonkamer".into(),
            rekenzone_id: "rz1".into(),
            floor_area: 34.5,
            usage_function: UsageFunction::Woonfunctie,
        };
        let json = serde_json::to_string(&efr).unwrap();
        let back: EnergiefunctieRuimte = serde_json::from_str(&json).unwrap();
        assert_eq!(efr, back);
    }

    #[test]
    fn efr_belongs_to_rekenzone() {
        let efr = EnergiefunctieRuimte {
            id: "efr-kantoor".into(),
            name: "Kantoorvleugel".into(),
            rekenzone_id: "rz-kantoor".into(),
            floor_area: 210.0,
            usage_function: UsageFunction::Kantoorfunctie,
        };
        assert_eq!(efr.rekenzone_id, "rz-kantoor");
    }
}
