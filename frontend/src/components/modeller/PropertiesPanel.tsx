import { useState } from "react";

import type { ModelRoom, ModelWall, ModelWindow, ModellerTool, Point2D, Selection, WallAlignment } from "./types";
import { polygonArea, segmentsShareEdge } from "./geometry";
import { useCatalogueStore } from "../../store/catalogueStore";
import type { CatalogueEntry, CatalogueCategory } from "../../lib/constructionCatalogue";
import { CATALOGUE_CATEGORY_LABELS } from "../../lib/constructionCatalogue";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface PropertiesPanelProps {
  room: ModelRoom | null;
  rooms: ModelRoom[];
  windows: ModelWindow[];
  walls?: ModelWall[];
  selection: Selection;
  onUpdateRoom?: (id: string, updates: Partial<Omit<ModelRoom, "id">>) => void;
  onRemoveRoom?: (id: string) => void;
  onUpdateWall?: (id: string, updates: Partial<Omit<ModelWall, "id">>) => void;
  onRemoveWall?: (id: string) => void;
  onUpdateWindow?: (roomId: string, wallIndex: number, offset: number, updates: Partial<ModelWindow>) => void;
  onRemoveWindow?: (roomId: string, wallIndex: number, offset: number) => void;
  wallConstructions?: Record<string, string>;
  floorConstructions?: Record<string, string>;
  roofConstructions?: Record<string, string>;
  standaloneWallConstructions?: Record<string, string>;
  onAssignWall?: (roomId: string, wallIndex: number, entryId: string | null) => void;
  onAssignFloor?: (roomId: string, entryId: string | null) => void;
  onAssignRoof?: (roomId: string, entryId: string | null) => void;
  onAssignStandaloneWall?: (wallId: string, entryId: string | null) => void;
  tool?: ModellerTool;
  wallAlignment?: WallAlignment;
  onWallAlignmentChange?: (alignment: WallAlignment) => void;
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
  walls = [],
  selection,
  onUpdateRoom,
  onRemoveRoom,
  onUpdateWall,
  onRemoveWall,
  onUpdateWindow,
  onRemoveWindow,
  wallConstructions = {},
  floorConstructions = {},
  roofConstructions = {},
  standaloneWallConstructions = {},
  onAssignWall,
  onAssignFloor,
  onAssignRoof,
  onAssignStandaloneWall,
  tool,
  wallAlignment = "exterior",
  onWallAlignmentChange,
}: PropertiesPanelProps) {
  const catalogueEntries = useCatalogueStore((s) => s.entries);

  // Standalone wall selected: show wall type selector
  if (selection?.type === "standalone_wall") {
    const wall = walls.find((w) => w.id === selection.wallId);
    if (wall && wall.points.length >= 2) {
      // Total wall length
      let totalLength = 0;
      for (let i = 0; i < wall.points.length - 1; i++) {
        const p = wall.points[i]!;
        const q = wall.points[i + 1]!;
        totalLength += Math.hypot(q.x - p.x, q.y - p.y);
      }
      const assignedId = standaloneWallConstructions[wall.id];
      const assigned = assignedId ? catalogueEntries.find((e) => e.id === assignedId) : null;

      return (
        <div className="w-72 shrink-0 overflow-y-auto border-l border-stone-200 bg-white">
          <div className="border-b border-stone-100 px-4 py-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-bold text-stone-800">Wand</span>
              {onRemoveWall && (
                <button
                  onClick={() => onRemoveWall(wall.id)}
                  className="rounded px-1.5 py-0.5 text-[10px] text-red-500 hover:bg-red-50"
                >
                  Verwijderen
                </button>
              )}
            </div>
            <div className="mt-1 text-xs text-stone-500">
              {wall.points.length - 1} segment{wall.points.length > 2 ? "en" : ""}
            </div>
          </div>
          <div className="space-y-3 px-4 py-3">
            <Section title="Eigenschappen">
              <dl className="space-y-1 text-xs">
                <Row label="Lengte" value={`${(totalLength / 1000).toFixed(2)} m`} />
                <Row label="Segmenten" value={String(wall.points.length - 1)} />
                <Row label="Dikte" value="200 mm" />
                <div className="flex items-center justify-between text-xs">
                  <span className="text-stone-500">Plaatsingspunt</span>
                  <select
                    value={wall.alignment}
                    onChange={(e) => onUpdateWall?.(wall.id, { alignment: e.target.value as WallAlignment })}
                    className="rounded border border-stone-200 bg-white px-1.5 py-0.5 text-xs text-stone-800"
                  >
                    <option value="exterior">Buitenkant</option>
                    <option value="center">Hart</option>
                    <option value="interior">Binnenkant</option>
                  </select>
                </div>
              </dl>
            </Section>

            <Section title="Wandtype">
              {assigned ? (
                <div className="rounded border border-green-200 bg-green-50 px-2 py-1.5 text-xs">
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-green-800">{assigned.name}</span>
                    <button onClick={() => onAssignStandaloneWall?.(wall.id, null)} className="text-red-400 hover:text-red-600">x</button>
                  </div>
                  <div className="mt-0.5 text-green-600">U = {assigned.uValue} W/(m²·K)</div>
                  {assigned.layers && assigned.layers.length > 0 && (
                    <div className="mt-2 space-y-0.5">
                      {assigned.layers.map((layer, i) => (
                        <div key={i} className="flex items-center justify-between text-[10px] text-green-700">
                          <span>{layer.materialId}</span>
                          <span>{layer.thickness} mm</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ) : (
                <ConstructionPickerInline
                  entries={catalogueEntries}
                  filterCategory="wanden"
                  onSelect={(entryId) => onAssignStandaloneWall?.(wall.id, entryId)}
                />
              )}
            </Section>
          </div>
        </div>
      );
    }
  }

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
        <div className="w-72 shrink-0 overflow-y-auto border-l border-stone-200 bg-white">
          <div className="border-b border-stone-100 px-4 py-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-bold text-stone-800">Raam</span>
              {onRemoveWindow && (
                <button
                  onClick={() => onRemoveWindow(win.roomId, win.wallIndex, win.offset)}
                  className="rounded px-1.5 py-0.5 text-[10px] text-red-500 hover:bg-red-50"
                >
                  Verwijderen
                </button>
              )}
            </div>
            <div className="mt-1 text-xs text-stone-500">
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

  // Wall selected: show wall details
  if (selection?.type === "wall" && room) {
    const wi = selection.wallIndex;
    const poly = room.polygon;
    const a = poly[wi]!;
    const b = poly[(wi + 1) % poly.length]!;
    const length = Math.sqrt((b.x - a.x) ** 2 + (b.y - a.y) ** 2);
    const dir = wallDirection(poly, wi);

    let wallType: "exterior" | "interior" = "exterior";
    let adjacentName: string | null = null;
    for (const other of rooms) {
      if (other.id === room.id) continue;
      if (hasSharedWall(a, b, other.polygon)) { wallType = "interior"; adjacentName = other.name; break; }
    }

    const wallWindows = windows.filter((w) => w.roomId === room.id && w.wallIndex === wi);
    const assignedId = wallConstructions[`${room.id}:${wi}`];
    const assigned = assignedId ? catalogueEntries.find((e) => e.id === assignedId) : null;

    return (
      <div className="w-72 shrink-0 overflow-y-auto border-l border-stone-200 bg-white">
        <div className="border-b border-stone-100 px-4 py-3">
          <span className="text-sm font-bold text-stone-800">Wand {dir}</span>
          <div className="mt-1 text-xs text-stone-500">{room.id} {room.name}</div>
        </div>
        <div className="space-y-3 px-4 py-3">
          <Section title="Eigenschappen">
            <dl className="space-y-1 text-xs">
              <Row label="Richting" value={dir} />
              <Row label="Lengte" value={`${(length / 1000).toFixed(2)} m`} />
              <Row label="Hoogte" value={`${room.height} mm`} />
              <Row label="Oppervlak" value={`${((length / 1000) * (room.height / 1000)).toFixed(2)} m\u00B2`} />
              <Row
                label="Type"
                value={wallType === "exterior" ? "Gevel (buitenwand)" : `Intern (${adjacentName})`}
              />
            </dl>
          </Section>

          {wallWindows.length > 0 && (
            <Section title={`Ramen (${wallWindows.length})`}>
              <div className="space-y-1">
                {wallWindows.map((w, i) => (
                  <div key={i} className="flex items-center justify-between rounded border border-stone-100 px-2 py-1 text-xs">
                    <span>Raam — {w.width} mm breed</span>
                    <span className="text-stone-400">op {(w.offset / 1000).toFixed(2)} m</span>
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
                  <button onClick={() => onAssignWall?.(room.id, wi, null)} className="text-red-400 hover:text-red-600">x</button>
                </div>
                <div className="mt-0.5 text-green-600">U = {assigned.uValue} W/(m²·K)</div>
              </div>
            ) : (
              <ConstructionPickerInline
                entries={catalogueEntries}
                filterCategory="wanden"
                onSelect={(entryId) => onAssignWall?.(room.id, wi, entryId)}
              />
            )}
          </Section>
        </div>
      </div>
    );
  }

  // No room selected
  if (!room) {
    // Wall drawing tool active: show wall settings
    if (tool === "draw_wall") {
      return (
        <div className="w-72 shrink-0 overflow-y-auto border-l border-stone-200 bg-white">
          <div className="border-b border-stone-100 px-4 py-3">
            <span className="text-sm font-bold text-stone-800">Wand tekenen</span>
          </div>
          <div className="space-y-3 px-4 py-3">
            <Section title="Plaatsingspunt">
              <div className="flex overflow-hidden rounded border border-stone-200">
                {(["exterior", "center", "interior"] as WallAlignment[]).map((opt) => (
                  <button
                    key={opt}
                    onClick={() => onWallAlignmentChange?.(opt)}
                    className={`flex-1 px-2 py-1.5 text-[11px] font-medium ${
                      wallAlignment === opt
                        ? "bg-stone-800 text-white"
                        : "text-stone-500 hover:bg-stone-100"
                    }`}
                  >
                    {opt === "exterior" ? "Buiten" : opt === "interior" ? "Binnen" : "Hart"}
                  </button>
                ))}
              </div>
              <p className="mt-1 text-[10px] text-stone-400">Spatie = wissel tijdens tekenen</p>
            </Section>

            <Section title="Sneltoetsen">
              <dl className="space-y-1 text-[10px] text-stone-500">
                <div className="flex justify-between"><span>Exacte maat</span><span className="font-mono text-stone-700">getal + Enter</span></div>
                <div className="flex justify-between"><span>Spiegelen</span><span className="font-mono text-stone-700">Spatie</span></div>
                <div className="flex justify-between"><span>Afronden</span><span className="font-mono text-stone-700">Dubbelklik / Esc</span></div>
                <div className="flex justify-between"><span>Sluiten (loop)</span><span className="font-mono text-stone-700">Klik startpunt</span></div>
              </dl>
            </Section>

            <Section title="Wandtype">
              <ConstructionPickerInline
                entries={catalogueEntries}
                filterCategory="wanden"
                onSelect={() => {/* TODO: pre-assign construction to new walls */}}
              />
            </Section>
          </div>
        </div>
      );
    }

    return (
      <div className="w-72 shrink-0 border-l border-stone-200 bg-white p-4">
        <p className="text-xs text-stone-400">Selecteer een ruimte of wand om de eigenschappen te bekijken.</p>
        <div className="mt-6">
          <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-400">Ruimten ({rooms.length})</h3>
          <ul className="space-y-1">
            {rooms.map((r) => (
              <li key={r.id} className="text-xs text-stone-600">
                <span className="font-mono font-medium">{r.id}</span> {r.name} — {(polygonArea(r.polygon) / 1e6).toFixed(1)} m²
              </li>
            ))}
          </ul>
        </div>
        {walls.length > 0 && (
          <div className="mt-4">
            <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-400">Wanden ({walls.length})</h3>
            <ul className="space-y-1">
              {walls.map((w) => {
                let len = 0;
                for (let i = 0; i < w.points.length - 1; i++) len += Math.hypot(w.points[i + 1]!.x - w.points[i]!.x, w.points[i + 1]!.y - w.points[i]!.y);
                return (
                  <li key={w.id} className="text-xs text-stone-600">
                    <span className="font-mono font-medium">{w.id.slice(0, 6)}</span> — {(len / 1000).toFixed(2)} m
                  </li>
                );
              })}
            </ul>
          </div>
        )}
      </div>
    );
  }

  // Room selected
  const area = polygonArea(room.polygon) / 1e6;
  const roomWalls = getWallInfo(room, rooms, windows);

  return (
    <div className="w-72 shrink-0 overflow-y-auto border-l border-stone-200 bg-white">
      {/* Header */}
      <div className="border-b border-stone-100 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-baseline gap-2">
            <span className="font-mono text-sm font-bold text-stone-800">{room.id}</span>
            <span className="text-sm text-stone-600">{room.name}</span>
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
              <span className="text-stone-500">Functie</span>
              <select
                value={room.function}
                onChange={(e) => onUpdateRoom?.(room.id, { function: e.target.value })}
                className="rounded border border-stone-200 bg-white px-1.5 py-0.5 text-xs text-stone-800"
              >
                {FUNCTION_OPTIONS.map(([key, label]) => (
                  <option key={key} value={key}>{label}</option>
                ))}
              </select>
            </div>
            <Row label="Vloeroppervlak" value={`${area.toFixed(2)} m\u00B2`} />
            <EditableNumberField label="Hoogte (mm)" value={room.height} onChange={(val) => onUpdateRoom?.(room.id, { height: val })} />
            <Row label="Volume" value={`${((area * room.height) / 1e3).toFixed(1)} m\u00B3`} />
          </div>
        </Section>

        {/* Walls */}
        <Section title={`Wanden (${roomWalls.length})`}>
          <div className="space-y-1.5">
            {roomWalls.map((w, i) => (
              <WallCard
                key={i}
                wall={w}
                assignedEntryId={wallConstructions[`${room.id}:${i}`]}
                catalogueEntries={catalogueEntries}
                onAssign={(entryId) => onAssignWall?.(room.id, i, entryId)}
              />
            ))}
          </div>
        </Section>

        <Section title="Vloer">
          <ConstructionCard
            label={`Vloer — ${area.toFixed(2)} m\u00B2`}
            badge="Grond" badgeColor="green"
            assignedEntryId={floorConstructions[room.id]}
            catalogueEntries={catalogueEntries}
            filterCategory="vloeren_plafonds"
            onAssign={(entryId) => onAssignFloor?.(room.id, entryId)}
          />
        </Section>

        <Section title="Plafond / Dak">
          <ConstructionCard
            label={`Plafond — ${area.toFixed(2)} m\u00B2`}
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
      <h3 className="mb-1.5 text-xs font-semibold uppercase tracking-wider text-stone-400">{title}</h3>
      {children}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between text-xs">
      <dt className="text-stone-500">{label}</dt>
      <dd className="font-mono text-stone-800">{value}</dd>
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
        <span className="text-stone-500">{label}</span>
        <input
          autoFocus value={draft} onChange={(e) => setDraft(e.target.value)}
          onBlur={() => { if (draft.trim()) onChange(draft.trim()); setEditing(false); }}
          onKeyDown={(e) => {
            if (e.key === "Enter") { if (draft.trim()) onChange(draft.trim()); setEditing(false); }
            if (e.key === "Escape") setEditing(false);
          }}
          className="w-28 rounded border border-amber-300 bg-amber-50 px-1.5 py-0.5 text-right text-xs text-stone-800 outline-none"
        />
      </div>
    );
  }
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-stone-500">{label}</span>
      <button onClick={() => { setDraft(value); setEditing(true); }} className="font-mono text-stone-800 hover:text-amber-700 hover:underline">{value}</button>
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
        <span className="text-stone-500">{label}</span>
        <input
          autoFocus type="number" value={draft} onChange={(e) => setDraft(e.target.value)}
          onBlur={() => { const n = parseInt(draft, 10); if (!isNaN(n) && n > 0) onChange(n); setEditing(false); }}
          onKeyDown={(e) => {
            if (e.key === "Enter") { const n = parseInt(draft, 10); if (!isNaN(n) && n > 0) onChange(n); setEditing(false); }
            if (e.key === "Escape") setEditing(false);
          }}
          className="w-20 rounded border border-amber-300 bg-amber-50 px-1.5 py-0.5 text-right text-xs text-stone-800 outline-none"
        />
      </div>
    );
  }
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-stone-500">{label}</span>
      <button onClick={() => { setDraft(String(value)); setEditing(true); }} className="font-mono text-stone-800 hover:text-amber-700 hover:underline">{value}</button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Wall card
// ---------------------------------------------------------------------------

interface WallInfo { direction: string; length: number; type: "exterior" | "interior"; adjacentName: string | null; windowCount: number; }

function WallCard({ wall, assignedEntryId, catalogueEntries, onAssign }: {
  wall: WallInfo; assignedEntryId?: string; catalogueEntries: CatalogueEntry[]; onAssign?: (entryId: string | null) => void;
}) {
  const [picking, setPicking] = useState(false);
  const assigned = assignedEntryId ? catalogueEntries.find((e) => e.id === assignedEntryId) : null;

  return (
    <div className="rounded border border-stone-100 px-2 py-1.5 text-xs">
      <div className="flex items-center justify-between">
        <span className="font-medium text-stone-700">{wall.direction} — {(wall.length / 1000).toFixed(2)} m</span>
        <span className={`rounded px-1.5 py-0.5 text-[10px] font-medium ${wall.type === "exterior" ? "bg-red-50 text-red-700" : "bg-blue-50 text-blue-700"}`}>
          {wall.type === "exterior" ? "Gevel" : wall.adjacentName ?? "Intern"}
        </span>
      </div>
      {wall.windowCount > 0 && <div className="mt-0.5 text-blue-600">{wall.windowCount} kozijn{wall.windowCount > 1 ? "en" : ""}</div>}
      {assigned ? (
        <div className="mt-1 flex items-center justify-between">
          <span className="text-[10px] text-green-700">{assigned.name} (U={assigned.uValue})</span>
          <button onClick={() => onAssign?.(null)} className="text-[10px] text-red-400 hover:text-red-600">x</button>
        </div>
      ) : picking ? (
        <ConstructionPicker entries={catalogueEntries} filterCategory="wanden" onSelect={(id) => { onAssign?.(id); setPicking(false); }} onCancel={() => setPicking(false)} />
      ) : (
        <button onClick={() => setPicking(true)} className="mt-1 text-[10px] text-amber-600 hover:text-amber-800">Constructie toewijzen...</button>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Construction card + picker
// ---------------------------------------------------------------------------

function ConstructionCard({ label, badge, badgeColor, assignedEntryId, catalogueEntries, filterCategory, onAssign }: {
  label: string; badge: string; badgeColor: "green" | "purple"; assignedEntryId?: string; catalogueEntries: CatalogueEntry[];
  filterCategory: CatalogueCategory; onAssign?: (entryId: string | null) => void;
}) {
  const [picking, setPicking] = useState(false);
  const assigned = assignedEntryId ? catalogueEntries.find((e) => e.id === assignedEntryId) : null;
  const colors = badgeColor === "green" ? "bg-green-50 text-green-700" : "bg-purple-50 text-purple-700";

  return (
    <div className="rounded border border-stone-100 px-2 py-1.5 text-xs">
      <div className="flex items-center justify-between">
        <span className="font-medium text-stone-700">{label}</span>
        <span className={`rounded px-1.5 py-0.5 text-[10px] font-medium ${colors}`}>{badge}</span>
      </div>
      {assigned ? (
        <div className="mt-1 flex items-center justify-between">
          <span className="text-[10px] text-green-700">{assigned.name} (U={assigned.uValue})</span>
          <button onClick={() => onAssign?.(null)} className="text-[10px] text-red-400 hover:text-red-600">x</button>
        </div>
      ) : picking ? (
        <ConstructionPicker entries={catalogueEntries} filterCategory={filterCategory} onSelect={(id) => { onAssign?.(id); setPicking(false); }} onCancel={() => setPicking(false)} />
      ) : (
        <button onClick={() => setPicking(true)} className="mt-1 text-[10px] text-amber-600 hover:text-amber-800">Constructie toewijzen...</button>
      )}
    </div>
  );
}

function ConstructionPickerInline({ entries, filterCategory, onSelect }: {
  entries: CatalogueEntry[]; filterCategory: CatalogueCategory; onSelect: (entryId: string) => void;
}) {
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);
  const filtered = entries.filter((e) => e.category === filterCategory && (!search || e.name.toLowerCase().includes(search.toLowerCase())));

  if (!open) {
    return <button onClick={() => setOpen(true)} className="text-[10px] text-amber-600 hover:text-amber-800">Constructie toewijzen...</button>;
  }

  return (
    <div className="rounded border border-amber-200 bg-amber-50/50 p-1.5">
      <input
        autoFocus placeholder={`Zoek in ${CATALOGUE_CATEGORY_LABELS[filterCategory]}...`}
        value={search} onChange={(e) => setSearch(e.target.value)}
        className="mb-1 w-full rounded border border-stone-200 bg-white px-1.5 py-0.5 text-[10px] outline-none focus:border-amber-400"
      />
      <div className="max-h-40 overflow-y-auto">
        {filtered.length === 0 ? <div className="py-1 text-center text-[10px] text-stone-400">Geen resultaten</div> : (
          filtered.map((e) => (
            <button key={e.id} onClick={() => onSelect(e.id)} className="block w-full rounded px-1.5 py-1 text-left text-[10px] text-stone-700 hover:bg-amber-100">
              <span className="font-medium">{e.name}</span> <span className="text-stone-400">U={e.uValue}</span>
            </button>
          ))
        )}
      </div>
    </div>
  );
}

function ConstructionPicker({ entries, filterCategory, onSelect, onCancel }: {
  entries: CatalogueEntry[]; filterCategory: CatalogueCategory; onSelect: (entryId: string) => void; onCancel: () => void;
}) {
  const [search, setSearch] = useState("");
  const filtered = entries.filter((e) => e.category === filterCategory && (!search || e.name.toLowerCase().includes(search.toLowerCase())));

  return (
    <div className="mt-1 rounded border border-amber-200 bg-amber-50/50 p-1.5">
      <div className="mb-1 flex items-center gap-1">
        <input
          autoFocus placeholder={`Zoek in ${CATALOGUE_CATEGORY_LABELS[filterCategory]}...`}
          value={search} onChange={(e) => setSearch(e.target.value)}
          className="flex-1 rounded border border-stone-200 bg-white px-1.5 py-0.5 text-[10px] outline-none focus:border-amber-400"
        />
        <button onClick={onCancel} className="text-[10px] text-stone-400 hover:text-stone-600">Annuleer</button>
      </div>
      <div className="max-h-40 overflow-y-auto">
        {filtered.length === 0 ? <div className="py-1 text-center text-[10px] text-stone-400">Geen resultaten</div> : (
          filtered.map((e) => (
            <button key={e.id} onClick={() => onSelect(e.id)} className="block w-full rounded px-1.5 py-1 text-left text-[10px] text-stone-700 hover:bg-amber-100">
              <span className="font-medium">{e.name}</span> <span className="text-stone-400">U={e.uValue}</span>
            </button>
          ))
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Wall analysis
// ---------------------------------------------------------------------------

function getWallInfo(room: ModelRoom, allRooms: ModelRoom[], allWindows: ModelWindow[]): WallInfo[] {
  const poly = room.polygon;
  const n = poly.length;
  const roomWindows = allWindows.filter((w) => w.roomId === room.id);
  const walls: WallInfo[] = [];

  for (let i = 0; i < n; i++) {
    const a = poly[i]!;
    const b = poly[(i + 1) % n]!;
    const length = Math.sqrt((b.x - a.x) ** 2 + (b.y - a.y) ** 2);
    const direction = wallDirection(poly, i);

    let type: "exterior" | "interior" = "exterior";
    let adjacentName: string | null = null;
    for (const other of allRooms) {
      if (other.id === room.id) continue;
      if (hasSharedWall(a, b, other.polygon)) { type = "interior"; adjacentName = other.name; break; }
    }

    walls.push({ direction, length, type, adjacentName, windowCount: roomWindows.filter((w) => w.wallIndex === i).length });
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
