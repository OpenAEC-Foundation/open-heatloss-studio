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

    /// PV-opbrengst [MJ] (hernieuwbare elektriciteitsproductie ter plaatse).
    ///
    /// Telt op twee plaatsen mee: als **vermeden primair-fossiele energie** in
    /// BENG 2 (`fP;exp;el = 1,45`, tabel 5.2) en als **hernieuwbare primaire
    /// elektriciteit** in BENG 3 (`fPren;renelect = 1,45`, tabel 5.4). De volledige
    /// productie telt (zelfgebruik én export). Negatieve waarden zijn niet zinvol.
    pub pv_yield: f64,

    /// Omgevingswarmte [MJ] die als hernieuwbaar telt: de bronzijdige warmtestroom
    /// van warmtepompen (verwarming + tapwater, `QH;hp;in`/`QW;ren;hp;in`) en
    /// thermische zonne-energie.
    ///
    /// NTA 8800:2025+C1:2026 §5.6.2.1/§5.6.2.3 (formules 5.31/5.36) — telt in BENG 3
    /// mee met `fPren;renheat = 1,0` (tabel 5.4). Voor een warmtepomp is dit de
    /// omgevingswarmte `Q_use × (SCOP − 1)`. Levert **geen** bijdrage aan BENG 2
    /// (omgevingswarmte is geen afgenomen fossiele energie).
    ///
    /// `serde(default)` = 0,0 zodat bestaande [`EpInputs`]-payloads geldig blijven.
    #[serde(default)]
    pub renewable_ambient_heat_mj: f64,

    /// Omgevingskoude [MJ] die als hernieuwbaar telt: koude uit systemen met
    /// `EER ≥ 8` (vrije koeling, WKO, bodemkoeling), `QC;gen;out`.
    ///
    /// NTA 8800:2025+C1:2026 §5.6.2.2 (formule 5.34) — telt in BENG 3 mee met
    /// `fPren;rencold = 1,0` (tabel 5.4). Wordt gevuld door de koel-keten (F3b);
    /// `serde(default)` = 0,0.
    #[serde(default)]
    pub renewable_ambient_cold_mj: f64,

    /// Gebouwgeometrie voor specifiek energiegebruik [MJ/m²].
    pub building_area: BuildingArea,
}
