//! Manifest-resolver: `Rekenzone` id-lijsten → opgeloste object-verzamelingen.
//!
//! Een [`crate::zoning::Rekenzone`] verwijst naar zijn schil-elementen via
//! id-lijsten (`constructions`, `windows`, `openings`,
//! `thermal_bridges_linear`, `thermal_bridges_point`) en zijn ruimten via
//! `efr_ids`. De daadwerkelijke objecten leven op project-manifest-niveau —
//! als losse collecties naast de zones (zie de `Nta8800View` in
//! `openaec-project-shared`, of een toekomstig gebouw-manifest).
//!
//! Tot nu toe lazen de rekencrates die id-lijsten **niet**: callers gaven de
//! opgeloste objecten al als slices door en `calculate_transmission` deed
//! `let _ = zone;` (zie `nta8800-transmission/src/calc/mod.rs`). Deze module
//! sluit dat gat: [`resolve_zone`] zet één zone + de manifest-collecties om in
//! een [`ResolvedZone`] met referenties naar precies de objecten die in die
//! zone horen, in de volgorde van de id-lijsten. De F2b-orchestrator kan
//! daarmee per rekenzone de service-crates voeden.
//!
//! De resolver hoort in deze crate omdat álle opgeloste object-types hier
//! wonen ([`crate::geometry`] + [`crate::zoning`]) en [`crate::ModelError`]
//! al een [`ReferenceNotFound`]-variant heeft voor precies dit doel. Zo is de
//! resolver bruikbaar voor zowel de view-mapper als de orchestrator zonder
//! nieuwe dependencies of een concurrerend container-type.
//!
//! [`ReferenceNotFound`]: crate::ModelError::ReferenceNotFound

use std::collections::HashMap;

use crate::error::{ModelError, ModelResult};
use crate::geometry::{
    Construction, Opening, ThermalBridgeLinear, ThermalBridgePoint, Window,
};
use crate::zoning::{EnergiefunctieRuimte, Rekenzone};

/// De opgeloste object-verzamelingen van één [`Rekenzone`].
///
/// Alle velden zijn referenties in de manifest-collecties die aan
/// [`resolve_zone`] zijn meegegeven; er wordt niets gekloond. De volgorde
/// volgt de id-lijsten op de zone, zodat downstream-berekeningen een
/// deterministische volgorde zien.
#[derive(Debug)]
pub struct ResolvedZone<'a> {
    /// De zone waarvoor is opgelost.
    pub zone: &'a Rekenzone,

    /// Energiefunctieruimten in deze zone (`Rekenzone::efr_ids`).
    pub efrs: Vec<&'a EnergiefunctieRuimte>,

    /// Opake constructies in deze zone (`Rekenzone::constructions`).
    pub constructions: Vec<&'a Construction>,

    /// Ramen in deze zone (`Rekenzone::windows`).
    pub windows: Vec<&'a Window>,

    /// Niet-transparante openingen in deze zone (`Rekenzone::openings`).
    pub openings: Vec<&'a Opening>,

    /// Lineaire koudebruggen (`Rekenzone::thermal_bridges_linear`).
    pub thermal_bridges_linear: Vec<&'a ThermalBridgeLinear>,

    /// Puntkoudebruggen (`Rekenzone::thermal_bridges_point`).
    pub thermal_bridges_point: Vec<&'a ThermalBridgePoint>,
}

/// Los de id-lijsten van `zone` op tegen de manifest-collecties.
///
/// Elke id in de zone-lijsten wordt opgezocht in de bijbehorende collectie.
/// De resulterende [`ResolvedZone`] bevat de objecten in dezelfde volgorde als
/// de id-lijsten. Ids in de collecties die *niet* door de zone worden
/// genoemd, worden genegeerd (een collectie kan objecten van meerdere zones
/// bevatten).
///
/// # Errors
///
/// [`ModelError::ReferenceNotFound`] zodra een id in een zone-lijst geen
/// overeenkomstig object in de bijbehorende collectie heeft. Het `kind`-veld
/// benoemt het gezochte type (`"Construction"`, `"Window"`, `"Opening"`,
/// `"ThermalBridgeLinear"`, `"ThermalBridgePoint"`, `"EnergiefunctieRuimte"`).
pub fn resolve_zone<'a>(
    zone: &'a Rekenzone,
    efrs: &'a [EnergiefunctieRuimte],
    constructions: &'a [Construction],
    windows: &'a [Window],
    openings: &'a [Opening],
    thermal_bridges_linear: &'a [ThermalBridgeLinear],
    thermal_bridges_point: &'a [ThermalBridgePoint],
) -> ModelResult<ResolvedZone<'a>> {
    let efr_by_id: HashMap<&str, &EnergiefunctieRuimte> =
        efrs.iter().map(|e| (e.id.as_str(), e)).collect();
    let construction_by_id: HashMap<&str, &Construction> =
        constructions.iter().map(|c| (c.id.as_str(), c)).collect();
    let window_by_id: HashMap<&str, &Window> =
        windows.iter().map(|w| (w.id.as_str(), w)).collect();
    let opening_by_id: HashMap<&str, &Opening> =
        openings.iter().map(|o| (o.id.as_str(), o)).collect();
    let linear_bridge_by_id: HashMap<&str, &ThermalBridgeLinear> = thermal_bridges_linear
        .iter()
        .map(|b| (b.id.as_str(), b))
        .collect();
    let point_bridge_by_id: HashMap<&str, &ThermalBridgePoint> = thermal_bridges_point
        .iter()
        .map(|b| (b.id.as_str(), b))
        .collect();

    Ok(ResolvedZone {
        zone,
        efrs: resolve_ids(&zone.efr_ids, &efr_by_id, "EnergiefunctieRuimte")?,
        constructions: resolve_ids(&zone.constructions, &construction_by_id, "Construction")?,
        windows: resolve_ids(&zone.windows, &window_by_id, "Window")?,
        openings: resolve_ids(&zone.openings, &opening_by_id, "Opening")?,
        thermal_bridges_linear: resolve_ids(
            &zone.thermal_bridges_linear,
            &linear_bridge_by_id,
            "ThermalBridgeLinear",
        )?,
        thermal_bridges_point: resolve_ids(
            &zone.thermal_bridges_point,
            &point_bridge_by_id,
            "ThermalBridgePoint",
        )?,
    })
}

/// Zoek elke id op in `lookup`; geef een [`ModelError::ReferenceNotFound`]
/// terug zodra er één ontbreekt.
fn resolve_ids<'a, T>(
    ids: &[String],
    lookup: &HashMap<&str, &'a T>,
    kind: &str,
) -> ModelResult<Vec<&'a T>> {
    ids.iter()
        .map(|id| {
            lookup
                .get(id.as_str())
                .copied()
                .ok_or_else(|| ModelError::ReferenceNotFound {
                    kind: kind.to_string(),
                    id: id.clone(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{ConstructionLayer, ThermalBridgeCategory};
    use crate::location::{Orientation, Tilt};
    use crate::zoning::UsageFunction;

    fn construction(id: &str) -> Construction {
        Construction {
            id: id.into(),
            name: format!("wand {id}"),
            layers: vec![ConstructionLayer {
                material_name: "beton".into(),
                thickness: 0.2,
                lambda: 2.0,
            }],
            r_si: 0.13,
            r_se: 0.04,
        }
    }

    fn window(id: &str, construction_id: &str) -> Window {
        Window::new(
            id,
            construction_id,
            2.0,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.1,
            0.6,
            0.25,
        )
        .unwrap()
    }

    fn opening(id: &str) -> Opening {
        Opening {
            id: id.into(),
            area: 1.8,
            u_value: 1.4,
            orientation: Orientation::Noord,
            tilt: Tilt::VERTICAL,
        }
    }

    fn tb_linear(id: &str) -> ThermalBridgeLinear {
        ThermalBridgeLinear {
            id: id.into(),
            length: 10.0,
            psi: 0.1,
            category: ThermalBridgeCategory::AansluitingVloerGevel,
        }
    }

    fn tb_point(id: &str) -> ThermalBridgePoint {
        ThermalBridgePoint {
            id: id.into(),
            chi: 0.02,
            category: ThermalBridgeCategory::DoorgaandeConstructie,
        }
    }

    fn efr(id: &str, zone_id: &str) -> EnergiefunctieRuimte {
        EnergiefunctieRuimte {
            id: id.into(),
            name: format!("efr {id}"),
            rekenzone_id: zone_id.into(),
            floor_area: 30.0,
            usage_function: UsageFunction::Woonfunctie,
        }
    }

    fn zone_with(ids: &[&str]) -> Rekenzone {
        Rekenzone {
            id: "rz1".into(),
            name: "Hoofdzone".into(),
            gebouw_id: "g1".into(),
            floor_area: 60.0,
            volume: 150.0,
            efr_ids: vec!["efr1".into()],
            constructions: ids.iter().map(|s| (*s).to_string()).collect(),
            windows: vec!["w1".into()],
            openings: vec!["o1".into()],
            thermal_bridges_linear: vec!["tbl1".into()],
            thermal_bridges_point: vec!["tbp1".into()],
        }
    }

    #[test]
    fn resolves_all_collections_in_id_order() {
        let zone = zone_with(&["c2", "c1"]);
        let constructions = vec![construction("c1"), construction("c2"), construction("c3")];
        let windows = vec![window("w1", "c1")];
        let openings = vec![opening("o1")];
        let linear_bridges = vec![tb_linear("tbl1")];
        let point_bridges = vec![tb_point("tbp1")];
        let efrs = vec![efr("efr1", "rz1")];

        let resolved = resolve_zone(
            &zone,
            &efrs,
            &constructions,
            &windows,
            &openings,
            &linear_bridges,
            &point_bridges,
        )
        .unwrap();

        // Volgorde volgt de zone-id-lijst (c2 vóór c1), niet de collectie.
        assert_eq!(resolved.constructions.len(), 2);
        assert_eq!(resolved.constructions[0].id, "c2");
        assert_eq!(resolved.constructions[1].id, "c1");
        // c3 zit in de collectie maar niet in de zone → genegeerd.
        assert_eq!(resolved.efrs.len(), 1);
        assert_eq!(resolved.windows.len(), 1);
        assert_eq!(resolved.openings.len(), 1);
        assert_eq!(resolved.thermal_bridges_linear.len(), 1);
        assert_eq!(resolved.thermal_bridges_point.len(), 1);
        assert_eq!(resolved.zone.id, "rz1");
    }

    #[test]
    fn unknown_construction_id_is_reference_not_found() {
        let zone = zone_with(&["c1", "does-not-exist"]);
        let constructions = vec![construction("c1")];
        let err = resolve_zone(
            &zone,
            &[efr("efr1", "rz1")],
            &constructions,
            &[window("w1", "c1")],
            &[opening("o1")],
            &[tb_linear("tbl1")],
            &[tb_point("tbp1")],
        )
        .unwrap_err();

        match err {
            ModelError::ReferenceNotFound { kind, id } => {
                assert_eq!(kind, "Construction");
                assert_eq!(id, "does-not-exist");
            }
            other => panic!("verwacht ReferenceNotFound, kreeg {other:?}"),
        }
    }

    #[test]
    fn unknown_window_id_names_window_kind() {
        let mut zone = zone_with(&["c1"]);
        zone.windows = vec!["ghost".into()];
        let err = resolve_zone(
            &zone,
            &[efr("efr1", "rz1")],
            &[construction("c1")],
            &[],
            &[opening("o1")],
            &[tb_linear("tbl1")],
            &[tb_point("tbp1")],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ModelError::ReferenceNotFound { kind, .. } if kind == "Window"
        ));
    }

    #[test]
    fn empty_id_lists_resolve_to_empty_vecs() {
        let zone = Rekenzone {
            id: "rz1".into(),
            name: "Leeg".into(),
            gebouw_id: "g1".into(),
            floor_area: 0.0,
            volume: 0.0,
            efr_ids: vec![],
            constructions: vec![],
            windows: vec![],
            openings: vec![],
            thermal_bridges_linear: vec![],
            thermal_bridges_point: vec![],
        };
        let resolved = resolve_zone(&zone, &[], &[], &[], &[], &[], &[]).unwrap();
        assert!(resolved.constructions.is_empty());
        assert!(resolved.efrs.is_empty());
        assert!(resolved.windows.is_empty());
    }
}
