/**
 * IFC4X3 STEP file generator voor warmteverlies-projecten.
 *
 * Mirror van Open Calc Studio's `services/ifc/ifcCostGenerator.ts` patroon —
 * produceert een geldige ISO-10303-21 STEP-file met de spatial hierarchy
 * (IfcProject → IfcSite → IfcBuilding → IfcSpace per Room) plus IfcWall /
 * IfcSlab / IfcRoof children per construction. Werkelijke geometrie zit
 * niet in deze versie (zou IfcExtrudedAreaSolid + IfcRectangleProfileDef
 * etc. vereisen) — voor visualisatie is de spatial hierarchy + properties
 * het belangrijkste. ISSO 51 norm-data komt mee als IfcPropertySet.
 */
import type { Project, Room, ConstructionElement } from "../types";

/** Generate a 22-character compressed-base64 IFC GUID (not RFC 4122 — IFC's own). */
const IFC_GUID_CHARS =
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$";

export function generateIfcGuid(): string {
  // Use random bytes; IFC GUIDs are 22 chars, base64-like.
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  let result = "";
  for (let i = 0; i < 22; i++) {
    const idx = bytes[Math.floor((i * bytes.length) / 22)]! % IFC_GUID_CHARS.length;
    result += IFC_GUID_CHARS[idx];
  }
  return result;
}

/** Encode a string for STEP (escape quotes + backslashes). */
function encodeStepString(s: string | undefined | null): string {
  if (!s) return "";
  return s.replace(/\\/g, "\\\\").replace(/'/g, "\\'");
}

/** Format a float for STEP (no trailing zeros, dot-separator). */
function formatStepFloat(n: number | undefined | null): string {
  if (n == null || !Number.isFinite(n)) return "$";
  return n.toString();
}

function isoTimestamp(): string {
  return new Date().toISOString().replace(/\.\d+Z$/, "");
}

interface StepLine {
  id: number;
  entity: string;
}

/**
 * Build an IFC4X3 STEP file from a Project. Output is the full text content
 * (newline-joined STEP lines). Used by the IFC tab to show "what would my
 * project look like as IFC?".
 */
export function generateIfcStepFromProject(project: Project): string {
  const lines: StepLine[] = [];
  let nextId = 1;
  const getId = () => nextId++;

  const projectName = project.info.name || "Heat Loss Project";
  const author = project.info.engineer || "Open Heatloss Studio";
  const projectGuid = generateIfcGuid();
  const siteGuid = generateIfcGuid();
  const buildingGuid = generateIfcGuid();

  const ts = isoTimestamp();

  const header = [
    "ISO-10303-21;",
    "HEADER;",
    `FILE_DESCRIPTION(('ViewDefinition [HeatLossView]'),'2;1');`,
    `FILE_NAME('${encodeStepString(projectName)}.ifc','${ts}',('${encodeStepString(author)}'),(''),'',' ','');`,
    "FILE_SCHEMA(('IFC4X3'));",
    "ENDSEC;",
    "DATA;",
  ].join("\n");

  // --- Foundation entities ---
  const orgId = getId();
  lines.push({ id: orgId, entity: `IFCORGANIZATION($,'Open Heatloss Studio',$,$,$)` });

  const appId = getId();
  lines.push({
    id: appId,
    entity: `IFCAPPLICATION(#${orgId},'0.1.1','Open Heatloss Studio','OHS')`,
  });

  const personId = getId();
  lines.push({
    id: personId,
    entity: `IFCPERSON($,'${encodeStepString(author)}','',$,$,$,$,$)`,
  });

  const pOrgId = getId();
  lines.push({ id: pOrgId, entity: `IFCPERSONANDORGANIZATION(#${personId},#${orgId},$)` });

  const ownerHistId = getId();
  lines.push({
    id: ownerHistId,
    entity: `IFCOWNERHISTORY(#${pOrgId},#${appId},$,.ADDED.,$,$,$,${Math.floor(Date.now() / 1000)})`,
  });

  // --- Units ---
  const unitLengthId = getId();
  lines.push({ id: unitLengthId, entity: `IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.)` });

  const unitAreaId = getId();
  lines.push({ id: unitAreaId, entity: `IFCSIUNIT(*,.AREAUNIT.,$,.SQUARE_METRE.)` });

  const unitVolumeId = getId();
  lines.push({ id: unitVolumeId, entity: `IFCSIUNIT(*,.VOLUMEUNIT.,$,.CUBIC_METRE.)` });

  const unitTimeId = getId();
  lines.push({ id: unitTimeId, entity: `IFCSIUNIT(*,.TIMEUNIT.,$,.SECOND.)` });

  const unitTempId = getId();
  lines.push({
    id: unitTempId,
    entity: `IFCSIUNIT(*,.THERMODYNAMICTEMPERATUREUNIT.,$,.KELVIN.)`,
  });

  const unitPowerId = getId();
  lines.push({ id: unitPowerId, entity: `IFCSIUNIT(*,.POWERUNIT.,$,.WATT.)` });

  const unitAssignId = getId();
  lines.push({
    id: unitAssignId,
    entity: `IFCUNITASSIGNMENT((#${unitLengthId},#${unitAreaId},#${unitVolumeId},#${unitTimeId},#${unitTempId},#${unitPowerId}))`,
  });

  // --- Geometric Representation Context ---
  const ctxId = getId();
  lines.push({
    id: ctxId,
    entity: `IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.E-5,$,$)`,
  });

  // --- IfcProject ---
  const projectId = getId();
  lines.push({
    id: projectId,
    entity: `IFCPROJECT('${projectGuid}',#${ownerHistId},'${encodeStepString(projectName)}','ISSO 51 warmteverliesberekening',$,$,$,(#${ctxId}),#${unitAssignId})`,
  });

  // --- IfcSite ---
  const siteId = getId();
  lines.push({
    id: siteId,
    entity: `IFCSITE('${siteGuid}',#${ownerHistId},'Site',$,$,$,$,$,.ELEMENT.,$,$,$,$,$)`,
  });

  // --- IfcBuilding ---
  const buildingId = getId();
  lines.push({
    id: buildingId,
    entity: `IFCBUILDING('${buildingGuid}',#${ownerHistId},'${encodeStepString(projectName)}','Building',$,$,$,$,.ELEMENT.,$,$,$)`,
  });

  // --- IfcSpaces (one per Room) ---
  const spaceIds: { id: number; room: Room; spaceGuid: string }[] = [];
  for (const room of project.rooms) {
    const spaceGuid = generateIfcGuid();
    const id = getId();
    spaceIds.push({ id, room, spaceGuid });
    lines.push({
      id,
      entity: `IFCSPACE('${spaceGuid}',#${ownerHistId},'${encodeStepString(room.name)}','${encodeStepString(String(room.function))}',$,$,$,$,.ELEMENT.,.INTERNAL.,${formatStepFloat(room.height ?? 2.6)})`,
    });
  }

  // --- IfcWall / IfcSlab / IfcRoof per construction ---
  const elementsBySpace: Map<number, number[]> = new Map();
  for (const sp of spaceIds) {
    const elemIds: number[] = [];
    for (const c of sp.room.constructions) {
      const elemId = getId();
      const elemGuid = generateIfcGuid();
      const ifcType = ifcTypeForConstruction(c);
      lines.push({
        id: elemId,
        entity: `${ifcType}('${elemGuid}',#${ownerHistId},'${encodeStepString(c.description)}','${encodeStepString(c.boundary_type)}',$,$,$,$,$)`,
      });

      // IfcPropertySet with isso51 properties
      const psetId = generateIsso51PropertySet(lines, getId, ownerHistId, c, elemId);
      void psetId;
      elemIds.push(elemId);
    }
    elementsBySpace.set(sp.id, elemIds);
  }

  // --- IfcRelAggregates (Project → Site → Building → Spaces) ---
  const relProjSiteId = getId();
  lines.push({
    id: relProjSiteId,
    entity: `IFCRELAGGREGATES('${generateIfcGuid()}',#${ownerHistId},$,$,#${projectId},(#${siteId}))`,
  });

  const relSiteBuildId = getId();
  lines.push({
    id: relSiteBuildId,
    entity: `IFCRELAGGREGATES('${generateIfcGuid()}',#${ownerHistId},$,$,#${siteId},(#${buildingId}))`,
  });

  if (spaceIds.length > 0) {
    const relBuildSpacesId = getId();
    const spaceRefs = spaceIds.map((s) => `#${s.id}`).join(",");
    lines.push({
      id: relBuildSpacesId,
      entity: `IFCRELAGGREGATES('${generateIfcGuid()}',#${ownerHistId},$,$,#${buildingId},(${spaceRefs}))`,
    });
  }

  // --- IfcRelContainedInSpatialStructure (Space → child elements) ---
  for (const sp of spaceIds) {
    const elems = elementsBySpace.get(sp.id);
    if (!elems || elems.length === 0) continue;
    const relId = getId();
    const refs = elems.map((e) => `#${e}`).join(",");
    lines.push({
      id: relId,
      entity: `IFCRELCONTAINEDINSPATIALSTRUCTURE('${generateIfcGuid()}',#${ownerHistId},$,$,(${refs}),#${sp.id})`,
    });
  }

  // --- Final assembly ---
  const dataLines = lines.map((l) => `#${l.id}=${l.entity};`);
  return [header, ...dataLines, "ENDSEC;", "END-ISO-10303-21;"].join("\n");
}

function ifcTypeForConstruction(c: ConstructionElement): string {
  switch (c.vertical_position) {
    case "floor":
      return "IFCSLAB";
    case "ceiling":
      return c.boundary_type === "exterior" ? "IFCROOF" : "IFCSLAB";
    case "wall":
    default:
      return "IFCWALL";
  }
}

/**
 * Append an IfcPropertySet with isso51:: data on a construction element.
 * Returns the IfcRelDefinesByProperties id.
 */
function generateIsso51PropertySet(
  lines: StepLine[],
  getId: () => number,
  ownerHistId: number,
  c: ConstructionElement,
  elementId: number,
): number {
  // IfcPropertySingleValue per attribute
  const propIds: number[] = [];

  const areaProp = getId();
  lines.push({
    id: areaProp,
    entity: `IFCPROPERTYSINGLEVALUE('isso51::area','${encodeStepString(c.description)}',IFCAREAMEASURE(${formatStepFloat(c.area)}),$)`,
  });
  propIds.push(areaProp);

  const uProp = getId();
  lines.push({
    id: uProp,
    entity: `IFCPROPERTYSINGLEVALUE('isso51::u_value','U-waarde',IFCTHERMALTRANSMITTANCEMEASURE(${formatStepFloat(c.u_value)}),$)`,
  });
  propIds.push(uProp);

  const boundaryProp = getId();
  lines.push({
    id: boundaryProp,
    entity: `IFCPROPERTYSINGLEVALUE('isso51::boundary_type','Boundary type',IFCLABEL('${encodeStepString(c.boundary_type)}'),$)`,
  });
  propIds.push(boundaryProp);

  const matProp = getId();
  lines.push({
    id: matProp,
    entity: `IFCPROPERTYSINGLEVALUE('isso51::material_type','Material type',IFCLABEL('${encodeStepString(c.material_type)}'),$)`,
  });
  propIds.push(matProp);

  // IfcPropertySet bundling them
  const psetId = getId();
  const propRefs = propIds.map((p) => `#${p}`).join(",");
  lines.push({
    id: psetId,
    entity: `IFCPROPERTYSET('${generateIfcGuid()}',#${ownerHistId},'Pset_isso51',$,(${propRefs}))`,
  });

  // IfcRelDefinesByProperties linking pset to element
  const relId = getId();
  lines.push({
    id: relId,
    entity: `IFCRELDEFINESBYPROPERTIES('${generateIfcGuid()}',#${ownerHistId},$,$,(#${elementId}),#${psetId})`,
  });

  return relId;
}
