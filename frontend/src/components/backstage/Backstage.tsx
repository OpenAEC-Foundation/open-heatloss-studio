import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";

import {
  isTauri,
  createProject,
  updateProject,
  ConflictError,
  SessionExpiredError,
} from "../../lib/backend";
import {
  openProjectFile,
  exportIfcEnergy,
  extractAndLinkConstructions,
} from "../../lib/importExport";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import { useRecentFilesStore, type RecentFile } from "../../store/recentFilesStore";
import { useDocumentsStore } from "../../store/documentsStore";
import { useModellerStore } from "../modeller/modellerStore";
import ExtensionManagerPanel from "./ExtensionManagerPanel";
import NormChoiceModal from "./NormChoiceModal";
import RecentFilesPanel from "./RecentFilesPanel";
import type { ActiveNorm } from "../../types/projectV2";
import "./Backstage.css";

const ICONS = {
  new: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><path d="M14 2v6h6"/><path d="M12 18v-6m-3 3h6"/></svg>',
  open: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>',
  save: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3H5a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2V7l-4-4z"/><path d="M17 3v4a1 1 0 01-1 1H8"/><path d="M7 14h10v7H7z"/></svg>',
  saveAs: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3H5a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2V7l-4-4z"/><path d="M17 3v4a1 1 0 01-1 1H8"/><path d="M12 12v6m-3-3h6"/></svg>',
  close: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 9l6 6m0-6l-6 6"/></svg>',
  preferences: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>',
  about: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>',
  exit: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>',
  extensions: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/></svg>',
  recent: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',
  server: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>',
  file: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><path d="M14 2v6h6"/></svg>',
  import: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>',
  vabi: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>',
};

function MenuItem({
  icon,
  label,
  shortcut,
  active,
  onClick,
}: {
  icon: string;
  label: string;
  shortcut?: string;
  active?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      className={`backstage-item${active ? " active" : ""}`}
      onClick={onClick}
    >
      <span
        className="backstage-item-icon"
        dangerouslySetInnerHTML={{ __html: icon }}
      />
      <span className="backstage-item-label">{label}</span>
      {shortcut && (
        <span className="backstage-item-shortcut">{shortcut}</span>
      )}
    </button>
  );
}

function SubMenuItem({
  icon,
  label,
  onClick,
  disabled,
}: {
  icon: string;
  label: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      className="backstage-item backstage-sub-item"
      onClick={onClick}
      disabled={disabled}
      style={{ opacity: disabled ? 0.4 : 1 }}
    >
      <span
        className="backstage-item-icon"
        style={{ width: 18, height: 18 }}
        dangerouslySetInnerHTML={{ __html: icon }}
      />
      <span className="backstage-item-label" style={{ fontSize: 12 }}>
        {label}
      </span>
    </button>
  );
}

function Divider() {
  return <div className="backstage-divider" />;
}

interface BackstageProps {
  open: boolean;
  onClose: () => void;
  onOpenSettings: () => void;
  /**
   * Opent de `NormSwitchModal` (data-conversie + back-up flow voor ISSO 51 ↔ 53).
   * Geplaatst onder "Voorkeuren" als bewust afgezonderde menu-actie, niet als
   * toggle in de chrome — mixed-use (woon + utiliteit) is niet ondersteund.
   */
  onOpenNormSwitch?: () => void;
  onNavigate?: (path: string) => void;
}

export default function Backstage({
  open,
  onClose,
  onOpenSettings,
  onOpenNormSwitch,
  onNavigate,
}: BackstageProps) {
  const { t } = useTranslation("backstage");
  const [activePanel, setActivePanel] = useState<string>("none");
  // Openen + Opslaan als zijn standaard uitgeklapt zodat de Lokaal /
  // Server sub-items direct zichtbaar zijn (anders moet de gebruiker
  // eerst op het hoofd-item klikken om het submenu te ontdekken).
  const [openExpanded, setOpenExpanded] = useState(true);
  const [saveAsExpanded, setSaveAsExpanded] = useState(true);
  const [importExpanded, setImportExpanded] = useState(true);
  /**
   * Fase 2 ISSO 53: norm-keuze modal vóór een nieuw project aangemaakt
   * wordt. `null` = modal dicht, anders = open en wachtend op user-keuze.
   */
  const [showNormChoice, setShowNormChoice] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const addToast = useToastStore((s) => s.addToast);

  const project = useProjectStore((s) => s.project);
  const result = useProjectStore((s) => s.result);
  const activeProjectId = useProjectStore((s) => s.activeProjectId);
  const serverUpdatedAt = useProjectStore((s) => s.serverUpdatedAt);
  const setProject = useProjectStore((s) => s.setProject);
  const setActiveProjectId = useProjectStore((s) => s.setActiveProjectId);
  const setServerUpdatedAt = useProjectStore((s) => s.setServerUpdatedAt);
  const setNorm = useProjectStore((s) => s.setNorm);
  const reset = useProjectStore((s) => s.reset);

  const resetToExample = useModellerStore((s) => s.resetToExample);
  const isWeb = !isTauri();

  const actionAndClose = useCallback(
    (fn?: () => void) => {
      onClose();
      fn?.();
    },
    [onClose],
  );

  // Map a server-save failure to a meaningful toast. A bare "save error"
  // string hides the real cause (most often an expired Authentik session,
  // which otherwise surfaces as a cryptic JSON parse error).
  const reportSaveError = useCallback(
    (err: unknown) => {
      if (err instanceof SessionExpiredError) {
        addToast(t("sessionExpired"), "error", 8000, {
          label: t("loginAgain"),
          onClick: () => window.location.reload(),
        });
      } else if (err instanceof ConflictError) {
        addToast(t("conflictError"), "error", 6000);
      } else {
        const detail = err instanceof Error ? err.message : String(err);
        addToast(`${t("saveError")}: ${detail}`, "error");
      }
    },
    [addToast, t],
  );

  useEffect(() => {
    if (!open) {
      setActivePanel("none");
      return;
    }
    // Bij elke opening: forceer sub-items uitgeklapt zodat user direct de
    // Lokaal/Server/Vabi sub-keuzes ziet zonder eerst te moeten klikken.
    setOpenExpanded(true);
    setSaveAsExpanded(true);
    setImportExpanded(true);
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  // --- File actions ---

  const handleNew = useCallback(() => {
    // Fase 2 ISSO 53: norm-keuze gaat vooraf aan de feitelijke
    // project-aanmaak. Backstage blijft open totdat de user of bevestigt
    // of annuleert; de eigenlijke `newTab()` flow verhuist naar
    // `handleNormConfirm`.
    setShowNormChoice(true);
  }, []);

  const handleNormConfirm = useCallback(
    (norm: ActiveNorm) => {
      setShowNormChoice(false);
      try {
        // Tabbed views: open een nieuwe tab i.p.v. de huidige te overschrijven.
        // De documents-store snapshot eerst de huidige tab, reset projectStore
        // naar leeg, en zet de nieuwe tab actief.
        useDocumentsStore.getState().newTab();
        resetToExample();
        // newTab() roept reset() aan, die norm naar default "isso51" zet.
        // Daarna zetten we de door de user gekozen norm.
        setNorm(norm);
        onClose();
        onNavigate?.("/project");
        addToast(t("newProject"), "info");
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        addToast(`Nieuw mislukt: ${msg}`, "error");
        // eslint-disable-next-line no-console
        console.error("[backstage] handleNormConfirm failed:", err);
      }
    },
    [resetToExample, setNorm, onClose, onNavigate, addToast, t],
  );

  // Vabi import handler — momenteel disabled in de UI omdat de Vabi-keten
  // nog in ontwikkeling is. Verwijderd uit de file; herinschakelen vereist
  // dat het SubMenuItem onClick weer wijst naar een nieuwe handleImportVabi
  // implementatie (Git-history bevat de oude versie).

  const handleOpenServer = useCallback(() => {
    onClose();
    onNavigate?.("/projects");
  }, [onClose, onNavigate]);

  const handleOpenLocal = useCallback(async () => {
    // Tauri: native open dialog met .ifcenergy filter — geeft een echt pad
    // terug zodat de recent-files-lijst persistent kan refereren.
    if (isTauri()) {
      try {
        const [{ open }, { readTextFile }] = await Promise.all([
          import("@tauri-apps/plugin-dialog"),
          import("@tauri-apps/plugin-fs"),
        ]);
        const selected = await open({
          multiple: false,
          filters: [
            {
              name: "Open Heatloss Studio Project",
              extensions: ["ifcenergy", "json", "isso51.json"],
            },
          ],
        });
        if (!selected || Array.isArray(selected)) return;
        const text = await readTextFile(selected);
        const imported = openProjectFile(text);
        if (imported.type === "thermal") {
          addToast(
            "Thermal import gedetecteerd — open via de wizard i.p.v. Bestand",
            "info",
          );
          return;
        }
        // Tabbed views: nieuwe tab voor het geopende project
        useDocumentsStore.getState().newTab(
          imported.project.info?.name ||
            selected.split(/[\\/]/).pop() ||
            "Lokaal bestand",
        );
        extractAndLinkConstructions(imported.project);
        setProject(imported.project);
        useProjectStore.getState().setCurrentLocalPath(selected);
        if (imported.result) {
          useProjectStore.getState().setResult(imported.result);
        }
        const fileName = selected.split(/[\\/]/).pop() ?? "project.ifcenergy";
        useRecentFilesStore.getState().push({
          name: imported.project.info.name || fileName,
          fileName,
          path: selected,
        });
        onClose();
        onNavigate?.("/rooms");
        addToast(t("opened"), "success");
        return;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        addToast(`${t("importError")}: ${msg}`, "error");
        return;
      }
    }
    // Browser fallback: hidden file input
    fileInputRef.current?.click();
  }, [setProject, onClose, onNavigate, addToast, t]);

  const handleFileSelected = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        const imported = openProjectFile(text);

        // Thermal import detected — redirect to wizard with raw JSON
        if (imported.type === "thermal") {
          onClose();
          onNavigate?.("/import/thermal");
          // Store raw JSON in sessionStorage so the wizard can pick it up
          sessionStorage.setItem("thermalImportJson", imported.rawJson);
          addToast("Thermal import gedetecteerd — wizard geopend", "info");
          e.target.value = "";
          return;
        }

        // Regular project import
        extractAndLinkConstructions(imported.project);
        setProject(imported.project);
        if (imported.result) {
          useProjectStore.getState().setResult(imported.result);
        }
        // Track in recent files (file-input has no absolute path in browser,
        // so we record just the file name + project name).
        useRecentFilesStore.getState().push({
          name: imported.project.info.name || file.name,
          fileName: file.name,
        });
        onClose();
        onNavigate?.("/rooms");
        addToast(t("opened"), "success");
      } catch (err) {
        addToast(
          `${t("importError")}: ${err instanceof Error ? err.message : String(err)}`,
          "error",
        );
      }

      // Reset file input so the same file can be selected again
      e.target.value = "";
    },
    [setProject, onClose, onNavigate, addToast, t],
  );

  const recent = useRecentFilesStore((s) => s.recent);
  const removeRecent = useRecentFilesStore((s) => s.remove);
  const clearRecent = useRecentFilesStore((s) => s.clear);

  const handleOpenRecent = useCallback(
    async (entry: RecentFile) => {
      // Tauri path: lees via plugin-fs als we een absoluut pad hebben
      if (entry.path && isTauri()) {
        try {
          const { readTextFile } = await import("@tauri-apps/plugin-fs");
          const text = await readTextFile(entry.path);
          const imported = openProjectFile(text);
          if (imported.type === "thermal") {
            addToast(
              "Recent: thermal-import bestand, open via Bestand → Openen",
              "info",
            );
            return;
          }
          extractAndLinkConstructions(imported.project);
          setProject(imported.project);
          useProjectStore.getState().setCurrentLocalPath(entry.path);
          if (imported.result) {
            useProjectStore.getState().setResult(imported.result);
          }
          useRecentFilesStore.getState().push({
            name: imported.project.info.name || entry.fileName,
            fileName: entry.fileName,
            path: entry.path,
          });
          onClose();
          onNavigate?.("/rooms");
          addToast(t("opened"), "success");
          return;
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          addToast(`Recent openen mislukt: ${msg}`, "error");
          // Fall through to the file-picker hint below.
        }
      }
      // Browser of geen pad: trigger de file-input zodat user opnieuw selecteert
      addToast(
        "Kies het bestand opnieuw — recent-lijst houdt geen browser-pad bij.",
        "info",
      );
      fileInputRef.current?.click();
    },
    [setProject, onClose, onNavigate, addToast, t],
  );

  const handleSave = useCallback(async () => {
    if (activeProjectId && isWeb) {
      // Server save — update existing project
      try {
        const resp = await updateProject(activeProjectId, {
          project_data: project,
          expected_updated_at: serverUpdatedAt ?? undefined,
        });
        setServerUpdatedAt(resp.updated_at);
        onClose();
        addToast(t("savedToServer"), "success");
      } catch (err) {
        reportSaveError(err);
      }
    } else if (isWeb) {
      // Server save — new project, prompt for name
      const name = window.prompt(
        t("projectNamePrompt"),
        project.info.name || "",
      );
      if (!name) return;
      try {
        const resp = await createProject(name, project);
        setActiveProjectId(resp.id);
        onClose();
        addToast(t("savedToServer"), "success");
      } catch (err) {
        reportSaveError(err);
      }
    } else {
      // Not logged in — schrijf als .ifcenergy. Bestand → Opslaan moet
      // ALTIJD silent: gebruik currentLocalPath als bekend, anders derive
      // een default-pad in <Documents>/Open Heatloss Studio/<naam>.ifcenergy.
      // Save As (apart menu-item) toont wel de dialog.
      try {
        let targetPath = useProjectStore.getState().currentLocalPath;
        if (!targetPath && isTauri()) {
          const safe = (project.info.name || "project")
            .replace(/[^a-zA-Z0-9_\-\s]/g, "")
            .trim() || "project";
          const [{ documentDir, join }, { mkdir }] = await Promise.all([
            import("@tauri-apps/api/path"),
            import("@tauri-apps/plugin-fs"),
          ]);
          const docs = await documentDir();
          const folder = await join(docs, "Open Heatloss Studio");
          try {
            await mkdir(folder, { recursive: true });
          } catch {
            // Folder bestaat al — geen probleem.
          }
          targetPath = await join(folder, `${safe}.ifcenergy`);
        }
        const writtenPath = await exportIfcEnergy(project, result, targetPath);
        if (writtenPath) {
          useProjectStore.getState().setCurrentLocalPath(writtenPath);
        }
        onClose();
        addToast(t("savedLocally"), "success");
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        addToast(`${t("saveError")}: ${msg}`, "error");
        // eslint-disable-next-line no-console
        console.error("[backstage] exportIfcEnergy failed:", err);
      }
    }
  }, [
    activeProjectId,
    isWeb,
    project,
    result,
    serverUpdatedAt,
    setActiveProjectId,
    setServerUpdatedAt,
    onClose,
    addToast,
    reportSaveError,
    t,
  ]);

  const handleSaveAsServer = useCallback(async () => {
    const name = window.prompt(
      t("projectNamePrompt"),
      project.info.name || "",
    );
    if (!name) return;
    try {
      const resp = await createProject(name, project);
      setActiveProjectId(resp.id);
      onClose();
      addToast(t("savedToServer"), "success");
    } catch (err) {
      reportSaveError(err);
    }
  }, [project, setActiveProjectId, onClose, addToast, reportSaveError, t]);

  const handleSaveAsLocal = useCallback(async () => {
    // "Opslaan als" → altijd save-as dialog, ook als currentLocalPath bekend.
    try {
      const writtenPath = await exportIfcEnergy(project, result, undefined);
      if (writtenPath) {
        useProjectStore.getState().setCurrentLocalPath(writtenPath);
      }
      onClose();
      addToast(t("savedLocally"), "success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      addToast(`${t("saveError")}: ${msg}`, "error");
      // eslint-disable-next-line no-console
      console.error("[backstage] exportIfcEnergy (saveAs) failed:", err);
    }
  }, [project, result, onClose, addToast, t]);

  const handleClose = useCallback(() => {
    reset();
    onClose();
    addToast(t("closed"), "info");
  }, [reset, onClose, addToast, t]);

  if (!open) return null;

  return (
    <>
      {/* Transparente fullscreen click-catcher achter de sidebar:
          klik buiten de backstage sluit hem, maar de app blijft zichtbaar. */}
      <div className="backstage-backdrop" onClick={onClose} />
      <div className="backstage-overlay">
        <div className="backstage-sidebar">
        <button className="backstage-back" onClick={onClose}>
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M19 12H5M12 19l-7-7 7-7" />
          </svg>
          <span>{t("file")}</span>
        </button>
        <div className="backstage-items">
          {/* Nieuw */}
          <MenuItem
            icon={ICONS.new}
            label={t("new")}
            shortcut="Ctrl+N"
            onClick={handleNew}
          />

          {/* Openen */}
          <MenuItem
            icon={ICONS.open}
            label={t("open")}
            shortcut="Ctrl+O"
            onClick={() => setOpenExpanded((v) => !v)}
          />
          {openExpanded && (
            <>
              {isWeb && (
                <SubMenuItem
                  icon={ICONS.server}
                  label={t("fromServer")}
                  onClick={handleOpenServer}
                />
              )}
              <SubMenuItem
                icon={ICONS.file}
                label={t("localFile")}
                onClick={handleOpenLocal}
              />
              {!isWeb && (
                <SubMenuItem
                  icon={ICONS.vabi}
                  label={`${t("vabiElements")} — in ontwikkeling`}
                  onClick={() => {
                    addToast(
                      "Vabi import is nog in ontwikkeling — werkt nog niet betrouwbaar.",
                      "info",
                    );
                  }}
                  disabled={true}
                />
              )}
              {recent.length > 0 && (
                <div className="mt-2 ml-3 border-l border-border pl-3">
                  <div className="mb-1 flex items-center justify-between pr-1">
                    <span className="text-[10px] font-semibold uppercase tracking-wide text-on-surface-muted">
                      Recent
                    </span>
                    <button
                      type="button"
                      onClick={() => clearRecent()}
                      className="text-[10px] text-on-surface-muted hover:text-on-surface-secondary"
                      title="Lijst wissen"
                    >
                      wissen
                    </button>
                  </div>
                  {recent.map((entry) => (
                    <div
                      key={(entry.path ?? "") + entry.fileName + entry.openedAt}
                      className="group flex items-center justify-between gap-2 rounded px-2 py-1 text-xs text-on-surface-secondary hover:bg-[var(--oaec-hover)]"
                    >
                      <button
                        type="button"
                        onClick={() => handleOpenRecent(entry)}
                        className="min-w-0 flex-1 text-left"
                        title={entry.path ?? entry.fileName}
                      >
                        <div className="truncate text-on-surface">{entry.name}</div>
                        <div className="truncate text-[10px] text-on-surface-muted">
                          {entry.fileName}
                        </div>
                      </button>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeRecent(entry);
                        }}
                        className="opacity-0 group-hover:opacity-100 text-on-surface-muted hover:text-on-surface-secondary"
                        title="Uit lijst halen"
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </>
          )}

          {/* Opslaan */}
          <MenuItem
            icon={ICONS.save}
            label={t("save")}
            shortcut="Ctrl+S"
            onClick={handleSave}
          />

          {/* Opslaan als */}
          <MenuItem
            icon={ICONS.saveAs}
            label={t("saveAs")}
            shortcut="Ctrl+Shift+S"
            onClick={() => setSaveAsExpanded((v) => !v)}
          />
          {saveAsExpanded && (
            <>
              {isWeb && (
                <SubMenuItem
                  icon={ICONS.server}
                  label={t("toServer")}
                  onClick={handleSaveAsServer}
                />
              )}
              <SubMenuItem
                icon={ICONS.file}
                label={t("localExport")}
                onClick={handleSaveAsLocal}
              />
            </>
          )}

          {/* Importeer */}
          {!isWeb && (
            <>
              <MenuItem
                icon={ICONS.import}
                label={t("import")}
                onClick={() => setImportExpanded((v) => !v)}
              />
              {importExpanded && (
                <SubMenuItem
                  icon={ICONS.vabi}
                  label={`${t("vabiElements")} — in ontwikkeling`}
                  onClick={() => {
                    addToast(
                      "Vabi import is nog in ontwikkeling — werkt nog niet betrouwbaar.",
                      "info",
                    );
                  }}
                  disabled={true}
                />
              )}
            </>
          )}

          <Divider />

          {/* Sluiten */}
          <MenuItem
            icon={ICONS.close}
            label={t("close")}
            onClick={handleClose}
          />

          <Divider />

          {/* Voorkeuren */}
          <MenuItem
            icon={ICONS.preferences}
            label={t("preferences")}
            shortcut="Ctrl+,"
            onClick={() => actionAndClose(onOpenSettings)}
          />

          {/* Norm wisselen (ISSO 51 ↔ 53) — verplaatst vanuit de TitleBar */}
          {onOpenNormSwitch && (
            <MenuItem
              icon={ICONS.preferences}
              label={t("normSwitchEntry")}
              onClick={() => actionAndClose(onOpenNormSwitch)}
            />
          )}

          <Divider />

          {/* Recent files */}
          <MenuItem
            icon={ICONS.recent}
            label={t("recent")}
            active={activePanel === "recent"}
            onClick={() => setActivePanel("recent")}
          />

          {/* Extensies */}
          <MenuItem
            icon={ICONS.extensions}
            label={t("extensions")}
            active={activePanel === "extensions"}
            onClick={() => setActivePanel("extensions")}
          />

          {/* Over */}
          <MenuItem
            icon={ICONS.about}
            label={t("about")}
            active={activePanel === "about"}
            onClick={() => setActivePanel("about")}
          />

          <Divider />

          {/* Afsluiten */}
          <MenuItem
            icon={ICONS.exit}
            label={t("exit")}
            shortcut="Alt+F4"
            onClick={() => {
              onClose();
              import("@tauri-apps/api/window")
                .then(({ getCurrentWindow }) => getCurrentWindow().close())
                .catch(() => {
                  /* web mode — no-op */
                });
            }}
          />
        </div>
      </div>
      {activePanel !== "none" && (
        <div className="backstage-content">
          {activePanel === "about" && <AboutPanel />}
          {activePanel === "extensions" && <ExtensionManagerPanel />}
          {activePanel === "recent" && (
            <RecentFilesPanel
              onOpen={async (entry) => {
                await handleOpenRecent(entry);
              }}
            />
          )}
        </div>
      )}

      {/* Hidden file input for local open */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".ifcenergy,.json,.isso51.json"
        onChange={handleFileSelected}
        style={{ display: "none" }}
      />
      </div>

      {/* Fase 2 ISSO 53: norm-keuze modal — getoond bij Bestand → Nieuw. */}
      <NormChoiceModal
        open={showNormChoice}
        onClose={() => setShowNormChoice(false)}
        onConfirm={handleNormConfirm}
        defaultNorm="isso51"
      />
    </>
  );
}

function AboutPanel() {
  const { t } = useTranslation("backstage");
  return (
    <div className="bs-about-panel">
      <h2 className="bs-about-title">{t("aboutPanel.title")}</h2>
      <div className="bs-about-app">
        <div className="bs-about-logo">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="var(--theme-accent)"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M3 21h18M5 21V7l8-4v18M19 21V11l-6-4" />
          </svg>
        </div>
        <div className="bs-about-app-info">
          <h1 className="bs-about-app-name">Open Heatloss Studio</h1>
          <p className="bs-about-version">{t("aboutPanel.version")} {__APP_VERSION__}</p>
        </div>
      </div>
      <p className="bs-about-tagline">Warmteverliesberekening volgens ISSO 51:2023</p>
      <p className="bs-about-description">
        Complete tool voor warmteverliesberekeningen volgens de ISSO 51 norm.
        Bruikbaar als web applicatie, desktop app (Tauri) en rekenbibliotheek.
      </p>
      <div className="bs-about-company">
        <h3 className="bs-about-company-name">OpenAEC</h3>
        <p className="bs-about-company-desc">
          Open source engineering tools voor de gebouwde omgeving.
        </p>
      </div>
      <div className="bs-about-links">
        <a href="https://open-aec.com" className="bs-about-link" target="_blank" rel="noreferrer">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10" />
            <path d="M2 12h20M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10A15.3 15.3 0 0112 2z" />
          </svg>
          {t("aboutPanel.website")}
        </a>
        <a href="https://github.com/3bm-bouwkunde" className="bs-about-link" target="_blank" rel="noreferrer">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 00-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0020 4.77 5.07 5.07 0 0019.91 1S18.73.65 16 2.48a13.38 13.38 0 00-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 005 4.77a5.44 5.44 0 00-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 009 18.13V22" />
          </svg>
          {t("aboutPanel.github")}
        </a>
      </div>
      <div className="bs-about-footer">
        <p className="bs-about-copyright">
          &copy; 2025 3BM Bouwkunde Cooperatie. Licensed under MIT.
        </p>
      </div>
    </div>
  );
}
