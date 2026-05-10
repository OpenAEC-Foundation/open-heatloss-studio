//! Datamodel voor EP-score berekeningen.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod energy_carriers;

pub use energy_carriers::EnergyCarrier;

/// Gebouwgeometrie voor EP-score berekeningen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingArea {
    /// Gebruiksoppervlakte A_g [m²].
    ///
    /// Conform NTA 8800 definitie: netto vloeroppervlakte van verwarmde en
    /// gekoelde ruimtes, gemeten tot binnenzijde draagconstructies.
    pub a_g: f64,
}

/// Input-gegevens voor EP-score berekening.
///
/// Verzamelt het netto energiegebruik per dienst per energiedrager,
/// zoals berekend door de individuele NTA 8800 modules (heating, cooling, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpInputs {
    /// Netto energiegebruik verwarming [MJ] per energiedrager.
    pub heating: HashMap<EnergyCarrier, f64>,

    /// Netto energiegebruik koeling [MJ] per energiedrager.
    pub cooling: HashMap<EnergyCarrier, f64>,

    /// Netto energiegebruik warmtapwater [MJ] per energiedrager.
    pub dhw: HashMap<EnergyCarrier, f64>,

    /// Netto energiegebruik verlichting [MJ] per energiedrager.
    pub lighting: HashMap<EnergyCarrier, f64>,

    /// Netto energiegebruik ventilatie (hulpenergie) [MJ] per energiedrager.
    pub ventilation_aux: HashMap<EnergyCarrier, f64>,

    /// Netto energiegebruik gebouwautomatisering [MJ] per energiedrager.
    ///
    /// **Opmerking:** In V1 is dit typisch alleen elektriciteit voor
    /// BACS-systemen (sensoren, actuators, regelapparatuur).
    pub automation: HashMap<EnergyCarrier, f64>,

    /// PV-opbrengst [MJ] (hernieuwbare energieproductie ter plaatse).
    ///
    /// Wordt gebruikt voor hernieuwbaar aandeel berekening volgens H.5.
    /// Negatieve waarden betekenen netto energie-overschot.
    pub pv_yield: f64,

    /// Gebouwgeometrie voor specifiek energiegebruik [MJ/m²].
    pub building_area: BuildingArea,
}
