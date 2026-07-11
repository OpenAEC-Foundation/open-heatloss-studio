//! Ramen — transparante gevelopeningen met U, g-waarde en kozijnfractie.
//!
//! NTA 8800 §8.5 — het raam heeft een samengestelde U-waarde (glas + kozijn),
//! een zonnewarmtedoorlatingsfactor g (dimensieloos 0..=1), en een
//! kozijnfractie (aandeel opake frame ten opzichte van totale opening).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{ModelError, ModelResult};
use crate::location::{Orientation, Tilt};
use crate::units::{Area, ThermalTransmittance};

/// Glastype voor referentiewaarden.
///
/// De bijbehorende default U- en g-waarden staan in NTA 8800 bijlage G;
/// die tabel wordt door de `nta8800-tables` crate ontsloten — hier
/// definiëren we alleen de enumeratie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GlassType {
    /// Enkelglas.
    Enkel,
    /// HR-glas (dubbelglas met low-e coating).
    Hr,
    /// HR+-glas.
    HrPlus,
    /// HR++-glas.
    HrPlusPlus,
    /// Triple glas.
    Triple,
    /// Maatwerk — waarden moeten expliciet worden gespecificeerd op het [`Window`].
    Custom,
}

/// Kozijnmateriaal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FrameType {
    /// Hout.
    Hout,
    /// Kunststof.
    Kunststof,
    /// Aluminium zonder thermische onderbreking.
    Aluminium,
    /// Aluminium met thermische onderbreking.
    AluminiumThermischOnderbroken,
    /// Staal.
    Staal,
}

/// Bedieningsregime van beweegbare zonwering — bepaalt de gewogen inzetfractie
/// `f_sh;with` (NTA 8800 §7.6.6.1.4).
///
/// De concrete maand/oriëntatie-profielen (tabel 7.7 resp. 7.9) leven in de
/// `nta8800-demand`-crate; dit model draagt alleen de keuze.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShadingControl {
    /// Handbediende zonwering, woningbouw (schakelcriterium 300 W/m², tabel 7.7).
    ManualResidential,
    /// Automatisch geregelde zonwering (schakelcriterium 150 W/m², tabel 7.9).
    Automatic,
}

/// Beweegbare zonwering op een raam (screen, jaloezie, rolluik, uitval-/
/// knikarmscherm), NTA 8800 §7.6.6.1.4, formules (7.42)/(7.43).
///
/// Het maandgemiddelde van de totale zontoetredingsfactor wordt met beweegbare
/// zonwering:
///
/// ```text
/// g_gl;wi;mi = (1 − f_sh;with;mi) · g_gl;wi + f_sh;with;mi · (F_c · g_gl;wi)      (7.42)/(7.43)
/// ```
///
/// zodat de effectieve g-reductiefactor per maand `(1 − f_sh;with) + f_sh;with · F_c`
/// bedraagt. `f_sh;with` volgt uit [`ShadingControl`], `F_c` uit tabel 7.5/7.6.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MovableSunShading {
    /// Reductiefactor `F_c` voor de totale zontoetredingsfactor (0..=1),
    /// forfaitair uit NTA 8800 tabel 7.5 (screens/jaloezieën/rolluiken/
    /// gemetalliseerde weefsels) of tabel 7.6 (uitval-/knikarmschermen).
    pub f_c: f64,
    /// Bedieningsregime (bepaalt het `f_sh;with`-profiel).
    pub control: ShadingControl,
}

/// Raam-element in de gevelberekening.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Window {
    /// Unieke identificatie binnen het project.
    pub id: String,

    /// Id van een [`super::Construction`] die het omliggende kozijn/paneel
    /// beschrijft (voor thermische brug-effecten en visualisatie). De rekencrate
    /// resolvet deze referentie via het project-manifest.
    pub construction_id: String,

    /// Bruto vensteroppervlak in m² (buitenwerks, kozijn + glas).
    pub area: Area,

    /// Oriëntatie van het venster.
    pub orientation: Orientation,

    /// Helling van het venster.
    pub tilt: Tilt,

    /// Samengestelde U-waarde van het venster in W/(m²·K) (glas + kozijn).
    pub u_value: ThermalTransmittance,

    /// Zonnewarmtedoorlatingsfactor g (0..=1).
    pub g_value: f64,

    /// Kozijnfractie (0..=1) — aandeel opaak frame in totale opening.
    pub frame_fraction: f64,

    /// Beweegbare zonwering (NTA 8800 §7.6.6.1.4). `None` = geen zonwering; de
    /// effectieve zontoetreding is dan onveranderd (reductiefactor 1,0). Het
    /// veld wordt bij afwezigheid niet geserialiseerd zodat bestaande project-
    /// JSON byte-identiek blijft.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movable_shading: Option<MovableSunShading>,
}

impl Window {
    /// Construct een [`Window`] met bereikvalidatie op g-waarde, frame-fractie
    /// en oppervlakte.
    ///
    /// # Errors
    /// - [`ModelError::OutOfRange`] als `g_value` of `frame_fraction` buiten
    ///   `0.0..=1.0` valt of niet-eindig is.
    /// - [`ModelError::InvalidInput`] als `area <= 0.0` of niet-eindig.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        construction_id: impl Into<String>,
        area: Area,
        orientation: Orientation,
        tilt: Tilt,
        u_value: ThermalTransmittance,
        g_value: f64,
        frame_fraction: f64,
    ) -> ModelResult<Self> {
        if !area.is_finite() || area <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: "Window.area".into(),
                reason: "moet > 0 en eindig zijn".into(),
            });
        }
        if !g_value.is_finite() || !(0.0..=1.0).contains(&g_value) {
            return Err(ModelError::OutOfRange {
                field: "Window.g_value".into(),
                range: "0.0..=1.0".into(),
                value: format!("{g_value}"),
            });
        }
        if !frame_fraction.is_finite() || !(0.0..=1.0).contains(&frame_fraction) {
            return Err(ModelError::OutOfRange {
                field: "Window.frame_fraction".into(),
                range: "0.0..=1.0".into(),
                value: format!("{frame_fraction}"),
            });
        }
        Ok(Self {
            id: id.into(),
            construction_id: construction_id.into(),
            area,
            orientation,
            tilt,
            u_value,
            g_value,
            frame_fraction,
            movable_shading: None,
        })
    }

    /// Voeg beweegbare zonwering toe (builder-stijl), NTA 8800 §7.6.6.1.4.
    #[must_use]
    pub fn with_movable_shading(mut self, shading: MovableSunShading) -> Self {
        self.movable_shading = Some(shading);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_window() -> Window {
        Window::new(
            "w1",
            "c-kozijn",
            2.5,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.1,
            0.55,
            0.25,
        )
        .unwrap()
    }

    #[test]
    fn constructor_happy_path() {
        let w = ok_window();
        assert_eq!(w.id, "w1");
        assert_eq!(w.orientation, Orientation::Zuid);
    }

    #[test]
    fn constructor_rejects_non_positive_area() {
        let err = Window::new(
            "w",
            "c",
            0.0,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.0,
            0.5,
            0.25,
        )
        .unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn constructor_rejects_g_value_above_one() {
        let err = Window::new(
            "w",
            "c",
            2.0,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.0,
            1.1,
            0.25,
        )
        .unwrap_err();
        assert!(matches!(err, ModelError::OutOfRange { .. }));
    }

    #[test]
    fn constructor_rejects_frame_fraction_below_zero() {
        let err = Window::new(
            "w",
            "c",
            2.0,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.0,
            0.5,
            -0.1,
        )
        .unwrap_err();
        assert!(matches!(err, ModelError::OutOfRange { .. }));
    }

    #[test]
    fn window_serde_round_trip() {
        let w = ok_window();
        let json = serde_json::to_string(&w).unwrap();
        let back: Window = serde_json::from_str(&json).unwrap();
        assert_eq!(w, back);
    }

    #[test]
    fn glass_type_serde_snake_case() {
        let json = serde_json::to_string(&GlassType::HrPlusPlus).unwrap();
        assert_eq!(json, "\"hr_plus_plus\"");
    }

    #[test]
    fn frame_type_serde_snake_case() {
        let json = serde_json::to_string(&FrameType::AluminiumThermischOnderbroken).unwrap();
        assert_eq!(json, "\"aluminium_thermisch_onderbroken\"");
    }
}
