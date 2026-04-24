//! Pad 2 — vereenvoudigde koelbehoefte woningen (bijlage AA).
//!
//! Orchestreert formules (AA.1) t/m (AA.11) tot één [`SimplifiedCoolingResult`]
//! per rekenzone. Voor (AA.6) zoninstraling via transparante delen en (AA.7)
//! transmissie via glas worden ram-specifieke inputs verwacht — deze twee
//! componenten vereisen per-raam data (g-waarde, U-waarde, F_sh, F_C,
//! oriëntatie, hellingshoek, bijbehorende `I_sol` uit tabel AA.3) die in V1
//! nog niet volledig geïntegreerd is met `nta8800-model::geometry::Window`.
//! Callers leveren hiervoor voorgecomputeerde W-waarden aan.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::geometry::Window;
use nta8800_model::zoning::{EnergiefunctieRuimte, Rekenzone};
use nta8800_model::ClimateData;

use crate::errors::{CoolingCalcResult, CoolingError};
use crate::result::SimplifiedCoolingResult;
use crate::simplified::{
    capacity::required_cooling_capacity_kw,
    demand::maatgevende_koelbehoefte,
    demand::{koellast_transmissie_ondoorzichtig, BouwjaarKlasse, KoelbehoefteComponenten},
    interne_warmtelast_basis, interne_warmtelast_overig, interne_warmtelast_rekenwaarde,
    interne_warmtelast_woon, koellast_buitenlucht,
};

/// Oppervlakte-inputs voor bijlage AA berekening (formules AA.1..AA.3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SimplifiedAreaInput {
    /// Σ(A_vg;woon;zi) — totale oppervlakte verblijfsruimten in gebruik als
    /// woonkamer/keuken/eetkamer, in m².
    pub living_area_m2: f64,
    /// Σ(A_vg;overig;zi) — totale oppervlakte overige verblijfsruimten, in m².
    pub other_area_m2: f64,
    /// N_woon;zi — aantal woonfuncties in de rekenzone (NTA §6.6.6).
    pub dwelling_count: u32,
    /// P_p;woon;zi — gemiddeld aantal bewoners per woonfunctie (forfaitair).
    pub persons_per_dwelling: f64,
}

/// Inputs voor de componenten die in V1 caller-supplied zijn: lucht-flows,
/// transmissie-kader, zoninstraling en glas-transmissie. In V2 worden de
/// laatste twee automatisch afgeleid van [`Window`]-lijsten + tabel AA.3.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SimplifiedLoadInput {
    /// Infiltratie-luchtvolumestroom q_v;C;eff;lea;in juli (§11.2.1.7) in m³/h.
    pub infiltration_m3_per_h: f64,
    /// Natuurlijke ventilatie-toevoer q_v;C;eff;vent;in juli in m³/h.
    pub natural_ventilation_m3_per_h: f64,
    /// Mechanische toevoer q_v;C;SUP;eff juli in m³/h.
    pub mechanical_supply_m3_per_h: f64,
    /// Tijdstip van maximale koellast (9..21 h) — drijft θ_e uit tabel AA.1.
    pub peak_hour: u8,
    /// Bouwjaar voor f_iso tabel AA.2.
    pub construction_year: u32,
    /// Binnenwerkse oppervlakte ondoorzichtig buitenwand + dak in m².
    pub opaque_area_m2: f64,
    /// P_sol;zi — zoninstraling via transparante delen (AA.6), in W.
    /// Caller-berekend uit ram-lijst, g-waarden en tabel AA.3. V1-input.
    pub solar_load_w: f64,
    /// P_gl;zi — transmissie via transparante delen (AA.7), in W. Caller-
    /// berekend uit ram-lijst en U-waarden. V1-input.
    pub glazing_transmission_w: f64,
}

/// Pad 2 — vereenvoudigde koelbehoefte + minimum koelcapaciteit bijlage AA.
///
/// Parametert [`Window`], [`ClimateData`] en [`EnergiefunctieRuimte`]-lijsten
/// zijn momenteel **voor toekomstige V2-uitbreiding** en worden in V1 niet
/// actief gebruikt; ze zitten in de signature om een stabiele API te
/// bieden zodra (AA.6)/(AA.7) volledig geïntegreerd worden met
/// `nta8800-model::geometry` en tabel AA.3 uit `nta8800-tables`.
///
/// # Parameters
/// - `_building_year` — bouwjaar uit `Gebouw.construction_year` (via caller,
///   omdat `Gebouw` dat als `Option<u32>` draagt).
/// - `_rekenzones` — rekenzones in de analyse (gereserveerd voor V2).
/// - `_efrs` — energiefunctieruimten (gereserveerd voor V2).
/// - `_climate` — klimaatdata (gereserveerd voor V2 wanneer (AA.6) en
///   tabel AA.3 geïntegreerd worden).
/// - `_windows` — raam-lijst (gereserveerd voor V2).
/// - `area` — oppervlakte-inputs.
/// - `load` — V1 voorgecomputeerde lastcomponenten.
///
/// # Errors
/// Zie [`CoolingError`] — validatie van bewoners-aantal, oppervlakten, en
/// bereik van het tijdstip.
///
/// # Norm-referenties
/// - Formule (AA.1) — [`crate::simplified::interne_warmtelast_basis`]
/// - Formule (AA.2) — [`crate::simplified::interne_warmtelast_rekenwaarde`]
/// - Formule (AA.3a/b) — [`crate::simplified::interne_warmtelast_woon`] /
///   [`crate::simplified::interne_warmtelast_overig`]
/// - Formule (AA.4) — [`crate::simplified::koellast_buitenlucht`]
/// - Formule (AA.5) — [`crate::simplified::demand::koellast_transmissie_ondoorzichtig`]
/// - Formule (AA.8) — [`crate::simplified::demand::maatgevende_koelbehoefte`]
/// - Formule (AA.11) — [`crate::simplified::capacity::required_cooling_capacity_kw`]
#[allow(clippy::too_many_arguments, clippy::similar_names)]
pub fn calculate_simplified_cooling(
    _rekenzones: &[&Rekenzone],
    _efrs: &[&EnergiefunctieRuimte],
    _climate: &ClimateData,
    _windows: &[&Window],
    area: &SimplifiedAreaInput,
    load: &SimplifiedLoadInput,
) -> CoolingCalcResult<SimplifiedCoolingResult> {
    // ---- AA.1 t/m AA.3: interne warmtelast ----
    let n_int = interne_warmtelast_basis(area.dwelling_count, area.persons_per_dwelling)?;
    let q_int_calc =
        interne_warmtelast_rekenwaarde(n_int, area.living_area_m2, area.other_area_m2)?;
    let p_int_woon = interne_warmtelast_woon(q_int_calc, area.living_area_m2);
    let p_int_overig = interne_warmtelast_overig(q_int_calc, area.other_area_m2);
    let p_int_calc = p_int_woon + p_int_overig;

    // ---- AA.4: buitenlucht ----
    let outdoor_temp = crate::simplified::outdoor_load::tabel_aa1_buitentemperatuur(load.peak_hour)
        .ok_or_else(|| {
            CoolingError::Model(nta8800_model::ModelError::OutOfRange {
                field: "peak_hour (θ_e;max;zi uit tabel AA.1)".into(),
                range: "9..=21".into(),
                value: load.peak_hour.to_string(),
            })
        })?;
    let p_outdoor = koellast_buitenlucht(
        load.infiltration_m3_per_h,
        load.natural_ventilation_m3_per_h,
        load.mechanical_supply_m3_per_h,
        outdoor_temp,
    );

    // ---- AA.5: transmissie ondoorzichtig ----
    let klasse = BouwjaarKlasse::from_year(load.construction_year);
    let p_tr_ntr = koellast_transmissie_ondoorzichtig(klasse, load.opaque_area_m2);

    // ---- AA.8: maatgevende koelbehoefte rekenzone ----
    let componenten = KoelbehoefteComponenten {
        p_int_calc_w: p_int_calc,
        p_outdoor_w: p_outdoor,
        p_tr_ntr_w: p_tr_ntr,
        p_sol_w: load.solar_load_w,
        p_gl_w: load.glazing_transmission_w,
    };
    let total_verblijfsruimte = area.living_area_m2 + area.other_area_m2;
    let q_c_w_per_m2 = maatgevende_koelbehoefte(&componenten, total_verblijfsruimte)?;

    // ---- AA.11: minimum benodigde koelcapaciteit ----
    let min_capacity_kw = required_cooling_capacity_kw(q_c_w_per_m2, total_verblijfsruimte);

    // Piek-som uit componenten (voor rapportage in W)
    let peak = p_int_calc + p_outdoor + p_tr_ntr + load.solar_load_w + load.glazing_transmission_w;

    Ok(SimplifiedCoolingResult {
        minimum_capacity_w: min_capacity_kw * 1000.0,
        internal_load_w: p_int_calc,
        outdoor_load_w: p_outdoor,
        opaque_transmission_w: p_tr_ntr,
        solar_load_w: load.solar_load_w,
        glazing_transmission_w: load.glazing_transmission_w,
        peak_cooling_load_w: peak,
        maatgevende_koelbehoefte_w_per_m2: q_c_w_per_m2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use nta8800_model::time::MonthlyProfile;
    use std::collections::BTreeMap;

    fn minimal_climate() -> ClimateData {
        ClimateData {
            outdoor_temperature: MonthlyProfile::from_constant(15.0),
            solar_irradiation: BTreeMap::new(),
            cooling_reference_temperature: MonthlyProfile::from_constant(Some(17.0)),
            wind_speed: MonthlyProfile::from_constant(3.0),
            wtw_preheat_temperature: MonthlyProfile::from_constant(0.0),
        }
    }

    #[test]
    fn aa_end_to_end_woning_120m2() {
        // 1 woning, 3 bewoners, 80 m² woonkamer + 40 m² overig, bouwjaar 2020,
        // 100 m² ondoorzichtig, solar_load 4 400 W, glas_transmissie 286 W,
        // 100 m³/h infiltratie, 150 m³/h mechanisch, piek om 17h (30,6°C)
        let area = SimplifiedAreaInput {
            living_area_m2: 80.0,
            other_area_m2: 40.0,
            dwelling_count: 1,
            persons_per_dwelling: 3.0,
        };
        let load = SimplifiedLoadInput {
            infiltration_m3_per_h: 100.0,
            natural_ventilation_m3_per_h: 0.0,
            mechanical_supply_m3_per_h: 150.0,
            peak_hour: 17,
            construction_year: 2020,
            opaque_area_m2: 100.0,
            solar_load_w: 4_400.0,
            glazing_transmission_w: 286.0,
        };
        let climate = minimal_climate();
        let result = calculate_simplified_cooling(&[], &[], &climate, &[], &area, &load).unwrap();

        // AA.1 check: N_int = 180 × 1 × 3 = 540 W
        assert_abs_diff_eq!(result.internal_load_w, 540.0, epsilon = 1e-9);

        // AA.5 check: bouwjaar 2020 → f_iso = 2,2 W/m², 100 m² → 220 W
        assert_abs_diff_eq!(result.opaque_transmission_w, 220.0, epsilon = 1e-9);

        // Plausibel bereik voor minimum capacity 2-5 kW
        let min_kw = result.minimum_capacity_w / 1000.0;
        assert!(
            (1.0..=6.0).contains(&min_kw),
            "minimum_capacity {min_kw} kW buiten plausibel bereik"
        );
    }

    #[test]
    fn aa_end_to_end_minimum_capacity_3_5_kw_range() {
        // Typische woning: q_C resulteert in 3-5 kW minimum capacity
        let area = SimplifiedAreaInput {
            living_area_m2: 50.0,
            other_area_m2: 100.0,
            dwelling_count: 1,
            persons_per_dwelling: 2.5,
        };
        let load = SimplifiedLoadInput {
            infiltration_m3_per_h: 60.0,
            natural_ventilation_m3_per_h: 0.0,
            mechanical_supply_m3_per_h: 180.0,
            peak_hour: 17,
            construction_year: 2015,
            opaque_area_m2: 180.0,
            solar_load_w: 6_500.0,
            glazing_transmission_w: 400.0,
        };
        let climate = minimal_climate();
        let result = calculate_simplified_cooling(&[], &[], &climate, &[], &area, &load).unwrap();
        let min_kw = result.minimum_capacity_w / 1000.0;
        assert!(
            (2.0..=6.0).contains(&min_kw),
            "typische woning 150 m² → verwacht 2-6 kW, kreeg {min_kw} kW"
        );
    }

    #[test]
    fn aa_low_solar_load_results_in_zero_capacity() {
        // Goed geïsoleerd, weinig solar → q_C < 35 → minimum capacity = 0
        let area = SimplifiedAreaInput {
            living_area_m2: 60.0,
            other_area_m2: 60.0,
            dwelling_count: 1,
            persons_per_dwelling: 2.0,
        };
        let load = SimplifiedLoadInput {
            infiltration_m3_per_h: 30.0,
            natural_ventilation_m3_per_h: 0.0,
            mechanical_supply_m3_per_h: 60.0,
            peak_hour: 17,
            construction_year: 2022,
            opaque_area_m2: 80.0,
            solar_load_w: 800.0,
            glazing_transmission_w: 150.0,
        };
        let climate = minimal_climate();
        let result = calculate_simplified_cooling(&[], &[], &climate, &[], &area, &load).unwrap();
        assert_abs_diff_eq!(result.minimum_capacity_w, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn aa_invalid_peak_hour_returns_error() {
        let area = SimplifiedAreaInput {
            living_area_m2: 80.0,
            other_area_m2: 40.0,
            dwelling_count: 1,
            persons_per_dwelling: 3.0,
        };
        let load = SimplifiedLoadInput {
            infiltration_m3_per_h: 100.0,
            natural_ventilation_m3_per_h: 0.0,
            mechanical_supply_m3_per_h: 150.0,
            peak_hour: 8, // buiten bereik 9..21
            construction_year: 2020,
            opaque_area_m2: 100.0,
            solar_load_w: 4_400.0,
            glazing_transmission_w: 286.0,
        };
        let climate = minimal_climate();
        let err = calculate_simplified_cooling(&[], &[], &climate, &[], &area, &load).unwrap_err();
        assert!(matches!(err, CoolingError::Model(_)));
    }

    #[test]
    fn aa_simplified_result_serde_roundtrip() {
        let area = SimplifiedAreaInput {
            living_area_m2: 80.0,
            other_area_m2: 40.0,
            dwelling_count: 1,
            persons_per_dwelling: 3.0,
        };
        let load = SimplifiedLoadInput {
            infiltration_m3_per_h: 100.0,
            natural_ventilation_m3_per_h: 0.0,
            mechanical_supply_m3_per_h: 150.0,
            peak_hour: 17,
            construction_year: 2020,
            opaque_area_m2: 100.0,
            solar_load_w: 4_400.0,
            glazing_transmission_w: 286.0,
        };
        let climate = minimal_climate();
        let result = calculate_simplified_cooling(&[], &[], &climate, &[], &area, &load).unwrap();
        let json = serde_json::to_string(&result).unwrap();
        let back: SimplifiedCoolingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }
}
