/**
 * Hoeken-omrekentool — graden ↔ hellingspercentage ↔ verhouding (1:n).
 *
 * Pure wiskundige omrekening (tan-relatie), GEEN norm nodig — anders dan
 * `hwaCalculation.ts` / `hellingbaanCalculation.ts` zijn hier geen
 * `SourcedValue`-normconstanten in het spel. Frontend-only, state-loos,
 * 1-op-1 conform het "hoeken"-tabblad van het rekenblad van de eigenaar.
 *
 * **Conventie:** een verhouding `1:n` betekent 1 (verticaal) op n
 * (horizontaal) — dezelfde conventie als hellingbaan-vuistregels (1:12,
 * 1:20) en dakhelling-notatie. Daarmee geldt `procent = 100 / n` en
 * `n = 100 / procent`.
 *
 * **Rekenblad-ankers (ter documentatie/verificatie in de tests):**
 * - 1:12 = 8,333% ≈ 4,7636°
 * - 8% ≈ 4,5739°
 * - 45° = 100% = 1:1
 *
 * **Randgevallen:**
 * - 0% / 0° → vlak, verhouding 1:∞ (`Infinity`, geen fout).
 * - 90° (verticaal) is wiskundig ongedefinieerd (`tan(90°)` → ±∞) en wordt
 *   afgewezen; ook negatieve hoeken/percentages/verhoudingen zijn geen
 *   geldige fysieke invoer voor dit tabblad en worden afgewezen.
 * - Een verhouding van 1:0 (n=0) komt overeen met 90° (verticaal) en levert
 *   dus `Infinity` op als percentage — net als 0% → 1:∞, is dit geen fout,
 *   het is de wiskundige limiet.
 */

/** Bovengrens voor een geldige hoek in graden — 90° (verticaal) is ongedefinieerd. */
export const MAX_GRADEN = 90;

function assertFiniteNietNegatief(value: number, label: string): void {
  if (!Number.isFinite(value) && value !== Infinity) {
    throw new RangeError(`${label} (${value}) moet een geldig getal zijn`);
  }
  if (value < 0) {
    throw new RangeError(`${label} (${value}) mag niet negatief zijn`);
  }
}

/**
 * Hoek in graden → hellingspercentage (`tan(graden) × 100`).
 *
 * Gooit bij een negatieve hoek of een hoek ≥ 90° (verticaal, `tan`
 * ongedefinieerd).
 */
export function gradenNaarProcent(graden: number): number {
  assertFiniteNietNegatief(graden, "hoek");
  if (graden >= MAX_GRADEN) {
    throw new RangeError(`hoek (${graden}°) moet kleiner zijn dan 90° (verticaal, ongedefinieerd)`);
  }
  return Math.tan((graden * Math.PI) / 180) * 100;
}

/**
 * Hellingspercentage → hoek in graden (`atan(procent / 100)`).
 *
 * `Infinity` (verticaal, zie {@link verhoudingNaarProcent}) levert exact
 * 90° op — de wiskundige limiet. Gooit bij een negatief percentage.
 */
export function procentNaarGraden(procent: number): number {
  assertFiniteNietNegatief(procent, "percentage");
  return (Math.atan(procent / 100) * 180) / Math.PI;
}

/**
 * Hellingspercentage → verhouding `1:n` (`n = 100 / procent`).
 *
 * 0% → `Infinity` (vlak, verhouding 1:∞). Gooit bij een negatief
 * percentage.
 */
export function procentNaarVerhouding(procent: number): number {
  assertFiniteNietNegatief(procent, "percentage");
  return 100 / procent;
}

/**
 * Verhouding `1:n` → hellingspercentage (`procent = 100 / n`).
 *
 * `n = 0` (verhouding 1:0, verticaal) → `Infinity`. Gooit bij een
 * negatieve `n`.
 */
export function verhoudingNaarProcent(n: number): number {
  assertFiniteNietNegatief(n, "verhouding");
  return 100 / n;
}

/** Hoek in graden → verhouding `1:n` (samengestelde afgeleide). */
export function gradenNaarVerhouding(graden: number): number {
  return procentNaarVerhouding(gradenNaarProcent(graden));
}

/** Verhouding `1:n` → hoek in graden (samengestelde afgeleide). */
export function verhoudingNaarGraden(n: number): number {
  return procentNaarGraden(verhoudingNaarProcent(n));
}

/** Eén rij van drie onderling consistente waarden (graden, procent, verhouding). */
export interface HoekWaarden {
  graden: number;
  procent: number;
  /** `n` in de verhouding `1:n`. */
  verhoudingN: number;
}

/** Bouw een consistente {@link HoekWaarden}-set op vanuit een hoek in graden. */
export function hoekWaardenVanGraden(graden: number): HoekWaarden {
  const procent = gradenNaarProcent(graden);
  return { graden, procent, verhoudingN: procentNaarVerhouding(procent) };
}

/** Bouw een consistente {@link HoekWaarden}-set op vanuit een hellingspercentage. */
export function hoekWaardenVanProcent(procent: number): HoekWaarden {
  const graden = procentNaarGraden(procent);
  return { graden, procent, verhoudingN: procentNaarVerhouding(procent) };
}

/** Bouw een consistente {@link HoekWaarden}-set op vanuit een verhouding `1:n`. */
export function hoekWaardenVanVerhouding(n: number): HoekWaarden {
  const procent = verhoudingNaarProcent(n);
  return { graden: procentNaarGraden(procent), procent, verhoudingN: n };
}
