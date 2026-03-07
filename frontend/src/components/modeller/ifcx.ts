/**
 * Core IFCX (IFC5) types.
 *
 * Based on the IFC5 alpha spec from buildingSMART/IFC5-development.
 * This file is self-contained — no imports from outside the modeller directory.
 */

// ---------------------------------------------------------------------------
// Document structure
// ---------------------------------------------------------------------------

export interface IfcxHeader {
  id: string;
  ifcxVersion: string;
  dataVersion: string;
  author: string;
  timestamp: string;
}

export interface IfcxImport {
  uri: string;
}

export interface IfcxSchemaField {
  dataType: "String" | "Number" | "Boolean" | "Object" | "Array";
  objectRestrictions?: {
    values: Record<string, IfcxSchemaField>;
  };
  arrayRestrictions?: {
    items: IfcxSchemaField;
  };
}

export interface IfcxSchema {
  value: IfcxSchemaField;
}

export interface IfcxDataEntry {
  path: string;
  children?: Record<string, string>;
  inherits?: Record<string, string>;
  attributes?: Record<string, unknown>;
}

export interface IfcxDocument {
  header: IfcxHeader;
  imports: IfcxImport[];
  schemas: Record<string, IfcxSchema>;
  data: IfcxDataEntry[];
}

// ---------------------------------------------------------------------------
// Standard IFC class codes
// ---------------------------------------------------------------------------

export const IFC_CLASS = {
  Project: "IfcProject",
  Site: "IfcSite",
  Building: "IfcBuilding",
  BuildingStorey: "IfcBuildingStorey",
  Space: "IfcSpace",
  Wall: "IfcWall",
  WallStandardCase: "IfcWallStandardCase",
  Window: "IfcWindow",
  Door: "IfcDoor",
  Slab: "IfcSlab",
  Roof: "IfcRoof",
  Covering: "IfcCovering",
  Opening: "IfcOpeningElement",
} as const;

export type IfcClassCode = (typeof IFC_CLASS)[keyof typeof IFC_CLASS];

// ---------------------------------------------------------------------------
// Standard attribute namespaces
// ---------------------------------------------------------------------------

export const IFCX_NS = {
  /** IFC classification */
  class: "bsi::ifc::class",
  /** IFC properties */
  prop: "bsi::ifc::prop",
  /** Presentation / appearance */
  presentationColor: "bsi::ifc::presentation::diffuseColor",
  presentationOpacity: "bsi::ifc::presentation::opacity",
  /** Material */
  material: "bsi::ifc::material",
  /** USD geometry */
  mesh: "usd::usdgeom::mesh",
  transform: "usd::usdgeom::xformOp",
} as const;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Generate a random UUID v4. */
export function uuid(): string {
  return crypto.randomUUID();
}

/** Create a minimal IFCX document. */
export function createIfcxDocument(opts: {
  id: string;
  author: string;
  schemas?: Record<string, IfcxSchema>;
}): IfcxDocument {
  return {
    header: {
      id: opts.id,
      ifcxVersion: "ifcx_alpha",
      dataVersion: "1.0.0",
      author: opts.author,
      timestamp: new Date().toISOString(),
    },
    imports: [
      { uri: "https://ifcx.dev/@standards.buildingsmart.org/ifc/core/ifc@v5a.ifcx" },
      { uri: "https://ifcx.dev/@standards.buildingsmart.org/ifc/core/prop@v5a.ifcx" },
    ],
    schemas: opts.schemas ?? {},
    data: [],
  };
}

/** Create a data entry with IFC class classification. */
export function classifyEntry(
  path: string,
  classCode: IfcClassCode,
): IfcxDataEntry {
  return {
    path,
    attributes: {
      [IFCX_NS.class]: {
        code: classCode,
        uri: `https://identifier.buildingsmart.org/uri/buildingsmart/ifc/4.3/class/${classCode}`,
      },
    },
  };
}

/** Add a named property to a data entry. */
export function propEntry(
  path: string,
  propName: string,
  value: unknown,
): IfcxDataEntry {
  return {
    path,
    attributes: {
      [`${IFCX_NS.prop}::${propName}`]: value,
    },
  };
}

// ---------------------------------------------------------------------------
// Document composition
// ---------------------------------------------------------------------------

/**
 * Compose multiple IFCX documents/layers into a single resolved dataset.
 * Later entries override earlier ones (layer composition).
 */
export function composeIfcxDocuments(docs: IfcxDocument[]): IfcxDataEntry[] {
  const merged = new Map<string, IfcxDataEntry>();

  for (const doc of docs) {
    for (const entry of doc.data) {
      const existing = merged.get(entry.path);
      if (existing) {
        merged.set(entry.path, {
          path: entry.path,
          children: { ...existing.children, ...entry.children },
          inherits: { ...existing.inherits, ...entry.inherits },
          attributes: { ...existing.attributes, ...entry.attributes },
        });
      } else {
        merged.set(entry.path, { ...entry });
      }
    }
  }

  return Array.from(merged.values());
}
