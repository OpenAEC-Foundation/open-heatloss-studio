# Rust report-engine integratie — design

**Datum:** 2026-05-09
**Status:** Design — klaar voor planning + implementatie
**Branch:** `claude/laughing-kirch-752da4`
**Volgt op:** alleen geldige PDF-output via remote service; lokaal embedden is missing
**Volgende stap:** plan in `docs/superpowers/plans/2026-05-09-rust-report-integration-plan.md`

## 1. Doel + scope

Open Heatloss Studio (OHS) genereert PDF-rapporten nu door een JSON-payload over HTTPS te POST'en naar een remote OpenAEC Reports service (Python FastAPI). Voor **desktop-only** gebruik (Tauri build, geen netwerk) en voor predictability willen we de PDF-engine **lokaal in de Tauri Rust backend** draaien, zoals Open Calc Studio (OCS) dat al doet.

Concreet: vervang het pad `frontend → /api/v1/report/generate (axum proxy) → remote Reports service` door `frontend → invoke('generate_pdf_report') → openaec-layout (in-process) → bytes/file`. De **JSON-builders in TypeScript blijven onveranderd**; alleen de transport-laag en de renderer veranderen.

Out of scope: branding-revisie (we leveren een neutrale OHS-template als startpunt; finale 3BM-styling is een latere PR), de Python `openaec-reports` service uitfaseren in andere omgevingen, web-app build (die kan voorlopig de remote service blijven proxyen).

## 2. Bevindingen

### 2.1 `openaec-reports-rs` (Rust, private)

URL: `https://github.com/OpenAEC-Foundation/openaec-reports-rs` (private, owner OpenAEC-Foundation, language: Rust, default branch `master`, last push 2026-04-13).

Status (uit eigen `CLAUDE.md`): **Fase 0 — Project Setup + Schema Types**. Rust 2024 edition. Workspace met 4 crates:
```
crates/openaec-core/    # schema, brand, tenant, font_manager (geen rendering)
crates/openaec-server/  # axum API
crates/openaec-ffi/     # cdylib voor C-ABI
crates/openaec-cli/     # CLI tool
```

Dependencies: `typst = "0.13"`, `typst-pdf = "0.13"`, `lopdf = "0.39"`, `image`, `resvg`, `fontdb`, `axum`, `tokio`, `rusqlite`, `jsonwebtoken`. Rendering-pipeline gepland als `JSON → Typst compileren → lopdf stationery merge`.

Belangrijk: **rendering is nog niet geïmplementeerd**. De checklist toont alleen `schema.rs / brand.rs / tenant.rs / font_manager.rs` als done; Typst-rendering en stationery-merge staan op de TODO. Deze crate is dus **op dit moment niet bruikbaar als drop-in PDF-engine**.

### 2.2 `openaec-reports` (Python repo met Rust submap) — wat OCS écht gebruikt

URL: `https://github.com/OpenAEC-Foundation/openaec-reports`.

Naast Python ReportLab (de hoofd-implementatie voor de service) bevat deze repo een **Rust workspace onder `rust/crates/`** met:

| Crate | Doel |
|---|---|
| `openaec-layout` | PDF layout engine — Rust-equivalent van ReportLab Platypus. Public API: `Pt`, `Mm`, `Size`, `Rect`, `Color`, `A4`, `A3`, `Frame`, `Flowable`, `Paragraph`, `Table`, `ImageFlowable`, `Spacer`, `PageBreak`, `PageTemplate`, `DocTemplate`, `DrawList`. Dependencies: `printpdf 0.7`, `ttf-parser`, `image`, `thiserror`. Geen Typst, geen externe runtime. |
| `openaec-core` | Hogere abstracties bovenop layout: `schema.rs`, `brand.rs`, `block_renderer.rs`, `engine.rs`, `font_manager.rs`, `tenant.rs`, `toc.rs`, `special_pages.rs`, `stationery.rs`, `template_loader.rs`, `data_transform.rs`, `kadaster.rs`. **Dit** is de crate die je gebruikt als je de bestaande `report.schema.json` volgt. |
| `openaec-engine` | Hoger-niveau samenstelling. |
| `openaec-server` | Axum HTTP server (parallel aan Python). |
| `openaec-ffi` / `openaec-cli` | Wrappers. |

`schema.rs` bevat de Rust serde-types die 1:1 op `schemas/report.schema.json` mappen: `ReportData { template, project, format (A4/A3), orientation, project_number, client, author, date, version, status, cover: Cover, colofon: Colofon, toc: TocConfig, sections: Vec<Section>, backcover: BackcoverConfig, metadata }`.

Deze repo + Rust-submap is **wat OCS daadwerkelijk integreert** (zie 2.3) — dus dit, niet `openaec-reports-rs`, is onze target dependency.

### 2.3 Open Calc Studio integratie-patroon

Submodule in `.gitmodules`:
```
[submodule "libs/openaec-reports"]
    path = libs/openaec-reports
    url = https://github.com/OpenAEC-Foundation/openaec-reports.git
```

`src-tauri/Cargo.toml` neemt twee crates uit de submap als path-dependency:
```toml
openaec-layout = { path = "../libs/openaec-reports/rust/crates/openaec-layout" }
openaec-core   = { path = "../libs/openaec-reports/rust/crates/openaec-core" }
typst-as-lib   = { version = "0.15.4", features = ["typst-kit-fonts"] }
typst-pdf      = "0.14.2"
```

Interessant detail: OCS gebruikt **layout direct** (niet `openaec-core`'s schema) en heeft een eigen `src-tauri/src/reports/` met domein-specifieke serde-types (`ReportRequest`, `Schedule`, `CostItem`, `OfferteData`, …). De flow:

```
Frontend
  └─ invoke('generate_pdf_report', { request, output_path })
       OR invoke('generate_pdf_preview', { request })  → returns Vec<u8>

src-tauri/src/reports/mod.rs       # serde types + #[tauri::command] entrypoints
src-tauri/src/reports/generator.rs # bouwt flowables met openaec-layout
src-tauri/src/reports/bouw1.rs     # brand-specifieke styling (logo, kleuren)
src-tauri/src/reports/offerte.rs   # tweede rapport-template
```

`generator.rs` gebruikt `use openaec_layout::*;` en bouwt `Vec<Box<dyn Flowable>>` met `Paragraph`, `Table`, `Spacer`, `ImageFlowable`. Page chrome (header, footer, accent line, page numbers) via `PageCallback` — direct draw-ops met `Color::rgb(217, 119, 6)` etc.

`tenants/bouw1/brand.yaml` configureert het brand: kleuren, fonts (Arial), `logos/main`, footer-elementen, table-styles, optionele stationery PDF. `tenants/bouw1/templates/begroting.typ` is een Typst-template — voor cost-schedule rapporten gebruikt OCS Typst pixel-perfect, maar voor de offerte/algemeen blijft layout-engine genoeg. Voor OHS gebruiken we **alleen de layout engine** (geen Typst nodig — de bestaande PDF heeft geen complexe typesetting waarvoor Typst noodzakelijk is).

OCS commit-messages tonen dat de submodule pin via `git submodule update --remote` wordt vernieuwd; `cargo build` in CI vereist `git clone --recurse-submodules`. `cargo` resolved path-deps absoluut; geen registry-publicatie nodig.

### 2.4 Huidige TS-implementatie in OHS

**Frontend bouwers:**

| Bestand | LoC | Output |
|---|---|---|
| `frontend/src/lib/reportBuilder.ts` | 412 | Volledig warmteverlies-rapport (cover, colofon, toc, uitgangspunten, vertrekken-overzicht, room sections, diagrammen, gebouwresultaten) |
| `frontend/src/lib/rcReportBuilder.ts` | (n.b.) | Rc-berekening rapport (constructie-beschrijving, lagen, Glaser, vochtbalans) |
| `frontend/src/lib/reportCharts.ts` | 647 | SVG-generatie + rasterisatie (gestapelde bar, donut, constructie-losses) |
| `frontend/src/lib/reportClient.ts` | 67 | POST naar `/api/v1/report/generate`, return Blob |

Beide builders produceren een `Record<string, unknown>` conform `report.schema.json`: `{ template, format, orientation, project, …, cover, colofon, toc, sections: [{ title, level, content: [{ type: 'table' | 'paragraph' | 'spacer' | 'image' | 'calculation', … }] }], backcover, metadata }`.

`generateReportDirect` in `reportClient.ts`:
- POST naar `REPORTS_URL = "/api/v1/report/generate"`
- Stuurt OIDC Bearer token mee voor user-auth
- Server response: `application/pdf` blob (of error)

**Roeping:** drie plaatsen
- `frontend/src/pages/Results.tsx:43-44` — hoofdrapport
- `frontend/src/pages/RcCalculator.tsx:467` — Rc-rapport
- `frontend/src/components/ribbon/ResultatenTab.tsx:25-26` — duplicate van Results.tsx (ribbon-actie)

**Backend proxy:** `crates/isso51-api/src/handlers/report.rs:31-95` — `generate_report()` proxy. Service-token auth (Authentik `ak-…`) + `X-Original-Tenant` header. Roept `${REPORTS_API_URL}/api/generate/v2`. Faalt met `ServiceUnavailable` als `REPORTS_API_URL` niet geconfigureerd is.

**Wat moet behouden:**
- Alle JSON-bouwers in TS (412+ regels content). Het schema-contract is solide en wordt al door `openaec-core::schema::ReportData` ondersteund.
- SVG-charts → PNG rasterisatie in TS. Het Rust pad accepteert PNG-bytes via `ImageFlowable`.
- OIDC auth voor de **web-mode** (zie sectie 7).

**Wat vervangen wordt:**
- `reportClient.ts::generateReportDirect()` → switcht tussen Tauri `invoke('generate_report', ...)` voor desktop en huidige fetch-call voor web.
- De backend `generate_report()` handler blijft staan voor web-mode, maar krijgt geen wijzigingen in deze PR.

### 2.5 Memeleiland Kavel 4 — voorbeeld-PDF (target output)

Bron: `C:\3BM\50_projecten\7_3BM_bouwkunde\3017 Memeleiland Kavel 4\72_bouwfysica_regelgeving\Memeleiland Kavel 4.pdf`. 31 pagina's, A4 portrait, gegenereerd 2026-04-10 met "Microsoft: Print To PDF" (image-PDF; tekst niet selecteerbaar).

**Pagina-structuur:**

| Pagina | Inhoud |
|---|---|
| 1 | Cover: 3BM logo (linksbovenin), titel "Memeleiland Kavel 4", grote project-foto, drie tags ("MEEDENKEN", "PRAKTISCH", "BETROUWBAAR"), roze/teal accent-vormen, "Ontdek ons 3bm.co.nl" |
| 2 | Colofon: "Memeleiland Kavel 4" als sectiekop, project + opdrachtgever + adviseur metadata, normen, datum, fase, status, kenmerk, revisiehistorie-tabel |
| 3 | Inhoudsopgave: 4 hoofdsecties (Uitgangspunten, Vertrekken overzicht, Diagrammen, Gebouwresultaten), 19 subsecties voor vertrekken |
| 4 | Sectie 1 Uitgangspunten: drie tabellen (Gebouwgegevens, Klimaatgegevens, Ventilatiesysteem) met teal-headers; cursieve voetnoot bij theta_w |
| 5 | Sectie 2 Vertrekken overzicht: tabel "Samenvatting per vertrek" — 8 kolommen, 19 rijen |
| 6 | Sectie 2.1 [BG] Berging: detail-tabellen Transmissieverliezen, Ventilatie & infiltratie, Opwarmtoeslag, Totaal |
| 7-27 | Per-vertrek detail-secties (zelfde layout) |
| 28 | Sectie 3 Diagrammen: gestapelde bar-chart per vertrek, donut-chart gebouwtotaal (6.527 W aansluitvermogen), horizontale bars per constructietype |
| 29 | Sectie 4 Gebouwresultaten: tabel Totalen + highlight-block "Aansluitvermogen 6.527 W (Ref: ISSO 51:2023)" |
| 30 | Lege/witte pagina (collator) |
| 31 | Backcover: paarse hoek linksboven, grote teal driehoeken rechtsonder, 3BM logo gecentreerd, footer "3bm Bouwkunde · Wattstraat 17 · 3335 LV Zwijndrecht · T. 078 7400 250 · Ontdek ons 3bm.co.nl" |

**Brand-elementen:**
- Hoofdkleuren: teal `#3DB7A5` (accent vormen, table-headers, sectie-nummers), donker paars `#3F2649` (logo "3bm" tekst, hoek rechtsboven), tekst `#1A1A1A`, lichte grijs `#A8A8A8` voor secundaire labels
- Logo + onderschrift "Ingenieurs van oplossingen" in elke voet
- Voet bevat altijd de teal driehoek-decoratie linksonder en het 3BM-logo rechtsonder met paginanummer
- Sansserif font (lijkt Lato of soortgelijke)
- Tabellen: solid teal header met witte tekst, witte body met grijze cell-borders

**Wat zit in voorbeeld-PDF dat de TS-builder al produceert:** vrijwel alles. De `reportBuilder.ts` output mapt 1:1 op deze pagina's (cover, colofon, toc, sections met level=1/2, table, paragraph, spacer, image, calculation block).

**Wat de TS-builder NIET produceert:** de **branding zelf** — logo, decoratieve hoekvormen, kleurenpalet, header/footer-chrome — komen uit het server-side template `template: "standaard_rapport"`. Migratie betekent dus: dat brand-template lokaal reproduceren (of pragmatisch eerst neutraal, dan stylen — zie sectie 6).

## 3. Voorgestelde architectuur — alternatieven

### A. Submodule + path-dep (zoals OCS)

```
.gitmodules:                                                                              
  [submodule "libs/openaec-reports"]                                                      
      path = libs/openaec-reports                                                         
      url = https://github.com/OpenAEC-Foundation/openaec-reports.git

src-tauri/Cargo.toml:
  openaec-layout = { path = "../libs/openaec-reports/rust/crates/openaec-layout" }
  openaec-core   = { path = "../libs/openaec-reports/rust/crates/openaec-core" }
```

**Plus:**
- Identiek aan productie-patroon van OCS — bekend werkend
- Updates komen via `git submodule update --remote`
- Kan zonder publicatie naar crates.io werken
- Path-deps zijn cargo-friendly (`cargo build` werkt zonder registry)

**Min:**
- Submodule-mechaniek: clone-instructies veranderen (`--recurse-submodules`), CI moet submodules ophalen, contributors die de stap missen krijgen build errors
- We pinnen aan een commit-SHA in het Python-repo; updates vereisen handmatig `submodule update --remote` + commit
- Repo-grootte stijgt (de Python repo bevat ook Python-broncode die we niet nodig hebben — `cargo build` raakt het niet, maar git fetch wel)

### B. Cargo workspace member (eigen vendored copy)

Kopieer `openaec-layout` (en eventueel `openaec-core`) als nieuwe leden in onze workspace, onder `crates/openaec-layout-vendor/`.

**Plus:**
- Geen submodule, geen externe afhankelijkheden voor `cargo build`
- We kunnen kleine aanpassingen doen (bijv. extra brand-tokens) zonder upstream PR
- Repo blijft self-contained

**Min:**
- Code-duplicatie en fork-drift: upstream bug-fixes en feature-additions handmatig porteren
- Origineel bron-attributie moet duidelijk in de README/headers (LGPL/MIT compliance)
- Wij worden custodian van een copy die we niet ontwikkelen

### C. Crates.io-publicatie

Vraag OpenAEC Foundation om `openaec-layout` te publiceren naar crates.io en pin op `0.1.0`.

**Plus:**
- Mooist: gewoon `cargo add openaec-layout`, semver, geen submodule
- CI is triviaal — geen extra git fetch
- Anderen kunnen de crate ook gebruiken

**Min:**
- Vereist publicatie-workflow van een private/intern project — de eigenaar moet hier akkoord op geven
- Op dit moment **niet beschikbaar** op crates.io (`openaec-reports-rs` is private, `openaec-reports` Rust submap is niet gepubliceerd)
- Update-cadence wordt door upstream gecontroleerd — als we een fix nodig hebben, moeten we een release afwachten of een fork forken

## 4. Aanbeveling

**Optie A: Submodule + path-dep**, identiek aan OCS.

Redenen:
1. **Bewezen werkbaar** — OCS draait er al productie op; we kopiëren een bekend patroon in plaats van iets nieuws uit te denken.
2. **Snelste pad** — geen wachten op publicatie, geen vendoring overhead.
3. **Updates via submodule-bump** — controleerbaar, traceerbaar (commit-SHA in tree).
4. **Rust 2021 compatible** — `openaec-layout`'s eigen `Cargo.toml` zegt `edition.workspace = true` en de Python repo's workspace zet `edition = "2024"`. We pinnen `edition = "2021"` in onze `crates/` (zoals onze huidige workspace doet); het toevoegen van een **path-dep met andere edition is toegestaan in cargo** (compileert per crate). Geen workspace-merge issue. (Zie risk #1 in sectie 8 — testen voor we mergen.)
5. **Migratie-pad open** — als upstream ooit naar crates.io publiceert, switchen we met een 2-regelige `Cargo.toml` aanpassing.

We nemen alleen `openaec-layout` als path-dep; we **niet** `openaec-core`, omdat we onze eigen domein-types houden (zoals OCS doet — zie 2.3). De report-schema die de TS-builders al produceren komt rechtstreeks binnen via Tauri-IPC als `serde_json::Value`, en wij parsen + renderen het zelf.

## 5. Implementatie-overzicht

### Bestand-structuur

| Bestand | Status | Verantwoordelijkheid |
|---|---|---|
| `.gitmodules` | nieuw | Submodule-definitie voor `libs/openaec-reports` |
| `libs/openaec-reports/` | nieuw (submodule) | Bron van `openaec-layout` |
| `src-tauri/Cargo.toml` | wijzigen | Voeg `openaec-layout` path-dep toe + `image`, `serde_json` features |
| `src-tauri/src/reports/mod.rs` | nieuw | Module-root: serde-types voor `report.schema.json` subset, `#[tauri::command]` exports |
| `src-tauri/src/reports/schema.rs` | nieuw | Serde-types voor `ReportData`, `Section`, `Block` (paragraph/table/image/spacer/calculation), `Cover`, `Colofon`, `Toc` |
| `src-tauri/src/reports/generator.rs` | nieuw | Bouwt `Vec<Box<dyn Flowable>>` uit `ReportData` met `openaec-layout` |
| `src-tauri/src/reports/blocks.rs` | nieuw | Per-block-type rendering: `render_paragraph`, `render_table`, `render_image`, `render_calculation` |
| `src-tauri/src/reports/special_pages.rs` | nieuw | Cover, colofon, toc, backcover rendering |
| `src-tauri/src/reports/brand.rs` | nieuw | OHS brand-tokens: kleuren, fonts, logo, page-callback (header + footer + page-numbers) |
| `src-tauri/src/reports/fonts.rs` | nieuw | Bundle Liberation Sans (open-source Arial-equivalent) als embedded TTF — geen runtime install nodig |
| `src-tauri/resources/fonts/LiberationSans-*.ttf` | nieuw | 4 fonts (Regular, Bold, Italic, BoldItalic) — open-source, redistributable |
| `src-tauri/resources/brand/ohs/logo.png` | nieuw | Placeholder OHS-logo (300x80, "Open Heatloss Studio" tekst) — vervangbaar later |
| `src-tauri/src/commands.rs` | wijzigen | Registreer nieuwe commands: `generate_report_pdf`, `generate_report_pdf_bytes` |
| `src-tauri/src/lib.rs` | wijzigen | Mount `reports` module + register Tauri commands in `Builder::default()` |
| `src-tauri/capabilities/default.json` | wijzigen | Allow new commands |
| `frontend/src/lib/reportClient.ts` | wijzigen | Switcht tussen Tauri `invoke()` en HTTP fetch o.b.v. `window.__TAURI__` aanwezigheid |
| `frontend/src/lib/reportClient.tauri.ts` | nieuw | Tauri-pad: `invoke('generate_report_pdf_bytes', {...})` → `Uint8Array` → `Blob` |
| `src-tauri/tests/reports_smoke.rs` | nieuw | Integration-test: feed minimaal `ReportData`, generate PDF, assert > 0 bytes en valid PDF-magic |

### Volgorde

Volgorde is vastgelegd in het plan-document. Hoofdmijlpalen:

1. Submodule toevoegen + Cargo dependency wiring → `cargo build` succesvol
2. Skeleton module + smoke test → genereert lege PDF met cover + footer
3. Block-rendering: paragraph + spacer
4. Block-rendering: table (de meest gebruikte block-type)
5. Block-rendering: image (charts!)
6. Block-rendering: calculation
7. Cover + colofon + toc + backcover
8. Brand-styling (page callback met header, footer, accent-vormen)
9. Frontend wiring (Tauri client switch)
10. End-to-end test (gemockte ProjectResult → PDF → byte-vergelijking)

### Test-strategie

- **Unit-tests in elke module** (`#[cfg(test)] mod tests`) — bv. `render_paragraph` met fixed input geeft fixed flowable-count; `render_table` rejects mismatched row/col-count.
- **Snapshot-test in `src-tauri/tests/reports_smoke.rs`** — genereert PDF voor een fixture `ReportData`, checkt: (a) bytes > 1KB, (b) start met `%PDF-`, (c) `lopdf::Document::load_mem(...)` succeeds, (d) page count == verwacht.
- **Manual visual review** — gegenereerde PDF eerste keer naast Memeleiland-PDF leggen om branding-gap te documenteren (geen pixel-perfect match verwacht in de eerste implementatie).
- **Cargo build-tijd budget** — toevoegen van `openaec-layout` mag de cold cargo-build niet meer dan ~2 minuten extra kosten op CI windows-latest. Meten in CI-job; flag als > 3 min.

## 6. Branding/templates

**Fase 1 — neutraal (in deze PR):**
- OHS-eigen brand-tokens in `src-tauri/src/reports/brand.rs`:
  - Primary kleur: `#0F766E` (teal, ISSO/warmte-thematiek)
  - Secondary: `#374151` (donker grijs)
  - Tekst: `#111827`
  - Tabel-header bg: `#0F766E`, tekst wit
  - Subtle borders: `#D1D5DB`
- Logo: tekst-only PNG "Open Heatloss Studio" (300×80, generated met `image` crate of pre-rendered en commit als binary)
- Page chrome: dunne 0.5mm accent-line bovenin (primary), pagina-x van y rechtsonder, project-naam linksonder, "ISSO 51:2023" middenonder
- Geen decoratieve hoekvormen — die komen in fase 2 als de tenant-config opgehangen wordt

**Fase 2 — tenant-styling (latere PR, niet in deze plan):**
- `src-tauri/resources/brand/ohs/brand.yaml` introduceren naar OCS-pattern
- Optionele `OHS_TENANT_DIR` env-var voor user-override
- Echte 3BM-styling als opt-in tenant
- Stationery-PDF support (achtergrond-laag voor cover/backcover)

**Fonts:** Liberation Sans embedded (open-source, identieke metrics als Arial). Geen runtime install nodig. ~700KB extra binary-grootte voor 4 styles. Alternatief overwogen: gebruik `fontdb` om systeem-fonts te vinden. Verworpen: niet deterministisch, gebruiker zonder Arial krijgt fallback.

## 7. Migratie-pad

**Single-step, geen parallel-run.** De TS-builders veranderen niet; alleen `reportClient.ts` switcht het transport o.b.v. omgeving:

```ts
// reportClient.ts (post-migratie, schematisch)
export async function generateReportDirect(reportData: Record<string, unknown>): Promise<Blob> {
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    return generateViaTauri(reportData);   // nieuw, lokaal
  }
  return generateViaHttp(reportData);      // bestaand, server-side
}
```

**Web-mode** (waar de OIDC-auth en backend-proxy bestaan) blijft ongewijzigd — backend `generate_report` handler blijft staan, OAuth flow blijft. **Desktop-mode** krijgt het Rust-engine pad.

Backend `crates/isso51-api/src/handlers/report.rs` blijft onaangetast in deze PR. Een toekomstige PR kan, indien gewenst, ook web-mode lokaal gaan renderen via een `axum` extractor die `openaec-layout` aanroept (wat dan de remote service helemaal afschaft).

**Backwards compatibility:**
- Bestaande `report.schema.json` JSON's blijven werken — de Rust-renderer parsed dezelfde shape
- Geen frontend types verandering (TS bouwt `Record<string, unknown>` zoals nu)
- Tests die `generateReportDirect` mocken blijven valide (interface ongewijzigd)

## 8. Risico's en open vragen

1. **Cargo edition mismatch.** De Python repo's Rust workspace draait edition 2024; ons workspace draait edition 2021. Path-deps zouden onafhankelijk moeten compileren, maar **nodig om expliciet te testen** voor mergen — als het Rust 2024 features gebruikt en wij stable-2021 zitten, kan compilatie falen. Mitigation: test in Task 2 van het plan (`cargo build` direct na Cargo.toml wijziging). Als het faalt: switchen naar edition 2024 in onze workspace, of vendoren met edition-wijziging.

2. **`openaec-layout` API stabiliteit.** Geen semver-garantie (private repo, edition 2024 active dev). Mitigation: pin de submodule op een specifieke commit-SHA (handmatige bump, niet `--remote` automatisch).

3. **Submodule-CI.** GitHub Actions checkout-action moet `submodules: recursive` hebben. Mitigation: in `windows-installer.yml` (en eventuele andere workflows) updaten in dezelfde PR.

4. **Repo-grootte.** Het `openaec-reports` Python-repo bevat ook Python-bron, voorbeelden, deploy-scripts. Een fresh clone van OHS wordt ~20-50MB groter. Acceptabel.

5. **Open vraag — gebruiker:** Hebben we toegang tot `openaec-reports` (de Python repo met Rust submap)? Het is **public**, dus ja. Maar `openaec-reports-rs` is private en wordt **niet** gebruikt — bevestig dat dit OK is.

6. **Open vraag — gebruiker:** Mag de fontfile `LiberationSans-*.ttf` (SIL OFL 1.1) in de repo gecommit? Nederlandse open-source compliance is doorgaans OK, maar misschien wil je 'em in `git lfs` of `.gitattributes` markeren.

7. **Open vraag — gebruiker:** Plaatsen we het OHS placeholder-logo in de repo gecommit, of genereren we 'em build-time? Generated-build-time is reproducible maar voegt complexiteit toe; gecommit is simpel.

8. **Open vraag — gebruiker:** **Web-mode** — moet die echt blijven, of kunnen we de remote Reports service helemaal pensioen geven? Als web-mode onbelangrijk is voor jou, kunnen we sectie 7's "switch o.b.v. `__TAURI__`" simpeler maken (gewoon altijd via Tauri) — maar dan crashed de browser-mode `npm run dev` omdat `invoke` niet bestaat. Liever de switch.

9. **Open vraag — gebruiker:** Welke versie van Liberation Sans (of moeten we Inter / Roboto gebruiken)? De Memeleiland-PDF lijkt een sansserif-font dat *niet* Arial is (smaller letters, modernere x-height) — wellicht Lato of Open Sans. Default voorstel: Liberation Sans (= metric-compatible Arial); gebruiker kan later switchen via `brand.yaml`.

10. **Open vraag — gebruiker:** Pixel-perfecte match met Memeleiland-PDF in deze PR is **niet** scope. Akkoord? De target is een rapport dat: (a) inhoudelijk identiek is aan wat de TS-builder + remote-service nu produceert, (b) leesbaar/professioneel oogt, (c) een placeholder OHS-brand heeft die later 3BM-stijling kan worden via tenant-config. Final 3BM-stylig (logo, kleuren, hoekvormen, achtergrond-PDF) volgt in een aparte PR.
