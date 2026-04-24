//! Norm-identifier constanten voor NTA 8800:2025+C1:2026 hoofdstuk 9
//! (verwarming) en de daarbij behorende bijlagen M, N, O, Q, R.
//!
//! Zie [`nta8800_model::references`](../../nta8800_model/references/index.html)
//! voor de naamgevings-conventie. Deze crate levert de sub-verzameling
//! constanten voor §9 — afgifte, distributie, opwekking, regeling.

// ---------------------------------------------------------------------------
// Paragrafen hoofdstuk 9
// ---------------------------------------------------------------------------

/// §9.1 — Principe verwarmingssysteem (afgifte + distributie + opwekking).
pub const NTA_8800_2025_PARAG9_1: &str = "nta_8800_2025_parag9_1";

/// §9.2 — Energiegebruik voor ruimteverwarming (NEN-EN 15316-1).
pub const NTA_8800_2025_PARAG9_2: &str = "nta_8800_2025_parag9_2";

/// §9.3 — Warmteafgiftesysteem (ΔT-correcties, tabel 9.2).
pub const NTA_8800_2025_PARAG9_3: &str = "nta_8800_2025_parag9_3";

/// §9.4 — Warmtedistributiesysteem (leiding-verliezen, pompenergie).
pub const NTA_8800_2025_PARAG9_4: &str = "nta_8800_2025_parag9_4";

/// §9.5 — Warmteopwekking (forfaitair opwekkingsrendement per type).
pub const NTA_8800_2025_PARAG9_5: &str = "nta_8800_2025_parag9_5";

/// §9.6 — Regeling van het verwarmingssysteem.
pub const NTA_8800_2025_PARAG9_6: &str = "nta_8800_2025_parag9_6";

// ---------------------------------------------------------------------------
// Formules §9 — keten-rendement
// ---------------------------------------------------------------------------

/// Formule (9.1) — energiegebruik opwekking per energiedrager (parallelle bedrijfswijze).
///
/// `E_H;gen;in;cr = Σ E_H;gen;in;cr · (E_Y;gen;out / Σ E_Y;gen;out)`
///
/// V1 vereenvoudigt dit tot één opwekker per rekenzone; de sommaties vervallen.
pub const NTA_8800_2025_FORMULE9_1: &str = "nta_8800_2025_formule9_1";

// ---------------------------------------------------------------------------
// Tabellen hoofdstuk 9
// ---------------------------------------------------------------------------

/// Tabel 9.2 — temperatuurverschil afgiftesysteem (Δθ_str + Δθ_emb + Δθ_rad + Δθ_im).
///
/// Geeft correcties in K per afgiftetype (radiator, vloerverwarming,
/// luchtverwarming, ventilator-gedreven) voor de bepaling van de werkelijke
/// aanvoertemperatuur. V1 gebruikt deze tabel NIET direct; het mapping naar
/// η_em is een V1-abstractie.
pub const NTA_8800_2025_TABEL9_2: &str = "nta_8800_2025_tabel9_2";

/// Tabel op pg 327 — forfaitair opwekkingsrendement η_gen voor
/// individuele cv-toestellen (water), exclusief waakvlam, HT-kolom.
///
/// HR-100: 0,90 · HR-104: 0,925 · HR-107: 0,95 — geplaatst binnen
/// thermische begrenzing, hoofdverwarming.
pub const NTA_8800_2025_TABEL_HR_INDIVIDUEEL: &str = "nta_8800_2025_tabel_hr_individueel_pg327";

// ---------------------------------------------------------------------------
// Bijlagen
// ---------------------------------------------------------------------------

/// Bijlage M — warmteopwekkers met verbranding (detail conform EN 15316-4-1).
///
/// V1 beperkt zich tot HR-klasse lookup; volledige bijlage M mapping is V2.
pub const NTA_8800_2025_BIJLAGE_M: &str = "nta_8800_2025_bijlage_m";

/// Bijlage N — warmteopwekkers inclusief kachels en pellets.
///
/// Niet geïmplementeerd in V1 (alleen HR-ketel, warmtepomp, elektrisch,
/// stadsverwarming).
pub const NTA_8800_2025_BIJLAGE_N: &str = "nta_8800_2025_bijlage_n";

/// Bijlage O — elektrisch hulpenergiegebruik CV (pompen, ventilatoren).
///
/// Niet geïmplementeerd in V1 (hulpenergie-berekening weggelaten).
pub const NTA_8800_2025_BIJLAGE_O: &str = "nta_8800_2025_bijlage_o";

/// Bijlage Q — warmtepompen (W/W, B/W, L/W, L/L type-specifieke correcties).
///
/// V1 gebruikt één generieke [`HeatPump`](crate::GenerationSystem::HeatPump)
/// met user-supplied SCOP; type-specifieke Q-correcties zijn V2.
pub const NTA_8800_2025_BIJLAGE_Q: &str = "nta_8800_2025_bijlage_q";

/// Bijlage R — biomassa-opwekkers (emissies en rendement).
///
/// Niet geïmplementeerd in V1 (biomassa-opwekker buiten scope).
pub const NTA_8800_2025_BIJLAGE_R: &str = "nta_8800_2025_bijlage_r";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG9_1,
        NTA_8800_2025_PARAG9_2,
        NTA_8800_2025_PARAG9_3,
        NTA_8800_2025_PARAG9_4,
        NTA_8800_2025_PARAG9_5,
        NTA_8800_2025_PARAG9_6,
        NTA_8800_2025_FORMULE9_1,
        NTA_8800_2025_TABEL9_2,
        NTA_8800_2025_TABEL_HR_INDIVIDUEEL,
        NTA_8800_2025_BIJLAGE_M,
        NTA_8800_2025_BIJLAGE_N,
        NTA_8800_2025_BIJLAGE_O,
        NTA_8800_2025_BIJLAGE_Q,
        NTA_8800_2025_BIJLAGE_R,
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
