import { useEffect, useState, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

import { AppShell } from "./components/layout/AppShell";
import { Library } from "./pages/Library";
import { Projects } from "./pages/Projects";
import { ProjectSetup } from "./pages/ProjectSetup";
import { RoomEditor } from "./pages/RoomEditor";
import { RcCalculator } from "./pages/RcCalculator";
import { Results } from "./pages/Results";
import { Modeller } from "./pages/Modeller";
import { ProjectConstructions } from "./pages/ProjectConstructions";
import { isTauri } from "./lib/backend";

/** Whether OIDC env vars are baked in at build time. */
const OIDC_CONFIGURED =
  !!import.meta.env.VITE_OIDC_ISSUER && !!import.meta.env.VITE_OIDC_CLIENT_ID;

/**
 * Wrapper that initializes OIDC for web builds.
 * Uses dynamic import so oidc-spa is never loaded when OIDC is not configured.
 */
function OidcBootstrap({ children }: { children: ReactNode }) {
  const [state, setState] = useState<"loading" | "ready" | "failed">("loading");
  const [Gate, setGate] = useState<React.ComponentType<{ children: ReactNode }> | null>(null);

  useEffect(() => {
    const issuer = import.meta.env.VITE_OIDC_ISSUER;
    const clientId = import.meta.env.VITE_OIDC_CLIENT_ID;

    import("./lib/oidc").then(({ bootstrapOidc, OidcInitializationGate }) => {
      const timeout = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error("OIDC bootstrap timed out")), 5000),
      );

      Promise.race([
        bootstrapOidc({
          implementation: "real",
          issuerUri: issuer,
          clientId,
          scopes: ["openid", "email", "profile"],
        }),
        timeout,
      ])
        .then(() => {
          setGate(() => OidcInitializationGate);
          setState("ready");
        })
        .catch((err) => {
          console.error("OIDC bootstrap failed, continuing without auth:", err);
          setState("failed");
        });
    });
  }, []);

  if (state === "loading") {
    return (
      <div className="flex h-screen items-center justify-center text-on-surface-muted bg-surface">
        Laden...
      </div>
    );
  }

  if (state === "ready" && Gate) {
    return <Gate>{children}</Gate>;
  }

  return <>{children}</>;
}

export function App() {
  const content = (
    <BrowserRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<Navigate to="/project" replace />} />
          <Route path="/project" element={<ProjectSetup />} />
          <Route path="/rooms" element={<RoomEditor />} />
          <Route path="/constructies" element={<ProjectConstructions />} />
          <Route path="/rc" element={<RcCalculator />} />
          <Route path="/library" element={<Library />} />
          <Route path="/materialen" element={<Library initialSection="materialen" />} />
          <Route path="/results" element={<Results />} />
          <Route path="/modeller" element={<Modeller />} />
          <Route path="/projects" element={<Projects />} />
        </Routes>
      </AppShell>
    </BrowserRouter>
  );

  // Tauri desktop or no OIDC: render directly without any oidc-spa involvement.
  if (isTauri() || !OIDC_CONFIGURED) {
    return content;
  }

  // Web with OIDC configured: wrap with bootstrap.
  return <OidcBootstrap>{content}</OidcBootstrap>;
}
