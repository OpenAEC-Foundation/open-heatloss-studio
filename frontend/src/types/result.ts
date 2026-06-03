// Generated from schemas/v1/result.schema.json
// Re-generate: npm run generate-types

export interface TransmissionResult {
  h_t_exterior: number;
  h_t_adjacent_rooms: number;
  h_t_unheated: number;
  h_t_adjacent_buildings: number;
  h_t_ground: number;
  /**
   * Transmissieverlies naar water (H_T,iw) in W/K. Optioneel voor
   * backward-compat met oude backend-responses zonder water boundary.
   */
  h_t_water?: number;
  phi_t: number;
  norm_refs?: string[];
}

export interface InfiltrationResult {
  h_i: number;
  z_i: number;
  phi_i: number;
  norm_refs?: string[];
}

export interface VentilationResult {
  h_v: number;
  f_v: number;
  q_v: number;
  phi_v: number;
  phi_vent: number;
  norm_refs?: string[];
}

export interface HeatingUpResult {
  phi_hu: number;
  /** Specifieke opwarmtoeslag P in W/m² (ISSO 51:2023 Tabel 2.10). */
  p: number;
  /** Gebruiksoppervlak A_g in m² waarop Φ_hu is berekend. */
  a_g: number;
  norm_refs?: string[];
}

export interface SystemLossResult {
  phi_floor_loss: number;
  phi_wall_loss: number;
  phi_ceiling_loss: number;
  phi_system_total: number;
  norm_refs?: string[];
}

export interface RoomResult {
  room_id: string;
  room_name: string;
  theta_i: number;
  transmission: TransmissionResult;
  infiltration: InfiltrationResult;
  ventilation: VentilationResult;
  heating_up: HeatingUpResult;
  system_losses: SystemLossResult;
  total_heat_loss: number;
  basis_heat_loss: number;
  extra_heat_loss: number;
}

export interface BuildingSummary {
  total_envelope_loss: number;
  total_neighbor_loss: number;
  total_ventilation_loss: number;
  total_heating_up: number;
  total_system_losses: number;
  connection_capacity: number;
  collective_contribution: number;
}

export interface ProjectResult {
  rooms: RoomResult[];
  summary: BuildingSummary;
}
