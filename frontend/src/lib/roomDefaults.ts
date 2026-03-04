import type { CatalogueEntry } from "./constructionCatalogue";

import type { ConstructionElement, Room } from "../types";

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
    ventilation_rate: 0,
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
    boundary_type: entry.boundaryType,
    material_type: entry.materialType,
    vertical_position: entry.verticalPosition,
    use_forfaitaire_thermal_bridge: true,
  };
}
