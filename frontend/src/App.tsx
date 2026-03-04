import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

import { AppShell } from "./components/layout/AppShell";
import { ProjectSetup } from "./pages/ProjectSetup";
import { RoomEditor } from "./pages/RoomEditor";
import { Results } from "./pages/Results";

export function App() {
  return (
    <BrowserRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<Navigate to="/project" replace />} />
          <Route path="/project" element={<ProjectSetup />} />
          <Route path="/rooms" element={<RoomEditor />} />
          <Route path="/results" element={<Results />} />
        </Routes>
      </AppShell>
    </BrowserRouter>
  );
}
