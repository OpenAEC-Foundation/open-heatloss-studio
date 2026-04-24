//! Norm-identifier constanten voor NTA 8800:2025+C1:2026 hoofdstuk 14
//! (verlichting) en bijlage Y (daglichttoetreding hellende ramen).
//!
//! Zie [`nta8800_model::references`](../../nta8800_model/references/index.html)
//! voor de naamgevings-conventie. Deze crate levert de sub-verzameling
//! constanten voor §14 — utiliteit + woon — plus bijlage Y.

// ---------------------------------------------------------------------------
// Paragrafen hoofdstuk 14
// ---------------------------------------------------------------------------

/// §14.1 — Principe energiebehoefte voor verlichting (W_L + W_P splitsing).
pub const NTA_8800_2025_PARAG14_1: &str = "nta_8800_2025_parag14_1";

/// §14.2 — Energiebehoefte verlichting (onderscheid woon- vs utiliteit).
pub const NTA_8800_2025_PARAG14_2: &str = "nta_8800_2025_parag14_2";

/// §14.2.1 — Energiebehoefte verlichting woonfunctie.
///
/// V1 niet geïmplementeerd — woonfunctie-verlichting wordt voor de
/// nEP-indicator op 0 kWh/m² gesteld (formule 14.2 met W_L;spec = 0).
pub const NTA_8800_2025_PARAG14_2_1: &str = "nta_8800_2025_parag14_2_1";

/// §14.2.2 — Energiebehoefte verlichting utiliteitsfuncties (V1 scope).
pub const NTA_8800_2025_PARAG14_2_2: &str = "nta_8800_2025_parag14_2_2";

/// §14.3 — Geïnstalleerd vermogen voor verlichting.
pub const NTA_8800_2025_PARAG14_3: &str = "nta_8800_2025_parag14_3";

/// §14.3.4 — Forfaitaire rekenwaarden P_n;spec (tabel 14.3).
pub const NTA_8800_2025_PARAG14_3_4: &str = "nta_8800_2025_parag14_3_4";

/// §14.4 — Nieuwwaarde-compensatiefactor F_C.
pub const NTA_8800_2025_PARAG14_4: &str = "nta_8800_2025_parag14_4";

/// §14.5 — Aanwezigheid-afhankelijkheidsfactoren F_o;D en F_o;N.
pub const NTA_8800_2025_PARAG14_5: &str = "nta_8800_2025_parag14_5";

/// §14.6 — Daglichtafhankelijkheidsfactor F_D.
pub const NTA_8800_2025_PARAG14_6: &str = "nta_8800_2025_parag14_6";

// ---------------------------------------------------------------------------
// Formules hoofdstuk 14
// ---------------------------------------------------------------------------

/// Formule (14.7) — jaarlijkse energiebehoefte voor verlichting per
/// verlichtingszone.
///
/// ```text
/// W_L;j = {(P_n;j × F_C;j) × [(t_D × F_o;D;j × F_D;j) + (t_N × F_o;N;j)]} / 1000  [kWh]
/// ```
///
/// V1 decomposeert deze naar maandelijkse waarden door `(t_D × F_o;D × F_D +
/// t_N × F_o;N) / 8760` als gecombineerde jaar-bezettingsfactor `F_u` te
/// nemen en dat te vermenigvuldigen met de kalenderuren per maand.
pub const NTA_8800_2025_FORMULE14_7: &str = "nta_8800_2025_formule14_7";

/// Formule (14.13) — forfaitaire rekenwaarde voor `P_n` op basis van
/// `P_n;spec` (tabel 14.3) en gebruiksoppervlakte.
///
/// ```text
/// P_n = P_n;spec × f_prac × A_use;vzi    met f_prac = 1
/// ```
pub const NTA_8800_2025_FORMULE14_13: &str = "nta_8800_2025_formule14_13";

// ---------------------------------------------------------------------------
// Tabellen hoofdstuk 14
// ---------------------------------------------------------------------------

/// Tabel 14.1 — maximale brandduur per jaar overdag (t_D) en 's avonds/nachts
/// (t_N) per gebruiksfunctie.
pub const NTA_8800_2025_TABEL14_1: &str = "nta_8800_2025_tabel14_1";

/// Tabel 14.3 — specifiek geïnstalleerd vermogen `P_n;spec` in W/m² per
/// gebruiksfunctie.
///
/// - 16 W/m²: bijeenkomst / bijeenkomst-kinderopvang / kantoor / onderwijs /
///   sport / gezondheidszorg-anders-dan-met-bedgebied.
/// - 17 W/m²: cel / logies / gezondheidszorg-met-bedgebied.
/// - 30 W/m²: winkel.
pub const NTA_8800_2025_TABEL14_3: &str = "nta_8800_2025_tabel14_3";

/// Tabel 14.4 — onderhoudsfactor MF voor de nieuwwaarde-compensatie.
pub const NTA_8800_2025_TABEL14_4: &str = "nta_8800_2025_tabel14_4";

// ---------------------------------------------------------------------------
// Bijlagen
// ---------------------------------------------------------------------------

/// Bijlage Y — daglichttoetreding voor hellende ramen (bepaalt `F_D` voor
/// daglichtafhankelijke regelingen in dakvlakken).
///
/// Niet geïmplementeerd in V1 (alleen verticale ramen via user-supplied
/// scalar F_d). V2 scope.
pub const NTA_8800_2025_BIJLAGE_Y: &str = "nta_8800_2025_bijlage_y";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG14_1,
        NTA_8800_2025_PARAG14_2,
        NTA_8800_2025_PARAG14_2_1,
        NTA_8800_2025_PARAG14_2_2,
        NTA_8800_2025_PARAG14_3,
        NTA_8800_2025_PARAG14_3_4,
        NTA_8800_2025_PARAG14_4,
        NTA_8800_2025_PARAG14_5,
        NTA_8800_2025_PARAG14_6,
        NTA_8800_2025_FORMULE14_7,
        NTA_8800_2025_FORMULE14_13,
        NTA_8800_2025_TABEL14_1,
        NTA_8800_2025_TABEL14_3,
        NTA_8800_2025_TABEL14_4,
        NTA_8800_2025_BIJLAGE_Y,
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
