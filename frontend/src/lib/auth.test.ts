import { afterEach, describe, expect, it, vi } from "vitest";

import { logoutRedirect } from "./auth";
import { useProjectStore } from "../store/projectStore";
import { useSaveStatusStore } from "../store/saveStatusStore";

/**
 * Tests voor het logout-pad (R1): `logoutRedirect` moet de gepersisteerde
 * serverbinding wissen vóór de redirect naar de Authentik sign-out, zodat
 * een volgende gebruiker op een gedeelde browser geen `activeProjectId`/
 * `serverUpdatedAt` van de vorige gebruiker erft. Het project zelf blijft
 * in de store — uitloggen gooit geen onopgeslagen werk weg.
 */

afterEach(() => {
  vi.unstubAllGlobals();
  useProjectStore.getState().reset();
  useSaveStatusStore.getState().resetStatus();
});

describe("logoutRedirect — serverbinding wissen (R1)", () => {
  it("wist de binding en redirect daarna naar de Authentik sign-out", () => {
    const assign = vi.fn();
    vi.stubGlobal("window", { location: { assign } });

    useProjectStore.setState({
      project: {
        ...useProjectStore.getState().project,
        info: { name: "Project van user A" },
      },
      isDirty: true,
      activeProjectId: "proj-A",
      serverUpdatedAt: "2026-06-10 09:00:00",
      hasConflict: true,
    });
    useSaveStatusStore.getState().setError("save mislukt");

    logoutRedirect();

    const s = useProjectStore.getState();
    expect(s.activeProjectId).toBeNull();
    expect(s.serverUpdatedAt).toBeNull();
    expect(s.hasConflict).toBe(false);
    // Onopgeslagen werk blijft staan — alleen de serverbinding gaat los.
    expect(s.project.info.name).toBe("Project van user A");
    expect(s.isDirty).toBe(true);
    expect(useSaveStatusStore.getState().status).toBe("idle");
    expect(assign).toHaveBeenCalledWith("/outpost.goauthentik.io/sign_out");
  });
});
