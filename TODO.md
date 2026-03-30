# TODO

## Huidige focus: IFCX als universeel formaat + web-app IFC integratie

Zie `docs/ifc-herontwerp-verslag.md` sectie 10-11 voor het volledige implementatieplan.

---

## Fase 1: IFC Parser (Python sidecar) — GROTENDEELS KLAAR
- [x] Python project opzetten (`tools/ifc-tool/`) met IfcOpenShell
- [x] Import: IfcSpace → polygonen, verdiepingen
- [x] Storey clustering (nabije bouwlagen samenvoegen)
- [x] Polygon simplificatie pipeline
- [x] Shared edge detectie (binnenwanden herkennen)
- [x] Gap closing (polygonen uitbreiden naar wandhartlijn)
- [x] IfcWindow/IfcDoor extractie (hoogte, borstwering)
- [x] IfcWallType + materiaallagen extractie
- [x] PyInstaller bundeling
- [x] Tauri sidecar integratie
- [ ] Output converteren naar IFCX (i.p.v. bare JSON)
- [ ] Export command: IFCX → IFC4 SPF

## Fase 2: IFCX als universeel formaat — KLAAR
- [x] IFCX parser/writer crate in Rust (`crates/isso51-ifcx/`)
- [x] isso51:: namespace definitie (welke properties)
- [x] Mapper: bestaande Project types ↔ IFCX isso51:: namespace
- [x] isso51-core accepteert IFCX input, produceert IFCX output
- [x] REST API endpoint voor IFCX berekening (`POST /api/v1/calculate/ifcx`)
- [x] IFCX JSON schema in schema-endpoint (`GET /api/v1/schemas/ifcx`)
- [x] Adjacent room resolving (second pass, bidirectioneel)
- [x] Ground parameters mapping (`isso51::construction::ground`)
- [x] ProjectInfo metadata mapping (`isso51::project_info`)
- [ ] IFC parser output converteren naar IFCX (→ verplaatst naar Fase 3)

## Fase 3: Web-app IFC integratie
- [x] IFC parser als server-side service (Docker)
- [x] REST endpoint: `POST /api/v1/ifc/import` (file upload → JSON)
- [x] Frontend: IFC upload → server → modeller store (met web-ifc fallback)
- [ ] Modeller toont geïmporteerde ruimtes in 2D/3D
- [ ] Modeller → IFCX → isso51-core → resultaten

## Fase 4: Space Boundaries & Export
- [ ] 2nd level boundary lezer in IFC parser
- [ ] 1st level → 2nd level splitter
- [ ] Geometrie-based boundary calculator (Vabi-aanpak)
- [ ] Boundary UI in modeller
- [ ] IFC4 SPF export (met thermal psets)
- [ ] IFCX export met isso51::calc:: resultaten

## Fase 5: Herbruikbaarheid & distributie
- [ ] isso51-core als DLL (C ABI via cbindgen)
- [ ] isso51-core als WASM module
- [ ] isso51-core als Python package (PyO3)
- [ ] Modeller als standalone npm package
- [ ] API documentatie + IFCX namespace specificatie

---

## Bugs & correctheid
- [x] **PerFloorArea infiltratie bug** — gefixed (commit 7464e78)
- [x] **BBL ventilatie magic numbers** — gefixed, gebruikt nu `BBL_QV_*` constanten
- [x] **Runtime validatie server-responses** — `validateProjectResult()` toegevoegd, blinde casts vervangen in Projects.tsx, ConflictDialog.tsx, importExport.ts

## Verificatie & testing
- [x] Vabi vrijstaande woning test fixture (9 kamers, 110 constructies, verwachte resultaten)
- [x] DR Engineering woningbouw test fixture
- [x] ISSO 51 portiekwoning test fixture
- [ ] Referentieberekeningen cross-valideren met python-hvac (EN 12831)
- [ ] Kwadratische sommatie unit test: sqrt(101² + 651²) = 659 W

## Code kwaliteit — Rust
- [ ] Constanten definiëren: `RHO_CP_AIR = 1.2`, `GROUND_CORRECTION_FACTOR = 1.45`, `R_SI_*`, `R_SE_*`
- [ ] DRY: `default_one()`/`default_true()` naar gedeeld module
- [ ] DRY: SQL upsert user naar gedeelde functie (handlers/user.rs + handlers/projects.rs)
- [ ] Dead code opruimen: `ventilation_requirement_living()`, `ventilation_requirement_wet_room()`, ongebruikte error varianten
- [ ] Infiltratie tabelnotatie vereenvoudigen (`0.08` ipv `0.08e-3 * 1000.0`)
- [ ] VentilationConfig validatie toevoegen (bijv. heat_recovery_efficiency > 1.0)

## Code kwaliteit — Frontend
- [ ] `MATERIAL_TYPE_LABELS` centraliseren naar `constants.ts` (nu 3x gedupliceerd)
- [ ] `niceMax()` utility centraliseren (nu 4x gedupliceerd in chart/svg bestanden)
- [ ] `FUNCTION_COLORS` centraliseren (nu 3x gedupliceerd in modeller)
- [ ] `Library.tsx` (1052 regels) splitsen in component-bestanden
- [ ] `FloorCanvas.tsx` (1729 regels) splitsen: shapes, room rendering, drawing, utils
- [ ] Dead code verwijderen: `ModellerToolbar.tsx`, `DrawingToolsPanel.tsx` (vervangen door Ribbon)
- [ ] Store snapshot mist constructie-assignments (undo/redo verliest wall/floor/roof toewijzingen)

## Cloud integratie — BACKEND KLAAR
- [x] `openaec-cloud` dependency (gedeelde Nextcloud cloud crate)
- [x] Multi-tenant config (`TENANTS_CONFIG`, `DEFAULT_TENANT` env vars)
- [x] `GET /api/v1/cloud/status` — cloud storage beschikbaarheid
- [x] `GET /api/v1/cloud/projects` — projecten uit Nextcloud
- [x] `GET /api/v1/cloud/projects/{project}/models` — IFC bestanden
- [x] `GET /api/v1/cloud/projects/{project}/calculations` — berekeningen
- [x] `POST /api/v1/cloud/projects/{project}/save` — berekening opslaan + manifest update
- [ ] Server-side deployment: volume mount + env vars in docker-compose
- [ ] Frontend: cloud storage browser in de UI
- [ ] Frontend: "Opslaan naar cloud" knop in Backstage/resultaten

## App features
- [x] OIDC login/logout op productie
- [x] Projecten opslaan/laden
- [x] Vertrekken invoer + bewerken
- [x] Resultaten weergave + grafieken
- [x] JSON import/export
- [x] Rc-calculator met laag-editor
- [x] Glaser-analyse + diagram
- [x] Constructiebibliotheek + materialendatabase
- [x] PDF rapportgeneratie
- [x] Conflict detectie (optimistic locking)
- [x] Auto-save + dark/light theme
- [ ] Materialen: inline bewerken, lambda nat, zoekwoorden

## Modeller features
- [x] 2D/3D modeller met pan/zoom, grid, polygonen, wanden, ramen, deuren
- [x] Ribbon toolbar, teken-tools, snap, meten
- [x] Room splitsen/samenvoegen/verplaatsen
- [x] Constructiebibliotheek koppelen, boundary override
- [x] Onderlegger import, undo/redo, verdiepingen, context menu
- [x] IFC import (IfcSpace → ModelRoom)
- [x] IFC Phase 2: window/door hoogte extractie
- [x] IFC Phase 3: storey clustering, polygon simplificatie, shared edges, gap closing
- [ ] Modeller data ↔ IFCX synchronisatie
- [ ] PDF/DWG onderlegger
- [ ] Schuine daken en dakkapellen

## Roadmap — toekomst
- [ ] BAG-data import (postcode + huisnummer)
- [ ] Quick-calc wizard (5-10 min berekening)
- [ ] ISSO 53 (utiliteitsgebouwen)
- [ ] ISSO 57 (vloerverwarming)
- [ ] Radiatorselectie + hydraulische balancering
- [ ] R3F viewer migratie (ThatOpen → React Three Fiber)
- [ ] Multi-user: projecten delen, rollen
- [ ] Template-projecten: veelvoorkomende woningtypes
