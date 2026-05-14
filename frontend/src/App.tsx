import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

import { AppShell } from "./components/layout/AppShell";
import { AppErrorBoundary } from "./components/errors/AppErrorBoundary";
import { Library } from "./pages/Library";
import { Projects } from "./pages/Projects";
import { ProjectSetup } from "./pages/ProjectSetup";
import { RoomEditor } from "./pages/RoomEditor";
import { RcCalculator } from "./pages/RcCalculator";
import { Results } from "./pages/Results";
import { Tojuli } from "./pages/Tojuli";
import { Modeller } from "./pages/Modeller";
import { ProjectConstructions } from "./pages/ProjectConstructions";
import { ThermalImportWizard } from "./components/import/ThermalImportWizard";

/**
 * Application root.
 *
 * Authentication is handled by Caddy + Authentik forward_auth on the public
 * domain (`warmteverlies.open-aec.com`). When the request reaches the
 * frontend the user is already logged in; user info is fetched via
 * `GET /api/v1/me`. No client-side OIDC bootstrap needed.
 *
 * Tauri desktop builds skip the API entirely and run against the local
 * `tauri::invoke` backend.
 */
export function App() {
  return (
    <AppErrorBoundary>
      <BrowserRouter>
        <AppShell>
          <Routes>
            <Route path="/" element={<Navigate to="/project" replace />} />
            <Route path="/project" element={<ProjectSetup />} />
            <Route path="/rooms" element={<RoomEditor />} />
            <Route path="/constructies" element={<ProjectConstructions />} />
            <Route path="/rc" element={<RcCalculator />} />
            <Route path="/tojuli" element={<Tojuli />} />
            <Route path="/library" element={<Library />} />
            <Route path="/materialen" element={<Library initialSection="materialen" />} />
            <Route path="/results" element={<Results />} />
            <Route path="/modeller" element={<Modeller />} />
            <Route path="/import/thermal" element={<ThermalImportWizard />} />
            <Route path="/projects" element={<Projects />} />
          </Routes>
        </AppShell>
      </BrowserRouter>
    </AppErrorBoundary>
  );
}
