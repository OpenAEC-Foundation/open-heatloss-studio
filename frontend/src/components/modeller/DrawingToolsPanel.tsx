import type { ModellerTool, SnapMode, SnapSettings } from "./types";

interface DrawingToolsPanelProps {
  tool: ModellerTool;
  snap: SnapSettings;
  onToolChange: (tool: ModellerTool) => void;
  onSnapChange: (snap: SnapSettings) => void;
  onImportDwg: () => void;
  onImportPdf: () => void;
  onImportIfc: () => void;
  onExportIfc: () => void;
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

interface ToolDef {
  id: ModellerTool;
  label: string;
  icon: string;
  shortcut?: string;
}

const SELECTION_TOOLS: ToolDef[] = [
  { id: "select", label: "Selecteer", icon: "\u2190", shortcut: "V" },
  { id: "pan", label: "Verschuif", icon: "\u2725", shortcut: "H" },
];

const SHAPE_TOOLS: ToolDef[] = [
  { id: "draw_rect", label: "Rechthoek", icon: "\u25AD", shortcut: "R" },
  { id: "draw_polygon", label: "Polygoon", icon: "\u2B23", shortcut: "P" },
  { id: "draw_circle", label: "Cirkel", icon: "\u25CB", shortcut: "C" },
];

const ELEMENT_TOOLS: ToolDef[] = [
  { id: "draw_window", label: "Raam", icon: "\u25A8", shortcut: "N" },
  { id: "draw_door", label: "Deur", icon: "\u2395" },
  { id: "split_room", label: "Splitsen", icon: "\u2702", shortcut: "S" },
];

const ANNOTATION_TOOLS: ToolDef[] = [
  { id: "annotate_text", label: "Tekst", icon: "T" },
  { id: "annotate_dimension", label: "Maatvoering", icon: "\u2194" },
  { id: "annotate_leader", label: "Leider", icon: "\u2198" },
  { id: "measure", label: "Meten", icon: "\u21A6", shortcut: "M" },
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

export function DrawingToolsPanel({
  tool,
  snap,
  onToolChange,
  onSnapChange,
  onImportDwg,
  onImportPdf,
  onImportIfc,
  onExportIfc,
}: DrawingToolsPanelProps) {
  const toggleSnapMode = (mode: SnapMode) => {
    const modes = snap.modes.includes(mode)
      ? snap.modes.filter((m) => m !== mode)
      : [...snap.modes, mode];
    onSnapChange({ ...snap, modes });
  };

  return (
    <div className="flex w-48 shrink-0 flex-col border-r border-stone-200 bg-white">
      <div className="flex-1 overflow-y-auto px-2 py-2">
        <ToolSection title="Selectie" tools={SELECTION_TOOLS} active={tool} onSelect={onToolChange} />
        <ToolSection title="Vormen" tools={SHAPE_TOOLS} active={tool} onSelect={onToolChange} />
        <ToolSection title="Bouwelementen" tools={ELEMENT_TOOLS} active={tool} onSelect={onToolChange} />
        <ToolSection title="Annotatie" tools={ANNOTATION_TOOLS} active={tool} onSelect={onToolChange} />

        {/* Snap settings */}
        <div className="mt-3 border-t border-stone-100 pt-2">
          <div className="mb-1.5 flex items-center justify-between">
            <h4 className="text-[10px] font-semibold uppercase tracking-wider text-stone-400">
              Snap
            </h4>
            <button
              onClick={() => onSnapChange({ ...snap, enabled: !snap.enabled })}
              className={`rounded px-1.5 py-0.5 text-[10px] font-medium transition-colors ${
                snap.enabled
                  ? "bg-amber-100 text-amber-800"
                  : "bg-stone-100 text-stone-400"
              }`}
            >
              {snap.enabled ? "AAN" : "UIT"}
            </button>
          </div>
          <div className="space-y-0.5">
            {SNAP_OPTIONS.map((opt) => (
              <label
                key={opt.mode}
                className="flex cursor-pointer items-center gap-1.5 rounded px-1.5 py-0.5 text-[11px] text-stone-600 hover:bg-stone-50"
              >
                <input
                  type="checkbox"
                  checked={snap.modes.includes(opt.mode)}
                  onChange={() => toggleSnapMode(opt.mode)}
                  disabled={!snap.enabled}
                  className="h-3 w-3 rounded border-stone-300 text-amber-600 focus:ring-amber-500"
                />
                {opt.label}
              </label>
            ))}
          </div>
          <div className="mt-1.5 flex items-center gap-1">
            <span className="text-[10px] text-stone-400">Raster:</span>
            <select
              value={snap.gridSize}
              onChange={(e) => onSnapChange({ ...snap, gridSize: Number(e.target.value) })}
              disabled={!snap.enabled}
              className="flex-1 rounded border border-stone-200 bg-white px-1 py-0.5 text-[10px]"
            >
              <option value={10}>10 mm</option>
              <option value={50}>50 mm</option>
              <option value={100}>100 mm</option>
              <option value={250}>250 mm</option>
              <option value={500}>500 mm</option>
              <option value={1000}>1000 mm</option>
            </select>
          </div>
        </div>

        {/* Import / Export */}
        <div className="mt-3 border-t border-stone-100 pt-2">
          <h4 className="mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-stone-400">
            Import / Export
          </h4>
          <div className="grid grid-cols-2 gap-1">
            <ActionButton label="DWG" onClick={onImportDwg} />
            <ActionButton label="PDF" onClick={onImportPdf} />
            <ActionButton label="IFC import" onClick={onImportIfc} />
            <ActionButton label="IFC export" onClick={onExportIfc} />
          </div>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function ToolSection({
  title,
  tools,
  active,
  onSelect,
}: {
  title: string;
  tools: ToolDef[];
  active: ModellerTool;
  onSelect: (tool: ModellerTool) => void;
}) {
  return (
    <div className="mb-2">
      <h4 className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-stone-400">
        {title}
      </h4>
      <div className="space-y-px">
        {tools.map((t) => (
          <button
            key={t.id}
            onClick={() => onSelect(t.id)}
            title={t.shortcut ? `${t.label} (${t.shortcut})` : t.label}
            className={`flex w-full items-center gap-2 rounded px-2 py-1 text-left text-[11px] transition-colors ${
              active === t.id
                ? "bg-amber-100 font-medium text-amber-800"
                : "text-stone-600 hover:bg-stone-50 hover:text-stone-800"
            }`}
          >
            <span className="w-4 text-center text-xs">{t.icon}</span>
            <span className="flex-1">{t.label}</span>
            {t.shortcut && (
              <kbd className="rounded bg-stone-100 px-1 py-px text-[9px] text-stone-400">
                {t.shortcut}
              </kbd>
            )}
          </button>
        ))}
      </div>
    </div>
  );
}

function ActionButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="rounded border border-stone-200 px-1.5 py-1 text-[10px] text-stone-500 transition-colors hover:bg-stone-50 hover:text-stone-700"
    >
      {label}
    </button>
  );
}
