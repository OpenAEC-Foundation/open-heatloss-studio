/**
 * Uitzetting — rekenkern voor thermische lengte-uitzetting + vochtzwelling
 * plaatmateriaal.
 *
 * Frontend-only rekenmodel (zelfde patroon als `hwaCalculation.ts` /
 * `hellingbaanCalculation.ts`): puur-TS, state-loos, geen Rust/API. Elke
 * normconstante draagt een {@link SourcedValue} met bronlabel — zie
 * `types/uitzetting.ts`.
 *
 * **A. Thermische uitzetting** (`Δl = α·ΔT·l₀`, 1-op-1 het "uitzetting"-
 * tabblad van het rekenblad van de eigenaar):
 * - `α` in 10⁻⁶/K (uit de materialenbibliotheek of handmatig), `ΔT` in K,
 *   `l₀` in m → `Δl [mm] = α · ΔT · l₀ · 10⁻³`.
 * - Referentie-ankers (staal l₀=1m, ref 20/min -10/max 60): krimp 0,36 mm,
 *   vergroting 0,48 mm. Zink idem: 1,08/1,44 mm. Beton (ref 20/min 17/max
 *   27): 0,036/0,084 mm.
 *
 * **B. Vochtzwelling plaatmateriaal** (EN 318, het "uitzetting hout"-
 * tabblad): `Δl [mm] = zwelling [mm/m per %RV] · ΔRV [%] · lengte [m]`.
 * Referentie-anker: 0,8 m, 50→65% RV → 0,654 mm toename; 50→35% → 0,654 mm
 * krimp (symmetrische ΔRV van 15 procentpunt in dit voorbeeld).
 */
import type {
  MoistureSwellingInput,
  MoistureSwellingResult,
  SourcedValue,
  ThermalExpansionInput,
  ThermalExpansionResult,
} from "../types/uitzetting";

// ---------------------------------------------------------------------------
// Normconstanten / defaults
// ---------------------------------------------------------------------------

/** Default referentietemperatuur (montage/opname) in °C. */
export const DEFAULT_REF_TEMP_C: SourcedValue<number> = {
  value: 20,
  source: "rekenblad-eigenaar",
  reference: "rekenblad-eigenaar, tabblad 'uitzetting' — referentietemperatuur bij montage",
};

/** Default minimumtemperatuur (buitentoepassing) in °C. */
export const DEFAULT_MIN_TEMP_C: SourcedValue<number> = {
  value: -10,
  source: "rekenblad-eigenaar",
  reference: "rekenblad-eigenaar, tabblad 'uitzetting' — minimumtemperatuur buitentoepassing",
};

/** Default maximumtemperatuur (buitentoepassing) in °C. */
export const DEFAULT_MAX_TEMP_C: SourcedValue<number> = {
  value: 60,
  source: "rekenblad-eigenaar",
  reference: "rekenblad-eigenaar, tabblad 'uitzetting' — maximumtemperatuur buitentoepassing (bv. donker beplaat/gevelbekleding in de zon)",
};

/**
 * Waarschuwingsdrempel (mm) op de ingevoerde lengte waarboven een dilatatie-
 * of schuifbevestiging overweegbaar is. Puur informatief/vuistregel — geen
 * normclaim, beïnvloedt de berekening niet.
 */
export const DILATATIE_WARNING_THRESHOLD_MM = 0.5;

/** Default relatieve luchtvochtigheid bij installatie, in %. */
export const DEFAULT_RV_INSTALL_PERCENT: SourcedValue<number> = {
  value: 50,
  source: "rekenblad-eigenaar",
  reference: "rekenblad-eigenaar, tabblad 'uitzetting hout' — RV bij installatie",
};

/** Default maximale relatieve luchtvochtigheid in het gebruiksklimaat, in %. */
export const DEFAULT_RV_MAX_PERCENT: SourcedValue<number> = {
  value: 65,
  source: "rekenblad-eigenaar",
  reference: "rekenblad-eigenaar, tabblad 'uitzetting hout' — RV max gebruiksklimaat",
};

/** Default minimale relatieve luchtvochtigheid in het gebruiksklimaat, in %. */
export const DEFAULT_RV_MIN_PERCENT: SourcedValue<number> = {
  value: 35,
  source: "rekenblad-eigenaar",
  reference: "rekenblad-eigenaar, tabblad 'uitzetting hout' — RV min gebruiksklimaat",
};

/**
 * Default lineaire zwelling in mm per strekkende meter per %RV — preset
 * "OSB klasse O2, EN 318". Handmatig overschrijfbaar voor ander
 * plaatmateriaal (bv. multiplex, spaanplaat) met een andere testwaarde.
 */
export const DEFAULT_SWELLING_MM_PER_M_PER_PERCENT: SourcedValue<number> = {
  value: 0.0545,
  source: "EN 318",
  reference: "EN 318 — lineaire zwelling, gangbare waarde OSB klasse O2",
};

/**
 * Diktezwelling na 24u onderdompeling (OSB klasse O2, EN 318) — statisch
 * weetje, GEEN onderdeel van de berekening in dit tabblad (die rekent
 * uitsluitend de lineaire zwelling door). Puur ter documentatie in de
 * bronvoetnoot.
 */
export const THICKNESS_SWELLING_NOTE_OSB_O2 =
  "Diktezwelling na 24u onderdompeling (OSB klasse O2, EN 318): ≤ 12,7 mm → max. 15%; > 12,7 mm → max. 10%. Statisch weetje, niet doorgerekend in dit tabblad.";

// ---------------------------------------------------------------------------
// A. Thermische uitzetting
// ---------------------------------------------------------------------------

/**
 * Bereken de thermische lengte-uitzetting (krimp + vergroting) voor één
 * lengtemaat. `Δl [mm] = α [10⁻⁶/K] · ΔT [K] · l₀ [m] · 10⁻³`.
 *
 * Geeft een nette waarschuwing terug (resultaat 0) wanneer `alphaPer1e6PerK`
 * `null` is — bv. een isolatie-, folie- of spouwmateriaal uit de
 * bibliotheek zonder zinvolle α — in plaats van te gooien: de tool moet
 * bruikbaar blijven ook als de gebruiker eerst een materiaal kiest en pas
 * daarna de temperaturen invult.
 */
export function calculateThermalExpansion(
  input: ThermalExpansionInput,
): ThermalExpansionResult {
  const warnings: string[] = [];

  if (input.alphaPer1e6PerK === null) {
    warnings.push(
      "geen α bekend voor dit materiaal (isolatie/folie/spouw of niet ingevuld) — geen thermische uitzetting berekend",
    );
    return {
      deltaTKrimpK: 0,
      deltaTUitzettingK: 0,
      krimpMm: 0,
      vergrotingMm: 0,
      krimpMmPerM: 0,
      vergrotingMmPerM: 0,
      warnings,
    };
  }

  if (input.lengthM < 0) {
    warnings.push(`lengte (${input.lengthM} m) is negatief, als 0 behandeld`);
  }
  const lengthM = Math.max(0, input.lengthM);

  const deltaTKrimpK = input.refTempC - input.minTempC;
  const deltaTUitzettingK = input.maxTempC - input.refTempC;

  if (deltaTKrimpK < 0) {
    warnings.push(
      `minimumtemperatuur (${input.minTempC}°C) ligt boven de referentietemperatuur (${input.refTempC}°C) — krimp-ΔT is negatief`,
    );
  }
  if (deltaTUitzettingK < 0) {
    warnings.push(
      `maximumtemperatuur (${input.maxTempC}°C) ligt onder de referentietemperatuur (${input.refTempC}°C) — uitzettings-ΔT is negatief`,
    );
  }

  const alpha = input.alphaPer1e6PerK;
  const krimpMmPerM = alpha * deltaTKrimpK * 1e-3;
  const vergrotingMmPerM = alpha * deltaTUitzettingK * 1e-3;
  const krimpMm = krimpMmPerM * lengthM;
  const vergrotingMm = vergrotingMmPerM * lengthM;

  if (
    Math.abs(krimpMm) >= DILATATIE_WARNING_THRESHOLD_MM ||
    Math.abs(vergrotingMm) >= DILATATIE_WARNING_THRESHOLD_MM
  ) {
    warnings.push(
      `uitzetting/krimp op deze lengte bereikt ≥ ${DILATATIE_WARNING_THRESHOLD_MM} mm — dilatatie/schuifbevestiging overwegen (informatief, geen normclaim)`,
    );
  }

  return {
    deltaTKrimpK,
    deltaTUitzettingK,
    krimpMm,
    vergrotingMm,
    krimpMmPerM,
    vergrotingMmPerM,
    warnings,
  };
}

// ---------------------------------------------------------------------------
// B. Vochtzwelling plaatmateriaal (EN 318)
// ---------------------------------------------------------------------------

/**
 * Bereken de vochtzwelling (toename + krimp) van plaatmateriaal.
 * `Δl [mm] = zwelling [mm/m per %RV] · ΔRV [%] · lengte [m]`.
 *
 * Negatieve ΔRV (bv. RV max < RV installatie) wordt geclampt naar 0 met een
 * waarschuwing — een "toename" bij een RV-dáling is geen valide invoer.
 */
export function calculateMoistureSwelling(
  input: MoistureSwellingInput,
): MoistureSwellingResult {
  const warnings: string[] = [];

  if (input.lengthM < 0) {
    warnings.push(`lengte (${input.lengthM} m) is negatief, als 0 behandeld`);
  }
  const lengthM = Math.max(0, input.lengthM);

  let deltaRvMaxPercent = input.rvMaxPercent - input.rvInstallPercent;
  if (deltaRvMaxPercent < 0) {
    warnings.push(
      `RV max (${input.rvMaxPercent}%) ligt onder RV bij installatie (${input.rvInstallPercent}%) — toename-ΔRV op 0 gezet`,
    );
    deltaRvMaxPercent = 0;
  }

  let deltaRvMinPercent = input.rvInstallPercent - input.rvMinPercent;
  if (deltaRvMinPercent < 0) {
    warnings.push(
      `RV min (${input.rvMinPercent}%) ligt boven RV bij installatie (${input.rvInstallPercent}%) — krimp-ΔRV op 0 gezet`,
    );
    deltaRvMinPercent = 0;
  }

  const swelling = input.swellingMmPerMPerPercent;
  const toenameMm = swelling * deltaRvMaxPercent * lengthM;
  const krimpMm = swelling * deltaRvMinPercent * lengthM;

  return {
    deltaRvMaxPercent,
    deltaRvMinPercent,
    toenameMm,
    krimpMm,
    warnings,
  };
}
