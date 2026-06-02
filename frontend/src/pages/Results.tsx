import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

import { ConstructionLossChart } from "../components/charts/ConstructionLossChart";
import { StackedBarChart } from "../components/charts/StackedBarChart";
import { SummaryDonut } from "../components/charts/SummaryDonut";
import { ChartZoomModal } from "../components/ui/ChartZoomModal";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Table, Th, Td } from "../components/ui/Table";
import { PageHeader } from "../components/layout/PageHeader";
import { useProjectStore } from "../store/projectStore";
import { useToastStore } from "../store/toastStore";
import { exportProject } from "../lib/importExport";
import { buildReportData } from "../lib/reportBuilder";
import { buildIsso53Report } from "../lib/isso53ReportBuilder";
import { bblMinimumVentilationRate } from "../lib/roomDefaults";
import { HDD_NL, computeAnnualHeatDemandKWh } from "../lib/annualEnergy";
import type { Project, ProjectResult } from "../types";
import type { Isso53ProjectResult } from "../types/isso53Result";
import type { Isso53RoomState } from "../types/projectV2";
import {
  isso53DonutSegments,
  isso53DonutSummary,
  isso53StackedBarRooms,
} from "../lib/isso53ChartData";
import { generateReportDirect } from "../lib/reportClient";

/** Format a number as W (watts) with locale formatting. */
function fmtW(value: number): string {
  return `${Math.round(value).toLocaleString("nl-NL")} W`;
}

/** Format a number with 2 decimals. */
function fmt2(value: number): string {
  return value.toLocaleString("nl-NL", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

export function Results() {
  const navigate = useNavigate();
  const { project, result } = useProjectStore();
  const norm = useProjectStore((s) => s.norm);
  const isso53Building = useProjectStore((s) => s.isso53Building);
  const isso53Rooms = useProjectStore((s) => s.isso53Rooms);
  const addToast = useToastStore((s) => s.addToast);
  const [isGenerating, setIsGenerating] = useState(false);
  const [zoomedChart, setZoomedChart] = useState<"bar" | "donut" | "construction" | null>(null);

  const handleExport = useCallback(() => {
    exportProject(project, result as ProjectResult | null);
  }, [project, result]);

  const handleGenerateReport = useCallback(async () => {
    if (!result) return;
    setIsGenerating(true);
    try {
      // Norm-routing — ISSO 53 gebruikt een eigen builder; backend is
      // norm-onafhankelijk en accepteert beide JSON-shapes.
      const reportData =
        norm === "isso53"
          ? await buildIsso53Report(
              project,
              result as unknown as Isso53ProjectResult,
              isso53Building,
              isso53Rooms,
            )
          : await buildReportData(project, result as ProjectResult);
      const blob = await generateReportDirect(reportData);

      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${project.info.name || "rapport"}.pdf`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      addToast("Rapport gegenereerd", "success");
    } catch (err) {
      const message = err instanceof Error ? err.message : "Onbekende fout";
      addToast(`Rapport mislukt: ${message}`, "error", 5000);
    } finally {
      setIsGenerating(false);
    }
  }, [project, result, norm, isso53Building, isso53Rooms, addToast]);

  if (!result) {
    return (
      <div>
        <PageHeader
          title="Resultaten"
          actions={
            <Button variant="secondary" onClick={() => navigate("/project")}>
              Terug
            </Button>
          }
        />
        <div className="p-6">
          <Card>
            <p className="text-center text-sm text-on-surface-muted">
              Nog geen berekening uitgevoerd. Ga naar Project en klik op Berekenen.
            </p>
          </Card>
        </div>
      </div>
    );
  }

  // Fase 4: norm-aware splitsing. Bij ISSO 53 rendert een eigen tabel +
  // gebouw-samenvatting op basis van het echte Isso53ProjectResult-shape
  // (zonder isso51-charts). Het ISSO 51-pad hieronder blijft ongewijzigd.
  if (norm === "isso53") {
    return (
      <Isso53Results
        result={result as Isso53ProjectResult}
        project={project}
        isso53Rooms={isso53Rooms}
        isGenerating={isGenerating}
        onGenerateReport={handleGenerateReport}
        onExport={handleExport}
        onBack={() => navigate("/project")}
      />
    );
  }

  // ISSO 51-pad: result is hier gegarandeerd het ProjectResult-shape.
  const { summary, rooms } = result as ProjectResult;

  // Gebouwtotalen — onafhankelijk van norm (geldt voor zowel ISSO 51 als 53).
  // q_v effectief = som van per-kamer q_v (of BBL-fallback als q_v leeg is).
  // Schil-oppervlak = som van constructie-areas met boundary_type
  // exterior/ground/water (= de daadwerkelijke gebouwschil).
  const { totalQv, totalEnvelopeArea } = useMemo(() => {
    let qvSum = 0;
    let areaSum = 0;
    for (const room of project.rooms) {
      qvSum +=
        room.ventilation_rate ??
        bblMinimumVentilationRate(room.function, room.floor_area);
      for (const ce of room.constructions) {
        if (
          ce.boundary_type === "exterior" ||
          ce.boundary_type === "ground" ||
          ce.boundary_type === "water"
        ) {
          areaSum += ce.area;
        }
      }
    }
    return { totalQv: qvSum, totalEnvelopeArea: areaSum };
  }, [project.rooms]);

  // Jaarverbruik warmtebehoefte (graaddagen-schatting, geen norm-conform BENG/NTA 8800).
  const { hExternal: hExternalAnnual, annualKWh: annualHeatDemandKWh } = useMemo(
    () => computeAnnualHeatDemandKWh(rooms),
    [rooms],
  );

  return (
    <div>
      <PageHeader
        title="Resultaten"
        subtitle={`${rooms.length} vertrekken`}
        actions={
          <div className="flex gap-2">
            <Button
              variant="primary"
              onClick={handleGenerateReport}
              disabled={isGenerating}
            >
              {isGenerating ? "Genereren..." : "Genereer rapport"}
            </Button>
            <Button variant="ghost" onClick={handleExport}>
              Export JSON
            </Button>
            <Button variant="secondary" onClick={() => navigate("/project")}>
              Terug naar project
            </Button>
          </div>
        }
      />

      <div className="space-y-6 p-6">
        {/* Summary metric cards */}
        <div className="grid grid-cols-4 gap-4">
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.connection_capacity)}</div>
            <div className="metric-card-label">Aansluitvermogen</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">Totaal benodigd vermogen van de warmteopwekker</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_envelope_loss)}</div>
            <div className="metric-card-label">Transmissie (schil)</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">Warmteverlies door wanden, dak, vloer en ramen</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_ventilation_loss)}</div>
            <div className="metric-card-label">Ventilatie</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">Warmteverlies door mechanische ventilatie</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.collective_contribution)}</div>
            <div className="metric-card-label">Collectief</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">Bijdrage van collectieve verwarmingsvoorzieningen</div>
          </div>
        </div>

        {/* Additional summary */}
        <div className="grid grid-cols-3 gap-4">
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_neighbor_loss)}</div>
            <div className="metric-card-label">Buurwoningverlies</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">Warmteverlies naar aangrenzende woningen</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_heating_up)}</div>
            <div className="metric-card-label">Opwarmtoeslag</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">Extra vermogen om op te warmen na nachtsetback</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_system_losses)}</div>
            <div className="metric-card-label">Systeemverliezen</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">Verliezen in leidingen en afgiftesysteem</div>
          </div>
        </div>

        {/* Charts */}
        <div className="grid grid-cols-2 gap-6">
          <Card title="Verliezen per vertrek">
            <div
              className="cursor-pointer"
              onClick={() => setZoomedChart("bar")}
              title="Klik om te vergroten"
            >
              <StackedBarChart rooms={rooms} />
            </div>
            <p className="mt-1 text-center text-[10px] text-on-surface-muted">
              Klik om te vergroten
            </p>
          </Card>
          <Card title="Gebouwtotaal">
            <div
              className="cursor-pointer"
              onClick={() => setZoomedChart("donut")}
              title="Klik om te vergroten"
            >
              <SummaryDonut summary={summary} />
            </div>
            <p className="mt-1 text-center text-[10px] text-on-surface-muted">
              Klik om te vergroten
            </p>
            <div className="mt-4 grid grid-cols-2 gap-3 border-t border-[var(--oaec-border-subtle)] pt-3">
              <div>
                <div className="text-[11px] uppercase tracking-wider text-on-surface-muted">
                  Ventilatiedebiet (gebouw)
                </div>
                <div className="text-sm font-semibold tabular-nums text-on-surface">
                  {totalQv.toFixed(1)} dm³/s
                </div>
                <div className="text-[10px] text-on-surface-muted tabular-nums">
                  {(totalQv * 3.6).toFixed(0)} m³/h
                </div>
              </div>
              <div>
                <div className="text-[11px] uppercase tracking-wider text-on-surface-muted">
                  Oppervlak gebouwschil
                </div>
                <div className="text-sm font-semibold tabular-nums text-on-surface">
                  {totalEnvelopeArea.toFixed(1)} m²
                </div>
                <div className="text-[10px] text-on-surface-muted">
                  exterior + ground + water
                </div>
              </div>
            </div>
            <div className="mt-3 border-t border-[var(--oaec-border-subtle)] pt-3">
              <div className="text-[11px] uppercase tracking-wider text-on-surface-muted">
                Jaarverbruik warmtebehoefte (graaddagen)*
              </div>
              <div className="text-sm font-semibold tabular-nums text-on-surface">
                {Math.round(annualHeatDemandKWh).toLocaleString("nl-NL")} kWh/jaar
              </div>
              <div className="text-[10px] text-on-surface-muted tabular-nums">
                H = {Math.round(hExternalAnnual)} W/K · HDD {HDD_NL} K·d
              </div>
            </div>
            <p className="mt-3 text-[10px] leading-tight text-on-surface-muted">
              *Schatting via graaddagen-methode (HDD {HDD_NL} K·d NL-gemiddelde). Niet
              norm-conform BENG/NTA 8800 — werkelijk verbruik wijkt af door zoninstraling,
              interne warmte en gebruikersgedrag.
            </p>
          </Card>
        </div>

        {/* Chart zoom modals */}
        <ChartZoomModal
          open={zoomedChart === "bar"}
          onClose={() => setZoomedChart(null)}
          title="Verliezen per vertrek"
        >
          <StackedBarChart rooms={rooms} />
        </ChartZoomModal>
        <ChartZoomModal
          open={zoomedChart === "donut"}
          onClose={() => setZoomedChart(null)}
          title="Gebouwtotaal"
        >
          <SummaryDonut summary={summary} />
        </ChartZoomModal>

        {/* Construction type loss chart */}
        <Card title="Verlies per constructietype">
          <div
            className="mx-auto max-w-2xl cursor-pointer"
            onClick={() => setZoomedChart("construction")}
            title="Klik om te vergroten"
          >
            <ConstructionLossChart
              rooms={project.rooms}
              thetaE={project.climate.theta_e ?? -10}
              thetaWater={project.climate.theta_water}
            />
          </div>
          <p className="mt-1 text-center text-[10px] text-on-surface-muted">
            Klik om te vergroten
          </p>
        </Card>
        <ChartZoomModal
          open={zoomedChart === "construction"}
          onClose={() => setZoomedChart(null)}
          title="Verlies per constructietype"
        >
          <ConstructionLossChart
            rooms={project.rooms}
            thetaE={project.climate.theta_e ?? -10}
          />
        </ChartZoomModal>

        {/* Room results table */}
        <Card title="Resultaten per vertrek">
          <Table>
            <thead>
              <tr>
                <Th>
                  Vertrek
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Ruimtenaam</span>
                </Th>
                <Th className="text-right">
                  &theta;_i
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Binnentemp.</span>
                </Th>
                <Th className="text-right">
                  &Phi;_T
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Transmissie</span>
                </Th>
                <Th className="text-right">
                  &Phi;_i
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Infiltratie</span>
                </Th>
                <Th className="text-right">
                  &Phi;_v
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Ventilatie</span>
                </Th>
                <Th className="text-right">
                  &Phi;_hu
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Opwarmtoeslag</span>
                </Th>
                <Th className="text-right">
                  &Phi;_sys
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Systeemverl.</span>
                </Th>
                <Th className="text-right">
                  &Phi;_basis
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Trans.+vent.+inf.</span>
                </Th>
                <Th className="text-right">
                  &Phi;_extra
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Opwarm+systeem</span>
                </Th>
                <Th className="text-right font-bold">
                  &Phi;_totaal
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">Totaal verlies</span>
                </Th>
              </tr>
            </thead>
            <tbody>
              {rooms.map((room) => (
                <tr key={room.room_id} className="hover:bg-[var(--oaec-hover)]">
                  <Td className="font-medium">{room.room_name}</Td>
                  <Td numeric>{fmt2(room.theta_i)} &deg;C</Td>
                  <Td numeric>{fmtW(room.transmission.phi_t)}</Td>
                  <Td numeric>{fmtW(room.infiltration.phi_i)}</Td>
                  <Td numeric>{fmtW(room.ventilation.phi_v)}</Td>
                  <Td numeric>{fmtW(room.heating_up.phi_hu)}</Td>
                  <Td numeric>{fmtW(room.system_losses.phi_system_total)}</Td>
                  <Td numeric>{fmtW(room.basis_heat_loss)}</Td>
                  <Td numeric>{fmtW(room.extra_heat_loss)}</Td>
                  <Td numeric className="font-bold">{fmtW(room.total_heat_loss)}</Td>
                </tr>
              ))}
            </tbody>
          </Table>
        </Card>

        {/* Per-room detail cards */}
        {rooms.map((room) => (
          <Card key={room.room_id} title={`${room.room_name} (${room.room_id})`}>
            <div className="grid grid-cols-2 gap-6">
              {/* Transmission */}
              <div>
                <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                  Transmissie
                </h4>
                <dl className="space-y-1 text-sm">
                  <DetailRow label="H_T,ie (schil)" value={`${fmt2(room.transmission.h_t_exterior)} W/K`} description="Warmtegeleiding naar buitenlucht" />
                  <DetailRow label="H_T,ia (intern)" value={`${fmt2(room.transmission.h_t_adjacent_rooms)} W/K`} description="Warmtegeleiding naar verwarmde buurruimten" />
                  <DetailRow label="H_T,io (onverwarmd)" value={`${fmt2(room.transmission.h_t_unheated)} W/K`} description="Warmtegeleiding naar onverwarmde ruimten" />
                  <DetailRow label="H_T,ib (buurwoning)" value={`${fmt2(room.transmission.h_t_adjacent_buildings)} W/K`} description="Warmtegeleiding naar aangrenzende woningen" />
                  <DetailRow label="H_T,ig (grond)" value={`${fmt2(room.transmission.h_t_ground)} W/K`} description="Warmtegeleiding naar de grond" />
                  <DetailRow label={<strong>&Phi;_T totaal</strong>} value={<strong>{fmtW(room.transmission.phi_t)}</strong>} description="Totaal transmissieverlies van dit vertrek" />
                </dl>
              </div>

              {/* Ventilation & infiltration */}
              <div>
                <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                  Ventilatie &amp; infiltratie
                </h4>
                <dl className="space-y-1 text-sm">
                  <DetailRow label="q_v" value={`${fmt2(room.ventilation.q_v)} dm³/s`} description="Ventilatieluchtstroom" />
                  <DetailRow label="H_v" value={`${fmt2(room.ventilation.h_v)} W/K`} description="Warmteoverdrachtscoëfficiënt ventilatie" />
                  <DetailRow label="f_v" value={fmt2(room.ventilation.f_v)} description="Verwarmingsfactor ventilatie" />
                  <DetailRow label={<strong>&Phi;_v</strong>} value={<strong>{fmtW(room.ventilation.phi_v)}</strong>} description="Totaal ventilatieverlies" />
                  <DetailRow label="H_i" value={`${fmt2(room.infiltration.h_i)} W/K`} description="Warmteoverdrachtscoëfficiënt infiltratie" />
                  <DetailRow label="&Phi;_i" value={fmtW(room.infiltration.phi_i)} description="Warmteverlies door luchtlekkage" />
                </dl>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}

function DetailRow({ label, value, description }: { label: React.ReactNode; value: React.ReactNode; description?: string }) {
  return (
    <div className="flex items-baseline justify-between gap-4">
      <dt className="text-on-surface-secondary">
        {label}
        {description && <span className="block text-[10px] leading-tight text-on-surface-muted">{description}</span>}
      </dt>
      <dd className="shrink-0 font-mono text-on-surface">{value}</dd>
    </div>
  );
}

/** ISSO 53 resultatenweergave — per-vertrek tabel + gebouw-samenvatting.
 *
 * Spiegelt de ISSO 51-presentatie: 3 diagrammen (verliezen per vertrek,
 * gebouwtotaal per type, verlies per constructietype) + per-vertrek
 * transmissie/ventilatie-breakdowns. Chart-data wordt afgeleid via
 * `lib/isso53ChartData.ts` zodat de bestaande SVG-componenten herbruikt
 * worden zonder ze te herschrijven.
 *
 * ISSO 51-specifieke teksten (graaddagen-jaarverbruik, θ_w-water) zijn
 * bewust weggelaten — die zijn niet fysisch overdraagbaar naar ISSO 53.
 */
function Isso53Results({
  result,
  project,
  isso53Rooms,
  isGenerating,
  onGenerateReport,
  onExport,
  onBack,
}: {
  result: Isso53ProjectResult;
  project: Project;
  isso53Rooms: Record<string, Isso53RoomState>;
  isGenerating: boolean;
  onGenerateReport: () => void;
  onExport: () => void;
  onBack: () => void;
}) {
  const { t } = useTranslation();
  const { rooms, summary } = result;
  const maatgevend = Math.max(
    summary.connectionCapacityIndividual,
    summary.connectionCapacityCollective,
  );

  const [zoomedChart, setZoomedChart] = useState<
    "bar" | "donut" | "construction" | null
  >(null);

  // Chart-data afgeleid uit het ISSO 53-result (zie lib/isso53ChartData.ts).
  const barRooms = useMemo(() => isso53StackedBarRooms(result), [result]);
  const donutSummary = useMemo(() => isso53DonutSummary(summary), [summary]);
  const donutSegments = useMemo(() => isso53DonutSegments(summary), [summary]);

  return (
    <div>
      <PageHeader
        title="Resultaten"
        subtitle={`ISSO 53 · ${rooms.length} vertrekken`}
        actions={
          <div className="flex gap-2">
            <Button variant="primary" onClick={onGenerateReport} disabled={isGenerating}>
              {isGenerating ? "Genereren..." : "Genereer rapport"}
            </Button>
            <Button variant="ghost" onClick={onExport}>
              Export JSON
            </Button>
            <Button variant="secondary" onClick={onBack}>
              Terug naar project
            </Button>
          </div>
        }
      />

      <div className="space-y-6 p-6">
        {/* Gebouw-samenvatting metric cards */}
        <div className="grid grid-cols-4 gap-4">
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(maatgevend)}</div>
            <div className="metric-card-label">{t("isso53.results.connectionCapacity")}</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">
              {t("isso53.results.connectionCapacityDescription")}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.totalBuildingHeatLoss)}</div>
            <div className="metric-card-label">{t("isso53.results.totalBuildingHeatLoss")}</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">
              {t("isso53.results.totalBuildingHeatLossDescription")}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.totalTransmissionLoss)}</div>
            <div className="metric-card-label">{t("isso53.results.totalTransmissionLoss")}</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">
              {t("isso53.results.totalTransmissionLossDescription")}
            </div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmt2(summary.infiltrationReductionFactorZ)}</div>
            <div className="metric-card-label">{t("isso53.results.infiltrationReductionFactorZ")}</div>
            <div className="mt-0.5 text-[10px] leading-tight text-on-surface-muted">
              {t("isso53.results.infiltrationReductionFactorZDescription")}
            </div>
          </div>
        </div>

        <div className="grid grid-cols-4 gap-4">
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.totalVentilationLoss)}</div>
            <div className="metric-card-label">{t("isso53.results.totalVentilationLoss")}</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.totalInfiltrationLoss)}</div>
            <div className="metric-card-label">{t("isso53.results.totalInfiltrationLoss")}</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.totalHeatingUp)}</div>
            <div className="metric-card-label">{t("isso53.results.totalHeatingUp")}</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.totalSystemLosses)}</div>
            <div className="metric-card-label">{t("isso53.results.totalSystemLosses")}</div>
          </div>
        </div>

        {/* Aansluitvermogen detail */}
        <Card title={t("isso53.results.connectionTitle")}>
          <dl className="grid grid-cols-3 gap-6 text-sm">
            <DetailRow
              label={t("isso53.results.connectionIndividual")}
              value={fmtW(summary.connectionCapacityIndividual)}
            />
            <DetailRow
              label={t("isso53.results.connectionCollective")}
              value={fmtW(summary.connectionCapacityCollective)}
            />
            <DetailRow
              label={t("isso53.results.shellHeatLoss")}
              value={fmtW(summary.shellHeatLoss)}
            />
          </dl>
        </Card>

        {/* Diagrammen — gespiegeld van het ISSO 51-scherm. */}
        <div className="grid grid-cols-2 gap-6">
          <Card title="Verliezen per vertrek">
            <div
              className="cursor-pointer"
              onClick={() => setZoomedChart("bar")}
              title="Klik om te vergroten"
            >
              <StackedBarChart rooms={barRooms} />
            </div>
            <p className="mt-1 text-center text-[10px] text-on-surface-muted">
              Klik om te vergroten
            </p>
          </Card>
          <Card title="Gebouwtotaal">
            <div
              className="cursor-pointer"
              onClick={() => setZoomedChart("donut")}
              title="Klik om te vergroten"
            >
              <SummaryDonut summary={donutSummary} segments={donutSegments} />
            </div>
            <p className="mt-1 text-center text-[10px] text-on-surface-muted">
              Klik om te vergroten
            </p>
            <p className="mt-3 text-[10px] leading-tight text-on-surface-muted">
              ISSO 53 splitst infiltratie áf van ventilatie; beide worden los
              getoond. Het centrum toont het maatgevende aansluitvermogen
              (max. individueel/collectief).
            </p>
          </Card>
        </div>

        {/* Chart zoom modals */}
        <ChartZoomModal
          open={zoomedChart === "bar"}
          onClose={() => setZoomedChart(null)}
          title="Verliezen per vertrek"
        >
          <StackedBarChart rooms={barRooms} />
        </ChartZoomModal>
        <ChartZoomModal
          open={zoomedChart === "donut"}
          onClose={() => setZoomedChart(null)}
          title="Gebouwtotaal"
        >
          <SummaryDonut summary={donutSummary} segments={donutSegments} />
        </ChartZoomModal>

        {/* Verlies per constructietype — ISSO 53-modus: ruimtetemperaturen
            uit de sidecar-ruimteType (tabel 2.2), onverwarmd als eigen
            categorie met f_k=0,5 default. Zie deltaT.ts / isso53Temperature.ts. */}
        <Card title="Verlies per constructietype">
          <div
            className="mx-auto max-w-2xl cursor-pointer"
            onClick={() => setZoomedChart("construction")}
            title="Klik om te vergroten"
          >
            <ConstructionLossChart
              rooms={project.rooms}
              thetaE={project.climate.theta_e ?? -10}
              thetaWater={project.climate.theta_water}
              norm="isso53"
              isso53Rooms={isso53Rooms}
            />
          </div>
          <p className="mt-1 text-center text-[10px] text-on-surface-muted">
            Klik om te vergroten
          </p>
        </Card>
        <ChartZoomModal
          open={zoomedChart === "construction"}
          onClose={() => setZoomedChart(null)}
          title="Verlies per constructietype"
        >
          <ConstructionLossChart
            rooms={project.rooms}
            thetaE={project.climate.theta_e ?? -10}
            thetaWater={project.climate.theta_water}
            norm="isso53"
            isso53Rooms={isso53Rooms}
          />
        </ChartZoomModal>

        {/* Per-vertrek tabel */}
        <Card title={t("isso53.results.roomTableTitle")}>
          <Table>
            <thead>
              <tr>
                <Th>
                  {t("isso53.results.colRoom")}
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colRoomDescription")}
                  </span>
                </Th>
                <Th className="text-right">
                  &theta;_i
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colThetaI")}
                  </span>
                </Th>
                <Th className="text-right">
                  &Phi;_T
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colPhiT")}
                  </span>
                </Th>
                <Th className="text-right">
                  &Phi;_v
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colPhiV")}
                  </span>
                </Th>
                <Th className="text-right">
                  &Phi;_i
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colPhiI")}
                  </span>
                </Th>
                <Th className="text-right">
                  &Phi;_hu
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colPhiHu")}
                  </span>
                </Th>
                <Th className="text-right">
                  &Phi;_sys
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colPhiSystem")}
                  </span>
                </Th>
                <Th className="text-right font-bold">
                  &Phi;_HL
                  <span className="block text-[10px] font-normal normal-case tracking-normal text-on-surface-muted">
                    {t("isso53.results.colTotal")}
                  </span>
                </Th>
              </tr>
            </thead>
            <tbody>
              {rooms.map((room) => (
                <tr key={room.roomId} className="hover:bg-[var(--oaec-hover)]">
                  <Td className="font-medium">{room.roomName}</Td>
                  <Td numeric>{fmt2(room.thetaI)} &deg;C</Td>
                  <Td numeric>{fmtW(room.phiT)}</Td>
                  <Td numeric>{fmtW(room.phiV)}</Td>
                  <Td numeric>{fmtW(room.phiI)}</Td>
                  <Td numeric>{fmtW(room.phiHu)}</Td>
                  <Td numeric>{fmtW(room.phiSystem)}</Td>
                  <Td numeric className="font-bold">{fmtW(room.totalHeatLoss)}</Td>
                </tr>
              ))}
            </tbody>
          </Table>
        </Card>

        {/* Per-vertrek detail — transmissie + ventilatie/infiltratie
            breakdown. ISSO 53 levert geen f_v/q_v per vertrek; die rijen
            (anders dan in de ISSO 51-tak) zijn daarom weggelaten. */}
        {rooms.map((room) => (
          <Card key={room.roomId} title={`${room.roomName} (${room.roomId})`}>
            <div className="grid grid-cols-2 gap-6">
              {/* Transmissie */}
              <div>
                <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                  Transmissie
                </h4>
                <dl className="space-y-1 text-sm">
                  <DetailRow label="H_T,ie (schil)" value={`${fmt2(room.hTExterior)} W/K`} description="Warmtegeleiding naar buitenlucht" />
                  <DetailRow label="H_T,ia (intern)" value={`${fmt2(room.hTAdjacentRooms)} W/K`} description="Warmtegeleiding naar verwarmde buurruimten" />
                  <DetailRow label="H_T,iae (onverwarmd)" value={`${fmt2(room.hTUnheated)} W/K`} description="Warmtegeleiding naar onverwarmde ruimten" />
                  <DetailRow label="H_T,iaBE (buurwoning)" value={`${fmt2(room.hTAdjacentBuildings)} W/K`} description="Warmtegeleiding naar aangrenzende gebouwen" />
                  <DetailRow label="H_T,ig (grond)" value={`${fmt2(room.hTGround)} W/K`} description="Warmtegeleiding naar de grond" />
                  <DetailRow label={<strong>&Phi;_T totaal</strong>} value={<strong>{fmtW(room.phiT)}</strong>} description="Totaal transmissieverlies van dit vertrek" />
                </dl>
              </div>

              {/* Ventilatie & infiltratie */}
              <div>
                <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                  Ventilatie &amp; infiltratie
                </h4>
                <dl className="space-y-1 text-sm">
                  <DetailRow label="H_v" value={`${fmt2(room.hV)} W/K`} description="Warmteoverdrachtscoëfficiënt ventilatie" />
                  <DetailRow label={<strong>&Phi;_v</strong>} value={<strong>{fmtW(room.phiV)}</strong>} description="Totaal ventilatieverlies" />
                  <DetailRow label="H_i" value={`${fmt2(room.hI)} W/K`} description="Warmteoverdrachtscoëfficiënt infiltratie" />
                  <DetailRow label="&Phi;_i" value={fmtW(room.phiI)} description="Warmteverlies door luchtlekkage" />
                </dl>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
