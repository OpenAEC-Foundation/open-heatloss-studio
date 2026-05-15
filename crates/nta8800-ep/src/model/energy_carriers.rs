//! Energiedragers voor EP-score berekeningen.

use serde::{Deserialize, Serialize};

/// Energiedragers ondersteund in EP-score berekeningen.
///
/// Conform bijlage Z en AB van NTA 8800:2025. Elke energiedrager heeft
/// specifieke primaire energiefactoren en CO2-beleidsfactoren.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EnergyCarrier {
    /// Aardgas — fossiele energiedrager.
    ///
    /// Typisch voor verwarming en warmtapwater in woningen en utiliteit.
    /// Primaire energiefactor f_prim ≈ 1.05 (incl. transport/distributie).
    Aardgas,

    /// Elektriciteit uit het openbare net.
    ///
    /// Voor koeling, verlichting, ventilatie, pompen, BACS.
    /// Primaire energiefactor f_prim ≈ 2.5 (2023 Nederlandse energiemix).
    Elektriciteit,

    /// Stadswarmte (warmtenet).
    ///
    /// Collectieve warmtevoorziening, veelal restwarmte of biomassa.
    /// Primaire energiefactor f_prim varieert per leverancier (≈ 0.3-1.2).
    Stadswarmte,

    /// Biomassa — houtpellets, -snippers, andere biobrandstoffen.
    ///
    /// Hernieuwbare energiedrager met lage primaire energiefactor.
    /// f_prim ≈ 1.1 (incl. transport en bewerking).
    Biomassa,

    /// Hernieuwbare elektriciteit — PV, wind, etc.
    ///
    /// Elektriciteit opgewekt uit hernieuwbare bronnen ter plaatse.
    /// Primaire energiefactor f_prim = 1.0 (geen omzettingsverliezen).
    HernieuwbareElektriciteit,

    /// Houtpellets — biomassa-specifiek.
    ///
    /// Gestandaardiseerde biomassabrandstof voor automatische ketels.
    /// f_prim ≈ 1.05 (transport en bewerking minimaal).
    Pellets,
}

impl EnergyCarrier {
    /// Geeft een mensleesbare naam van de energiedrager.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Aardgas => "Aardgas",
            Self::Elektriciteit => "Elektriciteit",
            Self::Stadswarmte => "Stadswarmte",
            Self::Biomassa => "Biomassa",
            Self::HernieuwbareElektriciteit => "Hernieuwbare elektriciteit",
            Self::Pellets => "Houtpellets",
        }
    }

    /// Geeft alle beschikbare energiedragers.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Aardgas,
            Self::Elektriciteit,
            Self::Stadswarmte,
            Self::Biomassa,
            Self::HernieuwbareElektriciteit,
            Self::Pellets,
        ]
    }

    /// Controleert of de energiedrager hernieuwbaar is volgens NTA 8800.
    #[must_use]
    pub fn is_renewable(self) -> bool {
        matches!(
            self,
            Self::Biomassa | Self::HernieuwbareElektriciteit | Self::Pellets
        )
    }
}
