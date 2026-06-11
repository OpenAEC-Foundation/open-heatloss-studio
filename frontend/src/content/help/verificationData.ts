/**
 * Data-ontsluiting voor de Help-sectie "Verificatie".
 *
 * Gekozen route: **directe JSON-imports** uit `tests/verification/`
 * (repo-root, buiten de Vite-root). Het bron-pad blijft daarmee single
 * source of truth — dezelfde bestanden die de Rust-integratietests
 * (`crates/isso51-core/tests/integration_test.rs`) en de ISSO 53
 * golden-tests (`crates/isso53-core/tests/vabi_*_golden.rs`) lezen,
 * zonder kopieer-/sync-stap die kan driften.
 *
 * Werking per omgeving:
 * - **Production build & Tauri:** Rollup bundelt de JSON compile-time mee
 *   in de JS-chunk; runtime is géén bestandstoegang buiten de bundle nodig.
 * - **Dev-server:** bestanden buiten de Vite-root worden via `/@fs/`
 *   geserveerd; daarvoor is `server.fs.allow` in `vite.config.ts`
 *   uitgebreid met `../tests/verification`.
 * - **Vitest (node-env):** resolvet dezelfde imports via de Vite-pipeline.
 * - **Docker (`Dockerfile`, node-builder stage):** de builder kopieert
 *   standaard alleen `frontend/` + `schemas/`; daarom staat er expliciet
 *   `COPY tests/verification/ /build/tests/verification/` vóór
 *   `RUN npm run build`. Die COPY NIET verwijderen — zonder faalt de
 *   container-build met TS2307 op onderstaande imports.
 */
import type { Project } from "../../types";
import type {
  Isso53ExpectedMetric,
  VerificationExpected,
} from "../../lib/verificationCompare";
import type { Iso53Inputs, ProjectV2 } from "../../types/projectV2";
import { SCHEMA_VERSION_V2 } from "../../types/projectV2";

import vrijstaandeWoningInput from "../../../../tests/verification/isso51_vabi3.8.1.14_vrijstaande-woning/input.json";
import vrijstaandeWoningExpected from "../../../../tests/verification/isso51_vabi3.8.1.14_vrijstaande-woning/expected.json";
import drEngineeringInput from "../../../../tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/input.json";
import drEngineeringExpected from "../../../../tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/expected.json";
import houtfabriek3FloorsInput from "../../../../tests/verification/isso53_vabi3.11.2.23_houtfabriek-3floors/input.json";
import houtfabriek3FloorsExpected from "../../../../tests/verification/isso53_vabi3.11.2.23_houtfabriek-3floors/expected.json";
import bedrijfsruimte4Input from "../../../../tests/verification/isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4/input.json";
import bedrijfsruimte4Expected from "../../../../tests/verification/isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4/expected.json";
import kantoorwestInput from "../../../../tests/verification/isso53_vabi3.12.0.127_dr-engineering-kantoorwest/input.json";
import kantoorwestExpected from "../../../../tests/verification/isso53_vabi3.12.0.127_dr-engineering-kantoorwest/expected.json";

// ---------------------------------------------------------------------------
// ISSO 51
// ---------------------------------------------------------------------------

/**
 * Presentatie-modus van een verificatieproject.
 * - `reference`: volwaardige referentie — pass/fail-verdicts tegen tolerantie.
 * - `informative`: ter illustratie — Δ-kolommen blijven zichtbaar, maar GEEN
 *   ✓/✗-verdicts en geen rood/groen-badge (bv. normversie-mismatch tussen
 *   Vabi-rapport en engine).
 */
export type VerificationMode = "reference" | "informative";

/** Eén ISSO 51-verificatieproject zoals de Help-sectie het presenteert. */
export interface VerificationProjectDef {
  /** Stabiele sleutel (= directory-naam onder `tests/verification/`). */
  id: string;
  title: string;
  /** Norm + versie van het Vabi-referentierapport. */
  norm: string;
  /** Vabi Elements versie (+ rekenkern indien bekend). */
  software: string;
  /** Ventilatie-omschrijving uit de project-README. */
  ventilation: string;
  /** Presentatie-modus: referentie (verdicts) of informatief (geen verdicts). */
  mode: VerificationMode;
  /** Verplichte uitleg wanneer `mode === "informative"`. */
  disclaimer?: string;
  /**
   * Bekende, gedocumenteerde afwijkingen tussen engine en Vabi
   * (uit README / expected.json-notes). Leeg = geen.
   */
  knownDeviations: ReadonlyArray<string>;
  /** Bron-pad relatief aan de repo-root (documentatie in de UI). */
  sourcePath: string;
  /** Projectinvoer in heatloss-studio formaat — gaat 1-op-1 naar de engine. */
  input: Project;
  /** Vabi-rapport truth (per vertrek + gebouwtotaal). */
  expected: VerificationExpected;
}

/**
 * De input.json-bestanden zijn exact het `Project`-formaat dat de
 * Rust-engine deserialiseert; de JSON-module-inferentie van TypeScript is
 * echter te letterlijk (string-literals i.p.v. enums) — vandaar de cast
 * via `unknown`.
 */
function asProject(raw: unknown): Project {
  return raw as Project;
}

function asExpected(raw: unknown): VerificationExpected {
  return raw as VerificationExpected;
}

/** ISSO 51-verificatieprojecten, in chronologische volgorde van de bron. */
export const VERIFICATION_PROJECTS: ReadonlyArray<VerificationProjectDef> = [
  {
    id: "isso51_vabi3.8.1.14_vrijstaande-woning",
    title: "Vrijstaande woning",
    norm: "ISSO 51:2017 (incl. 53/57)",
    software: "Vabi Elements 3.8.1.14, rekenkern Warmteverlies 2.30",
    ventilation: "Systeem C, continu bedrijf (geen nachtverlaging)",
    mode: "informative",
    disclaimer:
      "Dit Vabi-rapport is gerekend volgens ISSO 51:2017; de engine rekent " +
      "ISSO 51:2023 incl. erratum. Afwijkingen (per vertrek +4–8%, " +
      "gebouwtotaal −20% door een andere sommatiemethode) zijn " +
      "normversie-verschillen, geen rekenfouten. Dit project dient ter " +
      "illustratie, niet als referentie.",
    knownDeviations: [
      "Normversie-mismatch: referentierapport ISSO 51:2017, engine ISSO 51:2023 (incl. erratum) — per vertrek +4–8%, gebouwtotaal −20% (lineaire vs. kwadratische sommatie).",
      "De bijbehorende Rust-fixture staat daarom op #[ignore] in crates/isso51-core/tests/integration_test.rs.",
    ],
    sourcePath: "tests/verification/isso51_vabi3.8.1.14_vrijstaande-woning/",
    input: asProject(vrijstaandeWoningInput),
    expected: asExpected(vrijstaandeWoningExpected),
  },
  {
    id: "isso51_vabi3.12.0.127_dr-engineering-woningbouw",
    title: "DR Engineering — Woningbouw",
    norm: "ISSO 51:2024 (incl. erratum 2023, kwadratische sommatie)",
    software: "Vabi Elements 3.12.0.127",
    ventilation: "Systeem D met WTW (η = 0,8)",
    mode: "reference",
    knownDeviations: [
      "Infiltratie: Vabi gebruikt qi;spec per m² A_g × correctie 1,10 (systeem D) — wijkt af van de engine-methode.",
      "Temperatuurgelaagdheid via de tussenvloer is niet gemodelleerd in de engine.",
    ],
    sourcePath: "tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/",
    input: asProject(drEngineeringInput),
    expected: asExpected(drEngineeringExpected),
  },
];

// ---------------------------------------------------------------------------
// ISSO 53
// ---------------------------------------------------------------------------

/**
 * Eén ISSO 53-verificatieproject. Anders dan bij ISSO 51 zijn de
 * expected.json-shapes heterogeen (per project gegroeid rond de
 * bijbehorende golden-test); daarom normaliseren we hier naar een platte
 * lijst {@link Isso53ExpectedMetric}-rijen waarvan de toleranties 1-op-1
 * de `close()`-asserts van de Rust golden-tests spiegelen.
 */
export interface Isso53VerificationProjectDef {
  id: string;
  title: string;
  norm: string;
  software: string;
  ventilation: string;
  /** Verwarmingssysteem-omschrijving (relevant voor Δθ_v / f_v). */
  heating: string;
  /** Ontwerpbuitentemperatuur θ_e in °C (uit input.json). */
  thetaE: number;
  /** Samenvatting van de vertrekken-invoer, incl. stub-ruimten. */
  roomsSummary: string;
  knownDeviations: ReadonlyArray<string>;
  sourcePath: string;
  /** Pad naar de Rust golden-test die dit project in CI bewaakt. */
  goldenTest: string;
  /** Legacy ISSO 53-projectblob (camelCase) — gaat inline onder `calcs.isso53`. */
  input: Iso53Inputs;
  /** Genormaliseerde Vabi-truth: per (vertrek, grootheid) + CI-tolerantie. */
  metrics: ReadonlyArray<Isso53ExpectedMetric>;
}

/**
 * Bouw de minimale `ProjectV2`-payload voor een live ISSO 53-verificatie.
 *
 * De Rust-route (`calculate_v2` → `view::to_isso53_project`) leest UITSLUITEND
 * `calcs.isso53` (serde-flatten van de legacy projectblob) — `shared` en
 * `geometry` zijn alleen nodig om de envelope te deserialiseren. Doordat
 * `isso51` op null staat en `isso53` gevuld is, routeert
 * `Calcs::active_norm()` naar `ActiveNorm::Isso53`.
 */
export function buildIsso53VerifyPayload(def: Isso53VerificationProjectDef): ProjectV2 {
  return {
    schema_version: SCHEMA_VERSION_V2,
    shared: {
      name: def.title,
      building_type: { kind: "utiliteit", subtype: "other" },
    },
    geometry: { spaces: [] },
    calcs: { isso51: null, isso53: def.input, tojuli: null },
  };
}

/** Vertreknaam uit een legacy ISSO 53-inputblob (fallback: het id zelf). */
function isso53RoomName(input: Iso53Inputs, roomId: string): string {
  const rooms = (input as { rooms?: Array<{ id?: unknown; name?: unknown }> }).rooms ?? [];
  const room = rooms.find((r) => r.id === roomId);
  return typeof room?.name === "string" ? room.name : roomId;
}

function asIso53Inputs(raw: unknown): Iso53Inputs {
  return raw as Iso53Inputs;
}

// --- Houtfabriek 3 floors -------------------------------------------------

/** Shape van `isso53_vabi3.11.2.23_houtfabriek-3floors/expected.json`. */
const HOUTFABRIEK_3FLOORS_EXPECTED = houtfabriek3FloorsExpected as {
  phi_t_tolerance_pct: number;
  phi_i_tolerance_pct: number;
  total_tolerance_pct: number;
  rooms: Array<{
    roomId: string;
    phiT: number;
    phiI: number;
    phiHu: number;
    totalHeatLoss: number;
  }>;
};

/**
 * Spiegel van `vabi_houtfabriek_3floors_golden.rs`: per room Φ_T
 * (phi_t_tolerance_pct), Φ_I (phi_i_tolerance_pct), Φ_hu (5% — hardcoded in
 * de golden-test) en totaal (total_tolerance_pct).
 */
const HOUTFABRIEK_3FLOORS_METRICS: ReadonlyArray<Isso53ExpectedMetric> =
  HOUTFABRIEK_3FLOORS_EXPECTED.rooms.flatMap((r): Isso53ExpectedMetric[] => {
    const roomLabel = isso53RoomName(asIso53Inputs(houtfabriek3FloorsInput), r.roomId);
    return [
      {
        roomId: r.roomId,
        roomLabel,
        metric: "phiT",
        expectedW: r.phiT,
        tolerancePct: HOUTFABRIEK_3FLOORS_EXPECTED.phi_t_tolerance_pct,
      },
      {
        roomId: r.roomId,
        roomLabel,
        metric: "phiI",
        expectedW: r.phiI,
        tolerancePct: HOUTFABRIEK_3FLOORS_EXPECTED.phi_i_tolerance_pct,
      },
      {
        roomId: r.roomId,
        roomLabel,
        metric: "phiHu",
        expectedW: r.phiHu,
        tolerancePct: 5.0, // vabi_3floors_phi_hu_matches: close(..., 5.0)
      },
      {
        roomId: r.roomId,
        roomLabel,
        metric: "total",
        expectedW: r.totalHeatLoss,
        tolerancePct: HOUTFABRIEK_3FLOORS_EXPECTED.total_tolerance_pct,
      },
    ];
  });

// --- Houtfabriek Bedrijfsruimte 4 ------------------------------------------

/** Shape van `isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4/expected.json`. */
const BEDRIJFSRUIMTE4_EXPECTED = bedrijfsruimte4Expected as {
  tolerance_pct: number;
  room: {
    roomId: string;
    phiT: number;
    phiV_plus_phiI: number;
    phiHu: number;
    totalHeatLoss: number;
  };
};

/**
 * Spiegel van `vabi_golden.rs` (bedrijfsruimte4). Vabi's Φ_V is hier 0
 * (WTW + luchtverwarming f_v=0), dus de gerapporteerde `phiV_plus_phiI`
 * is 1-op-1 Vabi's infiltratie — afzonderlijk geborgd zoals in de test.
 * Het totaal heeft géén eigen golden-test-assert tegen Vabi; daarvoor
 * geldt de projectbrede `tolerance_pct` (15%) uit expected.json.
 */
const BEDRIJFSRUIMTE4_METRICS: ReadonlyArray<Isso53ExpectedMetric> = (() => {
  const r = BEDRIJFSRUIMTE4_EXPECTED.room;
  const roomLabel = isso53RoomName(asIso53Inputs(bedrijfsruimte4Input), r.roomId);
  return [
    // vabi_bedrijfsruimte4_phi_v_zero: Φ_V exact 0 (absolute 1 W-grens).
    { roomId: r.roomId, roomLabel, metric: "phiV", expectedW: 0, tolerancePct: 0 },
    // vabi_bedrijfsruimte4_phi_t_matches: close(..., 5.0).
    { roomId: r.roomId, roomLabel, metric: "phiT", expectedW: r.phiT, tolerancePct: 5.0 },
    // vabi_bedrijfsruimte4_phi_i_matches: close(phiI, 3080, 3.0).
    {
      roomId: r.roomId,
      roomLabel,
      metric: "phiI",
      expectedW: r.phiV_plus_phiI,
      tolerancePct: 3.0,
    },
    // vabi_bedrijfsruimte4_phi_hu_matches: close(..., 5.0).
    { roomId: r.roomId, roomLabel, metric: "phiHu", expectedW: r.phiHu, tolerancePct: 5.0 },
    // Geen golden-test-assert tegen Vabi-totaal → projectbrede tolerance_pct.
    {
      roomId: r.roomId,
      roomLabel,
      metric: "total",
      expectedW: r.totalHeatLoss,
      tolerancePct: BEDRIJFSRUIMTE4_EXPECTED.tolerance_pct,
    },
  ];
})();

// --- DR Engineering Kantoor West --------------------------------------------

/** Shape van `isso53_vabi3.12.0.127_dr-engineering-kantoorwest/expected.json`. */
const KANTOORWEST_EXPECTED = kantoorwestExpected as {
  rooms: Array<{
    roomId: string;
    roomName: string;
    phiT: number;
    phiV: number;
    phiI: number;
    totalHeatLoss: number;
  }>;
};

/**
 * Spiegel van `vabi_dr_golden.rs` (Kantoor West 0.03). De golden-test
 * hanteert AANGESCHERPTE toleranties t.o.v. expected.json (V2-verstrakking:
 * Φ_T 4% i.p.v. 10%, Φ_I 2,5% i.p.v. 5%) — wij volgen de test, niet het
 * bestand. CI heeft géén Vabi-totaal-assert; deze pagina hanteert 5%
 * (gedocumenteerde Vabi-afwijking +3,1%, zelfde grens als de CI-snapshot).
 */
const KANTOORWEST_METRICS: ReadonlyArray<Isso53ExpectedMetric> = (() => {
  const r = KANTOORWEST_EXPECTED.rooms[0];
  if (!r) {
    throw new Error("kantoorwest expected.json: rooms[0] ontbreekt");
  }
  const roomLabel = r.roomName;
  return [
    // vabi_dr_kantoorwest_phi_v_zero: Φ_V = 0 (absolute 1 W-grens).
    { roomId: r.roomId, roomLabel, metric: "phiV", expectedW: r.phiV, tolerancePct: 0 },
    // vabi_dr_kantoorwest_phi_t_matches: close(phiT, 3059, 4.0).
    { roomId: r.roomId, roomLabel, metric: "phiT", expectedW: r.phiT, tolerancePct: 4.0 },
    // vabi_dr_kantoorwest_phi_i_matches: close(phiI, 681, 2.5).
    { roomId: r.roomId, roomLabel, metric: "phiI", expectedW: r.phiI, tolerancePct: 2.5 },
    // Pagina-eigen totaal-grens (5%): geen CI-assert tegen Vabi-totaal.
    {
      roomId: r.roomId,
      roomLabel,
      metric: "total",
      expectedW: r.totalHeatLoss,
      tolerancePct: 5.0,
    },
  ];
})();

/** ISSO 53-verificatieprojecten — zelfde bron als de Rust golden-tests. */
export const ISSO53_VERIFICATION_PROJECTS: ReadonlyArray<Isso53VerificationProjectDef> = [
  {
    id: "isso53_vabi3.11.2.23_houtfabriek-3floors",
    title: "TR02 Houtfabriek — 3 verdiepingen",
    norm: "ISSO 53",
    software: "Vabi Elements 3.11.2.23, rekenkern Warmteverlies 2.43.1",
    ventilation: "Systeem D met WTW (η = 0,85)",
    heating: "Vloerverwarming (Δθ_v = −1 K, norm-conform — Vabi past dit niet toe op infiltratie)",
    thetaE: -9.0,
    roomsSummary: "3 identieke bedrijfsruimten op 3 verdiepingen + 5 stub-ruimten (adjacent-coupling)",
    knownDeviations: [
      "Dak 3.10a: Vabi gebruikt onverklaard f=1,138 waar de norm-strikte engine f=1,000 hanteert — ±60 W per dak-element.",
      "Φ_I: norm-conforme Δθ_v-correctie (vloerverwarming, −1 K) die Vabi niet toepast op infiltratie — ≈ −3,5% per vertrek.",
      "Inter-floor gedeelde vloeren/plafonds via virtuele stub-temperaturen gemodelleerd (Vabi's onverwarmd-tussenvloer-conventie).",
    ],
    sourcePath: "tests/verification/isso53_vabi3.11.2.23_houtfabriek-3floors/",
    goldenTest: "crates/isso53-core/tests/vabi_houtfabriek_3floors_golden.rs",
    input: asIso53Inputs(houtfabriek3FloorsInput),
    metrics: HOUTFABRIEK_3FLOORS_METRICS,
  },
  {
    id: "isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4",
    title: "TR02 Houtfabriek — Bedrijfsruimte 4",
    norm: "ISSO 53",
    software: "Vabi Elements 3.11.2.23, rekenkern Warmteverlies 2.43.1",
    ventilation: "Systeem D met WTW (η = 0,85) + vorstbeveiliging",
    heating: "Luchtverwarming (toevoertemperatuur 21 °C → f_v = 0)",
    thetaE: -9.0,
    roomsSummary: "1 bedrijfsruimte (16p, industriefunctie als kantoor gemapt) + 5 stub-ruimten",
    knownDeviations: [
      "Vabi's Φ_V = 0 door WTW + luchtverwarming; de Vabi-kolom 'Φ_V+Φ_I' (3080 W) is dus volledig infiltratie.",
      "Vabi's verwarmd-plafond-conventie (corr = 0 bij 18 °C-buurruimten) vs. norm-strikt f_ia,k = 2/29 — ≈ +95 W op Φ_T.",
      "ΔU_TB = 0,05 (nieuw gebouw) via customDeltaUTb-overrides; engine-default is 0,10.",
    ],
    sourcePath: "tests/verification/isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4/",
    goldenTest: "crates/isso53-core/tests/vabi_golden.rs",
    input: asIso53Inputs(bedrijfsruimte4Input),
    metrics: BEDRIJFSRUIMTE4_METRICS,
  },
  {
    id: "isso53_vabi3.12.0.127_dr-engineering-kantoorwest",
    title: "DR Engineering — Kantoor West 0.03",
    norm: "ISSO 53",
    software: "Vabi Elements 3.12.0.127",
    ventilation: "Systeem D met WTW (η = 0,85)",
    heating: "Luchtverwarming (toevoertemperatuur 21,5 °C → f_v = 0)",
    thetaE: -6.0,
    roomsSummary: "1 kantoorruimte + 2 stub-ruimten (gang, verdieping boven)",
    knownDeviations: [
      "Φ_I via InfiltrationMethod::UnknownVabiCompat (NEN 8088-1 + NTA 8800 power-law) — bewust Vabi-compat, niet norm-puur (norm-strikt pad geeft 177 W).",
      "Φ_T +3,5% restgap (Vabi 3059 W): gedocumenteerd sinds de Optie C dubbeltelling-fix.",
      "Gebouwtotaal heeft geen CI-tolerantie tegen Vabi; deze pagina hanteert 5% (gedocumenteerde afwijking +3,1%).",
    ],
    sourcePath: "tests/verification/isso53_vabi3.12.0.127_dr-engineering-kantoorwest/",
    goldenTest: "crates/isso53-core/tests/vabi_dr_golden.rs",
    input: asIso53Inputs(kantoorwestInput),
    metrics: KANTOORWEST_METRICS,
  },
];
