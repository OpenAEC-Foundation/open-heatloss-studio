//! Validatie van een Gebouw-indeling tegen NTA 8800 H.6 begrenzingsregels.
//!
//! Zie [`validate_building`] voor de publieke API.

use nta8800_model::error::{ModelError, ModelResult};
use nta8800_model::zoning::{EnergiefunctieRuimte, Gebouw, Rekenzone};

use super::rules::are_usage_functions_compatible;

/// Tolerantie voor vloeroppervlak-som vergelijking (1 mm² = 1e-6 m²).
const AREA_TOLERANCE_M2: f64 = 1.0e-6;

/// Valideer een Gebouw-indeling tegen NTA 8800 H.6 begrenzingsregels.
///
/// Checkt de volgende invarianten:
///
/// 1. Elke `Rekenzone.gebouw_id` verwijst naar het `building.id`.
/// 2. Elke Rekenzone heeft **volume > 0** (§6.5 — rekenzone is een
///    geometrisch volume, niet-positief is onzinnig).
/// 3. Elke Rekenzone bevat **minimaal één** `EnergiefunctieRuimte`.
/// 4. Voor elke Rekenzone verwijst elke `efr_ids`-entry naar een EFR in
///    `efrs`, en omgekeerd verwijst elke EFR via `rekenzone_id` terug.
/// 5. EFR's binnen één Rekenzone hebben **compatible** `UsageFunction`
///    (zie [`super::rules::are_usage_functions_compatible`]) — §6.5.2
///    verbiedt mengen van woonfunctie met andere gebruiksfuncties.
/// 6. Rekenzone `floor_area` ≥ Σ EFR `floor_area` (met tolerantie
///    [`AREA_TOLERANCE_M2`]). De som mag niet grósser zijn dan de
///    rekenzone-oppervlakte.
///
/// # Errors
/// Retourneert [`ModelError::InvalidInput`] of [`ModelError::ReferenceNotFound`]
/// bij de eerste schending, met context die verwijst naar de betreffende
/// `Rekenzone.id` / `EnergiefunctieRuimte.id`.
pub fn validate_building(
    building: &Gebouw,
    rekenzones: &[Rekenzone],
    efrs: &[EnergiefunctieRuimte],
) -> ModelResult<()> {
    for rz in rekenzones {
        if rz.gebouw_id != building.id {
            return Err(ModelError::InvalidInput {
                context: format!("Rekenzone {} .gebouw_id", rz.id),
                reason: format!(
                    "verwijst naar {} maar gebouw heeft id {}",
                    rz.gebouw_id, building.id
                ),
            });
        }

        if !rz.volume.is_finite() || rz.volume <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("Rekenzone {}.volume", rz.id),
                reason: format!("moet > 0 en eindig zijn, gekregen {}", rz.volume),
            });
        }

        if !rz.floor_area.is_finite() || rz.floor_area <= 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("Rekenzone {}.floor_area", rz.id),
                reason: format!("moet > 0 en eindig zijn, gekregen {}", rz.floor_area),
            });
        }

        if rz.efr_ids.is_empty() {
            return Err(ModelError::InvalidInput {
                context: format!("Rekenzone {}.efr_ids", rz.id),
                reason: "moet minimaal één EnergiefunctieRuimte bevatten (NTA 8800 §6.5)".into(),
            });
        }

        // Verzamel de EFR's van deze rekenzone.
        let mut zone_efrs: Vec<&EnergiefunctieRuimte> = Vec::with_capacity(rz.efr_ids.len());
        for efr_id in &rz.efr_ids {
            let efr = efrs.iter().find(|e| &e.id == efr_id).ok_or_else(|| {
                ModelError::ReferenceNotFound {
                    kind: "EnergiefunctieRuimte".into(),
                    id: efr_id.clone(),
                }
            })?;
            if efr.rekenzone_id != rz.id {
                return Err(ModelError::InvalidInput {
                    context: format!("EnergiefunctieRuimte {}.rekenzone_id", efr.id),
                    reason: format!(
                        "verwijst naar {} maar EFR staat in efr_ids van Rekenzone {}",
                        efr.rekenzone_id, rz.id
                    ),
                });
            }
            zone_efrs.push(efr);
        }

        // Compatibiliteit van gebruiksfuncties binnen deze rekenzone (§6.5.2).
        for i in 0..zone_efrs.len() {
            for j in (i + 1)..zone_efrs.len() {
                let a = zone_efrs[i];
                let b = zone_efrs[j];
                if !are_usage_functions_compatible(a.usage_function, b.usage_function) {
                    return Err(ModelError::InvalidInput {
                        context: format!("Rekenzone {}: EFR {} vs EFR {}", rz.id, a.id, b.id),
                        reason: format!(
                            "gebruiksfuncties {:?} en {:?} mogen niet in één rekenzone (NTA 8800 §6.5.2)",
                            a.usage_function, b.usage_function
                        ),
                    });
                }
            }
        }

        // Oppervlakte-som: Σ(EFR.floor_area) ≤ Rekenzone.floor_area (+tolerantie).
        let efr_sum: f64 = zone_efrs.iter().map(|e| e.floor_area).sum();
        if efr_sum.is_nan() || !efr_sum.is_finite() {
            return Err(ModelError::InvalidInput {
                context: format!("Rekenzone {}: Σ EFR.floor_area", rz.id),
                reason: "bevat NaN of infinite waarde".into(),
            });
        }
        if efr_sum > rz.floor_area + AREA_TOLERANCE_M2 {
            return Err(ModelError::InvalidInput {
                context: format!("Rekenzone {}.floor_area", rz.id),
                reason: format!(
                    "som van EFR-oppervlakten ({efr_sum}) overschrijdt rekenzone-oppervlak ({}), tolerantie {AREA_TOLERANCE_M2}",
                    rz.floor_area
                ),
            });
        }
    }

    // Elke EFR moet verwijzen naar een bestaande rekenzone die in deze lijst zit.
    for efr in efrs {
        if !rekenzones.iter().any(|rz| rz.id == efr.rekenzone_id) {
            return Err(ModelError::ReferenceNotFound {
                kind: "Rekenzone".into(),
                id: efr.rekenzone_id.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::location::{ClimateZone, Location};
    use nta8800_model::zoning::UsageFunction;

    fn make_building() -> Gebouw {
        Gebouw {
            id: "g1".into(),
            name: "Testgebouw".into(),
            usage_function: UsageFunction::Woonfunctie,
            construction_year: None,
            location: Location {
                postcode: "3511AB".into(),
                coordinates: None,
                climate_zone: ClimateZone::DeBilt,
            },
            rekenzone_ids: vec!["rz1".into()],
        }
    }

    fn make_rekenzone(id: &str, floor_area: f64, volume: f64, efr_ids: Vec<String>) -> Rekenzone {
        Rekenzone {
            id: id.into(),
            name: format!("Zone {id}"),
            gebouw_id: "g1".into(),
            floor_area,
            volume,
            efr_ids,
            constructions: vec![],
            windows: vec![],
            openings: vec![],
            thermal_bridges_linear: vec![],
            thermal_bridges_point: vec![],
        }
    }

    fn make_efr(id: &str, rz: &str, area: f64, uf: UsageFunction) -> EnergiefunctieRuimte {
        EnergiefunctieRuimte {
            id: id.into(),
            name: format!("EFR {id}"),
            rekenzone_id: rz.into(),
            floor_area: area,
            usage_function: uf,
        }
    }

    #[test]
    fn happy_path_een_woonzone_met_een_efr() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 125.0, 312.5, vec!["efr1".into()]);
        let efr = make_efr("efr1", "rz1", 120.0, UsageFunction::Woonfunctie);
        assert!(validate_building(&g, &[rz], &[efr]).is_ok());
    }

    #[test]
    fn rekenzone_zonder_efr_wordt_afgewezen() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 125.0, 312.5, vec![]);
        let err = validate_building(&g, &[rz], &[]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn negatief_volume_wordt_afgewezen() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 125.0, -1.0, vec!["efr1".into()]);
        let efr = make_efr("efr1", "rz1", 100.0, UsageFunction::Woonfunctie);
        let err = validate_building(&g, &[rz], &[efr]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn nul_volume_wordt_afgewezen() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 125.0, 0.0, vec!["efr1".into()]);
        let efr = make_efr("efr1", "rz1", 100.0, UsageFunction::Woonfunctie);
        let err = validate_building(&g, &[rz], &[efr]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn mismatched_usage_function_woon_plus_kantoor_wordt_afgewezen() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 200.0, 500.0, vec!["efr1".into(), "efr2".into()]);
        let efr1 = make_efr("efr1", "rz1", 100.0, UsageFunction::Woonfunctie);
        let efr2 = make_efr("efr2", "rz1", 80.0, UsageFunction::Kantoorfunctie);
        let err = validate_building(&g, &[rz], &[efr1, efr2]).unwrap_err();
        match err {
            ModelError::InvalidInput { reason, .. } => {
                assert!(reason.contains("§6.5.2"));
            }
            other => panic!("onverwachte error: {other:?}"),
        }
    }

    #[test]
    fn mix_van_utiliteitsfuncties_is_toegestaan() {
        let g = Gebouw {
            usage_function: UsageFunction::Kantoorfunctie,
            ..make_building()
        };
        let rz = make_rekenzone("rz1", 500.0, 1500.0, vec!["efr1".into(), "efr2".into()]);
        let efr1 = make_efr("efr1", "rz1", 200.0, UsageFunction::Kantoorfunctie);
        let efr2 = make_efr("efr2", "rz1", 150.0, UsageFunction::Bijeenkomstfunctie);
        assert!(validate_building(&g, &[rz], &[efr1, efr2]).is_ok());
    }

    #[test]
    fn onbekende_efr_referentie_wordt_afgewezen() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 125.0, 312.5, vec!["efr-unknown".into()]);
        let err = validate_building(&g, &[rz], &[]).unwrap_err();
        assert!(
            matches!(err, ModelError::ReferenceNotFound { kind, .. } if kind == "EnergiefunctieRuimte")
        );
    }

    #[test]
    fn rekenzone_gebouw_id_mismatch_wordt_afgewezen() {
        let g = make_building();
        let rz = Rekenzone {
            gebouw_id: "ander-gebouw".into(),
            ..make_rekenzone("rz1", 125.0, 312.5, vec!["efr1".into()])
        };
        let efr = make_efr("efr1", "rz1", 100.0, UsageFunction::Woonfunctie);
        let err = validate_building(&g, &[rz], &[efr]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn efr_sum_mag_niet_groter_dan_rekenzone_oppervlak() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 50.0, 125.0, vec!["efr1".into(), "efr2".into()]);
        let efr1 = make_efr("efr1", "rz1", 30.0, UsageFunction::Woonfunctie);
        let efr2 = make_efr("efr2", "rz1", 25.0, UsageFunction::Woonfunctie);
        let err = validate_building(&g, &[rz], &[efr1, efr2]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn efr_rekenzone_back_reference_moet_kloppen() {
        let g = make_building();
        let rz = make_rekenzone("rz1", 125.0, 312.5, vec!["efr1".into()]);
        // EFR wijst naar rz2 terwijl hij in efr_ids van rz1 zit.
        let efr = make_efr("efr1", "rz2", 100.0, UsageFunction::Woonfunctie);
        let err = validate_building(&g, &[rz], &[efr]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }
}
