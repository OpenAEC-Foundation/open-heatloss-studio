//! Norm-identifier constanten voor NTA 8800:2025+C1:2026 hoofdstuk 13
//! (warm tapwater) en de daarbij behorende bijlagen T, U, W.
//!
//! Zie [`nta8800_model::references`](../../nta8800_model/references/index.html)
//! voor de naamgevings-conventie. Deze crate levert de sub-verzameling
//! constanten voor §13 — nettowarmtebehoefte, afgifte, distributie,
//! DWTW en opwekking.

// ---------------------------------------------------------------------------
// Paragrafen hoofdstuk 13
// ---------------------------------------------------------------------------

/// §13.1 — Energiegebruik per warmtapwatersysteem (overkoepelend principe).
pub const NTA_8800_2025_PARAG13_1: &str = "nta_8800_2025_parag13_1";

/// §13.1.2.3 formule (13.3) — energiegebruik opwekking per systeem/opwekker/
/// energiedrager: `E_W = Q_W;dis;nren × F_W;gen / η_W;gen;prac`.
pub const NTA_8800_2025_FORMULE13_3: &str = "nta_8800_2025_formule13_3";

/// §13.2 — Nettowarmtebehoefte `Q_W;nd` en tappatronen.
pub const NTA_8800_2025_PARAG13_2: &str = "nta_8800_2025_parag13_2";

/// Formule (13.15) — nettowarmtebehoefte categorie woningbouw
/// `Q_W;nd;zi,mi = Q_W;nd;spec;p × N_woon × N_P × (t_mi / t_an)`.
pub const NTA_8800_2025_FORMULE13_15: &str = "nta_8800_2025_formule13_15";

/// Formules (13.16) t/m (13.18) — aantal bewoners per woonfunctie
/// afhankelijk van gebruiksoppervlakte per woning A_g/N_woon.
pub const NTA_8800_2025_FORMULE13_16_18: &str = "nta_8800_2025_formule13_16_18";

/// Formule (13.19) — nettowarmtebehoefte categorie utiliteitsbouw
/// `Q_W;nd;zi,mi = Q_W;nd;spec;usi × A_g × (t_mi / t_an)`.
pub const NTA_8800_2025_FORMULE13_19: &str = "nta_8800_2025_formule13_19";

/// §13.2.3.1 — rekenwaarde woningbouw: 856 kWh/jaar per bewoner.
///
/// Voetnoot norm: specifiek gebruik tapwater = 40,29 l/dag per persoon bij 60 °C.
pub const NTA_8800_2025_PARAG13_2_3_1: &str = "nta_8800_2025_parag13_2_3_1";

/// §13.2.3.2 — rekenwaarden utiliteitsbouw (tabel 13.1).
pub const NTA_8800_2025_PARAG13_2_3_2: &str = "nta_8800_2025_parag13_2_3_2";

/// Tabel 13.1 — jaarlijkse specifieke nettowarmtebehoefte warm tapwater
/// per gebruiksfunctie, in kWh/m² per jaar.
///
/// - Bijeenkomstfunctie: 2,8
/// - Celfunctie: 4,2
/// - Gezondheidszorgfunctie met bedgebied: 15,3
/// - Gezondheidszorgfunctie overig: 2,8
/// - Kantoorfunctie: 1,4
/// - Logiesfunctie: 12,5
/// - Onderwijsfunctie: 1,4
/// - Sportfunctie: 12,5
/// - Winkelfunctie: 1,4
pub const NTA_8800_2025_TABEL13_1: &str = "nta_8800_2025_tabel13_1";

/// §13.3 — Afgifteverliezen (η_W;em).
pub const NTA_8800_2025_PARAG13_3: &str = "nta_8800_2025_parag13_3";

/// Formule (13.23) — afgifterendement gecombineerd keuken+badruimte:
/// `η_W;em = 1 / (C_W;nd;b / η_W;em;b + C_W;nd;k / η_W;em;k)`.
pub const NTA_8800_2025_FORMULE13_23: &str = "nta_8800_2025_formule13_23";

/// Tabel 13.2 — afgifterendement woningbouw per leidinglengte l_k (keuken)
/// en l_b (badruimte). Keuken start op 1,00 bij 0-2 m, daalt tot 0,24 bij
/// ≥ 14 m; badruimte start 1,00 bij 0-2 m, daalt tot 0,72 bij ≥ 14 m.
pub const NTA_8800_2025_TABEL13_2: &str = "nta_8800_2025_tabel13_2";

/// Tabel 13.3 — afgifterendement utiliteitsbouw: 1,0 bij ≤ 3 m,
/// 0,8 bij > 3 m gemiddelde uittapleidinglengte.
pub const NTA_8800_2025_TABEL13_3: &str = "nta_8800_2025_tabel13_3";

/// §13.4 — Distributieverliezen (η_W;dis).
pub const NTA_8800_2025_PARAG13_4: &str = "nta_8800_2025_parag13_4";

/// §13.5 — Warmteterugwinning uit douchewater.
pub const NTA_8800_2025_PARAG13_5: &str = "nta_8800_2025_parag13_5";

/// Formule (13.51) — warmtebijdrage DWTW woningbouw:
/// `Q_W;rcd;d = C_W;nd;sh × Q_W;nd;d × η_W;sh;rcd × f_prac;sh × C_W;sh;rcd;T × C_W;sh;rcd;conf`.
pub const NTA_8800_2025_FORMULE13_51: &str = "nta_8800_2025_formule13_51";

/// §13.6 — Voorraadvat-verliezen (niet in V1 scope).
pub const NTA_8800_2025_PARAG13_6: &str = "nta_8800_2025_parag13_6";

/// §13.7 — Zonne-energiesysteem (niet in V1 scope).
pub const NTA_8800_2025_PARAG13_7: &str = "nta_8800_2025_parag13_7";

/// §13.8 — Warmteopwekking / opwekkingsrendement `η_W;gen;prac`.
pub const NTA_8800_2025_PARAG13_8: &str = "nta_8800_2025_parag13_8";

// ---------------------------------------------------------------------------
// Bijlagen
// ---------------------------------------------------------------------------

/// Bijlage T — bepaling opwekkingsrendement warmtapwatertoestellen ten behoeve
/// van de koppeling met Gaskeur. V2 scope — V1 gebruikt één forfaitair η_gen
/// per [`crate::DhwGenerationSystem`] variant.
pub const NTA_8800_2025_BIJLAGE_T: &str = "nta_8800_2025_bijlage_t";

/// Bijlage U — bepaling rendement douchewaterwarmteterugwinning (DWTW).
/// V1 exposeert alleen het netto thermisch rendement η_W;sh;rcd; de volledige
/// meet-methodiek (correctie-factoren C_T, C_conf, meerdere douches) is V2.
pub const NTA_8800_2025_BIJLAGE_U: &str = "nta_8800_2025_bijlage_u";

/// Bijlage W — bepaling opwekkingsrendement boosterwarmtepompen.
/// Niet in V1 (V1 modelleert warmtepomp tapwater met user-supplied SCOP_W).
pub const NTA_8800_2025_BIJLAGE_W: &str = "nta_8800_2025_bijlage_w";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG13_1,
        NTA_8800_2025_FORMULE13_3,
        NTA_8800_2025_PARAG13_2,
        NTA_8800_2025_FORMULE13_15,
        NTA_8800_2025_FORMULE13_16_18,
        NTA_8800_2025_FORMULE13_19,
        NTA_8800_2025_PARAG13_2_3_1,
        NTA_8800_2025_PARAG13_2_3_2,
        NTA_8800_2025_TABEL13_1,
        NTA_8800_2025_PARAG13_3,
        NTA_8800_2025_FORMULE13_23,
        NTA_8800_2025_TABEL13_2,
        NTA_8800_2025_TABEL13_3,
        NTA_8800_2025_PARAG13_4,
        NTA_8800_2025_PARAG13_5,
        NTA_8800_2025_FORMULE13_51,
        NTA_8800_2025_PARAG13_6,
        NTA_8800_2025_PARAG13_7,
        NTA_8800_2025_PARAG13_8,
        NTA_8800_2025_BIJLAGE_T,
        NTA_8800_2025_BIJLAGE_U,
        NTA_8800_2025_BIJLAGE_W,
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
