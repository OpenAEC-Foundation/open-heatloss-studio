//! Gestandaardiseerde formule-identifiers voor ISSO 51:2023.
//!
//! Elke constante verwijst naar een specifieke formule, tabel, figuur of
//! paragraaf uit de norm. De naamconventie is:
//!
//! | Type      | Patroon                         | Voorbeeld                        |
//! |-----------|---------------------------------|----------------------------------|
//! | Formule   | `ISSO_51_2023_FORMULE{nr}`      | `ISSO_51_2023_FORMULE4_3A`       |
//! | Tabel     | `ISSO_51_2023_TABEL{nr}`        | `ISSO_51_2023_TABEL2_12`         |
//! | Figuur    | `ISSO_51_2023_FIGUUR{nr}`       | `ISSO_51_2023_FIGUUR4_2`         |
//! | Paragraaf | `ISSO_51_2023_PARAG{nr}`        | `ISSO_51_2023_PARAG4_3`          |
//! | Erratum   | suffix `_ERRATUM`               | `ISSO_51_2023_FORMULE4_1_ERRATUM`|

// ---------------------------------------------------------------------------
// Formules
// ---------------------------------------------------------------------------

/// Phi_T totaal transmissieverlies.
/// Formule (4.2): Phi_T = (H_T,ie + H_T,ia + H_T,io + H_T,ib + H_T,ig) x (theta_i - theta_e)
pub const ISSO_51_2023_FORMULE4_2: &str = "ISSO_51_2023_formule4_2";

/// H_T,ie naar buiten.
/// Formule (4.3a): H_T,ie = Sigma(A_k x f_k x (U_k + DeltaU_TB))
pub const ISSO_51_2023_FORMULE4_3A: &str = "ISSO_51_2023_formule4_3a";

/// H_T,ia naar aangrenzende ruimten.
/// Formule (4.6): H_T,ia = Sigma(A_k x U_k x f_ia,k)
pub const ISSO_51_2023_FORMULE4_6: &str = "ISSO_51_2023_formule4_6";

/// H_T,io naar onverwarmde ruimten.
/// Formule (4.10): H_T,io = Sigma(A_k x U_k x f_k)
pub const ISSO_51_2023_FORMULE4_10: &str = "ISSO_51_2023_formule4_10";

/// H_T,ib naar aangrenzend gebouw.
/// Formule (4.14): H_T,ib = c_z x Sigma(A_k x U_k x f_b)
pub const ISSO_51_2023_FORMULE4_14: &str = "ISSO_51_2023_formule4_14";

/// f_b wand naar aangrenzend gebouw.
/// Formule (4.15): f_b = (theta_i - theta_b) / (theta_i - theta_e)
pub const ISSO_51_2023_FORMULE4_15: &str = "ISSO_51_2023_formule4_15";

/// f_b vloer naar aangrenzend gebouw.
/// Formule (4.16): f_b = (theta_i + Delta_2 - theta_b) / (theta_i - theta_e)
pub const ISSO_51_2023_FORMULE4_16: &str = "ISSO_51_2023_formule4_16";

/// f_b plafond naar aangrenzend gebouw.
/// Formule (4.17): f_b = (theta_i + Delta_1 - theta_b) / (theta_i - theta_e)
pub const ISSO_51_2023_FORMULE4_17: &str = "ISSO_51_2023_formule4_17";

/// H_T,ig naar de grond.
/// Formule (4.18): H_T,ig = 1.45 x G_w x Sigma(A_k x f_g2 x U_e,k)
pub const ISSO_51_2023_FORMULE4_18: &str = "ISSO_51_2023_formule4_18";

/// f_v temperatuurcorrectiefactor buitenlucht (erratum).
/// Formule (4.6a erratum): f_v = ((theta_i + Delta_v) - theta_t) / (theta_i - theta_e)
pub const ISSO_51_2023_FORMULE4_6A_ERRATUM: &str = "ISSO_51_2023_formule4_6a_erratum";

/// f_v temperatuurcorrectiefactor aangrenzende ruimte (erratum).
/// Formule (4.6b erratum): f_v = ((theta_i + Delta_v) - theta_a) / (theta_i - theta_e)
pub const ISSO_51_2023_FORMULE4_6B_ERRATUM: &str = "ISSO_51_2023_formule4_6b_erratum";

/// H_v ventilatieverlies (erratum).
/// Formule (4.3 erratum): H_v = 1.2 x q_v x f_v
pub const ISSO_51_2023_FORMULE4_3_ERRATUM: &str = "ISSO_51_2023_formule4_3_erratum";

/// H_v gemengde luchttoevoer (erratum).
/// Formule (4.7 erratum): H_v = 1.2 x ((a x q_v x f_v1) + (1-a) x q_v x f_v2)
pub const ISSO_51_2023_FORMULE4_7_ERRATUM: &str = "ISSO_51_2023_formule4_7_erratum";

/// Phi_i infiltratieverlies (erratum).
/// Formule (4.1 erratum): Phi_i = z_i x H_i x (theta_i - theta_e)
pub const ISSO_51_2023_FORMULE4_1_ERRATUM: &str = "ISSO_51_2023_formule4_1_erratum";

/// H_i infiltratie op gebouwniveau (erratum).
/// Formule (E.5 erratum): H_i = 1.2 x q_i,spec x z x Sigma(A_g)
pub const ISSO_51_2023_FORMULE_E5_ERRATUM: &str = "ISSO_51_2023_formule_e5_erratum";

/// Kwadratische sommatie niet-gelijktijdige verliezen (erratum).
/// Formule (3.11 erratum): Phi_extra = sqrt(Phi_vent^2 + Phi_T,iaBE^2 + Phi_hu^2)
pub const ISSO_51_2023_FORMULE3_11_ERRATUM: &str = "ISSO_51_2023_formule3_11_erratum";

/// Phi_vent = Phi_v - Phi_i (erratum).
/// Formule (3.3 erratum): netto ventilatieverlies voor kwadratische som.
pub const ISSO_51_2023_FORMULE3_3_ERRATUM: &str = "ISSO_51_2023_formule3_3_erratum";

// ---------------------------------------------------------------------------
// Tabellen
// ---------------------------------------------------------------------------

/// Forfaitaire thermische brugwaarden.
/// Tabel 2.8: DeltaU_TB per materiaaltype.
pub const ISSO_51_2023_TABEL2_8: &str = "ISSO_51_2023_tabel2_8";

/// Specifieke opwarmtoeslag P [W/m²] (opwarmtoeslag-tabel).
/// Tabel 2.10: P als functie van afkoeling (1 K / 2 K), gebouwzwaarte
/// (ZL+L+M / Z) en toegestane opwarmtijd. Gebruikt in `Φ_hu = P × A_g`
/// (zie `calc/heating_up.rs` + §4.3 Formule 4.15).
///
/// LET OP: dit is NIET de c_z-veiligheidsfactortabel. De c_z-factoren voor
/// transmissie naar aangrenzende woningen (Formule 4.14) staan in **Tabel
/// 2.16** (zie `calc/transmission.rs`), niet in Tabel 2.10. Een eerdere
/// doc-comment labelde deze constante abusievelijk als "Veiligheidsfactoren
/// c_z" — gecorrigeerd.
pub const ISSO_51_2023_TABEL2_10: &str = "ISSO_51_2023_tabel2_10";

/// Temperatuurcorrecties Delta_1, Delta_2, Delta_v (erratum).
/// Tabel 2.12 (erratum): correcties per afgiftesysteem.
pub const ISSO_51_2023_TABEL2_12_ERRATUM: &str = "ISSO_51_2023_tabel2_12_erratum";

/// Systeemverliezen vloerverwarming.
/// Tabel 2.17: fractie f_vvw per R_c-klasse.
pub const ISSO_51_2023_TABEL2_17: &str = "ISSO_51_2023_tabel2_17";

/// Systeemverliezen wandverwarming (erratum).
/// Tabel 2.18 (erratum): fractie f_wvw per R_c-klasse.
pub const ISSO_51_2023_TABEL2_18_ERRATUM: &str = "ISSO_51_2023_tabel2_18_erratum";

/// Specifiek infiltratiedebiet q_i,spec.
/// Tabel 4.3: q_i,spec per qv10-klasse.
pub const ISSO_51_2023_TABEL4_3: &str = "ISSO_51_2023_tabel4_3";

/// Nachtkoeling en opwarmfactoren.
/// Tabel 4.6: Delta_t en f_RH per gebouwtype.
pub const ISSO_51_2023_TABEL4_6: &str = "ISSO_51_2023_tabel4_6";

// ---------------------------------------------------------------------------
// Figuren
// ---------------------------------------------------------------------------

/// Equivalente grondweerstand U_e.
/// Figuur 4.2: U_e als functie van B' en R_f.
pub const ISSO_51_2023_FIGUUR4_2: &str = "ISSO_51_2023_figuur4_2";

// ---------------------------------------------------------------------------
// Paragrafen
// ---------------------------------------------------------------------------

/// Forfaitaire thermische bruggen.
/// Paragraaf 2.5.1: DeltaU_TB bepaling.
pub const ISSO_51_2023_PARAG2_5_1: &str = "ISSO_51_2023_parag2_5_1";

/// Systeemverliezen ingebouwde verwarming (erratum).
/// Paragraaf 2.9.1 (erratum): plafondverwarming.
pub const ISSO_51_2023_PARAG2_9_1_ERRATUM: &str = "ISSO_51_2023_parag2_9_1_erratum";

/// Opwarmtoeslag.
/// Paragraaf 4.3: berekening Phi_hu.
pub const ISSO_51_2023_PARAG4_3: &str = "ISSO_51_2023_parag4_3";

/// Totaal warmteverlies.
/// Paragraaf 4.5.3: Phi_HL,i = Phi_basis + Phi_extra.
pub const ISSO_51_2023_PARAG4_5_3: &str = "ISSO_51_2023_parag4_5_3";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Alle constanten moeten het prefix "ISSO_51_2023_" hebben.
    #[test]
    fn test_all_constants_have_prefix() {
        let all: &[&str] = &[
            // Formules
            ISSO_51_2023_FORMULE4_2,
            ISSO_51_2023_FORMULE4_3A,
            ISSO_51_2023_FORMULE4_6,
            ISSO_51_2023_FORMULE4_10,
            ISSO_51_2023_FORMULE4_14,
            ISSO_51_2023_FORMULE4_15,
            ISSO_51_2023_FORMULE4_16,
            ISSO_51_2023_FORMULE4_17,
            ISSO_51_2023_FORMULE4_18,
            ISSO_51_2023_FORMULE4_6A_ERRATUM,
            ISSO_51_2023_FORMULE4_6B_ERRATUM,
            ISSO_51_2023_FORMULE4_3_ERRATUM,
            ISSO_51_2023_FORMULE4_7_ERRATUM,
            ISSO_51_2023_FORMULE4_1_ERRATUM,
            ISSO_51_2023_FORMULE_E5_ERRATUM,
            ISSO_51_2023_FORMULE3_11_ERRATUM,
            ISSO_51_2023_FORMULE3_3_ERRATUM,
            // Tabellen
            ISSO_51_2023_TABEL2_8,
            ISSO_51_2023_TABEL2_10,
            ISSO_51_2023_TABEL2_12_ERRATUM,
            ISSO_51_2023_TABEL2_17,
            ISSO_51_2023_TABEL2_18_ERRATUM,
            ISSO_51_2023_TABEL4_3,
            ISSO_51_2023_TABEL4_6,
            // Figuren
            ISSO_51_2023_FIGUUR4_2,
            // Paragrafen
            ISSO_51_2023_PARAG2_5_1,
            ISSO_51_2023_PARAG2_9_1_ERRATUM,
            ISSO_51_2023_PARAG4_3,
            ISSO_51_2023_PARAG4_5_3,
        ];

        assert_eq!(all.len(), 29, "Verwacht 29 constanten");

        for id in all {
            assert!(
                id.starts_with("ISSO_51_2023_"),
                "Constante {id:?} mist prefix \"ISSO_51_2023_\""
            );
        }
    }
}
