/**
 * ISSO 53 — detectie van ONVERWARMDE doelruimtes + f_k-default.
 *
 * Het model kent geen expliciete heated/unheated-vlag per ruimte. Een ruimte
 * is "onverwarmd doel" doordat een (verwarmde) ruimte een constructie heeft
 * met `boundary_type === "unheated_space"` waarvan `adjacent_room_id` naar die
 * onverwarmde ruimte wijst.
 *
 * Voor zulke doelruimtes is de temperatuurreductiefactor f_k instelbaar via de
 * sidecar (`Isso53RoomState.unheatedFactor`). Ontbreekt die → norm-default
 * {@link DEFAULT_UNHEATED_FACTOR} (0,5), isso51-consistent met de Rust
 * `h_t_unheated_element` `unwrap_or(0.5)`.
 */
import type { Room } from "../types/project";
import type { Isso53RoomState } from "../types/projectV2";

/** Norm-default temperatuurreductiefactor f_k voor onverwarmde ruimtes. */
export const DEFAULT_UNHEATED_FACTOR = 0.5;

/**
 * Verzamel de set room-ids die ergens in het project als ONVERWARMD doel
 * fungeren: het `adjacent_room_id` van een `unheated_space`-constructie.
 *
 * Pure functie — geen store-coupling. Lege/ontbrekende `adjacent_room_id`'s
 * worden genegeerd.
 */
export function collectUnheatedTargetIds(rooms: Room[]): Set<string> {
  const ids = new Set<string>();
  for (const room of rooms) {
    for (const ce of room.constructions) {
      if (ce.boundary_type === "unheated_space" && ce.adjacent_room_id) {
        ids.add(ce.adjacent_room_id);
      }
    }
  }
  return ids;
}

/**
 * Is `roomId` ergens in het project een onverwarmd-doel? Convenience-wrapper
 * rond {@link collectUnheatedTargetIds} voor een enkele check (UI-gating).
 */
export function isUnheatedTarget(rooms: Room[], roomId: string): boolean {
  return collectUnheatedTargetIds(rooms).has(roomId);
}

/**
 * Gecombineerde set ONVERWARMDE room-ids:
 *   {@link collectUnheatedTargetIds}(rooms)  (impliciete doelen via
 *   `unheated_space`-constructies)  ∪  alle room-ids met
 *   `isso53Rooms[id].isUnheated === true`  (expliciet gemarkeerde vertrekken).
 *
 * Wanden van buren náár een room in deze set worden in mapper én chart als
 * grensvlak naar een onverwarmde ruimte behandeld (f_k-reductie), ongeacht of
 * de bron-constructie `unheated_space` of `adjacent_room` is.
 *
 * Pure functie — geen store-coupling.
 */
export function resolveUnheatedRoomIds(
  rooms: Room[],
  isso53Rooms: Record<string, Isso53RoomState>,
): Set<string> {
  const ids = collectUnheatedTargetIds(rooms);
  for (const [roomId, sidecar] of Object.entries(isso53Rooms)) {
    if (sidecar.isUnheated) {
      ids.add(roomId);
    }
  }
  return ids;
}
