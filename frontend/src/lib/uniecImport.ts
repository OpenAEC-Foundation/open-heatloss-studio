/**
 * Uniec 3 (`.uniec3`) import-client (Tauri invoke of web fetch), naar het
 * dispatch-patroon van `bengClient.ts`.
 *
 * Contract (zie `crates/isso51-api/src/handlers/uniec_import.rs` +
 * `src-tauri/src/commands.rs`):
 *  - Web:  `POST {API_PREFIX}/beng/import-uniec3` met body `{ file_base64 }` →
 *          `{ project, certified, warnings }`. 422 = import-fout (corrupt/
 *          buiten-scope, o.a. multi-zone — de `detail` komt letterlijk door),
 *          400 = ongeldige base64.
 *  - Tauri: `invoke("import_uniec3", { fileBase64 })` →
 *          `Result<Uniec3ImportResult, String>` (invoke-arg heet **`fileBase64`**).
 *
 * Het bestand gaat base64-gecodeerd over de lijn zodat beide paden identiek zijn.
 */
import type { Uniec3ImportResult } from "../types/uniec";
import { isTauri } from "./backend";
import { API_PREFIX } from "./constants";

/**
 * Fout bij het importeren van een `.uniec3` (corrupt archief, incompleet of
 * buiten de V1-scope — o.a. multi-zone/appartementen en utiliteitsbouw). De
 * `message` is de letterlijke backend-boodschap (Nederlandse `Display`-tekst),
 * bedoeld om ongewijzigd aan de gebruiker te tonen.
 */
export class Uniec3ImportError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Uniec3ImportError";
  }
}

/** Lees een `File` en codeer de bytes als standaard-base64 (met padding). */
async function fileToBase64(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  // Chunk-gewijs naar een binary string — `String.fromCharCode(...bytes)` in
  // één keer overschrijdt de arg-limiet bij grotere bestanden.
  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
}

/** Importeer een `.uniec3`-bestand (Tauri of web). */
export async function importUniec3(file: File): Promise<Uniec3ImportResult> {
  const fileBase64 = await fileToBase64(file);

  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    try {
      return await invoke<Uniec3ImportResult>("import_uniec3", { fileBase64 });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new Uniec3ImportError(message);
    }
  }

  const res = await fetch(`${API_PREFIX}/beng/import-uniec3`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ file_base64: fileBase64 }),
  });
  if (!res.ok) {
    const body = await res
      .json()
      .catch(() => ({ detail: res.statusText }) as { detail?: string });
    const detail = (body as { detail?: string }).detail ?? `HTTP ${res.status}`;
    throw new Uniec3ImportError(detail);
  }
  return res.json() as Promise<Uniec3ImportResult>;
}
