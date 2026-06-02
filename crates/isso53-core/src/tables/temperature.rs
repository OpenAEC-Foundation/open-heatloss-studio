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
use crate::model::Room;

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

/// Bepaal de effectieve ontwerpbinnentemperatuur θ_i voor een ruimte.
///
/// Respecteert in volgorde:
/// 1. `room.custom_temperature` indien gezet;
/// 2. de tabel 2.2-waarde [`design_indoor_temperature`];
/// 3. vervangt de sentinel [`TEMPERATURE_IS_EXTERIOR`] (ruimten buiten de
///    thermische schil, bv. [`RuimteType::Garage`]) door de meegegeven `theta_e`.
///
/// Hierdoor kan de sentinel `f64::MIN` nooit in een verlies-berekening
/// lekken: voor een garage zonder `custom_temperature` wordt θ_i = θ_e,
/// zodat Φ_T over de garage-schil eindig (en typisch ~0) is.
pub fn resolve_theta_i(room: &Room, theta_e: f64) -> f64 {
    if let Some(custom) = room.custom_temperature {
        return custom;
    }
    let theta_i = design_indoor_temperature(room.gebruiks_functie, room.ruimte_type);
    if theta_i == TEMPERATURE_IS_EXTERIOR {
        theta_e
    } else {
        theta_i
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
    fn test_resolve_theta_i_garage_uses_theta_e() {
        // D1-regressie: garage zonder custom_temperature → sentinel
        // f64::MIN mag NIET lekken; resolve_theta_i moet θ_e teruggeven.
        use crate::calc::transmission::calculate_transmission;
        use crate::model::*;

        let garage = Room {
            id: "garage1".to_string(),
            name: "Garage".to_string(),
            gebruiks_functie: GebruiksFunctie::Industrie,
            ruimte_type: RuimteType::Garage,
            floor_area: 30.0,
            height: 3.0,
            custom_temperature: None,
            constructions: vec![ConstructionElement {
                id: "wall1".to_string(),
                description: "Garage exterior wall".to_string(),
                area: 20.0,
                u_value: 1.5,
                boundary_type: BoundaryType::Exterior,
                material_type: MaterialType::Masonry,
                temperature_factor: None,
                adjacent_room_id: None,
                adjacent_temperature: None,
                vertical_position: VerticalPosition::Wall,
                use_forfaitaire_thermal_bridge: false,
                custom_delta_u_tb: None,
                ground_params: None,
                has_embedded_heating: false,
                unheated_space: None,
            }],
            bezetting: Bezetting {
                personen: None,
                personen_per_m2_default: None,
            },
            infiltration_reduction_z: 1.0,
            has_mechanical_supply: None,
            ventilation_q_v_established: None,
        };

        let building = Building {
            building_shape: BuildingShape::Meerlaags,
            construction_year: 2020,
            building_position: GebouwTypePositie::MeerlaagsTussen,
            ventilation_system: VentilationSystemType::SystemB,
            thermal_mass: ThermalMass::Gemiddeld,
            wind_pressure_type: crate::model::enums::GebouwTypeWinddruk::MeerlaagsStandaard,
            building_height: None,
            building_length: None,
            building_width: None,
            heating_system: Default::default(),
            source_zone_config: Default::default(),
        };

        let climate = DesignConditions::default();

        // resolve_theta_i moet exact θ_e zijn (sentinel vervangen).
        let theta_i = resolve_theta_i(&garage, climate.theta_e);
        assert_eq!(theta_i, climate.theta_e, "garage θ_i moet θ_e zijn");

        // En de transmissie-berekening mag geen astronomisch verlies geven.
        let rooms = vec![garage.clone()];
        let result =
            calculate_transmission(&garage, &rooms, &building, &climate).unwrap();
        assert!(
            result.phi_t.is_finite(),
            "Φ_T over garage-schil moet eindig zijn, kreeg {}",
            result.phi_t
        );
        // θ_i = θ_e → Φ_T precies 0 (geen temperatuurverschil over de schil).
        assert!(
            result.phi_t.abs() < 1e-6,
            "Φ_T moet ~0 zijn bij θ_i = θ_e, kreeg {}",
            result.phi_t
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
