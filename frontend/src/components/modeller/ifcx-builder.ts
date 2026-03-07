/**
 * Convert between modeller data (ModelRoom, etc.) and IFCX documents.
 *
 * Self-contained — no imports from outside the modeller directory.
 */

import type { ModelRoom, ModelWindow, ModelDoor, Point2D } from "./types";
import type { IfcxDocument } from "./ifcx";
import {
  uuid,
  createIfcxDocument,
  classifyEntry,
  propEntry,
  IFC_CLASS,
  IFCX_NS,
} from "./ifcx";

// ---------------------------------------------------------------------------
// Modeller → IFCX
// ---------------------------------------------------------------------------

export interface ModelToIfcxOptions {
  projectName: string;
  siteName?: string;
  buildingName?: string;
  author: string;
  /** Map of floor index → storey name. */
  storeyNames?: Record<number, string>;
}

/**
 * Build an IFCX document from modeller data.
 */
export function modelToIfcx(
  rooms: ModelRoom[],
  windows: ModelWindow[],
  doors: ModelDoor[],
  opts: ModelToIfcxOptions,
): IfcxDocument {
  const doc = createIfcxDocument({
    id: uuid(),
    author: opts.author,
  });

  // IDs for the spatial hierarchy
  const projectId = uuid();
  const siteId = uuid();
  const buildingId = uuid();

  // Group rooms by floor
  const floors = new Map<number, ModelRoom[]>();
  for (const room of rooms) {
    const list = floors.get(room.floor) ?? [];
    list.push(room);
    floors.set(room.floor, list);
  }

  const storeyIds = new Map<number, string>();
  for (const floor of floors.keys()) {
    storeyIds.set(floor, uuid());
  }

  // --- Spatial hierarchy ---

  // Project → Site
  doc.data.push({
    path: projectId,
    children: { [opts.siteName ?? "Site"]: siteId },
  });
  doc.data.push(classifyEntry(projectId, IFC_CLASS.Project));
  doc.data.push(propEntry(projectId, "Name", opts.projectName));

  // Site → Building
  doc.data.push({
    path: siteId,
    children: { [opts.buildingName ?? "Gebouw"]: buildingId },
  });
  doc.data.push(classifyEntry(siteId, IFC_CLASS.Site));
  doc.data.push(propEntry(siteId, "Name", opts.siteName ?? "Site"));

  // Building → Storeys
  const buildingChildren: Record<string, string> = {};
  for (const [floor, storeyId] of storeyIds) {
    const name = opts.storeyNames?.[floor] ?? `Verdieping ${floor}`;
    buildingChildren[name] = storeyId;

    doc.data.push(classifyEntry(storeyId, IFC_CLASS.BuildingStorey));
    doc.data.push(propEntry(storeyId, "Name", name));
    doc.data.push(propEntry(storeyId, "Elevation", floor * 3000));
  }
  doc.data.push({
    path: buildingId,
    children: buildingChildren,
  });
  doc.data.push(classifyEntry(buildingId, IFC_CLASS.Building));
  doc.data.push(propEntry(buildingId, "Name", opts.buildingName ?? "Gebouw"));

  // --- Rooms as IfcSpace ---
  // Track room UUID mapping for windows/doors
  const roomUuids = new Map<string, string>();

  for (const room of rooms) {
    const roomId = uuid();
    roomUuids.set(room.id, roomId);

    const storeyId = storeyIds.get(room.floor)!;

    // Add room as child of storey
    doc.data.push({
      path: storeyId,
      children: { [room.name]: roomId },
    });

    // Classify as IfcSpace
    doc.data.push(classifyEntry(roomId, IFC_CLASS.Space));
    doc.data.push(propEntry(roomId, "Name", room.name));
    doc.data.push(propEntry(roomId, "LongName", room.id));
    doc.data.push(propEntry(roomId, "Function", room.function));

    // Store polygon geometry as custom attribute
    doc.data.push({
      path: roomId,
      attributes: {
        "modeller::polygon": room.polygon,
        "modeller::height": room.height,
      },
    });

    // Generate mesh from polygon extrusion
    const mesh = polygonToMesh(room.polygon, room.height);
    doc.data.push({
      path: roomId,
      attributes: {
        [IFCX_NS.mesh]: mesh,
      },
    });
  }

  // --- Windows as IfcWindow ---
  for (const win of windows) {
    const winId = uuid();
    const parentRoomId = roomUuids.get(win.roomId);
    if (!parentRoomId) continue;

    doc.data.push({
      path: parentRoomId,
      children: { [`Window_${winId.slice(0, 8)}`]: winId },
    });

    doc.data.push(classifyEntry(winId, IFC_CLASS.Window));
    doc.data.push(propEntry(winId, "OverallWidth", win.width));
    doc.data.push({
      path: winId,
      attributes: {
        "modeller::wallIndex": win.wallIndex,
        "modeller::offset": win.offset,
      },
    });
  }

  // --- Doors as IfcDoor ---
  for (const door of doors) {
    const doorId = uuid();
    const parentRoomId = roomUuids.get(door.roomId);
    if (!parentRoomId) continue;

    doc.data.push({
      path: parentRoomId,
      children: { [`Door_${doorId.slice(0, 8)}`]: doorId },
    });

    doc.data.push(classifyEntry(doorId, IFC_CLASS.Door));
    doc.data.push(propEntry(doorId, "OverallWidth", door.width));
    doc.data.push({
      path: doorId,
      attributes: {
        "modeller::wallIndex": door.wallIndex,
        "modeller::offset": door.offset,
        "modeller::swing": door.swing,
      },
    });
  }

  return doc;
}

// ---------------------------------------------------------------------------
// IFCX → Modeller
// ---------------------------------------------------------------------------

/**
 * Parse an IFCX document back into modeller data.
 */
export function ifcxToModel(doc: IfcxDocument): {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
} {
  // Merge all entries by path
  const entries = new Map<string, {
    children: Record<string, string>;
    inherits: Record<string, string>;
    attributes: Record<string, unknown>;
  }>();

  for (const entry of doc.data) {
    const existing = entries.get(entry.path);
    if (existing) {
      Object.assign(existing.children, entry.children ?? {});
      Object.assign(existing.inherits, entry.inherits ?? {});
      Object.assign(existing.attributes, entry.attributes ?? {});
    } else {
      entries.set(entry.path, {
        children: { ...entry.children },
        inherits: { ...entry.inherits },
        attributes: { ...entry.attributes },
      });
    }
  }

  const rooms: ModelRoom[] = [];
  const windows: ModelWindow[] = [];
  const doors: ModelDoor[] = [];

  // Find storeys to determine floor index
  const storeyPaths = new Map<string, number>();
  let floorIdx = 0;
  for (const [path, entry] of entries) {
    const cls = entry.attributes["bsi::ifc::class"] as { code?: string } | undefined;
    if (cls?.code === "IfcBuildingStorey") {
      const elevation = entry.attributes["bsi::ifc::prop::Elevation"] as number | undefined;
      storeyPaths.set(path, elevation != null ? Math.round(elevation / 3000) : floorIdx);
      floorIdx++;
    }
  }

  // Find which storey each space belongs to
  const pathToFloor = new Map<string, number>();
  for (const [storeyPath, floor] of storeyPaths) {
    const storey = entries.get(storeyPath);
    if (storey) {
      for (const childPath of Object.values(storey.children)) {
        pathToFloor.set(childPath, floor);
      }
    }
  }

  // Parse spaces, windows, doors
  for (const [path, entry] of entries) {
    const cls = entry.attributes["bsi::ifc::class"] as { code?: string } | undefined;
    if (!cls?.code) continue;

    if (cls.code === "IfcSpace") {
      const polygon = entry.attributes["modeller::polygon"] as Point2D[] | undefined;
      const height = entry.attributes["modeller::height"] as number | undefined;
      const name = entry.attributes["bsi::ifc::prop::Name"] as string | undefined;
      const longName = entry.attributes["bsi::ifc::prop::LongName"] as string | undefined;
      const func = entry.attributes["bsi::ifc::prop::Function"] as string | undefined;

      if (polygon) {
        rooms.push({
          id: longName ?? path.slice(0, 8),
          name: name ?? "Ruimte",
          function: func ?? "custom",
          polygon,
          floor: pathToFloor.get(path) ?? 0,
          height: height ?? 2600,
        });

        // Find child windows/doors
        for (const [, childPath] of Object.entries(entry.children)) {
          const child = entries.get(childPath);
          if (!child) continue;
          const childCls = child.attributes["bsi::ifc::class"] as { code?: string } | undefined;

          if (childCls?.code === "IfcWindow") {
            windows.push({
              roomId: longName ?? path.slice(0, 8),
              wallIndex: (child.attributes["modeller::wallIndex"] as number) ?? 0,
              offset: (child.attributes["modeller::offset"] as number) ?? 0,
              width: (child.attributes["bsi::ifc::prop::OverallWidth"] as number) ?? 1000,
            });
          }

          if (childCls?.code === "IfcDoor") {
            doors.push({
              roomId: longName ?? path.slice(0, 8),
              wallIndex: (child.attributes["modeller::wallIndex"] as number) ?? 0,
              offset: (child.attributes["modeller::offset"] as number) ?? 0,
              width: (child.attributes["bsi::ifc::prop::OverallWidth"] as number) ?? 900,
              swing: (child.attributes["modeller::swing"] as "left" | "right") ?? "left",
            });
          }
        }
      }
    }
  }

  return { rooms, windows, doors };
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

interface UsdMesh {
  points: number[];
  faceVertexIndices: number[];
}

/**
 * Extrude a 2D polygon into a 3D mesh (USD format).
 * Y-down screen coords → Y-up world coords for 3D.
 */
function polygonToMesh(polygon: Point2D[], height: number): UsdMesh {
  const n = polygon.length;
  const points: number[] = [];
  const faceVertexIndices: number[] = [];

  // Bottom face points (y=0)
  for (const p of polygon) {
    points.push(p.x, 0, p.y); // x, y(up), z(=screen-y)
  }
  // Top face points (y=height)
  for (const p of polygon) {
    points.push(p.x, height, p.y);
  }

  // Bottom face (reversed winding for outward normals)
  for (let i = n - 2; i >= 0; i--) {
    faceVertexIndices.push(0, i + 1, i);
  }

  // Top face
  for (let i = 0; i < n - 2; i++) {
    faceVertexIndices.push(n, n + i + 1, n + i + 2);
  }

  // Side faces
  for (let i = 0; i < n; i++) {
    const j = (i + 1) % n;
    const b0 = i;
    const b1 = j;
    const t0 = i + n;
    const t1 = j + n;
    faceVertexIndices.push(b0, b1, t1, b0, t1, t0);
  }

  return { points, faceVertexIndices };
}
