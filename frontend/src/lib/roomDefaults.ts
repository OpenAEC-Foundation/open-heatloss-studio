import type { CatalogueEntry } from "./constructionCatalogue";

import type { ConstructionElement, Room, RoomFunction } from "../types";

/** BBL specific ventilation rate for verblijfsruimten in dm³/s per m². */
const BBL_QV_SPEC_LIVING = 0.9;
/** BBL minimum ventilation per verblijfsruimte in dm³/s. */
const BBL_QV_MIN_LIVING = 7.0;

/**
 * Calculate the BBL minimum ventilation rate for a room.
 * BBL Afdeling 3.6: Luchtverversing.
 *
 * @param func - Room function
 * @param floorArea - Floor area in m²
 * @returns Minimum ventilation rate in dm³/s
 */
export function bblMinimumVentilationRate(
  func: RoomFunction,
  floorArea: number,
): number {
  switch (func) {
    case "living_room":
    case "bedroom":
    case "attic":
      return Math.max(BBL_QV_SPEC_LIVING * floorArea, BBL_QV_MIN_LIVING);
    case "kitchen":
      return 21;
    case "bathroom":
      return 14;
    case "toilet":
      return 7;
    case "hallway":
    case "landing":
    case "storage":
    case "custom":
      return 0;
  }
}

/** Create a new Room with sensible defaults. */
export function createRoom(): Room {
  return {
    id: crypto.randomUUID(),
    name: "Nieuw vertrek",
    function: "living_room",
    floor_area: 0,
    height: 2.6,
    constructions: [],
    heating_system: "radiator_ht",
  };
}

/** Create a new ConstructionElement with sensible defaults. */
export function createConstruction(): ConstructionElement {
  return {
    id: crypto.randomUUID(),
    description: "",
    area: 0,
    u_value: 0,
    boundary_type: "exterior",
    material_type: "masonry",
    vertical_position: "wall",
    use_forfaitaire_thermal_bridge: true,
  };
}

/** Create a ConstructionElement pre-filled from a catalogue entry. */
export function createConstructionFromCatalogue(
  entry: CatalogueEntry,
): ConstructionElement {
  return {
    id: crypto.randomUUID(),
    description: entry.name,
    area: 0,
    u_value: entry.uValue,
    boundary_type: entry.boundaryType ?? "exterior",
    material_type: entry.materialType,
    vertical_position: entry.verticalPosition,
    use_forfaitaire_thermal_bridge: true,
    layers: entry.layers?.map((l) => ({ ...l })),
  };
}
