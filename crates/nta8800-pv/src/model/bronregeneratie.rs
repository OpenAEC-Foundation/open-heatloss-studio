//! Bronregeneratie-configuratie (NTA 8800 bijlage V).
//!
//! **V1-status: Stub implementatie.** Bronregeneratie van warmtepomp-bronnen
//! door PV-overschot en/of zonnethermisch is complex en vereist uitgebreide
//! modellering van bodem-thermiek en seizoens-opslag.
//!
//! V2 zal volledige implementatie bevatten conform bijlage V.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuratie voor bronregeneratie van warmtepomp-bronnen.
///
/// **V1: Placeholder-implementatie.** De types en velden zijn voorbereid
/// voor V2, maar de berekening-logica in [`crate::calc`] retourneert
/// een [`crate::errors::PvError::IncompleteBronregeneratieConfig`] fout.
///
/// Bronregeneratie behelst het actief regenereren van bodem- of aquifer-
/// warmte-bronnen door overtollige energie uit PV-installaties en/of
/// zonnethermische collectoren, conform NTA 8800 bijlage V.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BronregeneratieConfig {
    /// Type warmtepomp-bron dat geregenereerd wordt.
    pub bron_type: BronType,

    /// Regeneratie-methode(s) beschikbaar.
    pub regeneratie_methodes: Vec<RegeneratieMethode>,

    /// Maximaal regeneratie-vermogen in kW thermisch.
    ///
    /// Bepaald door de capaciteit van de regeneratie-installatie
    /// (PV-surplus omvormer, zonnethermische collectoren).
    pub max_regeneratie_vermogen_kw: f64,

    /// Effectiviteit van de regeneratie-koppeling (dimensieloos, 0-1).
    ///
    /// Hoe efficient overtollige PV-energie of zonnethermische warmte
    /// wordt omgezet naar bron-regeneratie. Inclusief transport-verliezen.
    pub regeneratie_effectiviteit: f64,

    /// Temperatuur-niveaus voor regeneratie-controle.
    pub temperatuur_controle: TemperatuurControle,
}

/// Type warmtepomp-bron voor regeneratie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum BronType {
    /// Gesloten bodembron-systeem (verticaal of horizontaal).
    BodemBron {
        /// Effectieve boor-diepte of lus-lengte in meters.
        effectieve_lengte_m: f64,
        /// Bodem-thermische geleidbaarheid in W/(m·K).
        thermische_geleidbaarheid: f64,
    },
    /// Open aquifer-systeem (grondwater).
    AquiferBron {
        /// Debiet in m³/h voor regeneratie.
        regeneratie_debiet_m3h: f64,
        /// Aquifer-capaciteit voor warmte-opslag.
        warmte_capaciteit_mj_k: f64,
    },
    /// Oppervlaktewater-bron (rivier, meer, kanaal).
    OppervlakteWaterBron {
        /// Regeneratie-capaciteit van het oppervlaktewater.
        water_capaciteit_mj_k: f64,
    },
}

/// Regeneratie-methode voor bron-opwarming.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum RegeneratieMethode {
    /// Regeneratie via PV-surplus (elektrische weerstand in bron).
    PvSurplus {
        /// Elektrische weerstand-vermogen in kW.
        weerstand_vermogen_kw: f64,
        /// COP van elektrische regeneratie (typisch ~1.0).
        elektrische_cop: f64,
    },
    /// Regeneratie via zonnethermische collectoren.
    ZonneThermisch {
        /// Collectoren oppervlak in m².
        collector_oppervlak_m2: f64,
        /// Collector-efficiëntie bij regeneratie-temperatuur.
        collector_efficientie: f64,
    },
    /// Hybride regeneratie (PV + zonnethermisch).
    Hybride {
        /// PV-component.
        pv_component: Box<RegeneratieMethode>,
        /// Zonnethermisch-component.
        zt_component: Box<RegeneratieMethode>,
        /// Schakel-logica tussen de methodes.
        schakel_strategie: SchakelStrategie,
    },
}

/// Strategie voor schakelen tussen regeneratie-methodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum SchakelStrategie {
    /// Voorrang aan PV-surplus, zonnethermisch als backup.
    PvVoorrang,
    /// Voorrang aan zonnethermisch, PV-surplus als backup.
    ZonneThermischVoorrang,
    /// Optimalisatie op basis van momentane efficiëntie.
    EfficiëntieOptimalisatie,
}

/// Temperatuur-controle parameters voor regeneratie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TemperatuurControle {
    /// Minimale bron-temperatuur voor WP-operatie in °C.
    ///
    /// Als de bron-temperatuur onder deze waarde zakt, wordt regeneratie
    /// geactiveerd (indien surplus beschikbaar).
    pub min_bron_temperatuur_c: f64,

    /// Maximale bron-temperatuur in °C.
    ///
    /// Regeneratie wordt gestopt als deze temperatuur wordt bereikt,
    /// om overopwarming te voorkomen.
    pub max_bron_temperatuur_c: f64,

    /// Target-temperatuur voor regeneratie in °C.
    ///
    /// Ideale bron-temperatuur voor optimale WP-efficiëntie.
    pub target_bron_temperatuur_c: f64,
}

impl BronregeneratieConfig {
    /// Controleert of de configuratie compleet is voor V2-berekeningen.
    ///
    /// V1 retourneert altijd een fout omdat bronregeneratie nog niet
    /// geïmplementeerd is. V2 zal deze methode gebruiken voor validatie.
    ///
    /// # Errors
    ///
    /// Retourneert altijd [`crate::errors::PvError::IncompleteBronregeneratieConfig`]
    /// in V1.
    pub fn validate(&self) -> Result<(), crate::errors::PvError> {
        use crate::errors::PvError;

        // V1: Altijd fout — implementatie volgt in V2
        Err(PvError::IncompleteBronregeneratieConfig {
            details: "Bronregeneratie (bijlage V) is nog niet geïmplementeerd. \
                     V1 ondersteunt alleen standaard PV-opbrengst berekening. \
                     V2 zal volledige bronregeneratie-modellering bevatten."
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bronregeneratie_config_validation_fails_in_v1() {
        let config = BronregeneratieConfig {
            bron_type: BronType::BodemBron {
                effectieve_lengte_m: 150.0,
                thermische_geleidbaarheid: 2.5,
            },
            regeneratie_methodes: vec![RegeneratieMethode::PvSurplus {
                weerstand_vermogen_kw: 5.0,
                elektrische_cop: 1.0,
            }],
            max_regeneratie_vermogen_kw: 5.0,
            regeneratie_effectiviteit: 0.85,
            temperatuur_controle: TemperatuurControle {
                min_bron_temperatuur_c: 8.0,
                max_bron_temperatuur_c: 25.0,
                target_bron_temperatuur_c: 15.0,
            },
        };

        let result = config.validate();
        assert!(result.is_err());

        // Check that it's the right error type
        if let Err(crate::errors::PvError::IncompleteBronregeneratieConfig { details }) = result {
            assert!(details.contains("V1 ondersteunt alleen"));
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn bron_types_serialize_correctly() {
        let bodem_bron = BronType::BodemBron {
            effectieve_lengte_m: 150.0,
            thermische_geleidbaarheid: 2.5,
        };

        let json = serde_json::to_string(&bodem_bron).unwrap();
        let deserialized: BronType = serde_json::from_str(&json).unwrap();
        assert_eq!(bodem_bron, deserialized);
    }

    #[test]
    fn regeneratie_methodes_serialize_correctly() {
        let pv_surplus = RegeneratieMethode::PvSurplus {
            weerstand_vermogen_kw: 5.0,
            elektrische_cop: 1.0,
        };

        let json = serde_json::to_string(&pv_surplus).unwrap();
        let deserialized: RegeneratieMethode = serde_json::from_str(&json).unwrap();
        assert_eq!(pv_surplus, deserialized);
    }
}
