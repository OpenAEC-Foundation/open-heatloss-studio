/**
 * Material layer lookup, ported from the fase-1 PoC (`lib/materials.mjs`).
 */
import type * as WebIfc from "web-ifc";

import type { MaterialLayerSet } from "./types";

interface RawMaterialLayer {
  Name?: { value?: unknown };
  Material?: { Name?: { value?: unknown } };
  LayerThickness?: { value?: unknown; _representationValue?: unknown };
}

interface RawMaterial {
  Name?: { value?: unknown };
  LayerSetName?: { value?: unknown };
  MaterialLayers?: RawMaterialLayer[];
}

/**
 * Look up the material layer set (name-ordered layers + thickness) for a
 * building element, if any. Returns null when no IfcMaterialLayerSet is
 * found (e.g. windows/doors, or elements associated only with a single
 * IfcMaterial).
 */
export async function getMaterialLayers(
  api: WebIfc.IfcAPI,
  modelID: number,
  expressID: number,
): Promise<MaterialLayerSet | null> {
  let mats: RawMaterial[];
  try {
    mats = (await api.properties.getMaterialsProperties(modelID, expressID, true, true)) as RawMaterial[];
  } catch {
    return null;
  }
  if (!mats || !mats.length) return null;

  for (const m of mats) {
    if (m.MaterialLayers) {
      const layers = m.MaterialLayers.map((l) => ({
        name: (l.Name?.value ?? l.Material?.Name?.value ?? null) as string | null,
        materialName: (l.Material?.Name?.value ?? null) as string | null,
        thicknessMM: (l.LayerThickness?.value ?? l.LayerThickness?._representationValue ?? null) as
          | number
          | null,
      }));
      const totalThicknessMM = layers.reduce((s, l) => s + (l.thicknessMM ?? 0), 0);
      return {
        layerSetName: (m.LayerSetName?.value ?? null) as string | null,
        layers,
        totalThicknessMM,
      };
    }
  }
  // Fallback: single IfcMaterial (no layering info), still useful to report.
  const single = mats.find((m) => m.Name && !m.MaterialLayers);
  if (single) {
    const name = single.Name?.value as string;
    return {
      layerSetName: null,
      layers: [{ name, materialName: name, thicknessMM: null }],
      totalThicknessMM: null,
    };
  }
  return null;
}
