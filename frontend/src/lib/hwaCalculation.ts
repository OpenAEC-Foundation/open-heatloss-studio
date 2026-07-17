/**
 * Hemelwaterafvoer (HWA) — rekenkern voor dakafvoer-dimensionering.
 *
 * Frontend-only rekenmodel (zelfde patroon als `doorGap.ts`): puur-TS,
 * state-loos, geen Rust/API. Alle normconstanten zijn {@link SourcedValue}
 * met `source: "rekenblad-eigenaar"` — dit model is nog NIET geverifieerd
 * tegen NEN 3215 / NTR 3216, zie `reference` op elke constante.
 *
 * **Platdak-afwerking-interpretatie (belangrijke keuze, zie test):** het
 * bronrekenblad bevat twee verschillende reductiewaarden voor een plat dak:
 * het uitgewerkte voorbeeldblok rekent met reductiefactor 1,0 (géén
 * reductie), terwijl de losse reductietabel ernaast 0,6 (grind) / 0,75
 * (plat, zonder grind) geeft. Dit model implementeert de reductietabel als
 * de algemene rekenregel ({@link FLAT_ROOF_FACTORS}) en behandelt
 * `flatRoofFinish: null` (geen afwerking opgegeven) als de ongereduceerde
 * basiswaarde (1,0) uit het voorbeeldblok — zo blijven beide bronnen van het
 * rekenblad expliciet en zonder onderlinge tegenspraak in de API bruikbaar.
 */

import type {
  HwaFlatRoofFinish,
  HwaInput,
  HwaResult,
  HwaRoofSurface,
  HwaSurfaceResult,
  SourcedValue,
} from "../types/hwa";

// ---------------------------------------------------------------------------
// Normconstanten — alle nog "rekenblad-eigenaar", te verifiëren
// ---------------------------------------------------------------------------

/** Default regenintensiteit in l/(min·m²). */
export const DEFAULT_RAIN_INTENSITY_LP_MIN_M2: SourcedValue<number> = {
  value: 1.8,
  source: "rekenblad-eigenaar",
  reference: "te verifiëren tegen NEN 3215/NTR 3216",
};

/**
 * Hellingreductie-tabelpunten (graden → factor), lineair te interpoleren
 * tussen opeenvolgende punten (45–60, 60–85, 85–90). Voor `pitchDeg ≤ 45°`
 * geldt een vlakke factor 1,0 (geen reductie). De interpolatiemethode zelf
 * (lineair tussen tabelpunten) is een aanname van dit model en dus zelf ook
 * te verifiëren, niet alleen de tabelwaarden.
 */
export const PITCH_REDUCTION_TABLE: SourcedValue<
  ReadonlyArray<{ deg: number; factor: number }>
> = {
  value: [
    { deg: 45, factor: 1.0 },
    { deg: 60, factor: 0.8 },
    { deg: 85, factor: 0.6 },
    { deg: 90, factor: 0.3 },
  ],
  source: "rekenblad-eigenaar",
  reference:
    "te verifiëren tegen NEN 3215/NTR 3216 — inclusief de lineaire interpolatiemethode tussen tabelpunten",
};

/**
 * Platdakfactoren uit de reductietabel (`pitchDeg === 0`). `null`
 * (geen afwerking opgegeven) is bewust ongereduceerd (1,0) — zie de
 * module-doc-comment voor de interpretatiekeuze t.o.v. het voorbeeldblok.
 */
export const FLAT_ROOF_FACTORS: SourcedValue<
  Record<Exclude<HwaFlatRoofFinish, null>, number>
> = {
  value: { grind: 0.6, plat: 0.75 },
  source: "rekenblad-eigenaar",
  reference: "te verifiëren tegen NEN 3215/NTR 3216",
};

/** Factor 1,0 voor bijdragend gevel-/opstandoppervlak. */
export const FACADE_CONTRIBUTION_FACTOR: SourcedValue<number> = {
  value: 1.0,
  source: "rekenblad-eigenaar",
  reference: "bijdragefactor gevel te verifiëren tegen NEN 3215/NTR 3216",
};

/**
 * Capaciteitstabel diameter (mm) → afvoercapaciteit (l/min). Bewust geen
 * 125/160 mm — zo staat het in het bronrekenblad.
 */
export const DOWNPIPE_CAPACITY_TABLE: SourcedValue<
  ReadonlyArray<{ diameterMm: number; capacityLpMin: number }>
> = {
  value: [
    { diameterMm: 75, capacityLpMin: 75 },
    { diameterMm: 80, capacityLpMin: 117 },
    { diameterMm: 90, capacityLpMin: 163 },
    { diameterMm: 100, capacityLpMin: 210 },
    { diameterMm: 120, capacityLpMin: 338 },
    { diameterMm: 200, capacityLpMin: 870 },
    { diameterMm: 315, capacityLpMin: 2150 },
    { diameterMm: 400, capacityLpMin: 3420 },
  ],
  source: "rekenblad-eigenaar",
  reference: "te verifiëren tegen NEN 3215/NTR 3216",
};

/** Vaste waarschuwing bij UV-systemen en platte daken (traditioneel): noodafvoer valt buiten deze toets. */
export const EMERGENCY_OVERFLOW_WARNING =
  "noodafvoer verplicht dimensioneren (buiten scope van deze toets)";

// ---------------------------------------------------------------------------
// Clamping helpers — edge-cases netjes afvangen met warning
// ---------------------------------------------------------------------------

function clampPitchDeg(pitchDeg: number): { value: number; warning: string | null } {
  if (pitchDeg < 0 || pitchDeg > 90) {
    const clamped = Math.min(90, Math.max(0, pitchDeg));
    return {
      value: clamped,
      warning: `hellingshoek ${pitchDeg}° ligt buiten het bereik 0–90°, gecorrigeerd naar ${clamped}°`,
    };
  }
  return { value: pitchDeg, warning: null };
}

function clampDownpipeCount(count: number): { value: number; warning: string | null } {
  if (!Number.isFinite(count) || count < 1) {
    return {
      value: 1,
      warning: `aantal afvoeren (${count}) is ongeldig, gecorrigeerd naar 1`,
    };
  }
  return { value: Math.floor(count), warning: null };
}

// ---------------------------------------------------------------------------
// Reductiefactor
// ---------------------------------------------------------------------------

/**
 * Hellingreductiefactor via lineaire interpolatie tussen de tabelpunten van
 * {@link PITCH_REDUCTION_TABLE}. `pitchDeg` moet al geclampt zijn naar 0–90.
 */
export function pitchReductionFactor(pitchDeg: number): number {
  const table = PITCH_REDUCTION_TABLE.value;
  const first = table[0]!;
  if (pitchDeg <= first.deg) return first.factor;

  for (let i = 0; i < table.length - 1; i++) {
    const lower = table[i]!;
    const upper = table[i + 1]!;
    if (pitchDeg <= upper.deg) {
      const fraction = (pitchDeg - lower.deg) / (upper.deg - lower.deg);
      return lower.factor + fraction * (upper.factor - lower.factor);
    }
  }
  return table[table.length - 1]!.factor;
}

/**
 * Reductiefactor voor een dakvlak: platdakfactor bij `pitchDeg === 0`
 * (incl. de `null`-interpretatie, zie module-doc-comment), anders de
 * hellingreductie via {@link pitchReductionFactor}.
 */
export function surfaceReductionFactor(
  pitchDeg: number,
  flatRoofFinish: HwaFlatRoofFinish,
): number {
  if (pitchDeg === 0) {
    if (flatRoofFinish === null) return 1.0;
    return FLAT_ROOF_FACTORS.value[flatRoofFinish];
  }
  return pitchReductionFactor(pitchDeg);
}

// ---------------------------------------------------------------------------
// Diameteradvies
// ---------------------------------------------------------------------------

/** Kleinste diameter (mm) uit {@link DOWNPIPE_CAPACITY_TABLE} met capaciteit ≥ `flowPerPipeLpMin`, of `null` als zelfs Ø400 niet volstaat. */
export function adviesDiameterMm(flowPerPipeLpMin: number): number | null {
  const table = DOWNPIPE_CAPACITY_TABLE.value;
  for (const row of table) {
    if (row.capacityLpMin >= flowPerPipeLpMin) return row.diameterMm;
  }
  return null;
}

// ---------------------------------------------------------------------------
// Basisoppervlak
// ---------------------------------------------------------------------------

function baseAreaM2(surface: HwaRoofSurface): { value: number; warning: string | null } {
  if (surface.areaInputMode === "vrij") {
    const area = surface.areaM2 ?? 0;
    return { value: Math.max(0, area), warning: null };
  }
  const length = surface.lengthM ?? 0;
  const width = surface.widthM ?? 0;
  if (surface.lengthM === undefined || surface.widthM === undefined) {
    return {
      value: Math.max(0, length * width),
      warning: "lengte en/of breedte ontbreekt bij invoermodus 'lxb', oppervlak op 0 gezet waar nodig",
    };
  }
  return { value: Math.max(0, length * width), warning: null };
}

// ---------------------------------------------------------------------------
// Per-vlak berekening
// ---------------------------------------------------------------------------

/** Bereken het HWA-resultaat voor één dakvlak. */
export function calculateSurface(
  surface: HwaRoofSurface,
  rainIntensityLpMinM2: number,
): HwaSurfaceResult {
  const warnings: string[] = [];

  const { value: pitchDeg, warning: pitchWarning } = clampPitchDeg(surface.pitchDeg);
  if (pitchWarning) warnings.push(pitchWarning);

  const { value: downpipeCount, warning: downpipeWarning } = clampDownpipeCount(
    surface.downpipeCount,
  );
  if (downpipeWarning) warnings.push(downpipeWarning);

  const { value: baseArea, warning: areaWarning } = baseAreaM2(surface);
  if (areaWarning) warnings.push(areaWarning);
  if (baseArea === 0 && surface.facadeContributionM2 <= 0) {
    warnings.push("dakvlak heeft een effectief oppervlak van 0 m²");
  }

  const reductionFactor = surfaceReductionFactor(pitchDeg, surface.flatRoofFinish);
  const facadeContribution =
    Math.max(0, surface.facadeContributionM2) * FACADE_CONTRIBUTION_FACTOR.value;
  const effectiveAreaM2 = baseArea * reductionFactor + facadeContribution;

  const flowLpMin = effectiveAreaM2 * rainIntensityLpMinM2;
  const flowPerPipeLpMin = flowLpMin / downpipeCount;

  const adviesdiameterMm = adviesDiameterMm(flowPerPipeLpMin);
  if (adviesdiameterMm === null) {
    warnings.push(
      `zelfs Ø${DOWNPIPE_CAPACITY_TABLE.value[DOWNPIPE_CAPACITY_TABLE.value.length - 1]!.diameterMm} volstaat niet voor ${flowPerPipeLpMin.toFixed(1)} l/min per afvoer`,
    );
  }

  const altPipeCount = downpipeCount + 1;
  const altFlowPerPipeLpMin = flowLpMin / altPipeCount;
  const altDiameterMm = adviesDiameterMm(altFlowPerPipeLpMin);
  const alternatief =
    altDiameterMm !== null && (adviesdiameterMm === null || altDiameterMm < adviesdiameterMm)
      ? {
          downpipeCount: altPipeCount,
          diameterMm: altDiameterMm,
          flowPerPipeLpMin: altFlowPerPipeLpMin,
        }
      : null;

  return {
    surfaceId: surface.id,
    effectiveAreaM2,
    flowLpMin,
    flowPerPipeLpMin,
    adviesdiameterMm,
    alternatief,
    warnings,
  };
}

// ---------------------------------------------------------------------------
// Totaalberekening
// ---------------------------------------------------------------------------

/** Bereken het volledige HWA-resultaat voor alle dakvlakken + (bij UV) de systeemtoets. */
export function calculateHwa(input: HwaInput): HwaResult {
  const warnings: string[] = [];

  if (input.surfaces.length === 0) {
    warnings.push("geen dakvlakken ingevoerd");
  }

  const rainIntensity =
    input.rainIntensityLpMinM2 > 0
      ? input.rainIntensityLpMinM2
      : DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value;
  if (input.rainIntensityLpMinM2 <= 0) {
    warnings.push(
      `regenintensiteit (${input.rainIntensityLpMinM2}) is ongeldig, default ${DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value} l/(min·m²) gebruikt`,
    );
  }

  const surfaceResults = input.surfaces.map((surface) =>
    calculateSurface(surface, rainIntensity),
  );

  const totaalEffectiveAreaM2 = surfaceResults.reduce((sum, r) => sum + r.effectiveAreaM2, 0);
  const totaalFlowLpMin = surfaceResults.reduce((sum, r) => sum + r.flowLpMin, 0);

  const hasFlatRoof = input.surfaces.some((s) => s.pitchDeg === 0);

  let uvToets: HwaResult["uvToets"] = null;
  if (input.systemMode === "uv") {
    warnings.push(EMERGENCY_OVERFLOW_WARNING);
    if (input.uvSystemCapacityLpMin === undefined) {
      warnings.push("UV-systeemcapaciteit ontbreekt, toets niet uitgevoerd");
    } else {
      uvToets = {
        pass: totaalFlowLpMin <= input.uvSystemCapacityLpMin,
        totaalFlowLpMin,
        capaciteitLpMin: input.uvSystemCapacityLpMin,
      };
    }
  } else if (hasFlatRoof) {
    warnings.push(EMERGENCY_OVERFLOW_WARNING);
  }

  return {
    surfaceResults,
    totaalEffectiveAreaM2,
    totaalFlowLpMin,
    uvToets,
    warnings,
  };
}
