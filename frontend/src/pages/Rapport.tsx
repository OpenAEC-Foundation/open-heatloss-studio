/**
 * Rapport-page — toont de gegenereerde PDF in een iframe + een
 * inklapbaar opties-paneel.
 *
 * Layout: collapsible sidebar links (toggle via chevron-button), PDF iframe
 * vult de rest. Default: ingeklapt zodat de preview maximale breedte heeft;
 * klap open wanneer je voorbladafbeelding of sectie-opties wilt aanpassen.
 *
 * Workflow: gebruiker klikt "Genereren" in de Ribbon (RapportTab) → PDF
 * wordt gebouwd en de Blob URL belandt in `useReportStore.pdfBlobUrl` →
 * deze page rendert 'm in een iframe.
 */
import { useCallback, useRef, useState } from "react";

import RapportOpmaakDialog from "../components/rapport/RapportOpmaakDialog";
import { useReportStore, type ReportSections } from "../store/reportStore";
import { useProjectStore } from "../store/projectStore";
import { useToastStore } from "../store/toastStore";

/** Volgorde + labels van sectie-toggles in het opties-paneel. */
const SECTION_LABELS: ReadonlyArray<readonly [keyof ReportSections, string]> = [
  ["colofon", "Colofon"],
  ["toc", "Inhoudsopgave"],
  ["uitgangspunten", "Uitgangspunten"],
  ["constructies", "Constructie-opbouw & Rc-waarden"],
  ["vertrekkenOverzicht", "Vertrekken overzicht"],
  ["perVertrek", "Per vertrek (invoer + resultaten)"],
  ["diagrammen", "Diagrammen"],
  ["gebouwresultaten", "Gebouwresultaten"],
  ["tojuli", "TO-juli (koelbehoefte indicatie)"],
  ["backcover", "Backcover"],
];

const MAX_COVER_IMAGE_SIZE = 2 * 1024 * 1024; // 2 MB

export function Rapport() {
  const pdfBlobUrl = useReportStore((s) => s.pdfBlobUrl);
  const generatedAt = useReportStore((s) => s.generatedAt);
  const result = useProjectStore((s) => s.result);
  const project = useProjectStore((s) => s.project);
  const [optionsOpen, setOptionsOpen] = useState(true);
  const [opmaakOpen, setOpmaakOpen] = useState(false);
  const updateProject = useProjectStore((s) => s.updateProject);
  const sections = useReportStore((s) => s.sections);
  const setSection = useReportStore((s) => s.setSection);
  const resetSections = useReportStore((s) => s.resetSections);
  const pageSize = useReportStore((s) => s.pageSize);
  const setPageSize = useReportStore((s) => s.setPageSize);
  const orientation = useReportStore((s) => s.orientation);
  const setOrientation = useReportStore((s) => s.setOrientation);
  const addToast = useToastStore((s) => s.addToast);

  // Helper: update only the `info` sub-object, leaving the rest of the project
  // intact. The store exposes the broader `updateProject(partial)` action — we
  // wrap a thin shim instead of adding a dedicated `updateProjectInfo` action.
  const updateProjectInfo = useCallback(
    (partial: Partial<typeof project.info>) => {
      updateProject({ info: { ...project.info, ...partial } });
    },
    [updateProject, project.info],
  );

  const fileInputRef = useRef<HTMLInputElement>(null);

  const coverImage = project.info.cover_image ?? null;

  const handleCoverImageChange = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      if (file.size > MAX_COVER_IMAGE_SIZE) {
        addToast("Afbeelding is groter dan 2 MB.", "error");
        e.target.value = "";
        return;
      }
      if (file.type !== "image/png" && file.type !== "image/jpeg") {
        addToast("Alleen PNG of JPEG worden ondersteund.", "error");
        e.target.value = "";
        return;
      }
      try {
        const dataUrl: string = await new Promise((resolve, reject) => {
          const fr = new FileReader();
          fr.onload = () => resolve(String(fr.result));
          fr.onerror = () => reject(fr.error);
          fr.readAsDataURL(file);
        });
        const base64 = dataUrl.replace(/^data:[^;]+;base64,/, "");
        updateProjectInfo({
          cover_image: {
            data: base64,
            media_type: file.type as "image/png" | "image/jpeg",
            filename: file.name,
          },
        });
        addToast("Voorbladafbeelding opgeslagen.", "success");
      } catch (err) {
        addToast(
          `Inlezen mislukt: ${err instanceof Error ? err.message : String(err)}`,
          "error",
        );
      }
      e.target.value = "";
    },
    [updateProjectInfo, addToast],
  );

  const handleCoverImageClear = useCallback(() => {
    updateProjectInfo({ cover_image: null });
    addToast("Voorbladafbeelding verwijderd.", "info");
  }, [updateProjectInfo, addToast]);

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

      <div className="flex flex-1 overflow-hidden">
        {/* Left: opties paneel — collapsible (default ingeklapt zodat preview
            de volledige breedte krijgt). Toggle-rail blijft altijd zichtbaar. */}
        <aside
          className={`relative shrink-0 overflow-hidden border-r border-border bg-surface transition-[width] duration-150 ease-out ${
            optionsOpen ? "w-[260px]" : "w-10"
          }`}
        >
          <button
            type="button"
            onClick={() => setOptionsOpen((v) => !v)}
            className="absolute right-1 top-1 z-10 flex h-7 w-7 items-center justify-center rounded text-on-surface-secondary hover:bg-[var(--oaec-hover)]"
            title={optionsOpen ? "Opties inklappen" : "Opties uitklappen"}
            aria-label={optionsOpen ? "Opties inklappen" : "Opties uitklappen"}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className={`transition-transform duration-150 ${
                optionsOpen ? "" : "rotate-180"
              }`}
            >
              <polyline points="15 18 9 12 15 6" />
            </svg>
          </button>
          {!optionsOpen && (
            <div
              className="absolute left-1/2 top-12 -translate-x-1/2 whitespace-nowrap text-[10px] uppercase tracking-wide text-on-surface-muted"
              style={{ writingMode: "vertical-rl", textOrientation: "mixed" }}
            >
              Opties
            </div>
          )}
        {optionsOpen && (
        <div className="h-full overflow-y-auto p-4 pt-10">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-on-surface">Opties</h2>
            <button
              type="button"
              onClick={() => setOpmaakOpen(true)}
              className="rounded border border-border bg-surface px-2 py-1 text-[11px] text-on-surface-secondary hover:bg-surface-alt"
              title="Rapport opmaak instellingen openen"
            >
              Opmaak…
            </button>
          </div>

          {/* Pagina-instellingen — bovenaan zodat formaat + oriëntatie
              direct zichtbaar zijn vóór de inhoud-toggles. */}
          <section className="mb-4 rounded-md border border-border bg-surface-2 p-3">
            <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-on-surface-secondary">
              Pagina
            </h3>
            <div className="space-y-2">
              <div>
                <label className="mb-1 block text-[10px] uppercase tracking-wide text-on-surface-muted">
                  Formaat
                </label>
                <div className="flex gap-1">
                  {(["A4", "A3"] as const).map((size) => (
                    <button
                      key={size}
                      type="button"
                      onClick={() => setPageSize(size)}
                      className={`flex-1 rounded border px-3 py-1.5 text-xs transition-colors ${
                        pageSize === size
                          ? "border-[var(--theme-accent)] bg-[var(--theme-accent-soft)] text-on-surface"
                          : "border-border bg-surface text-on-surface-secondary hover:bg-surface-alt"
                      }`}
                    >
                      {size}
                    </button>
                  ))}
                </div>
              </div>
              <div>
                <label className="mb-1 block text-[10px] uppercase tracking-wide text-on-surface-muted">
                  Oriëntatie
                </label>
                <div className="flex gap-1">
                  {(
                    [
                      ["portrait", "Portret"],
                      ["landscape", "Landschap"],
                    ] as const
                  ).map(([key, label]) => (
                    <button
                      key={key}
                      type="button"
                      onClick={() => setOrientation(key)}
                      className={`flex-1 rounded border px-3 py-1.5 text-xs transition-colors ${
                        orientation === key
                          ? "border-[var(--theme-accent)] bg-[var(--theme-accent-soft)] text-on-surface"
                          : "border-border bg-surface text-on-surface-secondary hover:bg-surface-alt"
                      }`}
                    >
                      {label}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </section>

          {/* Voorbladafbeelding */}
          <section className="mb-4 rounded-md border border-border bg-surface-2 p-3">
            <h3 className="mb-1 text-xs font-semibold uppercase tracking-wide text-on-surface-secondary">
              Voorbladafbeelding
            </h3>
            <p className="mb-2 text-[10px] text-on-surface-muted">
              PNG of JPEG, max 2 MB. Verschijnt op het voorblad.
            </p>

            {coverImage ? (
              <div className="flex flex-col gap-2">
                <img
                  src={`data:${coverImage.media_type};base64,${coverImage.data}`}
                  alt="Voorblad"
                  className="h-32 w-full rounded border border-border object-cover"
                />
                <div className="flex items-center justify-between gap-2">
                  <span className="truncate text-xs text-on-surface-muted">
                    {coverImage.filename ?? "afbeelding"}
                  </span>
                  <button
                    type="button"
                    onClick={handleCoverImageClear}
                    className="shrink-0 rounded border border-border px-2 py-1 text-xs text-on-surface-secondary hover:bg-surface-alt"
                  >
                    Verwijderen
                  </button>
                </div>
                <button
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  className="rounded border border-border px-2 py-1 text-xs text-on-surface-secondary hover:bg-surface-alt"
                >
                  Andere kiezen…
                </button>
              </div>
            ) : (
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="w-full rounded border border-dashed border-border px-3 py-4 text-xs text-on-surface-secondary hover:bg-surface-alt"
              >
                Afbeelding kiezen…
              </button>
            )}
            <input
              ref={fileInputRef}
              type="file"
              accept="image/png,image/jpeg"
              onChange={handleCoverImageChange}
              className="hidden"
            />
          </section>

          {/* Secties toggles */}
          <section className="rounded-md border border-border bg-surface-2 p-3">
            <div className="mb-2 flex items-center justify-between">
              <h3 className="text-xs font-semibold uppercase tracking-wide text-on-surface-secondary">
                Inhoud
              </h3>
              <button
                type="button"
                onClick={resetSections}
                className="text-[10px] text-on-surface-muted hover:text-on-surface-secondary"
                title="Alles aan"
              >
                reset
              </button>
            </div>
            <p className="mb-2 text-[10px] text-on-surface-muted">
              Vink secties uit om ze uit het PDF-rapport weg te laten.
            </p>
            <div className="space-y-1.5">
              {SECTION_LABELS.map(([key, label]) => (
                <label
                  key={key}
                  className="flex cursor-pointer items-center gap-2 text-[11px] text-on-surface"
                >
                  <input
                    type="checkbox"
                    checked={sections[key]}
                    onChange={(e) => setSection(key, e.target.checked)}
                    className="h-3.5 w-3.5 cursor-pointer accent-[var(--theme-accent)]"
                  />
                  <span>{label}</span>
                </label>
              ))}
            </div>
            <p className="mt-2 text-[10px] text-on-surface-muted">
              Cover staat altijd aan. Bij volgende "Genereren" worden de
              toggles toegepast.
            </p>
          </section>
        </div>
        )}
        </aside>

        <RapportOpmaakDialog
          open={opmaakOpen}
          onClose={() => setOpmaakOpen(false)}
        />

        {/* Right: PDF preview */}
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
    </div>
  );
}
