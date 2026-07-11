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
///     renewable_ambient_heat_mj: 0.0,
///     renewable_ambient_cold_mj: 0.0,
///     building_area: BuildingArea { a_g: 150.0 },
/// };
///
/// let total_ep = total_primary_energy_mj(&inputs)?;
/// assert!(total_ep > 0.0); // 15 GJ gas × 1,0 − 5 GJ PV × 1,45 = positief saldo
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

    // PV-saldering (§5.5.2, formules 5.10/5.13). Op eigen perceel opgewekte PV
    // vermijdt primair-fossiele energie tegen fP;exp;el = fP;pr;us;el = 1,45 (tabel
    // 5.2). Omdat afname, zelfgebruik én export dezelfde factor 1,45 hebben, is de
    // netto-aftrek exact PV × 1,45 — ongeacht de zelfconsumptie/export-verdeling
    // (zie docs/2026-07-11-f3a-norm-analyse-ep.md §2). EPTot mag negatief worden
    // bij een groot PV-overschot (§5.5.2 opmerking 11) — geen clamp.
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

    // PV vermijdt net-CO2 tegen KCO2;exp;el = KCO2;del;el (tabel 5.3, §5.5.6.1):
    // zelf-gebruikte/geëxporteerde PV verlaagt de CO2-indicator, analoog aan de
    // primaire-energie-saldering.
    total_co2 -= inputs.pv_yield * co2_factor(EnergyCarrier::HernieuwbareElektriciteit)?;

    Ok(total_co2)
}

/// Primaire hernieuwbare energiefactor omgevingswarmte `fPren;renheat`
/// (NTA 8800:2025+C1:2026 tabel 5.4, p. 109).
const F_PREN_RENHEAT: f64 = 1.0;
/// Primaire hernieuwbare energiefactor omgevingskoude `fPren;rencold` (tabel 5.4).
const F_PREN_RENCOLD: f64 = 1.0;
/// Primaire hernieuwbare energiefactor lokaal opgewekte elektriciteit
/// `fPren;renelect` (tabel 5.4).
const F_PREN_RENELECT: f64 = 1.45;
/// V1-forfait voor de primaire hernieuwbare energiefactor van vaste biomassa.
/// De norm (tabel 5.4) onderscheidt bmA (1,0), bmB (0,5) en bmC (0) op basis van
/// thermisch vermogen en bijlage R; die classificatie ontbreekt in het huidige
/// invoermodel, dus wordt hier conservatief `bmA = 1,0` aangehouden op het
/// brandstofverbruik (F5: geleverde warmte × fPren;bmX i.p.v. brandstof).
const F_PREN_BIOMASSA_V1: f64 = 1.0;

/// Berekent het aandeel hernieuwbare energie `RERPrenTot` [0.0-1.0].
///
/// Implementeert NTA 8800:2025+C1:2026 formule (5.3) (§5.3.1.3, p. 72):
/// `RERPrenTot = EPrenTot / (EPTot + EPrenTot)`.
///
/// - `EPrenTot` (teller, §5.6) = som van de hernieuwbare primaire energie:
///   omgevingswarmte (warmtepompen + zonneboiler, `fPren;renheat = 1,0`),
///   omgevingskoude (EER ≥ 8, `fPren;rencold = 1,0`), lokaal opgewekte PV
///   (`fPren;renelect = 1,45`) en vaste biomassa (V1-forfait).
/// - `EPTot` (noemer-term, §5.5) = het karakteristieke primair-**fossiele**
///   energiegebruik *na* PV-saldering ([`total_primary_energy_mj`]).
///
/// Teller en noemer staan hier in MJ; als dimensieloze verhouding valt de
/// MJ↔kWh-conversie weg.
///
/// # Errors
///
/// Retourneert [`EpError`] bij validatiefouten in de input data
/// (via [`total_primary_energy_mj`]).
pub fn renewable_share(inputs: &EpInputs) -> Result<f64, EpError> {
    let e_pren_tot = renewable_primary_energy_mj(inputs);
    let e_p_tot = total_primary_energy_mj(inputs)?;

    let denominator = e_p_tot + e_pren_tot;
    if denominator <= 0.0 {
        // Geen (of netto-negatief) primair-fossiel + geen hernieuwbaar → geen
        // zinvolle verhouding; volledig hernieuwbaar als edge-case.
        return Ok(1.0);
    }

    Ok((e_pren_tot / denominator).clamp(0.0, 1.0))
}

/// Hernieuwbaar primair energiegebruik `EPrenTot` [MJ] (NTA 8800 §5.6,
/// formule 5.29). Teller van de `RERPrenTot`-indicator.
fn renewable_primary_energy_mj(inputs: &EpInputs) -> f64 {
    // Omgevingswarmte/-koude: reeds als bronzijdige hoeveelheid aangeleverd,
    // omgerekend met fPren = 1,0 (tabel 5.4).
    let mut e_pren = inputs
        .renewable_ambient_heat_mj
        .mul_add(F_PREN_RENHEAT, inputs.renewable_ambient_cold_mj * F_PREN_RENCOLD);

    // Lokaal opgewekte PV (§5.6.2.4, formule 5.39): volledige productie × 1,45.
    e_pren += inputs.pv_yield * F_PREN_RENELECT;

    // Vaste biomassa (§5.6.2.1/§5.6.2.3): V1-forfait op het brandstofverbruik.
    for &carrier in &[EnergyCarrier::Biomassa, EnergyCarrier::Pellets] {
        let biomass: f64 = [
            &inputs.heating,
            &inputs.cooling,
            &inputs.dhw,
            &inputs.lighting,
            &inputs.ventilation_aux,
            &inputs.automation,
        ]
        .iter()
        .filter_map(|m| m.get(&carrier))
        .sum();
        e_pren += biomass * F_PREN_BIOMASSA_V1;
    }

    e_pren
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
            renewable_ambient_heat_mj: 0.0,
            renewable_ambient_cold_mj: 0.0,
            building_area: BuildingArea { a_g: 150.0 },
        }
    }

    #[test]
    fn test_total_primary_energy() {
        let inputs = create_test_inputs();
        let total = total_primary_energy_mj(&inputs).unwrap();

        // Gas: 15000 * 1.000 = 15000 MJ
        // Elektriciteit: 3000 * 1.450 = 4350 MJ
        // PV: 5000 * 1.450 = 7250 MJ afgetrokken (fP;exp;el, tabel 5.2)
        // Totaal: 15000 + 4350 − 7250 = 12100 MJ
        let expected = 15000.0_f64.mul_add(1.000, 3000.0 * 1.450) - 5000.0 * 1.450;
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

        // RERPrenTot = EPrenTot / (EPTot + EPrenTot), formule (5.3):
        //   EPrenTot = 5000 PV × 1,45 (fPren;renelect)          = 7250 MJ
        //   EPTot    = 15000 gas + 3000 el × 1,45 − 5000 PV × 1,45
        //            = 15000 + 4350 − 7250                       = 12100 MJ
        //   RER      = 7250 / (12100 + 7250)                     ≈ 0,3747
        let expected = 7250.0 / (12100.0 + 7250.0);
        assert!((share - expected).abs() < 1e-9, "kreeg {share}, verwacht {expected}");
    }

    #[test]
    fn renewable_share_counts_heat_pump_ambient_heat() {
        // All-electric WP zonder PV: omgevingswarmte moet BENG 3 > 0 maken.
        // Q_H;use = 4000 MJ elektrisch, SCOP 4 → QH;hp;in = 4000 × (4−1) = 12000 MJ.
        let mut heating = HashMap::new();
        heating.insert(EnergyCarrier::Elektriciteit, 4000.0);
        let inputs = EpInputs {
            heating,
            cooling: HashMap::new(),
            dhw: HashMap::new(),
            lighting: HashMap::new(),
            ventilation_aux: HashMap::new(),
            automation: HashMap::new(),
            pv_yield: 0.0,
            renewable_ambient_heat_mj: 12_000.0,
            renewable_ambient_cold_mj: 0.0,
            building_area: BuildingArea { a_g: 100.0 },
        };
        let share = renewable_share(&inputs).unwrap();
        // EPrenTot = 12000; EPTot = 4000 × 1,45 = 5800 → 12000/(5800+12000) ≈ 0,674.
        let expected = 12_000.0 / (5_800.0 + 12_000.0);
        assert!((share - expected).abs() < 1e-9, "kreeg {share}, verwacht {expected}");
        assert!(share > 0.0, "omgevingswarmte moet BENG 3 boven 0 tillen");
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
            renewable_ambient_heat_mj: 0.0,
            renewable_ambient_cold_mj: 0.0,
            building_area: BuildingArea { a_g: 150.0 },
        };

        assert!(matches!(
            total_primary_energy_mj(&inputs),
            Err(EpError::NegativeEnergyUse { .. })
        ));
    }
}
