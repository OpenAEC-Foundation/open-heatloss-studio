/**
 * 3D building viewer using ThatOpen Components + Three.js.
 *
 * Vabi-style rendering: transparent colored surfaces per room face,
 * white wireframe edges, blue window panes.
 *
 * Supports two render modes:
 * - "normal": surfaces colored by room function (light transparent)
 * - "uvalue": surfaces colored by thermal performance (green→yellow→red)
 *
 * Architecture: each room polygon edge = 1 wall surface. These can later
 * be split into sub-surfaces for multiple construction types per wall.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import * as OBC from "@thatopen/components";

import type { ModelRoom, ModelWindow, ModelDoor, Selection } from "./types";
import { polygonCenter, getSharedEdges } from "./geometry";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export type RenderMode = "normal" | "uvalue";

interface FloorCanvas3DProps {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
  selection: Selection;
  onSelect: (sel: Selection) => void;
  onDeleteRoom?: (id: string) => void;
  // Construction assignments for U-value coloring
  wallConstructions?: Record<string, string>;
  floorConstructions?: Record<string, string>;
  roofConstructions?: Record<string, string>;
  catalogueUValues?: Record<string, number>;
}

// ---------------------------------------------------------------------------
// Colors & constants
// ---------------------------------------------------------------------------

const FUNCTION_COLORS: Record<string, number> = {
  living_room: 0xfef3c7,
  kitchen: 0xfef9c3,
  bedroom: 0xdbeafe,
  bathroom: 0xcffafe,
  toilet: 0xe0e7ff,
  hallway: 0xf5f5f4,
  landing: 0xf5f5f4,
  storage: 0xe7e5e4,
  attic: 0xfce7f3,
  custom: 0xf3f4f6,
};

const SELECTED_COLOR = 0xf59e0b;
const WINDOW_COLOR = 0x3b82f6;
const UNASSIGNED_COLOR = 0xcccccc;
const SURFACE_OPACITY = 0.55;
const SELECTED_OPACITY = 0.7;
const WINDOW_SILL_H = 0.8;  // m
const WINDOW_HEAD_H = 2.1;  // m
const DOOR_HEAD_H = 2.1;    // m
const WIREFRAME_COLOR = 0xffffff;
const WIREFRAME_WIDTH = 2;

// ---------------------------------------------------------------------------
// U-value → color mapping (thermal performance)
// ---------------------------------------------------------------------------

/** Map U-value to a color on a green→yellow→red gradient. */
function uValueToColor(u: number): number {
  // Excellent: U < 0.25 → dark green
  // Good:     U 0.25-0.5 → green
  // Fair:     U 0.5-1.0 → yellow-green
  // Poor:     U 1.0-2.0 → yellow
  // Bad:      U 2.0-3.5 → orange
  // Terrible: U > 3.5 → red
  const t = Math.min(1, Math.max(0, u / 4.0));

  // Hue: 120 (green) → 0 (red)
  const hue = (1 - t) * 120;
  const saturation = 0.85;
  const lightness = 0.50;

  return new THREE.Color().setHSL(hue / 360, saturation, lightness).getHex();
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/** Surface identity stored per mesh for hit testing. */
interface SurfaceId {
  roomId: string;
  type: "wall" | "floor" | "ceiling";
  wallIndex?: number;
}

export function FloorCanvas3D({
  rooms,
  windows,
  doors,
  selection,
  onSelect,
  onDeleteRoom,
  wallConstructions = {},
  floorConstructions: _floorConstructions = {},
  roofConstructions: _roofConstructions = {},
  catalogueUValues = {},
}: FloorCanvas3DProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const componentsRef = useRef<OBC.Components | null>(null);
  const modelGroupRef = useRef<THREE.Group>(new THREE.Group());
  const wireframeGroupRef = useRef<THREE.Group>(new THREE.Group());
  const raycasterRef = useRef(new THREE.Raycaster());
  const mouseRef = useRef(new THREE.Vector2());
  const surfaceMeshMapRef = useRef(new Map<THREE.Mesh, SurfaceId>());
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);

  // Derive selectedRoomId from selection
  const selectedRoomId = selection?.type === "room" ? selection.roomId
    : selection?.type === "wall" ? selection.roomId
    : selection?.type === "window" ? selection.roomId
    : null;
  const selectedWallIndex = selection?.type === "wall" ? selection.wallIndex : null;

  // Render mode toggle
  const [renderMode, setRenderMode] = useState<RenderMode>("normal");

  // Section plane state
  const [sectionX, setSectionX] = useState<number | null>(null);
  const [sectionY, setSectionY] = useState<number | null>(null);
  const [sectionZ, setSectionZ] = useState<number | null>(null);

  // Context menu
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);

  // Bounding box for section plane sliders
  const bounds = useMemo(() => {
    let minX = Infinity, maxX = -Infinity;
    let minZ = Infinity, maxZ = -Infinity;
    let maxY = 0;
    for (const room of rooms) {
      for (const p of room.polygon) {
        const x = p.x / 1000;
        const z = p.y / 1000;
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (z < minZ) minZ = z;
        if (z > maxZ) maxZ = z;
      }
      const top = room.floor * (room.height / 1000 + 0.3) + room.height / 1000;
      if (top > maxY) maxY = top;
    }
    const m = 1;
    return {
      minX: (minX === Infinity ? -5 : minX) - m,
      maxX: (maxX === -Infinity ? 15 : maxX) + m,
      minY: -0.1,
      maxY: Math.max(maxY + m, 4),
      minZ: (minZ === Infinity ? -5 : minZ) - m,
      maxZ: (maxZ === -Infinity ? 15 : maxZ) + m,
    };
  }, [rooms]);

  // Initial camera target
  const initialCenter = useMemo(() => {
    if (rooms.length === 0) return { x: 5, y: 1.3, z: 5 };
    let cx = 0, cz = 0;
    for (const room of rooms) {
      const c = polygonCenter(room.polygon);
      cx += c.x / 1000;
      cz += c.y / 1000;
    }
    cx /= rooms.length;
    cz /= rooms.length;
    return { x: cx, y: 1.3, z: cz };
  }, [rooms]);

  // -----------------------------------------------------------------------
  // Initialize ThatOpen scene ONCE
  // -----------------------------------------------------------------------
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const components = new OBC.Components();
    componentsRef.current = components;

    const worlds = components.get(OBC.Worlds);
    const world = worlds.create<OBC.SimpleScene, OBC.SimpleCamera, OBC.SimpleRenderer>();

    world.scene = new OBC.SimpleScene(components);
    world.renderer = new OBC.SimpleRenderer(components, container);
    world.camera = new OBC.SimpleCamera(components);

    components.init();
    world.scene.setup();

    const threeRenderer = world.renderer.three as THREE.WebGLRenderer;
    threeRenderer.localClippingEnabled = true;
    rendererRef.current = threeRenderer;

    const scene = world.scene.three;
    scene.background = new THREE.Color(0xf5f5f4);
    scene.children
      .filter((c) => c instanceof THREE.Light)
      .forEach((l) => scene.remove(l));

    scene.add(new THREE.AmbientLight(0xffffff, 0.8));
    const dir1 = new THREE.DirectionalLight(0xffffff, 0.6);
    dir1.position.set(20, 30, 10);
    scene.add(dir1);
    const dir2 = new THREE.DirectionalLight(0xffffff, 0.25);
    dir2.position.set(-15, 20, -15);
    scene.add(dir2);

    const grids = components.get(OBC.Grids);
    grids.create(world);

    const c = initialCenter;
    world.camera.controls.setLookAt(
      c.x + 12, 10, c.z + 12,
      c.x, c.y, c.z,
      true,
    );

    scene.add(modelGroupRef.current);
    scene.add(wireframeGroupRef.current);

    return () => {
      modelGroupRef.current.removeFromParent();
      wireframeGroupRef.current.removeFromParent();
      components.dispose();
      componentsRef.current = null;
      rendererRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // -----------------------------------------------------------------------
  // Update clipping planes
  // -----------------------------------------------------------------------
  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;

    const planes: THREE.Plane[] = [];
    if (sectionX !== null) planes.push(new THREE.Plane(new THREE.Vector3(-1, 0, 0), sectionX));
    if (sectionY !== null) planes.push(new THREE.Plane(new THREE.Vector3(0, -1, 0), sectionY));
    if (sectionZ !== null) planes.push(new THREE.Plane(new THREE.Vector3(0, 0, -1), sectionZ));
    renderer.clippingPlanes = planes;
  }, [sectionX, sectionY, sectionZ]);

  // -----------------------------------------------------------------------
  // Build/update 3D geometry
  // -----------------------------------------------------------------------
  useEffect(() => {
    const group = modelGroupRef.current;
    const wireGroup = wireframeGroupRef.current;
    clearGroup(group);
    clearGroup(wireGroup);
    surfaceMeshMapRef.current.clear();

    const sharedEdges = getSharedEdges(rooms);

    for (const room of rooms) {
      const isRoomSelected = room.id === selectedRoomId;
      const floorY = room.floor * (room.height / 1000 + 0.3);
      const h = room.height / 1000;
      const poly = room.polygon;
      const n = poly.length;
      const baseColor = FUNCTION_COLORS[room.function] ?? FUNCTION_COLORS.custom!;

      // --- Wall surfaces (one per polygon edge) ---
      const roomWindows = windows.filter((w) => w.roomId === room.id);
      const roomDoors = doors.filter((d) => d.roomId === room.id);

      for (let i = 0; i < n; i++) {
        const ni = (i + 1) % n;
        const a = poly[i]!;
        const b = poly[ni]!;
        const edgeLen = Math.hypot(b.x - a.x, b.y - a.y);
        if (edgeLen < 1) continue;

        const isShared = sharedEdges.has(`${room.id}:${i}`);
        const wallKey = `${room.id}:${i}`;
        const isWallSelected = isRoomSelected && selectedWallIndex === i;
        const wallColor = getSurfaceColor(
          renderMode, isWallSelected, baseColor,
          wallConstructions[wallKey], catalogueUValues,
          isShared,
        );

        // Collect openings on this wall
        const edgeOpenings: { tStart: number; tEnd: number; sillH: number; headH: number }[] = [];
        for (const win of roomWindows) {
          if (win.wallIndex % n !== i) continue;
          const tS = Math.max(0, (win.offset - win.width / 2) / edgeLen);
          const tE = Math.min(1, (win.offset + win.width / 2) / edgeLen);
          edgeOpenings.push({ tStart: tS, tEnd: tE, sillH: WINDOW_SILL_H, headH: Math.min(WINDOW_HEAD_H, h) });
        }
        for (const dr of roomDoors) {
          if (dr.wallIndex % n !== i) continue;
          const tS = Math.max(0, (dr.offset - dr.width / 2) / edgeLen);
          const tE = Math.min(1, (dr.offset + dr.width / 2) / edgeLen);
          edgeOpenings.push({ tStart: tS, tEnd: tE, sillH: 0, headH: Math.min(DOOR_HEAD_H, h) });
        }
        edgeOpenings.sort((x, y) => x.tStart - y.tStart);

        // Generate wall pieces around openings
        const pieces = computeWallPieces(h, edgeOpenings);

        const ax = a.x / 1000, az = a.y / 1000;
        const bx = b.x / 1000, bz = b.y / 1000;

        for (const piece of pieces) {
          const geom = createWallSurfaceGeom(ax, az, bx, bz, piece);
          const mat = new THREE.MeshStandardMaterial({
            color: wallColor,
            side: THREE.DoubleSide,
            transparent: true,
            opacity: isWallSelected ? SELECTED_OPACITY : SURFACE_OPACITY,
            depthWrite: false,
          });
          const mesh = new THREE.Mesh(geom, mat);
          mesh.position.y = floorY;
          mesh.renderOrder = 1;
          group.add(mesh);
          surfaceMeshMapRef.current.set(mesh, { roomId: room.id, type: "wall", wallIndex: i });
        }

        // Window glass panes (blue)
        for (const win of roomWindows) {
          if (win.wallIndex % n !== i) continue;
          const paneGeom = createPaneGeom(ax, az, bx, bz, edgeLen, win, h);
          if (paneGeom) {
            const paneMat = new THREE.MeshStandardMaterial({
              color: WINDOW_COLOR,
              transparent: true,
              opacity: 0.6,
              side: THREE.DoubleSide,
              depthWrite: false,
            });
            const paneMesh = new THREE.Mesh(paneGeom, paneMat);
            paneMesh.position.y = floorY;
            paneMesh.renderOrder = 2;
            group.add(paneMesh);
          }
        }
      }

      // --- Wireframe edges ---
      const edgePositions: number[] = [];
      for (let i = 0; i < n; i++) {
        const ni = (i + 1) % n;
        const ax = poly[i]!.x / 1000;
        const az = poly[i]!.y / 1000;
        const bx = poly[ni]!.x / 1000;
        const bz = poly[ni]!.y / 1000;

        // Bottom edge
        edgePositions.push(ax, floorY, az, bx, floorY, bz);
        // Top edge
        edgePositions.push(ax, floorY + h, az, bx, floorY + h, bz);
        // Vertical edge at each vertex
        edgePositions.push(ax, floorY, az, ax, floorY + h, az);
      }

      const edgeGeom = new THREE.BufferGeometry();
      edgeGeom.setAttribute("position", new THREE.Float32BufferAttribute(edgePositions, 3));
      const edgeMat = new THREE.LineBasicMaterial({ color: WIREFRAME_COLOR, linewidth: WIREFRAME_WIDTH });
      const edgeLines = new THREE.LineSegments(edgeGeom, edgeMat);
      edgeLines.renderOrder = 10;
      wireGroup.add(edgeLines);

      // --- Room label ---
      const center = polygonCenter(poly);
      const sprite = createLabelSprite(room.id, room.name, isRoomSelected);
      sprite.position.set(center.x / 1000, floorY + h / 2, center.y / 1000);
      sprite.scale.set(2, 1, 1);
      group.add(sprite);
    }
  }, [rooms, windows, doors, selectedRoomId, selectedWallIndex, renderMode, wallConstructions, catalogueUValues]);

  // -----------------------------------------------------------------------
  // Click handler (room selection)
  // -----------------------------------------------------------------------
  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      setCtxMenu(null);
      if (e.button !== 0) return;

      const container = containerRef.current;
      if (!container) return;

      const rect = container.getBoundingClientRect();
      mouseRef.current.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      mouseRef.current.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;

      const components = componentsRef.current;
      if (!components) return;
      const worlds = components.get(OBC.Worlds);
      const worldsList = Array.from(worlds.list.values());
      if (worldsList.length === 0) return;
      const cam = worldsList[0]!.camera.three;
      raycasterRef.current.setFromCamera(mouseRef.current, cam);

      const meshes = Array.from(surfaceMeshMapRef.current.keys());
      const intersects = raycasterRef.current.intersectObjects(meshes, false);
      if (intersects.length > 0) {
        const surface = surfaceMeshMapRef.current.get(intersects[0]!.object as THREE.Mesh);
        if (surface) {
          if (surface.type === "wall" && surface.wallIndex !== undefined) {
            onSelect({ type: "wall", roomId: surface.roomId, wallIndex: surface.wallIndex });
          } else {
            onSelect({ type: "room", roomId: surface.roomId });
          }
          return;
        }
      }
      onSelect(null);
    },
    [onSelect],
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      e.preventDefault();
      setCtxMenu({ x: e.clientX, y: e.clientY });
    },
    [],
  );

  return (
    <div
      ref={containerRef}
      className="relative h-full w-full"
      onClick={handleClick}
      onContextMenu={handleContextMenu}
    >
      {/* Render mode toggle */}
      <div
        className="absolute right-3 top-3 z-10 flex overflow-hidden rounded-lg border border-stone-200 bg-white/95 shadow-sm backdrop-blur-sm text-xs select-none"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={() => setRenderMode("normal")}
          className={`px-3 py-1.5 font-medium transition-colors ${
            renderMode === "normal" ? "bg-stone-800 text-white" : "text-stone-500 hover:bg-stone-100"
          }`}
        >
          Normaal
        </button>
        <button
          onClick={() => setRenderMode("uvalue")}
          className={`px-3 py-1.5 font-medium transition-colors ${
            renderMode === "uvalue" ? "bg-stone-800 text-white" : "text-stone-500 hover:bg-stone-100"
          }`}
        >
          U-waarde
        </button>
      </div>

      {/* U-value legend */}
      {renderMode === "uvalue" && (
        <div
          className="absolute right-3 top-12 z-10 rounded-lg bg-white/95 p-2.5 shadow-lg backdrop-blur-sm text-[10px] select-none"
          onClick={(e) => e.stopPropagation()}
        >
          <div className="mb-1 font-semibold text-stone-700 text-[11px]">U-waarde (W/m²K)</div>
          <div className="flex flex-col gap-0.5">
            <LegendRow color="#22c55e" label="< 0.5 (goed)" />
            <LegendRow color="#a3e635" label="0.5 – 1.0" />
            <LegendRow color="#facc15" label="1.0 – 2.0" />
            <LegendRow color="#f97316" label="2.0 – 3.5" />
            <LegendRow color="#ef4444" label="> 3.5 (slecht)" />
            <LegendRow color="#9ca3af" label="Niet toegewezen" />
            <LegendRow color="#3b82f6" label="Raam / deur" />
          </div>
        </div>
      )}

      {/* Section plane controls */}
      <div
        className="absolute left-3 bottom-3 z-10 flex flex-col gap-1.5 rounded-lg bg-white/95 p-3 shadow-lg backdrop-blur-sm text-xs select-none"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="font-semibold text-stone-700 text-[11px] tracking-wide uppercase">Doorsnede</div>

        <SectionRow label="X" enabled={sectionX !== null}
          value={sectionX ?? (bounds.minX + bounds.maxX) / 2}
          min={bounds.minX} max={bounds.maxX}
          onToggle={(on) => setSectionX(on ? (bounds.minX + bounds.maxX) / 2 : null)}
          onChange={setSectionX}
        />
        <SectionRow label="Y" enabled={sectionY !== null}
          value={sectionY ?? bounds.maxY / 2}
          min={bounds.minY} max={bounds.maxY}
          onToggle={(on) => setSectionY(on ? bounds.maxY / 2 : null)}
          onChange={setSectionY}
        />
        <SectionRow label="Z" enabled={sectionZ !== null}
          value={sectionZ ?? (bounds.minZ + bounds.maxZ) / 2}
          min={bounds.minZ} max={bounds.maxZ}
          onToggle={(on) => setSectionZ(on ? (bounds.minZ + bounds.maxZ) / 2 : null)}
          onChange={setSectionZ}
        />

        <button
          className="mt-1 rounded bg-stone-100 px-2 py-1 text-[10px] text-stone-500 hover:bg-stone-200 hover:text-stone-700"
          onClick={() => {
            setSectionX(null); setSectionY(null); setSectionZ(null);
          }}
        >
          Wis alles
        </button>
      </div>

      {/* Right-click context menu */}
      {ctxMenu && (
        <div
          className="fixed z-50 min-w-[140px] rounded-lg bg-white/95 py-1 shadow-xl backdrop-blur-sm text-xs"
          style={{ left: ctxMenu.x, top: ctxMenu.y }}
          onClick={() => setCtxMenu(null)}
        >
          {selectedRoomId && (
            <>
              <button
                className="w-full px-3 py-1.5 text-left hover:bg-stone-100 text-stone-700"
                onClick={() => onDeleteRoom?.(selectedRoomId)}
              >
                Verwijder ruimte
              </button>
              <div className="my-0.5 border-t border-stone-200" />
            </>
          )}
          {!selectedRoomId && (
            <div className="px-3 py-1.5 text-stone-400 italic">Geen selectie</div>
          )}
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Section row sub-component
// ---------------------------------------------------------------------------

function SectionRow({ label, enabled, value, min, max, onToggle, onChange }: {
  label: string; enabled: boolean; value: number; min: number; max: number;
  onToggle: (on: boolean) => void; onChange: (v: number) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <label className="flex items-center gap-1.5 cursor-pointer">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => onToggle(e.target.checked)}
          className="h-3.5 w-3.5 rounded border-stone-300 accent-amber-500"
        />
        <span className="font-mono font-semibold text-stone-600 w-3">{label}</span>
      </label>
      {enabled && (
        <input
          type="range"
          min={min}
          max={max}
          step={(max - min) / 200}
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value))}
          className="w-28 h-1.5 accent-amber-500"
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Legend row
// ---------------------------------------------------------------------------

function LegendRow({ color, label }: { color: string; label: string }) {
  return (
    <div className="flex items-center gap-1.5">
      <div className="h-2.5 w-4 rounded-sm" style={{ backgroundColor: color }} />
      <span className="text-stone-600">{label}</span>
    </div>
  );
}

// =============================================================================
// Geometry helpers
// =============================================================================

function clearGroup(group: THREE.Group): void {
  while (group.children.length > 0) {
    const child = group.children[0]!;
    group.remove(child);
    if (child instanceof THREE.Mesh) {
      child.geometry.dispose();
      if (child.material instanceof THREE.Material) child.material.dispose();
    }
    if (child instanceof THREE.LineSegments) {
      child.geometry.dispose();
      if (child.material instanceof THREE.Material) child.material.dispose();
    }
    if (child instanceof THREE.Sprite) {
      child.material.map?.dispose();
      child.material.dispose();
    }
  }
}

// ---------------------------------------------------------------------------
// Surface color resolution
// ---------------------------------------------------------------------------

function getSurfaceColor(
  mode: RenderMode,
  isSelected: boolean,
  functionColor: number,
  assignedEntryId: string | undefined,
  catalogueUValues: Record<string, number>,
  isShared?: boolean,
): number {
  if (isSelected) return SELECTED_COLOR;

  if (mode === "uvalue") {
    if (!assignedEntryId) return UNASSIGNED_COLOR;
    const u = catalogueUValues[assignedEntryId];
    if (u === undefined) return UNASSIGNED_COLOR;
    // Shared (interior) walls: lighter shade
    if (isShared) return 0xb0e0e6; // light blue for interior
    return uValueToColor(u);
  }

  // Normal mode: use room function color
  return functionColor;
}

// ---------------------------------------------------------------------------
// Wall pieces (segments around openings) — flat surface version
// ---------------------------------------------------------------------------

interface WallPiece {
  t1: number; // 0..1 along wall
  t2: number;
  yBot: number; // m
  yTop: number; // m
}

function computeWallPieces(
  wallH: number,
  openings: { tStart: number; tEnd: number; sillH: number; headH: number }[],
): WallPiece[] {
  if (openings.length === 0) {
    return [{ t1: 0, t2: 1, yBot: 0, yTop: wallH }];
  }

  const pieces: WallPiece[] = [];
  let cursor = 0;

  for (const op of openings) {
    if (op.tStart > cursor + 0.001) {
      pieces.push({ t1: cursor, t2: op.tStart, yBot: 0, yTop: wallH });
    }
    if (op.sillH > 0.01) {
      pieces.push({ t1: op.tStart, t2: op.tEnd, yBot: 0, yTop: op.sillH });
    }
    if (op.headH < wallH - 0.01) {
      pieces.push({ t1: op.tStart, t2: op.tEnd, yBot: op.headH, yTop: wallH });
    }
    cursor = op.tEnd;
  }

  if (cursor < 1 - 0.001) {
    pieces.push({ t1: cursor, t2: 1, yBot: 0, yTop: wallH });
  }
  return pieces;
}

// ---------------------------------------------------------------------------
// Wall surface geometry (flat vertical quad)
// ---------------------------------------------------------------------------

function createWallSurfaceGeom(
  ax: number, az: number, bx: number, bz: number,
  piece: WallPiece,
): THREE.BufferGeometry {
  // Lerp along wall edge
  const x1 = ax + (bx - ax) * piece.t1;
  const z1 = az + (bz - az) * piece.t1;
  const x2 = ax + (bx - ax) * piece.t2;
  const z2 = az + (bz - az) * piece.t2;
  const yb = piece.yBot;
  const yt = piece.yTop;

  const positions = new Float32Array([
    x1, yb, z1,  // 0: start-bot
    x2, yb, z2,  // 1: end-bot
    x2, yt, z2,  // 2: end-top
    x1, yt, z1,  // 3: start-top
  ]);
  const indices = new Uint16Array([0, 1, 2, 0, 2, 3]);

  const geom = new THREE.BufferGeometry();
  geom.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geom.setIndex(new THREE.BufferAttribute(indices, 1));
  geom.computeVertexNormals();
  return geom;
}

// ---------------------------------------------------------------------------
// Window glass pane
// ---------------------------------------------------------------------------

function createPaneGeom(
  ax: number, az: number, bx: number, bz: number,
  edgeLenMm: number, win: ModelWindow, wallH: number,
): THREE.BufferGeometry | null {
  if (edgeLenMm < 1) return null;

  const tLeft = Math.max(0, (win.offset - win.width / 2) / edgeLenMm);
  const tRight = Math.min(1, (win.offset + win.width / 2) / edgeLenMm);
  const sill = WINDOW_SILL_H;
  const head = Math.min(WINDOW_HEAD_H, wallH);

  const lx = ax + (bx - ax) * tLeft;
  const lz = az + (bz - az) * tLeft;
  const rx = ax + (bx - ax) * tRight;
  const rz = az + (bz - az) * tRight;

  const positions = new Float32Array([
    lx, sill, lz,
    rx, sill, rz,
    rx, head, rz,
    lx, head, lz,
  ]);
  const indices = new Uint16Array([0, 1, 2, 0, 2, 3]);

  const geom = new THREE.BufferGeometry();
  geom.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geom.setIndex(new THREE.BufferAttribute(indices, 1));
  geom.computeVertexNormals();
  return geom;
}

// ---------------------------------------------------------------------------
// Room label sprite
// ---------------------------------------------------------------------------

function createLabelSprite(id: string, name: string, isSelected: boolean): THREE.Sprite {
  const canvas = document.createElement("canvas");
  canvas.width = 256;
  canvas.height = 96;
  const ctx = canvas.getContext("2d")!;

  ctx.clearRect(0, 0, 256, 96);

  ctx.fillStyle = isSelected ? "rgba(245, 158, 11, 0.9)" : "rgba(0, 0, 0, 0.65)";
  ctx.beginPath();
  ctx.roundRect(4, 4, 248, 88, 8);
  ctx.fill();

  ctx.fillStyle = "#ffffff";
  ctx.font = "bold 28px Inter, system-ui, sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(id, 128, 32);

  ctx.font = "22px Inter, system-ui, sans-serif";
  ctx.fillStyle = "rgba(255,255,255,0.85)";
  ctx.fillText(name, 128, 64);

  const texture = new THREE.CanvasTexture(canvas);
  const mat = new THREE.SpriteMaterial({ map: texture, transparent: true, depthTest: false });
  return new THREE.Sprite(mat);
}
