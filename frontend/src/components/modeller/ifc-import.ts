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

const DEFAULT_UNIT_TO_MM = 1000; // fallback: assume meters
const MIN_ROOM_AREA_MM2 = 500_000; // 0.5 m2
const MIN_POLYGON_POINTS = 3;
const FLOOR_HEIGHT_DEFAULT_MM = 2600;

// web-ifc entity type constants
const IFCSPACE = WebIfc.IFCSPACE;
const IFCBUILDINGSTOREY = WebIfc.IFCBUILDINGSTOREY;
const IFCPROJECT = WebIfc.IFCPROJECT;
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
// IFC length unit detection
// ---------------------------------------------------------------------------

/** SI prefix → multiplier to get base unit (meters). */
const SI_PREFIX_FACTOR: Record<string, number> = {
  ".EXA.": 1e18,  ".PETA.": 1e15, ".TERA.": 1e12, ".GIGA.": 1e9,
  ".MEGA.": 1e6,  ".KILO.": 1e3,  ".HECTO.": 1e2, ".DECA.": 1e1,
  ".DECI.": 1e-1, ".CENTI.": 1e-2, ".MILLI.": 1e-3, ".MICRO.": 1e-6,
  ".NANO.": 1e-9, ".PICO.": 1e-12,
};

/**
 * Detect the IFC file's length unit and return the conversion factor to mm.
 * Falls back to 1000 (assumes meters) when detection fails.
 */
function detectUnitToMm(api: WebIfc.IfcAPI, modelId: number): number {
  try {
    const projectIds = api.GetLineIDsWithType(modelId, IFCPROJECT);
    if (projectIds.size() === 0) return DEFAULT_UNIT_TO_MM;

    const project = api.GetLine(modelId, projectIds.get(0));
    const unitsCtxRef = project?.UnitsInContext;
    const unitsAssignment = unitsCtxRef?.value != null
      ? api.GetLine(modelId, unitsCtxRef.value)
      : unitsCtxRef;

    const units = unitsAssignment?.Units;
    if (!Array.isArray(units)) return DEFAULT_UNIT_TO_MM;

    for (const unitRef of units) {
      const unit = unitRef?.value != null
        ? api.GetLine(modelId, unitRef.value)
        : unitRef;
      if (!unit) continue;

      const unitType = String(unit.UnitType?.value ?? "");
      if (!unitType.includes("LENGTHUNIT")) continue;

      // IfcSIUnit: check prefix to determine scale
      const prefix = String(unit.Prefix?.value ?? "");
      if (prefix && prefix !== "undefined" && prefix !== "null") {
        const key = prefix.startsWith(".") ? prefix : `.${prefix}.`;
        const factor = SI_PREFIX_FACTOR[key.toUpperCase()];
        if (factor !== undefined) {
          // prefix gives base-unit fraction → meters * 1000 = mm
          return factor * 1000;
        }
      }

      // IfcConversionBasedUnit: check ConversionFactor
      const convFactor = unit.ConversionFactor;
      if (convFactor?.value != null) {
        const measure = api.GetLine(modelId, convFactor.value);
        const val = Number(
          measure?.ValueComponent?.value ?? measure?.ValueComponent ?? 1,
        );
        // val is the conversion to SI meters
        return val * 1000;
      }

      // No prefix, no conversion → base SI = meters
      return 1000;
    }
  } catch {
    // Detection failed
  }
  return DEFAULT_UNIT_TO_MM;
}

// ---------------------------------------------------------------------------
// Coordinate transform: IFC → Modeller (mm, Y-down screen)
// ---------------------------------------------------------------------------

function ifcToModeller(x: number, y: number, unitToMm: number): Point2D {
  return { x: x * unitToMm, y: -y * unitToMm };
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
  /** Floor elevation in mm (from mesh Y-axis). Used for storey matching. */
  floorElevationMm?: number;
}

function tryExtractProfile(
  api: WebIfc.IfcAPI,
  modelId: number,
  spaceId: number,
  unitToMm: number,
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
        const height = depth > 0 ? depth * unitToMm : FLOOR_HEIGHT_DEFAULT_MM;

        const sweptArea = item.SweptArea?.value != null
          ? api.GetLine(modelId, item.SweptArea.value)
          : item.SweptArea;

        if (!sweptArea) continue;

        const polygon = extractPolygonFromProfile(api, modelId, sweptArea, unitToMm);
        if (!polygon || polygon.length < MIN_POLYGON_POINTS) continue;

        // Apply extrusion's own Position (profile → representation space)
        let extrusionPos = IDENTITY_TRANSFORM;
        const posRef = item.Position;
        if (posRef?.value != null) {
          extrusionPos = extractAxisPlacement2D(
            api, modelId, posRef.value, unitToMm,
          );
        }

        // Apply space's ObjectPlacement chain (representation → global)
        const spaceTransform = getPlacementTransform(
          api, modelId, spaceId, unitToMm,
        );

        const combined = composeTransforms(spaceTransform, extrusionPos);
        const transformed = polygon.map((p) => applyTransform2D(p, combined));

        return { polygon: transformed, height };
      }
    }
  } catch (err) {
    console.error(`[IFC-DBG] tryExtractProfile CRASHED for space #${spaceId}:`, err);
  }
  return null;
}

function extractPolygonFromProfile(
  api: WebIfc.IfcAPI,
  modelId: number,
  profile: Record<string, unknown>,
  unitToMm: number,
): Point2D[] | null {
  const profileType = (profile as { type?: number }).type;

  // IfcArbitraryClosedProfileDef → OuterCurve → IfcPolyline
  if (profileType === IFCARBITRARYCLOSEDPROFILEDEF || (profile as { OuterCurve?: unknown }).OuterCurve) {
    const curveRef = (profile as { OuterCurve?: { value?: number } }).OuterCurve;
    const curve = curveRef?.value != null ? api.GetLine(modelId, curveRef.value) : curveRef;
    if (!curve) return null;

    return extractPointsFromCurve(api, modelId, curve as Record<string, unknown>, unitToMm);
  }

  // IfcRectangleProfileDef → generate 4-point rectangle
  if (profileType === IFCRECTANGLEPROFILEDEF || (profile as { XDim?: unknown }).XDim) {
    const xDim = Number((profile as { XDim?: { value?: number } }).XDim?.value ?? (profile as { XDim?: number }).XDim ?? 0);
    const yDim = Number((profile as { YDim?: { value?: number } }).YDim?.value ?? (profile as { YDim?: number }).YDim ?? 0);
    if (xDim <= 0 || yDim <= 0) return null;

    const hw = xDim / 2;
    const hh = yDim / 2;
    return [
      ifcToModeller(-hw, -hh, unitToMm),
      ifcToModeller(hw, -hh, unitToMm),
      ifcToModeller(hw, hh, unitToMm),
      ifcToModeller(-hw, hh, unitToMm),
    ];
  }

  return null;
}

function extractPointsFromCurve(
  api: WebIfc.IfcAPI,
  modelId: number,
  curve: Record<string, unknown>,
  unitToMm: number,
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

      points.push(ifcToModeller(x, y, unitToMm));
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
  unitToMm: number,
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
        const height = (ceilingFace.z - floorFace.z) * unitToMm;

        // Apply placement transform and convert to 2D
        const transform = getPlacementTransform(api, modelId, spaceId, unitToMm);
        const polygon = floorFace.points.map((p) => {
          const transformed = applyTransform2D(
            ifcToModeller(p.x, p.y, unitToMm),
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
  } catch (err) {
    console.error(`[IFC-DBG] tryExtractBrep CRASHED for space #${spaceId}:`, err);
  }
  return null;
}

// ---------------------------------------------------------------------------
// Strategy 3: Mesh extraction via GetFlatMesh (with flatTransformation)
//
// GetFlatMesh returns geometry with a per-geometry 4x4 transform matrix
// (flatTransformation) that places vertices in global model space.
// This is the most reliable strategy for correct room positioning.
// ---------------------------------------------------------------------------

const VERTEX_STRIDE = 6; // x, y, z, nx, ny, nz per vertex
/** 50mm tolerance for "same Z level", expressed in file units. */
const Z_TOLERANCE_MM = 50;
/**
 * GetFlatMesh always returns vertices in meters (web-ifc normalizes
 * internally), regardless of the IFC file's declared length unit.
 */
const MESH_UNIT_TO_MM = 1000;

/** Position key for deduplication — rounds to ~1mm in file units. */
function posKey(x: number, y: number, scale: number): string {
  const s = Math.max(100, scale);
  return `${Math.round(x * s)}_${Math.round(y * s)}`;
}

function tryExtractMesh(
  api: WebIfc.IfcAPI,
  modelId: number,
  spaceId: number,
): ExtractionResult | null {
  try {
    const flatMesh = api.GetFlatMesh(modelId, spaceId);
    if (!flatMesh || flatMesh.geometries.size() === 0) return null;

    // Collect all vertices and triangle indices, applying flatTransformation
    const allVertices: { x: number; y: number; z: number }[] = [];
    const allIndices: number[] = [];

    for (let g = 0; g < flatMesh.geometries.size(); g++) {
      const geom = flatMesh.geometries.get(g);
      const meshData = api.GetGeometry(modelId, geom.geometryExpressID);

      const vertexData = api.GetVertexArray(
        meshData.GetVertexData(),
        meshData.GetVertexDataSize(),
      );
      const indexData = api.GetIndexArray(
        meshData.GetIndexData(),
        meshData.GetIndexDataSize(),
      );

      // 4x4 column-major transformation matrix (local → global)
      const m = geom.flatTransformation;
      const baseIndex = allVertices.length;

      for (let v = 0; v + 2 < vertexData.length; v += VERTEX_STRIDE) {
        const lx = vertexData[v]!;
        const ly = vertexData[v + 1]!;
        const lz = vertexData[v + 2]!;
        // Apply 4x4 column-major matrix: M * [lx, ly, lz, 1]
        allVertices.push({
          x: m[0]! * lx + m[4]! * ly + m[8]! * lz + m[12]!,
          y: m[1]! * lx + m[5]! * ly + m[9]! * lz + m[13]!,
          z: m[2]! * lx + m[6]! * ly + m[10]! * lz + m[14]!,
        });
      }

      // Offset indices to global vertex array
      for (let i = 0; i < indexData.length; i++) {
        allIndices.push(indexData[i]! + baseIndex);
      }

      meshData.delete();
    }

    if (allVertices.length < MIN_POLYGON_POINTS) return null;

    // web-ifc GetFlatMesh uses Y-up (WebGL convention): X,Z = floor plan, Y = height
    let minY = Infinity;
    let maxY = -Infinity;
    for (const v of allVertices) {
      if (v.y < minY) minY = v.y;
      if (v.y > maxY) maxY = v.y;
    }
    const height = (maxY - minY) * MESH_UNIT_TO_MM;
    const floorElevationMm = Math.round(minY * MESH_UNIT_TO_MM);
    const yTol = Z_TOLERANCE_MM / MESH_UNIT_TO_MM; // 0.05m = 50mm

    // Try to extract accurate floor polygon outline from mesh triangles
    const outline = extractFloorOutlineYUp(
      allVertices, allIndices, minY, MESH_UNIT_TO_MM, yTol,
    );
    if (outline && outline.length >= MIN_POLYGON_POINTS) {
      return {
        polygon: outline,
        height: height > 0 ? height : FLOOR_HEIGHT_DEFAULT_MM,
        floorElevationMm,
      };
    }

    // Fallback: convex hull of bottom vertices (Y = vertical)
    const bottomVerts = allVertices.filter(
      (v) => Math.abs(v.y - minY) < yTol,
    );
    if (bottomVerts.length < MIN_POLYGON_POINTS) return null;

    // Project to XZ plane (floor plan), convert to mm
    const points2D = bottomVerts.map((v) => ({
      x: v.x * MESH_UNIT_TO_MM,
      y: -v.z * MESH_UNIT_TO_MM,
    }));
    const hull = convexHull2D(points2D);

    if (hull.length >= MIN_POLYGON_POINTS) {
      return {
        polygon: hull,
        height: height > 0 ? height : FLOOR_HEIGHT_DEFAULT_MM,
        floorElevationMm,
      };
    }
  } catch (err) {
    console.error(`[IFC-DBG] tryExtractMesh CRASHED for space #${spaceId}:`, err);
  }
  return null;
}

// ---------------------------------------------------------------------------
// Floor polygon outline from mesh boundary edges
// ---------------------------------------------------------------------------

/**
 * Extract the floor polygon outline from triangulated mesh data (Y-up).
 * web-ifc GetFlatMesh uses Y-up: X,Z are the floor plan, Y is height.
 * Finds bottom-face triangles (all vertices at minY), identifies boundary
 * edges (edges appearing in exactly one triangle), and chains them into
 * an ordered polygon. Preserves concave shapes like L-rooms.
 */
function extractFloorOutlineYUp(
  vertices: { x: number; y: number; z: number }[],
  indices: number[],
  minY: number,
  meshToMm: number,
  yTol: number,
): Point2D[] | null {
  // Snap scale for position deduplication (~1mm precision)
  const snapScale = meshToMm >= 100 ? 100 : 100_000;

  // 1. Find bottom-face triangles and collect edges by position key
  const edgeCounts = new Map<string, number>();
  const edgeEndpoints = new Map<string, [string, string]>();

  for (let i = 0; i + 2 < indices.length; i += 3) {
    const v0 = vertices[indices[i]!]!;
    const v1 = vertices[indices[i + 1]!]!;
    const v2 = vertices[indices[i + 2]!]!;

    // All 3 vertices must be at the floor level (Y = vertical)
    if (
      Math.abs(v0.y - minY) > yTol ||
      Math.abs(v1.y - minY) > yTol ||
      Math.abs(v2.y - minY) > yTol
    ) {
      continue;
    }

    // Use XZ as floor plan coordinates
    const k0 = posKey(v0.x, v0.z, snapScale);
    const k1 = posKey(v1.x, v1.z, snapScale);
    const k2 = posKey(v2.x, v2.z, snapScale);

    // Register 3 edges with canonical keys
    for (const [a, b] of [[k0, k1], [k1, k2], [k2, k0]] as [string, string][]) {
      const key = a < b ? `${a}|${b}` : `${b}|${a}`;
      edgeCounts.set(key, (edgeCounts.get(key) ?? 0) + 1);
      if (!edgeEndpoints.has(key)) {
        edgeEndpoints.set(key, a < b ? [a, b] : [b, a]);
      }
    }
  }

  if (edgeCounts.size === 0) return null;

  // 2. Boundary edges appear exactly once
  const adjacency = new Map<string, Set<string>>();
  for (const [key, count] of edgeCounts) {
    if (count !== 1) continue;
    const [a, b] = edgeEndpoints.get(key)!;
    if (!adjacency.has(a)) adjacency.set(a, new Set());
    if (!adjacency.has(b)) adjacency.set(b, new Set());
    adjacency.get(a)!.add(b);
    adjacency.get(b)!.add(a);
  }

  if (adjacency.size < MIN_POLYGON_POINTS) return null;

  // 3. Walk boundary to form ordered polygon chain
  const startKey = adjacency.keys().next().value!;
  const chain: string[] = [startKey];
  const visited = new Set<string>([startKey]);
  let current = startKey;

  for (let step = 0; step < adjacency.size; step++) {
    const neighbors = adjacency.get(current);
    if (!neighbors) break;
    let next: string | undefined;
    for (const n of neighbors) {
      if (!visited.has(n)) { next = n; break; }
    }
    if (!next) break;
    chain.push(next);
    visited.add(next);
    current = next;
  }

  if (chain.length < MIN_POLYGON_POINTS) return null;

  // 4. Map position keys back to actual XZ coordinates (average of vertices)
  const posAvg = new Map<string, { sx: number; sz: number; n: number }>();
  for (const v of vertices) {
    if (Math.abs(v.y - minY) > yTol) continue;
    const key = posKey(v.x, v.z, snapScale);
    const entry = posAvg.get(key);
    if (entry) {
      entry.sx += v.x;
      entry.sz += v.z;
      entry.n++;
    } else {
      posAvg.set(key, { sx: v.x, sz: v.z, n: 1 });
    }
  }

  // Project XZ → modeller (X stays, Z → -Y for screen Y-down)
  const polygon: Point2D[] = [];
  for (const key of chain) {
    const avg = posAvg.get(key);
    if (!avg) continue;
    polygon.push({
      x: (avg.sx / avg.n) * meshToMm,
      y: -(avg.sz / avg.n) * meshToMm,
    });
  }

  // 5. Remove collinear points
  return simplifyPolygon(polygon);
}

/** Remove collinear intermediate points from a polygon. */
function simplifyPolygon(polygon: Point2D[]): Point2D[] {
  if (polygon.length <= MIN_POLYGON_POINTS) return polygon;

  const COLLINEAR_TOLERANCE = 1; // 1 mm²
  const result: Point2D[] = [];

  for (let i = 0; i < polygon.length; i++) {
    const prev = polygon[(i - 1 + polygon.length) % polygon.length]!;
    const curr = polygon[i]!;
    const next = polygon[(i + 1) % polygon.length]!;

    const cross =
      (curr.x - prev.x) * (next.y - prev.y) -
      (curr.y - prev.y) * (next.x - prev.x);
    if (Math.abs(cross) > COLLINEAR_TOLERANCE) {
      result.push(curr);
    }
  }

  return result.length >= MIN_POLYGON_POINTS ? result : polygon;
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
// Placement transform — walks the full IfcLocalPlacement chain
// ---------------------------------------------------------------------------

interface Transform2D {
  tx: number;
  ty: number;
  cos: number;
  sin: number;
}

const IDENTITY_TRANSFORM: Transform2D = { tx: 0, ty: 0, cos: 1, sin: 0 };

/**
 * Compose two transforms: result = outer(inner(point)).
 * Both transforms are in modeller space (mm, Y-down).
 */
function composeTransforms(
  outer: Transform2D,
  inner: Transform2D,
): Transform2D {
  return {
    cos: outer.cos * inner.cos - outer.sin * inner.sin,
    sin: outer.sin * inner.cos + outer.cos * inner.sin,
    tx: outer.cos * inner.tx - outer.sin * inner.ty + outer.tx,
    ty: outer.sin * inner.tx + outer.cos * inner.ty + outer.ty,
  };
}

/**
 * Extract a 2D transform from an IfcAxis2Placement3D (or 2D).
 * Converts IFC coordinates (Y-up) → modeller (mm, Y-down).
 */
function extractAxisPlacement2D(
  api: WebIfc.IfcAPI,
  modelId: number,
  axisPlacementId: number,
  unitToMm: number,
): Transform2D {
  try {
    const ap = api.GetLine(modelId, axisPlacementId);

    let tx = 0;
    let ty = 0;
    let cos = 1;
    let sin = 0;

    // Location → IfcCartesianPoint
    const locationRef = ap?.Location;
    if (locationRef?.value != null) {
      const location = api.GetLine(modelId, locationRef.value);
      const coords = location?.Coordinates;
      if (Array.isArray(coords) && coords.length >= 2) {
        tx = Number(coords[0]?.value ?? coords[0]) * unitToMm;
        ty = -(Number(coords[1]?.value ?? coords[1])) * unitToMm;
      }
    }

    // RefDirection → IfcDirection (X-axis) for in-plane rotation
    const refDirRef = ap?.RefDirection;
    if (refDirRef?.value != null) {
      const refDir = api.GetLine(modelId, refDirRef.value);
      const ratios = refDir?.DirectionRatios;
      if (Array.isArray(ratios) && ratios.length >= 2) {
        const dx = Number(ratios[0]?.value ?? ratios[0]);
        const dy = Number(ratios[1]?.value ?? ratios[1]);
        const len = Math.hypot(dx, dy);
        if (len > 1e-10) {
          cos = dx / len;
          sin = -(dy / len); // flip for Y-down
        }
      }
    }

    return { tx, ty, cos, sin };
  } catch {
    return IDENTITY_TRANSFORM;
  }
}

/**
 * Recursively walk an IfcLocalPlacement chain and compose all transforms.
 * Returns the cumulative transform from local → global in modeller space.
 */
function resolveLocalPlacement(
  api: WebIfc.IfcAPI,
  modelId: number,
  placementId: number,
  unitToMm: number,
  visited?: Set<number>,
): Transform2D {
  const seen = visited ?? new Set<number>();
  if (seen.has(placementId)) return IDENTITY_TRANSFORM;
  seen.add(placementId);

  try {
    const placement = api.GetLine(modelId, placementId);

    // This level's relative placement
    let local = IDENTITY_TRANSFORM;
    const relRef = placement?.RelativePlacement;
    if (relRef?.value != null) {
      local = extractAxisPlacement2D(api, modelId, relRef.value, unitToMm);
    }

    // Parent placement (IfcLocalPlacement.PlacementRelTo)
    const parentRef = placement?.PlacementRelTo;
    if (parentRef?.value != null) {
      const parent = resolveLocalPlacement(
        api, modelId, parentRef.value, unitToMm, seen,
      );
      return composeTransforms(parent, local);
    }

    return local;
  } catch {
    return IDENTITY_TRANSFORM;
  }
}

/**
 * Get the full placement transform for an IFC product (e.g. IfcSpace).
 * Walks the ObjectPlacement → IfcLocalPlacement chain up to the root.
 */
function getPlacementTransform(
  api: WebIfc.IfcAPI,
  modelId: number,
  expressId: number,
  unitToMm: number,
): Transform2D {
  try {
    const obj = api.GetLine(modelId, expressId);
    const placementRef = obj?.ObjectPlacement;
    if (placementRef?.value != null) {
      return resolveLocalPlacement(
        api, modelId, placementRef.value, unitToMm,
      );
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
    // Detect length unit (meters, mm, etc.) → conversion factor to mm
    const unitToMm = detectUnitToMm(api, modelId);

    // Extract storey structure
    const storeyMap = extractStoreys(api, modelId);

    // Find all IfcSpace entities
    const spaceIds = api.GetLineIDsWithType(modelId, IFCSPACE);
    result.stats.spacesFound = spaceIds.size();

    for (let i = 0; i < spaceIds.size(); i++) {
      const spaceId = spaceIds.get(i);
      const spaceName = getSpaceName(api, modelId, spaceId);

      // Try extraction strategies in order.
      // Mesh first: uses flatTransformation for correct global positioning.
      // Profile/Brep are fallbacks for spaces without mesh geometry.
      let extracted: ExtractionResult | null = null;

      extracted = tryExtractMesh(api, modelId, spaceId);

      if (!extracted) {
        extracted = tryExtractProfile(api, modelId, spaceId, unitToMm);
      }

      if (!extracted) {
        extracted = tryExtractBrep(api, modelId, spaceId, unitToMm);
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
      let floorIndex: number;
      let elevation: number | undefined;

      if (extracted.floorElevationMm !== undefined && storeyMap.size > 0) {
        // Match mesh floor elevation against storey elevations
        const meshElev = extracted.floorElevationMm;
        let bestStorey: StoreyInfo | undefined;
        let bestDist = Infinity;
        for (const storey of storeyMap.values()) {
          // Storey elevation is in IFC file units
          const storeyMm = storey.elevation * unitToMm;
          const dist = Math.abs(meshElev - storeyMm);
          if (dist < bestDist) {
            bestDist = dist;
            bestStorey = storey;
          }
        }
        floorIndex = bestStorey?.floorIndex ?? 0;
        elevation = meshElev;
      } else {
        // Fallback: IFC spatial containment
        const spatial = findFloorForSpace(api, modelId, spaceId, storeyMap);
        floorIndex = spatial.floorIndex;
        elevation = spatial.elevationMeters !== undefined
          ? Math.round(spatial.elevationMeters * unitToMm)
          : undefined;
      }

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
