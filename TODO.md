# TODO

## 🔍 Audit 2026-07-02 (norm + code + infra) — fix-rondes
- [x] **F1 ✅ (02-07)** — C1 ontwerpbinnentemperaturen naar ISSO 51:2023 Tabel 2.11 (`enums.rs`, `constants.ts`, divergente kopie `ConstructionLossChart.tsx` opgeruimd) · C2 Vabi-mapper pint eigen ontwerptemp via `custom_temperature` i.p.v. `internal_air_temperature`. cargo/clippy/tsc/vitest 318/318 groen, golden-fixtures: portiekwoning gepind, woonboot herijkt (+6,7%).
- [x] **M1 ✅ (02-07)** — Φ_vent = Φ_v − Φ_i (clamp 0) voor systeem A/C, form. 4.4 p.65 + 4.9 p.67 (`crates/isso51-core/src/calc/room_load.rs`); B/D blijft Φ_v, E conservatief Φ_v. Goldens portiekwoning/woonboot geregenereerd (vertrekniveau −3…−14%, gebouwniveau ongewijzigd). 6 nieuwe unit-tests.
- [x] **M2 ✅ (02-07)** — aluminium spacer Ψ_g 0,06→0,08 EN-ISO 10077-1 Annex E (`frontend/src/lib/spacerTable.ts` + 2 UI-defaults `UwCalculator.tsx`) + 3 legacy testfiles naar vitest (318→365 groen) + CI-exclude weg.
- [x] **M3 ✅ (02-07)** — SQLite-pool via SqliteConnectOptions: WAL + busy_timeout 5s + synchronous Normal (`crates/isso51-api/src/main.rs`), was default rollback-journal + 0ms timeout → SQLITE_BUSY bij gelijktijdige saves.
- [x] **M4 ✅ (11-07)** — ISSO 53 §6.1/§6.2-goldens geactiveerd (`crates/isso53-core/tests/golden.rs`, `#[ignore]` weg, beide groen). Commits `0f1258c` (engine+6.2) · `fbe1423` (6.1-rebuild).
  - [x] **M4a ✅** — `calculate_h_t_adjacent_rooms` honoreert nu een expliciete `temperature_factor` direct als f_ia,k (voorrang boven ΔT, spiegelt het Unheated-pad). Φ_T 389,7→525,65 W (+0,12%).
  - [x] **M4b ✅** — bleek al geïmplementeerd (`Room.ventilation_q_v_established`); alleen de §6.2-fixture vulde de gegeven qv=100 m³/h niet in. Φ_vent 88,9→190 W.
  - [x] **§6.1-rebuild ✅** — input modelleert nu de gebouwschil (dak bewust weggelaten, θ_e=-9,5 gepind); bouwtotaal +0,46%, transmissie +0,0004%.
  - [ ] **Nieuw engine-gat: `calc::shell` gedetailleerd maken** — is nu een hoofdstuk-3 voorontwerp-schatting (hardcoded 0,5 ACH / 0,00001), reproduceert gepubliceerde shellHeatLoss niet → op `null`. Apart werkpakket.
  - [ ] **Nieuw engine-gat: directe q_is/A_u infiltratie-invoer** — §6.1 geeft q_is + A_u=halve gevel rechtstreeks; engine kent geen directe invoer, gebruikt volledige gevel → infiltratie +2,67%. Deeltotalen op `null`.
- [x] **M5 grotendeels ✅ (02-07)** — sqlx 0.8.0→0.8.6 (RUSTSEC-2024-0363) + resolver-vereiste rusqlite 0.31→0.32 in isso51-core/vabi-importer (libsqlite3-sys unified 0.30.1), quinn-proto→0.11.15, lopdf dev-dep→0.42.0, react-router(-dom) 7.14.1→7.18.1 (7 CVE's, prod-runtime) + fast-xml-parser/picomatch/postcss/@babel/core. cargo audit 9→6, npm audit 12→5. **Rest:** lopdf@0.31 via printpdf-pin, quick-xml via openaec-cloud-pin + tauri-plist, rsa geen fix beschikbaar; npm 5 resterend = dev-only vitest-toolchain (vereist vitest@4 major-upgrade, aparte chore-ronde).
- [x] **M6 ✅ (02-07)** — alle 6 workflows: 22 action-refs gepind op commit-SHA met tag-comment; reusable `deploy-site.yml` @main → SHA `b86eaa2`. Let op: `rust-toolchain@stable` en overige @main-refs op branch-HEAD gepind, niet op tag — bij upgrade handmatig herresolven.
- [x] **M7 ✅ (02-07)** — forward-auth trust-boundary geverifieerd (`docs/2026-07-02-forward-auth-trust-boundary.md`). Bijvangst: produktie-Caddy had CVE GHSA-7r4p-vjf4-gxv4 (copy_headers stripte client-identity-headers niet) → server geüpdatet naar Caddy v2.11.4 (server-actie, geen repo-wijziging). Open: shared-secret header Caddy↔backend.
- [x] **M8 ✅ (02-07)** — 6 docs geredigeerd: verbatim ISSO/NEN-tabel- en paginatranscripties (~200 regels) vervangen door bronverwijzingen, eigen verificatie-conclusies behouden, redactieregel bovenaan elk doc. HEAD geredigeerd; oude versies in git-history = aparte afweging.
- [x] **Minors ✅ (02-07)** — zones-naam-dedup (`zoneNames.ts` nieuw + ZonesCard + rename-pad, 6 tests), importExport `Array.isArray`-guard op building.zones + zoneGrouping-guard (3 tests), deurspleet invoer-UX (lokale tekststate, blur/Enter-normalisatie), compute-routes: expliciete 2MB body-limit + dependency-vrije per-IP rate-limiter (`ratelimit.rs` nieuw, 30/min default, env-override, 6 tests, ook `/calculate/ifcx`), `.dockerignore` (isso51.db, tenants.json, examples/, tests/ met `!tests/verification/`).
- [x] **M4 afgerond (11-07)** — zie M4-blok hierboven; twee vervolg-engine-gaten (calc::shell detail + directe q_is/A_u) apart genoteerd.
- [ ] **chore: vitest 2→4 major-upgrade** (dev-only vulns, resterend na M5 npm audit).
- [ ] **server: rrsync forced-command op DEPLOY_SSH_KEY** (aanbeveling M7-verwant, infra-actie op deploy-key scope).
- [ ] **docs-history-afweging [PM]** — M8 redigeerde alleen HEAD; oude verbatim-transcripties blijven in git-history bereikbaar. Besluit nodig of dat acceptabel is.
- [ ] **[USER] pachi-fork** — contact opnemen of GitHub-takedown starten (uit F5-audit, geen repo-actie mogelijk vanuit hier).
- [ ] **[USER] GitHub Support cache-purge** — voor beide repos (uit F5-audit, geen repo-actie mogelijk vanuit hier).

---

## 🔍 Fable 5 full-audit (10-06) — fix-rondes
> Bron: `audit-reports/09-fable5-full-audit-2026-06-10.md` (62 bevestigd: 4 critical / 33 major / 25 minor). Rondes daar in §7.
- [x] **R1 stille corruptie ✅ (10-06)** — garage-sentinel isso53 (2 call-sites incl. extra vondst transmission.rs adjacent) · tab-snapshot sidecars+serverbinding · newTab stale-snapshot (bonus-dataverlies-bug) · setResult run-epoch · persist isDirty/serverbinding · undo sidecars. cargo 145 + vitest 191 groen, 2 reviews ship.
- [x] **R2 security ✅ (10-06)** — X-Original-Tenant env-allowlist (TENANT_OVERRIDE_ACCOUNTS, default dicht) · cloud tenant-claim-resolve · 2× path-traversal dicht · optimistic locking atomair · Tauri fs-scope versmald + shell:allow-execute weg · logout-clear serverbinding. cargo 45 + vitest 197 groen, reviews ship.
- [ ] **R2 desktop-naverificatie [user]** — Tauri-build: open/save vanaf netwerkschijf, dubbelklik .ifcenergy, default-save Documenten, recent-file buiten scope (nette fallback), norm-wissel-backup buiten scope.
- [x] **R3 Vabi-import ✅ (10-06)** — temperature_factor per boundary-type · grondvloer 0W→afgeleide GroundParameters+warnings · dwelling_class Tabel 2.8-afleiding · night_setback default false (geen Vabi-veld) · UnconditionedSpace-mapping · extractor custom_temperature+infiltration_method. cargo 197 + 15 tests, reviews ship.
- [ ] **R3-besluit: Rust-mapper θ_i-veldkeuze [PM]** — mapper zet Vabi-ontwerptemp in internal_air_temperature (θ_a) maar custom_temperature=None → Rust-import gebruikt mogelijk tabel-θ_i waar Vabi eigen temps had (extractor doet het wél via custom_temperature). Gelijktrekken kan golden-fixtures verschuiven → eerst PM-analyse.
- [ ] **R3-naverificatie [user/andere machine]** — vabi-importer v2_import-tests (voorweg_210a, 24221) draaien op machine mét de gitignored referentie-.vp's; daarna extractor-fixture-run.
- [x] **R4 rekenkern-normvalidatie ✅ (10-06, PM-geverifieerd tegen norm-PDF's op Z:)** — U_equiv 4.24 norm-vorm (oude vorm: omgekeerde monotonie + misread-ijking) reproduceert beide normvoorbeelden · Ū opwarmtoeslag incl. ΔU_TB+grondvloer · NEN8088 Tabel 10 compleet + ISSO53 Tabel 4.7 bevestigd (twee normen, bewust niet geconsolideerd) · Φ_T,iaBE ≥0-clamp. Geen golden-shift. Review ship.
- [ ] **R4-besluit: isso53 Vabi-compat f_inf [PM, laag]** — compat-pad wijkt af van beide normen (bewust, DR-golden); gedocumenteerd, besluit Vabi-pariteit vs norm bij gelegenheid.
- [x] **R5 ventilatie/BBL ✅ (10-06, Bbl 4.122 via iplo.nl + NEN 1087-PDF geverifieerd)** — per-persoon-eisen utiliteit (onderwijs 8,5 pp; was vlakke 4,0 = >2× onderschat) + indicatief-markering zonder bezetting · overstroomverdeling plugin-port · systeem C max(toevoer,afvoer) · NEN 1087-docverankering spleetformule. 219/219, review ship.
- [ ] **R6 test-infra** · **R7 efficiency/UI** · **R8 cleanup** · **R9 niet-onderzocht (aparte audits)**

---

## 🌦️ KNMI-klimaatlaag + Rc-vergelijk / "WUFI light" (scope 05-06, korte termijn)

> Vervang de hardcoded forfaitaire klimaatwaarden in de vocht/Glaser-keten door een kiesbare KNMI-datalaag, en bouw daarop de geplande "Rc vergelijk"-tool (nu disabled placeholder `Sidebar.tsx:202-208`, `to:""`). 3 work-packages, volgorde WP1→WP2→WP3. Plan-detail WP1: zie sessie-handoff orchestrator + onderstaande beslissingen.
>
> **Vastgestelde beslissingen (user, 05-06):**
> - Databron = **gebundelde** KNMI-datasets (offline, geen live API) + herhaalbaar genereer-script.
> - "Per jaar" = **beide** kiesbaar: historisch kalenderjaar én NEN 5060-referentiejaar.
> - Reikwijdte = **alleen** vocht/Glaser-keten. Warmteverlies-θ_e blijft **norm-vast -10°C** (leeft apart in `constants.ts`/isso51-53, NIET aanraken).
> - **Glaser steady-state winterconditie blijft norm-vast -10°C** → `getGlaserWinterCondition` uit het plan VERVALT; klimaatlaag voedt enkel de jáárbalans.
> - **Default-selectie = `"1991-2020"` normaal** (geen stille resultaatwijziging; seed = huidige 12 waarden bit-gelijk).

### WP1 — KNMI-klimaatdatalaag (fundament) ✅ GEDAAN `fcefb96`
- [x] **Data-schema + `_meta`** — `frontend/src/data/climate/knmiClimate.json` (5 stations, 17 records, `_meta` CC BY 4.0).
- [x] **Generator** — `scripts/generate_climate_bundle.py` (KNMI daggegevens-API + offline etmgeg; dag→maand). **KNMI-fetch gelukt:** 15 historische records (5 stations × 2021/22/23, coverage 1.0).
- [x] **Seed-bundel** — De Bilt 1991-2020 bit-gelijk aan `MONTHLY_CLIMATE_NL` (test-geverifieerd) + 5 MVP-stations met lat/lon. **NEN5060 = eerlijke placeholder** (months=null; betaalde norm, user levert tabel).
- [x] **`frontend/src/lib/climateData.ts`** — `listStations/listAvailableYears/getMonthlyClimate` + 8 tests. Geen `getGlaserWinterCondition`.
- [x] **Scope-guard** — climateData alleen in eigen test geïmporteerd (WP1 standalone).

### WP2 — RcCalculator-upgrade (klimaatkiezer) ✅ GEDAAN `5e0e8a7`
- [x] **Klimaatkiezer-UI** in `RcCalculator.tsx` — station + selectie-dropdowns, default De Bilt/1991-2020 (bit-identiek resultaat). Dual-review ship (3 false-pos).
- [x] **`yearlyMoistureCalculation.ts`** — optionele `climate?`-param; refs vervangen, fallback `MONTHLY_CLIMATE_NL` bij ontbreken/`length!==12`.
- [x] **`glaserCalculation.ts`** — ONGEMOEID (Glaser-winter blijft -10). Bevestigd.
- [x] **NEN5060-fallback** — `getMonthlyClimate`→null → default + inline-melding, geen crash. Rapport toont gebruikt klimaat (`rcReportBuilder.ts`).
- [ ] **Follow-up [M]:** klimaatkeuze nu component-`useState` (niet persistent). Promoveer naar `SharedExtra.glaser_climate?: {stationId, selection}` (`projectV2.ts:599`) zodra Glaser-rapport projectbreed reproduceerbaar moet zijn (persist-keten gefixt in `8ccff9f`).

### WP3 — Rc-vergelijk-pagina (de "WUFI light") ✅ GEDAAN `9f6dd76`
- [x] **`pages/RcCompare.tsx`** (~560 r) + route `/rc-compare` + sidebar geactiveerd. 2 kolommen A/B: constructie-picker (bibliotheek + projectconstructies, kozijnen vallen af), Rc/U + Bouwbesluit-min-check, Glaser-oordeel (-10), jaarbalans (gedeelde KNMI-kiezer voedt beide), GlaserDiagram + MoistureYearTable per kolom, delta-samenvatting. Calc puur hergebruikt.
- [x] **Intentie bevestigd:** was "coming soon"-placeholder naast `/rc` + `/uw` → vergelijk-tool. Nu live.
- [ ] **→ Visuele check door user vereist** (na nginx-deploy) — UI-layout/leesbaarheid, niet alleen build.

### → Resterende follow-ups KNMI-feature
- [ ] **WP2-persistentie [M]** — klimaatkeuze (RcCalculator + RcCompare) is component-state; promoveer naar `SharedExtra.glaser_climate?: {stationId, selection}` (`projectV2.ts:599`) voor reproduceerbare Glaser-rapporten.
- [ ] **NEN5060-data [L, user]** — NEN 5060-maandtabel aanleveren → placeholder-record in `knmiClimate.json` invullen (betaalde norm, niet te fabriceren).
- [ ] **Meer historische jaren/stations [L]** — `scripts/generate_climate_bundle.py` opnieuw draaien met bredere jaar-/stationrange indien gewenst.

---

## 💨 Ventilatiebalans-module (plan: `docs/2026-06-06-ventilatiebalans-module-plan.md`)

> BBL + NEN 1087 + NTA 8800. Mode in de Modeller + eigen tab. Delegatie 1+2 (06-07): datamodel-sidecar, BBL-eis, ventiel-plaatsing, Konva-renderlaag. Delegatie 3+4 (09-06): zie hieronder.

- [x] **Delegatie 3 (09-06)** — zone-balans-zijpaneel + per-vertrek cijfertabel (`VentilationBalancePanel.tsx`, `aggregateVentilationBalance()`), systeem A–D-selector (`VentilationState.system`, default C; plugin kent geen A–E-lijst), personen-toeslag `max(opp×spec, pers×4,0 dm³/s, min)` geport uit plugin r.282-289 (`occupancy` op `VentilationRoomState`). Save→reopen-tests voor beide envelopes.
- [x] **Delegatie 4 (09-06)** — eigen tab `/ventilation` à la TO-juli (`pages/VentilationBalance.tsx`, sidebar-group `ventilatie`, NL+EN i18n); gedeelde bron via `hooks/useVentilationBalance.ts` + `components/ventilation/shared.tsx` (geen state-duplicatie met zijpaneel). Review 2× ship, 0 findings.
- [ ] **→ Visuele check door user** — zijpaneel + `/ventilation`-tab na deploy (build/tests groen, niet visueel bevestigd).
- [x] **Delegatie 5 (10-06) — apart ventilatiebalans-rapport** — pure builder ventilationReportBuilder.ts (uw/rc-patroon, standaard_rapport: uitgangspunten + per-vertrek balans-tabel met column_widths + gebouwbalans), rapport-knop op /ventilation-tab, NL+EN i18n, 15 tests (146/146 groen). Review 2× ship, 0 blockers.
- [ ] **Plattegrond-snapshot in rapport [M]** — Konva stage.toDataURL → base64 image-block; obstakel: FloorCanvas niet gemount vanaf /ventilation → offscreen Konva-Stage vanuit modeldata (±0,5-1 dag, herbruikbaar voor andere rapporten).
- [x] **Delegatie 6 (10-06) — WTW/MV-units + capaciteitstoets** — mechanisme-port (ventilatie_units.json bleek nergens te bestaan → indicatieve seed `data/ventilationUnits.json`, expliciet gemarkeerd): VentilationUnit-datamodel (zone-ready, toewijzing gebouwniveau), capaciteitstoets systeem-bewust (D=max(toevoer,afvoer), C=afvoer, B=toevoer, A=n.v.t.), UnitsCard op /ventilation + compact resultaat zijpaneel, optionele rapport-sectie, persistentie beide envelopes. Bugfix: removeRoom droppte ventilation.system/units (spread-fix + regressietest). 171/171 groen. Review 2 passes, fixes doorgevoerd.
- [ ] **Units-catalogus valideren [S, user]** — seed-data is indicatief; fabrikantgegevens (capaciteit/rendement/geluid) controleren en aanvullen.
- [x] **NEN 1087-exacte spleetformule ✅ (R5)** — C_d=0,6/Δp=1,0 Pa/n=0,5 verankerd in norm (Z: PDF gelezen), kantoor-Δp=2 Pa als constante (`OFFICE_DOOR_GAP_DELTA_P_PA`). Geen extra normpagina's nodig.
- [x] **Deurspleet-calculator /tools/deurspleet (12-06)** — standalone tool conform NEN 1087 spleethoogte-afronding, drempel 20mm, deurrooster-voorstel (indicatieve seed 40%/25% netto-fractie), geluidswerend-pad, vuistregel 12 cm²/dm³/s gereconcilieerd exacte 12,9. 318/318 vitest groen, 2 review-passes ship/0 blockers.
- [ ] **Deurspleet-integratie in ventilatiebalans + rapport-sectie** — vervolg, user-besluit eerst losse tool
- [ ] **pyRevit "Export naar web" + import-keten [M]** — `ventilation.json`-export in `pyrevit-gis2bim` + web-import met merge (revit overschrijven, manual behouden). Apart spoor (revit-bim-specialist).
- [ ] **`deriveModelDoors` blijft stub** — overstroom hangt aan gedeelde wanden; deur-objecten later.
- [ ] **Modeller-zijpaneel eenheden-toggle** — Modeller-zijpaneel laten meeschakelen met eenheden-toggle (`unit`-prop doorgeven in VentilationBalancePanel).
- [ ] **Unit-toewijzing per zone** — unit-toewijzing `zoneId` activeren nu zones bestaan (UnitsCard per zone ipv gebouwniveau).

---

## 🧪 Norm-conformiteit audit (02-06) — VOLLEDIGE LIJST

> Bron: 4 norm-audit-agents (ISSO 51/53 PDF regel-voor-regel) + UI-dekkingsaudit + Codex cross-check + PM-hardverificatie. Detail per item in `audit-reports/00-SAMENVATTING.md` (+ 01-06). Conform-beleid: **hybride** (norm leidend; Vabi-compat alleen achter gemarkeerd pad). Effort: [L]=laag [M]=middel [H]=hoog. ✅=hard geverifieerd.
> **ISSO 53 is voorgetrokken** (blokken A–C) vóór ISSO 51 (D–E).
> **Voortgang:** R1 ✅`f815c1f` · R2 ✅`bb70f7e` · R3a ✅`ce1ff3e` · R3b ✅`42eeeb9` · R4 ✅`fdbf39e` · review 3a+3b ✅ · R5 (ISSO 51 P×A_g) ✅`b65de61` + review-fixes ✅`3ffd13f` · review R5 ✅ (Ollama+coördinator; Codex kon niet — ChatGPT-account) · **R6 backend ✅ — 6a ISSO 53 (K2+V2+C1) 141 groen + 6b ISSO 51 (K3+C2+quick-wins) 177 groen.** **R6c UI ✅ — fase 1 rename+schema-sync (`4359280`) · fase 2 config-velden (`9856074`) · fase 3 rapport-velden. Gebruiker test visueel.** Norm-overhaul compleet. Formules: `audit-reports/07-...md` + `08-...md`.

### 🌅 MORGENOCHTEND — START HIER (aanbevolen volgorde)

> Alle items hieronder staan met detail in blokken A–F. Baseline: `cargo test -p isso53-core` = 111 groen. Werk per ronde: general-purpose agent (NIET rust-developer — worktree-faalt), foreground, daarna `cargo test`, dan git-release commit. Formules: `audit-reports/07-isso53-formules-ref.md`.

1. ~~**Ronde 3a — A5 (ISSO 53 stratificatie Δθ₁ + vide).**~~ ✅ **GEDAAN.** Datalaag `delta_theta_1/_v/_corrected` + `vide_factor` in `tables/temperature_stratification.rs` (12 systemen, volledig getest). Δθ₁ toegepast op exterior horizontaal (4.5/4.6) in `transmission.rs` + `shell.rs` (wanden 1,0). **Adjacent (4.11/4.12 + 4.19/4.20) bewust NIET** — eenzijdige Δθ₁ overschat (+33% artefact op DR-buurplafond); tweezijdige `(θ_i+Δθ₁−(θ_adj+Δθ_a1))` vereist per-element buur-heating_system → A5-vervolg (zie open item onder). Onverwarmd-tak (4.15/4.16) ongemoeid: Δθ₁ hoort bij berekende f_k-route (auto-f_k TODO), niet bij forfaitaire Tabel 4.2. Golden-tests onveranderd groen (geen fixture heeft exterior-horizontaal + Δθ₁>0-systeem). 121 lib-tests groen (+10).
   - [ ] **A5-vervolg [M]** — tweezijdige stratificatie op aangrenzend-vertrek (4.11/4.12) + -gebouw (4.19/4.20): vereist `heating_system` per buur-element in het model. Nu geparkeerd met `// TODO A5-vervolg`-markers in `calculate_h_t_adjacent_rooms/_buildings`.
   - [ ] **U6-afhankelijk** — vide-correctie ×(h/4) is geïmplementeerd maar onbereikbaar zolang room-validatie `height>4m` weigert. Ontgrendelt bij U6 (height-validatie versoepelen + UI-veld).
2. ~~**Ronde 3b — A4 + A7 (ISSO 53 grond + Δθ_v).**~~ ✅ **GEDAAN.** A4: ΔU_TB opgeteld bij U_k vóór 4.24 (`resolve_delta_u_tb()`, zelfde prioriteit als A6). **Grote vondst: `ground_params.rs` U_equiv stond als machtvorm `a·(…)^b` met b=−7,455 → altijd ~1e-13 → stille clamp 0,1 voor élke grondvloer zonder expliciete `uEquivalent`.** Gecorrigeerd naar norm-quotiëntvorm `\|a·b\|/(c₁B'^n₁+c₂(U_k+ΔU_TB)^n₂+c₃z^n₃+d)`; worked-example p.65 (U=2,43→0,1798≈0,177) reproduceert exact. + 2 tabelfouten (Floor `n₃`-teken, `c₃`). A7: form. 4.39 `f_v=(θ_i+Δθ_v−θ_e)/(θ_i−θ_e)` in ventilatie + infiltratie (4.30), met nieuwe `calc/rc_high.rs` (opp.-gewogen R_c van Exterior+Ground ≥3,5 → kolomkeuze). WTW-tak (4.38, θ_t) geparkeerd tot U5. Golden `expected.json` ongewijzigd; houtfabriek/bedrijfsruimte4 snapshots −1,7…−3,5% (vloerverwarming Δθ_v≠0) op norm-waarde geijkt + comment. 133 lib-tests groen (+12).
   - [ ] **A4-vervolg [L]** — PDF-dubbelcheck teller-definitie `a·b` (nu `\|a·b\|` omdat b<0 en norm positieve U_equiv levert; p.65 sluit, maar bevestig de exacte 4.24-teller in de PDF). + grondvloer-fixture die het U_equiv-pad écht raakt (komt mee met D4/Ronde 4, alle huidige fixtures leveren `uEquivalent` expliciet → pad ongetest door golden).
   - [ ] **A7-vervolg [L]** — Vabi past Δθ_v NIET toe op infiltratie; wij wel (norm leidend). Indien Vabi-reproductie gewenst: f_v=1,0-infiltratie achter expliciet Vabi-compat-pad (hoort bij C1/C2, Ronde 6 F-blok). rc_high-scope = strikt Exterior+Ground; Unheated/AdjacentBuilding meenemen = PDF-verificatie (A3-blok).
3. ~~**Ronde 4 — D2 + D4 (ISSO 53 common-case) backend-spoor.**~~ ✅ **GEDAAN.** D2: `VentilationConfig::bouwfase` (`model/ventilation.rs`) + `#[serde(default=Nieuwbouw)]` (backward-compat, géén norm-aanbeveling — projectkeuze via UI), `ventilation.rs` leest config → +89% bevestigd (6,5 vs 3,44 dm³/s·pp). D4: z=0-grondvloer was al opgelost door 3b-quotiëntvorm (audit-tekst sloeg op pre-3b machtvorm); e2e-test toegevoegd (z=0/0,5/5 geldig). Review-guards: z=0-**wand** → `Err(InvalidInput)` (n₃<0 → +inf→stille clamp); `R_SE_GROUND=0,0` in `rc_high.rs` (ISO 6946). 139 lib-tests groen (+6), geen golden-shift. **UI-dropdown (bouwfase) verschoven naar Ronde 6 U-blok.**
   - [ ] **Ceiling-grond z=0 edge** (review-twijfel) — `calculate_f_ig_auto` behandelt Ceiling-grondvlak als floor-params; de z=0-wand-guard raakt alleen `VerticalPosition::Wall`, niet Ceiling. Zeldzaam, noteren bij toekomstig Ceiling-grond-modelleren.
4. ~~**Ronde 5 — ISSO 51 A1 + A2 (opwarmtoeslag 2023-rewrite).**~~ ✅ **GEDAAN (nieuwbouw-scope).** `Φ_hu=P×A_g` met geverifieerde Tabel 2.10 (`audit-reports/08-isso51-opwarmtoeslag-ref.md`), afkoeling 2K/1K, regeltype §4.3.1/4.3.2, thermostaat→Err. Fout-test weg, V1-tests toegevoegd. 170 groen, Vabi-fixtures onveranderd (Φ_hu=0). Bestaande-bouw afkoeling (Afb 2.7) + §4.3.3 y-methode = follow-up (zie D-blok).
5. **Ronde 6 — afronding (LAATSTE).**
   - ✅ **6a ISSO 53 backend (GEDAAN):** K2 gelijktijdigheidsfactor (`simultaneity_factor`, default 1,0, grijpt aan op Φ_source 5.1/5.9) · V2 Φ_V/Φ_I-check gesplitst + toleranties verstrakt (DR Φ_T 10→4%, 3floors totaal 5→2,5%; geen expected-W gewijzigd) · C1 `infiltration_method_origin` (Isso53Norm/VabiCompat) in result.
   - ✅ **6b ISSO 51 backend (GEDAAN):** K3 split `phi_hl_build` (3.12) / `phi_hl_verdeler` (3.13); `connection_capacity` blijft 3.13 (= aansluit-/opwekkervermogen) · C2 `aggregation_method` in result · example-fix (`[[example]] required-features`) · V3 stale comment · formulas.rs doc-mislabel.
   - ✅ **6c UI (frontend) — GEDAAN (3 fasen, gebruiker test visueel).** Stack: **React 19 + Zustand + Tauri**. ISSO 53 onverwarmd/U-velden waren al compleet.
     - ✅ **Fase 1 — veld-rename `f_rh`→`p` / `accumulating_area`→`a_g` GEDAAN** (cross-cutting door hele stack: `result.rs`, `calc/room_load.rs`, `lib.rs`-test, `isso51-ifcx/namespace.rs`+`to_ifcx.rs`, `gen_pdf.rs`, `result.schema.json`, `types/result.ts`, `reportBuilder.ts`, `isso53ChartData.ts`). Norm-symbolen P/A_g, consistent met struct-conventie. cargo 177+8 groen, frontend build groen.
     - ⚠️ **PIPELINE-VONDST (kritisch voor fase 2/3):** (a) `json-schema-to-typescript` (`json2ts`) ontbrak volledig → `npm run generate-types` was kapot. Nu als devDependency toegevoegd. (b) De gecommitte schemas liepen achter op het Rust-model sinds R4/R5/R6 → nu **bijgetrokken via `cargo run -p isso51-core --example gen_schemas`** (puur additief: `Building` kreeg `built_after_2015`/`heating_control_type`/`c_eff`/`all_floor_heating`, `Room` kreeg `air_source_room_id`, nieuw enum `HeatingControlType`; result kreeg R6-velden). (c) **`npm run generate-types` MAG NIET volledig gedraaid worden** — json2ts degradeert hand-getunede types in `project.ts`/`result.ts` (HashMap/array-velden → `{}`, bv. `ConstructionElementLayer[]`, image `data/media_type`, plus het handmatige `Building.default_heating_system`). **Fase 2/3: voeg benodigde typevelden SURGISCH toe** aan `project.ts`/`result.ts`, draai NIET de generator. Schemas zijn nu wel honest (cargo-output, deterministisch).
     - ✅ **Fase 2 — config-invoervelden (commit `9856074`).** ISSO 51 (`Building`, `WarmteverliesInstellingen.tsx`): `built_after_2015`, `heating_control_type` (per_zone/self_learning/room_thermostat), `all_floor_heating`, `c_eff`. ISSO 53 (`Isso53BuildingFields.tsx`): `bouwfase` (nieuwbouw/bestaand) + `simultaneity_factor`. **Norm-split-vondst:** `simultaneity_factor`+`bouwfase` zitten in isso53-core (NIET project.schema/isso51) → in `projectV2.ts` getypeerd, niet project.ts. Store undo-aware + legacy-backfill, mapper-doorgifte (camelCase serde-match geverifieerd). bouwfase in `Isso53BuildingFields` i.p.v. `VentilationPanel` (dat is V1/isso51).
     - ✅ **Fase 3 — rapport-weergave (deze commit).** ISSO 51 (`reportBuilder.ts`, types in `result.ts` BuildingSummary): `phi_hl_build`/`phi_hl_verdeler` (K3) + `aggregation_method` (C2). ISSO 53 (`isso53ReportBuilder.ts`, types in `isso53Result.ts`): `heating_up_simultaneity_factor` (K2) + `infiltration_method_origin` (C1, nieuw type `InfiltrationMethodOrigin` = isso53Norm/vabiCompat). Enum→leesbare NL-labels. ISSO 51-velden optioneel (oude responses), ISSO 53 non-optional (geen serde-default).
     - 🔍 **UI-testen door gebruiker vereist** (visueel) — niet alleen build-check.
   - ⬜ **Resterende laag-prio backend (latere sessie):** A3-twijfelitems + A4-vervolg `\|a·b\|`-teller PDF-check (ISSO 53) · bestaande-bouw afkoeling Afb 2.7 + §4.3.3 y-methode (ISSO 51) · A5-vervolg tweezijdige adjacent-stratificatie.


### A. ISSO 53 — calc-conformiteit (urgent eerst)
- [x] **D1 [L] LANDMINE** ✅ `f815c1f` (resolve_theta_i helper) — `tables/temperature.rs:21,93` sentinel `f64::MIN` voor `Garage` wordt door callers (`calc/transmission.rs:38`, `ventilation.rs:71`, `infiltration.rs:94`) NIET vervangen door θ_e → `H×(f64::MIN−θ_e)` = **oneindig/astronomisch verlies**. ✅ Fix: enum/Option of sentinel centraal resolven.
- [x] **D2 [M]** ✅ GEDAAN Ronde 4 — `VentilationConfig::bouwfase` + serde-default Nieuwbouw; calc leest config. UI-dropdown = Ronde 6 U-blok.
- [x] **D4 [M]** ✅ GEDAAN Ronde 4 — z=0-grondvloer geldig (al opgelost door 3b-quotiëntvorm; e2e-test z=0/0,5/5 toegevoegd). z=0-wand → Err.
- [x] **D3 [L]** ✅ ronde 2 (resolve_building_dimensions helper) — `calc/infiltration.rs:117-119,134-136` `Unknown`/`UnknownVabiCompat` negeren `building_length/width/height` → f_wind=1,0 i.p.v. ~1,29 (~22% te laag). Fix: methode-dimensies gebruiken of verplicht maken.
- [x] **A6 [L]** ✅ `f815c1f` (shell.rs = transmission.rs) — `calc/shell.rs:52-56` ΔU_TB-prioriteit omgekeerd t.o.v. `transmission.rs` (forfaitair wint, custom genegeerd) → tot kW-orde voorontwerp.
- [x] **A4 [M]** ✅ GEDAAN Ronde 3b — ΔU_TB in U_k + U_equiv machtvorm→quotiëntvorm gecorrigeerd (was stille clamp 0,1) + 2 Tabel-4.3-fouten. Worked-example p.65 reproduceert. PDF-dubbelcheck `a·b`-teller = A4-vervolg.
- [x] **A7 [M]** ✅ GEDAAN Ronde 3b — form. 4.39 in ventilatie + infiltratie (4.30) via `delta_theta_v` (datalaag 3a) + nieuwe `calc/rc_high.rs` voor kolomkeuze. WTW-4.38-tak geparkeerd tot U5. Vabi-divergentie op infiltratie = A7-vervolg.
- [ ] **A3 [M]** — `calc/heating_up.rs:106-110` §4.8.3-reductie `−H_v·Δθ` wordt via project-brede vlag óók op natuurlijk geventileerde ruimten toegepast → Φ_hu te laag/0.
- [x] **K2 [M]** ✅ GEDAAN Ronde 6a — `HeatingUpConfig.simultaneity_factor` (serde-default 1,0) grijpt aan op Φ_source (5.1+5.9); per-vertrek φ_hu + rapporttotaal ongereduceerd. + `BuildingSummary.heating_up_simultaneity_factor` voor transparantie.
- [x] **A5 [H]** ✅ GEDAAN Ronde 3a (Δθ₁ exterior + vide-datalaag + Δθ_v-datalaag; adjacent geparkeerd) — PDF-bevestigd (tab 2.3 p.21-22 + voetnoot 2) — `tables/temperature_stratification.rs` had alléén Δθ₂ (1 call-site `ground.rs:189`, correct). Ontbreekt: **Δθ₁** (+4/+3/+2/+1/0/0,5 per systeem; nodig in form. 3.4/3.5, 4.5/4.6, 4.11/4.12, 4.15/4.16, 4.19/4.20 → ~+10% op dak/vloer-boven-buitenlucht), **Δθ_v** (=A7), Δθ_a1/Δθ_a2, en vide-correctie **Δθ₁×(h/4)** bij h>4m (voetnoot 2). Volledige tabel in `audit-reports/00-SAMENVATTING.md`. Mogelijk verklaart dit de verborgen +5,0% op dak-zwaar vertrek 3.10a.
- [ ] **D5 [H]** — `calc/shell.rs:88-94` voorontwerp-schil grove vaste aannames (0,5 ach + 0,00001 m³/s·m²) = niet norm-conform hfst 3. Fix: hfst 3 implementeren of API als niet-normatief labelen.

### A2. ISSO 53 — stille-fout defaults (fout antwoord zónder error)
- [x] **B1 [L]** ✅ `f815c1f` (InvalidHeatingUpParameters error) — `calc/heating_up.rs:97` `unwrap_or(0.0)` bij ongeldige setback-uren/graden → Φ_hu verdwijnt geruisloos.
- [ ] **B2 [L]** — `model/project.rs:27` `#[serde(default)]` → ontbrekend `heatingUp`-blok = Φ_hu=0 hele gebouw (third-party import ~10-28% te laag). Fix: expliciete waarschuwing/error.
- [x] **B3 [L]** ✅ ronde 2 (benoemde consts DEFAULT_OCCUPANCY_DENSITY/VENTILATION_RATE) — `calc/ventilation.rs:108,117` magic `unwrap_or(0.05/6.5)` zonder rapport-spoor.

### A3. ISSO 53 — twijfel (PDF-verificatie vóór fix)
- [ ] Formule 4.24 exacte `U_equiv`-machtsstructuur — `tables/ground_params.rs` geeft OCR-onzekerheid toe (verifieer tegen worked example p.65: U=2,43→U_equiv=0,177).
- [ ] Tabellen 4.13/4.14 dash-cellen — mag `tables/heating_up.rs:166-198` nearest-defined fallback gebruiken?
- [ ] Tabel 4.10 — behandeling afzuig/overstroomlucht in sanitair + keuken.
- [ ] Dode params: `material_type` (claimt ΔU_TB-invloed die niet bestaat — `DELTA_U_TB_DEFAULT` is constant) + `theta_b_adjacent_building` (hardcoded 15°C in `transmission.rs:178`).

### B. ISSO 53 — UI-veld-dekking (calc-input zónder invoerveld → stille default)
- [ ] **U1** — `source_zone_config` niet gemapt → Φ_source altijd z=0,5; gescheiden opwekker (z=1,0) onbereikbaar.
- [ ] **U2** — `unheated_space`-enum (15 norm-varianten tab 4.2) niet kiesbaar → reductiefactor altijd 0,5.
- [ ] **U3** — koudebrug-toggle + custom ΔU_TB geen UI → forfaitair altijd aan (raakt A6).
- [ ] **U4** — grond-params (u_equiv, f_gw, perimeter/diepte) alleen via thermal-import; f_gw altijd 1,0.
- [ ] **U5** — voorverwarming (`has_preheating`/temperatuur) geen UI.
- [ ] **U6** — vide/vertrekhoogte >4m: per-vertrek-calc leest `room.height` niet (raakt A5).

### C. ISSO 53 — testdekking
- [x] **V2** ✅ GEDAAN Ronde 6a — toleranties verstrakt tot net boven de werkelijke afwijking (DR Φ_T 10→4%, DR Φ_I 5→2,5%, 3floors totaal 5→2,5%, Φ_I eigen 4%), geen expected-W gewijzigd.
- [x] Split `vabi_golden.rs:37` ✅ GEDAAN Ronde 6a — Φ_V (=0, WTW) + Φ_I apart i.p.v. gecombineerd.
- [ ] Test bestaande-bouw ventilatiefase (dekt D2) + afzuig-only toilet/bad/keuken-eisen.
- [ ] End-to-end fixture met `source_fraction_z` (bronvermogen 5.1/5.9 heeft alleen synthetische units).
- [ ] Guard/test voor vertrekhoogte >4m (scope-grens, raakt A5).
- [ ] Fixture mét nachtverlaging die Φ_hu écht uitvoert.

### D. ISSO 51 — calc-conformiteit
- [x] **A1 [H]** ✅ GEDAAN Ronde 5 (nieuwbouw-scope) — 2017 `f_RH × ΣA_metselwerk` volledig verwijderd; `Φ_hu,i = P × A_g` (Form. 4.15) met **visueel-geverifieerde Tabel 2.10** (50 cellen, `audit-reports/08-isso51-opwarmtoeslag-ref.md`). `A_g = room.floor_area` per-vertrek (§4.3.1). Fout-codificerende test verwijderd. 170 tests groen.
  - [ ] **A1-vervolg [M]** — schil-context §3.3 (`A_g = grootste verblijfsgebied`): engine heeft geen schil-only rekenpad; hergebruik `building_thermal_mass`+`newbuild_cooling_k` als dat pad komt. + **veld-rename** `HeatingUpResult.f_rh`→P / `accumulating_area`→A_g (nu herbestemd met doc-comment, niet hernoemd om frontend/ifcx niet te breken) = Ronde 6.
- [x] **A2 [M]** ✅ GEDAAN Ronde 5 — afkoeling: nieuwbouw→2K, **Ū≤0,50→1K** (uit `u_bar`); zwaarte `c_eff≤70→ZL+L+M` else Z; opwarmtijd default 2h (Afb 2.6). Δt-uit-`building_type`-tabel weg.
- [x] **A1b** ✅ GEDAAN Ronde 5 — §4.3.1 P×A_g / §4.3.2 zelflerend→0 / vloerverw.-overal→0 / geen-nachtverlaging→0. **§4.3.3 kamerthermostaat → harde `InvalidInput`-error** (bestaande-bouw, buiten nieuwbouw-scope; géén stille 5 W/m²-gok).
  - [ ] **A1b-vervolg [M]** — bestaande-bouw: Afb 2.7-afkoeling-grafiek + §4.3.3 y-procentmethode (Form. 4.16/4.17). Buiten nieuwbouw-scope, gemarkeerd met `// TODO Ronde 5-vervolg`.
- [x] **K3 [M]** ✅ GEDAAN Ronde 6b — split `phi_hl_build` (3.12, zonder sys.verliezen) / `phi_hl_verdeler` (3.13, met). `connection_capacity` blijft 3.13 (=aansluit-/opwekkervermogen, minste breuk). Additieve velden, golden onveranderd (sys=0 → 3.12==3.13).
- [x] **vabi_import.rs [L]** ✅ GEDAAN Ronde 6b — `[[example]] required-features=["vabi-import"]` in Cargo.toml; alleen `vabi_import` had het nodig.

### E. ISSO 51 — testdekking
- [x] **V1** ✅ GEDAAN Ronde 5 — unit-tests mét nachtverlaging die de `P×A_g`-kern écht uitvoeren (2K/Z/2h→P=22, 2K/ZL+L+M/2h→P=13, 1K/ZL+L+M/2h→P=7 tegen Tabel 2.10) + Ū≤0,5→1K-clamp + zelflerend→0 + thermostaat→Err.
- [x] **V3** ✅ GEDAAN Ronde 6b — header herschreven naar actuele kwadratische-som-staat (DR slaagt ~6700 W); achterhaalde "moet falen"-claim weg.
- [ ] `integration_test.rs:323-334` slaat per-veld-checks over voor ruimten <1 W → kan teken-/componentfouten verbergen vóór clamp.

### F. Cross-cutting / Vabi-keuzes (hybride: markeren + dubbel testen)
- [x] **C1** ✅ GEDAAN Ronde 6a — `result::InfiltrationMethodOrigin{Isso53Norm,VabiCompat}` + `BuildingSummary.infiltration_method_origin` (Δp=3,14 = VabiCompat expliciet in result).
- [x] **C2** ✅ GEDAAN Ronde 6b — `BuildingSummary.aggregation_method` surfaced in result (VabiCompat-default niet omgegooid; NormStrict §3.5.1 ongewijzigd geverifieerd). formulas.rs Tabel-2.10 doc-mislabel ook gecorrigeerd.
- [ ] **frost_protection** — orphan in isso53-mapper (stuurt altijd null), wél isso51-relevant → opruimen of wiren.

---

## 🔍 ISSO 53 warmteverlies — ventilatie + onverwarmd (02-06, Reddingspost Kijkduin, 256 m² utiliteit)

> Context: gebruiker valideerde een ISSO 53-utiliteitsproject (reddingspost, kleedkamers/techniek/berging). 02-06 zijn 10 commits gemaakt (zie `sessions/warmteverlies_latest.md` in de orchestrator). Onderstaande items staan nog open; de oorspronkelijke 4 meldingen van 01-06 zijn opgelost of doorontwikkeld.

### ✅ Opgelost 02-06
- Berekenen crashte (serde regime `9c2bb2b`); opslaan verloor ISSO 53-config (`3e29bf4`, nu `.heatloss.json` met norm+sidecars); ruimte zonder ventilatie-eis crashte (`d32d497`).
- Ventilatie-rij: **vastgestelde toevoer-q_v** stuurt de calc (leeg=BBL-placeholder 0,9 dm³/s·m²), met **BBL-min / personen-min / gekozen** in de rij + snelknoppen (`5e9834d`/`365556b`/`ac62b4b`). Vervangt #2 "ventilatie te laag" + #4 "personen-ventilatie tonen".
- Chart transmissie: **onverwarmd eigen categorie** + f_k=0,5 i.p.v. volle ΔT + ISSO 53-temps (`95873cf`). Het "8000W naar binnenwanden" was puur deze weergavebug — echte binnenwanden = netto −772W.
- **f_k per onverwarmde ruimte instelbaar** (`5584384`), default 0,5, override per ruimte.

### ⬜ Open — calc/feature
- [ ] **Auto-f_k voor onverwarmde ruimtes** = `H_ue / (H_iu + H_ue)` uit de geometrie van de onverwarmde ruimte (ISSO 53 §4.4 / tabel 4.2). Goed geïsoleerde, "meeverwarmende" ruimtes → f_k≈0 → verlies ~0. **Geverifieerd op dit project: Berging 0,030 · Meterkast 0,026** (i.p.v. 0,5 → 16× lager, verlies 3843W→~230W). Handmatige `unheatedFactor` (`5584384`) blijft als override. Plek: `lib/isso53Unheated.ts` (helper aanwezig: `collectUnheatedTargetIds`) + `isso53ProjectMapper.ts` + chart `deltaT.ts`.
- [x] **Per-ruimte "Onverwarmd"-toggle** — checkbox + f_k-veld per ruimte (`Isso53RoomState.isUnheated`). Aanvinken → wanden van buren naar die ruimte worden als `unheated` geëmit met de f_k van de ruimte. Lost de inconsistente import-markering op (Techniek/afval als 10°C adjacent_room → nu handmatig op onverwarmd te zetten, f_k≈0,03 → ~0 verlies).
- [ ] **Onverwarmde ruimte uit gebouwtotaal halen.** Een als onverwarmd gemarkeerde ruimte telt nog steeds als eigen (10/15°C) ruimte mee in het totaal → kleine dubbeltelling met de buren-f_k-route. Flagged-unheated rooms zouden geen eigen verwarmingsvraag moeten produceren (hun schilverlies loopt via de buren-f_k).
- [ ] **Auto z-factor infiltratie (tabel 5.1) uit kompasrichtingen.** De z (1,0 / 0,7 / 0,5) hangt af van de gevel-configuratie per vertrek: 1 buitengevel of 2 niet-tegenover → 1,0; 2 tegenover elkaar → 0,5; overig → 0,7. Nu handmatig per ruimte, default 1,0 (max/conservatief → infiltratie hoog). De import heeft per wand een `compass` (N/O/Z/W) → z automatisch afleiden: heeft een vertrek exterior-wanden op tegenoverliggende richtingen → 0,5; één richting → 1,0. Analoog aan auto-f_k. `crates/isso51-core/src/import/thermal.rs` (kompas aanwezig) + `isso53Ventilation`/sidecar + UI z-dropdown (`Isso53RoomFunctionCell.tsx`).
- [ ] **Opwarmtoeslag §4.8 valideren tegen Vabi** — formule matcht PDF p.66 (test `regression_isso53_example_p66`), maar nog geen Vabi-ijkpunt voor dit project. In de huidige config staat `setbackActive=false` → φ_hu=0, dus alleen relevant zodra setback aan gaat. `crates/isso53-core/src/calc/heating_up.rs`.
- [ ] **Onverwarmde ruimtes lichte dubbeltelling** — Meterkast/Bergingen tellen óók als 15°C-ruimte mee in het gebouwtotaal (+365W netto). Conceptueel dubbel (onverwarmd-buur én 15°C-ruimte).

### ⬜ Open — opschoning/weergave
- [ ] **supply-toggle opruimen** (`514bbf9`, `has_mechanical_supply`-gate) — overbodig geworden nu de vastgestelde q_v leidend is (leeg/0 = geen toevoer). Verwarrend in de UI voor ISSO 53.
- [ ] **Chart adjacent_room: bruto-positief vs netto** — de chart sommeert alleen positieve bijdragen (1662W) terwijl de calc netto −772W oplevert (koude ruimtes winnen terug). Overweeg netto tonen of het label verduidelijken.
- [ ] **`.ifcenergy`-export draagt ISSO 53-sidecars niet** — alleen `.heatloss.json` persisteert norm+sidecars. Bij opslaan als `.ifcenergy` gaat ISSO 53-config verloren.
- [ ] **Infiltratie z-reporting inconsistentie** — `result.summary.infiltrationReductionFactorZ` toont `0.5` (oud ISSO 51-gebouwveld) terwijl de ISSO 53-calc de **per-ruimte** z gebruikt (default 1,0). Verwarrend in de samenvatting. Laat de gerapporteerde z matchen met wat de calc gebruikt (of verberg 'm bij isso53). 02-06 verifieerd op Reddingspost: infiltratie 5248W = q_is(0,00064)×A_u(231,6)×1200 met z=1,0 (impliciete factor exact 1,000 per ruimte) — rekenkundig correct, maar z=1,0 overal = conservatief.
- [ ] **Ventilatie-feedthrough — GEDIAGNOSEERD 03-06: stale result, geen calc-bug.** Op `Reddingspost_kijkduin.heatloss.json` (03-06) phiV per ruimte exact terug te rekenen op de **personen-fallback** (q_v=None-pad: `floor_area×0,05×6,5/1000×1200×f_v×ΔT`) i.p.v. de ingevulde q_v (Instructie 125→35W, Ieeftuimte 150→77W, Politiepost 75→0W via supply-gate). Mapper (`isso53ProjectMapper.ts:227` `ventilation_rate/1000`, 0 blijft 0) én Rust (`calc/ventilation.rs:96` vastgestelde q_v overruled gate, getest) zijn **correct**; het opgeslagen result dateert van vóór de q_v-invoer. Verse Berekenen → verwacht Instructie ~900W / Ieeftuimte ~1080W / Politiepost ~540W, totaal ~2520W (systeem D + WTW 80%). **Open vraag:** waarom blijft het result stale terwijl transmissie wél vers is — onderzoek de recompute-trigger (`/calculate_v2`-aanroep vanuit Results/save): wordt ventilatie bij élke Berekenen herrekend, of mist er een invalidatie na een q_v-edit? Zo niet → echte trigger-bug.
- [ ] **Rust `temperature_factor` `#[serde(default)]`** ontbreekt (`room.rs`); third-party clients zonder dit veld falen. Mapper vult het nu altijd, dus geen blocker.

---

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
- [x] **F0 — 3 BENG-fixtures uit RVO voorbeeldconcepten** ✅ (11-07) — Tussenwoning M (G13), Hoekwoning M (G11), **Vrijstaande L** i.p.v. M (Vrijstaande M bestaat niet als BENG-referentie), elk 3 concepten (9 cases) + 2 Uniec certified-replay (Gouda 2467, Aalten 2522). Rode goldens (`#[ignore]`, `compute_beng` volgt in F2) in `crates/openaec-project-shared/tests/beng_golden.rs`. Per-gevel geometrie (RVO "Bijlage 4"-Excel) ontbreekt nog — user vraagt op; F0 start met 2017-referentiegebouwen-PDF als geometriebron. Zie plan: `docs/2026-07-11-beng-onderzoek-implementatieplan.md` (F0 ✅ · F1a ✅ · F1b ✅ (TOjuli §5.7, QC-reviewed) · **F2 ✅ (11-07, F2a+F2b)** · F3-F5 open).
- [x] **F2 — `compute_beng(ProjectV2)` end-to-end orchestrator** ✅ (11-07) — F2a: additief energy-invoerblok op ProjectV2 + manifest-resolver (`nta8800-model::resolve_zone`). F2b: demand-tak hergebruikt de gevalideerde `compute_tojuli_full`-keten (volume→H_ve→τ gesloten); mapping-laag DTO→runtime met alle forfait-defaults op één plek; fan-out heating/dhw/cooling/ventilator-aux/PV/automation → EpInputs → `calculate_ep_score` → BENG 1/2/3-toets (Bbl 4.149) + TOjuli-screening + label; keten-volgorde en carrier-mapping naar referentie-orchestrator van Maarten Vroegindeweij (origin/claude/nta8800-core), zijn invoermodel niet overgenomen. F0-goldens blijven `#[ignore]` — kalibratie is F3.
  - [ ] **F3 — kalibratie tegen F0-goldens** — smoke-run Tussenwoning M: B1 +68%, B2 +167%, B3 −59pp; structureel EP-laag door vier gemeten gaten:
    - [x] **F3a ✅ (11-07)** BENG 3 renewable-share: RER-formule 5.3 (§5.3.1.3) incl. WP-omgevingswarmte Q_use×(SCOP−1) (form. 5.31/5.36, fPren=1,0 tabel 5.4)
    - [x] **F3a ✅ (11-07)** PV-netting §5.5: `fPrim(HernieuwbareElektriciteit)` 0→1,45 (tabel 5.2) + CO2-factor PV 0→0,0900 kg/MJ (tabel 5.3); negatief EP-totaal → A++++ (§5.5.2 opm. 11). Smoke all-electric WP: BENG 3 0%→20,5% zonder PV, 74,2% met 4 kWp.
    - [x] **F3b ✅ (11-07)** koel-COP FreeCooling ≈1 → koeling domineerde BENG 2: twee-termen-opwekking `Q_gen_out·[factor/EER_fc + (1−factor)/EER_backup]` (EER_fc=10 tabel 10.34, backup-EER=3,0 tabel 10.29, §10.5) + rencold-term additief (§5.6.2.2 form. 5.34, drempel EER≥8, fPren=1,0 tabel 5.4). Smoke WP-bodem: B2 75,5→41,8 · B3 20,5%→52,1% · koeling 56,2→22,5 kWh/m². Norm-analyse: `docs/2026-07-11-f3b-norm-analyse-koeling.md`.
    - [x] **F3c ✅ (11-07)** TOjuli per-oriëntatie §5.7.2-opdeling: 8 kompasrichtingen, maatgevend = max, toets 1,20 K per oriëntatie → pass/fail nu ook zonder actieve koeling (was pass=None); noemer norm-conform (A_T;or formule 5.41, horizontale elementen ≤5° helling §7.6.6.4 + H_ve/H_gr/C_m pro-rata, oriëntatiegebonden = azimuth aanwezig én helling >5°); teller = whole-zone Q_C;nd;juli zonwinst-gewogen verdeeld (gedocumenteerde benadering, norm-exacte per-oriëntatie-julibalans = F3d). Review-fix: dakvlak-classificatie op helling i.p.v. orientation_deg-aanwezigheid. Smoke zonder koeling: ZW maatgevend 18,8 K (overschat door F_sh=1,0 → F3d); met koeling 0/pass. Norm-analyse: `docs/2026-07-11-f3c-norm-analyse-tojuli.md`.
    - [x] **F3d-1 ✅ (11-07)** beweegbare zonwering §7.6.6.1.4 (form. 7.42/7.43) — `Window.movable_shading` additief (F_c + ManualResidential/Automatic); nieuwe `nta8800-demand::calc::shading` met f_sh;with-maandprofielen (tabellen 7.7/7.9, verticaal/45°/horizontaal) en r_mi = (1−f_sh;with) + f_sh;with·F_c per raam per maand op de zonwinst; DTO `Opening.movable_shading` + mapping; whole-zone shading_factor gedocumenteerd multiplicatief; default = geen zonwering = byte-identiek gedrag (regressie-pin-test). Smoke WP-tussenwoning met handbediende screens F_c=0,20: B2 41,8→33,4 · TOjuli 18,8→12,6 K · B1 60,9→40,5 (ondershoot = ontbrekende §17.3-belemmering, F3d-2). Norm-analyse: `docs/2026-07-11-f3d-norm-analyse-beschaduwing.md`.
      - [x] **F3d-2 ✅ (12-07)** §17.3 F_sh;obst — belemmering tabel 17.4 (minimale belemmering, verticaal/45°/horizontaal, PDF-steekproef 5/5 exact) via additief `Window/Opening.obstruction` (None/Minimal, default byte-identiek); tabel 17.5 triviaal 1,00 bij minimale belemmering (horizonblokkering raakt hoge zomerzon niet).
      - [x] **F3d-3 ✅ (12-07)** balans-splitsing Q_sol H/C-variant — Q_gn = Q_int + Q_sol nu apart voor warmte- (f_sh;with=0, §7.6.6.1.4 woningen) en koelbalans (f_sh;with-maandprofiel), elk eigen γ/η. Smoke WP-tussenwoning: B1 60,9→41,2 · B2 41,8→33,7 · koeling 22,5→13,9 kWh/m².
      - [x] **F3d-4 ✅ (12-07)** F_c-tabellen 7.5/7.6 (p.199) als consts verankerd.
      - [ ] **F3d-5** — helling-interpolatie f_sh;with (nu 3 discrete standen verticaal/45°/horizontaal) + tussenhellingen = V2
      - [ ] **F3d-6** — F3d-goldens activeren: **geprobeerd 12-07, 0/5 geactiveerd** (anti-fudge: `expected.json`/`input.json` onaangeraakt, gaps gemeten en gedocumenteerd in `#[ignore]`-redenen + README's). Geblokkeerd op: (a) RVO-cases (3×) — per-gevel-geometrie zit in niet-publieke Bijlage 4-Excel [USER moet opvragen]; `input.json` blijft documentatie-only. (b) ✅ opgelost door F3d-7 (`fe7cd41`) — was PV-west/noord ≈0 door cos-clamp zonder hoek-wrap. **Nieuwe dominante gap Uniec Gouda/Aalten:** PV-over-netting jaarbasis (Gouda B2 −8,2 vs cert 27,48) → F3d-8-heranalyse. Goldens blijven `#[ignore]`. Nieuwe diagnostiektest `uniec_measure` toegevoegd.
      - [x] **F3d-7 ✅ (12-07, `fe7cd41`)** — PV-tabel-16.2 hoek-wrap (`nta8800-pv/src/calc/mod.rs:164`) vervangen door NTA 8800 tabel-17.2 I_sol(β,γ,maand)-lookup (p.690-693) + koudebruggen-propagatie in tojuli/beng-keten (`SharedGeometry.thermal_bridges` → H_D, formule 8.1). Norm-analyse: `docs/2026-07-12-f3d4-norm-analyse-pv.md`.
      - [x] **F3d-8 ✅ (12-07)** — PV-saldering §5.5.2-5.5.4 maand-matching her-analyse — F3a-aanname "f_del=f_exp=1,45 dus splitsing valt weg" weerlegd door Uniec-cert, maar heranalyse toont: engine is norm-conform, Uniec-gap = normversie-verschil (geen code-fix). Identiteitsbewijs Max(0,a−b)−Max(0,b−a)=a−b: onder 2025+C1 valt maandmatching exact weg, PV-export salderert volledig tegen fP;exp;el=1,45. Certified Uniec crediteert ~64% (ouder-norm/AB-directgebruik-signatuur). Norm-analyse: `docs/2026-07-12-f3d8-norm-analyse-saldering.md`.
      - [x] **F3d-8b ✅ (13-07)** — bijlage-AB ZEB-indicator (EweP,ZEB;Tot) als losse additieve output geïmplementeerd (`crates/openaec-project-shared/src/beng/zeb.rs`; additief `BengResult.zeb_indicator` met `#[serde(default, skip_serializing_if)]`, wiring in `compute_beng`, transparantie-note). Maandmodel AB.9/AB.10 all-electric+PV: directgebruik AB.65 `Min[fdu×PV; 0,3·EEPus]` (tabel AB.1), factoren 1,35/1 (tabel AB.2); batterij/WKK niet gemodelleerd (termen=0). **Meting (`zeb_measure`, bridged): bijlage AB reproduceert certified NIET** — Gouda EweP;ZEB=20,82 vs cert 27,48 (−24%, zelfgebruik 26%), Aalten 31,77 vs 24,71 (+29%). Certified 27,48/24,71 is ouder-norm partieel-salderingsartefact, geen 2025+C1-grootheid (BENG2 óf ZEB). Golden blijft `#[ignore]` (anti-fudge); redenen dragen de gemeten gap. Norm-analyse §7 in `docs/2026-07-12-f3d8-norm-analyse-saldering.md`.
      - [x] **F3d-9 ✅ (12-07)** — q_v10;spec additief op ProjectV2 (shared + energy-VentilationInput), meting > forfait (§11.2.5, form. 11.86/11.85, eenheid per A_g OPMERKING 2 p.486) via effective_q_v10(); invoergrens-validatie InvalidQv10Spec; bron-note in BengResult.notes. Gemeten: Gouda qv10=0,98=forfait (drop-in bewezen), Aalten 0,40→Q_H;nd −0,4pp. Q_H;nd-kalibratiegap (−25..37%) zit bewezen in het demand-model, niet in de infiltratie → vervolg-werkpakket.
  - [x] **F4a ✅ (12-07)** — backend-exposure `compute_beng`: `POST /beng/calculate` in de compute-router (2MB body-limit + 30/min rate-limit, publiek conform overige compute-routes), `spawn_blocking` naar `compute_beng`; contract `{project: ProjectV2}` → `BengResult`; `MissingEnergyInput`/`EmptyProject` → 422, reken-fout → 400. Tauri-command `compute_beng` (invoke-arg `req`) geregistreerd. `ActiveNorm::Beng` additief met defensieve arms in beide `calculate_v2`-routers (verwijzen naar de dedicated route; `active_norm()` levert bewust nooit Beng — invoer leeft op `ProjectV2.energy`). 3 nieuwe route-tests + 4 routing-tests groen.
  - [x] **F4b ✅ (12-07)** — frontend-tab: `types/beng.ts` handmatig gespiegeld aan Rust-serde (surgisch, niet via generator — zie Fase-2/3-landmine hierboven), `bengClient.ts` web/Tauri-dispatch (POST /beng/calculate resp. invoke compute_beng, 422→melding), invoerpaneel per deelsysteem in `pages/Beng.tsx` (additief in projectStore `energy` + persist-migratie), resultaten: BENG 1/2/3-kaarten + limiet/pass-fail, TOjuli+methode, energielabel, service-breakdown, notes[] (aannames-transparantie). Nieuwe route `/beng` + Sidebar-entry (NL/EN i18n). Review-hardening: `updateEnergy` merget alleen gedefinieerde keys (undefined=niet aanraken, null=wissen) + dwtw-null-normalisatie, met eigen regressietest (`projectStore.energy.test.ts`). Dual-gereviewd (Ollama, 2 napunten gefixt), tsc schoon, vitest 385/385.
  - [ ] **F4c** — UX-verdieping (deels ✅ 12-07: bron-metadata).
    - [x] **Verklaarde-waarde-bronnen ✅ (12-07)** — `ValueSource{kind: forfait|kwaliteitsverklaring|gelijkwaardigheidsverklaring|meting|overig, reference}` additief per deelsysteem (heating/dhw/dwtw/ventilation/cooling/pv), puur metadata (bewezen geen invloed op de berekening); doorvoer naar `BengResult.notes` + gestructureerd `value_sources`-rapportveld; UI bron-select + referentieveld per kaart (alleen zichtbaar ≠ forfait), bronnen zichtbaar bij de resultaten; hardening reference getrimd + afgekapt op 200 (Rust `normalize_reference` + UI maxLength); NL/EN i18n. BCRG-databank-integratie bewust niet meegenomen (leverancierslicentie = later); handmatige route dekt ook niet-BCRG-gelijkwaardigheidscertificaten.
    - [x] **Uniec-velden-inventarisatie ✅ (12-07)** — Playwright-capture 20 pagina's golden-case 2522 Aalten → docs/2026-07-12-uniec-velden-inventarisatie.md (veldentabellen + mapping-analyse + BengGeometry-v1-spec); capture gearchiveerd in aalten-2522/uniec_fields_capture.json.
    - [ ] **F6 — BENG-geometrie-invoer gevel-georiënteerd (besluit user 12-07)** — additief beng_geometry-blok (bibliotheek → rekenzone → gevels → ramen, 1:1 Uniec) per spec §5 van de inventarisatie-doc; fase 1 data-laag + Aalten-fixture, fase 2 orchestrator-vertaling in compute_beng, fase 3 frontend-geveltab. Dicht gap #1-4 (Q_H;nd-kalibratie).
      - [x] **F6 fase 1 ✅ (12-07)** — beng_geometry data-laag: DTO's conform spec §5 (afwijkingen gedocumenteerd: oriëntatie in BengAdjacency, RcOrU-enum, AosForfaitair Option-oriëntatie), validate() (refs/plausibiliteit/raamopp≤gevel), additief op ProjectV2 (serde-regressie-gepind), Aalten-fixture 100% certified (alle 6 gevels opaak+ramen=bruto exact, her-capture v2). QC ship + 2 mediums verwerkt. Fase 2 = compute_beng-brug (Rc→U, per-oriëntatie aggregatie, P/A-methode) + deur/ggl-check.
      - [x] **F6 fase 2 ✅ (12-07)** — geometry_bridge: beng_geometry → gevalideerde demand-keten (Rc→U tabel C.2 p.778 via surface_resistances; bruto-opp-conventie; Raam zonder ggl → InvalidInput; bron-note in BengResult). Aalten-herkalibratie: BENG1 −26,0%→−0,8% · BENG2 −67,4%→−8,5% · BENG3 +8,4pp→−1,4pp — eerste GROENE Uniec-golden geactiveerd. Rest-delta label A++++/A+++ = PV-salderings-normversie (F3d-8). QC ship.
      - [x] **F6 fase 2b ✅ (12-07)** — certified Gouda-fixture (7 vlakken, kruipruimte P=48, 2 daken 30°) + bridged meting: BENG 1 −37,3%→−5,7% (binnen ±6%); BENG 2/3 buiten tol door PV-saldering-normversie (F3d-8, 8,4 kWp domineert) → gouda-golden blijft #[ignore] tot saldering geadresseerd. Bekende benaderingen gemarkeerd: belemmering V2-typen→minimal, buitenscreens Z niet gemodelleerd (F_c ontbreekt in capture).
      - [x] **F6-napunt ✅ (13-07, C1)** — (1) **P/A-grondmodel** §8.3/ISO 13370: `slab_on_ground_conductance` (B'_f=A/(0,5·P)→d_f;equi→U_fl 8.40/8.41→H_g) in nta8800-transmission; omtrek reist via nieuwe `Construction.ground_perimeter_m` (additief) door de brug; vervangt forfait h_g;an=10 zodra elke grondvloer een perimeter heeft (Aalten 10→13,16 W/K; Gouda=kruipruimte→b-factor, niet geraakt). (2) **Raam-U in demand-transmissie** (formule 8.1): `build_transmission_elements` splitst nu opaak `(A_bruto−ΣA_raam)·U_opaak` + elk raam `A_raam·U_window`. (3) **Gevel-id globaal uniek** in `BengGeometry::validate`. **Effect:** verwarming klopt nu met certified (Aalten primair 1544→2444 vs 2551; Gouda 2914→5131 vs 6506). **Compensatie opgeheven:** BENG1/2 overschieten nu door de out-of-scope koeling-F_sh=1,0-overschatting (Q_C;nd Aalten ~2847 vs cert 873) → `aalten_beng_geometry_within_certified_tolerance` #[ignore] met gemeten reden (anti-fudge, expected.json onaangeraakt); nieuwe green anchor `aalten_beng_geometry_heating_matches_certified`. Zie `docs/2026-07-13-c1-norm-analyse-transmissie.md`. **Rest-blokkade: F_sh-koeling (F3d).**
      - [x] **F6-napunt ✅ (13-07, C2)** — **koudebalans norm-conform** (§7.2.2). Diagnose weerlegt de C1-"F_sh=1,0"-hypothese: de capture toont `ZONW_GEEN`/`ZOMERNVENT_NAANW` (certified rekent óók zonder zonwering) → de gap zit in de koudebalans-formule. Twee correcties: (1) **warmteoverdracht voor koeling tegen de koel-setpoint** θ_int;set;C=24°C (§7.3.2 form. 7.15, tabel 7.13) i.p.v. de verwarmings-`Q_H;ht` op 20°C — tweede `calculate_transmission`/`calculate_ventilation` op 24°C in tojuli (`transmission_cooling`/`ventilation_cooling`, ventilatie-branch→closure); (2) **§7.2.2-poort** `(1/γ_C)>2,0 → Q_C;nd=0` (form. 7.6, `cooling_demand_gated`). `a_C;red` niet nodig (tabel 7.15: woonfunctie t_C;red=0). `calculate_demand_with_cooling_ht` (optioneel Q_C;ht-profiel; `calculate_demand`-terugval identiek op de §7.2.2-poort na — die geldt norm-correct óók zonder Q_C;ht-profiel, geen andere callers). **Effect (bridged):** Aalten BENG2 +36,7%→**−4,2%** ✓, BENG3 −3,8pp→**+1,0pp** ✓ (nieuwe green anchor `aalten_beng_geometry_beng2_matches_certified`); Gouda BENG1 +20,0%→**+1,0%** (koeling 3334→1969 kWh). Verwarming ongewijzigd (2444, anchor blijft groen). **Rest-gaps** (zie hieronder): Aalten BENG1 +11,2% = thermische massa; Gouda BENG2/3 = PV-normversie. `docs/2026-07-13-c2-norm-analyse-koelbalans.md`. cargo test --workspace groen.
      - [x] **C3a ✅ (13-07) thermische massa (bouwwijze→C_m)** — `beng/dynamics.rs::derive_thermal_mass`: Uniec-codes (`CONSTRM_FL_*`/`CONSTRM_W_*`, capture-confirmed FL_21=zwaar/FL_26=zeer zwaar/W_11=licht) → `FloorMassClass`/`WallMassClass` (tabel 7.11/7.12); woningbouw kolom 'geen/open plafond' (voetnoot b) → Aalten+Gouda D_m=180. Via `TojuliFullInputs.thermal_mass` (Option, additief) i.p.v. `light_woning`. Doc: `docs/2026-07-13-c3-norm-analyse-massa-interne-winst.md`.
      - [x] **C3b ✅ (13-07) interne warmtewinst woningbouw (form. 7.21)** — `beng/dynamics.rs::derive_internal_gains_woningbouw`: `Φ_int = 180·N_woon·N_P/A_g` [W/m², constant], N_P via 7.22–7.24 (**PDF-geverifieerd**: Aalten A_g=67 valt in band 7.23 → N_P=1,677, NIET 1,95; die 1,95 was de >100-formule). Aalten Φ_int=4,50, Gouda=3,53 W/m². Via `TojuliFullInputs.internal_gains` (Option, additief). N_woon=1 (grondgebonden).
      - [x] **C4 ✅ (13-07) demand-keten-analyse** — formule-audit §7.8 (7.46–7.57) bevestigt η_H;gn/a_H/τ/γ_H **norm-correct** (a_H;0=1,0, τ_H;0=15 PDF-geverifieerd) → géén utilization-fout. Oorzaak = twee **zonwinst-omissies** (formule 7.32) in `nta8800-demand::calc::solar_gains`: (1) invalshoek-correctie **F_w=0,90** (form. 7.40) ontbrak; (2) **Q_sky** hemelstraling (form. 7.39, §7.6.5: F_sky·R_se·U·A·h_lr·Δθ, h_lr=4,14 Δθ=11K) werd niet afgetrokken. Beide blazen zonwinst op → verwarming te laag + koeling te hoog (dubbel-signatuur). Fix: `g_gl=F_w·g_gl;n` + per-raam Q_sky. **Effect Aalten:** heating primair 2053→**2168 kWh (−19,5%→−15,0%)**, koudebehoefte 1329→1036 kWh (cert 873). Split beweegt correct richting certified; BENG-neutraal in totaal. Doc: `docs/2026-07-13-c4-demand-keten-analyse.md`. Additief: raakt TO-juli/isso51-callers (norm-correct, workspace groen).
      - [x] **C5a ✅ (13-07) opake zonwinst + Q_sky (form. 7.33, §7.6.3)** — `nta8800-demand::calc::solar_gains::{OpaqueElement, monthly_opaque_solar_gains}`: `α_sol·R_se·U·A·I_sol − Q_sky` per exterieur-opaak vlak, **α_sol=0,6** (§7.6.6.3, PDF-geverifieerd), **F_sh;obst=1** voor opaak (p. 185), F_sky uit helling (§7.6.6.4). Balans-onafhankelijk → opgeteld bij Q_H;gn én Q_C;gn. Additief via nieuwe `opaque_elements`-param op `calculate_demand_with_cooling_ht` (`calculate_demand` geeft `&[]`); `tojuli.rs::build_opaque_solar_elements` (A_opaak = bruto − openingen, geen dubbeltelling met ramen). **Effect Aalten:** BENG1 96,39→**97,58 (−7,0%→−5,9%)**, BENG2 14,73→15,38, heating 2168→**2172 kWh (−14,9%)** — klein (α·R_se·U-gedempt), fysisch juist (zomer-absorptie ↑ koeling, winter-Q_sky ↑ verwarming). Doc: `docs/2026-07-13-c5-norm-analyse-opaak-plafondkolom.md`.
      - [x] **C5b ✅ (13-07) plafondkolom voetnoot c (tabel 7.10) — GEEN auto-switch (anti-fudge)** — voetnoot c ("bovenzijde vloer zwaarder dan onderzijde vloer erboven → gesloten plafond, §7.7 OPMERKING 2) is een **twee-vloer-vergelijking** die de enkelvoudige `bouwwijze_vloer`-code niet draagt → **niet eenduidig afleidbaar**. Open-plafond-default (voetnoot b) blijft; gevoeligheid als `BengResult.notes`-regel bij zware/zeer-zware vloer. Gemeten gevoeligheid D_m110: Aalten BENG1 +1,0% / heating −11,2% — juist die betere fit maakt anti-fudge bindend (Uniec exposeert D_m niet, geen non-circulair bewijs). Doc idem.
      - [ ] **C5-RESTGAP [buiten C5-scope]** — na C5a blijft heating −14,9% (Aalten) / −26,6% (Gouda); alle Uniec-ankers `#[ignore]` met gemeten reden (BENG1 Aalten valt met C5a alléén binnen ±6%, aggregaat niet). Dominante hefboom = **plafondkolom-massa** (C5b-gevoeligheid, niet norm-afleidbaar zonder per-verdieping-vloerinvoer → V2-invoerveld) + voor Gouda de **PV-saldering-normversie** (F3d-8). De demand-keten zelf is nu norm-conform doorgemeten (C4 utilization-audit + C4/C5a zonwinst compleet).
      - [ ] **C1-napunt (QC 13-07)** — `BengBoundary::validate()` (`beng_geometry.rs:399-433`) cross-checkt `constructie_ref`-VlakType niet tegen `grenst_aan`: een Kelderwand kan een "Vloer...Grond"-adjacency + `omtrek_p_m` krijgen en zo het vloer-specifieke B'/d_equi-model op een verticale wand toepassen (kelderwand vereist eigen diepte-onder-maaiveld-correctie). Sinds C1 is `omtrek_p_m` functioneel → nieuwe blootstelling; niet getriggerd door Aalten/Gouda.
    - [ ] BCRG-datalicentie [USER-besluit] — databank-koppeling voor automatische bronvalidatie/lookup.
    - [ ] Rapport-PDF-doorvoer van `value_sources` — bronnen nu alleen zichtbaar in-app, nog niet in het gegenereerde PDF-rapport.
    - [ ] validatie-ranges op energy-invoervelden
    - [ ] per-raam zonwering-koppeling met de modeller (nu info-regel)
- [ ] **Utiliteitsbouw peak-koellast fixture** — folder + expected.json klaar (2026-05-28), wacht op peak-cooling engine

### Optioneel later

- [x] **F3d-5 fase 1 ✅ (12-07)** — ISSO 54 EDR-attesteringstestset (BRL 9501 NTA8800 v2.0, InstallQ CCvD 12-05-2022) geëxtraheerd als rode golden-laag: 6 EPW-fixtures (epw001/002c/004d/101p/203f/301a) onder `tests/verification/beng_edr_epw/`, invoer volledig normatief uit de PDF-tekst (spiegelbeeld van de RVO-set: dáár ontbreekt de invoer, hier de uitkomsten). Officiële afkeurtolerantie ±1,0%, provenance (pagina/figuur) per waarde, PDF zelf buiten de repo (licentie). Harnas: `crates/openaec-project-shared/tests/edr_golden.rs`, 1 passed / 7 ignored. Analyse: `docs/2026-07-12-f3d5-edr-testset-analyse.md`.
- [ ] **F3d-5 fase 2a** — `edr_to_projectv2`-builder + geometrie-golden activeren (Ag≈96 m²/Als≈247,2 m² op EPW001, niet Excel-geblokkeerd, ±1%).
- [ ] **[USER] EDR Bijlage 2-Excel (eindwaarden)** verwerven via InstallQ/ISSO 54-bron — blokkeert alle energie-eindwaarde-asserts in `edr_golden.rs` tot dan.
- [ ] Uniec voorbeeldproject — Uniec is cloud-only SaaS, geen lokale bestanden mogelijk zonder DGMR-samenwerking

## 🎯 v1.0 Release Criteria

**Vastgelegd 2026-05-26.** v1.0 wordt uitgegeven wanneer onderstaande punten allemaal afgevinkt zijn. v0.2.0 (huidige tag) markeerde ISSO 51 feature-complete; v1.0 markeert het volledige platform (ISSO 51 + 53 + TO-juli) als productie-klaar.

### Blokkades

- [ ] **Alle test-fixtures aanwezig**
  - [x] Spoor 4 fixture-bundeling completeren — Bedrijfsruimte4 en 1.10a gedecomposeerd naar 1-op-1 Vabi-mapping, beide `#[ignore]` weg (sessie 14, 2026-05-29)
  - [ ] ISSO 53 batch 2d norm-verificatie afronden (infrastructuur klaar, verificatie pending)
  - _TO-juli Vabi-cross-validatie fixtures verschoven naar v1.1 (geen Vabi TO-juli PDF beschikbaar, sessie 14)_

- [ ] **Alle tests groen**
  - [ ] `cargo test` workspace — alle crates passend (isso51-core, isso53-core, nta8800-cooling, vabi-importer, ifcx)
  - [ ] `cd frontend && npm run build` slaagt
  - [ ] `cd frontend && npm test` slaagt (indien aanwezig)
  - [ ] CI groen op de release-commit

- [ ] **ISSO 53 productie-klaar**
  - [x] Vabi end-to-end verificatie op minimaal 2 reëele projecten binnen norm-tolerantie — 5 fixtures binnen ≤6% tol: Bedrijfsruimte4 (+3.6%), DR Kantoor West (+3.5%), 1.10a (+0.1%), 2.10a (+0.3%), 3.10a (+5.0%) (sessie 14, 2026-05-29)
  - [ ] Alle ISSO 53-specifieke UI-flows getest (norm-switch, utiliteit-velden, rapport)
  - [x] Geen `TODO:` of `FIXME:` in `crates/isso53-core/` en isso53-gerelateerde frontend code (commit `40b905c`, 2026-05-28)

- [ ] **TO-juli productie-klaar**
  - [ ] UI-flow `/tojuli` + `/tojuli-full` getest door user
  - _Vabi-cross-validatie groen op referentie-project — verschoven naar v1.1 (geen Vabi TO-juli PDF beschikbaar, sessie 14)_
  - _PDF-rapport TO-juli verifieerbaar tegen Vabi-uitvoer — verschoven naar v1.1 (sessie 14)_

### v1.1 doelen (post-v1.0)

- [ ] TO-juli Vabi-cross-validatie fixture vullen wanneer Vabi BENG/TO-juli PDF beschikbaar is (folder `tests/verification/tojuli_vabi3.12.0.127_dr-engineering-woningbouw/`)
- [ ] TO-juli PDF-rapport cross-val tegen Vabi-uitvoer
- [ ] Utiliteitsbouw peak-koellast fixture invullen wanneer peak-cooling engine af is
- [x] 3 BENG-fixtures uit RVO voorbeeldconcepten — zie F0 hierboven (Vrijstaande L i.p.v. M); goldens rood tot `compute_beng` (F2). Plan: `docs/2026-07-11-beng-onderzoek-implementatieplan.md`.
- [ ] ISSO 54 testset (optioneel, BRL 9501 attestering)

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
- [ ] ISSO 53 voorbeeld 6.2 input-rebuild (modulenkantoor, PDF p.60-62, gedetailleerde methode — past bij engine) + tolerance_pct→tolerancePct keyfix in voorbeeld_62_expected.json
- [ ] ISSO 53 voorbeeld 6.1 vereist schilmethode-uitbreiding engine (shell.rs te grof: 0,5 ACH hardcoded, geen WTW-f_v) — pas daarna input-rebuild zinvol
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

## 🌐 Server-opslag
- [x] **Envelope-pariteit server-save (10-06)** — server-save/-load gebruikt dezelfde volledige envelope als file-save (geometrie + alle sidecars), backward-compat legacy kaal project_data, race-guard projectwissel, persistente save-statusindicator, body-limit 20 MB. Fixt: geometrie-verlies op server + per-pc divergentie. 180/180 + cargo 28 groen.
- [ ] **Onderlegger (underlay.dataUrl) niet in envelope [besluit]** — bewust uitgesloten (1-10+ MB base64); wordt ook bij file-open niet hersteld. Later: aparte upload/opslag overwegen.

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
- [x] In-app help-sectie — gebruik, formules, afwijkingen + live Vabi-verificatie
- [ ] Materialen: inline bewerken, lambda nat, zoekwoorden
- [x] U_w kozijn-calculator Fase 1: `uw_breakdown`-datamodel + `Spacer`-enum (`7727e79`)
- [x] U_w kozijn-calculator Fase 2: `uwCalculation.ts` + spacer-tabel + `/uw`-calculatorpagina
- [x] U_w kozijn-calculator Fase 3: opslaan op kozijn-element + opbouw in project-rapport + zelfstandig U_w-rapport
- [x] U_w kozijn-calculator: fabrikant-catalogus (profiel/glas) + Ψ_g-correctie naar EN-ISO 10077-1 Annex E-richtwaarde
- [x] U_w kozijn-calculator: afronding — setTimeout-cleanup, edit-param-feedback, catalogus-herkomst persistent in rapport
- [x] #21 rekenexpressies (=1,5*2,6) in numerieke tabelcellen
- [ ] Help verificatie-sectie uitbreiden met isso53/koellast-projecten + woonhuis-A zodra input/expected compleet

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

---

## 🌱 MPG-tab (indicatieve milieuprestatie) — planning gestart 2026-07-05
> Ontwerp: `docs/2026-07-05-mpg-tab-ontwerp.md` · Mockup: `mockups/pages/mpg.html`
- [x] Ontwerpdoc: fasemodel kengetal→preset→lagen, `mpg-core` per ADR-002 `calcs["mpg"]`, NMD-profiel-snapshots in projectbestand
- [x] UI-mockup (score-meter + bandbreedte, hotspots, modules A-D, variantenvergelijking MPG↔warmteverlies, koppelingsmatrix) — paletten dataviz-gevalideerd light+dark
- [ ] **[USER, loopt]** NMD Cat. 3 Viewer-API-key — aangevraagd 05-07 (bèta, gratis)
- [ ] **[USER, loopt]** Demo's MPGcalc 3 (DGMR) + GPR Materiaal (W/E) — aangevraagd 05-07; kijklijst in sessienotities (invoerflow, eenheden per NMD-kaart, forfaitaire posten, module D)
- [ ] `mpg-core` scaffold: model + schemas + weegfactoren set-A2 + kengetallen-starter
- [ ] Referentie-fixture (gepubliceerde MPG-berekening nabouwen)
- [ ] A2-grenswaarden per gebouwfunctie verifiëren (Bbl 1-7-2026) — als datatabel, niet hardcoded
- [ ] Cat. 3 seed-db (±40 profielen handmatig uit NMD Viewer) → later vervangen door API-sync
