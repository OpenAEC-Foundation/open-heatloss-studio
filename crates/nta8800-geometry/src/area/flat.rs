//! Vlakvormige elementen — bruto en netto oppervlakten.
//!
//! Volgt de schematisering uit bijlage K.1:
//!
//! - Bruto wandoppervlak = `height × width` (geprojecteerd, K.1.3).
//! - Netto wandoppervlak = bruto − Σ openingen (ramen + opake openingen).
//! - Netto vloeroppervlak = bruto − Σ uitsluitingen (vides, traptuiten,
//!   schacht-doorbraken, etc.).
//!
//! Deze helpers voeren geen eenheidconversie uit — alle invoer is in meter
//! of m² volgens `nta8800_model::units`.

use nta8800_model::error::{ModelError, ModelResult};
use nta8800_model::units::{Area, Length};

/// Bereken het bruto wandoppervlak (geprojecteerd, bijlage K.1.3).
///
/// # Errors
/// Retourneert [`ModelError::InvalidInput`] als `height` of `width`
/// niet-eindig of ≤ 0 is.
pub fn gross_wall_area(height: Length, width: Length) -> ModelResult<Area> {
    if !height.is_finite() || height <= 0.0 {
        return Err(ModelError::InvalidInput {
            context: "gross_wall_area.height".into(),
            reason: format!("moet > 0 en eindig zijn, gekregen {height}"),
        });
    }
    if !width.is_finite() || width <= 0.0 {
        return Err(ModelError::InvalidInput {
            context: "gross_wall_area.width".into(),
            reason: format!("moet > 0 en eindig zijn, gekregen {width}"),
        });
    }
    Ok(height * width)
}

/// Bereken het netto wandoppervlak door openingen van het bruto oppervlak
/// af te trekken.
///
/// Volgt bijlage K.1: openingen (ramen, deuren, opake panelen) worden
/// eenmalig van het bruto-wandoppervlak afgetrokken.
///
/// # Errors
/// - [`ModelError::InvalidInput`] als `gross` of een van de openingen
///   niet-eindig of negatief is.
/// - [`ModelError::InvalidInput`] als de som van openingen het bruto-
///   oppervlak overschrijdt (resultaat zou negatief worden).
pub fn net_wall_area(gross: Area, openings: &[Area]) -> ModelResult<Area> {
    validate_finite_non_negative("net_wall_area.gross", gross)?;
    let mut sum = 0.0_f64;
    for (i, a) in openings.iter().enumerate() {
        validate_finite_non_negative(&format!("net_wall_area.openings[{i}]"), *a)?;
        sum += *a;
    }
    if sum > gross {
        return Err(ModelError::InvalidInput {
            context: "net_wall_area".into(),
            reason: format!("som openingen ({sum}) overschrijdt bruto wandoppervlak ({gross})"),
        });
    }
    Ok(gross - sum)
}

/// Bereken het netto vloeroppervlak door uitsluitingen (vides, trapgaten,
/// schacht-doorbraken) van het bruto-oppervlak af te trekken.
///
/// Semantisch identiek aan [`net_wall_area`] maar met een duidelijker
/// domein-naam: vloeren hebben doorgaans "exclusions" in plaats van
/// "openings".
///
/// # Errors
/// Gelijk aan [`net_wall_area`].
pub fn net_floor_area(gross: Area, exclusions: &[Area]) -> ModelResult<Area> {
    validate_finite_non_negative("net_floor_area.gross", gross)?;
    let mut sum = 0.0_f64;
    for (i, a) in exclusions.iter().enumerate() {
        validate_finite_non_negative(&format!("net_floor_area.exclusions[{i}]"), *a)?;
        sum += *a;
    }
    if sum > gross {
        return Err(ModelError::InvalidInput {
            context: "net_floor_area".into(),
            reason: format!("som exclusions ({sum}) overschrijdt bruto vloeroppervlak ({gross})"),
        });
    }
    Ok(gross - sum)
}

fn validate_finite_non_negative(context: &str, v: f64) -> ModelResult<()> {
    if !v.is_finite() || v < 0.0 {
        return Err(ModelError::InvalidInput {
            context: context.into(),
            reason: format!("moet eindig en ≥ 0 zijn, gekregen {v}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gross_wall_area_multipliceert_hoogte_breedte() {
        let a = gross_wall_area(2.6, 5.0).unwrap();
        assert!((a - 13.0).abs() < 1e-9);
    }

    #[test]
    fn gross_wall_area_weigert_nul_afmetingen() {
        assert!(gross_wall_area(0.0, 5.0).is_err());
        assert!(gross_wall_area(2.6, 0.0).is_err());
        assert!(gross_wall_area(-1.0, 5.0).is_err());
    }

    #[test]
    fn gross_wall_area_weigert_nan() {
        assert!(gross_wall_area(f64::NAN, 5.0).is_err());
        assert!(gross_wall_area(2.6, f64::INFINITY).is_err());
    }

    #[test]
    fn net_wall_area_trekt_openingen_af() {
        // Wand 2.6 × 5.0 = 13 m², 1 raam 2.5 m², 1 deur 2.1 m² ⇒ 8.4 m²
        let net = net_wall_area(13.0, &[2.5, 2.1]).unwrap();
        assert!((net - 8.4).abs() < 1e-9);
    }

    #[test]
    fn net_wall_area_zero_if_openings_equal_gross() {
        // Pathologisch maar toegestaan — net = 0
        let net = net_wall_area(10.0, &[6.0, 4.0]).unwrap();
        assert!(net.abs() < 1e-12);
    }

    #[test]
    fn net_wall_area_weigert_overschrijding() {
        let err = net_wall_area(10.0, &[6.0, 5.0]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn net_wall_area_lege_lijst_is_gross() {
        let net = net_wall_area(15.5, &[]).unwrap();
        assert!((net - 15.5).abs() < 1e-9);
    }

    #[test]
    fn net_floor_area_trekt_vide_af() {
        // Bruto 120 m², vide 8 m², trapgat 3 m² ⇒ 109 m²
        let net = net_floor_area(120.0, &[8.0, 3.0]).unwrap();
        assert!((net - 109.0).abs() < 1e-9);
    }

    #[test]
    fn net_floor_area_weigert_negatieve_exclusion() {
        let err = net_floor_area(120.0, &[8.0, -1.0]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }
}
