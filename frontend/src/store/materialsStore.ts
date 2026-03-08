import { create } from "zustand";
import { persist } from "zustand/middleware";

import {
  MATERIALS_DATABASE,
  MATERIAL_CATEGORY_ORDER,
  type Material,
  type MaterialCategory,
} from "../lib/materialsDatabase";

const BUILT_IN_MAP = new Map(
  MATERIALS_DATABASE.map((m) => [m.id, m]),
);

/** Shape persisted to localStorage (delta only). */
interface PersistedDelta {
  customMaterials: Material[];
  modifiedBuiltIns: Material[];
  deletedBuiltInIds: string[];
}

interface MaterialsStore {
  /** All materials (built-in + custom). */
  materials: Material[];

  /** Add a custom material. */
  addMaterial: (material: Omit<Material, "id" | "isBuiltIn">) => void;
  /** Update an existing material. */
  updateMaterial: (id: string, partial: Partial<Material>) => void;
  /** Remove a material by id. */
  removeMaterial: (id: string) => void;

  /** Reset a single built-in material to its default values. */
  resetMaterial: (id: string) => void;
  /** Reset all materials to factory defaults. */
  resetAll: () => void;
  /** Check if a built-in material has been modified. */
  isModified: (id: string) => boolean;

  /** Get materials grouped by category in display order. */
  byCategory: () => Map<MaterialCategory, Material[]>;
}

/** Reconstruct full materials array from defaults + persisted delta. */
function mergeWithDefaults(delta: PersistedDelta): Material[] {
  const deletedSet = new Set(delta.deletedBuiltInIds);
  const modifiedMap = new Map(
    delta.modifiedBuiltIns.map((m) => [m.id, m]),
  );

  const builtIns = MATERIALS_DATABASE
    .filter((m) => !deletedSet.has(m.id))
    .map((m) => modifiedMap.get(m.id) ?? m);

  return [...builtIns, ...delta.customMaterials];
}

/** Check if a built-in material differs from its default. */
function materialDiffersFromDefault(material: Material): boolean {
  const def = BUILT_IN_MAP.get(material.id);
  if (!def) return false;
  return (
    material.name !== def.name ||
    material.brand !== def.brand ||
    material.lambda !== def.lambda ||
    material.lambdaWet !== def.lambdaWet ||
    material.mu !== def.mu ||
    material.rho !== def.rho ||
    material.category !== def.category
  );
}

const STORAGE_KEY = "isso51-materials";

export const useMaterialsStore = create<MaterialsStore>()(
  persist(
    (set, get) => ({
      materials: [...MATERIALS_DATABASE],

      addMaterial: (material) =>
        set((state) => ({
          materials: [
            ...state.materials,
            { ...material, id: crypto.randomUUID(), isBuiltIn: false },
          ],
        })),

      updateMaterial: (id, partial) =>
        set((state) => ({
          materials: state.materials.map((m) =>
            m.id === id ? { ...m, ...partial } : m,
          ),
        })),

      removeMaterial: (id) =>
        set((state) => ({
          materials: state.materials.filter((m) => m.id !== id),
        })),

      resetMaterial: (id) =>
        set((state) => {
          const def = BUILT_IN_MAP.get(id);
          if (!def) return state;
          return {
            materials: state.materials.map((m) => (m.id === id ? { ...def } : m)),
          };
        }),

      resetAll: () =>
        set({ materials: [...MATERIALS_DATABASE] }),

      isModified: (id) => {
        const material = get().materials.find((m) => m.id === id);
        if (!material || !material.isBuiltIn) return false;
        return materialDiffersFromDefault(material);
      },

      byCategory: () => {
        const map = new Map<MaterialCategory, Material[]>();
        // Initialize in display order
        for (const cat of MATERIAL_CATEGORY_ORDER) {
          map.set(cat, []);
        }
        for (const m of get().materials) {
          const list = map.get(m.category);
          if (list) {
            list.push(m);
          } else {
            map.set(m.category, [m]);
          }
        }
        // Remove empty categories
        for (const [cat, list] of map) {
          if (list.length === 0) map.delete(cat);
        }
        return map;
      },
    }),
    {
      name: STORAGE_KEY,
      version: 1,

      partialize: (state): PersistedDelta => {
        const customMaterials: Material[] = [];
        const modifiedBuiltIns: Material[] = [];
        const presentBuiltInIds = new Set<string>();

        for (const material of state.materials) {
          if (material.isBuiltIn) {
            presentBuiltInIds.add(material.id);
            if (materialDiffersFromDefault(material)) {
              modifiedBuiltIns.push(material);
            }
          } else {
            customMaterials.push(material);
          }
        }

        const deletedBuiltInIds = MATERIALS_DATABASE
          .map((m) => m.id)
          .filter((id) => !presentBuiltInIds.has(id));

        return { customMaterials, modifiedBuiltIns, deletedBuiltInIds };
      },

      merge: (persisted, currentState) => {
        const delta = persisted as PersistedDelta | undefined;
        if (!delta || !delta.customMaterials) {
          return currentState;
        }
        return {
          ...currentState,
          materials: mergeWithDefaults(delta),
        };
      },
    },
  ),
);
