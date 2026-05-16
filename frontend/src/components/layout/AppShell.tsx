import { useState, useEffect, useCallback, type ReactNode } from "react";
import { useNavigate, useLocation } from "react-router-dom";

import { useAuth } from "../../hooks/useAuth";
import { updateProject, createProject, isTauri } from "../../lib/backend";
import {
  exportIfcEnergy,
  openProjectFile,
  extractAndLinkConstructions,
} from "../../lib/importExport";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import { useRecentFilesStore } from "../../store/recentFilesStore";
import i18next from "../../i18n/config";
import { getSetting } from "../../tauriStore";
import { useModellerStore } from "../modeller/modellerStore";
import TitleBar from "../TitleBar";
import TabBar from "../TabBar";
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

/** Derive een default save-pad in Tauri zodat "Bestand → Opslaan" zelfs
 * zonder eerder save-as nooit een dialog hoeft te tonen.
 *
 * Pad: `<Documents>/Open Heatloss Studio/<safeProjectName>.ifcenergy`.
 * Map wordt aangemaakt als 'ie nog niet bestaat. In web-mode (geen Tauri)
 * retourneren we `null` — daar regelt exportIfcEnergy een blob-download.
 */
async function deriveDefaultSavePath(
  projectName: string,
): Promise<string | null> {
  if (!isTauri()) return null;
  try {
    const [{ documentDir, join }, { mkdir }] = await Promise.all([
      import("@tauri-apps/api/path"),
      import("@tauri-apps/plugin-fs"),
    ]);
    const docs = await documentDir();
    const folder = await join(docs, "Open Heatloss Studio");
    await mkdir(folder, { recursive: true }).catch(() => {
      // Already exists — fine. Other errors fall through.
    });
    const safe = (projectName || "project")
      .replace(/[^a-zA-Z0-9_\-\s]/g, "")
      .trim() || "project";
    return await join(folder, `${safe}.ifcenergy`);
  } catch {
    return null;
  }
}

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

  // ─── File-association handling ────────────────────────────────────────
  // When Windows opens a `.ifcenergy` file via double-click, Tauri's backend
  // emits an `open-file` event with the absolute path. Listen once and run
  // the same import pipeline as Bestand → Openen → Lokaal bestand.
  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    (async () => {
      try {
        const [{ listen }, { readTextFile }, { invoke }] = await Promise.all([
          import("@tauri-apps/api/event"),
          import("@tauri-apps/plugin-fs"),
          import("@tauri-apps/api/core"),
        ]);

        // Helper that imports the project at `path` and routes the user
        // to /rooms — identical to Backstage.handleFileSelected.
        const importPath = async (path: string) => {
          try {
            const text = await readTextFile(path);
            const imported = openProjectFile(text);
            if (imported.type === "thermal") {
              addToast(
                "Thermal import — open via de wizard i.p.v. dubbelklik",
                "info",
              );
              return;
            }
            extractAndLinkConstructions(imported.project);
            useProjectStore.getState().setProject(imported.project);
            // setProject reset currentLocalPath naar null; daarna pas
            // het echte pad zetten zodat Bestand → Opslaan stil terug-
            // schrijft naar dit bestand.
            useProjectStore.getState().setCurrentLocalPath(path);
            if (imported.result) {
              useProjectStore.getState().setResult(imported.result);
            }
            const fileName = path.split(/[\\/]/).pop() ?? "project.ifcenergy";
            useRecentFilesStore.getState().push({
              name: imported.project.info.name || fileName,
              fileName,
              path,
            });
            navigate("/rooms");
            addToast(i18next.t("backstage:opened"), "success");
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            addToast(`Openen mislukt: ${msg}`, "error");
          }
        };

        // 1. Cold-launch: check if argv carried a path
        try {
          const initial = await invoke<string | null>("launched_with_file");
          if (initial && !cancelled) {
            void importPath(initial);
          }
        } catch {
          // Tauri command missing (dev mode or older build) — ignore
        }

        // 2. Subsequent open-file events (single-instance pickup)
        unlisten = await listen<string>("open-file", (event) => {
          if (cancelled) return;
          if (typeof event.payload === "string" && event.payload.length > 0) {
            void importPath(event.payload);
          }
        });
      } catch {
        // Event API or fs plugin failed to load — silently skip
      }
    })();

    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }, [navigate, addToast]);

  // --- Save action ---
  // Eén canonieke save-flow voor zowel het TitleBar quick-access knopje
  // als de Ctrl+S keyboard shortcut. Logged-in op web → upsert naar server;
  // anders (Tauri desktop of anoniem web) → exporteer als .ifcenergy via de
  // native save-dialog (Tauri) of een browser-download (web fallback).
  const performSave = useCallback(async () => {
    const state = useProjectStore.getState();
    if (state.activeProjectId && isLoggedIn) {
      try {
        const resp = await updateProject(state.activeProjectId, {
          project_data: state.project,
          expected_updated_at: state.serverUpdatedAt ?? undefined,
        });
        useProjectStore.getState().setServerUpdatedAt(resp.updated_at);
        addToast(i18next.t("savedToServer"), "success");
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        addToast(`${i18next.t("saveFailed")}: ${msg}`, "error");
      }
      return;
    }
    if (isLoggedIn) {
      const name = window.prompt(
        i18next.t("projectNamePrompt"),
        state.project.info.name || "",
      );
      if (!name) return;
      try {
        const resp = await createProject(name, state.project);
        useProjectStore.getState().setActiveProjectId(resp.id);
        addToast(i18next.t("savedToServer"), "success");
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        addToast(`${i18next.t("saveFailed")}: ${msg}`, "error");
      }
      return;
    }
    // Anoniem of Tauri-desktop: opslaan als .ifcenergy.
    // Bestand → Opslaan moet ALTIJD silent:
    //   1. currentLocalPath bekend → schrijf daar
    //   2. anders (Tauri) → derive default-pad in Documents/Open Heatloss
    //      Studio/<projectnaam>.ifcenergy (maak dir aan indien nodig) en
    //      schrijf daar — geen dialog.
    //   3. web fallback → blob-download (exportIfcEnergy regelt dat)
    try {
      let targetPath = state.currentLocalPath;
      if (!targetPath) {
        targetPath = await deriveDefaultSavePath(state.project.info.name);
      }
      const writtenPath = await exportIfcEnergy(
        state.project,
        state.result,
        targetPath,
      );
      if (writtenPath) {
        useProjectStore.getState().setCurrentLocalPath(writtenPath);
      }
      addToast(i18next.t("savedLocally"), "success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      addToast(`Opslaan mislukt: ${msg}`, "error");
    }
  }, [isLoggedIn, addToast]);

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
        void performSave();
        return;
      }

      if (ctrl && e.shiftKey && e.key === "S") {
        // Ctrl+Shift+S = "Opslaan als" — altijd dialog, ook als
        // currentLocalPath bekend is. Update pad na succesvolle save.
        e.preventDefault();
        const state = useProjectStore.getState();
        exportIfcEnergy(state.project, state.result, undefined)
          .then((writtenPath) => {
            if (writtenPath) {
              useProjectStore.getState().setCurrentLocalPath(writtenPath);
            }
            addToast("Lokaal opgeslagen", "success");
          })
          .catch((err) => {
            const msg = err instanceof Error ? err.message : String(err);
            addToast(`Opslaan mislukt: ${msg}`, "error");
          });
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
    [navigate, addToast, location.pathname, performSave],
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
        onSave={performSave}
      />
      <Ribbon onFileTabClick={() => setBackstageOpen(true)} />
      <TabBar />

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
