/**
 * Bouwt BM Reports JSON data op vanuit project + berekeningsresultaat.
 *
 * Output conform report.schema.json (OpenAEC Reports API).
 */
import type { Project, ProjectResult, Room, RoomResult } from "../types";
import type { ProjectConstruction } from "../components/modeller/types";
import {
  BOUNDARY_TYPE_LABELS,
  BUILDING_TYPE_LABELS,
  DEFAULT_THETA_WATER,
  HEATING_SYSTEM_LABELS,
  ROOM_FUNCTION_LABELS,
  ROOM_FUNCTION_TEMPERATURES,
  SECURITY_CLASS_LABELS,
  VENTILATION_SYSTEM_LABELS,
  VERTICAL_POSITION_LABELS,
} from "./constants";
import { calculateRc, type LayerInput } from "./rcCalculation";
import { getMaterialById } from "./materialsDatabase";
import {
  buildConstructionLossSvg,
  buildStackedBarSvg,
  buildSummaryDonutSvg,
  buildTemperatureGradientSvg,
  rasterizeSvgToPng,
} from "./reportCharts";

/** Default buitentemperatuur (°C) voor fallback in rapport charts. */
const DEFAULT_THETA_E = -10;

/** Format number as string without locale (PDF renderer handelt opmaak). */
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

/** Welke top-level secties in de PDF terechtkomen.
 *
 * Default = alles aan; gebruiker schakelt secties uit via de Rapport-page
 * toggles. Cover staat ALTIJD aan (de PDF moet een voorblad hebben), dus
 * die zit hier niet bij.
 */
export interface ReportSectionToggles {
  colofon: boolean;
  toc: boolean;
  uitgangspunten: boolean;
  constructies: boolean;
  vertrekkenOverzicht: boolean;
  perVertrek: boolean;
  diagrammen: boolean;
  gebouwresultaten: boolean;
  tojuli: boolean;
  backcover: boolean;
}

const ALL_SECTIONS_ON: ReportSectionToggles = {
  colofon: true,
  toc: true,
  uitgangspunten: true,
  constructies: true,
  vertrekkenOverzicht: true,
  perVertrek: true,
  diagrammen: true,
  gebouwresultaten: true,
  tojuli: false,
  backcover: true,
};

/** Build BM Reports JSON from project input + calculation result.
 *
 * `projectConstructions` is optioneel — wanneer aangeleverd voegt de
 * builder een "Constructie-opbouw & Rc-waarden" sectie toe (per
 * opbouw met layers één sub-sectie met Laagopbouw-tabel en
 * R/U-resultaten). Wordt door RapportTab doorgegeven vanuit
 * `useModellerStore.projectConstructions`.
 *
 * `toggles` bepaalt welke top-level secties opgenomen worden in de PDF.
 * Default: alles aan.
 */
export async function buildReportData(
  project: Project,
  result: ProjectResult,
  projectConstructions: ProjectConstruction[] = [],
  toggles: ReportSectionToggles = ALL_SECTIONS_ON,
): Promise<Record<string, unknown>> {
  const today = todayIso();
  const projectName = project.info.name || "Naamloos project";
  const thetaWater = project.climate.theta_water ?? DEFAULT_THETA_WATER;
  const diagrammenSection = await buildDiagrammenSection(project, result);
  const thetaIDefault = 20;
  const thetaEDefault = project.climate.theta_e ?? DEFAULT_THETA_E;
  const constructiesSection = await buildConstructiesSection(
    projectConstructions,
    thetaIDefault,
    thetaEDefault,
  );
  const tojuliSection = toggles.tojuli
    ? await buildTojuliSection(project)
    : null;

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
      subtitle: "Warmteverliesberekening conform ISSO 51:2023",
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
      enabled: toggles.colofon,
      opdrachtgever_naam: project.info.client ?? "",
      adviseur_bedrijf: "3BM Bouwkunde",
      adviseur_naam: project.info.engineer ?? "",
      normen: "ISSO 51:2023 — Warmteverliesberekening voor woningen en woongebouwen",
      datum: project.info.date ?? today,
      fase: "",
      status_colofon: "CONCEPT",
      kenmerk: project.info.project_number ?? "",
      revision_history: [
        {
          version: "1.0",
          date: today,
          author: project.info.engineer ?? "",
          description: "Eerste opzet",
        },
      ],
    },

    toc: {
      enabled: toggles.toc,
      title: "Inhoudsopgave",
      max_depth: 2,
    },

    sections: [
      ...(toggles.uitgangspunten ? [buildUitgangspuntenSection(project)] : []),
      // Diagrammen + gebouwresultaten staan vooraan zodat de lezer eerst de
      // samenvatting/overzicht ziet voordat de detail-secties (constructies +
      // per-vertrek) volgen.
      ...(toggles.diagrammen && diagrammenSection ? [diagrammenSection] : []),
      ...(toggles.gebouwresultaten ? [buildGebouwresultatenSection(result)] : []),
      ...(toggles.constructies && constructiesSection ? [constructiesSection] : []),
      // "Vertrekken" is één parent-sectie (level 1) met overzicht-tabel als
      // eerste content, gevolgd door per-vertrek sub-chapters (level 2). In
      // de TOC krijgen de rooms zo automatisch een geneste positie.
      ...buildVertrekkenChapter(project, result, toggles),
      ...(toggles.tojuli && tojuliSection ? [tojuliSection] : []),
    ],

    backcover: { enabled: toggles.backcover },

    metadata: {
      engine: "isso51-core",
      generated_at: new Date().toISOString(),
      theta_water: thetaWater,
    },
  };
}

/** Sectie 1: Uitgangspunten. */
function buildUitgangspuntenSection(project: Project): Record<string, unknown> {
  const { building, climate, ventilation } = project;
  const thetaWater = climate.theta_water ?? DEFAULT_THETA_WATER;

  return {
    title: "Uitgangspunten",
    level: 1,
    content: [
      {
        type: "table",
        title: "Gebouwgegevens",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Gebouwtype", BUILDING_TYPE_LABELS[building.building_type] ?? building.building_type],
          ["Beveiligingsklasse", SECURITY_CLASS_LABELS[building.security_class] ?? building.security_class],
          ["q_v10-waarde", `${building.qv10} dm³/s`],
          ["Totaal vloeroppervlak", `${building.total_floor_area} m²`],
          ["Aantal bouwlagen", String(building.num_floors ?? 1)],
          ["Nacht-setback", building.has_night_setback ? "Ja" : "Nee"],
          ["Opwarmtijd", `${building.warmup_time ?? 2} uur`],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "table",
        title: "Klimaatgegevens",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Buitentemperatuur (θ_e)", `${climate.theta_e ?? -10} °C`],
          ["Grondtemperatuur (θ_b)", `${climate.theta_b_residential ?? 17} °C`],
          ["Watertemperatuur (θ_w) *", `${thetaWater} °C`],
          ["Windfactor", String(climate.wind_factor ?? 1.0)],
        ],
      },
      {
        type: "paragraph",
        text:
          "<i>* De watertemperatuur θ_w is een engineering-aanname en komt " +
          "<b>niet</b> uit ISSO 51:2023. De standaardwaarde van 5 °C is " +
          "conservatief gekozen voor Nederlandse binnenwateren in " +
          "winterconditie. Deze waarde wordt uitsluitend gebruikt voor " +
          "constructie-elementen met begrenzing 'water' (bijv. " +
          "woonboten of drijvende constructies).</i>",
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "table",
        title: "Ventilatiesysteem",
        headers: ["Parameter", "Waarde"],
        rows: [
          ["Systeemtype", VENTILATION_SYSTEM_LABELS[ventilation.system_type] ?? ventilation.system_type],
          ["Warmteterugwinning", ventilation.has_heat_recovery ? "Ja" : "Nee"],
          ...(ventilation.has_heat_recovery && ventilation.heat_recovery_efficiency != null
            ? [["WTW rendement (η_hr)", `${(ventilation.heat_recovery_efficiency * 100).toFixed(0)}%`]]
            : []),
        ],
      },
      ...(ventilation.has_heat_recovery && ventilation.heat_recovery_efficiency != null
        ? [
            {
              type: "paragraph",
              text:
                "<i>Het WTW-rendement η_hr is ingevoerd via de BCRG-" +
                "productselector of handmatig opgegeven. Verifieer een " +
                "catalogus-waarde tegen de kwaliteitsverklaring op bcrg.nl.</i>",
            },
          ]
        : []),
    ],
  };
}

/** Sectie "Vertrekken" — parent met overzicht-tabel + sub-chapters per vertrek.
 *
 * Output: array sections. Eerste is het parent-hoofdstuk "Vertrekken"
 * (level 1) met de samenvatting-tabel als content. Daarna volgen per vertrek
 * sub-secties (level 2) — die verschijnen in de TOC als geneste items onder
 * het parent-hoofdstuk.
 *
 * Toggle-gedrag:
 * - vertrekkenOverzicht uit + perVertrek uit -> niets
 * - vertrekkenOverzicht aan + perVertrek uit -> alleen parent met tabel
 * - vertrekkenOverzicht uit + perVertrek aan -> parent zonder tabel + sub-chapters
 * - beide aan -> parent met tabel + sub-chapters (default)
 */
function buildVertrekkenChapter(
  project: Project,
  result: ProjectResult,
  toggles: ReportSectionToggles,
): Record<string, unknown>[] {
  if (!toggles.vertrekkenOverzicht && !toggles.perVertrek) {
    return [];
  }
  const parentContent: Record<string, unknown>[] = [];
  if (toggles.vertrekkenOverzicht) {
    parentContent.push({
      type: "table",
      title: "Samenvatting per vertrek",
      headers: [
        "Vertrek",
        "θ_i [°C]",
        "Φ_T [W]",
        "Φ_i [W]",
        "Φ_v [W]",
        "Φ_hu [W]",
        "Φ_sys [W]",
        "Φ_totaal [W]",
      ],
      rows: result.rooms.map((r) => [
        r.room_name,
        fmt2(r.theta_i),
        fmtW(r.transmission.phi_t),
        fmtW(r.infiltration.phi_i),
        fmtW(r.ventilation.phi_v),
        fmtW(r.heating_up.phi_hu),
        fmtW(r.system_losses.phi_system_total),
        fmtW(r.total_heat_loss),
      ]),
    });
  }
  const parent: Record<string, unknown> = {
    title: "Vertrekken",
    level: 1,
    content: parentContent,
  };
  return [
    parent,
    ...(toggles.perVertrek ? buildRoomSections(project, result) : []),
  ];
}

/** Sectie 3.x: Detail per vertrek. */
function buildRoomSections(
  project: Project,
  result: ProjectResult,
): Record<string, unknown>[] {
  return result.rooms.map((room) => buildRoomDetailSection(project, room));
}

/** Eén vertrek-detailsectie — invoer (Algemeen + Constructie-elementen) + reken-resultaten. */
function buildRoomDetailSection(
  project: Project,
  room: RoomResult,
): Record<string, unknown> {
  const projectRoom = project.rooms.find((r) => r.id === room.room_id);
  const heatingLabel = projectRoom
    ? (HEATING_SYSTEM_LABELS[projectRoom.heating_system] ?? projectRoom.heating_system)
    : "";

  const inputBlocks = projectRoom ? buildRoomInputBlocks(projectRoom) : [];

  return {
    title: room.room_name,
    level: 2,
    content: [
      {
        type: "paragraph",
        text: `<b>Verwarmingssysteem:</b> ${heatingLabel}`,
      },
      { type: "spacer", height_mm: 2 },
      ...inputBlocks,
      {
        type: "table",
        title: "Transmissieverliezen",
        headers: ["Component", "Waarde"],
        rows: [
          ["H_T,ie (schil)", `${fmt2(room.transmission.h_t_exterior)} W/K`],
          ["H_T,ia (intern)", `${fmt2(room.transmission.h_t_adjacent_rooms)} W/K`],
          ["H_T,io (onverwarmd)", `${fmt2(room.transmission.h_t_unheated)} W/K`],
          ["H_T,ib (buurwoning)", `${fmt2(room.transmission.h_t_adjacent_buildings)} W/K`],
          ["H_T,ig (grond)", `${fmt2(room.transmission.h_t_ground)} W/K`],
          ["\u03A6_T totaal", `${fmtW(room.transmission.phi_t)} W`],
        ],
      },
      { type: "spacer", height_mm: 2 },
      {
        type: "table",
        title: "Ventilatie & infiltratie",
        headers: ["Component", "Waarde"],
        rows: [
          ["q_v (ventilatie)", `${fmt2(room.ventilation.q_v)} dm³/s`],
          ["H_v", `${fmt2(room.ventilation.h_v)} W/K`],
          ["f_v", fmt2(room.ventilation.f_v)],
          ["\u03A6_v (ventilatie)", `${fmtW(room.ventilation.phi_v)} W`],
          ["H_i (infiltratie)", `${fmt2(room.infiltration.h_i)} W/K`],
          ["\u03A6_i (infiltratie)", `${fmtW(room.infiltration.phi_i)} W`],
        ],
      },
      { type: "spacer", height_mm: 2 },
      {
        type: "table",
        title: "Opwarmtoeslag & systeemverliezen",
        headers: ["Component", "Waarde"],
        rows: [
          ["f_RH", fmt2(room.heating_up.f_rh)],
          ["A_acc", `${fmt2(room.heating_up.accumulating_area)} m²`],
          ["\u03A6_hu", `${fmtW(room.heating_up.phi_hu)} W`],
          ["\u03A6_sys (totaal)", `${fmtW(room.system_losses.phi_system_total)} W`],
        ],
      },
      { type: "spacer", height_mm: 2 },
      {
        type: "table",
        title: "Totaal",
        headers: ["Component", "Waarde"],
        rows: [
          ["\u03A6_basis", `${fmtW(room.basis_heat_loss)} W`],
          ["\u03A6_extra", `${fmtW(room.extra_heat_loss)} W`],
          ["\u03A6_totaal", `<b>${fmtW(room.total_heat_loss)} W</b>`],
        ],
      },
    ],
  };
}

/**
 * Sectie Diagrammen — gestapelde bar, donut, constructie-losses.
 *
 * Elke chart wordt individueel overgeslagen wanneer er geen data is.
 * Wanneer alle charts leeg zijn returnt de functie `null` en wordt
 * de sectie volledig uit het rapport weggelaten.
 *
 * SVG-charts worden client-side gerasterized naar PNG omdat de
 * BM Reports (PyMuPDF) backend `image/svg+xml` niet echt kan parsen.
 */
async function buildDiagrammenSection(
  project: Project,
  result: ProjectResult,
): Promise<Record<string, unknown> | null> {
  const STACKED_WIDTH_MM = 170;
  const DONUT_WIDTH_MM = 170;
  const CONSTRUCTION_WIDTH_MM = 150;
  const SPACER_MM = 4;

  const content: Record<string, unknown>[] = [];

  const stackedSvg = buildStackedBarSvg(result.rooms);
  if (stackedSvg) {
    const png = await rasterizeSvgToPng(stackedSvg);
    content.push({
      type: "image",
      src: {
        data: png.data,
        media_type: "image/png",
        filename: "verliezen-per-vertrek.png",
      },
      caption: "Warmteverliezen per vertrek",
      width_mm: STACKED_WIDTH_MM,
      alignment: "center",
    });
    content.push({ type: "spacer", height_mm: SPACER_MM });
  }

  const donutSvg = buildSummaryDonutSvg(result.summary);
  if (donutSvg) {
    const png = await rasterizeSvgToPng(donutSvg);
    content.push({
      type: "image",
      src: {
        data: png.data,
        media_type: "image/png",
        filename: "gebouwtotaal.png",
      },
      caption: "Gebouwtotaal warmteverliezen per type",
      width_mm: DONUT_WIDTH_MM,
      alignment: "center",
    });
    content.push({ type: "spacer", height_mm: SPACER_MM });
  }

  const constructionSvg = buildConstructionLossSvg(
    project.rooms,
    project.climate.theta_e ?? DEFAULT_THETA_E,
    project.climate.theta_water,
  );
  if (constructionSvg) {
    const png = await rasterizeSvgToPng(constructionSvg);
    content.push({
      type: "image",
      src: {
        data: png.data,
        media_type: "image/png",
        filename: "verlies-per-constructietype.png",
      },
      caption: "Verlies per constructietype",
      width_mm: CONSTRUCTION_WIDTH_MM,
      alignment: "center",
    });
  }

  if (content.length === 0) return null;

  return {
    title: "Diagrammen",
    level: 1,
    content,
  };
}

/** Sectie 4: Gebouwresultaten. */
function buildGebouwresultatenSection(result: ProjectResult): Record<string, unknown> {
  const { summary } = result;

  return {
    title: "Gebouwresultaten",
    level: 1,
    content: [
      {
        type: "table",
        title: "Totalen",
        headers: ["Component", "Waarde"],
        rows: [
          ["Transmissie (schil)", `${fmtW(summary.total_envelope_loss)} W`],
          ["Buurwoningverlies", `${fmtW(summary.total_neighbor_loss)} W`],
          ["Ventilatie", `${fmtW(summary.total_ventilation_loss)} W`],
          ["Opwarmtoeslag", `${fmtW(summary.total_heating_up)} W`],
          ["Systeemverliezen", `${fmtW(summary.total_system_losses)} W`],
          ["Collectieve bijdrage", `${fmtW(summary.collective_contribution)} W`],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "calculation",
        title: "Aansluitvermogen",
        result: fmtW(summary.connection_capacity),
        unit: "W",
        reference: "ISSO 51:2023",
      },
    ],
  };
}

// ---------------------------------------------------------------------------
// Invoer per vertrek — Algemeen + Constructie-elementen
// ---------------------------------------------------------------------------

/** Format °C with one decimal for design temperatures. */
function fmtTemp(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return "—";
  return `${value.toFixed(1)} °C`;
}

/** Resolve design temperature for a room: explicit override or ROOM_FUNCTION_TEMPERATURES. */
function resolveDesignTemp(room: Room): number {
  if (room.custom_temperature != null) return room.custom_temperature;
  return ROOM_FUNCTION_TEMPERATURES[room.function] ?? 20;
}

/** "Wand"/"Vloer"/"Plafond" — fallback to material_type when vertical_position is missing. */
function elementTypeLabel(
  element: Room["constructions"][number],
): string {
  if (element.vertical_position) {
    return VERTICAL_POSITION_LABELS[element.vertical_position] ?? element.vertical_position;
  }
  // Fallback: surface material types (window/door/etc.) keep their original tag
  return element.material_type ?? "—";
}

/** Build the per-room "Invoer" blocks: Algemeen + Constructie-elementen. */
function buildRoomInputBlocks(room: Room): Record<string, unknown>[] {
  const designTemp = resolveDesignTemp(room);
  const functionLabel = ROOM_FUNCTION_LABELS[room.function] ?? room.function;

  const algemeenRows: [string, string][] = [
    ["Functie", functionLabel],
    ["Ontwerptemperatuur (θ_i)", fmtTemp(designTemp)],
    ["Vloeroppervlak", `${fmt2(room.floor_area)} m²`],
  ];
  if (room.height != null) {
    algemeenRows.push(["Hoogte", `${fmt2(room.height)} m`]);
  }

  // Constructie-elementen — one row per element
  const elementRows: string[][] = room.constructions.map((el) => {
    const boundaryLabel = BOUNDARY_TYPE_LABELS[el.boundary_type] ?? el.boundary_type;
    let aangrenzend = "—";
    if (el.boundary_type === "adjacent_room" && el.adjacent_temperature != null) {
      aangrenzend = fmtTemp(el.adjacent_temperature);
    } else if (
      el.boundary_type === "adjacent_room" &&
      el.temperature_factor != null
    ) {
      aangrenzend = `b = ${fmt2(el.temperature_factor)}`;
    }
    const embedded = el.has_embedded_heating ? "ja" : "—";
    return [
      el.description || "—",
      elementTypeLabel(el),
      `${fmt2(el.area)} m²`,
      `${fmt2(el.u_value)} W/m²K`,
      boundaryLabel,
      aangrenzend,
      embedded,
    ];
  });

  const blocks: Record<string, unknown>[] = [
    {
      type: "table",
      title: "Algemeen",
      headers: ["Parameter", "Waarde"],
      rows: algemeenRows,
    },
    { type: "spacer", height_mm: 2 },
  ];

  if (elementRows.length > 0) {
    blocks.push({
      type: "table",
      title: "Constructie-elementen (invoer)",
      headers: [
        "Omschrijving",
        "Type",
        "Oppervlak",
        "U",
        "Grens",
        "Aangrenzend",
        "Embedded heating",
      ],
      rows: elementRows,
    });
    blocks.push({ type: "spacer", height_mm: 2 });
  }

  return blocks;
}

// ---------------------------------------------------------------------------
// Constructie-opbouw & Rc-waarden — sectie ná Uitgangspunten, vóór Vertrekken
// ---------------------------------------------------------------------------

/** Build the "Constructie-opbouw & Rc-waarden" section.
 *
 * Per ProjectConstruction met layers: ISO 6946 Rc/U-resultaat via `calculateRc`.
 * Voor elke layered opbouw ook een temperatuurverloop-grafiek
 * (`buildTemperatureGradientSvg` → PNG) zodat de lezer per laag de
 * grensvlak-temperaturen ziet bij ontwerpcondities θ_i / θ_e.
 * Layer-loze constructies (kozijnen/glas/deuren met directe U) krijgen een
 * mini-blok met alleen de U-waarde.
 *
 * Returns null when there are no constructions to render.
 */
async function buildConstructiesSection(
  projectConstructions: ProjectConstruction[],
  thetaI: number,
  thetaE: number,
): Promise<Record<string, unknown> | null> {
  if (projectConstructions.length === 0) return null;

  const content: Record<string, unknown>[] = [];

  // Overzicht-tabel — alle constructies met Rc + U op een rij, zodat de
  // lezer eerst de samenvatting ziet voordat de per-laag detail-blokken
  // beginnen. Layer-loze opbouwen (kozijnen/glas) tonen alleen U.
  const overviewRows: string[][] = projectConstructions.map((pc) => {
    const typeLabel = pc.verticalPosition
      ? VERTICAL_POSITION_LABELS[pc.verticalPosition] ?? pc.verticalPosition
      : "—";
    if (pc.layers.length === 0) {
      const uText = pc.uValue != null ? fmt2(pc.uValue) : "—";
      return [pc.name, typeLabel, "—", uText];
    }
    try {
      const rcResult = calculateRc(
        pc.layers.map((l) => ({
          materialId: l.materialId,
          thickness: l.thickness,
          lambdaOverride: l.lambdaOverride,
          stud: l.stud,
        })),
        pc.verticalPosition,
      );
      return [pc.name, typeLabel, fmt2(rcResult.rc), fmt2(rcResult.uValue)];
    } catch {
      return [pc.name, typeLabel, "—", "—"];
    }
  });
  content.push({
    type: "table",
    title: "Overzicht — alle constructies",
    headers: ["Naam", "Type", "Rc [m²·K/W]", "U [W/m²·K]"],
    rows: overviewRows,
  });
  content.push({ type: "spacer", height_mm: 6 });

  for (const pc of projectConstructions) {
    // Sub-heading per constructie (level 2)
    content.push({
      type: "paragraph",
      text: `<b>${pc.name}</b>`,
    });
    content.push({ type: "spacer", height_mm: 1 });

    if (pc.layers.length > 0) {
      // Layered construction — compute Rc via ISO 6946.
      // ProjectConstruction.layers IS already CatalogueLayer[] which matches
      // LayerInput shape (materialId / thickness / lambdaOverride / stud).
      const layerInputs: LayerInput[] = pc.layers.map((l) => ({
        materialId: l.materialId,
        thickness: l.thickness,
        lambdaOverride: l.lambdaOverride,
        stud: l.stud,
      }));

      let rcResult;
      try {
        rcResult = calculateRc(layerInputs, pc.verticalPosition);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        content.push({
          type: "paragraph",
          text: `<i>Rc-berekening mislukt: ${msg}</i>`,
        });
        content.push({ type: "spacer", height_mm: 4 });
        continue;
      }

      // Laagopbouw tabel
      const layerRows = pc.layers.map((l, i) => {
        const r = rcResult.layers[i];
        const material = getMaterialById(l.materialId);
        const materialName = material?.name ?? l.materialId;
        const lambdaVal = l.lambdaOverride ?? material?.lambda;
        const rValue = r ? fmt2(r.r) : "—";
        return [
          materialName,
          `${fmt2(l.thickness)} mm`,
          lambdaVal != null ? fmt2(lambdaVal) : "—",
          rValue,
        ];
      });

      content.push({
        type: "table",
        title: "Laagopbouw",
        headers: ["Materiaal", "Dikte", "λ (W/m·K)", "R (m²·K/W)"],
        rows: layerRows,
      });
      content.push({ type: "spacer", height_mm: 2 });

      // Resultaten tabel
      const resultRows: [string, string][] = [
        ["R_si (binnenoppervlakteweerstand)", `${fmt2(rcResult.rSi)} m²·K/W`],
        ["Σ R_lagen", `${fmt2(rcResult.rc)} m²·K/W`],
        ["R_se (buitenoppervlakteweerstand)", `${fmt2(rcResult.rSe)} m²·K/W`],
        ["R_totaal", `${fmt2(rcResult.rTotal)} m²·K/W`],
        ["<b>Rc</b>", `<b>${fmt2(rcResult.rc)} m²·K/W</b>`],
        ["<b>U</b>", `<b>${fmt2(rcResult.uValue)} W/m²·K</b>`],
      ];

      if (rcResult.rUpper != null && rcResult.rLower != null) {
        resultRows.push(
          ["R'_T (bovengrens)", `${fmt2(rcResult.rUpper)} m²·K/W`],
          ["R''_T (ondergrens)", `${fmt2(rcResult.rLower)} m²·K/W`],
        );
        if (rcResult.ratio != null) {
          const ratioOk = rcResult.ratio < 1.5;
          resultRows.push([
            "Ratio R'_T/R''_T",
            `${fmt2(rcResult.ratio)} ${ratioOk ? "(< 1,5 — ok)" : "(≥ 1,5 — buiten ISO 6946 §6.7.2)"}`,
          ]);
        }
      }

      if (rcResult.deltaUf != null && rcResult.deltaUf > 0) {
        resultRows.push([
          "ΔU_f (bevestigingsmiddelen)",
          `${fmt2(rcResult.deltaUf)} W/m²·K`,
        ]);
      }

      content.push({
        type: "table",
        title: "Resultaten (ISO 6946)",
        headers: ["Parameter", "Waarde"],
        rows: resultRows,
      });
      content.push({ type: "spacer", height_mm: 3 });

      // Temperatuurverloop diagram — stationair regime, θ_i / θ_e uit project
      const tempLayers = pc.layers.map((l, i) => {
        const r = rcResult.layers[i];
        const material = getMaterialById(l.materialId);
        return {
          name: material?.name ?? l.materialId,
          thickness: l.thickness,
          r: r?.r ?? 0,
          // Geef materiaal-categorie mee zodat de doorsnede architectonisch
          // gekleurd wordt (baksteen rood, hout bruin, isolatie geel, enz.)
          // i.p.v. wisselende grijstinten.
          category: material?.category,
        };
      });
      const tempSvg = buildTemperatureGradientSvg(
        tempLayers,
        rcResult.rSi,
        rcResult.rSe,
        thetaI,
        thetaE,
      );
      if (tempSvg) {
        try {
          const png = await rasterizeSvgToPng(tempSvg);
          content.push({
            type: "image",
            src: {
              data: png.data,
              media_type: "image/png",
              filename: `temp-gradient-${pc.id}.png`,
            },
            caption: `Temperatuurverloop bij θ_i = ${thetaI.toFixed(0)}°C / θ_e = ${thetaE.toFixed(0)}°C`,
            width_mm: 160,
            alignment: "center",
          });
        } catch (err) {
          // SVG rasterize can fail in non-DOM environments; degrade silently.
          // eslint-disable-next-line no-console
          console.warn("[report] temp gradient rasterize failed:", err);
        }
      }
    } else {
      // Layerless — direct U-value (kozijnen/glas/deuren)
      const uText =
        pc.uValue != null ? `${fmt2(pc.uValue)} W/m²·K` : "niet ingevoerd";
      content.push({
        type: "table",
        title: "U-waarde (direct ingevoerd)",
        headers: ["Parameter", "Waarde"],
        rows: [["U", uText]],
      });
    }

    content.push({ type: "spacer", height_mm: 6 });
  }

  return {
    title: "Constructie-opbouw & Rc-waarden",
    level: 1,
    content,
  };
}

// =====================================================================
// TO-juli sectie — vereenvoudigde koelbehoefte (NTA 8800 bijlage AA)
// =====================================================================

/** Resultaat-shape van `simplified_cooling` Tauri command — mirror van
 *  `nta8800_cooling::SimplifiedCoolingResult`. */
interface SimplifiedCoolingResult {
  minimum_capacity_w: number;
  internal_load_w: number;
  outdoor_load_w: number;
  opaque_transmission_w: number;
  solar_load_w: number;
  glazing_transmission_w: number;
  peak_cooling_load_w: number;
  maatgevende_koelbehoefte_w_per_m2: number;
}

/** Leid Simplified-cooling inputs af uit project + sane defaults.
 *
 * Wat afleidbaar is uit project geometrie/instellingen:
 * - living_area_m2 = som van rooms[].area (woonkamers + slaapkamers)
 * - infiltration_m3_per_h = qv10 (dm³/s) × 3.6
 * - mechanical_supply_m3_per_h uit ventilation.q_v indien beschikbaar
 * - construction_year uit project.info indien aanwezig, anders 1990
 * - opaque_area_m2 = som van rooms[].boundaries waar boundary_type
 *   exterior is en construction geen glas-aandeel heeft (heuristiek)
 *
 * Wat default krijgt (V1 — gebruiker kan via TojuliFull-page de echte
 * waardes invullen voor de norm-conforme berekening):
 * - dwelling_count = 1
 * - persons_per_dwelling = 2.4 (Nederlandse default)
 * - peak_hour = 14 (warmste uur — bijlage AA referentie)
 * - solar_load_w / glazing_transmission_w = 0 (vereist V2 zon-pad)
 */
function deriveTojuliInputs(project: Project): {
  living_area_m2: number;
  other_area_m2: number;
  dwelling_count: number;
  persons_per_dwelling: number;
  infiltration_m3_per_h: number;
  natural_ventilation_m3_per_h: number;
  mechanical_supply_m3_per_h: number;
  peak_hour: number;
  construction_year: number;
  opaque_area_m2: number;
  solar_load_w: number;
  glazing_transmission_w: number;
} {
  const livingArea = project.rooms.reduce(
    (sum, r) => sum + (r.floor_area ?? 0),
    0,
  );
  const qv10DmPerS = project.building.qv10 ?? 0;
  const infiltrationM3PerH = qv10DmPerS * 3.6; // dm³/s → m³/h
  // ISSO 51 ventilation-model heeft geen centraal q_v veld; afleiden uit
  // ruimte-rates wanneer beschikbaar (ventilation_rate in dm³/s per ruimte).
  const mechanicalSupplyDmPerS = project.rooms.reduce(
    (sum, r) => sum + (r.has_mechanical_supply ? r.ventilation_rate ?? 0 : 0),
    0,
  );
  const mechanicalSupplyM3PerH = mechanicalSupplyDmPerS * 3.6;
  const naturalVentM3PerH = 0;
  // ProjectInfo bevat geen construction_year — defaulten op 2000 (recent
  // bouwbesluit); user kan via TojuliFull-page exacter rekenen.
  const constructionYear = 2000;
  // Heuristiek: tel exterior-constructie-oppervlakken (m²).
  let opaqueAreaM2 = 0;
  for (const room of project.rooms) {
    for (const c of room.constructions ?? []) {
      if (c.boundary_type === "exterior" && c.area) {
        opaqueAreaM2 += c.area;
      }
    }
  }

  return {
    living_area_m2: livingArea,
    other_area_m2: 0,
    dwelling_count: 1,
    persons_per_dwelling: 2.4,
    infiltration_m3_per_h: infiltrationM3PerH,
    natural_ventilation_m3_per_h: naturalVentM3PerH,
    mechanical_supply_m3_per_h: mechanicalSupplyM3PerH,
    peak_hour: 14,
    construction_year: constructionYear,
    opaque_area_m2: opaqueAreaM2,
    solar_load_w: 0,
    glazing_transmission_w: 0,
  };
}

/** Bouw TO-juli rapport-sectie (Simplified — NTA 8800 bijlage AA).
 *
 * Roept `simplified_cooling` Tauri command aan met afgeleide inputs uit het
 * project; faalt graceful (geeft een placeholder-paragraaf) als Tauri niet
 * beschikbaar is of de berekening mislukt — het rapport moet altijd
 * gegenereerd kunnen worden.
 */
async function buildTojuliSection(
  project: Project,
): Promise<Record<string, unknown> | null> {
  const inputs = deriveTojuliInputs(project);

  let result: SimplifiedCoolingResult | null = null;
  let errorMessage: string | null = null;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    result = await invoke<SimplifiedCoolingResult>("simplified_cooling", {
      req: inputs,
    });
  } catch (err) {
    errorMessage =
      err instanceof Error ? err.message : String(err ?? "onbekende fout");
  }

  const content: Array<Record<string, unknown>> = [
    {
      type: "paragraph",
      text:
        "Indicatieve koelbehoefte op basis van project-geometrie en standaard-aannames " +
        "(1 woning, 2,4 personen, piekuur 14:00, geen zon-aandeel). Voor een norm-conforme " +
        "TO-juli berekening volgens NTA 8800 hoofdstuk 10 zie de TO-juli pagina in de tool.",
    },
    {
      type: "table",
      title: "Afgeleide invoer",
      headers: ["Parameter", "Waarde"],
      rows: [
        ["Vloeroppervlak (leefzone)", `${fmt2(inputs.living_area_m2)} m²`],
        ["Infiltratie", `${fmt2(inputs.infiltration_m3_per_h)} m³/h`],
        [
          "Mechanische toevoer",
          `${fmt2(inputs.mechanical_supply_m3_per_h)} m³/h`,
        ],
        ["Bouwjaar", String(inputs.construction_year)],
        ["Opaak gevel-oppervlak", `${fmt2(inputs.opaque_area_m2)} m²`],
        ["Aantal woningen", String(inputs.dwelling_count)],
        ["Personen per woning", fmt2(inputs.persons_per_dwelling)],
        ["Piekuur", `${inputs.peak_hour}:00`],
      ],
    },
  ];

  if (result) {
    content.push({
      type: "table",
      title: "Resultaten (NTA 8800 bijlage AA)",
      headers: ["Grootheid", "Waarde"],
      rows: [
        ["Piek-koelvermogen", `${fmtW(result.peak_cooling_load_w)} W`],
        [
          "Maatgevende koelbehoefte",
          `${fmt2(result.maatgevende_koelbehoefte_w_per_m2)} W/m²`,
        ],
        ["Minimum koelcapaciteit", `${fmtW(result.minimum_capacity_w)} W`],
        ["Interne warmtelast", `${fmtW(result.internal_load_w)} W`],
        ["Buitenlucht warmtelast", `${fmtW(result.outdoor_load_w)} W`],
        [
          "Transmissie (opaak)",
          `${fmtW(result.opaque_transmission_w)} W`,
        ],
        ["Zoninstraling", `${fmtW(result.solar_load_w)} W`],
        [
          "Transmissie (glas)",
          `${fmtW(result.glazing_transmission_w)} W`,
        ],
      ],
    });
  } else {
    content.push({
      type: "paragraph",
      text: errorMessage
        ? `TO-juli berekening niet uitgevoerd: ${errorMessage}`
        : "TO-juli berekening niet beschikbaar in deze omgeving (alleen desktop-versie).",
    });
  }

  return {
    title: "TO-juli — vereenvoudigde koelbehoefte",
    level: 1,
    content,
  };
}
