/**
 * Chart-data-afleiding voor het ISSO 53-resultatenscherm.
 *
 * De bestaande SVG-chart-componenten (`StackedBarChart`, `SummaryDonut`,
 * `ConstructionLossChart`) zijn geschreven tegen het ISSO 51-resultshape
 * (`RoomResult` / `BuildingSummary`). ISSO 53 levert een eigen, vlak shape
 * (`Isso53ProjectResult`) met losse `phiT`/`phiV`/`phiI`/... velden.
 *
 * Deze module adapteert het ISSO 53-result naar exact de shapes die de
 * chart-componenten consumeren, zónder die componenten te herschrijven.
 *
 * HERGEBRUIK: de latere PDF-rapport-agent (`isso53ReportBuilder.ts`) kan
 * dezelfde derivaties gebruiken om chart-images server-side/Canvas-side op te
 * bouwen, zodat scherm en rapport identieke cijfers tonen.
 *
 * Mapping-notities:
 * - StackedBarChart: ISSO 53 heeft per vertrek alle 5 categorieën
 *   (transmissie, ventilatie, infiltratie, opwarmtoeslag, systeem) als losse
 *   velden → 1:1 te mappen naar het geneste `RoomResult`-shape dat de chart
 *   uitleest.
 * - SummaryDonut: ISSO 53 splitst infiltratie áf van ventilatie (anders dan
 *   ISSO 51). De donut wordt daarom met een eigen `segments`-set gevoed
 *   (incl. infiltratie, géén buurwoningverlies — dat veld bestaat niet als
 *   gebouwtotaal in het ISSO 53-result). Het centrum-label toont het
 *   maatgevende aansluitvermogen (max van individueel/collectief).
 * - ConstructionLossChart: norm-onafhankelijk — die rekent rechtstreeks op
 *   `Project.rooms` + θ_e en hoeft geen result-adapter. Daarom hier geen
 *   helper voor; geef in de pagina direct `project.rooms` mee.
 */

import type { BuildingSummary, RoomResult } from "../types";
import type {
  Isso53BuildingSummary,
  Isso53ProjectResult,
  Isso53RoomResult,
} from "../types/isso53Result";
import { LOSS_TYPE_COLORS } from "./chartColors";
import type { DonutSegment } from "../components/charts/SummaryDonut";

/**
 * Map één ISSO 53-vertrekresultaat naar het geneste `RoomResult`-shape dat
 * `StackedBarChart` uitleest. Alleen de velden die de chart daadwerkelijk
 * gebruikt worden gevuld; overige (norm_refs etc.) blijven leeg/0 — de chart
 * raakt ze niet aan.
 */
function isso53RoomToChartRoom(room: Isso53RoomResult): RoomResult {
  return {
    room_id: room.roomId,
    room_name: room.roomName,
    theta_i: room.thetaI,
    transmission: {
      h_t_exterior: room.hTExterior,
      h_t_adjacent_rooms: room.hTAdjacentRooms,
      h_t_unheated: room.hTUnheated,
      h_t_adjacent_buildings: room.hTAdjacentBuildings,
      h_t_ground: room.hTGround,
      phi_t: room.phiT,
    },
    infiltration: {
      h_i: room.hI,
      z_i: 0,
      phi_i: room.phiI,
    },
    ventilation: {
      h_v: room.hV,
      f_v: 0,
      q_v: 0,
      phi_v: room.phiV,
      phi_vent: room.phiV,
    },
    heating_up: {
      phi_hu: room.phiHu,
      f_rh: 0,
      accumulating_area: 0,
    },
    system_losses: {
      phi_floor_loss: 0,
      phi_wall_loss: 0,
      phi_ceiling_loss: 0,
      phi_system_total: room.phiSystem,
    },
    total_heat_loss: room.totalHeatLoss,
    basis_heat_loss: room.phiT + room.phiV + room.phiI,
    extra_heat_loss: room.phiHu + room.phiSystem,
  };
}

/**
 * Vertrekken voor `StackedBarChart` (verliezen per vertrek), afgeleid uit het
 * ISSO 53-result. Categorieën: transmissie / ventilatie / infiltratie /
 * opwarmtoeslag / systeem.
 */
export function isso53StackedBarRooms(
  result: Isso53ProjectResult,
): RoomResult[] {
  return result.rooms.map(isso53RoomToChartRoom);
}

/** Maatgevend aansluitvermogen = max(individueel, collectief). */
export function isso53MaatgevendVermogen(
  summary: Isso53BuildingSummary,
): number {
  return Math.max(
    summary.connectionCapacityIndividual,
    summary.connectionCapacityCollective,
  );
}

/**
 * `BuildingSummary`-shaped object voor `SummaryDonut`. ISSO 53 kent géén
 * buurwoning-gebouwtotaal en splitst infiltratie áf van ventilatie. We vullen
 * de bekende velden; de donut wordt met een ISSO 53-specifieke segmentenset
 * gevoed (zie `ISSO53_DONUT_SEGMENTS`) die infiltratie meeneemt en
 * buurwoningverlies weglaat.
 *
 * Het centrum-label van de donut leest `connection_capacity` → we zetten dat
 * op het maatgevende aansluitvermogen.
 */
export function isso53DonutSummary(
  summary: Isso53BuildingSummary,
): BuildingSummary {
  return {
    total_envelope_loss: summary.totalTransmissionLoss,
    total_ventilation_loss: summary.totalVentilationLoss,
    total_heating_up: summary.totalHeatingUp,
    total_system_losses: summary.totalSystemLosses,
    // Hergebruik het `neighbor`-veld als drager voor infiltratie zou
    // misleidend zijn in de legenda; we laten het op 0 en voeren de
    // infiltratie via een eigen segment-definitie aan (zie hieronder).
    total_neighbor_loss: 0,
    connection_capacity: isso53MaatgevendVermogen(summary),
    collective_contribution: summary.connectionCapacityCollective,
  };
}

/**
 * Donut-segmenten voor ISSO 53 (gebouwtotaal per verliestype). Wijkt af van de
 * ISSO 51-default: infiltratie is een eigen segment, buurwoningverlies
 * ontbreekt (bestaat niet als gebouwtotaal in het ISSO 53-result).
 *
 * De `value`-extractor leest rechtstreeks uit het ISSO 53-summary, zodat we
 * niet afhankelijk zijn van het op-0-gezette `total_neighbor_loss` in
 * `isso53DonutSummary`.
 */
export function isso53DonutSegments(
  summary: Isso53BuildingSummary,
): DonutSegment[] {
  return [
    {
      key: "transmission",
      label: "Transmissie",
      color: LOSS_TYPE_COLORS.transmission,
      value: Math.max(0, summary.totalTransmissionLoss),
    },
    {
      key: "ventilation",
      label: "Ventilatie",
      color: LOSS_TYPE_COLORS.ventilation,
      value: Math.max(0, summary.totalVentilationLoss),
    },
    {
      key: "infiltration",
      label: "Infiltratie",
      color: LOSS_TYPE_COLORS.neighbor,
      value: Math.max(0, summary.totalInfiltrationLoss),
    },
    {
      key: "heating_up",
      label: "Opwarmtoeslag",
      color: LOSS_TYPE_COLORS.heatingUp,
      value: Math.max(0, summary.totalHeatingUp),
    },
    {
      key: "system",
      label: "Systeemverliezen",
      color: LOSS_TYPE_COLORS.system,
      value: Math.max(0, summary.totalSystemLosses),
    },
  ];
}
