# Open Heatloss Studio

> Open-source warmteverliesberekening volgens **ISSO 51:2023** — desktop-app met pure-Rust rekenkern, IFCX-bestandsformaat, native PDF-rapportgenerator en MCP-integratie voor Claude.

[![Status](https://img.shields.io/badge/status-public%20testing%20(alpha)-orange)](https://github.com/OpenAEC-Foundation/open-heatloss-studio/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB)](https://v2.tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org/)

Onderdeel van het [OpenAEC Foundation](https://github.com/OpenAEC-Foundation) ecosysteem, samen met [Open Calc Studio](https://github.com/OpenAEC-Foundation/open-calc-studio), [openaec-reports](https://github.com/OpenAEC-Foundation/openaec-reports) en de [OpenAEC style-book](https://github.com/OpenAEC-Foundation/OpenAEC-style-book) Tauri+React templates.

---

## Status

**Public testing (alpha).** Stabiel genoeg voor échte warmteverliesberekeningen op woningprojecten — drie referentie-fixtures uit ISSO 51, DR Engineering en Vabi lopen end-to-end groen. De Modeller staat momenteel als **read-only viewer** (frozen) gemarkeerd terwijl de editor-architectuur op een aparte branch wordt herontworpen. Niet code-signed dus de Windows installer triggert SmartScreen — klik "Meer info" → "Toch uitvoeren".

Laatste release: [v0.1.0-alpha.1](https://github.com/OpenAEC-Foundation/open-heatloss-studio/releases/tag/v0.1.0-alpha.1)

---

## Snel beginnen

### Installeren

**Windows.** Download `Open Heatloss Studio_<versie>_x64-setup.exe` van de [Releases pagina](https://github.com/OpenAEC-Foundation/open-heatloss-studio/releases). Per-user installatie in `%LOCALAPPDATA%\Open Heatloss Studio\` — geen admin nodig. NSIS-wizard in het Nederlands. Bij elke push naar een feature-branch bouwt CI automatisch een nieuwe installer; de laatste artifacts liggen op de [Actions pagina](https://github.com/OpenAEC-Foundation/open-heatloss-studio/actions/workflows/build-installer.yml).

**Linux (AppImage).** Download `<versie>_amd64.AppImage` van [Actions → Build Linux AppImage](https://github.com/OpenAEC-Foundation/open-heatloss-studio/actions/workflows/build-appimage.yml). `chmod +x` en draaien.

**macOS / Web.** Build from source (zie hieronder); een `.dmg` distributie komt in een latere release.

### Eerste gebruik

1. Start Open Heatloss Studio
2. Klik **Bestand → Nieuw** voor een leeg project of **Bestand → Openen** voor een bestaand `.ifcenergy`
3. Vul **Project** in: naam, opdrachtgever, adviseur, optionele voorbladafbeelding voor het PDF-rapport
4. Vul **Vertrekken** in: per ruimte de functie, oppervlak, hoogte, ventilatie en alle constructie-elementen (wanden, vloer, dak, ramen, deuren) met hun U-waarde en grenstype
5. Bouw eventueel je eigen **Constructies** met de Rc-calculator (ISO 6946 combined method)
6. **Berekenen** — Rust core rekent transmissie/ventilatie/infiltratie/opwarm/systeem
7. **Rapport → Genereren** voor de PDF; opties (sectie aan/uit, papierformaat) in het linker zijbalk
8. **Bestand → Opslaan** schrijft naar `.ifcenergy` (IFCX) — silent overschrijven van je open bestand, of save-as dialog voor nieuwe projecten

Bij dubbelklik op een `.ifcenergy` in Verkenner opent de app automatisch (Windows file-association).

---

## Functionaliteit

### Rekenkern (ISSO 51:2023)

Volledig in pure Rust (`crates/isso51-core`), getest tegen drie referentie-fixtures (ISSO 51 portiekwoning, DR Engineering woningbouw, Vabi vrijstaande woning met 9 kamers / 110 constructies):

- **Transmissieverliezen** — ISSO 51 formules 4.2, 4.3a, 4.6, 4.14, 4.18 (schil · adjacent room · adjacent building · unheated space · ground · water)
- **Ventilatieverliezen** — erratum 2023 formules 4.3, 4.6a, 3.3 met f_v-factor
- **Infiltratieverliezen** — qi_spec methode + qv10 over schil-oppervlak
- **Opwarmtoeslag** — ISSO 51 tabel 4.6 + paragraaf 4.3, f_RH per gebouwtype
- **Systeemverliezen** — embedded heating (paragraaf 2.9.1), **inclusief water-grenzen** voor woonboten (fix 2026-04-17)
- **Ground/water boundary correcties** — Annex E erratum
- **Per-room totalen** + building-level kwadratische sommatie

### Bestandsformaat (`.ifcenergy`)

Het projectformaat is een **geldige IFCX (IFC5 alpha) JSON** — zelfde structuur als buildingSMART's [hello-wall voorbeeld](https://github.com/buildingSMART/IFC5-development/blob/main/examples/Hello%20Wall/hello-wall.ifcx) (header met `ifcxVersion: "ifcx_alpha"`, `imports` naar `ifc@v5a`, `data`-array met IFC class classificatie op IfcProject/IfcSite/IfcBuilding).

In v0.1 zit alle warmteverlies-data nog in één vendor-namespace attribute (`isso51::envelope::v1`) op IfcProject — **Phase 1: JSON-in-IFCX envelope**. Decompositie naar per-entity IFCX (IfcSpace per vertrek, IfcWall/IfcSlab/IfcRoof per constructie, IfcWindow/IfcDoor per opening) staat op de roadmap als [issue #12](https://github.com/OpenAEC-Foundation/open-heatloss-studio/issues/12).

Daarnaast ondersteund:
- **`.isso51.json`** (legacy read-only) — eerdere proprietary envelope; volledig backwards-compatibel bij openen
- **IFC import** via Python `ifc-tool` sidecar (PyInstaller bundle, IfcOpenShell-based) — `.ifc` STEP files → ruimtes, ramen, deuren en wandtypes
- **IFC4X3 STEP export** — pure-TS generator in IFC-tab (geen Python afhankelijkheid)
- **IFCX export** met `isso51::` namespace + `isso51::modeller::` constants voor 2D/3D-geometrie

### UI

Office-stijl Ribbon met zeven tabs:

| Tab | Inhoud |
|---|---|
| **Bestand** (Backstage) | Nieuw · Openen (Tauri-native dialog met `.ifcenergy` filter) · Recent (eigen panel rechts) · Opslaan / Opslaan als · Voorkeuren · Extensies · Over · Afsluiten |
| **Vertrekken** | Tabel met ruimtes + constructie-elementen; per element area · U-waarde · grenstype · adjacent. Single source of truth voor de berekening. |
| **Constructies** | Bibliotheek met opbouwen + Rc-calculator (ISO 6946 combined method · Annex F bevestigingsmiddelen) + Glaser dampdiffusie-analyse |
| **Modeller ❄️** | Frozen read-only viewer afgeleid van `project.rooms`. Editable-modus komt terug in een latere release. |
| **IFC** | Split-pane: IFC4X3 STEP (links, line-numbered, syntax-highlighted) + IFCX `.ifcenergy` JSON (rechts, collapsible tree). Beide met copy / download per panel. |
| **Resultaten** | Per-vertrek + gebouwtotalen. Donut + stacked-bar charts. |
| **Rapport** | PDF preview iframe + collapsible opties-zijbalk (voorbladafbeelding upload + 9 sectie-toggles + A4/A3 + portret/landschap). |

Verder:

- **Backstage met Recent files paneel** — 10 laatst-geopende projecten met relative timestamp, klik om te openen
- **Extensies paneel** — overzicht core onderdelen (ISSO 51 rekenkern · IFC importer · PDF rapport-engine · MCP server · Glaser) + roadmap items
- **TitleBar met Save quick-access knop** — schrijft naar het huidige `.ifcenergy` pad zonder dialog (silent save); save-as via Ctrl+Shift+S of Bestand → Opslaan als
- **Dark theme** met OHS-teal + amber accenten; native form controls (dropdowns/scrollbars) zijn dark-mode via `color-scheme: dark`
- **DevTools** met F12 ook in productie-builds
- **i18n** Nederlands + Engels via i18next; auto-detect of expliciete keuze in Voorkeuren

### Rapport (PDF)

Native Rust PDF generator via [`openaec-layout`](https://github.com/OpenAEC-Foundation/openaec-reports) crate (printpdf/lopdf-based), Liberation Sans fonts embedded onder OFL 1.1.

Standaard secties (allemaal afzonderlijk uit te schakelen):

1. **Cover** — projectnaam + ondertitel + datum + voorblad-afbeelding (PNG/JPEG, optioneel)
2. **Colofon** — opdrachtgever · adviseur · normen · revisiehistorie
3. **Inhoudsopgave** (TOC)
4. **Uitgangspunten** — gebouwgegevens · klimaat (θ_e, θ_b, θ_water voor water-grenzen) · ventilatiesysteem
5. **Constructie-opbouw & Rc-waarden** — per opbouw laagopbouw-tabel (Materiaal · Dikte · λ · R) + resultaten-tabel (R_si · R_se · R_totaal · Rc · U + ISO 6946 §6.7.2 ratio-check + ΔU_f) + **temperatuurverloop-grafiek** met grensvlak-temperaturen
6. **Vertrekken overzicht** — samenvattingstabel met θ_i, Φ_T, Φ_v, Φ_hu, Φ_sys, Φ_totaal per ruimte
7. **Per vertrek** — invoer (Algemeen · Constructie-elementen) + reken-resultaten (Transmissie · Ventilatie & infiltratie · Opwarmtoeslag & systeem · Totaal)
8. **Diagrammen** — warmteverliezen per vertrek (gestapelde bar) · gebouwtotaal donut · verlies per constructietype (horizontale bar)
9. **Gebouwresultaten** — totalen + Aansluitvermogen met ISO 51:2023 referentie
10. **Backcover** — projectmeta + GitHub URL + MIT license

`gen_pdf` **standalone CLI** in `src-tauri/src/bin/gen_pdf.rs` voor PDF-generatie zonder UI. Wordt door de MCP server gebruikt en draait in CI op portiekwoning-fixture voor smoke-tests.

### Distributie en integratie

- **Windows installer** (NSIS, per-user, NL wizard) via GitHub Actions
- **Linux AppImage** via GitHub Actions
- **`.ifcenergy` file-association** op Windows — dubbelklik in Verkenner opent app met het bestand geladen
- **REST API** (`crates/isso51-api`) — public `/calculate`, `/calculate/ifcx`, `/schemas/*`; authenticated `/projects/*`, `/cloud/*`, `/report` (Authentik forward-auth)
- **MCP server** (`mcp-server/`) — Model Context Protocol server voor Claude Desktop / Claude Code. Tools: `calculate` · `calculate_file` · `generate_pdf` · `parse_ifcenergy` · `get_schema` · `list_constructions`. Resources: `project://current` · `result://current`
- **Cloud opslag** via Nextcloud (multi-tenant, group-folder mounts) — voor server-deployments

---

## Build from source

### Vereisten

- [Node.js 22+](https://nodejs.org/) (LTS aanbevolen)
- [Rust stable](https://www.rust-lang.org/tools/install) — minimaal 1.78
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) voor jouw platform:
  - **Windows:** Visual Studio Build Tools 2022 met C++ workload
  - **Linux:** `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf` (zie `.github/workflows/build-appimage.yml`)
- Git met submodules-ondersteuning

### Setup

```bash
# Clone met submodules (openaec-reports nodig voor PDF generator)
git clone --recurse-submodules https://github.com/OpenAEC-Foundation/open-heatloss-studio
cd open-heatloss-studio

# Frontend dependencies
cd frontend && npm install && cd ..

# Dev mode (browser only — snel, geen Tauri runtime)
cd frontend && npm run dev

# Dev mode (Tauri desktop met hot-reload)
cd src-tauri && cargo tauri dev

# Productie build (NSIS installer op Windows)
cd src-tauri && cargo tauri build --bundles nsis

# Productie build (AppImage op Linux)
cd src-tauri && cargo tauri build --bundles appimage
```

De installer staat in `target/release/bundle/nsis/` of `target/release/bundle/appimage/`.

### Tests

```bash
# Rust core (rekenkern + IFCX adapter)
cargo test -p isso51-core
cargo test -p isso51-ifcx

# Rust PDF (smoke-test)
cargo test -p isso51-desktop --test reports_smoke

# Frontend type-check
cd frontend && npx tsc --noEmit

# Frontend overflow-detector (handig na layout wijzigingen)
python tools/check_pdf_overflow.py path/to/output.pdf
```

---

## Architectuur

```
open-heatloss-studio/
├── crates/                       # Rust workspace
│   ├── isso51-core/              # Pure rekenkern (geen I/O, geen async, geen unsafe)
│   ├── isso51-ifcx/              # IFCX adapter, isso51:: namespace constants
│   ├── isso51-api/               # REST API (axum)
│   └── nta8800-*/                # Energieprestatie modules (NTA 8800, parallel werk)
├── frontend/                     # React 19 + TypeScript + Vite
│   ├── src/components/           # UI: ribbon, modeller, ifc, backstage, …
│   ├── src/pages/                # Routes: project, rooms, modeller, ifc, rapport
│   ├── src/lib/                  # Calc client, importExport, ifcenergy, rcCalculation, reportBuilder
│   └── src/store/                # Zustand stores (project · modeller · report · recentFiles)
├── src-tauri/                    # Tauri 2 desktop app
│   ├── src/                      # Rust: commands, reports module, gen_pdf bin
│   ├── icons/                    # App icons (huis + warmtegolven, OHS-teal)
│   └── capabilities/             # Tauri 2 permissions (window · dialog · fs · shell)
├── libs/openaec-reports/         # Submodule — PDF rendering crates (openaec-layout)
├── tools/                        # Build + test scripts
│   ├── ifc-tool/                 # Python sidecar (IfcOpenShell + PyInstaller)
│   ├── sync-version.ps1          # Workspace versie → tauri.conf.json + package.json
│   ├── make-logo.ps1             # Genereert app-icon source.png
│   └── check_pdf_overflow.py     # Regressie-detector voor PDF layout
├── api/                          # REST API documentatie
├── mcp-server/                   # MCP server (TypeScript Node.js)
├── schemas/v1/                   # JSON schemas (Project, Result, IFCX)
├── tests/fixtures/               # Test JSON projecten
└── docs/                         # Specs, plans, design docs
```

---

## Documentatie

- **Spec + plan documenten:** [`docs/superpowers/specs/`](docs/superpowers/specs) en [`docs/superpowers/plans/`](docs/superpowers/plans)
- **REST API:** [`api/README.md`](api/README.md)
- **MCP server:** [`mcp-server/README.md`](mcp-server/README.md)
- **Installer building:** [`docs/building-installer.md`](docs/building-installer.md)
- **CHANGELOG:** [`CHANGELOG.md`](CHANGELOG.md)

---

## Roadmap

### Korte termijn (open punten)

- **TOC paginanummers** — vereist two-pass layout in `openaec-layout` submodule (build → inspecteer section page-positions → rebuild TOC → re-render). Substantieel werk.
- **`.ifcenergy` Phase 2** — envelope splitsen naar per-entity IFCX entries (IfcSpace · IfcWall · IfcSlab · IfcRoof · IfcWindow · IfcDoor · IfcMaterial). Zie [issue #12](https://github.com/OpenAEC-Foundation/open-heatloss-studio/issues/12). Voorbereiding al gedaan via namespace-constants in `crates/isso51-ifcx/src/namespace.rs`.
- **Tabbed views** — meerdere projecten tegelijk geopend, geïnspireerd op Open Calc Studio's `FileTabBar`. Refactor van `projectStore` naar een per-document model.
- **Modeller editable mode** — momenteel read-only viewer; editor-architectuur op branch [`claude/modeller-2d3d-viewer`](https://github.com/OpenAEC-Foundation/open-heatloss-studio/tree/claude/modeller-2d3d-viewer)
- **Echte 3D IFC viewer** in IFC-tab (`@thatopen/components` setup)
- **TitleBar Undo/Redo/Print** quick-access knoppen wiren (zelfde patroon als Save fix)

### Middellange termijn

- **Plugin-runtime** — Extensies-paneel UI staat er; runtime voor `.zip` installatie via manifest-schema komt erbij
- **BAG-data import** — postcode + huisnummer → automatische geometrie + bouwjaar via Basisregistratie Adressen en Gebouwen
- **Vabi Elements converter** — externe Vabi-projecten lezen + naar `.ifcenergy` converteren
- **Quick-calc wizard** — 5–10 min berekening voor klein woonhuis met defaults
- **Diagrammen-verbeteringen** — meer chart-types, configureerbaar per rapport
- **Code-signing** — Authenticode certificaat voor Windows zodat SmartScreen niet meer waarschuwt
- **macOS installer** (`.dmg`) + macOS-specifieke fonts
- **WiX MSI** naast NSIS — voor enterprise rollout via Group Policy / Intune
- **Auto-update** — Tauri updater plugin met signed-update server

### Lange termijn

- **WASM module** — browser-side rekenen zonder server
- **Python package** (PyO3 wrapper) voor scripting + integratie in bestaande NTA-tools
- **DLL via cbindgen** — voor Excel- / Revit-plugins
- **ISSO 53** — utiliteitsgebouwen (kantoren, scholen, ziekenhuizen)
- **ISSO 57** — detailberekening vloerverwarming (warmteflux, leiding-spacing)
- **Tenant huisstijl** — per-organisatie PDF-branding (logo, accent-kleur, contact-footer, decoratieve voorbladen)

---

## Contribueren

Pull requests en issues welkom. Conventies:

- **Rust:** `cargo test` moet altijd slagen; geen `unsafe`, geen `unwrap()` in productie-code
- **Frontend:** `npx tsc --noEmit` clean, geen `as any` casts (gebruik proper types)
- **Eenheden:** mm voor afmetingen, dm³/s voor luchtvolumestroom, W voor vermogen, W/K voor H-waarden, °C voor temperaturen
- **Norm-referenties:** doc-comments verwijzen expliciet naar ISSO 51 formulenummers
- **Verifiëren vóór completion:** elke claim "fixed" / "passing" / "done" moet ondersteund worden door uitgevoerde verificatie (test-output, screenshot, etc.)

Project-specifieke conventies en de Claude Code agent broker config staan in [`CLAUDE.md`](CLAUDE.md).

---

## Licentie

[MIT](LICENSE). Zie ook de licentie van de bundled fonts (Liberation Sans, OFL 1.1) en submodules.

---

## Contact

- **Repo:** https://github.com/OpenAEC-Foundation/open-heatloss-studio
- **Releases:** https://github.com/OpenAEC-Foundation/open-heatloss-studio/releases
- **OpenAEC Foundation:** https://github.com/OpenAEC-Foundation
- **Productie deployment (web mode):** https://open-heatloss-studio.open-aec.com
