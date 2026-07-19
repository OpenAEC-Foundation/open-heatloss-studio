/**
 * Result model for the IFC space/wall reconstruction pipeline.
 *
 * Ported from the fase-1 PoC (see orchestrator session notes,
 * `RAPPORT-fase1.md`). Ground truth for every field below is the PoC's
 * `run-fase1.mjs` per-face record — field names here are the English
 * equivalents (this repo's established convention, see
 * `components/modeller/ifc-import.ts`) of the Dutch vocabulary used when the
 * work was scoped.
 */

/** 3D vector / point, always millimetres, world Z-up frame (see geom.ts). */
export type Vec3 = [number, number, number];

/** One triangle of an extracted element/space mesh, world Z-up mm. */
export interface Triangle {
  p0: Vec3;
  p1: Vec3;
  p2: Vec3;
  normal: Vec3;
  /** mm^2. */
  area: number;
  /** IFC express ID of the element/space this triangle was extracted from. */
  sourceId: number;
}

/** Geometric orientation from the (Z-up) face normal, pyrevit-scanner convention. */
export type GeometricOrientation = "wand" | "vloer" | "plafond";

/** Reported envelope zone — same as GeometricOrientation, plus "dak" when the
 * resolved host element is an IfcRoof (see pipeline.ts `resolveZone`). */
export type FaceZone = GeometricOrientation | "dak";

/** Category of the resolved host element for a face. */
export type HostCategory = "opaak" | "raam" | "deur" | "geen-host";

/** Final per-face classification (adjacency / envelope). */
export type FaceClassification =
  | "exterieur"
  | "grond"
  | "gemengd"
  | "buurruimte"
  | "onbepaald";

/** Which source produced the reported host element for a face. */
export type HostSource = "sb" | "raycast" | "none";

/** SB InternalOrExternalBoundary enumeration (IFC schema values, passed through). */
export type SbInternalOrExternal = "INTERNAL" | "EXTERNAL" | "NOTDEFINED";

export interface MaterialLayer {
  name: string | null;
  materialName: string | null;
  thicknessMM: number | null;
}

export interface MaterialLayerSet {
  layerSetName: string | null;
  layers: MaterialLayer[];
  totalThicknessMM: number | null;
}

export interface HostElementRef {
  id: number;
  name: string | null;
  ifcType: string | null;
}

export interface NeighbourSpaceRef {
  /** "PSEUDO_BUITEN" for a raycast hit on a modelled outdoor pseudo-space
   * (see pipeline.ts `isOutdoorPseudoSpace`) — treated as exterior, never
   * reported as a real adjacency. */
  id: number | "PSEUDO_BUITEN";
  name: string | null;
}

export interface MixedSplit {
  groundM2: number;
  exteriorM2: number;
}

export interface StoreyRef {
  id: number;
  name: string | null;
}

/**
 * One reconstructed, connectivity-clustered planar face of a space's
 * boundary mesh, with its hybrid SB/raycast host resolution and envelope
 * classification.
 */
export interface ReconstructedFace {
  zone: FaceZone;
  geometricOrientation: GeometricOrientation;
  normal: Vec3;
  centroidMM: Vec3;
  /** This face's own area. Equal to grossAreaM2 for every face — the
   * netto/bruto distinction only becomes meaningful when aggregated per
   * zone across opaque/window/door faces, see ReconstructedSpace.zoneTotals.
   * Kept per-face for direct UI display: netAreaM2 is 0 for non-opaque
   * hostCategory so a naive per-face sum still yields a correct net total. */
  grossAreaM2: number;
  netAreaM2: number;
  hostCategory: HostCategory;
  classification: FaceClassification;
  neighbourSpace: NeighbourSpaceRef | null;
  mixedSplit: MixedSplit | null;
  hostElement: HostElementRef | null;
  hostSource: HostSource;
  /** Raw QC flag, e.g. "sb-raycast-mismatch" — see pipeline.ts QC_REASON. */
  qcFlag: string;
  /** Human-readable (Dutch) explanation of qcFlag for UI display. */
  qcReason: string;
  materialLayers: MaterialLayerSet | null;
  measuredThicknessMM: number | null;
  /** The space's own storey (nearest by world Z), repeated per face for
   * convenience — see pipeline.ts deviation note in the final report. */
  storey: StoreyRef | null;
  sbInternalOrExternal: SbInternalOrExternal | null;
  sbAreaM2: number | null;
  raycastHostElement: HostElementRef | null;
  sampleCount: number;
  /** Fraction (0-1) of per-sample raycast votes agreeing with the mode
   * classification for this face — low values flag a noisy/ambiguous face. */
  voteAgreement: number;
}

export interface ZoneTotals {
  opaqueM2: number;
  windowM2: number;
  doorM2: number;
  noHostM2: number;
  netM2: number;
  grossM2: number;
}

export interface ReconstructedSpace {
  id: number;
  name: string | null;
  longName: string | null;
  storey: StoreyRef | null;
  faces: ReconstructedFace[];
  floorAreaM2: number;
  /** Convex-hull-of-floor-mesh area — an independent sanity cross-check,
   * NOT a second source of truth (always >= floorAreaM2 for non-convex
   * rooms, see RAPPORT-fase1.md §3.2). */
  footprintEstimateM2: number;
  zoneTotals: Partial<Record<FaceZone, ZoneTotals>>;
}

export interface ReconstructionMeta {
  ifcSchema: string;
  generatedAt: string;
  maaiveldMM: number;
  maaiveldSource: string;
  hostMaxDistMM: number;
  totalMaxDistMM: number;
  gridCellMM: number;
}

export interface ReconstructionResult {
  meta: ReconstructionMeta;
  storeys: StoreyRef[];
  outdoorPseudoSpaces: { id: number; name: string | null }[];
  spaces: ReconstructedSpace[];
}

export interface ReconstructionOptions {
  /** Explicit maaiveld override, world Z-up mm (same frame as StoreyRef
   * placement Z). Default: the storey with "peil" (case-insensitive) in its
   * name, else 0. */
  maaiveldMM?: number;
}

export type ProgressPhase =
  | "opening"
  | "elements"
  | "spaces"
  | "grid"
  | "spaceBoundaries"
  | "classifying"
  | "done";

export interface ProgressEvent {
  phase: ProgressPhase;
  /** 0-100. */
  percent: number;
  message?: string;
}

export type ProgressCallback = (event: ProgressEvent) => void;
