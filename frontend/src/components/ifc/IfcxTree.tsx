/**
 * IFCX (IFC5 alpha JSON) tree-viewer.
 *
 * Toont de inhoud van een .ifcx of .ifcenergy bestand: header, imports,
 * schemas en per-entry attributes. Elk entry is collapsible. Namespaces
 * krijgen een gekleurde badge (bsi::ifc, isso51::, isso51::modeller::, etc.)
 * zodat in één oogopslag te zien is welke standaard + extensies er gebruikt
 * worden.
 */
import { useCallback, useMemo, useRef, useState } from "react";

import type { IfcxDataEntry, IfcxDocument } from "../modeller/ifcx";

interface IfcxTreeProps {
  /** Optional pre-loaded document, bypasses file picker. */
  initialDoc?: IfcxDocument | null;
  initialFileName?: string;
}

/** Color-code a namespace by its top-level prefix. */
function namespaceBadgeColor(ns: string): string {
  if (ns.startsWith("bsi::ifc::class")) return "bg-blue-100 text-blue-900";
  if (ns.startsWith("bsi::ifc::prop")) return "bg-blue-50 text-blue-800";
  if (ns.startsWith("bsi::ifc")) return "bg-blue-50 text-blue-700";
  if (ns.startsWith("isso51::modeller::")) return "bg-purple-100 text-purple-900";
  if (ns.startsWith("isso51::calc::")) return "bg-amber-100 text-amber-900";
  if (ns.startsWith("isso51::envelope")) return "bg-emerald-100 text-emerald-900";
  if (ns.startsWith("isso51::")) return "bg-teal-100 text-teal-900";
  if (ns.startsWith("usd::")) return "bg-fuchsia-100 text-fuchsia-900";
  return "bg-slate-100 text-slate-800";
}

function EntryRow({ entry, idx }: { entry: IfcxDataEntry; idx: number }) {
  const [open, setOpen] = useState(false);
  const ifcClass =
    (entry.attributes?.["bsi::ifc::class"] as { code?: string } | undefined)?.code ??
    "—";
  const name =
    (entry.attributes?.["bsi::ifc::prop::Name"] as string | undefined) ?? null;

  const namespaces = useMemo(() => {
    if (!entry.attributes) return [];
    return Object.keys(entry.attributes).sort();
  }, [entry.attributes]);

  return (
    <div className="border-b border-border/50">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="flex w-full items-center gap-2 py-1.5 text-left text-xs hover:bg-surface-2"
      >
        <span className="w-4 select-none font-mono text-scaffold-gray">
          {open ? "▼" : "▶"}
        </span>
        <span className="w-12 text-right font-mono text-[10px] text-scaffold-gray">
          {idx}
        </span>
        <span className="rounded bg-blue-50 px-1.5 py-0.5 font-mono text-[10px] text-blue-800">
          {ifcClass}
        </span>
        {name && <span className="font-medium text-on-surface">{name}</span>}
        <span className="ml-auto font-mono text-[10px] text-scaffold-gray">
          {namespaces.length} attr · {Object.keys(entry.children ?? {}).length}{" "}
          ref
        </span>
      </button>

      {open && (
        <div className="bg-surface-2 px-6 py-2 text-xs">
          <div className="mb-1 text-[10px] uppercase tracking-wide text-scaffold-gray">
            Path
          </div>
          <code className="mb-2 block break-all rounded bg-surface px-2 py-1 font-mono text-[10px]">
            {entry.path}
          </code>

          {Object.keys(entry.children ?? {}).length > 0 && (
            <>
              <div className="mb-1 mt-2 text-[10px] uppercase tracking-wide text-scaffold-gray">
                Children
              </div>
              <div className="mb-2 space-y-0.5">
                {Object.entries(entry.children ?? {}).map(([slot, p]) => (
                  <div key={slot} className="flex gap-2 font-mono text-[10px]">
                    <span className="text-scaffold-gray">{slot}</span>
                    <span className="text-on-surface-2">→ {p}</span>
                  </div>
                ))}
              </div>
            </>
          )}

          <div className="mb-1 mt-2 text-[10px] uppercase tracking-wide text-scaffold-gray">
            Attributes
          </div>
          <div className="space-y-1.5">
            {namespaces.map((ns) => (
              <div key={ns}>
                <div className="mb-0.5 flex items-center gap-2">
                  <span
                    className={`rounded px-1.5 py-0.5 font-mono text-[10px] ${namespaceBadgeColor(ns)}`}
                  >
                    {ns}
                  </span>
                </div>
                <pre className="overflow-x-auto rounded bg-surface px-2 py-1 font-mono text-[10px] text-on-surface-2">
                  {JSON.stringify(entry.attributes?.[ns], null, 2)}
                </pre>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export function IfcxTree({ initialDoc, initialFileName }: IfcxTreeProps) {
  const [doc, setDoc] = useState<IfcxDocument | null>(initialDoc ?? null);
  const [fileName, setFileName] = useState<string | null>(
    initialFileName ?? null,
  );
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadDoc = useCallback(async (file: File) => {
    setError(null);
    try {
      const text = await file.text();
      const parsed = JSON.parse(text);
      if (!parsed.header?.ifcxVersion || !Array.isArray(parsed.data)) {
        throw new Error("Geen geldige IFCX header / data array");
      }
      setDoc(parsed as IfcxDocument);
      setFileName(file.name);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(`IFCX laden mislukt: ${msg}`);
    }
  }, []);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) void loadDoc(file);
    e.target.value = "";
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files?.[0];
    if (file) void loadDoc(file);
  };

  return (
    <div
      onDragOver={(e) => e.preventDefault()}
      onDrop={handleDrop}
      className="flex h-full w-full flex-col bg-surface-2"
    >
      {/* Toolbar */}
      <div className="flex items-center gap-2 border-b border-border bg-surface px-3 py-2">
        <button
          onClick={() => fileInputRef.current?.click()}
          className="rounded bg-primary px-3 py-1 text-xs font-medium text-on-primary hover:bg-primary/90"
        >
          📁 Open .ifcx of .ifcenergy...
        </button>
        <span className="text-xs text-scaffold-gray">
          {fileName ?? "of sleep een bestand"}
        </span>
        <input
          ref={fileInputRef}
          type="file"
          accept=".ifcx,.ifcenergy,.json"
          className="hidden"
          onChange={handleFileChange}
        />
      </div>

      {error && (
        <div className="border-b border-red-200 bg-red-50 px-3 py-2 text-xs text-red-800">
          {error}
        </div>
      )}

      <div className="flex-1 overflow-auto p-3">
        {!doc ? (
          <div className="flex h-full items-center justify-center">
            <div className="max-w-md rounded-lg border border-border bg-surface p-6 text-center text-sm">
              <div className="mb-2 font-medium text-on-surface">
                Geen IFCX-document geladen
              </div>
              <div className="text-xs text-scaffold-gray">
                Sleep een <code>.ifcx</code> of <code>.ifcenergy</code> bestand
                hierheen, of klik "Open" hierboven. Het document wordt
                getoond als boom van entries met hun attributen per namespace.
              </div>
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {/* Header card */}
            <div className="rounded-lg border border-border bg-surface p-3 text-xs">
              <div className="mb-1 font-semibold text-on-surface">Header</div>
              <dl className="grid grid-cols-[120px_1fr] gap-x-3 gap-y-0.5 font-mono text-[11px]">
                <dt className="text-scaffold-gray">id</dt>
                <dd className="break-all">{doc.header.id}</dd>
                <dt className="text-scaffold-gray">ifcxVersion</dt>
                <dd>{doc.header.ifcxVersion}</dd>
                <dt className="text-scaffold-gray">dataVersion</dt>
                <dd>{doc.header.dataVersion}</dd>
                <dt className="text-scaffold-gray">author</dt>
                <dd>{doc.header.author}</dd>
                <dt className="text-scaffold-gray">timestamp</dt>
                <dd>{doc.header.timestamp}</dd>
              </dl>
            </div>

            {/* Imports */}
            {doc.imports.length > 0 && (
              <div className="rounded-lg border border-border bg-surface p-3 text-xs">
                <div className="mb-1 font-semibold text-on-surface">
                  Imports ({doc.imports.length})
                </div>
                <ul className="space-y-0.5 font-mono text-[11px]">
                  {doc.imports.map((imp, i) => (
                    <li key={i} className="break-all text-on-surface-2">
                      {imp.uri}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {/* Schemas (just keys) */}
            {Object.keys(doc.schemas ?? {}).length > 0 && (
              <div className="rounded-lg border border-border bg-surface p-3 text-xs">
                <div className="mb-1 font-semibold text-on-surface">
                  Schemas ({Object.keys(doc.schemas).length})
                </div>
                <div className="flex flex-wrap gap-1">
                  {Object.keys(doc.schemas).map((k) => (
                    <span
                      key={k}
                      className={`rounded px-1.5 py-0.5 font-mono text-[10px] ${namespaceBadgeColor(k)}`}
                    >
                      {k}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Entries */}
            <div className="rounded-lg border border-border bg-surface text-xs">
              <div className="border-b border-border px-3 py-2 font-semibold text-on-surface">
                Entries ({doc.data.length})
              </div>
              <div>
                {doc.data.map((entry, i) => (
                  <EntryRow key={entry.path + i} entry={entry} idx={i} />
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
