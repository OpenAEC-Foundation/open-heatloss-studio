/**
 * @open-aec/modeller — 2D/3D building modeller
 *
 * This directory is self-contained with no imports from outside.
 * It can be extracted into a separate package when ready.
 */

// Components
export { FloorCanvas } from "./FloorCanvas";
export { FloorCanvas3D } from "./FloorCanvas3D";
export type { RenderMode } from "./FloorCanvas3D";
export { PropertiesPanel } from "./PropertiesPanel";
export { Ribbon } from "./Ribbon";

// Store
export { useModellerStore } from "./modellerStore";
export type { UnderlayImage } from "./modellerStore";

// Geometry helpers
export { polygonArea, signedPolygonArea, polygonCenter, pointInPolygon, segmentsShareEdge, getSharedEdges, offsetPolygon, splitPolygon, mergePolygons, removeCollinearVertices } from "./geometry";

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
  WallBoundaryType,
  ProjectConstruction,
} from "./types";
export { DEFAULT_SNAP_SETTINGS, BOUNDARY_TYPE_LABELS } from "./types";

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

// IFC import (web-ifc based, IFC2x3/IFC4 STEP files)
export { importIfcFile } from "./ifc-import";
export type { IfcImportResult } from "./ifc-import";

// IFC wall type extraction
export { extractWallTypes, extractWallTypesFromFile } from "./ifc-wall-types";
export type { IfcWallTypeInfo, IfcWallTypeLayer } from "./ifc-wall-types";

// Project library UI
export { ProjectLibraryPanel } from "./ProjectLibraryPanel";
export { IfcWallTypeReview } from "./IfcWallTypeReview";

// Example data (for development/testing)
export { EXAMPLE_ROOMS, EXAMPLE_WINDOWS, FLOOR_LABELS } from "./exampleData";
