/**
 * Data-ontsluiting voor de Help-sectie "Verificatie".
 *
 * Gekozen route: **directe JSON-imports** uit `tests/verification/`
 * (repo-root, buiten de Vite-root). Het bron-pad blijft daarmee single
 * source of truth — dezelfde bestanden die de Rust-integratietests
 * (`crates/isso51-core/tests/integration_test.rs`) lezen, zonder
 * kopieer-/sync-stap die kan driften.
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
import type { VerificationExpected } from "../../lib/verificationCompare";

import vrijstaandeWoningInput from "../../../../tests/verification/isso51_vabi3.8.1.14_vrijstaande-woning/input.json";
import vrijstaandeWoningExpected from "../../../../tests/verification/isso51_vabi3.8.1.14_vrijstaande-woning/expected.json";
import drEngineeringInput from "../../../../tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/input.json";
import drEngineeringExpected from "../../../../tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/expected.json";

/** Eén verificatieproject zoals de Help-sectie het presenteert. */
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
    knownDeviations: [
      "Geen — werkt 1-op-1 met het Vabi-rapport binnen de 2%-vertrektolerantie.",
      "Let op: het referentierapport is ISSO 51:2017; de engine rekent ISSO 51:2023. De bijbehorende Rust-fixture staat daarom op #[ignore].",
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
    knownDeviations: [
      "Infiltratie: Vabi gebruikt qi;spec per m² A_g × correctie 1,10 (systeem D) — wijkt af van de engine-methode.",
      "Temperatuurgelaagdheid via de tussenvloer is niet gemodelleerd in de engine.",
    ],
    sourcePath: "tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/",
    input: asProject(drEngineeringInput),
    expected: asExpected(drEngineeringExpected),
  },
];
