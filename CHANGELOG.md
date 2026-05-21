# Changelog

Belangrijke wijzigingen in Open Heatloss Studio. Volgt [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) en [Semver](https://semver.org/lang/nl/).

## [Unreleased] — 2026-05-21

### 🐛 Bug fixes

- **NTA 8800 drukmodel integratie (C2.3)**: Norm-exacte massabalans (§11.2.1.5/§11.2.1.6) gewired in TO-juli rekenketen met conditionele inzet bij gebouwen binnen C2-scope (H < 15m, bekend bouwjaar) en veilige terugval op heuristiek buiten scope. Bevat forfaitaire `derive_building_height_m()` en `derive_building_leakage_type()` afleiders plus `h_path_lea` detectie-logica.

## [Unreleased] — branch `claude/laughing-kirch-752da4`

Eerste publieke release van Open Heatloss Studio (voorheen "ISSO 51 Warmteverliesberekening"). Deze build bundelt 9 hoofdwijzigingen — installer + format + UI + reports + IFC + MCP.

### ✨ Nieuw

#### Installer + distributie (PR 1)
- **Windows installer** (NSIS, per-user install, Nederlandse wizard, ~6 MB) — `Open Heatloss Studio_<versie>_x64-setup.exe`
- Installeert in `%LOCALAPPDATA%\Open Heatloss Studio\` zonder admin-rechten
- Start-menu shortcut + uninstaller (`uninstall.exe`)
- GitHub Actions CI workflow [`build-installer.yml`](.github/workflows/build-installer.yml) bouwt op `windows-latest`, upload als artifact
- Versie-sync: één bron-of-truth in `Cargo.toml` → `tauri.conf.json` + `frontend/package.json` via `tools/sync-version.ps1`
- Placeholder iconen ("I51" op ISSO-blauw, gegenereerd via `tools/make-placeholder-icon.ps1`)

#### Rebrand
- **"ISSO 51 Warmteverliesberekening" → "Open Heatloss Studio"** — productName, window title, NSIS wizard, app-binary, alle UI strings (NL + EN i18n)

#### Modeller architectuur (PR D)
- **Modeller is nu een read-only viewer** afgeleid van `project.rooms` (Vertrekken-tabel als single source of truth)
- `frontend/src/lib/deriveRoomGeometry.ts` — pure functie die polygonen berekent uit constructie-walls (`perimeter = Σ wall.area / height` + rectangle solve)
- `frontend/src/components/ribbon/RapportTab.tsx` + `IfcTab.tsx` — nieuwe ribbon tabs
- Frozen-banner overlay + ❄️ emoji in tab-label markeren modeller als WIP
- Read-only FloorCanvas (Konva) leest derived rooms; edit-handlers blijven gewireed voor latere editable iteratie

#### File format `.ifcenergy` (PR B Phase 1)
- **`.ifcenergy`** als nieuw default save-formaat — IFCX (IFC5 alpha) document met `isso51::envelope::v1` payload
- Bevat project + result + volledige modeller-snapshot (rooms, windows, doors, project_constructions, wall/floor/roof assignments, underlay)
- `frontend/src/lib/ifcenergy.ts` — builder, parser, format-detectie
- `frontend/src/lib/importExport.ts` → `openProjectFile()` dispatcher (auto-detect `ifcenergy` / `isso51-legacy` / `thermal-import`)
- Legacy `.isso51.json` blijft volledig leesbaar (importProject behouden)
- File input accepts: `.ifcenergy,.json,.isso51.json`
- **Native Windows save-dialog** (Verkenner) via `@tauri-apps/plugin-dialog` + `plugin-fs` in desktop-mode

#### IFC support (PR I)
- **IFC tab** met split-pane viewer (gespiegeld op Open Calc Studio's `IfcPreview`):
  - Links: **IFC4X3 STEP** — line-numbered, syntax-highlighted (#refs blauw, IFCENTITY-types groen, 'strings' bruin, .ENUMS. oranje)
  - Rechts: **IFCX (.ifcenergy) JSON** — collapsible tree, namespace-gekleurde badges
- Beide panels: copy + download knoppen, draggable splitter
- `frontend/src/lib/ifcStepGenerator.ts` — Rust-vrije IFC4X3 STEP generator (IfcProject + Site + Building + Spaces + IfcWalls/IfcSlabs/IfcRoofs met `Pset_isso51` per construction)
- IFCX namespace `isso51::modeller::*` voorbereid in `crates/isso51-ifcx/src/namespace.rs` (toekomst-proof voor PR B Phase 2)
- IFC4X3 + IFCX worden **live geregenereerd** uit project-state — geen save nodig

#### PDF rapport-engine (PR F)
- **Native Rust PDF generator** via `openaec-layout` crate (printpdf/lopdf-based) — submodule `libs/openaec-reports`
- **Liberation Sans** TTF fonts embedded (OFL 1.1 license)
- `src-tauri/src/reports/` — 6 modules (schema, fonts, brand, blocks, special_pages, generator) + 2 Tauri commands (`generate_report_pdf`, `generate_report_pdf_bytes`)
- **OHS brand tokens** + page callbacks; cover · colofon · TOC · backcover · paragraphs · spacers · tables · images · calculation blocks
- Tauri-mode rendert lokaal; web-mode behoudt de remote `/api/v1/report` proxy naar `report.open-aec.com`
- Smoke-test in `src-tauri/tests/reports_smoke.rs` valideert minimal ReportData → valid PDF (>1KB, %PDF magic, lopdf-parsable, ≥1 pagina)

#### CLI binary `gen_pdf`
- **Standalone PDF generator**: `gen_pdf <input.json> <output.pdf>`
- Accepteert raw Project, `.isso51.json` envelope, of `.ifcenergy` IFCX als input
- CI bouwt + smoke-test op `portiekwoning.json` fixture, upload als `gen-pdf-cli` artifact
- MCP server tool `generate_pdf` roept dit binary aan

#### MCP server (`mcp-server/`)
- **TypeScript Node.js Model Context Protocol server** voor Claude Desktop / Claude Code
- Tools: `calculate`, `calculate_file`, `generate_pdf`, `parse_ifcenergy`, `get_schema`, `list_constructions`
- Resources: `project://current`, `result://current`
- Pattern gespiegeld op `open-calc-studio/mcp-server`
- README met config-snippet voor MCP-clients

#### REST API documentatie (`api/README.md`)
- Endpoint-tabel public + authenticated
- Curl voorbeelden, lokaal draaien instructies
- Auth-flow (Authentik forward-auth) uitleg
- Foutmeldings-shape

### 🐛 Bug fixes

- **Calc fix**: `useMemo([])` cached web-backend silently in installed app → "Internal Server Error" bij Berekenen. Per-call Tauri detection in `createBackend()` lost dit op. Plus robuustere `isTauri()` check via `__TAURI_INTERNALS__` + legacy `__TAURI__` + user-agent fragment.
- **Memeleiland mismatch**: `.isso51.json` envelope bevatte geen modeller-geometrie → 2D/3D toonde stale rooms uit een vorig project (modellerStore persist in localStorage). Fix in `importProject()`: bij ontbrekende modeller-data, store wordt geleegd via `importModel([])`.
- **System losses water-boundary** (commit `664999f`, 2026-04-17): ISSO 51 §2.9.1 systeemverliezen voor embedded heating in water-grenzende vloeren werden niet meegenomen — relevant voor woonboot-projecten zoals Memeleiland Kavel 4 (+105 W aansluitvermogen vs vorige PDF's).
- **NSIS config**: `installMode: "perUser"` → `"currentUser"` (Tauri 2 schema), `shortcutName` key verwijderd (niet bestaand in NsisConfig — NSIS pakt automatisch productName).
- **Cargo target path**: `target/` zit in workspace-root, niet in `src-tauri/`. Cache + locate-step gecorrigeerd.
- **Tauri capabilities**: window operations (`show`/`hide`/`minimize`/`close`/`is-maximized`) waren impliciet in `core:default` maar werken niet zonder expliciete grants — uitgebreid in `src-tauri/capabilities/default.json` waardoor het window niet meer in 15×15 pixel modus blijft hangen.
- **Stale .exe in CI cache**: vorige build's `.exe` bleef in cargo-cache, alphabetisch eerste werd geüpload (oude productName). Cleanup-stap toegevoegd + sort-by-date in locate.
- **Tauri version mismatch**: `@tauri-apps/api` 2.11.0 incompatibel met Rust tauri 2.10.2; pinned. Plus `plugin-dialog` (2.6.0) en `plugin-fs` (2.4.5) gepind op exact de Rust crate-versies.

### ♻️ Refactor

- `useModellerStore.rooms/windows/doors` worden niet meer gerenderd in de viewer (modeller is derived) — store blijft bestaan voor `projectConstructions`, `wallConstructions`, `wallBoundaryTypes`, `underlay` (per-project bibliotheek + assignments).
- IfcTab ribbon: Importeer/Export knoppen verwijderd (overlap met IfcPreview-toolbars per panel).
- Modeller page: WIP placeholder → ReadOnlyModellerViewer (SVG) → uiteindelijk FloorCanvas met derived data (per gebruikersfeedback "wel FloorCanvas blijven gebruiken").

### 📚 Documentatie

- Top-level [`README.md`](README.md) met functionaliteit, architectuur, build instructies, roadmap.
- Design specs in [`docs/superpowers/specs/`](docs/superpowers/specs):
  - `2026-05-08-windows-installer-design.md`
  - `2026-05-09-ifcenergy-format-design.md`
  - `2026-05-09-rust-report-integration-design.md`
- Plan documenten in [`docs/superpowers/plans/`](docs/superpowers/plans):
  - `2026-05-08-windows-installer-pr1.md`
  - `2026-05-09-rust-report-integration-plan.md`
- [`docs/building-installer.md`](docs/building-installer.md) — hoe de Windows installer te bouwen / triggeren via CI

### 🔧 Tooling

- `tools/sync-version.ps1` — synchroniseert `Cargo.toml` workspace versie naar `tauri.conf.json` + `frontend/package.json`.
- `tools/make-placeholder-icon.ps1` — genereert 1024×1024 placeholder PNG via .NET System.Drawing.
- `crates/isso51-core/examples/calc_from_file.rs` — debug-CLI: project JSON → result JSON met per-room tabel.

### ⚠️ Bekende issues

- **Modeller is read-only** — geen 2D/3D drawing tools meer. Tekenen + IFC-import-naar-modeller komen terug in een latere PR (`claude/modeller-2d3d-viewer` branch).
- **PDF charts ontbreken** — Diagrammen-sectie staat niet in de huidige `gen_pdf` ReportData builder. Visuele charts (warmteverliezen per vertrek, donut, verlies per constructietype) worden in een latere release toegevoegd.
- **IFC-import via sidecar** schrijft naar `useModellerStore.rooms`, niet naar `project.rooms` — IFC-imported rooms verschijnen daardoor (nog) niet in de read-only viewer. Workaround: gebruik IFC tab JSON-tree om geïmporteerde data te inspecteren.
- **SmartScreen waarschuwing** bij installer-download — niet code-signed. Klik "Meer info" → "Toch uitvoeren". Code-signing volgt zodra Authenticode-certificaat beschikbaar is.
- **Sidecar `ifc-tool-x86_64-pc-windows-msvc.exe`** is 0 bytes in repo (placeholder) — IFC import via Tauri werkt pas na lokaal bouwen via `tools/ifc-tool/` (Python + PyInstaller).

### 📦 Distributie

- Branch: `claude/laughing-kirch-752da4`
- 43 commits sinds `master`
- Installer artifact: `windows-installer` (downloadbaar via Actions UI of `gh run download`)
- gen_pdf CLI artifact: `gen-pdf-cli` (Windows binary + smoke PDF)

### 🔗 Referenties

- [Open Calc Studio](https://github.com/OpenAEC-Foundation/open-calc-studio) — patroon voor IfcPreview, ribbon, MCP server
- [openaec-reports](https://github.com/OpenAEC-Foundation/openaec-reports) — PDF rendering submodule
- [IFC5 alpha spec](https://github.com/buildingSMART/IFC5-development) — IFCX format basis
