import { create } from "zustand";
import { persist } from "zustand/middleware";

import {
  CONSTRUCTION_CATALOGUE,
  type CatalogueCategory,
  type CatalogueEntry,
} from "../lib/constructionCatalogue";

const BUILT_IN_MAP = new Map(
  CONSTRUCTION_CATALOGUE.map((e) => [e.id, e]),
);

/** Shape persisted to localStorage (delta only). */
interface PersistedDelta {
  customEntries: CatalogueEntry[];
  modifiedBuiltIns: CatalogueEntry[];
  deletedBuiltInIds: string[];
}

interface CatalogueStore {
  /** All catalogue entries (built-in + custom). */
  entries: CatalogueEntry[];

  /** Add a custom entry to the catalogue. */
  addEntry: (entry: Omit<CatalogueEntry, "id" | "isBuiltIn">) => void;
  /** Update an existing entry. */
  updateEntry: (id: string, partial: Partial<CatalogueEntry>) => void;
  /** Remove an entry by id. */
  removeEntry: (id: string) => void;
  /** Duplicate an existing entry. */
  duplicateEntry: (id: string) => void;

  /** Reset a single built-in entry to its default values. */
  resetEntry: (id: string) => void;
  /** Reset all entries to factory defaults. */
  resetAll: () => void;
  /** Check if a built-in entry has been modified. */
  isModified: (id: string) => boolean;

  /** Get entries grouped by category. */
  byCategory: () => Map<CatalogueCategory, CatalogueEntry[]>;
}

/** Reconstruct full entries array from defaults + persisted delta. */
function mergeWithDefaults(delta: PersistedDelta): CatalogueEntry[] {
  const deletedSet = new Set(delta.deletedBuiltInIds);
  const modifiedMap = new Map(
    delta.modifiedBuiltIns.map((e) => [e.id, e]),
  );

  const builtIns = CONSTRUCTION_CATALOGUE
    .filter((e) => !deletedSet.has(e.id))
    .map((e) => modifiedMap.get(e.id) ?? e);

  return [...builtIns, ...delta.customEntries];
}

/** Check if layers arrays differ. */
function layersDiffer(
  a: CatalogueEntry["layers"],
  b: CatalogueEntry["layers"],
): boolean {
  if (!a && !b) return false;
  if (!a || !b) return true;
  if (a.length !== b.length) return true;
  return a.some(
    (layer, i) =>
      layer.materialId !== b[i]?.materialId ||
      layer.thickness !== b[i]?.thickness,
  );
}

/** Check if a built-in entry differs from its default. */
function entryDiffersFromDefault(entry: CatalogueEntry): boolean {
  const def = BUILT_IN_MAP.get(entry.id);
  if (!def) return false;
  return (
    entry.name !== def.name ||
    entry.uValue !== def.uValue ||
    entry.materialType !== def.materialType ||
    entry.verticalPosition !== def.verticalPosition ||
    (entry.boundaryType ?? "exterior") !== (def.boundaryType ?? "exterior") ||
    entry.category !== def.category ||
    layersDiffer(entry.layers, def.layers)
  );
}

const STORAGE_KEY = "isso51-catalogue";

export const useCatalogueStore = create<CatalogueStore>()(
  persist(
    (set, get) => ({
      entries: [...CONSTRUCTION_CATALOGUE],

      addEntry: (entry) =>
        set((state) => ({
          entries: [
            ...state.entries,
            { ...entry, id: crypto.randomUUID(), isBuiltIn: false },
          ],
        })),

      updateEntry: (id, partial) =>
        set((state) => ({
          entries: state.entries.map((e) =>
            e.id === id ? { ...e, ...partial } : e,
          ),
        })),

      removeEntry: (id) =>
        set((state) => ({
          entries: state.entries.filter((e) => e.id !== id),
        })),

      duplicateEntry: (id) =>
        set((state) => {
          const source = state.entries.find((e) => e.id === id);
          if (!source) return state;
          return {
            entries: [
              ...state.entries,
              {
                ...source,
                id: crypto.randomUUID(),
                name: `${source.name} (kopie)`,
                isBuiltIn: false,
              },
            ],
          };
        }),

      resetEntry: (id) =>
        set((state) => {
          const def = BUILT_IN_MAP.get(id);
          if (!def) return state;
          return {
            entries: state.entries.map((e) => (e.id === id ? { ...def } : e)),
          };
        }),

      resetAll: () =>
        set({ entries: [...CONSTRUCTION_CATALOGUE] }),

      isModified: (id) => {
        const entry = get().entries.find((e) => e.id === id);
        if (!entry || !entry.isBuiltIn) return false;
        return entryDiffersFromDefault(entry);
      },

      byCategory: () => {
        const map = new Map<CatalogueCategory, CatalogueEntry[]>();
        for (const entry of get().entries) {
          const list = map.get(entry.category) ?? [];
          list.push(entry);
          map.set(entry.category, list);
        }
        return map;
      },
    }),
    {
      name: STORAGE_KEY,
      version: 1,

      /** Store only the delta: custom entries, modified built-ins, deleted IDs. */
      partialize: (state): PersistedDelta => {
        const customEntries: CatalogueEntry[] = [];
        const modifiedBuiltIns: CatalogueEntry[] = [];
        const presentBuiltInIds = new Set<string>();

        for (const entry of state.entries) {
          if (entry.isBuiltIn) {
            presentBuiltInIds.add(entry.id);
            if (entryDiffersFromDefault(entry)) {
              modifiedBuiltIns.push(entry);
            }
          } else {
            customEntries.push(entry);
          }
        }

        const deletedBuiltInIds = CONSTRUCTION_CATALOGUE
          .map((e) => e.id)
          .filter((id) => !presentBuiltInIds.has(id));

        return { customEntries, modifiedBuiltIns, deletedBuiltInIds };
      },

      /** Reconstruct full state from defaults + persisted delta. */
      merge: (persisted, currentState) => {
        const delta = persisted as PersistedDelta | undefined;
        if (!delta || !delta.customEntries) {
          return currentState;
        }
        return {
          ...currentState,
          entries: mergeWithDefaults(delta),
        };
      },
    },
  ),
);
