# TODO

## 🎯 Sprint v1.0 — BENG/TO-juli/koellast strategie (mei-juni 2026)

### Beschikbaar lokaal (`tests/references/`, gitignored)

- [x] **RVO Rekentool Bijlage AA NTA 8800 2025.04** (`rekentool-bijlage-aa-nta8800-2025.04.xlsm`) — officiële golden master voor BENG-koelbehoefte
- [x] **RVO BENG-voorbeeldconcepten woningbouw 2021** (`rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf`) — DGMR-rapport met 93 doorgerekende cases incl. TO-juli per concept
- [x] **DR Engineering Koellast woningbouw** (`dr-engineering-koellast-woningbouw-2024.pdf`) — Vabi 3.12.0.127, Ag 191.7 m², peak 6420 W
- [x] **Koellastberekeningen.nl Woning B** (`vabi-koellastberekeningen-woning-B-2024.pdf`) — Vabi 3.11.2.23, Ag 182.6 m², peak 8894 W, 17 pp gedetailleerd
- [x] **Vabi statistieken-export Woning C** (`vabi-koellast-statistieken-woning-C.xls`) — 3 ruimtes, 5260 W totaal voelbaar
- [x] **DR Engineering Koellast utiliteitsbouw** (`dr-engineering-koellast-utiliteitsbouw-2024.pdf`)
- [x] **Leever Utiliteit Horeca 2015** (`vabi-koellast-utiliteit-leever-2015.pdf` + `.xls`) — historisch NEN 5067:1985, structurele referentie

### Strategie — Bijlage AA Rekentool als golden master

Met de officiële RVO-rekentool kunnen we **onbeperkt fixtures genereren** zonder externe afhankelijkheden. Workflow:
1. Bijlage AA module implementeren in `crates/nta8800-cooling/src/bijlage_aa.rs` (formules AA.1-AA.13 + Tabel AA.3 lookup)
2. Per fixture-case: invoer in `rekentool-bijlage-aa-nta8800-2025.04.xlsm` → Rekentool output → `expected.json`
3. Onze engine runt met identieke input → vergelijk

DGMR-aanvraag is hiermee **niet meer nodig**.

### Implementatie

- [x] **Bijlage AA module in nta8800-cooling** (Bijlage AA NTA 8800:2025 concept, ~1300 LOC Rust)
  - [x] Formules AA.1 (P_int) t/m AA.13 (capaciteits-toets)
  - [x] Tabel AA.1 (θ_e per uur), AA.2 (f_iso per bouwjaar), AA.3 (I_sol 240 waarden)
  - [x] Per-room max-zoek over 9-18h × 8 oriëntaties × 5 hellingshoeken
  - [x] F_F (kozijnfactor, default 0.9) toegevoegd na cross-val (2026-05-28)
  - [x] **Cross-validatie tegen RVO-rekentool xlsm sample case 1** — groen binnen 0.07% (max 0.26 W op 377 W). Test: `golden_master_xlsm_cross_validatie`. Zie `tests/verification/INSTRUCTIES-bijlage-aa-cross-validatie.md` voor reproductie.
- [ ] **Peak-koellast engine** (separaat, EN 12831/NEN 5060 TO2) voor de Vabi Koellast cases
  - Twee fixture-cases met expected.json klaar: DR Engineering (6420W) + Koellastberekeningen.nl Woning B (8894W)
  - Statistieken-export Woning C als 3e fixture indien gewenst (kleinere case)
- [ ] **3 BENG-fixtures uit RVO voorbeeldconcepten** (Tussenwoning M, Hoekwoning M, Vrijstaande M)
  - Eindwaardes (BENG-1/2/3, TO-juli) staan in PDF
  - Volledige invoer-reconstructie via Rekentool xlsm
- [ ] **Utiliteitsbouw peak-koellast fixture** — folder + expected.json klaar (2026-05-28), wacht op peak-cooling engine

### Optioneel later

- [ ] ISSO 54 testset (BRL 9501 attestering, ~€1500 BouwZo trial) — alleen relevant voor formele software-attestering
- [ ] Uniec voorbeeldproject — Uniec is cloud-only SaaS, geen lokale bestanden mogelijk zonder DGMR-samenwerking

## 🎯 v1.0 Release Criteria

**Vastgelegd 2026-05-26.** v1.0 wordt uitgegeven wanneer onderstaande punten allemaal afgevinkt zijn. v0.2.0 (huidige tag) markeerde ISSO 51 feature-complete; v1.0 markeert het volledige platform (ISSO 51 + 53 + TO-juli) als productie-klaar.

### Blokkades

- [ ] **Alle test-fixtures aanwezig**
  - [ ] TO-juli Vabi-cross-validatie fixtures (scaffold staat in `51dc6ae`, referentie-PDF in `tests/references/dr-engineering-koellast-woningbouw-2024.pdf`)
  - [ ] Spoor 4 fixture-bundeling completeren (calc-algoritme norm-conform, fixtures missen volledige input — zie `docs/PDF_GAPS.md`)
  - [ ] ISSO 53 batch 2d norm-verificatie afronden (infrastructuur klaar, verificatie pending)

- [ ] **Alle tests groen**
  - [ ] `cargo test` workspace — alle crates passend (isso51-core, isso53-core, nta8800-cooling, vabi-importer, ifcx)
  - [ ] `cd frontend && npm run build` slaagt
  - [ ] `cd frontend && npm test` slaagt (indien aanwezig)
  - [ ] CI groen op de release-commit

- [ ] **ISSO 53 productie-klaar**
  - [ ] Vabi end-to-end verificatie op minimaal 2 reëele projecten binnen norm-tolerantie
  - [ ] Alle ISSO 53-specifieke UI-flows getest (norm-switch, utiliteit-velden, rapport)
  - [ ] Geen `TODO:` of `FIXME:` in `crates/isso53-core/` en isso53-gerelateerde frontend code

- [ ] **TO-juli productie-klaar**
  - [ ] Vabi-cross-validatie groen op referentie-project
  - [ ] UI-flow `/tojuli` + `/tojuli-full` getest door user
  - [ ] PDF-rapport TO-juli verifieerbaar tegen Vabi-uitvoer

### Release-actie wanneer alles ✅
1. Versie bump → `1.0.0` in `Cargo.toml` workspace + `frontend/package.json` + `src-tauri/tauri.conf.json`
2. CHANGELOG sectie `[1.0.0]` met milestone-statement
3. Tag `v1.0.0` (annotated)
4. Tauri Windows-installer build via CI (`build-installer.yml`)
5. GitHub Release met installer als artifact + release notes

---

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
- [x] **NTA 8800 drukmodel integratie (C2.3)** — gefixed, norm-exacte massabalans (§11.2.1) gewired in TO-juli rekenketen
- [x] #20 foutmelding server-opslag verbeterd (sessie-verlopen-detectie) — root-cause nog open
- [x] **Jaarverbruik schatting (graaddagen-methode)** — nieuwe Results-veld toont geschat netto jaarverbruik via H_extern × HDD_NL × 24/1000 met expliciete disclaimer (commit 8458a5a)

## Thermal-import — Revit-exporter audit follow-ups (2026-05-22)

> Uit de read-only audit van de PyRevit warmteverlies-exporter. Deze items vereisen éérst een schema-uitbreiding aan deze kant; daarna kan de exporter ze vullen. Exporter-zijdige items staan in de pyRevit-repo `TODO.md`.
- [ ] D3 — optioneel `u_value`/`rc` per construction in `schemas/v1/thermal-import.schema.json` + deserialisatie in `crates/isso51-core/src/import/thermal.rs` → Rc-calculatorstap voor-ingevuld i.p.v. U=0 placeholder
- [ ] D4 — `sfb_code` per construction in schema + `thermal.rs` → betere catalog-groepering; NLRS/SfB-parameter komt uit het Revit-type
- [x] Construction-catalog refactor (`docs/thermal-import-construction-catalog-spec.md`) — geverifieerd volledig geïmplementeerd in `thermal.rs` + frontend; spec-status mag van "Approved" naar "Implemented"

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

## UI / Theming — light theme afmaken
**Status:** Echte light theme staat sinds 2026-05-16 op master (`a88999e`); 3 themes via Settings → Uiterlijk werken via `var(--theme-*)`.
- **2026-05-17 (`12de603`):** `--oaec-*` tokens binnen `[data-theme="light"]` in `themes.css` overschreven (17 vars, gemapt naar `--theme-*`). Lost de `#44444C` cards en `#2E2E36` inputs op voor `/project` (ProjectSetup → AlgemeenTab) en bij Vertrekken (RoomTable). Upstream PR: `OpenAEC-Foundation/openaec-ui#1` (token-split + v0.2.0) — bij merge `package.json` bumpen en het lokale override-blok kan dan verdwijnen.
- Resterend: import-wizard files gebruiken hardcoded Tailwind dark-utility classes (`bg-gray-800/*`, `border-gray-*`) en negeren daardoor zowel `--theme-*` als `--oaec-*`. Zichtbaar in `/import/thermal` flow.
- [ ] `components/import/ConstructionImportStep.tsx` — vervang `bg-gray-800/50`, `border-gray-700`, `bg-gray-700/60` door theme-aware (`var(--theme-surface)`, `var(--theme-border)`, `var(--theme-bg-lighter)`)
- [ ] `components/import/FileUploadStep.tsx` — idem (`bg-gray-800/50`, `border-gray-600`, `bg-gray-700`, `border-gray-700`)
- [ ] `components/import/ImportSummary.tsx` — idem (`bg-gray-800/50`, `border-gray-700`)
- [ ] `components/import/OpeningImportStep.tsx` — idem (`bg-gray-800/{30,40,80}`, `border-gray-{600,700}`, `text-gray-{400,500,600}`, `placeholder-gray-600`)
- [ ] `components/import/RoomImportStep.tsx` — idem (`bg-gray-800/{40,80}`, `border-gray-{600,700}`, `text-gray-{400,500}`)
- [ ] `components/import/ThermalImportWizard.tsx` — idem (`bg-gray-{700,800}`, `border-gray-{500,600,700}`, `text-gray-{300,400}`)
- [ ] `components/layout/Topbar.tsx` — `bg-[#27272A]` hover-states (regels 70/103/112/119) → `var(--theme-hover-strong)`. **Eerst checken of Topbar nog actief is** — volgens CLAUDE.md UI-migratie is hij vervangen door TitleBar+Ribbon; mogelijk dead code (verwijderen i.p.v. fixen).
- [ ] Sweep-strategie: per file beoordelen of theme-aware classes (via `:where([data-theme="light"]) .X { ... }` in component.css) of inline CSS-vars (`style={{ background: "var(--theme-surface)" }}`) de schoonste route is. Inline vars zijn pragmatischer voor de import-wizard (Tailwind utility-overflow).
- [ ] Acceptance: in light mode geen `bg-gray-*` zichtbaar; switch tussen 3 themes verandert alle wizard-screens.

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
- [x] Rc-calculator: inhomogene lagen (ISO 6946 combined method) + bevestigingsmiddelencorrectie (Annex F)
- [x] Glaser-analyse + diagram
- [x] Constructiebibliotheek + materialendatabase
- [x] PDF rapportgeneratie
- [x] Conflict detectie (optimistic locking)
- [x] Auto-save + dark/light theme
- [ ] Materialen: inline bewerken, lambda nat, zoekwoorden
- [x] U_w kozijn-calculator Fase 1: `uw_breakdown`-datamodel + `Spacer`-enum (`7727e79`)
- [x] U_w kozijn-calculator Fase 2: `uwCalculation.ts` + spacer-tabel + `/uw`-calculatorpagina
- [x] U_w kozijn-calculator Fase 3: opslaan op kozijn-element + opbouw in project-rapport + zelfstandig U_w-rapport
- [x] U_w kozijn-calculator: fabrikant-catalogus (profiel/glas) + Ψ_g-correctie naar EN-ISO 10077-1 Annex E-richtwaarde
- [x] U_w kozijn-calculator: afronding — setTimeout-cleanup, edit-param-feedback, catalogus-herkomst persistent in rapport
- [x] #21 rekenexpressies (=1,5*2,6) in numerieke tabelcellen

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

## Architectuur / open ontwerpen
- [ ] **Zone-model ADR** — `docs/2026-05-23-zone-model-adr.md` — ontwerp voor mixed-use support via norm-keuze per rekenzone (spike/draft)

## Roadmap — toekomst
- [ ] BAG-data import (postcode + huisnummer)
- [ ] Quick-calc wizard (5-10 min berekening)
- [ ] ISSO 53 (utiliteitsgebouwen)
  - [x] Batch 1: skelet + model-setup (`crates/isso53-core/`)
  - [x] Batch 2a: opzoektabellen (11 tabel-modules in `tables/`)
  - [x] Batch 2b: calc-kern (theta_i, q_h,nd)
  - [x] Batch 2c: orkestratie + CLI werkend
  - [x] Batch 2d: test fixtures + verificatie — infrastructuur klaar, norm-verificatie pending
  - [x] **ISSO 53 UI-spoor** — dual-calc support in bestaande web-app (COMPLEET)
    - [x] Fase 1: backend dual-pipeline (KLAAR — commit 86e8ab6)
    - [x] Fase 2: norm-keuze UI + topbar-badge (KLAAR — commit 8ffa728)
    - [x] Fase 3: conditional rendering bestaande screens (KLAAR — commit 28c429f)
    - [x] Fase 4: wissel-flow met waarschuwing (KLAAR — commit e697c97)
    - [x] Fase 5: isso53-report-builder (KLAAR — commit 7d8a307)
  - [x] **ISSO 53 - calc-core warmteverlies sporen** — AFGESLOTEN sessie 8 (2026-05-25)
    - [x] **§4.6 embedded heating clause geïmplementeerd** (commit 0f4293a)
      - phiT: 4385→2918 W vs Vabi 2919 W (<0.1% afwijking) ✅
      - f_ig = 0.0 voor elementen met has_embedded_heating = true
    - [x] **Adjacent-room transmissie sporen 1/2/3** — OPGELOST via Optie C wrapper-schrap (sessie 8)
      - Dubbeltelling adjacent-room-bijdrage weg (5-7% overschatting gefixed)
      - Tests: 92 passed / 0 failed / 4 ignored
    - [x] **Spoor 4 fixture-artefact** — GEDIAGNOSEERD en GEDOCUMENTEERD (PDF_GAPS.md)
      - Plan-agent bewijs: gap zit in fixture-bundeling, niet calc-core algoritme
      - Norm-conforme implementatie formule 4.18 bevestigd
  - [x] **ISSO 53 - "toekomstige sporen" geverifieerd norm-conform** (2026-05-26)
    - [x] **WTW ventilatie** — implementatie was al norm-conform (ISSO 53 §4.7.2 formule 4.38)
      - Verificatie: f_v ≈ 0.15 bij η_wtw=85% → ~85% reductie van Φ_V (test `test_wtw_ventilation_efficiency_applied` in `calc/ventilation.rs`)
      - "phiV = 3076 W" was absolute waarde bij groot debiet, niet bewijs van bug
    - [x] **Infiltratie systeem-D** — ISSO 53 tabel 4.7 schrijft f_inf=1.15 voor SystemD vs 0.80 voor SystemA
      - Hogere infiltratie bij balanced ventilation is fysisch correct (ventiel-drukverschillen)
      - Regressie-test: `test_systemd_infiltration_norm_compliant` in `calc/infiltration.rs`
- [ ] ISSO 57 (vloerverwarming)
- [ ] Radiatorselectie + hydraulische balancering
- [ ] R3F viewer migratie (ThatOpen → React Three Fiber)
- [ ] Multi-user: projecten delen, rollen
- [ ] Template-projecten: veelvoorkomende woningtypes
