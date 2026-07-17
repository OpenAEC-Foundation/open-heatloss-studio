/**
 * Hemelwaterafvoer (HWA) — rekenkern voor dakafvoer-dimensionering.
 *
 * Frontend-only rekenmodel (zelfde patroon als `doorGap.ts`): puur-TS,
 * state-loos, geen Rust/API. Elke normconstante draagt een
 * {@link SourcedValue} met bronlabel — zie `types/hwa.ts` voor de
 * betekenis van `"rekenblad-eigenaar"` vs. `"fabrikanten-doc"`.
 *
 * **Formule-structuur (bevestigd tegen fabrikanten-doc):**
 * `Qh = α · i · β · F` — α = platdakfactor ({@link FLAT_ROOF_FACTORS}),
 * i = regenintensiteit ({@link DEFAULT_RAIN_INTENSITY_LP_MIN_M2}),
 * β = hellingreductie- of gevelfactor ({@link PITCH_REDUCTION_TABLE},
 * {@link FACADE_CONTRIBUTION_FACTOR}), F = oppervlak.
 *
 * **Platdak-afwerking-interpretatie (herzien na bronverificatie):** het
 * bronrekenblad bevatte een ongereduceerde (1,0) reductiefactor voor een
 * plat dak in het uitgewerkte voorbeeldblok — dat bleek NIET norm-conform.
 * Volgens de fabrikanten-documentatie is een plat dak (pitchDeg 0) ALTIJD
 * gereduceerd: grind 0,6 / overig plat 0,75. `flatRoofFinish: null` (geen
 * afwerking opgegeven) geeft daarom géén 1,0 meer, maar valt terug op 0,75
 * (overig plat dak) — met een expliciete warning, zie `calculateSurface`.
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
// Normconstanten
// ---------------------------------------------------------------------------

/** Default regenintensiteit in l/(min·m²) — bevestigd tegen fabrikanten-doc. */
export const DEFAULT_RAIN_INTENSITY_LP_MIN_M2: SourcedValue<number> = {
  value: 1.8,
  source: "fabrikanten-doc",
  reference:
    "bevestigd tegen NTR 3216-gebaseerde fabrikanten-documentatie (NedZink/Dyka)",
};

/**
 * Hellingreductie-banden (β): trapsgewijs, GEEN interpolatie — de eerste
 * band waarvan `pitchDeg` binnen de bovengrens valt, bepaalt de factor.
 * `≤ 45° → 1,0`, `> 45–60° → 0,8`, `> 60–85° → 0,6`, `> 85–90° → 0,3`.
 */
export const PITCH_REDUCTION_TABLE: SourcedValue<
  ReadonlyArray<{ maxDeg: number; factor: number }>
> = {
  value: [
    { maxDeg: 45, factor: 1.0 },
    { maxDeg: 60, factor: 0.8 },
    { maxDeg: 85, factor: 0.6 },
    { maxDeg: 90, factor: 0.3 },
  ],
  source: "fabrikanten-doc",
  reference: "NTR 3216-gebaseerde fabrikanten-documentatie (NedZink/Dyka)",
};

/**
 * Platdakfactoren (α, `pitchDeg === 0`): een plat dak is norm-conform
 * ALTIJD gereduceerd. `null` (geen afwerking opgegeven) geeft daarom géén
 * 1,0 meer, maar valt terug op `plat` (0,75) — met een expliciete warning
 * in `calculateSurface`, zie de module-doc-comment.
 */
export const FLAT_ROOF_FACTORS: SourcedValue<
  Record<Exclude<HwaFlatRoofFinish, null>, number>
> = {
  value: { grind: 0.6, plat: 0.75 },
  source: "fabrikanten-doc",
  reference: "NTR 3216-gebaseerde fabrikanten-documentatie (NedZink/Dyka)",
};

/**
 * Factor voor bijdragend gevel-/opstandoppervlak: muren tellen mee met
 * dezelfde reductie als een (nagenoeg) verticaal vlak — β = 0,3, gelijk aan
 * de 85–90°-band van {@link PITCH_REDUCTION_TABLE}.
 */
export const FACADE_CONTRIBUTION_FACTOR: SourcedValue<number> = {
  value: 0.3,
  source: "fabrikanten-doc",
  reference: "NTR 3216-gebaseerde fabrikanten-documentatie (NedZink/Dyka)",
};

/**
 * Capaciteitstabel diameter (mm) → afvoercapaciteit (l/min). Bewust geen
 * 125/160 mm — zo staat het in het bronrekenblad.
 *
 * **Bronnen spreken elkaar tegen:** fabrikanten-doc (NedZink, zink) geeft
 * voor Ø80 een capaciteit van 150 l/min, andere bronnen ~99 l/min — dit
 * rekenblad (117 l/min) zit daar tussenin. De tabel blijft daarom
 * `"rekenblad-eigenaar"` (conservatiever dan de zink-fabrikantwaarden, dus
 * aan de veilige kant) totdat een eenduidige normbron is vastgesteld.
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
  reference:
    "te verifiëren tegen NEN 3215/NTR 3216 — rekenblad is conservatiever dan zink-fabrikantwaarden (bv. NedZink Ø80 → 150 l/min vs. hier 117 l/min)",
};

/** Vaste waarschuwing bij UV-systemen en platte daken (traditioneel): noodafvoer valt buiten deze toets. */
export const EMERGENCY_OVERFLOW_WARNING =
  "noodafvoer verplicht dimensioneren (buiten scope van deze toets)";

// ---------------------------------------------------------------------------
// Clamping helpers — edge-cases netjes afvangen met warning
// ---------------------------------------------------------------------------

function clampPitchDeg(pitchDeg: number): { value: number; warning: string | null } {
  if (!Number.isFinite(pitchDeg)) {
    return {
      value: 0,
      warning: `hellingshoek (${pitchDeg}) is ongeldig, gecorrigeerd naar 0°`,
    };
  }
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
 * Hellingreductiefactor (β): trapsgewijs via {@link PITCH_REDUCTION_TABLE},
 * GEEN interpolatie — de eerste band waarvan `pitchDeg` binnen de
 * bovengrens (`maxDeg`) valt, bepaalt de factor. `pitchDeg` moet al
 * geclampt zijn naar 0–90.
 */
export function pitchReductionFactor(pitchDeg: number): number {
  const table = PITCH_REDUCTION_TABLE.value;
  for (const band of table) {
    if (pitchDeg <= band.maxDeg) return band.factor;
  }
  return table[table.length - 1]!.factor;
}

/**
 * Reductiefactor voor een dakvlak: platdakfactor (α) bij `pitchDeg === 0`
 * — `null` (geen afwerking opgegeven) valt terug op `plat` (0,75), zie de
 * module-doc-comment — anders de hellingreductie (β) via
 * {@link pitchReductionFactor}.
 */
export function surfaceReductionFactor(
  pitchDeg: number,
  flatRoofFinish: HwaFlatRoofFinish,
): number {
  if (pitchDeg === 0) {
    return FLAT_ROOF_FACTORS.value[flatRoofFinish ?? "plat"];
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
    if (surface.areaM2 === undefined) {
      return {
        value: 0,
        warning: "oppervlak ontbreekt bij invoermodus 'vrij', oppervlak op 0 gezet",
      };
    }
    return { value: Math.max(0, surface.areaM2), warning: null };
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

  if (pitchDeg === 0 && surface.flatRoofFinish === null) {
    warnings.push("platdak-afwerking niet opgegeven, 0,75 (overig plat dak) aangenomen");
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

  // Geclampte pitch, niet de ruwe invoer — anders mist bv. pitchDeg: -5 (dat
  // in calculateSurface naar 0 geclampt wordt en als plat dak doorrekent)
  // de noodafvoer-warning.
  const hasFlatRoof = input.surfaces.some((s) => clampPitchDeg(s.pitchDeg).value === 0);

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
