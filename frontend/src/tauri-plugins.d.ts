/**
 * Type declarations for Tauri plugins that are only available at runtime.
 * These modules are dynamically imported and will fail gracefully in web mode.
 */

declare module "@tauri-apps/plugin-store" {
  interface StoreOptions {
    autoSave?: boolean;
    defaults?: Record<string, unknown>;
  }

  interface Store {
    get<T>(key: string): Promise<T | undefined>;
    set(key: string, value: unknown): Promise<void>;
    save(): Promise<void>;
  }

  export function load(path: string, options?: StoreOptions): Promise<Store>;
}

declare module "@tauri-apps/plugin-os" {
  export function type(): string;
  export function version(): string;
  export function arch(): string;
}
