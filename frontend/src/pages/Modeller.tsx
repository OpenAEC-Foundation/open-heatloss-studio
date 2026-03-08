import { useCallback, useEffect, useMemo, useState } from "react";

import {
  FloorCanvas,
  FloorCanvas3D,
  PropertiesPanel,
  DEFAULT_SNAP_SETTINGS,
} from "../components/modeller";
import { Ribbon } from "../components/modeller/Ribbon";
import { useModellerStore } from "../components/modeller/modellerStore";
import type { ModellerTool, Point2D, Selection, SnapSettings, ViewMode } from "../components/modeller";
import type { WallAlignment } from "../components/modeller/types";
import { useToastStore } from "../store/toastStore";
import { useCatalogueStore } from "../store/catalogueStore";

export function Modeller() {
  const [tool, setTool] = useState<ModellerTool>("select");
  const [viewMode, setViewMode] = useState<ViewMode>("2d");
  const [activeFloor, setActiveFloor] = useState(0);
  const [selection, setSelection] = useState<Selection>(null);
  const [snap, setSnap] = useState<SnapSettings>(DEFAULT_SNAP_SETTINGS);
  const [wallAlignment, setWallAlignment] = useState<WallAlignment>("exterior");
  const addToast = useToastStore((s) => s.addToast);

  // Store
  const rooms = useModellerStore((s) => s.rooms);
  const windows = useModellerStore((s) => s.windows);
  const doors = useModellerStore((s) => s.doors);
  const walls = useModellerStore((s) => s.walls);
  const underlay = useModellerStore((s) => s.underlay);
  const wallConstructions = useModellerStore((s) => s.wallConstructions);
  const floorConstructions = useModellerStore((s) => s.floorConstructions);
  const roofConstructions = useModellerStore((s) => s.roofConstructions);
  const standaloneWallConstructions = useModellerStore((s) => s.standaloneWallConstructions);

  const addRoom = useModellerStore((s) => s.addRoom);
  const updateRoom = useModellerStore((s) => s.updateRoom);
  const removeRoom = useModellerStore((s) => s.removeRoom);
  const addWindow = useModellerStore((s) => s.addWindow);
  const updateWindow = useModellerStore((s) => s.updateWindow);
  const removeWindow = useModellerStore((s) => s.removeWindow);
  const addDoor = useModellerStore((s) => s.addDoor);
  const addWall = useModellerStore((s) => s.addWall);
  const updateWall = useModellerStore((s) => s.updateWall);
  const removeWall = useModellerStore((s) => s.removeWall);
  const setUnderlay = useModellerStore((s) => s.setUnderlay);
  const assignWallConstruction = useModellerStore((s) => s.assignWallConstruction);
  const assignFloorConstruction = useModellerStore((s) => s.assignFloorConstruction);
  const assignRoofConstruction = useModellerStore((s) => s.assignRoofConstruction);
  const assignStandaloneWallConstruction = useModellerStore((s) => s.assignStandaloneWallConstruction);
  const undo = useModellerStore((s) => s.undo);
  const redo = useModellerStore((s) => s.redo);

  // Catalogue U-values (entryId → U-value)
  const catalogueEntries = useCatalogueStore((s) => s.entries);
  const catalogueUValues = useMemo(() => {
    const map: Record<string, number> = {};
    for (const e of catalogueEntries) {
      map[e.id] = e.uValue;
    }
    return map;
  }, [catalogueEntries]);

  // Fit view trigger counter
  const [fitViewTrigger, setFitViewTrigger] = useState(0);

  // Filter by floor (memoized to prevent unnecessary 3D scene re-init)
  const floorRooms = useMemo(() => rooms.filter((r) => r.floor === activeFloor), [rooms, activeFloor]);
  const floorWindows = useMemo(() => windows.filter((w) => floorRooms.some((r) => r.id === w.roomId)), [windows, floorRooms]);
  const floorDoors = useMemo(() => doors.filter((d) => floorRooms.some((r) => r.id === d.roomId)), [doors, floorRooms]);
  const floorWalls = useMemo(() => walls.filter((w) => w.floor === activeFloor), [walls, activeFloor]);

  // Selected room (for properties panel)
  const selectedRoomId = selection?.type === "room" ? selection.roomId
    : selection?.type === "wall" ? selection.roomId
    : selection?.type === "window" ? selection.roomId
    : null;
  const selectedRoom = rooms.find((r) => r.id === selectedRoomId) ?? null;

  // --- Handlers ---

  const handleAddRoom = useCallback(
    (polygon: Point2D[]) => {
      const id = addRoom({
        name: "Nieuwe ruimte",
        function: "custom",
        polygon,
        floor: activeFloor,
        height: 2600,
      });
      setSelection({ type: "room", roomId: id });
      setTool("select");
      addToast(`Ruimte ${id} aangemaakt`, "success");
    },
    [addRoom, activeFloor, addToast],
  );

  const handleAddWindow = useCallback(
    (roomId: string, wallIndex: number, offset: number, width: number) => {
      addWindow({ roomId, wallIndex, offset, width });
      addToast("Raam geplaatst", "success");
    },
    [addWindow, addToast],
  );

  const handleAddDoor = useCallback(
    (roomId: string, wallIndex: number, offset: number, width: number) => {
      addDoor({ roomId, wallIndex, offset, width, swing: "left" });
      addToast("Deur geplaatst", "success");
    },
    [addDoor, addToast],
  );

  const handleAddWall = useCallback(
    (points: Point2D[]) => {
      addWall({ points, floor: activeFloor, alignment: wallAlignment });
      addToast("Wand getekend", "success");
    },
    [addWall, activeFloor, wallAlignment, addToast],
  );

  const handleRemoveWall = useCallback(
    (id: string) => {
      removeWall(id);
      addToast("Wand verwijderd", "info");
    },
    [removeWall, addToast],
  );

  const handleMoveRoom = useCallback(
    (roomId: string, dx: number, dy: number) => {
      const room = rooms.find((r) => r.id === roomId);
      if (!room) return;
      const gs = snap.enabled && snap.modes.includes("grid") ? snap.gridSize : 1;
      const sdx = Math.round(dx / gs) * gs;
      const sdy = Math.round(dy / gs) * gs;
      const newPoly = room.polygon.map((p) => ({ x: p.x + sdx, y: p.y + sdy }));
      updateRoom(roomId, { polygon: newPoly });
    },
    [rooms, snap, updateRoom],
  );

  const handleMoveVertex = useCallback(
    (roomId: string, vertexIndex: number, x: number, y: number) => {
      const room = rooms.find((r) => r.id === roomId);
      if (!room) return;
      const newPoly = room.polygon.map((p, i) => (i === vertexIndex ? { x, y } : p));
      updateRoom(roomId, { polygon: newPoly });
    },
    [rooms, updateRoom],
  );

  const handleUpdateWindow = useCallback(
    (roomId: string, wallIndex: number, offset: number, updates: Partial<{ offset: number; width: number }>) => {
      updateWindow(roomId, wallIndex, offset, updates);
    },
    [updateWindow],
  );

  const handleRemoveRoom = useCallback(
    (id: string) => {
      removeRoom(id);
      if (selectedRoomId === id) setSelection(null);
      addToast("Ruimte verwijderd", "info");
    },
    [removeRoom, selectedRoomId, addToast],
  );

  const handleRemoveWindow = useCallback(
    (roomId: string, wallIndex: number, offset: number) => {
      removeWindow(roomId, wallIndex, offset);
      setSelection(selectedRoomId ? { type: "room", roomId: selectedRoomId } : null);
      addToast("Raam verwijderd", "info");
    },
    [removeWindow, selectedRoomId, addToast],
  );

  const handleImportPdf = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*,.pdf";
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) return;
      if (!file.type.startsWith("image/")) {
        addToast("Momenteel alleen afbeeldingen ondersteund als onderlegger (PNG, JPG)", "info");
        return;
      }
      const reader = new FileReader();
      reader.onload = () => {
        const dataUrl = reader.result as string;
        const img = new Image();
        img.onload = () => {
          const scale = 10;
          setUnderlay({
            dataUrl,
            fileName: file.name,
            x: 0, y: 0,
            width: img.width * scale,
            height: img.height * scale,
            opacity: 0.3,
            rotation: 0,
            locked: false,
          });
          addToast(`Onderlegger "${file.name}" geladen`, "success");
        };
        img.src = dataUrl;
      };
      reader.readAsDataURL(file);
    };
    input.click();
  }, [setUnderlay, addToast]);

  const handleImportDwg = useCallback(() => {
    addToast("DWG import wordt binnenkort beschikbaar. Gebruik een afbeelding (PNG/JPG) als onderlegger.", "info");
  }, [addToast]);

  const handleImportIfc = useCallback(() => {
    addToast("IFC import wordt binnenkort beschikbaar", "info");
  }, [addToast]);

  const handleExportIfc = useCallback(() => {
    addToast("IFC export wordt binnenkort beschikbaar", "info");
  }, [addToast]);

  const handleFitView = useCallback(() => {
    setFitViewTrigger((n) => n + 1);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      if (e.ctrlKey && e.key === "z") { e.preventDefault(); undo(); return; }
      if (e.ctrlKey && e.key === "y") { e.preventDefault(); redo(); return; }

      if (e.key === "Delete") {
        if (selection?.type === "room") { handleRemoveRoom(selection.roomId); return; }
        if (selection?.type === "window") { handleRemoveWindow(selection.roomId, selection.wallIndex, selection.offset); return; }
        if (selection?.type === "standalone_wall") { handleRemoveWall(selection.wallId); return; }
      }

      const keyMap: Record<string, ModellerTool> = {
        v: "select", h: "pan", r: "draw_rect", p: "draw_polygon",
        c: "draw_circle", w: "draw_wall", n: "draw_window",
        d: "draw_door", m: "measure",
      };
      const mapped = keyMap[e.key.toLowerCase()];
      if (mapped) setTool(mapped);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [undo, redo, selection, handleRemoveRoom, handleRemoveWindow, handleRemoveWall]);

  return (
    <div className="flex h-screen flex-col">
      <Ribbon
        tool={tool}
        viewMode={viewMode}
        activeFloor={activeFloor}
        snap={snap}
        onToolChange={setTool}
        onViewModeChange={setViewMode}
        onFloorChange={setActiveFloor}
        onSnapChange={setSnap}
        onFitView={handleFitView}
        onUndo={undo}
        onRedo={redo}
        onImportDwg={handleImportDwg}
        onImportPdf={handleImportPdf}
        onImportIfc={handleImportIfc}
        onExportIfc={handleExportIfc}
      />

      <div className="flex min-h-0 flex-1">
        {viewMode === "2d" ? (
          <div className="min-w-0 flex-1">
            <FloorCanvas
              rooms={floorRooms}
              windows={floorWindows}
              doors={floorDoors}
              selection={selection}
              tool={tool}
              snap={snap}
              underlay={underlay}
              wallConstructions={wallConstructions}
              catalogueUValues={catalogueUValues}
              onSelect={setSelection}
              onAddRoom={handleAddRoom}
              onAddWindow={handleAddWindow}
              onAddDoor={handleAddDoor}
              onMoveRoom={handleMoveRoom}
              walls={floorWalls}
              onMoveVertex={handleMoveVertex}
              onUpdateWindow={handleUpdateWindow}
              onRemoveRoom={handleRemoveRoom}
              onRemoveWindow={handleRemoveWindow}
              onAddWall={handleAddWall}
              onRemoveWall={handleRemoveWall}
              wallAlignment={wallAlignment}
              onWallAlignmentChange={setWallAlignment}
              fitViewTrigger={fitViewTrigger}
            />
          </div>
        ) : (
          <div className="min-w-0 flex-1">
            <FloorCanvas3D
              rooms={floorRooms}
              windows={floorWindows}
              doors={floorDoors}
              selectedRoomId={selectedRoomId}
              onSelectRoom={(id) => setSelection(id ? { type: "room", roomId: id } : null)}
              onDeleteRoom={handleRemoveRoom}
            />
          </div>
        )}

        <PropertiesPanel
          room={selectedRoom}
          rooms={floorRooms}
          windows={floorWindows}
          walls={floorWalls}
          selection={selection}
          onUpdateRoom={updateRoom}
          onRemoveRoom={handleRemoveRoom}
          onUpdateWall={updateWall}
          onRemoveWall={handleRemoveWall}
          onUpdateWindow={handleUpdateWindow}
          onRemoveWindow={handleRemoveWindow}
          wallConstructions={wallConstructions}
          floorConstructions={floorConstructions}
          roofConstructions={roofConstructions}
          standaloneWallConstructions={standaloneWallConstructions}
          onAssignWall={assignWallConstruction}
          onAssignFloor={assignFloorConstruction}
          onAssignRoof={assignRoofConstruction}
          onAssignStandaloneWall={assignStandaloneWallConstruction}
          tool={tool}
          wallAlignment={wallAlignment}
          onWallAlignmentChange={setWallAlignment}
        />
      </div>
    </div>
  );
}
