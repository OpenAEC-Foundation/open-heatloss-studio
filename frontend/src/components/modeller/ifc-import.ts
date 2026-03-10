/**
 * IFC importer for the 2D modeller.
 *
 * Reads IFC2x3/IFC4 files via web-ifc, extracts IfcSpace entities,
 * and converts them to ModelRoom[] with 2D polygons.
 *
 * Phase 1: rooms only (IfcSpace → ModelRoom).
 */
import * as WebIfc from "web-ifc";

import type { ModelRoom, Point2D } from "./types";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const IFC_TO_MM = 1000;
const MIN_ROOM_AREA_MM2 = 500_000; // 0.5 m2
const MIN_POLYGON_POINTS = 3;
const FLOOR_HEIGHT_DEFAULT_MM = 2600;

// web-ifc entity type constants
const IFCSPACE = WebIfc.IFCSPACE;
const IFCBUILDINGSTOREY = WebIfc.IFCBUILDINGSTOREY;
const IFCEXTRUDEDAREASOLID = WebIfc.IFCEXTRUDEDAREASOLID;
const IFCARBITRARYCLOSEDPROFILEDEF = WebIfc.IFCARBITRARYCLOSEDPROFILEDEF;
const IFCRECTANGLEPROFILEDEF = WebIfc.IFCRECTANGLEPROFILEDEF;
const IFCPOLYLINE = WebIfc.IFCPOLYLINE;
const IFCFACETEDBREP = WebIfc.IFCFACETEDBREP;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

export interface IfcImportResult {
  rooms: Omit<ModelRoom, "id">[];
  warnings: { spaceName: string; message: string }[];
  stats: { spacesFound: number; spacesImported: number; spacesSkipped: number };
}

// ---------------------------------------------------------------------------
// IfcAPI singleton with lazy init
// ---------------------------------------------------------------------------

let apiInstance: WebIfc.IfcAPI | null = null;

async function getIfcApi(): Promise<WebIfc.IfcAPI> {
  if (apiInstance) return apiInstance;

  const api = new WebIfc.IfcAPI();
  api.SetWasmPath("/wasm/");
  await api.Init();
  apiInstance = api;
  return api;
}

// ---------------------------------------------------------------------------
// Keyword → room function mapping
// ---------------------------------------------------------------------------

const FUNCTION_KEYWORDS: [RegExp, string][] = [
  [/woonkamer|huiskamer|living|zitkamer/i, "woonkamer"],
  [/slaapkamer|bedroom/i, "slaapkamer"],
  [/keuken|kitchen/i, "keuken"],
  [/badkamer|bathroom/i, "badkamer"],
  [/toilet|wc/i, "toilet"],
  [/hal|gang|entree|corridor|overloop/i, "hal"],
  [/berging|storage|opslag/i, "berging"],
  [/garage/i, "garage"],
  [/kantoor|office|studeerkamer|werkruimte/i, "kantoor"],
  [/wasruimte|laundry|bijkeuken/i, "bijkeuken"],
  [/zolder|attic/i, "zolder"],
  [/kelder|basement|souterrain/i, "kelder"],
];

function matchRoomFunction(name: string): string {
  for (const [pattern, func] of FUNCTION_KEYWORDS) {
    if (pattern.test(name)) return func;
  }
  return "custom";
}

// ---------------------------------------------------------------------------
// Coordinate transform: IFC (meters, Z-up) → Modeller (mm, Y-down screen)
// ---------------------------------------------------------------------------

function ifcToModeller(x: number, y: number): Point2D {
  return { x: x * IFC_TO_MM, y: -y * IFC_TO_MM };
}

// ---------------------------------------------------------------------------
// Polygon area (2D, signed) for filtering degenerate spaces
// ---------------------------------------------------------------------------

function polygonAreaMm2(polygon: Point2D[]): number {
  let area = 0;
  for (let i = 0; i < polygon.length; i++) {
    const j = (i + 1) % polygon.length;
    const pi = polygon[i]!;
    const pj = polygon[j]!;
    area += pi.x * pj.y;
    area -= pj.x * pi.y;
  }
  return Math.abs(area / 2);
}

// ---------------------------------------------------------------------------
// Spatial structure: extract storey elevations for floor index assignment
// ---------------------------------------------------------------------------

interface StoreyInfo {
  expressId: number;
  elevation: number;
  floorIndex: number;
  childSpaceIds: Set<number>;
}

function extractStoreys(
  api: WebIfc.IfcAPI,
  modelId: number,
): Map<number, StoreyInfo> {
  const storeyMap = new Map<number, StoreyInfo>();
  const storeyIds = api.GetLineIDsWithType(modelId, IFCBUILDINGSTOREY);
  const storeys: { expressId: number; elevation: number }[] = [];

  for (let i = 0; i < storeyIds.size(); i++) {
    const id = storeyIds.get(i);
    const props = api.GetLine(modelId, id);
    const elevation = props?.Elevation?.value ?? 0;
    storeys.push({ expressId: id, elevation: Number(elevation) });
  }

  // Sort by elevation ascending → floor index 0, 1, 2...
  storeys.sort((a, b) => a.elevation - b.elevation);

  for (let idx = 0; idx < storeys.length; idx++) {
    const s = storeys[idx]!;
    storeyMap.set(s.expressId, {
      expressId: s.expressId,
      elevation: s.elevation,
      floorIndex: idx,
      childSpaceIds: new Set(),
    });
  }

  return storeyMap;
}

interface FloorAssignment {
  floorIndex: number;
  /** Storey elevation in meters (IFC units). */
  elevationMeters: number | undefined;
}

function findFloorForSpace(
  api: WebIfc.IfcAPI,
  modelId: number,
  spaceId: number,
  storeyMap: Map<number, StoreyInfo>,
): FloorAssignment {
  // Walk spatial containment: IfcSpace is typically contained in IfcBuildingStorey
  // via IfcRelContainedInSpatialStructure or IfcRelAggregates.
  // We check the space's ObjectPlacement decomposes chain.
  try {
    const spaceProps = api.GetLine(modelId, spaceId);
    // Check IfcRelAggregates / Decomposes
    const decomposes = spaceProps?.Decomposes;
    if (decomposes) {
      const rels = Array.isArray(decomposes) ? decomposes : [decomposes];
      for (const rel of rels) {
        const relObj = rel?.value != null ? api.GetLine(modelId, rel.value) : null;
        if (relObj?.RelatingObject?.value != null) {
          const parentId = relObj.RelatingObject.value;
          if (storeyMap.has(parentId)) {
            const storey = storeyMap.get(parentId)!;
            return { floorIndex: storey.floorIndex, elevationMeters: storey.elevation };
          }
        }
      }
    }
  } catch {
    // Spatial lookup failed — fall through to fallback
  }

  // Fallback: floor 0, no elevation
  return { floorIndex: 0, elevationMeters: undefined };
}

// ---------------------------------------------------------------------------
// Strategy 1: Profile extraction (IfcExtrudedAreaSolid)
// ---------------------------------------------------------------------------

interface ExtractionResult {
  polygon: Point2D[];
  height: number;
}

function tryExtractProfile(
  api: WebIfc.IfcAPI,
  modelId: number,
  spaceId: number,
): ExtractionResult | null {
  try {
    const space = api.GetLine(modelId, spaceId);
    const representation = space?.Representation;
    if (!representation) return null;

    const reps = representation.Representations ?? representation.value?.Representations;
    const repList = Array.isArray(reps) ? reps : [];

    for (const repRef of repList) {
      const rep = repRef?.value != null ? api.GetLine(modelId, repRef.value) : repRef;
      const items = rep?.Items ?? [];
      const itemList = Array.isArray(items) ? items : [];

      for (const itemRef of itemList) {
        const item = itemRef?.value != null ? api.GetLine(modelId, itemRef.value) : itemRef;
        if (!item) continue;

        // Check if this is an IfcExtrudedAreaSolid
        const itemType = item.constructor?.name ?? item.type;
        const isExtrusion = item.type === IFCEXTRUDEDAREASOLID ||
          itemType === "IfcExtrudedAreaSolid" ||
          item.Depth != null;

        if (!isExtrusion) continue;

        const depth = Number(item.Depth?.value ?? item.Depth ?? 0);
        const height = depth > 0 ? depth * IFC_TO_MM : FLOOR_HEIGHT_DEFAULT_MM;

        const sweptArea = item.SweptArea?.value != null
          ? api.GetLine(modelId, item.SweptArea.value)
          : item.SweptArea;

        if (!sweptArea) continue;

        const polygon = extractPolygonFromProfile(api, modelId, sweptArea);
        if (!polygon || polygon.length < MIN_POLYGON_POINTS) continue;

        // Apply placement transform
        const transform = getPlacementTransform(api, modelId, spaceId);
        const transformed = polygon.map((p) => applyTransform2D(p, transform));

        return { polygon: transformed, height };
      }
    }
  } catch {
    // Profile extraction failed
  }
  return null;
}

function extractPolygonFromProfile(
  api: WebIfc.IfcAPI,
  modelId: number,
  profile: Record<string, unknown>,
): Point2D[] | null {
  const profileType = (profile as { type?: number }).type;

  // IfcArbitraryClosedProfileDef → OuterCurve → IfcPolyline
  if (profileType === IFCARBITRARYCLOSEDPROFILEDEF || (profile as { OuterCurve?: unknown }).OuterCurve) {
    const curveRef = (profile as { OuterCurve?: { value?: number } }).OuterCurve;
    const curve = curveRef?.value != null ? api.GetLine(modelId, curveRef.value) : curveRef;
    if (!curve) return null;

    return extractPointsFromCurve(api, modelId, curve as Record<string, unknown>);
  }

  // IfcRectangleProfileDef → generate 4-point rectangle
  if (profileType === IFCRECTANGLEPROFILEDEF || (profile as { XDim?: unknown }).XDim) {
    const xDim = Number((profile as { XDim?: { value?: number } }).XDim?.value ?? (profile as { XDim?: number }).XDim ?? 0);
    const yDim = Number((profile as { YDim?: { value?: number } }).YDim?.value ?? (profile as { YDim?: number }).YDim ?? 0);
    if (xDim <= 0 || yDim <= 0) return null;

    const hw = xDim / 2;
    const hh = yDim / 2;
    return [
      ifcToModeller(-hw, -hh),
      ifcToModeller(hw, -hh),
      ifcToModeller(hw, hh),
      ifcToModeller(-hw, hh),
    ];
  }

  return null;
}

function extractPointsFromCurve(
  api: WebIfc.IfcAPI,
  modelId: number,
  curve: Record<string, unknown>,
): Point2D[] | null {
  const curveType = (curve as { type?: number }).type;

  // IfcPolyline → Points
  if (curveType === IFCPOLYLINE || (curve as { Points?: unknown }).Points) {
    const pointRefs = (curve as { Points?: unknown[] }).Points;
    if (!Array.isArray(pointRefs)) return null;

    const points: Point2D[] = [];
    for (const pRef of pointRefs) {
      const p = (pRef as { value?: number })?.value != null
        ? api.GetLine(modelId, (pRef as { value: number }).value)
        : pRef;

      const coords = (p as { Coordinates?: unknown[] })?.Coordinates;
      if (!Array.isArray(coords) || coords.length < 2) continue;

      const x = Number((coords[0] as { value?: number })?.value ?? coords[0]);
      const y = Number((coords[1] as { value?: number })?.value ?? coords[1]);

      points.push(ifcToModeller(x, y));
    }

    // Remove duplicate closing point if present
    if (points.length > 1) {
      const first = points[0]!;
      const last = points[points.length - 1]!;
      if (Math.abs(first.x - last.x) < 0.1 && Math.abs(first.y - last.y) < 0.1) {
        points.pop();
      }
    }

    return points.length >= MIN_POLYGON_POINTS ? points : null;
  }

  return null;
}

// ---------------------------------------------------------------------------
// Strategy 2: Brep extraction (IfcFacetedBrep)
// ---------------------------------------------------------------------------

function tryExtractBrep(
  api: WebIfc.IfcAPI,
  modelId: number,
  spaceId: number,
): ExtractionResult | null {
  try {
    const space = api.GetLine(modelId, spaceId);
    const representation = space?.Representation;
    if (!representation) return null;

    const reps = representation.Representations ?? representation.value?.Representations;
    const repList = Array.isArray(reps) ? reps : [];

    for (const repRef of repList) {
      const rep = repRef?.value != null ? api.GetLine(modelId, repRef.value) : repRef;
      const items = rep?.Items ?? [];
      const itemList = Array.isArray(items) ? items : [];

      for (const itemRef of itemList) {
        const item = itemRef?.value != null ? api.GetLine(modelId, itemRef.value) : itemRef;
        if (!item || item.type !== IFCFACETEDBREP) continue;

        // Get the ClosedShell → CfsFaces
        const shellRef = item.Outer;
        const shell = shellRef?.value != null ? api.GetLine(modelId, shellRef.value) : shellRef;
        if (!shell) continue;

        const faces = shell.CfsFaces ?? [];
        const faceList = Array.isArray(faces) ? faces : [];

        // Find horizontal faces, group by Z, pick the floor (lowest Z)
        const facesWithZ: { z: number; points: { x: number; y: number; z: number }[] }[] = [];

        for (const faceRef of faceList) {
          const face = faceRef?.value != null ? api.GetLine(modelId, faceRef.value) : faceRef;
          const bounds = face?.Bounds ?? [];
          const boundList = Array.isArray(bounds) ? bounds : [];

          for (const boundRef of boundList) {
            const bound = boundRef?.value != null ? api.GetLine(modelId, boundRef.value) : boundRef;
            const loopRef = bound?.Bound;
            const loop = loopRef?.value != null ? api.GetLine(modelId, loopRef.value) : loopRef;
            if (!loop) continue;

            const polyPoints = loop.Polygon ?? [];
            const pointList = Array.isArray(polyPoints) ? polyPoints : [];
            const pts: { x: number; y: number; z: number }[] = [];

            for (const ptRef of pointList) {
              const pt = ptRef?.value != null ? api.GetLine(modelId, ptRef.value) : ptRef;
              const coords = pt?.Coordinates ?? [];
              const coordList = Array.isArray(coords) ? coords : [];
              if (coordList.length < 3) continue;

              pts.push({
                x: Number(coordList[0]?.value ?? coordList[0]),
                y: Number(coordList[1]?.value ?? coordList[1]),
                z: Number(coordList[2]?.value ?? coordList[2]),
              });
            }

            if (pts.length < MIN_POLYGON_POINTS) continue;

            // Check if face is horizontal (all Z values similar)
            const avgZ = pts.reduce((sum, p) => sum + p.z, 0) / pts.length;
            const isHorizontal = pts.every((p) => Math.abs(p.z - avgZ) < 0.01);
            if (isHorizontal) {
              facesWithZ.push({ z: avgZ, points: pts });
            }
          }
        }

        if (facesWithZ.length === 0) continue;

        // Sort by Z, pick lowest (floor) and highest (ceiling) for height
        facesWithZ.sort((a, b) => a.z - b.z);
        const floorFace = facesWithZ[0]!;
        const ceilingFace = facesWithZ[facesWithZ.length - 1]!;
        const height = (ceilingFace.z - floorFace.z) * IFC_TO_MM;

        // Apply placement transform and convert to 2D
        const transform = getPlacementTransform(api, modelId, spaceId);
        const polygon = floorFace.points.map((p) => {
          const transformed = applyTransform2D(
            ifcToModeller(p.x, p.y),
            transform,
          );
          return transformed;
        });

        if (polygon.length >= MIN_POLYGON_POINTS) {
          return {
            polygon,
            height: height > 0 ? height : FLOOR_HEIGHT_DEFAULT_MM,
          };
        }
      }
    }
  } catch {
    // Brep extraction failed
  }
  return null;
}

// ---------------------------------------------------------------------------
// Strategy 3: Mesh fallback via GetFlatMesh
// ---------------------------------------------------------------------------

function tryExtractMesh(
  api: WebIfc.IfcAPI,
  modelId: number,
  spaceId: number,
): ExtractionResult | null {
  try {
    const flatMesh = api.GetFlatMesh(modelId, spaceId);
    if (!flatMesh || flatMesh.geometries.size() === 0) return null;

    // Collect all vertices from all geometries
    const allVertices: { x: number; y: number; z: number }[] = [];

    for (let g = 0; g < flatMesh.geometries.size(); g++) {
      const geom = flatMesh.geometries.get(g);
      const meshData = api.GetGeometry(modelId, geom.geometryExpressID);
      const vertexData = api.GetVertexArray(
        meshData.GetVertexData(),
        meshData.GetVertexDataSize(),
      );

      // vertex data is: x, y, z, nx, ny, nz (6 floats per vertex)
      const stride = 6;
      for (let v = 0; v + 2 < vertexData.length; v += stride) {
        allVertices.push({
          x: vertexData[v] ?? 0,
          y: vertexData[v + 1] ?? 0,
          z: vertexData[v + 2] ?? 0,
        });
      }
    }

    if (allVertices.length < MIN_POLYGON_POINTS) return null;

    // Find min/max Z for height
    let minZ = Infinity;
    let maxZ = -Infinity;
    for (const v of allVertices) {
      if (v.z < minZ) minZ = v.z;
      if (v.z > maxZ) maxZ = v.z;
    }
    const height = (maxZ - minZ) * IFC_TO_MM;

    // Extract bottom-face vertices (Z near minZ)
    const zTolerance = 0.05; // 5cm
    const bottomVerts = allVertices.filter(
      (v) => Math.abs(v.z - minZ) < zTolerance,
    );

    if (bottomVerts.length < MIN_POLYGON_POINTS) return null;

    // Compute convex hull of bottom vertices in 2D
    const points2D = bottomVerts.map((v) => ifcToModeller(v.x, v.y));
    const hull = convexHull2D(points2D);

    if (hull.length >= MIN_POLYGON_POINTS) {
      return {
        polygon: hull,
        height: height > 0 ? height : FLOOR_HEIGHT_DEFAULT_MM,
      };
    }
  } catch {
    // Mesh extraction failed
  }
  return null;
}

// ---------------------------------------------------------------------------
// Convex hull (Andrew's monotone chain)
// ---------------------------------------------------------------------------

function convexHull2D(points: Point2D[]): Point2D[] {
  const pts = [...points].sort((a, b) => a.x - b.x || a.y - b.y);
  if (pts.length <= 2) return pts;

  // Remove exact duplicates
  const unique: Point2D[] = [pts[0]!];
  for (let i = 1; i < pts.length; i++) {
    const cur = pts[i]!;
    const prev = pts[i - 1]!;
    if (cur.x !== prev.x || cur.y !== prev.y) {
      unique.push(cur);
    }
  }
  if (unique.length < MIN_POLYGON_POINTS) return unique;

  const cross = (o: Point2D, a: Point2D, b: Point2D) =>
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

  // Lower hull
  const lower: Point2D[] = [];
  for (const p of unique) {
    while (lower.length >= 2 && cross(lower[lower.length - 2]!, lower[lower.length - 1]!, p) <= 0) {
      lower.pop();
    }
    lower.push(p);
  }

  // Upper hull
  const upper: Point2D[] = [];
  for (let i = unique.length - 1; i >= 0; i--) {
    const p = unique[i]!;
    while (upper.length >= 2 && cross(upper[upper.length - 2]!, upper[upper.length - 1]!, p) <= 0) {
      upper.pop();
    }
    upper.push(p);
  }

  // Remove last point of each half because it's repeated
  lower.pop();
  upper.pop();

  return [...lower, ...upper];
}

// ---------------------------------------------------------------------------
// Placement transform (simplified 2D)
// ---------------------------------------------------------------------------

interface Transform2D {
  tx: number;
  ty: number;
  cos: number;
  sin: number;
}

const IDENTITY_TRANSFORM: Transform2D = { tx: 0, ty: 0, cos: 1, sin: 0 };

function getPlacementTransform(
  api: WebIfc.IfcAPI,
  modelId: number,
  expressId: number,
): Transform2D {
  try {
    const matrix = api.GetCoordinationMatrix(modelId);
    // The coordination matrix is a flat 4x4 matrix (16 values).
    // We only need the 2D part: translation and rotation from XY plane.
    if (matrix && matrix.length >= 16) {
      return {
        tx: (matrix[12] ?? 0) * IFC_TO_MM,
        ty: -(matrix[13] ?? 0) * IFC_TO_MM, // flip Y
        cos: matrix[0] ?? 1,
        sin: -(matrix[1] ?? 0), // flip for Y-down
      };
    }
  } catch {
    // Fall through to identity
  }

  // Try to get the object's own placement via properties
  try {
    const obj = api.GetLine(modelId, expressId);
    const placement = obj?.ObjectPlacement;
    if (placement?.value != null) {
      const placementObj = api.GetLine(modelId, placement.value);
      const relPlacement = placementObj?.RelativePlacement;
      if (relPlacement?.value != null) {
        const axisPlacement = api.GetLine(modelId, relPlacement.value);
        const locationRef = axisPlacement?.Location;
        if (locationRef?.value != null) {
          const location = api.GetLine(modelId, locationRef.value);
          const coords = location?.Coordinates;
          if (Array.isArray(coords) && coords.length >= 2) {
            const x = Number(coords[0]?.value ?? coords[0]);
            const y = Number(coords[1]?.value ?? coords[1]);
            return {
              tx: x * IFC_TO_MM,
              ty: -y * IFC_TO_MM,
              cos: 1,
              sin: 0,
            };
          }
        }
      }
    }
  } catch {
    // Fall through
  }

  return IDENTITY_TRANSFORM;
}

function applyTransform2D(point: Point2D, t: Transform2D): Point2D {
  if (t === IDENTITY_TRANSFORM) return point;
  return {
    x: point.x * t.cos - point.y * t.sin + t.tx,
    y: point.x * t.sin + point.y * t.cos + t.ty,
  };
}

// ---------------------------------------------------------------------------
// Space name extraction
// ---------------------------------------------------------------------------

function getSpaceName(
  api: WebIfc.IfcAPI,
  modelId: number,
  spaceId: number,
): string {
  try {
    const space = api.GetLine(modelId, spaceId);
    const longName = space?.LongName?.value ?? space?.LongName;
    const name = space?.Name?.value ?? space?.Name;
    return String(longName || name || `Ruimte ${spaceId}`);
  } catch {
    return `Ruimte ${spaceId}`;
  }
}

// ---------------------------------------------------------------------------
// Main import function
// ---------------------------------------------------------------------------

export async function importIfcFile(file: File): Promise<IfcImportResult> {
  const api = await getIfcApi();

  const buffer = await file.arrayBuffer();
  const data = new Uint8Array(buffer);

  const modelId = api.OpenModel(data, {
    COORDINATE_TO_ORIGIN: true,
  });

  const result: IfcImportResult = {
    rooms: [],
    warnings: [],
    stats: { spacesFound: 0, spacesImported: 0, spacesSkipped: 0 },
  };

  try {
    // Extract storey structure
    const storeyMap = extractStoreys(api, modelId);

    // Find all IfcSpace entities
    const spaceIds = api.GetLineIDsWithType(modelId, IFCSPACE);
    result.stats.spacesFound = spaceIds.size();

    for (let i = 0; i < spaceIds.size(); i++) {
      const spaceId = spaceIds.get(i);
      const spaceName = getSpaceName(api, modelId, spaceId);

      // Try extraction strategies in order
      let extracted: ExtractionResult | null = null;

      extracted = tryExtractProfile(api, modelId, spaceId);

      if (!extracted) {
        extracted = tryExtractBrep(api, modelId, spaceId);
      }

      if (!extracted) {
        extracted = tryExtractMesh(api, modelId, spaceId);
      }

      if (!extracted) {
        result.warnings.push({
          spaceName,
          message: "Geen geometrie gevonden",
        });
        result.stats.spacesSkipped++;
        continue;
      }

      // Validate polygon
      if (extracted.polygon.length < MIN_POLYGON_POINTS) {
        result.warnings.push({
          spaceName,
          message: `Te weinig punten (${extracted.polygon.length})`,
        });
        result.stats.spacesSkipped++;
        continue;
      }

      const area = polygonAreaMm2(extracted.polygon);
      if (area < MIN_ROOM_AREA_MM2) {
        result.warnings.push({
          spaceName,
          message: `Te klein oppervlak (${(area / 1_000_000).toFixed(2)} m2)`,
        });
        result.stats.spacesSkipped++;
        continue;
      }

      // Determine floor and elevation
      const { floorIndex, elevationMeters } = findFloorForSpace(api, modelId, spaceId, storeyMap);
      const elevation = elevationMeters !== undefined
        ? Math.round(elevationMeters * IFC_TO_MM)
        : undefined;

      // Build room
      const roomFunction = matchRoomFunction(spaceName);
      const room: Omit<ModelRoom, "id"> = {
        name: spaceName,
        function: roomFunction,
        polygon: extracted.polygon,
        floor: floorIndex,
        height: Math.round(extracted.height),
      };
      if (elevation !== undefined) {
        room.elevation = elevation;
      }
      result.rooms.push(room);
      result.stats.spacesImported++;
    }
  } finally {
    api.CloseModel(modelId);
  }

  return result;
}
