/**
 * Main-thread convenience wrapper around `ifcReconstruction.worker.ts`.
 *
 * Fase 2b (3D view + oppervlaktenlijst) is expected to call
 * `runIfcReconstructionInWorker` once per uploaded/opened IFC file: it
 * spawns a fresh worker, forwards progress events, and resolves with the
 * full `ReconstructionResult` (or rejects on a worker-reported error).
 * The worker is terminated after settling either way, so callers don't need
 * to manage its lifecycle.
 */
import type {
  ReconstructRequest,
  WorkerResponse,
} from "./ifcReconstruction.worker";
import type { ProgressCallback, ReconstructionOptions, ReconstructionResult } from "../lib/ifcReconstruction/types";

/**
 * Run the IFC reconstruction pipeline in a dedicated worker.
 *
 * @param file The IFC file (or its already-read bytes).
 * @param opts Reconstruction options (maaiveld override).
 * @param onProgress Optional progress callback, invoked on the main thread.
 */
export function runIfcReconstructionInWorker(
  file: File | ArrayBuffer,
  opts: ReconstructionOptions = {},
  onProgress?: ProgressCallback,
): Promise<ReconstructionResult> {
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL("./ifcReconstruction.worker.ts", import.meta.url), {
      type: "module",
    });

    let settled = false;
    function settle(fn: () => void): void {
      if (settled) return;
      settled = true;
      fn();
      worker.terminate();
    }

    worker.onmessage = (e: MessageEvent<WorkerResponse>) => {
      const msg = e.data;
      if (msg.type === "progress") {
        onProgress?.(msg.event);
      } else if (msg.type === "result") {
        settle(() => resolve(msg.result));
      } else if (msg.type === "error") {
        settle(() => reject(new Error(msg.message)));
      }
    };
    worker.onerror = (ev: ErrorEvent) => {
      settle(() => reject(new Error(ev.message || "IFC reconstruction worker crashed")));
    };

    void readAsArrayBuffer(file).then((buffer) => {
      const req: ReconstructRequest = { type: "reconstruct", buffer, options: opts };
      worker.postMessage(req, [buffer]);
    });
  });
}

async function readAsArrayBuffer(file: File | ArrayBuffer): Promise<ArrayBuffer> {
  if (file instanceof ArrayBuffer) return file;
  return await file.arrayBuffer();
}
