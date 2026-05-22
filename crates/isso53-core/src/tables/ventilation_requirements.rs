//! Minimale ventilatie-eisen volgens het Bouwbesluit.
//!
//! Bron: ISSO 53 (2016) tabel 4.10, PDF p.48-50.
//!
//! Het Bouwbesluit drukt de ventilatie-eis uit als luchtvolumestroom **per
//! persoon** (dm³/s·pp). Het aantal personen per ruimte volgt uit de
//! bezetting (zie [`crate::tables::occupancy`], tabel 4.11). De eis verschilt
//! tussen nieuwbouw en bestaande bouw.
//!
//! Afvoereisen (PDF p.50): toiletten ≥ 7 dm³/s, badkamers/douches ≥ 14 dm³/s
//! per douche. Keukens (max 50 m²): 3 dm³/s per m². Grootkeukenventilatie valt
//! buiten ISSO 53 (zie ISSO-publicatie 112).

use crate::model::enums::{GebruiksFunctie, RuimteType, VentilatieBouwfase};

/// Eén regel uit ventilatie-eisentabel 4.10.
/// ISSO 53 (PDF p.48-50).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VentilationRequirement {
    /// Beschrijving van de ruimte zoals in tabel 4.10.
    pub description: &'static str,
    /// Nieuwbouw-eis in dm³/s per persoon. `None` = geen eis ("-").
    pub nieuwbouw_dm3_s_pp: Option<f64>,
    /// Richtwaarde bezetting in personen/m². `None` = n.v.t.
    pub personen_per_m2: Option<f64>,
    /// Eis bestaande bouw in dm³/s per persoon. `None` = geen eis ("-").
    pub bestaand_dm3_s_pp: Option<f64>,
}

/// Tabel 4.10 — minimaal vereiste ventilatie-luchtvolumestromen.
/// ISSO 53 (PDF p.48-50).
///
/// Voor `personen_per_m2` met meerdere waarden in de PDF (bv. "0,125/0,3" of
/// "0,125/0,05") is de eerste/standaardwaarde overgenomen; de afwijkende
/// waarde (sport-/bezoekerscontext) staat in een `// PDF:`-comment.
pub const VENTILATION_REQUIREMENTS_4_10: &[VentilationRequirement] = &[
    // --- Bijeenkomstfunctie ---
    VentilationRequirement {
        description: "Eetruimte",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Bar",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Bedrijfsrestaurant",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Kantine",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        // PDF: pers/m² = 0,125/0,3 — 0,3 bij sport-toeschouwers.
        description: "Toeschouwersruimte",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Bibliotheek (bijeenkomst)",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Museum",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Bioscoop",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Concertzaal",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Schouwburg/theater",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Casino",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Vergaderruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(3.44),
    },
    // --- Kantoorfunctie ---
    VentilationRequirement {
        description: "Kantoorruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Receptie",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(3.44),
    },
    // --- Celfunctie ---
    VentilationRequirement {
        description: "Cel niet voor dag- en nachtverblijf",
        nieuwbouw_dm3_s_pp: Some(12.0),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(6.4),
    },
    VentilationRequirement {
        // PDF: pers/m² = 0,05 — 0,125 bij bezoekersruimte.
        description: "Cel voor dag- en nachtverblijf",
        nieuwbouw_dm3_s_pp: Some(12.0),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(6.4),
    },
    VentilationRequirement {
        // PDF: pers/m² = 0,125/0,05.
        description: "Andere ruimte (cel)",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    // --- Gezondheidszorgfunctie ---
    VentilationRequirement {
        description: "Patiëntenkamer",
        nieuwbouw_dm3_s_pp: Some(12.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Ontwaakkamer",
        nieuwbouw_dm3_s_pp: Some(12.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Intensive care",
        nieuwbouw_dm3_s_pp: Some(12.0),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Operatiekamer",
        nieuwbouw_dm3_s_pp: Some(12.0),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Onderzoekruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Fysiotherapie",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Sectieruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    // --- Logiesfunctie ---
    VentilationRequirement {
        description: "Hotelkamer",
        nieuwbouw_dm3_s_pp: Some(12.0),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(6.4),
    },
    // --- Onderwijsfunctie ---
    VentilationRequirement {
        description: "Lesruimte",
        nieuwbouw_dm3_s_pp: Some(8.5),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Collegezaal",
        nieuwbouw_dm3_s_pp: Some(8.5),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Werkplaats",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Bureauruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.05),
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Gymzaal",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Aula",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: Some(0.125),
        bestaand_dm3_s_pp: Some(3.44),
    },
    // --- Sportfunctie ---
    VentilationRequirement {
        description: "Sportzaal",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Bowlingruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "IJsvloerspeelruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Zwembad",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    // --- Industriefunctie ---
    VentilationRequirement {
        description: "Industrie algemeen",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Verfspuitinrichting",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    VentilationRequirement {
        description: "Accuruimte",
        nieuwbouw_dm3_s_pp: Some(6.5),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(3.44),
    },
    // --- Winkelfunctie ---
    VentilationRequirement {
        description: "Apotheek",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Beautyshop",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Bibliotheek (winkel)",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Bloemist",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Kapper",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Postkantoor",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Supermarkt",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Warenhuis",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Slagerij",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Verkoopruimte",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
    VentilationRequirement {
        description: "Wasserette",
        nieuwbouw_dm3_s_pp: Some(4.0),
        personen_per_m2: None,
        bestaand_dm3_s_pp: Some(2.12),
    },
];

/// Minimale afvoereis voor een toiletruimte in dm³/s.
/// ISSO 53 tabel 4.10, toelichting (PDF p.50).
pub const TOILET_EXTRACT_MIN_DM3_S: f64 = 7.0;

/// Minimale afvoereis voor een badkamer/douche in dm³/s per douche.
/// ISSO 53 tabel 4.10, toelichting (PDF p.50).
pub const SHOWER_EXTRACT_MIN_DM3_S: f64 = 14.0;

/// Ventilatie-eis voor keukens (max 50 m²) in dm³/s per m².
/// ISSO 53 tabel 4.10 (PDF p.50). Grootkeukens vallen buiten ISSO 53.
pub const KITCHEN_VENTILATION_DM3_S_M2: f64 = 3.0;

/// Zoekt een ventilatie-eis op via de letterlijke ruimte-omschrijving.
/// ISSO 53 tabel 4.10 (PDF p.48-50).
pub fn requirement_by_description(description: &str) -> Option<&'static VentilationRequirement> {
    VENTILATION_REQUIREMENTS_4_10
        .iter()
        .find(|r| r.description.eq_ignore_ascii_case(description))
}

/// Mapt een (gebruiksfunctie, ruimtetype)-combinatie naar de tabel 4.10-regel.
/// ISSO 53 tabel 4.10 (PDF p.48-50).
///
/// Voor ruimtetypen die niet één-op-één in tabel 4.10 voorkomen wordt de
/// dichtstbijzijnde gebruiksfunctie-categorie gekozen. Retourneert `None`
/// voor combinaties zonder eis (bv. berg-/technische ruimten).
pub fn requirement(
    functie: GebruiksFunctie,
    ruimte: RuimteType,
) -> Option<&'static VentilationRequirement> {
    let description = match (functie, ruimte) {
        // Kantoor
        (GebruiksFunctie::Kantoor, RuimteType::Kantoorruimte)
        | (GebruiksFunctie::Kantoor, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Kantoor, RuimteType::Verblijfsgebied) => "Kantoorruimte",
        (GebruiksFunctie::Kantoor, RuimteType::Receptie) => "Receptie",
        (_, RuimteType::Vergaderruimte) => "Vergaderruimte",
        // Onderwijs
        (GebruiksFunctie::Onderwijs, RuimteType::Lesruimte)
        | (GebruiksFunctie::Onderwijs, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Onderwijs, RuimteType::Verblijfsgebied) => "Lesruimte",
        (GebruiksFunctie::Onderwijs, RuimteType::Collegezaal) => "Collegezaal",
        (GebruiksFunctie::Onderwijs, RuimteType::Werkplaats) => "Werkplaats",
        (GebruiksFunctie::Onderwijs, RuimteType::Bureauruimte) => "Bureauruimte",
        // Gezondheidszorg
        (GebruiksFunctie::Gezondheidszorg, RuimteType::Patientenkamer)
        | (GebruiksFunctie::Gezondheidszorg, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Gezondheidszorg, RuimteType::Verblijfsgebied) => "Patiëntenkamer",
        (GebruiksFunctie::Gezondheidszorg, RuimteType::Operatiekamer) => "Operatiekamer",
        (GebruiksFunctie::Gezondheidszorg, RuimteType::Onderzoekruimte) => "Onderzoekruimte",
        // Bijeenkomst
        (GebruiksFunctie::Bijeenkomst, RuimteType::Eetruimte) => "Eetruimte",
        (GebruiksFunctie::Bijeenkomst, RuimteType::Restaurant) => "Bedrijfsrestaurant",
        (GebruiksFunctie::Bijeenkomst, RuimteType::Kantine) => "Kantine",
        // Logies
        (GebruiksFunctie::Logies, RuimteType::Hotelkamer)
        | (GebruiksFunctie::Logies, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Logies, RuimteType::Verblijfsgebied) => "Hotelkamer",
        // Sport
        (GebruiksFunctie::Sport, RuimteType::Sportzaal)
        | (GebruiksFunctie::Sport, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Sport, RuimteType::Verblijfsgebied) => "Sportzaal",
        // Winkel
        (GebruiksFunctie::Winkel, RuimteType::Verkoopruimte)
        | (GebruiksFunctie::Winkel, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Winkel, RuimteType::Verblijfsgebied) => "Verkoopruimte",
        (GebruiksFunctie::Winkel, RuimteType::Supermarkt) => "Supermarkt",
        (GebruiksFunctie::Winkel, RuimteType::Warenhuis) => "Warenhuis",
        // Cel
        (GebruiksFunctie::Cel, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Cel, RuimteType::Verblijfsgebied) => {
            "Cel voor dag- en nachtverblijf"
        }
        // Industrie
        (GebruiksFunctie::Industrie, RuimteType::Werkplaats)
        | (GebruiksFunctie::Industrie, RuimteType::Verblijfsruimte)
        | (GebruiksFunctie::Industrie, RuimteType::Verblijfsgebied) => "Industrie algemeen",
        // Geen ventilatie-eis (berg-/technische/verkeers-/sanitaire ruimten).
        _ => return None,
    };
    requirement_by_description(description)
}

/// Ventilatie-eis in dm³/s per persoon voor de gevraagde bouwfase.
/// ISSO 53 tabel 4.10 (PDF p.48-50).
pub fn ventilation_rate_per_person(
    requirement: &VentilationRequirement,
    fase: VentilatieBouwfase,
) -> Option<f64> {
    match fase {
        VentilatieBouwfase::Nieuwbouw => requirement.nieuwbouw_dm3_s_pp,
        VentilatieBouwfase::Bestaand => requirement.bestaand_dm3_s_pp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kantoorruimte_requirement() {
        let r = requirement_by_description("Kantoorruimte").unwrap();
        assert_eq!(r.nieuwbouw_dm3_s_pp, Some(6.5));
        assert_eq!(r.personen_per_m2, Some(0.05));
        assert_eq!(r.bestaand_dm3_s_pp, Some(3.44));
    }

    #[test]
    fn test_lesruimte_requirement() {
        let r = requirement_by_description("Lesruimte").unwrap();
        assert_eq!(r.nieuwbouw_dm3_s_pp, Some(8.5));
        assert_eq!(r.personen_per_m2, Some(0.125));
        assert_eq!(r.bestaand_dm3_s_pp, Some(3.44));
    }

    #[test]
    fn test_patientenkamer_requirement() {
        let r = requirement_by_description("Patiëntenkamer").unwrap();
        assert_eq!(r.nieuwbouw_dm3_s_pp, Some(12.0));
        assert_eq!(r.personen_per_m2, Some(0.125));
        assert_eq!(r.bestaand_dm3_s_pp, Some(3.44));
    }

    #[test]
    fn test_lookup_by_functie_ruimte() {
        let r = requirement(GebruiksFunctie::Kantoor, RuimteType::Kantoorruimte).unwrap();
        assert_eq!(r.nieuwbouw_dm3_s_pp, Some(6.5));

        let r = requirement(GebruiksFunctie::Onderwijs, RuimteType::Lesruimte).unwrap();
        assert_eq!(r.nieuwbouw_dm3_s_pp, Some(8.5));
    }

    #[test]
    fn test_no_requirement_for_storage() {
        assert!(requirement(GebruiksFunctie::Kantoor, RuimteType::Bergruimte).is_none());
        assert!(requirement(GebruiksFunctie::Kantoor, RuimteType::TechnischeRuimte).is_none());
    }

    #[test]
    fn test_rate_per_person_per_phase() {
        let r = requirement_by_description("Kantoorruimte").unwrap();
        assert_eq!(
            ventilation_rate_per_person(r, VentilatieBouwfase::Nieuwbouw),
            Some(6.5)
        );
        assert_eq!(
            ventilation_rate_per_person(r, VentilatieBouwfase::Bestaand),
            Some(3.44)
        );
    }

    #[test]
    fn test_extract_constants() {
        assert_eq!(TOILET_EXTRACT_MIN_DM3_S, 7.0);
        assert_eq!(SHOWER_EXTRACT_MIN_DM3_S, 14.0);
        assert_eq!(KITCHEN_VENTILATION_DM3_S_M2, 3.0);
    }
}
