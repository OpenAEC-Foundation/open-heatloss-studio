import { useCallback, useEffect, useMemo, useState } from "react";

import {
  FloorCanvas,
  FloorCanvas3D,
  PropertiesPanel,
  DEFAULT_SNAP_SETTINGS,
} from "../components/modeller";
import { Ribbon } from "../components/modeller/Ribbon";
import { useModellerStore } from "../components/modeller/modellerStore";
import type { ModellerTool, ModelRoom, ModelWindow, Point2D, Selection, SnapSettings, ViewMode } from "../components/modeller";
import { splitPolygon } from "../components/modeller";
import { importIfcFile } from "../components/modeller/ifc-import";
import { extractWallTypesFromFile, type IfcWallTypeInfo } from "../components/modeller/ifc-wall-types";
import { IfcWallTypeReview } from "../components/modeller/IfcWallTypeReview";
import { ProjectLibraryPanel } from "../components/modeller/ProjectLibraryPanel";
import { modelToIfcx } from "../components/modeller/ifcx-builder";
import { renderPdfFirstPage } from "../components/modeller/pdf-underlay";
import { useToastStore } from "../store/toastStore";
import { useProjectStore } from "../store/projectStore";
import { useAllConstructions } from "../hooks/useAllConstructions";
import { importProject, exportProject } from "../lib/importExport";
import { FLOOR_LABELS } from "../components/modeller/exampleData";
import { polygonArea, segmentsShareEdge, mergePolygons, removeCollinearVertices } from "../components/modeller";

export function Modeller() {
  const [tool, setTool] = useState<ModellerTool>("select");
  const [viewMode, setViewMode] = useState<ViewMode>("2d");
  const [activeFloor, setActiveFloor] = useState(0);
  const [selection, setSelection] = useState<Selection>(null);
  const [snap, setSnap] = useState<SnapSettings>(DEFAULT_SNAP_SETTINGS);
  const addToast = useToastStore((s) => s.addToast);

  // Store
  const rooms = useModellerStore((s) => s.rooms);
  const windows = useModellerStore((s) => s.windows);
  const doors = useModellerStore((s) => s.doors);

  const underlay = useModellerStore((s) => s.underlay);
  const wallConstructions = useModellerStore((s) => s.wallConstructions);
  const floorConstructions = useModellerStore((s) => s.floorConstructions);
  const roofConstructions = useModellerStore((s) => s.roofConstructions);

  const addRoom = useModellerStore((s) => s.addRoom);
  const updateRoom = useModellerStore((s) => s.updateRoom);
  const removeRoom = useModellerStore((s) => s.removeRoom);
  const addWindow = useModellerStore((s) => s.addWindow);
  const updateWindow = useModellerStore((s) => s.updateWindow);
  const removeWindow = useModellerStore((s) => s.removeWindow);
  const addDoor = useModellerStore((s) => s.addDoor);

  const setUnderlay = useModellerStore((s) => s.setUnderlay);
  const assignWallConstruction = useModellerStore((s) => s.assignWallConstruction);
  const assignFloorConstruction = useModellerStore((s) => s.assignFloorConstruction);
  const assignRoofConstruction = useModellerStore((s) => s.assignRoofConstruction);
  const wallBoundaryTypes = useModellerStore((s) => s.wallBoundaryTypes);
  const assignWallBoundaryType = useModellerStore((s) => s.assignWallBoundaryType);

  const importModel = useModellerStore((s) => s.importModel);
  const importProjectConstructions = useModellerStore(
    (s) => s.importProjectConstructions,
  );
  const undo = useModellerStore((s) => s.undo);
  const redo = useModellerStore((s) => s.redo);

  // IFC wall type review dialog state
  const [ifcWallTypes, setIfcWallTypes] = useState<IfcWallTypeInfo[] | null>(
    null,
  );

  // All constructions U-values (entryId → U-value), includes project constructions
  const allConstructionEntries = useAllConstructions();
  const catalogueUValues = useMemo(() => {
    const map: Record<string, number> = {};
    for (const e of allConstructionEntries) {
      map[e.id] = e.uValue;
    }
    return map;
  }, [allConstructionEntries]);

  // Fit view trigger counter
  const [fitViewTrigger, setFitViewTrigger] = useState(0);

  // Filter by floor (memoized to prevent unnecessary 3D scene re-init)
  const floorRooms = useMemo(() => rooms.filter((r) => r.floor === activeFloor), [rooms, activeFloor]);
  const floorWindows = useMemo(() => windows.filter((w) => floorRooms.some((r) => r.id === w.roomId)), [windows, floorRooms]);
  const floorDoors = useMemo(() => doors.filter((d) => floorRooms.some((r) => r.id === d.roomId)), [doors, floorRooms]);
  const belowFloorRooms = useMemo(() => activeFloor > 0 ? rooms.filter((r) => r.floor === activeFloor - 1) : [], [rooms, activeFloor]);

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
      // Clean up collinear vertices on same-floor neighbours after removal
      const removed = rooms.find((r) => r.id === id);
      removeRoom(id);
      if (removed) {
        const neighbours = rooms.filter((r) => r.id !== id && r.floor === removed.floor);
        for (const nb of neighbours) {
          const cleaned = removeCollinearVertices(nb.polygon);
          if (cleaned.length < nb.polygon.length) {
            updateRoom(nb.id, { polygon: cleaned });
          }
        }
      }
      if (selectedRoomId === id) setSelection(null);
      addToast("Ruimte verwijderd", "info");
    },
    [rooms, removeRoom, updateRoom, selectedRoomId, addToast],
  );

  const handleRemoveWindow = useCallback(
    (roomId: string, wallIndex: number, offset: number) => {
      removeWindow(roomId, wallIndex, offset);
      setSelection(selectedRoomId ? { type: "room", roomId: selectedRoomId } : null);
      addToast("Raam verwijderd", "info");
    },
    [removeWindow, selectedRoomId, addToast],
  );

  const handleSplitRoom = useCallback(
    (roomId: string, edgeA: number, tA: number, edgeB: number, tB: number, intermediatePoints?: Point2D[]) => {
      const room = rooms.find((r) => r.id === roomId);
      if (!room) return;
      const result = splitPolygon(room.polygon, edgeA, tA, edgeB, tB, intermediatePoints);
      if (!result) {
        addToast("Splitsen mislukt — probeer twee verschillende wanden", "info");
        return;
      }
      const [poly1, poly2] = result;
      // Remove original room, add two new ones
      removeRoom(roomId);
      const id1 = addRoom({ name: room.name, function: room.function, polygon: poly1, floor: room.floor, height: room.height });
      addRoom({ name: `${room.name} (2)`, function: room.function, polygon: poly2, floor: room.floor, height: room.height });
      setSelection({ type: "room", roomId: id1 });
      addToast("Ruimte gesplitst", "success");
    },
    [rooms, removeRoom, addRoom, addToast],
  );

  const handleMergeRooms = useCallback(
    (roomIdA: string, wallA: number, roomIdB: string, wallB: number) => {
      const rA = rooms.find((r) => r.id === roomIdA);
      const rB = rooms.find((r) => r.id === roomIdB);
      if (!rA || !rB) return;
      const merged = mergePolygons(rA.polygon, wallA, rB.polygon, wallB);
      if (!merged) {
        addToast("Samenvoegen mislukt — controleer de gedeelde wand", "info");
        return;
      }
      removeRoom(roomIdA);
      removeRoom(roomIdB);
      const id = addRoom({
        name: rA.name,
        function: rA.function,
        polygon: merged,
        floor: rA.floor,
        height: Math.max(rA.height, rB.height),
      });
      // Clean up collinear vertices on same-floor neighbours
      const neighbours = rooms.filter(
        (r) => r.id !== roomIdA && r.id !== roomIdB && r.floor === rA.floor,
      );
      for (const nb of neighbours) {
        const cleaned = removeCollinearVertices(nb.polygon);
        if (cleaned.length < nb.polygon.length) {
          updateRoom(nb.id, { polygon: cleaned });
        }
      }
      setSelection({ type: "room", roomId: id });
      addToast("Ruimten samengevoegd", "success");
    },
    [rooms, removeRoom, addRoom, updateRoom, addToast],
  );

  const handleImportPdf = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*,.pdf";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;

      const isPdf = file.type === "application/pdf" || file.name.toLowerCase().endsWith(".pdf");

      if (isPdf) {
        try {
          addToast(`PDF "${file.name}" wordt gerenderd...`, "info");
          const { dataUrl, width, height } = await renderPdfFirstPage(file);
          const scale = 10; // px → mm
          setUnderlay({
            dataUrl,
            fileName: file.name,
            x: 0, y: 0,
            width: width * scale,
            height: height * scale,
            opacity: 0.3,
            rotation: 0,
            locked: false,
          });
          addToast(`Onderlegger "${file.name}" geladen`, "success");
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          addToast(`PDF import mislukt: ${msg}`, "error");
        }
        return;
      }

      if (!file.type.startsWith("image/")) {
        addToast("Gebruik een afbeelding (PNG, JPG) of PDF als onderlegger", "info");
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
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".ifc";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;

      addToast(`IFC bestand "${file.name}" wordt geladen...`, "info");

      try {
        const result = await importIfcFile(file);

        if (result.rooms.length === 0) {
          addToast(
            `Geen ruimten gevonden in "${file.name}". ${result.stats.spacesFound} IfcSpace entiteiten gevonden, ${result.stats.spacesSkipped} overgeslagen.`,
            "info",
          );
          return;
        }

        // Generate IDs for imported rooms
        const existingRooms = useModellerStore.getState().rooms;
        const roomsWithIds: ModelRoom[] = [];
        const usedIds = new Set(existingRooms.map((r) => r.id));

        for (const room of result.rooms) {
          let id: string;
          let num = 1;
          do {
            id = `${room.floor}.${String(num).padStart(2, "0")}`;
            num++;
          } while (usedIds.has(id));
          usedIds.add(id);
          roomsWithIds.push({ ...room, id });
        }

        importModel(roomsWithIds);

        const msg = `${result.stats.spacesImported} ruimten geimporteerd uit "${file.name}"`;
        addToast(msg, "success");

        if (result.warnings.length > 0) {
          const warnMsg = result.warnings
            .map((w) => `${w.spaceName}: ${w.message}`)
            .join(", ");
          addToast(`Waarschuwingen: ${warnMsg}`, "info");
        }

        // Phase 2: extract wall types for project construction import
        try {
          const wallTypes = await extractWallTypesFromFile(file);
          if (wallTypes.length > 0) {
            setIfcWallTypes(wallTypes);
            addToast(
              `${wallTypes.length} wandtype(n) gevonden — controleer de matching`,
              "info",
            );
          }
        } catch {
          // Wall type extraction is optional — don't fail the import
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        addToast(`IFC import mislukt: ${message}`, "error");
      }
    };
    input.click();
  }, [addToast, importModel]);

  const handleExportIfc = useCallback(() => {
    const state = useModellerStore.getState();
    if (state.rooms.length === 0) {
      addToast("Geen ruimten om te exporteren", "info");
      return;
    }

    const projectName = useProjectStore.getState().project.info.name || "Model";

    try {
      const doc = modelToIfcx(state.rooms, state.windows, state.doors, {
        projectName,
        author: "ISSO 51 Warmteverliesberekening",
      });

      const json = JSON.stringify(doc, null, 2);
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);

      const safeName = projectName.replace(/[^a-zA-Z0-9_\-\s]/g, "").trim() || "model";
      const a = document.createElement("a");
      a.href = url;
      a.download = `${safeName}.ifcx`;
      a.click();
      URL.revokeObjectURL(url);

      addToast(`IFCX export "${safeName}.ifcx" gedownload`, "success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      addToast(`IFCX export mislukt: ${msg}`, "error");
    }
  }, [addToast]);

  const handleExportJson = useCallback(() => {
    const { project, result } = useProjectStore.getState();
    exportProject(project, result);
    addToast("Project geexporteerd", "success");
  }, [addToast]);

  const handleImportJson = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,.isso51.json";
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const imported = importProject(reader.result as string);
          useProjectStore.getState().setProject(imported.project);
          if (imported.result) {
            useProjectStore.getState().setResult(imported.result);
          }
          addToast(`Project "${imported.project.info.name || file.name}" geladen`, "success");
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          addToast(`Import mislukt: ${msg}`, "error");
        }
      };
      reader.readAsText(file);
    };
    input.click();
  }, [addToast]);

  const handleClearView = useCallback(() => {
    importModel([], [], []);
    setSelection(null);
    addToast("Beeld geleegd", "info");
  }, [importModel, addToast]);

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

      if (e.key === "Escape") { setTool("select"); setSelection(null); return; }

      if (e.key === "Delete") {
        if (selection?.type === "room") { handleRemoveRoom(selection.roomId); return; }
        if (selection?.type === "window") { handleRemoveWindow(selection.roomId, selection.wallIndex, selection.offset); return; }
      }

      const keyMap: Record<string, ModellerTool> = {
        v: "select", h: "pan", r: "draw_rect", p: "draw_polygon",
        c: "draw_circle", n: "draw_window",
        d: "draw_door", s: "split_room", m: "measure",
      };
      const mapped = keyMap[e.key.toLowerCase()];
      if (mapped) setTool(mapped);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [undo, redo, selection, handleRemoveRoom, handleRemoveWindow]);

  // IFC wall type review handlers
  const handleImportWallTypes = useCallback(
    (constructions: Omit<import("../components/modeller/types").ProjectConstruction, "id">[]) => {
      importProjectConstructions(constructions);
      setIfcWallTypes(null);
      addToast(
        `${constructions.length} constructie(s) geimporteerd als projectconstructie`,
        "success",
      );
    },
    [importProjectConstructions, addToast],
  );

  const handleCancelWallTypes = useCallback(() => {
    setIfcWallTypes(null);
  }, []);

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
        onImportJson={handleImportJson}
        onExportJson={handleExportJson}
        onClearView={handleClearView}
      />

      <div className="flex min-h-0 flex-1">
        {/* Left: Project Browser */}
        <ProjectBrowser
          rooms={rooms}
          floorRooms={floorRooms}
          windows={windows}
          selection={selection}
          selectedRoom={selectedRoom}
          activeFloor={activeFloor}
          onFloorChange={setActiveFloor}
          onSelect={setSelection}
          onUpdateRoom={updateRoom}
          onRemoveRoom={handleRemoveRoom}
          onUpdateWindow={handleUpdateWindow}
          onRemoveWindow={handleRemoveWindow}
          wallConstructions={wallConstructions}
          floorConstructions={floorConstructions}
          roofConstructions={roofConstructions}
          onAssignWall={assignWallConstruction}
          onAssignFloor={assignFloorConstruction}
          onAssignRoof={assignRoofConstruction}
        />

        {/* Center: Canvas area with 2D/3D overlay */}
        <div className="relative min-w-0 flex-1">
          {viewMode === "2d" ? (
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
              wallBoundaryTypes={wallBoundaryTypes}
              ghostRooms={belowFloorRooms}
              onSelect={setSelection}
              onAddRoom={handleAddRoom}
              onAddWindow={handleAddWindow}
              onAddDoor={handleAddDoor}
              onMoveRoom={handleMoveRoom}
              onMoveVertex={handleMoveVertex}
              onUpdateWindow={handleUpdateWindow}
              onRemoveRoom={handleRemoveRoom}
              onRemoveWindow={handleRemoveWindow}
              onSplitRoom={handleSplitRoom}
              onMergeRooms={handleMergeRooms}
              fitViewTrigger={fitViewTrigger}
            />
          ) : (
            <FloorCanvas3D
              rooms={rooms}
              windows={windows}
              doors={doors}
              selection={selection}
              onSelect={setSelection}
              onDeleteRoom={handleRemoveRoom}
              wallConstructions={wallConstructions}
              floorConstructions={floorConstructions}
              roofConstructions={roofConstructions}
              catalogueUValues={catalogueUValues}
            />
          )}

          {/* 2D / 3D toggle — top left overlay */}
          <div className="pointer-events-auto absolute left-3 top-3 z-20 flex overflow-hidden rounded-lg border border-stone-200 bg-white/95 shadow-sm backdrop-blur-sm text-xs">
            <button
              onClick={() => setViewMode("2d")}
              className={`px-3 py-1.5 font-medium transition-colors ${
                viewMode === "2d" ? "bg-stone-800 text-white" : "text-stone-500 hover:bg-stone-100"
              }`}
            >
              2D
            </button>
            <button
              onClick={() => setViewMode("3d")}
              className={`px-3 py-1.5 font-medium transition-colors ${
                viewMode === "3d" ? "bg-stone-800 text-white" : "text-stone-500 hover:bg-stone-100"
              }`}
            >
              3D
            </button>
          </div>
        </div>

        {/* Right: Properties Panel */}
        <PropertiesPanel
          room={selectedRoom}
          rooms={floorRooms}
          windows={floorWindows}
          selection={selection}
          onUpdateRoom={updateRoom}
          onRemoveRoom={handleRemoveRoom}
          onUpdateWindow={handleUpdateWindow}
          onRemoveWindow={handleRemoveWindow}
          wallConstructions={wallConstructions}
          floorConstructions={floorConstructions}
          roofConstructions={roofConstructions}
          onAssignWall={assignWallConstruction}
          onAssignFloor={assignFloorConstruction}
          onAssignRoof={assignRoofConstruction}
          wallBoundaryTypes={wallBoundaryTypes}
          onAssignBoundaryType={assignWallBoundaryType}
        />
      </div>

      {/* IFC Wall Type Review Dialog */}
      {ifcWallTypes && ifcWallTypes.length > 0 && (
        <IfcWallTypeReview
          wallTypes={ifcWallTypes}
          onImport={handleImportWallTypes}
          onCancel={handleCancelWallTypes}
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Project Browser — right panel with floor/room/surface hierarchy
// ---------------------------------------------------------------------------


interface ProjectBrowserProps {
  rooms: ModelRoom[];
  floorRooms: ModelRoom[];
  windows: ModelWindow[];
  selection: Selection;
  selectedRoom: ModelRoom | null;
  activeFloor: number;
  onFloorChange: (floor: number) => void;
  onSelect: (sel: Selection) => void;
  onUpdateRoom: (id: string, updates: Partial<Omit<ModelRoom, "id">>) => void;
  onRemoveRoom: (id: string) => void;
  onUpdateWindow: (roomId: string, wallIndex: number, offset: number, updates: Partial<{ offset: number; width: number }>) => void;
  onRemoveWindow: (roomId: string, wallIndex: number, offset: number) => void;
  wallConstructions: Record<string, string>;
  floorConstructions: Record<string, string>;
  roofConstructions: Record<string, string>;
  onAssignWall: (roomId: string, wallIndex: number, entryId: string | null) => void;
  onAssignFloor: (roomId: string, entryId: string | null) => void;
  onAssignRoof: (roomId: string, entryId: string | null) => void;
}

function ProjectBrowser({
  rooms,
  windows,
  selection,
  activeFloor,
  onFloorChange,
  onSelect,
  onRemoveRoom,
  wallConstructions,
}: ProjectBrowserProps) {
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  const toggle = (key: string) => setCollapsed((prev) => ({ ...prev, [key]: !prev[key] }));

  // Group rooms by floor
  const floorGroups = FLOOR_LABELS.map((label, floor) => ({
    label,
    floor,
    rooms: rooms.filter((r) => r.floor === floor),
  }));

  const catalogueEntries = useAllConstructions();

  return (
    <div className="w-64 shrink-0 overflow-y-auto border-r border-stone-200 bg-white text-xs">
      <div className="border-b border-stone-100 px-3 py-2">
        <span className="text-[11px] font-semibold uppercase tracking-wider text-stone-500">Project</span>
      </div>

      {floorGroups.map(({ label, floor, rooms: floorRooms }) => {
        const floorKey = `floor-${floor}`;
        const isFloorCollapsed = collapsed[floorKey];
        const isActive = floor === activeFloor;

        return (
          <div key={floor}>
            {/* Floor header */}
            <button
              onClick={() => { toggle(floorKey); onFloorChange(floor); }}
              className={`flex w-full items-center gap-1.5 px-3 py-1.5 text-left transition-colors ${
                isActive ? "bg-amber-50 font-semibold text-amber-900" : "text-stone-700 hover:bg-stone-50"
              }`}
            >
              <span className="text-[10px] text-stone-400">{isFloorCollapsed ? "\u25B6" : "\u25BC"}</span>
              <span>{label}</span>
              <span className="ml-auto text-[10px] text-stone-400">{floorRooms.length}</span>
            </button>

            {/* Rooms under this floor */}
            {!isFloorCollapsed && floorRooms.map((room) => {
              const roomKey = `room-${room.id}`;
              const isRoomCollapsed = collapsed[roomKey];
              const isSelected = selection?.type === "room" && selection.roomId === room.id;
              const area = polygonArea(room.polygon) / 1e6;
              const roomWindows = windows.filter((w) => w.roomId === room.id);

              return (
                <div key={room.id}>
                  {/* Room header */}
                  <div
                    className={`flex items-center gap-1 pl-6 pr-3 py-1 cursor-pointer transition-colors ${
                      isSelected ? "bg-amber-100 text-amber-900" : "text-stone-600 hover:bg-stone-50"
                    }`}
                    onClick={() => onSelect({ type: "room", roomId: room.id })}
                  >
                    <button
                      onClick={(e) => { e.stopPropagation(); toggle(roomKey); }}
                      className="text-[10px] text-stone-400 w-3"
                    >
                      {isRoomCollapsed ? "\u25B6" : "\u25BC"}
                    </button>
                    <span className="font-mono font-medium text-[10px]">{room.id}</span>
                    <span className="truncate flex-1">{room.name}</span>
                    <span className="text-[10px] text-stone-400">{area.toFixed(1)}m²</span>
                  </div>

                  {/* Surfaces under this room */}
                  {!isRoomCollapsed && (
                    <div className="pl-10 pr-3">
                      {/* Walls */}
                      {room.polygon.map((_, wi) => {
                        const ni = (wi + 1) % room.polygon.length;
                        const a = room.polygon[wi]!;
                        const b = room.polygon[ni]!;
                        const len = Math.hypot(b.x - a.x, b.y - a.y);
                        const dir = wallDir(room.polygon, wi);
                        const isWallSel = selection?.type === "wall" && selection.roomId === room.id && selection.wallIndex === wi;
                        const assignedId = wallConstructions[`${room.id}:${wi}`];
                        const assigned = assignedId ? catalogueEntries.find((e) => e.id === assignedId) : null;

                        // Check if shared
                        let isShared = false;
                        for (const other of rooms) {
                          if (other.id === room.id) continue;
                          for (let oj = 0; oj < other.polygon.length; oj++) {
                            if (segmentsShareEdge(a, b, other.polygon[oj]!, other.polygon[(oj + 1) % other.polygon.length]!)) {
                              isShared = true; break;
                            }
                          }
                          if (isShared) break;
                        }

                        const wallWins = roomWindows.filter((w) => w.wallIndex === wi);

                        return (
                          <div
                            key={`w-${wi}`}
                            className={`flex items-center gap-1 py-0.5 cursor-pointer rounded px-1 ${
                              isWallSel ? "bg-amber-100" : "hover:bg-stone-50"
                            }`}
                            onClick={() => onSelect({ type: "wall", roomId: room.id, wallIndex: wi })}
                          >
                            <span className="text-stone-400 w-10 text-[10px]">{dir}</span>
                            <span className="text-[10px] flex-1">{(len / 1000).toFixed(2)}m</span>
                            {isShared && <span className="text-[9px] text-blue-500">int</span>}
                            {!isShared && <span className="text-[9px] text-red-500">ext</span>}
                            {wallWins.length > 0 && <span className="text-[9px] text-blue-400">{wallWins.length}R</span>}
                            {assigned && <span className="text-[9px] text-green-600">U={assigned.uValue}</span>}
                          </div>
                        );
                      })}

                      {/* Floor surface */}
                      <div className="flex items-center gap-1 py-0.5 px-1 text-[10px] text-stone-500">
                        <span className="w-10">Vloer</span>
                        <span className="flex-1">{area.toFixed(2)}m²</span>
                      </div>

                      {/* Ceiling surface */}
                      <div className="flex items-center gap-1 py-0.5 px-1 text-[10px] text-stone-500">
                        <span className="w-10">Plafond</span>
                        <span className="flex-1">{area.toFixed(2)}m²</span>
                      </div>

                      {/* Delete button */}
                      <button
                        onClick={() => onRemoveRoom(room.id)}
                        className="mt-0.5 mb-1 text-[10px] text-red-400 hover:text-red-600 px-1"
                      >
                        Verwijderen
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        );
      })}

      {/* Project constructions library */}
      <div className="border-t border-stone-100">
        <ProjectLibraryPanel />
      </div>
    </div>
  );
}

function wallDir(polygon: Point2D[], edgeIndex: number): string {
  const n = polygon.length;
  const a = polygon[edgeIndex]!;
  const b = polygon[(edgeIndex + 1) % n]!;
  const cx = polygon.reduce((s, p) => s + p.x, 0) / n;
  const cy = polygon.reduce((s, p) => s + p.y, 0) / n;
  const nx = (a.x + b.x) / 2 - cx;
  const ny = (a.y + b.y) / 2 - cy;
  if (Math.abs(nx) > Math.abs(ny)) return nx > 0 ? "O" : "W";
  return ny > 0 ? "Z" : "N";
}
