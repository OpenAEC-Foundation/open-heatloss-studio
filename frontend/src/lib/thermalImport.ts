/**
 * Thermal Import — types, parsing, and API call for Revit thermal export.
 *
 * Matches the thermal-import.schema.json v1 contract.
 */
import { API_PREFIX } from "./constants";
import { authFetch } from "./backend";
import type { Project } from "../types";
import type { ImportedBoundary } from "../components/modeller/types";

// ---------------------------------------------------------------------------
// Input types (from Revit thermal export JSON)
// ---------------------------------------------------------------------------

export type ThermalRoomType = "heated" | "unheated" | "outside" | "ground" | "water";

export interface ThermalRoom {
  id: string;
  revit_id?: number;
  name: string;
  type: ThermalRoomType;
  level?: string;
  area_m2?: number;
  height_m?: number;
  volume_m3?: number;
  boundary_polygon?: [number, number][];
}

export interface ThermalConstructionLayer {
  material: string;
  thickness_mm: number;
  distance_from_interior_mm?: number;
  type?: "solid" | "air_gap";
  lambda?: number;
}

export interface ThermalConstruction {
  id: string;
  room_a: string;
  room_b: string;
  orientation: "wall" | "floor" | "ceiling" | "roof";
  compass?: "N" | "NE" | "E" | "SE" | "S" | "SW" | "W" | "NW";
  gross_area_m2: number;
  revit_element_id?: number;
  revit_type_name?: string;
  layers?: ThermalConstructionLayer[];
}

export type ThermalOpeningType = "window" | "door" | "curtain_wall";

export interface ThermalOpening {
  id: string;
  construction_id: string;
  type: ThermalOpeningType;
  width_mm: number;
  height_mm: number;
  sill_height_mm?: number;
  u_value?: number;
  revit_element_id?: number;
  revit_type_name?: string;
}

export interface ThermalOpenConnection {
  room_a: string;
  room_b: string;
  area_m2: number;
}

export interface ThermalImportFile {
  version: string;
  source: "revit-eam" | "revit-raycast" | "ifc";
  exported_at: string;
  project_name?: string;
  rooms: ThermalRoom[];
  constructions: ThermalConstruction[];
  openings?: ThermalOpening[];
  open_connections?: ThermalOpenConnection[];
}

// ---------------------------------------------------------------------------
// Backend response types
// ---------------------------------------------------------------------------

export interface ThermalImportConstructionLayer {
  material: string;
  thickness_mm: number;
  /** "solid" or "air_gap" — air gaps are rendered as Spouw in the SfB name. */
  type?: "solid" | "air_gap";
  distance_from_interior_mm?: number;
  lambda?: number;
}

/**
 * One unique construction in the project catalog. The Rust backend groups
 * raw surfaces by layer fingerprint and returns one entry per unique
 * construction. Each `ConstructionElement.catalog_ref` points to one of these.
 */
export interface CatalogEntry {
  /** Catalog ID, format `cat-{n}`. */
  id: string;
  /** SfB-based description, may carry a `_<thickness>mm` collision suffix. */
  description: string;
  /** Layer composition from interior to exterior. */
  layers: ThermalImportConstructionLayer[];
  /** First-encountered Revit type name (debug info). */
  revit_type_name?: string;
  /**
   * Distinct (BoundaryType, Orientation) combinations in which this entry is
   * used. Tuples are serialized as 2-element arrays by serde.
   */
  used_for: [string, string][];
  /** Total area in m² across every surface that uses this entry. */
  total_area_m2: number;
  /** Number of raw surfaces in the source export that use this entry. */
  surface_count: number;
}

export interface ThermalImportRoomPolygon {
  room_id: string;
  polygon: [number, number][];
}

export interface ThermalImportResult {
  project: Project;
  warnings: string[];
  /** Unique constructions, one entry per layer fingerprint. */
  construction_catalog: CatalogEntry[];
  room_polygons: ThermalImportRoomPolygon[];
}

// ---------------------------------------------------------------------------
// Parse thermal import JSON
// ---------------------------------------------------------------------------

/**
 * Parse and validate a thermal import JSON string.
 * Throws on invalid structure.
 */
export function parseThermalImportFile(jsonString: string): ThermalImportFile {
  let data: unknown;
  try {
    data = JSON.parse(jsonString);
  } catch {
    throw new Error("Ongeldig JSON bestand");
  }

  if (typeof data !== "object" || data === null) {
    throw new Error("Ongeldig bestandsformaat");
  }

  const obj = data as Record<string, unknown>;

  if (obj.version !== "1.0") {
    throw new Error(`Onbekende versie: ${String(obj.version)}. Verwacht: 1.0`);
  }

  if (!Array.isArray(obj.rooms) || obj.rooms.length === 0) {
    throw new Error("Verplicht veld 'rooms' ontbreekt of is leeg");
  }

  if (!Array.isArray(obj.constructions)) {
    throw new Error("Verplicht veld 'constructions' ontbreekt");
  }

  const source = obj.source as string;
  if (!["revit-eam", "revit-raycast", "ifc"].includes(source)) {
    throw new Error(`Onbekende bron: ${source}. Verwacht: revit-eam, revit-raycast of ifc`);
  }

  return data as ThermalImportFile;
}

// ---------------------------------------------------------------------------
// API call
// ---------------------------------------------------------------------------

/**
 * POST the thermal import data to the backend for processing.
 * Returns a mapped ISSO 51 project + review data.
 */
export async function importThermal(
  data: ThermalImportFile,
): Promise<ThermalImportResult> {
  const res = await authFetch(`${API_PREFIX}/import/thermal`, {
    method: "POST",
    body: JSON.stringify(data),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error(
      (err as { detail?: string }).detail ?? `Thermal import mislukt (HTTP ${res.status})`,
    );
  }

  return res.json() as Promise<ThermalImportResult>;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Check if a room is a pseudo-room (outside, ground, water). */
export function isPseudoRoom(room: ThermalRoom): boolean {
  return room.type === "outside" || room.type === "ground" || room.type === "water";
}

/** Get a display label for room type. */
export function roomTypeLabel(type: ThermalRoomType): string {
  const labels: Record<ThermalRoomType, string> = {
    heated: "Verwarmd",
    unheated: "Onverwarmd",
    outside: "Buiten",
    ground: "Grond",
    water: "Water",
  };
  return labels[type] ?? type;
}

/** Get a display label for opening type. */
export function openingTypeLabel(type: ThermalOpeningType): string {
  const labels: Record<ThermalOpeningType, string> = {
    window: "Raam",
    door: "Deur",
    curtain_wall: "Vliesgevel",
  };
  return labels[type] ?? type;
}

/** Convert constructions to ImportedBoundary format for the modeller 3D viewer. */
export function toImportedBoundaries(
  constructions: ThermalConstruction[],
  rooms: ThermalRoom[],
): ImportedBoundary[] {
  const roomMap = new Map(rooms.map((r) => [r.id, r]));

  return constructions.map((c) => {
    const adjRoom = roomMap.get(c.room_b);
    let boundaryCondition: ImportedBoundary["boundaryCondition"];

    if (adjRoom) {
      const typeMap: Record<ThermalRoomType, ImportedBoundary["boundaryCondition"]> = {
        outside: "exterior",
        ground: "ground",
        water: "water",
        unheated: "unheated",
        heated: "adjacent",
      };
      boundaryCondition = typeMap[adjRoom.type];
    } else {
      boundaryCondition = "exterior"; // fallback for unknown rooms
    }

    return {
      id: c.id,
      roomId: c.room_a,
      adjacentRoomId: c.room_b,
      orientation: c.orientation,
      boundaryCondition,
      area_m2: c.gross_area_m2,
      compass: c.compass,
    };
  });
}

/**
 * Apply user edits to the backend-mapped project before loading into store.
 * Merges editedRooms (type changes) and editedOpenings (U-value changes)
 * into the project returned by the backend.
 *
 * `catalogUValues` is keyed by `CatalogEntry.id`. Every `ConstructionElement`
 * with a matching `catalog_ref` receives the calculated U-value, so a single
 * edit in the LayerEditor fans out to every room that uses that construction.
 */
export function applyEditsToProject(
  project: Project,
  editedRooms: ThermalRoom[],
  editedOpenings: ThermalOpening[],
  catalogUValues?: Map<string, number>,
): Project {
  // Build lookup of edited room types
  const roomTypeMap = new Map(editedRooms.map((r) => [r.id, r.type]));
  // Build lookup of edited opening U-values
  const openingUMap = new Map(
    editedOpenings
      .filter((o) => o.u_value != null)
      .map((o) => [o.id, o.u_value!]),
  );

  return {
    ...project,
    rooms: project.rooms.map((room) => {
      const editedType = roomTypeMap.get(room.id);

      // Update room function if user changed type from heated↔unheated
      let updatedRoom = room;
      if (editedType === "unheated" && room.function !== "storage") {
        updatedRoom = { ...room, function: "storage" as any };
      }

      // Update U-values on constructions from LayerEditor and openings
      const updatedConstructions = updatedRoom.constructions.map((ce) => {
        // Catalog-entry U-value applies to every element with the same catalog_ref.
        if (ce.catalog_ref) {
          const calcU = catalogUValues?.get(ce.catalog_ref);
          if (calcU != null && calcU > 0) {
            return { ...ce, u_value: calcU };
          }
        }
        // Opening U-value uses the per-element id (openings have catalog_ref == null).
        const openingU = openingUMap.get(ce.id);
        if (openingU != null && openingU > 0) {
          return { ...ce, u_value: openingU };
        }
        return ce;
      });

      return { ...updatedRoom, constructions: updatedConstructions };
    }),
  };
}
