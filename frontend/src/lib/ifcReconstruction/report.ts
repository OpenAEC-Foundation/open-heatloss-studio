/**
 * Fase 2b — oppervlaktenlijst + vergelijking met de bestaande (pyrevit)
 * warmteverlies-import, voor de "IFC-reconstructie (bèta)"-pagina.
 *
 * Pure/testable helpers only — no DOM, no React. Consumes the fase-2a
 * `ReconstructionResult` (see `./types.ts`) READ-ONLY; nothing here mutates
 * or reaches into the pipeline modules.
 *
 * NOTE on `ReconstructedFace` geometry: the fase-2a result model exposes only
 * a scalar `grossAreaM2`/`netAreaM2` + `centroidMM` + `normal` per face — the
 * clustered face's own boundary polygon is NOT part of the public result
 * (see `pipeline.ts::classifyFace`, which discards `face.triangles` after
 * classification). The 3D viewer therefore approximates each face as a flat
 * square centred on `centroidMM`; that approximation lives in the viewer
 * component, not here.
 */
import type {
  FaceClassification,
  FaceZone,
  HostCategory,
  ReconstructedFace,
  ReconstructedSpace,
  ReconstructionResult,
} from "./types";

// =============================================================================
// 1. Flat per-face rows (table + CSV + 3D selection key)
// =============================================================================

export interface FlatFaceRow {
  /** `${spaceIndex}:${faceIndex}` — stable key for table<->3D selection sync. */
  rowKey: string;
  spaceIndex: number;
  faceIndex: number;
  spaceId: number;
  spaceName: string;
  zone: FaceZone;
  classification: FaceClassification;
  hostCategory: HostCategory;
  grossAreaM2: number;
  netAreaM2: number;
  hostName: string;
  hostSource: string;
  qcFlag: string;
  qcReason: string;
  qcFlagged: boolean;
  storeyName: string;
}

/** Hard QC flags from `pipeline.ts::QC_REASON` that mean "resolution is not
 * trustworthy" (as opposed to e.g. a clean SB-only match). */
const HARD_QC_FLAGS = new Set(["sb-raycast-mismatch", "no-host-found"]);

/** A face is flagged for review when its host resolution used a fallback/
 * mismatch path, or when the raycast vote was noisy (< 60% agreement). Vote
 * agreement is 0 for SB-resolved faces with no raycast votes at all, so it is
 * only applied when there WERE raycast samples (sampleCount > 0). */
export function isQcFlagged(face: Pick<ReconstructedFace, "qcFlag" | "voteAgreement" | "sampleCount">): boolean {
  if (HARD_QC_FLAGS.has(face.qcFlag)) return true;
  if (face.sampleCount > 0 && face.voteAgreement < 0.6) return true;
  return false;
}

/** Flatten every space/face pair in a `ReconstructionResult` into display rows. */
export function flattenFaces(result: ReconstructionResult): FlatFaceRow[] {
  const rows: FlatFaceRow[] = [];
  result.spaces.forEach((space, spaceIndex) => {
    space.faces.forEach((face, faceIndex) => {
      rows.push({
        rowKey: `${spaceIndex}:${faceIndex}`,
        spaceIndex,
        faceIndex,
        spaceId: space.id,
        spaceName: space.name ?? space.longName ?? `Ruimte ${space.id}`,
        zone: face.zone,
        classification: face.classification,
        hostCategory: face.hostCategory,
        grossAreaM2: face.grossAreaM2,
        netAreaM2: face.netAreaM2,
        hostName: face.hostElement?.name ?? "",
        hostSource: face.hostSource,
        qcFlag: face.qcFlag,
        qcReason: face.qcReason,
        qcFlagged: isQcFlagged(face),
        storeyName: face.storey?.name ?? space.storey?.name ?? "",
      });
    });
  });
  return rows;
}

// =============================================================================
// 2. Per-space totals (category + classification), for the table's room rows
// =============================================================================

export interface SpaceCategoryTotals {
  opaakM2: number;
  raamM2: number;
  deurM2: number;
}

export interface SpaceClassificationTotals {
  exterieurM2: number;
  grondM2: number;
  buurruimteM2: number;
  gemengdM2: number;
  onbepaaldM2: number;
}

export function spaceCategoryTotals(space: ReconstructedSpace): SpaceCategoryTotals {
  const totals: SpaceCategoryTotals = { opaakM2: 0, raamM2: 0, deurM2: 0 };
  for (const f of space.faces) {
    if (f.hostCategory === "opaak") totals.opaakM2 += f.grossAreaM2;
    else if (f.hostCategory === "raam") totals.raamM2 += f.grossAreaM2;
    else if (f.hostCategory === "deur") totals.deurM2 += f.grossAreaM2;
  }
  totals.opaakM2 = round2(totals.opaakM2);
  totals.raamM2 = round2(totals.raamM2);
  totals.deurM2 = round2(totals.deurM2);
  return totals;
}

export function spaceClassificationTotals(space: ReconstructedSpace): SpaceClassificationTotals {
  const totals: SpaceClassificationTotals = {
    exterieurM2: 0,
    grondM2: 0,
    buurruimteM2: 0,
    gemengdM2: 0,
    onbepaaldM2: 0,
  };
  for (const f of space.faces) {
    if (f.classification === "exterieur") totals.exterieurM2 += f.grossAreaM2;
    else if (f.classification === "grond") totals.grondM2 += f.grossAreaM2;
    else if (f.classification === "buurruimte") totals.buurruimteM2 += f.grossAreaM2;
    else if (f.classification === "gemengd") totals.gemengdM2 += f.grossAreaM2;
    else totals.onbepaaldM2 += f.grossAreaM2;
  }
  totals.exterieurM2 = round2(totals.exterieurM2);
  totals.grondM2 = round2(totals.grondM2);
  totals.buurruimteM2 = round2(totals.buurruimteM2);
  totals.gemengdM2 = round2(totals.gemengdM2);
  totals.onbepaaldM2 = round2(totals.onbepaaldM2);
  return totals;
}

function round2(n: number): number {
  return Math.round(n * 100) / 100;
}

// =============================================================================
// 3. CSV / clipboard serialisation of the flat face list
// =============================================================================

const CSV_HEADER = [
  "ruimte",
  "vlak",
  "oriëntatie",
  "classificatie",
  "categorie",
  "bruto_m2",
  "netto_m2",
  "host",
  "bron",
  "qc",
] as const;

function csvEscape(value: string): string {
  if (/[";\n]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

function faceRowToCsvFields(row: FlatFaceRow, faceLabel: string): string[] {
  return [
    row.spaceName,
    faceLabel,
    row.zone,
    row.classification,
    row.hostCategory,
    row.grossAreaM2.toFixed(2),
    row.netAreaM2.toFixed(2),
    row.hostName,
    row.hostSource,
    row.qcFlagged ? row.qcFlag : "",
  ];
}

/** Serialise the flat face list to `;`-delimited CSV (NL locale convention —
 * Excel-NL treats `,` as decimal separator, so `;` is the safe field
 * delimiter). One row per face; `vlak` is a 1-based index within its room. */
export function serializeFacesToCsv(rows: readonly FlatFaceRow[]): string {
  const lines = [CSV_HEADER.join(";")];
  const faceCounters = new Map<number, number>();
  for (const row of rows) {
    const n = (faceCounters.get(row.spaceIndex) ?? 0) + 1;
    faceCounters.set(row.spaceIndex, n);
    lines.push(faceRowToCsvFields(row, String(n)).map(csvEscape).join(";"));
  }
  return lines.join("\n");
}

/** Tab-separated variant for clipboard paste into Excel/Sheets — same column
 * order as `serializeFacesToCsv`, tabs instead of `;` so pasted cells split
 * without needing "text to columns". */
export function serializeFacesToClipboardText(rows: readonly FlatFaceRow[]): string {
  const lines = [CSV_HEADER.join("\t")];
  const faceCounters = new Map<number, number>();
  for (const row of rows) {
    const n = (faceCounters.get(row.spaceIndex) ?? 0) + 1;
    faceCounters.set(row.spaceIndex, n);
    lines.push(faceRowToCsvFields(row, String(n)).join("\t"));
  }
  return lines.join("\n");
}

// =============================================================================
// 4. Colour mapping (classification / hostCategory / QC -> UI colour)
// =============================================================================

export interface FaceColor {
  /** Primary hex fill colour (no `#`... actually WITH `#`, CSS-ready). */
  fill: string;
  /** Present only for "gemengd" — the second colour for a 2-tone/stripe render. */
  fillSecondary?: string;
  /** True when the face should get the QC (red outline/hatch) treatment. */
  qcFlagged: boolean;
}

const COLOR_EXTERIEUR = "#0d9488"; // teal-600
const COLOR_GROND = "#78350f"; // amber-900 (petrol/bruin)
const COLOR_BUURRUIMTE = "#6ee7b7"; // emerald-300 (mint)
const COLOR_ONBEPAALD = "#9ca3af"; // gray-400
const COLOR_RAAM = "#3b82f6"; // blue-500 — same window blue as FloorCanvas3D
const COLOR_DEUR = "#8b5cf6"; // violet-500
const COLOR_QC_OUTLINE = "#ef4444"; // red-500

export const FACE_LEGEND_COLORS = {
  exterieur: COLOR_EXTERIEUR,
  grond: COLOR_GROND,
  buurruimte: COLOR_BUURRUIMTE,
  onbepaald: COLOR_ONBEPAALD,
  raam: COLOR_RAAM,
  deur: COLOR_DEUR,
  qcOutline: COLOR_QC_OUTLINE,
} as const;

function classificationColor(classification: FaceClassification): string {
  switch (classification) {
    case "exterieur":
      return COLOR_EXTERIEUR;
    case "grond":
      return COLOR_GROND;
    case "buurruimte":
      return COLOR_BUURRUIMTE;
    case "gemengd":
      return COLOR_EXTERIEUR; // fallback single colour; gemengd gets fillSecondary too
    case "onbepaald":
      return COLOR_ONBEPAALD;
    default:
      return COLOR_ONBEPAALD;
  }
}

/**
 * Resolve the render colour for one face. Window/door hosts always render in
 * their own tint regardless of classification (their classification is
 * almost always "exterieur", which would otherwise be visually
 * indistinguishable from an opaque exterior wall). "gemengd" faces get a
 * two-tone (ground + exterior) result for a striped/split render. QC-flagged
 * faces keep their classification colour but signal `qcFlagged: true` so the
 * caller adds a red outline/hatch on top — QC status and envelope
 * classification are independent axes and must stay visually separable.
 */
export function resolveFaceColor(
  face: Pick<ReconstructedFace, "classification" | "hostCategory" | "qcFlag" | "voteAgreement" | "sampleCount">,
): FaceColor {
  const qcFlagged = isQcFlagged(face);

  if (face.hostCategory === "raam") return { fill: COLOR_RAAM, qcFlagged };
  if (face.hostCategory === "deur") return { fill: COLOR_DEUR, qcFlagged };

  if (face.classification === "gemengd") {
    return { fill: COLOR_GROND, fillSecondary: COLOR_EXTERIEUR, qcFlagged };
  }

  return { fill: classificationColor(face.classification), qcFlagged };
}

// =============================================================================
// 5. Pyrevit (bestaande methode) comparison-JSON parsing
// =============================================================================

/** Minimal shape this page needs from a warmteverlies project-JSON export —
 * see `types/project.ts` `Room`/`ConstructionElement` for the full type this
 * is a structural subset of. Accepts either a bare `{ rooms: [...] }` or a
 * full project export (which also has `rooms` at the top level). */
export interface PyrevitConstruction {
  boundary_type: string;
  vertical_position?: string;
  area: number;
}

export interface PyrevitRoom {
  id: string;
  name: string;
  floor_area?: number;
  constructions: PyrevitConstruction[];
}

export interface PyrevitImportFile {
  rooms: PyrevitRoom[];
}

export class PyrevitParseError extends Error {}

/** Parse + structurally validate a pyrevit-warmteverlies-JSON (or full
 * project-JSON — same `rooms[]` shape). Lenient: unknown/extra fields are
 * ignored; only `rooms[].constructions[].{boundary_type,area}` are required. */
export function parsePyrevitJson(raw: unknown): PyrevitImportFile {
  if (typeof raw !== "object" || raw === null || !("rooms" in raw)) {
    throw new PyrevitParseError("Geen geldig warmteverlies-project-JSON: veld 'rooms' ontbreekt.");
  }
  const rooms = (raw as { rooms: unknown }).rooms;
  if (!Array.isArray(rooms)) {
    throw new PyrevitParseError("'rooms' is geen array.");
  }
  const parsedRooms: PyrevitRoom[] = rooms.map((r, i) => {
    if (typeof r !== "object" || r === null) {
      throw new PyrevitParseError(`rooms[${i}] is geen object.`);
    }
    const room = r as Record<string, unknown>;
    const constructionsRaw = room.constructions;
    if (!Array.isArray(constructionsRaw)) {
      throw new PyrevitParseError(`rooms[${i}].constructions is geen array.`);
    }
    const constructions: PyrevitConstruction[] = constructionsRaw.map((c, j) => {
      if (typeof c !== "object" || c === null) {
        throw new PyrevitParseError(`rooms[${i}].constructions[${j}] is geen object.`);
      }
      const con = c as Record<string, unknown>;
      if (typeof con.boundary_type !== "string") {
        throw new PyrevitParseError(`rooms[${i}].constructions[${j}].boundary_type ontbreekt of is geen string.`);
      }
      if (typeof con.area !== "number") {
        throw new PyrevitParseError(`rooms[${i}].constructions[${j}].area ontbreekt of is geen number.`);
      }
      return {
        boundary_type: con.boundary_type,
        vertical_position: typeof con.vertical_position === "string" ? con.vertical_position : undefined,
        area: con.area,
      };
    });
    return {
      id: typeof room.id === "string" ? room.id : String(room.id ?? i),
      name: typeof room.name === "string" ? room.name : `Ruimte ${i + 1}`,
      floor_area: typeof room.floor_area === "number" ? room.floor_area : undefined,
      constructions,
    };
  });
  return { rooms: parsedRooms };
}

// =============================================================================
// 6. Comparison: reconstructie vs. bestaande (pyrevit) methode
// =============================================================================

export type VerticalPositionKey = "floor" | "ceiling" | "wall";
export type BoundaryTypeKey =
  | "exterior"
  | "ground"
  | "adjacent_room"
  | "adjacent_building"
  | "unheated_space"
  | "water";

function mapZoneToVerticalPosition(zone: FaceZone): VerticalPositionKey {
  if (zone === "vloer") return "floor";
  if (zone === "plafond" || zone === "dak") return "ceiling";
  return "wall";
}

function isBoundaryTypeKey(v: string): v is BoundaryTypeKey {
  return (
    v === "exterior" ||
    v === "ground" ||
    v === "adjacent_room" ||
    v === "adjacent_building" ||
    v === "unheated_space" ||
    v === "water"
  );
}

interface Bucket {
  reconM2: number;
  pyrevitM2: number;
}

function bucketKey(vp: VerticalPositionKey, bt: BoundaryTypeKey): string {
  return `${vp}|${bt}`;
}

export interface ComparisonCell {
  verticalPosition: VerticalPositionKey;
  boundaryType: BoundaryTypeKey;
  reconM2: number;
  pyrevitM2: number;
  /** `null` when both sides are 0 (nothing to compare). */
  deltaPercent: number | null;
  flagged: boolean;
}

export interface RoomComparison {
  reconRoomId: number | null;
  reconRoomName: string | null;
  pyrevitRoomId: string | null;
  pyrevitRoomName: string | null;
  cells: ComparisonCell[];
  reconTotalM2: number;
  pyrevitTotalM2: number;
  /** Sum of recon face area classified "onbepaald" — excluded from `cells`
   * (no pyrevit boundary_type equivalent), surfaced so the UI can flag it. */
  reconExcludedOnbepaaldM2: number;
}

export interface ComparisonResult {
  matched: RoomComparison[];
  unmatchedRecon: ReconstructedSpace[];
  unmatchedPyrevit: PyrevitRoom[];
}

/** Flag threshold for |Δ%| — cells beyond this are highlighted in the UI. */
export const DELTA_FLAG_THRESHOLD_PERCENT = 10;

function normalizeRoomName(name: string): string {
  return name.trim().toLowerCase().replace(/\s+/g, " ");
}

/** Leading room-number token, e.g. "0.12 Woonkamer" -> "0.12". `null` when
 * the name doesn't start with a number. */
function leadingNumber(name: string): string | null {
  const m = /^\s*([0-9]+(?:[.,][0-9]+)?)\b/.exec(name);
  return m ? m[1]!.replace(",", ".") : null;
}

/** Accumulate one space's per-face gross area into (verticalPosition,
 * boundaryType) buckets. "gemengd" faces are split via their own
 * `mixedSplit` (ground/exterior m²) rather than being dumped in a bucket the
 * pyrevit schema has no equivalent for. "onbepaald" faces are excluded
 * (returned separately) since there's no pyrevit `boundary_type` to compare
 * against. */
function accumulateReconBuckets(space: ReconstructedSpace, buckets: Map<string, Bucket>): number {
  let excluded = 0;
  for (const f of space.faces) {
    const vp = mapZoneToVerticalPosition(f.zone);
    const add = (bt: BoundaryTypeKey, m2: number) => {
      const key = bucketKey(vp, bt);
      const b = buckets.get(key) ?? { reconM2: 0, pyrevitM2: 0 };
      b.reconM2 += m2;
      buckets.set(key, b);
    };
    if (f.classification === "exterieur") add("exterior", f.grossAreaM2);
    else if (f.classification === "grond") add("ground", f.grossAreaM2);
    else if (f.classification === "buurruimte") add("adjacent_room", f.grossAreaM2);
    else if (f.classification === "gemengd" && f.mixedSplit) {
      add("ground", f.mixedSplit.groundM2);
      add("exterior", f.mixedSplit.exteriorM2);
    } else {
      excluded += f.grossAreaM2;
    }
  }
  return excluded;
}

function accumulatePyrevitBuckets(room: PyrevitRoom, buckets: Map<string, Bucket>): void {
  for (const c of room.constructions) {
    if (!isBoundaryTypeKey(c.boundary_type)) continue; // unknown/legacy boundary_type — skip
    const vp: VerticalPositionKey =
      c.vertical_position === "floor" || c.vertical_position === "ceiling" || c.vertical_position === "wall"
        ? c.vertical_position
        : "wall";
    const key = bucketKey(vp, c.boundary_type);
    const b = buckets.get(key) ?? { reconM2: 0, pyrevitM2: 0 };
    b.pyrevitM2 += c.area;
    buckets.set(key, b);
  }
}

function cellsFromBuckets(buckets: Map<string, Bucket>): ComparisonCell[] {
  const cells: ComparisonCell[] = [];
  for (const [key, b] of buckets) {
    const [vp, bt] = key.split("|") as [VerticalPositionKey, BoundaryTypeKey];
    const reconM2 = round2(b.reconM2);
    const pyrevitM2 = round2(b.pyrevitM2);
    let deltaPercent: number | null = null;
    if (pyrevitM2 !== 0) {
      deltaPercent = round2(((reconM2 - pyrevitM2) / pyrevitM2) * 100);
    } else if (reconM2 !== 0) {
      deltaPercent = Infinity; // pyrevit reports nothing, recon reports area — undefined %, treat as "always flagged"
    }
    cells.push({
      verticalPosition: vp,
      boundaryType: bt,
      reconM2,
      pyrevitM2,
      deltaPercent,
      flagged: deltaPercent !== null && Math.abs(deltaPercent) > DELTA_FLAG_THRESHOLD_PERCENT,
    });
  }
  // Stable, readable order: vertical position then boundary type.
  const vpOrder: VerticalPositionKey[] = ["floor", "wall", "ceiling"];
  cells.sort((a, b) => {
    const vpDiff = vpOrder.indexOf(a.verticalPosition) - vpOrder.indexOf(b.verticalPosition);
    if (vpDiff !== 0) return vpDiff;
    return a.boundaryType.localeCompare(b.boundaryType);
  });
  return cells;
}

/**
 * Compare a fase-2a reconstruction against a pyrevit-warmteverlies-JSON of
 * (presumably) the same building. Rooms are matched by normalized name
 * first, then by a leading room-number token (e.g. "0.12"); each pyrevit
 * room is consumed at most once (greedy, first-match-wins in recon space
 * order). Unmatched rooms on either side are returned separately so the UI
 * can list them without silently dropping data.
 */
export function compareWithPyrevit(recon: ReconstructionResult, pyrevit: PyrevitImportFile): ComparisonResult {
  const pyrevitByName = new Map<string, PyrevitRoom>();
  const pyrevitByNumber = new Map<string, PyrevitRoom>();
  for (const room of pyrevit.rooms) {
    pyrevitByName.set(normalizeRoomName(room.name), room);
    const num = leadingNumber(room.name);
    if (num && !pyrevitByNumber.has(num)) pyrevitByNumber.set(num, room);
  }

  const consumed = new Set<string>();
  const matched: RoomComparison[] = [];
  const unmatchedRecon: ReconstructedSpace[] = [];

  for (const space of recon.spaces) {
    const spaceName = space.name ?? space.longName ?? "";
    const normalized = normalizeRoomName(spaceName);
    const num = leadingNumber(spaceName);

    let match: PyrevitRoom | undefined;
    const byName = pyrevitByName.get(normalized);
    if (byName && !consumed.has(byName.id)) match = byName;
    if (!match && num) {
      const byNumber = pyrevitByNumber.get(num);
      if (byNumber && !consumed.has(byNumber.id)) match = byNumber;
    }

    if (!match) {
      unmatchedRecon.push(space);
      continue;
    }
    consumed.add(match.id);

    const buckets = new Map<string, Bucket>();
    const reconExcludedOnbepaaldM2 = round2(accumulateReconBuckets(space, buckets));
    accumulatePyrevitBuckets(match, buckets);
    const cells = cellsFromBuckets(buckets);

    matched.push({
      reconRoomId: space.id,
      reconRoomName: spaceName || null,
      pyrevitRoomId: match.id,
      pyrevitRoomName: match.name,
      cells,
      reconTotalM2: round2(cells.reduce((s, c) => s + c.reconM2, 0)),
      pyrevitTotalM2: round2(cells.reduce((s, c) => s + c.pyrevitM2, 0)),
      reconExcludedOnbepaaldM2,
    });
  }

  const unmatchedPyrevit = pyrevit.rooms.filter((r) => !consumed.has(r.id));

  return { matched, unmatchedRecon, unmatchedPyrevit };
}
