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
 *  - `"1991-2020"` (de KNMI-normaal; tevens de default).
 */
export type YearSelection = number | "NEN5060" | "1991-2020";

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

/** Default-station: De Bilt (KNMI-referentiestation NL). */
export const CLIMATE_DEFAULT_STATION = "260" as const;

/** Default-selectie: de 1991-2020-normaal (= huidige forfaitaire waarden;
 *  geen resultaatwijziging t.o.v. de bestaande `MONTHLY_CLIMATE_NL`). */
export const CLIMATE_DEFAULT_SELECTION = "1991-2020" as const;

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

/**
 * 12 maanden (Jan–Dec) klimaat voor een station + selectie, of `null` wanneer:
 *  - het station/selectie-record niet bestaat, of
 *  - het record een placeholder is (NEN5060 met nog niet-ingevulde maanden).
 *
 * De maanden komen terug in de bundel-volgorde (Jan–Dec).
 */
export function getMonthlyClimate(
  stationId: string,
  selection: YearSelection,
): MonthlyClimate[] | null {
  const key = selectionKey(selection);
  const record = bundle.records.find(
    (r) => r.stationId === stationId && selectionKey(r.selection as YearSelection) === key,
  );
  if (!record || !isComplete(record)) {
    return null;
  }

  return record.months.map((m) => {
    const out: MonthlyClimate = {
      month: m.month,
      // Veilig na `isComplete`: thetaE/rhE zijn hier gegarandeerd numeriek.
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
