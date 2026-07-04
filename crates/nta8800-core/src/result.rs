//! Resultaat-typen van de nta8800-core keten.
//!
//! Eigen serialiseerbare structen (serde + schemars) — bewust ontkoppeld van
//! de interne result-typen van de sub-crates zodat het publieke JSON-contract
//! stabiel blijft bij interne refactors. Alle energie in **MJ** conform de
//! workspace-conventie; jaartotalen ook in kWh voor rapportage-gemak.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Volledig keten-resultaat: per dienst een samenvatting + de EP-eindscore.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Nta8800Result {
    /// Warmte-/koudebehoefte (H.7) + thermische kern-parameters.
    pub demand: DemandSummary,
    /// Verwarming (H.9).
    pub heating: ServiceSummary,
    /// Koeling (H.10) — `None` als geen actieve koeling is ingevoerd.
    pub cooling: Option<ServiceSummary>,
    /// Warm tapwater (H.13).
    pub dhw: ServiceSummary,
    /// Verlichting (H.14) — alleen gevuld voor utiliteitsfuncties.
    pub lighting: Option<ServiceSummary>,
    /// Ventilatie-warmteverlies + hulpenergie (H.11).
    pub ventilation: VentilationSummary,
    /// PV-opbrengst (H.16) — `None` als geen PV is ingevoerd.
    pub pv: Option<PvSummary>,
    /// EP-score + energielabel (H.5).
    pub ep: EpSummary,
    /// BENG 1/2/3-indicatoren met indicatieve nieuwbouw-toetsing.
    pub beng: BengSummary,
}

/// BENG-indicatoren (Bouwbesluit nieuwbouw-eisen, afgeleid uit de keten).
///
/// - **BENG 1** — energiebehoefte: `(Q_H;nd + Q_C;nd) / A_g` in kWh/(m²·jaar)
/// - **BENG 2** — primair (fossiel) energiegebruik: `E_P;tot / A_g` in
///   kWh/(m²·jaar). In deze keten tellen hernieuwbare dragers met
///   `f_prim = 0`, dus het EP-totaal is per constructie het fossiele deel.
/// - **BENG 3** — aandeel hernieuwbare energie in %.
///
/// **Woonfunctie**: de BENG 1-grens volgt de BBL/Bouwbesluit-2021
/// compactheids-formule op basis van `A_ls/A_g` (verliesoppervlak van de
/// thermische schil gedeeld door gebruiksoppervlak):
///
/// ```text
/// ratio ≤ 1,83          → 55
/// 1,83 < ratio ≤ 3,0    → 55 + 30·(ratio − 1,83)
/// ratio > 3,0           → 100 + 50·(ratio − 3,0)
/// ```
///
/// **Utiliteit**: vaste indicatieve grenzen per gebruiksfunctie
/// (consistent met de OpenAEC open-energy-studio referentie-tabel); de
/// formele utiliteits-formules met eigen compactheids- en
/// daglicht-correcties zijn V2.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BengSummary {
    /// BENG 1 — energiebehoefte in kWh/(m²·jaar).
    pub beng1_kwh_per_m2: f64,
    /// BENG 2 — primair energiegebruik in kWh/(m²·jaar).
    pub beng2_kwh_per_m2: f64,
    /// BENG 3 — hernieuwbaar aandeel in % (0-100).
    pub beng3_pct: f64,
    /// Compactheid `A_ls/A_g` — verliesoppervlak (som van alle
    /// schil-elementen incl. ramen) gedeeld door gebruiksoppervlak.
    pub a_ls_over_a_g: f64,
    /// BENG 1-grens (≤). Woonfunctie: BBL-compactheidsformule;
    /// utiliteit: indicatief vast.
    pub beng1_limit: f64,
    /// BENG 2-grens (≤). Indicatief vast per gebruiksfunctie.
    pub beng2_limit: f64,
    /// BENG 3-grens (≥). Indicatief vast per gebruiksfunctie.
    pub beng3_limit: f64,
    /// BENG 1 binnen de grens.
    pub beng1_pass: bool,
    /// BENG 2 binnen de grens.
    pub beng2_pass: bool,
    /// BENG 3 op of boven de grens.
    pub beng3_pass: bool,
}

/// Samenvatting van de behoefte-keten (H.7).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DemandSummary {
    /// Maandelijkse warmtebehoefte Q_H;nd in MJ.
    pub monthly_q_h_nd_mj: [f64; 12],
    /// Maandelijkse koudebehoefte Q_C;nd in MJ.
    pub monthly_q_c_nd_mj: [f64; 12],
    /// Jaarlijkse warmtebehoefte in MJ.
    pub annual_q_h_nd_mj: f64,
    /// Jaarlijkse koudebehoefte in MJ.
    pub annual_q_c_nd_mj: f64,
    /// Transmissie-conductance H_tr in W/K (som H_D + H_U + H_g;an + H_A).
    pub h_tr_w_per_k: f64,
    /// Ventilatie-conductance H_ve in W/K (voedt de tijdconstante).
    pub h_ve_w_per_k: f64,
    /// Tijdconstante τ van de rekenzone in uren.
    pub tau_hours: f64,
}

/// Samenvatting van één energiedienst (verwarming / koeling / tapwater /
/// verlichting).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceSummary {
    /// Energiedrager waarop deze dienst draait.
    pub energy_carrier: String,
    /// Maandelijks eindgebruik in MJ.
    pub monthly_use_mj: [f64; 12],
    /// Jaarlijks eindgebruik in MJ.
    pub annual_use_mj: f64,
    /// Jaarlijks eindgebruik in kWh (MJ / 3,6).
    pub annual_use_kwh: f64,
    /// Totaal keten-rendement (η_em × η_dist × η_gen × f_reg) waar van
    /// toepassing; voor warmtepompen > 1 mogelijk (SCOP in de keten).
    pub total_efficiency: Option<f64>,
}

/// Samenvatting van de ventilatie-keten (H.11).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VentilationSummary {
    /// Jaarlijks ventilatie-warmteverlies Q_V in MJ (onderdeel van de
    /// behoefte, geen aparte dienst).
    pub annual_q_v_mj: f64,
    /// Jaarlijkse ventilator-hulpenergie W_fan in MJ elektrisch — telt als
    /// `ventilation_aux` mee in de EP-score.
    pub annual_w_fan_mj: f64,
    /// Jaarlijkse WTW-warmteterugwinning in MJ (0 zonder WTW).
    pub annual_wtw_recovery_mj: f64,
}

/// Samenvatting van de PV-keten (H.16).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PvSummary {
    /// Maandelijkse opbrengst in MJ elektrisch.
    pub monthly_yield_mj: [f64; 12],
    /// Jaarlijkse opbrengst in MJ elektrisch.
    pub annual_yield_mj: f64,
    /// Jaarlijkse opbrengst in kWh.
    pub annual_yield_kwh: f64,
}

/// EP-eindscore (H.5).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EpSummary {
    /// Energielabel (A++++ t/m G) als string.
    pub label: String,
    /// Totale primaire energie in MJ.
    pub primary_energy_mj: f64,
    /// Specifieke primaire energie in MJ/m² (label-bepalend).
    pub primary_energy_mj_per_m2: f64,
    /// Specifieke primaire energie in kWh/m² (rapportage-gemak).
    pub primary_energy_kwh_per_m2: f64,
    /// Aandeel hernieuwbare energie (0..=1).
    pub renewable_share: f64,
    /// CO₂-emissie in kg/m² per jaar.
    pub co2_kg_per_m2: f64,
    /// Primaire energie per dienst in MJ (traceability).
    pub per_service_primary_mj: PerServicePrimary,
}

/// Primaire energie per dienst (MJ) — PV als negatieve post.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerServicePrimary {
    /// Verwarming.
    pub heating: f64,
    /// Koeling.
    pub cooling: f64,
    /// Warm tapwater.
    pub dhw: f64,
    /// Verlichting.
    pub lighting: f64,
    /// Ventilatoren (hulpenergie).
    pub ventilation_aux: f64,
    /// PV-opbrengst (negatief = opbrengst).
    pub pv: f64,
}
