//! EP-score berekeningen volgens NTA 8800:2025 H.5.

use nta8800_model::zoning::UsageFunction;

use crate::{EpBreakdown, EpError, EpInputs, EpResult, ServiceBreakdown};

pub mod co2_factor;
pub mod ep_score;
pub mod label;
pub mod primary_energy;

/// Berekent de EP-score en label van een gebouw.
///
/// Integreert netto energiegebruik van alle diensten met primaire energiefactoren
/// en CO2-beleidsfactoren volgens NTA 8800:2025 H.5 + bijlagen Z en AB.
///
/// # Parameters
///
/// - `inputs`: Netto energiegebruik per dienst per energiedrager [MJ]
/// - `function`: Gebruiksfunctie voor label-drempel bepaling
///
/// # Errors
///
/// Retourneert [`EpError`] bij:
/// - Ontbrekende primaire energiefactoren of CO2-factoren
/// - Ongeldige gebouwgeometrie (A_g ≤ 0)
/// - Negatief energiegebruik
/// - Energiebalans validatiefouten
///
/// # Voorbeeld
///
/// ```rust,no_run
/// use nta8800_ep::{calculate_ep_score, EpInputs, EnergyCarrier, BuildingArea};
/// use nta8800_model::zoning::UsageFunction;
/// use std::collections::HashMap;
///
/// let mut heating = HashMap::new();
/// heating.insert(EnergyCarrier::Aardgas, 15000.0); // 15 GJ aardgas
///
/// let inputs = EpInputs {
///     heating,
///     cooling: HashMap::new(),
///     dhw: HashMap::new(),
///     lighting: HashMap::new(),
///     ventilation_aux: HashMap::new(),
///     automation: HashMap::new(),
///     pv_yield: 0.0,
///     building_area: BuildingArea { a_g: 150.0 },
/// };
///
/// let result = calculate_ep_score(&inputs, UsageFunction::Woonfunctie)?;
/// println!("EP-label: {}", result.ep_label.as_str());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn calculate_ep_score(inputs: &EpInputs, function: UsageFunction) -> Result<EpResult, EpError> {
    use ep_score::{
        renewable_share, specific_primary_energy_mj_per_m2, total_co2_kg, total_primary_energy_mj,
    };

    // 1. Bereken totalen
    let ep_total_mj = total_primary_energy_mj(inputs)?;
    let ep_total_mj_per_m2 = specific_primary_energy_mj_per_m2(inputs, &inputs.building_area)?;
    let ep_renewable_share = renewable_share(inputs)?;
    let total_co2 = total_co2_kg(inputs)?;
    let ep_co2_kg_per_m2 = total_co2 / inputs.building_area.a_g;

    // 2. Bepaal EP-label
    let ep_label = label::assign_label(ep_total_mj_per_m2, function)?;

    // 3. Maak breakdown per dienst
    let breakdown = create_service_breakdown(inputs)?;

    Ok(EpResult {
        ep_label,
        ep_total_mj,
        ep_total_mj_per_m2,
        ep_renewable_share,
        ep_co2_kg_per_m2,
        breakdown,
    })
}

/// Creëert gedetailleerde breakdown per energiedienst.
fn create_service_breakdown(inputs: &EpInputs) -> Result<EpBreakdown, EpError> {
    let heating = calculate_service_breakdown(&inputs.heating)?;
    let cooling = calculate_service_breakdown(&inputs.cooling)?;
    let dhw = calculate_service_breakdown(&inputs.dhw)?;
    let lighting = calculate_service_breakdown(&inputs.lighting)?;
    let ventilation_aux = calculate_service_breakdown(&inputs.ventilation_aux)?;
    let automation = calculate_service_breakdown(&inputs.automation)?;

    // PV als negatieve dienst
    let pv = ServiceBreakdown {
        primary_energy_mj: -inputs.pv_yield
            * primary_energy::primary_factor(crate::EnergyCarrier::HernieuwbareElektriciteit)?,
        co2_kg: -inputs.pv_yield
            * co2_factor::co2_factor(crate::EnergyCarrier::HernieuwbareElektriciteit)?,
        renewable_fraction: if inputs.pv_yield > 0.0 { 1.0 } else { 0.0 },
    };

    Ok(EpBreakdown {
        heating,
        cooling,
        dhw,
        lighting,
        ventilation_aux,
        automation,
        pv,
    })
}

/// Berekent breakdown voor één energiedienst.
fn calculate_service_breakdown(
    service: &std::collections::HashMap<crate::EnergyCarrier, f64>,
) -> Result<ServiceBreakdown, EpError> {
    let mut primary_energy_mj = 0.0;
    let mut co2_kg = 0.0;
    let mut renewable_energy = 0.0;
    let mut total_energy = 0.0;

    for (&carrier, &energy) in service {
        let pf = primary_energy::primary_factor(carrier)?;
        let cf = co2_factor::co2_factor(carrier)?;

        primary_energy_mj += energy * pf;
        co2_kg += energy * cf;
        total_energy += energy;

        if carrier.is_renewable() {
            renewable_energy += energy;
        }
    }

    let renewable_fraction = if total_energy > 0.0 {
        renewable_energy / total_energy
    } else {
        0.0
    };

    Ok(ServiceBreakdown {
        primary_energy_mj,
        co2_kg,
        renewable_fraction,
    })
}
