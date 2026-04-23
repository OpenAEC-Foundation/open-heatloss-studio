/**
 * Pure ΔT-logica voor transmissieverliezen per construction-element.
 *
 * Losgekoppeld van `ConstructionLossChart.tsx` zodat de helpers
 * testbaar blijven zonder JSX/React runtime.
 *
 * Spec: sessions/warmteverlies_adjacent_room_temp_spec.md §4.3
 */

import type { BoundaryType, ConstructionElement, Room } from "../../types/project.ts";
import { DEFAULT_THETA_WATER, ROOM_FUNCTION_TEMPERATURES } from "../../lib/constants.ts";

/**
 * Bepaalt de design-temperatuur voor een (buur-)ruimte.
 *
 * Voorrang:
 *   1. `custom_temperature` indien gezet
 *   2. `ROOM_FUNCTION_TEMPERATURES[function]` default
 *   3. 20 °C fallback
 */
export function getRoomDesignTemperature(room: Room): number {
  if (room.custom_temperature != null) {
    return room.custom_temperature;
  }
  return ROOM_FUNCTION_TEMPERATURES[room.function] ?? 20;
}

/** Bouw een id → Room lookup voor snelle adjacent-room resolutie. */
export function buildRoomLookup(rooms: Room[]): Map<string, Room> {
  const map = new Map<string, Room>();
  for (const r of rooms) {
    map.set(r.id, r);
  }
  return map;
}

/** Context voor `computeDeltaT`. */
export interface DeltaTContext {
  rooms: Map<string, Room>;
  thetaWater: number;
}

/**
 * Bereken ΔT voor een construction-element op basis van zijn boundary-type.
 *
 * - `adjacent_room`: live lookup via `adjacent_room_id` → `getRoomDesignTemperature`.
 *   Valt terug op legacy `adjacent_temperature` als er geen ctx-match is.
 * - `water`: gebruikt `ctx.thetaWater` (default 5 °C).
 * - `adjacent_building`: legacy `adjacent_temperature` of θ_e.
 * - `unheated_space`: past `temperature_factor` toe als aanwezig.
 * - `exterior` / `ground` / fallback: θ_i − θ_e.
 */
export function computeDeltaT(
  boundaryType: BoundaryType,
  thetaI: number,
  thetaE: number,
  ce: {
    temperature_factor?: number | null;
    adjacent_temperature?: number | null;
    adjacent_room_id?: string | null;
  },
  ctx?: DeltaTContext,
): number {
  switch (boundaryType) {
    case "exterior":
      return thetaI - thetaE;
    case "ground":
      return thetaI - thetaE;
    case "unheated_space":
      if (ce.temperature_factor != null) {
        return ce.temperature_factor * (thetaI - thetaE);
      }
      return thetaI - thetaE;
    case "adjacent_building":
      return thetaI - (ce.adjacent_temperature ?? thetaE);
    case "adjacent_room": {
      // Live lookup via room-id — one source of truth.
      if (ce.adjacent_room_id && ctx) {
        const adjacent = ctx.rooms.get(ce.adjacent_room_id);
        if (adjacent) {
          return thetaI - getRoomDesignTemperature(adjacent);
        }
      }
      // Legacy fallback — oude projecten met enkel adjacent_temperature.
      if (ce.adjacent_temperature != null) {
        return thetaI - ce.adjacent_temperature;
      }
      return 0;
    }
    case "water": {
      const thetaW = ctx?.thetaWater ?? DEFAULT_THETA_WATER;
      return thetaI - thetaW;
    }
    default:
      return thetaI - thetaE;
  }
}

/** Heeft het project ten minste één water-grensvlak? */
export function hasWaterBoundaries(rooms: Room[]): boolean {
  for (const room of rooms) {
    for (const ce of room.constructions as ConstructionElement[]) {
      if (ce.boundary_type === "water") return true;
    }
  }
  return false;
}
