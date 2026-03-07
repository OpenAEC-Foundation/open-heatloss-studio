import { useEffect, useRef, useMemo, useCallback } from "react";
import * as THREE from "three";
import * as OBC from "@thatopen/components";

import type { ModelRoom, ModelWindow, Point2D } from "./types";
import { polygonCenter } from "./geometry";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface FloorCanvas3DProps {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  selectedRoomId: string | null;
  onSelectRoom: (id: string | null) => void;
}

// ---------------------------------------------------------------------------
// Color scheme
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

const WALL_COLOR = 0x78716c;
const WALL_THICKNESS = 0.2; // meters
const SELECTED_COLOR = 0xf59e0b;
const WINDOW_COLOR = 0x93c5fd;
const FLOOR_OPACITY = 0.95;
const CEILING_OPACITY = 0.15;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function FloorCanvas3D({
  rooms,
  windows,
  selectedRoomId,
  onSelectRoom,
}: FloorCanvas3DProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const worldRef = useRef<OBC.SimpleScene | null>(null);
  const componentsRef = useRef<OBC.Components | null>(null);
  const modelGroupRef = useRef<THREE.Group>(new THREE.Group());
  const raycasterRef = useRef(new THREE.Raycaster());
  const mouseRef = useRef(new THREE.Vector2());
  const roomMeshMapRef = useRef(new Map<THREE.Mesh, string>());

  // Scene center for camera positioning
  const sceneCenter = useMemo(() => {
    if (rooms.length === 0) return new THREE.Vector3(0, 1.3, 0);
    let cx = 0, cy = 0;
    for (const room of rooms) {
      const c = polygonCenter(room.polygon);
      cx += c.x / 1000;
      cy += c.y / 1000;
    }
    cx /= rooms.length;
    cy /= rooms.length;
    return new THREE.Vector3(cx, 1.3, cy);
  }, [rooms]);

  // Initialize ThatOpen components
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

    // Background
    world.scene.three.background = new THREE.Color(0xf5f5f4);

    // Lighting
    const scene = world.scene.three;
    scene.children
      .filter((c) => c instanceof THREE.Light)
      .forEach((l) => scene.remove(l));

    const ambient = new THREE.AmbientLight(0xffffff, 0.6);
    scene.add(ambient);

    const dir1 = new THREE.DirectionalLight(0xffffff, 0.8);
    dir1.position.set(20, 30, 10);
    dir1.castShadow = true;
    scene.add(dir1);

    const dir2 = new THREE.DirectionalLight(0xffffff, 0.3);
    dir2.position.set(-15, 20, -15);
    scene.add(dir2);

    // Grid
    const grids = components.get(OBC.Grids);
    grids.create(world);

    // Camera position
    world.camera.controls.setLookAt(
      sceneCenter.x + 12, 10, sceneCenter.z + 12,
      sceneCenter.x, sceneCenter.y, sceneCenter.z,
      true,
    );

    // Add model group to scene
    scene.add(modelGroupRef.current);

    worldRef.current = world.scene;

    return () => {
      modelGroupRef.current.removeFromParent();
      components.dispose();
      componentsRef.current = null;
      worldRef.current = null;
    };
  }, [sceneCenter]);

  // Build/update 3D geometry when rooms/windows/selection change
  useEffect(() => {
    const group = modelGroupRef.current;

    // Clear previous
    while (group.children.length > 0) {
      const child = group.children[0]!;
      group.remove(child);
      if (child instanceof THREE.Mesh) {
        child.geometry.dispose();
        if (child.material instanceof THREE.Material) child.material.dispose();
      }
    }
    roomMeshMapRef.current.clear();

    // Build rooms
    for (const room of rooms) {
      const isSelected = room.id === selectedRoomId;
      const floorY = room.floor * (room.height / 1000 + 0.3);
      const h = room.height / 1000;

      // Floor slab
      const floorGeom = createPolygonGeometry(room.polygon, 0);
      const floorColor = isSelected
        ? SELECTED_COLOR
        : (FUNCTION_COLORS[room.function] ?? FUNCTION_COLORS.custom!);
      const floorMat = new THREE.MeshStandardMaterial({
        color: floorColor,
        side: THREE.DoubleSide,
        transparent: true,
        opacity: FLOOR_OPACITY,
        polygonOffset: true,
        polygonOffsetFactor: 1,
        polygonOffsetUnits: 1,
      });
      const floorMesh = new THREE.Mesh(floorGeom, floorMat);
      floorMesh.position.y = floorY + 0.01; // slight offset to avoid z-fighting with grid
      group.add(floorMesh);
      roomMeshMapRef.current.set(floorMesh, room.id);

      // Ceiling
      const ceilGeom = createPolygonGeometry(room.polygon, 0);
      const ceilMat = new THREE.MeshStandardMaterial({
        color: 0xfafaf9,
        side: THREE.DoubleSide,
        transparent: true,
        opacity: CEILING_OPACITY,
      });
      const ceilMesh = new THREE.Mesh(ceilGeom, ceilMat);
      ceilMesh.position.y = floorY + h;
      group.add(ceilMesh);

      // Walls
      const poly = room.polygon;
      const n = poly.length;
      for (let i = 0; i < n; i++) {
        const a = poly[i]!;
        const b = poly[(i + 1) % n]!;

        const ax = a.x / 1000;
        const az = a.y / 1000;
        const bx = b.x / 1000;
        const bz = b.y / 1000;

        const dx = bx - ax;
        const dz = bz - az;
        const len = Math.sqrt(dx * dx + dz * dz);
        if (len < 0.001) continue;

        const nx = -dz / len;
        const nz = dx / len;

        const wallGeom = createWallGeometry(ax, az, bx, bz, h, WALL_THICKNESS, nx, nz);
        const wallMat = new THREE.MeshStandardMaterial({
          color: WALL_COLOR,
          roughness: 0.8,
        });
        const wallMesh = new THREE.Mesh(wallGeom, wallMat);
        wallMesh.position.y = floorY;
        group.add(wallMesh);

        // Check for windows on this wall
        const wallWindows = windows.filter(
          (w) => w.roomId === room.id && w.wallIndex % n === i,
        );
        for (const win of wallWindows) {
          const winGeom = createWindowGeometry(ax, az, bx, bz, win, nx, nz);
          if (winGeom) {
            const winMat = new THREE.MeshStandardMaterial({
              color: WINDOW_COLOR,
              transparent: true,
              opacity: 0.5,
              side: THREE.DoubleSide,
            });
            const winMesh = new THREE.Mesh(winGeom, winMat);
            winMesh.position.y = floorY;
            group.add(winMesh);
          }
        }
      }

      // Room label sprite
      const center = polygonCenter(room.polygon);
      const sprite = createLabelSprite(room.id, room.name, isSelected);
      sprite.position.set(center.x / 1000, floorY + h / 2, center.y / 1000);
      sprite.scale.set(2, 1, 1);
      group.add(sprite);
    }
  }, [rooms, windows, selectedRoomId]);

  // Click handler for room selection
  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const container = containerRef.current;
      if (!container || !worldRef.current) return;

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

      const meshes = Array.from(roomMeshMapRef.current.keys());
      const intersects = raycasterRef.current.intersectObjects(meshes, false);

      if (intersects.length > 0) {
        const roomId = roomMeshMapRef.current.get(intersects[0]!.object as THREE.Mesh);
        if (roomId) {
          onSelectRoom(roomId);
          return;
        }
      }
      onSelectRoom(null);
    },
    [onSelectRoom],
  );

  return (
    <div
      ref={containerRef}
      className="relative h-full w-full"
      onClick={handleClick}
    />
  );
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

function createPolygonGeometry(polygon: Point2D[], _yOffset: number): THREE.BufferGeometry {
  const shape = new THREE.Shape();
  const p0 = polygon[0]!;
  shape.moveTo(p0.x / 1000, p0.y / 1000);
  for (let i = 1; i < polygon.length; i++) {
    const p = polygon[i]!;
    shape.lineTo(p.x / 1000, p.y / 1000);
  }
  shape.closePath();

  const geom = new THREE.ShapeGeometry(shape);
  geom.rotateX(-Math.PI / 2);
  return geom;
}

function createWallGeometry(
  ax: number, az: number,
  bx: number, bz: number,
  h: number, t: number,
  nx: number, nz: number,
): THREE.BufferGeometry {
  const vertices = new Float32Array([
    ax, 0, az,
    bx, 0, bz,
    ax + nx * t, 0, az + nz * t,
    bx + nx * t, 0, bz + nz * t,
    ax, h, az,
    bx, h, bz,
    ax + nx * t, h, az + nz * t,
    bx + nx * t, h, bz + nz * t,
  ]);

  const indices = new Uint16Array([
    2, 3, 7, 2, 7, 6,   // outer
    1, 0, 4, 1, 4, 5,   // inner
    0, 2, 6, 0, 6, 4,   // left cap
    3, 1, 5, 3, 5, 7,   // right cap
    4, 6, 7, 4, 7, 5,   // top
    0, 1, 3, 0, 3, 2,   // bottom
  ]);

  const geom = new THREE.BufferGeometry();
  geom.setAttribute("position", new THREE.BufferAttribute(vertices, 3));
  geom.setIndex(new THREE.BufferAttribute(indices, 1));
  geom.computeVertexNormals();
  return geom;
}

function createWindowGeometry(
  ax: number, az: number,
  bx: number, bz: number,
  win: ModelWindow,
  nx: number, nz: number,
): THREE.BufferGeometry | null {
  const dx = bx - ax;
  const dz = bz - az;
  const len = Math.sqrt(dx * dx + dz * dz);
  if (len < 0.001) return null;

  const ux = dx / len;
  const uz = dz / len;
  const t = WALL_THICKNESS;

  const offset = win.offset / 1000;
  const hw = win.width / 2000;

  const cx = ax + ux * offset + nx * t * 0.5;
  const cz = az + uz * offset + nz * t * 0.5;

  const sillH = 0.8;
  const headH = 2.0;

  const vertices = new Float32Array([
    cx - ux * hw, sillH, cz - uz * hw,
    cx + ux * hw, sillH, cz + uz * hw,
    cx + ux * hw, headH, cz + uz * hw,
    cx - ux * hw, headH, cz - uz * hw,
  ]);
  const indices = new Uint16Array([0, 1, 2, 0, 2, 3]);

  const geom = new THREE.BufferGeometry();
  geom.setAttribute("position", new THREE.BufferAttribute(vertices, 3));
  geom.setIndex(new THREE.BufferAttribute(indices, 1));
  geom.computeVertexNormals();
  return geom;
}

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
