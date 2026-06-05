/**
 * climateData — typed loader voor de KNMI-klimaatdatalaag.
 *
 * Leest de statische, frontend-bundled KNMI-maandklimaatdata uit
 * `data/climate/knmiClimate.json` (via `resolveJsonModule`). Géén netwerk-/
 * backend-call: de bundel is een vast bestand in de repo, gegenereerd door
 * `scripts/generate_climate_bundle.py` (zie `_meta.how_to_regenerate` daarin).
 *
 * STANDALONE datalaag. Deze module voedt UITSLUITEND de vocht-/Glaser-
 * jaarbalans (NEN-EN-ISO 13788 maandmethode). Hij mag NOOIT in de
 * warmteverlies-θ_e-keten (ISSO 51/53, `constants.ts`) geïmporteerd worden:
 * de ontwerptemperatuur −10 °C en de Glaser steady-state winterconditie
 * blijven norm-vast en leven apart.
 *
 * De seed-selectie `"1991-2020"` voor station `"260"` (De Bilt) reproduceert
 * bit-gelijk `MONTHLY_CLIMATE_NL` uit `yearlyMoistureCalculation.ts`, zodat de
 * default-keuze géén resultaatwijziging geeft (backward-compat).
 */
import climateJson from "../data/climate/knmiClimate.json";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/**
 * Eén maand buitenklimaat. Superset van het bestaande
 * `MonthlyClimate` in `yearlyMoistureCalculation.ts` (`coverage` toegevoegd),
 * zodat het schoon in de Glaser-maandmethode valt.
 */
export interface MonthlyClimate {
  /** NL-maandlabel: "Jan" .. "Dec". */
  month: string;
  /** Gemiddelde buitentemperatuur θ_e [°C]. */
  thetaE: number;
  /** Gemiddelde relatieve buitenluchtvochtigheid RH_e [%]. */
  rhE: number;
  /** Kalenderdagen in de maand (schrikkeljaar-aware). */
  days: number;
  /** Fractie 0..1 geldige meetdagen in de aggregatie (alleen bij
   *  historische records uit de generator; ontbreekt bij normaal/seed). */
  coverage?: number;
}

/** Eén KNMI-station met coördinaten (voor latere postcode-auto-mapping). */
export interface Station {
  id: string;
  name: string;
  lat: number;
  lon: number;
}

/**
 * Klimaatselectie:
 *  - een historisch kalenderjaar (number, bv. 2022),
 *  - `"NEN5060"` (referentiejaar — placeholder tot de norm-tabel is ingevuld),
 *  - `"1991-2020"` (de KNMI-normaal),
 *  - `"forfaitair"` (de genormeerde, station-agnostische standaard NL-maandreeks;
 *    tevens de default). Levert dezelfde 12 maanden als het De Bilt
 *    `1991-2020`-record, maar generiek gelabeld en losgekoppeld van de
 *    stationkeuze. Zie {@link getMonthlyClimate}.
 */
export type YearSelection = number | "NEN5060" | "1991-2020" | "forfaitair";

/**
 * Eén optie voor de gecombineerde klimaat-dropdown.
 *
 * `value` is het encoded value-formaat (zie {@link encodeClimateValue}):
 *  - `"forfaitair"` voor de forfaitaire norm-reeks,
 *  - `"<stationId>|<year-or-key>"` voor een concrete KNMI-selectie
 *    (bv. `"260|2023"`).
 * `group` (optioneel) = stationnaam, voor `<optgroup>`-gruppering. Ontbreekt
 * bij de forfaitaire optie (die staat los bovenaan).
 */
export interface ClimateOption {
  value: string;
  label: string;
  group?: string;
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

/** Default-station: De Bilt (KNMI-referentiestation NL). */
export const CLIMATE_DEFAULT_STATION = "260" as const;

/** Sentinel-selectie: de forfaitaire (genormeerde) standaard NL-maandreeks,
 *  station-agnostisch. Tevens de default. */
export const CLIMATE_FORFAITAIR = "forfaitair" as const;

/** Bron-record voor de forfaitaire reeks: het De Bilt (260) `1991-2020`-record.
 *  De forfaitaire optie hergebruikt deze 12 maanden 1:1 (NIET gedupliceerd). */
const FORFAITAIR_SOURCE_STATION = "260" as const;
const FORFAITAIR_SOURCE_SELECTION = "1991-2020" as const;

/** Default-selectie: forfaitair (norm). Levert dezelfde waarden als het oude
 *  De Bilt `1991-2020`-record → geen resultaatwijziging t.o.v. de bestaande
 *  `MONTHLY_CLIMATE_NL`, alleen generiek gelabeld en stationonafhankelijk. */
export const CLIMATE_DEFAULT_SELECTION = CLIMATE_FORFAITAIR;

// ---------------------------------------------------------------------------
// JSON-contract (interne vorm van de bundel)
// ---------------------------------------------------------------------------

type RecordKind = "normal" | "reference" | "historical";

/** Eén maand zoals in de JSON: thetaE/rhE mogen `null` zijn (placeholder). */
interface RawMonth {
  month: string;
  thetaE: number | null;
  rhE: number | null;
  days: number;
  coverage?: number;
}

interface ClimateRecord {
  stationId: string;
  /** Serialisatie van `YearSelection`: een jaar (number) of "NEN5060" /
   *  "1991-2020". */
  selection: number | string;
  kind: RecordKind;
  months: RawMonth[];
}

interface ClimateBundle {
  stations: Station[];
  records: ClimateRecord[];
}

// `resolveJsonModule` levert de JSON al getypeerd; we casten naar het
// expliciete contract zodat de helpers een stabiel API-oppervlak houden. De
// `_meta`-sleutel (bron-/licentie-/instructietekst) wordt bewust genegeerd.
const bundle = climateJson as unknown as ClimateBundle;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Normaliseer een `YearSelection` naar de string-vorm in de JSON. */
function selectionKey(selection: YearSelection): string {
  return typeof selection === "number" ? String(selection) : selection;
}

/** Een record is "compleet" (bruikbaar) als alle 12 maanden numerieke
 *  thetaE én rhE hebben. Placeholder-records (NEN5060 met null) zijn dat niet. */
function isComplete(record: ClimateRecord): boolean {
  return (
    record.months.length === 12 &&
    record.months.every((m) => m.thetaE !== null && m.rhE !== null)
  );
}

// ---------------------------------------------------------------------------
// API
// ---------------------------------------------------------------------------

/** Alle bekende KNMI-stations, op naam gesorteerd. */
export function listStations(): Station[] {
  return [...bundle.stations].sort((a, b) => a.name.localeCompare(b.name, "nl"));
}

/**
 * Beschikbare selecties voor een station, in een vaste voorkeursvolgorde:
 * eerst de `1991-2020`-normaal (default), dan `NEN5060` (indien aanwezig,
 * óók als placeholder — zie `getMonthlyClimate` voor de null-afhandeling),
 * dan historische jaren oplopend.
 *
 * Lege array wanneer het station onbekend is.
 */
export function listAvailableYears(stationId: string): YearSelection[] {
  const recs = bundle.records.filter((r) => r.stationId === stationId);
  if (recs.length === 0) {
    return [];
  }

  const out: YearSelection[] = [];
  if (recs.some((r) => r.selection === "1991-2020")) {
    out.push("1991-2020");
  }
  if (recs.some((r) => r.selection === "NEN5060")) {
    out.push("NEN5060");
  }

  const years = recs
    .map((r) => r.selection)
    .filter((s): s is number => typeof s === "number")
    .sort((a, b) => a - b);
  out.push(...years);

  return out;
}

/** Map een ruwe bundel-maandlijst → publieke `MonthlyClimate[]` (na completeness-
 *  check; thetaE/rhE zijn dan gegarandeerd numeriek). */
function toMonthlyClimate(record: ClimateRecord): MonthlyClimate[] {
  return record.months.map((m) => {
    const out: MonthlyClimate = {
      month: m.month,
      thetaE: m.thetaE as number,
      rhE: m.rhE as number,
      days: m.days,
    };
    if (typeof m.coverage === "number") {
      out.coverage = m.coverage;
    }
    return out;
  });
}

/**
 * 12 maanden (Jan–Dec) klimaat voor een station + selectie, of `null` wanneer:
 *  - het station/selectie-record niet bestaat, of
 *  - het record een placeholder is (NEN5060 met nog niet-ingevulde maanden).
 *
 * Bij `selection === "forfaitair"` wordt `stationId` GENEGEERD: er komt altijd
 * de genormeerde standaard NL-maandreeks terug (= het De Bilt `1991-2020`-
 * record, uit de bundel gelezen — niet gedupliceerd).
 *
 * De maanden komen terug in de bundel-volgorde (Jan–Dec).
 */
export function getMonthlyClimate(
  stationId: string,
  selection: YearSelection,
): MonthlyClimate[] | null {
  // Forfaitair: station-agnostisch, altijd de De Bilt 1991-2020-bronreeks.
  if (selection === CLIMATE_FORFAITAIR) {
    const record = bundle.records.find(
      (r) =>
        r.stationId === FORFAITAIR_SOURCE_STATION &&
        selectionKey(r.selection as YearSelection) === FORFAITAIR_SOURCE_SELECTION,
    );
    if (!record || !isComplete(record)) {
      return null;
    }
    return toMonthlyClimate(record);
  }

  const key = selectionKey(selection);
  const record = bundle.records.find(
    (r) => r.stationId === stationId && selectionKey(r.selection as YearSelection) === key,
  );
  if (!record || !isComplete(record)) {
    return null;
  }

  return toMonthlyClimate(record);
}

// ---------------------------------------------------------------------------
// Single-dropdown: value-encoding + gecombineerde optielijst
// ---------------------------------------------------------------------------

/**
 * Value-formaat voor de gecombineerde klimaat-dropdown:
 *  - `"forfaitair"` → de forfaitaire norm-reeks (station-agnostisch),
 *  - `"<stationId>|<year-or-key>"` → een concrete KNMI-selectie
 *    (bv. `"260|2023"` of `"240|NEN5060"`).
 */
export function encodeClimateValue(
  stationId: string,
  selection: YearSelection,
): string {
  if (selection === CLIMATE_FORFAITAIR) {
    return CLIMATE_FORFAITAIR;
  }
  return `${stationId}|${selectionKey(selection)}`;
}

/**
 * Inverse van {@link encodeClimateValue}. Mapt een dropdown-value terug naar
 * `(stationId, selection)`. Voor de forfaitaire optie is `stationId` het
 * default-station (genegeerd door `getMonthlyClimate`, maar gevuld voor
 * label-/state-consistentie).
 */
export function decodeClimateValue(value: string): {
  stationId: string;
  selection: YearSelection;
} {
  if (value === CLIMATE_FORFAITAIR) {
    return {
      stationId: CLIMATE_DEFAULT_STATION,
      selection: CLIMATE_FORFAITAIR,
    };
  }
  const sep = value.indexOf("|");
  const stationId = value.slice(0, sep);
  const rawSel = value.slice(sep + 1);
  // Numeriek jaar → number; anders een string-key ("NEN5060" / "1991-2020").
  const selection: YearSelection = /^\d+$/.test(rawSel)
    ? Number(rawSel)
    : (rawSel as YearSelection);
  return { stationId, selection };
}

/** Leesbaar label per (niet-forfaitaire) selectie. */
function selectionLabel(selection: YearSelection): string {
  if (selection === "1991-2020") return "1991-2020 (normaal)";
  if (selection === "NEN5060") return "NEN5060 (referentie)";
  return String(selection);
}

/**
 * Eén gecombineerde optielijst voor de single-dropdown:
 *
 *  1. `{ value: "forfaitair", label: "Forfaitair (norm)" }` — altijd EERST.
 *  2. Daarna per station de beschikbare KNMI-selecties, gegroepeerd via
 *     `group = stationnaam`, met label `"<station> — <selectie>"`.
 *
 * Het De Bilt `1991-2020`-record wordt WEGGELATEN: dat is nu de forfaitaire
 * optie, dus opnemen zou een dubbele identieke entry geven. De `NEN5060`-
 * placeholder wordt eveneens WEGGELATEN uit deze lijst (hij levert via
 * `getMonthlyClimate` toch `null` → geen bruikbare keuze; consistent met het
 * vermijden van niet-selecteerbare ruis). Alleen complete historische
 * jaarrecords komen erin.
 */
export function listClimateOptions(): ClimateOption[] {
  const out: ClimateOption[] = [
    { value: CLIMATE_FORFAITAIR, label: "Forfaitair (norm)" },
  ];

  for (const station of listStations()) {
    const years = listAvailableYears(station.id);
    for (const sel of years) {
      // Forfaitair-bron (De Bilt 1991-2020) en placeholder NEN5060 overslaan.
      if (sel === "1991-2020" || sel === "NEN5060") {
        continue;
      }
      out.push({
        value: encodeClimateValue(station.id, sel),
        label: `${station.name} — ${selectionLabel(sel)}`,
        group: station.name,
      });
    }
  }

  return out;
}
