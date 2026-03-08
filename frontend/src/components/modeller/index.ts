/**
 * @open-aec/modeller — 2D/3D building modeller
 *
 * This directory is self-contained with no imports from outside.
 * It can be extracted into a separate package when ready.
 */

// Components
export { DrawingToolsPanel } from "./DrawingToolsPanel";
export { FloorCanvas } from "./FloorCanvas";
export { FloorCanvas3D } from "./FloorCanvas3D";
export type { RenderMode } from "./FloorCanvas3D";
export { ModellerToolbar } from "./ModellerToolbar";
export { PropertiesPanel } from "./PropertiesPanel";
export { Ribbon } from "./Ribbon";

// Store
export { useModellerStore } from "./modellerStore";
export type { UnderlayImage } from "./modellerStore";

// Geometry helpers
export { polygonArea, signedPolygonArea, polygonCenter, pointInPolygon, segmentsShareEdge, getSharedEdges, offsetPolygon } from "./geometry";

// Types
export type {
  Point2D,
  ModelRoom,
  ModelWindow,
  ModelDoor,
  ModellerTool,
  ViewMode,
  SnapMode,
  SnapSettings,
  Selection,
} from "./types";
export { DEFAULT_SNAP_SETTINGS } from "./types";

// IFCX (IFC5) core types and helpers
export type {
  IfcxHeader,
  IfcxImport,
  IfcxSchema,
  IfcxSchemaField,
  IfcxDataEntry,
  IfcxDocument,
  IfcClassCode,
} from "./ifcx";
export {
  IFC_CLASS,
  IFCX_NS,
  uuid,
  createIfcxDocument,
  classifyEntry,
  propEntry,
  composeIfcxDocuments,
} from "./ifcx";

// IFCX builder (modeller ↔ IFCX conversion)
export { modelToIfcx, ifcxToModel } from "./ifcx-builder";
export type { ModelToIfcxOptions } from "./ifcx-builder";

// Example data (for development/testing)
export { EXAMPLE_ROOMS, EXAMPLE_WINDOWS, FLOOR_LABELS } from "./exampleData";
