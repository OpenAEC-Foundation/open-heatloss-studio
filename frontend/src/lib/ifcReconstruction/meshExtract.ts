/**
 * Element/space mesh extraction via web-ifc's `GetFlatMesh`, ported from the
 * fase-1 PoC (`lib/model.mjs`).
 */
import type * as WebIfc from "web-ifc";

import { toZUpMM, triangleArea, triangleNormal } from "./geom";
import type { Triangle } from "./types";

/** Vector<T> -> plain array (web-ifc's Vector doesn't implement Array). */
export function vecToArray<T>(vec: WebIfc.Vector<T>): T[] {
  const out: T[] = [];
  for (let i = 0; i < vec.size(); i++) out.push(vec.get(i));
  return out;
}

/**
 * Extract world-space (Z-up mm) triangles for one IFC element/space via its
 * FlatMesh. Degenerate triangles (area < 1e-3 mm^2) are dropped.
 */
export function getElementTriangles(
  api: WebIfc.IfcAPI,
  modelID: number,
  expressID: number,
): Triangle[] {
  const triangles: Triangle[] = [];
  let flatMesh: WebIfc.FlatMesh;
  try {
    flatMesh = api.GetFlatMesh(modelID, expressID);
  } catch {
    return triangles;
  }
  if (!flatMesh || flatMesh.geometries.size() === 0) return triangles;

  for (let i = 0; i < flatMesh.geometries.size(); i++) {
    const pg = flatMesh.geometries.get(i);
    const geom = api.GetGeometry(modelID, pg.geometryExpressID);
    const vertexData = api.GetVertexArray(geom.GetVertexData(), geom.GetVertexDataSize());
    const indexData = api.GetIndexArray(geom.GetIndexData(), geom.GetIndexDataSize());
    const nVerts = vertexData.length / 6;
    const pts: [number, number, number][] = new Array(nVerts);
    for (let v = 0; v < nVerts; v++) {
      pts[v] = toZUpMM(
        pg.flatTransformation,
        vertexData[v * 6]!,
        vertexData[v * 6 + 1]!,
        vertexData[v * 6 + 2]!,
      );
    }
    const nTris = indexData.length / 3;
    for (let t = 0; t < nTris; t++) {
      const i0 = indexData[t * 3]!;
      const i1 = indexData[t * 3 + 1]!;
      const i2 = indexData[t * 3 + 2]!;
      const p0 = pts[i0]!;
      const p1 = pts[i1]!;
      const p2 = pts[i2]!;
      const area = triangleArea(p0, p1, p2);
      if (area < 1e-3) continue; // degenerate sliver
      const normal = triangleNormal(p0, p1, p2);
      triangles.push({ p0, p1, p2, normal, area, sourceId: expressID });
    }
    geom.delete();
  }
  return triangles;
}

/** Read `line[key].value` (web-ifc's boxed-value convention), or null. */
export function getLineValue(line: unknown, key: string): unknown {
  const record = line as Record<string, { value?: unknown } | undefined> | null | undefined;
  return record?.[key]?.value ?? null;
}
