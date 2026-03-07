import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";

import { StackedBarChart } from "../components/charts/StackedBarChart";
import { SummaryDonut } from "../components/charts/SummaryDonut";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Table, Th, Td } from "../components/ui/Table";
import { PageHeader } from "../components/layout/PageHeader";
import { useProjectStore } from "../store/projectStore";
import { useToastStore } from "../store/toastStore";
import { exportProject } from "../lib/importExport";
import { buildReportData } from "../lib/reportBuilder";
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
  const addToast = useToastStore((s) => s.addToast);
  const [isGenerating, setIsGenerating] = useState(false);

  const handleExport = useCallback(() => {
    exportProject(project, result);
  }, [project, result]);

  const handleGenerateReport = useCallback(async () => {
    if (!result) return;
    setIsGenerating(true);
    try {
      const reportData = buildReportData(project, result);
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
  }, [project, result, addToast]);

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
            <p className="text-center text-sm text-stone-500">
              Nog geen berekening uitgevoerd. Ga naar Project en klik op Berekenen.
            </p>
          </Card>
        </div>
      </div>
    );
  }

  const { summary, rooms } = result;

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
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_envelope_loss)}</div>
            <div className="metric-card-label">Transmissie (schil)</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_ventilation_loss)}</div>
            <div className="metric-card-label">Ventilatie</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.collective_contribution)}</div>
            <div className="metric-card-label">Collectief</div>
          </div>
        </div>

        {/* Additional summary */}
        <div className="grid grid-cols-3 gap-4">
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_neighbor_loss)}</div>
            <div className="metric-card-label">Buurwoningverlies</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_heating_up)}</div>
            <div className="metric-card-label">Opwarmtoeslag</div>
          </div>
          <div className="metric-card">
            <div className="metric-card-value">{fmtW(summary.total_system_losses)}</div>
            <div className="metric-card-label">Systeemverliezen</div>
          </div>
        </div>

        {/* Charts */}
        <div className="grid grid-cols-2 gap-6">
          <Card title="Verliezen per vertrek">
            <StackedBarChart rooms={rooms} />
          </Card>
          <Card title="Gebouwtotaal">
            <SummaryDonut summary={summary} />
          </Card>
        </div>

        {/* Room results table */}
        <Card title="Resultaten per vertrek">
          <Table>
            <thead>
              <tr>
                <Th>Vertrek</Th>
                <Th className="text-right">&theta;_i</Th>
                <Th className="text-right">&Phi;_T</Th>
                <Th className="text-right">&Phi;_i</Th>
                <Th className="text-right">&Phi;_v</Th>
                <Th className="text-right">&Phi;_hu</Th>
                <Th className="text-right">&Phi;_sys</Th>
                <Th className="text-right">&Phi;_basis</Th>
                <Th className="text-right">&Phi;_extra</Th>
                <Th className="text-right font-bold">&Phi;_totaal</Th>
              </tr>
            </thead>
            <tbody>
              {rooms.map((room) => (
                <tr key={room.room_id} className="hover:bg-stone-50">
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
                <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-500">
                  Transmissie
                </h4>
                <dl className="space-y-1 text-sm">
                  <DetailRow label="H_T,ie (schil)" value={`${fmt2(room.transmission.h_t_exterior)} W/K`} />
                  <DetailRow label="H_T,ia (intern)" value={`${fmt2(room.transmission.h_t_adjacent_rooms)} W/K`} />
                  <DetailRow label="H_T,io (onverwarmd)" value={`${fmt2(room.transmission.h_t_unheated)} W/K`} />
                  <DetailRow label="H_T,ib (buurwoning)" value={`${fmt2(room.transmission.h_t_adjacent_buildings)} W/K`} />
                  <DetailRow label="H_T,ig (grond)" value={`${fmt2(room.transmission.h_t_ground)} W/K`} />
                  <DetailRow label={<strong>&Phi;_T totaal</strong>} value={<strong>{fmtW(room.transmission.phi_t)}</strong>} />
                </dl>
              </div>

              {/* Ventilation & infiltration */}
              <div>
                <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-500">
                  Ventilatie &amp; infiltratie
                </h4>
                <dl className="space-y-1 text-sm">
                  <DetailRow label="q_v" value={`${fmt2(room.ventilation.q_v)} dm\u00B3/s`} />
                  <DetailRow label="H_v" value={`${fmt2(room.ventilation.h_v)} W/K`} />
                  <DetailRow label="f_v" value={fmt2(room.ventilation.f_v)} />
                  <DetailRow label={<strong>&Phi;_v</strong>} value={<strong>{fmtW(room.ventilation.phi_v)}</strong>} />
                  <DetailRow label="H_i" value={`${fmt2(room.infiltration.h_i)} W/K`} />
                  <DetailRow label="&Phi;_i" value={fmtW(room.infiltration.phi_i)} />
                </dl>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}

function DetailRow({ label, value }: { label: React.ReactNode; value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between">
      <dt className="text-stone-600">{label}</dt>
      <dd className="font-mono text-stone-900">{value}</dd>
    </div>
  );
}
