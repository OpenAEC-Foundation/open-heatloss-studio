import { useState } from "react";

import type { ModelRoom, ModelWindow, Point2D, Selection, WallBoundaryType } from "./types";
import { BOUNDARY_TYPE_LABELS } from "./types";
import { polygonArea, segmentsShareEdge, computeWallSegments } from "./geometry";
import { useAllConstructions, type UnifiedConstructionEntry } from "../../hooks/useAllConstructions";
import type { CatalogueCategory } from "../../lib/constructionCatalogue";
import { CATALOGUE_CATEGORY_LABELS } from "../../lib/constructionCatalogue";
import { formatArea } from "../../lib/formatNumber";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface PropertiesPanelProps {
  room: ModelRoom | null;
  rooms: ModelRoom[];
  windows: ModelWindow[];
  selection: Selection;
  onUpdateRoom?: (id: string, updates: Partial<Omit<ModelRoom, "id">>) => void;
  onRemoveRoom?: (id: string) => void;
  onUpdateWindow?: (roomId: string, wallIndex: number, offset: number, updates: Partial<ModelWindow>) => void;
  onRemoveWindow?: (roomId: string, wallIndex: number, offset: number) => void;
  wallConstructions?: Record<string, string>;
  floorConstructions?: Record<string, string>;
  roofConstructions?: Record<string, string>;
  onAssignWall?: (roomId: string, wallIndex: number, entryId: string | null) => void;
  onAssignFloor?: (roomId: string, entryId: string | null) => void;
  onAssignRoof?: (roomId: string, entryId: string | null) => void;
  wallBoundaryTypes?: Record<string, WallBoundaryType>;
  onAssignBoundaryType?: (roomId: string, wallIndex: number, boundaryType: WallBoundaryType) => void;
}

const FUNCTION_LABELS: Record<string, string> = {
  living_room: "Woonkamer",
  kitchen: "Keuken",
  bedroom: "Slaapkamer",
  bathroom: "Badkamer",
  toilet: "Toilet",
  hallway: "Hal / Gang",
  landing: "Overloop",
  storage: "Berging",
  attic: "Zolder",
  custom: "Overig",
};

const FUNCTION_OPTIONS = Object.entries(FUNCTION_LABELS);

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function PropertiesPanel({
  room,
  rooms,
  windows,
  selection,
  onUpdateRoom,
  onRemoveRoom,
  onUpdateWindow,
  onRemoveWindow,
  wallConstructions = {},
  floorConstructions = {},
  roofConstructions = {},
  onAssignWall,
  onAssignFloor,
  onAssignRoof,
  wallBoundaryTypes = {},
  onAssignBoundaryType,
}: PropertiesPanelProps) {
  const catalogueEntries = useAllConstructions();

  // Window selected: show window editor
  if (selection?.type === "window" && room) {
    const win = windows.find(
      (w) => w.roomId === selection.roomId && w.wallIndex === selection.wallIndex && Math.abs(w.offset - selection.offset) < 1,
    );
    if (win) {
      const poly = room.polygon;
      const a = poly[win.wallIndex]!;
      const b = poly[(win.wallIndex + 1) % poly.length]!;
      const wallLen = Math.sqrt((b.x - a.x) ** 2 + (b.y - a.y) ** 2);

      return (
        <div className="w-72 shrink-0 overflow-y-auto border-l border-[var(--oaec-border)] bg-surface-alt">
          <div className="border-b border-[var(--oaec-border-subtle)] px-4 py-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-bold text-on-surface">Raam</span>
              {onRemoveWindow && (
                <button
                  onClick={() => onRemoveWindow(win.roomId, win.wallIndex, win.offset)}
                  className="rounded px-1.5 py-0.5 text-[10px] text-red-500 hover:bg-red-50"
                >
                  Verwijderen
                </button>
              )}
            </div>
            <div className="mt-1 text-xs text-on-surface-muted">
              Wand {wallDirection(room.polygon, win.wallIndex)} van {room.name}
            </div>
          </div>
          <div className="space-y-2 px-4 py-3">
            <EditableNumberField
              label="Breedte (mm)"
              value={win.width}
              onChange={(val) => onUpdateWindow?.(win.roomId, win.wallIndex, win.offset, { width: val })}
            />
            <EditableNumberField
              label="Positie (mm)"
              value={Math.round(win.offset)}
              onChange={(val) => onUpdateWindow?.(win.roomId, win.wallIndex, win.offset, { offset: val })}
            />
            <Row label="Wand lengte" value={`${(wallLen / 1000).toFixed(2)} m`} />
          </div>
        </div>
      );
    }
  }

  // Wall selected: show wall segment details
  if (selection?.type === "wall" && room) {
    const segEdges = selection.segmentEdges ?? [selection.wallIndex];
    const poly = room.polygon;
    const n = poly.length;

    // Compute segment-level properties
    const segLength = segEdges.reduce((sum, ei) => {
      const a = poly[ei]!;
      const b = poly[(ei + 1) % n]!;
      return sum + Math.hypot(b.x - a.x, b.y - a.y);
    }, 0);

    // Direction from first to last point of segment
    const firstPt = poly[segEdges[0]!]!;
    const lastPt = poly[(segEdges[segEdges.length - 1]! + 1) % n]!;
    const cx = poly.reduce((s, p) => s + p.x, 0) / n;
    const cy = poly.reduce((s, p) => s + p.y, 0) / n;
    const mx = (firstPt.x + lastPt.x) / 2 - cx;
    const my = (firstPt.y + lastPt.y) / 2 - cy;
    const dir = Math.abs(mx) > Math.abs(my) ? (mx > 0 ? "Oost" : "West") : (my > 0 ? "Zuid" : "Noord");

    let autoType: "exterior" | "interior" = "exterior";
    let adjacentName: string | null = null;
    for (const ei of segEdges) {
      const a = poly[ei]!;
      const b = poly[(ei + 1) % n]!;
      for (const other of rooms) {
        if (other.id === room.id) continue;
        if (hasSharedWall(a, b, other.polygon)) { autoType = "interior"; adjacentName = other.name; break; }
      }
      if (autoType === "interior") break;
    }

    const currentBoundary = wallBoundaryTypes[`${room.id}:${segEdges[0]}`] ?? "auto";
    const wallWindows = windows.filter((w) => w.roomId === room.id && segEdges.includes(w.wallIndex));
    const assignedId = wallConstructions[`${room.id}:${segEdges[0]}`];
    const assigned = assignedId ? catalogueEntries.find((e) => e.id === assignedId) : null;

    return (
      <div className="w-72 shrink-0 overflow-y-auto border-l border-[var(--oaec-border)] bg-surface-alt">
        <div className="border-b border-[var(--oaec-border-subtle)] px-4 py-3">
          <span className="text-sm font-bold text-on-surface">Wand {dir}</span>
          <div className="mt-1 text-xs text-on-surface-muted">
            {room.id} {room.name}
            {segEdges.length > 1 && <span className="ml-1 text-on-surface-muted">({segEdges.length} edges)</span>}
          </div>
        </div>
        <div className="space-y-3 px-4 py-3">
          <Section title="Eigenschappen">
            <dl className="space-y-1 text-xs">
              <Row label="Richting" value={dir} />
              <Row label="Lengte" value={`${(segLength / 1000).toFixed(2)} m`} />
              <Row label="Hoogte" value={`${room.height} mm`} />
              <Row
                label="Oppervlak"
                value={`${formatArea((segLength / 1000) * (room.height / 1000))} m\u00B2`}
              />
              <div className="flex items-center justify-between text-xs">
                <span className="text-on-surface-muted">Grenstype</span>
                <select
                  value={currentBoundary}
                  onChange={(e) => {
                    const bt = e.target.value as WallBoundaryType;
                    for (const ei of segEdges) {
                      onAssignBoundaryType?.(room.id, ei, bt);
                    }
                  }}
                  className="rounded border border-[var(--oaec-border)] bg-surface-alt px-1.5 py-0.5 text-xs text-on-surface"
                >
                  {Object.entries(BOUNDARY_TYPE_LABELS).map(([key, label]) => (
                    <option key={key} value={key}>
                      {key === "auto" ? `${label} (${autoType === "exterior" ? "Gevel" : "Binnenwand"})` : label}
                    </option>
                  ))}
                </select>
              </div>
            </dl>
          </Section>

          <Section title="Aangrenzend">
            <dl className="space-y-1 text-xs">
              <Row label="Van" value={`${room.id} ${room.name}`} />
              <Row label="Naar" value={adjacentName ? adjacentName : "Buiten"} />
            </dl>
          </Section>

          {wallWindows.length > 0 && (
            <Section title={`Ramen (${wallWindows.length})`}>
              <div className="space-y-1">
                {wallWindows.map((w, i) => (
                  <div key={i} className="flex items-center justify-between rounded border border-[var(--oaec-border-subtle)] px-2 py-1 text-xs">
                    <span>Raam — {w.width} mm breed</span>
                    <span className="text-on-surface-muted">op {(w.offset / 1000).toFixed(2)} m</span>
                  </div>
                ))}
              </div>
            </Section>
          )}

          <Section title="Constructie">
            {assigned ? (
              <div className="rounded border border-green-200 bg-green-50 px-2 py-1.5 text-xs">
                <div className="flex items-center justify-between">
                  <span className="font-medium text-green-800">{assigned.name}</span>
                  <button onClick={() => { for (const ei of segEdges) onAssignWall?.(room.id, ei, null); }} className="text-red-400 hover:text-red-600">x</button>
                </div>
                <div className="mt-0.5 text-green-400">U = {assigned.uValue} W/(m²·K)</div>
              </div>
            ) : (
              <ConstructionPickerInline
                entries={catalogueEntries}
                filterCategory="wanden"
                onSelect={(entryId) => { for (const ei of segEdges) onAssignWall?.(room.id, ei, entryId); }}
              />
            )}
          </Section>
        </div>
      </div>
    );
  }

  // No room selected
  if (!room) {
    return (
      <div className="w-72 shrink-0 border-l border-[var(--oaec-border)] bg-surface-alt p-4">
        <p className="text-xs text-on-surface-muted">Selecteer een ruimte of wand om de eigenschappen te bekijken.</p>
        <div className="mt-6">
          <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">Ruimten ({rooms.length})</h3>
          <ul className="space-y-1">
            {rooms.map((r) => (
              <li key={r.id} className="text-xs text-on-surface-secondary">
                <span className="font-mono font-medium">{r.id}</span> {r.name} — {formatArea(polygonArea(r.polygon) / 1e6)} m²
              </li>
            ))}
          </ul>
        </div>
      </div>
    );
  }

  // Room selected
  const area = polygonArea(room.polygon) / 1e6;
  const roomWalls = getWallInfo(room, rooms, windows, wallBoundaryTypes);

  return (
    <div className="w-72 shrink-0 overflow-y-auto border-l border-[var(--oaec-border)] bg-surface-alt">
      {/* Header */}
      <div className="border-b border-[var(--oaec-border-subtle)] px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-baseline gap-2">
            <span className="font-mono text-sm font-bold text-on-surface">{room.id}</span>
            <span className="text-sm text-on-surface-secondary">{room.name}</span>
          </div>
          {onRemoveRoom && (
            <button onClick={() => onRemoveRoom(room.id)} className="rounded px-1.5 py-0.5 text-[10px] text-red-500 hover:bg-red-50" title="Verwijderen">
              Verwijderen
            </button>
          )}
        </div>
      </div>

      <div className="space-y-4 px-4 py-3">
        <Section title="Eigenschappen">
          <div className="space-y-2">
            <EditableField label="Naam" value={room.name} onChange={(val) => onUpdateRoom?.(room.id, { name: val })} />
            <div className="flex items-center justify-between text-xs">
              <span className="text-on-surface-muted">Functie</span>
              <select
                value={room.function}
                onChange={(e) => onUpdateRoom?.(room.id, { function: e.target.value })}
                className="rounded border border-[var(--oaec-border)] bg-surface-alt px-1.5 py-0.5 text-xs text-on-surface"
              >
                {FUNCTION_OPTIONS.map(([key, label]) => (
                  <option key={key} value={key}>{label}</option>
                ))}
              </select>
            </div>
            <Row label="Vloeroppervlak" value={`${formatArea(area)} m\u00B2`} />
            <EditableNumberField label="Hoogte (mm)" value={room.height} onChange={(val) => onUpdateRoom?.(room.id, { height: val })} />
            <ElevationField
              value={room.elevation}
              fallback={room.floor * (room.height + 300)}
              onChange={(val) => onUpdateRoom?.(room.id, { elevation: val })}
            />
            <EditableNumberField
              label="Temperatuur (°C)"
              value={room.temperature ?? 20}
              onChange={(val) => onUpdateRoom?.(room.id, { temperature: val })}
            />
            <Row label="Volume" value={`${((area * room.height) / 1e3).toFixed(1)} m\u00B3`} />
          </div>
        </Section>

        {/* Walls */}
        <Section title={`Wanden (${roomWalls.length})`}>
          <div className="space-y-1.5">
            {roomWalls.map((w) => (
              <WallCard
                key={w.wallIndex}
                wall={w}
                assignedEntryId={wallConstructions[`${room.id}:${w.wallIndex}`]}
                catalogueEntries={catalogueEntries}
                onAssign={(entryId) => {
                  for (const ei of w.edgeIndices) {
                    onAssignWall?.(room.id, ei, entryId);
                  }
                }}
                onChangeBoundary={(_wi, bt) => {
                  for (const ei of w.edgeIndices) {
                    onAssignBoundaryType?.(room.id, ei, bt);
                  }
                }}
              />
            ))}
          </div>
        </Section>

        <Section title="Vloer">
          <ConstructionCard
            label={`Vloer — ${formatArea(area)} m\u00B2`}
            badge="Grond" badgeColor="green"
            assignedEntryId={floorConstructions[room.id]}
            catalogueEntries={catalogueEntries}
            filterCategory="vloeren_plafonds"
            onAssign={(entryId) => onAssignFloor?.(room.id, entryId)}
          />
        </Section>

        <Section title="Plafond / Dak">
          <ConstructionCard
            label={`Plafond — ${formatArea(area)} m\u00B2`}
            badge="Verdieping" badgeColor="purple"
            assignedEntryId={roofConstructions[room.id]}
            catalogueEntries={catalogueEntries}
            filterCategory="daken"
            onAssign={(entryId) => onAssignRoof?.(room.id, entryId)}
          />
        </Section>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Generic sub-components
// ---------------------------------------------------------------------------

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h3 className="mb-1.5 text-xs font-semibold uppercase tracking-wider text-on-surface-muted">{title}</h3>
      {children}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between text-xs">
      <dt className="text-on-surface-muted">{label}</dt>
      <dd className="font-mono text-on-surface">{value}</dd>
    </div>
  );
}

function EditableField({ label, value, onChange }: { label: string; value: string; onChange?: (val: string) => void }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);
  if (!onChange) return <Row label={label} value={value} />;
  if (editing) {
    return (
      <div className="flex items-center justify-between text-xs">
        <span className="text-on-surface-muted">{label}</span>
        <input
          autoFocus value={draft} onChange={(e) => setDraft(e.target.value)}
          onBlur={() => { if (draft.trim()) onChange(draft.trim()); setEditing(false); }}
          onKeyDown={(e) => {
            if (e.key === "Enter") { if (draft.trim()) onChange(draft.trim()); setEditing(false); }
            if (e.key === "Escape") setEditing(false);
          }}
          className="w-28 rounded border border-amber-300 bg-amber-50 px-1.5 py-0.5 text-right text-xs text-on-surface outline-none"
        />
      </div>
    );
  }
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-on-surface-muted">{label}</span>
      <button onClick={() => { setDraft(value); setEditing(true); }} className="font-mono text-on-surface hover:text-amber-400 hover:underline">{value}</button>
    </div>
  );
}

function EditableNumberField({ label, value, onChange }: { label: string; value: number; onChange?: (val: number) => void }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(String(value));
  if (!onChange) return <Row label={label} value={String(value)} />;
  if (editing) {
    return (
      <div className="flex items-center justify-between text-xs">
        <span className="text-on-surface-muted">{label}</span>
        <input
          autoFocus type="number" value={draft} onChange={(e) => setDraft(e.target.value)}
          onBlur={() => { const n = parseInt(draft, 10); if (!isNaN(n) && n > 0) onChange(n); setEditing(false); }}
          onKeyDown={(e) => {
            if (e.key === "Enter") { const n = parseInt(draft, 10); if (!isNaN(n) && n > 0) onChange(n); setEditing(false); }
            if (e.key === "Escape") setEditing(false);
          }}
          className="w-20 rounded border border-amber-300 bg-amber-50 px-1.5 py-0.5 text-right text-xs text-on-surface outline-none"
        />
      </div>
    );
  }
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-on-surface-muted">{label}</span>
      <button onClick={() => { setDraft(String(value)); setEditing(true); }} className="font-mono text-on-surface hover:text-amber-400 hover:underline">{value}</button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Elevation (peil) field — allows negative/zero, shows fallback as placeholder
// ---------------------------------------------------------------------------

function ElevationField({ value, fallback, onChange }: {
  value: number | undefined;
  fallback: number;
  onChange?: (val: number | undefined) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value !== undefined ? String(value) : "");
  const display = value !== undefined ? String(value) : `${fallback} (auto)`;

  if (!onChange) return <Row label="Peil (mm)" value={display} />;

  if (editing) {
    return (
      <div className="flex items-center justify-between text-xs">
        <span className="text-on-surface-muted">Peil (mm)</span>
        <div className="flex items-center gap-1">
          <input
            autoFocus
            type="number"
            value={draft}
            placeholder={String(fallback)}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={() => {
              if (draft.trim() === "") { onChange(undefined); }
              else { const n = parseInt(draft, 10); if (!isNaN(n)) onChange(n); }
              setEditing(false);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                if (draft.trim() === "") { onChange(undefined); }
                else { const n = parseInt(draft, 10); if (!isNaN(n)) onChange(n); }
                setEditing(false);
              }
              if (e.key === "Escape") setEditing(false);
            }}
            className="w-20 rounded border border-amber-300 bg-amber-50 px-1.5 py-0.5 text-right text-xs text-on-surface outline-none"
          />
          {value !== undefined && (
            <button
              onClick={() => { onChange(undefined); setEditing(false); }}
              className="text-[10px] text-on-surface-muted hover:text-on-surface-secondary"
              title="Reset naar automatisch"
            >
              x
            </button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-on-surface-muted">Peil (mm)</span>
      <button
        onClick={() => { setDraft(value !== undefined ? String(value) : ""); setEditing(true); }}
        className={`font-mono hover:text-amber-400 hover:underline ${value !== undefined ? "text-on-surface" : "text-on-surface-muted"}`}
      >
        {display}
      </button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Wall card
// ---------------------------------------------------------------------------

interface WallInfo {
  direction: string;
  length: number;
  autoType: "exterior" | "interior";
  adjacentName: string | null;
  adjacentId: string | null;
  windowCount: number;
  /** Representative edge index (first edge in the segment). */
  wallIndex: number;
  /** All polygon edge indices that belong to this wall segment. */
  edgeIndices: number[];
  boundaryType: WallBoundaryType;
}

/** Describe what's on the other side of this wall based on effective boundary type. */
function wallTarget(wall: WallInfo): string {
  const bt = wall.boundaryType === "auto"
    ? (wall.autoType === "interior" ? "interior" : "exterior")
    : wall.boundaryType;
  if (bt === "interior" && wall.adjacentName) return wall.adjacentName;
  if (bt === "exterior") return "Buiten";
  if (bt === "curtain_wall") return "Buiten (vliesgevel)";
  if (bt === "neighbor") return "Buren";
  if (bt === "unheated") return "Onverwarmde ruimte";
  if (bt === "ground") return "Grond";
  if (wall.adjacentName) return wall.adjacentName;
  return "Buiten";
}

function boundaryBadge(wall: WallInfo): { label: string; colors: string } {
  const bt = wall.boundaryType;
  if (bt === "auto" || bt === "exterior") {
    if (bt === "auto" && wall.autoType === "interior") {
      return { label: wall.adjacentName ?? "Intern", colors: "bg-blue-600/15 text-blue-400" };
    }
    return { label: "Gevel", colors: "bg-red-50 text-red-400" };
  }
  if (bt === "interior") return { label: wall.adjacentName ?? "Intern", colors: "bg-blue-600/15 text-blue-400" };
  if (bt === "curtain_wall") return { label: "Vliesgevel", colors: "bg-cyan-50 text-cyan-700" };
  if (bt === "neighbor") return { label: "Buren", colors: "bg-orange-50 text-orange-700" };
  if (bt === "unheated") return { label: "Onverwarmd", colors: "bg-purple-50 text-purple-700" };
  if (bt === "ground") return { label: "Grond", colors: "bg-green-50 text-green-400" };
  return { label: "Gevel", colors: "bg-red-50 text-red-400" };
}

function WallCard({ wall, assignedEntryId, catalogueEntries, onAssign, onChangeBoundary }: {
  wall: WallInfo; assignedEntryId?: string; catalogueEntries: UnifiedConstructionEntry[];
  onAssign?: (entryId: string | null) => void;
  onChangeBoundary?: (wallIndex: number, bt: WallBoundaryType) => void;
}) {
  const [picking, setPicking] = useState(false);
  const assigned = assignedEntryId ? catalogueEntries.find((e) => e.id === assignedEntryId) : null;
  const badge = boundaryBadge(wall);

  return (
    <div className="rounded border border-[var(--oaec-border-subtle)] px-2 py-1.5 text-xs">
      <div className="flex items-center justify-between">
        <span className="font-medium text-on-surface-secondary">{wall.direction} — {(wall.length / 1000).toFixed(2)} m</span>
        <select
          value={wall.boundaryType}
          onChange={(e) => onChangeBoundary?.(wall.wallIndex, e.target.value as WallBoundaryType)}
          className={`rounded border-0 px-1.5 py-0.5 text-[10px] font-medium ${badge.colors} cursor-pointer`}
        >
          {Object.entries(BOUNDARY_TYPE_LABELS).map(([key, label]) => (
            <option key={key} value={key}>
              {key === "auto" ? `Auto (${wall.autoType === "exterior" ? "Gevel" : "Intern"})` : label}
            </option>
          ))}
        </select>
      </div>
      <div className="mt-0.5 text-on-surface-muted">→ {wallTarget(wall)}</div>
      {wall.windowCount > 0 && <div className="mt-0.5 text-blue-400">{wall.windowCount} kozijn{wall.windowCount > 1 ? "en" : ""}</div>}
      {assigned ? (
        <div className="mt-1 flex items-center justify-between">
          <span className="text-[10px] text-green-400">{assigned.name} (U={assigned.uValue})</span>
          <button onClick={() => onAssign?.(null)} className="text-[10px] text-red-400 hover:text-red-600">x</button>
        </div>
      ) : picking ? (
        <ConstructionPicker entries={catalogueEntries} filterCategory="wanden" onSelect={(id) => { onAssign?.(id); setPicking(false); }} onCancel={() => setPicking(false)} />
      ) : (
        <button onClick={() => setPicking(true)} className="mt-1 text-[10px] text-amber-600 hover:text-amber-400">Constructie toewijzen...</button>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Construction card + picker
// ---------------------------------------------------------------------------

function ConstructionCard({ label, badge, badgeColor, assignedEntryId, catalogueEntries, filterCategory, onAssign }: {
  label: string; badge: string; badgeColor: "green" | "purple"; assignedEntryId?: string; catalogueEntries: UnifiedConstructionEntry[];
  filterCategory: CatalogueCategory; onAssign?: (entryId: string | null) => void;
}) {
  const [picking, setPicking] = useState(false);
  const assigned = assignedEntryId ? catalogueEntries.find((e) => e.id === assignedEntryId) : null;
  const colors = badgeColor === "green" ? "bg-green-50 text-green-400" : "bg-purple-50 text-purple-700";

  return (
    <div className="rounded border border-[var(--oaec-border-subtle)] px-2 py-1.5 text-xs">
      <div className="flex items-center justify-between">
        <span className="font-medium text-on-surface-secondary">{label}</span>
        <span className={`rounded px-1.5 py-0.5 text-[10px] font-medium ${colors}`}>{badge}</span>
      </div>
      {assigned ? (
        <div className="mt-1 flex items-center justify-between">
          <span className="text-[10px] text-green-400">{assigned.name} (U={assigned.uValue})</span>
          <button onClick={() => onAssign?.(null)} className="text-[10px] text-red-400 hover:text-red-600">x</button>
        </div>
      ) : picking ? (
        <ConstructionPicker entries={catalogueEntries} filterCategory={filterCategory} onSelect={(id) => { onAssign?.(id); setPicking(false); }} onCancel={() => setPicking(false)} />
      ) : (
        <button onClick={() => setPicking(true)} className="mt-1 text-[10px] text-amber-600 hover:text-amber-400">Constructie toewijzen...</button>
      )}
    </div>
  );
}

function PickerEntryButton({ entry, onSelect }: {
  entry: UnifiedConstructionEntry; onSelect: (id: string) => void;
}) {
  return (
    <button
      onClick={() => onSelect(entry.id)}
      className="block w-full rounded px-1.5 py-1 text-left text-[10px] text-on-surface-secondary hover:bg-amber-600/15"
    >
      {entry.isProjectEntry && (
        <span className="mr-1 rounded bg-teal-50 px-1 py-0.5 text-[8px] font-medium text-teal-700">
          Project
        </span>
      )}
      <span className="font-medium">{entry.name}</span>{" "}
      <span className="text-on-surface-muted">U={entry.uValue}</span>
    </button>
  );
}

function PickerList({ entries, filterCategory, search, onSelect }: {
  entries: UnifiedConstructionEntry[];
  filterCategory: CatalogueCategory;
  search: string;
  onSelect: (id: string) => void;
}) {
  const filtered = entries.filter(
    (e) =>
      e.category === filterCategory &&
      (!search || e.name.toLowerCase().includes(search.toLowerCase())),
  );

  // Split into project and catalogue entries
  const projectEntries = filtered.filter((e) => e.isProjectEntry);
  const catalogueOnly = filtered.filter((e) => !e.isProjectEntry);

  if (filtered.length === 0) {
    return (
      <div className="py-1 text-center text-[10px] text-on-surface-muted">
        Geen resultaten
      </div>
    );
  }

  return (
    <>
      {projectEntries.length > 0 && (
        <>
          <div className="px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wider text-teal-600">
            Project
          </div>
          {projectEntries.map((e) => (
            <PickerEntryButton key={e.id} entry={e} onSelect={onSelect} />
          ))}
          {catalogueOnly.length > 0 && (
            <div className="my-0.5 border-t border-[var(--oaec-border)]" />
          )}
        </>
      )}
      {catalogueOnly.length > 0 && (
        <>
          {projectEntries.length > 0 && (
            <div className="px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wider text-on-surface-muted">
              Catalogus
            </div>
          )}
          {catalogueOnly.map((e) => (
            <PickerEntryButton key={e.id} entry={e} onSelect={onSelect} />
          ))}
        </>
      )}
    </>
  );
}

function ConstructionPickerInline({ entries, filterCategory, onSelect }: {
  entries: UnifiedConstructionEntry[]; filterCategory: CatalogueCategory; onSelect: (entryId: string) => void;
}) {
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);

  if (!open) {
    return <button onClick={() => setOpen(true)} className="text-[10px] text-amber-600 hover:text-amber-400">Constructie toewijzen...</button>;
  }

  return (
    <div className="rounded border border-amber-200 bg-amber-50/50 p-1.5">
      <input
        autoFocus placeholder={`Zoek in ${CATALOGUE_CATEGORY_LABELS[filterCategory]}...`}
        value={search} onChange={(e) => setSearch(e.target.value)}
        className="mb-1 w-full rounded border border-[var(--oaec-border)] bg-surface-alt px-1.5 py-0.5 text-[10px] outline-none focus:border-amber-400"
      />
      <div className="max-h-40 overflow-y-auto">
        <PickerList entries={entries} filterCategory={filterCategory} search={search} onSelect={onSelect} />
      </div>
    </div>
  );
}

function ConstructionPicker({ entries, filterCategory, onSelect, onCancel }: {
  entries: UnifiedConstructionEntry[]; filterCategory: CatalogueCategory; onSelect: (entryId: string) => void; onCancel: () => void;
}) {
  const [search, setSearch] = useState("");

  return (
    <div className="mt-1 rounded border border-amber-200 bg-amber-50/50 p-1.5">
      <div className="mb-1 flex items-center gap-1">
        <input
          autoFocus placeholder={`Zoek in ${CATALOGUE_CATEGORY_LABELS[filterCategory]}...`}
          value={search} onChange={(e) => setSearch(e.target.value)}
          className="flex-1 rounded border border-[var(--oaec-border)] bg-surface-alt px-1.5 py-0.5 text-[10px] outline-none focus:border-amber-400"
        />
        <button onClick={onCancel} className="text-[10px] text-on-surface-muted hover:text-on-surface-secondary">Annuleer</button>
      </div>
      <div className="max-h-40 overflow-y-auto">
        <PickerList entries={entries} filterCategory={filterCategory} search={search} onSelect={onSelect} />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Wall analysis
// ---------------------------------------------------------------------------

function getWallInfo(room: ModelRoom, allRooms: ModelRoom[], allWindows: ModelWindow[], boundaryTypes: Record<string, WallBoundaryType>): WallInfo[] {
  const poly = room.polygon;
  const n = poly.length;
  const roomWindows = allWindows.filter((w) => w.roomId === room.id);
  const segments = computeWallSegments(poly);
  const walls: WallInfo[] = [];

  for (const seg of segments) {
    // Auto type: if ANY edge in the segment is shared → interior
    let autoType: "exterior" | "interior" = "exterior";
    let adjacentName: string | null = null;
    let adjacentId: string | null = null;

    for (const ei of seg.edgeIndices) {
      const a = poly[ei]!;
      const b = poly[(ei + 1) % n]!;
      for (const other of allRooms) {
        if (other.id === room.id) continue;
        if (hasSharedWall(a, b, other.polygon)) {
          autoType = "interior";
          adjacentName = other.name;
          adjacentId = other.id;
          break;
        }
      }
      if (autoType === "interior") break;
    }

    // Window count: sum of windows on all edges in the segment
    const windowCount = seg.edgeIndices.reduce(
      (sum, ei) => sum + roomWindows.filter((w) => w.wallIndex === ei).length,
      0,
    );

    // Boundary type: use the first edge's override as representative
    const boundaryType = boundaryTypes[`${room.id}:${seg.edgeIndices[0]}`] ?? "auto";

    walls.push({
      direction: seg.direction,
      length: seg.length,
      autoType,
      adjacentName,
      adjacentId,
      windowCount,
      wallIndex: seg.edgeIndices[0]!,
      edgeIndices: seg.edgeIndices,
      boundaryType,
    });
  }
  return walls;
}

function wallDirection(polygon: Point2D[], edgeIndex: number): string {
  const n = polygon.length;
  const a = polygon[edgeIndex]!;
  const b = polygon[(edgeIndex + 1) % n]!;
  const cx = polygon.reduce((s, p) => s + p.x, 0) / n;
  const cy = polygon.reduce((s, p) => s + p.y, 0) / n;
  const nx = (a.x + b.x) / 2 - cx;
  const ny = (a.y + b.y) / 2 - cy;
  if (Math.abs(nx) > Math.abs(ny)) return nx > 0 ? "Oost" : "West";
  return ny > 0 ? "Zuid" : "Noord";
}

function hasSharedWall(a: Point2D, b: Point2D, polygon: Point2D[]): boolean {
  const n = polygon.length;
  for (let i = 0; i < n; i++) {
    if (segmentsShareEdge(a, b, polygon[i]!, polygon[(i + 1) % n]!)) return true;
  }
  return false;
}
