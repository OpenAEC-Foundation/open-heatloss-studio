# Fable 5 — Volledige codebase-audit open-heatloss-studio

**Datum:** 2026-06-10 · **Model:** Fable 5 · **Methode:** finder-agents → dubbele verificatie (2 verifiers per bevinding) → synthese

---

## 1. Managementsamenvatting

Audit met meervoudige finder-agents (bugs, security, norm-correctheid, efficiency, quality/consistency) gevolgd door dubbele adversariële verificatie. Alleen bevindingen die door beide verifiers stand hielden tellen als **bevestigd**.

### Aantallen

| Categorie | Critical | Major | Minor | Nit | Totaal |
|-----------|:--------:|:-----:|:-----:|:---:|:------:|
| **Bevestigd** | 4 | 33 | 25 | — | **62** |
| **Twijfel** (1 verifier weerlegde) | 1 | 5 | 10 | — | **16** |
| **Nits** (ongeverifieerd) | — | — | — | 5 | **5** |
| **Weerlegd** (transparantie) | — | — | — | — | **9** |

> NB: enkele bevestigde items zijn dezelfde bug via verschillende finders (garage-sentinel critical 2×; Results rules-of-hooks 2×; IFC-upload path-traversal 2×). Reëel aantal unieke bevestigde issues ≈ 59.

### Bevestigd per dimensie

| Dimensie | Aantal |
|----------|:------:|
| bug | 24 |
| norm-correctheid | 9 |
| security | 7 |
| quality/consistency | 14 |
| efficiency | 8 |

### Top-5 (hoogste prioriteit)

1. **[CRITICAL] Garage-sentinel `f64::MIN` lekt naar resultaat (isso53)** — `crates/isso53-core/src/calc/room_load.rs:24` gebruikt rauwe `design_indoor_temperature` i.p.v. `resolve_theta_i`; elke garage zonder custom temp produceert -1.8e308 in θ_i/ventilatie/opwarmtoeslag → stille corruptie van de hele projectberekening.
2. **[CRITICAL] Tab-wissel lekt `norm`/`isso53Building`/`isso53Rooms`/`ventilation` tussen projecten** — `frontend/src/store/documentsStore.ts:55` snapshot deze velden niet → foute rekenkern-routing (isso53 op isso51) én sidecar-corruptie in opgeslagen bestanden.
3. **[CRITICAL] Tab-wissel laat `activeProjectId` staan → auto-save schrijft tab B over serverproject A** — `documentsStore.ts:178` reset `activeProjectId` niet → dataverlies op de server zonder conflict-check.
4. **[MAJOR/bug+norm-cluster Vabi-import] temperature_factor=1.0, Ground=None, VabiCompat zonder dwelling_class, UnconditionedSpace→Exterior, has_night_setback hardcoded** — `crates/isso51-core/src/import/vabi/mapper.rs` (r.442/454/211/207/624): elk geïmporteerd Vabi-project rekent fout (fantoomverlies binnenwanden, 0 W grondvloeren, harde import-fail, of fictieve opwarmtoeslag).
5. **[MAJOR/security-cluster API]** `X-Original-Tenant` onbeperkt vertrouwd (`auth.rs:493`, cross-tenant impersonatie) + cloud-routes negeren tenant-claim én laten path-traversal toe (`handlers/cloud.rs:66/118`) + IFC-upload path-traversal write (`ifc_import.rs:108`) + Tauri fs-scope `**` (`capabilities/default.json:32`).

---

## 2. Bevestigde bevindingen (per severity)

### 2.1 CRITICAL (4)

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `crates/isso53-core/src/calc/room_load.rs:24` | `calculate_room` berekent θ_i via rauwe `design_indoor_temperature` i.p.v. `resolve_theta_i`; Garage-sentinel `f64::MIN` lekt naar RoomResult.theta_i, ventilatie en opwarmtoeslag. (Door 2 finders gevonden: bugs:isso53-core én quality:rust.) | Vervang door `resolve_theta_i(room, climate.theta_e)`; regressietest met Garage-ruimte. |
| `frontend/src/store/documentsStore.ts:55` | `ProjectSnapshot` mist `norm`/`isso53Building`/`isso53Rooms`/`ventilation` → blijven van vorige tab staan; foute rekenkern-routing + sidecar-corruptie bij save. | Voeg velden toe aan capture/loadSnapshot (zelfde patroon als sharedExtra). |
| `frontend/src/store/documentsStore.ts:178` | `loadSnapshot` reset `serverUpdatedAt`/`hasConflict` maar niet `activeProjectId`; auto-save in tab B overschrijft serverproject A zonder conflict-check. | Neem `activeProjectId`+`serverUpdatedAt`+`hasConflict` in snapshot op, óf reset `activeProjectId` expliciet. |
| `crates/isso51-ifcx/src/to_ifcx.rs:208` *(zie ook §3)* | — verplaatst naar twijfel; zie sectie 3. | — |

### 2.2 MAJOR (33)

**Rekenkern / norm (isso51-core & isso53-core)**

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `crates/isso51-core/src/import/vabi/mapper.rs:442` | `temperature_factor: Some(1.0)` onvoorwaardelijk → binnen-/buurwanden tellen als volle buitenschil (bv. +729 W per binnenwand). | Zet `Some(1.0)` alleen voor `Exterior`; laat `None` voor Adjacent/Unheated. |
| `crates/isso51-core/src/import/vabi/mapper.rs:454` | Ground/CrawlSpace krijgen `ground_params: None` → grondvloeren stil 0 W. | Vul GroundParameters met afgeleide u_equiv, of fail/warn. |
| `crates/isso51-core/src/import/vabi/mapper.rs:211` | VabiCompat zonder `dwelling_class` → elke berekening van geïmporteerd project faalt (Isso51Error). | Leid DwellingClass af, of val terug op methode zonder dwelling_class. |
| `crates/isso51-core/src/import/vabi/mapper.rs:207` | `has_night_setback: true` hardcoded → P×A_g opwarmtoeslag ongeacht Vabi-instelling (+550 W woonkamer). | Lees nachtverlaging uit Vabi-DB, default false met WARNING. |
| `crates/isso51-core/src/import/vabi/mapper.rs:624` | `UnconditionedSpace` niet gemapt → `_ => Exterior` met volle ΔT. | Voeg `UnheatedSpace`-mapping + warning op onbekende types. |
| `crates/isso51-core/src/lib.rs:76` | Ū voor opwarmtoeslag exclusief ΔU_TB en grondvloer → P-waarde tot ~factor 2 fout nabij Ū=0,5. | Tel ΔU_TB bij U op en neem Ground-vloeren mee in Ū-weging. |
| `crates/isso53-core/src/calc/ground.rs:191` | U_equiv-quotiëntvorm heeft U_k met positieve exponent in noemer → betere isolatie geeft hóger grondverlies (omgekeerde monotonie). | PDF-verificatie 4.24 met 2e (U_k,U_equiv)-paar; auto-pad tot dan onbetrouwbaar markeren. |
| `crates/isso51-core/src/tables/infiltration.rs:137` | NEN 8088-1 Tabel 10 (f_inf) dubbel én tegenstrijdig: isso51 D=1.10/placeholder 1.0 vs isso53 D=1.00/A=1.10/B=C=1.05. | Verifieer tegen brondoc, consolideer naar één tabel-module. |

**Tooling / Python-extractor & vabi-importer**

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `tools/vabi-validation/extract_vp.py:654` | `if func == "custom"` onbereikbaar → Vabi-ontwerptemperaturen altijd weggegooid. | Emit `custom_temperature` onvoorwaardelijk bij theta_i uit DB. |
| `tools/vabi-validation/extract_vp.py:460` | `infiltration_method` ontbreekt in output → engine valt op legacy PerExteriorArea, negeert qv10. | Zet `measured_qv10`/`vabi_compat` analoog aan Rust-mapper. |

**Ventilatie-/BBL-norm (frontend)**

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `frontend/src/lib/ventilationBalance.ts:446` | Overdruk-verdeling (`_bereken_overdruk_verdeling`) niet geport → balans-criterium wijkt af van plugin. | Port toevoer-overschot-verdeling naar exhaust naar rato oppervlak. |
| `frontend/src/lib/ventilationUnits.ts:166` | Systeem C toetst MV-box alleen op afvoer-minima i.p.v. `max(toevoer,afvoer)` → tot ~40% onderschatting. | Gebruik overstroom-gecorrigeerde afvoer-eis voor C/D. |
| `frontend/src/types/ventilation.ts:282` | Utiliteitsfuncties op 0,9 dm³/(s·m²) + vlakke 4,0 dm³/s p.p. i.p.v. BBL per-persoon → onderwijs >2× onderschat. | Voeg per gebruiksfunctie BBL-per-persoon-debiet toe; tot dan indicatief markeren. |

**Frontend state / UI**

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `frontend/src/store/projectStore.ts:746` | Undo na `removeRoom` herstelt isso53/ventilatie-sidecars niet → stil verlies per-ruimte config. | Breid ProjectSnapshot uit met isso53Rooms+ventilation. |
| `frontend/src/store/projectStore.ts:677` | `setResult` zet onvoorwaardelijk `isDirty:false` → edits tijdens lopende calc als clean gemarkeerd (stale result). | Dirty-token/run-id; negeer stale responses. |
| `frontend/src/store/projectStore.ts:996` | Persist-merge forceert `isDirty:false` bij rehydrate; `activeProjectId` niet gepersisteerd → auto-save stopt stil na 'herlaad om in te loggen'. | Persisteer isDirty/activeProjectId/serverUpdatedAt of zet isDirty:true bij rehydrate. |
| `frontend/src/pages/Modeller.tsx:490` | `handleExportIfc` leest `modellerStore` (EXAMPLE_ROOMS/stale) i.p.v. getoonde project.rooms → exporteert verborgen data. | Exporteer derived rooms, of verberg knop in read-only viewer. |
| `frontend/src/pages/Results.tsx:139` | Rules-of-hooks-schending: early returns vóór 2× useMemo → crash bij norm-/result-transitie. (Door bugs:frontend-ui én efficiency:frontend.) | Verplaats useMemo's vóór early returns of splits isso51-pad af. |

**API / security**

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `crates/isso51-api/src/handlers/projects.rs:231` | Optimistic-locking gap: `expected_updated_at` alleen pre-SELECT, UPDATE mist `AND updated_at=expected` → lost update mogelijk. | Versie-check in UPDATE-WHERE; rows_affected==0 → 409. |
| `crates/isso51-api/src/handlers/ifc_import.rs:108` | Ongesanitiseerde multipart-filename → path-traversal write als service-user. (Door bugs:api-tauri én security:api.) | `Path::new(&filename).file_name()` of vaste tempnaam. |
| `crates/isso51-api/src/auth.rs:493` | `X-Original-Tenant` onbeperkt vertrouwd na Bearer → cross-tenant impersonatie. | Allowlist service-accounts; anders override negeren. |
| `crates/isso51-api/src/handlers/cloud.rs:118` | `{project}` path-param ongesanitiseerd doorgegeven → directory-traversal/info-disclosure. | Weiger `/`,`\`,`..`; sanitize vóór doorgeven. |
| `crates/isso51-api/src/handlers/cloud.rs:66` | Alle cloud-handlers `cloud_client(None)` → DEFAULT_TENANT; tenant-claim genegeerd, geen ownership-check. | Resolve client uit `claims.tenant`; valideer ownership. |
| `src-tauri/capabilities/default.json:32` | `fs:scope` allow `**` + read/write/mkdir → webview kan elk bestand lezen/schrijven. | Beperk scope tot `$HOME`/`$DOCUMENT`/`$APPDATA` of via dialog. |

**Quality / consistency / efficiency**

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `crates/isso51-ifcx/src/from_ifcx.rs:273` | IFCX-laag draagt R5/R6-velden niet → `/calculate/ifcx` rekent stil met defaults. | Breid namespace uit met model-velden + roundtrip-test. |
| `crates/isso51-ifcx/src/namespace.rs:96` | Serde-default `warmup_time` 0.0 vs model 2.0 → Φ_hu stil 0 ondanks setback. | Geef zelfde default als model of `Option<f64>` fallback 2.0. |
| `frontend/src/lib/uwCalculation.test.ts:29` | 3 testfiles met mini-harness → 'No test suite found'; `npm test` faalt structureel. | Converteer naar vitest `describe/it`. |
| `crates/vabi-importer/tests/v2_import.rs:7` | Tests `.expect()` op gitignored referentiebestanden → `cargo test --workspace` rood op schone checkout. | Skip-guard met `exists()` of `#[ignore]`. |
| `crates/isso53-core/tests/vabi_golden.rs:49` | Alle 3 golden-fixtures Φ_V=0 (luchtverwarming) → niet-nul ventilatieverliespad ongedekt. | Voeg fixture met Φ_V>0 toe; `unwrap()` i.p.v. `unwrap_or(0.0)`. |
| `frontend/src/store/projectStore.ts:49` | `structuredClone(project)` + volledige localStorage-serialisatie per toetsaanslag. | Commit tekstvelden op blur; structural sharing voor undo. |
| `frontend/src/components/modeller/modellerStore.ts:610` | `underlay.dataUrl` (1-10+ MB base64) gepersisteerd naar localStorage per mutatie. | Sluit dataUrl uit van partialize / verplaats naar IndexedDB. |
| `frontend/src/App.tsx:17` | *(verplaatst naar twijfel — al gedocumenteerd; zie §3).* | — |

### 2.3 MINOR (25)

| Bestand:regel | Beschrijving | Fix |
|---|---|---|
| `crates/isso51-core/src/calc/room_load.rs:299` | Negatieve Φ_T,iaBE (warmere buurwoning) wordt gekwadrateerd in Φ_extra → warmtewinst telt als verlies. | Clamp `phi_t_adj_building` op ≥0 vóór kwadratische som. |
| `crates/isso51-ifcx/src/from_ifcx.rs:176` | Constructievolgorde/-ids niet-deterministisch (HashMap-iteratie). | BTreeMap of expliciete volgorde-index. |
| `frontend/src/lib/serverProjects.ts:99` | Save-completion wist `isDirty` ook voor edits tijdens in-flight save → lost update server. | Mutatie-teller bij payload-bouw vergelijken. |
| `frontend/src/lib/serverProjects.ts:163` | Mislukte server-open laat stores half-gemuteerd (modeller nieuw, project oud). | Valideer result_data vóór modeller-mutaties; atomair toepassen. |
| `frontend/src/store/reportStore.ts:84` | `pdfBlobUrl` niet gewist bij project-wissel → Rapport toont PDF van vorig project. | `clear()` aanroepen in reset/setProject/loadServerProject/loadSnapshot. |
| `frontend/src/pages/Modeller.tsx:597` | Sneltoetsen negeren modifiers → Ctrl+C/S/P wisselen ongewenst tool. | `if (e.ctrlKey||e.metaKey||e.altKey) return;` vóór keyMap. |
| `frontend/src/components/settings/SettingsDialog.tsx:93` | `resetToDefaults`/`resetConfirm` ontbreken in nl/settings.json → rauwe key zichtbaar (NL=default+fallback). | Voeg beide keys toe aan nl/settings.json. |
| `frontend/src/components/modeller/FloorCanvas.tsx:1567` | Vrije ventielen (`positionMm`) op stale wereldcoördinaten → schuiven niet mee met noord-rotatie/reflow. | Sla relatief op (fractie/barycentrisch) of transformeer mee. |
| `crates/isso51-api/src/handlers/cloud.rs:195` | Lege gesaniteerde bestandsnaam → `Internal` (500) i.p.v. 400. | 400-mappende foutsoort (BadRequest-variant ontbreekt). |
| `crates/isso51-api/src/handlers/report.rs:74` | Upstream Reports-API foutdetail onverkort naar client → info-disclosure. | Log server-side, generieke client-melding. |
| `frontend/src/types/ventilation.ts:319` | Onbekende functie → `verblijfsruimte` (toevoer) waar plugin DEFAULT_NORM (afvoer); geporte DEFAULT_BBL_REQUIREMENT dead code. | Documenteer afwijking + markeer ongeclassificeerd in UI. |
| `crates/isso51-core/src/import/vabi/mapper.rs:422` | U-waarde-fouten stil naar defaults 0,5/2,5 W/m²K zonder warning. | Propageer fout of markeer element + log. |
| `frontend/src/components/layout/AppShell.tsx:78` | 10 call-sites pakken hele store zonder selector → re-render op elke tick. | Granulaire selectors / `useShallow`. |
| `frontend/src/components/modeller/FloorCanvas3D.tsx:551` | THREE-scene volledig herbouwd bij elke selectie-klik/edit. | Splits selectie-highlight van scene-opbouw. |
| `frontend/src/components/modeller/FloorCanvas.tsx:387` | Alle Konva-nodes re-render per mousemove tijdens pannen/tekenen. | Pan via Konva-node-ref; cursor-preview isoleren. |
| `crates/isso53-core/src/calc/ventilation.rs:79` | Dode pub-wrappers `calculate_h_v`/`calculate_phi_vent` (0 callers). | Verwijderen of room_load erlangs routeren. |
| `crates/isso53-core/src/calc/infiltration.rs:183` | Magic numbers 3.14/0.67 terwijl isso51 benoemde constanten heeft. | Benoemde constanten of gedeelde vabi-infiltration-helper. |
| `frontend/src/components/charts/deltaT.test.ts:248` | 3 legacy-tests via nooit-aangeroepen `runAllTests()`. | Wrap in `it()` binnen describe; verwijder runAllTests. |
| `frontend/src/components/ribbon/RapportTab.tsx:56` | 3× `result as unknown as Isso53ProjectResult` + stale comment (calculate_v2-bridge bestaat al). | Type-guard + centrale norm-routing-helper. |
| `frontend/src/store/documentsStore.ts:122` | Snapshot-shapes via `as unknown as` mirror-types → geen type-check, union al versmald. | Exporteer snapshot-shapes uit bron-stores. |
| `frontend/src/components/DocumentBar.tsx:10` | Dode componenten DocumentBar (+CSS, mock-tabs) en RibbonButtonStack (0 imports). | Verwijderen. |
| `frontend/src/lib/reportBuilder.ts:46` | fmtW/fmt2 3× + formatWatts/describeArc 2× naast centraal formatNumber.ts. | Verhuis naar lib/formatNumber.ts + svg-util. |
| `crates/isso51-core/src/model/climate.rs:48` | Dood schema-veld `theta_ground`: docstring claimt frontend-effect dat niet bestaat; ontbreekt in TS-types. | Aansluiten of verwijderen/als ongebruikt documenteren. |
| `crates/nta8800-cooling/tests/bijlage_aa_test.rs:425` | 4/11 cross-validatiewaarden hand-voorspeld uit eigen formule i.p.v. xlsm. | Draai xlsm opnieuw, lees werkelijke celwaarden. |
| `tests/verification/.../bedrijfsruimte4/expected.json:4` | `tolerance_pct`-velden wijken af van hardcoded toleranties → dode/misleidende waarden. | Synchroniseer of laat tests waarde uit json lezen. |

---

## 3. Twijfelgevallen (1 van 2 verifiers weerlegde)

| Bestand:regel | Sev | Beschrijving | Verifier A (bevestigt) | Verifier B (weerlegt) |
|---|:---:|---|---|---|
| `crates/isso51-core/src/import/vabi/mapper.rs:738` | major | Vabi-temp in `internal_air_temperature` i.p.v. `custom_temperature` → kamers op 20°C default. | Code klopt: r.738 schrijft internal_air_temperature, custom=None, LivingRoom hardcoded. | Technisch klopt maar room_load leest internal_air_temperature wél onder bepaalde voorwaarden — impact niet de geclaimde 20°C-default. |
| `crates/isso53-core/src/calc/infiltration.rs:129` | minor | `building_height.unwrap_or(3.0)` → laagste q_is-klasse, tot ~58% te lage infiltratie. | Is in productie zelfs het DEFAULT-pad (frontend stuurt buildingHeight nooit mee). | Al expliciet gedocumenteerd in audit-reports/05 — geen nieuw issue. |
| `crates/isso51-ifcx/src/to_ifcx.rs:208` | critical | IFCX-roundtrip verliest constructies met gelijke omschrijving (HashMap-key=description). | Code klopt: children HashMap, key=description → duplicaten overschrijven. | Analyse klopt maar geen realistische trigger in huidige flow → niet critical. |
| `crates/vabi-importer/src/mapper.rs:357` | major | Gefabriceerde isso53-sidecar (setback aan, P=10) + niet-deserialiseerbare legacy-blob. | Alle deelclaims kloppen in code. | Geen realistische trigger → niet major. |
| `mcp-server/src/index.ts:173` | major | MCP `calculate_file` negeert `norm` → isso53 stil als isso51 doorgerekend. | index.ts:173 leest norm/isso53 nooit. | Failure mode onjuist: isso53-keys in heating_system zorgen niet voor stil-fout-resultaat. |
| `mcp-server/src/index.ts:104` | minor | Tempbestand getrunceerd i.p.v. verwijderd + botsbare `Date.now()`-naam. | Code klopt op beide punten. | 0-byte files in gitignored target/ → geen reële impact. |
| `frontend/src/components/modeller/modellerStore.ts:291` | minor | `removeRoom` ruimt constructie-assignments/boundary-types niet op; ID-hergebruik erft ze. | Code klopt: assignment-maps blijven, generateNextRoomId hergebruikt. | Geen realistische trigger: canvas is read-only viewer sinds refactor. |
| `crates/isso51-api/src/handlers/projects.rs:212` | minor | No-op update → 500 i.p.v. 400. | Code klopt, docstring belooft 400. | Geen realistisch client-pad triggert dit. |
| `crates/isso51-api/src/main.rs:92` | minor | Reken-/import-endpoints zonder auth-extractor → DoS-oppervlak. | Sterker dan geclaimd: CPU-zware endpoints zonder AuthClaims. | AuthClaims vertrouwt zelf plaintext headers → impact-claim hol in geschetst scenario. |
| `src-tauri/tauri.conf.json:28` | major | CSP volledig uit (`csp:null`) → geen XSS-verdediging. | csp:null aanwezig sinds initial commit, niet bewust besloten. | Alle innerHTML-sinks renderen statische app-data → geen realistische trigger, niet major. |
| `crates/isso53-core/src/calc/transmission.rs:179` | minor | Adjacent-room lookup O(rooms²×elementen) lineaire find. | Patroon bestaat exact in beide kernen. | Alleen per gebruikersactie → geen merkbare impact. |
| `crates/isso53-core/src/calc/room_load.rs:33` | minor | Infiltratieketen 2× per ruimte, room_rc_high 3×. | Klopt feitelijk, bereikbaar in productiepad. | Geen reële impact. |
| `crates/openaec-project-shared/src/view.rs:37` | minor | Volledige legacy-blob geclonet per /calculate_v2-request. | Klopt, beide mappers in hot path. | Constante-factor op pad dat al volledig parse+IO doet → verwaarloosbaar. |
| `crates/isso51-api/src/handlers/projects.rs:321` | minor | calculate_and_save: dubbele JSON-parse + re-serialisatie. | Klopt: 2× parse + 1× stringify. | 'Megabytes'-aanname onjuist; grootste echte projecten klein. |
| `frontend/src/App.tsx:17` | major | Geen route-level code-splitting (three/web-ifc/pdfjs/konva in hoofdbundel). | Empirisch bevestigd: 0 lazy/Suspense, geen manualChunks. | Al bekend/gedocumenteerd in frontend/TODO.md:59. |
| `crates/isso51-core/tests/integration_test.rs:313` | major | Vergelijkingslus checkt alleen actual.rooms → weggevallen expected-room passeert. | Klopt: geen reverse-match assert. | Tweede vangnet bestaat (lib.rs::test_dr_engineering_woningbouw) → niet major. |

---

## 4. Nits (ongeverifieerd, compact)

| Bestand:regel | Beschrijving |
|---|---|
| `crates/isso51-ifcx/src/document.rs:142` | `get_attr` clonet Value per read; second pass deserialiseert elke constructie dubbel → `T::deserialize(v)`. |
| `crates/isso53-core/src/calc/infiltration.rs:129` | Known-pad `unwrap_or(3.0)` omzeilt de benoemde `FALLBACK_BUILDING_HEIGHT_M` in hetzelfde bestand. |
| `crates/isso53-core/src/calc/ground.rs:483` | Enige compiler-warning: ongebruikte `climate` in test_auto_f_ig_formules. |
| `frontend/src/pages/Results.tsx:267` | Inline `* 3.6` dm³/s→m³/h 4× terwijl `dm3sToM3h()` bestaat. |
| `frontend/src/types/result.ts:30` | TS-types lopen achter op result.schema.json: `q_v_minimum` (required!) + 6 summary-velden ontbreken. |

---

## 5. Weerlegde bevindingen (transparantie, 1 regel elk)

- IFCX schema-drift R5/R6 roundtrip reset config — major-scenario niet reproduceerbaar.
- Niet-atomische upload + manifest-update verweesde berekening — bewust gedocumenteerd besluit.
- Volledig vertrouwen op X-Authentik-headers zonder shared secret — trust-model bewust, achter Caddy.
- `shell:allow-execute` aan webview — frontend gebruikt alleen `open()`, kern-claim onjuist.
- `shell:allow-open` ongescopet — plugin past default-scope toe; claim technisch onjuist.
- `bblRequirementFor` mist substring-match plugin — als defect weerlegd.
- Vabi-import adjacent_room zonder id → Φ_T,ia 0 W — dood pad, geen adjacent_room-vlakken in praktijk.
- f_v-formule gedupliceerd isso53-core — divergentierisico bij nadere beschouwing verwaarloosbaar.
- `vabi_3floors_total_matches` herberekent totaal — aggregatieregressie onbereikbaar (ongekoppelde som).

---

## 6. Niet-onderzochte gebieden (completeness-critic)

| # | Gebied | Risico |
|---|---|---|
| 1 | **CI/CD-workflows** (deploy/live/build-installer/build-appimage) | Secret-handling, ongetekende installers, Watchtower-auto-pull GHCR → onge-audit supply-chain-pad naar productie. |
| 2 | **Dependency-audit** | Geen cargo audit/deny/npm audit in CI; 3 rusqlite-versies (0.31 vs 0.33) + sqlx 0.8 → CVE's structureel onopgemerkt. |
| 3 | **mcp-server/, pyrevit/, tools/, scripts/** | Buiten finder-scope; pyRevit voedt 3D-geometrie-pipeline, MCP = extra attack surface + rekenlogica-duplicatie. |
| 4 | **Licentie-compliance** | Bundled SQLite, gevendorde openaec-reports-lib, ISSO/KNMI-normdata in fixtures → distributie-/auteursrechtrisico. |
| 5 | **Error-telemetrie** | Alleen tracing→stdout, geen Sentry/structured reporting → 'stille corruptie'-bugs onzichtbaar in productie. |
| 6 | **SQLite onder concurrency** | WAL/busy_timeout/sqlx-poolgrootte ongetest → write-contention → 500's/extra lost writes bovenop optimistic-locking-gap. |
| 7 | **Backup/restore api-SQLite** | Geen backup-stap in deploy.sh/compose; één migratie zonder rollbackstrategie → corrupt volume = verlies alle cloud-projecten. |
| 8 | **Accessibility** | Canvas-modeller met sneltoetsen, geen ARIA/keyboard-alternatief; modifier-key-bug toont fragiele input-laag. |

---

## 7. Aanbevolen vervolgvolgorde (delegeerbare rondes)

| Ronde | Focus | Items | Agent |
|:--:|---|---|---|
| **R1 — Stille corruptie (eerst)** | Resultaatcorruptie & dataverlies | Garage-sentinel (room_load.rs:24), tab-wissel norm/sidecars (documentsStore.ts:55), tab-wissel activeProjectId (:178), setResult isDirty (projectStore.ts:677), persist-merge isDirty (:996), undo removeRoom sidecars (:746) | rust-developer + frontend-developer (parallel) |
| **R2 — Security** | API + Tauri | X-Original-Tenant (auth.rs:493), cloud tenant-claim (cloud.rs:66), cloud path-param (:118), IFC-upload traversal (ifc_import.rs:108), optimistic-locking UPDATE (projects.rs:231), fs-scope `**` (capabilities/default.json) | rust-developer + infra-engineer |
| **R3 — Vabi-import norm-correctheid** | Foute rekenresultaten geïmporteerde projecten | mapper.rs temperature_factor(442)/ground_params(454)/dwelling_class(211)/night_setback(207)/UnconditionedSpace(624)/U-defaults(422); extract_vp.py custom_temp(654)+infiltration_method(460) | rust-developer + python-developer |
| **R4 — Rekenkern norm-validatie** | ISSO-conformiteit | Ū+ΔU_TB (lib.rs:76), U_equiv-monotonie (ground.rs:191 — PDF-verificatie), Φ_T,iaBE clamp (room_load.rs:299), NEN8088 Tabel 10 consolidatie | rust-developer (na normbron-verificatie PM) |
| **R5 — Ventilatie/BBL** | Plugin-pariteit | overdruk-verdeling (ventilationBalance.ts:446), systeem C max(toevoer,afvoer) (ventilationUnits.ts:166), BBL per-persoon utiliteit (ventilation.ts:282) | frontend-developer |
| **R6 — Test-infra** | Build groen | mini-harness→vitest (uwCalculation.test.ts), vabi-importer skip-guard (v2_import.rs), golden Φ_V>0-fixture, IFCX R5/R6 + warmup_time serde | rust-developer + frontend-developer |
| **R7 — Efficiency + UI-polish** | Performance/UX | structuredClone/persist (projectStore.ts:49), underlay.dataUrl (modellerStore.ts:610), Results rules-of-hooks (Results.tsx:139), modifier-key-bug (Modeller.tsx:597), code-splitting (App.tsx) | frontend-developer |
| **R8 — Quality/cleanup** | Tech-debt | dode componenten, formatter-dup, dubbel-cast, theta_ground, nits | frontend-developer + rust-developer |
| **R9 — Niet-onderzocht** | Aparte audit-opdrachten | CI/CD supply-chain, dependency-CVE-scan, backup/restore, telemetrie, accessibility, licentie | infra-engineer + PM-scoping |

> Elke ronde via git-release voor commit; qc-reviewer vóór commit op R1/R2 (gevoelig). R4 vereist PM-validatie tegen ISSO-brondocumenten vóór code-wijziging.
