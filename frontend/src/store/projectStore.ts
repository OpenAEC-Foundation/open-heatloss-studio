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
  Zone,
} from "../types";
import type { Isso53ProjectResult } from "../types/isso53Result";
import type { EnergyInput } from "../types/beng";
import type { BengGeometry } from "../types/bengGeometry";
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
import type {
  VentilationRoomState,
  VentilationState,
  VentilationSystemKey,
  VentilationTerminal,
  VentilationUnit,
  VentilationUnitAssignment,
} from "../types/ventilation";
import { VENTILATION_SYSTEMS } from "../types/ventilation";
import { useSaveStatusStore } from "./saveStatusStore";

// ---------------------------------------------------------------------------
// Undo/Redo history
// ---------------------------------------------------------------------------

const MAX_HISTORY = 50;

/**
 * Undo-snapshot. Naast het project zelf ook de per-ruimte sidecars
 * (`isso53Rooms` + `ventilation`): `removeRoom` schoont die mee op, dus
 * een undo moet ze ook mee terugzetten — anders is per-ruimte config stil
 * weg na een undo (audit 09 §2.1).
 *
 * `isso53Building` hoort hier bewust NIET bij: geen enkele history-tracked
 * mutatie raakt het, en meenemen zou een undo van een room-mutatie stil
 * latere (untracked) gebouw-instellingen terugdraaien.
 */
interface ProjectSnapshot {
  project: Project;
  isso53Rooms: Record<string, Isso53RoomState>;
  ventilation: VentilationState;
}

function takeProjectSnapshot(state: {
  project: Project;
  isso53Rooms: Record<string, Isso53RoomState>;
  ventilation: VentilationState;
}): ProjectSnapshot {
  return {
    project: structuredClone(state.project),
    isso53Rooms: structuredClone(state.isso53Rooms),
    ventilation: structuredClone(state.ventilation),
  };
}

/** Detecteer de norm van een geladen project uit de verwarmings-shape:
 * camelCase ISSO 53-waarden (bv. radiatorenConvLt) => isso53, anders isso51. */
export function detectNormFromProject(project: Project): ActiveNorm {
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
   * Ventilatiebalans-sidecar (frontend, geen Rust): ventielen + per-room
   * ventilatie-velden (gebruiksfunctie + afgeleide BBL-eisen in dm³/s).
   * Leeft buiten het V1 `Project` type en wordt mee-geserialiseerd in de
   * opslag-envelope (zie `importExport.ts`) zodat het een save→reopen
   * overleeft (valkuil commit `8ccff9f`).
   */
  ventilation: VentilationState;
  /**
   * NTA 8800 / BENG installatie-invoerblok (`ProjectV2::energy`). Additief op
   * het project en alleen door de BENG-tab gebruikt; `null` = nog niets
   * ingevuld. Wordt gepersisteerd (localStorage) maar reist nog NIET mee in de
   * server-/`.ifcenergy`-envelope (F4c). Bij projectwissel/-reset teruggezet
   * naar `null` zodat installatie-invoer niet naar een ander project lekt.
   */
  energy: EnergyInput | null;
  /**
   * NTA 8800 / BENG gevel-georiënteerde geometrie-invoerblok
   * (`ProjectV2::beng_geometry`, F6). Additief op het project en alleen door de
   * BENG-tab gebruikt; `null` = nog niets ingevuld. Zelfde levensloop als
   * {@link ProjectStore.energy}: gepersisteerd (localStorage), reist nog NIET
   * mee in de server-/`.ifcenergy`-envelope, en wordt bij projectwissel/-reset
   * teruggezet naar `null` zodat gevel-invoer niet naar een ander project lekt.
   */
  bengGeometry: BengGeometry | null;
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
   * Merge een partial op het BENG `energy`-blok (bootstrapt `{}` als er nog
   * niets is). Zet `isDirty`.
   */
  updateEnergy: (partial: Partial<EnergyInput>) => void;
  /** Vervang (of wis met `null`) het volledige BENG `energy`-blok. */
  setEnergy: (energy: EnergyInput | null) => void;
  /**
   * Merge een partial op het `beng_geometry`-blok (bootstrapt `{}` als er nog
   * niets is). Zelfde merge-semantiek als {@link ProjectStore.updateEnergy}:
   * `undefined` = niet aanraken, expliciet `null` = wis die sleutel. Zet
   * `isDirty`.
   */
  updateBengGeometry: (partial: Partial<BengGeometry>) => void;
  /** Vervang (of wis met `null`) het volledige `beng_geometry`-blok. */
  setBengGeometry: (geometry: BengGeometry | null) => void;
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

  // -- Ventilatiebalans (frontend-sidecar) --
  /** Voeg een ventilatie-ventiel toe (genereert een id wanneer leeg). */
  addVentilationTerminal: (
    terminal: Omit<VentilationTerminal, "id"> & { id?: string },
  ) => string;
  /** Werk een ventiel bij (partial merge op id). */
  updateVentilationTerminal: (
    id: string,
    partial: Partial<Omit<VentilationTerminal, "id">>,
  ) => void;
  /** Verwijder een ventiel op id. */
  removeVentilationTerminal: (id: string) => void;
  /** Set of update de per-room ventilatie-sidecar (partial merge). */
  updateVentilationRoom: (
    roomId: string,
    partial: Partial<VentilationRoomState>,
  ) => void;
  /** Vervang de volledige ventilatie-sidecar (gebruikt bij envelope-load). */
  setVentilation: (ventilation: VentilationState) => void;
  /** Zet het gebouw-niveau ventilatiesysteem (A–D). */
  setVentilationSystem: (system: VentilationSystemKey) => void;
  /**
   * Voeg een WTW/MV-unit toe aan de project-unitbibliotheek (genereert een id
   * wanneer leeg). No-op (geeft bestaand id terug) wanneer het id al bestaat —
   * catalogus-snapshots worden maximaal één keer gekopieerd.
   */
  addVentilationUnit: (
    unit: Omit<VentilationUnit, "id"> & { id?: string },
  ) => string;
  /** Werk een unit bij (partial merge op id). */
  updateVentilationUnit: (
    id: string,
    partial: Partial<Omit<VentilationUnit, "id">>,
  ) => void;
  /** Verwijder een unit op id, inclusief de toewijzingen die ernaar wijzen. */
  removeVentilationUnit: (id: string) => void;
  /**
   * Zet het toegewezen aantal voor een unit (absoluut, upsert).
   * `aantal <= 0` verwijdert de toewijzing.
   */
  setVentilationUnitAssignment: (unitId: string, aantal: number) => void;

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
      /**
       * V2-only sidecar-velden (bouwjaar etc.) uit de envelope. Herstelt de
       * `sharedExtra`-sidecar i.p.v. naar defaults te resetten. Afwezig →
       * defaults (huidig gedrag voor oude bestanden zonder dit veld).
       */
      sharedExtra?: SharedExtra;
      /**
       * Ventilatiebalans-sidecar uit de envelope. Herstelt ventielen +
       * per-room ventilatie-velden i.p.v. naar leeg te resetten. Afwezig →
       * leeg (huidig gedrag voor bestanden zonder ventilatie-data).
       */
      ventilation?: VentilationState;
    },
  ) => void;
  /** Set the active server-side project ID. */
  setActiveProjectId: (id: string | null) => void;
  /** Set the local filesystem path (or clear with null on New). */
  setCurrentLocalPath: (path: string | null) => void;
  /**
   * Set the calculation result.
   *
   * `runEpoch` (optioneel) is de waarde van {@link getProjectInputEpoch}
   * op het moment dat de berekening startte. Wanneer de input sindsdien
   * gewijzigd is (epoch-mismatch) wordt het result wél getoond (het is
   * het meest recente dat bestaat) maar blijft `isDirty` staan — edits
   * tijdens een lopende berekening mogen niet als "clean" gemarkeerd
   * worden (audit 09 §2.1). Zonder `runEpoch` (load-/importflows die het
   * result samen met het project zetten) blijft het oude gedrag:
   * onvoorwaardelijk `isDirty: false`.
   */
  setResult: (
    result: ProjectResult | Isso53ProjectResult,
    runEpoch?: number,
  ) => void;
  /** Set an error from a failed calculation. */
  setError: (error: string) => void;
  /** Clear the current error. */
  clearError: () => void;
  /** Set calculating state. */
  setCalculating: (isCalculating: boolean) => void;
  /**
   * Load a server project atomically (project + id + result in one set).
   *
   * `opts` spiegelt {@link ProjectStore.setProject}: envelope-sidecars
   * (norm, ISSO 53, sharedExtra, ventilatie) uit de server-`project_data`
   * worden hersteld i.p.v. naar defaults gereset. Afwezig (legacy kaal
   * `project_data`) → defaults, exact het oude gedrag.
   */
  loadServerProject: (
    id: string,
    project: Project,
    result: ProjectResult | Isso53ProjectResult | null,
    updatedAt?: string,
    opts?: {
      norm?: ActiveNorm;
      isso53Building?: Isso53BuildingState;
      isso53Rooms?: Record<string, Isso53RoomState>;
      sharedExtra?: SharedExtra;
      ventilation?: VentilationState;
    },
  ) => void;
  /** Update the server timestamp after a successful save. */
  setServerUpdatedAt: (updatedAt: string | null) => void;
  /**
   * Koppel de serverbinding los: `activeProjectId`/`serverUpdatedAt`/
   * `hasConflict` wissen + save-status naar idle. Aangeroepen bij logout en
   * bij een definitief verlopen Authentik-sessie (R1): de binding wordt
   * gepersisteerd in localStorage en zou anders op een gedeelde browser
   * overerven naar de volgende ingelogde gebruiker. Het project zelf,
   * `isDirty` en `currentLocalPath` blijven staan — onopgeslagen werk wordt
   * niet weggegooid, alleen de koppeling met het serverproject vervalt.
   */
  clearServerBinding: () => void;
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

  // -- Zones (datalaag; UI volgt in een vervolg-delegatie) --
  /**
   * Voeg een zone toe aan `building.zones` (genereert een id) en geef het
   * id terug. Undo-aware.
   */
  addZone: (name: string) => string;
  /** Hernoem een zone op id. No-op wanneer het id niet bestaat. Undo-aware. */
  renameZone: (zoneId: string, name: string) => void;
  /**
   * Verwijder een zone op id en zet `zoneId` van alle ruimten die ernaar
   * verwijzen op `undefined` (geen dangling referenties). Undo-aware.
   */
  removeZone: (zoneId: string) => void;
  /**
   * Koppel een ruimte aan een zone (`zoneId`) of ontkoppel haar
   * (`undefined`). Undo-aware.
   */
  setRoomZone: (roomId: string, zoneId: string | undefined) => void;

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
      ventilation: {
        terminals: [],
        rooms: {},
      },
      energy: null,
      bengGeometry: null,
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

      updateEnergy: (partial) =>
        set((state) => {
          // Merge alleen gedefinieerde keys: `undefined` betekent "niet
          // aanraken" (voorkomt dat een stray `undefined` een bestaand
          // deelsysteem stil wist bij een spread). Expliciet `null` blijft
          // "wis dit deelsysteem" — de clear-conventie van dit blok.
          const next: EnergyInput = { ...(state.energy ?? {}) };
          for (const [key, value] of Object.entries(partial)) {
            if (value !== undefined) {
              (next as Record<string, unknown>)[key] = value;
            }
          }
          return { energy: next, isDirty: true };
        }),

      setEnergy: (energy) => set({ energy, isDirty: true }),

      updateBengGeometry: (partial) =>
        set((state) => {
          // Identieke merge-semantiek als updateEnergy: `undefined` = "niet
          // aanraken" (voorkomt dat een stray undefined een bestaande lijst
          // stil wist), expliciet `null` blijft "wis deze sleutel".
          const next: BengGeometry = { ...(state.bengGeometry ?? {}) };
          for (const [key, value] of Object.entries(partial)) {
            if (value !== undefined) {
              (next as Record<string, unknown>)[key] = value;
            }
          }
          return { bengGeometry: next, isDirty: true };
        }),

      setBengGeometry: (bengGeometry) => set({ bengGeometry, isDirty: true }),

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

      // -- Ventilatiebalans (frontend-sidecar) --
      addVentilationTerminal: (terminal) => {
        const id = terminal.id ?? `vent-${crypto.randomUUID()}`;
        set((state) => ({
          ventilation: {
            ...state.ventilation,
            terminals: [...state.ventilation.terminals, { ...terminal, id }],
          },
          isDirty: true,
        }));
        return id;
      },

      updateVentilationTerminal: (id, partial) =>
        set((state) => ({
          ventilation: {
            ...state.ventilation,
            terminals: state.ventilation.terminals.map((t) =>
              t.id === id ? { ...t, ...partial } : t,
            ),
          },
          isDirty: true,
        })),

      removeVentilationTerminal: (id) =>
        set((state) => ({
          ventilation: {
            ...state.ventilation,
            terminals: state.ventilation.terminals.filter((t) => t.id !== id),
          },
          isDirty: true,
        })),

      updateVentilationRoom: (roomId, partial) =>
        set((state) => {
          const current = state.ventilation.rooms[roomId];
          const base: VentilationRoomState =
            current ?? {
              ventilationFunction: "verblijfsruimte",
              requiredSupplyDm3s: 0,
              requiredExhaustDm3s: 0,
              airSourceRoomId: null,
            };
          return {
            ventilation: {
              ...state.ventilation,
              rooms: {
                ...state.ventilation.rooms,
                [roomId]: { ...base, ...partial },
              },
            },
            isDirty: true,
          };
        }),

      setVentilation: (ventilation) => set({ ventilation }),

      setVentilationSystem: (system) =>
        set((state) => ({
          ventilation: { ...state.ventilation, system },
          isDirty: true,
        })),

      addVentilationUnit: (unit) => {
        const id = unit.id ?? `unit-${crypto.randomUUID()}`;
        set((state) => {
          const units = state.ventilation.units ?? [];
          // Bestaand id (bv. catalogus-snapshot al gekopieerd) → no-op.
          if (units.some((u) => u.id === id)) return state;
          return {
            ventilation: {
              ...state.ventilation,
              units: [...units, { ...unit, id }],
            },
            isDirty: true,
          };
        });
        return id;
      },

      updateVentilationUnit: (id, partial) =>
        set((state) => ({
          ventilation: {
            ...state.ventilation,
            units: (state.ventilation.units ?? []).map((u) =>
              u.id === id ? { ...u, ...partial } : u,
            ),
          },
          isDirty: true,
        })),

      removeVentilationUnit: (id) =>
        set((state) => ({
          ventilation: {
            ...state.ventilation,
            units: (state.ventilation.units ?? []).filter((u) => u.id !== id),
            unitAssignments: (state.ventilation.unitAssignments ?? []).filter(
              (a) => a.unitId !== id,
            ),
          },
          isDirty: true,
        })),

      setVentilationUnitAssignment: (unitId, aantal) =>
        set((state) => {
          const current = state.ventilation.unitAssignments ?? [];
          const exists = current.some((a) => a.unitId === unitId);
          const unitAssignments: VentilationUnitAssignment[] =
            aantal > 0
              ? exists
                ? current.map((a) =>
                    a.unitId === unitId ? { ...a, aantal } : a,
                  )
                : [...current, { unitId, aantal }]
              : current.filter((a) => a.unitId !== unitId);
          return {
            ventilation: { ...state.ventilation, unitAssignments },
            isDirty: true,
          };
        }),

      setActiveProjectId: (id) => set({ activeProjectId: id }),
      setServerUpdatedAt: (updatedAt) => set({ serverUpdatedAt: updatedAt }),

      clearServerBinding: () => {
        // Save-status hoort bij de binding die we loskoppelen — een stale
        // "Opslaan…"/"Fout"-indicator mag niet blijven staan terwijl er
        // geen serverproject meer actief is.
        useSaveStatusStore.getState().resetStatus();
        set({
          activeProjectId: null,
          serverUpdatedAt: null,
          hasConflict: false,
        });
      },
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
            // SharedExtra (bouwjaar etc.) uit de envelope herstellen indien
            // meegegeven, anders resetten naar defaults (huidig gedrag voor
            // oude bestanden). Backfill-spread net als isso53Building zodat
            // nieuwe velden in toekomstige versies hun default krijgen.
            sharedExtra: opts?.sharedExtra
              ? { ...DEFAULT_SHARED_EXTRA, ...opts.sharedExtra }
              : { ...DEFAULT_SHARED_EXTRA },
            norm,
            // Sidecars uit de envelope herstellen indien meegegeven, anders
            // resetten naar defaults (huidig gedrag voor oude bestanden).
            // Backfill nieuwe velden (bv. `bouwfase`, `simultaneityFactor`)
            // voor envelopes van vóór ronde 6c — spread defaults eerst.
            isso53Building: opts?.isso53Building
              ? {
                  ...DEFAULT_ISSO53_BUILDING,
                  ...opts.isso53Building,
                  heatingUp: normalizeIsso53HeatingUp(
                    opts.isso53Building.heatingUp,
                  ),
                }
              : { ...DEFAULT_ISSO53_BUILDING },
            isso53Rooms: opts?.isso53Rooms ?? {},
            // Ventilatie-sidecar uit de envelope herstellen indien meegegeven,
            // anders leeg (huidig gedrag voor bestanden zonder ventilatie-data).
            // `system` expliciet meenemen — `undefined` voor oude bestanden
            // valt downstream terug op DEFAULT_VENTILATION_SYSTEM.
            ventilation: opts?.ventilation
              ? {
                  terminals: opts.ventilation.terminals ?? [],
                  rooms: opts.ventilation.rooms ?? {},
                  ...(opts.ventilation.system
                    ? { system: opts.ventilation.system }
                    : {}),
                  // WTW/MV-units + toewijzingen — alleen meenemen wanneer
                  // aanwezig (oude bestanden hebben deze velden niet).
                  ...(opts.ventilation.units
                    ? { units: opts.ventilation.units }
                    : {}),
                  ...(opts.ventilation.unitAssignments
                    ? { unitAssignments: opts.ventilation.unitAssignments }
                    : {}),
                }
              : { terminals: [], rooms: {} },
            // BENG-invoer reist (nog) niet mee in de envelope → altijd leeg bij
            // een projectwissel; geen lek naar het volgende project.
            energy: null,
            bengGeometry: null,
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

      loadServerProject: (id, project, result, updatedAt, opts) =>
        set({
          project,
          // Expliciete envelope-norm is autoritatief (zelfde semantiek als
          // setProject). Zonder envelope: detecteer uit de verwarmings-shape —
          // server-projecten met ISSO 53 heating mogen niet naar ISSO 51.
          norm: opts?.norm ?? detectNormFromProject(project),
          // Sidecars uit de server-envelope herstellen indien meegegeven,
          // anders defaults (gedrag voor legacy kale `project_data`-rijen).
          // Backfill-spreads identiek aan setProject zodat nieuwe velden in
          // toekomstige versies hun default krijgen.
          sharedExtra: opts?.sharedExtra
            ? { ...DEFAULT_SHARED_EXTRA, ...opts.sharedExtra }
            : { ...DEFAULT_SHARED_EXTRA },
          isso53Building: opts?.isso53Building
            ? {
                ...DEFAULT_ISSO53_BUILDING,
                ...opts.isso53Building,
                heatingUp: normalizeIsso53HeatingUp(
                  opts.isso53Building.heatingUp,
                ),
              }
            : { ...DEFAULT_ISSO53_BUILDING },
          isso53Rooms: opts?.isso53Rooms ?? {},
          ventilation: opts?.ventilation
            ? {
                terminals: opts.ventilation.terminals ?? [],
                rooms: opts.ventilation.rooms ?? {},
                ...(opts.ventilation.system
                  ? { system: opts.ventilation.system }
                  : {}),
                ...(opts.ventilation.units
                  ? { units: opts.ventilation.units }
                  : {}),
                ...(opts.ventilation.unitAssignments
                  ? { unitAssignments: opts.ventilation.unitAssignments }
                  : {}),
              }
            : { terminals: [], rooms: {} },
          // BENG-invoer zit nog niet in de server-envelope → leeg bij load.
          energy: null,
          bengGeometry: null,
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

      setResult: (result, runEpoch) =>
        set((state) => ({
          result,
          // Stale-guard: is de calc-input gewijzigd sinds de run startte
          // (epoch-mismatch), dan blijft de dirty-vlag staan zodat
          // auto-save en de "resultaat verouderd"-hints blijven werken.
          isDirty:
            runEpoch !== undefined && runEpoch !== projectInputEpoch
              ? state.isDirty
              : false,
          error: null,
          isCalculating: false,
        })),

      setError: (error) =>
        set({ error, isCalculating: false }),

      clearError: () =>
        set({ error: null }),

      setCalculating: (isCalculating) =>
        set({ isCalculating }),

      reset: () => {
        // Save-status hoort bij het project dat we sluiten — een stale
        // "Conflict"/"Offline"/"Fout"-indicator mag niet blijven staan op
        // het nieuwe (lege) project. Zelfde reset als openServerProject.
        useSaveStatusStore.getState().resetStatus();
        set({
          project: DEFAULT_PROJECT,
          sharedExtra: { ...DEFAULT_SHARED_EXTRA },
          // Reset valt terug op de default norm — NormChoiceModal in
          // Backstage zet hierna de gekozen norm via `setNorm`.
          norm: "isso51",
          isso53Building: { ...DEFAULT_ISSO53_BUILDING },
          isso53Rooms: {},
          ventilation: { terminals: [], rooms: {} },
          energy: null,
          bengGeometry: null,
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
        });
      },

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
          const { [roomId]: _removedVent, ...remainingVentRooms } =
            state.ventilation.rooms;
          return {
            project: {
              ...state.project,
              rooms: state.project.rooms.filter((r) => r.id !== roomId),
            },
            isso53Rooms: remainingIsso53,
            // Spread behoudt gebouw-niveau velden (system, units,
            // unitAssignments) — alleen room-gebonden data wordt opgeschoond.
            ventilation: {
              ...state.ventilation,
              terminals: state.ventilation.terminals.filter(
                (t) => t.roomId !== roomId,
              ),
              rooms: remainingVentRooms,
            },
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

      // -- Zones (datalaag; UI volgt in een vervolg-delegatie) --
      addZone: (name) => {
        const id = `zone-${crypto.randomUUID()}`;
        const snap = takeProjectSnapshot(get());
        set((state) => {
          const zone: Zone = { id, name };
          return {
            project: {
              ...state.project,
              building: {
                ...state.project.building,
                zones: [...(state.project.building.zones ?? []), zone],
              },
            },
            isDirty: true,
            error: null,
            _past: [...state._past, snap].slice(-MAX_HISTORY),
            _future: [],
          };
        });
        return id;
      },

      renameZone: (zoneId, name) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            building: {
              ...state.project.building,
              zones: (state.project.building.zones ?? []).map((z) =>
                z.id === zoneId ? { ...z, name } : z,
              ),
            },
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      removeZone: (zoneId) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            building: {
              ...state.project.building,
              zones: (state.project.building.zones ?? []).filter(
                (z) => z.id !== zoneId,
              ),
            },
            // Dangling room-referenties opruimen — zelfde cascade-patroon
            // als removeVentilationUnit → unitAssignments.
            rooms: state.project.rooms.map((r) =>
              r.zoneId === zoneId ? { ...r, zoneId: undefined } : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      setRoomZone: (roomId, zoneId) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId ? { ...r, zoneId } : r,
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
          // Per-ruimte sidecars mee terugzetten — een undo van removeRoom
          // moet ook de isso53-/ventilatie-config van die ruimte herstellen.
          isso53Rooms: prev.isso53Rooms ?? state.isso53Rooms,
          ventilation: prev.ventilation ?? state.ventilation,
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
          isso53Rooms: next.isso53Rooms ?? state.isso53Rooms,
          ventilation: next.ventilation ?? state.ventilation,
          _past: [...state._past, currentSnap],
          _future: state._future.slice(0, -1),
          isDirty: true,
        });
      },
    }),
    {
      name: "isso51-project",
      version: 1,
      partialize: partializeProjectStore,
      merge: mergePersistedProjectStore,
    },
  ),
);

/**
 * Persist-partialize (geëxporteerd voor tests — de `store.persist`-API is
 * in de node-testomgeving niet beschikbaar zonder `window.localStorage`).
 *
 * Server-binding + dirty-vlag worden mee-gepersisteerd (audit 09 §2.2):
 * na een reload ("herlaad om in te loggen") moet de auto-save doorlopen
 * op hetzelfde serverproject i.p.v. stil te stoppen. `hasConflict` bewust
 * NIET — een conflict wordt bij de eerste save na reload opnieuw
 * gedetecteerd via `serverUpdatedAt`.
 *
 * Keerzijde (R1): juist doordat de binding persist, moet hij bij logout en
 * bij een definitief verlopen sessie expliciet gewist worden — zie
 * {@link ProjectStore.clearServerBinding} (aangeroepen in `lib/auth.ts`
 * `logoutRedirect` en `lib/serverProjects.ts` `recordSaveFailure`).
 */
export function partializeProjectStore(state: ProjectStore) {
  return {
    project: state.project,
    sharedExtra: state.sharedExtra,
    norm: state.norm,
    isso53Building: state.isso53Building,
    isso53Rooms: state.isso53Rooms,
    ventilation: state.ventilation,
    energy: state.energy,
    bengGeometry: state.bengGeometry,
    result: state.result,
    isDirty: state.isDirty,
    activeProjectId: state.activeProjectId,
    serverUpdatedAt: state.serverUpdatedAt,
  };
}

/** Persist-merge (geëxporteerd voor tests — zie partializeProjectStore). */
export function mergePersistedProjectStore(
  persisted: unknown,
  current: ProjectStore,
): ProjectStore {
  return {
    ...current,
    ...(persisted as Pick<
      ProjectStore,
      | "project"
      | "sharedExtra"
      | "norm"
      | "isso53Building"
      | "isso53Rooms"
      | "ventilation"
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
        // Backfill nieuwe velden (bv. `bouwfase`) voor projecten van vóór
        // ronde 6c — spread defaults eerst, dan de gepersisteerde waarden.
        ...DEFAULT_ISSO53_BUILDING,
        ...persistedBuilding,
        heatingUp: normalizeIsso53HeatingUp(persistedBuilding.heatingUp),
      };
    })(),
    isso53Rooms:
      (persisted as Partial<ProjectStore>)?.isso53Rooms ?? current.isso53Rooms,
    // Silent migration voor projecten van vóór de BENG-tab (geen energy-blok).
    energy: (persisted as Partial<ProjectStore>)?.energy ?? null,
    // Silent migration voor projecten van vóór het beng_geometry-blok (F6).
    bengGeometry: (persisted as Partial<ProjectStore>)?.bengGeometry ?? null,
    // Silent migration voor projecten van vóór de ventilatiebalans-module.
    ventilation: (() => {
      const v = (persisted as Partial<ProjectStore>)?.ventilation;
      if (!v) return current.ventilation;
      return {
        terminals: Array.isArray(v.terminals) ? v.terminals : [],
        rooms: v.rooms && typeof v.rooms === "object" ? v.rooms : {},
        // Alleen geldige systeemsleutels doorlaten; ontbrekend/ongeldig →
        // undefined (default-fallback, projecten van vóór de selector).
        ...(v.system && v.system in VENTILATION_SYSTEMS
          ? { system: v.system }
          : {}),
        // WTW/MV-units + toewijzingen (projecten van vóór de
        // units-module hebben deze velden niet).
        ...(Array.isArray(v.units) ? { units: v.units } : {}),
        ...(Array.isArray(v.unitAssignments)
          ? { unitAssignments: v.unitAssignments }
          : {}),
      };
    })(),
    // Rehydrate-keuze (audit 09 §2.2): isDirty + serverbinding worden
    // gepersisteerd en hier hersteld. Legacy persisted state (van vóór
    // deze velden) → isDirty: true. Dat is de veilige kant: hooguit één
    // overbodige save/dirty-indicator, nooit een stil-stoppende
    // auto-save terwijl er onopgeslagen werk staat.
    isDirty: (persisted as Partial<ProjectStore>)?.isDirty ?? true,
    activeProjectId:
      (persisted as Partial<ProjectStore>)?.activeProjectId ?? null,
    serverUpdatedAt:
      (persisted as Partial<ProjectStore>)?.serverUpdatedAt ?? null,
    hasConflict: false,
    isCalculating: false,
    error: null,
  };
}

// ---------------------------------------------------------------------------
// Calc-input epoch — stale-result guard voor setResult
// ---------------------------------------------------------------------------

/**
 * Monotone teller die elke wijziging aan de berekenings-/save-relevante
 * input telt (project + alle sidecars + norm). `useRunCalculation` legt de
 * epoch vast bij de start van een berekening en geeft hem mee aan
 * {@link ProjectStore.setResult}; bij een mismatch (edit of project-wissel
 * tijdens de lopende run) blijft `isDirty` staan.
 *
 * Module-level i.p.v. store-state: de teller is geen UI-state en mag geen
 * re-renders of persist-writes triggeren.
 */
let projectInputEpoch = 0;

useProjectStore.subscribe((state, prev) => {
  if (
    state.project !== prev.project ||
    state.sharedExtra !== prev.sharedExtra ||
    state.norm !== prev.norm ||
    state.isso53Building !== prev.isso53Building ||
    state.isso53Rooms !== prev.isso53Rooms ||
    state.ventilation !== prev.ventilation
  ) {
    projectInputEpoch += 1;
  }
});

/** Huidige calc-input epoch — zie {@link ProjectStore.setResult}. */
export function getProjectInputEpoch(): number {
  return projectInputEpoch;
}
