/**
 * Bouwt BM Reports JSON data op vanuit de ventilatiebalans-state.
 *
 * Output conform report.schema.json (OpenAEC Reports API).
 * Secties: uitgangspunten (systeem A–D + norm-grondslag + projectgegevens),
 * balans per vertrek (eis vs. aanwezig, BBL afd. 3.6) en de gebouwbalans
 * (totalen + balansoordeel, zoals het Modeller-zijpaneel toont).
 *
 * Eigen, zelfstandig rapport — bewust GEEN sectie in het ISSO 51-rapport
 * (user-besluit 09-06). Mirror van `uwReportBuilder.ts` / `rcReportBuilder.ts`
 * qua envelope (template "standaard_rapport", cover/colofon/toc/backcover).
 *
 * Pure functie: geen React, geen store-toegang — alle balansdata komt binnen
 * via {@link VentilationReportInput} en wordt geaggregeerd met
 * {@link aggregateVentilationBalance} (zelfde cijfers als de `/ventilation`-tab).
 *
 * **Eenheden:** dm³/s primair, m³/h als afgeleide weergave (× 3,6) — conform
 * de UI (`components/ventilation/shared.tsx`).
 */

import type { ProjectInfo, Room } from "../types/project";
import {
  DEFAULT_OCCUPANCY_DM3S_PER_PERSON,
  ventilationSystemOf,
  type VentilationRoomState,
  type VentilationSystemInfo,
  type VentilationSystemKey,
  type VentilationTerminal,
} from "../types/ventilation";
import {
  aggregateVentilationBalance,
  BALANCE_TOLERANCE_DM3S,
  type BuildingVentilationBalance,
  type RoomVentilationBalance,
} from "./ventilationBalance";
import { flowLabel, m3hLabel } from "../components/ventilation/shared";
import { formatArea, formatDecimals } from "./formatNumber";

/** ISO date string for today. */
function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

/** Minimale per-vertrek invoer die het rapport nodig heeft. */
export type VentilationReportRoom = Pick<Room, "id" | "name" | "floor_area">;

export interface VentilationReportInput {
  /** Projectgegevens (naam, nummer, adres, opdrachtgever, …). */
  info: ProjectInfo;
  /** Vertrekken in project-volgorde (subset van `Project.rooms`). */
  rooms: ReadonlyArray<VentilationReportRoom>;
  /** Per-room ventilatie-state uit `deriveVentilationDemand` (gekeyed op room.id). */
  ventilationRooms: Record<string, VentilationRoomState>;
  /** Geplaatste ventielen/roosters (uit `projectStore.ventilation.terminals`). */
  terminals: VentilationTerminal[];
  /** Ventilatiesysteem A–D (`undefined` = default, zie `ventilationSystemOf`). */
  system?: VentilationSystemKey;
}

/** Build BM Reports JSON from ventilatiebalans-state. */
export function buildVentilationReportData(
  input: VentilationReportInput,
): Record<string, unknown> {
  const today = todayIso();
  const title = input.info.name || "Ventilatiebalans";
  const sys = ventilationSystemOf({ system: input.system });
  const balance = aggregateVentilationBalance(
    input.ventilationRooms,
    input.terminals,
    input.system,
  );

  return {
    template: "standaard_rapport",
    format: "A4",
    orientation: "portrait",
    project: title,
    author: "3BM Bouwkunde",
    date: input.info.date || today,
    version: "1.0",
    status: "CONCEPT",

    cover: {
      subtitle: "Ventilatiebalans conform BBL afd. 3.6 (NEN 1087 indicatief)",
    },

    colofon: {
      enabled: true,
      adviseur_bedrijf: "3BM Bouwkunde",
      normen:
        "BBL afd. 3.6 (ventilatiedebieten per gebruiksfunctie), " +
        "NEN 1087 (indicatief — overstroom/doorstroomopeningen)",
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
      buildAssumptionsSection(input, sys),
      buildRoomBalanceSection(input, balance, sys),
      buildBuildingBalanceSection(balance),
    ],

    backcover: { enabled: true },

    metadata: {
      engine: "ventilation-balance",
      generated_at: new Date().toISOString(),
    },
  };
}

// ---------------------------------------------------------------------------
// Sectie 1: Uitgangspunten
// ---------------------------------------------------------------------------

function buildAssumptionsSection(
  input: VentilationReportInput,
  sys: VentilationSystemInfo,
): Record<string, unknown> {
  const { info } = input;

  const projectRows: string[][] = [["Projectnaam", info.name || "Naamloos"]];
  if (info.project_number) projectRows.push(["Projectnummer", info.project_number]);
  if (info.address) projectRows.push(["Adres", info.address]);
  if (info.client) projectRows.push(["Opdrachtgever", info.client]);
  if (info.engineer) projectRows.push(["Adviseur", info.engineer]);
  projectRows.push(["Datum", info.date || todayIso()]);

  const basisRows: string[][] = [
    ["Ventilatiesysteem", sys.label],
    [
      "Toetsing toevoer",
      sys.supplyMechanical
        ? "Mechanisch — getoetst op ventielen"
        : "Natuurlijk — via gevelroosters (geen ventiel-toetsing)",
    ],
    [
      "Toetsing afvoer",
      sys.exhaustMechanical
        ? "Mechanisch — getoetst op ventielen"
        : "Natuurlijk — geen ventiel-toetsing",
    ],
    ["Eisen per gebruiksfunctie", "BBL afd. 3.6 (Bouwbesluit)"],
    ["Overstroom / doorstroomopeningen", "NEN 1087 — indicatief"],
    [
      "Personen-toeslag",
      `${formatDecimals(DEFAULT_OCCUPANCY_DM3S_PER_PERSON, 1)} dm³/s per persoon`,
    ],
  ];

  return {
    title: "Uitgangspunten",
    level: 1,
    content: [
      {
        type: "table",
        title: "Projectgegevens",
        headers: ["Parameter", "Waarde"],
        rows: projectRows,
      },
      { type: "spacer", height_mm: 2 },
      {
        type: "table",
        title: "Ventilatiesysteem & norm-grondslag",
        headers: ["Parameter", "Waarde"],
        rows: basisRows,
      },
      {
        type: "paragraph",
        text:
          "<i>Eis per vertrek: eis = max(oppervlak × dm³/(s·m²), " +
          "personen × 4,0 dm³/s, minimum) volgens BBL afd. 3.6. " +
          "Debieten zijn intern in dm³/s; m³/h is afgeleide weergave " +
          "(× 3,6).</i>",
      },
    ],
  };
}

// ---------------------------------------------------------------------------
// Sectie 2: Balans per vertrek
// ---------------------------------------------------------------------------

/**
 * Status-tekst per vertrek — zelfde semantiek als de `StatusBadge` op de
 * `/ventilation`-tab (✓ / tekort / natuurlijk / geen eis), maar dan als
 * plain-text voor de PDF-tabel.
 */
function statusLabel(
  hasDemand: boolean,
  mechanical: boolean,
  deficitDm3s: number,
): string {
  if (!hasDemand) return "geen eis";
  if (!mechanical) return "natuurlijk";
  if (deficitDm3s > 0) return `tekort ${formatDecimals(deficitDm3s, 1)} dm³/s`;
  return "✔ voldoet";
}

/** "Aanwezig"-cel: debiet + markering voor ventielen zonder debiet. */
function presentCell(
  hasDemand: boolean,
  isSupply: boolean,
  mechanical: boolean,
  presentDm3s: number,
  missingFlowCount: number,
): string {
  if (!hasDemand) return "—";
  if (!mechanical) return isSupply ? "via gevelroosters" : "natuurlijk";
  let cell = `${flowLabel(presentDm3s)} (${m3hLabel(presentDm3s)})`;
  if (missingFlowCount > 0) {
    cell += ` — ${missingFlowCount} ventiel${missingFlowCount > 1 ? "en" : ""} zonder debiet`;
  }
  return cell;
}

function buildRoomRow(
  room: VentilationReportRoom,
  vr: VentilationRoomState,
  row: RoomVentilationBalance,
  sys: VentilationSystemInfo,
): string[] {
  const isSupply = vr.requiredSupplyDm3s > 0;
  const isExhaust = vr.requiredExhaustDm3s > 0;
  const hasDemand = isSupply || isExhaust;
  const required = isSupply
    ? vr.requiredSupplyDm3s
    : isExhaust
      ? vr.requiredExhaustDm3s
      : 0;
  const present = isSupply ? row.presentSupplyDm3s : row.presentExhaustDm3s;
  const mechanical = isSupply ? sys.supplyMechanical : sys.exhaustMechanical;
  const deficit = isSupply ? row.supplyDeficitDm3s : row.exhaustDeficitDm3s;

  return [
    room.name,
    vr.ventilationFunction,
    `${formatArea(room.floor_area)} m²`,
    vr.occupancy !== undefined ? String(vr.occupancy) : "—",
    isSupply ? "toevoer" : isExhaust ? "afvoer" : "geen",
    hasDemand ? `${flowLabel(required)} (${m3hLabel(required)})` : "—",
    presentCell(hasDemand, isSupply, mechanical, present, row.missingFlowCount),
    statusLabel(hasDemand, mechanical, deficit),
  ];
}

function buildRoomBalanceSection(
  input: VentilationReportInput,
  balance: BuildingVentilationBalance,
  sys: VentilationSystemInfo,
): Record<string, unknown> {
  const rows: string[][] = [];
  for (const room of input.rooms) {
    const vr = input.ventilationRooms[room.id];
    const row = balance.rooms[room.id];
    if (!vr || !row) continue; // ruimte zonder afgeleide state (defensief)
    rows.push(buildRoomRow(room, vr, row, sys));
  }

  const content: Record<string, unknown>[] = [];
  if (rows.length === 0) {
    content.push({
      type: "paragraph",
      text: "Geen vertrekken in het project.",
    });
  } else {
    content.push({
      type: "table",
      title: "Eis vs. aanwezig per vertrek (BBL afd. 3.6)",
      headers: [
        "Vertrek",
        "Gebruiksfunctie (BBL)",
        "Opp.",
        "Pers.",
        "Type",
        "Eis",
        "Aanwezig",
        "Status",
      ],
      // Proportionele kolombreedtes (relatieve gewichten → renderer schaalt
      // naar beschikbare breedte, renderer_v2.py:1378). Voorkomt afgekapte
      // kolommen bij de brede per-vertrek-tabel.
      column_widths: [14, 16, 8, 6, 8, 16, 20, 12],
      rows,
    });
    content.push({
      type: "paragraph",
      text:
        "<i>Ventielen zonder ingevoerd debiet tellen als 0 dm³/s en zijn in " +
        "de kolom Aanwezig gemarkeerd. Status “natuurlijk”: deze " +
        "richting wordt bij het gekozen systeem niet op ventielen getoetst " +
        "(toevoer via gevelroosters resp. natuurlijke afvoer).</i>",
    });
  }

  return {
    title: "Balans per vertrek",
    level: 1,
    content,
  };
}

// ---------------------------------------------------------------------------
// Sectie 3: Gebouwbalans
// ---------------------------------------------------------------------------

function buildBuildingBalanceSection(
  balance: BuildingVentilationBalance,
): Record<string, unknown> {
  const sys = balance.system;

  const verdict = balance.balanced
    ? "<b>✔ In balans</b> — de toevoer- en afvoer-eis liggen binnen de tolerantie."
    : balance.imbalanceDm3s > 0
      ? `<b>Overdruk +${formatDecimals(balance.imbalanceDm3s, 1)} dm³/s</b> — de toevoer-eis is groter dan de afvoer-eis.`
      : `<b>Onderdruk ${formatDecimals(balance.imbalanceDm3s, 1)} dm³/s</b> — de afvoer-eis is groter dan de toevoer-eis.`;

  return {
    title: "Gebouwbalans",
    level: 1,
    content: [
      {
        type: "table",
        title: "Totalen (eis + aanwezig per richting)",
        headers: ["Grootheid", "Waarde"],
        rows: [
          [
            "Toevoer-eis",
            `${flowLabel(balance.totalRequiredSupplyDm3s)} (${m3hLabel(balance.totalRequiredSupplyDm3s)})`,
          ],
          [
            "Afvoer-eis",
            `${flowLabel(balance.totalRequiredExhaustDm3s)} (${m3hLabel(balance.totalRequiredExhaustDm3s)})`,
          ],
          [
            sys.supplyMechanical
              ? "Aanwezig toevoer"
              : "Aanwezig toevoer (gevelroosters)",
            `${flowLabel(balance.totalPresentSupplyDm3s)} (${m3hLabel(balance.totalPresentSupplyDm3s)})`,
          ],
          [
            sys.exhaustMechanical
              ? "Aanwezig afvoer"
              : "Aanwezig afvoer (natuurlijk)",
            `${flowLabel(balance.totalPresentExhaustDm3s)} (${m3hLabel(balance.totalPresentExhaustDm3s)})`,
          ],
        ],
      },
      { type: "spacer", height_mm: 4 },
      {
        type: "calculation",
        title: "Balans eis (toevoer − afvoer)",
        result: formatDecimals(balance.imbalanceDm3s, 1),
        unit: "dm³/s",
        reference: `tolerantie ±${formatDecimals(BALANCE_TOLERANCE_DM3S, 1)} dm³/s`,
      },
      {
        type: "paragraph",
        text: verdict,
      },
    ],
  };
}
