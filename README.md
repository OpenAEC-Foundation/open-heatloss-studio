# Open Heatloss Studio

> Open-source warmteverliesberekening voor woningen — een moderne, vrije implementatie van de Nederlandse norm **ISSO 51:2023**.

[![Status](https://img.shields.io/badge/status-public%20testing-orange)](https://github.com/OpenAEC-Foundation/open-heatloss-studio)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB)](https://v2.tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org/)

Open Heatloss Studio brengt professionele warmteverliesberekeningen naar een open, modern platform. Geen black box, geen vendor lock-in — een desktop app (Tauri + React + Rust) plus een herbruikbare rekenkern.

Onderdeel van het [OpenAEC Foundation](https://github.com/OpenAEC-Foundation) ecosysteem, samen met [Open Calc Studio](https://github.com/OpenAEC-Foundation/open-calc-studio), [openaec-reports](https://github.com/OpenAEC-Foundation/openaec-reports), en andere AEC-tools.

## Status

**Public testing** — actief in ontwikkeling. Stabiel genoeg voor echt gebruik, met de kanttekening dat de **2D/3D Modeller momenteel als read-only viewer (frozen)** gemarkeerd is terwijl we de architectuur herontwerpen (zie roadmap onderin).

## Functionaliteit

### Berekening (ISSO 51:2023)

- **Volledige norm-implementatie** in pure Rust (`crates/isso51-core`):
  - Transmissieverliezen (formules 4.2, 4.3a, 4.6, 4.14, 4.18)
  - Ventilatieverliezen (erratum 2023, formule 4.3, 4.6a, 3.3)
  - Infiltratie (qi_spec methode + qv10 over schil)
  - Opwarmtoeslag (tabel 4.6, paragraaf 4.3)
  - Systeemverliezen voor embedded heating (paragraaf 2.9.1, **inclusief water-grenzen** voor woonboten — fix 2026-04-17)
  - Ground/water boundary correcties (Annex E erratum)
  - Boundary-types: exterior · adjacent_room · adjacent_building · unheated_space · ground · water
  - Heat-up factor f_RH per gebouwtype
  - Per-room en building-level totalen (kwadratische sommatie)
- **Validatie**: drie referentie-fixtures slagen end-to-end
  - ISSO 51 portiekwoning voorbeeld
  - DR Engineering woningbouw
  - Vabi vrijstaande woning (9 kamers, 110 constructies)

### File-formaten

- **`.ifcenergy`** (default save) — IFCX (IFC5 alpha) document met `isso51::envelope::v1` payload. Toekomst-proof, open formaat, herbruikbaar in andere AEC-tools.
- **`.isso51.json`** (legacy read-only) — eerdere proprietary envelope; volledig backwards-compatibel bij openen.
- **Raw Project JSON** — direct serializable Project type voor scripting/CLI.
- **IFC import** via Python `ifc-tool` sidecar (PyInstaller bundle, IfcOpenShell-based) — `.ifc` STEP files → ruimtes + ramen + deuren + wandtypes.
- **IFCX export** met `isso51::` namespace + `isso51::modeller::` voor 2D/3D-geometrie.

### UI (desktop + web)

- **Office-stijl Ribbon** met 7 tabs: Bestand · Vertrekken · Constructies · Modeller ❄️ · IFC · Resultaten · Rapport
- **Vertrekken-tabel** als bron-van-waarheid voor calc-data (rooms + constructions)
- **Modeller** als read-only 2D/3D viewer derived van project.rooms (zie roadmap)
- **IFC tab** met IFC4X3 STEP + IFCX JSON side-by-side (gespiegeld op Open Calc Studio's `IfcPreview`)
- **Rapport tab** met PDF preview iframe + page-format/orientation opties
- **Resultaten tab** met diagrammen + per-room en building totalen
- **Constructie-bibliotheek** + Rc-calculator (ISO 6946 combined method, Annex F bevestigingsmiddelen)
- **Glaser-analyse** + diagram voor vochtcondensatie
- **Native save dialog** (Windows Verkenner) voor `.ifcenergy` export

### Rapportage

- **PDF rapport** native via Rust [`openaec-layout`](https://github.com/OpenAEC-Foundation/openaec-reports) crate (printpdf/lopdf-based)
- **Liberation Sans** fonts embedded (OFL 1.1)
- **OHS brand tokens** voor styling (uitbreidbaar naar tenant-specifieke huisstijlen)
- **Standaard rapport sections**: Cover · Colofon · Uitgangspunten · Vertrekken overzicht · Per-room details · Diagrammen · Gebouwresultaten
- **Standalone CLI**: `gen_pdf <input.json> <output.pdf>` — geen UI nodig voor PDF-generatie

### Distributie & integratie

- **Windows installer** (NSIS, per-user install, NL wizard) via GitHub Actions CI
- **REST API** ([`crates/isso51-api`](crates/isso51-api)): public `/calculate`, `/calculate/ifcx`, `/schemas/*`; authenticated `/projects/*`, `/cloud/*`, `/report` (Authentik forward-auth)
- **MCP server** ([`mcp-server/`](mcp-server)) — Model Context Protocol server voor Claude Desktop / Claude Code integratie
- **Cloud opslag** via Nextcloud (multi-tenant, group-folder mounts) — voor server-deployments
- **Geplande herbruikbaarheid**: WASM module · Python package (PyO3) · DLL via cbindgen

## Quick start

### Download

Download de Windows installer vanaf de [GitHub Actions runs](https://github.com/OpenAEC-Foundation/open-heatloss-studio/actions/workflows/build-installer.yml) — kies een succesvolle run en download het `windows-installer` artifact. Pak uit en draai het `.exe` (per-user installatie, geen admin-rechten nodig).

### Build from source

Vereisten:
- [Node.js 22+](https://nodejs.org/) (LTS aanbevolen)
- [Rust stable](https://www.rust-lang.org/tools/install) — minimaal 1.78
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) voor jouw platform (op Windows: Visual Studio Build Tools 2022 met C++ workload)
- Git met submodules ondersteuning

```bash
# Clone met submodules (openaec-reports nodig voor PDF generator)
git clone --recurse-submodules https://github.com/OpenAEC-Foundation/open-heatloss-studio
cd open-heatloss-studio

# Frontend dependencies
cd frontend && npm install && cd ..

# Dev mode (browser only, snel)
cd frontend && npm run dev

# Dev mode (Tauri desktop)
cd src-tauri && cargo tauri dev

# Productie build (NSIS installer)
cd src-tauri && cargo tauri build --bundles nsis
```

De installer staat in `target/release/bundle/nsis/`.

### Tests

```bash
# Rust core (rekenkern + IFCX)
cargo test -p isso51-core
cargo test -p isso51-ifcx

# Frontend type-check
cd frontend && npx tsc --noEmit
```

## Architectuur

```
open-heatloss-studio/
├── crates/                       # Rust workspace
│   ├── isso51-core/              # Pure rekenkern (geen I/O, geen async)
│   ├── isso51-ifcx/              # IFCX adapter, isso51:: namespace
│   ├── isso51-api/               # REST API (axum)
│   └── nta8800-*/                # Energieprestatie modules (NTA 8800)
├── frontend/                     # React 19 + TypeScript + Vite
│   ├── src/components/           # UI: ribbon, modeller, ifc, backstage, ...
│   ├── src/pages/                # Routes: project, rooms, modeller, ifc, rapport
│   ├── src/lib/                  # Calc client, importExport, ifcenergy, ...
│   └── src/store/                # Zustand stores (project + modeller + report)
├── src-tauri/                    # Tauri 2 desktop app
│   ├── src/                      # Rust: commands, reports module, gen_pdf bin
│   ├── icons/                    # App icons (placeholder branding I51)
│   └── capabilities/             # Tauri 2 permissions (per-window grants)
├── libs/openaec-reports/         # Submodule — PDF rendering crates
├── tools/ifc-tool/               # Python sidecar (IfcOpenShell)
├── api/                          # REST API documentatie
├── mcp-server/                   # MCP server (TypeScript Node.js)
├── schemas/v1/                   # JSON schemas (Project, Result, IFCX)
├── tests/fixtures/               # Test JSON projecten
└── docs/                         # Specs, plans, design docs
```

## Documentatie

- **Spec + plan documenten**: [`docs/superpowers/specs/`](docs/superpowers/specs) en [`docs/superpowers/plans/`](docs/superpowers/plans)
- **REST API**: [`api/README.md`](api/README.md)
- **MCP server**: [`mcp-server/README.md`](mcp-server/README.md)
- **Installer building**: [`docs/building-installer.md`](docs/building-installer.md)
- **IFC herontwerp**: [`docs/ifc-herontwerp-verslag.md`](docs/ifc-herontwerp-verslag.md)

## Roadmap

### Op korte termijn (open punten)

- **Modeller editable mode** — momenteel read-only viewer; eigen branch [`claude/modeller-2d3d-viewer`](https://github.com/OpenAEC-Foundation/open-heatloss-studio/tree/claude/modeller-2d3d-viewer) voor editor-architectuur
- **`.ifcenergy` Phase 2** — envelope splitsen naar proper per-entry IFCX namespace attributen (zie `crates/isso51-ifcx/src/namespace.rs` voor de gedefinieerde constants)
- **Charts in PDF** — `Diagrammen` sectie met SVG → PNG raster (warmteverliezen per vertrek, donut + verlies per constructietype)
- **PageSize/Orientation parameters** doorgeven aan `buildReportData()` voor regeneratie zonder app-reload
- **Echte 3D IFC viewer** in `/ifc` page (vereist `@thatopen/components` setup)

### Op langere termijn

- WASM module (browser-side rekenen zonder server)
- Python package (PyO3 wrapper voor scripting)
- DLL via cbindgen (Excel/Revit plugins)
- Code signing (verwijdert SmartScreen-warning bij installer)
- macOS / Linux installers (vereist sidecar build voor die platforms)
- WiX MSI naast NSIS (enterprise rollout via Group Policy / Intune)
- Auto-update via Tauri updater plugin
- Quick-calc wizard (5-10 min berekening voor klein woonhuis)
- BAG-data import (postcode + huisnummer → automatische geometrie)
- ISSO 53 (utiliteitsgebouwen) en ISSO 57 (vloerverwarming)

## Contributing

PR's en issues welkom. Houd rekening met:

- **Rust:** `cargo test` moet altijd slagen
- **Frontend:** `npx tsc --noEmit` clean, geen `as any` casts (gebruik proper types)
- **Eenheden:** mm voor afmetingen, dm³/s voor luchtvolumestroom, W voor vermogen, W/K voor H-waarden
- **Norm-referenties:** Doc-comments verwijzen expliciet naar ISSO 51 formulenummers

Zie [`CLAUDE.md`](CLAUDE.md) voor projectspecifieke conventies.

## Licentie

[MIT](LICENSE) — zie ook de licentie van de bundled fonts (Liberation Sans, OFL 1.1) en submodules.

## Contact

- **Repo:** https://github.com/OpenAEC-Foundation/open-heatloss-studio
- **OpenAEC Foundation:** https://github.com/OpenAEC-Foundation
- **Productie deployment (web mode):** https://open-heatloss-studio.open-aec.com
