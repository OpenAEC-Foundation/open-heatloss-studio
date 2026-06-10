/**
 * Zustand store voor de persistente server-save-statusindicator (StatusBar).
 *
 * Vervangt het stille falen van de auto-save: waar een toast na enkele
 * seconden verdwijnt, blijft deze status zichtbaar totdat een volgende save
 * slaagt. Gevuld door `lib/serverProjects.ts` (de gedeelde save-helpers) en
 * gelezen door `components/StatusBar.tsx`.
 *
 * Niet gepersisteerd — de status beschrijft de huidige sessie.
 */
import { create } from "zustand";

/** Toestand van de laatste server-save-poging. */
export type SaveStatus =
  /** Geen serverproject actief of nog geen save-poging gedaan. */
  | "idle"
  /** Save-request loopt. */
  | "saving"
  /** Laatste save geslaagd (zie `lastSavedAt`). */
  | "saved"
  /** Netwerkfout terwijl de browser offline is — retry volgt bij reconnect. */
  | "offline"
  /** Save mislukt (server-/sessiefout) — handmatige retry mogelijk. */
  | "error"
  /** 409: project is elders gewijzigd (zie ConflictDialog). */
  | "conflict";

interface SaveStatusStore {
  status: SaveStatus;
  /** Tijdstip (epoch ms) van de laatste geslaagde save, voor "opgeslagen HH:MM". */
  lastSavedAt: number | null;
  /** Detail van de laatste fout (alleen bij status "error"). */
  errorDetail: string | null;
  /**
   * Retry-callback, geregistreerd door `useAutoSave` zodat de StatusBar een
   * mislukte save direct opnieuw kan triggeren zonder op de debounce of het
   * online-event te wachten.
   */
  retryHandler: (() => void) | null;

  setSaving: () => void;
  setSaved: () => void;
  setOffline: () => void;
  setError: (detail: string) => void;
  setConflict: () => void;
  /** Terug naar idle — bij project-wissel / nieuw project. */
  resetStatus: () => void;
  registerRetryHandler: (handler: (() => void) | null) => void;
  /** Trigger een handmatige retry (no-op zonder geregistreerde handler). */
  retry: () => void;
}

export const useSaveStatusStore = create<SaveStatusStore>()((set, get) => ({
  status: "idle",
  lastSavedAt: null,
  errorDetail: null,
  retryHandler: null,

  setSaving: () => set({ status: "saving", errorDetail: null }),
  setSaved: () =>
    set({ status: "saved", lastSavedAt: Date.now(), errorDetail: null }),
  setOffline: () => set({ status: "offline", errorDetail: null }),
  setError: (detail) => set({ status: "error", errorDetail: detail }),
  setConflict: () => set({ status: "conflict", errorDetail: null }),
  resetStatus: () => set({ status: "idle", errorDetail: null }),
  registerRetryHandler: (handler) => set({ retryHandler: handler }),
  retry: () => {
    get().retryHandler?.();
  },
}));
