//! Fase 4e — `summary.json` + `RESULT-*`-entities → [`Uniec3CertifiedResults`].
//!
//! Dit is de **certified referentie**: de door BengCert afgemelde uitkomsten die
//! naast de eigen `compute_beng`-uitkomst gelegd worden (F8 fase 4g). De
//! BENG-kernindicatoren + eisen + label komen uit `summary.json` (het pad van de
//! minste weerstand); de per-functie primaire energie, PV-productie en de
//! geometrie-kentallen uit de `RESULT-*`-entities. Zie analyse §5c.
//!
//! **Per-functie som-definitie (open vraag 2, empirisch geijkt op de goldens):**
//! per `RESULT-ENERGIEFUNCTIE_CAT` de som van `RES_ENER_PRIM` (dus zónder
//! hulpenergie). Dat reproduceert de certified `expected.json` exact
//! (heating 2551, tapw 1813, koeling 422, ventilatoren 443 voor Aalten). De
//! gebouw- en unit-niveau-instances worden beide gesommeerd; voor een
//! single-unit woning staat het unit-niveau op 0, dus dubbeltellen kan niet.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::parse::{parse_num, EntityIndex, Meta};

/// De door Uniec/BengCert **gecertificeerde** uitkomsten van één afgemeld
/// gebouw — het vergelijkingsobject naast de eigen `compute_beng`-uitkomst.
///
/// Alle velden zijn optioneel: ontbreekt een bron-veld, dan blijft het `None`
/// (tolerante extractie). Eenheden: BENG 1/2 in kWh/(m²·jr), BENG 3 in %,
/// primaire energie in kWh, oppervlakten in m².
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Uniec3CertifiedResults {
    /// App-versie waaruit geëxporteerd is (bv. `"3.3.3.1"`), provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,

    /// BENG 1 — energiebehoefte, kWh/(m²·jr) (`EP_BENG1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beng1_kwh_m2_jr: Option<f64>,
    /// BENG 1-eis (`EP_BENG1_EIS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beng1_limit_kwh_m2_jr: Option<f64>,
    /// BENG 2 — primair fossiel energiegebruik, kWh/(m²·jr) (`EP_BENG2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beng2_kwh_m2_jr: Option<f64>,
    /// BENG 2-eis (`EP_BENG2_EIS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beng2_limit_kwh_m2_jr: Option<f64>,
    /// BENG 3 — aandeel hernieuwbare energie, % (`EP_BENG3`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beng3_pct: Option<f64>,
    /// BENG 3-eis (`EP_BENG3_EIS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beng3_limit_pct: Option<f64>,

    /// TOjuli-waarde (`EP_TOJULI`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tojuli: Option<f64>,
    /// TOjuli-eis (`EP_TOJULI_EIS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tojuli_limit: Option<f64>,

    /// Energielabel (`EP_ENERGIELABEL`, bv. `"A+++"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_label: Option<String>,

    /// Primaire energie verwarming, kWh (Σ `RES_ENER_PRIM` van `RESULT_VERW`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heating_primary_kwh: Option<f64>,
    /// Primaire energie warm tapwater, kWh (`RESULT_TAPW`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hot_water_primary_kwh: Option<f64>,
    /// Primaire energie koeling, kWh (`RESULT_KOEL`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooling_primary_kwh: Option<f64>,
    /// Primaire energie ventilatoren, kWh (`RESULT_VENT`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fans_primary_kwh: Option<f64>,

    /// Opgewekte PV-elektriciteit, kWh (`RESULT-HERNIEUW_ELEKTR`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pv_production_kwh: Option<f64>,
    /// Netto koudebehoefte, kWh (`KOEL-OPWEK_GEL_KOUDE_NON`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooling_demand_kwh: Option<f64>,

    /// Netto warmtebehoefte, kWh/(m²·jr) (`RESULT-EP_WARMTEBEHOEFTE`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warmtebehoefte_kwh_m2: Option<f64>,
    /// Vormfactor A_ls/A_g (`RESULT-OPP_VORMFACTOR`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vormfactor: Option<f64>,
    /// Verliesoppervlak A_ls, m² (`RESULT-OPP_VERLOPP`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verlies_opp_m2: Option<f64>,
    /// Gebruiksoppervlak A_g, m² (`RESULT-OPP_GEBROPP`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gebruiks_opp_m2: Option<f64>,
}

/// Extraheer de certified resultaten uit `summary.json` + de `RESULT-*`-entities.
pub fn extract_results(
    summary: &serde_json::Value,
    idx: &EntityIndex,
    meta: &Meta,
) -> Uniec3CertifiedResults {
    let s = |key: &str| summary.get(key).and_then(|v| v.as_str());
    let sn = |key: &str| s(key).and_then(parse_num);

    // Per-functie primaire energie: Σ RES_ENER_PRIM per categorie.
    let mut heating = None;
    let mut tapw = None;
    let mut koel = None;
    let mut vent = None;
    for e in idx.of_type("RESULT-ENERGIEFUNCTIE") {
        let Some(prim) = e.num("RESULT-ENERGIEFUNCTIE_RES_ENER_PRIM") else {
            continue;
        };
        let slot = match e.prop("RESULT-ENERGIEFUNCTIE_CAT") {
            Some("RESULT_VERW") => &mut heating,
            Some("RESULT_TAPW") => &mut tapw,
            Some("RESULT_KOEL") => &mut koel,
            Some("RESULT_VENT") => &mut vent,
            _ => continue,
        };
        *slot = Some(slot.unwrap_or(0.0) + prim);
    }

    // Geometrie-/PV-kentallen uit de gevulde RESULT-ENERGIEGEBRUIK-instance.
    let gebruik = idx
        .of_type("RESULT-ENERGIEGEBRUIK")
        .into_iter()
        .find(|e| e.num("RESULT-OPP_GEBROPP").is_some());

    let cooling_demand = idx
        .first_of_type("KOEL-OPWEK")
        .and_then(|k| k.num_or_non("KOEL-OPWEK_GEL_KOUDE"));

    Uniec3CertifiedResults {
        app_version: meta.app_version(),
        beng1_kwh_m2_jr: sn("EP_BENG1"),
        beng1_limit_kwh_m2_jr: sn("EP_BENG1_EIS"),
        beng2_kwh_m2_jr: sn("EP_BENG2"),
        beng2_limit_kwh_m2_jr: sn("EP_BENG2_EIS"),
        beng3_pct: sn("EP_BENG3"),
        beng3_limit_pct: sn("EP_BENG3_EIS"),
        tojuli: sn("EP_TOJULI"),
        tojuli_limit: sn("EP_TOJULI_EIS"),
        energy_label: s("EP_ENERGIELABEL").map(str::to_string),
        heating_primary_kwh: heating,
        hot_water_primary_kwh: tapw,
        cooling_primary_kwh: koel,
        fans_primary_kwh: vent,
        pv_production_kwh: gebruik.and_then(|g| g.num("RESULT-HERNIEUW_ELEKTR")),
        cooling_demand_kwh: cooling_demand,
        warmtebehoefte_kwh_m2: gebruik.and_then(|g| g.num("RESULT-EP_WARMTEBEHOEFTE")),
        vormfactor: gebruik.and_then(|g| g.num("RESULT-OPP_VORMFACTOR")),
        verlies_opp_m2: gebruik.and_then(|g| g.num("RESULT-OPP_VERLOPP")),
        gebruiks_opp_m2: gebruik.and_then(|g| g.num("RESULT-OPP_GEBROPP")),
    }
}
