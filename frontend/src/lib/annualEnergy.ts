import type { RoomResult } from "../types/result";
import type { Isso53RoomResult } from "../types/isso53Result";

/**
 * Klimaat-graaddagen (HDD) NL gemiddelde — KNMI De Bilt 1991-2020 normaal,
 * basis 18/18 °C. Eenheid: K·d/jaar.
 *
 * Bron: KNMI klimatologie graaddagen (https://www.knmi.nl/), gebruikt als
 * standaard referentie voor geschatte jaarlijkse warmtebehoefte.
 */
export const HDD_NL = 2900;

/**
 * Schat de jaarlijkse netto warmtebehoefte (kWh/jaar) volgens de
 * graaddagen-methode.
 *
 * Formule:
 *   H_extern (W/K) = Σ rooms (transmission.h_t_exterior + h_t_unheated
 *     + h_t_adjacent_buildings + h_t_ground + h_t_water
 *     + infiltration.h_i + ventilation.h_v)
 *   Q_jaar (kWh/jaar) = H_extern × HDD_NL × 24 / 1000
 *
 * NB: `h_t_adjacent_rooms` (interne transmissie tussen verwarmde vertrekken)
 * wordt bewust uitgesloten — dat is geen netto jaarverlies van de woning.
 *
 * Dit is een eerste-orde schatting; niet norm-conform BENG/NTA 8800.
 * Werkelijk verbruik wijkt af door zoninstraling, interne warmte en gebruik.
 */
export function computeAnnualHeatDemandKWh(rooms: RoomResult[]): {
  hExternal: number;
  annualKWh: number;
} {
  let hExternal = 0;
  for (const room of rooms) {
    const t = room.transmission;
    hExternal +=
      t.h_t_exterior +
      t.h_t_unheated +
      t.h_t_adjacent_buildings +
      t.h_t_ground +
      (t.h_t_water ?? 0) +
      room.infiltration.h_i +
      room.ventilation.h_v;
  }
  const annualKWh = (hExternal * HDD_NL * 24) / 1000;
  return { hExternal, annualKWh };
}

/**
 * Schat de jaarlijkse netto warmtebehoefte (kWh/jaar) volgens de
 * graaddagen-methode — ISSO 53-variant.
 *
 * Formule:
 *   H_extern (W/K) = Σ rooms (hTExterior + hTUnheated + hTAdjacentBuildings
 *     + hTGround + hV + hI)
 *   Q_jaar (kWh/jaar) = H_extern × HDD_NL × 24 / 1000
 *
 * NB: `hTAdjacentRooms` (interne transmissie tussen verwarmde vertrekken)
 * wordt bewust uitgesloten — dat is geen netto jaarverlies van het gebouw.
 * ISSO 53 kent geen water-boundary, dus dat veld ontbreekt hier.
 *
 * Dit is een eerste-orde schatting; niet norm-conform BENG/NTA 8800.
 * Werkelijk verbruik wijkt af door zoninstraling, interne warmte en gebruik.
 */
export function computeAnnualHeatDemandKWhIsso53(rooms: Isso53RoomResult[]): {
  hExternal: number;
  annualKWh: number;
} {
  let hExternal = 0;
  for (const room of rooms) {
    hExternal +=
      room.hTExterior +
      room.hTUnheated +
      room.hTAdjacentBuildings +
      room.hTGround +
      room.hV +
      room.hI;
  }
  const annualKWh = (hExternal * HDD_NL * 24) / 1000;
  return { hExternal, annualKWh };
}
