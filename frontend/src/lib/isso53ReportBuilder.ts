/**
 * BM Reports JSON-builder voor ISSO 53 (utiliteit ≤ 4 m).
 *
 * Parallelle versie naast `reportBuilder.ts` (ISSO 51) — produceert dezelfde
 * `ReportData`-structuur (cover + colofon + toc + sections + blocks +
 * backcover) maar met norm-specifieke titels, kolommen en gebouw-totalen.
 *
 * Backend (`src-tauri/src/reports/`) is norm-onafhankelijk: het accepteert
 * elke geldige BM Reports JSON. Per norm bouwt de frontend de JSON; het
 * Rust-PDF-renderpad blijft hetzelfde.
 */
import i18next from "../i18n/config";
import type { Project } from "../types";
import type {
  Isso53BuildingState,
  Isso53RoomState,
} from "../types/projectV2";
import type { Isso53ProjectResult } from "../types/isso53Result";
import {
  BOUNDARY_TYPE_LABELS,
  VERTICAL_POSITION_LABELS,
} from "./constants";

/** Default outdoor temperature when project.climate.theta_e is undefined. */
const DEFAULT_THETA_E = -10;

/** Format watts as integer (no decimals, no locale — PDF renderer formats). */
function fmtW(value: number): string {
  return String(Math.round(value));
}

/** Format number with 2 decimals. */
function fmt2(value: number): string {
  return value.toFixed(2);
}

/** ISO date string for today. */
function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

/** Translate ISSO 53 building shape enum to a human label. */
function shapeLabel(value: Isso53BuildingState["buildingShape"]): string {
  return i18next.t(`isso53.building.shapeOptions.${value}`, { defaultValue: value });
}

function positionLabel(value: Isso53BuildingState["buildingPosition"]): string {
  return i18next.t(`isso53.building.positionOptions.${value}`, { defaultValue: value });
}

function windPressureLabel(
  value: Isso53BuildingState["windPressureType"],
): string {
  return i18next.t(`isso53.building.windPressureOptions.${value}`, {
    defaultValue: value,
  });
}

function thermalMassLabel(value: Isso53BuildingState["thermalMass"]): string {
  return i18next.t(`isso53.building.thermalMassOptions.${value}`, {
    defaultValue: value,
  });
}

function ventilationSystemLabel(
  value: Isso53BuildingState["ventilationSystem"],
): string {
  return i18next.t(`isso53.building.ventilationSystemOptions.${value}`, {
    defaultValue: value,
  });
}

function gebruiksFunctieLabel(value: Isso53RoomState["gebruiksFunctie"]): string {
  return i18next.t(`isso53.room.gebruiksFunctieOptions.${value}`, {
    defaultValue: value,
  });
}

function ruimteTypeLabel(value: Isso53RoomState["ruimteType"]): string {
  return i18next.t(`isso53.room.ruimteTypeOptions.${value}`, {
    defaultValue: value,
  });
}

/** Lookup ISSO 53 sidecar voor een room — falsy fallback wanneer ontbrekend. */
function lookupIsso53Room(
  isso53Rooms: Record<string, Isso53RoomState>,
  roomId: string,
): Isso53RoomState | null {
  return isso53Rooms[roomId] ?? null;
}

/**
 * Bouw een complete BM Reports JSON voor een ISSO 53 project.
 *
 * Signature spiegelt het werkpakket fase 5:
 * - `project`        — V1 metadata-houder (info, ondertekening — hergebruikt)
 * - `result`         — output van `calculate_v2` voor norm `isso53`
 * - `isso53Building` — sidecar gebouw-state (buildingShape, position, …)
 * - `isso53Rooms`    — sidecar per-ruimte state (gebruiksFunctie + ruimteType)
 */
export function buildIsso53Report(
  project: Project,
  result: Isso53ProjectResult,
  isso53Building: Isso53BuildingState,
  isso53Rooms: Record<string, Isso53RoomState>,
): Record<string, unknown> {
  const today = todayIso();
  const projectName =
    project.info.name || i18next.t("isso53.report.untitled", { defaultValue: "Naamloos project" });

  return {
    template: "standaard_rapport",
    format: "A4",
    orientation: "portrait",
    project: projectName,
    project_number: project.info.project_number ?? "",
    client: project.info.client ?? "",
    author: project.info.engineer ?? "3BM Bouwkunde",
    date: project.info.date ?? today,
    version: "1.0",
    status: "CONCEPT",

    cover: {
      subtitle: i18next.t("isso53.report.coverSubtitle", {
        defaultValue: "ISSO 53 — Warmteverliesberekening utiliteit",
      }),
      ...(project.info.cover_image
        ? {
            image: {
              data: project.info.cover_image.data,
              media_type: project.info.cover_image.media_type,
              ...(project.info.cover_image.filename
                ? { filename: project.info.cover_image.filename }
                : {}),
            },
          }
        : {}),
    },

    ...(project.info.footer_image
      ? {
          footer: {
            image: {
              data: project.info.footer_image.data,
              media_type: project.info.footer_image.media_type,
              ...(project.info.footer_image.filename
                ? { filename: project.info.footer_image.filename }
                : {}),
            },
          },
        }
      : {}),

    ...(project.info.header_image
      ? {
          header: {
            image: {
              data: project.info.header_image.data,
              media_type: project.info.header_image.media_type,
              ...(project.info.header_image.filename
                ? { filename: project.info.header_image.filename }
                : {}),
            },
          },
        }
      : {}),

    ...(project.info.report_style
      ? {
          style: {
            ...(project.info.report_style.margin_top_mm != null
              ? { margin_top_mm: project.info.report_style.margin_top_mm }
              : {}),
            ...(project.info.report_style.margin_bottom_mm != null
              ? { margin_bottom_mm: project.info.report_style.margin_bottom_mm }
              : {}),
            ...(project.info.report_style.margin_horizontal_mm != null
              ? { margin_horizontal_mm: project.info.report_style.margin_horizontal_mm }
              : {}),
            ...(project.info.report_style.accent_color_hex
              ? { accent_color_hex: project.info.report_style.accent_color_hex }
              : {}),
          },
        }
      : {}),

    colofon: {
      enabled: true,
      opdrachtgever_naam: project.info.client ?? "",
      adviseur_bedrijf: "3BM Bouwkunde",
      adviseur_naam: project.info.engineer ?? "",
      normen: i18next.t("isso53.report.normenLine", {
        defaultValue:
          "ISSO 53 — Warmteverliesberekening voor utiliteitsgebouwen (vertrekhoogte ≤ 4 m)",
      }),
      datum: project.info.date ?? today,
      fase: "",
      status_colofon: "CONCEPT",
      kenmerk: project.info.project_number ?? "",
      revision_history: [
        {
          version: "1.0",
          date: today,
          author: project.info.engineer ?? "",
          description: i18next.t("isso53.report.revisionInitial", {
            defaultValue: "Eerste opzet",
          }),
        },
      ],
    },

    toc: {
      enabled: true,
      title: i18next.t("isso53.report.tocTitle", { defaultValue: "Inhoudsopgave" }),
      max_depth: 2,
    },

    sections: [
      buildUitgangspuntenSection(project, isso53Building),
      buildGebouwresultatenSection(result),
      ...buildVertrekkenChapter(project, result, isso53Rooms),
    ],

    backcover: { enabled: true },

    metadata: {
      engine: "isso53-core",
      generated_at: new Date().toISOString(),
      theta_e: project.climate.theta_e ?? DEFAULT_THETA_E,
    },
  };
}

/** Sectie "Uitgangspunten" voor ISSO 53.
 *
 * Toont buildingShape + position + windPressureType + thermalMass +
 * ventilationSystem (vervangt BuildingType uit ISSO 51).
 */
function buildUitgangspuntenSection(
  project: Project,
  isso53Building: Isso53BuildingState,
): Record<string, unknown> {
  const { climate } = project;
  return {
    title: i18next.t("isso53.report.sectionUitgangspunten", {
      defaultValue: "Uitgangspunten",
    }),
    level: 1,
    content: [
      {
        type: "table",
        title: i18next.t("isso53.report.tableBuilding", {
          defaultValue: "Gebouwgegevens (ISSO 53)",
        }),
        headers: [
          i18next.t("isso53.report.colParameter", { defaultValue: "Parameter" }),
          i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
        ],
        rows: [
          [
            i18next.t("isso53.building.shape", { defaultValue: "Gebouwvorm" }),
            shapeLabel(isso53Building.buildingShape),
          ],
          [
            i18next.t("isso53.building.position", {
              defaultValue: "Positie in complex",
            }),
            positionLabel(isso53Building.buildingPosition),
          ],
          [
            i18next.t("isso53.building.windPressureType", {
              defaultValue: "Winddrukverdelingstype",
            }),
            windPressureLabel(isso53Building.windPressureType),
          ],
          [
            i18next.t("isso53.building.thermalMass", {
              defaultValue: "Thermische massa",
            }),
            thermalMassLabel(isso53Building.thermalMass),
          ],
          [
            i18next.t("isso53.building.ventilationSystem", {
              defaultValue: "Ventilatiesysteem",
            }),
            ventilationSystemLabel(isso53Building.ventilationSystem),
          ],
          [
            i18next.t("isso53.building.constructionYear", {
              defaultValue: "Bouwjaar",
            }),
            isso53Building.constructionYear != null
              ? String(isso53Building.constructionYear)
              : "—",
          ],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "table",
        title: i18next.t("isso53.report.tableClimate", {
          defaultValue: "Klimaatgegevens",
        }),
        headers: [
          i18next.t("isso53.report.colParameter", { defaultValue: "Parameter" }),
          i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
        ],
        rows: [
          ["θ_e", `${climate.theta_e ?? DEFAULT_THETA_E} °C`],
          ["θ_b", `${climate.theta_b_non_residential ?? climate.theta_b_residential ?? 14} °C`],
        ],
      },
    ],
  };
}

/** Sectie "Gebouwresultaten" — ISSO 53-specifieke gebouw-totalen.
 *
 * Toont totalen + connectionCapacityIndividual + connectionCapacityCollective
 * + shellHeatLoss (uniek voor 53 t.o.v. 51).
 */
function buildGebouwresultatenSection(
  result: Isso53ProjectResult,
): Record<string, unknown> {
  const { summary } = result;
  return {
    title: i18next.t("isso53.report.sectionGebouwresultaten", {
      defaultValue: "Gebouwresultaten",
    }),
    level: 1,
    content: [
      {
        type: "table",
        title: i18next.t("isso53.report.tableTotalen", {
          defaultValue: "Totalen",
        }),
        headers: [
          i18next.t("isso53.report.colComponent", { defaultValue: "Component" }),
          i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
        ],
        rows: [
          ["Φ_T (transmissie)", `${fmtW(summary.totalTransmissionLoss)} W`],
          ["Φ_V (ventilatie)", `${fmtW(summary.totalVentilationLoss)} W`],
          ["Φ_I (infiltratie)", `${fmtW(summary.totalInfiltrationLoss)} W`],
          ["Φ_hu (opwarmtoeslag)", `${fmtW(summary.totalHeatingUp)} W`],
          ["Φ_system (systeemverliezen)", `${fmtW(summary.totalSystemLosses)} W`],
          ["Φ_gain (interne winsten)", `${fmtW(summary.totalInternalGains)} W`],
          [
            "<b>Φ_HL,build (gebouwtotaal)</b>",
            `<b>${fmtW(summary.totalBuildingHeatLoss)} W</b>`,
          ],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "table",
        title: i18next.t("isso53.report.tableCapacity", {
          defaultValue: "Aansluitvermogen & schilverliezen",
        }),
        headers: [
          i18next.t("isso53.report.colComponent", { defaultValue: "Component" }),
          i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
        ],
        rows: [
          [
            i18next.t("isso53.report.connectionIndividual", {
              defaultValue: "Aansluitvermogen individueel (formule 5.1)",
            }),
            `${fmtW(summary.connectionCapacityIndividual)} W`,
          ],
          [
            i18next.t("isso53.report.connectionCollective", {
              defaultValue: "Aansluitvermogen collectief (formule 5.9)",
            }),
            `${fmtW(summary.connectionCapacityCollective)} W`,
          ],
          [
            i18next.t("isso53.report.shellHeatLoss", {
              defaultValue: "Schil-methode (Φ_HL,shell)",
            }),
            `${fmtW(summary.shellHeatLoss)} W`,
          ],
          [
            i18next.t("isso53.report.infiltrationReductionZ", {
              defaultValue: "Infiltratie-reductiefactor z (tabel 5.1)",
            }),
            fmt2(summary.infiltrationReductionFactorZ),
          ],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "calculation",
        title: i18next.t("isso53.report.calcConnection", {
          defaultValue: "Maatgevend aansluitvermogen",
        }),
        result: fmtW(
          Math.max(
            summary.connectionCapacityIndividual,
            summary.connectionCapacityCollective,
          ),
        ),
        unit: "W",
        reference: "ISSO 53",
      },
    ],
  };
}

/** Sectie "Vertrekken" — parent + per-vertrek sub-secties. */
function buildVertrekkenChapter(
  project: Project,
  result: Isso53ProjectResult,
  isso53Rooms: Record<string, Isso53RoomState>,
): Record<string, unknown>[] {
  const parent: Record<string, unknown> = {
    title: i18next.t("isso53.report.sectionVertrekken", {
      defaultValue: "Vertrekken",
    }),
    level: 1,
    content: [
      {
        type: "table",
        title: i18next.t("isso53.report.tableRoomsSummary", {
          defaultValue: "Samenvatting per vertrek",
        }),
        headers: [
          i18next.t("isso53.report.colRoom", { defaultValue: "Vertrek" }),
          "θ_i [°C]",
          "Φ_T [W]",
          "Φ_V [W]",
          "Φ_I [W]",
          "Φ_hu [W]",
          "Φ_HL [W]",
        ],
        rows: result.rooms.map((r) => [
          r.roomName,
          fmt2(r.thetaI),
          fmtW(r.phiT),
          fmtW(r.phiV),
          fmtW(r.phiI),
          fmtW(r.phiHu),
          fmtW(r.totalHeatLoss),
        ]),
      },
    ],
  };
  return [parent, ...buildRoomDetailSections(project, result, isso53Rooms)];
}

/** Per-vertrek detail-sectie (level 2). */
function buildRoomDetailSections(
  project: Project,
  result: Isso53ProjectResult,
  isso53Rooms: Record<string, Isso53RoomState>,
): Record<string, unknown>[] {
  return result.rooms.map((room) => {
    const projectRoom = project.rooms.find((r) => r.id === room.roomId);
    const sidecar = lookupIsso53Room(isso53Rooms, room.roomId);

    const inputBlocks = projectRoom
      ? buildRoomInputBlocks(projectRoom, sidecar)
      : [];

    return {
      title: room.roomName,
      level: 2,
      content: [
        ...inputBlocks,
        {
          type: "table",
          title: i18next.t("isso53.report.tableTransmission", {
            defaultValue: "Transmissieverliezen",
          }),
          headers: [
            i18next.t("isso53.report.colComponent", { defaultValue: "Component" }),
            i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
          ],
          rows: [
            ["H_T,ie (schil)", `${fmt2(room.hTExterior)} W/K`],
            ["H_T,ia (intern)", `${fmt2(room.hTAdjacentRooms)} W/K`],
            ["H_T,iae (onverwarmd)", `${fmt2(room.hTUnheated)} W/K`],
            ["H_T,iaBE (buurgebouw)", `${fmt2(room.hTAdjacentBuildings)} W/K`],
            ["H_T,ig (grond)", `${fmt2(room.hTGround)} W/K`],
            ["Φ_T", `${fmtW(room.phiT)} W`],
          ],
        },
        { type: "spacer", height_mm: 2 },
        {
          type: "table",
          title: i18next.t("isso53.report.tableVentilationInfiltration", {
            defaultValue: "Ventilatie & infiltratie",
          }),
          headers: [
            i18next.t("isso53.report.colComponent", { defaultValue: "Component" }),
            i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
          ],
          rows: [
            ["H_v", `${fmt2(room.hV)} W/K`],
            ["Φ_V (ventilatie)", `${fmtW(room.phiV)} W`],
            ["H_i", `${fmt2(room.hI)} W/K`],
            ["Φ_I (infiltratie)", `${fmtW(room.phiI)} W`],
          ],
        },
        { type: "spacer", height_mm: 2 },
        {
          type: "table",
          title: i18next.t("isso53.report.tableHeatingUpSystem", {
            defaultValue: "Opwarmtoeslag & systeemverliezen",
          }),
          headers: [
            i18next.t("isso53.report.colComponent", { defaultValue: "Component" }),
            i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
          ],
          rows: [
            ["Φ_hu", `${fmtW(room.phiHu)} W`],
            ["Φ_system", `${fmtW(room.phiSystem)} W`],
            ["Φ_gain", `${fmtW(room.phiGain)} W`],
          ],
        },
        { type: "spacer", height_mm: 2 },
        {
          type: "table",
          title: i18next.t("isso53.report.tableTotal", { defaultValue: "Totaal" }),
          headers: [
            i18next.t("isso53.report.colComponent", { defaultValue: "Component" }),
            i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
          ],
          rows: [
            ["θ_i", `${fmt2(room.thetaI)} °C`],
            ["<b>Φ_HL,i</b>", `<b>${fmtW(room.totalHeatLoss)} W</b>`],
          ],
        },
      ],
    };
  });
}

/** Per-vertrek invoer-blokken (Algemeen + Constructie-elementen) voor ISSO 53.
 *
 * `Algemeen` toont gebruiksFunctie + ruimteType (i.p.v. RoomFunction in 51).
 */
function buildRoomInputBlocks(
  room: Project["rooms"][number],
  sidecar: Isso53RoomState | null,
): Record<string, unknown>[] {
  const algemeenRows: [string, string][] = [];
  if (sidecar) {
    algemeenRows.push([
      i18next.t("isso53.room.gebruiksFunctie", { defaultValue: "Gebruiksfunctie" }),
      gebruiksFunctieLabel(sidecar.gebruiksFunctie),
    ]);
    algemeenRows.push([
      i18next.t("isso53.room.ruimteType", { defaultValue: "Ruimtetype" }),
      ruimteTypeLabel(sidecar.ruimteType),
    ]);
  }
  algemeenRows.push([
    i18next.t("isso53.report.floorArea", { defaultValue: "Vloeroppervlak" }),
    `${fmt2(room.floor_area)} m²`,
  ]);
  if (room.height != null) {
    algemeenRows.push([
      i18next.t("isso53.report.height", { defaultValue: "Hoogte" }),
      `${fmt2(room.height)} m`,
    ]);
  }

  const blocks: Record<string, unknown>[] = [
    {
      type: "table",
      title: i18next.t("isso53.report.tableAlgemeen", {
        defaultValue: "Algemeen",
      }),
      headers: [
        i18next.t("isso53.report.colParameter", { defaultValue: "Parameter" }),
        i18next.t("isso53.report.colValue", { defaultValue: "Waarde" }),
      ],
      rows: algemeenRows,
    },
    { type: "spacer", height_mm: 2 },
  ];

  if (room.constructions && room.constructions.length > 0) {
    const elementRows: string[][] = room.constructions.map((el) => {
      const boundaryLabel =
        BOUNDARY_TYPE_LABELS[el.boundary_type] ?? el.boundary_type;
      const typeLabel = el.vertical_position
        ? VERTICAL_POSITION_LABELS[el.vertical_position] ?? el.vertical_position
        : el.material_type ?? "—";
      return [
        el.description || "—",
        typeLabel,
        `${fmt2(el.area)} m²`,
        `${fmt2(el.u_value)} W/m²K`,
        boundaryLabel,
      ];
    });
    blocks.push({
      type: "table",
      title: i18next.t("isso53.report.tableConstructions", {
        defaultValue: "Constructie-elementen (invoer)",
      }),
      headers: [
        i18next.t("isso53.report.colDescription", { defaultValue: "Omschrijving" }),
        i18next.t("isso53.report.colType", { defaultValue: "Type" }),
        i18next.t("isso53.report.colArea", { defaultValue: "Oppervlak" }),
        "U",
        i18next.t("isso53.report.colBoundary", { defaultValue: "Grens" }),
      ],
      rows: elementRows,
    });
    blocks.push({ type: "spacer", height_mm: 2 });
  }

  return blocks;
}
