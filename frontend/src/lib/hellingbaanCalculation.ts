/**
 * Hellingbaan (parkeergarage) — rekenkern voor dimensionering volgens NEN 2443.
 *
 * Frontend-only rekenmodel (zelfde patroon als `hwaCalculation.ts`): puur-TS,
 * state-loos, geen Rust/API. De rekenlogica is 1-op-1 geport uit de
 * bestaande pyRevit-tool — `pyrevit/extensions/bouwkunde.extension/
 * Bouwkunde.tab/Bouwbesluit.panel/HellingbaanGenerator.pushbutton/script.py`,
 * functie `bereken_hellingbaan` (regels ~68-151). Alleen de rekenkern is
 * geport, NIET de Revit/DirectShape-geometrie.
 *
 * **Zone-indeling:**
 * - `"vast"` — garagetype heeft `min === max` (geen bandbreedte).
 * - `"kort"` — hoogte ≤ `LEN_MIN` bij max-helling → helling = max.
 * - `"lang"` — hoogte ≥ `LEN_MAX` bij min-helling → helling = min.
 * - `"midden"` — kwadratische optimalisatie: de helling die de
 *   hellingbaanlengte lineair laat interpoleren tussen `LEN_MIN` (bij
 *   max-helling) en `LEN_MAX` (bij min-helling), rekening houdend met de
 *   vaste overgangslengte. Zie de discriminant-oplossing in
 *   {@link calculateHellingbaan}.
 * - `"simpel"` — `metOvergang: false`, geen zone-logica: helling = max
 *   (of de override), rechte lijn zonder overgangshellingen.
 *
 * **Segmenten (bij `metOvergang: true`):** overgang onder (voetboog, lengte
 * = volle wielbasis) → hoofdhelling → overgang boven (topboog, lengte =
 * halve wielbasis), beide overgangen op de HALVE gebruikte helling.
 *
 * **Bekende bron-discrepantie (2770/1385 vs. 2720/1360):** de pyRevit-tool
 * rekent de overgangslengtes uit de volle/halve wielbasis (2770 mm resp.
 * 1385 mm). Het rekenblad van de eigenaar gebruikt afgeronde waarden
 * 2720/1360 mm. Bij een referentiecase (3600 mm, stalling, 16% override)
 * geeft dat een lichte afwijking in de totale lengte — zie
 * `hellingbaanCalculation.test.ts` voor de exacte getallen. Dit rekenmodel
 * volgt de pyRevit-waarden (bron van waarheid volgens de opdracht); de
 * rekenblad-waarde staat uitsluitend ter documentatie in
 * {@link WIELBASIS_MM.reference}.
 */

import type {
  HellingbaanGarageType,
  HellingbaanGarageTypeId,
  HellingbaanInput,
  HellingbaanReferentieResult,
  HellingbaanResult,
  HellingbaanSegment,
  HellingbaanZone,
  SourcedValue,
} from "../types/hellingbaan";

// ---------------------------------------------------------------------------
// Normconstanten
// ---------------------------------------------------------------------------

const WIELBASIS_BRON =
  "pyRevit HellingbaanGenerator / NEN 2443 (wielbasis-gebaseerde overgangslengtes; nog te verifiëren tegen normtekst — het rekenblad van de eigenaar gebruikt 2720/1360)";

/** Wielbasis (mm) — grondslag voor de overgangslengtes (voet = volle, top = halve wielbasis). */
export const WIELBASIS_MM: SourcedValue<number> = {
  value: 2770,
  source: "pyRevit-referentie",
  reference: WIELBASIS_BRON,
};

/** Lengte van de overgangshelling aan de voet (mm) — gelijk aan de volle wielbasis. */
export const OVERGANG_VOET_MM: SourcedValue<number> = {
  value: WIELBASIS_MM.value,
  source: "pyRevit-referentie",
  reference: WIELBASIS_BRON,
};

/** Lengte van de overgangshelling aan de top (mm) — gelijk aan de halve wielbasis. */
export const OVERGANG_TOP_MM: SourcedValue<number> = {
  value: WIELBASIS_MM.value / 2,
  source: "pyRevit-referentie",
  reference: WIELBASIS_BRON,
};

/** Som van de twee overgangslengtes (mm) — gebruikt in de kwadratische midden-zone-oplossing. */
export const OVERGANG_TOTAAL_MM = OVERGANG_VOET_MM.value + OVERGANG_TOP_MM.value;

/** Minimale hellingbaanlengte (mm) die de kort-zone begrenst (bij max-helling van het type). */
export const LEN_MIN_MM: SourcedValue<number> = {
  value: 10000,
  source: "pyRevit-referentie",
  reference:
    "pyRevit HellingbaanGenerator / NEN 2443 (minimale hellingbaanlengte, ondergrens van de midden-zone)",
};

/** Maximale hellingbaanlengte (mm) die de lang-zone begrenst (bij min-helling van het type). */
export const LEN_MAX_MM: SourcedValue<number> = {
  value: 40000,
  source: "pyRevit-referentie",
  reference:
    "pyRevit HellingbaanGenerator / NEN 2443 (maximale hellingbaanlengte, bovengrens van de midden-zone)",
};

/** Bovengrens voor een handmatige helling-override (%) — zelfde grens als de pyRevit-UI (`0 < val ≤ 30`). */
export const HELLING_OVERRIDE_MAX_PERCENT = 30;

/**
 * Garagetypes met helling- en breedtegrenzen volgens NEN 2443, 1-op-1
 * overgenomen uit de pyRevit-tool. `maxHellingPercent` bepaalt de kort-zone
 * (kortste toelaatbare baan), `minHellingPercent` de lang-zone.
 */
export const GARAGE_TYPES: SourcedValue<ReadonlyArray<HellingbaanGarageType>> = {
  value: [
    { id: "openbaar", maxHellingPercent: 14, minHellingPercent: 14, breedteMinMm: 3000 },
    { id: "openbaar_dhumy", maxHellingPercent: 15, minHellingPercent: 14, breedteMinMm: 3000 },
    { id: "niet_openbaar", maxHellingPercent: 20, minHellingPercent: 14, breedteMinMm: 2750 },
    { id: "stalling", maxHellingPercent: 24, minHellingPercent: 14, breedteMinMm: 2750 },
  ],
  source: "pyRevit-referentie",
  reference:
    "pyRevit HellingbaanGenerator / NEN 2443 — garagetype-indeling met helling- en breedtegrenzen (openbaar 14/14% b≥3000 mm, openbaar d'Humy 15/14% b≥3000 mm, niet-openbaar 20/14% b≥2750 mm, stalling 24/14% b≥2750 mm)",
};

/** Zoek een garagetype op id; gooit bij een onbekend id (programmeerfout, geen gebruikersinvoer-pad). */
export function getGarageType(id: HellingbaanGarageTypeId): HellingbaanGarageType {
  const found = GARAGE_TYPES.value.find((g) => g.id === id);
  if (!found) {
    throw new Error(`onbekend garagetype: ${id}`);
  }
  return found;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Helling in % → hellingshoek in graden (`atan`). */
export function hellingPercentToDegrees(hellingPercent: number): number {
  return (Math.atan(hellingPercent / 100) * 180) / Math.PI;
}

interface SegmentBuildResult {
  segments: HellingbaanSegment[];
  lengteTotaalMm: number;
  warning: string | null;
}

/**
 * Bouw de segmenten van de hellingbaan op voor een gegeven hoogte en
 * (reeds bepaalde) helling. Gedeeld door {@link calculateHellingbaan} en
 * {@link calculateHellingbaanReferentie} zodat beide exact hetzelfde
 * segment-model gebruiken.
 */
function buildSegments(
  hoogteMm: number,
  hellingPercent: number,
  metOvergang: boolean,
): SegmentBuildResult {
  const hellingGraden = hellingPercentToDegrees(hellingPercent);

  if (!metOvergang) {
    if (hellingPercent <= 0) {
      return {
        segments: [{ type: "enkel", lengteMm: 0, hellingPercent, hellingGraden, hoogteMm }],
        lengteTotaalMm: 0,
        warning: `helling (${hellingPercent}%) moet groter zijn dan 0%, lengte niet te berekenen`,
      };
    }
    const lengteMm = (hoogteMm / hellingPercent) * 100;
    return {
      segments: [{ type: "enkel", lengteMm, hellingPercent, hellingGraden, hoogteMm }],
      lengteTotaalMm: lengteMm,
      warning: null,
    };
  }

  const overgangHellingPercent = hellingPercent / 2;
  const overgangHellingGraden = hellingPercentToDegrees(overgangHellingPercent);
  const overgangOnderLengteMm = OVERGANG_VOET_MM.value;
  const overgangBovenLengteMm = OVERGANG_TOP_MM.value;

  const overgangOnderHoogteMm = (overgangOnderLengteMm * overgangHellingPercent) / 100;
  const overgangBovenHoogteMm = (overgangBovenLengteMm * overgangHellingPercent) / 100;

  let hoofdHoogteMm = hoogteMm - overgangOnderHoogteMm - overgangBovenHoogteMm;
  let warning: string | null = null;

  if (hoofdHoogteMm < 0) {
    warning = `de overgangshellingen overbruggen samen al ${(overgangOnderHoogteMm + overgangBovenHoogteMm).toFixed(0)} mm, meer dan het hoogteverschil (${hoogteMm} mm) — hoofdsegment op 0 mm gezet, verlaag de helling of vergroot het hoogteverschil`;
    hoofdHoogteMm = 0;
  } else if (hellingPercent <= 0) {
    warning = `helling (${hellingPercent}%) moet groter zijn dan 0%, hoofdsegment-lengte niet te berekenen`;
  }

  const hoofdLengteMm = hellingPercent > 0 ? (hoofdHoogteMm / hellingPercent) * 100 : 0;

  const segments: HellingbaanSegment[] = [
    {
      type: "overgang_onder",
      lengteMm: overgangOnderLengteMm,
      hellingPercent: overgangHellingPercent,
      hellingGraden: overgangHellingGraden,
      hoogteMm: overgangOnderHoogteMm,
    },
    {
      type: "hoofd",
      lengteMm: hoofdLengteMm,
      hellingPercent,
      hellingGraden,
      hoogteMm: hoofdHoogteMm,
    },
    {
      type: "overgang_boven",
      lengteMm: overgangBovenLengteMm,
      hellingPercent: overgangHellingPercent,
      hellingGraden: overgangHellingGraden,
      hoogteMm: overgangBovenHoogteMm,
    },
  ];

  const lengteTotaalMm = segments.reduce((sum, s) => sum + s.lengteMm, 0);

  return { segments, lengteTotaalMm, warning };
}

/**
 * Bepaal de zone en de norm-berekende helling (%) voor een gegeven hoogte
 * en garagetype — de kwadratische midden-zone-oplossing uit
 * `bereken_hellingbaan` (regels 110-121 van het pyRevit-script).
 */
function bepaalZoneEnHelling(
  hoogteMm: number,
  garageType: HellingbaanGarageType,
  metOvergang: boolean,
): { zone: HellingbaanZone; hellingBerekendPercent: number } {
  const { maxHellingPercent: maxHelling, minHellingPercent: minHelling } = garageType;

  if (!metOvergang) {
    return { zone: "simpel", hellingBerekendPercent: maxHelling };
  }

  if (maxHelling === minHelling) {
    return { zone: "vast", hellingBerekendPercent: maxHelling };
  }

  const hoogteMin = (LEN_MIN_MM.value * maxHelling) / 100;
  const hoogteMax = (LEN_MAX_MM.value * minHelling) / 100;

  if (hoogteMm <= hoogteMin) {
    return { zone: "kort", hellingBerekendPercent: maxHelling };
  }
  if (hoogteMm >= hoogteMax) {
    return { zone: "lang", hellingBerekendPercent: minHelling };
  }

  // Midden-zone: los de helling op waarvoor de hellingbaanlengte lineair
  // interpoleert tussen LEN_MIN (bij maxHelling) en LEN_MAX (bij minHelling),
  // inclusief de vaste overgangslengte (OVERGANG_TOTAAL_MM).
  const factor = (LEN_MAX_MM.value - LEN_MIN_MM.value) / (maxHelling - minHelling);
  const a = -factor;
  const b = LEN_MIN_MM.value + factor * maxHelling - OVERGANG_TOTAAL_MM / 2;
  const c = -hoogteMm * 100;
  const discriminant = b * b - 4 * a * c;

  let hellingBerekendPercent: number;
  if (discriminant >= 0) {
    const raw = (-b - Math.sqrt(discriminant)) / (2 * a);
    hellingBerekendPercent = Math.min(maxHelling, Math.max(minHelling, raw));
  } else {
    hellingBerekendPercent = minHelling;
  }

  return { zone: "midden", hellingBerekendPercent };
}

// ---------------------------------------------------------------------------
// Hoofdberekening
// ---------------------------------------------------------------------------

/** Bereken de hellingbaan-dimensionering voor de gegeven invoer. */
export function calculateHellingbaan(input: HellingbaanInput): HellingbaanResult {
  const warnings: string[] = [];
  const garageType = getGarageType(input.garageTypeId);

  const isBreedteOnderMinimum = input.breedteMm < garageType.breedteMinMm;
  if (isBreedteOnderMinimum) {
    warnings.push(
      `breedte (${input.breedteMm} mm) ligt onder de norm-minimumbreedte van ${garageType.breedteMinMm} mm voor dit garagetype`,
    );
  }

  let hoogteMm = input.hoogteMm;
  if (!Number.isFinite(hoogteMm) || hoogteMm < 0) {
    warnings.push(`hoogteverschil (${input.hoogteMm} mm) is ongeldig, gecorrigeerd naar 0 mm`);
    hoogteMm = 0;
  }

  const { zone, hellingBerekendPercent } = bepaalZoneEnHelling(
    hoogteMm,
    garageType,
    input.metOvergang,
  );

  const rawOverride = input.hellingOverridePercent;
  let isOverride = false;
  if (rawOverride !== undefined) {
    if (!Number.isFinite(rawOverride) || rawOverride <= 0 || rawOverride > HELLING_OVERRIDE_MAX_PERCENT) {
      warnings.push(
        `handmatige helling (${rawOverride}%) is ongeldig (moet tussen 0 en ${HELLING_OVERRIDE_MAX_PERCENT}% liggen), norm-berekende helling gebruikt`,
      );
    } else {
      isOverride = true;
    }
  }

  const hellingPercent = isOverride ? (rawOverride as number) : hellingBerekendPercent;

  const isOverrideBuitenZone =
    isOverride &&
    (hellingPercent < garageType.minHellingPercent || hellingPercent > garageType.maxHellingPercent);
  if (isOverrideBuitenZone) {
    warnings.push(
      `handmatige helling (${hellingPercent}%) ligt buiten de norm-bandbreedte [${garageType.minHellingPercent}%, ${garageType.maxHellingPercent}%] van dit garagetype`,
    );
  }

  const { segments, lengteTotaalMm, warning: segmentWarning } = buildSegments(
    hoogteMm,
    hellingPercent,
    input.metOvergang,
  );
  if (segmentWarning) warnings.push(segmentWarning);

  return {
    hellingPercent,
    hellingBerekendPercent,
    isOverride,
    zone,
    segments,
    lengteTotaalMm,
    breedteMm: input.breedteMm,
    isBreedteOnderMinimum,
    isOverrideBuitenZone,
    warnings,
  };
}

/**
 * Vergelijkingsberekening "zonder optimalisatie": rekent altijd met de
 * vaste max-helling van het garagetype (de kort-zone-waarde), ongeacht de
 * werkelijke zone of een eventuele override. Bedoeld om in de UI het
 * verschil met de zone-optimalisatie te tonen.
 *
 * **Let op — dit is GEEN toepasbaar alternatief in alle zones.** De
 * max-helling is volgens de zone-systematiek alleen zonder reductie
 * toegestaan zolang het hoogteverschil binnen de kort-zone valt (zie
 * {@link isReferentieNormConform}). In de midden- en lang-zone geeft deze
 * functie dus een NIET norm-conforme referentiewaarde — uitsluitend
 * bedoeld als illustratief contrast, niet als geldig alternatief. De UI
 * moet dat expliciet markeren aan de hand van `isReferentieNormConform`.
 */
export function calculateHellingbaanReferentie(
  input: Pick<HellingbaanInput, "hoogteMm" | "garageTypeId" | "metOvergang">,
): HellingbaanReferentieResult {
  const garageType = getGarageType(input.garageTypeId);
  const hoogteMm = Number.isFinite(input.hoogteMm) && input.hoogteMm >= 0 ? input.hoogteMm : 0;
  const hellingPercent = garageType.maxHellingPercent;
  const { lengteTotaalMm } = buildSegments(hoogteMm, hellingPercent, input.metOvergang);
  return { hellingPercent, lengteTotaalMm };
}

/**
 * Is de vaste max-helling van het garagetype (de waarde die
 * {@link calculateHellingbaanReferentie} gebruikt) norm-conform voor de
 * zone waarin het werkelijke hoogteverschil valt?
 *
 * Volgens de zone-systematiek (`bepaalZoneEnHelling`) mag de max-helling
 * zonder reductie alleen toegepast worden:
 * - in de `"kort"`-zone (hoogte ≤ LEN_MIN × max/100 — daar IS max-helling
 *   letterlijk de norm-berekende waarde),
 * - bij een `"vast"`-type (min === max, er is geen andere optie),
 * - in `"simpel"` (zonder overgangshellingen rekent de norm-berekening
 *   zelf ook altijd met de max-helling, ongeacht hoogte).
 *
 * In de `"midden"`- en `"lang"`-zone verlangt de norm juist een minder
 * steile helling (richting min-helling) voor grotere hoogteverschillen —
 * daar is de vaste max-helling-vergelijking dus NIET norm-conform.
 */
export function isReferentieNormConform(zone: HellingbaanZone): boolean {
  return zone === "kort" || zone === "vast" || zone === "simpel";
}
