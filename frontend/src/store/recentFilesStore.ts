/**
 * Recent-files store.
 *
 * Houdt de laatst-geopende projecten bij (max 10) met naam, optioneel
 * absoluut pad (Tauri save-dialog) en timestamp. Persisted in
 * localStorage zodat de lijst sessies overleeft.
 *
 * Bedoeld om in Backstage onder "Openen" een snelle-toegang-lijst te
 * tonen. Klikken op een entry triggert dezelfde flow als "Lokaal
 * bestand…" maar via Tauri's `readTextFile` op het bekende pad.
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface RecentFile {
  /** Display naam — typisch `project.info.name` of de bestandsnaam. */
  name: string;
  /**
   * Absoluut pad zoals Tauri's `dialog.save` / `dialog.open` het teruggaf.
   * Optioneel — in browser-mode (geen file-system access) blijft dit
   * undefined en kan de entry alleen ter referentie dienen.
   */
  path?: string;
  /** Bestandsnaam (alleen filename, geen pad). */
  fileName: string;
  /** ISO timestamp van laatst geopend / opgeslagen. */
  openedAt: string;
}

interface RecentFilesStore {
  recent: RecentFile[];
  /** Push een entry naar de top. Dedup op `path` (of `name` als path leeg). */
  push: (entry: Omit<RecentFile, "openedAt">) => void;
  /** Verwijder een entry uit de lijst. */
  remove: (entry: RecentFile) => void;
  /** Leeg de hele lijst. */
  clear: () => void;
}

const MAX_RECENT = 10;

export const useRecentFilesStore = create<RecentFilesStore>()(
  persist(
    (set, get) => ({
      recent: [],
      push: (entry) => {
        const dedupKey = entry.path ?? entry.name;
        const filtered = get().recent.filter(
          (r) => (r.path ?? r.name) !== dedupKey,
        );
        const next: RecentFile = {
          ...entry,
          openedAt: new Date().toISOString(),
        };
        set({ recent: [next, ...filtered].slice(0, MAX_RECENT) });
      },
      remove: (entry) => {
        const key = entry.path ?? entry.name;
        set({
          recent: get().recent.filter((r) => (r.path ?? r.name) !== key),
        });
      },
      clear: () => set({ recent: [] }),
    }),
    {
      name: "ohs-recent-files",
      version: 1,
    },
  ),
);
