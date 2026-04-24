//! Indelingsregels voor rekenzones conform NTA 8800:2025+C1:2026 §6.5.2.
//!
//! De norm stelt voorwaarden aan het samenvoegen van gebouwdelen in één
//! rekenzone. De hoofdregels zijn:
//!
//! - **§6.5.2**: In een rekenzone met woonfunctie mogen geen andere
//!   gebruiksfuncties voorkomen.
//! - **§6.5.2**: Alleen delen met vergelijkbare specifieke interne
//!   warmtecapaciteit mogen gecombineerd worden (anders splitsen, zie ook
//!   §6.5 over bouwtype-verschillen).
//! - **§6.4/§6.5**: Een rekenzone hoort binnen één klimatiseringszone
//!   (één type klimatiseringssysteem, één tapwaterzone-context).
//!
//! De onderstaande `ZoneGroupingRule`-enum geeft deze bindingsregels
//! declaratief weer; `rules_for_usage_function` retourneert welke regels
//! van toepassing zijn voor een gegeven primaire gebruiksfunctie.

use nta8800_model::zoning::UsageFunction;

use crate::references::{NTA_8800_2025_PARAG6_4, NTA_8800_2025_PARAG6_5, NTA_8800_2025_PARAG6_5_2};

/// Normatieve voorwaarde waaraan een rekenzone-indeling moet voldoen.
///
/// Elke variant verwijst in de doc-comment naar de NTA 8800 paragraaf die de
/// regel voorschrijft. `rules_for_usage_function` levert de set regels die
/// voor een specifieke primaire gebruiksfunctie bindend zijn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ZoneGroupingRule {
    /// §6.5.2 — In een rekenzone met woonfunctie mogen geen andere
    /// gebruiksfuncties voorkomen (behalve hetgeen bij fictie als onderdeel
    /// van de woonfunctie mag worden beschouwd).
    ///
    /// Normverwijzing: [`NTA_8800_2025_PARAG6_5_2`].
    WoonfunctieGeenAndereGebruiksfuncties,

    /// §6.5.2 — Delen met sterk uiteenlopende specifieke interne
    /// warmtecapaciteit (bv. zware vs. lichte bouwwijze) moeten in
    /// afzonderlijke rekenzones. Factor ≈ 3 als leidende drempel.
    ///
    /// Normverwijzing: [`NTA_8800_2025_PARAG6_5_2`].
    HomogeneWarmteCapaciteit,

    /// §6.5 — Alle EFR's binnen één rekenzone moeten dezelfde
    /// binnenklimaat-eisen (gebruiksprofiel / temperatuur-setpoint) delen,
    /// óf, indien zij verschillen, moet de rekenzone gesplitst worden zodat
    /// elke zone een éénduidig binnenklimaat heeft.
    ///
    /// Normverwijzing: [`NTA_8800_2025_PARAG6_5`].
    EfrHebbenCompatibelBinnenklimaat,

    /// §6.4 + §6.5 — Een rekenzone valt binnen precies één klimatiseringszone
    /// (één type klimatiseringssysteem: verwarming / koeling / ventilatie /
    /// bevochtiging).
    ///
    /// Normverwijzing: [`NTA_8800_2025_PARAG6_4`].
    EenKlimatiseringssysteemPerRekenzone,
}

impl ZoneGroupingRule {
    /// Norm-referentie-constante voor deze regel (zie [`crate::references`]).
    #[must_use]
    pub const fn norm_reference(self) -> &'static str {
        match self {
            Self::WoonfunctieGeenAndereGebruiksfuncties | Self::HomogeneWarmteCapaciteit => {
                NTA_8800_2025_PARAG6_5_2
            }
            Self::EfrHebbenCompatibelBinnenklimaat => NTA_8800_2025_PARAG6_5,
            Self::EenKlimatiseringssysteemPerRekenzone => NTA_8800_2025_PARAG6_4,
        }
    }
}

/// Regels die voor *elke* rekenzone van toepassing zijn, ongeacht
/// gebruiksfunctie.
const COMMON_RULES: &[ZoneGroupingRule] = &[
    ZoneGroupingRule::HomogeneWarmteCapaciteit,
    ZoneGroupingRule::EfrHebbenCompatibelBinnenklimaat,
    ZoneGroupingRule::EenKlimatiseringssysteemPerRekenzone,
];

/// Regels die extra gelden wanneer een rekenzone een woonfunctie bevat.
const WOON_RULES: &[ZoneGroupingRule] = &[
    ZoneGroupingRule::WoonfunctieGeenAndereGebruiksfuncties,
    ZoneGroupingRule::HomogeneWarmteCapaciteit,
    ZoneGroupingRule::EfrHebbenCompatibelBinnenklimaat,
    ZoneGroupingRule::EenKlimatiseringssysteemPerRekenzone,
];

/// Retourneer de set regels die bindend zijn voor een rekenzone met de
/// opgegeven primaire gebruiksfunctie.
///
/// Voor [`UsageFunction::Woonfunctie`] komen er strenge regels bij (een
/// woonfunctie mag niet vermengd worden met andere gebruiksfuncties in
/// dezelfde rekenzone).
#[must_use]
pub const fn rules_for_usage_function(u: UsageFunction) -> &'static [ZoneGroupingRule] {
    match u {
        UsageFunction::Woonfunctie => WOON_RULES,
        _ => COMMON_RULES,
    }
}

/// Test of twee gebruiksfuncties in dezelfde rekenzone mogen liggen volgens
/// §6.5.2.
///
/// Basisregel: twee *identieke* gebruiksfuncties zijn altijd compatibel.
/// Is één van beide een woonfunctie, dan mag alleen een andere woonfunctie
/// ernaast. Niet-woon utiliteitsfuncties onderling: toegestaan (mits de
/// andere regels — klimaat, warmtecapaciteit — ook compatibel zijn; dat
/// valideert [`super::validate_building`] separaat).
#[must_use]
pub fn are_usage_functions_compatible(a: UsageFunction, b: UsageFunction) -> bool {
    if a == b {
        return true;
    }
    // Woonfunctie is exclusief — mag alleen met zichzelf.
    if matches!(a, UsageFunction::Woonfunctie) || matches!(b, UsageFunction::Woonfunctie) {
        return false;
    }
    // Overige utiliteitsfuncties: toegestaan op dit niveau. Andere regels
    // (klimaat/warmtecapaciteit) filteren verder.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn woonfunctie_krijgt_strengere_regels() {
        let woon = rules_for_usage_function(UsageFunction::Woonfunctie);
        let kantoor = rules_for_usage_function(UsageFunction::Kantoorfunctie);
        assert!(woon.len() > kantoor.len());
        assert!(woon.contains(&ZoneGroupingRule::WoonfunctieGeenAndereGebruiksfuncties));
        assert!(!kantoor.contains(&ZoneGroupingRule::WoonfunctieGeenAndereGebruiksfuncties));
    }

    #[test]
    fn utiliteits_regels_bevatten_common_rules() {
        for uf in [
            UsageFunction::Kantoorfunctie,
            UsageFunction::Onderwijsfunctie,
            UsageFunction::Sportfunctie,
            UsageFunction::Winkelfunctie,
        ] {
            let regels = rules_for_usage_function(uf);
            assert!(regels.contains(&ZoneGroupingRule::HomogeneWarmteCapaciteit));
            assert!(regels.contains(&ZoneGroupingRule::EfrHebbenCompatibelBinnenklimaat));
            assert!(regels.contains(&ZoneGroupingRule::EenKlimatiseringssysteemPerRekenzone));
        }
    }

    #[test]
    fn norm_references_zijn_geldige_constanten() {
        for rule in [
            ZoneGroupingRule::WoonfunctieGeenAndereGebruiksfuncties,
            ZoneGroupingRule::HomogeneWarmteCapaciteit,
            ZoneGroupingRule::EfrHebbenCompatibelBinnenklimaat,
            ZoneGroupingRule::EenKlimatiseringssysteemPerRekenzone,
        ] {
            let refstr = rule.norm_reference();
            assert!(refstr.starts_with("nta_8800_2025_parag6"));
        }
    }

    #[test]
    fn woon_compatibel_met_zichzelf_niet_met_utiliteit() {
        assert!(are_usage_functions_compatible(
            UsageFunction::Woonfunctie,
            UsageFunction::Woonfunctie,
        ));
        assert!(!are_usage_functions_compatible(
            UsageFunction::Woonfunctie,
            UsageFunction::Kantoorfunctie,
        ));
        assert!(!are_usage_functions_compatible(
            UsageFunction::Industriefunctie,
            UsageFunction::Woonfunctie,
        ));
    }

    #[test]
    fn utiliteits_functies_onderling_compatibel_op_usage_niveau() {
        assert!(are_usage_functions_compatible(
            UsageFunction::Kantoorfunctie,
            UsageFunction::Bijeenkomstfunctie,
        ));
        assert!(are_usage_functions_compatible(
            UsageFunction::Onderwijsfunctie,
            UsageFunction::Sportfunctie,
        ));
    }
}
