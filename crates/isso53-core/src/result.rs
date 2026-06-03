//! Result types for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Complete calculation results for an entire project.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResult {
    /// Results for each room.
    pub rooms: Vec<RoomResult>,

    /// Building-level summary.
    pub summary: BuildingSummary,
}

/// Heat loss calculation results for a single room.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoomResult {
    /// Room identifier.
    pub room_id: String,

    /// Room name.
    pub room_name: String,

    /// Design indoor temperature θ_i in °C.
    pub theta_i: f64,

    /// Transmission heat loss Φ_T in W.
    pub phi_t: f64,

    /// Ventilation heat loss Φ_v in W.
    pub phi_v: f64,

    /// Infiltration heat loss Φ_i in W.
    pub phi_i: f64,

    /// Heating-up supplement Φ_hu in W.
    pub phi_hu: f64,

    /// System losses Φ_system in W.
    pub phi_system: f64,

    /// Internal heat gains Φ_gain in W (negative = heat source).
    pub phi_gain: f64,

    /// Total heat loss Φ_HL,i in W.
    pub total_heat_loss: f64,

    /// Transmission coefficient H_T,ie to exterior in W/K.
    pub h_t_exterior: f64,

    /// Transmission coefficient H_T,ia to adjacent rooms in W/K.
    pub h_t_adjacent_rooms: f64,

    /// Transmission coefficient H_T,iae to unheated spaces in W/K.
    pub h_t_unheated: f64,

    /// Transmission coefficient H_T,iaBE to adjacent buildings in W/K.
    pub h_t_adjacent_buildings: f64,

    /// Transmission coefficient H_T,ig to ground in W/K.
    pub h_t_ground: f64,

    /// Ventilation coefficient H_v in W/K.
    pub h_v: f64,

    /// Infiltration coefficient H_i in W/K.
    pub h_i: f64,

    /// Ventilation flow rate q_v in dm³/s (= m³/s × 1000).
    pub q_v: f64,
}

/// Herkomst van de actieve infiltratie-rekenmethode (hybride-beleid, C1).
///
/// Onder het hybride conform-beleid (norm leidend; Vabi-compat alleen achter een
/// expliciet gemarkeerd pad) moet het rapport transparant kunnen tonen of de
/// infiltratie volgens de ISSO 53-norm óf via de Vabi-compat power-law
/// (NEN 8088-1, NTA 8800, Δp-power-law) is berekend. Deze flag draagt die
/// herkomst naar de resultaat-output zodat de rapportagelaag het kan markeren.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InfiltrationMethodOrigin {
    /// ISSO 53-norm-pad: tabel 4.5 (Known) of formule 4.31 (Unknown). Norm-puur.
    Isso53Norm,
    /// Vabi-compat-pad: NEN 8088-1 (f_type/f_inf) + NTA 8800 (f_jaar) met
    /// power-law drukconversie (Δp ≈ 3,14 Pa). **Geen ISSO 53-norm** — bewust
    /// gekozen voor Vabi-reproductie; rapport markeert dit expliciet.
    VabiCompat,
}

/// Building-level summary results.
/// ISSO 53 uses simple addition, not quadratic summation like ISSO 51.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildingSummary {
    /// Total transmission heat loss Φ_T,build in W.
    pub total_transmission_loss: f64,

    /// Total ventilation heat loss Φ_V,build in W.
    pub total_ventilation_loss: f64,

    /// Total building ventilation flow Σ q_v in dm³/s.
    pub total_ventilation_flow: f64,

    /// Total infiltration heat loss Φ_I,build in W.
    pub total_infiltration_loss: f64,

    /// Total heating-up supplement Φ_hu,build in W.
    pub total_heating_up: f64,

    /// Total system losses Φ_system,build in W.
    pub total_system_losses: f64,

    /// Total internal gains Φ_gain,build in W.
    pub total_internal_gains: f64,

    /// Total building heat loss Φ_HL,build in W.
    pub total_building_heat_loss: f64,

    /// Connection capacity individual method Φ_source in W (formule 5.1).
    pub connection_capacity_individual: f64,

    /// Connection capacity collective method Φ_source in W (formule 5.9).
    pub connection_capacity_collective: f64,

    /// Shell method heat loss Φ_HL,shell in W (hoofdstuk 3).
    pub shell_heat_loss: f64,

    /// Infiltration reduction factor z applied at building level (tabel 5.1).
    pub infiltration_reduction_factor_z: f64,

    /// Gelijktijdigheidsfactor `g` toegepast op Σ Φ_hu in het aansluitvermogen
    /// (ISSO 53 §4.1/§5.1). `1,0` = 100% gelijktijdigheid (default, engine-aanname).
    /// Het rapport kan hiermee tonen welke gelijktijdigheid is aangenomen en dat
    /// dit met de opdrachtgever moet worden afgestemd.
    pub heating_up_simultaneity_factor: f64,

    /// Herkomst van de gebruikte infiltratie-rekenmethode (C1, hybride-beleid):
    /// `isso53Norm` of `vabiCompat`. Maakt expliciet of de infiltratie norm-puur
    /// of via de Vabi-compat power-law is berekend, zodat het rapport het
    /// transparant kan markeren.
    pub infiltration_method_origin: InfiltrationMethodOrigin,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_room() -> RoomResult {
        RoomResult {
            room_id: "r1".to_string(),
            room_name: "Woonkamer".to_string(),
            theta_i: 20.0,
            phi_t: 1000.0,
            phi_v: 500.0,
            phi_i: 200.0,
            phi_hu: 100.0,
            phi_system: 0.0,
            phi_gain: 0.0,
            total_heat_loss: 1800.0,
            h_t_exterior: 25.0,
            h_t_adjacent_rooms: 0.0,
            h_t_unheated: 10.0,
            h_t_adjacent_buildings: 0.0,
            h_t_ground: 5.0,
            h_v: 15.0,
            h_i: 8.0,
            q_v: 50.0,
        }
    }

    fn sample_summary() -> BuildingSummary {
        BuildingSummary {
            total_transmission_loss: 1000.0,
            total_ventilation_loss: 500.0,
            total_ventilation_flow: 50.0,
            total_infiltration_loss: 200.0,
            total_heating_up: 100.0,
            total_system_losses: 0.0,
            total_internal_gains: 0.0,
            total_building_heat_loss: 1800.0,
            connection_capacity_individual: 1900.0,
            connection_capacity_collective: 1850.0,
            shell_heat_loss: 1700.0,
            infiltration_reduction_factor_z: 1.0,
            heating_up_simultaneity_factor: 1.0,
            infiltration_method_origin: InfiltrationMethodOrigin::Isso53Norm,
        }
    }

    /// Borgt dat het q_v-veld als JSON-key `qV` serialiseert (serde
    /// single-letter camelCase-landmijn) en dat de roundtrip identiek blijft.
    #[test]
    fn room_result_serializes_qv_key_and_roundtrips() {
        let room = sample_room();
        let value = serde_json::to_value(&room).expect("serialize room");
        assert!(
            value.get("qV").is_some(),
            "RoomResult moet JSON-key `qV` bevatten, kreeg: {value}"
        );
        assert_eq!(value.get("qV").and_then(|v| v.as_f64()), Some(50.0));

        let back: RoomResult =
            serde_json::from_value(value).expect("deserialize room");
        assert_eq!(back.q_v, room.q_v);
        assert_eq!(back.room_id, room.room_id);
    }

    /// Borgt dat het total_ventilation_flow-veld als JSON-key
    /// `totalVentilationFlow` serialiseert en de roundtrip identiek blijft.
    #[test]
    fn building_summary_serializes_total_ventilation_flow_key_and_roundtrips() {
        let summary = sample_summary();
        let value = serde_json::to_value(&summary).expect("serialize summary");
        assert!(
            value.get("totalVentilationFlow").is_some(),
            "BuildingSummary moet JSON-key `totalVentilationFlow` bevatten, kreeg: {value}"
        );
        assert_eq!(
            value
                .get("totalVentilationFlow")
                .and_then(|v| v.as_f64()),
            Some(50.0)
        );

        let back: BuildingSummary =
            serde_json::from_value(value).expect("deserialize summary");
        assert_eq!(back.total_ventilation_flow, summary.total_ventilation_flow);
        assert_eq!(back.total_ventilation_loss, summary.total_ventilation_loss);
    }
}