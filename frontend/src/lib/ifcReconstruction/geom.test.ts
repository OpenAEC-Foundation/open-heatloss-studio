/**
 * Tests voor `geom.ts` — de coordinate-conventies (toZUpMM) en de
 * Moeller-Trumbore ray/triangle-intersectie zijn de twee stukken die bij een
 * web-ifc-versie-upgrade stil kunnen breken (zie de convention notes in
 * geom.ts zelf); deze tests pinnen het verwachte gedrag vast.
 */
import { describe, expect, it } from "vitest";

import { polygon2DArea, polygon2DAreaWithHoles, rayTriangleIntersect, toZUpMM } from "./geom";
import type { Vec3 } from "./types";

describe("toZUpMM — Y-up metres (web-ifc FlatMesh) naar Z-up millimeters", () => {
  const IDENTITY = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

  it("permuteert assen: Z-up = (x, -z, y) van de Y-up-invoer, en schaalt m -> mm", () => {
    // Y-up input (1, 2, 3) meter -> verwacht Z-up (1000, -3000, 2000) mm:
    // Z-up.x = Y-up.x, Z-up.y = -Y-up.z, Z-up.z = Y-up.y (zie geom.ts-commentaar).
    const [x, y, z] = toZUpMM(IDENTITY, 1, 2, 3);
    expect(x).toBeCloseTo(1000, 6);
    expect(y).toBeCloseTo(-3000, 6);
    expect(z).toBeCloseTo(2000, 6);
  });

  it("neemt de matrix-translatie (kolom 4) mee, ook geschaald naar mm", () => {
    const withTranslation = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1];
    const [x, y, z] = toZUpMM(withTranslation, 0, 0, 0);
    expect(x).toBeCloseTo(10000, 6);
    expect(y).toBeCloseTo(-30000, 6);
    expect(z).toBeCloseTo(20000, 6);
  });
});

describe("rayTriangleIntersect — Moeller-Trumbore, bekende snijgevallen", () => {
  // Driehoek in het xy-vlak (z=0): (0,0,0)-(10,0,0)-(0,10,0), mm.
  const p0: Vec3 = [0, 0, 0];
  const p1: Vec3 = [10, 0, 0];
  const p2: Vec3 = [0, 10, 0];

  it("raakt de driehoek loodrecht van boven, op de juiste afstand t", () => {
    const origin: Vec3 = [2, 2, 10];
    const dir: Vec3 = [0, 0, -1];
    const t = rayTriangleIntersect(origin, dir, p0, p1, p2);
    expect(t).not.toBeNull();
    expect(t!).toBeCloseTo(10, 6);
  });

  it("mist de driehoek als het punt buiten de hypotenusa valt (u+v > 1)", () => {
    const origin: Vec3 = [8, 8, 10]; // 8+8=16 > 10 -> buiten de driehoek
    const dir: Vec3 = [0, 0, -1];
    expect(rayTriangleIntersect(origin, dir, p0, p1, p2)).toBeNull();
  });

  it("geeft null voor een straal evenwijdig aan het vlak", () => {
    const origin: Vec3 = [2, 2, 5];
    const dir: Vec3 = [1, 0, 0]; // loodrecht op de vlaknormaal (0,0,1)
    expect(rayTriangleIntersect(origin, dir, p0, p1, p2)).toBeNull();
  });

  it("negeert een snijpunt achter de oorsprong van de straal (t <= epsilon)", () => {
    const origin: Vec3 = [2, 2, -5];
    const dir: Vec3 = [0, 0, -1]; // beweegt van het vlak af, snijpunt zou t=-5 zijn
    expect(rayTriangleIntersect(origin, dir, p0, p1, p2)).toBeNull();
  });
});

describe("polygon2DAreaWithHoles — shoelace met gaten (SB-vlakken met opening)", () => {
  it("trekt een enkel gat correct af van de buitencontour", () => {
    const outer: [number, number][] = [
      [0, 0],
      [100, 0],
      [100, 100],
      [0, 100],
    ];
    const hole: [number, number][] = [
      [20, 20],
      [80, 20],
      [80, 80],
      [20, 80],
    ];
    // 100x100 buiten - 60x60 gat = 10000 - 3600 = 6400
    expect(polygon2DAreaWithHoles(outer, [hole])).toBeCloseTo(6400, 6);
  });

  it("gedraagt zich als polygon2DArea zonder gaten", () => {
    const outer: [number, number][] = [
      [0, 0],
      [100, 0],
      [100, 50],
      [0, 50],
    ];
    expect(polygon2DAreaWithHoles(outer, [])).toBeCloseTo(polygon2DArea(outer), 6);
  });

  it("trekt meerdere gaten af (som van gat-oppervlaktes)", () => {
    const outer: [number, number][] = [
      [0, 0],
      [100, 0],
      [100, 100],
      [0, 100],
    ];
    const holeA: [number, number][] = [
      [10, 10],
      [20, 10],
      [20, 20],
      [10, 20],
    ]; // 100
    const holeB: [number, number][] = [
      [70, 70],
      [90, 70],
      [90, 90],
      [70, 90],
    ]; // 400
    expect(polygon2DAreaWithHoles(outer, [holeA, holeB])).toBeCloseTo(10000 - 100 - 400, 6);
  });
});
