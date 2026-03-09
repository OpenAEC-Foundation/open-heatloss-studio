/**
 * 2D floor plan editor using Konva.js (react-konva).
 *
 * Supports: room drawing (rect/polygon/wall-polyline), wall/window/door selection,
 * dimension annotations, grid, snap, underlay rendering.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Stage, Layer, Group, Line, Rect, Text, Shape, Circle } from "react-konva";
import Konva from "konva";

import type { ModelRoom, ModelWindow, ModelDoor, ModellerTool, Point2D, SnapSettings, Selection, WallBoundaryType } from "./types";
import { pointInPolygon, polygonArea, polygonCenter, getSharedEdges, segmentsShareEdge } from "./geometry";
import type { UnderlayImage } from "./modellerStore";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface FloorCanvasProps {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
  selection: Selection;
  tool: ModellerTool;
  snap: SnapSettings;
  underlay: UnderlayImage | null;
  wallConstructions?: Record<string, string>;
  catalogueUValues?: Record<string, number>;
  onSelect: (sel: Selection) => void;
  onAddRoom: (polygon: Point2D[]) => void;
  onAddWindow: (roomId: string, wallIndex: number, offset: number, width: number) => void;
  onAddDoor: (roomId: string, wallIndex: number, offset: number, width: number) => void;
  onMoveRoom: (roomId: string, dx: number, dy: number) => void;
  onMoveVertex: (roomId: string, vertexIndex: number, x: number, y: number) => void;
  onUpdateWindow: (roomId: string, wallIndex: number, offset: number, updates: Partial<ModelWindow>) => void;
  onRemoveRoom?: (id: string) => void;
  onRemoveWindow?: (roomId: string, wallIndex: number, offset: number) => void;
  onSplitRoom?: (roomId: string, edgeA: number, tA: number, edgeB: number, tB: number, intermediatePoints?: Point2D[]) => void;
  onMergeRooms?: (roomIdA: string, wallA: number, roomIdB: string, wallB: number) => void;
  /** Wall boundary type overrides (key = "roomId:wallIndex"). */
  wallBoundaryTypes?: Record<string, WallBoundaryType>;
  /** Rooms from the floor below, rendered as ghost outlines. */
  ghostRooms?: ModelRoom[];
  /** Increment to trigger a fit-view zoom. */
  fitViewTrigger?: number;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const WALL_THICKNESS_MM = 200;
const DEFAULT_WINDOW_WIDTH = 1200;
const DEFAULT_DOOR_WIDTH = 900;
const MIN_WALL_PX = 3;

const FUNCTION_COLORS: Record<string, string> = {
  living_room: "#fef3c7",
  kitchen: "#fef9c3",
  bedroom: "#dbeafe",
  bathroom: "#cffafe",
  toilet: "#e0e7ff",
  hallway: "#f5f5f4",
  landing: "#f5f5f4",
  storage: "#e7e5e4",
  attic: "#fce7f3",
  custom: "#f3f4f6",
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function FloorCanvas({
  rooms,
  windows,
  doors,
  selection,
  tool,
  snap,
  underlay,
  wallConstructions = {},
  catalogueUValues = {},
  onSelect,
  onAddRoom,
  onAddWindow,
  onAddDoor,
  onMoveRoom,
  onMoveVertex,
  onUpdateWindow,
  onRemoveRoom,
  onRemoveWindow,
  onSplitRoom,
  onMergeRooms,
  wallBoundaryTypes = {},
  ghostRooms = [],
  fitViewTrigger = 0,
}: FloorCanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const stageRef = useRef<Konva.Stage>(null);
  const [size, setSize] = useState({ width: 800, height: 600 });
  const [viewCenter, setViewCenter] = useState<Point2D>({ x: 5000, y: 5000 });
  const [zoom, setZoom] = useState(0.07);

  // Drawing state
  const [drawPoints, setDrawPoints] = useState<Point2D[]>([]);
  const [cursorWorld, setCursorWorld] = useState<Point2D | null>(null);
  // Numeric length input (mm) while drawing
  const [numericInput, setNumericInput] = useState("");
  // Measure tool state
  const [measurePoints, setMeasurePoints] = useState<Point2D[]>([]);
  // Context menu
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);
  // Split room tool: stores first hit info between clicks
  const splitHitRef = useRef<{ roomId: string; wallIndex: number; offset: number }[] | null>(null);
  // Dimension edit overlay
  const [editingDim, setEditingDim] = useState<{ roomId: string; wallIndex: number; draft: string } | null>(null);

  // Panning
  const isPanningRef = useRef(false);
  const panStartRef = useRef<{ sx: number; sy: number; cx: number; cy: number }>({ sx: 0, sy: 0, cx: 0, cy: 0 });

  // Underlay image
  const [underlayImg, setUnderlayImg] = useState<HTMLImageElement | null>(null);
  useEffect(() => {
    if (!underlay) { setUnderlayImg(null); return; }
    const img = new Image();
    img.src = underlay.dataUrl;
    img.onload = () => setUnderlayImg(img);
  }, [underlay?.dataUrl]);

  // Group transform: world mm → screen px
  const groupX = size.width / 2 - viewCenter.x * zoom;
  const groupY = size.height / 2 - viewCenter.y * zoom;

  // Screen ↔ World conversion
  const screenToWorld = useCallback(
    (sx: number, sy: number): Point2D => ({
      x: (sx - size.width / 2) / zoom + viewCenter.x,
      y: (sy - size.height / 2) / zoom + viewCenter.y,
    }),
    [viewCenter, zoom, size],
  );

  // Snap — all geometry modes compete on distance, grid is fallback
  const applySnap = useCallback(
    (p: Point2D, forceNearest = false): Point2D => {
      // For tools like split_room: always snap to geometry even if snap is disabled
      const useNearest = forceNearest || snap.modes.includes("nearest");
      if (!snap.enabled && !forceNearest) return p;

      let best = p;
      let bestDist = Infinity;
      const tolerance = snap.gridSize * 2;

      // Snap targets: active rooms + ghost rooms from floor below
      const allSnapRooms = [...rooms, ...ghostRooms];

      // 1. Nearest (wall edge projection) — runs first to establish baseline
      //    Endpoints/midpoints can still win if they're closer.
      if (useNearest) {
        for (const room of allSnapRooms) {
          const poly = room.polygon;
          const n = poly.length;
          for (let i = 0; i < n; i++) {
            const a = poly[i]!;
            const b = poly[(i + 1) % n]!;
            const dx = b.x - a.x;
            const dy = b.y - a.y;
            const lenSq = dx * dx + dy * dy;
            if (lenSq < 1) continue;
            let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / lenSq;
            t = Math.max(0, Math.min(1, t));
            const proj = { x: a.x + t * dx, y: a.y + t * dy };
            const d = Math.hypot(p.x - proj.x, p.y - proj.y);
            if (d < bestDist && d < tolerance) { bestDist = d; best = proj; }
          }
        }
      }

      // 2. Endpoint snap (wins over nearest if closer)
      if (snap.modes.includes("endpoint")) {
        for (const room of allSnapRooms) {
          for (const v of room.polygon) {
            const d = Math.hypot(v.x - p.x, v.y - p.y);
            if (d < bestDist && d < tolerance) { bestDist = d; best = v; }
          }
        }
        for (const v of drawPoints) {
          const d = Math.hypot(v.x - p.x, v.y - p.y);
          if (d < bestDist && d < tolerance) { bestDist = d; best = v; }
        }
      }

      // 3. Midpoint snap (wins over nearest if closer)
      if (snap.modes.includes("midpoint")) {
        for (const room of allSnapRooms) {
          const poly = room.polygon;
          for (let i = 0; i < poly.length; i++) {
            const a = poly[i]!;
            const b = poly[(i + 1) % poly.length]!;
            const mid = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
            const d = Math.hypot(mid.x - p.x, mid.y - p.y);
            if (d < bestDist && d < tolerance) { bestDist = d; best = mid; }
          }
        }
      }

      // 4. Grid snap (lowest priority — only if no geometry matched)
      if (snap.modes.includes("grid") && bestDist === Infinity) {
        const gs = snap.gridSize;
        best = { x: Math.round(p.x / gs) * gs, y: Math.round(p.y / gs) * gs };
      }

      return best;
    },
    [snap, rooms, ghostRooms, drawPoints],
  );

  // Cancel drawing on tool change / Escape
  useEffect(() => {
    setDrawPoints([]); setCursorWorld(null); setMeasurePoints([]); setNumericInput("");
  }, [tool]); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      if (e.key === "Escape") {
        setDrawPoints([]);
        setNumericInput("");
        return;
      }

      // Numeric input: digits, period, Backspace, Enter
      if (isDrawingTool(tool) && drawPoints.length > 0) {
        if ((e.key >= "0" && e.key <= "9") || e.key === ".") {
          e.preventDefault();
          setNumericInput((prev) => prev + e.key);
          return;
        }
        if (e.key === "Backspace" && numericInput.length > 0) {
          e.preventDefault();
          setNumericInput((prev) => prev.slice(0, -1));
          return;
        }
        if (e.key === "Enter" && numericInput.length > 0 && cursorWorld) {
          e.preventDefault();
          const mm = parseFloat(numericInput) * 1000; // input in meters → mm
          if (!isNaN(mm) && mm > 0) {
            const lastPt = drawPoints[drawPoints.length - 1]!;
            const dx = cursorWorld.x - lastPt.x;
            const dy = cursorWorld.y - lastPt.y;
            const len = Math.hypot(dx, dy);
            if (len > 0.001) {
              const snapped = { x: lastPt.x + (dx / len) * mm, y: lastPt.y + (dy / len) * mm };
              setDrawPoints([...drawPoints, snapped]);
            }
          }
          setNumericInput("");
          return;
        }
      }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, [tool, drawPoints, numericInput, cursorWorld, selection]);

  // Resize observer
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      const { width, height } = entries[0]!.contentRect;
      setSize({ width: Math.floor(width), height: Math.floor(height) });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // --- Fit view ---
  useEffect(() => {
    if (fitViewTrigger === 0 || rooms.length === 0) return;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const room of rooms) {
      for (const p of room.polygon) {
        if (p.x < minX) minX = p.x;
        if (p.y < minY) minY = p.y;
        if (p.x > maxX) maxX = p.x;
        if (p.y > maxY) maxY = p.y;
      }
    }
    const margin = 2000; // 2m margin
    minX -= margin; minY -= margin; maxX += margin; maxY += margin;
    const bw = maxX - minX;
    const bh = maxY - minY;
    if (bw < 1 || bh < 1) return;
    const zx = size.width / bw;
    const zy = size.height / bh;
    const newZoom = Math.max(0.005, Math.min(0.5, Math.min(zx, zy)));
    setViewCenter({ x: (minX + maxX) / 2, y: (minY + maxY) / 2 });
    setZoom(newZoom);
  }, [fitViewTrigger]); // eslint-disable-line react-hooks/exhaustive-deps

  // --- Wheel zoom ---
  const handleWheel = useCallback((e: Konva.KonvaEventObject<WheelEvent>) => {
    e.evt.preventDefault();
    const stage = stageRef.current;
    if (!stage) return;

    const pointer = stage.getPointerPosition();
    if (!pointer) return;

    const oldZoom = zoom;
    const factor = e.evt.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.005, Math.min(0.5, oldZoom * factor));

    // Zoom toward cursor
    const wx = (pointer.x - size.width / 2) / oldZoom + viewCenter.x;
    const wy = (pointer.y - size.height / 2) / oldZoom + viewCenter.y;
    const wx2 = (pointer.x - size.width / 2) / newZoom + viewCenter.x;
    const wy2 = (pointer.y - size.height / 2) / newZoom + viewCenter.y;

    setViewCenter({ x: viewCenter.x + (wx - wx2), y: viewCenter.y + (wy - wy2) });
    setZoom(newZoom);
  }, [zoom, viewCenter, size]);

  // --- Mouse handlers ---
  const handleMouseDown = useCallback((e: Konva.KonvaEventObject<MouseEvent>) => {
    if (e.evt.button === 1 || e.evt.button === 2 || (tool === "pan" && e.evt.button === 0)) {
      isPanningRef.current = true;
      panStartRef.current = { sx: e.evt.clientX, sy: e.evt.clientY, cx: viewCenter.x, cy: viewCenter.y };
      e.evt.preventDefault();
    }
  }, [tool, viewCenter]);

  const handleMouseMove = useCallback((e: Konva.KonvaEventObject<MouseEvent>) => {
    if (isPanningRef.current) {
      const dx = (e.evt.clientX - panStartRef.current.sx) / zoom;
      const dy = (e.evt.clientY - panStartRef.current.sy) / zoom;
      setViewCenter({ x: panStartRef.current.cx - dx, y: panStartRef.current.cy - dy });
      return;
    }

    const stage = stageRef.current;
    if (!stage) return;
    const pointer = stage.getPointerPosition();
    if (!pointer) return;

    const raw = screenToWorld(pointer.x, pointer.y);
    const forceNearest = tool === "split_room" || tool === "draw_window" || tool === "draw_door";
    const snapped = applySnap(raw, forceNearest);
    if (isDrawingTool(tool) || tool === "measure") setCursorWorld(snapped);
    else setCursorWorld(null);
  }, [zoom, screenToWorld, applySnap, tool]);

  const handleMouseUp = useCallback(() => { isPanningRef.current = false; }, []);

  // --- Stage click (background) ---
  const handleStageClick = useCallback((e: Konva.KonvaEventObject<MouseEvent>) => {
    setCtxMenu(null);
    if (isPanningRef.current || e.evt.button !== 0) return;

    const stage = stageRef.current;
    if (!stage) return;
    const pointer = stage.getPointerPosition();
    if (!pointer) return;
    const raw = screenToWorld(pointer.x, pointer.y);
    const forceNearest = tool === "split_room" || tool === "draw_window" || tool === "draw_door";
    const snapped = applySnap(raw, forceNearest);

    // Drawing tools
    if (tool === "draw_rect") {
      if (drawPoints.length === 0) {
        setDrawPoints([snapped]);
      } else {
        const p0 = drawPoints[0]!;
        if (Math.abs(snapped.x - p0.x) > 100 && Math.abs(snapped.y - p0.y) > 100) {
          onAddRoom([
            { x: p0.x, y: p0.y }, { x: snapped.x, y: p0.y },
            { x: snapped.x, y: snapped.y }, { x: p0.x, y: snapped.y },
          ]);
        }
        setDrawPoints([]);
      }
      return;
    }

    if (tool === "draw_polygon") {
      if (drawPoints.length >= 3) {
        const first = drawPoints[0]!;
        if (Math.hypot(snapped.x - first.x, snapped.y - first.y) < snap.gridSize * 1.5) {
          onAddRoom([...drawPoints]);
          setDrawPoints([]);
          return;
        }
      }
      setDrawPoints([...drawPoints, snapped]);
      return;
    }

    if (tool === "draw_window") {
      const hit = findWallHit(snapped, rooms, snap.gridSize * 3);
      if (hit) onAddWindow(hit.roomId, hit.wallIndex, hit.offset, DEFAULT_WINDOW_WIDTH);
      return;
    }

    if (tool === "draw_door") {
      const hit = findWallHit(snapped, rooms, snap.gridSize * 3);
      if (hit) onAddDoor(hit.roomId, hit.wallIndex, hit.offset, DEFAULT_DOOR_WIDTH);
      return;
    }

    // Split room tool: click wall → optional intermediate points → click wall
    if (tool === "split_room") {
      const firstHit = splitHitRef.current?.[0];

      // After first click: search ONLY the target room so shared inner walls
      // always resolve to the correct room (not the adjacent neighbour).
      const searchRooms = firstHit
        ? rooms.filter((r) => r.id === firstHit.roomId)
        : rooms;
      const hit = findWallHit(snapped, searchRooms, snap.gridSize * 3)
        ?? findWallHit(raw, searchRooms, snap.gridSize * 3);

      if (drawPoints.length === 0) {
        // First click: must be on a wall
        if (!hit) return;
        splitHitRef.current = [hit];
        const room = rooms.find((r) => r.id === hit.roomId);
        if (room) {
          const a = room.polygon[hit.wallIndex]!;
          const b = room.polygon[(hit.wallIndex + 1) % room.polygon.length]!;
          const len = Math.hypot(b.x - a.x, b.y - a.y);
          const t = len > 0 ? hit.offset / len : 0;
          setDrawPoints([{ x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t }]);
        }
      } else if (hit && hit.wallIndex !== firstHit!.wallIndex) {
        // Click on a different wall of the target room → execute split
        const room = rooms.find((r) => r.id === hit.roomId);
        if (room) {
          const polyLen = room.polygon.length;
          const a1 = room.polygon[firstHit!.wallIndex]!;
          const b1 = room.polygon[(firstHit!.wallIndex + 1) % polyLen]!;
          const len1 = Math.hypot(b1.x - a1.x, b1.y - a1.y);
          const tA = len1 > 0 ? firstHit!.offset / len1 : 0;

          const a2 = room.polygon[hit.wallIndex]!;
          const b2 = room.polygon[(hit.wallIndex + 1) % polyLen]!;
          const len2 = Math.hypot(b2.x - a2.x, b2.y - a2.y);
          const tB = len2 > 0 ? hit.offset / len2 : 0;

          // Intermediate points = drawPoints[1..n-1] (skip first wall point)
          const intermediatePoints = drawPoints.slice(1);
          onSplitRoom?.(hit.roomId, firstHit!.wallIndex, tA, hit.wallIndex, tB, intermediatePoints);
        }
        setDrawPoints([]);
        splitHitRef.current = null;
      } else {
        // Not on a wall (or same wall) → add as intermediate polyline point
        setDrawPoints([...drawPoints, snapped]);
      }
      return;
    }

    // Measure tool: 2 clicks, then click again to restart
    if (tool === "measure") {
      if (measurePoints.length === 0 || measurePoints.length === 2) {
        setMeasurePoints([snapped]);
      } else {
        setMeasurePoints([measurePoints[0]!, snapped]);
      }
      return;
    }

    // Circle tool: click center, click edge
    if (tool === "draw_circle") {
      if (drawPoints.length === 0) {
        setDrawPoints([snapped]);
      } else {
        const center = drawPoints[0]!;
        const radius = Math.hypot(snapped.x - center.x, snapped.y - center.y);
        if (radius > 100) {
          // Approximate circle as 24-sided polygon
          const sides = 24;
          const poly: Point2D[] = [];
          for (let i = 0; i < sides; i++) {
            const angle = (i / sides) * Math.PI * 2;
            poly.push({
              x: Math.round(center.x + radius * Math.cos(angle)),
              y: Math.round(center.y + radius * Math.sin(angle)),
            });
          }
          onAddRoom(poly);
        }
        setDrawPoints([]);
      }
      return;
    }

    // Select tool: click on empty → deselect
    if (tool === "select") {
      // Check if clicked on a room
      for (let i = rooms.length - 1; i >= 0; i--) {
        if (pointInPolygon(raw, rooms[i]!.polygon)) {
          onSelect({ type: "room", roomId: rooms[i]!.id });
          return;
        }
      }
      onSelect(null);
    }
  }, [tool, drawPoints, rooms, screenToWorld, applySnap, snap.gridSize, onAddRoom, onAddWindow, onAddDoor, onSelect, onSplitRoom]);

  const handleDblClick = useCallback(() => {
    if (tool === "draw_polygon" && drawPoints.length >= 3) {
      onAddRoom([...drawPoints]);
      setDrawPoints([]);
    }
  }, [tool, drawPoints, onAddRoom]);

  // Wall thickness in mm, with minimum pixel width
  const wallStroke = Math.max(WALL_THICKNESS_MM, MIN_WALL_PX / zoom);

  // Inverse zoom for fixed-size screen elements
  const invZoom = 1 / zoom;

  // Shared edges between rooms (interior walls — rendered as thin lines)
  const sharedEdges = useMemo(() => getSharedEdges(rooms), [rooms]);

  // Selected room ID (for highlighting)
  const selectedRoomId = selection?.type === "room" ? selection.roomId
    : selection?.type === "wall" ? selection.roomId
    : selection?.type === "window" ? selection.roomId
    : null;

  const cursor = tool === "pan"
    ? (isPanningRef.current ? "grabbing" : "grab")
    : isDrawingTool(tool) ? "crosshair" : "default";

  return (
    <div ref={containerRef} className="relative h-full w-full overflow-hidden bg-stone-50" style={{ cursor }}>
      <Stage
        ref={stageRef}
        width={size.width}
        height={size.height}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onClick={handleStageClick}
        onDblClick={handleDblClick}
        onContextMenu={(e) => {
          e.evt.preventDefault();
          setCtxMenu({ x: e.evt.clientX, y: e.evt.clientY });
        }}
      >
        {/* Grid layer (screen coords) */}
        <Layer listening={false}>
          <GridShape width={size.width} height={size.height} viewCenter={viewCenter} zoom={zoom} />
        </Layer>

        {/* World-coordinate layer */}
        <Layer>
          <Group x={groupX} y={groupY} scaleX={zoom} scaleY={zoom}>
            {/* Underlay */}
            {underlay && underlayImg && (
              <UnderlayShape ul={underlay} img={underlayImg} />
            )}

            {/* Ghost rooms from floor below */}
            {ghostRooms.map((room) => {
              const flatPts = room.polygon.flatMap((p) => [p.x, p.y]);
              return (
                <Group key={`ghost-${room.id}`} listening={false}>
                  <Line points={flatPts} closed fill="#f5f5f4" opacity={0.3} />
                  {room.polygon.map((_, gi) => {
                    const ni = (gi + 1) % room.polygon.length;
                    const a = room.polygon[gi]!;
                    const b = room.polygon[ni]!;
                    return (
                      <Line
                        key={`ghost-wall-${room.id}-${gi}`}
                        points={[a.x, a.y, b.x, b.y]}
                        stroke="#d6d3d1"
                        strokeWidth={Math.max(40, 1 / zoom)}
                        dash={[200, 150]}
                      />
                    );
                  })}
                  <Text
                    x={polygonCenter(room.polygon).x}
                    y={polygonCenter(room.polygon).y}
                    text={room.name}
                    fontSize={9 * invZoom}
                    fontFamily="Inter, system-ui, sans-serif"
                    fill="#d6d3d1"
                    align="center"
                    offsetX={30 * invZoom}
                    width={60 * invZoom}
                    listening={false}
                  />
                </Group>
              );
            })}

            {/* Room fills */}
            {rooms.map((room) => (
              <RoomFill
                key={`fill-${room.id}`}
                room={room}
                isSelected={room.id === selectedRoomId}
                tool={tool}
                onSelect={() => onSelect({ type: "room", roomId: room.id })}
                onDragEnd={(dx, dy) => onMoveRoom(room.id, dx, dy)}
              />
            ))}

            {/* Room edges — style depends on boundary type override or auto-detection */}
            {rooms.map((room) =>
              room.polygon.map((_, wi) => {
                const ni = (wi + 1) % room.polygon.length;
                const isWallSelected = selection?.type === "wall"
                  && selection.roomId === room.id && selection.wallIndex === wi;
                const isShared = sharedEdges.has(`${room.id}:${wi}`);
                const a = room.polygon[wi]!;
                const b = room.polygon[ni]!;
                const boundaryKey = `${room.id}:${wi}`;
                const boundary = wallBoundaryTypes[boundaryKey] ?? "auto";

                // Determine if this wall renders as "interior" (thin/light) or "exterior" (thick/dark)
                const isInteriorStyle = boundary === "auto"
                  ? isShared
                  : boundary === "interior" || boundary === "neighbor" || boundary === "unheated";

                const wallColor = isWallSelected ? "#d97706"
                  : isInteriorStyle ? "#d6d3d1"
                  : "#1c1917";

                return (
                  <Line
                    key={`wall-${room.id}-${wi}`}
                    points={[a.x, a.y, b.x, b.y]}
                    stroke={wallColor}
                    strokeWidth={isInteriorStyle ? Math.max(40, 1 / zoom) : Math.max(80, 2 / zoom)}
                    hitStrokeWidth={Math.max(WALL_THICKNESS_MM, 400)}
                    onClick={(e) => {
                      if (tool === "select") { e.cancelBubble = true; onSelect({ type: "wall", roomId: room.id, wallIndex: wi }); }
                    }}
                  />
                );
              }),
            )}

            {/* Windows */}
            {windows.map((win) => {
              const room = rooms.find((r) => r.id === win.roomId);
              if (!room) return null;
              const isWinSelected = selection?.type === "window"
                && selection.roomId === win.roomId
                && selection.wallIndex === win.wallIndex
                && Math.abs(selection.offset - win.offset) < 1;
              return (
                <WindowMarker
                  key={`win-${win.roomId}-${win.wallIndex}-${win.offset}`}
                  room={room}
                  win={win}
                  strokeWidth={wallStroke * 0.85}
                  isSelected={isWinSelected}
                  tool={tool}
                  zoom={zoom}
                  onSelect={() => onSelect({ type: "window", roomId: win.roomId, wallIndex: win.wallIndex, offset: win.offset })}
                  onDragAlongWall={(newOffset) => onUpdateWindow(win.roomId, win.wallIndex, win.offset, { offset: newOffset })}
                />
              );
            })}

            {/* Doors */}
            {doors.map((door) => {
              const room = rooms.find((r) => r.id === door.roomId);
              if (!room) return null;
              return (
                <DoorMarker
                  key={`door-${door.roomId}-${door.wallIndex}-${door.offset}`}
                  room={room}
                  door={door}
                  strokeWidth={wallStroke * 0.85}
                  zoom={zoom}
                />
              );
            })}

            {/* Room labels */}
            {rooms.map((room) => (
              <RoomLabel key={`label-${room.id}`} room={room} invZoom={invZoom} isSelected={room.id === selectedRoomId} />
            ))}

            {/* U-value labels on walls */}
            {rooms.map((room) =>
              room.polygon.map((_, wi) => {
                const key = `${room.id}:${wi}`;
                const conId = wallConstructions[key];
                const uVal = conId ? catalogueUValues[conId] : undefined;
                if (uVal === undefined) return null;
                const a = room.polygon[wi]!;
                const b = room.polygon[(wi + 1) % room.polygon.length]!;
                const mx = (a.x + b.x) / 2;
                const my = (a.y + b.y) / 2;
                const angle = Math.atan2(b.y - a.y, b.x - a.x);
                const off = 28 * invZoom;
                return (
                  <Text
                    key={`u-${room.id}-${wi}`}
                    x={mx + Math.cos(angle - Math.PI / 2) * off}
                    y={my + Math.sin(angle - Math.PI / 2) * off}
                    text={`U=${uVal.toFixed(2)}`}
                    fontSize={9 * invZoom}
                    fontFamily="Inter, system-ui, sans-serif"
                    fill="#6366f1"
                    align="center"
                    offsetX={22 * invZoom}
                    offsetY={5 * invZoom}
                    width={44 * invZoom}
                    listening={false}
                  />
                );
              }),
            )}

            {/* Dimension annotations on selected room */}
            {selectedRoomId && (() => {
              const sel = rooms.find((r) => r.id === selectedRoomId);
              return sel ? <DimensionAnnotations room={sel} invZoom={invZoom} onSelectWall={(wallIndex) => onSelect({ type: "wall", roomId: selectedRoomId, wallIndex })} onStartEdit={(wallIndex) => {
                const r = rooms.find((r) => r.id === selectedRoomId);
                if (!r) return;
                const a = r.polygon[wallIndex]!;
                const b = r.polygon[(wallIndex + 1) % r.polygon.length]!;
                const len = Math.hypot(b.x - a.x, b.y - a.y);
                setEditingDim({ roomId: selectedRoomId, wallIndex, draft: (len / 1000).toFixed(2) });
              }} /> : null;
            })()}

            {/* Vertex grips on selected room */}
            {selectedRoomId && tool === "select" && (() => {
              const sel = rooms.find((r) => r.id === selectedRoomId);
              if (!sel) return null;
              return sel.polygon.map((v, vi) => (
                <Circle
                  key={`grip-${vi}`}
                  x={v.x}
                  y={v.y}
                  radius={6 * invZoom}
                  fill="#ffffff"
                  stroke="#d97706"
                  strokeWidth={2 * invZoom}
                  draggable
                  hitStrokeWidth={10 * invZoom}
                  onDragEnd={(e) => {
                    const nx = e.target.x();
                    const ny = e.target.y();
                    e.target.position({ x: 0, y: 0 });
                    const snapped = applySnap({ x: nx, y: ny });
                    onMoveVertex(sel.id, vi, snapped.x, snapped.y);
                  }}
                />
              ));
            })()}

            {/* Measure result */}
            {measurePoints.length === 2 && (() => {
              const [mp0, mp1] = measurePoints as [Point2D, Point2D];
              const dist = Math.hypot(mp1.x - mp0.x, mp1.y - mp0.y);
              const mx = (mp0.x + mp1.x) / 2;
              const my = (mp0.y + mp1.y) / 2;
              return (
                <Group listening={false}>
                  <Line points={[mp0.x, mp0.y, mp1.x, mp1.y]} stroke="#ef4444" strokeWidth={2 * invZoom} dash={[8 * invZoom, 4 * invZoom]} />
                  <Circle x={mp0.x} y={mp0.y} radius={4 * invZoom} fill="#ef4444" />
                  <Circle x={mp1.x} y={mp1.y} radius={4 * invZoom} fill="#ef4444" />
                  <Text
                    x={mx}
                    y={my - 18 * invZoom}
                    text={`${(dist / 1000).toFixed(3)} m`}
                    fontSize={12 * invZoom}
                    fontStyle="bold"
                    fontFamily="Inter, system-ui, sans-serif"
                    fill="#ef4444"
                    align="center"
                    offsetX={35 * invZoom}
                    width={70 * invZoom}
                  />
                </Group>
              );
            })()}

            {/* Measure preview (1 point placed, cursor moving) */}
            {measurePoints.length === 1 && cursorWorld && (() => {
              const mp0 = measurePoints[0]!;
              const dist = Math.hypot(cursorWorld.x - mp0.x, cursorWorld.y - mp0.y);
              const mx = (mp0.x + cursorWorld.x) / 2;
              const my = (mp0.y + cursorWorld.y) / 2;
              return (
                <Group listening={false}>
                  <Line points={[mp0.x, mp0.y, cursorWorld.x, cursorWorld.y]} stroke="#ef4444" strokeWidth={1.5 * invZoom} dash={[6 * invZoom, 4 * invZoom]} opacity={0.6} />
                  <Circle x={mp0.x} y={mp0.y} radius={4 * invZoom} fill="#ef4444" />
                  <Text
                    x={mx}
                    y={my - 18 * invZoom}
                    text={`${(dist / 1000).toFixed(3)} m`}
                    fontSize={11 * invZoom}
                    fontStyle="bold"
                    fontFamily="Inter, system-ui, sans-serif"
                    fill="#ef4444"
                    align="center"
                    offsetX={35 * invZoom}
                    width={70 * invZoom}
                    opacity={0.7}
                  />
                </Group>
              );
            })()}

            {/* Drawing preview */}
            <DrawPreview tool={tool} points={drawPoints} cursor={cursorWorld} invZoom={invZoom} snapGridSize={snap.gridSize} numericInput={numericInput} />

            {/* Split room preview — polyline from first wall point through intermediates to cursor */}
            {tool === "split_room" && drawPoints.length >= 1 && cursorWorld && (() => {
              const allPts = drawPoints;

              // Determine end point: snap to wall if near, otherwise follow cursor
              let endPt = cursorWorld;
              let onWall = false;
              const firstHitRef = splitHitRef.current?.[0];
              const targetRooms = firstHitRef
                ? rooms.filter((r) => r.id === firstHitRef.roomId)
                : rooms;
              const hit = findWallHit(cursorWorld, targetRooms, snap.gridSize * 3);
              if (hit && firstHitRef && hit.wallIndex !== firstHitRef.wallIndex) {
                const room = rooms.find((r) => r.id === hit.roomId);
                if (room) {
                  const a = room.polygon[hit.wallIndex]!;
                  const b = room.polygon[(hit.wallIndex + 1) % room.polygon.length]!;
                  const len = Math.hypot(b.x - a.x, b.y - a.y);
                  const t = len > 0 ? hit.offset / len : 0;
                  endPt = { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t };
                  onWall = true;
                }
              }

              // Build flat points array for the full polyline
              const linePts: number[] = [];
              for (const pt of allPts) {
                linePts.push(pt.x, pt.y);
              }
              linePts.push(endPt.x, endPt.y);

              return (
                <Group listening={false}>
                  <Line
                    points={linePts}
                    stroke="#ef4444"
                    strokeWidth={(onWall ? 2 : 1.5) * invZoom}
                    dash={[8 * invZoom, 4 * invZoom]}
                    opacity={onWall ? 1 : 0.5}
                  />
                  {/* Dots on each placed point */}
                  {allPts.map((pt, i) => (
                    <Circle key={i} x={pt.x} y={pt.y} radius={4 * invZoom} fill={i === 0 ? "#ef4444" : "#f97316"} />
                  ))}
                  {/* End dot — highlight when on wall */}
                  <Circle
                    x={endPt.x} y={endPt.y} radius={5 * invZoom}
                    fill={onWall ? "#ef4444" : "#f9731680"}
                    stroke={onWall ? "#ffffff" : undefined}
                    strokeWidth={onWall ? 1.5 * invZoom : 0}
                  />
                </Group>
              );
            })()}

            {/* Circle preview (center placed, sizing with cursor) */}
            {tool === "draw_circle" && drawPoints.length === 1 && cursorWorld && (() => {
              const center = drawPoints[0]!;
              const radius = Math.hypot(cursorWorld.x - center.x, cursorWorld.y - center.y);
              const sides = 48;
              const circlePts: number[] = [];
              for (let i = 0; i <= sides; i++) {
                const angle = (i / sides) * Math.PI * 2;
                circlePts.push(center.x + radius * Math.cos(angle), center.y + radius * Math.sin(angle));
              }
              return (
                <Group listening={false}>
                  <Line points={circlePts} stroke="#d97706" strokeWidth={2 * invZoom} dash={[6 * invZoom, 4 * invZoom]} />
                  <Circle x={center.x} y={center.y} radius={4 * invZoom} fill="#d97706" />
                  <Line points={[center.x, center.y, cursorWorld.x, cursorWorld.y]} stroke="#d97706" strokeWidth={invZoom} dash={[4 * invZoom, 3 * invZoom]} opacity={0.5} />
                  <Text
                    x={(center.x + cursorWorld.x) / 2}
                    y={(center.y + cursorWorld.y) / 2 - 16 * invZoom}
                    text={`r=${(radius / 1000).toFixed(2)} m`}
                    fontSize={10 * invZoom}
                    fontStyle="bold"
                    fontFamily="Inter, system-ui, sans-serif"
                    fill="#d97706"
                    align="center"
                    offsetX={30 * invZoom}
                    width={60 * invZoom}
                  />
                </Group>
              );
            })()}

            {/* Snap cursor */}
            {cursorWorld && (
              <Group x={cursorWorld.x} y={cursorWorld.y}>
                <Line points={[-12 * invZoom, 0, 12 * invZoom, 0]} stroke="#d97706" strokeWidth={invZoom} opacity={0.6} />
                <Line points={[0, -12 * invZoom, 0, 12 * invZoom]} stroke="#d97706" strokeWidth={invZoom} opacity={0.6} />
                <Circle radius={3 * invZoom} fill="#d97706" />
              </Group>
            )}
          </Group>
        </Layer>

        {/* Overlay layer (screen coords) */}
        <Layer listening={false}>
          <ScaleBarShape width={size.width} height={size.height} zoom={zoom} />
          {snap.enabled && (
            <SnapBadge width={size.width} count={snap.modes.length} />
          )}
          {/* Drawing hint */}
          {isDrawingTool(tool) && (
            <Text
              x={size.width / 2}
              y={size.height - 40}
              text={getDrawingHint(tool, drawPoints.length)}
              fontSize={11}
              fill="white"
              align="center"
              offsetX={120}
              width={240}
              padding={6}
              cornerRadius={4}
              // Background via rect behind it
            />
          )}
        </Layer>
      </Stage>

      {/* Drawing / measure hint overlay (HTML for better styling) */}
      {(isDrawingTool(tool) || tool === "measure") && (
        <div className="pointer-events-none absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-1">
          {/* Numeric input display */}
          {numericInput && (
            <div className="rounded bg-emerald-600/90 px-3 py-1 font-mono text-sm text-white">
              {numericInput} m <span className="text-emerald-200">Enter</span>
            </div>
          )}
          <div className="rounded bg-black/70 px-3 py-1.5 text-[11px] text-white">
            {tool === "measure" ? getMeasureHint(measurePoints.length) : getDrawingHint(tool, drawPoints.length)}
          </div>
        </div>
      )}

      {/* Scale ratio */}
      <div className="pointer-events-none absolute right-3 top-3 rounded bg-black/60 px-2 py-1 font-mono text-[10px] text-white">
        1:{Math.round(1000 / (zoom * 1000))}
      </div>

      {/* Dimension edit overlay */}
      {editingDim && (() => {
        const room = rooms.find((r) => r.id === editingDim.roomId);
        if (!room) return null;
        const a = room.polygon[editingDim.wallIndex]!;
        const b = room.polygon[(editingDim.wallIndex + 1) % room.polygon.length]!;
        const mx = (a.x + b.x) / 2;
        const my = (a.y + b.y) / 2;
        // Convert world to screen
        const sx = (mx - viewCenter.x) * zoom + size.width / 2;
        const sy = (my - viewCenter.y) * zoom + size.height / 2;
        return (
          <div className="absolute z-30" style={{ left: sx - 40, top: sy - 14 }}>
            <input
              autoFocus
              value={editingDim.draft}
              onChange={(e) => setEditingDim({ ...editingDim, draft: e.target.value })}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  const newLen = parseFloat(editingDim.draft) * 1000;
                  if (!isNaN(newLen) && newLen > 100 && room) {
                    const oldA = room.polygon[editingDim.wallIndex]!;
                    const oldB = room.polygon[(editingDim.wallIndex + 1) % room.polygon.length]!;
                    const dx = oldB.x - oldA.x;
                    const dy = oldB.y - oldA.y;
                    const oldLen = Math.hypot(dx, dy);
                    if (oldLen > 0) {
                      const ux = dx / oldLen;
                      const uy = dy / oldLen;
                      const newB = { x: Math.round(oldA.x + ux * newLen), y: Math.round(oldA.y + uy * newLen) };
                      onMoveVertex(editingDim.roomId, (editingDim.wallIndex + 1) % room.polygon.length, newB.x, newB.y);
                    }
                  }
                  setEditingDim(null);
                }
                if (e.key === "Escape") setEditingDim(null);
              }}
              onBlur={() => setEditingDim(null)}
              className="w-20 rounded border border-amber-400 bg-amber-50 px-1.5 py-0.5 text-center text-xs font-mono font-bold text-amber-800 outline-none shadow-lg"
            />
          </div>
        );
      })()}

      {/* Right-click context menu */}
      {ctxMenu && (
        <div
          className="fixed z-50 min-w-[160px] rounded-lg bg-white/95 py-1 shadow-xl backdrop-blur-sm text-xs"
          style={{ left: ctxMenu.x, top: ctxMenu.y }}
          onClick={() => setCtxMenu(null)}
        >
          {selection?.type === "room" && (
            <button
              className="w-full px-3 py-1.5 text-left hover:bg-stone-100 text-stone-700"
              onClick={() => { onRemoveRoom?.(selection.roomId); setCtxMenu(null); }}
            >
              Verwijder ruimte
            </button>
          )}
          {selection?.type === "window" && (
            <button
              className="w-full px-3 py-1.5 text-left hover:bg-stone-100 text-stone-700"
              onClick={() => { onRemoveWindow?.(selection.roomId, selection.wallIndex, selection.offset); setCtxMenu(null); }}
            >
              Verwijder raam
            </button>
          )}
          {selection?.type === "wall" && (() => {
            const isShared = sharedEdges.has(`${selection.roomId}:${selection.wallIndex}`);
            const partner = isShared
              ? sharedEdgePartner(selection.roomId, selection.wallIndex, rooms, sharedEdges)
              : null;
            return (
              <>
                {partner && (
                  <button
                    className="w-full px-3 py-1.5 text-left hover:bg-stone-100 text-stone-700"
                    onClick={() => {
                      onMergeRooms?.(selection.roomId, selection.wallIndex, partner.roomId, partner.wallIndex);
                      setCtxMenu(null);
                    }}
                  >
                    Wand verwijderen (samenvoegen)
                  </button>
                )}
                <button
                  className="w-full px-3 py-1.5 text-left hover:bg-stone-100 text-stone-700"
                  onClick={() => { onRemoveRoom?.(selection.roomId); setCtxMenu(null); }}
                >
                  Verwijder ruimte
                </button>
              </>
            );
          })()}
          {!selection && (
            <div className="px-3 py-1.5 text-stone-400 italic">Geen selectie</div>
          )}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// Sub-components
// =============================================================================

/** Custom Konva shape for the grid (drawn in screen coords). */
function GridShape({ width, height, viewCenter, zoom }: { width: number; height: number; viewCenter: Point2D; zoom: number }) {
  return (
    <Shape
      sceneFunc={(ctx, shape) => {
        const pxPerM = zoom * 1000;
        let minor: number, major: number;
        if (pxPerM > 150) { minor = 100; major = 1000; }
        else if (pxPerM > 40) { minor = 500; major = 1000; }
        else if (pxPerM > 15) { minor = 1000; major = 5000; }
        else { minor = 5000; major = 10000; }

        const wL = viewCenter.x - width / (2 * zoom);
        const wT = viewCenter.y - height / (2 * zoom);
        const wR = viewCenter.x + width / (2 * zoom);
        const wB = viewCenter.y + height / (2 * zoom);

        // Minor
        ctx.strokeStyle = "#e7e5e4";
        ctx.lineWidth = 0.5;
        ctx.beginPath();
        for (let wx = Math.floor(wL / minor) * minor; wx <= wR; wx += minor) {
          const sx = (wx - viewCenter.x) * zoom + width / 2;
          ctx.moveTo(sx, 0); ctx.lineTo(sx, height);
        }
        for (let wy = Math.floor(wT / minor) * minor; wy <= wB; wy += minor) {
          const sy = (wy - viewCenter.y) * zoom + height / 2;
          ctx.moveTo(0, sy); ctx.lineTo(width, sy);
        }
        ctx.stroke();

        // Major
        ctx.strokeStyle = "#d6d3d1";
        ctx.lineWidth = 1;
        ctx.beginPath();
        for (let wx = Math.floor(wL / major) * major; wx <= wR; wx += major) {
          const sx = (wx - viewCenter.x) * zoom + width / 2;
          ctx.moveTo(sx, 0); ctx.lineTo(sx, height);
        }
        for (let wy = Math.floor(wT / major) * major; wy <= wB; wy += major) {
          const sy = (wy - viewCenter.y) * zoom + height / 2;
          ctx.moveTo(0, sy); ctx.lineTo(width, sy);
        }
        ctx.stroke();

        // Axis labels
        if (pxPerM > 20) {
          ctx.fillStyle = "#a8a29e";
          ctx.font = "10px Inter, system-ui, sans-serif";
          ctx.textAlign = "left";
          ctx.textBaseline = "top";
          for (let wx = Math.floor(wL / major) * major; wx <= wR; wx += major) {
            const sx = (wx - viewCenter.x) * zoom + width / 2;
            if (sx > 5 && sx < width - 30) ctx.fillText(`${(wx / 1000).toFixed(0)}m`, sx + 3, 3);
          }
        }

        ctx.fillStrokeShape(shape);
      }}
    />
  );
}

/** Room floor fill polygon. Draggable in select mode. */
function RoomFill({ room, isSelected, tool, onSelect, onDragEnd }: {
  room: ModelRoom; isSelected: boolean; tool: ModellerTool;
  onSelect: () => void; onDragEnd: (dx: number, dy: number) => void;
}) {
  const flatPts = useMemo(() => room.polygon.flatMap((p) => [p.x, p.y]), [room.polygon]);
  const color = isSelected ? "#fef3c7" : (FUNCTION_COLORS[room.function] ?? "#f3f4f6");

  return (
    <Line
      points={flatPts}
      closed
      fill={color}
      opacity={0.9}
      hitStrokeWidth={0}
      draggable={tool === "select"}
      onClick={(e) => { e.cancelBubble = true; onSelect(); }}
      onDragEnd={(e) => {
        const dx = e.target.x();
        const dy = e.target.y();
        e.target.position({ x: 0, y: 0 }); // reset visual position
        if (Math.abs(dx) > 1 || Math.abs(dy) > 1) {
          onDragEnd(dx, dy);
        }
      }}
    />
  );
}


/** Window marker on a wall. Draggable along wall when selected. */
function WindowMarker({ room, win, strokeWidth, isSelected, tool, onSelect, onDragAlongWall }: {
  room: ModelRoom; win: ModelWindow; strokeWidth: number; isSelected: boolean; tool: ModellerTool; zoom: number;
  onSelect: () => void; onDragAlongWall: (newOffset: number) => void;
}) {
  const poly = room.polygon;
  const n = poly.length;
  const i = win.wallIndex % n;
  const a = poly[i]!;
  const b = poly[(i + 1) % n]!;

  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.sqrt(dx * dx + dy * dy);
  if (len < 1) return null;
  const ux = dx / len;
  const uy = dy / len;

  const cx = a.x + ux * win.offset;
  const cy = a.y + uy * win.offset;
  const hw = win.width / 2;

  const p1x = cx - ux * hw;
  const p1y = cy - uy * hw;
  const p2x = cx + ux * hw;
  const p2y = cy + uy * hw;

  return (
    <Line
      points={[p1x, p1y, p2x, p2y]}
      stroke={isSelected ? "#d97706" : "#3b82f6"}
      strokeWidth={strokeWidth}
      lineCap="butt"
      hitStrokeWidth={Math.max(strokeWidth, 400)}
      draggable={tool === "select" && isSelected}
      onClick={(e) => {
        if (tool === "select") { e.cancelBubble = true; onSelect(); }
      }}
      onDragEnd={(e) => {
        // Project drag position back onto wall
        const dragX = e.target.x();
        const dragY = e.target.y();
        e.target.position({ x: 0, y: 0 });

        const newOffset = win.offset + dragX * ux + dragY * uy;
        const clampedOffset = Math.max(hw, Math.min(len - hw, newOffset));
        onDragAlongWall(clampedOffset);
      }}
    />
  );
}

/** Door marker with swing arc. */
function DoorMarker({ room, door, strokeWidth, zoom }: {
  room: ModelRoom; door: ModelDoor; strokeWidth: number; zoom: number;
}) {
  const poly = room.polygon;
  const n = poly.length;
  const i = door.wallIndex % n;
  const a = poly[i]!;
  const b = poly[(i + 1) % n]!;

  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.sqrt(dx * dx + dy * dy);
  if (len < 1) return null;
  const ux = dx / len;
  const uy = dy / len;
  // Normal (inward)
  const nx = dy / len;
  const ny = -dx / len;

  const cx = a.x + ux * door.offset;
  const cy = a.y + uy * door.offset;
  const hw = door.width / 2;

  const p1x = cx - ux * hw;
  const p1y = cy - uy * hw;
  const p2x = cx + ux * hw;
  const p2y = cy + uy * hw;

  // Door opening line
  const hingeX = door.swing === "left" ? p1x : p2x;
  const hingeY = door.swing === "left" ? p1y : p2y;
  const endX = hingeX + nx * door.width;
  const endY = hingeY + ny * door.width;

  return (
    <Group>
      {/* Door opening on wall */}
      <Line
        points={[p1x, p1y, p2x, p2y]}
        stroke="#059669"
        strokeWidth={strokeWidth}
        lineCap="butt"
      />
      {/* Swing line */}
      <Line
        points={[hingeX, hingeY, endX, endY]}
        stroke="#059669"
        strokeWidth={Math.max(1 / zoom, 30)}
        dash={[100, 80]}
      />
      {/* Arc */}
      <Shape
        sceneFunc={(ctx, shape) => {
          const startAngle = Math.atan2(endY - hingeY, endX - hingeX);
          const endAngle = Math.atan2(
            (door.swing === "left" ? p2y : p1y) - hingeY,
            (door.swing === "left" ? p2x : p1x) - hingeX,
          );
          ctx.beginPath();
          ctx.arc(hingeX, hingeY, door.width, startAngle, endAngle, door.swing === "right");
          ctx.strokeStyle = "#059669";
          ctx.lineWidth = Math.max(1 / zoom, 30);
          ctx.setLineDash([100, 80]);
          ctx.stroke();
          ctx.fillStrokeShape(shape);
        }}
      />
    </Group>
  );
}

/** Room label (ID + name + area). */
function RoomLabel({ room, invZoom, isSelected }: { room: ModelRoom; invZoom: number; isSelected: boolean }) {
  const center = useMemo(() => polygonCenter(room.polygon), [room.polygon]);
  const area = useMemo(() => polygonArea(room.polygon) / 1e6, [room.polygon]);

  return (
    <Group x={center.x} y={center.y} listening={false}>
      <Text
        text={room.id}
        fontSize={11 * invZoom}
        fontStyle="bold"
        fontFamily="Inter, system-ui, sans-serif"
        fill={isSelected ? "#92400e" : "#44403c"}
        align="center"
        offsetX={30 * invZoom}
        offsetY={12 * invZoom}
        width={60 * invZoom}
      />
      <Text
        text={room.name}
        fontSize={10 * invZoom}
        fontFamily="Inter, system-ui, sans-serif"
        fill="#78716c"
        align="center"
        offsetX={50 * invZoom}
        y={2 * invZoom}
        width={100 * invZoom}
      />
      <Text
        text={`${area.toFixed(1)} m\u00B2`}
        fontSize={10 * invZoom}
        fontFamily="Inter, system-ui, sans-serif"
        fill="#78716c"
        align="center"
        offsetX={40 * invZoom}
        y={15 * invZoom}
        width={80 * invZoom}
      />
    </Group>
  );
}

/** Dimension annotations on all edges of a room. */
function DimensionAnnotations({ room, invZoom, onSelectWall, onStartEdit }: { room: ModelRoom; invZoom: number; onSelectWall?: (wallIndex: number) => void; onStartEdit?: (wallIndex: number) => void }) {
  const poly = room.polygon;
  const n = poly.length;

  return (
    <Group>
      {Array.from({ length: n }, (_, i) => {
        const a = poly[i]!;
        const b = poly[(i + 1) % n]!;
        const length = Math.sqrt((b.x - a.x) ** 2 + (b.y - a.y) ** 2);
        const mx = (a.x + b.x) / 2;
        const my = (a.y + b.y) / 2;

        // Outward offset
        const angle = Math.atan2(b.y - a.y, b.x - a.x);
        const off = 18 * invZoom;
        const nx = Math.cos(angle - Math.PI / 2) * off;
        const ny = Math.sin(angle - Math.PI / 2) * off;

        return (
          <Group key={i} onClick={() => { onSelectWall?.(i); onStartEdit?.(i); }} onTap={() => { onSelectWall?.(i); onStartEdit?.(i); }}>
            {/* Dimension line */}
            <Line
              points={[a.x + nx, a.y + ny, b.x + nx, b.y + ny]}
              stroke="#d97706"
              strokeWidth={invZoom}
              opacity={0.6}
              hitStrokeWidth={8 * invZoom}
            />
            {/* Ticks */}
            <Line
              points={[a.x + nx * 0.5, a.y + ny * 0.5, a.x + nx * 1.5, a.y + ny * 1.5]}
              stroke="#d97706"
              strokeWidth={invZoom}
            />
            <Line
              points={[b.x + nx * 0.5, b.y + ny * 0.5, b.x + nx * 1.5, b.y + ny * 1.5]}
              stroke="#d97706"
              strokeWidth={invZoom}
            />
            {/* Label */}
            <Text
              x={mx + nx * 1.8}
              y={my + ny * 1.8}
              text={(length / 1000).toFixed(2)}
              fontSize={10 * invZoom}
              fontStyle="bold"
              fontFamily="Inter, system-ui, sans-serif"
              fill="#d97706"
              align="center"
              offsetX={25 * invZoom}
              offsetY={5 * invZoom}
              width={50 * invZoom}
            />
          </Group>
        );
      })}
    </Group>
  );
}

/** Drawing preview (rect, polygon). */
function DrawPreview({ tool, points, cursor, invZoom, snapGridSize, numericInput: _numericInput = "" }: {
  tool: ModellerTool; points: Point2D[]; cursor: Point2D | null; invZoom: number; snapGridSize: number;
  numericInput?: string;
}) {
  if (points.length === 0 && !cursor) return null;

  // Rectangle preview
  if (tool === "draw_rect" && points.length === 1 && cursor) {
    const p0 = points[0]!;
    const x = Math.min(p0.x, cursor.x);
    const y = Math.min(p0.y, cursor.y);
    const w = Math.abs(cursor.x - p0.x);
    const h = Math.abs(cursor.y - p0.y);
    return (
      <Group listening={false}>
        <Rect x={x} y={y} width={w} height={h} fill="rgba(217, 119, 6, 0.08)" stroke="#d97706" strokeWidth={2 * invZoom} dash={[6 * invZoom, 4 * invZoom]} />
        {/* Width label */}
        <Text
          x={x + w / 2}
          y={y - 20 * invZoom}
          text={`${(w / 1000).toFixed(2)} m`}
          fontSize={11 * invZoom}
          fontStyle="bold"
          fontFamily="Inter, system-ui, sans-serif"
          fill="#d97706"
          align="center"
          offsetX={30 * invZoom}
          width={60 * invZoom}
        />
        {/* Height label */}
        <Text
          x={x + w + 8 * invZoom}
          y={y + h / 2}
          text={`${(h / 1000).toFixed(2)} m`}
          fontSize={11 * invZoom}
          fontStyle="bold"
          fontFamily="Inter, system-ui, sans-serif"
          fill="#d97706"
          offsetY={5 * invZoom}
        />
      </Group>
    );
  }

  // Polygon preview (non-wall)
  if (tool === "draw_polygon" && points.length > 0) {
    const allPts = cursor ? [...points, cursor] : points;
    const flatPts = allPts.flatMap((p) => [p.x, p.y]);

    return (
      <Group listening={false}>
        <Line points={flatPts} closed fill="rgba(217, 119, 6, 0.08)" stroke="#d97706" strokeWidth={2 * invZoom} dash={[6 * invZoom, 4 * invZoom]} />
        {cursor && points.length >= 3 && (() => {
          const first = points[0]!;
          const dist = Math.hypot(cursor.x - first.x, cursor.y - first.y);
          if (dist < snapGridSize * 1.5) {
            return <Circle x={first.x} y={first.y} radius={10 * invZoom} stroke="#d97706" strokeWidth={2 * invZoom} />;
          }
          return null;
        })()}
        {points.map((p, i) => (
          <Circle key={i} x={p.x} y={p.y} radius={4 * invZoom} fill="#d97706" stroke="#ffffff" strokeWidth={1.5 * invZoom} />
        ))}
        {points.map((p, i) => {
          const next = i < points.length - 1 ? points[i + 1]! : cursor;
          if (!next) return null;
          const len = Math.hypot(next.x - p.x, next.y - p.y);
          if (len < 100) return null;
          return (
            <Text key={`len-${i}`} x={(p.x + next.x) / 2} y={(p.y + next.y) / 2 - 14 * invZoom}
              text={`${(len / 1000).toFixed(2)} m`} fontSize={10 * invZoom} fontStyle="bold"
              fontFamily="Inter, system-ui, sans-serif" fill="#d97706" align="center"
              offsetX={25 * invZoom} width={50 * invZoom} />
          );
        })}
      </Group>
    );
  }

  return null;
}

/** Underlay image. */
function UnderlayShape({ ul, img }: { ul: UnderlayImage; img: HTMLImageElement }) {
  const imageRef = useRef<Konva.Image>(null);

  useEffect(() => {
    imageRef.current?.cache();
  }, [img]);

  // Use Konva.Image via Shape with sceneFunc since react-konva Image needs special handling
  return (
    <Shape
      sceneFunc={(ctx) => {
        ctx.save();
        ctx.globalAlpha = ul.opacity;
        if (ul.rotation !== 0) {
          const cx = ul.x + ul.width / 2;
          const cy = ul.y + ul.height / 2;
          ctx.translate(cx, cy);
          ctx.rotate((ul.rotation * Math.PI) / 180);
          ctx.drawImage(img, -ul.width / 2, -ul.height / 2, ul.width, ul.height);
        } else {
          ctx.drawImage(img, ul.x, ul.y, ul.width, ul.height);
        }
        ctx.restore();
      }}
    />
  );
}

/** Scale bar (screen coords). */
function ScaleBarShape({ height, zoom }: { width: number; height: number; zoom: number }) {
  return (
    <Shape
      sceneFunc={(ctx, shape) => {
        const pxPerMm = zoom;
        const maxBarPx = 200;
        const niceSteps = [100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000];
        let barMm = 1000;
        for (const step of niceSteps) {
          if (step * pxPerMm <= maxBarPx && step * pxPerMm >= 40) barMm = step;
        }
        const barPx = barMm * pxPerMm;
        const x = 20;
        const y = height - 24;
        const h = 8;

        ctx.fillStyle = "rgba(255,255,255,0.85)";
        ctx.fillRect(x - 6, y - 16, barPx + 12, h + 28);

        const segments = 4;
        const segPx = barPx / segments;
        for (let i = 0; i < segments; i++) {
          ctx.fillStyle = i % 2 === 0 ? "#1c1917" : "#ffffff";
          ctx.fillRect(x + i * segPx, y, segPx, h);
        }

        ctx.strokeStyle = "#1c1917";
        ctx.lineWidth = 1;
        ctx.strokeRect(x, y, barPx, h);
        ctx.beginPath();
        ctx.moveTo(x, y - 3); ctx.lineTo(x, y + h + 3);
        ctx.moveTo(x + barPx, y - 3); ctx.lineTo(x + barPx, y + h + 3);
        ctx.stroke();

        ctx.fillStyle = "#1c1917";
        ctx.font = "bold 10px Inter, system-ui, sans-serif";
        ctx.textBaseline = "top";
        ctx.textAlign = "left";
        ctx.fillText("0", x, y + h + 4);
        ctx.textAlign = "right";
        ctx.fillText(barMm >= 1000 ? `${barMm / 1000} m` : `${barMm} mm`, x + barPx, y + h + 4);
        ctx.textAlign = "center";
        ctx.font = "9px Inter, system-ui, sans-serif";
        ctx.fillStyle = "#78716c";
        ctx.fillText(`1:${Math.round(1000 / (zoom * 1000))}`, x + barPx / 2, y - 13);

        ctx.fillStrokeShape(shape);
      }}
    />
  );
}

/** Snap badge (screen coords). */
function SnapBadge({ width, count }: { width: number; count: number }) {
  return (
    <Group x={width - 80} y={8}>
      <Rect width={60} height={16} fill="rgba(217, 119, 6, 0.15)" cornerRadius={3} />
      <Text
        text={`SNAP: ${count}`}
        x={6}
        y={3}
        fontSize={9}
        fontStyle="bold"
        fontFamily="Inter, system-ui, sans-serif"
        fill="#92400e"
      />
    </Group>
  );
}

// =============================================================================
// Utilities
// =============================================================================

/** Find the partner room that shares this wall edge (for dedup rendering). */
function sharedEdgePartner(
  roomId: string, wallIndex: number, rooms: ModelRoom[], sharedEdges: Set<string>,
): { roomId: string; wallIndex: number } | null {
  if (!sharedEdges.has(`${roomId}:${wallIndex}`)) return null;
  const room = rooms.find((r) => r.id === roomId);
  if (!room) return null;
  const a = room.polygon[wallIndex]!;
  const b = room.polygon[(wallIndex + 1) % room.polygon.length]!;
  for (const other of rooms) {
    if (other.id === roomId) continue;
    for (let oj = 0; oj < other.polygon.length; oj++) {
      if (!sharedEdges.has(`${other.id}:${oj}`)) continue;
      const c = other.polygon[oj]!;
      const d = other.polygon[(oj + 1) % other.polygon.length]!;
      if (segmentsShareEdge(a, b, c, d)) return { roomId: other.id, wallIndex: oj };
    }
  }
  return null;
}

function isDrawingTool(tool: ModellerTool): boolean {
  return tool.startsWith("draw_") || tool === "split_room";
}

function getDrawingHint(tool: ModellerTool, pointCount: number): string {
  if (tool === "draw_rect") return pointCount === 0 ? "Klik om eerste hoek te plaatsen" : "Klik om rechthoek af te ronden";
  if (tool === "draw_polygon") {
    if (pointCount < 3) return `Klik om punt ${pointCount + 1} te plaatsen`;
    return "Klik om punt toe te voegen, dubbelklik of klik bij startpunt om te sluiten";
  }
  if (tool === "draw_circle") return pointCount === 0 ? "Klik om middelpunt te plaatsen" : "Klik om straal in te stellen";
  if (tool === "draw_window") return "Klik op een wand om een raam te plaatsen";
  if (tool === "draw_door") return "Klik op een wand om een deur te plaatsen";
  if (tool === "split_room") {
    if (pointCount === 0) return "Klik op een wand om splitpunt te plaatsen";
    if (pointCount === 1) return "Klik op een andere wand, of klik vrij voor tussenpunten";
    return "Klik op een wand om te splitsen, of voeg meer tussenpunten toe";
  }
  return "Klik om te tekenen";
}

function getMeasureHint(pointCount: number): string {
  if (pointCount === 0) return "Klik om startpunt te plaatsen";
  if (pointCount === 1) return "Klik om eindpunt te plaatsen";
  return "Meting voltooid — klik opnieuw om te meten";
}

function findWallHit(p: Point2D, rooms: ModelRoom[], maxDist: number): { roomId: string; wallIndex: number; offset: number } | null {
  let best: { roomId: string; wallIndex: number; offset: number } | null = null;
  let bestDist = maxDist;

  for (const room of rooms) {
    const poly = room.polygon;
    const n = poly.length;
    for (let i = 0; i < n; i++) {
      const a = poly[i]!;
      const b = poly[(i + 1) % n]!;
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const lenSq = dx * dx + dy * dy;
      if (lenSq < 1) continue;
      let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / lenSq;
      t = Math.max(0, Math.min(1, t));
      const px = a.x + t * dx;
      const py = a.y + t * dy;
      const dist = Math.hypot(p.x - px, p.y - py);
      if (dist < bestDist) {
        bestDist = dist;
        best = { roomId: room.id, wallIndex: i, offset: t * Math.sqrt(lenSq) };
      }
    }
  }
  return best;
}
