/**
 * Orchestration for the IFC space/wall reconstruction pipeline, ported from
 * the fase-1 PoC (`run-fase1.mjs`). Pure/async, no DOM/CLI/file-system
 * assumptions -- the web worker (`workers/ifcReconstruction.worker.ts`) is
 * the only caller that touches I/O (reading the uploaded File into an
 * ArrayBuffer) and progress reporting.
 */
import * as WebIfc from "web-ifc";

import { classifyGroundSplit } from "./envelopeClassification";
import { classifyOrientation, groupPlanarFacesConnected, sampleFacePoints } from "./faceClustering";
import type { Face } from "./faceClustering";
import { convexHull2D, polygon2DArea } from "./geom";
import { getMaterialLayers } from "./materials";
import { getElementTriangles, getLineValue, vecToArray } from "./meshExtract";
import { classifyRay, SpatialGrid } from "./raycast";
import type { TaggedTriangle } from "./raycast";
import {
  findBestPhysicalBoundaryForFace,
  groupPhysicalBoundariesBySpace,
  parseSpaceBoundaries,
} from "./spaceBoundaries";
import type { ParsedSpaceBoundary } from "./spaceBoundaries";
import type {
  FaceZone,
  GeometricOrientation,
  HostCategory,
  HostElementRef,
  MaterialLayerSet,
  MixedSplit,
  NeighbourSpaceRef,
  ProgressCallback,
  ProgressPhase,
  ReconstructedFace,
  ReconstructedSpace,
  ReconstructionOptions,
  ReconstructionResult,
  StoreyRef,
  Triangle,
  Vec3,
  ZoneTotals,
} from "./types";

const HOST_MAX_DIST_MM = 1500;
const TOTAL_MAX_DIST_MM = 5000;
const RAY_EPS_MM = 2;
const GRID_CELL_MM = 500;
const RAY_STEP_MM = 250;
const SAMPLE_SPACING_MM = 250;
const MIN_SAMPLES_PER_FACE = 5;

const OPAQUE_TYPES = new Set(["IFCWALL", "IFCSLAB", "IFCROOF", "IFCCOVERING"]);
const ELEMENT_TYPE_NAMES = ["IFCWALL", "IFCSLAB", "IFCROOF", "IFCCOVERING", "IFCWINDOW", "IFCDOOR"] as const;
const ELEMENT_TYPE_CODES: Record<(typeof ELEMENT_TYPE_NAMES)[number], number> = {
  IFCWALL: WebIfc.IFCWALL,
  IFCSLAB: WebIfc.IFCSLAB,
  IFCROOF: WebIfc.IFCROOF,
  IFCCOVERING: WebIfc.IFCCOVERING,
  IFCWINDOW: WebIfc.IFCWINDOW,
  IFCDOOR: WebIfc.IFCDOOR,
};

/** Human-readable (Dutch, for UI display) explanation of each hybrid QC flag. */
const QC_REASON: Record<string, string> = {
  "sb-raycast-match": "SB-boundary en raycast wijzen naar dezelfde host — hoogste vertrouwen.",
  "sb-raycast-mismatch":
    "SB-boundary en raycast wijzen naar verschillende hosts; SB-host heeft voorrang (granulariteitsverschil, geen fout — zie fase-1-rapport §2).",
  "sb-only-no-raycast-host": "Alleen een SB-boundary-match; raycast vond geen host op dit vlak.",
  "raycast-only-no-sb": "Geen gezonde SB-boundary-match; raycast-fallback gebruikt als host.",
  "no-host-found": "Geen host gevonden, noch via SB-boundary, noch via raycast.",
};

function mode<T>(arr: readonly T[]): { value: T; count: number; total: number } | null {
  if (!arr.length) return null;
  const counts = new Map<T, number>();
  for (const v of arr) counts.set(v, (counts.get(v) ?? 0) + 1);
  let best: T | null = null;
  let bestCount = -1;
  for (const [v, c] of counts) {
    if (c > bestCount) {
      best = v;
      bestCount = c;
    }
  }
  return best === null ? null : { value: best, count: bestCount, total: arr.length };
}

function median(arr: readonly (number | null)[]): number | null {
  const a = arr.filter((x): x is number => x != null).sort((x, y) => x - y);
  if (!a.length) return null;
  const mid = Math.floor(a.length / 2);
  return a.length % 2 ? a[mid]! : (a[mid - 1]! + a[mid]!) / 2;
}

/** Both-case check: the exact casing of GetNameFromTypeCode's output wasn't
 * pinned down with certainty during the PoC build (see RAPPORT-fase1.md) --
 * kept defensive rather than assuming one casing. */
function hostCategory(ifcType: string | null): HostCategory {
  if (ifcType === "IFCWINDOW" || ifcType === "IfcWindow") return "raam";
  if (ifcType === "IFCDOOR" || ifcType === "IfcDoor") return "deur";
  if (!ifcType) return "geen-host";
  return "opaak"; // opaque types + fallback: unknown-but-present host treated as opaque
}

function isRoofHost(ifcType: string | null): boolean {
  return ifcType === "IFCROOF" || ifcType === "IfcRoof";
}

function isOutdoorPseudoSpace(name: string | null): boolean {
  return typeof name === "string" && /buiten/i.test(name);
}

interface ElementInfo {
  id: number;
  ifcType: string;
  name: string | null;
  materials: MaterialLayerSet | null;
}

interface SpaceInfo {
  id: number;
  name: string | null;
  longName: string | null;
  triangles: Triangle[];
  faces: Face[];
}

function report(onProgress: ProgressCallback | undefined, phase: ProgressPhase, percent: number, message?: string): void {
  onProgress?.({ phase, percent, message });
}

/**
 * Run the full reconstruction pipeline against an IFC file buffer.
 *
 * @param buffer Raw IFC file bytes.
 * @param opts Reconstruction options (maaiveld override).
 * @param onProgress Optional progress callback, called at each pipeline phase.
 */
export async function reconstructFromIfc(
  buffer: ArrayBuffer | Uint8Array,
  opts: ReconstructionOptions = {},
  onProgress?: ProgressCallback,
): Promise<ReconstructionResult> {
  const api = new WebIfc.IfcAPI();
  api.SetWasmPath("/wasm/");
  await api.Init();

  report(onProgress, "opening", 0, "Model openen");
  const data = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
  const modelID = api.OpenModel(data, { COORDINATE_TO_ORIGIN: false });
  if (modelID < 0) {
    throw new Error("web-ifc OpenModel gaf een ongeldig modelID terug — bestand niet geopend.");
  }

  try {
    const schema = api.GetModelSchema(modelID);

    // -----------------------------------------------------------------------
    // 1. Storeys + maaiveld
    // -----------------------------------------------------------------------
    // IMPORTANT (see RAPPORT-fase1.md "maaiveld-frame"): IFCBUILDINGSTOREY.
    // Elevation is relative to the BUILDING's own local placement origin,
    // NOT the same absolute frame the raycast geometry lives in (world
    // Z-up mm, derived from FlatMesh.flatTransformation — see
    // geom.ts::toZUpMM). The two frames differ by a constant per-model
    // offset that can be large enough to silently break every grond/gemengd
    // classification. Fix: read the storey's OWN ObjectPlacement world Z
    // (native Z-up mm) instead of trusting the Elevation attribute directly.
    const storeyIds = vecToArray(api.GetLineIDsWithType(modelID, WebIfc.IFCBUILDINGSTOREY, false));
    const storeys = storeyIds
      .map((id) => {
        const line = api.GetLine(modelID, id) as { ObjectPlacement?: { value?: number } } | null;
        const placementId = line?.ObjectPlacement?.value;
        const worldZMM = placementId != null ? api.GetWorldTransformMatrix(modelID, placementId)[14]! : null;
        return { id, name: getLineValue(line, "Name") as string | null, worldZMM };
      })
      .sort((a, b) => (a.worldZMM ?? 0) - (b.worldZMM ?? 0));

    let maaiveldMM: number;
    let maaiveldSource: string;
    if (opts.maaiveldMM != null) {
      maaiveldMM = opts.maaiveldMM;
      maaiveldSource = `expliciete optie-override (${opts.maaiveldMM} mm, wereld Z-up mm)`;
    } else {
      const peilStorey = storeys.find((s) => /peil/i.test(s.name ?? ""));
      if (peilStorey && peilStorey.worldZMM != null) {
        maaiveldMM = peilStorey.worldZMM;
        maaiveldSource = `storey "${peilStorey.name}" (placement world Z ${peilStorey.worldZMM.toFixed(1)} mm)`;
      } else {
        maaiveldMM = 0;
        maaiveldSource = "default 0 (geen storey met 'peil' in de naam gevonden)";
      }
    }

    // -----------------------------------------------------------------------
    // 2. Building element meshes + materials
    // -----------------------------------------------------------------------
    report(onProgress, "elements", 10, "Bouwelementen inlezen");
    const elementInfo = new Map<number, ElementInfo>();
    const elementTagged: TaggedTriangle[] = [];

    for (const tn of ELEMENT_TYPE_NAMES) {
      const code = ELEMENT_TYPE_CODES[tn];
      const ids = vecToArray(api.GetLineIDsWithType(modelID, code, true));
      for (const id of ids) {
        const line = api.GetLine(modelID, id);
        const tris = getElementTriangles(api, modelID, id);
        const materials = OPAQUE_TYPES.has(tn) ? await getMaterialLayers(api, modelID, id) : null;
        elementInfo.set(id, {
          id,
          ifcType: tn,
          name: getLineValue(line, "Name") as string | null,
          materials,
        });
        for (const tri of tris) elementTagged.push({ tag: { kind: "element", id, ifcType: tn }, tri });
      }
    }

    // -----------------------------------------------------------------------
    // 3. Space meshes (connectivity-aware face clustering)
    // -----------------------------------------------------------------------
    report(onProgress, "spaces", 35, "Ruimtes clusteren");
    const spaceIdsAll = vecToArray(api.GetLineIDsWithType(modelID, WebIfc.IFCSPACE, false));
    const spaceInfo = new Map<number, SpaceInfo>();
    const spaceTagged: TaggedTriangle[] = [];
    const outdoorPseudoSpaceIds = new Set<number>();

    for (const id of spaceIdsAll) {
      const line = api.GetLine(modelID, id);
      const name = getLineValue(line, "Name") as string | null;
      const longName = getLineValue(line, "LongName") as string | null;
      const tris = getElementTriangles(api, modelID, id);
      const faces = groupPlanarFacesConnected(tris);
      spaceInfo.set(id, { id, name, longName, triangles: tris, faces });
      for (const tri of tris) spaceTagged.push({ tag: { kind: "space", id }, tri });
      if (isOutdoorPseudoSpace(name) || isOutdoorPseudoSpace(longName)) outdoorPseudoSpaceIds.add(id);
    }
    const spaceIds = spaceIdsAll.filter((id) => !outdoorPseudoSpaceIds.has(id));

    // Per-space storey assignment: nearest storey by world Z (same
    // placement-matrix convention as maaiveld above). The PoC never needed
    // this (it only tracked storeys building-wide); added here because the
    // per-face result model reports a storey. Deviation from the PoC, see
    // final report.
    const spaceStorey = new Map<number, StoreyRef | null>();
    for (const id of spaceIds) {
      const line = api.GetLine(modelID, id) as { ObjectPlacement?: { value?: number } } | null;
      const placementId = line?.ObjectPlacement?.value;
      const worldZMM = placementId != null ? api.GetWorldTransformMatrix(modelID, placementId)[14]! : null;
      if (worldZMM == null || storeys.length === 0) {
        spaceStorey.set(id, null);
        continue;
      }
      let best = storeys[0]!;
      let bestDist = Infinity;
      for (const s of storeys) {
        if (s.worldZMM == null) continue;
        const dist = Math.abs(s.worldZMM - worldZMM);
        if (dist < bestDist) {
          bestDist = dist;
          best = s;
        }
      }
      spaceStorey.set(id, { id: best.id, name: best.name });
    }

    // -----------------------------------------------------------------------
    // 4. Spatial grid (perf) + space boundaries
    // -----------------------------------------------------------------------
    report(onProgress, "grid", 55, "Ruimtelijke index + space boundaries");
    const allTagged = elementTagged.concat(spaceTagged);
    const grid = new SpatialGrid(allTagged, GRID_CELL_MM);
    const sbData = parseSpaceBoundaries(api, modelID, WebIfc.IFCRELSPACEBOUNDARY);
    const physicalBoundariesBySpace = groupPhysicalBoundariesBySpace(sbData.boundaries);

    // -----------------------------------------------------------------------
    // 5. Per-space, per-face classification: hybrid SB/raycast host resolution
    // -----------------------------------------------------------------------
    report(onProgress, "classifying", 60, "Vlakken classificeren");
    const resultSpaces: ReconstructedSpace[] = [];

    for (let spaceIdx = 0; spaceIdx < spaceIds.length; spaceIdx++) {
      const spaceId = spaceIds[spaceIdx]!;
      const sp = spaceInfo.get(spaceId)!;
      const storeyRef = spaceStorey.get(spaceId) ?? null;
      const faceResults: ReconstructedFace[] = [];

      for (const face of sp.faces) {
        faceResults.push(
          classifyFace({
            face,
            spaceId,
            storeyRef,
            elementInfo,
            physicalBoundariesBySpace,
            grid,
            outdoorPseudoSpaceIds,
            spaceInfo,
            maaiveldMM,
          }),
        );
      }

      const xyPoints: [number, number][] = sp.triangles
        .flatMap((t) => [t.p0, t.p1, t.p2])
        .map((p) => [p[0], p[1]] as [number, number]);
      const hull = convexHull2D(xyPoints);
      const footprintM2 = polygon2DArea(hull) / 1e6;

      const zoneTotals = buildZoneTotals(faceResults);
      const floorAreaM2 = faceResults.filter((f) => f.zone === "vloer").reduce((s, f) => s + f.grossAreaM2, 0);

      resultSpaces.push({
        id: spaceId,
        name: sp.name,
        longName: sp.longName,
        storey: storeyRef,
        faces: faceResults,
        floorAreaM2: Number(floorAreaM2.toFixed(2)),
        footprintEstimateM2: Number(footprintM2.toFixed(2)),
        zoneTotals,
      });

      if (spaceIds.length > 0) {
        const pct = 60 + Math.round((60 * (spaceIdx + 1)) / spaceIds.length);
        report(onProgress, "classifying", Math.min(pct, 95), `Ruimte ${spaceIdx + 1}/${spaceIds.length}`);
      }
    }

    report(onProgress, "done", 100, "Klaar");

    return {
      meta: {
        ifcSchema: schema,
        generatedAt: new Date().toISOString(),
        hostMaxDistMM: HOST_MAX_DIST_MM,
        totalMaxDistMM: TOTAL_MAX_DIST_MM,
        gridCellMM: GRID_CELL_MM,
        maaiveldMM,
        maaiveldSource,
      },
      storeys: storeys.map((s) => ({ id: s.id, name: s.name })),
      outdoorPseudoSpaces: [...outdoorPseudoSpaceIds].map((id) => ({
        id,
        name: spaceInfo.get(id)?.name ?? null,
      })),
      spaces: resultSpaces,
    };
  } finally {
    api.CloseModel(modelID);
  }
}

interface ZoneAccumulator {
  opaak: number;
  raam: number;
  deur: number;
  geenHost: number;
}

function buildZoneTotals(faces: readonly ReconstructedFace[]): Partial<Record<FaceZone, ZoneTotals>> {
  const byZone = new Map<FaceZone, ZoneAccumulator>();
  for (const f of faces) {
    let cats = byZone.get(f.zone);
    if (!cats) {
      cats = { opaak: 0, raam: 0, deur: 0, geenHost: 0 };
      byZone.set(f.zone, cats);
    }
    if (f.hostCategory === "opaak") cats.opaak += f.grossAreaM2;
    else if (f.hostCategory === "raam") cats.raam += f.grossAreaM2;
    else if (f.hostCategory === "deur") cats.deur += f.grossAreaM2;
    else cats.geenHost += f.grossAreaM2;
  }
  const zoneTotals: Partial<Record<FaceZone, ZoneTotals>> = {};
  for (const [zone, cats] of byZone) {
    zoneTotals[zone] = {
      opaqueM2: Number(cats.opaak.toFixed(2)),
      windowM2: Number(cats.raam.toFixed(2)),
      doorM2: Number(cats.deur.toFixed(2)),
      noHostM2: Number(cats.geenHost.toFixed(2)),
      netM2: Number(cats.opaak.toFixed(2)),
      grossM2: Number((cats.opaak + cats.raam + cats.deur + cats.geenHost).toFixed(2)),
    };
  }
  return zoneTotals;
}

interface ClassifyFaceArgs {
  face: Face;
  spaceId: number;
  storeyRef: StoreyRef | null;
  elementInfo: Map<number, ElementInfo>;
  physicalBoundariesBySpace: Map<number, ParsedSpaceBoundary[]>;
  grid: SpatialGrid;
  outdoorPseudoSpaceIds: Set<number>;
  spaceInfo: Map<number, SpaceInfo>;
  maaiveldMM: number;
}

function classifyFace(args: ClassifyFaceArgs): ReconstructedFace {
  const { face, spaceId, storeyRef, elementInfo, physicalBoundariesBySpace, grid, outdoorPseudoSpaceIds, spaceInfo, maaiveldMM } = args;

  const geomOrientation: GeometricOrientation = classifyOrientation(face.normal);
  const areaM2 = face.area / 1e6;

  // (a) SB-PHYSICAL match for this face, if any.
  const sbMatch = findBestPhysicalBoundaryForFace(face, spaceId, physicalBoundariesBySpace);

  // (b) raycast classification (grid-filtered candidates).
  const samples = sampleFacePoints(face, SAMPLE_SPACING_MM, MIN_SAMPLES_PER_FACE);
  const votes = samples.map(({ point }) => {
    const origin: Vec3 = [
      point[0] + face.normal[0] * RAY_EPS_MM,
      point[1] + face.normal[1] * RAY_EPS_MM,
      point[2] + face.normal[2] * RAY_EPS_MM,
    ];
    const candidates = grid
      .queryRay(origin, face.normal, TOTAL_MAX_DIST_MM, RAY_STEP_MM)
      .filter((tt) => !(tt.tag.kind === "space" && tt.tag.id === spaceId));
    return classifyRay(origin, face.normal, candidates, {
      hostMaxDist: HOST_MAX_DIST_MM,
      totalMaxDist: TOTAL_MAX_DIST_MM,
    });
  });

  const classMode = mode(votes.map((v) => v.classification));
  const hostMode = mode(votes.map((v) => v.host).filter((h): h is number => h != null));
  const hostVotes = votes.filter((v) => v.host === hostMode?.value);
  const thicknessSamples = hostVotes.map((v) => v.thicknessMM).filter((v): v is number => v != null);
  const thicknessMedian = median(thicknessSamples);
  const raycastHostInfo = hostMode != null ? elementInfo.get(hostMode.value) : null;
  const raycastHostEl: HostElementRef | null = raycastHostInfo
    ? { id: raycastHostInfo.id, name: raycastHostInfo.name, ifcType: raycastHostInfo.ifcType }
    : null;

  let neighbourSpaceId: number | "PSEUDO_BUITEN" | null = null;
  let neighbourSpaceName: string | null = null;
  let isPseudoBuiten = false;
  if (classMode != null && typeof classMode.value === "string" && classMode.value.startsWith("space:")) {
    const rawNeighbourId = Number(classMode.value.split(":")[1]);
    // Model quirk: "outside" is sometimes modelled as its own IfcSpace
    // ("Buiten"/"buiten"). Semantically that IS exterior, not an interior
    // neighbour -- flag it so classification falls through to the same
    // exterior/grond/gemengd logic as a "no space hit" raycast result,
    // instead of being reported as "buurruimte".
    isPseudoBuiten = outdoorPseudoSpaceIds.has(rawNeighbourId);
    if (isPseudoBuiten) {
      neighbourSpaceId = "PSEUDO_BUITEN";
      neighbourSpaceName = spaceInfo.get(rawNeighbourId)?.name ?? "Buiten";
    } else {
      neighbourSpaceId = rawNeighbourId;
      neighbourSpaceName = spaceInfo.get(rawNeighbourId)?.longName ?? spaceInfo.get(rawNeighbourId)?.name ?? null;
    }
  }
  const realNeighbourSpaceId = isPseudoBuiten ? null : neighbourSpaceId;

  // (c) hybrid resolution: SB primary if healthy match, else raycast fallback.
  const sbBoundary = sbMatch?.boundary ?? null;
  const sbHostEl: HostElementRef | null = sbBoundary?.hostElement
    ? { id: sbBoundary.hostElement.id, name: sbBoundary.hostElement.name, ifcType: sbBoundary.hostElement.ifcType }
    : null;

  let hostSource: ReconstructedFace["hostSource"];
  let finalHost: HostElementRef | null;
  let qcFlag: string;
  if (sbHostEl) {
    hostSource = "sb";
    finalHost = sbHostEl;
    qcFlag = raycastHostEl
      ? raycastHostEl.id === sbHostEl.id
        ? "sb-raycast-match"
        : "sb-raycast-mismatch"
      : "sb-only-no-raycast-host";
  } else if (raycastHostEl) {
    hostSource = "raycast";
    finalHost = raycastHostEl;
    qcFlag = "raycast-only-no-sb";
  } else {
    hostSource = "none";
    finalHost = null;
    qcFlag = "no-host-found";
  }
  const finalHostIfcType = finalHost?.ifcType ?? raycastHostEl?.ifcType ?? null;
  const category = hostCategory(finalHostIfcType);

  // (d) envelope zone: geometric orientation, with IfcRoof host override to
  // "dak" -- but never for a downward-facing (vloer) surface, since a
  // roof-typed element can legitimately be the FLOOR of a room above it
  // (roof terrace / walkable deck).
  let zone: FaceZone = geomOrientation;
  if (isRoofHost(finalHostIfcType) && geomOrientation !== "vloer") zone = "dak";

  // (e) ground/mixed (maaiveld split), excluded for raam/deur hosts.
  // IMPORTANT (see RAPPORT-fase1.md "maaiveld-frame"): a "vloer" face is the
  // room's own INSIDE (top) surface of the floor construction, which sits
  // AT/ABOVE the storey's "peil" reference by definition -- the actual
  // ground-contact surface is the UNDERSIDE, i.e. peil minus the build-up
  // thickness. So for zone==="vloer" only, shift the test point down by
  // that thickness before comparing to maaiveldMM.
  const materialLayersForFace = finalHost ? elementInfo.get(finalHost.id)?.materials ?? null : null;
  const groundTestShiftMM = zone === "vloer" ? materialLayersForFace?.totalThicknessMM ?? thicknessMedian ?? 0 : 0;

  let classification: ReconstructedFace["classification"];
  let mixedSplit: MixedSplit | null = null;
  const isExterior = realNeighbourSpaceId == null;
  if (realNeighbourSpaceId != null) {
    classification = "buurruimte";
  } else if (isExterior) {
    if (category === "raam" || category === "deur") {
      classification = "exterieur";
    } else {
      const groundSplit = classifyGroundSplit(face.triangles, maaiveldMM, groundTestShiftMM);
      classification = groundSplit.classification;
      mixedSplit = groundSplit.mixedSplit;
    }
  } else {
    classification = "onbepaald";
  }

  const neighbourSpace: NeighbourSpaceRef | null =
    neighbourSpaceId != null ? { id: neighbourSpaceId, name: neighbourSpaceName } : null;

  const netAreaM2 = category === "opaak" ? Number(areaM2.toFixed(3)) : 0;

  return {
    zone,
    geometricOrientation: geomOrientation,
    normal: face.normal.map((n) => Number(n.toFixed(4))) as Vec3,
    centroidMM: face.centroid,
    grossAreaM2: Number(areaM2.toFixed(3)),
    netAreaM2,
    hostCategory: category,
    classification,
    neighbourSpace,
    mixedSplit,
    hostElement: finalHost,
    hostSource,
    qcFlag,
    qcReason: QC_REASON[qcFlag] ?? "Onbekende QC-status.",
    materialLayers: materialLayersForFace,
    measuredThicknessMM: thicknessMedian != null ? Number(thicknessMedian.toFixed(1)) : null,
    storey: storeyRef,
    sbInternalOrExternal: sbBoundary?.internalOrExternal ?? null,
    sbAreaM2: sbBoundary?.areaMM2 != null ? Number((sbBoundary.areaMM2 / 1e6).toFixed(3)) : null,
    raycastHostElement: raycastHostEl,
    sampleCount: votes.length,
    voteAgreement: classMode ? Number((classMode.count / classMode.total).toFixed(2)) : 0,
  };
}
