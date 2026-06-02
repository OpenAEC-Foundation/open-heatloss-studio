//! Heating-up configuration model for ISSO 53 (§4.8).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tables::heating_up::AirChanges;

/// Afkoel-regime tijdens de bedrijfsbeperking (§4.8.1 / §4.8.2).
///
/// Bepaalt welke tabel gebruikt wordt voor de specifieke toeslag φ_hu,i:
/// - [`CoolingRegime::Free`] → tabel 4.13 (vrije afkoeling), keuze op het
///   aantal úren verlaging {8, 14, 62};
/// - [`CoolingRegime::Limited`] → tabel 4.14 (beperkte afkoeling), keuze op
///   het aantal gráden verlaging {1..5}.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CoolingRegime {
    /// Vrije afkoeling — tabel 4.13. De installatie staat uit en de ruimte
    /// koelt vrij af; de toeslag hangt af van het aantal úren verlaging.
    Free {
        /// Aantal uren verlaging doordeweeks (kolom-keuze {8, 14}).
        /// 8 = tweeploegendienst (tabel 4.13 voetnoot 2); 14 = standaard.
        ///
        /// NB: `rename_all = "camelCase"` op enum-niveau hernoemt alléén de
        /// variant-tags (`Free` → "free"), niet de velden binnen de
        /// struct-varianten. De camelCase-veldnaam moet dus expliciet via
        /// `#[serde(rename = ...)]`, anders verwacht serde snake_case en faalt
        /// de mapper-payload op `missing field setbackHoursWeekday`.
        #[serde(rename = "setbackHoursWeekday")]
        setback_hours_weekday: u32,
        /// Aantal uren verlaging in het weekend (kolom-keuze, doorgaans 62).
        #[serde(rename = "setbackHoursWeekend")]
        setback_hours_weekend: u32,
    },
    /// Beperkte afkoeling — tabel 4.14. De temperatuur zakt slechts enkele
    /// graden; de toeslag hangt af van het aantal gráden verlaging {1..5}.
    Limited {
        /// Aantal graden verlaging doordeweeks {1..5}.
        #[serde(rename = "degreesWeekday")]
        degrees_weekday: u32,
        /// Aantal graden verlaging in het weekend {1..5}.
        #[serde(rename = "degreesWeekend")]
        degrees_weekend: u32,
    },
}

impl Default for CoolingRegime {
    fn default() -> Self {
        // Standaard: vrije afkoeling met 14 uur doordeweekse verlaging en
        // 62 uur (weekend) verlaging — het meest voorkomende kantoorregime.
        CoolingRegime::Free {
            setback_hours_weekday: 14,
            setback_hours_weekend: 62,
        }
    }
}

/// Aantal luchtwisselingen tijdens de afkoelperiode (serialiseerbaar).
/// §4.8 voetnoot: bij gesloten ramen/deuren + installatie uit → 0,1.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AirChangeRate {
    /// 0,1 luchtwisselingen (gesloten ramen/deuren, installatie uit).
    #[default]
    Low,
    /// 0,5 luchtwisselingen.
    High,
}

impl From<AirChangeRate> for AirChanges {
    fn from(rate: AirChangeRate) -> Self {
        match rate {
            AirChangeRate::Low => AirChanges::Low,
            AirChangeRate::High => AirChanges::High,
        }
    }
}

/// Configuration for heating-up supplement calculation (§4.8).
///
/// De specifieke toeslag φ_hu,i [W/m²] wordt automatisch opgezocht uit
/// tabel 4.13 (vrije afkoeling) of 4.14 (beperkte afkoeling) afhankelijk van
/// [`CoolingRegime`], met lineaire interpolatie over de opwarmtijd. Het
/// maatgevende resultaat is `max(doordeweekse verlaging, weekendverlaging)`.
///
/// `p_w_per_m2_override` blijft beschikbaar als **handmatige override**: als
/// gezet (`Some`) wordt die waarde direct gebruikt i.p.v. de tabel-lookup.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeatingUpConfig {
    /// Whether setback/heating-up supplement is active.
    pub setback_active: bool,

    /// Handmatige override voor de specifieke toeslag φ_hu,i [W/m²].
    /// Indien `Some(p)` → gebruik `p` direct (formule 4.43: Φ_op = A_vl · p),
    /// de automatische tabel-lookup wordt dan overgeslagen.
    /// Indien `None` → automatische §4.8-berekening (tabel 4.13/4.14).
    #[serde(default)]
    pub p_w_per_m2_override: Option<f64>,

    /// Afkoel-regime (vrije of beperkte afkoeling) → tabel 4.13 of 4.14.
    #[serde(default)]
    pub regime: CoolingRegime,

    /// Aantal luchtwisselingen tijdens de afkoelperiode (0,1 of 0,5).
    #[serde(default)]
    pub air_changes: AirChangeRate,

    /// Maximaal toegestane opwarmtijd doordeweeks [h] (rij-as tabel 4.13/4.14).
    #[serde(default = "default_warmup_weekday")]
    pub warmup_hours_weekday: f64,

    /// Maximaal toegestane opwarmtijd na het weekend [h]
    /// (tabel 4.13 voetnoot 3: bij voorkeur langere opwarmtijd na weekend).
    #[serde(default = "default_warmup_weekend")]
    pub warmup_hours_weekend: f64,

    /// `true` wanneer de mechanische toevoer van ventilatielucht tijdens de
    /// bedrijfsbeperking wordt uitgeschakeld. Dan geldt §4.8.3 formule 4.45
    /// met `a = 1`: Φ_hu,i = Φ_op − H_v · (θ_i − θ_e), geclamp op ≥ 0.
    /// Bij `false` (mechanische ventilatie blijft aan, of geen mechanische
    /// toevoer aanwezig) is `a = 0` → Φ_hu,i = Φ_op.
    #[serde(default)]
    pub mechanical_supply_off: bool,
}

fn default_warmup_weekday() -> f64 {
    2.0
}

fn default_warmup_weekend() -> f64 {
    4.0
}

impl Default for HeatingUpConfig {
    fn default() -> Self {
        Self {
            setback_active: false,
            p_w_per_m2_override: None,
            regime: CoolingRegime::default(),
            air_changes: AirChangeRate::default(),
            warmup_hours_weekday: default_warmup_weekday(),
            warmup_hours_weekend: default_warmup_weekend(),
            mechanical_supply_off: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regressie: de frontend-mapper (`isso53ProjectMapper.ts`) stuurt de
    /// regime-velden in camelCase (`setbackHoursWeekday`). Voor de §4.8-fix was
    /// `rename_all="camelCase"` op enum-niveau onvoldoende — dat hernoemt alleen
    /// de variant-tags, niet de struct-velden — waardoor deserialisatie faalde
    /// met `missing field setback_hours_weekday`. Deze test borgt dat de
    /// camelCase-vorm permanent landt op `CoolingRegime::Free`.
    #[test]
    fn cooling_regime_free_deserializes_from_camel_case() {
        let json = r#"{"type":"free","setbackHoursWeekday":14,"setbackHoursWeekend":62}"#;
        let regime: CoolingRegime =
            serde_json::from_str(json).expect("free regime camelCase deserialize");
        assert_eq!(
            regime,
            CoolingRegime::Free {
                setback_hours_weekday: 14,
                setback_hours_weekend: 62,
            }
        );
    }

    /// Idem voor de beperkte-afkoeling-variant (`degreesWeekday`/`degreesWeekend`).
    #[test]
    fn cooling_regime_limited_deserializes_from_camel_case() {
        let json = r#"{"type":"limited","degreesWeekday":3,"degreesWeekend":5}"#;
        let regime: CoolingRegime =
            serde_json::from_str(json).expect("limited regime camelCase deserialize");
        assert_eq!(
            regime,
            CoolingRegime::Limited {
                degrees_weekday: 3,
                degrees_weekend: 5,
            }
        );
    }

    /// De serialisatie moet exact de shape opleveren die de mapper verstuurt
    /// (variant-tag in `type`, velden in camelCase) — borgt de roundtrip.
    #[test]
    fn cooling_regime_serializes_to_camel_case() {
        let regime = CoolingRegime::Free {
            setback_hours_weekday: 8,
            setback_hours_weekend: 62,
        };
        let json = serde_json::to_value(regime).expect("serialize");
        assert_eq!(json["type"], "free");
        assert_eq!(json["setbackHoursWeekday"], 8);
        assert_eq!(json["setbackHoursWeekend"], 62);

        let regime = CoolingRegime::Limited {
            degrees_weekday: 2,
            degrees_weekend: 4,
        };
        let json = serde_json::to_value(regime).expect("serialize");
        assert_eq!(json["type"], "limited");
        assert_eq!(json["degreesWeekday"], 2);
        assert_eq!(json["degreesWeekend"], 4);
    }

    /// Deserializeert exact de `heatingUp`-blokvorm die `isso53ProjectMapper.ts`
    /// (regels 270-278) bouwt: volledige camelCase, met geneste `regime`-enum.
    /// Dit is de payload die elke "Berekenen"-klik genereert en die vóór de fix
    /// crashte op `missing field setback_hours_weekday`.
    #[test]
    fn heating_up_config_deserializes_from_mapper_shape() {
        let json = r#"{
            "setbackActive": true,
            "pWPerM2Override": null,
            "regime": {
                "type": "free",
                "setbackHoursWeekday": 14,
                "setbackHoursWeekend": 62
            },
            "airChanges": "low",
            "warmupHoursWeekday": 2.0,
            "warmupHoursWeekend": 4.0,
            "mechanicalSupplyOff": false
        }"#;
        let config: HeatingUpConfig =
            serde_json::from_str(json).expect("HeatingUpConfig mapper-shape deserialize");
        assert!(config.setback_active);
        assert_eq!(config.p_w_per_m2_override, None);
        assert_eq!(
            config.regime,
            CoolingRegime::Free {
                setback_hours_weekday: 14,
                setback_hours_weekend: 62,
            }
        );
        assert_eq!(config.air_changes, AirChangeRate::Low);
        assert_eq!(config.warmup_hours_weekday, 2.0);
        assert_eq!(config.warmup_hours_weekend, 4.0);
        assert!(!config.mechanical_supply_off);
    }

    /// Idem maar met de beperkte-afkoeling-variant en een override-waarde.
    #[test]
    fn heating_up_config_deserializes_limited_with_override() {
        let json = r#"{
            "setbackActive": true,
            "pWPerM2Override": 12.5,
            "regime": {
                "type": "limited",
                "degreesWeekday": 3,
                "degreesWeekend": 5
            },
            "airChanges": "high",
            "warmupHoursWeekday": 1.5,
            "warmupHoursWeekend": 3.0,
            "mechanicalSupplyOff": true
        }"#;
        let config: HeatingUpConfig =
            serde_json::from_str(json).expect("HeatingUpConfig limited mapper-shape deserialize");
        assert_eq!(config.p_w_per_m2_override, Some(12.5));
        assert_eq!(
            config.regime,
            CoolingRegime::Limited {
                degrees_weekday: 3,
                degrees_weekend: 5,
            }
        );
        assert_eq!(config.air_changes, AirChangeRate::High);
        assert!(config.mechanical_supply_off);
    }
}
