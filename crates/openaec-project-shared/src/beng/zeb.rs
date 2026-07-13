//! Bijlage AB (informatief) — ZEB-indicator `EweP,ZEB;Tot`.
//!
//! NTA 8800:2025+C1:2026 bijlage AB introduceert de **primair-totale-energie-
//! indicator** `EweP,ZEB;Tot` (EPBD IV, "Zero Emission Building"). Anders dan de
//! primair-**fossiele** BENG 2-indicator (`EwePTot`, hoofdstuk 5) salderert de
//! ZEB-indicator lokaal opgewekte elektriciteit **niet volledig**: afgenomen
//! net-energie krijgt een andere primaire-totale-energiefactor (`fP,ZEB;del;el =
//! 1,35`, tabel AB.2) dan teruggeleverde energie (`fP,ZEB;exp;el,ren = 1`), en op
//! eigen perceel **direct gebruikte** PV telt met factor 0 (AB.65 + tabel AB.2).
//!
//! Deze module is een **losse, additieve** implementatie naast de norm-conforme
//! BENG-keten: [`crate::beng::compute_beng`] blijft BENG 1/2/3 exact volgens
//! hoofdstuk 5 berekenen (volledige saldering, `fP;exp;el = 1,45`). De
//! ZEB-indicator is informatief (AB.0: "komt vooralsnog niet op het
//! energieprestatiecertificaat") maar wordt door de norm gelijktijdig met de
//! reguliere berekening bepaald.
//!
//! ## Waarom deze indicator naast BENG 2 bestaat
//!
//! Certified Uniec 3.3.x crediteert PV maar ~64 % (maand-directgebruik-fractie),
//! terwijl de norm-conforme 2025+C1-BENG 2 de export **volledig** salderert (zie
//! `docs/2026-07-12-f3d8-norm-analyse-saldering.md`). Het directgebruik-
//! fractiemodel dat certified Uniec benadert, staat in 2025+C1 **uitsluitend** in
//! bijlage AB (formule AB.65 + tabel AB.1). Deze module reproduceert dat model —
//! als eigen grootheid, niet als vervanging van BENG 2 (anti-fudge).
//!
//! ## Scope-beperkingen (V1)
//!
//! - **Geen batterij** (AB.2.3.3): het invoer-DTO codeert geen opslag; zonder
//!   ≥ 5 kWh geldt `fBAT;el;corr = 0` (tabel AB, p. 1155) en vallen de
//!   batterij-termen `EBAT;tot;in/out` exact weg — de implementatie zet ze op 0.
//! - **Geen niet-hernieuwbare eigen productie** (WKK): `Epr;el,nren;tot = 0`
//!   (formule AB.64), dus de nren-directgebruik- en nren-export-termen zijn 0.
//! - **Geen export van warmte/koude** (AB.10 `Eexp;T;gi`): OPMERKING 3 bij de
//!   norm stelt dat hiervoor geen bepalingsmethode bestaat; term = 0.
//! - **Stadswarmte/-koude** als dienst-drager wordt (nog) niet ondersteund: de
//!   temperatuurafhankelijke `fP,ZEB;weeg` (tabel AB.2) is niet gemodelleerd. De
//!   orchestrator laat de indicator dan weg (`None`) i.p.v. een fout te fabriceren.

use nta8800_model::zoning::UsageFunction;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_ep::EnergyCarrier as EpCarrier;

/// Omrekenfactor MJ → kWh (1 kWh = 3,6 MJ). Bijlage AB rekent in kWh.
const MJ_PER_KWH: f64 = 3.6;

/// Primaire-totale-energiefactor voor uit het net afgenomen elektriciteit
/// `fP,ZEB;del;el` (NTA 8800:2025+C1:2026 tabel AB.2, p. 1156).
const F_P_ZEB_DEL_EL: f64 = 1.35;
/// Weegfactor bij de primaire-totale-energiefactor voor elektriciteit
/// `fP,ZEB;weeg;el` (tabel AB.2). Voor zowel afname als hernieuwbare export = 1.
const F_P_ZEB_WEEG_EL: f64 = 1.0;
/// Primaire-totale-energiefactor voor geëxporteerde hernieuwbare elektriciteit
/// `fP,ZEB;exp;el,ren` (tabel AB.2).
const F_P_ZEB_EXP_EL_REN: f64 = 1.0;
/// Bovengrensfractie op het directgebruik t.o.v. de EP-elektriciteitsvraag in
/// formule (AB.65): `Epr;el,ren;directuse = Min[fdu × Epr;el,ren;tot; 0,3 × EEPus;el]`.
const DIRECT_USE_EP_CAP_FRACTION: f64 = 0.3;

/// Tabel AB.1 (p. 1153) — maandelijkse directgebruik-fractie `fdu;el,ren;mi`,
/// **woningbouw** (Januari eerst).
const FDU_WONINGBOUW: [f64; 12] = [
    0.75, 0.75, 0.5, 0.25, 0.25, 0.15, 0.15, 0.15, 0.25, 0.5, 0.75, 0.75,
];
/// Tabel AB.1 — `fdu;el,ren;mi`, utiliteitsbouw **onderwijsfunctie** (juli/aug 0,01).
const FDU_ONDERWIJS: [f64; 12] = [
    0.55, 0.55, 0.35, 0.20, 0.20, 0.15, 0.01, 0.01, 0.20, 0.35, 0.55, 0.55,
];
/// Tabel AB.1 — `fdu;el,ren;mi`, utiliteitsbouw **andere gebruiksfuncties**.
const FDU_ANDERE: [f64; 12] = [
    0.55, 0.55, 0.35, 0.20, 0.20, 0.15, 0.15, 0.15, 0.20, 0.35, 0.55, 0.55,
];

/// Kies de tabel AB.1-directgebruik-fracties bij een gebruiksfunctie.
///
/// Woonfunctie → woningbouw-kolom; onderwijsfunctie → de aparte onderwijs-kolom
/// (juli/aug 0,01); alle overige functies → "andere gebruiksfuncties". Bij
/// gemengd gebruik schrijft de norm een A_g-gewogen fractie voor (p. 1153) — dat
/// vergt een multi-zone-invoermodel (F5) en valt buiten deze V1.
#[must_use]
fn direct_use_fractions(usage: UsageFunction) -> [f64; 12] {
    match usage {
        UsageFunction::Woonfunctie => FDU_WONINGBOUW,
        UsageFunction::Onderwijsfunctie => FDU_ONDERWIJS,
        _ => FDU_ANDERE,
    }
}

/// Primaire-totale-energiefactor `fP,ZEB;del;ci` voor een **niet-elektrische**
/// afgenomen energiedrager (tabel AB.2, p. 1156), inclusief de weegfactor.
///
/// Aardgas (ook waterstof), stookolie en biomassa hebben `fP,ZEB;del = 1` en
/// `weeg = 1`. `None` betekent: geen ZEB-ondersteunde factor voor deze drager
/// (stadswarmte/-koude — temperatuurafhankelijk, niet gemodelleerd; of een
/// elektriciteits-/PV-drager die niet via dit pad hoort te lopen).
#[must_use]
fn zeb_nonelectric_del_factor(carrier: EpCarrier) -> Option<f64> {
    match carrier {
        EpCarrier::Aardgas | EpCarrier::Biomassa | EpCarrier::Pellets => Some(1.0),
        // Stadswarmte: temperatuurafhankelijke weegfactor (0,45/0,29/0,14) — F5.
        // Elektriciteit/PV horen via het elektriciteitspad, niet hier.
        EpCarrier::Stadswarmte
        | EpCarrier::Elektriciteit
        | EpCarrier::HernieuwbareElektriciteit => None,
    }
}

/// Invoer voor de ZEB-indicator-berekening (bijlage AB), per maand in kWh.
///
/// Alle maandprofielen staan met Januari op index 0 (idem `MonthlyProfile`).
#[derive(Debug, Clone, PartialEq)]
pub struct ZebInputs {
    /// Maandelijkse EP-relevante elektriciteitsvraag `EEPus;el;mi` [kWh]
    /// (NTA 8800 §5.5.3): de som van de eindelektriciteit voor alle
    /// EP-diensten (verwarming, tapwater, koeling, ventilatoren, …) vóór
    /// PV-verrekening.
    pub monthly_ep_electricity_kwh: [f64; 12],
    /// Maandelijkse op eigen perceel opgewekte **hernieuwbare** elektriciteit
    /// `Epr;el,ren;tot;mi` [kWh] (PV/wind), formule (AB.63).
    pub monthly_renewable_pv_kwh: [f64; 12],
    /// Maandelijkse primaire-totale energie van de **niet-elektrische** dragers
    /// (gas, biomassa, …): `Σ EEPdel;ci × fP,ZEB;del;ci × fP,ZEB;weeg;ci` [kWh]
    /// (formules AB.10/AB.11). 0 voor all-electric.
    pub monthly_nonelectric_primary_kwh: [f64; 12],
    /// Gebruiksfunctie → tabel AB.1-directgebruik-fracties.
    pub usage: UsageFunction,
    /// Gebruiksoppervlak `A_g;tot` [m²] (formule AB.1).
    pub a_g_m2: f64,
}

/// Resultaat van de bijlage-AB ZEB-indicator.
///
/// Losse, additieve informatieve output naast BENG 1/2/3. Zie de moduledoc voor
/// de scope-beperkingen (geen batterij/WKK/warmte-export/stadswarmte).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ZebIndicator {
    /// `EweP,ZEB;Tot` [kWh/(m²·jr)] — de ZEB-indicator, naar boven afgerond op een
    /// veelvoud van 0,01 (formule AB.1). Mag negatief zijn bij PV-overschot.
    pub ewep_zeb_tot_kwh_m2: f64,
    /// Jaarlijks karakteristiek primair-totaal-energiegebruik `EP,ZEB;Tot;an`
    /// [kWh] (formule AB.9), vóór deling door `A_g` en afronding.
    pub ep_zeb_tot_an_kwh: f64,
    /// Jaarlijks op eigen perceel direct gebruikte hernieuwbare PV
    /// `Epr;el,ren;directuse` [kWh] (formule AB.65) — diagnostisch.
    pub direct_use_kwh: f64,
    /// Jaarlijks geëxporteerde hernieuwbare PV `Eexp;el,ren;tot` [kWh]
    /// (formule AB.61) — diagnostisch.
    pub export_kwh: f64,
    /// Direct-gebruik-aandeel van de totale PV-opbrengst [0..=1]
    /// (`Σ directuse / Σ PV`) — het "zelfgebruik-percentage" dat de ZEB-indicator
    /// van de volledig-salderende BENG 2 onderscheidt.
    pub self_use_fraction: f64,
}

/// Rond naar boven af op een veelvoud van 0,01 (formule AB.1: "Rond […] naar
/// boven af op een veelvoud van 0,01"). Werkt ook voor negatieve waarden.
#[must_use]
fn round_up_to_cent(x: f64) -> f64 {
    (x * 100.0).ceil() / 100.0
}

/// Bereken de ZEB-indicator `EweP,ZEB;Tot` volgens bijlage AB.
///
/// Implementeert het maandmodel AB.9/AB.10 voor het all-electric + PV-geval
/// (geen batterij, geen WKK, geen warmte-/koude-export — zie moduledoc):
///
/// 1. **Directgebruik** (AB.65 + bovengrenzen AB.67/AB.68):
///    `directuse = Min[fdu × PV; 0,3 × EEPus;el]`, begrensd op `≤ EEPus;el` en
///    `≤ PV`.
/// 2. **Afgenomen elektriciteit** (AB.15): `EEPdel,ZEB;el = EEPus;el − directuse`.
/// 3. **Geëxporteerde hernieuwbare elektriciteit** (AB.61): `PV − directuse`.
/// 4. **Maandtotaal** (AB.10/AB.11a/AB.13): `EEPdel × 1 × 1,35 − export × 1 × 1`
///    plus de niet-elektrische primair-totale energie.
/// 5. **Jaarsom** (AB.9) `/A_g` (AB.1), naar boven afgerond op 0,01.
#[must_use]
pub fn compute_zeb_indicator(inputs: &ZebInputs) -> ZebIndicator {
    let fdu = direct_use_fractions(inputs.usage);

    let mut ep_tot_an = 0.0_f64;
    let mut direct_use_an = 0.0_f64;
    let mut export_an = 0.0_f64;
    let mut pv_an = 0.0_f64;

    for (month, &fdu_m) in fdu.iter().enumerate() {
        let eep = inputs.monthly_ep_electricity_kwh[month].max(0.0);
        let pv = inputs.monthly_renewable_pv_kwh[month].max(0.0);
        pv_an += pv;

        // (AB.65) directgebruik hernieuwbare PV, met de bovengrenzen (AB.67) ≤
        // EEPus;el en (AB.68) ≤ Epr;el,ren;tot. Geen batterij (V1): EBAT;tot;out = 0.
        let direct_use = (fdu_m * pv)
            .min(DIRECT_USE_EP_CAP_FRACTION * eep)
            .min(eep)
            .min(pv)
            .max(0.0);

        // (AB.15) afgenomen elektriciteit (geen nren-directgebruik, geen batterij).
        let eep_del = (eep - direct_use).max(0.0);
        // (AB.61) geëxporteerde hernieuwbare elektriciteit (geen batterij-opslag).
        let export = (pv - direct_use).max(0.0);

        // (AB.11a) afgenomen primair-totaal + (AB.13) vermeden export-primair →
        // (AB.10) maandtotaal, plus de niet-elektrische dragers (AB.11).
        let ep_del_el = eep_del * F_P_ZEB_WEEG_EL * F_P_ZEB_DEL_EL;
        let ep_exp_el_ren = export * F_P_ZEB_WEEG_EL * F_P_ZEB_EXP_EL_REN;
        ep_tot_an += ep_del_el - ep_exp_el_ren + inputs.monthly_nonelectric_primary_kwh[month];

        direct_use_an += direct_use;
        export_an += export;
    }

    let ewep = round_up_to_cent(ep_tot_an / inputs.a_g_m2);
    ZebIndicator {
        ewep_zeb_tot_kwh_m2: ewep,
        ep_zeb_tot_an_kwh: ep_tot_an,
        direct_use_kwh: direct_use_an,
        export_kwh: export_an,
        self_use_fraction: if pv_an > 0.0 { direct_use_an / pv_an } else { 0.0 },
    }
}

/// Vouw één dienst-eindenergieprofiel [MJ] in de ZEB-accumulatoren.
///
/// - Elektrische diensten (warmtepomp, ventilator, koeling) → `monthly_el_mj`
///   (de EP-elektriciteitsvraag `EEPus;el`).
/// - Gas/biomassa → `monthly_nonel_primary_kwh` met `fP,ZEB;del;ci = 1`
///   (tabel AB.2), meteen omgerekend MJ → kWh.
///
/// Retourneert `false` als de drager niet ZEB-ondersteund is (stadswarmte) — de
/// orchestrator laat de indicator dan weg i.p.v. een verkeerde factor te kiezen.
#[must_use]
pub(crate) fn fold_zeb_service(
    monthly_use_mj: &[f64; 12],
    bac_factor: f64,
    carrier: EpCarrier,
    monthly_el_mj: &mut [f64; 12],
    monthly_nonel_primary_kwh: &mut [f64; 12],
) -> bool {
    match carrier {
        EpCarrier::Elektriciteit => {
            for i in 0..12 {
                monthly_el_mj[i] += monthly_use_mj[i] * bac_factor;
            }
            true
        }
        EpCarrier::Aardgas | EpCarrier::Biomassa | EpCarrier::Pellets => {
            let Some(f) = zeb_nonelectric_del_factor(carrier) else {
                return false;
            };
            for i in 0..12 {
                monthly_nonel_primary_kwh[i] += monthly_use_mj[i] * bac_factor / MJ_PER_KWH * f;
            }
            true
        }
        EpCarrier::Stadswarmte | EpCarrier::HernieuwbareElektriciteit => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_inputs() -> ZebInputs {
        ZebInputs {
            monthly_ep_electricity_kwh: [0.0; 12],
            monthly_renewable_pv_kwh: [0.0; 12],
            monthly_nonelectric_primary_kwh: [0.0; 12],
            usage: UsageFunction::Woonfunctie,
            a_g_m2: 100.0,
        }
    }

    #[test]
    fn no_pv_all_electricity_is_delivered_at_1_35() {
        // Zonder PV: alle EP-elektriciteit is afname → EP,ZEB = Σ EEPus × 1,35.
        let mut inp = base_inputs();
        inp.monthly_ep_electricity_kwh = [100.0; 12]; // 1200 kWh/jr
        let z = compute_zeb_indicator(&inp);
        assert!((z.ep_zeb_tot_an_kwh - 1200.0 * 1.35).abs() < 1e-9);
        assert!(z.direct_use_kwh.abs() < 1e-12);
        assert!(z.export_kwh.abs() < 1e-12);
        // 1620 / 100 m² = 16,2 → afronding blijft 16,2.
        assert!((z.ewep_zeb_tot_kwh_m2 - 16.2).abs() < 1e-9);
    }

    #[test]
    fn direct_use_is_capped_at_30pct_of_demand_in_summer() {
        // Juli: veel PV, weinig vraag → directgebruik gekapt op 0,3·EEPus (AB.65),
        // niet fdu·PV (0,15·PV zou hoger zijn dan 0,3·EEPus).
        let mut inp = base_inputs();
        // Alleen juli (index 6) actief.
        inp.monthly_ep_electricity_kwh[6] = 100.0;
        inp.monthly_renewable_pv_kwh[6] = 1000.0;
        let z = compute_zeb_indicator(&inp);
        // fdu_juli(woning)=0,15 → 0,15·1000 = 150; cap 0,3·100 = 30 → directuse 30.
        assert!((z.direct_use_kwh - 30.0).abs() < 1e-9);
        // export = 1000 − 30 = 970; afname = 100 − 30 = 70.
        assert!((z.export_kwh - 970.0).abs() < 1e-9);
        // EP = 70·1,35 − 970·1 = 94,5 − 970 = −875,5 kWh (negatief: PV-overschot).
        assert!((z.ep_zeb_tot_an_kwh - (70.0 * 1.35 - 970.0)).abs() < 1e-9);
    }

    #[test]
    fn winter_direct_use_follows_fdu_times_pv() {
        // Januari: fdu=0,75; als 0,75·PV < 0,3·EEPus wint fdu·PV (AB.65 min-arm).
        let mut inp = base_inputs();
        inp.monthly_ep_electricity_kwh[0] = 1000.0; // 0,3·EEPus = 300
        inp.monthly_renewable_pv_kwh[0] = 100.0; // 0,75·100 = 75 < 300
        let z = compute_zeb_indicator(&inp);
        assert!((z.direct_use_kwh - 75.0).abs() < 1e-9);
        assert!((z.export_kwh - 25.0).abs() < 1e-9);
    }

    #[test]
    fn nonelectric_primary_adds_linearly() {
        // Gas-bijdrage staat los van het PV-directgebruik-model (AB.11).
        let mut inp = base_inputs();
        inp.monthly_nonelectric_primary_kwh = [50.0; 12]; // 600 kWh/jr × 1
        let z = compute_zeb_indicator(&inp);
        assert!((z.ep_zeb_tot_an_kwh - 600.0).abs() < 1e-9);
    }

    #[test]
    fn round_up_to_cent_rounds_upward() {
        assert!((round_up_to_cent(16.201) - 16.21).abs() < 1e-9);
        assert!((round_up_to_cent(-4.401) - (-4.40)).abs() < 1e-9);
    }

    #[test]
    fn fold_electric_service_accumulates_demand() {
        let mut el = [0.0; 12];
        let mut nonel = [0.0; 12];
        let profile = [3.6; 12]; // 3,6 MJ = 1 kWh
        let ok = fold_zeb_service(&profile, 1.0, EpCarrier::Elektriciteit, &mut el, &mut nonel);
        assert!(ok);
        assert!((el[0] - 3.6).abs() < 1e-9); // nog in MJ; conversie bij ZebInputs
        assert!(nonel.iter().all(|v| v.abs() < 1e-12));
    }

    #[test]
    fn fold_district_heat_is_unsupported() {
        let mut el = [0.0; 12];
        let mut nonel = [0.0; 12];
        let ok = fold_zeb_service(&[10.0; 12], 1.0, EpCarrier::Stadswarmte, &mut el, &mut nonel);
        assert!(!ok, "stadswarmte is niet ZEB-ondersteund in V1");
    }
}
