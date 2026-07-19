/// <reference lib="webworker" />

/**
 * Web worker wrapper for the IFC space/wall reconstruction pipeline
 * (`lib/ifcReconstruction/pipeline.ts`). Keeps the (CPU-heavy, synchronous
 * per-face raycast loop) pipeline off the main thread so the UI stays
 * responsive while a model is being processed.
 *
 * Message protocol (see types below): post a `ReconstructRequest` with the
 * raw IFC file bytes, receive zero or more `WorkerProgressMessage`s followed
 * by exactly one `WorkerResultMessage` or `WorkerErrorMessage`.
 *
 * `/// <reference lib="webworker" />` scopes this file's ambient globals
 * (`self`, `postMessage`, `onmessage`) to the DedicatedWorkerGlobalScope
 * typings, overriding the project-wide "DOM" lib for this file only -- the
 * standard TS pattern for a dedicated worker in a DOM-lib project (see
 * https://www.typescriptlang.org/docs/handbook/tsconfig-json.html#types-lib).
 */
import { reconstructFromIfc } from "../lib/ifcReconstruction/pipeline";
import type {
  ProgressEvent as ReconstructionProgressEvent,
  ReconstructionOptions,
  ReconstructionResult,
} from "../lib/ifcReconstruction/types";

export interface ReconstructRequest {
  type: "reconstruct";
  buffer: ArrayBuffer;
  options?: ReconstructionOptions;
}

export type WorkerRequest = ReconstructRequest;

export interface WorkerProgressMessage {
  type: "progress";
  event: ReconstructionProgressEvent;
}

export interface WorkerResultMessage {
  type: "result";
  result: ReconstructionResult;
}

export interface WorkerErrorMessage {
  type: "error";
  message: string;
}

export type WorkerResponse = WorkerProgressMessage | WorkerResultMessage | WorkerErrorMessage;

self.onmessage = async (e: MessageEvent<WorkerRequest>) => {
  const req = e.data;
  if (req.type !== "reconstruct") return;

  try {
    const result = await reconstructFromIfc(req.buffer, req.options ?? {}, (event) => {
      const progress: WorkerProgressMessage = { type: "progress", event };
      self.postMessage(progress);
    });
    const response: WorkerResultMessage = { type: "result", result };
    self.postMessage(response);
  } catch (err) {
    const response: WorkerErrorMessage = {
      type: "error",
      message: err instanceof Error ? err.message : String(err),
    };
    self.postMessage(response);
  }
};
