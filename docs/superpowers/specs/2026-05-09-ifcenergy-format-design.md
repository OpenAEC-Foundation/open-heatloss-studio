# `.ifcenergy` IFCX file format — design

**Datum:** 2026-05-09
**Status:** Design — klaar voor planning + implementatie
**Branch:** `claude/laughing-kirch-752da4`
**Volgt op:** PR A (modeller-table sync fix in `.isso51.json` envelope)

## Doel

Vervang het huidige proprietary `.isso51.json` envelope-formaat door een open IFCX-gebaseerd formaat met extensie `.ifcenergy`. Het nieuwe formaat:
- Is een geldige IFCX (IFC5 alpha) document
- Gebruikt de bestaande `isso51::` namespace voor norm-data + nieuwe `isso51::modeller::` namespace voor 2D/3D-geometrie
- Vervangt `.isso51.json` als default save-format
- Houdt `.isso51.json` (legacy) read-only ondersteund — gebruikers kunnen oude bestanden openen, niet meer naar dat formaat exporteren

## Beslissingen (vastgelegd in brainstorm)

| Onderwerp | Keuze |
|---|---|
| File extensie nieuw | `.ifcenergy` |
| File extensie legacy | `.isso51.json` (read-only ondersteund) |
| Onderliggend formaat | IFCX (IFC5 alpha) |
| Namespace voor norm-data | `isso51::` (ongewijzigd) |
| Namespace voor modeller-geometrie | `isso51::modeller::` (nieuw) |
| Format-detectie | Gebaseerd op bestand-shape (zie 4) |
| Reference-implementatie | `OpenAEC-Foundation/open-calc-studio/src/services/file/nativeFileService.ts` |

## Niet-doelen

- IFCX native geometry (`IfcShapeRepresentation`, `IfcExtrudedAreaSolid`, etc.). Wij gebruiken `isso51::modeller::*` als simpele JSON-attributen op IfcSpace/IfcWindow/IfcDoor entries. Native IFCX geometry kan in een latere PR voor interop met andere tools.
- Schrijven naar `.isso51.json` (legacy export). Alleen lezen.
- Migratie van server-stored projecten (Postgres). Die gebruiken hun eigen `Project` schema, los van file-formaat.
- Schema-validatie tegen IFCX schema-definitie. We doen pragmatische `serde_json` parsing met required-field checks.
- IFCX-import van externe tools (Revit, ArchiCAD). Buiten scope; wij produceren én lezen alleen onze eigen `.ifcenergy` files.

## 1. File-naam conventie

```
<safe-project-name>.ifcenergy
```

`<safe-project-name>` = project.info.name met niet-ASCII tekens vervangen (zoals al gebeurt in `exportProject`).

Voorbeeld: `Memeleiland Kavel 4.ifcenergy`.

## 2. IFCX document-structuur

Het `.ifcenergy` bestand bevat één geldige IFCX document met:

```
IfcxDocument {
  header: { id, ifcxVersion: "ifcx_alpha", dataVersion: "1.0.0", author, timestamp }
  imports: [
    "https://ifcx.dev/@standards.buildingsmart.org/ifc/core/ifc@v5a.ifcx",
    "https://ifcx.dev/@standards.buildingsmart.org/ifc/core/prop@v5a.ifcx"
  ]
  schemas: { /* schema's voor isso51:: + isso51::modeller:: namespaces */ }
  data: [
    /* IfcProject met isso51::conditions, isso51::ventilation, isso51::project_info */
    /* IfcSite */
    /* IfcBuilding met isso51::building en isso51::report (na berekening) */
    /* IfcSpace per calc Room met isso51::room en evt. isso51::modeller::room */
    /* Construction children van IfcSpace met isso51::construction (en evt. ::layers, ::ground) */
    /* IfcWindow / IfcDoor children met isso51::modeller::window / ::door */
  ]
}
```

### 2.1 Linking-strategie (calc Room ↔ ModelRoom)

Calc rooms (uit `Project.rooms`) en modeller rooms (uit `useModellerStore.rooms`) hebben **onafhankelijke ID-namespaces**. Linken kan op naam of expliciet via UI.

**Voor PR B:** name-based linking (heuristisch). Als `ModelRoom.name == calcRoom.name`, dan worden ze gekoppeld aan dezelfde IfcSpace entry. Anders krijgt elk een eigen IfcSpace.

```
Voor elke calcRoom:
  IfcSpace path = uuid()
  attributes:
    - bsi::ifc::class = "IfcSpace"
    - bsi::ifc::prop::Name = calcRoom.name
    - isso51::room = { function, floor_area, height, ... }
    - constructions als children (zie bestaande project_to_ifcx)
  ALS er een ModelRoom met dezelfde naam is:
    - isso51::modeller::room = { polygon, floor, height, elevation, ... }
    - windows/doors van die ModelRoom als children met isso51::modeller::window/::door

Voor elke ModelRoom zonder calc-match:
  IfcSpace path = uuid()
  attributes:
    - bsi::ifc::class = "IfcSpace"
    - bsi::ifc::prop::Name = modelRoom.name
    - isso51::modeller::room = { polygon, ... }
    - (geen isso51::room — alleen geometrie, geen calc-data)
```

Risico: namen-collisie (twee rooms met dezelfde naam) — twee calc-rooms en één modelRoom matchen niet eenduidig. Mitigatie: bij collisie alleen de eerste linken, rest wordt orphan ModelRoom met eigen IfcSpace.

### 2.2 Composities en overlays

Het `.ifcenergy` bestand is **één geconsolideerd document**, niet een set overlays. Geen `compose()`-aanroep nodig bij save. Bij open lezen we het als één document.

(IFCX overlays blijven beschikbaar voor de API-route `/calculate/ifcx` — buiten scope van PR B.)

## 3. Nieuwe namespace `isso51::modeller::`

### 3.1 `isso51::modeller::room`

Op `IfcSpace` entries. Bevat 2D-polygon en hoogtes.

```ts
interface Isso51ModellerRoom {
  polygon: Array<{ x: number; y: number }>; // Closed polygon, in mm
  floor: number;                             // Floor index (e.g. 0 = ground)
  height: number;                            // Room height in mm
  elevation?: number;                        // Floor elevation above 0.0 in mm
  temperature?: number;                      // Design temperature in °C (overrides function default)
}
```

### 3.2 `isso51::modeller::window`

Op `IfcWindow` entries (children van IfcSpace).

```ts
interface Isso51ModellerWindow {
  wall_index: number;       // Edge index (0 = first edge of room polygon)
  offset: number;           // Center offset from wall start, in mm
  width: number;            // Window width in mm
  height?: number;          // Window height in mm (from IFC OverallHeight)
  sill_height?: number;     // Sill height above floor in mm
}
```

### 3.3 `isso51::modeller::door`

Op `IfcDoor` entries.

```ts
interface Isso51ModellerDoor {
  wall_index: number;
  offset: number;
  width: number;
  height?: number;
  swing: "left" | "right";
}
```

### 3.4 `isso51::modeller::project_constructions`

Op IfcProject entry. Snapshot van `useModellerStore.projectConstructions` (per-project layer-stack library), zodat heropen de juiste constructie-bibliotheek toont.

```ts
interface Isso51ModellerProjectConstructions {
  entries: ProjectConstruction[]; // Volledige array (zoals nu in JSON envelope)
}
```

(De huidige `.isso51.json` had `project_constructions` als top-level field. We hangen 't nu aan IfcProject — natuurlijke plek.)

### 3.5 Wall-/floor-/roof-construction assignments

`useModellerStore` heeft maps `wallConstructions`, `floorConstructions`, `roofConstructions` (key: `"roomId:wallIndex"` → catalogueEntryId). Plus `wallBoundaryTypes`.

Deze gaan in een `isso51::modeller::assignments` attribute op IfcBuilding:

```ts
interface Isso51ModellerAssignments {
  wall_constructions: Record<string, string>;
  floor_constructions: Record<string, string>;
  roof_constructions: Record<string, string>;
  wall_boundary_types: Record<string, string>;
}
```

### 3.6 Underlay (image background)

Optioneel. Als gebruiker een floor-plan PNG als onderlegger heeft geladen, gaat die mee in `isso51::modeller::underlay` op IfcBuilding:

```ts
interface Isso51ModellerUnderlay {
  data_url: string;         // base64 inline (kan groot zijn — caveat in plan)
  scale: number;            // mm per pixel
  rotation: number;         // degrees
  position: { x: number; y: number }; // mm offset
  locked: boolean;
}
```

## 4. Format-detectie bij open

```ts
function detectFormat(content: string): "ifcenergy" | "isso51-legacy" | "thermal-import" | "unknown" {
  let parsed: unknown;
  try { parsed = JSON.parse(content); } catch { return "unknown"; }
  if (typeof parsed !== "object" || parsed === null) return "unknown";
  const obj = parsed as Record<string, unknown>;

  // 1. Thermal import (Revit/IFC) — check first because legacy "source" field
  if (typeof obj.source === "string" &&
      ["revit-eam", "revit-raycast", "ifc"].includes(obj.source)) {
    return "thermal-import";
  }

  // 2. IFCX shape — has header.ifcxVersion + data array
  const header = obj.header as Record<string, unknown> | undefined;
  if (header && typeof header.ifcxVersion === "string" && Array.isArray(obj.data)) {
    return "ifcenergy";
  }

  // 3. Legacy envelope — has schema field
  if (obj.schema === "isso51-project-v1" && obj.project) {
    return "isso51-legacy";
  }

  // 4. Raw Project JSON (no envelope)
  if (obj.building && obj.climate && obj.ventilation && Array.isArray(obj.rooms)) {
    return "isso51-legacy"; // treat as legacy raw form
  }

  return "unknown";
}
```

Routing:
- `ifcenergy` → new path: parse IFCX, extract Project + Modeller via `isso51-ifcx`
- `isso51-legacy` → existing path: `importProject()` (current code, unchanged)
- `thermal-import` → existing path: thermal wizard
- `unknown` → user-friendly error: "Niet-herkend bestandsformaat"

## 5. Rust crate `isso51-ifcx` — uitbreidingen

### 5.1 Nieuw module: `crates/isso51-ifcx/src/modeller.rs`

```rust
// Type-mirrors van frontend Modeller types (Point2D, ModelRoom, ModelWindow, ModelDoor)
pub struct Point2D { x: f64, y: f64 }
pub struct ModellerRoom { polygon: Vec<Point2D>, floor: i32, height: f64, ... }
pub struct ModellerWindow { wall_index: u32, offset: f64, width: f64, ... }
pub struct ModellerDoor { wall_index: u32, offset: f64, width: f64, swing: String }
pub struct ModellerProjectConstructions { entries: Vec<JsonValue> }  // pass-through
pub struct ModellerAssignments { ... }
pub struct ModellerUnderlay { data_url: String, scale: f64, ... }

// Top-level container voor alle modeller-data
pub struct ModellerData {
  rooms: Vec<(String /* room name */, ModellerRoom)>,
  windows: Vec<(String /* roomId */, ModellerWindow)>,
  doors: Vec<(String /* roomId */, ModellerDoor)>,
  project_constructions: ModellerProjectConstructions,
  assignments: ModellerAssignments,
  underlay: Option<ModellerUnderlay>,
}
```

### 5.2 `isso51-ifcx` namespace constants

Toevoegen aan `namespace.rs`:

```rust
pub mod ns {
  // ... bestaande ...

  /// Modeller geometry on IfcSpace
  pub const MODELLER_ROOM: &str = "isso51::modeller::room";
  pub const MODELLER_WINDOW: &str = "isso51::modeller::window";
  pub const MODELLER_DOOR: &str = "isso51::modeller::door";
  pub const MODELLER_PROJECT_CONSTRUCTIONS: &str = "isso51::modeller::project_constructions";
  pub const MODELLER_ASSIGNMENTS: &str = "isso51::modeller::assignments";
  pub const MODELLER_UNDERLAY: &str = "isso51::modeller::underlay";
}
```

### 5.3 Composer-functies

In nieuwe module `combined.rs`:

```rust
/// Build een IFCX document met zowel calc-data (Project) als modeller-geometrie.
pub fn ifcenergy_to_document(
    project: &Project,
    result: Option<&ProjectResult>,
    modeller: &ModellerData,
) -> IfcxDocument;

/// Reverse: extracteer Project, Result en ModellerData uit een IFCX doc.
pub fn document_to_ifcenergy(
    doc: &IfcxDocument,
) -> Result<(Project, Option<ProjectResult>, ModellerData)>;
```

Implementatie:
- `ifcenergy_to_document` start met `project_to_ifcx(project)` (bestaand) → krijgt IFCX met IfcProject/Site/Building/Spaces/Constructions
- Voegt resultaat-overlay toe via `result_to_ifcx` (bestaand) als `result` Some is
- Loopt door modeller.rooms, vindt matchende calc IfcSpace by name, voegt `isso51::modeller::room` attribute toe (of maakt nieuwe orphan IfcSpace)
- Voor windows/doors: ze worden als children toegevoegd aan de juiste IfcSpace
- Voegt `isso51::modeller::project_constructions` toe op IfcProject
- Voegt `isso51::modeller::assignments` + `::underlay` toe op IfcBuilding

### 5.4 Tests

In `crates/isso51-ifcx/src/lib.rs` test-module:
- `test_ifcenergy_roundtrip_minimal` — Project + lege ModellerData → doc → terug → equal
- `test_ifcenergy_roundtrip_with_geometry` — Project + 2 rooms met polygonen → doc → terug → polygons preserved
- `test_ifcenergy_orphan_modeller_rooms` — ModellerRoom zonder calc match → eigen IfcSpace, importeerbaar
- `test_ifcenergy_with_result_overlay` — Project + result → doc → terug → calc results preserved
- `test_ifcenergy_assignments_roundtrip` — wall/floor/roof construction maps preserved
- `test_ifcenergy_legacy_compose` — `compose()` werkt nog steeds met de nieuwe doc-structuur

Plus property-based: een random `(Project, ModellerData)` pair moet roundtrippen zonder data-verlies (modulo non-deterministische velden zoals UUID-paden).

## 6. Frontend wijzigingen

### 6.1 Nieuw bestand: `frontend/src/lib/ifcenergy.ts`

```ts
export interface IfcEnergyContent {
  project: Project;
  result: ProjectResult | null;
  modeller: ModellerStateSnapshot;
}

interface ModellerStateSnapshot {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
  projectConstructions: ProjectConstruction[];
  assignments: { wallConstructions, floorConstructions, roofConstructions, wallBoundaryTypes };
  underlay: UnderlayImage | null;
}

/** Bouw een IFCX-document (als string) uit project+result+modeller. */
export function buildIfcEnergy(content: IfcEnergyContent): string;

/** Parse een IFCX-document string naar IfcEnergyContent. Throws bij invalid. */
export function parseIfcEnergy(jsonString: string): IfcEnergyContent;
```

Implementatie kan op twee manieren:
- **A.** Rust-via-Tauri-invoke: `invoke("build_ifcenergy", { project, result, modeller })` → string. Voordeel: één bron van waarheid (Rust). Nadeel: vereist Tauri (werkt niet in web-mode).
- **B.** Pure TypeScript: parallelle implementatie aan Rust crate. Voordeel: werkt overal. Nadeel: dubbele code, drift-risico.

**Keuze:** **B (pure TypeScript)** voor de frontend, want web-mode moet ook kunnen exporteren. Rust crate is de **referentie-implementatie** voor server/API; TS bouwt zelf het IFCX document met dezelfde structuur (geverifieerd via gedeelde testfixtures).

### 6.2 Wijzigingen in `frontend/src/lib/importExport.ts`

```ts
// Nieuwe top-level function:
export async function openProjectFile(content: string): Promise<ImportResult | ThermalImportDetected> {
  const fmt = detectFormat(content);
  switch (fmt) {
    case "ifcenergy":   return importIfcEnergy(content);
    case "isso51-legacy": return importProject(content); // bestaande
    case "thermal-import": return { type: "thermal", rawJson: content };
    default: throw new Error("Niet-herkend bestandsformaat");
  }
}

// Nieuwe export — vervangt exportProject voor nieuw werk:
export function exportIfcEnergy(project: Project, result: ProjectResult | null): void {
  const modeller = snapshotModellerState();
  const ifcxJson = buildIfcEnergy({ project, result, modeller });
  // download as <name>.ifcenergy
}
```

`exportProject()` (legacy) blijft bestaan tijdens transitie maar wordt **niet meer aangeroepen** vanuit UI. Verwijderen in volgende PR.

### 6.3 Wijzigingen in dialog-aanroepen

Drie call sites identificeerd: `Modeller.tsx`, `ProjectSetup.tsx`, `Backstage.tsx`. Alle drie aanpassen:
- Open-knop: file-input accepteert `.ifcenergy,.json,.isso51.json`. Auto-detect via `detectFormat()`.
- Save-knop: roept `exportIfcEnergy()` aan (niet meer `exportProject()`).
- Tauri-mode: nieuwe `nativeFileService.ts` met save-dialog filter `.ifcenergy` (default), open-dialog filters voor beide.

### 6.4 Nieuw bestand: `frontend/src/lib/nativeFileService.ts`

Geïnspireerd op Open Calc Studio's gelijknamige service. Bevat:

```ts
const PRIMARY_FILTER = { name: "Open Heatloss Studio", extensions: ["ifcenergy"] };
const LEGACY_FILTER  = { name: "Legacy ISSO 51 JSON", extensions: ["isso51.json", "json"] };
const ALL_FILTER     = { name: "Alle ondersteunde bestanden", extensions: ["ifcenergy", "isso51.json", "json"] };

export async function openProjectNative(): Promise<{ path: string; content: string } | null>;
export async function saveProjectAsNative(content: string, defaultName: string): Promise<string | null>;
```

In web-mode: fallback naar standaard `<input type="file">` en `<a download>`, zoals nu in `importExport.ts` gebeurt.

## 7. Test-strategie

### 7.1 Rust unit tests (in `crates/isso51-ifcx/src/lib.rs`)

Zie 5.4. Roundtrip tests met `assert_eq!` op alle relevante fields.

### 7.2 Frontend unit tests

Nieuwe `frontend/src/lib/ifcenergy.test.ts` (of test-runner van keuze; check bestaand framework via package.json).

Tests:
- `buildIfcEnergy → parseIfcEnergy` roundtrip met fixture project + 3 modeller rooms + 2 windows + 1 door
- `detectFormat()` voor:
  - Een geldig `.ifcenergy` bestand
  - Een legacy `.isso51.json` envelope
  - Een raw Project JSON
  - Een thermal-import file
  - Garbage input
- `parseIfcEnergy()` met corrupted/incomplete data → useful errors

### 7.3 Cross-validation: gedeelde fixture

Een test-fixture `tests/fixtures/ifcenergy/sample.ifcenergy.json`:
- Geschreven door Rust roundtrip test
- Gelezen door zowel Rust als TS, asserts op identieke output

Dit voorkomt drift tussen de twee implementaties.

### 7.4 Manual test (in installer build)

1. Open een legacy `.isso51.json` → check tabellen + modeller correct
2. Save → krijgt `.ifcenergy` extensie
3. Sluit + heropen `.ifcenergy` → tabellen + modeller identiek aan vorige stap
4. Check filename in Tauri save-dialog default
5. Check open-dialog filters (`.ifcenergy` als primary, `.json` als secondary)

## 8. Migratie-pad

### 8.1 Voor de gebruiker

- Bestaande `.isso51.json` → blijft openbaar via "Openen" dialog
- Bij volgende save → krijgt automatisch `.ifcenergy` extensie + IFCX content
- Geen automatische conversie of "save as" gedwongen — natuurlijke transitie via gewoon werken

### 8.2 Voor de developer

- `exportProject()` (legacy JSON) blijft bestaan tot na PR B+C, dan opruimen
- `importProject()` blijft bestaan voor legacy support (forever)

## 9. Bestanden gewijzigd

### Nieuw
- `crates/isso51-ifcx/src/modeller.rs` — Rust modeller-namespace types
- `crates/isso51-ifcx/src/combined.rs` — `ifcenergy_to_document` + `document_to_ifcenergy`
- `frontend/src/lib/ifcenergy.ts` — TS builder + parser
- `frontend/src/lib/nativeFileService.ts` — Tauri dialog wrapper
- `tests/fixtures/ifcenergy/sample.ifcenergy.json` — gedeelde fixture
- Tests inline in bestaande modules

### Gewijzigd
- `crates/isso51-ifcx/src/namespace.rs` — namespace constants toevoegen
- `crates/isso51-ifcx/src/lib.rs` — re-exports + tests
- `frontend/src/lib/importExport.ts` — `detectFormat` + `openProjectFile` toevoegen
- `frontend/src/pages/Modeller.tsx` — gebruikt `openProjectFile` + `exportIfcEnergy`
- `frontend/src/pages/ProjectSetup.tsx` — idem
- `frontend/src/components/backstage/Backstage.tsx` — idem
- `crates/isso51-ifcx/Cargo.toml` — nieuwe deps indien nodig (geen verwacht)
