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
/// aan `nta8800_cooling::model::CoolingSystem` etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TojuliInputs {
    /// Optionele "quick check" mode (bijlage AA, woningen only) — bevat
    /// de 12-veld request van de huidige `/tojuli/quick` MVP voor
    /// retro-compat. Volledig H.10-pad komt apart als
    /// `Option<TojuliH10Inputs>` in F7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quick_check: Option<serde_json::Value>,
}
