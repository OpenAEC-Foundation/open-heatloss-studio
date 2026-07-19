/**
 * IfcRelSpaceBoundary parsing + hybrid SB/raycast host resolution, ported
 * from the fase-1 PoC (`lib/spaceboundary.mjs`).
 *
 * NOTE on "neighbour space identity": the base IfcRelSpaceBoundary class
 * (no ParentBoundary/CorrespondingBoundary -- 2ndLevel schema not present on
 * either PoC model) cannot tell you WHICH space is on the other side of an
 * INTERNAL boundary, only THAT it is internal. So SB is used here for host +
 * internal/external certainty; raycast is still the (only) source for the
 * actual neighbour space id.
 */
import type * as WebIfc from "web-ifc";

import {
  add,
  applyMat4LinearOnly,
  applyMat4Plain,
  cross,
  dot,
  normalize,
  polygon2DAreaWithHoles,
  scale,
  sub,
} from "./geom";
import { getLineValue, vecToArray } from "./meshExtract";
import type { Face } from "./faceClustering";
import type { HostElementRef, SbInternalOrExternal, Vec3 } from "./types";

/** A raw IFC scalar value as returned by web-ifc: either a plain number or a
 * boxed measure ({value: number}) / derived measure ({_representationValue}). */
type Boxed = { value?: unknown; _representationValue?: unknown } | number;

function lv(x: Boxed | undefined | null): unknown {
  if (x == null) return x;
  if (typeof x === "number") return x;
  return x.value ?? x._representationValue ?? x;
}

function toNumber(x: unknown): number {
  return Number(lv(x as Boxed));
}

interface RawPoint {
  Coordinates?: unknown[];
}

function polylinePoints2D(polyline: { Points?: RawPoint[] } | undefined): [number, number][] {
  const points = polyline?.Points ?? [];
  return points
    .map((p) => (p.Coordinates ?? []).map(toNumber))
    .filter((p): p is [number, number] => p.length === 2);
}

export interface ParsedSpaceBoundary {
  id: number;
  spaceId: number | null;
  areaMM2: number | null;
  axisZ: number | null;
  orientation: "plafond" | "vloer" | "wand" | "onbekend";
  physicalOrVirtual: "PHYSICAL" | "VIRTUAL" | null;
  internalOrExternal: SbInternalOrExternal | null;
  hostElement: HostElementRef | null;
  worldCentroidMM: Vec3 | null;
  worldNormal: Vec3 | null;
}

export interface ParseSpaceBoundariesResult {
  boundaries: ParsedSpaceBoundary[];
}

/** Minimal shape of the values we read off a web-ifc GetLine() result. */
type IfcLine = Record<string, unknown>;

function asLine(x: unknown): IfcLine | null {
  return (x as IfcLine | null) ?? null;
}

function refValue(x: unknown): number | null {
  const v = (x as { value?: number } | null | undefined)?.value;
  return v ?? null;
}

export function parseSpaceBoundaries(
  api: WebIfc.IfcAPI,
  modelID: number,
  ifcRelSpaceBoundaryType: number,
): ParseSpaceBoundariesResult {
  const sbIds = vecToArray(api.GetLineIDsWithType(modelID, ifcRelSpaceBoundaryType, false));
  const boundaries: ParsedSpaceBoundary[] = [];
  const placementMatrixCache = new Map<number, number[] | null>();

  function getSpacePlacementMatrix(spaceId: number | null): number[] | null {
    if (spaceId == null) return null;
    const cached = placementMatrixCache.get(spaceId);
    if (cached !== undefined) return cached;
    const spaceLine = asLine(api.GetLine(modelID, spaceId));
    const placementId = refValue(spaceLine?.["ObjectPlacement"]);
    const m = placementId != null ? api.GetWorldTransformMatrix(modelID, placementId) : null;
    placementMatrixCache.set(spaceId, m);
    return m;
  }

  function hostElementInfo(hostId: number | null): HostElementRef | null {
    if (hostId == null) return null;
    const line = asLine(api.GetLine(modelID, hostId));
    let ifcType: string | null = null;
    try {
      ifcType = api.GetNameFromTypeCode((line as { type: number } | null)?.type ?? -1);
    } catch {
      ifcType = null;
    }
    return { id: hostId, name: getLineValue(line, "Name") as string | null, ifcType };
  }

  for (const id of sbIds) {
    const sb = asLine(api.GetLine(modelID, id));
    const spaceId = refValue(sb?.["RelatingSpace"]);
    const hostId = refValue(sb?.["RelatedBuildingElement"]);
    const physicalOrVirtual = (sb?.["PhysicalOrVirtualBoundary"] as { value?: string } | undefined)
      ?.value as "PHYSICAL" | "VIRTUAL" | undefined ?? null;
    const internalOrExternal = (sb?.["InternalOrExternalBoundary"] as { value?: string } | undefined)
      ?.value as SbInternalOrExternal | undefined ?? null;

    let axisZ: number | null = null;
    let worldCentroidMM: Vec3 | null = null;
    let worldNormal: Vec3 | null = null;
    let areaMM2: number | null = null;

    try {
      const cgRef = sb?.["ConnectionGeometry"] as { value?: number } | undefined;
      const cg = cgRef?.value != null ? asLine(api.GetLine(modelID, cgRef.value, true)) : null;
      const surf = cg?.["SurfaceOnRelatingElement"] as
        | {
            OuterBoundary?: { Points?: RawPoint[] };
            InnerBoundaries?: unknown;
            BasisSurface?: {
              Position?: {
                Location?: { Coordinates?: unknown[] };
                Axis?: { DirectionRatios?: unknown[] };
                RefDirection?: { DirectionRatios?: unknown[] };
              };
            };
          }
        | undefined;
      const outerPts2D = polylinePoints2D(surf?.OuterBoundary);

      if (outerPts2D.length) {
        const inner = surf?.InnerBoundaries;
        const innerList = Array.isArray(inner) ? inner : inner ? [inner] : [];
        const holePts2D = innerList.map((hole) =>
          polylinePoints2D(hole as { Points?: RawPoint[] } | undefined),
        );
        areaMM2 = polygon2DAreaWithHoles(outerPts2D, holePts2D);
      }

      const pos = surf?.BasisSurface?.Position;
      const location = pos?.Location?.Coordinates?.map(toNumber);
      const axisLocal = pos?.Axis?.DirectionRatios?.map(toNumber);
      const refDirLocal = pos?.RefDirection?.DirectionRatios?.map(toNumber);
      axisZ = axisLocal?.[2] ?? null;

      // ConnectionGeometry for a base-schema SB is defined relative to the
      // RelatingSpace's OWN ObjectPlacement (small, human-scale numbers),
      // NOT the world frame directly, and NOT the same local frame as the
      // tessellated FlatMesh vertex data. Build the plane's local 3D basis
      // (Gram-Schmidt against Axis, per the IfcAxis2Placement3D convention),
      // place the 2D boundary centroid in that basis, then push through the
      // space's placement matrix to world Z-up mm (plain apply -- see
      // geom.ts::applyMat4Plain, and its doc comment on why this differs
      // from the FlatMesh flatTransformation convention used elsewhere).
      if (
        location &&
        location.length === 3 &&
        axisLocal &&
        axisLocal.length === 3 &&
        refDirLocal &&
        refDirLocal.length === 3 &&
        outerPts2D.length
      ) {
        const locationVec = location as Vec3;
        const zAxis = normalize(axisLocal as Vec3);
        const xAxisRaw = sub(refDirLocal as Vec3, scale(zAxis, dot(refDirLocal as Vec3, zAxis)));
        const xAxis = normalize(xAxisRaw);
        const yAxis = cross(zAxis, xAxis);

        const centroid2D = outerPts2D.reduce<[number, number]>(
          (acc, p) => [acc[0] + p[0] / outerPts2D.length, acc[1] + p[1] / outerPts2D.length],
          [0, 0],
        );
        const centroidLocal3D = add(
          locationVec,
          add(scale(xAxis, centroid2D[0]), scale(yAxis, centroid2D[1])),
        );

        const m = getSpacePlacementMatrix(spaceId);
        if (m) {
          worldCentroidMM = applyMat4Plain(m, centroidLocal3D);
          worldNormal = applyMat4LinearOnly(m, zAxis);
        }
      }
    } catch {
      // Leave axisZ/world* null; still record the boundary as "seen but unparsed".
    }

    const orientation: ParsedSpaceBoundary["orientation"] =
      axisZ == null ? "onbekend" : axisZ > 0.7 ? "plafond" : axisZ < -0.7 ? "vloer" : "wand";

    boundaries.push({
      id,
      spaceId,
      areaMM2,
      axisZ,
      orientation,
      physicalOrVirtual,
      internalOrExternal,
      hostElement: hostElementInfo(hostId),
      worldCentroidMM,
      worldNormal,
    });
  }

  return { boundaries };
}

export interface RaycastFaceForMatch {
  centroidMM: Vec3 | null;
  normal: Vec3;
  hostElement: HostElementRef | null;
  classification: string;
  areaM2: number;
}

export interface RaycastSpaceForMatch {
  id: number;
  faces: RaycastFaceForMatch[];
}

export interface BoundaryRaycastMatch {
  sbId: number;
  spaceId: number | null;
  sbHost: HostElementRef | null;
  sbAreaM2: number | null;
  sbOrientation: ParsedSpaceBoundary["orientation"];
  sbInternalOrExternal: SbInternalOrExternal | null;
  matchedFace: {
    distanceMM: number;
    normalDot: number;
    raycastHost: HostElementRef | null;
    raycastClassification: string;
    raycastAreaM2: number;
  } | null;
  hostAgreement: "match" | "mismatch" | "no-match";
}

export interface MatchOptions {
  maxCentroidDistMM?: number;
  minNormalDot?: number;
}

/**
 * Match each PHYSICAL space boundary to the raycast face with the closest
 * world centroid AND a well-aligned normal, within the same space. Returns
 * match records exposing both sides' host element for a direct
 * SB-vs-raycast host comparison. Used for model-wide QC statistics, not for
 * the per-face hybrid resolution itself (see findBestPhysicalBoundaryForFace).
 */
export function matchBoundariesToRaycastFaces(
  boundaries: readonly ParsedSpaceBoundary[],
  resultSpaces: readonly RaycastSpaceForMatch[],
  { maxCentroidDistMM = 800, minNormalDot = 0.85 }: MatchOptions = {},
): BoundaryRaycastMatch[] {
  const matches: BoundaryRaycastMatch[] = [];
  const spaceById = new Map(resultSpaces.map((s) => [s.id, s]));

  for (const b of boundaries) {
    if (b.physicalOrVirtual !== "PHYSICAL" || !b.worldCentroidMM || !b.worldNormal) continue;
    const rs = spaceById.get(b.spaceId ?? -1);
    if (!rs) continue;

    let best: { face: RaycastFaceForMatch; distanceMM: number; normalDot: number } | null = null;
    let bestScore = -Infinity;
    for (const f of rs.faces) {
      if (!f.centroidMM) continue;
      const d = Math.hypot(
        b.worldCentroidMM[0] - f.centroidMM[0],
        b.worldCentroidMM[1] - f.centroidMM[1],
        b.worldCentroidMM[2] - f.centroidMM[2],
      );
      const nd = Math.abs(dot(b.worldNormal, f.normal));
      if (d <= maxCentroidDistMM && nd >= minNormalDot) {
        const score = nd - d / maxCentroidDistMM;
        if (score > bestScore) {
          bestScore = score;
          best = { face: f, distanceMM: d, normalDot: nd };
        }
      }
    }

    matches.push({
      sbId: b.id,
      spaceId: b.spaceId,
      sbHost: b.hostElement,
      sbAreaM2: b.areaMM2 != null ? b.areaMM2 / 1e6 : null,
      sbOrientation: b.orientation,
      sbInternalOrExternal: b.internalOrExternal,
      matchedFace: best
        ? {
            distanceMM: Number(best.distanceMM.toFixed(0)),
            normalDot: Number(best.normalDot.toFixed(3)),
            raycastHost: best.face.hostElement,
            raycastClassification: best.face.classification,
            raycastAreaM2: best.face.areaM2,
          }
        : null,
      hostAgreement: best
        ? best.face.hostElement?.id === b.hostElement?.id
          ? "match"
          : "mismatch"
        : "no-match",
    });
  }
  return matches;
}

export interface PhysicalBoundaryMatch {
  boundary: ParsedSpaceBoundary;
  distanceMM: number;
  normalDot: number;
}

/**
 * Hybrid host resolution, inverse direction of matchBoundariesToRaycastFaces:
 * for ONE raycast face, find the best-matching "healthy" PHYSICAL boundary
 * (has a resolvable host element) for the same space. Used per-face during
 * the main classification loop so SB data can be preferred over the
 * raycast-derived host BEFORE the raycast result is finalised ("match -> SB
 * host als primaire semantiek").
 */
export function findBestPhysicalBoundaryForFace(
  face: Pick<Face, "normal" | "centroid">,
  spaceId: number,
  physicalBoundariesBySpace: ReadonlyMap<number, ParsedSpaceBoundary[]>,
  { maxCentroidDistMM = 800, minNormalDot = 0.85 }: MatchOptions = {},
): PhysicalBoundaryMatch | null {
  const candidates = physicalBoundariesBySpace.get(spaceId) ?? [];
  const faceCentroid = face.centroid;
  let best: PhysicalBoundaryMatch | null = null;
  let bestScore = -Infinity;
  for (const b of candidates) {
    if (!b.worldCentroidMM || !b.worldNormal || !b.hostElement) continue;
    const d = Math.hypot(
      b.worldCentroidMM[0] - faceCentroid[0],
      b.worldCentroidMM[1] - faceCentroid[1],
      b.worldCentroidMM[2] - faceCentroid[2],
    );
    const nd = Math.abs(dot(b.worldNormal, face.normal));
    if (d <= maxCentroidDistMM && nd >= minNormalDot) {
      const score = nd - d / maxCentroidDistMM;
      if (score > bestScore) {
        bestScore = score;
        best = { boundary: b, distanceMM: Number(d.toFixed(0)), normalDot: Number(nd.toFixed(3)) };
      }
    }
  }
  return best;
}

/** Group parsed boundaries by space, PHYSICAL-only + "healthy" (host resolves). */
export function groupPhysicalBoundariesBySpace(
  boundaries: readonly ParsedSpaceBoundary[],
): Map<number, ParsedSpaceBoundary[]> {
  const bySpace = new Map<number, ParsedSpaceBoundary[]>();
  for (const b of boundaries) {
    if (b.physicalOrVirtual !== "PHYSICAL" || !b.hostElement || b.spaceId == null) continue;
    let arr = bySpace.get(b.spaceId);
    if (!arr) {
      arr = [];
      bySpace.set(b.spaceId, arr);
    }
    arr.push(b);
  }
  return bySpace;
}
