//! Rekenkundige kern: absolute vochtigheid, bevochtiging, ontvochtiging.
//!
//! Alle publieke reken-entry's nemen input in °C en g/kg, en produceren
//! energie in MJ conform workspace-conventie.

pub mod absolute_humidity;
pub mod humidification;
pub mod dehumidification;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Temperature;
use nta8800_model::{ClimateData, Rekenzone};
use nta8800_tables::climate::de_bilt::DE_BILT_MONTH_LENGTHS_HOURS;

use crate::errors::HumidityError;
use crate::model::HumiditySystemConfig;
use crate::result::HumidityResult;
// Referenties gebruikt in doc-comments

/// Verdampingswarmte water bij 0°C in kJ/kg — NTA 8800 formule (12.1).
pub const WATER_LATENT_HEAT_KJ_PER_KG: f64 = 2501.0;

/// Luchtdichtheid ρ_a in kg/m³ bij 20°C — standaard voor volumestroom conversie.
pub const AIR_DENSITY_KG_PER_M3: f64 = 1.205;

/// Atmosferische druk in Pa — standaard voor dampdruk berekeningen.
pub const ATMOSPHERIC_PRESSURE_PA: f64 = 101_325.0;

/// Bereken humidity energiegebruik voor één [`Rekenzone`] conform
/// NTA 8800:2025+C1:2026 H.12.
///
/// # Algoritme
///
/// Voor elke maand:
/// 1. Bepaal absolute vochtigheid buitenlucht x_ODA (formule 12.3)
/// 2. Bepaal gewenste absolute vochtigheid binnenlucht x_IDA via targets
/// 3. Bereken bevochtigingsbehoefte Q_hum indien x_IDA > x_ODA (formule 12.1)
/// 4. Bereken ontvochtigingsbehoefte Q_dhum indien x_ODA > x_IDA (formule 12.2)
/// 5. Converteer naar elektrisch energiegebruik via systeem-rendementen
///
/// # Eenheden
///
/// | Grootheid | Input | Output |
/// |---|---|---|
/// | Temperaturen | °C | — |
/// | Vochtigheid | g/kg | — |
/// | Q_hum/Q_dhum (thermisch) | — | MJ |
/// | W_hum (elektrisch) | — | MJ |
///
/// # Referenties
///
/// - Formule (12.1): `Q_hum = ṁ_a · (x_IDA - x_ODA) · r_w`
/// - Formule (12.2): `Q_dhum = ṁ_a · (x_ODA - x_IDA) · r_w`
/// - Formule (12.3): absolute vochtigheid uit RH en temperatuur
///
/// # Errors
///
/// - [`HumidityError::InvalidSteamEfficiency`] bij rendement buiten [0,1]
/// - [`HumidityError::InvalidDehumidificationCop`] bij COP ≤ 0
/// - [`HumidityError::InvalidHumidityRange`] bij min > max in targets
/// - [`HumidityError::InvalidZoneVolume`] bij negatief volume
pub fn calculate_humidity(
    zone: &Rekenzone,
    system_config: &HumiditySystemConfig,
    indoor_temperature: &MonthlyProfile<Temperature>,
    climate: &ClimateData,
) -> Result<HumidityResult, HumidityError> {
    // Input validatie
    if zone.volume <= 0.0 {
        return Err(HumidityError::InvalidZoneVolume {
            volume: zone.volume,
        });
    }

    if system_config.target.min_g_per_kg >= system_config.target.max_g_per_kg {
        return Err(HumidityError::InvalidHumidityRange {
            min: system_config.target.min_g_per_kg,
            max: system_config.target.max_g_per_kg,
        });
    }

    // Systeem validatie
    if let Some(ref hum_sys) = system_config.humidification {
        let efficiency = hum_sys.efficiency();
        if !(0.0..=1.0).contains(&efficiency) {
            return Err(HumidityError::InvalidSteamEfficiency(efficiency));
        }
    }

    if let Some(ref dhum_sys) = system_config.dehumidification {
        let cop = dhum_sys.cop();
        if cop <= 0.0 {
            return Err(HumidityError::InvalidDehumidificationCop(cop));
        }
    }

    // Geschatte luchtvolumestroom uit zone volume (10 air changes/hour als heuristiek)
    let air_flow_m3_h = zone.volume * 10.0;
    let mass_flow_kg_h = air_flow_m3_h * AIR_DENSITY_KG_PER_M3;

    let mut monthly_humidification = [0.0_f64; 12];
    let mut monthly_dehumidification = [0.0_f64; 12];
    let mut monthly_electrical = [0.0_f64; 12];

    for month in Month::all() {
        let theta_e = climate.outdoor_temperature[month];
        let theta_i = indoor_temperature[month];
        // V1 caveat: relatieve vochtigheid geschat op basis van temperatuur
        // Winter (< 10°C): 85%, lente/herfst (10-20°C): 75%, zomer (>20°C): 70%
        let rh_e = if theta_e < 10.0 {
            0.85 // Winter: hogere relatieve vochtigheid
        } else if theta_e < 20.0 {
            0.75 // Lente/herfst: gemiddelde vochtigheid
        } else {
            0.70 // Zomer: lagere relatieve vochtigheid
        };
        let t_mi = DE_BILT_MONTH_LENGTHS_HOURS[month];

        // Valideer temperaturen
        if !(-40.0..=60.0).contains(&theta_e) {
            return Err(HumidityError::InvalidTemperatureRange { temp: theta_e });
        }
        if !(-40.0..=60.0).contains(&theta_i) {
            return Err(HumidityError::InvalidTemperatureRange { temp: theta_i });
        }

        // Absolute vochtigheid buitenlucht
        let x_oda = absolute_humidity::calculate_absolute_humidity(theta_e, rh_e)?;

        // Gewenste binnenlucht vochtigheid - neem gemiddelde van range als target
        let x_ida_target = f64::midpoint(system_config.target.min_g_per_kg, system_config.target.max_g_per_kg);

        // Seizoenslogica: winter (< 15°C) prioriteit bevochtiging, zomer (≥ 15°C) prioriteit ontvochtiging
        let (thermal_humidify, thermal_dehumidify, electrical_energy) = if theta_e < 15.0 {
            // Winter: focus op bevochtiging als buitenlucht te droog
            if x_oda < x_ida_target {
                let delta_x = x_ida_target - x_oda;
                let q_hum = humidification::calculate_humidification_energy(
                    mass_flow_kg_h,
                    delta_x / 1000.0, // Convert g/kg to kg/kg
                    t_mi,
                );

                let w_hum = if let Some(ref hum_sys) = system_config.humidification {
                    q_hum / hum_sys.efficiency()
                } else {
                    0.0
                };

                (q_hum, 0.0, w_hum)
            } else {
                (0.0, 0.0, 0.0)
            }
        } else {
            // Zomer: focus op ontvochtiging als buitenlucht te vochtig
            if x_oda > x_ida_target {
                let delta_x = x_oda - x_ida_target;
                let q_dhum = dehumidification::calculate_dehumidification_energy(
                    mass_flow_kg_h,
                    delta_x / 1000.0, // Convert g/kg to kg/kg
                    t_mi,
                );

                let w_dhum = if let Some(ref dhum_sys) = system_config.dehumidification {
                    q_dhum / dhum_sys.cop()
                } else {
                    0.0
                };

                (0.0, q_dhum, w_dhum)
            } else {
                (0.0, 0.0, 0.0)
            }
        };

        monthly_humidification[month.index()] = thermal_humidify;
        monthly_dehumidification[month.index()] = thermal_dehumidify;
        monthly_electrical[month.index()] = electrical_energy;
    }

    let humidification_profile = MonthlyProfile::new(monthly_humidification);
    let dehumidification_profile = MonthlyProfile::new(monthly_dehumidification);
    let electrical_profile = MonthlyProfile::new(monthly_electrical);

    Ok(HumidityResult::new(
        humidification_profile,
        dehumidification_profile,
        electrical_profile,
    ))
}