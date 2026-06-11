/**
 * Ventilatiebalans — volwaardige tab (BBL afd. 3.6 / NEN 1087).
 *
 * Zelfde data en handlers als het Modeller-zijpaneel
 * (`components/modeller/VentilationBalancePanel.tsx`) via de gedeelde hook
 * `useVentilationBalance` — één bron van waarheid in `projectStore.ventilation`,
 * dus wijzigingen hier zijn direct zichtbaar in de Modeller en vice versa.
 *
 * Pagina-opbouw volgt het TO-juli-patroon (`pages/TojuliFull.tsx`):
 * PageHeader + Cards op een ruim canvas met volledige tabelbreedte.
 * Visuele referentie: `mockups/pages/ventilation-balance.html`.
 *
 * **Eenheden:** dm³/s intern; weergave omschakelbaar dm³/s ↔ m³/h via de
 * persistente toggle in de header (`FlowUnitToggle` / `ventilationUiStore`) —
 * conversie alleen aan de UI-rand, de store blijft dm³/s.
 */

import { Fragment, useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { useVentilationBalance } from "../hooks/useVentilationBalance";
import { generateReportDirect } from "../lib/reportClient";
import { buildVentilationReportData } from "../lib/ventilationReportBuilder";
import { useToastStore } from "../store/toastStore";
import {
  FLOW_UNIT_LABELS,
  ventilationSystemOf,
  type BblFunctionKey,
  type FlowDisplayUnit,
  type VentilationRoomState,
  type VentilationSystemInfo,
} from "../types/ventilation";
import {
  aggregateVentilationBalance,
  type RoomVentilationBalance,
} from "../lib/ventilationBalance";
import {
  BuildingBalanceSummary,
  FUNCTION_OPTIONS,
  FlowUnitToggle,
  IndicativeOccupancyBadge,
  StatusBadge,
  SystemSelector,
  UnitCapacitySummary,
  flowDisplayLabel,
  flowSecondaryLabel,
} from "../components/ventilation/shared";
import { UnitsCard } from "../components/ventilation/UnitsCard";
import { useVentilationUiStore } from "../components/ventilation/ventilationUiStore";
import {
  groupRoomsByZone,
  sumZoneBalance,
  type ZoneSubtotal,
} from "../components/ventilation/zoneGrouping";
import { formatArea } from "../lib/formatNumber";
import type { Room } from "../types";

export function VentilationBalance() {
  const {
    project,
    ventilation,
    ventilationRooms,
    changeFunction,
    changeOccupancy,
    setSystem,
    unitCapacity,
    assignCatalogUnit,
    addCustomUnit,
    updateUnit,
    removeUnit,
    setUnitAssignment,
  } = useVentilationBalance();

  const { t } = useTranslation();
  const addToast = useToastStore((s) => s.addToast);
  const [isGenerating, setIsGenerating] = useState(false);
  // Weergave-eenheid (persistent UI-voorkeur) — puur display; store blijft dm³/s.
  const flowUnit = useVentilationUiStore((s) => s.flowUnit);

  const balance = useMemo(
    () =>
      aggregateVentilationBalance(
        ventilationRooms,
        ventilation.terminals,
        ventilation.system,
      ),
    [ventilationRooms, ventilation.terminals, ventilation.system],
  );
  const sys = ventilationSystemOf(ventilation);

  // Zone-groepering — alleen actief wanneer het project zones heeft.
  // `null` = geen zones gedefinieerd → exact de bestaande platte weergave.
  const zones = project.building.zones;
  const zoneGroups = useMemo(
    () =>
      zones !== undefined && zones.length > 0
        ? groupRoomsByZone(project.rooms, zones)
        : null,
    [project.rooms, zones],
  );

  // Zelfstandig ventilatiebalans-rapport — zelfde UX-patroon als de
  // uw/rc-rapport-knoppen (UwCalculator.handleGenerateReport).
  const handleGenerateReport = useCallback(async () => {
    setIsGenerating(true);
    const reportName = t("ventilation.reportName");
    try {
      const reportData = buildVentilationReportData({
        info: project.info,
        rooms: project.rooms,
        ventilationRooms,
        terminals: ventilation.terminals,
        system: ventilation.system,
        units: ventilation.units,
        unitAssignments: ventilation.unitAssignments,
      });
      const blob = await generateReportDirect(reportData);

      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${reportName}.pdf`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      addToast(t("ventilation.reportSuccess"), "success");
    } catch (err) {
      const message =
        err instanceof Error ? err.message : t("ventilation.reportError");
      addToast(`${t("ventilation.reportFailed")}: ${message}`, "error", 5000);
    } finally {
      setIsGenerating(false);
    }
  }, [project, ventilationRooms, ventilation, addToast, t]);

  // Eén rij per vertrek — gedeeld tussen de platte en de zone-gegroepeerde
  // weergave (zelfde props, alleen de omringende structuur verschilt).
  const renderRoomRow = (room: Room) => {
    const vr = ventilationRooms[room.id];
    const row = balance.rooms[room.id];
    if (!vr || !row) return null;
    return (
      <RoomTableRow
        key={room.id}
        room={room}
        vr={vr}
        row={row}
        unit={flowUnit}
        supplyMechanical={sys.supplyMechanical}
        exhaustMechanical={sys.exhaustMechanical}
        onChangeFunction={(fn) => changeFunction(room.id, fn)}
        onChangeOccupancy={(n) => changeOccupancy(room.id, n)}
      />
    );
  };

  return (
    <div>
      <PageHeader
        title="Ventilatiebalans"
        subtitle="BBL afd. 3.6 — eis per vertrek + gebouwbalans"
        actions={
          <div className="flex items-center gap-2">
            <FlowUnitToggle />
            <Button
              variant="secondary"
              size="sm"
              onClick={handleGenerateReport}
              disabled={isGenerating || project.rooms.length === 0}
            >
              {isGenerating
                ? t("ventilation.generating")
                : t("ventilation.report")}
            </Button>
          </div>
        }
      />

      <div className="space-y-4 p-6">
        {/* Korte uitleg / legend */}
        <p className="max-w-3xl text-sm text-on-surface-muted">
          Eisen per gebruiksfunctie volgens Bbl art. 4.122 — woonfunctie:{" "}
          <code className="text-xs">
            eis = max(oppervlak × dm³/(s·m²), personen × 4,0 dm³/s, minimum)
          </code>
          ; utiliteitsfuncties (lid 2):{" "}
          <code className="text-xs">eis = personen × dm³/s p.p.</code> (zonder
          bezetting: indicatieve m²-benadering). Debieten zijn intern in dm³/s;
          m³/h is afgeleide weergave (× 3,6). Ventielen plaats je in de
          Modeller (2D → Ventilatie).
        </p>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
          {/* Systeem A–D */}
          <Card title="Ventilatiesysteem" className="lg:col-span-2">
            <div className="max-w-md">
              <SystemSelector value={sys.key} onChange={setSystem} />
            </div>
          </Card>

          {/* Gebouwbalans */}
          <Card title="Gebouwbalans">
            <BuildingBalanceSummary balance={balance} unit={flowUnit} />
            <UnitCapacitySummary check={unitCapacity} unit={flowUnit} />
          </Card>
        </div>

        {/* WTW/MV-units + capaciteitstoets — niet bij systeem A (natuurlijk) */}
        {sys.key !== "A" && (
          <Card title={t("ventilation.units.cardTitle")}>
            <UnitsCard
              ventilation={ventilation}
              unitCapacity={unitCapacity}
              unit={flowUnit}
              onAssignCatalogUnit={assignCatalogUnit}
              onAddCustomUnit={addCustomUnit}
              onUpdateUnit={updateUnit}
              onRemoveUnit={removeUnit}
              onSetAssignment={setUnitAssignment}
            />
          </Card>
        )}

        {/* Balans per vertrek */}
        <Card title="Balans per vertrek">
          {project.rooms.length === 0 ? (
            <p className="text-sm text-on-surface-muted">
              Geen vertrekken in het project. Voeg vertrekken toe via{" "}
              <span className="font-medium">Vertrekken</span> of de{" "}
              <span className="font-medium">Modeller</span>.
            </p>
          ) : (
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr className="border-b border-[var(--oaec-border)] text-left text-xs font-semibold text-scaffold-gray">
                  <th className="px-2 py-2">Vertrek</th>
                  <th className="px-2 py-2">Gebruiksfunctie (BBL)</th>
                  <th className="px-2 py-2 text-right">Opp. (m²)</th>
                  <th className="px-2 py-2 text-right">Personen</th>
                  <th className="px-2 py-2">Type</th>
                  <th className="px-2 py-2 text-right">Eis</th>
                  <th className="px-2 py-2 text-right">Aanwezig</th>
                  <th className="px-2 py-2">Status</th>
                </tr>
              </thead>
              <tbody>
                {zoneGroups === null ? (
                  project.rooms.map(renderRoomRow)
                ) : (
                  <>
                    {zoneGroups.map((group) => {
                      const name = group.zone?.name ?? "Niet ingedeeld";
                      const subtotal = sumZoneBalance(
                        group.rooms.map((r) => r.id),
                        balance.rooms,
                      );
                      return (
                        <Fragment key={group.zone?.id ?? "__unassigned__"}>
                          {/* Zone-kopregel */}
                          <tr className="border-b border-[var(--oaec-border)] bg-surface-alt">
                            <td
                              colSpan={8}
                              className="px-2 py-1.5 text-xs font-semibold uppercase tracking-wider text-on-surface-muted"
                            >
                              {name}
                              <span className="ml-2 font-normal normal-case tracking-normal text-scaffold-gray">
                                ({group.rooms.length} vertrek
                                {group.rooms.length === 1 ? "" : "ken"})
                              </span>
                            </td>
                          </tr>
                          {group.rooms.map(renderRoomRow)}
                          <SubtotalRow
                            label={`Subtotaal ${name}`}
                            subtotal={subtotal}
                            unit={flowUnit}
                            sys={sys}
                          />
                        </Fragment>
                      );
                    })}
                    {/* Gebouwtotaal onder de zone-subtotalen */}
                    <SubtotalRow
                      label="Totaal gebouw"
                      subtotal={{
                        requiredSupplyDm3s: balance.totalRequiredSupplyDm3s,
                        requiredExhaustDm3s: balance.totalRequiredExhaustDm3s,
                        presentSupplyDm3s: balance.totalPresentSupplyDm3s,
                        presentExhaustDm3s: balance.totalPresentExhaustDm3s,
                      }}
                      unit={flowUnit}
                      sys={sys}
                      emphasized
                    />
                  </>
                )}
              </tbody>
            </table>
          )}
        </Card>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Zone-subtotaal / gebouwtotaal-rij (alleen bij zone-groepering)
// ---------------------------------------------------------------------------

/**
 * Toevoer + afvoer gestapeld in één cel (eis- of aanwezig-kolom). Een
 * natuurlijke kant van het systeem toont — net als de vertrek-rijen — geen
 * ventiel-som maar een gedimde "via gevelroosters"/"natuurlijk"-tekst.
 */
function SupplyExhaustCell({
  supplyDm3s,
  exhaustDm3s,
  unit,
  supplyNatural = false,
  exhaustNatural = false,
}: {
  supplyDm3s: number;
  exhaustDm3s: number;
  /** Weergave-eenheid voor debieten (store blijft dm³/s). */
  unit: FlowDisplayUnit;
  supplyNatural?: boolean;
  exhaustNatural?: boolean;
}) {
  return (
    <div className="leading-snug">
      <div>
        <span className="text-[10px] text-scaffold-gray">toevoer </span>
        {supplyNatural ? (
          <span className="text-xs text-scaffold-gray">via gevelroosters</span>
        ) : (
          <span className="font-medium text-on-surface">
            {flowDisplayLabel(supplyDm3s, unit)}
          </span>
        )}
      </div>
      <div>
        <span className="text-[10px] text-scaffold-gray">afvoer </span>
        {exhaustNatural ? (
          <span className="text-xs text-scaffold-gray">natuurlijk</span>
        ) : (
          <span className="font-medium text-on-surface">
            {flowDisplayLabel(exhaustDm3s, unit)}
          </span>
        )}
      </div>
    </div>
  );
}

/**
 * Subtotaal-rij per zone (of het gebouwtotaal, `emphasized`). Toont de
 * eis- en aanwezig-sommen per richting in de Eis-/Aanwezig-kolommen;
 * respecteert de eenheden-toggle via `unit`.
 */
function SubtotalRow({
  label,
  subtotal,
  unit,
  sys,
  emphasized = false,
}: {
  label: string;
  subtotal: ZoneSubtotal;
  /** Weergave-eenheid voor debieten (store blijft dm³/s). */
  unit: FlowDisplayUnit;
  sys: VentilationSystemInfo;
  emphasized?: boolean;
}) {
  return (
    <tr
      className={`border-b-2 border-[var(--oaec-border)] ${
        emphasized ? "bg-primary/5" : ""
      }`}
    >
      <td
        colSpan={5}
        className={`px-2 py-1.5 text-right text-xs text-on-surface ${
          emphasized ? "font-bold" : "font-semibold"
        }`}
      >
        {label}
      </td>
      <td className="px-2 py-1.5 text-right text-xs tabular-nums">
        <SupplyExhaustCell
          supplyDm3s={subtotal.requiredSupplyDm3s}
          exhaustDm3s={subtotal.requiredExhaustDm3s}
          unit={unit}
        />
      </td>
      <td className="px-2 py-1.5 text-right text-xs tabular-nums">
        <SupplyExhaustCell
          supplyDm3s={subtotal.presentSupplyDm3s}
          exhaustDm3s={subtotal.presentExhaustDm3s}
          unit={unit}
          supplyNatural={!sys.supplyMechanical}
          exhaustNatural={!sys.exhaustMechanical}
        />
      </td>
      <td />
    </tr>
  );
}

// ---------------------------------------------------------------------------
// Tabelrij per vertrek
// ---------------------------------------------------------------------------

function RoomTableRow({
  room,
  vr,
  row,
  unit,
  supplyMechanical,
  exhaustMechanical,
  onChangeFunction,
  onChangeOccupancy,
}: {
  room: Room;
  vr: VentilationRoomState;
  row: RoomVentilationBalance;
  /** Weergave-eenheid voor debieten (store blijft dm³/s). */
  unit: FlowDisplayUnit;
  supplyMechanical: boolean;
  exhaustMechanical: boolean;
  onChangeFunction: (fn: BblFunctionKey) => void;
  onChangeOccupancy: (occupancy: number | undefined) => void;
}) {
  const isSupply = vr.requiredSupplyDm3s > 0;
  const isExhaust = vr.requiredExhaustDm3s > 0;
  const required = isSupply
    ? vr.requiredSupplyDm3s
    : isExhaust
      ? vr.requiredExhaustDm3s
      : 0;
  const present = isSupply ? row.presentSupplyDm3s : row.presentExhaustDm3s;
  const mechanical = isSupply ? supplyMechanical : exhaustMechanical;
  const deficit = isSupply ? row.supplyDeficitDm3s : row.exhaustDeficitDm3s;

  return (
    <tr className="border-b border-[var(--oaec-border-subtle)] hover:bg-primary/5">
      {/* Vertrek */}
      <td className="px-2 py-1.5 font-medium text-on-surface">{room.name}</td>

      {/* Gebruiksfunctie */}
      <td className="px-2 py-1.5">
        <select
          value={vr.ventilationFunction}
          onChange={(e) => onChangeFunction(e.target.value as BblFunctionKey)}
          className="w-full max-w-[14rem] rounded border border-primary/20 bg-surface px-1.5 py-1 text-xs text-on-surface"
          title="BBL-gebruiksfunctie (override)"
        >
          {FUNCTION_OPTIONS.map((fn) => (
            <option key={fn} value={fn}>
              {fn}
            </option>
          ))}
        </select>
      </td>

      {/* Oppervlak */}
      <td className="px-2 py-1.5 text-right tabular-nums text-on-surface">
        {formatArea(room.floor_area)}
      </td>

      {/* Personen */}
      <td className="px-2 py-1.5 text-right">
        <input
          type="number"
          min={0}
          step={1}
          value={vr.occupancy ?? ""}
          placeholder="–"
          onChange={(e) => {
            const v = e.target.value;
            if (v === "") {
              onChangeOccupancy(undefined);
              return;
            }
            const n = Number(v);
            onChangeOccupancy(
              Number.isFinite(n) && n > 0 ? Math.floor(n) : undefined,
            );
          }}
          className="w-16 rounded border border-primary/20 bg-surface px-1.5 py-1 text-right text-xs tabular-nums text-on-surface"
          title="Bezetting (personen-toeslag: max(opp×dm³/m², pers×pp, minimum))"
        />
      </td>

      {/* Type */}
      <td className="px-2 py-1.5">
        {isSupply ? (
          <span className="rounded-full bg-green-100 px-2 py-0.5 text-[10px] font-semibold text-green-700">
            toevoer
          </span>
        ) : isExhaust ? (
          <span className="rounded-full bg-blue-100 px-2 py-0.5 text-[10px] font-semibold text-blue-700">
            afvoer
          </span>
        ) : (
          <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-scaffold-gray">
            geen
          </span>
        )}
      </td>

      {/* Eis */}
      <td className="px-2 py-1.5 text-right tabular-nums">
        {isSupply || isExhaust ? (
          <>
            <span className="font-medium text-on-surface">
              {flowDisplayLabel(required, unit)}
            </span>{" "}
            <span className="text-xs text-scaffold-gray">
              ({flowSecondaryLabel(required, unit)})
            </span>
            <IndicativeOccupancyBadge
              fn={vr.ventilationFunction}
              occupancy={vr.occupancy}
            />
          </>
        ) : (
          <span className="text-scaffold-gray">—</span>
        )}
      </td>

      {/* Aanwezig */}
      <td className="px-2 py-1.5 text-right tabular-nums">
        {isSupply || isExhaust ? (
          mechanical ? (
            <>
              <span className="font-medium text-on-surface">
                {flowDisplayLabel(present, unit)}
              </span>{" "}
              <span className="text-xs text-scaffold-gray">
                ({flowSecondaryLabel(present, unit)})
              </span>
              {row.missingFlowCount > 0 && (
                <div
                  className="text-[10px] font-medium text-amber-600"
                  title={`Ventielen zonder debiet tellen als 0 ${FLOW_UNIT_LABELS[unit]}`}
                >
                  ⚠ {row.missingFlowCount} ventiel
                  {row.missingFlowCount > 1 ? "en" : ""} zonder debiet
                </div>
              )}
            </>
          ) : (
            <span className="text-xs text-scaffold-gray">
              {isSupply ? "via gevelroosters" : "natuurlijk"}
            </span>
          )
        ) : (
          <span className="text-scaffold-gray">—</span>
        )}
      </td>

      {/* Status */}
      <td className="px-2 py-1.5">
        <StatusBadge
          isSupply={isSupply}
          isExhaust={isExhaust}
          mechanical={mechanical}
          deficit={deficit}
          unit={unit}
        />
      </td>
    </tr>
  );
}
