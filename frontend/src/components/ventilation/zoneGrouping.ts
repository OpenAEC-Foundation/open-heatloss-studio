/**
 * Zone-groepering voor de ventilatiebalans — pure helpers.
 *
 * Groepeert de project-rooms op `Room.zoneId` (zie `types/project.ts::Zone`)
 * in de volgorde van `Building.zones`, met een "Niet ingedeeld"-restgroep
 * voor ruimtes zonder (of met een dangling) `zoneId`. Zonder zones rendert
 * `pages/VentilationBalance.tsx` de bestaande platte tabel — deze helpers
 * worden dan niet aangeroepen.
 *
 * **Eenheden:** dm³/s intern, conform de rest van de ventilatie-keten;
 * weergave-conversie (dm³/s ↔ m³/h) gebeurt aan de UI-rand.
 */

import type { Zone } from "../../types";
import type { RoomVentilationBalance } from "../../lib/ventilationBalance";

/** Eén weergave-groep: een zone (of `undefined` = "Niet ingedeeld") + rooms. */
export interface ZoneGroup<R extends { zoneId?: string }> {
  /** De zone; `undefined` voor de "Niet ingedeeld"-restgroep. */
  zone: Zone | undefined;
  rooms: R[];
}

/**
 * Groepeer rooms op `zoneId`, in de volgorde van `zones`. Regels:
 *   - lege zones (geen rooms) worden weggelaten — geen lege kopjes;
 *   - rooms zonder `zoneId` óf met een id dat niet (meer) in `zones`
 *     voorkomt, landen in de restgroep (`zone: undefined`) achteraan;
 *   - de room-volgorde binnen een groep volgt de project-volgorde.
 */
export function groupRoomsByZone<R extends { zoneId?: string }>(
  rooms: R[],
  zones: Zone[],
): ZoneGroup<R>[] {
  // Defensief: een corrupt project-JSON (of een niet-getypte caller) kan een
  // niet-array als `zones` binnenlaten. `importExport.validateProject` stript
  // dat al bij import, maar deze guard voorkomt hoe dan ook een `.map`-crash.
  const zoneList = Array.isArray(zones) ? zones : [];
  const groups: ZoneGroup<R>[] = zoneList.map((zone) => ({ zone, rooms: [] }));
  const byZoneId = new Map(groups.map((g) => [g.zone!.id, g]));
  const unassigned: ZoneGroup<R> = { zone: undefined, rooms: [] };

  for (const room of rooms) {
    const group =
      room.zoneId !== undefined ? byZoneId.get(room.zoneId) : undefined;
    (group ?? unassigned).rooms.push(room);
  }

  const out = groups.filter((g) => g.rooms.length > 0);
  if (unassigned.rooms.length > 0) out.push(unassigned);
  return out;
}

/** Subtotalen (dm³/s) van een zone-groep — eis + aanwezig per richting. */
export interface ZoneSubtotal {
  requiredSupplyDm3s: number;
  requiredExhaustDm3s: number;
  presentSupplyDm3s: number;
  presentExhaustDm3s: number;
}

/**
 * Som de balans-regels van de gegeven room-ids op tot een zone-subtotaal.
 * Ids zonder balans-regel (verwijderde/onbekende ruimte) tellen als 0 —
 * zelfde tolerantie als `aggregateVentilationBalance` voor terminals.
 */
export function sumZoneBalance(
  roomIds: string[],
  balanceRooms: Record<string, RoomVentilationBalance>,
): ZoneSubtotal {
  const out: ZoneSubtotal = {
    requiredSupplyDm3s: 0,
    requiredExhaustDm3s: 0,
    presentSupplyDm3s: 0,
    presentExhaustDm3s: 0,
  };
  for (const id of roomIds) {
    const row = balanceRooms[id];
    if (!row) continue;
    out.requiredSupplyDm3s += row.requiredSupplyDm3s;
    out.requiredExhaustDm3s += row.requiredExhaustDm3s;
    out.presentSupplyDm3s += row.presentSupplyDm3s;
    out.presentExhaustDm3s += row.presentExhaustDm3s;
  }
  return out;
}
