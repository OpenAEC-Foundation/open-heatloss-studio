/**
 * Bouwt BM Reports JSON data op vanuit de U_w-calculator state.
 *
 * Output conform report.schema.json (OpenAEC Reports API).
 * Secties: raam-beschrijving, geometrie, materialen + Ψ_g-bron,
 * U_w-formule en uitkomst.
 *
 * Mirror van `rcReportBuilder.ts` maar voor de samengestelde
 * raam-U-waarde U_w volgens NEN-EN-ISO 10077-1.
 */

import { SPACER_LABELS_NL } from "./spacerTable";
import type { UwInput, UwResult } from "./uwCalculation";

/** ISO date string for today. */
function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

export interface UwReportInput {
  /** Naam van het kozijn / de raam-configuratie (mag leeg zijn). */
  name: string;
  input: UwInput;
  result: UwResult;
}

/** Build BM Reports JSON from U_w-calculator state. */
export async function buildUwReportData(
  arg: UwReportInput,
): Promise<Record<string, unknown>> {
  const today = todayIso();
  const title = arg.name || "Kozijn U_w-analyse";

  return {
    template: "standaard_rapport",
    format: "A4",
    orientation: "portrait",
    project: title,
    author: "3BM Bouwkunde",
    date: today,
    version: "1.0",
    status: "CONCEPT",

    cover: {
      subtitle: "Kozijn U_w-analyse conform NEN-EN-ISO 10077-1",
    },

    colofon: {
      enabled: true,
      adviseur_bedrijf: "3BM Bouwkunde",
      normen: "NEN-EN-ISO 10077-1 (samengestelde raam-U-waarde U_w)",
      datum: today,
      status_colofon: "CONCEPT",
      revision_history: [
        {
          version: "1.0",
          date: today,
          author: "",
          description: "Eerste opzet",
        },
      ],
    },

    toc: {
      enabled: true,
      title: "Inhoudsopgave",
      max_depth: 2,
    },

    sections: [
      buildDescriptionSection(arg),
      buildGeometrySection(arg),
      buildMaterialsSection(arg),
      buildResultSection(arg),
    ],

    backcover: { enabled: true },

    metadata: {
      engine: "uw-calculator",
      generated_at: new Date().toISOString(),
    },
  };
}

/** Sectie 1: Raam-beschrijving. */
function buildDescriptionSection(arg: UwReportInput): Record<string, unknown> {
  const { input } = arg;
  return {
    title: "Raam-beschrijving",
    level: 1,
    content: [
      {
        type: "table",
        title: "Algemeen",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Naam", arg.name || "Naamloos"],
          ["Breedte (buitenwerks)", `${input.width_mm} mm`],
          ["Hoogte (buitenwerks)", `${input.height_mm} mm`],
          ["Profielbreedte", `${input.frame_width_mm} mm`],
          [
            "Ruit-indeling",
            `${input.pane_columns} × ${input.pane_rows} (kolommen × rijen)`,
          ],
        ],
      },
    ],
  };
}

/** Sectie 2: Geometrie. */
function buildGeometrySection(arg: UwReportInput): Record<string, unknown> {
  const { geometry } = arg.result;
  return {
    title: "Geometrie",
    level: 1,
    content: [
      {
        type: "table",
        title: "Afgeleide oppervlakten en lengtes",
        headers: ["Grootheid", "Waarde"],
        rows: [
          ["A_w (totaal raamoppervlak)", `${geometry.a_w_m2.toFixed(4)} m²`],
          ["ΣA_g (glasoppervlak)", `${geometry.a_g_m2.toFixed(4)} m²`],
          ["ΣA_f (profieloppervlak)", `${geometry.a_f_m2.toFixed(4)} m²`],
          ["Σl_g (zichtbare glasrand-omtrek)", `${geometry.l_g_m.toFixed(3)} m`],
          [
            "Ruit-afmeting (b × h)",
            `${geometry.pane_width_mm.toFixed(0)} × ${geometry.pane_height_mm.toFixed(0)} mm`,
          ],
        ],
      },
      {
        type: "paragraph",
        text:
          "De glasoppervlakte volgt uit de buitenmaat verminderd met het " +
          "buitenkozijn en de tussenprofielen. Voor een uniform c×r " +
          "ruit-raster met identieke ruiten valt de raster-som samen tot " +
          "één glas-rechthoek.",
      },
    ],
  };
}

/** Sectie 3: Materialen + Ψ_g-bron. */
function buildMaterialsSection(arg: UwReportInput): Record<string, unknown> {
  const { input, result } = arg;

  const spacerLabel =
    !input.psi_g_is_manual && input.spacer
      ? SPACER_LABELS_NL[input.spacer]
      : null;
  const psiGSource = input.psi_g_is_manual
    ? "Handmatige invoer (override op de spacer-tabelwaarde)"
    : spacerLabel
      ? `Tabelwaarde — randafstandhouder: ${spacerLabel}`
      : "Handmatige invoer";

  return {
    title: "Materialen & beglazingsrand",
    level: 1,
    content: [
      {
        type: "table",
        title: "U-waarden en Ψ_g",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["U_g (glas)", `${input.u_g.toFixed(3)} W/(m²·K)`],
          ["U_f (profiel)", `${input.u_f.toFixed(3)} W/(m²·K)`],
          ["Ψ_g (beglazingsrand)", `${result.psi_g.toFixed(3)} W/(m·K)`],
          ["Ψ_g-bron", psiGSource],
        ],
      },
      {
        type: "paragraph",
        text:
          "<i>U_g en U_f zijn handmatige invoer van respectievelijk de " +
          "glasleverancier en de profielfabrikant. Ψ_g is de lineaire " +
          "warmtedoorgangscoëfficiënt van de beglazingsrand.</i>",
      },
    ],
  };
}

/** Sectie 4: U_w-formule en resultaat. */
function buildResultSection(arg: UwReportInput): Record<string, unknown> {
  const { input, result } = arg;
  const { geometry } = result;

  const numerator =
    geometry.a_g_m2 * input.u_g +
    geometry.a_f_m2 * input.u_f +
    geometry.l_g_m * result.psi_g;

  return {
    title: "Samengestelde raam-U-waarde (U_w)",
    level: 1,
    content: [
      {
        type: "paragraph",
        text:
          "<b>U_w = (ΣA_g·U_g + ΣA_f·U_f + " +
          "Σl_g·Ψ_g) / (ΣA_g + ΣA_f)</b> " +
          "— NEN-EN-ISO 10077-1.",
      },
      { type: "spacer", height_mm: 2 },
      {
        type: "table",
        title: "Tussenwaarden",
        headers: ["Term", "Waarde"],
        rows: [
          [
            "ΣA_g·U_g",
            `${(geometry.a_g_m2 * input.u_g).toFixed(4)} W/K`,
          ],
          [
            "ΣA_f·U_f",
            `${(geometry.a_f_m2 * input.u_f).toFixed(4)} W/K`,
          ],
          [
            "Σl_g·Ψ_g",
            `${(geometry.l_g_m * result.psi_g).toFixed(4)} W/K`,
          ],
          ["Teller (som)", `${numerator.toFixed(4)} W/K`],
          [
            "Noemer (ΣA_g + ΣA_f)",
            `${(geometry.a_g_m2 + geometry.a_f_m2).toFixed(4)} m²`,
          ],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "calculation",
        title: "Samengestelde raam-U-waarde",
        result: result.u_w.toFixed(3),
        unit: "W/(m²·K)",
        reference: "NEN-EN-ISO 10077-1",
      },
    ],
  };
}
