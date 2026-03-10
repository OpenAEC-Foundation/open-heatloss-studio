/**
 * IFC wall type extractor.
 *
 * Traversal path:
 *   IfcWallType → IfcRelAssociatesMaterial → IfcMaterialLayerSet → IfcMaterialLayer[]
 *
 * Returns wall types with layer compositions for import into project constructions.
 */
import * as WebIfc from "web-ifc";

import {
  matchIfcMaterial,
  type MaterialMatch,
} from "../../lib/ifcMaterialMatcher";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const IFCWALLTYPE = WebIfc.IFCWALLTYPE;
const IFCRELASSOCIATESMATERIAL = WebIfc.IFCRELASSOCIATESMATERIAL;
const IFCPROJECT = WebIfc.IFCPROJECT;

/** SI prefix → multiplier to meters (same as ifc-import.ts). */
const SI_PREFIX_FACTOR: Record<string, number> = {
  ".EXA.": 1e18, ".PETA.": 1e15, ".TERA.": 1e12, ".GIGA.": 1e9,
  ".MEGA.": 1e6, ".KILO.": 1e3, ".HECTO.": 1e2, ".DECA.": 1e1,
  ".DECI.": 1e-1, ".CENTI.": 1e-2, ".MILLI.": 1e-3, ".MICRO.": 1e-6,
  ".NANO.": 1e-9, ".PICO.": 1e-12,
};

const DEFAULT_UNIT_TO_MM = 1000;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface IfcWallTypeLayer {
  /** IFC material name. */
  ifcMaterialName: string;
  /** Layer thickness in mm. */
  thickness: number;
  /** Best match from materialsDatabase. */
  match: MaterialMatch;
}

export interface IfcWallTypeInfo {
  /** IfcWallType name. */
  name: string;
  /** IfcWallType GlobalId. */
  globalId: string;
  /** Extracted layers with material matches. */
  layers: IfcWallTypeLayer[];
  /** All original IFC material names. */
  originalMaterialNames: string[];
}

// ---------------------------------------------------------------------------
// Unit detection (simplified from ifc-import.ts)
// ---------------------------------------------------------------------------

function detectUnitToMm(
  api: WebIfc.IfcAPI,
  modelId: number,
): number {
  try {
    const projectIds = api.GetLineIDsWithType(modelId, IFCPROJECT);
    if (projectIds.size() === 0) return DEFAULT_UNIT_TO_MM;

    const project = api.GetLine(modelId, projectIds.get(0));
    const unitsCtxRef = project?.UnitsInContext;
    const unitsAssignment =
      unitsCtxRef?.value != null
        ? api.GetLine(modelId, unitsCtxRef.value)
        : unitsCtxRef;

    const units = unitsAssignment?.Units;
    if (!Array.isArray(units)) return DEFAULT_UNIT_TO_MM;

    for (const unitRef of units) {
      const unit =
        unitRef?.value != null
          ? api.GetLine(modelId, unitRef.value)
          : unitRef;
      if (!unit) continue;

      const unitType = unit.UnitType?.value;
      if (unitType !== "LENGTHUNIT") continue;

      const name = unit.Name?.value;
      const prefix = unit.Prefix?.value;

      if (name === "METRE" || name === "METER") {
        if (!prefix) return 1000;
        const factor = SI_PREFIX_FACTOR[prefix];
        if (factor) return factor * 1000;
      }

      if (name === "FOOT" || name === "INCH") {
        return name === "FOOT" ? 304.8 : 25.4;
      }
    }
  } catch {
    // Fall through to default
  }
  return DEFAULT_UNIT_TO_MM;
}

// ---------------------------------------------------------------------------
// Material association lookup
// ---------------------------------------------------------------------------

/**
 * Build a map: expressID → material relating (IfcMaterialLayerSet etc.)
 * by traversing all IfcRelAssociatesMaterial relationships.
 */
function buildMaterialAssociations(
  api: WebIfc.IfcAPI,
  modelId: number,
): Map<number, unknown> {
  const map = new Map<number, unknown>();

  try {
    const relIds = api.GetLineIDsWithType(
      modelId,
      IFCRELASSOCIATESMATERIAL,
    );

    for (let i = 0; i < relIds.size(); i++) {
      const rel = api.GetLine(modelId, relIds.get(i));
      if (!rel) continue;

      const relatingMaterial = rel.RelatingMaterial;
      const relatedObjects = rel.RelatedObjects;

      if (!relatingMaterial || !Array.isArray(relatedObjects)) continue;

      // Resolve material definition
      const matDef =
        relatingMaterial.value != null
          ? api.GetLine(modelId, relatingMaterial.value)
          : relatingMaterial;

      for (const objRef of relatedObjects) {
        const objId =
          typeof objRef === "number"
            ? objRef
            : objRef?.value ?? objRef?.expressID;
        if (typeof objId === "number") {
          map.set(objId, matDef);
        }
      }
    }
  } catch {
    // Silent — some IFC files may have invalid relationships
  }

  return map;
}

/**
 * Extract IfcMaterialLayer[] from an IfcMaterialLayerSet or
 * IfcMaterialLayerSetUsage.
 */
function extractLayersFromMaterial(
  api: WebIfc.IfcAPI,
  modelId: number,
  matDef: Record<string, unknown>,
  unitToMm: number,
): IfcWallTypeLayer[] | null {
  if (!matDef) return null;

  // Could be IfcMaterialLayerSetUsage → follow ForLayerSet
  let layerSet = matDef;
  const forLayerSet = (matDef as Record<string, { value?: number }>)
    .ForLayerSet;
  if (forLayerSet?.value != null) {
    layerSet = api.GetLine(modelId, forLayerSet.value) as Record<
      string,
      unknown
    >;
  }

  // Get MaterialLayers array
  const materialLayers = (
    layerSet as Record<string, unknown[]>
  ).MaterialLayers;
  if (!Array.isArray(materialLayers) || materialLayers.length === 0) {
    return null;
  }

  const layers: IfcWallTypeLayer[] = [];

  for (const layerRef of materialLayers) {
    const layer =
      (layerRef as { value?: number })?.value != null
        ? (api.GetLine(
            modelId,
            (layerRef as { value: number }).value,
          ) as Record<string, unknown>)
        : (layerRef as Record<string, unknown>);

    if (!layer) continue;

    // Layer thickness
    const thicknessRaw = (layer.LayerThickness as { value?: number })
      ?.value;
    const thickness =
      typeof thicknessRaw === "number" ? thicknessRaw * unitToMm : 0;

    // Material name
    const materialRef = layer.Material as { value?: number };
    let ifcMaterialName = "Onbekend";

    if (materialRef?.value != null) {
      const materialObj = api.GetLine(
        modelId,
        materialRef.value,
      ) as Record<string, { value?: string }>;
      ifcMaterialName = materialObj?.Name?.value ?? "Onbekend";
    }

    const match = matchIfcMaterial(ifcMaterialName);

    layers.push({
      ifcMaterialName,
      thickness: Math.round(thickness * 10) / 10,
      match,
    });
  }

  return layers.length > 0 ? layers : null;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Extract wall types from an IFC file (standalone).
 *
 * Opens the file, extracts wall types, and closes the model.
 */
export async function extractWallTypesFromFile(
  file: File,
): Promise<IfcWallTypeInfo[]> {
  const api = new WebIfc.IfcAPI();
  api.SetWasmPath("/wasm/");
  await api.Init();

  const buffer = await file.arrayBuffer();
  const data = new Uint8Array(buffer);
  const modelId = api.OpenModel(data, {
    COORDINATE_TO_ORIGIN: false,
  });

  try {
    return extractWallTypes(api, modelId);
  } finally {
    api.CloseModel(modelId);
  }
}

/**
 * Extract all IfcWallType entities with their material layer compositions.
 *
 * Uses an already-opened web-ifc model.
 */
export function extractWallTypes(
  api: WebIfc.IfcAPI,
  modelId: number,
): IfcWallTypeInfo[] {
  const unitToMm = detectUnitToMm(api, modelId);
  const materialMap = buildMaterialAssociations(api, modelId);
  const results: IfcWallTypeInfo[] = [];

  try {
    const wallTypeIds = api.GetLineIDsWithType(modelId, IFCWALLTYPE);

    for (let i = 0; i < wallTypeIds.size(); i++) {
      const typeId = wallTypeIds.get(i);
      const wallType = api.GetLine(modelId, typeId) as Record<
        string,
        { value?: string }
      >;

      if (!wallType) continue;

      const name = wallType.Name?.value ?? `WallType_${typeId}`;
      const globalId = wallType.GlobalId?.value ?? "";

      // Find material association
      const matDef = materialMap.get(typeId);
      if (!matDef) continue;

      const layers = extractLayersFromMaterial(
        api,
        modelId,
        matDef as Record<string, unknown>,
        unitToMm,
      );
      if (!layers || layers.length === 0) continue;

      results.push({
        name,
        globalId,
        layers,
        originalMaterialNames: layers.map((l) => l.ifcMaterialName),
      });
    }
  } catch {
    // Silent — graceful degradation
  }

  return results;
}
