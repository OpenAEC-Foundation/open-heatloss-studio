/**
 * Tests voor `faceClustering.ts` tegen een synthetische kubus-mesh (6 vlakken
 * van 1x1 m, elk 2 driehoeken). Dekt zowel de connectivity-flood-fill
 * (12 driehoeken -> exact 6 vlakken, geen bridging over hoeken) als de
 * per-vlak oppervlakte-optelling.
 */
import { describe, expect, it } from "vitest";

import { classifyOrientation, groupPlanarFacesConnected } from "./faceClustering";
import { triangleArea, triangleNormal } from "./geom";
import type { Triangle, Vec3 } from "./types";

/** Build a Triangle record the way meshExtract.ts would, from three world-mm points. */
function tri(p0: Vec3, p1: Vec3, p2: Vec3, sourceId = 1): Triangle {
  return { p0, p1, p2, normal: triangleNormal(p0, p1, p2), area: triangleArea(p0, p1, p2), sourceId };
}

/**
 * A 1000mm cube, corners at (0|1000, 0|1000, 0|1000), each of the 6 faces
 * split into 2 triangles with CCW winding (viewed from outside) so the
 * cross-product normal points outward. Two triangles per face always share
 * the diagonal edge, so flood-fill trivially reconnects them; adjacent cube
 * faces also share an edge but have perpendicular normals, so they must NOT
 * merge (that's the actual thing under test).
 */
function buildCubeTriangles(): Triangle[] {
  const triangles: Triangle[] = [
    // Bottom (z=0), outward normal (0,0,-1)
    tri([0, 0, 0], [0, 1000, 0], [1000, 1000, 0]),
    tri([0, 0, 0], [1000, 1000, 0], [1000, 0, 0]),
    // Top (z=1000), outward normal (0,0,1)
    tri([0, 0, 1000], [1000, 0, 1000], [1000, 1000, 1000]),
    tri([0, 0, 1000], [1000, 1000, 1000], [0, 1000, 1000]),
    // Front (y=0), outward normal (0,-1,0)
    tri([0, 0, 0], [1000, 0, 0], [1000, 0, 1000]),
    tri([0, 0, 0], [1000, 0, 1000], [0, 0, 1000]),
    // Back (y=1000), outward normal (0,1,0)
    tri([0, 1000, 0], [0, 1000, 1000], [1000, 1000, 1000]),
    tri([0, 1000, 0], [1000, 1000, 1000], [1000, 1000, 0]),
    // Left (x=0), outward normal (-1,0,0)
    tri([0, 0, 0], [0, 0, 1000], [0, 1000, 1000]),
    tri([0, 0, 0], [0, 1000, 1000], [0, 1000, 0]),
    // Right (x=1000), outward normal (1,0,0)
    tri([1000, 0, 0], [1000, 1000, 0], [1000, 1000, 1000]),
    tri([1000, 0, 0], [1000, 1000, 1000], [1000, 0, 1000]),
  ];
  return triangles;
}

describe("groupPlanarFacesConnected — synthetische kubus (12 driehoeken, 6 vlakken)", () => {
  it("clustert exact 6 vlakken (geen bridging over loodrechte hoeken)", () => {
    const faces = groupPlanarFacesConnected(buildCubeTriangles());
    expect(faces).toHaveLength(6);
  });

  it("elk vlak krijgt precies 2 driehoeken (de twee die de diagonaal delen)", () => {
    const faces = groupPlanarFacesConnected(buildCubeTriangles());
    for (const f of faces) {
      expect(f.triangles).toHaveLength(2);
    }
  });

  it("elk vlak heeft de juiste oppervlakte: 1000x1000mm = 1e6 mm^2 (1 m^2)", () => {
    const faces = groupPlanarFacesConnected(buildCubeTriangles());
    for (const f of faces) {
      expect(f.area).toBeCloseTo(1_000_000, 3);
    }
  });

  it("de totale oppervlakte van alle vlakken samen is 6 m^2 (6e6 mm^2)", () => {
    const faces = groupPlanarFacesConnected(buildCubeTriangles());
    const total = faces.reduce((s, f) => s + f.area, 0);
    expect(total).toBeCloseTo(6_000_000, 3);
  });

  it("de 6 vlak-normalen zijn de 6 axis-aligned richtingen, elk precies eenmaal", () => {
    const faces = groupPlanarFacesConnected(buildCubeTriangles());
    const rounded = faces.map((f) => f.normal.map((n) => Math.round(n)).join(","));
    const expected = ["0,0,-1", "0,0,1", "0,-1,0", "0,1,0", "-1,0,0", "1,0,0"];
    expect(rounded.sort()).toEqual(expected.sort());
  });
});

describe("classifyOrientation — pyrevit z-conventie", () => {
  it("z > 0.7 -> plafond", () => {
    expect(classifyOrientation([0, 0, 1])).toBe("plafond");
  });
  it("z < -0.7 -> vloer", () => {
    expect(classifyOrientation([0, 0, -1])).toBe("vloer");
  });
  it("|z| <= 0.7 -> wand", () => {
    expect(classifyOrientation([1, 0, 0])).toBe("wand");
    expect(classifyOrientation([0, 1, 0])).toBe("wand");
  });
});
