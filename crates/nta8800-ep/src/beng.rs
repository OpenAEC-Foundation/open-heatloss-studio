//! BENG 1/2/3-indicatoren en -grenswaarden.
//!
//! Twee gescheiden werelden die hier samenkomen:
//!
//! - **De indicatoren** ([`BengIndicators`]) zijn een *herformulering* van
//!   NTA 8800:2025+C1:2026-grootheden naar de drie kentallen die het beleid
//!   toetst. NTA 8800 zelf kent de term "BENG" niet; het rekent de
//!   grootheden `E_weH+C;nd;ventsys=C1` (§5.4), het karakteristiek
//!   primair-fossiele-energiegebruik (§5.5) en het aandeel hernieuwbare
//!   energie (§5.6). De vertaling naar BENG 1/2/3 is een presentatiestap.
//! - **De grenswaarden** ([`BengLimits`]) staan niet in NTA 8800 maar in het
//!   **Besluit bouwwerken leefomgeving (Bbl), artikel 4.149** (nieuwbouw-eis;
//!   oorsprong Staatsblad 2019, 501). De oververhittingsgrens (TOjuli, §5.7)
//!   staat in **Bbl artikel 4.149b**.
//!
//! ## Bronnen-discipline
//!
//! Alleen formules met een geverifieerde bron zijn geïmplementeerd:
//!
//! | Onderdeel | Status | Bron |
//! |---|---|---|
//! | BENG 1-indicator | ✅ | NTA 8800 §5.4 formule (5.4), p. 78 |
//! | BENG 2-indicator | ✅ | NTA 8800 §5.5 (karakt. primair-fossiel) |
//! | BENG 3-indicator | 🟡 vereenvoudigd | NTA 8800 §5.6 — zie [`BengIndicators::beng3_from_share`] |
//! | BENG 1-grens **grondgebonden woonfunctie** | ✅ | Bbl art. 4.149; RVO-voorbeeldconcepten p. 7 |
//! | BENG 2/3-grens woonfunctie | ✅ | Bbl art. 4.149 (30 kWh/m², 50 %) |
//! | TOjuli / GTO-grens | ✅ | Bbl art. 4.149b lid 1 / lid 2 |
//! | BENG 1-grens **woongebouw** | ❌ niet geïmplementeerd | bronconflict basiswaarde 60 vs 65 — zie [`beng1_limit_woonfunctie_grondgebonden`] |
//! | BENG-grenzen **utiliteit** | ❌ `None` | tabel 4.149-utiliteitswaarden niet geverifieerd — zie [`BengLimits::for_utiliteit`] |

use nta8800_model::zoning::UsageFunction;
use serde::{Deserialize, Serialize};

/// Omrekenfactor MJ → kWh (1 kWh = 3,6 MJ).
pub const MJ_PER_KWH: f64 = 3.6;

// ---------------------------------------------------------------------------
// Grenswaarde-constanten — Besluit bouwwerken leefomgeving (Bbl)
// ---------------------------------------------------------------------------

/// BENG 1-basiswaarde grondgebonden woonfunctie [kWh/m²·jr].
///
/// Vlakke grens voor compacte woningen (vormfactor `A_ls/A_g ≤ 1,5`).
/// Bbl art. 4.149, tabel 4.149 — "woonfunctie niet in een woongebouw".
pub const BENG1_WOONFUNCTIE_BASIS_KWH_M2: f64 = 55.0;

/// Onderste knik in de BENG 1-vormfactorcurve voor de grondgebonden
/// woonfunctie. Onder deze `A_ls/A_g` geldt de vlakke basiswaarde.
/// Bbl art. 4.149 (grondgebonden woning; woongebouw kent knik 1,83).
pub const BENG1_WOONFUNCTIE_KNIK_LAAG: f64 = 1.5;

/// Bovenste knik in de BENG 1-vormfactorcurve. Boven deze `A_ls/A_g` geldt
/// het steilere derde segment. Bbl art. 4.149.
pub const BENG1_WOONFUNCTIE_KNIK_HOOG: f64 = 3.0;

/// Helling van het middensegment `1,5 < A_ls/A_g ≤ 3,0` [kWh/m² per eenheid
/// vormfactor]. Bbl art. 4.149.
pub const BENG1_WOONFUNCTIE_HELLING_MIDDEN: f64 = 30.0;

/// Helling van het bovensegment `A_ls/A_g > 3,0`. Bbl art. 4.149.
pub const BENG1_WOONFUNCTIE_HELLING_HOOG: f64 = 50.0;

/// Toeslag op de BENG 1-grens bij een lichte bouwwijze [kWh/m²·jr].
///
/// Van toepassing als de naar gebruiksoppervlak gewogen gemiddelde
/// specifieke interne warmtecapaciteit ≤ 180 kJ/(m²·K) is (bepaald volgens
/// NTA 8800). Bbl art. 4.149 lid 4.
pub const LICHTE_BOUWWIJZE_TOESLAG_KWH_M2: f64 = 5.0;

/// BENG 2-grens woonfunctie: max. primair fossiel energiegebruik
/// [kWh/m²·jr]. Bbl art. 4.149, tabel 4.149.
pub const BENG2_LIMIT_WOONFUNCTIE_KWH_M2: f64 = 30.0;

/// BENG 3-grens woonfunctie: min. aandeel hernieuwbare energie [%].
/// Bbl art. 4.149, tabel 4.149.
pub const BENG3_LIMIT_WOONFUNCTIE_PCT: f64 = 50.0;

/// Grenswaarde voor de TOjuli-indicator (oververhitting), in K.
///
/// NTA 8800 §5.7.2 (formule 5.40, p. 115) definieert TOjuli;or,zi in K
/// (kelvin) — een geschatte temperatuurstijging, niet dimensieloos. De grens
/// 1,20 is dus 1,20 K. Een woonfunctie heeft ten hoogste deze waarde voor
/// iedere rekenzone en oriëntatie (maand juli). Bbl art. 4.149b lid 1.
/// Zie [`crate::tojuli`] voor de berekening.
pub const TOJULI_LIMIT: f64 = 1.20;

/// Alternatieve grens bij TOjuli-overschrijding: max. aantal gewogen
/// temperatuuroverschrijdingen (GTO) per verblijfsruimte [uren].
///
/// Alleen toepasbaar voor een woonfunctie niet in een woongebouw.
/// Bbl art. 4.149b lid 2.
pub const GTO_LIMIT_HOURS: f64 = 450.0;

// ---------------------------------------------------------------------------
// BENG-indicatoren (NTA 8800-grootheden, herformuleerd)
// ---------------------------------------------------------------------------

/// De drie berekende BENG-indicatoren van een gebouw of rekenzone.
///
/// Puur presentatiemodel: de waarden volgen rechtstreeks uit de
/// NTA 8800-keten. Deze struct rekent niet zelf; hij herformuleert.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BengIndicators {
    /// BENG 1 — energiebehoefte voor verwarming en koeling [kWh/(m²·jr)].
    pub beng1_kwh_per_m2: f64,

    /// BENG 2 — karakteristiek primair fossiel energiegebruik [kWh/(m²·jr)].
    pub beng2_kwh_per_m2: f64,

    /// BENG 3 — aandeel hernieuwbare energie [%].
    pub beng3_renewable_pct: f64,
}

impl BengIndicators {
    /// BENG 1 — specifieke energiebehoefte uit de jaarlijkse warmte- en
    /// koudebehoefte.
    ///
    /// NTA 8800:2025+C1:2026 §5.4.2, formule (5.4) (p. 78):
    /// `Q_H+C;nd;ventsys=C1 = Q_H;nd;ventsys=C1 + Q_C;nd;ventsys=C1`, gedeeld
    /// door `A_g` voor de specifieke waarde `E_weH+C;nd;ventsys=C1`.
    ///
    /// De invoer moet in MJ zijn; de norm rekent in kWh, dus deze functie
    /// deelt door [`MJ_PER_KWH`].
    ///
    /// **Belangrijke normconventie (§5.4.1):** deze indicator wordt bepaald
    /// met een *vast* ventilatiesysteem C1 (S.2.1) — dat kan afwijken van het
    /// werkelijke ventilatieconcept — en met vaste interne warmtelasten voor
    /// tapwater en verlichting, *exclusief* terugwinbare systeemverliezen.
    /// De caller is er verantwoordelijk voor dat `q_h_nd_mj`/`q_c_nd_mj` op
    /// die C1-basis zijn berekend (de keten-wiring, F2).
    ///
    /// **De/bevochtiging telt NIET mee** in BENG 1: de energiebehoefte is
    /// uitsluitend warmte- + koudebehoefte (§5.4.2). Vocht­huishouding
    /// (NTA 8800 H.12) valt onder het primair energiegebruik (BENG 2), niet
    /// onder de energiebehoefte.
    ///
    /// Retourneert `0.0` als `a_g` niet-eindig (NaN/±∞) of `<= 0.0` is; een
    /// niet-eindige `a_g` duidt op een fout in de keten-invoer en zou anders
    /// stil NaN doorgeven.
    // q_h_nd_mj / q_c_nd_mj zijn de norm-symbolen Q_H;nd en Q_C;nd — bewust
    // gelijkend, geen naamgevingsfout.
    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn beng1_from_demand(q_h_nd_mj: f64, q_c_nd_mj: f64, a_g: f64) -> f64 {
        debug_assert!(a_g.is_finite(), "a_g moet eindig zijn, kreeg {a_g}");
        if !a_g.is_finite() || a_g <= 0.0 {
            return 0.0;
        }
        (q_h_nd_mj + q_c_nd_mj) / MJ_PER_KWH / a_g
    }

    /// BENG 2 — karakteristiek primair fossiel energiegebruik.
    ///
    /// NTA 8800:2025+C1:2026 §5.5 ("Karakteristiek primaire-fossiele-
    /// energiegebruik van een gebouw"). De bestaande EP-keten levert dit als
    /// [`crate::EpResult::ep_total_mj_per_m2`] in MJ/m²; deze functie
    /// converteert naar kWh/(m²·jr) door te delen door [`MJ_PER_KWH`].
    ///
    /// Retourneert `0.0` bij een niet-eindige invoer (NaN/±∞), die op een
    /// fout in de EP-keten duidt in plaats van een geldig energiegebruik.
    #[must_use]
    pub fn beng2_from_ep(ep_total_mj_per_m2: f64) -> f64 {
        debug_assert!(
            ep_total_mj_per_m2.is_finite(),
            "ep_total_mj_per_m2 moet eindig zijn, kreeg {ep_total_mj_per_m2}"
        );
        if !ep_total_mj_per_m2.is_finite() {
            return 0.0;
        }
        ep_total_mj_per_m2 / MJ_PER_KWH
    }

    /// BENG 3 — aandeel hernieuwbare energie in procenten.
    ///
    /// NTA 8800:2025+C1:2026 §5.6 ("Hernieuwbare energie"). Neemt het
    /// hernieuwbaar aandeel `0.0..=1.0` uit [`crate::EpResult::ep_renewable_share`]
    /// en schaalt naar procenten.
    ///
    /// **Vereenvoudiging (overgenomen uit de EP-crate):** het onderliggende
    /// aandeel is een saldobenadering zonder net-metering en zonder temporele
    /// effecten (zie [`crate::calc::ep_score::renewable_share`]). Voor een
    /// formeel BENG 3-oordeel is de §5.6-net-meteringregel nodig; dat is
    /// bewust nog niet geïmplementeerd (F3-kalibratie).
    ///
    /// De invoer wordt naar `0.0..=1.0` geklemd: het aandeel is per definitie
    /// een fractie, en de vereenvoudigde upstream-berekening kan bij
    /// randgevallen buiten dat bereik komen. Een `debug_assert!` markeert zulke
    /// invoer tijdens ontwikkeling.
    #[must_use]
    pub fn beng3_from_share(renewable_share_0_1: f64) -> f64 {
        debug_assert!(
            (0.0..=1.0).contains(&renewable_share_0_1),
            "renewable_share_0_1 hoort in 0..=1 te liggen, kreeg {renewable_share_0_1}"
        );
        renewable_share_0_1.clamp(0.0, 1.0) * 100.0
    }

    /// Stelt de drie indicatoren samen uit de ketenuitvoer.
    ///
    /// - `q_h_nd_mj` / `q_c_nd_mj`: jaarlijkse warmte-/koudebehoefte [MJ] op
    ///   §5.4 C1-basis;
    /// - `a_g`: gebruiksoppervlakte [m²];
    /// - `ep_total_mj_per_m2`: karakteristiek primair fossiel [MJ/m²];
    /// - `renewable_share_0_1`: hernieuwbaar aandeel [0..=1].
    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn from_chain(
        q_h_nd_mj: f64,
        q_c_nd_mj: f64,
        a_g: f64,
        ep_total_mj_per_m2: f64,
        renewable_share_0_1: f64,
    ) -> Self {
        Self {
            beng1_kwh_per_m2: Self::beng1_from_demand(q_h_nd_mj, q_c_nd_mj, a_g),
            beng2_kwh_per_m2: Self::beng2_from_ep(ep_total_mj_per_m2),
            beng3_renewable_pct: Self::beng3_from_share(renewable_share_0_1),
        }
    }
}

// ---------------------------------------------------------------------------
// BENG-grenswaarden (Bbl art. 4.149)
// ---------------------------------------------------------------------------

/// De drie BENG-grenswaarden waaraan een gebouw moet voldoen.
///
/// Bron: Besluit bouwwerken leefomgeving, artikel 4.149 (tabel 4.149).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BengLimits {
    /// BENG 1 — max. energiebehoefte [kWh/(m²·jr)]. Voor de woonfunctie
    /// vormfactor-afhankelijk (zie [`beng1_limit_woonfunctie_grondgebonden`]).
    pub beng1_max_kwh_per_m2: f64,

    /// BENG 2 — max. primair fossiel energiegebruik [kWh/(m²·jr)].
    pub beng2_max_kwh_per_m2: f64,

    /// BENG 3 — min. aandeel hernieuwbare energie [%].
    pub beng3_min_pct: f64,
}

impl BengLimits {
    /// Grenswaarden voor de **grondgebonden woonfunctie** ("woonfunctie niet
    /// in een woongebouw"), zware/normale bouwwijze.
    ///
    /// De BENG 1-grens volgt de vormfactorformule
    /// [`beng1_limit_woonfunctie_grondgebonden`]; BENG 2 = 30 kWh/(m²·jr),
    /// BENG 3 = 50 %. Bbl art. 4.149, tabel 4.149.
    ///
    /// Voor een lichte bouwwijze (interne warmtecapaciteit ≤ 180 kJ/(m²·K)):
    /// gebruik [`BengLimits::for_woonfunctie_lichte_bouwwijze`].
    #[must_use]
    pub fn for_woonfunctie(als_ag: f64) -> Self {
        Self {
            beng1_max_kwh_per_m2: beng1_limit_woonfunctie_grondgebonden(als_ag),
            beng2_max_kwh_per_m2: BENG2_LIMIT_WOONFUNCTIE_KWH_M2,
            beng3_min_pct: BENG3_LIMIT_WOONFUNCTIE_PCT,
        }
    }

    /// Als [`BengLimits::for_woonfunctie`], maar met de lichte-bouwwijze-
    /// toeslag van [`LICHTE_BOUWWIJZE_TOESLAG_KWH_M2`] op de BENG 1-grens.
    /// Bbl art. 4.149 lid 4.
    #[must_use]
    pub fn for_woonfunctie_lichte_bouwwijze(als_ag: f64) -> Self {
        let mut limits = Self::for_woonfunctie(als_ag);
        limits.beng1_max_kwh_per_m2 += LICHTE_BOUWWIJZE_TOESLAG_KWH_M2;
        limits
    }

    /// Grenswaarden voor een utiliteitsfunctie.
    ///
    /// **Bewust niet geïmplementeerd — retourneert `None`.** De numerieke
    /// utiliteitswaarden in tabel 4.149 (per functie, deels met eigen
    /// vormfactorformules en gewogen menging bij gemengde functies, Bbl
    /// art. 4.149 lid 2) konden in deze fase niet één-op-één tegen de
    /// wettekst geverifieerd worden. Conform de anti-fudge-discipline worden
    /// geen ongeverifieerde grenzen hardcoded. Woonfunctie is wél volledig
    /// gedekt. Utiliteit is F5-scope.
    #[must_use]
    pub fn for_utiliteit(_function: UsageFunction) -> Option<Self> {
        None
    }
}

/// BENG 1-grens voor de **grondgebonden woonfunctie** als functie van de
/// vormfactor `A_ls/A_g` (verliesoppervlakte / gebruiksoppervlakte),
/// in kWh/(m²·jr).
///
/// ```text
/// A_ls/A_g ≤ 1,5        → 55
/// 1,5 < A_ls/A_g ≤ 3,0  → 55 + 30·(A_ls/A_g − 1,5)
/// A_ls/A_g > 3,0        → 100 + 50·(A_ls/A_g − 3,0)
/// ```
///
/// De curve is **continu**: bij `A_ls/A_g = 1,5` sluit het middensegment aan
/// op 55; bij `A_ls/A_g = 3,0` geeft het middensegment `55 + 30·1,5 = 100`,
/// exact het startpunt van het bovensegment.
///
/// # Bronnen
///
/// - Besluit bouwwerken leefomgeving, art. 4.149, tabel 4.149 (grondgebonden
///   woonfunctie); oorsprong Staatsblad 2019, 501.
/// - RVO "BENG-voorbeeldconcepten" p. 7: Tussenwoning M (`A_ls/A_g = 2,03`
///   → 70,9), Hoekwoning M (1,87 → 66,2), Vrijstaande L (2,14 → 74,1) —
///   gereproduceerd door deze formule op ±0,1 (afrondingsartefact van de
///   op 0,1 gepubliceerde vormfactoren).
///
/// # Bekende beperking
///
/// Geldt uitsluitend voor de grondgebonden woonfunctie. Een **woongebouw**
/// (appartementen) heeft een andere curve met knik bij `A_ls/A_g = 1,83`;
/// de basiswaarde daarvan wordt in de bronnen inconsistent opgegeven
/// (60 vs 65 kWh/m²). Die variant is daarom bewust niet geïmplementeerd.
#[must_use]
pub fn beng1_limit_woonfunctie_grondgebonden(als_ag: f64) -> f64 {
    if als_ag <= BENG1_WOONFUNCTIE_KNIK_LAAG {
        BENG1_WOONFUNCTIE_BASIS_KWH_M2
    } else if als_ag <= BENG1_WOONFUNCTIE_KNIK_HOOG {
        beng1_middensegment(als_ag)
    } else {
        // Middensegment op de bovenknik = 55 + 30·(3,0 − 1,5) = 100; hieruit
        // rekenen borgt continuïteit uit dezelfde constanten.
        let waarde_bij_knik_hoog = beng1_middensegment(BENG1_WOONFUNCTIE_KNIK_HOOG);
        BENG1_WOONFUNCTIE_HELLING_HOOG
            .mul_add(als_ag - BENG1_WOONFUNCTIE_KNIK_HOOG, waarde_bij_knik_hoog)
    }
}

/// Middensegment-waarde `55 + 30·(A_ls/A_g − 1,5)`; los gehouden zodat de
/// bovenknik-aansluiting (continuïteit) uit dezelfde bron rekent.
fn beng1_middensegment(als_ag: f64) -> f64 {
    BENG1_WOONFUNCTIE_HELLING_MIDDEN.mul_add(
        als_ag - BENG1_WOONFUNCTIE_KNIK_LAAG,
        BENG1_WOONFUNCTIE_BASIS_KWH_M2,
    )
}

// ---------------------------------------------------------------------------
// Toetsing
// ---------------------------------------------------------------------------

/// Toetsingsresultaat van één BENG-indicator tegen zijn grenswaarde.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IndicatorAssessment {
    /// Berekende indicatorwaarde.
    pub value: f64,
    /// Grenswaarde uit het Bbl.
    pub limit: f64,
    /// Voldoet de indicator aan de eis?
    pub pass: bool,
}

/// Volledige BENG-toetsing: de drie indicatoren tegen hun grenzen.
///
/// BENG 1 en 2 zijn maximum-eisen (`value ≤ limit`); BENG 3 is een
/// minimum-eis (`value ≥ limit`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BengAssessment {
    /// BENG 1 — energiebehoefte (maximum-eis).
    pub beng1: IndicatorAssessment,
    /// BENG 2 — primair fossiel energiegebruik (maximum-eis).
    pub beng2: IndicatorAssessment,
    /// BENG 3 — aandeel hernieuwbare energie (minimum-eis).
    pub beng3: IndicatorAssessment,
}

impl BengAssessment {
    /// Toetst de indicatoren tegen de grenswaarden.
    #[must_use]
    pub fn assess(indicators: &BengIndicators, limits: &BengLimits) -> Self {
        Self {
            beng1: IndicatorAssessment {
                value: indicators.beng1_kwh_per_m2,
                limit: limits.beng1_max_kwh_per_m2,
                pass: indicators.beng1_kwh_per_m2 <= limits.beng1_max_kwh_per_m2,
            },
            beng2: IndicatorAssessment {
                value: indicators.beng2_kwh_per_m2,
                limit: limits.beng2_max_kwh_per_m2,
                pass: indicators.beng2_kwh_per_m2 <= limits.beng2_max_kwh_per_m2,
            },
            beng3: IndicatorAssessment {
                value: indicators.beng3_renewable_pct,
                limit: limits.beng3_min_pct,
                pass: indicators.beng3_renewable_pct >= limits.beng3_min_pct,
            },
        }
    }

    /// `true` als alle drie de indicatoren voldoen.
    #[must_use]
    pub fn all_pass(&self) -> bool {
        self.beng1.pass && self.beng2.pass && self.beng3.pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- BENG 1-grens: RVO-ankers (grondgebonden woonfunctie) --------------
    // Bron RVO-voorbeeldconcepten p. 7. Tolerantie ±0,2: de gepubliceerde
    // vormfactoren zijn op 0,1 afgerond, wat de eis met max. ~0,1 verschuift.

    #[test]
    fn rvo_anker_tussenwoning_m() {
        let eis = beng1_limit_woonfunctie_grondgebonden(2.03);
        assert!((eis - 70.9).abs() < 0.2, "kreeg {eis}, verwacht 70,9");
    }

    #[test]
    fn rvo_anker_hoekwoning_m() {
        let eis = beng1_limit_woonfunctie_grondgebonden(1.87);
        assert!((eis - 66.2).abs() < 0.2, "kreeg {eis}, verwacht 66,2");
    }

    #[test]
    fn rvo_anker_vrijstaande_l() {
        let eis = beng1_limit_woonfunctie_grondgebonden(2.14);
        assert!((eis - 74.1).abs() < 0.2, "kreeg {eis}, verwacht 74,1");
    }

    // -- BENG 1-grens: continuïteit op de segmentgrenzen -------------------

    #[test]
    fn continuiteit_op_knik_laag() {
        let eps = 1e-9;
        let onder = beng1_limit_woonfunctie_grondgebonden(BENG1_WOONFUNCTIE_KNIK_LAAG - eps);
        let op = beng1_limit_woonfunctie_grondgebonden(BENG1_WOONFUNCTIE_KNIK_LAAG);
        let boven = beng1_limit_woonfunctie_grondgebonden(BENG1_WOONFUNCTIE_KNIK_LAAG + eps);
        assert!((op - 55.0).abs() < 1e-9);
        assert!((onder - op).abs() < 1e-6, "sprong bij knik 1,5: {onder} vs {op}");
        assert!((boven - op).abs() < 1e-6, "sprong bij knik 1,5: {boven} vs {op}");
    }

    #[test]
    fn continuiteit_op_knik_hoog() {
        let eps = 1e-9;
        let onder = beng1_limit_woonfunctie_grondgebonden(BENG1_WOONFUNCTIE_KNIK_HOOG - eps);
        let op = beng1_limit_woonfunctie_grondgebonden(BENG1_WOONFUNCTIE_KNIK_HOOG);
        let boven = beng1_limit_woonfunctie_grondgebonden(BENG1_WOONFUNCTIE_KNIK_HOOG + eps);
        assert!((op - 100.0).abs() < 1e-9, "middensegment op 3,0 hoort 100 te zijn: {op}");
        assert!((onder - op).abs() < 1e-6, "sprong bij knik 3,0: {onder} vs {op}");
        assert!((boven - op).abs() < 1e-6, "sprong bij knik 3,0: {boven} vs {op}");
    }

    #[test]
    fn beng1_vlak_onder_knik_laag() {
        assert!((beng1_limit_woonfunctie_grondgebonden(1.0) - 55.0).abs() < 1e-9);
        assert!((beng1_limit_woonfunctie_grondgebonden(0.5) - 55.0).abs() < 1e-9);
    }

    #[test]
    fn beng1_bovensegment() {
        // 3,5 → 100 + 50·(3,5 − 3,0) = 125.
        assert!((beng1_limit_woonfunctie_grondgebonden(3.5) - 125.0).abs() < 1e-9);
    }

    #[test]
    fn beng1_stijgt_monotoon() {
        let mut vorige = f64::NEG_INFINITY;
        let mut r = 0.5;
        while r <= 5.0 {
            let eis = beng1_limit_woonfunctie_grondgebonden(r);
            assert!(eis >= vorige - 1e-12, "niet-monotoon bij {r}");
            vorige = eis;
            r += 0.01;
        }
    }

    // -- Lichte bouwwijze --------------------------------------------------

    #[test]
    fn lichte_bouwwijze_toeslag() {
        let normaal = BengLimits::for_woonfunctie(2.03);
        let licht = BengLimits::for_woonfunctie_lichte_bouwwijze(2.03);
        assert!((licht.beng1_max_kwh_per_m2 - normaal.beng1_max_kwh_per_m2 - 5.0).abs() < 1e-9);
        // BENG 2/3 blijven ongewijzigd.
        assert!((licht.beng2_max_kwh_per_m2 - normaal.beng2_max_kwh_per_m2).abs() < 1e-9);
        assert!((licht.beng3_min_pct - normaal.beng3_min_pct).abs() < 1e-9);
    }

    // -- Woonfunctie-grenswaarden ------------------------------------------

    #[test]
    fn woonfunctie_beng2_beng3_vast() {
        let limits = BengLimits::for_woonfunctie(2.0);
        assert!((limits.beng2_max_kwh_per_m2 - 30.0).abs() < 1e-9);
        assert!((limits.beng3_min_pct - 50.0).abs() < 1e-9);
    }

    #[test]
    fn utiliteit_bewust_none() {
        assert!(BengLimits::for_utiliteit(UsageFunction::Kantoorfunctie).is_none());
    }

    // -- Indicatoren -------------------------------------------------------

    #[test]
    fn beng1_indicator_mj_naar_kwh() {
        // 36000 MJ warmte + 3600 MJ koude over 100 m² = 39600/3,6/100 = 110 kWh/m².
        let v = BengIndicators::beng1_from_demand(36_000.0, 3_600.0, 100.0);
        assert!((v - 110.0).abs() < 1e-9, "kreeg {v}");
    }

    #[test]
    fn beng1_indicator_zonder_oppervlak() {
        assert!((BengIndicators::beng1_from_demand(1000.0, 0.0, 0.0)).abs() < 1e-12);
    }

    #[test]
    fn beng1_indicator_negatief_oppervlak_fallback() {
        // Negatieve a_g is eindig: debug_assert passeert, guard geeft 0,0.
        assert!((BengIndicators::beng1_from_demand(1000.0, 200.0, -5.0)).abs() < 1e-12);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn beng1_indicator_niet_eindig_oppervlak_fallback() {
        // In release-builds (debug_assert uit) geven NaN/∞ de 0,0-fallback.
        assert_eq!(BengIndicators::beng1_from_demand(1000.0, 0.0, f64::NAN), 0.0);
        assert_eq!(BengIndicators::beng1_from_demand(1000.0, 0.0, f64::INFINITY), 0.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "a_g moet eindig zijn")]
    fn beng1_indicator_nan_oppervlak_debug_assert() {
        let _ = BengIndicators::beng1_from_demand(1000.0, 0.0, f64::NAN);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn beng2_indicator_niet_eindig_fallback() {
        assert_eq!(BengIndicators::beng2_from_ep(f64::NAN), 0.0);
        assert_eq!(BengIndicators::beng2_from_ep(f64::INFINITY), 0.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "ep_total_mj_per_m2 moet eindig zijn")]
    fn beng2_indicator_nan_debug_assert() {
        let _ = BengIndicators::beng2_from_ep(f64::NAN);
    }

    #[test]
    fn beng3_indicator_clamp_buiten_bereik() {
        // Klemwaarden gelden in alle build-modi (naast de debug_assert).
        #[cfg(not(debug_assertions))]
        {
            assert!((BengIndicators::beng3_from_share(1.4) - 100.0).abs() < 1e-9);
            assert!((BengIndicators::beng3_from_share(-0.2)).abs() < 1e-9);
        }
        // Geldige randwaarden mogen nooit paniek geven.
        assert!((BengIndicators::beng3_from_share(0.0)).abs() < 1e-9);
        assert!((BengIndicators::beng3_from_share(1.0) - 100.0).abs() < 1e-9);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "renewable_share_0_1 hoort in 0..=1")]
    fn beng3_indicator_buiten_bereik_debug_assert() {
        let _ = BengIndicators::beng3_from_share(1.4);
    }

    #[test]
    fn beng2_indicator_mj_naar_kwh() {
        // 108 MJ/m² → 30 kWh/m².
        assert!((BengIndicators::beng2_from_ep(108.0) - 30.0).abs() < 1e-9);
    }

    #[test]
    fn beng3_indicator_share_naar_pct() {
        assert!((BengIndicators::beng3_from_share(0.55) - 55.0).abs() < 1e-9);
    }

    // -- Toetsing ----------------------------------------------------------

    #[test]
    fn assess_alles_voldoet() {
        let indicators = BengIndicators {
            beng1_kwh_per_m2: 54.8,
            beng2_kwh_per_m2: 29.3,
            beng3_renewable_pct: 59.0,
        };
        // Tussenwoning M, vormfactor 2,03 → grens 70,9.
        let limits = BengLimits::for_woonfunctie(2.03);
        let assessment = BengAssessment::assess(&indicators, &limits);
        assert!(assessment.beng1.pass);
        assert!(assessment.beng2.pass);
        assert!(assessment.beng3.pass);
        assert!(assessment.all_pass());
    }

    #[test]
    fn assess_beng_faalt_correct() {
        let indicators = BengIndicators {
            beng1_kwh_per_m2: 80.0, // > 70,9
            beng2_kwh_per_m2: 35.0, // > 30
            beng3_renewable_pct: 40.0, // < 50
        };
        let limits = BengLimits::for_woonfunctie(2.03);
        let assessment = BengAssessment::assess(&indicators, &limits);
        assert!(!assessment.beng1.pass);
        assert!(!assessment.beng2.pass);
        assert!(!assessment.beng3.pass);
        assert!(!assessment.all_pass());
    }

    #[test]
    fn from_chain_bouwt_alle_drie() {
        let ind = BengIndicators::from_chain(36_000.0, 3_600.0, 100.0, 108.0, 0.55);
        assert!((ind.beng1_kwh_per_m2 - 110.0).abs() < 1e-9);
        assert!((ind.beng2_kwh_per_m2 - 30.0).abs() < 1e-9);
        assert!((ind.beng3_renewable_pct - 55.0).abs() < 1e-9);
    }
}
