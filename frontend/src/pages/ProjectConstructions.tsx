/**
 * Project Constructions page — manages per-project construction library.
 *
 * Shows all constructions specific to this project (copied from catalogue,
 * imported from IFC, or created via Rc-calculator). Users can browse the
 * standard catalogue and copy entries into their project.
 */
import { useMemo, useState } from "react";

import { useModellerStore } from "../components/modeller/modellerStore";
import type { ProjectConstruction } from "../components/modeller/types";
import { useCatalogueStore } from "../store/catalogueStore";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
  type CatalogueEntry,
} from "../lib/constructionCatalogue";
import { calculateRc } from "../lib/rcCalculation";

const CATEGORY_ORDER: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "daken",
  "kozijnen_vullingen",
];

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

  // Count assignments for each project construction
  const wallConstructions = useModellerStore((s) => s.wallConstructions);
  const floorConstructions = useModellerStore((s) => s.floorConstructions);
  const roofConstructions = useModellerStore((s) => s.roofConstructions);

  const assignmentCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const id of Object.values(wallConstructions)) {
      counts[id] = (counts[id] ?? 0) + 1;
    }
    for (const id of Object.values(floorConstructions)) {
      counts[id] = (counts[id] ?? 0) + 1;
    }
    for (const id of Object.values(roofConstructions)) {
      counts[id] = (counts[id] ?? 0) + 1;
    }
    return counts;
  }, [wallConstructions, floorConstructions, roofConstructions]);

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
        ? "border-b-2 border-amber-500 text-amber-900"
        : "text-stone-400 hover:text-stone-600"
    }`;

  return (
    <div className="mx-auto max-w-4xl p-6">
      <h1 className="mb-1 text-xl font-bold text-stone-800">Constructies</h1>
      <p className="mb-4 text-sm text-stone-500">
        Beheer de constructies voor dit project. Kopieer vanuit de standaard
        bibliotheek of maak nieuwe constructies aan via de Rc-waarde tool.
      </p>

      {/* Tab strip */}
      <div className="mb-4 flex border-b border-stone-200">
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
          className="w-full max-w-sm rounded border border-stone-200 px-3 py-1.5 text-sm outline-none focus:border-amber-400"
        />
      </div>

      {/* Project tab */}
      {tab === "project" && (
        <>
          {projectConstructions.length === 0 ? (
            <div className="rounded border border-dashed border-stone-300 px-6 py-8 text-center">
              <p className="text-sm text-stone-500">
                Nog geen constructies in dit project.
              </p>
              <p className="mt-1 text-xs text-stone-400">
                Ga naar het tabblad "Standaard bibliotheek" om constructies toe
                te voegen, of maak een nieuwe aan via de{" "}
                <a href="/rc" className="text-amber-600 hover:underline">
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
                  <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-400">
                    {CATALOGUE_CATEGORY_LABELS[cat]}
                  </h2>
                  <div className="space-y-2">
                    {entries.map((pc) => {
                      const rcResult = calculateRc(pc.layers, pc.verticalPosition);
                      const uValue = Math.round(rcResult.uValue * 1000) / 1000;
                      const count = assignmentCounts[pc.id] ?? 0;

                      return (
                        <div
                          key={pc.id}
                          className="flex items-center gap-4 rounded border border-stone-200 bg-white px-4 py-3"
                        >
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                              <span className="text-sm font-medium text-stone-800">
                                {pc.name}
                              </span>
                            </div>
                            <div className="mt-0.5 flex items-center gap-3 text-xs text-stone-500">
                              <span>
                                U = {uValue} W/(m{"\u00B2"}{"\u00B7"}K)
                              </span>
                              <span>{pc.layers.length} lagen</span>
                              {count > 0 && (
                                <span className="text-green-600">
                                  {count}x toegewezen
                                </span>
                              )}
                              {pc.catalogueSourceId && (
                                <span className="text-stone-400">
                                  Bron: catalogus
                                </span>
                              )}
                              {pc.ifcSource && (
                                <span className="text-stone-400">
                                  IFC: {pc.ifcSource.wallTypeName}
                                </span>
                              )}
                            </div>
                          </div>
                          <div className="flex shrink-0 items-center gap-2">
                            <a
                              href="/rc"
                              className="rounded border border-stone-200 px-2.5 py-1 text-xs text-stone-600 hover:bg-stone-50"
                            >
                              Bewerken
                            </a>
                            {confirmDelete === pc.id ? (
                              <div className="flex gap-1">
                                <button
                                  onClick={() => {
                                    removeProjectConstruction(pc.id);
                                    setConfirmDelete(null);
                                  }}
                                  className="rounded bg-red-50 px-2.5 py-1 text-xs text-red-600 hover:bg-red-100"
                                >
                                  Bevestig
                                </button>
                                <button
                                  onClick={() => setConfirmDelete(null)}
                                  className="rounded px-2.5 py-1 text-xs text-stone-400 hover:bg-stone-50"
                                >
                                  Annuleer
                                </button>
                              </div>
                            ) : (
                              <button
                                onClick={() => setConfirmDelete(pc.id)}
                                className="rounded border border-stone-200 px-2.5 py-1 text-xs text-red-500 hover:bg-red-50"
                              >
                                Verwijderen
                              </button>
                            )}
                          </div>
                        </div>
                      );
                    })}
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
                <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-400">
                  {CATALOGUE_CATEGORY_LABELS[cat]}
                </h2>
                <div className="space-y-2">
                  {entries.map((entry) => {
                    const inProject = isInProject(entry.id);
                    const hasLayers = !!entry.layers?.length;

                    return (
                      <div
                        key={entry.id}
                        className="flex items-center gap-4 rounded border border-stone-200 bg-white px-4 py-3"
                      >
                        <div className="min-w-0 flex-1">
                          <div className="text-sm font-medium text-stone-800">
                            {entry.name}
                          </div>
                          <div className="mt-0.5 flex items-center gap-3 text-xs text-stone-500">
                            <span>
                              U = {entry.uValue} W/(m{"\u00B2"}{"\u00B7"}K)
                            </span>
                            {entry.layers && (
                              <span>{entry.layers.length} lagen</span>
                            )}
                          </div>
                        </div>
                        <div className="shrink-0">
                          {inProject ? (
                            <span className="rounded bg-teal-50 px-3 py-1 text-xs font-medium text-teal-700">
                              In project
                            </span>
                          ) : hasLayers ? (
                            <button
                              onClick={() => copyFromCatalogue(entry)}
                              className="rounded bg-amber-50 px-3 py-1 text-xs font-medium text-amber-700 hover:bg-amber-100"
                            >
                              Toevoegen aan project
                            </button>
                          ) : (
                            <span className="text-xs text-stone-400">
                              Geen laagopbouw
                            </span>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </>
      )}
    </div>
  );
}
