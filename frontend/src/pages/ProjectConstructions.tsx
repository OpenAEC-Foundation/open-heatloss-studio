/**
 * Project Constructions page — manages per-project construction library.
 *
 * Shows all constructions specific to this project (copied from catalogue,
 * imported from IFC, or created via Rc-calculator). Users can browse the
 * standard catalogue and copy entries into their project.
 */
import { Pencil } from "lucide-react";
import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type MouseEvent as ReactMouseEvent,
} from "react";
import { useNavigate } from "react-router-dom";

import { useModellerStore } from "../components/modeller/modellerStore";
import type { ProjectConstruction } from "../components/modeller/types";
import { useCatalogueStore } from "../store/catalogueStore";
import { useProjectStore } from "../store/projectStore";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
  type CatalogueEntry,
} from "../lib/constructionCatalogue";
import { calculateRc, roundUValue, type RcResult } from "../lib/rcCalculation";

const CATEGORY_ORDER: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "daken",
  "kozijnen_vullingen",
];

const NAME_EDIT_INPUT_CLASS =
  "min-w-0 flex-1 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0 text-sm font-medium text-on-surface outline-none focus:border-primary";

const U_VALUE_EDIT_INPUT_CLASS =
  "w-20 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0 text-xs text-on-surface outline-none focus:border-primary";

const NAME_EDIT_ICON_SIZE_CLASS = "h-3.5 w-3.5";

const FRAME_CATEGORY = "kozijnen_vullingen" as const;

type ViewTab = "project" | "catalogus";

export function ProjectConstructions() {
  const projectConstructions = useModellerStore(
    (s) => s.projectConstructions,
  );
  const removeProjectConstruction = useModellerStore(
    (s) => s.removeProjectConstruction,
  );
  const copyFromCatalogue = useModellerStore((s) => s.copyFromCatalogue);
  const catalogueEntries = useCatalogueStore((s) => s.entries);

  const [tab, setTab] = useState<ViewTab>("project");
  const [search, setSearch] = useState("");
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  // Rooms from project store — for showing linked rooms
  const rooms = useProjectStore((s) => s.project.rooms);

  // Map project construction ID → list of room names that use it
  const linkedRooms = useMemo(() => {
    const map: Record<string, string[]> = {};
    for (const room of rooms) {
      for (const ce of room.constructions) {
        if (ce.project_construction_id) {
          const list = map[ce.project_construction_id] ?? [];
          if (!list.includes(room.name)) {
            list.push(room.name);
          }
          map[ce.project_construction_id] = list;
        }
      }
    }
    return map;
  }, [rooms]);

  // Group project constructions by category
  const projectGrouped = useMemo(() => {
    const map = new Map<CatalogueCategory, ProjectConstruction[]>();
    for (const pc of projectConstructions) {
      if (search && !pc.name.toLowerCase().includes(search.toLowerCase())) {
        continue;
      }
      const list = map.get(pc.category) ?? [];
      list.push(pc);
      map.set(pc.category, list);
    }
    return map;
  }, [projectConstructions, search]);

  // Group catalogue entries by category (for catalogue tab)
  const catalogueGrouped = useMemo(() => {
    const map = new Map<CatalogueCategory, CatalogueEntry[]>();
    for (const entry of catalogueEntries) {
      if (search && !entry.name.toLowerCase().includes(search.toLowerCase())) {
        continue;
      }
      const list = map.get(entry.category) ?? [];
      list.push(entry);
      map.set(entry.category, list);
    }
    return map;
  }, [catalogueEntries, search]);

  const isInProject = (catalogueId: string): boolean =>
    projectConstructions.some((c) => c.catalogueSourceId === catalogueId);

  const tabClass = (t: ViewTab) =>
    `px-4 py-2 text-sm font-medium transition-colors ${
      tab === t
        ? "border-b-2 oa-accent-tab-active"
        : "text-on-surface-muted hover:text-on-surface-secondary"
    }`;

  return (
    <div className="mx-auto max-w-4xl p-6">
      <h1 className="mb-1 text-xl font-bold text-on-surface">Constructies</h1>
      <p className="mb-4 text-sm text-on-surface-muted">
        Beheer de constructies voor dit project. Kopieer vanuit de standaard
        bibliotheek of maak nieuwe constructies aan via de Rc-waarde tool.
      </p>

      {/* Tab strip */}
      <div className="mb-4 flex border-b border-[var(--oaec-border)]">
        <button onClick={() => setTab("project")} className={tabClass("project")}>
          Project ({projectConstructions.length})
        </button>
        <button onClick={() => setTab("catalogus")} className={tabClass("catalogus")}>
          Standaard bibliotheek
        </button>
      </div>

      {/* Search */}
      <div className="mb-4">
        <input
          placeholder="Zoeken op naam..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full max-w-sm rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] text-on-surface px-3 py-1.5 text-sm outline-none focus:border-primary"
        />
      </div>

      {/* Project tab */}
      {tab === "project" && (
        <>
          {projectConstructions.length === 0 ? (
            <div className="rounded border border-dashed border-[var(--oaec-border)] px-6 py-8 text-center">
              <p className="text-sm text-on-surface-muted">
                Nog geen constructies in dit project.
              </p>
              <p className="mt-1 text-xs text-on-surface-muted">
                Ga naar het tabblad "Standaard bibliotheek" om constructies toe
                te voegen, of maak een nieuwe aan via de{" "}
                <a href="/rc" className="oa-accent-link hover:underline">
                  Rc-waarde tool
                </a>
                .
              </p>
            </div>
          ) : (
            CATEGORY_ORDER.map((cat) => {
              const entries = projectGrouped.get(cat);
              if (!entries?.length) return null;

              return (
                <div key={cat} className="mb-6">
                  <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                    {CATALOGUE_CATEGORY_LABELS[cat]}
                  </h2>
                  <div className="space-y-2">
                    {entries.map((pc) => (
                      <ProjectConstructionCard
                        key={pc.id}
                        pc={pc}
                        roomNames={linkedRooms[pc.id] ?? []}
                        isExpanded={expanded.has(pc.id)}
                        onToggle={() => setExpanded((prev) => {
                          const next = new Set(prev);
                          if (next.has(pc.id)) next.delete(pc.id);
                          else next.add(pc.id);
                          return next;
                        })}
                        confirmDelete={confirmDelete === pc.id}
                        onConfirmDelete={() => setConfirmDelete(pc.id)}
                        onDelete={() => {
                          removeProjectConstruction(pc.id);
                          setConfirmDelete(null);
                        }}
                        onCancelDelete={() => setConfirmDelete(null)}
                      />
                    ))}
                  </div>
                </div>
              );
            })
          )}
        </>
      )}

      {/* Catalogue tab */}
      {tab === "catalogus" && (

        <>
          {CATEGORY_ORDER.map((cat) => {
            const entries = catalogueGrouped.get(cat);
            if (!entries?.length) return null;

            return (
              <div key={cat} className="mb-6">
                <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                  {CATALOGUE_CATEGORY_LABELS[cat]}
                </h2>
                <div className="space-y-2">
                  {entries.map((entry) => (
                    <CatalogueEntryCard
                      key={entry.id}
                      entry={entry}
                      inProject={isInProject(entry.id)}
                      onCopy={() => copyFromCatalogue(entry)}
                    />
                  ))}
                </div>
              </div>
            );
          })}
        </>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Project construction card with expandable layer details
// ---------------------------------------------------------------------------

function ProjectConstructionCard({
  pc,
  roomNames,
  isExpanded,
  onToggle,
  confirmDelete,
  onConfirmDelete,
  onDelete,
  onCancelDelete,
}: {
  pc: ProjectConstruction;
  roomNames: string[];
  isExpanded: boolean;
  onToggle: () => void;
  confirmDelete: boolean;
  onConfirmDelete: () => void;
  onDelete: () => void;
  onCancelDelete: () => void;
}) {
  const updateProjectConstruction = useModellerStore(
    (s) => s.updateProjectConstruction,
  );
  const navigate = useNavigate();

  const isFrame = pc.category === FRAME_CATEGORY;

  // Local edit state (niet in de store — UI state per card).
  const [isEditingName, setIsEditingName] = useState(false);
  const [draftName, setDraftName] = useState(pc.name);
  const nameInputRef = useRef<HTMLInputElement | null>(null);

  const [isEditingUValue, setIsEditingUValue] = useState(false);
  const [draftUValue, setDraftUValue] = useState<string>("");
  const uValueInputRef = useRef<HTMLInputElement | null>(null);

  // Focus + select-all bij openen van edit-mode.
  useEffect(() => {
    if (isEditingName && nameInputRef.current) {
      nameInputRef.current.focus();
      nameInputRef.current.select();
    }
  }, [isEditingName]);

  useEffect(() => {
    if (isEditingUValue && uValueInputRef.current) {
      uValueInputRef.current.focus();
      uValueInputRef.current.select();
    }
  }, [isEditingUValue]);

  const startEditName = (e: ReactMouseEvent<HTMLButtonElement>): void => {
    e.stopPropagation();
    setDraftName(pc.name);
    setIsEditingName(true);
  };

  const cancelEditName = (): void => {
    setIsEditingName(false);
    setDraftName(pc.name);
  };

  const commitEditName = (): void => {
    const trimmed = draftName.trim();
    if (trimmed.length === 0 || trimmed === pc.name) {
      cancelEditName();
      return;
    }
    updateProjectConstruction(pc.id, { name: trimmed });
    setIsEditingName(false);
  };

  const handleNameKeyDown = (
    e: ReactKeyboardEvent<HTMLInputElement>,
  ): void => {
    e.stopPropagation();
    if (e.key === "Enter") {
      e.preventDefault();
      commitEditName();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEditName();
    }
  };

  const startEditUValue = (e: ReactMouseEvent<HTMLButtonElement>): void => {
    e.stopPropagation();
    setDraftUValue(pc.uValue !== undefined ? pc.uValue.toString() : "");
    setIsEditingUValue(true);
  };

  const cancelEditUValue = (): void => {
    setIsEditingUValue(false);
    setDraftUValue("");
  };

  const commitEditUValue = (): void => {
    const parsed = Number(draftUValue);
    if (!Number.isFinite(parsed) || parsed <= 0) {
      cancelEditUValue();
      return;
    }
    if (pc.uValue !== undefined && parsed === pc.uValue) {
      cancelEditUValue();
      return;
    }
    updateProjectConstruction(pc.id, { uValue: parsed });
    setIsEditingUValue(false);
  };

  const handleUValueKeyDown = (
    e: ReactKeyboardEvent<HTMLInputElement>,
  ): void => {
    e.stopPropagation();
    if (e.key === "Enter") {
      e.preventDefault();
      commitEditUValue();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEditUValue();
    }
  };

  const hasLayers = pc.layers.length > 0;
  const rcResult: RcResult | null = hasLayers
    ? calculateRc(pc.layers, pc.verticalPosition)
    : null;
  // Bij entries zonder lagen (kozijnen/vullingen) tonen we de directe
  // `pc.uValue` — anders zou de card "geen U-waarde" tonen voor glas/deur.
  const uValue = rcResult
    ? roundUValue(rcResult.uValue)
    : pc.uValue ?? null;
  const totalThickness = rcResult
    ? rcResult.layers.reduce((sum, l) => sum + l.thickness, 0)
    : null;

  return (
    <div className="rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
      {/* Header — clickable for expand */}
      <div
        className="flex cursor-pointer items-center gap-4 px-4 py-3"
        onClick={onToggle}
      >
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-xs text-on-surface-muted">
              {isExpanded ? "\u25BC" : "\u25B6"}
            </span>
            {isEditingName ? (
              <input
                ref={nameInputRef}
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                onKeyDown={handleNameKeyDown}
                onBlur={commitEditName}
                onClick={(e) => e.stopPropagation()}
                onMouseDown={(e) => e.stopPropagation()}
                className={NAME_EDIT_INPUT_CLASS}
                aria-label="Constructienaam bewerken"
              />
            ) : (
              <>
                <span className="text-sm font-medium text-on-surface">
                  {pc.name}
                </span>
                <button
                  type="button"
                  onClick={startEditName}
                  className="rounded p-0.5 text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface-secondary"
                  aria-label="Naam bewerken"
                  title="Naam bewerken"
                >
                  <Pencil className={NAME_EDIT_ICON_SIZE_CLASS} />
                </button>
              </>
            )}
          </div>
          <div className="mt-0.5 flex flex-wrap items-center gap-3 text-xs text-on-surface-muted">
            {uValue !== null && (
              isFrame ? (
                <span
                  className="flex items-center gap-1"
                  onClick={(e) => e.stopPropagation()}
                  onMouseDown={(e) => e.stopPropagation()}
                >
                  {isEditingUValue ? (
                    <>
                      <span>U =</span>
                      <input
                        ref={uValueInputRef}
                        type="number"
                        step="0.01"
                        min="0.01"
                        value={draftUValue}
                        onChange={(e) => setDraftUValue(e.target.value)}
                        onKeyDown={handleUValueKeyDown}
                        onBlur={commitEditUValue}
                        onClick={(e) => e.stopPropagation()}
                        onMouseDown={(e) => e.stopPropagation()}
                        className={U_VALUE_EDIT_INPUT_CLASS}
                        aria-label="U-waarde bewerken"
                      />
                      <span>W/(m{"\u00B2"}{"\u00B7"}K)</span>
                    </>
                  ) : (
                    <>
                      <span>
                        U = {uValue} W/(m{"\u00B2"}{"\u00B7"}K)
                      </span>
                      <button
                        type="button"
                        onClick={startEditUValue}
                        className="rounded p-0.5 text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface-secondary"
                        aria-label="U-waarde bewerken"
                        title="U-waarde bewerken"
                      >
                        <Pencil className={NAME_EDIT_ICON_SIZE_CLASS} />
                      </button>
                    </>
                  )}
                </span>
              ) : (
                <span>
                  U = {uValue} W/(m{"\u00B2"}{"\u00B7"}K)
                </span>
              )
            )}
            {rcResult && (
              <span>
                Rc = {rcResult.rc.toFixed(2)} m{"\u00B2"}K/W
              </span>
            )}
            {totalThickness !== null && (
              <span>{Math.round(totalThickness)} mm</span>
            )}
            {hasLayers && (
              <span>{pc.layers.length} lagen</span>
            )}
            {pc.catalogueSourceId && (
              <span className="text-on-surface-muted">Bron: catalogus</span>
            )}
            {pc.ifcSource && (
              <span className="text-on-surface-muted">
                IFC: {pc.ifcSource.wallTypeName}
              </span>
            )}
          </div>
          {roomNames.length > 0 && (
            <div className="mt-1 flex flex-wrap gap-1">
              {roomNames.map((name) => (
                <span
                  key={name}
                  className="rounded bg-surface-alt px-1.5 py-0.5 text-[10px] text-on-surface-secondary"
                >
                  {name}
                </span>
              ))}
            </div>
          )}
        </div>
        <div
          className="flex shrink-0 items-center gap-2"
          onClick={(e) => e.stopPropagation()}
        >
          {!isFrame && hasLayers && (
            <button
              type="button"
              onClick={() => navigate(`/rc?editProject=${pc.id}`)}
              className="rounded border border-[var(--oaec-border)] px-2.5 py-1 text-xs text-on-surface-secondary hover:bg-[var(--oaec-hover)]"
            >
              Bewerken
            </button>
          )}
          {confirmDelete ? (
            <div className="flex gap-1">
              <button
                onClick={onDelete}
                className="rounded bg-red-600/15 px-2.5 py-1 text-xs text-red-400 hover:bg-red-600/20"
              >
                Bevestig
              </button>
              <button
                onClick={onCancelDelete}
                className="rounded px-2.5 py-1 text-xs text-on-surface-muted hover:bg-[var(--oaec-hover)]"
              >
                Annuleer
              </button>
            </div>
          ) : (
            <button
              onClick={onConfirmDelete}
              className="rounded border border-[var(--oaec-border)] px-2.5 py-1 text-xs text-red-400 hover:bg-red-600/15"
            >
              Verwijderen
            </button>
          )}
        </div>
      </div>

      {/* Expanded layer details */}
      {isExpanded && hasLayers && rcResult && (
        <div className="border-t border-[var(--oaec-border-subtle)] px-4 py-3">
          <table className="w-full text-xs">
            <thead>
              <tr className="text-left text-on-surface-muted">
                <th className="pb-1 font-medium">Laag</th>
                <th className="pb-1 text-right font-medium">d [mm]</th>
                <th className="pb-1 text-right font-medium">
                  {"\u03BB"} [W/(m{"\u00B7"}K)]
                </th>
                <th className="pb-1 text-right font-medium">
                  R [m{"\u00B2"}K/W]
                </th>
              </tr>
            </thead>
            <tbody>
              {rcResult.layers.map((layer, i) => (
                <tr
                  key={i}
                  className="border-t border-[var(--oaec-border-subtle)] text-on-surface-secondary"
                >
                  <td className="py-0.5">{layer.name}</td>
                  <td className="py-0.5 text-right tabular-nums">
                    {Math.round(layer.thickness)}
                  </td>
                  <td className="py-0.5 text-right tabular-nums">
                    {layer.lambda !== null ? layer.lambda.toFixed(3) : "-"}
                  </td>
                  <td className="py-0.5 text-right tabular-nums">
                    {layer.r.toFixed(3)}
                  </td>
                </tr>
              ))}
            </tbody>
            <tfoot>
              <tr className="border-t border-[var(--oaec-border)] font-medium text-on-surface">
                <td className="pt-1">Totaal</td>
                <td className="pt-1 text-right tabular-nums">
                  {totalThickness !== null ? Math.round(totalThickness) : "-"}
                </td>
                <td className="pt-1" />
                <td className="pt-1 text-right tabular-nums">
                  {rcResult.rc.toFixed(3)}
                </td>
              </tr>
              <tr className="text-on-surface-muted">
                <td className="py-0.5">Rsi + Rse</td>
                <td />
                <td />
                <td className="py-0.5 text-right tabular-nums">
                  {(rcResult.rSi + rcResult.rSe).toFixed(3)}
                </td>
              </tr>
              <tr className="font-medium text-on-surface">
                <td className="py-0.5">R_totaal</td>
                <td />
                <td />
                <td className="py-0.5 text-right tabular-nums">
                  {rcResult.rTotal.toFixed(3)}
                </td>
              </tr>
            </tfoot>
          </table>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Catalogue entry card (standaard bibliotheek tab)
// ---------------------------------------------------------------------------

function CatalogueEntryCard({
  entry,
  inProject,
  onCopy,
}: {
  entry: CatalogueEntry;
  inProject: boolean;
  onCopy: () => void;
}) {
  const updateEntry = useCatalogueStore((s) => s.updateEntry);

  // Local edit state (niet in de store — UI state per card).
  const [isEditingName, setIsEditingName] = useState(false);
  const [draftName, setDraftName] = useState(entry.name);
  const nameInputRef = useRef<HTMLInputElement | null>(null);

  // Focus + select-all bij openen van edit-mode.
  useEffect(() => {
    if (isEditingName && nameInputRef.current) {
      nameInputRef.current.focus();
      nameInputRef.current.select();
    }
  }, [isEditingName]);

  const startEditName = (e: ReactMouseEvent<HTMLButtonElement>): void => {
    e.stopPropagation();
    setDraftName(entry.name);
    setIsEditingName(true);
  };

  const cancelEditName = (): void => {
    setIsEditingName(false);
    setDraftName(entry.name);
  };

  const commitEditName = (): void => {
    const trimmed = draftName.trim();
    if (trimmed.length === 0 || trimmed === entry.name) {
      cancelEditName();
      return;
    }
    updateEntry(entry.id, { name: trimmed });
    setIsEditingName(false);
  };

  const handleNameKeyDown = (
    e: ReactKeyboardEvent<HTMLInputElement>,
  ): void => {
    e.stopPropagation();
    if (e.key === "Enter") {
      e.preventDefault();
      commitEditName();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEditName();
    }
  };

  return (
    <div className="flex items-center gap-4 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          {isEditingName ? (
            <input
              ref={nameInputRef}
              type="text"
              value={draftName}
              onChange={(e) => setDraftName(e.target.value)}
              onKeyDown={handleNameKeyDown}
              onBlur={commitEditName}
              onClick={(e) => e.stopPropagation()}
              onMouseDown={(e) => e.stopPropagation()}
              className={NAME_EDIT_INPUT_CLASS}
              aria-label="Constructienaam bewerken"
            />
          ) : (
            <>
              <span className="text-sm font-medium text-on-surface">
                {entry.name}
              </span>
              <button
                type="button"
                onClick={startEditName}
                className="rounded p-0.5 text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface-secondary"
                aria-label="Naam bewerken"
                title="Naam bewerken"
              >
                <Pencil className={NAME_EDIT_ICON_SIZE_CLASS} />
              </button>
            </>
          )}
        </div>
        <div className="mt-0.5 flex items-center gap-3 text-xs text-on-surface-muted">
          <span>
            U = {entry.uValue} W/(m{"\u00B2"}{"\u00B7"}K)
          </span>
          {entry.layers && entry.layers.length > 0 && (
            <span>{entry.layers.length} lagen</span>
          )}
        </div>
      </div>
      <div className="shrink-0">
        {inProject ? (
          <span className="rounded bg-teal-50 px-3 py-1 text-xs font-medium text-teal-700">
            In project
          </span>
        ) : (
          <button
            onClick={onCopy}
            className="rounded oa-accent-chip px-3 py-1 text-xs font-medium"
          >
            Toevoegen aan project
          </button>
        )}
      </div>
    </div>
  );
}
