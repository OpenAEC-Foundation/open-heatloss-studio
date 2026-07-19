/**
 * 3D-weergave voor de "IFC-reconstructie (bèta)"-pagina.
 *
 * Eigen three.js/@thatopen-instantiatie (los van `FloorCanvas3D.tsx` — dat
 * bestand raken we niet aan, maar het instantiatiepatroon (OBC.Components +
 * Worlds + SimpleScene/SimpleCamera/SimpleRenderer + Grids + camera-controls)
 * volgen we hier 1-op-1 voor consistentie met de rest van de app.
 *
 * GEOMETRIE-KANTTEKENING: `ReconstructedFace` (fase 2a, `types.ts`) draagt
 * geen vlak-polygon — alleen `centroidMM` + `normal` + `grossAreaM2`. De
 * originele clustered-face-polygon wordt in `pipeline.ts::classifyFace`
 * weggegooid na classificatie en is dus geen onderdeel van het publieke
 * resultaatmodel. Elk vlak wordt hier daarom benaderd als een plat vierkant,
 * gecentreerd op `centroidMM`, loodrecht op `normal`, met zijde
 * `sqrt(grossAreaM2)` — een visuele benadering, geen exacte omtrek. Zie ook
 * de kanttekening in `lib/ifcReconstruction/report.ts`.
 *
 * Coördinaten: de fase-2a pipeline werkt in Z-up mm (zie `geom.ts`). Voor een
 * Y-up three.js-wereld gebruiken we dezelfde conventie als elders in deze
 * app (`FloorCanvas3D.tsx` COORDINATE CONVENTION): world = (x, z, -y), in
 * meter.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import * as THREE from "three";
import * as OBC from "@thatopen/components";

import type { ReconstructionResult, StoreyRef, Vec3 } from "../../lib/ifcReconstruction/types";
import { resolveFaceColor } from "../../lib/ifcReconstruction/report";

interface ReconstructionViewer3DProps {
  result: ReconstructionResult | null;
  selectedRowKey: string | null;
  onSelectRowKey: (key: string | null) => void;
}

const SELECTED_COLOR = 0xf59e0b; // amber-500 — same selection tint as FloorCanvas3D
const QC_OUTLINE_COLOR = 0xef4444; // red-500

/** Z-up mm (fase-2a pipeline frame) -> Three.js Y-up world metres. */
function worldFromMM(mm: Vec3): THREE.Vector3 {
  return new THREE.Vector3(mm[0] / 1000, mm[2] / 1000, -mm[1] / 1000);
}

interface FaceMeshEntry {
  mesh: THREE.Mesh;
  outline: THREE.LineLoop | null;
  rowKey: string;
  spaceName: string;
  zone: string;
  classification: string;
  storeyId: number | null;
  qcFlagged: boolean;
  baseColor: number;
}

/** Build a flat quad centred on `centroidMM`, in the plane perpendicular to
 * `normal`, with side length `sqrt(areaM2)` (metres). See module doc for why
 * this is an approximation rather than the face's real boundary. */
function buildFaceQuad(centroidMM: Vec3, normal: Vec3, areaM2: number): THREE.BufferGeometry {
  const center = worldFromMM(centroidMM);
  const n = new THREE.Vector3(normal[0], normal[2], -normal[1]).normalize();
  const ref = Math.abs(n.x) < 0.9 ? new THREE.Vector3(1, 0, 0) : new THREE.Vector3(0, 1, 0);
  const u = new THREE.Vector3().crossVectors(ref, n).normalize();
  const w = new THREE.Vector3().crossVectors(n, u).normalize();
  const half = Math.sqrt(Math.max(areaM2, 0.01)) / 2;

  const p00 = center.clone().addScaledVector(u, -half).addScaledVector(w, -half);
  const p10 = center.clone().addScaledVector(u, half).addScaledVector(w, -half);
  const p11 = center.clone().addScaledVector(u, half).addScaledVector(w, half);
  const p01 = center.clone().addScaledVector(u, -half).addScaledVector(w, half);

  const positions = new Float32Array([
    p00.x, p00.y, p00.z,
    p10.x, p10.y, p10.z,
    p11.x, p11.y, p11.z,
    p01.x, p01.y, p01.z,
  ]);
  const indices = new Uint16Array([0, 1, 2, 0, 2, 3]);
  const geom = new THREE.BufferGeometry();
  geom.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geom.setIndex(new THREE.BufferAttribute(indices, 1));
  geom.computeVertexNormals();
  return geom;
}

/** Diagonally-striped canvas texture for "gemengd" (ground/exterior 2-tone)
 * faces — cheap two-colour signal without a second overlapping mesh. */
function buildStripeTexture(colorA: string, colorB: string): THREE.CanvasTexture {
  const size = 32;
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext("2d")!;
  ctx.fillStyle = colorA;
  ctx.fillRect(0, 0, size, size);
  ctx.strokeStyle = colorB;
  ctx.lineWidth = size / 4;
  for (let i = -size; i < size * 2; i += size / 2) {
    ctx.beginPath();
    ctx.moveTo(i, 0);
    ctx.lineTo(i + size, size);
    ctx.stroke();
  }
  const tex = new THREE.CanvasTexture(canvas);
  tex.wrapS = THREE.RepeatWrapping;
  tex.wrapT = THREE.RepeatWrapping;
  tex.repeat.set(2, 2);
  return tex;
}

function clearGroup(group: THREE.Group): void {
  while (group.children.length > 0) {
    const child = group.children[0]!;
    group.remove(child);
    if (child instanceof THREE.Mesh || child instanceof THREE.LineLoop || child instanceof THREE.LineSegments) {
      child.geometry.dispose();
      const mat = child.material;
      if (Array.isArray(mat)) mat.forEach((m) => m.dispose());
      else mat.dispose();
    }
  }
}

export function ReconstructionViewer3D({ result, selectedRowKey, onSelectRowKey }: ReconstructionViewer3DProps) {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement>(null);
  const componentsRef = useRef<OBC.Components | null>(null);
  const worldRef = useRef<OBC.World | null>(null);
  const groupRef = useRef<THREE.Group>(new THREE.Group());
  const entriesRef = useRef<FaceMeshEntry[]>([]);
  const raycasterRef = useRef(new THREE.Raycaster());

  const [hoverInfo, setHoverInfo] = useState<{ x: number; y: number; text: string } | null>(null);
  const [hiddenStoreys, setHiddenStoreys] = useState<Set<number>>(new Set());

  const storeys: StoreyRef[] = result?.storeys ?? [];

  const toggleStorey = useCallback((id: number) => {
    setHiddenStoreys((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  // -----------------------------------------------------------------------
  // Init scene once.
  // -----------------------------------------------------------------------
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const components = new OBC.Components();
    componentsRef.current = components;

    const worlds = components.get(OBC.Worlds);
    const world = worlds.create<OBC.SimpleScene, OBC.SimpleCamera, OBC.SimpleRenderer>();
    worldRef.current = world;
    world.scene = new OBC.SimpleScene(components);
    world.renderer = new OBC.SimpleRenderer(components, container);
    world.camera = new OBC.SimpleCamera(components);

    components.init();
    world.scene.setup();

    const scene = world.scene.three;
    scene.background = new THREE.Color(0xf5f5f4);
    scene.children.filter((c) => c instanceof THREE.Light).forEach((l) => scene.remove(l));
    scene.add(new THREE.AmbientLight(0xffffff, 0.9));
    const dir1 = new THREE.DirectionalLight(0xffffff, 0.6);
    dir1.position.set(20, 30, 10);
    scene.add(dir1);
    const dir2 = new THREE.DirectionalLight(0xffffff, 0.25);
    dir2.position.set(-15, 20, -15);
    scene.add(dir2);

    const grids = components.get(OBC.Grids);
    grids.create(world);

    world.camera.controls.setLookAt(15, 12, 15, 0, 1, 0, true);

    scene.add(groupRef.current);

    return () => {
      groupRef.current.removeFromParent();
      components.dispose();
      componentsRef.current = null;
      worldRef.current = null;
    };
  }, []);

  // -----------------------------------------------------------------------
  // Rebuild face meshes when the result changes.
  // -----------------------------------------------------------------------
  useEffect(() => {
    const group = groupRef.current;
    clearGroup(group);
    group.position.set(0, 0, 0);
    entriesRef.current = [];
    if (!result) return;

    result.spaces.forEach((space, spaceIndex) => {
      space.faces.forEach((face, faceIndex) => {
        const rowKey = `${spaceIndex}:${faceIndex}`;
        const color = resolveFaceColor(face);
        const geom = buildFaceQuad(face.centroidMM, face.normal, face.grossAreaM2);

        let mat: THREE.MeshStandardMaterial;
        if (color.fillSecondary) {
          mat = new THREE.MeshStandardMaterial({
            map: buildStripeTexture(color.fill, color.fillSecondary),
            side: THREE.DoubleSide,
          });
        } else {
          mat = new THREE.MeshStandardMaterial({ color: color.fill, side: THREE.DoubleSide });
        }
        const mesh = new THREE.Mesh(geom, mat);
        mesh.userData.rowKey = rowKey;
        group.add(mesh);

        let outline: THREE.LineLoop | null = null;
        if (color.qcFlagged) {
          const pos = geom.getAttribute("position") as THREE.BufferAttribute;
          const outlineGeom = new THREE.BufferGeometry();
          outlineGeom.setAttribute("position", pos.clone());
          const outlineMat = new THREE.LineBasicMaterial({ color: QC_OUTLINE_COLOR, linewidth: 2 });
          outline = new THREE.LineLoop(outlineGeom, outlineMat);
          group.add(outline);
        }

        entriesRef.current.push({
          mesh,
          outline,
          rowKey,
          spaceName: space.name ?? space.longName ?? `Ruimte ${space.id}`,
          zone: face.zone,
          classification: face.classification,
          storeyId: face.storey?.id ?? space.storey?.id ?? null,
          qcFlagged: color.qcFlagged,
          baseColor: new THREE.Color(color.fill).getHex(),
        });
      });
    });

    // Real IFC world coordinates can sit far from the origin (site survey
    // point / RD-coordinates) -- recentre the group on the model's own
    // bounding box and fit the camera to it, instead of assuming the model
    // lives near (0,0,0). Without this the grid renders at the origin while
    // the actual geometry sits far outside the camera frustum -- an "empty"
    // 3D view with real data underneath.
    if (entriesRef.current.length > 0) {
      const box = new THREE.Box3();
      for (const entry of entriesRef.current) box.expandByObject(entry.mesh);
      const center = box.getCenter(new THREE.Vector3());
      const size = box.getSize(new THREE.Vector3());
      group.position.set(-center.x, -center.y, -center.z);

      const maxDim = Math.max(size.x, size.y, size.z, 5);
      const camera = worldRef.current?.camera as OBC.SimpleCamera | undefined;
      if (camera) {
        const dist = maxDim * 1.3;
        camera.controls.setLookAt(dist, dist * 0.7, dist, 0, 0, 0, true);
      }
    }
  }, [result]);

  // -----------------------------------------------------------------------
  // Storey visibility filter.
  // -----------------------------------------------------------------------
  useEffect(() => {
    for (const entry of entriesRef.current) {
      const visible = entry.storeyId === null || !hiddenStoreys.has(entry.storeyId);
      entry.mesh.visible = visible;
      if (entry.outline) entry.outline.visible = visible;
    }
  }, [hiddenStoreys, result]);

  // -----------------------------------------------------------------------
  // Selection highlight.
  // -----------------------------------------------------------------------
  useEffect(() => {
    for (const entry of entriesRef.current) {
      const mat = entry.mesh.material as THREE.MeshStandardMaterial;
      if (entry.rowKey === selectedRowKey) {
        mat.color.set(SELECTED_COLOR);
        mat.map = null;
        mat.needsUpdate = true;
      } else if (!mat.map) {
        mat.color.set(entry.baseColor);
        mat.needsUpdate = true;
      }
    }
  }, [selectedRowKey]);

  const getWorldAndCamera = useCallback((): { world: OBC.World; cam: THREE.Camera } | null => {
    const components = componentsRef.current;
    if (!components) return null;
    const worlds = components.get(OBC.Worlds);
    const list = Array.from(worlds.list.values());
    if (list.length === 0) return null;
    const world = list[0]!;
    return { world, cam: world.camera.three };
  }, []);

  const pickEntry = useCallback(
    (clientX: number, clientY: number): FaceMeshEntry | null => {
      const container = containerRef.current;
      const wc = getWorldAndCamera();
      if (!container || !wc) return null;
      const rect = container.getBoundingClientRect();
      const mouse = new THREE.Vector2(
        ((clientX - rect.left) / rect.width) * 2 - 1,
        -((clientY - rect.top) / rect.height) * 2 + 1,
      );
      raycasterRef.current.setFromCamera(mouse, wc.cam);
      const meshes = entriesRef.current.filter((e) => e.mesh.visible).map((e) => e.mesh);
      const hits = raycasterRef.current.intersectObjects(meshes, false);
      if (hits.length === 0) return null;
      return entriesRef.current.find((e) => e.mesh === hits[0]!.object) ?? null;
    },
    [getWorldAndCamera],
  );

  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (e.button !== 0) return;
      const entry = pickEntry(e.clientX, e.clientY);
      onSelectRowKey(entry?.rowKey ?? null);
    },
    [pickEntry, onSelectRowKey],
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const entry = pickEntry(e.clientX, e.clientY);
      if (!entry) {
        setHoverInfo(null);
        return;
      }
      setHoverInfo({
        x: e.clientX,
        y: e.clientY,
        text: `${entry.spaceName} — ${entry.zone} (${entry.classification})`,
      });
    },
    [pickEntry],
  );

  const legend = useMemo(
    () => [
      { color: "#0d9488", labelKey: "ifcReconstruction.legend.exterieur" },
      { color: "#78350f", labelKey: "ifcReconstruction.legend.grond" },
      { color: "#6ee7b7", labelKey: "ifcReconstruction.legend.buurruimte" },
      { color: "#9ca3af", labelKey: "ifcReconstruction.legend.onbepaald" },
      { color: "#3b82f6", labelKey: "ifcReconstruction.legend.raam" },
      { color: "#8b5cf6", labelKey: "ifcReconstruction.legend.deur" },
    ],
    [],
  );

  return (
    <div
      ref={containerRef}
      className="relative h-full w-full min-h-[420px]"
      onClick={handleClick}
      onMouseMove={handleMouseMove}
      onMouseLeave={() => setHoverInfo(null)}
    >
      {/* Legend */}
      <div className="absolute right-3 top-3 z-10 rounded-lg bg-surface-alt/95 p-2.5 shadow-lg backdrop-blur-sm text-[10px] select-none">
        <div className="mb-1 font-semibold text-on-surface-secondary text-[11px]">
          {t("ifcReconstruction.legend.title")}
        </div>
        <div className="flex flex-col gap-0.5">
          {legend.map((item) => (
            <div key={item.labelKey} className="flex items-center gap-1.5">
              <span className="h-2.5 w-4 rounded-sm" style={{ backgroundColor: item.color }} />
              <span className="text-on-surface-secondary">{t(item.labelKey)}</span>
            </div>
          ))}
          <div className="flex items-center gap-1.5 pt-1">
            <span className="h-2.5 w-4 rounded-sm border-2" style={{ borderColor: "#ef4444" }} />
            <span className="text-on-surface-secondary">{t("ifcReconstruction.legend.qc")}</span>
          </div>
        </div>
      </div>

      {/* Storey filter */}
      {storeys.length > 0 && (
        <div className="absolute left-3 top-3 z-10 rounded-lg bg-surface-alt/95 p-2.5 shadow-lg backdrop-blur-sm text-[10px] select-none">
          <div className="mb-1 font-semibold text-on-surface-secondary text-[11px]">
            {t("ifcReconstruction.storeyFilter")}
          </div>
          <div className="flex flex-col gap-0.5">
            {storeys.map((s) => (
              <label key={s.id} className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="checkbox"
                  checked={!hiddenStoreys.has(s.id)}
                  onChange={() => toggleStorey(s.id)}
                  className="h-3 w-3"
                />
                <span className="text-on-surface-secondary">{s.name ?? `#${s.id}`}</span>
              </label>
            ))}
          </div>
        </div>
      )}

      {/* Hover tooltip */}
      {hoverInfo && (
        <div
          className="pointer-events-none fixed z-50 rounded bg-black/80 px-2 py-1 text-[11px] text-white"
          style={{ left: hoverInfo.x + 12, top: hoverInfo.y + 12 }}
        >
          {hoverInfo.text}
        </div>
      )}

      {!result && (
        <div className="absolute inset-0 flex items-center justify-center text-sm text-on-surface-muted">
          {t("ifcReconstruction.no3dData")}
        </div>
      )}
    </div>
  );
}
