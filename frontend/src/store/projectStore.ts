import { create } from "zustand";
import { persist } from "zustand/middleware";

import type {
  AggregationMethod,
  ConstructionElement,
  ConstructionElementLayer,
  HeatingSystem,
  MaterialType,
  Project,
  ProjectResult,
  Room,
  VerticalPosition,
} from "../types";
import type { Isso53ProjectResult } from "../types/isso53Result";
import { isIsso53Heating } from "../lib/normSwitch";
import {
  DEFAULT_ISSO53_BUILDING,
  DEFAULT_ISSO53_ROOM,
  DEFAULT_SHARED_EXTRA,
  normalizeIsso53HeatingUp,
  type ActiveNorm,
  type Isso53BuildingState,
  type Isso53RoomState,
  type SharedExtra,
} from "../types/projectV2";

// ---------------------------------------------------------------------------
// Undo/Redo history
// ---------------------------------------------------------------------------

const MAX_HISTORY = 50;

interface ProjectSnapshot {
  project: Project;
}

function takeProjectSnapshot(state: { project: Project }): ProjectSnapshot {
  return { project: structuredClone(state.project) };
}

/** Detecteer de norm van een geladen project uit de verwarmings-shape:
 * camelCase ISSO 53-waarden (bv. radiatorenConvLt) => isso53, anders isso51. */
function detectNormFromProject(project: Project): ActiveNorm {
  const heatings = [
    project.building?.default_heating_system,
    ...project.rooms.map((r) => r.heating_system),
  ];
  return heatings.some((h) => isIsso53Heating(h)) ? "isso53" : "isso51";
}

/** Default project for a new calculation. */
const DEFAULT_PROJECT: Project = {
  info: {
    name: "",
  },
  building: {
    building_type: "terraced",
    qv10: 100,
    total_floor_area: 80,
    security_class: "b",
    has_night_setback: true,
    warmup_time: 2,
    num_floors: 1,
    default_heating_system: "radiator_ht",
    aggregation_method: "vabi_compat",
  },
  climate: {
    theta_e: -10,
    theta_b_residential: 17,
    theta_b_non_residential: 14,
    wind_factor: 1.0,
    theta_water: 5,
  },
  ventilation: {
    system_type: "system_c",
    has_heat_recovery: false,
  },
  rooms: [],
};

interface ProjectStore {
  /** Current project input data (V1 schema, single source of truth). */
  project: Project;
  /**
   * V2-only sidecar velden (ADR-002 SharedProject extras). Worden
   * gepersisteerd maar niet meegestuurd naar backend tot V2 endpoint
   * live is. Zie `lib/projectV2Migration.ts` voor de mapping.
   */
  sharedExtra: SharedExtra;
  /**
   * Actieve norm voor dit project. Bepaalt welke ISSO-rekenkern wordt
   * aangeroepen (`calcs.isso51` of `calcs.isso53`) en welke UI-elementen
   * worden getoond. Wordt vastgelegd bij project-aanmaak (fase 2);
   * wissel-flow volgt in fase 4. Default `"isso51"` voor backward-compat
   * met bestaande projecten zonder norm-veld (silent migration).
   */
  norm: ActiveNorm;
  /**
   * ISSO 53 building-niveau sidecar-velden (BuildingShape,
   * BuildingPosition, WindPressureType, ThermalMass, VentilationSystem,
   * ConstructionYear). Alleen actief wanneer `norm === "isso53"`.
   * Fase 3: lokale UI-state; backend-routing volgt in fase 4/5.
   */
  isso53Building: Isso53BuildingState;
  /**
   * Per-ruimte ISSO 53 sidecar (gebruiksFunctie + ruimteType), gekeyed
   * op `room.id`. Alleen actief wanneer `norm === "isso53"`.
   */
  isso53Rooms: Record<string, Isso53RoomState>;
  /**
   * Calculation result (null if not yet calculated). Houdt een ISSO 51
   * (`ProjectResult`) of ISSO 53 (`Isso53ProjectResult`) resultaat —
   * consumers discrimineren op `norm`, niet op het result-shape zelf.
   */
  result: ProjectResult | Isso53ProjectResult | null;
  /** Error message from last calculation attempt. */
  error: string | null;
  /** Whether a calculation is in progress. */
  isCalculating: boolean;
  /** Whether the project has unsaved changes since last calculation. */
  isDirty: boolean;
  /** Server-side project ID (null for local-only projects). */
  activeProjectId: string | null;
  /** Server-side updated_at timestamp for conflict detection. */
  serverUpdatedAt: string | null;
  /** Whether a save conflict was detected. */
  hasConflict: boolean;
  /**
   * Local filesystem path waar dit project nu naartoe geschreven kan worden
   * (Tauri-mode). Wordt geset wanneer:
   *   - de user een `.ifcenergy` opent via de Tauri open-dialog
   *   - de app wordt gestart via file-association (argv pad)
   *   - de user "Opslaan als…" doet
   * Bestand → Opslaan schrijft stil naar dit pad als het bekend is;
   * anders valt het terug op de save-as dialog.
   */
  currentLocalPath: string | null;

  /** Update V2 sidecar velden (partial merge). */
  updateSharedExtra: (partial: Partial<SharedExtra>) => void;
  /** Vervang volledige sidecar (gebruikt bij load van V2 JSON). */
  setSharedExtra: (extra: SharedExtra) => void;
  /**
   * Zet de actieve norm. Wordt aangeroepen door de Backstage NormChoiceModal
   * bij nieuw-project en (in fase 4) door de wissel-flow.
   */
  setNorm: (norm: ActiveNorm) => void;

  /** Partial merge op `isso53Building` (fase 3 sidecar). */
  updateIsso53Building: (partial: Partial<Isso53BuildingState>) => void;
  /** Set of update een per-ruimte ISSO 53 sidecar (partial merge). */
  updateIsso53Room: (
    roomId: string,
    partial: Partial<Isso53RoomState>,
  ) => void;

  /** Undo history (not persisted). */
  _past: ProjectSnapshot[];
  /** Redo history (not persisted). */
  _future: ProjectSnapshot[];

  /** Update project data (partial merge). */
  updateProject: (partial: Partial<Project>) => void;
  /**
   * Zet (of wist) de project-brede override voor de U-waarde van
   * kozijnen. Geef `undefined` mee om de override te wissen — de
   * individuele per-element waardes blijven dan intact.
   */
  setFrameUValueOverride: (value: number | undefined) => void;
  /**
   * Replace the entire project.
   *
   * `opts` is optioneel en wordt gevuld door de import-flow voor
   * ISSO 53-bestanden die de norm + sidecar-state expliciet meedragen:
   *   - `norm` — autoritatief boven heating-shape-detectie. Afwezig →
   *     detectie + (huidig) "nooit downgraden" gedrag.
   *   - `isso53Building` / `isso53Rooms` — herstellen de sidecar-state
   *     i.p.v. naar defaults te resetten. Afwezig → defaults (huidig
   *     gedrag voor oude bestanden zonder sidecars).
   */
  setProject: (
    project: Project,
    opts?: {
      norm?: ActiveNorm;
      isso53Building?: Isso53BuildingState;
      isso53Rooms?: Record<string, Isso53RoomState>;
    },
  ) => void;
  /** Set the active server-side project ID. */
  setActiveProjectId: (id: string | null) => void;
  /** Set the local filesystem path (or clear with null on New). */
  setCurrentLocalPath: (path: string | null) => void;
  /** Set the calculation result. */
  setResult: (result: ProjectResult | Isso53ProjectResult) => void;
  /** Set an error from a failed calculation. */
  setError: (error: string) => void;
  /** Clear the current error. */
  clearError: () => void;
  /** Set calculating state. */
  setCalculating: (isCalculating: boolean) => void;
  /** Load a server project atomically (project + id + result in one set). */
  loadServerProject: (
    id: string,
    project: Project,
    result: ProjectResult | null,
    updatedAt?: string,
  ) => void;
  /** Update the server timestamp after a successful save. */
  setServerUpdatedAt: (updatedAt: string | null) => void;
  /** Reset to default state. */
  reset: () => void;

  /** Add a room to the project. */
  addRoom: (room: Room) => void;
  /** Update a room by id (partial merge). */
  updateRoom: (roomId: string, partial: Partial<Room>) => void;
  /** Remove a room by id. */
  removeRoom: (roomId: string) => void;
  /** Add a construction to a room. */
  addConstruction: (roomId: string, construction: ConstructionElement) => void;
  /** Update a construction in a room (partial merge). */
  updateConstruction: (
    roomId: string,
    constructionId: string,
    partial: Partial<ConstructionElement>,
  ) => void;
  /** Remove a construction from a room. */
  removeConstruction: (roomId: string, constructionId: string) => void;

  /** Apply a heating_system to all rooms in the project in one mutation (with undo). */
  applyHeatingSystemToAllRooms: (system: HeatingSystem) => void;

  /**
   * Propageer een bewerkte ProjectConstruction naar álle room-elementen die
   * eraan gekoppeld zijn (`project_construction_id === pcId`). Overschrijft
   * uitsluitend de type-definiërende velden (`description`, `u_value`,
   * `material_type`, `vertical_position`, `layers`). Element-specifieke velden
   * (`id`, `area`, `boundary_type`, `adjacent_room_id`, `uw_breakdown`, …)
   * blijven ongemoeid. Undo-aware: één undo-stap herstelt alle elementen.
   */
  syncProjectConstruction: (
    pcId: string,
    values: {
      description: string;
      u_value: number;
      material_type: MaterialType;
      vertical_position: VerticalPosition;
      layers: ConstructionElementLayer[];
    },
  ) => void;

  /**
   * Zet de aggregatiemethode voor `Φ_basis_gebouw`. Schakelt tussen
   * Vabi-conform (markt-default) en ISSO 51 §3.5.1 letterlijk. Undo-aware.
   */
  setAggregationMethod: (method: AggregationMethod) => void;

  /** Undo last project mutation. */
  undo: () => void;
  /** Redo last undone project mutation. */
  redo: () => void;
}

export const useProjectStore = create<ProjectStore>()(
  persist(
    (set, get) => ({
      project: DEFAULT_PROJECT,
      sharedExtra: { ...DEFAULT_SHARED_EXTRA },
      norm: "isso51",
      isso53Building: { ...DEFAULT_ISSO53_BUILDING },
      isso53Rooms: {},
      result: null,
      error: null,
      isCalculating: false,
      isDirty: true,
      activeProjectId: null,
      serverUpdatedAt: null,
      hasConflict: false,
      currentLocalPath: null,
      _past: [],
      _future: [],

      updateSharedExtra: (partial) =>
        set((state) => ({
          sharedExtra: { ...state.sharedExtra, ...partial },
          isDirty: true,
        })),

      setSharedExtra: (extra) => set({ sharedExtra: extra }),

      setNorm: (norm) => set({ norm, isDirty: true }),

      updateIsso53Building: (partial) =>
        set((state) => ({
          isso53Building: { ...state.isso53Building, ...partial },
          isDirty: true,
        })),

      updateIsso53Room: (roomId, partial) =>
        set((state) => {
          const current = state.isso53Rooms[roomId];
          const base: Isso53RoomState = current ?? { ...DEFAULT_ISSO53_ROOM };
          return {
            isso53Rooms: {
              ...state.isso53Rooms,
              [roomId]: { ...base, ...partial },
            },
            isDirty: true,
          };
        }),

      setActiveProjectId: (id) => set({ activeProjectId: id }),
      setServerUpdatedAt: (updatedAt) => set({ serverUpdatedAt: updatedAt }),
      setCurrentLocalPath: (path) => set({ currentLocalPath: path }),

      updateProject: (partial) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: { ...state.project, ...partial },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      setFrameUValueOverride: (value) => {
        const snap = takeProjectSnapshot(get());
        set((state) => {
          const next: Project = { ...state.project };
          if (value === undefined || !Number.isFinite(value) || value <= 0) {
            delete next.frameUValueOverride;
          } else {
            next.frameUValueOverride = value;
          }
          return {
            project: next,
            isDirty: true,
            error: null,
            _past: [...state._past, snap].slice(-MAX_HISTORY),
            _future: [],
          };
        });
      },

      setProject: (project, opts) =>
        set((state) => {
          // Expliciete envelope-norm is autoritatief: een ISSO 53-bestand
          // draagt zijn norm + sidecars mee en moet die exact herstellen.
          // Zonder expliciete norm → val terug op het bestaande gedrag:
          // detecteer uit de verwarmings-shape en downgrade NOOIT (een
          // gebruiker in ISSO 53-modus die een isso51-vormig bestand
          // importeert wil niet teruggezet worden naar ISSO 51).
          const norm: ActiveNorm =
            opts?.norm ??
            (detectNormFromProject(project) === "isso53" ? "isso53" : state.norm);
          return {
            project,
            sharedExtra: { ...DEFAULT_SHARED_EXTRA },
            norm,
            // Sidecars uit de envelope herstellen indien meegegeven, anders
            // resetten naar defaults (huidig gedrag voor oude bestanden).
            isso53Building: opts?.isso53Building ?? { ...DEFAULT_ISSO53_BUILDING },
            isso53Rooms: opts?.isso53Rooms ?? {},
            isDirty: true,
            result: null,
            error: null,
            activeProjectId: null,
            serverUpdatedAt: null,
            hasConflict: false,
            currentLocalPath: null,
            _past: [],
            _future: [],
          };
        }),

      loadServerProject: (id, project, result, updatedAt) =>
        set({
          project,
          // Detecteer de norm uit de verwarmings-shape (zie setProject) —
          // server-projecten met ISSO 53 heating mogen niet naar ISSO 51.
          norm: detectNormFromProject(project),
          isso53Building: { ...DEFAULT_ISSO53_BUILDING },
          isso53Rooms: {},
          activeProjectId: id,
          result,
          isDirty: false,
          error: null,
          isCalculating: false,
          serverUpdatedAt: updatedAt ?? null,
          hasConflict: false,
          currentLocalPath: null,
          _past: [],
          _future: [],
        }),

      setResult: (result) =>
        set({ result, isDirty: false, error: null, isCalculating: false }),

      setError: (error) =>
        set({ error, isCalculating: false }),

      clearError: () =>
        set({ error: null }),

      setCalculating: (isCalculating) =>
        set({ isCalculating }),

      reset: () =>
        set({
          project: DEFAULT_PROJECT,
          sharedExtra: { ...DEFAULT_SHARED_EXTRA },
          // Reset valt terug op de default norm — NormChoiceModal in
          // Backstage zet hierna de gekozen norm via `setNorm`.
          norm: "isso51",
          isso53Building: { ...DEFAULT_ISSO53_BUILDING },
          isso53Rooms: {},
          result: null,
          error: null,
          isCalculating: false,
          isDirty: true,
          activeProjectId: null,
          serverUpdatedAt: null,
          hasConflict: false,
          currentLocalPath: null,
          _past: [],
          _future: [],
        }),

      addRoom: (room) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: [...state.project.rooms, room],
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      updateRoom: (roomId, partial) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId ? { ...r, ...partial } : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      removeRoom: (roomId) => {
        const snap = takeProjectSnapshot(get());
        set((state) => {
          const { [roomId]: _removed, ...remainingIsso53 } = state.isso53Rooms;
          return {
            project: {
              ...state.project,
              rooms: state.project.rooms.filter((r) => r.id !== roomId),
            },
            isso53Rooms: remainingIsso53,
            isDirty: true,
            error: null,
            _past: [...state._past, snap].slice(-MAX_HISTORY),
            _future: [],
          };
        });
      },

      addConstruction: (roomId, construction) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId
                ? { ...r, constructions: [...r.constructions, construction] }
                : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      updateConstruction: (roomId, constructionId, partial) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId
                ? {
                    ...r,
                    constructions: r.constructions.map((c) =>
                      c.id === constructionId ? { ...c, ...partial } : c,
                    ),
                  }
                : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      removeConstruction: (roomId, constructionId) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId
                ? {
                    ...r,
                    constructions: r.constructions.filter(
                      (c) => c.id !== constructionId,
                    ),
                  }
                : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      applyHeatingSystemToAllRooms: (system) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) => ({
              ...r,
              heating_system: system,
            })),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      syncProjectConstruction: (pcId, values) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) => ({
              ...r,
              constructions: r.constructions.map((c) =>
                c.project_construction_id === pcId
                  ? {
                      ...c,
                      description: values.description,
                      u_value: values.u_value,
                      material_type: values.material_type,
                      vertical_position: values.vertical_position,
                      layers:
                        values.layers.length > 0
                          ? values.layers.map((l) => ({ ...l }))
                          : undefined,
                    }
                  : c,
              ),
            })),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      setAggregationMethod: (method) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            building: {
              ...state.project.building,
              aggregation_method: method,
            },
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      undo: () => {
        const state = get();
        if (state._past.length === 0) return;
        const currentSnap = takeProjectSnapshot(state);
        const prev = state._past[state._past.length - 1]!;
        set({
          project: prev.project,
          _past: state._past.slice(0, -1),
          _future: [...state._future, currentSnap],
          isDirty: true,
        });
      },

      redo: () => {
        const state = get();
        if (state._future.length === 0) return;
        const currentSnap = takeProjectSnapshot(state);
        const next = state._future[state._future.length - 1]!;
        set({
          project: next.project,
          _past: [...state._past, currentSnap],
          _future: state._future.slice(0, -1),
          isDirty: true,
        });
      },
    }),
    {
      name: "isso51-project",
      version: 1,
      partialize: (state) => ({
        project: state.project,
        sharedExtra: state.sharedExtra,
        norm: state.norm,
        isso53Building: state.isso53Building,
        isso53Rooms: state.isso53Rooms,
        result: state.result,
      }),
      merge: (persisted, current) => ({
        ...current,
        ...(persisted as Pick<
          ProjectStore,
          | "project"
          | "sharedExtra"
          | "norm"
          | "isso53Building"
          | "isso53Rooms"
          | "result"
        >),
        sharedExtra:
          (persisted as Partial<ProjectStore>)?.sharedExtra ?? current.sharedExtra,
        // Silent migration voor gepersisteerde projecten van vóór fase 2.
        norm: (persisted as Partial<ProjectStore>)?.norm ?? "isso51",
        // Silent migration voor projecten van vóór fase 3 (geen ISSO 53 sidecar).
        // Plus §4.8-migratie: normaliseer de heatingUp-blob (vervallen
        // pWPerM2/warmupMinutes → pWPerM2Override + defaults).
        isso53Building: (() => {
          const persistedBuilding = (persisted as Partial<ProjectStore>)
            ?.isso53Building;
          if (!persistedBuilding) return current.isso53Building;
          return {
            ...persistedBuilding,
            heatingUp: normalizeIsso53HeatingUp(persistedBuilding.heatingUp),
          };
        })(),
        isso53Rooms:
          (persisted as Partial<ProjectStore>)?.isso53Rooms ?? current.isso53Rooms,
        isDirty: false,
        isCalculating: false,
        error: null,
      }),
    },
  ),
);
