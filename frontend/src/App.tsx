import { useEffect, useState, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

import { AppShell } from "./components/layout/AppShell";
import { Library } from "./pages/Library";
import { Projects } from "./pages/Projects";
import { ProjectSetup } from "./pages/ProjectSetup";
import { RoomEditor } from "./pages/RoomEditor";
import { Results } from "./pages/Results";
import { isTauri } from "./lib/backend";
import { OidcInitializationGate, bootstrapOidc } from "./lib/oidc";

/** Wrapper that initializes OIDC for web builds (skipped in Tauri). */
function OidcBootstrap({ children }: { children: ReactNode }) {
  const [state, setState] = useState<"loading" | "ready" | "failed">("loading");

  useEffect(() => {
    const issuer = import.meta.env.VITE_OIDC_ISSUER;
    const clientId = import.meta.env.VITE_OIDC_CLIENT_ID;

    if (!issuer || !clientId) {
      console.warn("OIDC not configured (VITE_OIDC_ISSUER / VITE_OIDC_CLIENT_ID missing)");
      setState("failed");
      return;
    }

    // Race: if bootstrapOidc hangs, time out after 5s and continue without auth.
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
      .then(() => setState("ready"))
      .catch((err) => {
        console.error("OIDC bootstrap failed, continuing without auth:", err);
        setState("failed");
      });
  }, []);

  if (state === "loading") {
    return (
      <div className="flex h-screen items-center justify-center text-stone-400">
        Laden...
      </div>
    );
  }

  // OIDC failed or not configured — render without auth gate.
  if (state === "failed") {
    return <>{children}</>;
  }

  return <OidcInitializationGate>{children}</OidcInitializationGate>;
}

export function App() {
  const content = (
    <BrowserRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<Navigate to="/project" replace />} />
          <Route path="/project" element={<ProjectSetup />} />
          <Route path="/rooms" element={<RoomEditor />} />
          <Route path="/library" element={<Library />} />
          <Route path="/results" element={<Results />} />
          <Route path="/projects" element={<Projects />} />
        </Routes>
      </AppShell>
    </BrowserRouter>
  );

  // Tauri desktop: no OIDC, render directly.
  if (isTauri()) {
    return content;
  }

  // Web: wrap with OIDC bootstrap.
  return <OidcBootstrap>{content}</OidcBootstrap>;
}
