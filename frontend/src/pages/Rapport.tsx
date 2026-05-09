/**
 * Rapport-page — toont de gegenereerde PDF in een iframe.
 *
 * Workflow: gebruiker klikt "Genereren" in de Ribbon (RapportTab) → PDF
 * wordt gebouwd en de Blob URL belandt in `useReportStore.pdfBlobUrl` →
 * deze page rendert 'm in een iframe. Lege state vóór generatie toont een
 * uitnodigende placeholder met instructies.
 */
import { useReportStore } from "../store/reportStore";
import { useProjectStore } from "../store/projectStore";

export function Rapport() {
  const pdfBlobUrl = useReportStore((s) => s.pdfBlobUrl);
  const generatedAt = useReportStore((s) => s.generatedAt);
  const result = useProjectStore((s) => s.result);
  const project = useProjectStore((s) => s.project);

  // Bust iframe cache when a new PDF is generated.
  const iframeKey = generatedAt ?? 0;

  return (
    <div className="flex h-full w-full flex-col">
      <div className="flex items-center justify-between border-b border-border px-6 py-3">
        <div>
          <h1 className="text-lg font-semibold text-on-surface">Rapport</h1>
          <p className="text-xs text-scaffold-gray">
            {project.info.name || "Geen projectnaam"}
            {generatedAt
              ? ` — gegenereerd ${new Date(generatedAt).toLocaleTimeString("nl-NL")}`
              : ""}
          </p>
        </div>
      </div>

      <div className="flex-1 bg-surface-2">
        {pdfBlobUrl ? (
          <iframe
            key={iframeKey}
            src={pdfBlobUrl}
            title="Rapport preview"
            className="h-full w-full border-0"
          />
        ) : (
          <div className="flex h-full items-center justify-center">
            <div className="max-w-lg rounded-lg border border-border bg-surface p-8 text-center shadow-sm">
              <h2 className="mb-2 text-base font-semibold text-on-surface">
                Nog geen rapport gegenereerd
              </h2>
              <p className="mb-4 text-sm text-on-surface-2">
                {result
                  ? 'Klik op "Genereren" in de Ribbon hierboven om een PDF-rapport van de berekening te bouwen.'
                  : "Voer eerst een berekening uit (Vertrekken → Berekenen) — daarna kun je hier het rapport genereren."}
              </p>
              <p className="text-xs text-scaffold-gray">
                Het rapport wordt lokaal gegenereerd via de Rust openaec-layout
                engine (geen server nodig in de desktop-app).
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
