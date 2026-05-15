//! EP-score berekeningen volgens NTA 8800:2025 H.5.
//!
//! Implementeert de kernformules voor primair energiegebruik integratie,
//! specifieke EP-score bepaling en hernieuwbaar aandeel berekening.

use std::collections::HashMap;

use crate::{BuildingArea, EnergyCarrier, EpError, EpInputs};

use super::{co2_factor::co2_factor, primary_energy::primary_factor};

/// Berekent totaal primair energiegebruik [MJ] voor alle diensten.
///
/// Implementeert NTA 8800:2025 formule (5.2):
/// `E_P;tot = E_P;heating + E_P;cooling + E_P;dhw + E_P;lighting + E_P;vent + E_P;automation - E_P;pv`
///
/// Elke dienst wordt berekend volgens formule (5.1):
/// `E_P;dienst = Σ(Q_netto;dienst,drager × f_prim;drager)`
///
/// PV-opbrengst wordt afgetrokken als negatieve primaire energie.
///
/// # Errors
///
/// Retourneert [`EpError`] bij:
/// - Ontbrekende primaire energiefactoren
/// - Negatief energiegebruik per dienst
/// - Energiebalans validatiefouten
///
/// # Voorbeeld
///
/// ```
/// # use nta8800_ep::{EpInputs, EnergyCarrier, BuildingArea, calc::ep_score::total_primary_energy_mj};
/// # use std::collections::HashMap;
/// let mut heating = HashMap::new();
/// heating.insert(EnergyCarrier::Aardgas, 15000.0);
///
/// let inputs = EpInputs {
///     heating,
///     cooling: HashMap::new(),
///     dhw: HashMap::new(),
///     lighting: HashMap::new(),
///     ventilation_aux: HashMap::new(),
///     automation: HashMap::new(),
///     pv_yield: 5000.0,
///     building_area: BuildingArea { a_g: 150.0 },
/// };
///
/// let total_ep = total_primary_energy_mj(&inputs)?;
/// assert!(total_ep > 0.0); // 15 GJ gas - 5 GJ PV = positief saldo
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn total_primary_energy_mj(inputs: &EpInputs) -> Result<f64, EpError> {
    // Valideer inputs
    validate_energy_inputs(inputs)?;

    let mut total_ep = 0.0;

    // Verwarming — formule (5.1)
    total_ep += calculate_service_primary_energy(&inputs.heating, "heating")?;

    // Koeling
    total_ep += calculate_service_primary_energy(&inputs.cooling, "cooling")?;

    // Warmtapwater
    total_ep += calculate_service_primary_energy(&inputs.dhw, "dhw")?;

    // Verlichting
    total_ep += calculate_service_primary_energy(&inputs.lighting, "lighting")?;

    // Ventilatie hulpenergie
    total_ep += calculate_service_primary_energy(&inputs.ventilation_aux, "ventilation_aux")?;

    // Gebouwautomatisering
    total_ep += calculate_service_primary_energy(&inputs.automation, "automation")?;

    // PV-opbrengst als negatieve primaire energie (factor 0.000)
    // Per definitie: PV ter plaatse heeft f_prim = 0, dus geen aftrek van primair verbruik
    // Echter voor hernieuwbaar aandeel wordt PV wel meegeteld
    total_ep -= inputs.pv_yield * primary_factor(EnergyCarrier::HernieuwbareElektriciteit)?;

    Ok(total_ep)
}

/// Berekent specifiek primair energiegebruik [MJ/m²].
///
/// Implementeert NTA 8800:2025 formule (5.3):
/// `E_P;tot,spec = E_P;tot / A_g`
///
/// # Errors
///
/// Retourneert [`EpError::InvalidBuildingArea`] als A_g ≤ 0.
pub fn specific_primary_energy_mj_per_m2(
    inputs: &EpInputs,
    area: &BuildingArea,
) -> Result<f64, EpError> {
    if area.a_g <= 0.0 {
        return Err(EpError::InvalidBuildingArea { a_g: area.a_g });
    }

    let total_ep = total_primary_energy_mj(inputs)?;
    Ok(total_ep / area.a_g)
}

/// Berekent totale CO2-uitstoot [kg] voor alle diensten.
///
/// Implementeert NTA 8800:2025 formule (5.5):
/// `CO2_dienst = Σ(Q_netto;dienst,drager × f_CO2;drager)`
///
/// # Errors
///
/// Retourneert [`EpError`] bij ontbrekende CO2-beleidsfactoren.
pub fn total_co2_kg(inputs: &EpInputs) -> Result<f64, EpError> {
    let mut total_co2 = 0.0;

    total_co2 += calculate_service_co2(&inputs.heating, "heating")?;
    total_co2 += calculate_service_co2(&inputs.cooling, "cooling")?;
    total_co2 += calculate_service_co2(&inputs.dhw, "dhw")?;
    total_co2 += calculate_service_co2(&inputs.lighting, "lighting")?;
    total_co2 += calculate_service_co2(&inputs.ventilation_aux, "ventilation_aux")?;
    total_co2 += calculate_service_co2(&inputs.automation, "automation")?;

    // PV heeft geen CO2-uitstoot (factor 0.000)
    total_co2 -= inputs.pv_yield * co2_factor(EnergyCarrier::HernieuwbareElektriciteit)?;

    Ok(total_co2)
}

/// Berekent hernieuwbaar aandeel [0.0-1.0].
///
/// Implementeert NTA 8800:2025 formule (5.4):
/// `f_renewable = min(1.0, E_renewable / E_P;tot)`
///
/// Hernieuwbare energie omvat:
/// - PV-opbrengst ter plaatse
/// - Biomassa en pellets verbruik
///
/// **Opmerking:** Dit is een vereenvoudigde implementatie. In realiteit is
/// de berekening complexer door net-metering en temporele effecten.
///
/// # Errors
///
/// Retourneert [`EpError`] bij validatiefouten in de input data.
pub fn renewable_share(inputs: &EpInputs) -> Result<f64, EpError> {
    let total_energy = total_netto_energy_consumption(inputs);

    if total_energy <= 0.0 {
        return Ok(1.0); // Edge case: geen energiegebruik = 100% hernieuwbaar
    }

    let mut renewable_energy = inputs.pv_yield;

    // Hernieuwbare verbruikers
    for &carrier in &[EnergyCarrier::Biomassa, EnergyCarrier::Pellets] {
        renewable_energy += inputs.heating.get(&carrier).unwrap_or(&0.0);
        renewable_energy += inputs.cooling.get(&carrier).unwrap_or(&0.0);
        renewable_energy += inputs.dhw.get(&carrier).unwrap_or(&0.0);
        renewable_energy += inputs.lighting.get(&carrier).unwrap_or(&0.0);
        renewable_energy += inputs.ventilation_aux.get(&carrier).unwrap_or(&0.0);
        renewable_energy += inputs.automation.get(&carrier).unwrap_or(&0.0);
    }

    let fraction = renewable_energy / total_energy;
    Ok(fraction.clamp(0.0, 1.0))
}

// ---------------------------------------------------------------------------
// Helper functies
// ---------------------------------------------------------------------------

/// Valideert dat alle energie-inputs geldig zijn (≥ 0).
fn validate_energy_inputs(inputs: &EpInputs) -> Result<(), EpError> {
    let services = [
        ("heating", &inputs.heating),
        ("cooling", &inputs.cooling),
        ("dhw", &inputs.dhw),
        ("lighting", &inputs.lighting),
        ("ventilation_aux", &inputs.ventilation_aux),
        ("automation", &inputs.automation),
    ];

    for (service_name, service_map) in &services {
        for (carrier, &energy) in *service_map {
            if energy < 0.0 {
                return Err(EpError::NegativeEnergyUse {
                    service: format!("{service_name}_{carrier:?}"),
                    energy_mj: energy,
                });
            }
        }
    }

    // PV mag negatief zijn (netto-productie), maar check extreme waarden
    if inputs.pv_yield < -1_000_000.0 {
        return Err(EpError::EnergyBalanceError {
            message: format!("PV-opbrengst extreem negatief: {} MJ", inputs.pv_yield),
        });
    }

    Ok(())
}

/// Berekent primair energiegebruik voor één dienst.
fn calculate_service_primary_energy(
    service: &HashMap<EnergyCarrier, f64>,
    service_name: &str,
) -> Result<f64, EpError> {
    let mut total = 0.0;

    for (&carrier, &energy) in service {
        if energy < 0.0 {
            return Err(EpError::NegativeEnergyUse {
                service: service_name.to_string(),
                energy_mj: energy,
            });
        }

        let factor = primary_factor(carrier)?;
        total += energy * factor;
    }

    Ok(total)
}

/// Berekent CO2-uitstoot voor één dienst.
fn calculate_service_co2(
    service: &HashMap<EnergyCarrier, f64>,
    _service_name: &str,
) -> Result<f64, EpError> {
    let mut total = 0.0;

    for (&carrier, &energy) in service {
        let factor = co2_factor(carrier)?;
        total += energy * factor;
    }

    Ok(total)
}

/// Berekent totaal netto energieverbruik (alle diensten, voor hernieuwbaar aandeel).
fn total_netto_energy_consumption(inputs: &EpInputs) -> f64 {
    let mut total = 0.0;

    for service in [
        &inputs.heating,
        &inputs.cooling,
        &inputs.dhw,
        &inputs.lighting,
        &inputs.ventilation_aux,
        &inputs.automation,
    ] {
        for &energy in service.values() {
            total += energy;
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_inputs() -> EpInputs {
        let mut heating = HashMap::new();
        heating.insert(EnergyCarrier::Aardgas, 15000.0);

        let mut lighting = HashMap::new();
        lighting.insert(EnergyCarrier::Elektriciteit, 3000.0);

        EpInputs {
            heating,
            cooling: HashMap::new(),
            dhw: HashMap::new(),
            lighting,
            ventilation_aux: HashMap::new(),
            automation: HashMap::new(),
            pv_yield: 5000.0,
            building_area: BuildingArea { a_g: 150.0 },
        }
    }

    #[test]
    fn test_total_primary_energy() {
        let inputs = create_test_inputs();
        let total = total_primary_energy_mj(&inputs).unwrap();

        // Gas: 15000 * 1.000 = 15000 MJ
        // Elektriciteit: 3000 * 1.450 = 4350 MJ
        // PV: 5000 * 0.000 = 0 MJ afgetrokken
        // Totaal: 19350 MJ
        let expected = 15000.0_f64.mul_add(1.000, 3000.0 * 1.450) - 5000.0 * 0.000;
        assert_relative_eq!(total, expected, epsilon = 1e-9);
    }

    #[test]
    fn test_specific_primary_energy() {
        let inputs = create_test_inputs();
        let specific = specific_primary_energy_mj_per_m2(&inputs, &inputs.building_area).unwrap();

        let total = total_primary_energy_mj(&inputs).unwrap();
        let expected = total / 150.0;
        assert_relative_eq!(specific, expected, epsilon = 1e-9);
    }

    #[test]
    fn test_renewable_share() {
        let inputs = create_test_inputs();
        let share = renewable_share(&inputs).unwrap();

        // Total netto: 15000 + 3000 = 18000 MJ
        // Hernieuwbaar: 5000 MJ PV
        // Aandeel: 5000/18000 ≈ 0.278
        let expected = 5000.0 / 18000.0;
        assert!((share - expected).abs() < 0.001);
    }

    #[test]
    fn test_invalid_building_area() {
        let inputs = create_test_inputs();
        let area = BuildingArea { a_g: 0.0 };

        assert!(matches!(
            specific_primary_energy_mj_per_m2(&inputs, &area),
            Err(EpError::InvalidBuildingArea { a_g: 0.0 })
        ));
    }

    #[test]
    fn test_negative_energy_validation() {
        let mut heating = HashMap::new();
        heating.insert(EnergyCarrier::Aardgas, -1000.0);

        let inputs = EpInputs {
            heating,
            cooling: HashMap::new(),
            dhw: HashMap::new(),
            lighting: HashMap::new(),
            ventilation_aux: HashMap::new(),
            automation: HashMap::new(),
            pv_yield: 0.0,
            building_area: BuildingArea { a_g: 150.0 },
        };

        assert!(matches!(
            total_primary_energy_mj(&inputs),
            Err(EpError::NegativeEnergyUse { .. })
        ));
    }
}
