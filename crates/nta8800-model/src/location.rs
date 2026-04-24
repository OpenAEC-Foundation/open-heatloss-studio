//! Locatie-, oriëntatie- en hellingstypen.
//!
//! Voor de rekenkundige klimaatdata verwijst NTA 8800 naar **De Bilt** als
//! nationale referentiezone (bijlage E). Regionale varianten worden later
//! uitgebreid in de `nta8800-tables` crate. De [`ClimateZone`] enum is
//! `non_exhaustive` zodat die uitbreiding géén breaking change wordt.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{ModelError, ModelResult};

/// Geografische coördinaten (WGS84 decimaal).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LatLon {
    /// Breedtegraad in graden (positief = noord).
    pub latitude: f64,
    /// Lengtegraad in graden (positief = oost).
    pub longitude: f64,
}

/// Nederlandse klimaatzone volgens NTA 8800:2025+C1:2026 bijlage E.
///
/// Voor D2 van de implementatie is alleen `DeBilt` gedefinieerd. Regionale
/// varianten (kust/binnenland-indeling per RVO) volgen later in
/// `nta8800-tables`; de enum is daarom expliciet `#[non_exhaustive]`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClimateZone {
    /// KNMI referentiezone De Bilt — NTA 8800 bijlage E.
    #[default]
    DeBilt,
}

/// Locatie van een gebouw — postcode + optionele coördinaten + klimaatzone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Location {
    /// Nederlandse postcode (bv. `"3511AB"`). Geen externe geocoding-stap —
    /// voor rekenvalidatie is de combinatie `climate_zone` + `coordinates`
    /// leidend.
    pub postcode: String,

    /// Optionele WGS84-coördinaten voor zoninstraling / horizonberekening.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<LatLon>,

    /// Klimaatzone voor het ophalen van referentie-klimaatdata.
    #[serde(default)]
    pub climate_zone: ClimateZone,
}

/// Oriëntatie van een vlak op het horizontale grondvlak.
///
/// De hoofdrichtingen volgen de NTA 8800 conventie: 0° = Noord, hoek loopt
/// met de klok mee (Oost = 90°, Zuid = 180°, West = 270°).
/// [`Orientation::Horizontaal`] wordt apart behandeld — een plat vlak heeft
/// geen oriëntatie, alleen een helling.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    /// 0° — pal noord.
    Noord,
    /// 45° — noordoost.
    NoordOost,
    /// 90° — pal oost.
    Oost,
    /// 135° — zuidoost.
    ZuidOost,
    /// 180° — pal zuid.
    Zuid,
    /// 225° — zuidwest.
    ZuidWest,
    /// 270° — pal west.
    West,
    /// 315° — noordwest.
    NoordWest,
    /// Geen geldige kompasrichting — plat vlak (zoals een dak met tilt 0°).
    Horizontaal,
}

impl Orientation {
    /// Geef de oriëntatiehoek in graden vanaf noord (clockwise).
    ///
    /// Voor [`Orientation::Horizontaal`] is er geen zinvolle hoek; deze
    /// variant geeft [`None`].
    #[must_use]
    pub const fn degrees(self) -> Option<f64> {
        match self {
            Orientation::Noord => Some(0.0),
            Orientation::NoordOost => Some(45.0),
            Orientation::Oost => Some(90.0),
            Orientation::ZuidOost => Some(135.0),
            Orientation::Zuid => Some(180.0),
            Orientation::ZuidWest => Some(225.0),
            Orientation::West => Some(270.0),
            Orientation::NoordWest => Some(315.0),
            Orientation::Horizontaal => None,
        }
    }
}

/// Helling van een vlak in graden t.o.v. het horizontale grondvlak.
///
/// - `0.0` = horizontaal (vlak dak, vloer);
/// - `90.0` = verticaal (gevel, raam);
/// - `180.0` = omgekeerd horizontaal (onderkant uitkraging).
///
/// Validatie: de hoek moet binnen `0.0..=180.0` vallen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Tilt {
    /// De helling in graden.
    pub degrees: f64,
}

impl Tilt {
    /// Tilt voor een horizontaal vlak.
    pub const HORIZONTAL: Tilt = Tilt { degrees: 0.0 };
    /// Tilt voor een verticaal vlak (gevel).
    pub const VERTICAL: Tilt = Tilt { degrees: 90.0 };

    /// Construct een [`Tilt`] met bereikvalidatie `0.0..=180.0`.
    ///
    /// # Errors
    /// Geeft [`ModelError::OutOfRange`] als `degrees` buiten het bereik valt
    /// of niet-eindig is.
    pub fn new(degrees: f64) -> ModelResult<Self> {
        if !degrees.is_finite() || !(0.0..=180.0).contains(&degrees) {
            return Err(ModelError::OutOfRange {
                field: "Tilt.degrees".into(),
                range: "0.0..=180.0".into(),
                value: format!("{degrees}"),
            });
        }
        Ok(Self { degrees })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn climate_zone_default_is_de_bilt() {
        assert_eq!(ClimateZone::default(), ClimateZone::DeBilt);
    }

    #[test]
    fn orientation_degrees_north_is_zero() {
        assert_eq!(Orientation::Noord.degrees(), Some(0.0));
        assert_eq!(Orientation::Zuid.degrees(), Some(180.0));
        assert_eq!(Orientation::Horizontaal.degrees(), None);
    }

    #[test]
    fn tilt_new_accepts_valid_range() {
        assert!(Tilt::new(0.0).is_ok());
        assert!(Tilt::new(90.0).is_ok());
        assert!(Tilt::new(180.0).is_ok());
    }

    #[test]
    fn tilt_new_rejects_negative() {
        let err = Tilt::new(-0.1).unwrap_err();
        assert!(matches!(err, ModelError::OutOfRange { .. }));
    }

    #[test]
    fn tilt_new_rejects_above_180() {
        let err = Tilt::new(181.0).unwrap_err();
        assert!(matches!(err, ModelError::OutOfRange { .. }));
    }

    #[test]
    fn tilt_new_rejects_nan() {
        assert!(Tilt::new(f64::NAN).is_err());
    }

    #[test]
    fn tilt_constants() {
        assert!((Tilt::HORIZONTAL.degrees - 0.0).abs() < f64::EPSILON);
        assert!((Tilt::VERTICAL.degrees - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn location_serde_round_trip() {
        let loc = Location {
            postcode: "3511AB".to_string(),
            coordinates: Some(LatLon {
                latitude: 52.0907,
                longitude: 5.1214,
            }),
            climate_zone: ClimateZone::DeBilt,
        };
        let json = serde_json::to_string(&loc).unwrap();
        let back: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(loc, back);
    }

    #[test]
    fn orientation_serde_snake_case() {
        let json = serde_json::to_string(&Orientation::ZuidWest).unwrap();
        assert_eq!(json, "\"zuid_west\"");
    }
}
