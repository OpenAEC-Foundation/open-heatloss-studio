//! Netto-oppervlakte van een constructie na aftrek van openingen.
//!
//! Volgt bijlage K.1: een wand-constructie heeft een bruto geprojecteerd
//! oppervlak `A_T`; ramen ([`Window`]) en opake openingen ([`Opening`]) die
//! daarin geplaatst zijn, worden eenmalig afgetrokken om de netto dichte
//! constructie-oppervlakte te bepalen.
//!
//! Voor ramen geldt volgens K.2: `A_w = A_fr + A_gl` — het veld
//! [`Window::area`] bevat de totale raamoppervlakte (kozijn + glas), en dat
//! is wat van de wand wordt afgetrokken.

use nta8800_model::error::{ModelError, ModelResult};
use nta8800_model::geometry::{Opening, Window};
use nta8800_model::units::Area;

/// Bereken de netto dichte constructie-oppervlakte door alle openingen
/// (ramen + opake panelen/deuren) van het bruto constructie-oppervlak
/// af te trekken.
///
/// Volgt NTA 8800 bijlage K.1:
/// - `A_con_dicht = A_T_bruto - Σ A_window - Σ A_opening`
///
/// # Errors
/// - [`ModelError::InvalidInput`] als `gross_area` niet-eindig of negatief.
/// - [`ModelError::InvalidInput`] als de som van openingen > `gross_area`.
pub fn net_construction_area(
    gross_area: Area,
    windows: &[&Window],
    openings: &[&Opening],
) -> ModelResult<Area> {
    if !gross_area.is_finite() || gross_area < 0.0 {
        return Err(ModelError::InvalidInput {
            context: "net_construction_area.gross_area".into(),
            reason: format!("moet eindig en ≥ 0 zijn, gekregen {gross_area}"),
        });
    }
    let mut sum = 0.0_f64;
    for w in windows {
        if !w.area.is_finite() || w.area < 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("Window {}.area", w.id),
                reason: format!("moet eindig en ≥ 0 zijn, gekregen {}", w.area),
            });
        }
        sum += w.area;
    }
    for o in openings {
        if !o.area.is_finite() || o.area < 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("Opening {}.area", o.id),
                reason: format!("moet eindig en ≥ 0 zijn, gekregen {}", o.area),
            });
        }
        sum += o.area;
    }
    if sum > gross_area {
        return Err(ModelError::InvalidInput {
            context: "net_construction_area".into(),
            reason: format!(
                "som van openingen ({sum}) overschrijdt bruto constructie-oppervlak ({gross_area})"
            ),
        });
    }
    Ok(gross_area - sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::location::{Orientation, Tilt};

    fn mk_window(id: &str, area: Area) -> Window {
        Window::new(
            id,
            "c-kozijn",
            area,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.1,
            0.5,
            0.25,
        )
        .unwrap()
    }

    fn mk_opening(id: &str, area: Area) -> Opening {
        Opening {
            id: id.into(),
            area,
            u_value: 1.6,
            orientation: Orientation::Zuid,
            tilt: Tilt::VERTICAL,
        }
    }

    #[test]
    fn netto_met_ramen_en_deuren() {
        // Wand 25 m², raam 2.5 m², raam 1.8 m², deur 2.1 m² ⇒ 18.6 m²
        let w1 = mk_window("w1", 2.5);
        let w2 = mk_window("w2", 1.8);
        let d1 = mk_opening("d1", 2.1);
        let net = net_construction_area(25.0, &[&w1, &w2], &[&d1]).unwrap();
        assert!((net - 18.6).abs() < 1e-9);
    }

    #[test]
    fn netto_zonder_openingen_is_gross() {
        let net = net_construction_area(25.0, &[], &[]).unwrap();
        assert!((net - 25.0).abs() < 1e-9);
    }

    #[test]
    fn netto_alleen_ramen() {
        let w1 = mk_window("w1", 5.0);
        let net = net_construction_area(25.0, &[&w1], &[]).unwrap();
        assert!((net - 20.0).abs() < 1e-9);
    }

    #[test]
    fn netto_alleen_opake_openingen() {
        let d1 = mk_opening("d1", 2.1);
        let net = net_construction_area(25.0, &[], &[&d1]).unwrap();
        assert!((net - 22.9).abs() < 1e-9);
    }

    #[test]
    fn netto_nul_als_openingen_bruto_volledig_bedekken() {
        let w1 = mk_window("w1", 10.0);
        let w2 = mk_window("w2", 10.0);
        let net = net_construction_area(20.0, &[&w1, &w2], &[]).unwrap();
        assert!(net.abs() < 1e-12);
    }

    #[test]
    fn netto_weigert_overschrijding() {
        let w1 = mk_window("w1", 15.0);
        let d1 = mk_opening("d1", 15.0);
        let err = net_construction_area(25.0, &[&w1], &[&d1]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn netto_weigert_negatieve_gross() {
        let err = net_construction_area(-1.0, &[], &[]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }
}
