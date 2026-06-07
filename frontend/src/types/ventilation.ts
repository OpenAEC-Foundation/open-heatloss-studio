/**
 * Ventilatiebalans datamodel (frontend-sidecar, geen Rust).
 *
 * IFC/IFCX is geparkeerd; ventilatie wordt pragmatisch in het bestaande
 * calc-model gebouwd. Terminals + per-room ventilatie-velden leven sidecar in
 * `projectStore` (zelfde patroon als `isso53Rooms` / `sharedExtra`) en worden
 * mee-geserialiseerd in de opslag-envelope (`.ifcenergy` + `.heatloss.json`),
 * zodat ze een save→reopen overleven (valkuil commit `8ccff9f`).
 *
 * **Eenheden:** intern dm³/s (de hele codebase rekent in dm³/s voor
 * luchtvolumestroom — zie `CLAUDE.md` + `qi_spec`). m³/h is enkel een
 * afgeleide weergave (× {@link DM3S_TO_M3H}).
 */

/** Omrekenfactor dm³/s → m³/h (1 dm³/s = 3,6 m³/h). */
export const DM3S_TO_M3H = 3.6;

/** dm³/s → m³/h. */
export function dm3sToM3h(dm3s: number): number {
  return dm3s * DM3S_TO_M3H;
}

/** m³/h → dm³/s. */
export function m3hToDm3s(m3h: number): number {
  return m3h / DM3S_TO_M3H;
}

// ---------------------------------------------------------------------------
// Terminal (ventiel / rooster)
// ---------------------------------------------------------------------------

/** Toevoer (supply) of afvoer (exhaust). */
export type VentilationTerminalType = "supply" | "exhaust";

/** Herkomst van het ventiel — handmatig geplaatst of (later) uit Revit-import. */
export type VentilationTerminalSource = "manual" | "revit";

/**
 * Een ventilatie-ventiel of -rooster, gekoppeld aan een ruimte. Plaatsing is
 * wand-gebonden (`wallIndex` + `offsetMm`, net als ramen/deuren) of vrij in de
 * ruimte (`positionMm`, bv. een plafondventiel). `flowDm3s` is het ontworpen
 * debiet in dm³/s; `undefined` = nog niet bepaald (UI valt terug op de eis).
 */
export interface VentilationTerminal {
  id: string;
  roomId: string;
  type: VentilationTerminalType;
  source: VentilationTerminalSource;
  /** Edge-index in de room-polygon (NEN-gevel of binnenwand). */
  wallIndex?: number;
  /** Positie langs de wand-edge vanaf het beginpunt, in mm. */
  offsetMm?: number;
  /** Vrije positie in mm (plafondventiel) — alternatief voor wand-binding. */
  positionMm?: { x: number; y: number };
  /** Ontworpen luchtvolumestroom in dm³/s. */
  flowDm3s?: number;
  /** Optionele mark/label uit Revit. */
  mark?: string;
}

// ---------------------------------------------------------------------------
// Per-room ventilatie-velden
// ---------------------------------------------------------------------------

/**
 * Per-ruimte ventilatie-sidecar, gekeyed op `room.id`. Bevat de
 * gebruiksfunctie-classificatie (BBL-lookup) + de afgeleide eisen (dm³/s).
 * De overstroom-bron (`airSourceRoomId`) hergebruikt waar mogelijk
 * `Room.air_source_room_id`; deze sidecar houdt het als override/spiegel.
 */
export interface VentilationRoomState {
  /** BBL-gebruiksfunctie (key in {@link BBL_REQUIREMENTS}). */
  ventilationFunction: BblFunctionKey;
  /** Afgeleide toevoer-eis in dm³/s (0 voor afvoer/geen-ruimtes). */
  requiredSupplyDm3s: number;
  /** Afgeleide afvoer-eis in dm³/s (0 voor toevoer/geen-ruimtes). */
  requiredExhaustDm3s: number;
  /**
   * Bron-kamer voor overstroom (overflow). `null`/undefined = gevelrooster /
   * buitenlucht. Spiegelt/overschrijft `Room.air_source_room_id`.
   */
  airSourceRoomId?: string | null;
}

// ---------------------------------------------------------------------------
// Sidecar envelope (persistentie)
// ---------------------------------------------------------------------------

/**
 * Volledige ventilatie-sidecar zoals gepersisteerd in `projectStore` en
 * mee-geserialiseerd in de opslag-envelope.
 */
export interface VentilationState {
  terminals: VentilationTerminal[];
  rooms: Record<string, VentilationRoomState>;
}

export const DEFAULT_VENTILATION_STATE: VentilationState = {
  terminals: [],
  rooms: {},
};

// ---------------------------------------------------------------------------
// BBL-eisentabel (Bouwbesluit / BBL afd. 3.6)
// ---------------------------------------------------------------------------

/**
 * Eis-type per gebruiksfunctie: toevoer, afvoer of geen ventilatie-eis.
 * Port van `TYPE_TOEVOER` / `TYPE_AFVOER` / `TYPE_GEEN` uit de pyRevit-plugin.
 */
export type BblRequirementType = "supply" | "exhaust" | "none";

/**
 * Eén BBL-eisregel: specifiek debiet per m² (dm³/(s·m²)), een ondergrens
 * (dm³/s) en het eis-type. Geport uit `NORMEN_BBL` in
 * `VentilatieBalans.pushbutton/script.py:54-79`.
 */
export interface BblRequirement {
  /** Specifiek debiet in dm³/(s·m²). */
  dm3PerM2: number;
  /** Minimum-debiet in dm³/s. */
  minimumDm3s: number;
  /** Toevoer / afvoer / geen. */
  type: BblRequirementType;
}

/**
 * BBL-eisentabel per gebruiksfunctie (geport uit `NORMEN_BBL`).
 *
 * Sleutels zijn de Nederlandse gebruiksfunctie-namen uit de plugin, zodat een
 * latere Revit-import 1:1 mapt. Minima en specifieke debieten in dm³/s
 * respectievelijk dm³/(s·m²).
 */
export const BBL_REQUIREMENTS = {
  bijeenkomstfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply" },
  kantoorfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply" },
  onderwijsfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply" },
  sportfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply" },
  winkelfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply" },
  woonfunctie: { dm3PerM2: 0.7, minimumDm3s: 7, type: "supply" },
  gezondheidszorgfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply" },
  logiesfunctie: { dm3PerM2: 0.7, minimumDm3s: 7, type: "supply" },
  verblijfsgebied: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply" },
  verblijfsruimte: { dm3PerM2: 0.7, minimumDm3s: 7, type: "supply" },
  "overige gebruiksfunctie": { dm3PerM2: 0.7, minimumDm3s: 7, type: "exhaust" },
  toiletruimte: { dm3PerM2: 0, minimumDm3s: 7, type: "exhaust" },
  badruimte: { dm3PerM2: 0, minimumDm3s: 14, type: "exhaust" },
  keuken: { dm3PerM2: 0, minimumDm3s: 21, type: "exhaust" },
  wasruimte: { dm3PerM2: 0, minimumDm3s: 14, type: "exhaust" },
  "technische ruimte": { dm3PerM2: 0, minimumDm3s: 2, type: "exhaust" },
  meterruimte: { dm3PerM2: 0, minimumDm3s: 2, type: "exhaust" },
  bergruimte: { dm3PerM2: 0, minimumDm3s: 0, type: "none" },
  berging: { dm3PerM2: 0, minimumDm3s: 0, type: "none" },
  verkeersruimte: { dm3PerM2: 0, minimumDm3s: 0, type: "none" },
  gang: { dm3PerM2: 0, minimumDm3s: 0, type: "none" },
  hal: { dm3PerM2: 0, minimumDm3s: 0, type: "none" },
  trappenhuis: { dm3PerM2: 0, minimumDm3s: 0, type: "none" },
} as const satisfies Record<string, BblRequirement>;

/** Geldige sleutels in {@link BBL_REQUIREMENTS}. */
export type BblFunctionKey = keyof typeof BBL_REQUIREMENTS;

/** Fallback-eis (geport uit `DEFAULT_NORM`). */
export const DEFAULT_BBL_REQUIREMENT: BblRequirement = {
  dm3PerM2: 0.7,
  minimumDm3s: 7,
  type: "exhaust",
};

/** Default gebruiksfunctie voor een nog niet geclassificeerde ruimte. */
export const DEFAULT_BBL_FUNCTION: BblFunctionKey = "verblijfsruimte";

/**
 * Bepaal de BBL-eisregel voor een gebruiksfunctie-sleutel. Onbekende sleutel →
 * {@link DEFAULT_BBL_REQUIREMENT}.
 */
export function bblRequirementFor(fn: string): BblRequirement {
  return (
    (BBL_REQUIREMENTS as Record<string, BblRequirement>)[fn] ??
    DEFAULT_BBL_REQUIREMENT
  );
}

/**
 * Per-ruimte BBL-eis in dm³/s = `max(oppervlak × dm3PerM2, minimum)`.
 * Personen-toeslag wordt (bewust) overgeslagen — komt in delegatie 2.
 *
 * @param areaM2 vloeroppervlak in m²
 * @param fn gebruiksfunctie-sleutel
 */
export function bblDemandDm3s(areaM2: number, fn: string): number {
  const req = bblRequirementFor(fn);
  return Math.max(areaM2 * req.dm3PerM2, req.minimumDm3s);
}
