/**
 * ISSO 53 BBL-minimum ventilatie-helper.
 *
 * De BBL-minimumeis voor een verblijfsgebied is uniform 0,9 dm³/s per m²
 * vloeroppervlak. Dit wordt gebruikt als placeholder/auto-waarde voor het
 * q_v-veld in de vertrekkenrij (`VentilationRow`): een leeg q_v → BBL-minimum.
 *
 * De rekenkern (`ventilation_q_v_established`) blijft leidend op de uiteindelijk
 * doorgegeven waarde; deze helper levert puur de invul-/placeholder-waarde.
 */

/** ISSO 53 BBL-minimum ventilatie (verblijfsgebied): 0,9 dm³/s per m². */
export function isso53BblMinimumDm3s(floorAreaM2: number): number {
  return 0.9 * floorAreaM2;
}
