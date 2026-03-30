import { useState, useEffect, useCallback, type ReactNode } from "react";
import { useNavigate, useLocation } from "react-router-dom";

import { useAuth } from "../../hooks/useAuth";
import { updateProject, createProject } from "../../lib/backend";
import { exportProject } from "../../lib/importExport";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import i18next from "../../i18n/config";
import { getSetting } from "../../tauriStore";
import { useModellerStore } from "../modeller/modellerStore";
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
  const navigate = useNavigate();
  const location = useLocation();
  const { isLoggedIn } = useAuth();
  const { error, clearError } = useProjectStore();
  const addToast = useToastStore((s) => s.addToast);

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

  // --- Global keyboard shortcuts ---
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Skip when user is typing in an input/textarea
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      const ctrl = e.ctrlKey || e.metaKey;

      if (ctrl && !e.shiftKey && e.key === "n") {
        e.preventDefault();
        useProjectStore.getState().reset();
        useModellerStore.getState().resetToExample();
        navigate("/project");
        addToast(i18next.t("newProject"), "info");
        return;
      }

      if (ctrl && !e.shiftKey && e.key === "o") {
        e.preventDefault();
        setBackstageOpen(true);
        return;
      }

      if (ctrl && !e.shiftKey && e.key === "s") {
        e.preventDefault();
        const state = useProjectStore.getState();
        if (state.activeProjectId && isLoggedIn) {
          updateProject(state.activeProjectId, {
            project_data: state.project,
            expected_updated_at: state.serverUpdatedAt ?? undefined,
          })
            .then((resp) => {
              useProjectStore.getState().setServerUpdatedAt(resp.updated_at);
              addToast(i18next.t("savedToServer"), "success");
            })
            .catch((err) => {
              addToast(
                `${i18next.t("saveFailed")}: ${err instanceof Error ? err.message : String(err)}`,
                "error",
              );
            });
        } else if (isLoggedIn) {
          const name = window.prompt(
            i18next.t("projectNamePrompt"),
            state.project.info.name || "",
          );
          if (name) {
            createProject(name, state.project)
              .then((resp) => {
                useProjectStore.getState().setActiveProjectId(resp.id);
                addToast(i18next.t("savedToServer"), "success");
              })
              .catch((err) => {
                addToast(
                  `${i18next.t("saveFailed")}: ${err instanceof Error ? err.message : String(err)}`,
                  "error",
                );
              });
          }
        } else {
          exportProject(state.project, state.result);
          addToast(i18next.t("savedLocally"), "success");
        }
        return;
      }

      if (ctrl && e.shiftKey && e.key === "S") {
        e.preventDefault();
        const state = useProjectStore.getState();
        exportProject(state.project, state.result);
        addToast("Lokaal opgeslagen", "success");
        return;
      }

      // Undo/Redo
      if (ctrl && !e.shiftKey && e.key === "z") {
        e.preventDefault();
        if (location.pathname === "/modeller") {
          useModellerStore.getState().undo();
        } else {
          useProjectStore.getState().undo();
        }
        return;
      }

      if (ctrl && (e.key === "y" || (e.shiftKey && e.key === "Z"))) {
        e.preventDefault();
        if (location.pathname === "/modeller") {
          useModellerStore.getState().redo();
        } else {
          useProjectStore.getState().redo();
        }
        return;
      }
    },
    [navigate, isLoggedIn, addToast, location.pathname],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <TitleBar
        onSettingsClick={() => setSettingsOpen(true)}
        onFeedbackClick={() => setFeedbackOpen(true)}
      />
      <Ribbon onFileTabClick={() => setBackstageOpen(true)} />

      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex-1 overflow-auto bg-surface text-on-surface">
          {error && (
            <div className="flex items-center gap-2 bg-red-600/15 border-b border-red-600/30 px-4 py-2.5 text-sm text-red-400">
              <span className="flex-1">{error}</span>
              <button
                onClick={clearError}
                className="shrink-0 rounded p-0.5 hover:bg-red-600/20"
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
        onNavigate={navigate}
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
