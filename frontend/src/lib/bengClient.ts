/**
 * BENG-berekening client (Tauri invoke of web fetch), naar het patroon van
 * de TO-juli-dispatch in `lib/backend.ts`.
 *
 * Contract (zie `crates/isso51-api/src/handlers/beng.rs` +
 * `src-tauri/src/commands.rs`):
 *  - Web:  `POST {API_PREFIX}/beng/calculate` met body `{ project }` →
 *          `BengResult`. 422 = ontbrekend `energy`-blok of lege geometrie,
 *          400 = reken-fout.
 *  - Tauri: `invoke("compute_beng", { req: { project } })` →
 *          `Result<BengResult, String>` (invoke-arg heet **`req`**).
 *
 * Anders dan TO-juli is er géén los `inputs`-veld: alle installatie-invoer
 * zit in `project.energy`.
 */
import type { BengResult } from "../types/beng";
import type { ProjectV2 } from "../types/projectV2";
import { isTauri } from "./backend";
import { API_PREFIX } from "./constants";

/** Request-envelope voor de BENG-berekening (spiegelt `BengCalculateRequest`). */
export interface BengCalculateRequest {
  project: ProjectV2;
}

/**
 * Ontbrekende/onvolledige BENG-invoer (HTTP 422 of de Tauri-equivalent).
 *
 * De backend geeft 422 bij een ontbrekend `energy`-blok
 * (`MissingEnergyInput`) of een lege rekenzone (`EmptyProject`). De UI vangt
 * dit apart op om een "vul het energie-blok in"-melding te tonen i.p.v. een
 * kale foutregel.
 */
export class BengInputError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "BengInputError";
  }
}

/**
 * Herkent de Tauri-invoke-foutstring die overeenkomt met een 422:
 * `BengError::MissingEnergyInput` / `EmptyProject` serialiseren via `Display`
 * naar Nederlandse teksten met deze sleutelwoorden (geen HTTP-status
 * beschikbaar bij invoke, dus string-classificatie).
 */
function isMissingInputMessage(message: string): boolean {
  return /invoerblok|rekenzone|gebruiksoppervlak/i.test(message);
}

/** Voer de volledige BENG-keten uit (Tauri of web). */
export async function bengCalculate(
  req: BengCalculateRequest,
): Promise<BengResult> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    try {
      return await invoke<BengResult>("compute_beng", { req });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw isMissingInputMessage(message)
        ? new BengInputError(message)
        : new Error(message);
    }
  }

  const res = await fetch(`${API_PREFIX}/beng/calculate`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const body = await res
      .json()
      .catch(() => ({ detail: res.statusText }) as { detail?: string });
    const detail = (body as { detail?: string }).detail ?? `HTTP ${res.status}`;
    throw res.status === 422
      ? new BengInputError(detail)
      : new Error(detail);
  }
  return res.json() as Promise<BengResult>;
}
