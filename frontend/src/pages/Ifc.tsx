/**
 * IFC-page — toont status van de huidige IFC-import.
 *
 * Voor PR H is dit een lichte status-page (aantal rooms / wall types / etc).
 * Een echte IFC viewer (3D rendering van het bron-IFC bestand) komt in een
 * latere PR — vereist `@thatopen/components` setup vergelijkbaar met de
 * 3D-modus van FloorCanvas3D.
 */
import { useModellerStore } from "../components/modeller/modellerStore";
import { isTauri } from "../lib/backend";

export function Ifc() {
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
          {tauriMode
            ? "IFC-bestanden importeren en bekijken"
            : "IFC-import vereist de desktop-app — preview-mode toont alleen status"}
        </p>
      </div>

      <div className="flex-1 overflow-auto p-6">
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

            <p className="text-xs text-scaffold-gray">
              Een echte 3D IFC-viewer komt in een latere PR. Voor nu kun je
              de geometrie in de Modeller-tab inzien.
            </p>
          </div>
        ) : (
          <div className="flex h-full items-center justify-center">
            <div className="max-w-lg rounded-lg border border-border bg-surface p-8 text-center shadow-sm">
              <h2 className="mb-2 text-base font-semibold text-on-surface">
                Geen IFC geïmporteerd
              </h2>
              <p className="mb-2 text-sm text-on-surface-2">
                Klik op "Importeer IFC" in de Ribbon om een .ifc bestand te
                laden via de PyInstaller sidecar (IfcOpenShell).
              </p>
              {!tauriMode && (
                <p className="mt-3 text-xs text-scaffold-gray">
                  In web-mode is alleen status-weergave beschikbaar; de import
                  zelf werkt alleen in de desktop-app.
                </p>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
