import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

import { AppShell } from "./components/layout/AppShell";
import { AppErrorBoundary } from "./components/errors/AppErrorBoundary";
import { Library } from "./pages/Library";
import { Projects } from "./pages/Projects";
import { ProjectSetup } from "./pages/ProjectSetup";
import { RoomEditor } from "./pages/RoomEditor";
import { RcCalculator } from "./pages/RcCalculator";
import { RcCompare } from "./pages/RcCompare";
import { UwCalculator } from "./pages/UwCalculator";
import { Results } from "./pages/Results";
import { WarmteverliesInstellingen } from "./pages/WarmteverliesInstellingen";
import { Tojuli } from "./pages/Tojuli";
import { TojuliFull } from "./pages/TojuliFull";
import { Beng } from "./pages/Beng";
import { VentilationBalance } from "./pages/VentilationBalance";
import { DoorGapCalculator } from "./pages/DoorGapCalculator";
import { HwaCalculator } from "./pages/HwaCalculator";
import { Modeller } from "./pages/Modeller";
import { ProjectConstructions } from "./pages/ProjectConstructions";
import { Rapport } from "./pages/Rapport";
import { Ifc } from "./pages/Ifc";
import { Help } from "./pages/Help";
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
            <Route path="/rc-compare" element={<RcCompare />} />
            <Route path="/uw" element={<UwCalculator />} />
            <Route path="/tojuli/quick" element={<Tojuli />} />
            <Route path="/tojuli" element={<TojuliFull />} />
            <Route path="/beng" element={<Beng />} />
            <Route path="/ventilation" element={<VentilationBalance />} />
            <Route path="/tools/deurspleet" element={<DoorGapCalculator />} />
            <Route path="/tools/hwa" element={<HwaCalculator />} />
            <Route path="/library" element={<Library />} />
            <Route path="/materialen" element={<Library initialSection="materialen" />} />
            <Route path="/warmteverlies/instellingen" element={<WarmteverliesInstellingen />} />
            <Route path="/results" element={<Results />} />
            <Route path="/modeller" element={<Modeller />} />
            <Route path="/ifc" element={<Ifc />} />
            <Route path="/rapport" element={<Rapport />} />
            <Route path="/import/thermal" element={<ThermalImportWizard />} />
            <Route path="/help" element={<Help />} />
            <Route path="/projects" element={<Projects />} />
          </Routes>
        </AppShell>
      </BrowserRouter>
    </AppErrorBoundary>
  );
}
