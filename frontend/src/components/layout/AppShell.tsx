import { useState, useEffect, type ReactNode } from "react";

import { useProjectStore } from "../../store/projectStore";
import { getSetting } from "../../tauriStore";
import TitleBar from "../TitleBar";
import Ribbon from "../ribbon/Ribbon";
import StatusBar from "../StatusBar";
import Backstage from "../backstage/Backstage";
import SettingsDialog, { applyTheme } from "../settings/SettingsDialog";
import FeedbackDialog from "../feedback/FeedbackDialog";
import { Sidebar } from "./Sidebar";
import { ToastContainer } from "../ui/Toast";
import { ConflictDialog } from "../ui/ConflictDialog";
import { useAutoSave } from "../../hooks/useAutoSave";
import "../../themes.css";

interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  useAutoSave();
  const { error, clearError } = useProjectStore();

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [backstageOpen, setBackstageOpen] = useState(false);
  const [feedbackOpen, setFeedbackOpen] = useState(false);
  const [theme, setTheme] = useState("light");

  useEffect(() => {
    getSetting("theme", "light").then((saved) => {
      setTheme(saved);
      applyTheme(saved);
    });
    // Show window once content is rendered (avoids white flash on Tauri startup)
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => getCurrentWindow().show())
      .catch(() => {});
  }, []);

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <TitleBar
        onSettingsClick={() => setSettingsOpen(true)}
        onFeedbackClick={() => setFeedbackOpen(true)}
      />
      <Ribbon onFileTabClick={() => setBackstageOpen(true)} />

      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex-1 overflow-auto bg-white">
          {error && (
            <div className="flex items-center gap-2 bg-red-50 px-4 py-2.5 text-sm text-red-700">
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
      </div>

      <StatusBar />

      <Backstage
        open={backstageOpen}
        onClose={() => setBackstageOpen(false)}
        onOpenSettings={() => setSettingsOpen(true)}
      />
      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        theme={theme}
        onThemeChange={setTheme}
      />
      <FeedbackDialog
        open={feedbackOpen}
        onClose={() => setFeedbackOpen(false)}
      />
      <ToastContainer />
      <ConflictDialog />
    </div>
  );
}
