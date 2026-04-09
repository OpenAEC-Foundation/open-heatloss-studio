/**
 * Step 5 — Import summary and final import action.
 *
 * Shows counts of rooms, constructions, openings, and any warnings.
 * Provides the final "Importeer" button.
 */
import {
  AlertTriangle,
  CheckCircle2,
  Home,
  Layers,
  DoorOpen,
  ArrowRight,
} from "lucide-react";

import type {
  ThermalImportFile,
  ThermalImportResult,
  ThermalRoom,
  ThermalOpening,
} from "../../lib/thermalImport";
import { isPseudoRoom } from "../../lib/thermalImport";

interface ImportSummaryProps {
  importFile: ThermalImportFile;
  importResult: ThermalImportResult;
  editedRooms: ThermalRoom[];
  editedOpenings: ThermalOpening[];
  onImport: () => void;
}

export function ImportSummary({
  importFile,
  importResult,
  editedRooms,
  editedOpenings,
  onImport,
}: ImportSummaryProps) {
  const realRooms = editedRooms.filter((r) => !isPseudoRoom(r));
  const heatedRooms = realRooms.filter((r) => r.type === "heated");
  const unheatedRooms = realRooms.filter((r) => r.type === "unheated");
  const constructionCount = importResult.construction_catalog.length;
  const openingCount = editedOpenings.length;
  const warnings = importResult.warnings;

  return (
    <div>
      <h2 className="mb-2 text-lg font-semibold text-gray-100">
        Samenvatting
      </h2>
      <p className="mb-6 text-sm text-gray-400">
        Controleer de samenvatting hieronder en klik op "Importeer project" om
        het project te laden.
      </p>

      {/* Project info */}
      <div className="mb-6 rounded-lg border border-gray-700 bg-gray-800/50 px-4 py-3">
        <h3 className="text-sm font-medium text-gray-200">
          {importFile.project_name ?? "Naamloos project"}
        </h3>
        <p className="mt-1 text-xs text-gray-500">
          Bron: {importFile.source} &middot; Geexporteerd:{" "}
          {new Date(importFile.exported_at).toLocaleString("nl-NL")}
        </p>
      </div>

      {/* Stats grid */}
      <div className="mb-6 grid grid-cols-3 gap-4">
        <StatCard
          icon={Home}
          label="Ruimtes"
          value={realRooms.length}
          detail={`${heatedRooms.length} verwarmd, ${unheatedRooms.length} onverwarmd`}
        />
        <StatCard
          icon={Layers}
          label="Constructie-typen"
          value={constructionCount}
          detail={`samengevoegd uit ${importFile.constructions.length} grensvlakken`}
        />
        <StatCard
          icon={DoorOpen}
          label="Openingen"
          value={openingCount}
          detail={`${editedOpenings.filter((o) => o.type === "window").length} ramen, ${
            editedOpenings.filter((o) => o.type === "door").length
          } deuren`}
        />
      </div>

      {/* Warnings */}
      {warnings.length > 0 && (
        <div className="mb-6">
          <h3 className="mb-2 flex items-center gap-2 text-sm font-medium text-amber-400">
            <AlertTriangle className="h-4 w-4" />
            Waarschuwingen ({warnings.length})
          </h3>
          <ul className="space-y-1">
            {warnings.map((w, i) => (
              <li
                key={i}
                className="flex items-start gap-2 rounded border border-amber-500/20 bg-amber-500/5 px-3 py-2 text-xs text-amber-300"
              >
                <span className="mt-0.5 text-amber-500">&bull;</span>
                {w}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* No warnings */}
      {warnings.length === 0 && (
        <div className="mb-6 flex items-center gap-2 rounded-lg border border-green-500/20 bg-green-500/5 px-4 py-3 text-sm text-green-400">
          <CheckCircle2 className="h-4 w-4" />
          Geen waarschuwingen — klaar om te importeren.
        </div>
      )}

      {/* Import button */}
      <div className="flex justify-center">
        <button
          onClick={onImport}
          className="flex items-center gap-2 rounded-lg bg-[#45B6A8] px-6 py-3 text-sm font-semibold text-white shadow-lg transition-colors hover:bg-[#3da396]"
        >
          Importeer project
          <ArrowRight className="h-4 w-4" />
        </button>
      </div>

      <p className="mt-3 text-center text-xs text-gray-500">
        Na import wordt het project geladen in de modeller waar je de
        constructies en openingen verder kunt bewerken.
      </p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Sub-component: stat card
// ---------------------------------------------------------------------------

interface StatCardProps {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: number;
  detail: string;
}

function StatCard({ icon: Icon, label, value, detail }: StatCardProps) {
  return (
    <div className="rounded-lg border border-gray-700 bg-gray-800/50 px-4 py-3">
      <div className="flex items-center gap-2 text-gray-400">
        <Icon className="h-4 w-4" />
        <span className="text-xs font-medium uppercase tracking-wider">
          {label}
        </span>
      </div>
      <p className="mt-1 text-2xl font-bold tabular-nums text-gray-100">
        {value}
      </p>
      <p className="mt-0.5 text-xs text-gray-500">{detail}</p>
    </div>
  );
}
