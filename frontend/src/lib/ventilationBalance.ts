/**
 * Ventilatiebalans — afgeleide berekeningen (frontend, TypeScript).
 *
 * Pragmatische port van de pyRevit `VentilatieBalans`-plugin: BBL-eis per
 * ruimte (dm³/s), default gebruiksfunctie-classificatie uit de model-room,
 * overdruk-/overstroomverdeling (`_bereken_overdruk_verdeling`) en een
 * spleet-onder-deur-schatting, NEN 1087:2001-verankerd (zie
 * {@link estimateDoorGapAreaCm2}).
 *
 * **Eenheden:** dm³/s intern.
 */

import type { Project } from "../types";
import type { RoomFunction } from "../types/project";
import type { ModelRoom, Point2D } from "../components/modeller/types";
import { segmentsShareEdge } from "../components/modeller/geometry";
import {
  bblDemandDm3s,
  bblRequirementFor,
  DEFAULT_BBL_FUNCTION,
  ventilationSystemOf,
  type BblFunctionKey,
  type BblRequirementType,
  type VentilationRoomState,
  type VentilationState,
  type VentilationSystemInfo,
  type VentilationTerminal,
} from "../types/ventilation";

/**
 * Map de model-room-`function` (RoomFunction) op een BBL-gebruiksfunctie.
 * Onbekend → {@link DEFAULT_BBL_FUNCTION}.
 */
const ROOM_FUNCTION_TO_BBL: Record<RoomFunction, BblFunctionKey> = {
  living_room: "verblijfsruimte",
  kitchen: "keuken",
  bedroom: "verblijfsruimte",
  bathroom: "badruimte",
  toilet: "toiletruimte",
  hallway: "verkeersruimte",
  landing: "verkeersruimte",
  storage: "bergruimte",
  attic: "bergruimte",
  custom: DEFAULT_BBL_FUNCTION,
};

/** Bepaal de default BBL-functie voor een room-functie-string. */
export function defaultBblFunction(fn: string): BblFunctionKey {
  return ROOM_FUNCTION_TO_BBL[fn as RoomFunction] ?? DEFAULT_BBL_FUNCTION;
}

/**
 * Bereken de per-room ventilatie-state (gebruiksfunctie + eisen in dm³/s) voor
 * een gegeven oppervlak. Wanneer een bestaande state een handmatig gekozen
 * `ventilationFunction` heeft, wordt die gerespecteerd; anders valt het terug
 * op de afgeleide functie uit de room-`function`. Een opgegeven `occupancy`
 * (bezetting) verhoogt de eis via de personen-toeslag in
 * {@link bblDemandDm3s} (`max(opp × dm3/m², pers × pp, minimum)`).
 */
export function computeRoomVentilation(
  areaM2: number,
  roomFunction: string,
  existing?: VentilationRoomState,
): VentilationRoomState {
  const fn = existing?.ventilationFunction ?? defaultBblFunction(roomFunction);
  const req = bblRequirementFor(fn);
  const demand = bblDemandDm3s(areaM2, fn, existing?.occupancy);
  return {
    ventilationFunction: fn,
    requiredSupplyDm3s: req.type === "supply" ? demand : 0,
    requiredExhaustDm3s: req.type === "exhaust" ? demand : 0,
    airSourceRoomId: existing?.airSourceRoomId ?? null,
    ...(existing?.occupancy !== undefined
      ? { occupancy: existing.occupancy }
      : {}),
  };
}

/**
 * Bereken de BBL-eis (dm³/s) per ruimte voor het hele project. Resultaat is
 * gekeyed op `room.id`. Hergebruikt bestaande sidecar-states (handmatige
 * functie-override + overstroom-bron) waar aanwezig.
 *
 * Oppervlak komt uit `Room.floor_area` (m²) — de calc-bron van waarheid.
 */
export function deriveVentilationDemand(
  project: Project,
  existing?: Record<string, VentilationRoomState>,
): Record<string, VentilationRoomState> {
  const out: Record<string, VentilationRoomState> = {};
  for (const room of project.rooms) {
    const sourceId =
      existing?.[room.id]?.airSourceRoomId ?? room.air_source_room_id ?? null;
    const base = existing?.[room.id]
      ? { ...existing[room.id]!, airSourceRoomId: sourceId }
      : undefined;
    out[room.id] = computeRoomVentilation(
      room.floor_area,
      String(room.function),
      base,
    );
  }
  return out;
}

// ---------------------------------------------------------------------------
// Spleet onder de deur — indicatieve schatting
// ---------------------------------------------------------------------------

/**
 * Toelaatbaar drukverschil (Pa) over een doorstroomopening — default voor
 * woonfuncties. NEN 1087:2001 §5.1.3.2.7 (p. 21) hanteert per situatie:
 *   - toevoer bovendaks: 1 Pa;
 *   - dwarsventilatie (doorstroming binnen de woning): 1 Pa,
 *     **2 Pa voor kantoorgebouwen** ({@link DOOR_GAP_DELTA_P_OFFICE_PA});
 *   - afvoerkanaal: `0,4·Δh + 1` Pa (Δh = hoogteverschil in m).
 */
export const DOOR_GAP_DELTA_P_PA = 1.0;

/** Δp-criterium dwarsventilatie kantoorgebouwen (NEN 1087:2001 §5.1.3.2.7). */
export const DOOR_GAP_DELTA_P_OFFICE_PA = 2.0;

/**
 * Schatting van de benodigde vrije doorlaat (cm²) van een doorstroomopening
 * (spleet onder de deur) voor een gegeven overstroomdebiet.
 *
 * Orifice-benadering: `A = q / (C_d · √(2·ΔP/ρ))`, met
 *   - q in m³/s (= dm³/s ÷ 1000),
 *   - C_d = 0,6 (scherprandige opening),
 *   - ΔP = `deltaPPa` (default {@link DOOR_GAP_DELTA_P_PA} = 1,0 Pa),
 *   - ρ = 1,2 kg/m³ (lucht).
 *
 * **Norm-verankering (NEN 1087:2001, PDF-extractie geverifieerd 2026-06-10):**
 *   - §5.1.3.2.1 (p. 17) geeft de doorlaat-karakteristiek `qv = C·Δp^n`;
 *     deze orifice-vorm is daarvan het geval `n = 0,5` met
 *     `C = C_d·A·√(2/ρ)` — turbulente stroming door een scherprandige
 *     opening, de gangbare aanname voor een spleet onder de deur.
 *   - §5.1.3.2.7 (p. 21) geeft de Δp-criteria per situatie (zie
 *     {@link DOOR_GAP_DELTA_P_PA}); voor kantoor-dwarsventilatie geldt 2 Pa —
 *     geef dan {@link DOOR_GAP_DELTA_P_OFFICE_PA} mee als `deltaPPa`.
 *
 * Resultaat in cm².
 */
export function estimateDoorGapAreaCm2(
  flowDm3s: number,
  deltaPPa: number = DOOR_GAP_DELTA_P_PA,
): number {
  if (flowDm3s <= 0) return 0;
  const C_D = 0.6;
  const RHO = 1.2; // kg/m³
  const qM3s = flowDm3s / 1000;
  const aM2 = qM3s / (C_D * Math.sqrt((2 * deltaPPa) / RHO));
  return aM2 * 1e4; // m² → cm²
}

// ---------------------------------------------------------------------------
// Overdruk-/overstroomverdeling — toevoer-overschot naar afvoerruimtes
// ---------------------------------------------------------------------------

/**
 * Resultaat van {@link computeOverflowDistribution}: het toevoer-overschot
 * (overdruk) verdeeld over de afvoerruimtes, gekeyed op `room.id`.
 */
export interface OverflowDistribution {
  /**
   * Extra afvoer-correctie per afvoerruimte in dm³/s (verdeeld
   * toevoer-overschot, naar rato van oppervlak; 0 voor niet-afvoerruimtes).
   */
  exhaustCorrectionDm3s: Record<string, number>;
  /** Afvoer-totaal per ruimte: afvoer-eis + correctie (dm³/s). */
  exhaustTotalDm3s: Record<string, number>;
  /** Verdeeld toevoer-overschot in dm³/s (0 bij balans/onderdruk). */
  surplusDm3s: number;
}

/**
 * Verdeel het toevoer-overschot (overdruk) over de afvoerruimtes, naar rato
 * van hun vloeroppervlak. Port van `_bereken_overdruk_verdeling` uit de
 * pyRevit-plugin (`VentilatieBalans.pushbutton/script.py:632-651`):
 *
 *   - `overdruk = Σ toevoer-eis − Σ afvoer-eis`; alleen verdelen bij
 *     overdruk > 0 (onderdruk/balans → alle correcties 0);
 *   - per afvoerruimte: `correctie = round(overdruk × opp / Σ opp, 1)`
 *     (zelfde 1-decimaal-afronding als de plugin);
 *   - `afvoer_totaal = afvoer-eis + correctie` (plugin: `afvoer_totaal`).
 *
 * **Verdeelregel = plugin-conventie** — NEN 1087 definieert géén verdeling
 * van het toevoer-overschot over meerdere afvoerruimtes (§5.1.3.2 geeft alleen
 * de doorlaat-karakteristiek en Δp-criteria); naar-rato-van-oppervlak is een
 * engineering-keuze, 1:1 overgenomen uit de plugin.
 *
 * De plugin verdeelt per zone; het webmodel kent (nog) geen zones, dus hier
 * geldt het hele gebouw als één zone.
 *
 * @param ventilationRooms per-room ventilatie-state (eisen in dm³/s)
 * @param areasM2 vloeroppervlak per ruimte-id in m² (`Room.floor_area`)
 */
export function computeOverflowDistribution(
  ventilationRooms: Record<string, VentilationRoomState>,
  areasM2: Record<string, number>,
): OverflowDistribution {
  let totalSupply = 0;
  let totalExhaust = 0;
  let exhaustArea = 0;
  for (const [roomId, vr] of Object.entries(ventilationRooms)) {
    totalSupply += vr.requiredSupplyDm3s;
    totalExhaust += vr.requiredExhaustDm3s;
    if (vr.requiredExhaustDm3s > 0) exhaustArea += areasM2[roomId] ?? 0;
  }
  const surplus = totalSupply - totalExhaust;

  const exhaustCorrectionDm3s: Record<string, number> = {};
  const exhaustTotalDm3s: Record<string, number> = {};
  for (const [roomId, vr] of Object.entries(ventilationRooms)) {
    let correction = 0;
    if (surplus > 0 && vr.requiredExhaustDm3s > 0 && exhaustArea > 0) {
      // Plugin: round(overdruk * opp / totaal_opp, 1).
      correction =
        Math.round(((surplus * (areasM2[roomId] ?? 0)) / exhaustArea) * 10) /
        10;
    }
    exhaustCorrectionDm3s[roomId] = correction;
    exhaustTotalDm3s[roomId] = vr.requiredExhaustDm3s + correction;
  }
  return {
    exhaustCorrectionDm3s,
    exhaustTotalDm3s,
    surplusDm3s: Math.max(0, surplus),
  };
}

// ---------------------------------------------------------------------------
// Overstroom-relaties — afgeleid van gedeelde wanden (niet van deuren)
// ---------------------------------------------------------------------------

/**
 * Eén overstroom-relatie tussen twee aangrenzende ruimtes: lucht stroomt van de
 * bron- (toevoer) naar de doel- (afvoer) ruimte door de doorstroomopening in de
 * gedeelde scheidingswand.
 *
 * Geometrie is afgeleid uit de gedeelde wand-edge tussen de twee
 * room-polygonen: `mid` is het midden van de overlap, `nx/ny` wijst van de bron
 * naar de doel-ruimte (loodrecht op de wand), `ux/uy` loopt langs de wand.
 * `overlapMm` is de lengte van de gedeelde edge (voor de spleet-balkbreedte).
 */
export interface OverflowRelation {
  /** Stabiele sleutel op het ruimte-paar (gesorteerd, dedup). */
  key: string;
  /** Bron-ruimte (toevoer) id. */
  sourceRoomId: string;
  /** Doel-ruimte (afvoer) id. */
  targetRoomId: string;
  /** Midden van de gedeelde wand-edge (wereld-mm). */
  mid: Point2D;
  /** Eenheidsvector langs de gedeelde wand. */
  ux: number;
  uy: number;
  /** Eenheidsvector loodrecht op de wand, bron → doel. */
  nx: number;
  ny: number;
  /** Lengte van de gedeelde edge-overlap in mm. */
  overlapMm: number;
  /**
   * Overstroomdebiet in dm³/s door deze doorstroomopening: het afvoer-totaal
   * van de doel-ruimte (afvoer-eis + verdeelde overdruk-correctie, zie
   * {@link computeOverflowDistribution}), gelijk verdeeld over alle
   * binnenkomende relaties van die doel-ruimte.
   */
  flowDm3s: number;
}

/** Het BBL-type (supply/exhaust/none) voor een ruimte uit de ventilatie-state. */
function ventTypeOf(
  vr: VentilationRoomState | undefined,
): BblRequirementType {
  if (!vr) return "none";
  if (vr.requiredSupplyDm3s > 0) return "supply";
  if (vr.requiredExhaustDm3s > 0) return "exhaust";
  return "none";
}

/**
 * Bepaal de overstroom-richting voor een paar aangrenzende ruimtes.
 *
 * Prioriteit:
 *   1. `airSourceRoomId` — wanneer ruimte X die van ruimte Y heeft gezet,
 *      stroomt lucht Y → X (X betrekt z'n lucht uit Y). Hardste signaal.
 *   2. BBL-type-heuristiek — toevoer-ruimte (droog) → afvoer-ruimte (nat).
 *
 * Returns `[sourceId, targetId]` of `null` wanneer er geen overstroom is
 * (zelfde type aan beide kanten, of beide `none`).
 */
function resolveOverflowDirection(
  aId: string,
  bId: string,
  vrA: VentilationRoomState | undefined,
  vrB: VentilationRoomState | undefined,
): [string, string] | null {
  // 1. Expliciete overstroom-bron (hardste signaal).
  if (vrA?.airSourceRoomId === bId) return [bId, aId];
  if (vrB?.airSourceRoomId === aId) return [aId, bId];

  // 2. Type-heuristiek: toevoer → afvoer.
  const ta = ventTypeOf(vrA);
  const tb = ventTypeOf(vrB);
  if (ta === "supply" && tb === "exhaust") return [aId, bId];
  if (tb === "supply" && ta === "exhaust") return [bId, aId];

  // Geen netto overstroom-richting (zelfde type, of een verkeersruimte zonder
  // expliciete bron aan beide kanten).
  return null;
}

/**
 * Vind de gedeelde wand-edge tussen twee room-polygonen en geef de geometrie
 * voor de overstroom-pijl + spleet-balk terug. `nx/ny` wijst van `polyFrom`
 * (bron) naar `polyTo` (doel). Returns `null` als er geen gedeelde edge is.
 */
function sharedEdgeGeometry(
  polyFrom: Point2D[],
  polyTo: Point2D[],
): { mid: Point2D; ux: number; uy: number; nx: number; ny: number; overlapMm: number } | null {
  const nFrom = polyFrom.length;
  const nTo = polyTo.length;
  for (let i = 0; i < nFrom; i++) {
    const a = polyFrom[i]!;
    const b = polyFrom[(i + 1) % nFrom]!;
    for (let j = 0; j < nTo; j++) {
      const c = polyTo[j]!;
      const d = polyTo[(j + 1) % nTo]!;
      if (!segmentsShareEdge(a, b, c, d)) continue;

      // Overlap-interval langs de bron-edge (projecteer c,d op a→b).
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const len = Math.hypot(dx, dy);
      if (len < 1) continue;
      const ux = dx / len;
      const uy = dy / len;
      const tA = 0;
      const tB = len;
      const tC = (c.x - a.x) * ux + (c.y - a.y) * uy;
      const tD = (d.x - a.x) * ux + (d.y - a.y) * uy;
      const lo = Math.max(Math.min(tA, tB), Math.min(tC, tD));
      const hi = Math.min(Math.max(tA, tB), Math.max(tC, tD));
      const overlapMm = Math.max(0, hi - lo);
      const tMid = (lo + hi) / 2;
      const mid = { x: a.x + ux * tMid, y: a.y + uy * tMid };

      // Normaal loodrecht op de wand; oriënteer van bron- naar doel-polygon
      // (richting het centrum van de doel-ruimte).
      let nx = uy;
      let ny = -ux;
      const cTo = polygonCentroid(polyTo);
      if ((cTo.x - mid.x) * nx + (cTo.y - mid.y) * ny < 0) {
        nx = -nx;
        ny = -ny;
      }
      return { mid, ux, uy, nx, ny, overlapMm };
    }
  }
  return null;
}

/** Eenvoudig zwaartepunt (gemiddelde van de hoekpunten). */
function polygonCentroid(poly: Point2D[]): Point2D {
  const n = poly.length;
  return {
    x: poly.reduce((s, p) => s + p.x, 0) / n,
    y: poly.reduce((s, p) => s + p.y, 0) / n,
  };
}

/**
 * Leid de overstroom-relaties af voor een set model-ruimtes + hun ventilatie-
 * state. Voor elk paar aangrenzende ruimtes (gedeelde wand) wordt bepaald of
 * lucht overstroomt en in welke richting (`airSourceRoomId` > toevoer→afvoer-
 * heuristiek). Per ruimte-paar maximaal één relatie (dedup).
 *
 * Geen interactie met `deriveModelDoors` (die blijft een stub) — de
 * doorstroomopening hangt aan de scheidingswand, niet aan een deur-object.
 *
 * **Debiet per relatie:** het afvoer-totaal van de doel-ruimte (afvoer-eis +
 * verdeelde overdruk-correctie uit {@link computeOverflowDistribution}),
 * gelijk verdeeld over alle binnenkomende relaties van die doel-ruimte —
 * zodat de spleet-berekening op het verdeelde debiet werkt en niet per
 * ruimte-paar de volle afvoer-eis dubbel telt. De gelijke verdeling over
 * meerdere doorstroomopeningen is een engineering-keuze (NEN 1087 definieert
 * geen verdeling); de overdruk-verdeling zelf volgt de plugin-conventie.
 * Doel-ruimte zonder afvoer-eis → fallback op de toevoer-eis van de bron
 * (bv. expliciete `airSourceRoomId` naar een eis-loze ruimte).
 *
 * @param rooms model-ruimtes (polygonen in wereld-mm)
 * @param ventilationRooms per-room ventilatie-state (eisen + overstroom-bron)
 * @param distribution gebouwbrede overdruk-verdeling
 *   ({@link computeOverflowDistribution}); `undefined` → geen correctie
 *   (afvoer-totaal = afvoer-eis)
 */
export function deriveOverflowRelations(
  rooms: ModelRoom[],
  ventilationRooms: Record<string, VentilationRoomState>,
  distribution?: OverflowDistribution,
): OverflowRelation[] {
  const out: OverflowRelation[] = [];
  const seen = new Set<string>();

  for (let i = 0; i < rooms.length; i++) {
    const ri = rooms[i]!;
    for (let j = i + 1; j < rooms.length; j++) {
      const rj = rooms[j]!;
      const pairKey = ri.id < rj.id ? `${ri.id}|${rj.id}` : `${rj.id}|${ri.id}`;
      if (seen.has(pairKey)) continue;

      const dir = resolveOverflowDirection(
        ri.id,
        rj.id,
        ventilationRooms[ri.id],
        ventilationRooms[rj.id],
      );
      if (!dir) continue;
      const [sourceId, targetId] = dir;

      const sourceRoom = sourceId === ri.id ? ri : rj;
      const targetRoom = targetId === ri.id ? ri : rj;
      const geom = sharedEdgeGeometry(sourceRoom.polygon, targetRoom.polygon);
      if (!geom) continue; // delen wel een type-relatie maar geen geometrische wand

      seen.add(pairKey);

      out.push({
        key: pairKey,
        sourceRoomId: sourceId,
        targetRoomId: targetId,
        mid: geom.mid,
        ux: geom.ux,
        uy: geom.uy,
        nx: geom.nx,
        ny: geom.ny,
        overlapMm: geom.overlapMm,
        flowDm3s: 0, // tweede pass hieronder
      });
    }
  }

  // Tweede pass: debiet per relatie = afvoer-totaal van de doel-ruimte
  // (eis + verdeelde overdruk-correctie), gelijk verdeeld over het aantal
  // binnenkomende relaties van die doel-ruimte.
  const incomingCount = new Map<string, number>();
  for (const rel of out) {
    incomingCount.set(
      rel.targetRoomId,
      (incomingCount.get(rel.targetRoomId) ?? 0) + 1,
    );
  }
  for (const rel of out) {
    const vrTarget = ventilationRooms[rel.targetRoomId];
    const exhaustTotal =
      distribution?.exhaustTotalDm3s[rel.targetRoomId] ??
      vrTarget?.requiredExhaustDm3s ??
      0;
    if (exhaustTotal > 0) {
      rel.flowDm3s = exhaustTotal / (incomingCount.get(rel.targetRoomId) ?? 1);
    } else {
      // Fallback: doel zonder afvoer — gebruik de toevoer-eis van de bron.
      rel.flowDm3s =
        ventilationRooms[rel.sourceRoomId]?.requiredSupplyDm3s ?? 0;
    }
  }
  return out;
}

// ---------------------------------------------------------------------------
// Gebouwbalans — per-room aggregatie + totalen (zijpaneel)
// ---------------------------------------------------------------------------

/**
 * Tolerantie (dm³/s) waarbinnen totaal-toevoer en totaal-afvoer als "in
 * balans" gelden. Port van `abs(balans) < 1` uit de pyRevit-plugin
 * (`VentilatieBalans.pushbutton/script.py:1211`).
 */
export const BALANCE_TOLERANCE_DM3S = 1.0;

/** Per-room regel in de gebouwbalans (eis vs. aanwezig per richting). */
export interface RoomVentilationBalance {
  roomId: string;
  /** BBL-eis toevoer/afvoer in dm³/s (uit de afgeleide room-state). */
  requiredSupplyDm3s: number;
  requiredExhaustDm3s: number;
  /** Som van `flowDm3s` van de geplaatste ventielen (zonder debiet = 0). */
  presentSupplyDm3s: number;
  presentExhaustDm3s: number;
  /** Aantal ventielen in deze ruimte zónder `flowDm3s` ("debiet ontbreekt"). */
  missingFlowCount: number;
  /**
   * Tekort per richting in dm³/s (eis − aanwezig, geclamped op ≥ 0).
   * Bij een natuurlijke kant (zie {@link VentilationSystemInfo}) is het
   * ventiel-tekort niet van toepassing en is de waarde 0 — de toetsing loopt
   * dan via gevelroosters/natuurlijke afvoer i.p.v. ventielen.
   */
  supplyDeficitDm3s: number;
  exhaustDeficitDm3s: number;
}

/** Gebouwbrede ventilatiebalans: per-room regels + totalen + indicator. */
export interface BuildingVentilationBalance {
  rooms: Record<string, RoomVentilationBalance>;
  /** Effectief ventilatiesysteem (default-fallback voor oude bestanden). */
  system: VentilationSystemInfo;
  totalRequiredSupplyDm3s: number;
  totalRequiredExhaustDm3s: number;
  totalPresentSupplyDm3s: number;
  totalPresentExhaustDm3s: number;
  /** Eis-totalen in balans binnen {@link BALANCE_TOLERANCE_DM3S}? */
  balanced: boolean;
  /** Totaal toevoer-eis − totaal afvoer-eis in dm³/s (+ = overdruk). */
  imbalanceDm3s: number;
}

/**
 * Aggregeer de gebouwbalans uit de per-room eisen + geplaatste ventielen.
 *
 * - "Aanwezig" per ruimte = som `flowDm3s` van de terminals van dat type;
 *   terminals zonder `flowDm3s` tellen als 0 maar worden geteld in
 *   `missingFlowCount` zodat de UI ze kan markeren.
 * - Tekorten worden alleen gerapporteerd voor de mechanische kant(en) van het
 *   gekozen systeem; een natuurlijke kant (bv. toevoer bij systeem A/C) wordt
 *   via gevelroosters getoetst en krijgt tekort 0.
 * - De balans-indicator vergelijkt de eis-totalen (toevoer vs. afvoer) binnen
 *   {@link BALANCE_TOLERANCE_DM3S} — zelfde criterium als de plugin.
 */
export function aggregateVentilationBalance(
  ventilationRooms: Record<string, VentilationRoomState>,
  terminals: VentilationTerminal[],
  system?: VentilationState["system"],
): BuildingVentilationBalance {
  const sys = ventilationSystemOf({ system });
  const rooms: Record<string, RoomVentilationBalance> = {};

  for (const [roomId, vr] of Object.entries(ventilationRooms)) {
    rooms[roomId] = {
      roomId,
      requiredSupplyDm3s: vr.requiredSupplyDm3s,
      requiredExhaustDm3s: vr.requiredExhaustDm3s,
      presentSupplyDm3s: 0,
      presentExhaustDm3s: 0,
      missingFlowCount: 0,
      supplyDeficitDm3s: 0,
      exhaustDeficitDm3s: 0,
    };
  }

  for (const t of terminals) {
    const row = rooms[t.roomId];
    if (!row) continue; // ventiel van een verwijderde/onbekende ruimte
    if (t.flowDm3s === undefined) {
      row.missingFlowCount += 1;
      continue;
    }
    if (t.type === "supply") row.presentSupplyDm3s += t.flowDm3s;
    else row.presentExhaustDm3s += t.flowDm3s;
  }

  let totalRequiredSupply = 0;
  let totalRequiredExhaust = 0;
  let totalPresentSupply = 0;
  let totalPresentExhaust = 0;

  for (const row of Object.values(rooms)) {
    row.supplyDeficitDm3s = sys.supplyMechanical
      ? Math.max(0, row.requiredSupplyDm3s - row.presentSupplyDm3s)
      : 0;
    row.exhaustDeficitDm3s = sys.exhaustMechanical
      ? Math.max(0, row.requiredExhaustDm3s - row.presentExhaustDm3s)
      : 0;
    totalRequiredSupply += row.requiredSupplyDm3s;
    totalRequiredExhaust += row.requiredExhaustDm3s;
    totalPresentSupply += row.presentSupplyDm3s;
    totalPresentExhaust += row.presentExhaustDm3s;
  }

  const imbalance = totalRequiredSupply - totalRequiredExhaust;
  return {
    rooms,
    system: sys,
    totalRequiredSupplyDm3s: totalRequiredSupply,
    totalRequiredExhaustDm3s: totalRequiredExhaust,
    totalPresentSupplyDm3s: totalPresentSupply,
    totalPresentExhaustDm3s: totalPresentExhaust,
    balanced: Math.abs(imbalance) < BALANCE_TOLERANCE_DM3S,
    imbalanceDm3s: imbalance,
  };
}
