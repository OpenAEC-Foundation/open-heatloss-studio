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
// Weergave-eenheid (UI-voorkeur) — store blijft ALTIJD dm³/s
// ---------------------------------------------------------------------------

/**
 * Weergave-eenheid voor luchtdebieten in de UI. Puur een **weergave**-keuze
 * (persistent via `components/ventilation/ventilationUiStore.ts`): de store en
 * alle berekeningen blijven in dm³/s, conversie gebeurt uitsluitend aan de
 * UI-rand via {@link flowToDisplay} / {@link flowFromDisplay}.
 */
export type FlowDisplayUnit = "dm3s" | "m3h";

/** Eenheid-labels voor weergave. */
export const FLOW_UNIT_LABELS: Record<FlowDisplayUnit, string> = {
  dm3s: "dm³/s",
  m3h: "m³/h",
};

/**
 * Weergave-decimalen per eenheid: dm³/s op 1 decimaal (bestaande conventie,
 * zie `flowLabel`), m³/h op hele getallen (bestaande conventie, zie
 * `m3hLabel`). Alleen voor **weergave** — store-waarden niet afronden.
 */
export const FLOW_UNIT_DECIMALS: Record<FlowDisplayUnit, number> = {
  dm3s: 1,
  m3h: 0,
};

/** De andere eenheid (voor secundaire weergave tussen haakjes). */
export function otherFlowUnit(unit: FlowDisplayUnit): FlowDisplayUnit {
  return unit === "dm3s" ? "m3h" : "dm3s";
}

/**
 * Store-waarde (dm³/s) → weergavewaarde in de gekozen eenheid. **Onafgerond**
 * — afronden gebeurt pas bij het formatteren ({@link FLOW_UNIT_DECIMALS}).
 */
export function flowToDisplay(dm3s: number, unit: FlowDisplayUnit): number {
  return unit === "m3h" ? dm3sToM3h(dm3s) : dm3s;
}

/**
 * Invoerwaarde in de gekozen eenheid → store-waarde (dm³/s). **Onafgerond**:
 * de exacte deling door 3,6 gaat de store in, zodat invoer in m³/h bij
 * terugschakelen exact dezelfde dm³/s-waarde oplevert (geen afrondingsdrift).
 */
export function flowFromDisplay(value: number, unit: FlowDisplayUnit): number {
  return unit === "m3h" ? m3hToDm3s(value) : value;
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
  /**
   * Optionele bezetting (aantal personen) voor de personen-toeslag op de
   * BBL-eis: `eis = max(opp × dm3/m², personen × pp-debiet, minimum)`.
   * `undefined` = geen toeslag. Port van `aantal_personen` /
   * `_bereken_ventilatie_eis` uit de pyRevit-plugin
   * (`VentilatieBalans.pushbutton/script.py:272-289`).
   */
  occupancy?: number;
}

// ---------------------------------------------------------------------------
// WTW/MV-units (gebouwniveau)
// ---------------------------------------------------------------------------

/** Unit-type: balansventilatie met warmteterugwinning of mechanische afvoerbox. */
export type VentilationUnitType = "wtw" | "mv";

/** Herkomst van de unit: catalogus-snapshot of handmatig ingevoerd. */
export type VentilationUnitSource = "catalog" | "custom";

/**
 * Eén WTW- of MV-unit. Port van het unit-record uit de pyRevit-plugin
 * (`VentilatieBalans.pushbutton/script.py:117-126`, `load_units_database()`:
 * `{wtw_units:[], mv_units:[]}` met fabrikant/model/capaciteit_m3h/rendement/
 * geluid).
 *
 * Catalogus-units worden bij toewijzing als **snapshot** gekopieerd naar
 * `VentilationState.units` (source `"catalog"`), zodat een opgeslagen project
 * niet stilzwijgend verandert wanneer de seed-catalogus
 * (`data/ventilationUnits.json`) wordt bijgewerkt.
 *
 * **Capaciteit in m³/h** — fabrikant-conventie; de rest van de codebase rekent
 * in dm³/s, omrekenen via {@link m3hToDm3s} gebeurt in de capaciteitstoets.
 */
export interface VentilationUnit {
  id: string;
  type: VentilationUnitType;
  fabrikant: string;
  model: string;
  /** Nominale capaciteit in m³/h (indicatief — controleer fabrikantgegevens). */
  capaciteitM3h: number;
  /** WTW-rendement als fractie 0–1. Alleen zinvol bij type `"wtw"`. */
  rendement?: number;
  /** Geluidsniveau in dB(A). Optioneel, informatief. */
  geluidDb?: number;
  source: VentilationUnitSource;
}

/**
 * Toewijzing van een unit (uit {@link VentilationState.units}) met aantal.
 * Port van `ZoneUnitToewijzing.voeg_unit_toe` (plugin r.304-321:
 * totaalcapaciteit = Σ capaciteit × aantal).
 *
 * Toewijzing is op **gebouwniveau** — het webmodel kent (nog) geen
 * zone-concept. `zoneId` is zone-ready voorbereid voor een latere
 * zone-indeling en blijft tot die tijd `undefined`.
 */
export interface VentilationUnitAssignment {
  /** Verwijst naar `VentilationUnit.id` in `VentilationState.units`. */
  unitId: string;
  /** Aantal toestellen van deze unit (≥ 1). */
  aantal: number;
  /** Zone-ready (toekomstige zone-indeling); nu altijd `undefined`. */
  zoneId?: string;
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
  /**
   * Gebouw-niveau ventilatiesysteem (NL-standaard A–D). Optioneel zodat
   * oude opgeslagen projecten zonder dit veld blijven laden — `undefined`
   * valt terug op {@link DEFAULT_VENTILATION_SYSTEM} via
   * {@link ventilationSystemOf}.
   */
  system?: VentilationSystemKey;
  /**
   * Project-unitbibliotheek: catalogus-snapshots + custom units waar de
   * toewijzingen naar verwijzen. Optioneel — oude opgeslagen projecten
   * zonder dit veld blijven laden (`undefined` = geen units).
   */
  units?: VentilationUnit[];
  /**
   * Toegewezen units (gebouwniveau) met aantal. Optioneel — zelfde
   * backward-compat-regel als {@link VentilationState.units}.
   */
  unitAssignments?: VentilationUnitAssignment[];
}

export const DEFAULT_VENTILATION_STATE: VentilationState = {
  terminals: [],
  rooms: {},
};

// ---------------------------------------------------------------------------
// Ventilatiesysteem A–D (gebouw-niveau)
// ---------------------------------------------------------------------------

/**
 * NL-standaard ventilatiesystemen. De pyRevit-plugin
 * (`VentilatieBalans.pushbutton/script.py`) kent géén systeemlijst (alleen
 * WTW/MV-unitkeuze), dus we hanteren de standaard NEN 1087-indeling:
 *   - A — natuurlijke toevoer + natuurlijke afvoer
 *   - B — mechanische toevoer + natuurlijke afvoer
 *   - C — natuurlijke toevoer + mechanische afvoer
 *   - D — balansventilatie (WTW): mechanische toevoer + afvoer
 */
export type VentilationSystemKey = "A" | "B" | "C" | "D";

/** Metadata per ventilatiesysteem: label + welke kant mechanisch is. */
export interface VentilationSystemInfo {
  key: VentilationSystemKey;
  label: string;
  /** Toevoer mechanisch (via ventielen) — anders natuurlijk via gevelroosters. */
  supplyMechanical: boolean;
  /** Afvoer mechanisch (via ventielen) — anders natuurlijk (bv. kanalen/schacht). */
  exhaustMechanical: boolean;
}

export const VENTILATION_SYSTEMS: Record<VentilationSystemKey, VentilationSystemInfo> = {
  A: {
    key: "A",
    label: "Systeem A — natuurlijke toevoer + natuurlijke afvoer",
    supplyMechanical: false,
    exhaustMechanical: false,
  },
  B: {
    key: "B",
    label: "Systeem B — mechanische toevoer + natuurlijke afvoer",
    supplyMechanical: true,
    exhaustMechanical: false,
  },
  C: {
    key: "C",
    label: "Systeem C — natuurlijke toevoer + mechanische afvoer",
    supplyMechanical: false,
    exhaustMechanical: true,
  },
  D: {
    key: "D",
    label: "Systeem D — balansventilatie (WTW)",
    supplyMechanical: true,
    exhaustMechanical: true,
  },
};

/**
 * Default systeem voor projecten zonder expliciete keuze. Systeem C is de
 * meest gangbare bestaande-bouw/nieuwbouw-default in NL woningbouw en matcht
 * de `system_c`-default van het calc-model (`projectStore.DEFAULT_PROJECT`).
 */
export const DEFAULT_VENTILATION_SYSTEM: VentilationSystemKey = "C";

/** Effectief systeem van een sidecar-state (default-fallback voor oude files). */
export function ventilationSystemOf(
  state: Pick<VentilationState, "system"> | undefined,
): VentilationSystemInfo {
  return VENTILATION_SYSTEMS[state?.system ?? DEFAULT_VENTILATION_SYSTEM];
}

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
 * `VentilatieBalans.pushbutton/script.py:54-79`, uitgebreid met de
 * per-persoon-eisen van Bbl artikel 4.122 lid 2 (`personDm3s`).
 */
export interface BblRequirement {
  /** Specifiek debiet in dm³/(s·m²). */
  dm3PerM2: number;
  /** Minimum-debiet in dm³/s. */
  minimumDm3s: number;
  /** Toevoer / afvoer / geen. */
  type: BblRequirementType;
  /**
   * Per-persoon-eis in dm³/s per persoon — Bbl artikel 4.122 **lid 2**
   * (overige gebruiksfuncties; via iplo.nl geverifieerd 2026-06-10). Wanneer
   * gezet is de gebruiksfunctie **persoon-gebaseerd**: de eis is
   * `personen × personDm3s` (zie {@link bblDemandDm3s}); zonder ingevulde
   * bezetting valt de berekening terug op de m²-benadering en is de eis
   * **indicatief** ({@link isBblDemandIndicative}).
   * `undefined` = oppervlakte-gebaseerd (lid 1, woonfunctie-achtig).
   */
  personDm3s?: number;
}

/**
 * BBL-eisentabel per gebruiksfunctie (geport uit `NORMEN_BBL`).
 *
 * Sleutels zijn de Nederlandse gebruiksfunctie-namen uit de plugin, zodat een
 * latere Revit-import 1:1 mapt. Minima en specifieke debieten in dm³/s
 * respectievelijk dm³/(s·m²).
 *
 * **Norm-grondslag (Bbl artikel 4.122, via iplo.nl geverifieerd 2026-06-10):**
 *   - lid 1 (woonfunctie): verblijfsruimte 0,7 / verblijfsgebied 0,9
 *     dm³/(s·m²), minimum 7 dm³/s;
 *   - lid 2 (overige gebruiksfuncties, **per persoon**, `personDm3s`):
 *     gezondheidszorg bedgebied 12 · onderwijs + gezondheidszorg overig 8,5 ·
 *     kantoor/industrie/sport 6,5 · winkel/bijeenkomst (niet-kinderopvang) 4 ·
 *     bijeenkomst-kinderopvang 6,5 dm³/s per persoon;
 *   - lid 3: opstelplaats kooktoestel (keuken) 21 dm³/s;
 *   - lid 5: toiletruimte 7 / badruimte 14 dm³/s (alle functies).
 */
export const BBL_REQUIREMENTS = {
  bijeenkomstfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 4 },
  "bijeenkomstfunctie (kinderopvang)": { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 6.5 },
  kantoorfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 6.5 },
  industriefunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 6.5 },
  onderwijsfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 8.5 },
  sportfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 6.5 },
  winkelfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 4 },
  woonfunctie: { dm3PerM2: 0.7, minimumDm3s: 7, type: "supply" },
  gezondheidszorgfunctie: { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 8.5 },
  "gezondheidszorgfunctie (bedgebied)": { dm3PerM2: 0.9, minimumDm3s: 7, type: "supply", personDm3s: 12 },
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
 * Default ventilatiedebiet per persoon in dm³/s voor de **woonfunctie-
 * personentoeslag**. Port van `dm3_per_persoon` uit de pyRevit-plugin
 * (`VentilatieBalans.pushbutton/script.py:336` / NumericUpDown-default r.421:
 * 4,0 dm³/s = 14,4 m³/h pp).
 *
 * Geldt ALLEEN voor oppervlakte-gebaseerde functies (Bbl 4.122 lid 1,
 * woonfunctie e.d.) — persoon-gebaseerde utiliteitsfuncties gebruiken hun
 * eigen `personDm3s` uit {@link BBL_REQUIREMENTS} (lid 2).
 */
export const DEFAULT_OCCUPANCY_DM3S_PER_PERSON = 4.0;

/**
 * Is de afgeleide BBL-eis voor deze functie+bezetting **indicatief**?
 *
 * `true` voor een persoon-gebaseerde gebruiksfunctie (Bbl 4.122 lid 2,
 * `personDm3s` gezet) zónder ingevulde bezetting: {@link bblDemandDm3s} valt
 * dan terug op de m²-benadering, terwijl de wettelijke eis per persoon geldt.
 * UI en rapport markeren dit als "indicatief — bezetting invullen".
 */
export function isBblDemandIndicative(
  fn: string,
  occupancy?: number,
): boolean {
  const req = bblRequirementFor(fn);
  return (
    req.personDm3s !== undefined && !(occupancy !== undefined && occupancy > 0)
  );
}

/**
 * Per-ruimte BBL-eis in dm³/s.
 *
 * **Eis-formule per functie-type (Bbl artikel 4.122, geverifieerd via iplo.nl
 * 2026-06-10):**
 *   - **Oppervlakte-gebaseerd** (lid 1 — woonfunctie, verblijfsruimte/-gebied,
 *     logies; en de afvoerruimtes uit lid 3/5):
 *     `eis = max(oppervlak × dm3PerM2, personen × pp-toeslag, minimum)` —
 *     de optionele personen-toeslag van 4,0 dm³/s pp is de plugin-conventie
 *     uit `_bereken_ventilatie_eis` (`VentilatieBalans.pushbutton/
 *     script.py:282-289`), geen wettelijke eis.
 *   - **Persoon-gebaseerd** (lid 2 — utiliteitsfuncties met `personDm3s`):
 *     `eis = max(personen × personDm3s, minimum)`. Zonder ingevulde bezetting
 *     valt de berekening terug op de m²-benadering
 *     (`max(oppervlak × dm3PerM2, minimum)`) — die uitkomst is **indicatief**
 *     (zie {@link isBblDemandIndicative}; UI/rapport markeren dit).
 *
 * @param areaM2 vloeroppervlak in m²
 * @param fn gebruiksfunctie-sleutel
 * @param occupancy aantal personen (`undefined`/0 = geen bezetting)
 * @param dm3sPerPerson woonfunctie-pp-toeslag in dm³/s (default
 *   {@link DEFAULT_OCCUPANCY_DM3S_PER_PERSON}); persoon-gebaseerde functies
 *   negeren deze parameter en gebruiken hun eigen `personDm3s`.
 */
export function bblDemandDm3s(
  areaM2: number,
  fn: string,
  occupancy?: number,
  dm3sPerPerson: number = DEFAULT_OCCUPANCY_DM3S_PER_PERSON,
): number {
  const req = bblRequirementFor(fn);
  const hasOccupancy = occupancy !== undefined && occupancy > 0;

  // Persoon-gebaseerde utiliteitsfunctie (Bbl 4.122 lid 2).
  if (req.personDm3s !== undefined && hasOccupancy) {
    return Math.max(occupancy * req.personDm3s, req.minimumDm3s);
  }

  // Oppervlakte-gebaseerd (lid 1) — óf de indicatieve m²-fallback voor een
  // persoon-gebaseerde functie zonder bezetting.
  let demand = areaM2 * req.dm3PerM2;
  if (req.personDm3s === undefined && hasOccupancy && dm3sPerPerson > 0) {
    demand = Math.max(demand, occupancy * dm3sPerPerson);
  }
  return Math.max(demand, req.minimumDm3s);
}
