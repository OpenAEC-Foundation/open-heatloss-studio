import type { ReactNode } from "react";

import { useProjectStore } from "../../store/projectStore";
import { Sidebar } from "./Sidebar";
import { TitleBar } from "./TitleBar";
import { ToastContainer } from "../ui/Toast";
import { ConflictDialog } from "../ui/ConflictDialog";
import { useAutoSave } from "../../hooks/useAutoSave";
import { isTauri } from "../../lib/backend";

interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  useAutoSave();
  const { error, clearError } = useProjectStore();
  const hasTitleBar = isTauri();

  return (
    <div className={`flex min-h-screen bg-app-bg text-app-text ${hasTitleBar ? "pt-[32px]" : ""}`}>
      <TitleBar />
      <Sidebar />
      <main className="ml-sidebar flex-1 overflow-y-auto" style={{ height: hasTitleBar ? "calc(100vh - 32px)" : "100vh" }}>
        {error && (
          <div className="flex items-center gap-2 bg-app-error-bg px-4 py-2.5 text-sm text-app-error-text">
            <span className="flex-1">{error}</span>
            <button
              onClick={clearError}
              className="shrink-0 rounded p-0.5 hover:bg-red-100"
              aria-label="Sluiten"
            >
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
                <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
              </svg>
            </button>
          </div>
        )}
        {children}
      </main>
      <ToastContainer />
      <ConflictDialog />
    </div>
  );
}
