/**
 * Stand-alone IFC4x3 viewer powered by `@thatopen/components`.
 *
 * Mirrors the OCS `ThreeDViewer` pattern: drop een .ifc bestand, krijg een
 * 3D-scene met basic orbit/pan/zoom controls. Geen koppeling met de
 * modellerStore — deze viewer is puur voor inspectie van het bron-IFC.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import * as THREE from "three";
import * as OBC from "@thatopen/components";

interface IfcViewer3DProps {
  /** Optional pre-loaded ArrayBuffer (e.g. from drag-drop), bypasses file picker. */
  initialBuffer?: ArrayBuffer | null;
  initialFileName?: string;
}

export function IfcViewer3D({ initialBuffer, initialFileName }: IfcViewer3DProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const componentsRef = useRef<OBC.Components | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const worldRef = useRef<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadedFileName, setLoadedFileName] = useState<string | null>(null);

  // Initialize viewer once.
  useEffect(() => {
    if (!containerRef.current) return;
    const container = containerRef.current;

    const components = new OBC.Components();
    componentsRef.current = components;

    const worlds = components.get(OBC.Worlds);
    const world = worlds.create<
      OBC.SimpleScene,
      OBC.SimpleCamera,
      OBC.SimpleRenderer
    >();
    world.scene = new OBC.SimpleScene(components);
    world.renderer = new OBC.SimpleRenderer(components, container);
    world.camera = new OBC.SimpleCamera(components);

    components.init();
    world.scene.setup();
    world.scene.three.background = new THREE.Color(0xf5f7fa);

    const grids = components.get(OBC.Grids);
    grids.create(world);

    world.camera.controls.setLookAt(12, 6, 8, 0, 0, 0);
    worldRef.current = world;

    return () => {
      try {
        components.dispose();
      } catch {
        // Already disposed.
      }
      componentsRef.current = null;
      worldRef.current = null;
    };
  }, []);

  const loadIfc = useCallback(async (buffer: ArrayBuffer, fileName: string) => {
    if (!componentsRef.current || !worldRef.current) return;
    setLoading(true);
    setError(null);
    try {
      const ifcLoader = componentsRef.current.get(OBC.IfcLoader);
      // Use the bundled web-ifc.wasm copied to public/wasm/ by frontend postinstall.
      await ifcLoader.setup({
        autoSetWasm: false,
        wasm: { path: "/wasm/", absolute: false },
      });

      const model = await ifcLoader.load(new Uint8Array(buffer), false, fileName);
      // ThatOpen FragmentsModel exposes a THREE Object3D via `.object` (v3.x).
      // Older versions returned an Object3D directly; fall back if needed.
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const obj: THREE.Object3D = (model as any).object ?? (model as unknown as THREE.Object3D);
      worldRef.current.scene.three.add(obj);

      // Fit camera to model bounding box.
      const box = new THREE.Box3().setFromObject(obj);
      const center = box.getCenter(new THREE.Vector3());
      const size = box.getSize(new THREE.Vector3()).length();
      worldRef.current.camera.controls.setLookAt(
        center.x + size,
        center.y + size * 0.5,
        center.z + size,
        center.x,
        center.y,
        center.z,
      );
      setLoadedFileName(fileName);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(`IFC laden mislukt: ${msg}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // Auto-load if parent passed an initial buffer.
  useEffect(() => {
    if (initialBuffer && initialFileName) {
      void loadIfc(initialBuffer, initialFileName);
    }
  }, [initialBuffer, initialFileName, loadIfc]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    file.arrayBuffer().then((buf) => loadIfc(buf, file.name));
    e.target.value = "";
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files?.[0];
    if (file && file.name.toLowerCase().endsWith(".ifc")) {
      file.arrayBuffer().then((buf) => loadIfc(buf, file.name));
    }
  };

  return (
    <div
      ref={containerRef}
      onDragOver={(e) => e.preventDefault()}
      onDrop={handleDrop}
      className="relative h-full w-full bg-surface-2"
    >
      {/* Toolbar */}
      <div className="pointer-events-auto absolute left-3 top-3 z-10 flex items-center gap-2 rounded-md bg-surface/95 px-3 py-1.5 shadow-sm backdrop-blur-sm">
        <button
          onClick={() => fileInputRef.current?.click()}
          className="rounded bg-primary px-3 py-1 text-xs font-medium text-on-primary hover:bg-primary/90"
        >
          📁 Open .ifc...
        </button>
        <span className="text-xs text-scaffold-gray">
          {loadedFileName ? loadedFileName : "of sleep een .ifc bestand"}
        </span>
        <input
          ref={fileInputRef}
          type="file"
          accept=".ifc"
          className="hidden"
          onChange={handleFileChange}
        />
      </div>

      {loading && (
        <div className="absolute left-1/2 top-1/2 z-20 -translate-x-1/2 -translate-y-1/2 rounded-md bg-surface/95 px-5 py-3 text-sm shadow-md backdrop-blur-sm">
          IFC laden...
        </div>
      )}

      {error && (
        <div className="absolute left-1/2 top-1/2 z-20 -translate-x-1/2 -translate-y-1/2 rounded-md bg-red-50 px-5 py-3 text-sm text-red-800 shadow-md">
          {error}
        </div>
      )}
    </div>
  );
}
