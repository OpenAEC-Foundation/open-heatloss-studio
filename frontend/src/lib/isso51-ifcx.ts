/**
 * ISSO 51 extensions for IFCX.
 *
 * Defines the `isso51::` namespace for warmteverliesberekening results.
 * These are custom schemas that extend an IFCX document without breaking
 * IFC5 validation — custom namespaces are explicitly supported by the spec.
 *
 * Usage:
 *   1. gebouw.ifcx       — building model (geometry, spaces, walls)
 *   2. constructies.ifcx  — construction library (Rc values, material layers)
 *   3. berekening.ifcx    — calculation results as overlay on the model
 */

import type {
  IfcxDocument,
  IfcxDataEntry,
  IfcxSchema,
  IfcxSchemaField,
} from "../components/modeller/ifcx";
import { createIfcxDocument, uuid } from "../components/modeller/ifcx";

// ---------------------------------------------------------------------------
// ISSO51 namespace constants
// ---------------------------------------------------------------------------

export const ISSO51_NS = {
  /** Transmission loss per element or space */
  transmission: "isso51::calc::transmission",
  /** Ventilation loss per space */
  ventilation: "isso51::calc::ventilation",
  /** Reheat capacity per space */
  reheat: "isso51::calc::reheat",
  /** Total results per space */
  spaceResult: "isso51::calc::result",
  /** Construction Rc value on walls/slabs */
  construction: "isso51::construction",
  /** Material layers on walls/slabs */
  layers: "isso51::construction::layers",
  /** Report metadata on building */
  report: "isso51::report",
  /** Design conditions on project */
  conditions: "isso51::conditions",
} as const;

// ---------------------------------------------------------------------------
// Typed data interfaces
// ---------------------------------------------------------------------------

export interface Isso51Transmission {
  /** H_T in W/K */
  H_T: number;
  /** phi_T in W */
  phi_T: number;
  /** Rc in m²K/W (on construction elements) */
  Rc?: number;
  /** U-value in W/(m²K) */
  U?: number;
}

export interface Isso51Ventilation {
  /** H_V in W/K */
  H_V: number;
  /** phi_V in W */
  phi_V: number;
  /** Specific infiltration in dm³/(s·m²) */
  qi_spec?: number;
}

export interface Isso51Reheat {
  /** phi_RH in W */
  phi_RH: number;
  /** f_RH factor */
  f_RH: number;
}

export interface Isso51SpaceResult {
  /** Total design heat loss in W */
  phi_HL: number;
  /** Transmission loss in W */
  phi_T: number;
  /** Ventilation loss in W */
  phi_V: number;
  /** Reheat in W */
  phi_RH: number;
  /** Design temperature in °C */
  theta_int: number;
}

export interface Isso51Construction {
  /** Rc in m²K/W */
  Rc: number;
  /** U-value in W/(m²K) */
  U: number;
  /** Construction name */
  name: string;
  /** Library reference id */
  libraryId?: string;
}

export interface Isso51MaterialLayer {
  /** Material name */
  name: string;
  /** Thickness in mm */
  thickness: number;
  /** Lambda in W/(m·K) */
  lambda: number;
  /** R-value of this layer in m²K/W */
  R: number;
}

export interface Isso51Report {
  /** When the calculation was generated */
  generatedAt: string;
  /** ISSO norm version */
  normVersion: string;
  /** Total building heat loss in W */
  totalLoss: number;
  /** Total floor area in m² */
  totalArea: number;
  /** Specific loss in W/m² */
  specificLoss: number;
}

export interface Isso51Conditions {
  /** Outdoor design temperature in °C */
  theta_e: number;
  /** Wind class */
  windClass: string;
  /** Location description */
  location: string;
  /** Heating regime */
  regime: string;
}

// ---------------------------------------------------------------------------
// Schema definitions (for IFCX document)
// ---------------------------------------------------------------------------

const numberField: IfcxSchemaField = { dataType: "Number" };
const stringField: IfcxSchemaField = { dataType: "String" };

function objectSchema(fields: Record<string, IfcxSchemaField>): IfcxSchema {
  return {
    value: {
      dataType: "Object",
      objectRestrictions: { values: fields },
    },
  };
}

export const ISSO51_SCHEMAS: Record<string, IfcxSchema> = {
  [ISSO51_NS.transmission]: objectSchema({
    H_T: numberField,
    phi_T: numberField,
    Rc: numberField,
    U: numberField,
  }),
  [ISSO51_NS.ventilation]: objectSchema({
    H_V: numberField,
    phi_V: numberField,
    qi_spec: numberField,
  }),
  [ISSO51_NS.reheat]: objectSchema({
    phi_RH: numberField,
    f_RH: numberField,
  }),
  [ISSO51_NS.spaceResult]: objectSchema({
    phi_HL: numberField,
    phi_T: numberField,
    phi_V: numberField,
    phi_RH: numberField,
    theta_int: numberField,
  }),
  [ISSO51_NS.construction]: objectSchema({
    Rc: numberField,
    U: numberField,
    name: stringField,
    libraryId: stringField,
  }),
  [ISSO51_NS.layers]: {
    value: {
      dataType: "Array",
      arrayRestrictions: {
        items: {
          dataType: "Object",
          objectRestrictions: {
            values: {
              name: stringField,
              thickness: numberField,
              lambda: numberField,
              R: numberField,
            },
          },
        },
      },
    },
  },
  [ISSO51_NS.report]: objectSchema({
    generatedAt: stringField,
    normVersion: stringField,
    totalLoss: numberField,
    totalArea: numberField,
    specificLoss: numberField,
  }),
  [ISSO51_NS.conditions]: objectSchema({
    theta_e: numberField,
    windClass: stringField,
    location: stringField,
    regime: stringField,
  }),
};

// ---------------------------------------------------------------------------
// Builder: create an ISSO51 calculation results overlay document
// ---------------------------------------------------------------------------

export function createCalculationOverlay(opts: {
  author: string;
  conditions: Isso51Conditions;
  /** Project-level IFCX path (UUID) */
  projectPath: string;
  /** Building-level IFCX path (UUID) */
  buildingPath: string;
  /** Per-space results keyed by space IFCX path */
  spaceResults: Map<string, Isso51SpaceResult>;
  /** Report summary */
  report: Isso51Report;
}): IfcxDocument {
  const doc = createIfcxDocument({
    id: uuid(),
    author: opts.author,
    schemas: ISSO51_SCHEMAS,
  });

  // Project conditions
  doc.data.push({
    path: opts.projectPath,
    attributes: {
      [ISSO51_NS.conditions]: opts.conditions,
    },
  });

  // Per-space results
  for (const [spacePath, result] of opts.spaceResults) {
    doc.data.push({
      path: spacePath,
      attributes: {
        [ISSO51_NS.spaceResult]: result,
        [ISSO51_NS.transmission]: {
          H_T: result.phi_T / (result.theta_int - opts.conditions.theta_e),
          phi_T: result.phi_T,
        } satisfies Partial<Isso51Transmission>,
        [ISSO51_NS.ventilation]: {
          H_V: result.phi_V / (result.theta_int - opts.conditions.theta_e),
          phi_V: result.phi_V,
        } satisfies Partial<Isso51Ventilation>,
        [ISSO51_NS.reheat]: {
          phi_RH: result.phi_RH,
          f_RH: result.phi_RH > 0 ? result.phi_RH / result.phi_T : 0,
        } satisfies Isso51Reheat,
      },
    });
  }

  // Building-level report
  doc.data.push({
    path: opts.buildingPath,
    attributes: {
      [ISSO51_NS.report]: opts.report,
    },
  });

  return doc;
}

// ---------------------------------------------------------------------------
// Builder: create a construction library overlay document
// ---------------------------------------------------------------------------

export function createConstructionOverlay(opts: {
  author: string;
  /** Element IFCX path → construction + layers */
  elements: Map<string, {
    construction: Isso51Construction;
    layers: Isso51MaterialLayer[];
  }>;
}): IfcxDocument {
  const doc = createIfcxDocument({
    id: uuid(),
    author: opts.author,
    schemas: {
      [ISSO51_NS.construction]: ISSO51_SCHEMAS[ISSO51_NS.construction]!,
      [ISSO51_NS.layers]: ISSO51_SCHEMAS[ISSO51_NS.layers]!,
    },
  });

  for (const [elementPath, { construction, layers }] of opts.elements) {
    doc.data.push({
      path: elementPath,
      attributes: {
        [ISSO51_NS.construction]: construction,
        [ISSO51_NS.layers]: layers,
      },
    });
  }

  return doc;
}

// ---------------------------------------------------------------------------
// Parser: extract ISSO51 data from a composed IFCX dataset
// ---------------------------------------------------------------------------

export function extractIsso51Data(entries: IfcxDataEntry[]): {
  conditions: Isso51Conditions | null;
  report: Isso51Report | null;
  spaceResults: Map<string, Isso51SpaceResult>;
  constructions: Map<string, Isso51Construction>;
} {
  let conditions: Isso51Conditions | null = null;
  let report: Isso51Report | null = null;
  const spaceResults = new Map<string, Isso51SpaceResult>();
  const constructions = new Map<string, Isso51Construction>();

  for (const entry of entries) {
    if (!entry.attributes) continue;

    const cond = entry.attributes[ISSO51_NS.conditions] as Isso51Conditions | undefined;
    if (cond) conditions = cond;

    const rep = entry.attributes[ISSO51_NS.report] as Isso51Report | undefined;
    if (rep) report = rep;

    const result = entry.attributes[ISSO51_NS.spaceResult] as Isso51SpaceResult | undefined;
    if (result) spaceResults.set(entry.path, result);

    const constr = entry.attributes[ISSO51_NS.construction] as Isso51Construction | undefined;
    if (constr) constructions.set(entry.path, constr);
  }

  return { conditions, report, spaceResults, constructions };
}
