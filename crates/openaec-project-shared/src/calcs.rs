//! Per-calc inputs container.
//!
//! Elk veld is `Option<T>` zodat een project één of meerdere calcs kan
//! activeren door het bijbehorende veld te vullen. Een lege [`Calcs`] is
//! een vers project zonder berekening-keuze.
//!
//! ## Iso51Inputs / TojuliInputs design
//!
//! In V2 zijn deze structs **placeholders** met de specifieke (niet-uit-
//! shared-of-geometry-afleidbare) input-velden per norm. Calc-crates
//! definiëren hun eigen view-mapper die [`SharedProject`] +
//! [`SharedGeometry`] + de specifieke inputs combineert tot hun runtime
//! struct.
//!
//! [`SharedProject`]: crate::shared::SharedProject
//! [`SharedGeometry`]: crate::geometry::SharedGeometry

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Per-calc inputs. Open-ended container — nieuwe calcs worden hier
/// toegevoegd zonder `ProjectV2` schema-breuk.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Calcs {
    /// ISSO 51 warmteverlies-inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isso51: Option<Iso51Inputs>,

    /// TO-juli (NTA 8800 H.10 + bijlage AA) inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tojuli: Option<TojuliInputs>,

    /// ISSO 53 warmteverlies-inputs (utiliteitsgebouwen ≤ 4m).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isso53: Option<Iso53Inputs>,
}

/// ISSO 51-specifieke inputs die niet uit `shared`/`geometry` afgeleid
/// kunnen worden.
///
/// V2 placeholder — concrete velden worden vastgelegd in F3 wanneer de
/// view-mapper [`isso51_core::model::Project`] gebouwd wordt.
/// Voor nu: gebufferd JSON-blob van legacy V1 [`Project`] zonder geometry
/// (rooms zit in [`SharedGeometry`]).
///
/// [`isso51_core::model::Project`]: isso51_core::model::Project
/// [`Project`]: isso51_core::model::Project
/// [`SharedGeometry`]: crate::geometry::SharedGeometry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Iso51Inputs {
    /// Volledige legacy V1 Project JSON (transitional). F6 zal dit
    /// uitsplitsen in concrete velden zodra geometry mapping live is.
    #[serde(flatten)]
    pub legacy_v1: serde_json::Value,
}

/// TO-juli specifieke inputs (NTA 8800 H.10 / bijlage AA).
///
/// V2 placeholder — wordt in F7 ingevuld met cooling system + distribution
/// + emission + blinds_strategy + control_regime, geserialiseerd parallel
///   aan `nta8800_cooling::model::CoolingSystem` etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TojuliInputs {
    /// Optionele "quick check" mode (bijlage AA, woningen only) — bevat
    /// de 12-veld request van de huidige `/tojuli/quick` MVP voor
    /// retro-compat. Volledig H.10-pad komt apart als
    /// `Option<TojuliH10Inputs>` in F7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quick_check: Option<serde_json::Value>,
}

/// ISSO 53-specifieke inputs.
///
/// V2 placeholder, parallel aan Iso51Inputs. Bevat de niet-uit-shared/geometry-
/// afleidbare velden: gebruiksfunctie + ruimtetype per ruimte, ventilatiesysteem
/// (A-E met E voor ISSO 53), HeatingUpConfig, en project-niveau Building-velden
/// (wind_pressure_type, etc).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Iso53Inputs {
    /// Volledige ISSO 53 project-JSON (transitional, identiek pattern aan Iso51Inputs.legacy_v1).
    #[serde(flatten)]
    pub legacy: serde_json::Value,
}

impl Calcs {
    /// Welke calc is actief? Geeft de "primaire" calc terug —
    /// die met daadwerkelijke input, ISSO 51 als default fallback.
    pub fn active_norm(&self) -> ActiveNorm {
        match (self.isso51.is_some(), self.isso53.is_some()) {
            (false, true) => ActiveNorm::Isso53,
            _ => ActiveNorm::Isso51,  // default + multi-calc fallback
        }
    }
}

/// De norm die actief is voor een ProjectV2.
///
/// Geeft de UI/CLI/PDF-generator aan welke berekening primair is. Een
/// project kan technisch beide hebben (multi-calc), maar UI dwingt nu één
/// keuze bij aanmaak; `active_norm()` is de single source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ActiveNorm {
    /// ISSO 51 — warmteverliesberekening voor woningen.
    Isso51,
    /// ISSO 53 — warmteverliesberekening voor utiliteitsgebouwen (≤ 4m).
    Isso53,
    /// BENG — NTA 8800 energieprestatie (woningbouw).
    ///
    /// De BENG-invoer leeft in [`ProjectV2::energy`], niet in [`Calcs`];
    /// [`Calcs::active_norm`] leidt deze variant daarom **niet** af uit de
    /// aanwezige calc-inputs. De variant bestaat zodat UI/PDF een project als
    /// primair-BENG kunnen markeren en de dedicated `/beng/calculate`-route (en
    /// `compute_beng`-Tauri-command) hem kunnen dispatchen. De
    /// warmteverlies-routers (`calculate_v2`) weigeren deze norm bewust.
    ///
    /// [`ProjectV2::energy`]: crate::project::ProjectV2::energy
    Beng,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_norm_defaults_to_isso51() {
        let calcs = Calcs::default();
        assert_eq!(calcs.active_norm(), ActiveNorm::Isso51);
    }

    #[test]
    fn active_norm_with_only_isso51() {
        let calcs = Calcs {
            isso51: Some(Iso51Inputs {
                legacy_v1: serde_json::json!({"info": {"name": "test"}}),
            }),
            tojuli: None,
            isso53: None,
        };
        assert_eq!(calcs.active_norm(), ActiveNorm::Isso51);
    }

    #[test]
    fn active_norm_with_only_isso53() {
        let calcs = Calcs {
            isso51: None,
            tojuli: None,
            isso53: Some(Iso53Inputs {
                legacy: serde_json::json!({"info": {"name": "test isso53"}}),
            }),
        };
        assert_eq!(calcs.active_norm(), ActiveNorm::Isso53);
    }

    #[test]
    fn active_norm_with_both_prefers_isso51() {
        let calcs = Calcs {
            isso51: Some(Iso51Inputs {
                legacy_v1: serde_json::json!({"info": {"name": "test isso51"}}),
            }),
            tojuli: None,
            isso53: Some(Iso53Inputs {
                legacy: serde_json::json!({"info": {"name": "test isso53"}}),
            }),
        };
        // Multi-calc fallback: ISSO 51 heeft voorrang
        assert_eq!(calcs.active_norm(), ActiveNorm::Isso51);
    }
}
