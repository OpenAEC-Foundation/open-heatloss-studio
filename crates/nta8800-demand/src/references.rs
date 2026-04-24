//! Norm-identifier constanten voor NTA 8800:2025+C1:2026 hoofdstuk 7
//! (maandbalans warmte- en koudebehoefte).
//!
//! Zie [`nta8800_model::references`](../../nta8800_model/references/index.html)
//! voor de naamgevings-conventie. Deze crate levert de sub-verzameling
//! constanten voor §7.4/§7.5/§7.6 — de maand-balans met benuttingsfactor η.

// ---------------------------------------------------------------------------
// Paragrafen hoofdstuk 7
// ---------------------------------------------------------------------------

/// §7.1 — Inleiding maandmethode: energiebehoefte per gebouwzone.
pub const NTA_8800_2025_PARAG7_1: &str = "nta_8800_2025_parag7_1";

/// §7.3 — Warmte- en koude-overdracht per maand.
pub const NTA_8800_2025_PARAG7_3: &str = "nta_8800_2025_parag7_3";

/// §7.4 — Maandelijkse warmtebehoefte Q_H;nd.
pub const NTA_8800_2025_PARAG7_4: &str = "nta_8800_2025_parag7_4";

/// §7.5 — Maandelijkse koudebehoefte Q_C;nd.
pub const NTA_8800_2025_PARAG7_5: &str = "nta_8800_2025_parag7_5";

/// §7.6 — Benuttingsfactor voor warmte- (η_H,gn) en koudewinst (η_C,ls).
pub const NTA_8800_2025_PARAG7_6: &str = "nta_8800_2025_parag7_6";

/// §7.7 — Effectieve interne warmtecapaciteit (lookup in `nta8800-tables`).
pub const NTA_8800_2025_PARAG7_7: &str = "nta_8800_2025_parag7_7";

/// §7.8 — Tijdconstante τ_H / τ_C van de rekenzone.
pub const NTA_8800_2025_PARAG7_8: &str = "nta_8800_2025_parag7_8";

// ---------------------------------------------------------------------------
// Formules §7.4 / §7.5 — maand-balans
// ---------------------------------------------------------------------------

/// Formule (7.4) — maandelijkse netto warmtebehoefte.
///
/// `Q_H;nd;zi;mi = Q_H;ht;zi;mi − η_H;gn;zi;mi · Q_H;gn;zi;mi`
///
/// Clamped op 0 (geen negatieve warmtebehoefte).
pub const NTA_8800_2025_FORMULE7_4: &str = "nta_8800_2025_formule7_4";

/// Formule (7.5) — γ_H (dimensieloze winst/verlies-verhouding voor verwarming).
///
/// `γ_H = Q_H;gn / Q_H;ht`
pub const NTA_8800_2025_FORMULE7_5: &str = "nta_8800_2025_formule7_5";

/// Formule (7.6) — benuttingsfactor warmtewinst η_H,gn.
///
/// Voor `γ_H ≠ 1`:   `η_H;gn = (1 − γ_H^a_H) / (1 − γ_H^(a_H+1))`
/// Voor `γ_H = 1`:   `η_H;gn = a_H / (a_H + 1)`
/// Voor `γ_H ≤ 0`:   `η_H;gn = 1` (alle warmtewinst benutbaar).
pub const NTA_8800_2025_FORMULE7_6: &str = "nta_8800_2025_formule7_6";

/// Formule (7.7) — dimensieloze parameter a_H voor warmte-benuttingsfactor.
///
/// `a_H = a_H;0 + τ / τ_H;0`, met `a_H;0 = 1,0` en `τ_H;0 = 15 h`
/// (vaste constanten uit tabel 7.4, maandmethode).
pub const NTA_8800_2025_FORMULE7_7: &str = "nta_8800_2025_formule7_7";

/// Formule (7.10) — maandelijkse netto koudebehoefte.
///
/// `Q_C;nd;zi;mi = Q_C;gn;zi;mi − η_C;ls;zi;mi · Q_C;ht;zi;mi`
///
/// Clamped op 0 (geen negatieve koudebehoefte).
pub const NTA_8800_2025_FORMULE7_10: &str = "nta_8800_2025_formule7_10";

/// Formule (7.11) — γ_C (inverse van γ_H voor koelmodus).
///
/// `γ_C = Q_C;gn / Q_C;ht` — zelfde numerieke definitie als γ_H maar met
/// koel-setpoint Q_C.
pub const NTA_8800_2025_FORMULE7_11: &str = "nta_8800_2025_formule7_11";

/// Formule (7.12) — benuttingsfactor koudeverlies η_C,ls.
///
/// Voor `γ_C ≠ 1`:   `η_C;ls = (1 − γ_C^(-a_C)) / (1 − γ_C^(-(a_C+1)))`
/// Voor `γ_C = 1`:   `η_C;ls = a_C / (a_C + 1)`
/// Voor `γ_C → 0`:   `η_C;ls = 0` (geen winst te benutten).
pub const NTA_8800_2025_FORMULE7_12: &str = "nta_8800_2025_formule7_12";

/// Formule (7.13) — parameter a_C voor koude-benuttingsfactor.
///
/// `a_C = a_C;0 + τ / τ_C;0`, met `a_C;0 = 1,0` en `τ_C;0 = 15 h`.
pub const NTA_8800_2025_FORMULE7_13: &str = "nta_8800_2025_formule7_13";

// ---------------------------------------------------------------------------
// Formules §7.8 — tijdconstante
// ---------------------------------------------------------------------------

/// Formule (7.17) — tijdconstante τ van een rekenzone in uren.
///
/// `τ = (C_m;int;eff;zi / 3600) / (H_tr + H_ve)`  [h]
///
/// Met `C_m` in J/K (zie `nta8800_tables::thermal_capacity::zone_heat_capacity`),
/// factor 3600 converteert J/K naar Wh/K zodat de noemer (W/K) samen met de
/// teller het resultaat in uren geeft.
pub const NTA_8800_2025_FORMULE7_17: &str = "nta_8800_2025_formule7_17";

// ---------------------------------------------------------------------------
// Zonnewinst & interne warmtelast
// ---------------------------------------------------------------------------

/// §7.9 — Zoninstraling door transparante gebouwelementen.
pub const NTA_8800_2025_PARAG7_9: &str = "nta_8800_2025_parag7_9";

/// Formule (7.33) — zoninstraling per maand per transparant element.
///
/// `Q_sol;wi;mi = A_sol;wi · I_sol;or;mi · r_d;wi · (1 − F_F;wi)`
///
/// waarbij `A_sol = A_w · g × F_sh` en `I_sol` in MJ/m² per maand.
pub const NTA_8800_2025_FORMULE7_33: &str = "nta_8800_2025_formule7_33";

/// §7.10 — Interne warmtelast per gebruiksfunctie (tabel 7.6).
pub const NTA_8800_2025_PARAG7_10: &str = "nta_8800_2025_parag7_10";

/// Tabel 7.6 — forfaitaire interne warmtelast Φ_int in W/m² per gebruiksfunctie.
///
/// Woonfunctie: 3,0 W/m² (jaarrond), utiliteit: varieert per functie, typisch
/// 3,5–6,0 W/m² overdag.
pub const NTA_8800_2025_TABEL7_6: &str = "nta_8800_2025_tabel7_6";

/// Formule (7.35) — maandelijkse interne warmtewinst.
///
/// `Q_int;mi = Φ_int · A_g · t_mi · 0,0036`  [MJ]
/// met `Φ_int` in W/m², `A_g` in m², `t_mi` in h, factor 0,0036 = 3600/10^6.
pub const NTA_8800_2025_FORMULE7_35: &str = "nta_8800_2025_formule7_35";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG7_1,
        NTA_8800_2025_PARAG7_3,
        NTA_8800_2025_PARAG7_4,
        NTA_8800_2025_PARAG7_5,
        NTA_8800_2025_PARAG7_6,
        NTA_8800_2025_PARAG7_7,
        NTA_8800_2025_PARAG7_8,
        NTA_8800_2025_PARAG7_9,
        NTA_8800_2025_PARAG7_10,
        NTA_8800_2025_FORMULE7_4,
        NTA_8800_2025_FORMULE7_5,
        NTA_8800_2025_FORMULE7_6,
        NTA_8800_2025_FORMULE7_7,
        NTA_8800_2025_FORMULE7_10,
        NTA_8800_2025_FORMULE7_11,
        NTA_8800_2025_FORMULE7_12,
        NTA_8800_2025_FORMULE7_13,
        NTA_8800_2025_FORMULE7_17,
        NTA_8800_2025_FORMULE7_33,
        NTA_8800_2025_FORMULE7_35,
        NTA_8800_2025_TABEL7_6,
    ];

    #[test]
    fn canonical_strings_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(set.len(), ALL.len());
    }

    #[test]
    fn all_constants_have_prefix() {
        for id in ALL {
            assert!(id.starts_with("nta_8800_2025_"), "missing prefix: {id:?}");
        }
    }

    #[test]
    fn no_whitespace_in_canonical_strings() {
        for id in ALL {
            assert!(!id.chars().any(char::is_whitespace), "ws in {id:?}");
        }
    }
}
