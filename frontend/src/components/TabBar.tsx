/**
 * TabBar — horizontaal balkje boven de main content met geopende project-tabs.
 *
 * Click op tab → switch. ✕ knop → close. "+" knop → nieuwe lege tab.
 * Active tab heeft accent-styling. Dirty tabs tonen een ●.
 *
 * Style is OpenAEC huisstijl (theme-aware via CSS custom properties).
 * Hoort tussen TitleBar en main content area in AppShell.
 */
import { useEffect } from "react";

import { useDocumentsStore } from "../store/documentsStore";
import { useProjectStore } from "../store/projectStore";
import "./TabBar.css";

/** Auto-initialiseer een eerste tab als de store leeg is bij eerste render. */
function useEnsureFirstTab() {
  const tabs = useDocumentsStore((s) => s.tabs);
  const newTab = useDocumentsStore((s) => s.newTab);
  useEffect(() => {
    if (tabs.length === 0) {
      // Pak het project-name uit projectStore (mogelijk al via persist
      // ingeladen) zodat de eerste tab een betekenisvolle naam krijgt.
      const projName = useProjectStore.getState().project?.info?.name?.trim();
      newTab(projName && projName.length > 0 ? projName : undefined);
    }
  }, [tabs.length, newTab]);
}

/** Sync project naam → actieve tab naam (bij elke wijziging van project.info.name). */
function useSyncActiveTabName() {
  const projectName = useProjectStore((s) => s.project?.info?.name ?? "");
  const isDirty = useProjectStore((s) => s.isDirty);
  const setActiveName = useDocumentsStore((s) => s.setActiveName);
  const setActiveDirty = useDocumentsStore((s) => s.setActiveDirty);
  useEffect(() => {
    if (projectName.trim().length > 0) {
      setActiveName(projectName);
    }
  }, [projectName, setActiveName]);
  useEffect(() => {
    setActiveDirty(isDirty);
  }, [isDirty, setActiveDirty]);
}

export default function TabBar() {
  useEnsureFirstTab();
  useSyncActiveTabName();

  const tabs = useDocumentsStore((s) => s.tabs);
  const activeId = useDocumentsStore((s) => s.activeId);
  const switchTab = useDocumentsStore((s) => s.switchTab);
  const closeTab = useDocumentsStore((s) => s.closeTab);
  const newTab = useDocumentsStore((s) => s.newTab);

  return (
    <div className="tabbar" role="tablist">
      {tabs.map((tab) => {
        const active = tab.id === activeId;
        return (
          <div
            key={tab.id}
            role="tab"
            aria-selected={active}
            className={`tabbar-tab${active ? " tabbar-tab-active" : ""}`}
            onClick={() => switchTab(tab.id)}
            title={tab.name}
          >
            {tab.isDirty && <span className="tabbar-dirty">●</span>}
            <span className="tabbar-label">{tab.name || "Naamloos"}</span>
            <button
              type="button"
              className="tabbar-close"
              onClick={(e) => {
                e.stopPropagation();
                closeTab(tab.id);
              }}
              title="Tab sluiten"
              aria-label={`Tab ${tab.name} sluiten`}
            >
              ×
            </button>
          </div>
        );
      })}
      <button
        type="button"
        className="tabbar-new"
        onClick={() => newTab()}
        title="Nieuwe tab"
        aria-label="Nieuwe tab"
      >
        +
      </button>
    </div>
  );
}
