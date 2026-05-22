//! Ontwerpbinnentemperatuur θ_i per gebruiksfunctie × ruimtetype.
//!
//! Bron: ISSO 53 (2016) tabel 2.2, PDF p.20.
//!
//! Tabel 2.2 groepeert de gebruiksfuncties in twee categorieën:
//! - kantoor / onderwijs / cel (en overige niet-zorg functies);
//! - gezondheidszorg.
//!
//! Daarnaast geldt "buiten de thermische schil" (niet-verwarmde
//! stallingsruimte, garage) → θ_e. Voor de voorontwerpfase (schilmethode,
//! §3.1, PDF p.27) geldt een eenvoudiger regel: 22 °C zorg, 20 °C overig.

use crate::model::enums::{GebruiksFunctie, RuimteType};

/// Ontwerpbinnentemperatuur die aangeeft dat de waarde gelijk is aan de
/// ontwerpbuitentemperatuur θ_e (ruimten buiten de thermische schil).
///
/// De caller moet deze marker vervangen door de actuele θ_e uit de
/// klimaatgegevens. We gebruiken een sentinel omdat tabel 2.2 zelf geen
/// numerieke θ_e bevat.
pub const TEMPERATURE_IS_EXTERIOR: f64 = f64::MIN;

/// Geeft `true` als de gebruiksfunctie een gezondheidszorgfunctie is.
/// ISSO 53 tabel 2.2 onderscheidt zorg van alle overige functies.
fn is_zorg(functie: GebruiksFunctie) -> bool {
    matches!(functie, GebruiksFunctie::Gezondheidszorg)
}

/// Ontwerpbinnentemperatuur θ_i in °C volgens ISSO 53 tabel 2.2 (PDF p.20).
///
/// Retourneert de **minimale** ontwerpbinnentemperatuur voor de combinatie
/// gebruiksfunctie × ruimtetype. Voor ruimten "buiten de thermische schil"
/// (niet-verwarmde stallingsruimte / garage) wordt [`TEMPERATURE_IS_EXTERIOR`]
/// teruggegeven — de caller vult dan θ_e in.
///
/// # Toelichting per ruimtetype
/// Tabel 2.2 geeft voor toilet-, verkeers-, technische-, berg- en onbenoemde
/// ruimten "X of berekening via warmtebalans". Hier wordt de forfaitaire
/// waarde X teruggegeven; de warmtebalans-variant (bijlage F) is buiten
/// scope van deze milestone.
pub fn design_indoor_temperature(functie: GebruiksFunctie, ruimte: RuimteType) -> f64 {
    let zorg = is_zorg(functie);
    match ruimte {
        // Verblijfsruimte / verblijfsgebied: 20 °C overig, 22 °C zorg.
        RuimteType::Verblijfsruimte
        | RuimteType::Verblijfsgebied
        | RuimteType::Kantoorruimte
        | RuimteType::Receptie
        | RuimteType::Lesruimte
        | RuimteType::Collegezaal
        | RuimteType::Werkplaats
        | RuimteType::Bureauruimte
        | RuimteType::Patientenkamer
        | RuimteType::Operatiekamer
        | RuimteType::Onderzoekruimte
        | RuimteType::Eetruimte
        | RuimteType::Restaurant
        | RuimteType::Kantine
        | RuimteType::Vergaderruimte
        | RuimteType::Hotelkamer
        | RuimteType::Sportzaal
        | RuimteType::Verkoopruimte
        | RuimteType::Supermarkt
        | RuimteType::Warenhuis => {
            if zorg {
                22.0
            } else {
                20.0
            }
        }
        // Badruimte: 22 °C overig, 24 °C zorg.
        RuimteType::Badruimte => {
            if zorg {
                24.0
            } else {
                22.0
            }
        }
        // Toilet- en verkeersruimte: 18 °C (of warmtebalans), zorg + overig.
        RuimteType::Toiletruimte | RuimteType::Verkeersruimte => 18.0,
        // Technische, onbenoemde en bergruimte: 10 °C (of warmtebalans).
        RuimteType::TechnischeRuimte
        | RuimteType::OnbenoemdeRuimte
        | RuimteType::Bergruimte => 10.0,
        // Stallingsruimte / bergruimte (zorg-context): forfaitair 5 °C.
        // ISSO 53 tabel 2.2 voetnoot 2 (PDF p.20): deze 5 °C geldt *alleen*
        // indien de ruimte vorstvrij gehouden moet worden i.v.m. aanwezige
        // waterleidingen. Het is een forfaitaire waarde, niet de enig
        // toegestane — voor verwarmde stallingsruimten zonder vorstrisico
        // moet de θ_i in overleg met de opdrachtgever worden vastgelegd.
        RuimteType::Stallingsruimte => 5.0,
        // Garage: buiten de thermische schil → θ_e.
        RuimteType::Garage => TEMPERATURE_IS_EXTERIOR,
    }
}

/// Ontwerpbinnentemperatuur θ_i voor de **voorontwerpfase** (schilmethode).
///
/// ISSO 53 §3.1 (PDF p.27): 22 °C voor gezondheidszorgfuncties, 20 °C voor
/// alle overige gebruiksfuncties.
pub fn design_indoor_temperature_shell(functie: GebruiksFunctie) -> f64 {
    if is_zorg(functie) {
        22.0
    } else {
        20.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kantoor_verblijfsruimte() {
        assert_eq!(
            design_indoor_temperature(GebruiksFunctie::Kantoor, RuimteType::Verblijfsruimte),
            20.0
        );
    }

    #[test]
    fn test_zorg_verblijfsruimte() {
        assert_eq!(
            design_indoor_temperature(
                GebruiksFunctie::Gezondheidszorg,
                RuimteType::Verblijfsruimte
            ),
            22.0
        );
    }

    #[test]
    fn test_badruimte_kantoor_vs_zorg() {
        assert_eq!(
            design_indoor_temperature(GebruiksFunctie::Onderwijs, RuimteType::Badruimte),
            22.0
        );
        assert_eq!(
            design_indoor_temperature(GebruiksFunctie::Gezondheidszorg, RuimteType::Badruimte),
            24.0
        );
    }

    #[test]
    fn test_verkeers_en_technisch() {
        assert_eq!(
            design_indoor_temperature(GebruiksFunctie::Cel, RuimteType::Verkeersruimte),
            18.0
        );
        assert_eq!(
            design_indoor_temperature(GebruiksFunctie::Kantoor, RuimteType::TechnischeRuimte),
            10.0
        );
    }

    #[test]
    fn test_stallingsruimte_zorg() {
        assert_eq!(
            design_indoor_temperature(
                GebruiksFunctie::Gezondheidszorg,
                RuimteType::Stallingsruimte
            ),
            5.0
        );
    }

    #[test]
    fn test_garage_is_exterior() {
        assert_eq!(
            design_indoor_temperature(GebruiksFunctie::Industrie, RuimteType::Garage),
            TEMPERATURE_IS_EXTERIOR
        );
    }

    #[test]
    fn test_shell_method() {
        assert_eq!(
            design_indoor_temperature_shell(GebruiksFunctie::Gezondheidszorg),
            22.0
        );
        assert_eq!(design_indoor_temperature_shell(GebruiksFunctie::Winkel), 20.0);
    }
}
