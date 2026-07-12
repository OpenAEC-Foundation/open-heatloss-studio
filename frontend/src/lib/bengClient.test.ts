/**
 * Tests voor de BENG-client dispatch (`lib/bengClient.ts`).
 *
 * Geborgd gedrag:
 *  - Web-pad (geen Tauri-runtime): `POST {API_PREFIX}/beng/calculate` met de
 *    `{ project }`-envelope; parse-succes en de fout-classificatie (422 →
 *    `BengInputError`, overige → `Error`).
 *  - Tauri-pad: `invoke("compute_beng", { req })` met de req-arg-naam `req`;
 *    een missing-input-foutstring wordt naar `BengInputError` geclassificeerd.
 *
 * De vitest-omgeving is "node": zonder `window` geeft `isTauri()` false, dus
 * het web-pad is de default. Voor het Tauri-pad stubben we `window` met de
 * `__TAURI_INTERNALS__`-marker en mocken we `@tauri-apps/api/core`.
 */
import { afterEach, describe, expect, it, vi } from "vitest";

import { bengCalculate, BengInputError } from "./bengClient";
import { API_PREFIX } from "./constants";
import type { BengResult } from "../types/beng";
import type { ProjectV2 } from "../types/projectV2";
import { SCHEMA_VERSION_V2 } from "../types/projectV2";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function fixtureProject(): ProjectV2 {
  return {
    schema_version: SCHEMA_VERSION_V2,
    shared: {
      name: "BENG-test",
      building_type: { kind: "woning", subtype: "terraced" },
    },
    geometry: { spaces: [] },
    calcs: { isso51: null, tojuli: null },
    energy: {
      heating: { generator: "heat_pump_ground", cop: 4.5 },
    },
  };
}

function fixtureResult(): BengResult {
  return {
    beng1: { value: 42, limit: 55, pass: true },
    beng2: { value: 30, limit: 30, pass: true },
    beng3: { value: 80, limit: 50, pass: true },
    tojuli: {
      max_tojuli_k: 0.8,
      limit_k: 1.2,
      actively_cooled: false,
      pass: true,
      method: "per_orientation",
    },
    energy_label: "A+++",
    renewable_share: 0.8,
    co2_kg_per_m2: 3.1,
    a_g_m2: 87,
    a_ls_m2: 156,
    als_ag_ratio: 1.79,
    service_breakdown_kwh_m2: {
      heating: 20,
      cooling: 0,
      dhw: 8,
      ventilation_aux: 2,
      lighting: 0,
      pv: -10,
    },
    notes: ["TOjuli per oriëntatie (benadering)."],
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
  invokeMock.mockReset();
});

describe("bengCalculate — web-pad", () => {
  it("POST naar /beng/calculate met de { project }-envelope en parse-succes", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve(fixtureResult()),
    });
    vi.stubGlobal("fetch", fetchMock);

    const project = fixtureProject();
    const result = await bengCalculate({ project });

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, init] = fetchMock.mock.calls[0]!;
    expect(url).toBe(`${API_PREFIX}/beng/calculate`);
    expect(init.method).toBe("POST");
    const body = JSON.parse(init.body as string) as { project: ProjectV2 };
    expect(body.project.energy?.heating?.generator).toBe("heat_pump_ground");
    expect(result.energy_label).toBe("A+++");
    expect(result.beng1.pass).toBe(true);
  });

  it("422 → BengInputError met de backend-detail", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      status: 422,
      statusText: "Unprocessable Entity",
      json: () =>
        Promise.resolve({ detail: "project mist het `energy`-invoerblok" }),
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(bengCalculate({ project: fixtureProject() })).rejects.toThrow(
      BengInputError,
    );
    await expect(
      bengCalculate({ project: fixtureProject() }),
    ).rejects.toThrow(/invoerblok/);
  });

  it("400 → gewone Error (geen BengInputError)", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      status: 400,
      statusText: "Bad Request",
      json: () => Promise.resolve({ detail: "verwarming: onbekende opwekker" }),
    });
    vi.stubGlobal("fetch", fetchMock);

    const promise = bengCalculate({ project: fixtureProject() });
    await expect(promise).rejects.toThrow("verwarming: onbekende opwekker");
    await expect(
      bengCalculate({ project: fixtureProject() }),
    ).rejects.not.toBeInstanceOf(BengInputError);
  });
});

describe("bengCalculate — Tauri-pad", () => {
  it("invoke('compute_beng', { req }) met de req-arg-naam en parse-succes", async () => {
    vi.stubGlobal("window", { __TAURI_INTERNALS__: {} });
    invokeMock.mockResolvedValue(fixtureResult());

    const project = fixtureProject();
    const result = await bengCalculate({ project });

    expect(invokeMock).toHaveBeenCalledTimes(1);
    const [cmd, args] = invokeMock.mock.calls[0]!;
    expect(cmd).toBe("compute_beng");
    expect((args as { req: BengCalculateReq }).req.project.shared.name).toBe(
      "BENG-test",
    );
    expect(result.energy_label).toBe("A+++");
  });

  it("missing-input foutstring → BengInputError", async () => {
    vi.stubGlobal("window", { __TAURI_INTERNALS__: {} });
    invokeMock.mockRejectedValue(
      "project mist het `energy`-invoerblok (nodig voor de BENG-keten)",
    );

    await expect(bengCalculate({ project: fixtureProject() })).rejects.toThrow(
      BengInputError,
    );
  });
});

interface BengCalculateReq {
  project: ProjectV2;
}
