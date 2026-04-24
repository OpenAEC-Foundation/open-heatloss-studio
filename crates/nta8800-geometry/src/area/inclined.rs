//! Projectie van schuine vlakken op horizontaal of verticaal vlak.
//!
//! NTA 8800:2025+C1:2026 §6.8 en bijlage K.1.3 definiëren de geprojecteerde
//! oppervlakte van een scheidingsconstructie als de oppervlakte van een
//! denkbeeldig *plat* vlak dat de constructie begrenst. Voor schuine
//! oppervlakken (hellend dak, schuine wand) betekent dit dat het daadwerkelijke
//! constructie-oppervlak via een goniometrische factor geprojecteerd wordt.
//!
//! | Projectie op | Factor | Gebruik |
//! |---|---|---|
//! | Horizontaal vlak | cos(tilt) | Dakoppervlakte-bijdrage aan grondvlak |
//! | Verticaal vlak | sin(tilt) | Schuine wand-bijdrage aan verticale schil |
//!
//! §6.8 (pg 163) onderscheidt dakoppervlakken als *"niet-transparante
//! constructies met een hellingshoek van ten minste 15° ten opzichte van
//! de verticaal"*. Deze functies leveren daar de geometrische basis voor.

use nta8800_model::error::{ModelError, ModelResult};
use nta8800_model::location::Tilt;
use nta8800_model::units::Area;

/// Projecteer een schuin oppervlak op het horizontale vlak.
///
/// `A_horizontaal = A_hellend · cos(tilt)` — waarbij `tilt = 0°` een
/// horizontaal vlak is (projectie = volledig oppervlak) en `tilt = 90°`
/// een verticaal vlak (projectie = 0 m² op grondvlak).
///
/// # Errors
/// Retourneert [`ModelError::InvalidInput`] als `inclined_area` niet-eindig
/// of negatief is, of [`ModelError::OutOfRange`] als `tilt.degrees` niet in
/// `0..=180` valt (gevalideerd via [`Tilt::new`]-contract — `Tilt` staat
/// alleen geldige bereiken toe, maar deze laag herhaalt de check defensief
/// voor het geval een client een raw `Tilt` construeert).
pub fn horizontal_projection(inclined_area: Area, tilt: Tilt) -> ModelResult<Area> {
    validate_area("horizontal_projection.inclined_area", inclined_area)?;
    validate_tilt_range(tilt)?;
    let radians = tilt.degrees.to_radians();
    Ok(inclined_area * radians.cos().abs())
}

/// Projecteer een schuin oppervlak op het verticale vlak.
///
/// `A_verticaal = A_hellend · sin(tilt)` — waarbij `tilt = 0°` (horizontaal)
/// een verticale projectie van 0 m² geeft, en `tilt = 90°` (verticaal
/// vlak) het volledige oppervlak.
///
/// # Errors
/// Gelijk aan [`horizontal_projection`].
pub fn vertical_projection(inclined_area: Area, tilt: Tilt) -> ModelResult<Area> {
    validate_area("vertical_projection.inclined_area", inclined_area)?;
    validate_tilt_range(tilt)?;
    let radians = tilt.degrees.to_radians();
    Ok(inclined_area * radians.sin().abs())
}

fn validate_area(context: &str, v: f64) -> ModelResult<()> {
    if !v.is_finite() || v < 0.0 {
        return Err(ModelError::InvalidInput {
            context: context.into(),
            reason: format!("moet eindig en ≥ 0 zijn, gekregen {v}"),
        });
    }
    Ok(())
}

fn validate_tilt_range(tilt: Tilt) -> ModelResult<()> {
    if !tilt.degrees.is_finite() || !(0.0..=180.0).contains(&tilt.degrees) {
        return Err(ModelError::OutOfRange {
            field: "Tilt.degrees".into(),
            range: "0.0..=180.0".into(),
            value: format!("{}", tilt.degrees),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    #[test]
    fn horizontal_projection_van_horizontaal_vlak_is_volledig() {
        // tilt=0° ⇒ cos=1 ⇒ A_h = A
        let a = horizontal_projection(10.0, Tilt::HORIZONTAL).unwrap();
        assert!((a - 10.0).abs() < TOL);
    }

    #[test]
    fn horizontal_projection_van_verticaal_vlak_is_nul() {
        // tilt=90° ⇒ cos=0 ⇒ A_h ≈ 0
        let a = horizontal_projection(10.0, Tilt::VERTICAL).unwrap();
        assert!(a.abs() < 1e-12);
    }

    #[test]
    fn vertical_projection_van_horizontaal_vlak_is_nul() {
        let a = vertical_projection(10.0, Tilt::HORIZONTAL).unwrap();
        assert!(a.abs() < 1e-12);
    }

    #[test]
    fn vertical_projection_van_verticaal_vlak_is_volledig() {
        let a = vertical_projection(10.0, Tilt::VERTICAL).unwrap();
        assert!((a - 10.0).abs() < TOL);
    }

    #[test]
    fn projectie_van_45_graden_is_root_half_keer_oppervlak() {
        let tilt45 = Tilt::new(45.0).unwrap();
        let hor = horizontal_projection(10.0, tilt45).unwrap();
        let vert = vertical_projection(10.0, tilt45).unwrap();
        let expected = 10.0 * std::f64::consts::FRAC_1_SQRT_2; // 10/√2
        assert!((hor - expected).abs() < TOL);
        assert!((vert - expected).abs() < TOL);
        // Pythagoras-sanity: projecties loodrecht op elkaar → hor² + vert² = A²
        assert!((hor * hor + vert * vert - 100.0).abs() < TOL);
    }

    #[test]
    fn projectie_omgekeerde_dak_180_graden() {
        // tilt=180° = omgekeerd horizontaal (onderkant uitkraging)
        // cos(180°) = -1 ⇒ |cos| = 1 ⇒ horizontale projectie is volledig
        let tilt180 = Tilt::new(180.0).unwrap();
        let hor = horizontal_projection(10.0, tilt180).unwrap();
        assert!((hor - 10.0).abs() < TOL);
    }

    #[test]
    fn projectie_weigert_negatief_oppervlak() {
        let err = horizontal_projection(-1.0, Tilt::HORIZONTAL).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn projectie_weigert_nan_oppervlak() {
        let err = vertical_projection(f64::NAN, Tilt::VERTICAL).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn projectie_weigert_raw_tilt_buiten_bereik() {
        // Constructor weigert dit al, maar we checken defensief op raw Tilt.
        let bad_tilt = Tilt { degrees: 200.0 };
        let err = horizontal_projection(10.0, bad_tilt).unwrap_err();
        assert!(matches!(err, ModelError::OutOfRange { .. }));
    }
}
