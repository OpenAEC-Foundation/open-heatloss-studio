import { useState, useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useLocation, useNavigate } from "react-router-dom";
import RibbonTab from "./RibbonTab";
import VertrekkenTab from "./VertrekkenTab";
import ConstructiesTab from "./ConstructiesTab";
import ModellerTab from "./ModellerTab";
import ResultatenTab from "./ResultatenTab";
import RapportTab from "./RapportTab";
import IfcTab from "./IfcTab";
import "./Ribbon.css";

interface RibbonProps {
  onFileTabClick?: () => void;
}

const TABS = [
  "vertrekken",
  "constructies",
  "modeller",
  "ifc",
  "resultaten",
  "rapport",
] as const;
type TabId = (typeof TABS)[number];

/** Map routes to their corresponding ribbon tab. */
const ROUTE_TO_TAB: Record<string, TabId> = {
  "/project": "vertrekken",
  "/rooms": "vertrekken",
  "/constructies": "constructies",
  "/rc": "constructies",
  "/library": "constructies",
  "/materialen": "constructies",
  "/modeller": "modeller",
  "/ifc": "ifc",
  "/results": "resultaten",
  "/rapport": "rapport",
};

/** Map tabs to the default route when clicked. */
const TAB_TO_ROUTE: Record<TabId, string> = {
  vertrekken: "/rooms",
  constructies: "/constructies",
  modeller: "/modeller",
  ifc: "/ifc",
  resultaten: "/results",
  rapport: "/rapport",
};

export default function Ribbon({ onFileTabClick }: RibbonProps) {
  const { t, i18n } = useTranslation("ribbon");
  const location = useLocation();
  const navigate = useNavigate();

  // Determine active tab from current route
  const tabFromRoute = ROUTE_TO_TAB[location.pathname] ?? "vertrekken";
  const [activeTab, setActiveTab] = useState<TabId>(tabFromRoute);
  const [prevTab, setPrevTab] = useState<TabId | null>(null);
  const [animating, setAnimating] = useState(false);
  const [direction, setDirection] = useState<"left" | "right">("right");
  const tabsRef = useRef<HTMLDivElement>(null);
  const borderRef = useRef<HTMLDivElement>(null);
  const gapRef = useRef<HTMLDivElement>(null);

  // Sync tab when route changes (e.g. sidebar navigation)
  useEffect(() => {
    const newTab = ROUTE_TO_TAB[location.pathname];
    if (newTab && newTab !== activeTab) {
      setActiveTab(newTab);
    }
  }, [location.pathname]); // eslint-disable-line react-hooks/exhaustive-deps

  const updateHighlight = useCallback(() => {
    const tabsEl = tabsRef.current;
    const borderEl = borderRef.current;
    const gapEl = gapRef.current;
    if (!tabsEl || !borderEl || !gapEl) return;

    const activeEl = tabsEl.querySelector(".ribbon-tab.active") as HTMLElement | null;
    if (!activeEl) {
      borderEl.style.opacity = "0";
      gapEl.style.opacity = "0";
      return;
    }

    const tabsRect = tabsEl.getBoundingClientRect();
    const activeRect = activeEl.getBoundingClientRect();
    const left = activeRect.left - tabsRect.left;
    const top = activeRect.top - tabsRect.top;
    const width = activeRect.width;
    const height = activeRect.height;

    borderEl.style.opacity = "1";
    borderEl.style.left = `${left}px`;
    borderEl.style.top = `${top}px`;
    borderEl.style.width = `${width}px`;
    borderEl.style.height = `${height}px`;

    gapEl.style.opacity = "1";
    gapEl.style.left = `${left + 1}px`;
    gapEl.style.width = `${width - 2}px`;
  }, []);

  const switchTab = useCallback((newTab: TabId) => {
    if (newTab === activeTab) return;
    const oldIndex = TABS.indexOf(activeTab);
    const newIndex = TABS.indexOf(newTab);
    setDirection(newIndex > oldIndex ? "right" : "left");
    setPrevTab(activeTab);
    setActiveTab(newTab);
    setAnimating(true);

    // Navigate to the default route for the new tab
    navigate(TAB_TO_ROUTE[newTab]);
  }, [activeTab, navigate]);

  useEffect(() => {
    updateHighlight();
    requestAnimationFrame(updateHighlight);
  }, [activeTab, i18n.language, updateHighlight]);

  useEffect(() => {
    window.addEventListener("resize", updateHighlight);
    return () => window.removeEventListener("resize", updateHighlight);
  }, [updateHighlight]);

  useEffect(() => {
    if (!animating) return;
    const timer = setTimeout(() => {
      setAnimating(false);
      setPrevTab(null);
    }, 250);
    return () => clearTimeout(timer);
  }, [animating]);

  const renderContent = (tab: TabId) => {
    switch (tab) {
      case "vertrekken": return <VertrekkenTab />;
      case "constructies": return <ConstructiesTab />;
      case "modeller": return <ModellerTab />;
      case "ifc": return <IfcTab />;
      case "resultaten": return <ResultatenTab />;
      case "rapport": return <RapportTab />;
    }
  };

  return (
    <div className="ribbon-container">
      <div className="ribbon-tabs" ref={tabsRef}>
        <RibbonTab label={t("tabs.file")} isFileTab onClick={() => onFileTabClick?.()} />
        {TABS.map((tab) => (
          <RibbonTab
            key={tab}
            label={t(`tabs.${tab}`)}
            isActive={activeTab === tab}
            onClick={() => switchTab(tab)}
          />
        ))}
        <div className="ribbon-tab-border" ref={borderRef} />
        <div className="ribbon-tab-gap" ref={gapRef} />
      </div>

      <div className="ribbon-content-wrapper">
        {animating && prevTab && (
          <div
            className={`ribbon-content-panel ribbon-panel-exit-${direction}`}
            key={`prev-${prevTab}`}
          >
            {renderContent(prevTab)}
          </div>
        )}
        <div
          className={`ribbon-content-panel${animating ? ` ribbon-panel-enter-${direction}` : ""}`}
          key={`active-${activeTab}`}
        >
          {renderContent(activeTab)}
        </div>
      </div>
    </div>
  );
}
