# BENG/NTA 8800 — onderzoeksrapport (Fase A+B) + implementatieplan (Fase C)

**Datum:** 2026-07-11 · **Status:** ter goedkeuring, nog geen code · **Auteur:** PM/architectuur
**Methode:** 5 parallelle read-only audits (basis-crates, service-crates, wiring/goldens, RVO-bronnen, Uniec-crosscheck), alle claims geverifieerd tegen code/PDF met `bestand:regel`/paginanummer.
**Vervangt gedeeltelijk:** `2026-07-11-beng-integratie-model-mapping.md` (de "twee modellen"-aanname en de "scaffold nta8800-core"-fasering daarin zijn achterhaald — zie §A3).

---

## Fase A — Wat staat er nu

### A1. Samenvattend oordeel

De 14 `nta8800-*`-crates (~30k LOC) zijn een **schone, goed-geteste bibliotheeklaag**: ~770 unit-tests, 0 verborgen stubs (alle vereenvoudigingen expliciet als "V1" gedocumenteerd), sterke norm-referentie-discipline in doc-comments (formule-nummers, tabel-refs, deels PDF-paginanummers). **Wat ontbreekt is geen rekenwerk maar de integratielaag**: niemand vult `EpInputs`, `calculate_ep_score` wordt nergens aangeroepen, en 7 van de 14 crates zijn "weeskinderen" zonder consumer.

### A2. Per crate

| Crate | Normdekking | Norm-refs | #tests | Grootste gat |
|---|---|---|---|---|
| nta8800-model | H.6 zonering + schil-datamodel | `references.rs` (15 const.) | 51 | mist H.7/H.11/H.12-formule-IDs |
| nta8800-tables | H.17 klimaat + bijlagen E/F/G/H/I/L, §7.7 C_m | deels | 99 | **6 bijlagen = "V1 stub"** (`lib.rs:17-21`): representatieve waarden, niet de norm-tabel |
| nta8800-transmission | H.8 maandmethode (8.1, 8.52, §8.3 vereenv., 8.60/8.61) | ja | 46+4 | grond user-supplied (geen ISO 13370); geen ΔU_for |
| nta8800-ventilation | H.11 (11.106/11.108/11.142, systeem A-E) + **massabalans-drukmodel §11.2.1.6** | ja, sterkst | 81+ | geen bypass/zomerspui; heuristiek- én drukmodel-pad naast elkaar |
| nta8800-geometry | H.6 + bijlage K helpers | ja | 52 | niet gekoppeld aan reken-entries |
| nta8800-demand | H.7 maandbalans (7.4, 7.10, 7.17, 7.33, 7.35) | ja | 58 | **F_sh=1,0** (geen schaduw); γ_C=γ_H (V1); H_ve als losse f64 |
| nta8800-humidity | H.12 | **zwakst** | 36+ | RH hardcoded (85/75/70%), 10 ACH-heuristiek — grotendeels niet-normatief |
| nta8800-heating | H.9 forfaitair, 4 opwekkers (HR/WP/e/SV) | ja, sterk | 62 | ΔT-methodiek, bijlage M/N/O/Q, hybride = V2; geen hulpenergie |
| nta8800-cooling | H.10 + **Bijlage AA volledig** (AA.1-AA.13, tabel AA.3 560 waarden) | ja, zeer sterk | 89 (4 ignore) | peak-koellast-engine ontbreekt (`vabi_koellast_golden` `unimplemented!`) |
| nta8800-dhw | H.13 forfaitair woning+utiliteit, DWTW vereenv. | ja, zeer sterk | 70 | bijlage T/U/W, circulatie, zonneboiler = V2 |
| nta8800-lighting | H.14 alleen utiliteit (tabel 14.3/14.4) | ja | 28 | woonfunctie=0 (correct voor nEP); bijlage Y = user-scalar |
| nta8800-pv | H.16 (16.101-103) | ja | 41 | temp-correctie/schaduw/bijlage V = V2 |
| nta8800-ep | H.5 + bijlage Z/AB → EP-totaal, energielabel, CO₂ | ja | 21 | neemt `EpInputs`, geen gebouwmodel; **geen BENG-toets** |
| nta8800-automation | H.15/EN 15232 BACS-factoren | ja | 34 | levert factoren die **nergens toegepast** worden |

### A3. Model & wiring — belangrijke correcties op eerdere aannames

1. **Er is al één canoniek gebouwmodel.** `ProjectV2` (`openaec-project-shared/src/project.rs:20`) met drie view-mappers: `to_isso51_project`, `to_isso53_project` én `nta8800_view::geometry_to_nta8800` (`:80`). Die laatste **bestaat en draait in productie** (TO-juli-keten, `tojuli.rs:119`). Het model-mapping-doc ging uit van twee gescheiden modellen — achterhaald.
2. **De "twijfelvelden" bestaan al in nta8800-model:** `Window.orientation` (`geometry/window.rs:67`), `Window.g_value` met 0..=1-validatie (`:76`), `Rekenzone.volume` (`zoning/rekenzone.rs:32`). Zonwinst (formule 7.33) gebruikt ze correct. Wél onderbenut: volume voedt H_ve niet (τ-keten niet gesloten); F_sh hard 1,0.
3. **Wiring:** live zijn `/cooling/simplified` (Bijlage AA quick) en `/tojuli/calculate` → `compute_tojuli_full` (demand+transmission+ventilation+cooling op nta8800-model). **Niet gewired:** ep, heating, dhw, pv, lighting, humidity, automation — door géén enkele crate geconsumeerd (reverse-dep-scan alle Cargo.toml).
4. **Frontend:** geen BENG-route/pagina; enige jaarenergie is een graaddagen-schatting die zichzelf expliciet "niet norm-conform" noemt (`lib/annualEnergy.ts:26,62`).

### A4. Wat ontbreekt concreet voor een end-to-end BENG-run

| # | Gat | Bewijs |
|---|---|---|
| 1 | **EP-orchestrator**: `ProjectV2 → services → EpInputs → calculate_ep_score` bestaat niet; `calculate_ep_score` heeft 0 callers | grep hele repo: alleen ep-crate + docs |
| 2 | **EpInputs-contract onverzoend**: 4-5 onderling incompatibele `EnergyCarrier`-enums; services leveren `(carrier, scalar)`, ep wil `HashMap<Carrier, MJ>`; DistrictCold heeft geen ep-tegenhanger; alleen pv past 1-op-1 | `nta8800-ep/src/model/mod.rs:26-56` |
| 3 | **BENG 1 ontbreekt volledig** — nergens wordt (Q_H;nd + Q_C;nd)/Ag geaggregeerd | EpInputs kent alleen eindenergie |
| 4 | **BENG 2 half**: `ep_total_mj_per_m2` in MJ/m² (kWh-conversie ÷3,6 ontbreekt), geen eis-toets; wel energielabel | `ep_score.rs:92-102` |
| 5 | **BENG 3 proxy**: `ep_renewable_share` zonder net-metering/temporele effecten (doc zegt zelf "vereenvoudigd") | `ep_score.rs:137-164` |
| 6 | **TO-juli-indicator ontbreekt**: Bijlage AA levert q_C (W/m²) + B_C;req (kW), geen TOjuli-ratio (≤1,20) of GTO-uren (≤450 h); drempels alleen in doc-comment | `bijlage_aa.rs:74, 759-813` |
| 7 | **Manifest-resolver ontbreekt**: `Rekenzone`-id-lijsten worden door geen calc gelezen (`let _ = zone;`); callers leveren opgeloste objecten | `transmission/calc/mod.rs:144` e.a. |
| 8 | Geen `ActiveNorm::Beng`, geen route, geen UI | `calcs.rs:103-113`, `App.tsx:42-60` |
| 9 | Peak-koellast-engine (EN 12831/NEN 5060) ontbreekt — los van BENG, wel v1.0-blokkade | `vabi_koellast_golden.rs:50,64` |

---

## Fase B — Wat is bewezen conform, wat niet

### B1. Eerlijk validatiebeeld

| Onderdeel | Status | Referentie | Kwantificatie |
|---|---|---|---|
| Bijlage AA koelbehoefte | ✅ **bewezen** | RVO-rekentool xlsm 2025.04, actieve golden | binnen 0,07% (max 0,26 W op 377 W); toleranties 1-5 W |
| Transmissie H.8 | 🟡 unit-bewezen | analytische handcalc (`small_house.rs`, ε=1e-9) | geen externe referentie |
| Demand/ventilation/heating/dhw/lighting/pv/ep/automation | 🔴 **onbewezen** | alleen unit-tests tegen eigen normlezing (self-referential) | 0 externe goldens; grep `nta8800-ep` in tests/ = 0 hits |
| BENG 1/2/3 + TO-juli end-to-end | 🔴 **onbewezen én onmogelijk te bewijzen** | keten bestaat niet | n.v.t. |
| ISSO 51/53-kant (context) | ✅ apart bewezen | ISSO-publicaties + Vabi, actieve goldens 2-6% | los van BENG |

### B2. Beschikbare gezaghebbende referenties

| Bron | Dekt | Bruikbaarheid | Kanttekening |
|---|---|---|---|
| **RVO BENG-voorbeeldconcepten 2021** (DGMR, 21 p., in `tests/references/`) | BENG 1/2/3 + TOjuli + Wp PV, 15 concepten × woningtype (p. 13-14) | **eindwaarde-golden met tolerantieband** | ⚠️ berekend met NTA 8800:**2020** (tool v1.49); per-gevel geometrie zit in externe "Bijlage 4"-Excel die **niet in de map staat** (tabel 1 is gerasterde afbeelding); invoer-reconstructie dus niet deterministisch |
| **RVO Rekentool Bijlage AA xlsm** | koelbehoefte/oververhitting per ruimte | fixture-generator, onbeperkt | dekt NIET BENG 1/2/3 — ander domein dan de PDF |
| **Certified Uniec 3.3.x** (Johns 3 `.oes.json` + `meta.uniecReference`) | BENG 1/2/3 + limieten + label; woningen ook sub-totalen (verwarming/tapwater/ventilatoren/PV) | **replay-golden**: `project{}`-blok is deterministisch volledig voor de engine-invoer | geometrie is benadering van certificaten (residueel 1-5%); WTW-pad ongedekt (alle 3 η_wtw=0); "regressie-golden, geen certificerings-referentie" |
| **NTA 8800:2025+C1:2026** (32 MB, op `Z:\...\98_normen\`) + ISSO 75.1 / 82.1 (6e druk) | de norm zelf + bepalingsprotocollen | formule-verificatie | versie-mismatch met RVO-referenties (2020) — zie beslispunt 1 |
| John's TS-engine als "tweede normlezing" | ~52 clausule-refs | betrouwbaar voor: envelope-maandbalans, PV (§16 + tabel 17.1/17.2), koudebruggen (bijlage H), tapwater §13, hulpenergie §12 | **wantrouwen voor:** verlichting §14 (G1), TOjuli (heuristiek, G4), koeling (G2), limietbepaling (G5 = platte forfait-tabel i.p.v. geometrie-afhankelijke eis) |

### B3. Referentiewaarden voor de geplande goldens (RVO-PDF, p. 7 + 13-14)

**Naamgeving-correctie:** de PDF kent geen BENG-referentie "Vrijstaande M". De vrijstaande BENG-referentie is **Vrijstaande L** (massief, Ag 181); "Vrijstaande M Herten" is een markt-case (HSB gemengd licht, +5 kWh/m² eisophoging, bodem-WP-concepten voldoen er niet). **Advies: Vrijstaande L als derde golden.**

| Type | Ag | Als/Ag | BENG 1-eis | Voorbeeld-goldenrij (WP-bodem, C4c/BB+) |
|---|---|---|---|---|
| Tussenwoning M (G13) | 87 | 2,03 | ≤ 70,9 | B1 54,8 · B2 29,3 · B3 59% · TOjuli 0 |
| Hoekwoning M (G11) | 133 | 1,87 | ≤ 66,2 | B1 59,2 · B2 28,2 · B3 62% · TOjuli 0 |
| Vrijstaande L (G12) | 181 | 2,14 | ≤ 74,1 | (rijen p. 14) |

Per type staan 15 concepten (5 opwekkers × 3 vent/iso-pakketten) in Bijlage 1 — volledige tabellen geëxtraheerd in het B1-agentrapport (scratchpad `rvo_full.txt`). Uniec-referenties: Gouda 2467 (B1 95,86 / B2 27,48 / B3 83,7% / A+++), Aalten 2522 (103,69 / 24,71 / 85,0%), Kijkduin 2786 utiliteit (115,90 / 57,98 / 62,6%).

---

## Fase C — Implementatieplan (validatie-eerst)

### C0. Kernadvies: voortbouwen, niet herstructureren

| Optie | Voor | Tegen | Advies |
|---|---|---|---|
| **Voortbouwen op nta8800-crates** | fundament is schoon, getest (~770 tests), norm-gerefereerd, eerlijk over V1-stubs; TO-juli-keten bewijst dat het integratiepatroon werkt; gat is een afgebakende integratielaag | V1-vereenvoudigingen moeten stuk voor stuk expliciet gemaakt/gedicht | ✅ **dit** |
| Alles opnieuw ("nta8800-core" greenfield) | psychologisch schone lei | gooit 30k geteste LOC weg en lost het échte probleem (validatie tegen referenties) niet op; zelfde goldens nodig; maanden extra | ❌ |
| TS-engine porten als basis | 1-5% bewezen op woningen | dezelfde 5 gaten mee-porten; Rust-crates zijn normdieper (Bijlage AA, drukmodel §11.2.1.6) | ❌ (blijft cross-check) |

De betrouwbaarheid komt niet uit herbouw maar uit de **vangrail**: officiële eindwaarden als rode goldens vóór er één engine-regel wijzigt — exact het isso53 §6.1/§6.2-precedent. Anti-fudge blijft absoluut: haalt de engine een referentie niet → documenteren en analyseren, nooit expected aanpassen.

### C1. Fasering

| Fase | Wat | Acceptatie | Afhankelijk van |
|---|---|---|---|
| **F0 — Goldens eerst (rood)** | (a) 3 RVO-woningcases (Tussenwoning M / Hoekwoning M / Vrijstaande L; per case 2-3 concepten: WP-bodem C4c/BB+, WP-buiten D2/BB+, SV D5a/passief) als `#[ignore]`-fixtures met expected uit PDF p. 13-14; (b) 2 Uniec-replay-goldens (Gouda, Aalten) uit `.oes.json`, toleranties startend op Johns gekalibreerde band (±6-10%); (c) toleranties gemotiveerd vastleggen (versie-gap 2020↔2025 + geometrie-benadering) | fixtures compileren, staan rood/ignored, expected-provenance gedocumenteerd per waarde (paginanr/JSON-pad) | Bijlage 4-Excel (beslispunt 2) voor RVO-invoer-reconstructie |
| **F1 — Contract-unificatie** | één `EnergyCarrier` (workspace-breed, `serde`-compatibel); per service een `→ EpInputs`-mapping; automation-factoren daadwerkelijk toepassen; MJ↔kWh/m²-conversies; **BENG 1-aggregatie** (Q_H;nd+Q_C;nd)/Ag; BENG 2-eis-toets; BENG 3 conform (net-metering-regel); **geometrie-afhankelijke BENG 1-limiet** (Als/Ag-formule — Johns G5-les); TOjuli-indicator + GTO-drempels als output | `cargo test --workspace` + clippy groen; struct-wijzigingen workspace-wide (sed-les) | — |
| **F2 — Orchestrator** | `compute_beng(ProjectV2) → BengResult` in `openaec-project-shared`, naar het patroon van `compute_tojuli_full`: `geometry_to_nta8800` → demand → {heating, cooling, dhw, ventilation-aux, lighting, pv, automation} → EpInputs → ep + BENG-toets; manifest-resolver voor de Rekenzone-id-lijsten; volume→H_ve→τ-keten sluiten; additief `energy`-invoerblok op ProjectV2 (systemen: COP/η/WTW/SFP; PV: kWp/oriëntatie/tilt — structs spiegelen Johns `types.ts`) | to-juli-keten en isso-goldens blijven groen (`serde(default)` op alles) | F1 |
| **F3 — Kalibratie & normanalyse** | goldens uit F0 activeren; per afwijking norm-analyse (NTA 8800:2025-PDF via PyMuPDF) i.p.v. fudge; stubs prioriteren op gemeten impact (verwachte verdachten: tables-bijlage-stubs, F_sh=1,0, forfait-η's heating, humidity); afwijkingen kwantitatief rapporteren | 5 goldens groen binnen gemotiveerde toleranties, of eerlijk gedocumenteerd null-gap (§6.2-precedent) | F0+F2 |
| **F4 — API + frontend** | `POST /beng` route; `ActiveNorm::Beng`; invoerpaneel systemen/hernieuwbaar (UI-patronen uit open-energy-studio als referentie); resultatenpagina BENG 1/2/3 + TOjuli + label naast warmteverlies; graaddagen-schatter vervangen/degraderen | e2e vanuit de modeller-UI; rapport-integratie | F2 (parallel aan F3 mogelijk) |
| **F5 — Verbreding** | utiliteit-pad (ISSO 75, verlichting H.14 al aanwezig; Kijkduin-golden activeren), peak-koellast-engine (v1.0-blokkade), Herten-case als negatief-golden (B1-overschrijding), humidity normatief maken, versie-delta 2020→2025 documenteren | Kijkduin-golden groen; `vabi_koellast_golden` ignore weg | F3 |

### C2. Golden-validatiestrategie (samengevat)

- **Laag 1 — RVO-eindwaarden** (officieel, 45 rijen beschikbaar): pass/fail-golden per concept, tolerantieband te motiveren (versie 2020↔2025 + niet-exacte geometrie). Start ruim (bijv. ±10%), aanscherpen naarmate F3 vordert; elke aanscherping is winst, elke verruiming verboden zonder normanalyse.
- **Laag 2 — Certified Uniec-replay** (onafhankelijk): deterministische invoer, sub-totalen per dienst beschikbaar → diagnostisch veel sterker dan alleen eindwaarden. Woningen strak; Kijkduin pas na utiliteit-verlichting.
- **Laag 3 — Bijlage AA xlsm**: al groen (0,07%); uitbreiden met extra ruimte-fixtures is gratis.
- **Ontbrekend voor bit-exact:** officiële NTA 8800-validatietoolcases (BCRG/attestering) — pas relevant richting formele attestering (ISSO 54-testset, ~€1500, bewust "later").

### C3. Beslispunten voor de opdrachtgever

| # | Vraag | Advies | Besluit |
|---|---|---|---|
| 1 | **Doel-normversie:** referenties zijn NTA 8800:2020, norm op schijf is 2025+C1:2026 | implementeer **2025+C1** (de geldende norm), valideer tegen 2020-referenties met gemotiveerde tolerantie; documenteer bekende 2020→2025-wijzigingen per hoofdstuk in F3 | ✅ akkoord 11-07: altijd laatste normversie |
| 2 | **RVO "Bijlage 4"-Excel** (per-gevel geometrie van de voorbeeldconcepten) ontbreekt lokaal | opvragen bij RVO/DGMR; **Bijlage 4 is nooit los gepubliceerd** (alleen de PDF staat op rvo.nl). Alternatief direct beschikbaar: *Referentiegebouwen BENG* (RVO 2017, `rvo.nl/sites/default/files/2017/02/Referentiegebouwen BENG.pdf`) met geometrie van dezelfde referentiewoningen — bruikbaarheid verifiëren in F0 | 🔄 user vraagt op; F0 start met 2017-PDF als geometriebron |
| 3 | **Derde RVO-golden:** "Vrijstaande M" bestaat niet als BENG-referentie | **Vrijstaande L** (massief, echte referentie); Herten M eventueel later als negatief-golden | ✅ akkoord 11-07 |
| 4 | **Go/no-go F0+F1** | bij akkoord: F0 (goldens, geen engine-code) + F1 (contract-unificatie) als eerste delegaties | ✅ go 11-07 |

---

## Addendum 11-07 — branch `claude/nta8800-core` (Maarten Vroegindeweij, 04-07)

Na goedkeuring van dit plan bleek op origin een parallelle branch te staan (+6012 regels, 5 commits): nieuwe crate `nta8800-core` met een **volledige EP-keten-orchestrator** (transmissie→ventilatie→demand→alle diensten→`calculate_ep_score`→BENG 1/2/3), carrier-mappers, BBL-compactheidsformule en 20 fixtures. Read-only gereviewd; oordeel: **gedeeltelijk overnemen (b)**.

| Onderdeel branch | Oordeel | Consequentie voor dit plan |
|---|---|---|
| Bugfix `nta8800-pv` (dubbeltelling zoninstraling, ~2,3× te hoge opbrengst) | ✅ echte fix, direct overnemen | cherry-pick naar master vóór F3 (gedrag-breaking voor pv-consumers, juiste richting) |
| Bugfix `nta8800-ventilation` (f_SFP=0 voor systeem B/C → tabel-11.23 forfait 0,125 W/(m³/h)) | ✅ echte fix, direct overnemen | idem |
| `nta8800-ep` comment-fix (f_prim 2,5→1,45 bijlage Z) | ✅ doc-only | meenemen; verifiëren dat `primary_factor` zelf 1,45 rekent |
| EP-keten-orchestratie + carrier-mappers (`orchestrator.rs:82`, 760 r.) | ✅ als **logica** hergebruiken | verlicht F1/F2 substantieel: mapping-laag en keten-volgorde zijn uitgewerkt |
| BENG-module (`build_beng_summary`) | 🟡 mits gefixt | **BENG 1-grensformule heeft discontinuïteit bij Als/Ag=3,0** (segment 3 start op 100,0 i.p.v. ~90; eigen test cementeert de fout; geen bronvermelding Regeling Bouwbesluit/BBL) — fixen + bron citeren in F1 |
| Eigen `Project`-invoermodel (`model.rs:26`) | ❌ niet overnemen | concurreert met canoniek `ProjectV2`; dupliceert `q_v_oda_req` (`tojuli.rs:820`) en `orientation_from_degrees` (`nta8800_view.rs:287`). F2 blijft: orchestrator-kern áchter `ProjectV2` hangen (uitbreiding van het `compute_tojuli_full`-patroon) |
| 20 fixtures + `gen_fixtures.py` | 🟡 alleen als smoke-test | **self-referential** (geen externe expected; alleen plausibiliteits-asserts). README-claim "geverifieerde fixtures" is te sterk. Externe validatie blijft F0 (RVO + Uniec) |

**Governance:** branch niet overschrijven; fixes via cherry-pick (auteurschap behouden), orchestrator-hergebruik met bronvermelding. Afstemming met Maarten over de model-keuze vóór zijn branch verder groeit.
