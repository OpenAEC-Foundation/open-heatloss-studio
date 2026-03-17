import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { LucideIcon } from "lucide-react";
import {
  MousePointer2, Hand, Square, Pentagon, Circle,
  Grid2x2, DoorOpen, Scissors, MoveHorizontal,
  Type, CornerRightDown, Ruler, Undo2, Redo2,
  FileImage, FileText, Download, Upload, FileDown, FileUp,
  Trash2,
} from "lucide-react";

import { useModellerToolStore } from "../../store/modellerToolStore";
import { useModellerStore } from "../modeller/modellerStore";
import type { ModellerTool, SnapMode } from "../modeller/types";
import { FLOOR_LABELS } from "../modeller/exampleData";

// ---------------------------------------------------------------------------
// Sub-tab definitions
// ---------------------------------------------------------------------------

type SubTabId = "model" | "annotatie" | "beeld" | "invoegen";

const SUB_TABS: { id: SubTabId; labelKey: string }[] = [
  { id: "model", labelKey: "modeller.model" },
  { id: "annotatie", labelKey: "modeller.annotation" },
  { id: "beeld", labelKey: "modeller.view" },
  { id: "invoegen", labelKey: "modeller.insert" },
];

// ---------------------------------------------------------------------------
// Tool button definitions
// ---------------------------------------------------------------------------

interface ToolBtn {
  id: ModellerTool;
  labelKey: string;
  icon: LucideIcon;
  shortcut?: string;
}

const SELECT_TOOLS: ToolBtn[] = [
  { id: "select", labelKey: "modeller.select", icon: MousePointer2, shortcut: "V" },
  { id: "pan", labelKey: "modeller.pan", icon: Hand, shortcut: "H" },
];

const SHAPE_TOOLS: ToolBtn[] = [
  { id: "draw_rect", labelKey: "modeller.rectangle", icon: Square, shortcut: "R" },
  { id: "draw_polygon", labelKey: "modeller.polygon", icon: Pentagon, shortcut: "P" },
  { id: "draw_circle", labelKey: "modeller.circle", icon: Circle, shortcut: "C" },
];

const ELEMENT_TOOLS: ToolBtn[] = [
  { id: "draw_window", labelKey: "modeller.window", icon: Grid2x2, shortcut: "N" },
  { id: "draw_door", labelKey: "modeller.door", icon: DoorOpen },
  { id: "split_room", labelKey: "modeller.split", icon: Scissors, shortcut: "S" },
];

const ANNOTATION_TOOLS: ToolBtn[] = [
  { id: "annotate_dimension", labelKey: "modeller.dimension", icon: MoveHorizontal },
  { id: "annotate_text", labelKey: "modeller.text", icon: Type },
  { id: "annotate_leader", labelKey: "modeller.leader", icon: CornerRightDown },
  { id: "measure", labelKey: "modeller.measure", icon: Ruler, shortcut: "M" },
];

const SNAP_OPTIONS: { mode: SnapMode; labelKey: string }[] = [
  { mode: "grid", labelKey: "modeller.snapGrid" },
  { mode: "endpoint", labelKey: "modeller.snapEndpoint" },
  { mode: "midpoint", labelKey: "modeller.snapMidpoint" },
  { mode: "perpendicular", labelKey: "modeller.snapPerpendicular" },
  { mode: "nearest", labelKey: "modeller.snapNearest" },
  { mode: "underlay", labelKey: "modeller.snapUnderlay" },
];

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export default function ModellerTab() {
  const { t } = useTranslation("ribbon");
  const [activeSubTab, setActiveSubTab] = useState<SubTabId>("model");

  return (
    <div className="flex h-full flex-col">
      {/* Sub-tab strip */}
      <div className="flex items-center gap-px border-b px-1"
        style={{ borderColor: "var(--theme-ribbon-group-separator)" }}
      >
        {SUB_TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveSubTab(tab.id)}
            className="rounded-t px-3 py-0.5 text-[10px] font-medium transition-colors"
            style={{
              background: activeSubTab === tab.id ? "var(--theme-bg)" : "transparent",
              color: activeSubTab === tab.id
                ? "var(--theme-text)"
                : "var(--theme-ribbon-group-label)",
            }}
          >
            {t(tab.labelKey)}
          </button>
        ))}
      </div>

      {/* Sub-tab content */}
      <div className="flex flex-1 items-stretch px-1">
        {activeSubTab === "model" && <ModelSubTab />}
        {activeSubTab === "annotatie" && <AnnotatieSubTab />}
        {activeSubTab === "beeld" && <BeeldSubTab />}
        {activeSubTab === "invoegen" && <InvoegenSubTab />}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Sub-tab: Model
// ---------------------------------------------------------------------------

function ModelSubTab() {
  const { t } = useTranslation("ribbon");
  const tool = useModellerToolStore((s) => s.tool);
  const setTool = useModellerToolStore((s) => s.setTool);
  const undo = useModellerStore((s) => s.undo);
  const redo = useModellerStore((s) => s.redo);

  const dispatch = (eventName: string) => () => {
    window.dispatchEvent(new CustomEvent(eventName));
  };

  return (
    <>
      <Group label="">
        <div className="flex h-full flex-col items-center justify-center gap-1">
          <SmallBtn icon={Undo2} label={t("modeller.undo")} onClick={undo} title="Ctrl+Z" />
          <SmallBtn icon={Redo2} label={t("modeller.redo")} onClick={redo} title="Ctrl+Y" />
          <SmallBtn icon={Trash2} label={t("modeller.clear")} onClick={dispatch("modeller:clear-view")} />
        </div>
      </Group>
      <Group label={t("modeller.selection")}>
        <ToolRow tools={SELECT_TOOLS} active={tool} onSelect={setTool} />
      </Group>
      <Group label={t("modeller.shapes")}>
        <ToolRow tools={SHAPE_TOOLS} active={tool} onSelect={setTool} />
      </Group>
      <Group label={t("modeller.elements")}>
        <ToolRow tools={ELEMENT_TOOLS} active={tool} onSelect={setTool} />
      </Group>
    </>
  );
}

// ---------------------------------------------------------------------------
// Sub-tab: Annotatie
// ---------------------------------------------------------------------------

function AnnotatieSubTab() {
  const { t } = useTranslation("ribbon");
  const tool = useModellerToolStore((s) => s.tool);
  const setTool = useModellerToolStore((s) => s.setTool);

  return (
    <Group label={t("modeller.annotation")}>
      <ToolRow tools={ANNOTATION_TOOLS} active={tool} onSelect={setTool} />
    </Group>
  );
}

// ---------------------------------------------------------------------------
// Sub-tab: Beeld
// ---------------------------------------------------------------------------

function BeeldSubTab() {
  const { t } = useTranslation("ribbon");
  const viewMode = useModellerToolStore((s) => s.viewMode);
  const setViewMode = useModellerToolStore((s) => s.setViewMode);
  const activeFloor = useModellerToolStore((s) => s.activeFloor);
  const setActiveFloor = useModellerToolStore((s) => s.setActiveFloor);
  const snap = useModellerToolStore((s) => s.snap);
  const setSnap = useModellerToolStore((s) => s.setSnap);
  const toggleSnapEnabled = useModellerToolStore((s) => s.toggleSnapEnabled);
  const toggleSnapMode = useModellerToolStore((s) => s.toggleSnapMode);

  return (
    <>
      <Group label={t("modeller.display")}>
        <div className="flex h-full items-center gap-2">
          <div className="flex overflow-hidden rounded border" style={{ borderColor: "var(--theme-ribbon-group-separator)" }}>
            <button
              onClick={() => setViewMode("2d")}
              className="px-3 py-1 text-[10px] font-medium"
              style={{
                background: viewMode === "2d" ? "var(--theme-accent)" : "transparent",
                color: viewMode === "2d" ? "#fff" : "var(--theme-text)",
              }}
            >
              2D
            </button>
            <button
              onClick={() => setViewMode("3d")}
              className="px-3 py-1 text-[10px] font-medium"
              style={{
                background: viewMode === "3d" ? "var(--theme-accent)" : "transparent",
                color: viewMode === "3d" ? "#fff" : "var(--theme-text)",
              }}
            >
              3D
            </button>
          </div>
          <div className="flex flex-col gap-0.5">
            <div className="flex items-center gap-1">
              <span className="text-[9px]" style={{ color: "var(--theme-ribbon-group-label)" }}>
                {t("modeller.floor")}:
              </span>
              <select
                value={activeFloor}
                onChange={(e) => setActiveFloor(Number(e.target.value))}
                className="rounded border px-1 py-0.5 text-[10px]"
                style={{
                  borderColor: "var(--theme-ribbon-group-separator)",
                  background: "var(--theme-bg)",
                  color: "var(--theme-text)",
                }}
              >
                {FLOOR_LABELS.map((label, i) => (
                  <option key={i} value={i}>{label}</option>
                ))}
              </select>
            </div>
          </div>
        </div>
      </Group>

      <Group label={t("modeller.snap")}>
        <div className="flex h-full items-center gap-3">
          <button
            onClick={toggleSnapEnabled}
            className="rounded px-2 py-1 text-[10px] font-bold"
            style={{
              background: snap.enabled ? "var(--theme-ribbon-btn-active-bg)" : "var(--theme-bg-lighter)",
              color: snap.enabled ? "var(--theme-ribbon-btn-active-text)" : "var(--theme-ribbon-group-label)",
            }}
          >
            {snap.enabled ? t("modeller.snapOn") : t("modeller.snapOff")}
          </button>
          <div className="grid grid-cols-3 gap-x-3 gap-y-0">
            {SNAP_OPTIONS.map((opt) => (
              <label
                key={opt.mode}
                className="flex cursor-pointer items-center gap-1 text-[9px]"
                style={{ color: "var(--theme-text)" }}
              >
                <input
                  type="checkbox"
                  checked={snap.modes.includes(opt.mode)}
                  onChange={() => toggleSnapMode(opt.mode)}
                  disabled={!snap.enabled}
                  className="h-2.5 w-2.5"
                />
                {t(opt.labelKey)}
              </label>
            ))}
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[9px]" style={{ color: "var(--theme-ribbon-group-label)" }}>
              {t("modeller.gridSize")}:
            </span>
            <select
              value={snap.gridSize}
              onChange={(e) => setSnap({ ...snap, gridSize: Number(e.target.value) })}
              disabled={!snap.enabled}
              className="rounded border px-1 py-0.5 text-[9px]"
              style={{
                borderColor: "var(--theme-ribbon-group-separator)",
                background: "var(--theme-bg)",
                color: "var(--theme-text)",
              }}
            >
              <option value={10}>10mm</option>
              <option value={50}>50mm</option>
              <option value={100}>100mm</option>
              <option value={250}>250mm</option>
              <option value={500}>500mm</option>
              <option value={1000}>1m</option>
            </select>
          </div>
        </div>
      </Group>
    </>
  );
}

// ---------------------------------------------------------------------------
// Sub-tab: Invoegen
// ---------------------------------------------------------------------------

function InvoegenSubTab() {
  const { t } = useTranslation("ribbon");

  // These callbacks are handled by the Modeller page via event dispatch
  const dispatch = (eventName: string) => () => {
    window.dispatchEvent(new CustomEvent(eventName));
  };

  return (
    <>
      <Group label={t("modeller.underlay")}>
        <div className="flex h-full items-center gap-1">
          <LargeBtn icon={FileImage} label="DWG" onClick={dispatch("modeller:import-dwg")} />
          <LargeBtn icon={FileText} label="PDF" onClick={dispatch("modeller:import-pdf")} />
        </div>
      </Group>
      <Group label="IFC">
        <div className="flex h-full items-center gap-1">
          <LargeBtn icon={Download} label={t("modeller.import")} onClick={dispatch("modeller:import-ifc")} />
          <LargeBtn icon={Upload} label={t("modeller.export")} onClick={dispatch("modeller:export-ifc")} />
        </div>
      </Group>
      <Group label={t("modeller.project")}>
        <div className="flex h-full items-center gap-1">
          <LargeBtn icon={FileDown} label={t("modeller.import")} onClick={dispatch("modeller:import-json")} />
          <LargeBtn icon={FileUp} label={t("modeller.export")} onClick={dispatch("modeller:export-json")} />
        </div>
      </Group>
    </>
  );
}

// ---------------------------------------------------------------------------
// Shared sub-components (Tailwind-based, matching modeller ribbon style)
// ---------------------------------------------------------------------------

function Group({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div
      className="flex flex-col border-r px-2 last:border-r-0"
      style={{ borderColor: "var(--theme-ribbon-group-separator)" }}
    >
      <div className="flex flex-1 items-center">{children}</div>
      {label && (
        <div
          className="pb-0.5 text-center text-[8px] font-medium uppercase tracking-wider"
          style={{ color: "var(--theme-ribbon-group-label)" }}
        >
          {label}
        </div>
      )}
    </div>
  );
}

function ToolRow({
  tools,
  active,
  onSelect,
}: {
  tools: ToolBtn[];
  active: ModellerTool;
  onSelect: (tool: ModellerTool) => void;
}) {
  const { t } = useTranslation("ribbon");

  return (
    <div className="flex h-full items-center gap-0.5">
      {tools.map((tool) => {
        const Icon = tool.icon;
        const isActive = active === tool.id;
        return (
          <button
            key={tool.id}
            onClick={() => onSelect(tool.id)}
            title={tool.shortcut ? `${t(tool.labelKey)} (${tool.shortcut})` : t(tool.labelKey)}
            className="flex h-[48px] w-11 flex-col items-center justify-center gap-0.5 rounded px-1 transition-colors"
            style={{
              background: isActive ? "var(--theme-ribbon-btn-active-bg)" : "transparent",
              color: isActive ? "var(--theme-ribbon-btn-active-text)" : "var(--theme-text)",
            }}
          >
            <Icon size={16} />
            <span className="text-[8px] font-medium leading-tight">{t(tool.labelKey)}</span>
          </button>
        );
      })}
    </div>
  );
}

function LargeBtn({
  icon: Icon,
  label,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex h-[48px] w-14 flex-col items-center justify-center gap-0.5 rounded px-1 transition-colors"
      style={{ color: "var(--theme-text)" }}
    >
      <Icon size={16} />
      <span className="text-[8px] font-medium leading-tight">{label}</span>
    </button>
  );
}

function SmallBtn({
  icon: Icon,
  label,
  onClick,
  title,
}: {
  icon: LucideIcon;
  label: string;
  onClick: () => void;
  title?: string;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="flex items-center gap-1 rounded px-1.5 py-0.5 transition-colors"
      style={{ color: "var(--theme-text)" }}
    >
      <Icon size={12} />
      <span className="text-[8px] font-medium">{label}</span>
    </button>
  );
}
