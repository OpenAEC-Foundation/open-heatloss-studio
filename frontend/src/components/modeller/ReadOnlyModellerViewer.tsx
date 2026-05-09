/**
 * Read-only Modeller viewer.
 *
 * Renders rooms derived from `Project.rooms` (the calc-side data) as 2D
 * SVG polygons. The user does NOT draw or edit here — geometry is inferred
 * from each room's wall constructions (perimeter from Σ wall.area / height,
 * area from room.floor_area).
 *
 * Walls are colored by `ConstructionElement.boundary_type` so the user can
 * see at a glance which walls are exterior, ground, adjacent_room, etc.
 *
 * 3D view is intentionally not implemented in this iteration; the read-only
 * 2D view is the agreed starting point for the Modeller-as-viewer design.
 */
import { useMemo, useState } from "react";

import type { Project } from "../../types";
import {
  deriveModelRooms,
  deriveRoomGeometry,
  parseFloorFromName,
  wallConstructions,
} from "../../lib/deriveRoomGeometry";

interface ReadOnlyModellerViewerProps {
  project: Project;
}

/** Boundary type → CSS color (matches the existing palette in constants.ts). */
const BOUNDARY_COLOR: Record<string, string> = {
  exterior: "#3b82f6", // blue
  unheated_space: "#a855f7", // purple
  adjacent_room: "#22c55e", // green
  adjacent_building: "#f59e0b", // amber
  ground: "#78716c", // stone
  water: "#14b8a6", // teal
};

const BOUNDARY_LABEL_NL: Record<string, string> = {
  exterior: "Buiten",
  unheated_space: "Onverwarmd",
  adjacent_room: "Aangrenzend",
  adjacent_building: "Naburig gebouw",
  ground: "Grond",
  water: "Water",
};

/** Stroke for un-typed wall segments. */
const FALLBACK_WALL_STROKE = "#94a3b8"; // slate

export function ReadOnlyModellerViewer({ project }: ReadOnlyModellerViewerProps) {
  const [selectedRoomId, setSelectedRoomId] = useState<string | null>(null);

  const modelRooms = useMemo(() => deriveModelRooms(project), [project]);

  // Compute viewport bounds from the laid-out polygons.
  const bounds = useMemo(() => {
    if (modelRooms.length === 0) {
      return { minX: 0, minY: 0, maxX: 10000, maxY: 10000 };
    }
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const r of modelRooms) {
      for (const p of r.polygon) {
        if (p.x < minX) minX = p.x;
        if (p.y < minY) minY = p.y;
        if (p.x > maxX) maxX = p.x;
        if (p.y > maxY) maxY = p.y;
      }
    }
    return { minX, minY, maxX, maxY };
  }, [modelRooms]);

  // Padding for the viewBox so polygons aren't flush with the SVG edges.
  const PAD = 1500; // mm
  const viewBox = `${bounds.minX - PAD} ${bounds.minY - PAD} ${
    bounds.maxX - bounds.minX + 2 * PAD
  } ${bounds.maxY - bounds.minY + 2 * PAD}`;

  // Grouped legend for boundary types actually present in the project.
  const boundaryTypesPresent = useMemo(() => {
    const set = new Set<string>();
    for (const room of project.rooms) {
      for (const c of wallConstructions(room)) {
        set.add(c.boundary_type);
      }
    }
    return [...set].sort();
  }, [project.rooms]);

  if (modelRooms.length === 0) {
    return (
      <div className="flex h-full w-full items-center justify-center bg-surface-2">
        <div className="text-center">
          <div className="text-sm text-on-surface-2">Geen vertrekken om te tonen.</div>
          <div className="mt-1 text-xs text-scaffold-gray">
            Voeg ruimten toe via de Vertrekken-tabel.
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="relative h-full w-full bg-surface-2">
      <svg
        className="h-full w-full"
        viewBox={viewBox}
        preserveAspectRatio="xMidYMid meet"
      >
        {/* Y is positive downward in the modeller convention; flip so floors
            render with floor 0 at top. SVG default Y is downward already, so
            we keep as-is. */}
        {modelRooms.map((mr) => {
          const calcRoom = project.rooms.find((r) => r.id === mr.id);
          if (!calcRoom) return null;
          const geom = deriveRoomGeometry(calcRoom);
          const isSelected = selectedRoomId === mr.id;

          // Polygon path string from the (already-translated) polygon.
          const path =
            mr.polygon.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x} ${p.y}`).join(" ") + " Z";

          // Center point for the label.
          const cx = mr.polygon.reduce((s, p) => s + p.x, 0) / mr.polygon.length;
          const cy = mr.polygon.reduce((s, p) => s + p.y, 0) / mr.polygon.length;

          return (
            <g
              key={mr.id}
              onClick={() => setSelectedRoomId(isSelected ? null : mr.id)}
              style={{ cursor: "pointer" }}
            >
              {/* Room fill (very subtle, hover-highlights via :hover not used
                  here because SVG :hover with a filter would be heavier; we
                  rely on selection state instead) */}
              <path
                d={path}
                fill={isSelected ? "#fef3c7" : "#f8fafc"}
                fillOpacity={isSelected ? 0.9 : 0.5}
                stroke="none"
              />

              {/* Per-side wall strokes — color by boundary_type */}
              {mr.polygon.map((p, i) => {
                const next = mr.polygon[(i + 1) % mr.polygon.length];
                if (!next) return null;
                const wall = geom.walls[i];
                const stroke = wall ? BOUNDARY_COLOR[wall.boundary_type] : FALLBACK_WALL_STROKE;
                return (
                  <line
                    key={i}
                    x1={p.x}
                    y1={p.y}
                    x2={next.x}
                    y2={next.y}
                    stroke={stroke ?? FALLBACK_WALL_STROKE}
                    strokeWidth={120}
                    strokeLinecap="round"
                  />
                );
              })}

              {/* Room name + area label */}
              <text
                x={cx}
                y={cy}
                textAnchor="middle"
                dominantBaseline="central"
                fontSize={350}
                fontFamily="Inter, system-ui, sans-serif"
                fontWeight={500}
                fill="#0f172a"
              >
                {mr.name}
              </text>
              <text
                x={cx}
                y={cy + 500}
                textAnchor="middle"
                dominantBaseline="central"
                fontSize={250}
                fontFamily="Inter, system-ui, sans-serif"
                fill="#475569"
              >
                {calcRoom.floor_area.toFixed(1)} m²
              </text>
            </g>
          );
        })}

        {/* Floor labels (one per floor cluster) */}
        {(() => {
          const floors = new Map<number, { minX: number; minY: number }>();
          for (const mr of modelRooms) {
            const f = parseFloorFromName(mr.name);
            const top = mr.polygon.reduce(
              (acc, p) => ({ x: Math.min(acc.x, p.x), y: Math.min(acc.y, p.y) }),
              { x: Infinity, y: Infinity },
            );
            const cur = floors.get(f);
            if (!cur || top.y < cur.minY) {
              floors.set(f, { minX: top.x, minY: top.y });
            }
          }
          return [...floors.entries()].map(([floor, pos]) => (
            <text
              key={floor}
              x={pos.minX}
              y={pos.minY - 600}
              fontSize={400}
              fontFamily="Inter, system-ui, sans-serif"
              fontWeight={700}
              fill="#475569"
            >
              {floor === -1
                ? "Kelder"
                : floor === 0
                ? "Begane grond"
                : `Verdieping ${floor}`}
            </text>
          ));
        })()}
      </svg>

      {/* Legend */}
      <div className="absolute bottom-4 right-4 rounded-md bg-surface/95 p-3 shadow-md backdrop-blur-sm">
        <div className="mb-1 text-xs font-medium uppercase tracking-wide text-scaffold-gray">
          Wandtype
        </div>
        <ul className="space-y-1 text-xs">
          {boundaryTypesPresent.map((bt) => (
            <li key={bt} className="flex items-center gap-2">
              <span
                className="inline-block h-1 w-4 rounded-sm"
                style={{ backgroundColor: BOUNDARY_COLOR[bt] ?? FALLBACK_WALL_STROKE }}
              />
              <span className="text-on-surface">{BOUNDARY_LABEL_NL[bt] ?? bt}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Read-only badge — niet weghalen, hint dat de viewer geen edit-tools heeft */}
      <div className="absolute left-4 top-4 rounded-md bg-surface/95 px-2.5 py-1 text-xs font-medium uppercase tracking-wide text-scaffold-gray shadow-sm backdrop-blur-sm">
        Read-only viewer
      </div>
    </div>
  );
}
