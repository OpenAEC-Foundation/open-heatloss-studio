/**
 * IFC-page met drie sub-tabs:
 * - IFC4x3 → 3D viewer (ThatOpen) voor .ifc bron-bestanden
 * - IFCX → boom-view voor .ifcx en .ifcenergy bestanden
 * - Sidecar import → status van de Tauri PyInstaller import (rooms/walls/etc)
 *
 * Geïnspireerd op Open Calc Studio's IFC tab + viewers/ThreeDViewer.
 */
import { useState } from "react";

import { useModellerStore } from "../components/modeller/modellerStore";
import { isTauri } from "../lib/backend";
import { IfcViewer3D } from "../components/ifc/IfcViewer3D";
import { IfcxTree } from "../components/ifc/IfcxTree";

type Tab = "ifc4x3" | "ifcx" | "sidecar";

export function Ifc() {
  const [tab, setTab] = useState<Tab>("ifc4x3");
  const rooms = useModellerStore((s) => s.rooms);
  const windows = useModellerStore((s) => s.windows);
  const doors = useModellerStore((s) => s.doors);
  const importedBoundaries = useModellerStore((s) => s.importedBoundaries);

  const tauriMode = isTauri();
  const hasImport = rooms.length > 0 || importedBoundaries.length > 0;

  return (
    <div className="flex h-full w-full flex-col">
      <div className="border-b border-border px-6 py-3">
        <h1 className="text-lg font-semibold text-on-surface">IFC</h1>
        <p className="text-xs text-scaffold-gray">
          Inhoud van IFC4x3 (.ifc) en IFCX (.ifcx / .ifcenergy) bestanden
          inzien — plus status van de Tauri sidecar-import naar de modeller.
        </p>
      </div>

      {/* Sub-tabs */}
      <div className="flex gap-1 border-b border-border bg-surface-2 px-3 pt-2">
        <SubTab active={tab === "ifc4x3"} onClick={() => setTab("ifc4x3")}>
          IFC4x3 (.ifc)
        </SubTab>
        <SubTab active={tab === "ifcx"} onClick={() => setTab("ifcx")}>
          IFCX (.ifcx / .ifcenergy)
        </SubTab>
        <SubTab active={tab === "sidecar"} onClick={() => setTab("sidecar")}>
          Sidecar import {hasImport ? `(${rooms.length})` : ""}
        </SubTab>
      </div>

      {/* Active tab content */}
      <div className="flex-1 overflow-hidden">
        {tab === "ifc4x3" && <IfcViewer3D />}
        {tab === "ifcx" && <IfcxTree />}
        {tab === "sidecar" && (
          <SidecarStatus
            rooms={rooms}
            windows={windows}
            doors={doors}
            importedBoundaries={importedBoundaries}
            hasImport={hasImport}
            tauriMode={tauriMode}
          />
        )}
      </div>
    </div>
  );
}

function SubTab({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={
        "rounded-t-md border border-border px-3 py-1.5 text-xs font-medium transition-colors " +
        (active
          ? "border-b-transparent bg-surface text-on-surface"
          : "bg-surface-2 text-scaffold-gray hover:bg-surface")
      }
    >
      {children}
    </button>
  );
}

interface SidecarStatusProps {
  rooms: ReturnType<typeof useModellerStore.getState>["rooms"];
  windows: ReturnType<typeof useModellerStore.getState>["windows"];
  doors: ReturnType<typeof useModellerStore.getState>["doors"];
  importedBoundaries: ReturnType<
    typeof useModellerStore.getState
  >["importedBoundaries"];
  hasImport: boolean;
  tauriMode: boolean;
}

function SidecarStatus({
  rooms,
  windows,
  doors,
  importedBoundaries,
  hasImport,
  tauriMode,
}: SidecarStatusProps) {
  return (
    <div className="h-full overflow-auto p-6">
      {hasImport ? (
        <div className="space-y-4">
          <section className="rounded-lg border border-border bg-surface p-4">
            <h2 className="mb-2 text-sm font-semibold text-on-surface">
              Geïmporteerde geometrie
            </h2>
            <dl className="grid grid-cols-2 gap-3 text-sm">
              <div>
                <dt className="text-xs text-scaffold-gray">Ruimten</dt>
                <dd className="text-2xl font-semibold tabular-nums">
                  {rooms.length}
                </dd>
              </div>
              <div>
                <dt className="text-xs text-scaffold-gray">Ramen</dt>
                <dd className="text-2xl font-semibold tabular-nums">
                  {windows.length}
                </dd>
              </div>
              <div>
                <dt className="text-xs text-scaffold-gray">Deuren</dt>
                <dd className="text-2xl font-semibold tabular-nums">
                  {doors.length}
                </dd>
              </div>
              <div>
                <dt className="text-xs text-scaffold-gray">
                  Thermische grenzen
                </dt>
                <dd className="text-2xl font-semibold tabular-nums">
                  {importedBoundaries.length}
                </dd>
              </div>
            </dl>
          </section>

          {rooms.length > 0 && (
            <section className="rounded-lg border border-border bg-surface p-4">
              <h2 className="mb-2 text-sm font-semibold text-on-surface">
                Ruimten ({rooms.length})
              </h2>
              <div className="max-h-96 overflow-auto">
                <table className="w-full text-sm">
                  <thead className="sticky top-0 bg-surface text-xs uppercase tracking-wide text-scaffold-gray">
                    <tr>
                      <th className="py-1 text-left">Naam</th>
                      <th className="py-1 text-left">Functie</th>
                      <th className="py-1 text-right">Floor</th>
                      <th className="py-1 text-right">Hoogte (mm)</th>
                      <th className="py-1 text-right">Punten</th>
                    </tr>
                  </thead>
                  <tbody>
                    {rooms.map((r) => (
                      <tr key={r.id} className="border-t border-border/50">
                        <td className="py-1">{r.name}</td>
                        <td className="py-1 text-xs text-scaffold-gray">
                          {r.function}
                        </td>
                        <td className="py-1 text-right tabular-nums">
                          {r.floor}
                        </td>
                        <td className="py-1 text-right tabular-nums">
                          {r.height}
                        </td>
                        <td className="py-1 text-right tabular-nums">
                          {r.polygon.length}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          )}
        </div>
      ) : (
        <div className="flex h-full items-center justify-center">
          <div className="max-w-lg rounded-lg border border-border bg-surface p-8 text-center shadow-sm">
            <h2 className="mb-2 text-base font-semibold text-on-surface">
              Geen IFC geïmporteerd via sidecar
            </h2>
            <p className="mb-2 text-sm text-on-surface-2">
              Klik op "Importeer IFC" in de Ribbon om een .ifc bestand te
              laden via de PyInstaller sidecar (IfcOpenShell). Voor pure
              viewing zonder import: gebruik de IFC4x3-tab hierboven.
            </p>
            {!tauriMode && (
              <p className="mt-3 text-xs text-scaffold-gray">
                In web-mode is sidecar-import niet beschikbaar. De IFC4x3 +
                IFCX viewers werken wel.
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
