import { useState } from "react";

import type { ModellerTool, SnapMode, SnapSettings, ViewMode } from "./types";
import { FLOOR_LABELS } from "./exampleData";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface RibbonProps {
  tool: ModellerTool;
  viewMode: ViewMode;
  activeFloor: number;
  snap: SnapSettings;
  onToolChange: (tool: ModellerTool) => void;
  onViewModeChange: (mode: ViewMode) => void;
  onFloorChange: (floor: number) => void;
  onSnapChange: (snap: SnapSettings) => void;
  onFitView: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onImportDwg: () => void;
  onImportPdf: () => void;
  onImportIfc: () => void;
  onExportIfc: () => void;
}

// ---------------------------------------------------------------------------
// Tab definitions
// ---------------------------------------------------------------------------

type TabId = "model" | "annotatie" | "beeld" | "invoegen";

const TABS: { id: TabId; label: string }[] = [
  { id: "model", label: "Model" },
  { id: "annotatie", label: "Annotatie" },
  { id: "beeld", label: "Beeld" },
  { id: "invoegen", label: "Invoegen" },
];

// ---------------------------------------------------------------------------
// Tool button definitions
// ---------------------------------------------------------------------------

interface ToolBtn {
  id: ModellerTool;
  label: string;
  icon: string;
  shortcut?: string;
  large?: boolean;
}

const SELECT_TOOLS: ToolBtn[] = [
  { id: "select", label: "Selecteer", icon: "\u2190", shortcut: "V", large: true },
  { id: "pan", label: "Verschuif", icon: "\u2725", shortcut: "H", large: true },
];

const SHAPE_TOOLS: ToolBtn[] = [
  { id: "draw_rect", label: "Rechthoek", icon: "\u25AD", shortcut: "R", large: true },
  { id: "draw_polygon", label: "Polygoon", icon: "\u2B23", shortcut: "P", large: true },
  { id: "draw_circle", label: "Cirkel", icon: "\u25CB", shortcut: "C", large: true },
];

const ELEMENT_TOOLS: ToolBtn[] = [
  { id: "draw_window", label: "Raam", icon: "\u25A8", shortcut: "N", large: true },
  { id: "draw_door", label: "Deur", icon: "\u2395", large: true },
  { id: "split_room", label: "Splitsen", icon: "\u2702", shortcut: "S", large: true },
];

const ANNOTATION_TOOLS: ToolBtn[] = [
  { id: "annotate_dimension", label: "Maatvoering", icon: "\u2194", large: true },
  { id: "annotate_text", label: "Tekst", icon: "T", large: true },
  { id: "annotate_leader", label: "Leider", icon: "\u2198", large: true },
  { id: "measure", label: "Meten", icon: "\u21A6", shortcut: "M", large: true },
];

const SNAP_OPTIONS: { mode: SnapMode; label: string }[] = [
  { mode: "grid", label: "Raster" },
  { mode: "endpoint", label: "Eindpunt" },
  { mode: "midpoint", label: "Middelpunt" },
  { mode: "perpendicular", label: "Loodrecht" },
  { mode: "nearest", label: "Dichtstbij" },
  { mode: "underlay", label: "Onderlegger" },
];

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function Ribbon(props: RibbonProps) {
  const [activeTab, setActiveTab] = useState<TabId>("model");

  return (
    <div className="flex flex-col border-b border-stone-200 bg-white">
      {/* Tab strip */}
      <div className="flex items-center gap-px border-b border-stone-200 bg-stone-100 px-1">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`rounded-t px-4 py-1.5 text-xs font-medium transition-colors ${
              activeTab === tab.id
                ? "bg-white text-stone-800 shadow-sm"
                : "text-stone-500 hover:bg-stone-50 hover:text-stone-700"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Panel content */}
      <div className="flex h-[72px] items-stretch px-1">
        {activeTab === "model" && <ModelTab {...props} />}
        {activeTab === "annotatie" && <AnnotatieTab {...props} />}
        {activeTab === "beeld" && <BeeldTab {...props} />}
        {activeTab === "invoegen" && <InvoegenTab {...props} />}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Tab: Model
// ---------------------------------------------------------------------------

function ModelTab({ tool, onToolChange, onUndo, onRedo }: RibbonProps) {
  return (
    <>
      {/* Undo / Redo */}
      <RibbonGroup label="">
        <div className="flex h-full flex-col items-center justify-center gap-1">
          <SmallButton icon="&#x21B6;" label="Ongedaan" onClick={onUndo} title="Ctrl+Z" />
          <SmallButton icon="&#x21B7;" label="Opnieuw" onClick={onRedo} title="Ctrl+Y" />
        </div>
      </RibbonGroup>

      <RibbonGroup label="Selectie">
        <ToolRow tools={SELECT_TOOLS} active={tool} onSelect={onToolChange} />
      </RibbonGroup>

      <RibbonGroup label="Vormen">
        <ToolRow tools={SHAPE_TOOLS} active={tool} onSelect={onToolChange} />
      </RibbonGroup>

      <RibbonGroup label="Bouwelementen">
        <ToolRow tools={ELEMENT_TOOLS} active={tool} onSelect={onToolChange} />
      </RibbonGroup>

    </>
  );
}

// ---------------------------------------------------------------------------
// Tab: Annotatie
// ---------------------------------------------------------------------------

function AnnotatieTab({ tool, onToolChange }: RibbonProps) {
  return (
    <RibbonGroup label="Annotatie">
      <ToolRow tools={ANNOTATION_TOOLS} active={tool} onSelect={onToolChange} />
    </RibbonGroup>
  );
}

// ---------------------------------------------------------------------------
// Tab: Beeld
// ---------------------------------------------------------------------------

function BeeldTab({
  viewMode,
  activeFloor,
  snap,
  onViewModeChange,
  onFloorChange,
  onSnapChange,
  onFitView,
}: RibbonProps) {
  const toggleSnapMode = (mode: SnapMode) => {
    const modes = snap.modes.includes(mode)
      ? snap.modes.filter((m) => m !== mode)
      : [...snap.modes, mode];
    onSnapChange({ ...snap, modes });
  };

  return (
    <>
      <RibbonGroup label="Weergave">
        <div className="flex h-full items-center gap-2">
          <div className="flex overflow-hidden rounded border border-stone-200">
            <button
              onClick={() => onViewModeChange("2d")}
              className={`px-3 py-1.5 text-xs font-medium ${
                viewMode === "2d" ? "bg-stone-800 text-white" : "text-stone-500 hover:bg-stone-100"
              }`}
            >
              2D
            </button>
            <button
              onClick={() => onViewModeChange("3d")}
              className={`px-3 py-1.5 text-xs font-medium ${
                viewMode === "3d" ? "bg-stone-800 text-white" : "text-stone-500 hover:bg-stone-100"
              }`}
            >
              3D
            </button>
          </div>
          <div className="flex flex-col gap-1">
            <div className="flex items-center gap-1">
              <span className="text-[10px] text-stone-400">Verdieping:</span>
              <select
                value={activeFloor}
                onChange={(e) => onFloorChange(Number(e.target.value))}
                className="rounded border border-stone-200 bg-white px-1.5 py-0.5 text-[11px]"
              >
                {FLOOR_LABELS.map((label, i) => (
                  <option key={i} value={i}>{label}</option>
                ))}
              </select>
            </div>
            <button
              onClick={onFitView}
              className="rounded border border-stone-200 px-2 py-0.5 text-[10px] text-stone-500 hover:bg-stone-50"
            >
              Passend (F)
            </button>
          </div>
        </div>
      </RibbonGroup>

      <RibbonGroup label="Snap">
        <div className="flex h-full items-center gap-3">
          <button
            onClick={() => onSnapChange({ ...snap, enabled: !snap.enabled })}
            className={`rounded px-2 py-1 text-xs font-bold ${
              snap.enabled ? "bg-amber-100 text-amber-800" : "bg-stone-100 text-stone-400"
            }`}
          >
            {snap.enabled ? "AAN" : "UIT"}
          </button>
          <div className="grid grid-cols-3 gap-x-3 gap-y-0">
            {SNAP_OPTIONS.map((opt) => (
              <label
                key={opt.mode}
                className="flex cursor-pointer items-center gap-1 text-[10px] text-stone-600"
              >
                <input
                  type="checkbox"
                  checked={snap.modes.includes(opt.mode)}
                  onChange={() => toggleSnapMode(opt.mode)}
                  disabled={!snap.enabled}
                  className="h-2.5 w-2.5 rounded border-stone-300 text-amber-600"
                />
                {opt.label}
              </label>
            ))}
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[10px] text-stone-400">Raster:</span>
            <select
              value={snap.gridSize}
              onChange={(e) => onSnapChange({ ...snap, gridSize: Number(e.target.value) })}
              disabled={!snap.enabled}
              className="rounded border border-stone-200 bg-white px-1 py-0.5 text-[10px]"
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
      </RibbonGroup>
    </>
  );
}

// ---------------------------------------------------------------------------
// Tab: Invoegen
// ---------------------------------------------------------------------------

function InvoegenTab({
  onImportDwg,
  onImportPdf,
  onImportIfc,
  onExportIfc,
}: RibbonProps) {
  return (
    <>
      <RibbonGroup label="Onderlegger">
        <div className="flex h-full items-center gap-1">
          <LargeButton icon="\u2337" label="DWG" onClick={onImportDwg} />
          <LargeButton icon="\u2338" label="PDF" onClick={onImportPdf} />
        </div>
      </RibbonGroup>

      <RibbonGroup label="IFC">
        <div className="flex h-full items-center gap-1">
          <LargeButton icon="\u21E9" label="Importeren" onClick={onImportIfc} />
          <LargeButton icon="\u21E7" label="Exporteren" onClick={onExportIfc} />
        </div>
      </RibbonGroup>
    </>
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function RibbonGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col border-r border-stone-100 px-2 last:border-r-0">
      <div className="flex flex-1 items-center">{children}</div>
      {label && (
        <div className="pb-0.5 text-center text-[9px] font-medium uppercase tracking-wider text-stone-400">
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
  const large = tools.filter((t) => t.large);
  const small = tools.filter((t) => !t.large);

  return (
    <div className="flex h-full items-center gap-0.5">
      {large.map((t) => (
        <button
          key={t.id}
          onClick={() => onSelect(t.id)}
          title={t.shortcut ? `${t.label} (${t.shortcut})` : t.label}
          className={`flex h-[52px] w-11 flex-col items-center justify-center gap-0.5 rounded px-1 transition-colors ${
            active === t.id
              ? "bg-amber-100 text-amber-800"
              : "text-stone-500 hover:bg-stone-50 hover:text-stone-700"
          }`}
        >
          <span className="text-lg leading-none">{t.icon}</span>
          <span className="text-[9px] font-medium leading-tight">{t.label}</span>
        </button>
      ))}
      {small.length > 0 && (
        <div className="flex flex-col gap-px">
          {small.map((t) => (
            <button
              key={t.id}
              onClick={() => onSelect(t.id)}
              title={t.shortcut ? `${t.label} (${t.shortcut})` : t.label}
              className={`flex items-center gap-1.5 rounded px-1.5 py-0.5 text-left transition-colors ${
                active === t.id
                  ? "bg-amber-100 text-amber-800"
                  : "text-stone-500 hover:bg-stone-50 hover:text-stone-700"
              }`}
            >
              <span className="text-xs">{t.icon}</span>
              <span className="text-[10px] font-medium">{t.label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function LargeButton({
  icon,
  label,
  onClick,
}: {
  icon: string;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex h-[52px] w-14 flex-col items-center justify-center gap-0.5 rounded px-1 text-stone-500 transition-colors hover:bg-stone-50 hover:text-stone-700"
    >
      <span className="text-lg leading-none">{icon}</span>
      <span className="text-[9px] font-medium leading-tight">{label}</span>
    </button>
  );
}

function SmallButton({
  icon,
  label,
  onClick,
  title,
}: {
  icon: string;
  label: string;
  onClick: () => void;
  title?: string;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="flex items-center gap-1 rounded px-1.5 py-0.5 text-stone-500 transition-colors hover:bg-stone-50 hover:text-stone-700"
    >
      <span className="text-xs">{icon}</span>
      <span className="text-[9px] font-medium">{label}</span>
    </button>
  );
}
