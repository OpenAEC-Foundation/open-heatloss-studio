/**
 * Bouwt BM Reports JSON data op vanuit project + berekeningsresultaat.
 *
 * Output conform report.schema.json (OpenAEC Reports API).
 */
import type { Project, ProjectResult, RoomResult } from "../types";
import {
  BUILDING_TYPE_LABELS,
  DEFAULT_THETA_WATER,
  HEATING_SYSTEM_LABELS,
  SECURITY_CLASS_LABELS,
  VENTILATION_SYSTEM_LABELS,
} from "./constants";

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

/** Build BM Reports JSON from project input + calculation result. */
export function buildReportData(
  project: Project,
  result: ProjectResult,
): Record<string, unknown> {
  const today = todayIso();
  const projectName = project.info.name || "Naamloos project";
  const thetaWater = project.climate.theta_water ?? DEFAULT_THETA_WATER;

  return {
    template: "blank",
    brand: "3bm_cooperatie",
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

    colofon: {
      enabled: true,
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
      enabled: true,
      title: "Inhoudsopgave",
      max_depth: 2,
    },

    sections: [
      buildUitgangspuntenSection(project),
      buildVertrekkenOverzichtSection(result),
      ...buildRoomSections(project, result),
      buildGebouwresultatenSection(result),
    ],

    backcover: { enabled: true },

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
            ? [["WTW rendement", `${(ventilation.heat_recovery_efficiency * 100).toFixed(0)}%`]]
            : []),
        ],
      },
    ],
  };
}

/** Sectie 2: Vertrekken overzicht. */
function buildVertrekkenOverzichtSection(result: ProjectResult): Record<string, unknown> {
  return {
    title: "Vertrekken overzicht",
    level: 1,
    content: [
      {
        type: "table",
        title: "Samenvatting per vertrek",
        headers: [
          "Vertrek",
          "θ_i [°C]",
          "\u03A6_T [W]",
          "\u03A6_i [W]",
          "\u03A6_v [W]",
          "\u03A6_hu [W]",
          "\u03A6_sys [W]",
          "\u03A6_totaal [W]",
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
      },
    ],
  };
}

/** Sectie 3.x: Detail per vertrek. */
function buildRoomSections(
  project: Project,
  result: ProjectResult,
): Record<string, unknown>[] {
  return result.rooms.map((room) => buildRoomDetailSection(project, room));
}

/** Eén vertrek-detailsectie. */
function buildRoomDetailSection(
  project: Project,
  room: RoomResult,
): Record<string, unknown> {
  const projectRoom = project.rooms.find((r) => r.id === room.room_id);
  const heatingLabel = projectRoom
    ? (HEATING_SYSTEM_LABELS[projectRoom.heating_system] ?? projectRoom.heating_system)
    : "";

  return {
    title: room.room_name,
    level: 2,
    content: [
      {
        type: "paragraph",
        text: `<b>Verwarmingssysteem:</b> ${heatingLabel}`,
      },
      { type: "spacer", height_mm: 2 },
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
