import { useState } from "react";

import type { LucideIcon } from "lucide-react";
import {
  MousePointer2, Hand, Square, Pentagon, Circle,
  Grid2x2, DoorOpen, Scissors, MoveHorizontal,
  Type, CornerRightDown, Ruler, Undo2, Redo2,
  FileImage, FileText, Download, Upload, FileDown, FileUp,
  Trash2,
} from "lucide-react";

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
  onImportJson: () => void;
  onExportJson: () => void;
  onClearView: () => void;
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
  icon: LucideIcon;
  shortcut?: string;
  large?: boolean;
}

const SELECT_TOOLS: ToolBtn[] = [
  { id: "select", label: "Selecteer", icon: MousePointer2, shortcut: "V", large: true },
  { id: "pan", label: "Verschuif", icon: Hand, shortcut: "H", large: true },
];

const SHAPE_TOOLS: ToolBtn[] = [
  { id: "draw_rect", label: "Rechthoek", icon: Square, shortcut: "R", large: true },
  { id: "draw_polygon", label: "Polygoon", icon: Pentagon, shortcut: "P", large: true },
  { id: "draw_circle", label: "Cirkel", icon: Circle, shortcut: "C", large: true },
];

const ELEMENT_TOOLS: ToolBtn[] = [
  { id: "draw_window", label: "Raam", icon: Grid2x2, shortcut: "N", large: true },
  { id: "draw_door", label: "Deur", icon: DoorOpen, large: true },
  { id: "split_room", label: "Splitsen", icon: Scissors, shortcut: "S", large: true },
];

const ANNOTATION_TOOLS: ToolBtn[] = [
  { id: "annotate_dimension", label: "Maatvoering", icon: MoveHorizontal, large: true },
  { id: "annotate_text", label: "Tekst", icon: Type, large: true },
  { id: "annotate_leader", label: "Leider", icon: CornerRightDown, large: true },
  { id: "measure", label: "Meten", icon: Ruler, shortcut: "M", large: true },
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

function ModelTab({ tool, onToolChange, onUndo, onRedo, onClearView }: RibbonProps) {
  return (
    <>
      {/* Undo / Redo / Clear */}
      <RibbonGroup label="">
        <div className="flex h-full flex-col items-center justify-center gap-1">
          <SmallButton icon={Undo2} label="Ongedaan" onClick={onUndo} title="Ctrl+Z" />
          <SmallButton icon={Redo2} label="Opnieuw" onClick={onRedo} title="Ctrl+Y" />
          <SmallButton icon={Trash2} label="Leeg" onClick={onClearView} title="Leeg beeld" />
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
  onImportJson,
  onExportJson,
}: RibbonProps) {
  return (
    <>
      <RibbonGroup label="Onderlegger">
        <div className="flex h-full items-center gap-1">
          <LargeButton icon={FileImage} label="DWG" onClick={onImportDwg} />
          <LargeButton icon={FileText} label="PDF" onClick={onImportPdf} />
        </div>
      </RibbonGroup>

      <RibbonGroup label="IFC">
        <div className="flex h-full items-center gap-1">
          <LargeButton icon={Download} label="Importeren" onClick={onImportIfc} />
          <LargeButton icon={Upload} label="Exporteren" onClick={onExportIfc} />
        </div>
      </RibbonGroup>

      <RibbonGroup label="Project">
        <div className="flex h-full items-center gap-1">
          <LargeButton icon={FileDown} label="Importeren" onClick={onImportJson} />
          <LargeButton icon={FileUp} label="Exporteren" onClick={onExportJson} />
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
      {large.map((t) => {
        const Icon = t.icon;
        return (
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
            <Icon size={18} />
            <span className="text-[9px] font-medium leading-tight">{t.label}</span>
          </button>
        );
      })}
      {small.length > 0 && (
        <div className="flex flex-col gap-px">
          {small.map((t) => {
            const Icon = t.icon;
            return (
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
                <Icon size={14} />
                <span className="text-[10px] font-medium">{t.label}</span>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

function LargeButton({
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
      className="flex h-[52px] w-14 flex-col items-center justify-center gap-0.5 rounded px-1 text-stone-500 transition-colors hover:bg-stone-50 hover:text-stone-700"
    >
      <Icon size={18} />
      <span className="text-[9px] font-medium leading-tight">{label}</span>
    </button>
  );
}

function SmallButton({
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
      className="flex items-center gap-1 rounded px-1.5 py-0.5 text-stone-500 transition-colors hover:bg-stone-50 hover:text-stone-700"
    >
      <Icon size={14} />
      <span className="text-[9px] font-medium">{label}</span>
    </button>
  );
}
