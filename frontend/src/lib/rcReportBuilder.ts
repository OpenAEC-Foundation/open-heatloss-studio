/**
 * Bouwt BM Reports JSON data op vanuit Rc-calculator state.
 *
 * Output conform report.schema.json (OpenAEC Reports API).
 * Secties: constructie-beschrijving, lagen-opbouw, thermische samenvatting,
 * Glaser-analyse, jaarlijkse vochtbalans.
 */

import type { MaterialType, VerticalPosition } from "../types";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
} from "./constructionCatalogue";
import type { GlaserResult } from "./glaserCalculation";
import { generateGlaserSvg, svgToBase64 } from "./glaserSvg";
import { generateMoistureYearSvg } from "./moistureYearSvg";
import type { LayerInput, RcResult } from "./rcCalculation";
import { RC_MIN_BOUWBESLUIT } from "./rcCalculation";
import type { YearlyMoistureResult } from "./yearlyMoistureCalculation";

const POSITION_LABELS: Record<VerticalPosition, string> = {
  wall: "Wand",
  floor: "Vloer",
  ceiling: "Plafond/dak",
};

const MATERIAL_TYPE_LABELS: Record<MaterialType, string> = {
  masonry: "Steenachtig",
  non_masonry: "Niet-steenachtig",
};

/** ISO date string for today. */
function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

export interface RcReportInput {
  name: string;
  category: CatalogueCategory;
  materialType: MaterialType;
  position: VerticalPosition;
  layers: LayerInput[];
  rcResult: RcResult;
  glaserResult: GlaserResult;
  moistureResult: YearlyMoistureResult | null;
  thetaI: number;
  thetaE: number;
  rhI: number;
  rhE: number;
}

/** Build BM Reports JSON from Rc-calculator state. */
export function buildRcReportData(input: RcReportInput): Record<string, unknown> {
  const today = todayIso();
  const title = input.name || "Constructie-analyse";

  return {
    template: "blank",
    brand: "3bm_cooperatie",
    format: "A4",
    orientation: "portrait",
    project: title,
    author: "3BM Bouwkunde",
    date: today,
    version: "1.0",
    status: "CONCEPT",

    cover: {
      subtitle: "Constructie-analyse conform NEN-EN-ISO 6946 / 13788",
    },

    colofon: {
      enabled: true,
      adviseur_bedrijf: "3BM Bouwkunde",
      normen:
        "NEN-EN-ISO 6946 (Rc/U-waarde), NEN-EN-ISO 13788 (Glaser/vochtbalans)",
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
      buildDescriptionSection(input),
      buildLayersSection(input),
      buildThermalSection(input),
      buildGlaserSection(input),
      ...(input.moistureResult ? [buildMoistureSection(input)] : []),
    ],

    backcover: { enabled: true },

    metadata: {
      engine: "rc-calculator",
      generated_at: new Date().toISOString(),
    },
  };
}

/** Sectie 1: Constructie-beschrijving. */
function buildDescriptionSection(input: RcReportInput): Record<string, unknown> {
  return {
    title: "Constructie-beschrijving",
    level: 1,
    content: [
      {
        type: "table",
        title: "Algemeen",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Naam", input.name || "Naamloos"],
          ["Categorie", CATALOGUE_CATEGORY_LABELS[input.category] ?? input.category],
          ["Positie", POSITION_LABELS[input.position]],
          ["Materiaaltype", MATERIAL_TYPE_LABELS[input.materialType]],
          ["Totale dikte", `${input.glaserResult.totalThickness} mm`],
          ["Aantal lagen", String(input.layers.filter((l) => l.materialId).length)],
        ],
      },
    ],
  };
}

/** Sectie 2: Lagen-opbouw. */
function buildLayersSection(input: RcReportInput): Record<string, unknown> {
  const { rcResult } = input;

  const layerRows: string[][] = [];

  // Rsi
  layerRows.push(["Rsi (binnenoppervlakteweerstand)", "\u2014", "\u2014", rcResult.rSi.toFixed(2)]);

  // Lagen
  for (const layerResult of rcResult.layers) {
    const lambdaStr = layerResult.lambda !== null ? layerResult.lambda.toFixed(3) : "\u2014";
    const thicknessStr = layerResult.thickness > 0 ? String(layerResult.thickness) : "\u2014";
    layerRows.push([
      layerResult.name,
      thicknessStr,
      lambdaStr,
      layerResult.r.toFixed(3),
    ]);
  }

  // Rse
  layerRows.push(["Rse (buitenoppervlakteweerstand)", "\u2014", "\u2014", rcResult.rSe.toFixed(2)]);

  return {
    title: "Lagen-opbouw",
    level: 1,
    content: [
      {
        type: "table",
        title: "Constructielagen",
        headers: ["Materiaal", "Dikte [mm]", "\u03BB [W/(m·K)]", "R [m²K/W]"],
        rows: layerRows,
      },
    ],
  };
}

/** Sectie 3: Thermische samenvatting. */
function buildThermalSection(input: RcReportInput): Record<string, unknown> {
  const { rcResult, position } = input;
  const rcMin = RC_MIN_BOUWBESLUIT[position];
  const voldoet = rcResult.rc >= rcMin;

  return {
    title: "Thermische prestatie",
    level: 1,
    content: [
      {
        type: "table",
        title: "Samenvatting",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Rc (constructieweerstand)", `${rcResult.rc.toFixed(2)} m²K/W`],
          ["R_totaal (incl. Rsi + Rse)", `${rcResult.rTotal.toFixed(2)} m²K/W`],
          ["U-waarde", `${rcResult.uValue.toFixed(3)} W/(m²·K)`],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "table",
        title: "Toetsing Bouwbesluit 2024",
        headers: ["Eis", "Waarde", "Resultaat"],
        rows: [
          [
            `Rc ≥ ${rcMin} m²K/W`,
            `${rcResult.rc.toFixed(2)} m²K/W`,
            voldoet ? "\u2714 Voldoet" : "\u2718 Voldoet niet",
          ],
        ],
      },
    ],
  };
}

/** Sectie 4: Glaser-analyse. */
function buildGlaserSection(input: RcReportInput): Record<string, unknown> {
  const { glaserResult, thetaI, thetaE, rhI, rhE } = input;

  const condensText = glaserResult.hasCondensation
    ? "<b>Condensatierisico aanwezig.</b> De werkelijke dampdruk overschrijdt de verzadigingsdruk op \u00E9\u00E9n of meer grensvlakken."
    : "Geen condensatierisico. De werkelijke dampdruk blijft overal onder de verzadigingsdruk.";

  // Interface points table
  const pointRows = glaserResult.interfacePoints.map((p) => [
    String(Math.round(p.x)),
    p.temperature.toFixed(1),
    String(Math.round(p.pSat)),
    String(Math.round(p.pActual)),
    p.pActual >= p.pSat ? "Condensatie" : "OK",
  ]);

  // Generate Glaser SVG diagram
  const glaserSvg = generateGlaserSvg(glaserResult, thetaI, thetaE);
  const glaserImageBlock = glaserSvg
    ? {
        type: "image",
        src: {
          data: svgToBase64(glaserSvg),
          media_type: "image/svg+xml",
        },
        caption: "Dampspanningsverloop door de constructie",
        width_mm: 170,
        alignment: "center",
      }
    : null;

  return {
    title: "Dampspanningsanalyse (Glaser)",
    level: 1,
    content: [
      {
        type: "table",
        title: "Klimaatcondities",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Temperatuur binnen (θ_i)", `${thetaI} °C`],
          ["Relatieve vochtigheid binnen (RV_i)", `${rhI}%`],
          ["Temperatuur buiten (θ_e)", `${thetaE} °C`],
          ["Relatieve vochtigheid buiten (RV_e)", `${rhE}%`],
          ["Dampdruk binnen (p_i)", `${Math.round(glaserResult.pI)} Pa`],
          ["Dampdruk buiten (p_e)", `${Math.round(glaserResult.pE)} Pa`],
        ],
      },
      { type: "spacer", height_mm: 4 },
      ...(glaserImageBlock ? [glaserImageBlock, { type: "spacer", height_mm: 4 }] : []),
      {
        type: "table",
        title: "Dampdrukverloop per grensvlak",
        headers: ["Positie [mm]", "θ [°C]", "p_sat [Pa]", "p_act [Pa]", "Status"],
        rows: pointRows,
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "paragraph",
        text: condensText,
      },
    ],
  };
}

/** Sectie 5: Jaarlijkse vochtbalans. */
function buildMoistureSection(input: RcReportInput): Record<string, unknown> {
  const mr = input.moistureResult;
  if (!mr) return { title: "Vochtbalans", level: 1, content: [] };

  const STATUS_LABELS: Record<string, string> = {
    dry: "Droog",
    condensation: "Condensatie",
    drying: "Droging",
  };

  const monthRows = mr.months.map((m) => [
    m.month,
    m.thetaE.toFixed(1),
    String(m.rhE),
    m.thetaC.toFixed(1),
    String(Math.round(m.pSatC)),
    m.gc.toFixed(1),
    m.ma.toFixed(1),
    STATUS_LABELS[m.status] ?? m.status,
  ]);

  const riskText = mr.hasRisk
    ? "<b>Schimmelrisico:</b> Vocht is langer dan 90 dagen aanwezig."
    : mr.maxMa > 0.1
      ? "Tijdelijk vochtophoping, maar constructie droogt uit binnen \u00E9\u00E9n jaar."
      : "Geen vochtophoping. Constructie blijft droog over het jaar.";

  // Generate moisture year chart SVG
  const moistureSvg = generateMoistureYearSvg(mr);
  const moistureImageBlock = moistureSvg
    ? {
        type: "image",
        src: {
          data: svgToBase64(moistureSvg),
          media_type: "image/svg+xml",
        },
        caption: "Vochthuishouding over het jaar",
        width_mm: 170,
        alignment: "center",
      }
    : null;

  return {
    title: "Jaarlijkse vochtbalans (NEN-EN-ISO 13788)",
    level: 1,
    content: [
      {
        type: "paragraph",
        text: `Condensatievlak: tussen <b>${mr.planeInnerLayer}</b> en <b>${mr.planeOuterLayer}</b> (positie ${mr.planePosition} mm vanaf binnen).`,
      },
      { type: "spacer", height_mm: 2 },
      {
        type: "table",
        title: "Maandelijkse vochtbalans",
        headers: [
          "Maand",
          "θ_e [°C]",
          "RV [%]",
          "θ_c [°C]",
          "p_sat [Pa]",
          "g_c [g/m²]",
          "M_a [g/m²]",
          "Status",
        ],
        rows: monthRows,
      },
      { type: "spacer", height_mm: 4 },
      ...(moistureImageBlock ? [moistureImageBlock, { type: "spacer", height_mm: 4 }] : []),
      {
        type: "table",
        title: "Samenvatting",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Max. vochtophoping", `${mr.maxMa.toFixed(1)} g/m²`],
          ["Nat dagen", `${mr.wetDays} dagen`],
          ["Droogt volledig uit", mr.driesOut ? "Ja" : "Nee"],
          ["Schimmelrisico", mr.hasRisk ? "Ja" : "Nee"],
        ],
      },
      { type: "spacer", height_mm: 2 },
      {
        type: "paragraph",
        text: riskText,
      },
    ],
  };
}
