/**
 * Tauri preferences store wrapper.
 * Gracefully degrades to localStorage when not running in Tauri.
 *
 * All Tauri imports are fully dynamic (no static `import type` from plugins)
 * to prevent Rollup/Vite from trying to resolve them at build time.
 */

interface TauriStore {
  get<T>(key: string): Promise<T | undefined>;
  set(key: string, value: unknown): Promise<void>;
  save(): Promise<void>;
}

let _store: TauriStore | null = null;
let _useFallback = false;

async function getStore(): Promise<TauriStore | null> {
  if (_useFallback) return null;
  if (_store) return _store;

  try {
    const mod = await import("@tauri-apps/plugin-store");
    _store = await mod.load("preferences.json", { autoSave: true });
    return _store;
  } catch {
    _useFallback = true;
    return null;
  }
}

export async function getSetting<T>(key: string, fallback: T): Promise<T> {
  try {
    const store = await getStore();
    if (store) {
      const value = await store.get<T>(key);
      return value ?? fallback;
    }
    // Fallback to localStorage
    const raw = localStorage.getItem(`pref:${key}`);
    if (raw === null) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export async function setSetting<T>(key: string, value: T): Promise<void> {
  try {
    const store = await getStore();
    if (store) {
      await store.set(key, value);
      return;
    }
    // Fallback to localStorage
    localStorage.setItem(`pref:${key}`, JSON.stringify(value));
  } catch {
    // silently fail if store unavailable
  }
}
