# F8 — Formaat-analyse `.uniec3` native export + importer-spec

**Datum:** 2026-07-13
**Ticket:** TODO.md F8 stap 2 (format-analyse) + stap 3 (importer-spec)
**Status:** analyse afgerond, implementatie is een volgend pakket (F8 stap 4)
**Scope:** ontleden van het native Uniec 3-exportformaat (drie-puntjes-menu → exporteren)
en een mapping naar `BengGeometry` + `EnergyInput` + certified-`expected`, met
kruisvalidatie tegen de bestaande hand-fixtures.

**Analysebronnen:**
- `tests/verification/beng_uniec_crosscheck/aalten-2522/2522_woning-aalten_2024-11-22.uniec3` (Uniec 3.3.3.1)
- `tests/verification/beng_uniec_crosscheck/gouda-2467/2467_goejanverwelledijk-85-gouda_2024-09-17.uniec3` (Uniec 3.3.2.1)
- Versie-variatie: 6 extra exports uit `C:\Users\JochemK\Desktop\uniec\` (3.1.3.0 t/m 3.3.6.0, 2022–2025)
- Kruisreferentie: `beng_geometry.input.json`, `expected.json`, `uniec_fields_capture*.json` per case
- Verwant: `docs/2026-07-12-uniec-velden-inventarisatie.md` (UI-capture-inventarisatie, §5 mapping)

---

## 1. Kernconclusie

Het `.uniec3`-bestand is een **volledige, exacte** bron voor alles wat F8 nodig heeft.
Kruisvalidatie tegen de hand-fixtures: **28/28 velden OK (Aalten), 29/29 OK (Gouda), nul mismatches.**
Elke gevel sluit tot op 0,01 m² op de certified `BEGR_A` en op `CONSTRD_OPP + Σ ramen`.
Alle Rc/U/ggl/qv10/bouwwijze-codes én de certified BENG-uitkomsten (BENG 1/2/3, TOjuli,
label, eisen, PV-productie, per-functie primaire energie) zitten in het bestand.

**Voordeel t.o.v. Playwright-capture:** geen login/walker, geen stale-view-artefacten
(de her-capture-ellende bij Wand O/W in de UI-walk is hier een non-issue), geen leeg
grid-label-probleem. Het formaat is stabiel over 3 jaar app-versies (containerversie 2,
zie §6). Dit is de aanbevolen import-route.

---

## 2. Bestandsstructuur

`.uniec3` = **ZIP-archief** (geen extensie-magic; gewoon PKZIP). Alle JSON is
**UTF-8 met BOM** → lezen met `utf-8-sig` / in Rust de BOM strippen vóór `serde_json`.

```
<root>
├── meta.json            # app-versie, export-metadata, containerversie
├── folders.json         # mappenstructuur in de Uniec-cloud (irrelevant voor import)
├── projects.json        # projectmeta (naam, licentie) — 1 project
├── buildings.json       # lijst gebouwen met BuildingId + afmeldstatus
└── buildings/<BuildingId>/
    ├── summary.json     # ← certified BENG-resultaten, compact (KERNBRON resultaten)
    ├── entities.json    # ← alle invoer + resultaten als entity/property-graaf (KERNBRON invoer)
    ├── relations.json   # ← parent/child-relaties tussen entities (de hiërarchie)
    └── deltas.json      # wijzigingslog; in beide cases leeg `[]` → negeren
```

`meta.json`:
```json
{"Version":2,"App":"NTA8800, Version=3.3.3.1, ...","ExportedBy":"<guid>",
 "ExportedOn":"2024-11-22T09:42:06+01:00","RootFolderId":206818,"Environment":"app.uniec3.nl:443"}
```
- `Version` = **containerformaat-versie (2)** — stabiel over alle geteste app-versies.
- `App` → app-versie parsen uit `Version=x.y.z`, opslaan als provenance.

`buildings.json` → `[{"BuildingId":1556548,"ProjectId":...,"Afgemeld":true,"Afmeldstatus":20,...}]`.
Eén building per export in de praktijk (loop desondanks over de lijst).

---

## 3. Entity/property-datamodel

`entities.json` = platte **lijst** van entities (Aalten: 280 entities, 90 types). Elke entity:

```json
{ "NTAEntityId": "BEGR",                       // ENTITY-TYPE (de "tabelnaam")
  "NTAEntityDataId": "d97e0454-...",           // instance-GUID (primary key)
  "Order": 200.0,                              // volgorde binnen parent
  "NTAPropertyDatas": [
     { "NTAPropertyId": "BEGR_A", "Value": "21,96" },   // veldcode → waarde
     { "NTAPropertyId": "BEGR_GEVEL", "Value": "GVL_BTNL_N" }, ... ] }
```

`relations.json` = platte lijst van edges:
```json
{ "ParentId":"<dataId>", "NTAEntityIdParent":"UNIT-RZ",
  "ChildId":"<dataId>",  "NTAEntityIdChild":"BEGR", "OnDelete":1, "OnCopy":1 }
```

**Datatype-conventies (belangrijk voor de parser):**
| Conventie | Detail |
|---|---|
| Getallen | Nederlandse **decimaalkomma als string**: `"21,96"`, `"1,3"`. Parse: `replace(',', '.')` → f64. |
| Leeg/onbepaald | Property mist de `Value`-key **óf** `Value` is `""`/`"<none>"`/`"n.v.t."`. Behandel alle vier als "niet gezet". |
| Enums | Interne codes als string: `GVL_BTNL_N`, `VLAK_GEVEL`, `TGEB_GRWON`, `CONSTRM_FL_26`. |
| Referenties | GUID-string naar een andere entity: `CONSTRD_LIB = "2a983b24-..."` → LIBCONSTRD-instance. |
| `_NON`-suffix | Forfaitaire/berekende default. Basisveld (zonder `_NON`) = user-override; leeg ⇒ val terug op `_NON`. Bv. `VERW-OPWEK_COP` (leeg) vs `VERW-OPWEK_COP_NON` (4,10). |

De veldcodes (`BEGR_A`, `LIBCONSTRT_U`, `INFILUNIT_QV`, …) zijn **identiek** aan de
`uniec_fields_capture*.json`-codes uit de UI-walk. De mapping in
`docs/2026-07-12-uniec-velden-inventarisatie.md §5` geldt dus 1-op-1, maar leest nu
uit een stabiel bestand i.p.v. een grid-scrape.

---

## 4. Hiërarchie (relations-graaf)

Roots (entities zonder parent): `GEB`, `UNIT`, `BASIS`, `NTA-RESULTS`, `MWA-RESULTS`,
`INSTALLATIE`, `LIBCONSTRD`, `LIBCONSTRT`.

**Geometrie-pad (het pad dat F8 loopt):**
```
UNIT ──> UNIT-RZ ──> BEGR (n gevels) ──> CONSTRD  (opake delen; CONSTRD_LIB → LIBCONSTRD)
                                    └──> CONSTRT  (ramen/deuren; CONSTRT_LIB → LIBCONSTRT)
                                                   └──> BELEMMERING, CONSTRZOMNAC
```
- `BEGR` is de **begrenzing/gevel** (thermische schil). Bereikt via `UNIT-RZ`, niet via `RZ`.
  (`RZ` is een child van de installaties — het is de zone-toewijzing, niet de geometrie.)
- De lib-koppeling zit **dubbel**: als property (`CONSTRD_LIB`/`CONSTRT_LIB` = GUID) én als
  relation (`LIBCONSTRD → CONSTRD`). Gebruik de property; de relation is redundant.

**Installatie-pad:**
```
INSTALLATIE ──> VERW ──> VERW-OPWEK / VERW-AFG / VERW-DISTR / VERW-VAT
            ├─> TAPW ──> TAPW-OPWEK / TAPW-AFG / TAPW-DISTR / TAPW-VAT / TAPW-UNIT
            ├─> VENT ──> VENTILATOR / WARMTETERUG / VENTDEB / VENTDIS / ...
            ├─> KOEL ──> KOEL-OPWEK / KOEL-AFG / KOEL-DISTR
            └─> PV   ──> PV-VELD ──> BELEMMERING
UNIT ──> INFILUNIT   (qv10 per unit)
```

**Resultaat-pad:** `BASIS`/`NTA-RESULTS`/`GEB`/`UNIT` ──> `PRESTATIE`,
`RESULT-ENERGIEGEBRUIK`, `RESULT-ENERGIEFUNCTIE` (44×), `RESULT-TOJULI`, `RESULT-PV`,
`RESULT-CONSTRT`, `RESULT-GTO`, `RESULT-LSTRM`.

> **Duplicaat-patroon:** de meeste RESULT-types en `PRESTATIE` komen **2× (of 2 subsets)**
> voor — een gevulde instance (`Order` 100) en een lege/tweede (`Order` 200/300),
> plus gebouw- vs unit-niveau. Filter op de instance mét niet-lege `Value`s.
> Voor single-unit woningen zijn gebouw- en unit-niveau gelijk. Voor de compacte
> resultaten is `summary.json` het pad van de minste weerstand.

---

## 5. Mappingtabel

Legenda dekking: ✅ direct, ◑ transformatie/lookup nodig, ⚠ deels/aanname, ✖ niet in bestand.

### 5a. Geometrie → `BengGeometry` (`crates/openaec-project-shared/src/beng_geometry.rs`)

| Uniec entity.veld | capture-code | DTO-veld (`BengGeometry`) | transformatie | dekking |
|---|---|---|---|---|
| `UNIT-RZ.UNIT-RZAG` | RZAG | `BengZone.a_g_m2` | komma→f64 | ✅ |
| `RZ.RZ_BOUWW_VL` | RZ_BOUWW_VL | `BengZone.bouwwijze_vloer` | code-string 1:1 | ✅ |
| `RZ.RZ_BOUWW_W` | RZ_BOUWW_W | `BengZone.bouwwijze_wand` | code-string 1:1 | ✅ |
| `UNIT.UNIT_TYPEWON` | UNIT_TYPEWON | `BengZone.woningtype` | code-string 1:1 | ✅ |
| `BEGR.BEGR_OMSCHR` | — | `BengBoundary.omschrijving` | string | ✅ |
| `BEGR.BEGR_VLAK` | BEGR_VLAK | `BengBoundary.vlak_type` (`VlakType`) | `VLAK_VLOER`→Vloer, `VLAK_GEVEL`→Gevel, `VLAK_DAK`→Dak | ✅ |
| `BEGR.BEGR_A` | BEGR_A | `BengBoundary.bruto_buiten_opp_m2` | komma→f64 (= bruto buitenmaat) | ✅ |
| `BEGR.BEGR_GEVEL` | BEGR_GEVEL | `BengBoundary.grenst_aan` (oriëntatie) | `GVL_BTNL_N/O/Z/W`→noord/oost/zuid/west | ✅ |
| `BEGR.BEGR_HEL` | BEGR_HEL | `BengBoundary.helling_deg` | `"90"`→90, `"n.v.t."`→None, dak-getal 1:1 | ✅ |
| `BEGR.BEGR_VLOER` | BEGR_VLOER | `BengAdjacency` (vloer-subtype) | `VL_MV_GRSP`→vloer-op-maaiveld etc. | ◑ |
| `CONSTRD.CONSTRD_OPP` | CONSTRD_OPP | opaak-oppervlak (impliciet: `BEGR_A − Σ ramen`) | komma→f64 | ✅ |
| `CONSTRD.CONSTRD_LIB` | CONSTRD_LIB | `BengBoundary.constructie_ref` → `OpaqueConstructionDef` | GUID → LIBCONSTRD | ✅ |
| `LIBCONSTRD.LIBCONSTRD_OMSCHR` | — | `OpaqueConstructionDef.omschrijving` | string | ✅ |
| `LIBCONSTRD.LIBCONSTRD_TYPE` | — | `OpaqueConstructionDef.kind` (`VlakType`) | `LIBVLAK_VLOER/GEVEL/DAK` | ✅ |
| `LIBCONSTRD.LIBCONSTRD_RC` | LIBCONSTRD_RC | `OpaqueConstructionDef.thermal` (`RcOrU::Rc`) | komma→f64 | ✅ |
| `CONSTRT.CONSTRT_LIB` | CONSTRT_LIB | `BengWindowPlacement.kozijn_ref` → `WindowDef` | GUID → LIBCONSTRT | ✅ |
| `CONSTRT.CONSTRT_AANT` | CONSTRT_AANT | `BengWindowPlacement.aantal` | int | ✅ |
| `CONSTRT.CONSTRT_OPP` | CONSTRT_OPP | (controle: = `LIBCONSTRT_AC × CONSTRT_AANT`) | komma→f64 | ✅ |
| `CONSTRT.CONSTRT_BESCH` | CONSTRT_BESCH | `BengWindowPlacement.belemmering` (`Obstruction`) | `BELEMTYPE_MIN`→minimal, `n.v.t.`→none, `BELEMTYPE_ZIJ_*`→⚠ minimal (V1 kent geen zij) | ⚠ |
| `CONSTRT.CONSTRT_ZONW` | CONSTRT_ZONW | `BengWindowPlacement.zonwering` (`MovableSunShading`) | `ZONW_GEEN`→None | ◑ |
| `CONSTRT.CONSTRT_ZNVENT` | CONSTRT_ZNVENT | `BengWindowPlacement.zomernachtventilatie` | `ZOMERNVENT_NAANW`→false | ◑ |
| `LIBCONSTRT.LIBCONSTRT_OMSCHR` | — | `WindowDef.omschrijving` | string (merk A/B/C…) | ✅ |
| `LIBCONSTRT.LIBCONSTRT_TYPE` | — | `WindowDef.kind` (`KozijnType`) | `TRANSTYPE_RAAM`→raam; deur via ggl=0 | ◑ |
| `LIBCONSTRT.LIBCONSTRT_U` | LIBCONSTRT_U | `WindowDef.u_w_per_m2k` | komma→f64 | ✅ |
| `LIBCONSTRT.LIBCONSTRT_G` | LIBCONSTRT_G | `WindowDef.ggl` | komma→f64 | ✅ |
| `LIBCONSTRT.LIBCONSTRT_AC` | LIBCONSTRT_AC | `WindowDef.area_m2` | komma→f64 (per-merk oppervlak) | ✅ |

### 5b. Installaties + infiltratie → `EnergyInput` (`energy.rs`) + `shared` (`shared.rs`)

| Uniec entity.veld | DTO-veld | transformatie | dekking |
|---|---|---|---|
| `INFILUNIT.INFILUNIT_QV` | `shared.q_v10_spec_dm3_s_m2` | komma→f64 (zelfde eenheid dm³/s·m²) | ✅ |
| `INSTALLATIE.INSTALL_TYPE` | routeert naar heating/dhw/vent/cooling/pv | `INST_VERW/TAPW/...` | ✅ |
| `VERW-OPWEK.VERW-OPWEK_TYPE` + `_POMP` | `HeatingInput.generator` (`HeatGeneratorType`) | `VERW-OPWEK_POMP_BUWA`→HeatPumpAir/Ground; ketel→HrBoiler | ◑ |
| `VERW-OPWEK.VERW-OPWEK_COP` (of `_NON`) | `HeatingInput.cop` | komma→f64, `_NON`-fallback | ✅ |
| `VERW-AFG` afgifte-code | `HeatingInput.emission` (`HeatEmissionType`) | code-map | ◑ |
| `TAPW-OPWEK.TAPW-OPWEK_TYPE` + `_BRON_POMP` | `DhwInput.generator` (`DhwGeneratorType`) | code-map | ◑ |
| `TAPW-OPWEK.TAPW-OPWEK_COP_NON` / rend | `DhwInput.efficiency` | komma→f64 | ✅ |
| `VENT.VENT_SYS` + `VENT_VARIANT` | `VentilationInput.system` (`VentilationSystemType`) | `VENTSYS_MECHC`+`VARIANT_D2`→systeem-map (let op NTA-conventie B/C/D) | ◑ |
| `WARMTETERUG.WARMTETERUG_REND` + `_WTW` | `VentilationInput.wtw_efficiency` | `WARMTETERUG_WTW_NIET`→None; anders rend komma→f64 | ◑ |
| `KOEL-OPWEK` (aanwezig?) | `CoolingInput` (`None` = geen actieve koeling) | KOEL-subtree aanwezig → vul; SEER/COP uit `_NON` | ⚠ |
| `PV.PV_WPPRDT` / `PV_WPM2_NON` | `PvInput.peak_power_kwp` | Wp→kWp (÷1000) | ◑ |
| `PV-VELD.PV-VELD_ORIE` | `PvInput.azimuth_degrees` | `PVORIE_N/O/Z/W`→azimut-graden | ◑ |
| `PV-VELD.PV-VELD_HELLING` | `PvInput.tilt_degrees` | getal 1:1 | ✅ |
| `PV.PV_VEROUDERING` | `PvInput.system_efficiency` (of verouderingsfactor) | komma→f64 | ◑ |
| `SETTINGS` / BACS | `EnergyInput.automation` (`AutomationInput`) | ⚠ code onbekend → default klasse C | ⚠ |

> **Nuance PV-Wp:** `PV_WPPRDT` (6736) ≠ `PV-VELD_AANTALPNL × PV_WPPNL_NON` (10×410=4100).
> `PV_WPPRDT` is het productblad-totaal, de aantal×paneel het veld-totaal. Bij implementatie
> uitzoeken welke de certificering aanhoudt (verwachting: het veld-totaal per PV-VELD).

### 5c. Certified resultaten → `expected` (referentie-vergelijk in UI)

| Bron | expected-veld | transformatie | dekking |
|---|---|---|---|
| `summary.json.EP_BENG1` (= `PRESTATIE.EP_BENG1`) | `beng1_kwh_m2_jr` | komma→f64 | ✅ |
| `summary.json.EP_BENG2` | `beng2_kwh_m2_jr` | | ✅ |
| `summary.json.EP_BENG3` | `beng3_pct` | | ✅ |
| `summary.json.EP_BENG{1,2,3}_EIS` | `*_limit_*` | | ✅ |
| `summary.json.EP_TOJULI` + `_EIS` | TOjuli-waarde + eis | | ✅ |
| `summary.json.EP_ENERGIELABEL` | `energy_label` | string (`A+++`) | ✅ |
| `RESULT-ENERGIEGEBRUIK.RESULT-HERNIEUW_ELEKTR` | `pv_production_kwh` | komma→f64 (= `RESULT_KARAKT_OPGEW_E`) | ✅ |
| `RESULT-ENERGIEFUNCTIE` (`_CAT`=RESULT_VERW, `_RES_ENER_PRIM`) | `heating_primary_kwh` | Σ ENER_PRIM per categorie | ◑ |
| `RESULT-ENERGIEFUNCTIE` (`_CAT`=RESULT_TAPW) | `hot_water_primary_kwh` | Σ (ENER+HULP)_PRIM | ◑ |
| `RESULT-ENERGIEFUNCTIE` (`_CAT`=RESULT_KOEL) | `cooling_primary_kwh` | Σ per categorie | ◑ |
| `RESULT-ENERGIEFUNCTIE` (`_CAT`=RESULT_VENT) | `fans_primary_kwh` | Σ HULP_PRIM (ventilatoren) | ◑ |
| `RESULT-ENERGIEGEBRUIK.RESULT-EP_WARMTEBEHOEFTE` | warmtebehoefte kWh/m² | | ✅ |
| `RESULT-ENERGIEGEBRUIK.RESULT-OPP_VORMFACTOR` / `_VERLOPP` | vormfactor / verliesoppervlak | | ✅ |

> **Nuance per-functie primair:** de exacte `expected`-getallen (heating 2551, tapw 1813,
> koel 422, vent 443) zijn reproduceerbaar uit `RESULT-ENERGIEFUNCTIE`, maar de precieze
> som-definitie verschilt per categorie (heating = `RES_ENER_PRIM` **zonder** hulpenergie;
> vent = **alleen** hulpenergie = ventilatoren). Plus de 44-entities bevatten gebouw- én
> unit-niveau. Bij implementatie: filter op gevulde instances, aggregeer per `_CAT`, en
> ijk de som-definitie tegen `expected.json`. Voor de eerste versie volstaat `summary.json`
> (BENG 1/2/3 + label + eisen); de per-functie-uitsplitsing is een verfijning.

---

## 6. Versie-stabiliteit

`meta.Version` (containerformaat) = **2** in álle geteste exports, van app 3.1.3.0 (2022-08)
t/m 3.3.6.0 (2025-07). Het entity/property-model is stabiel; alleen het **aantal** codes
groeit met nieuwe NTA-features (81 types/627 props in 3.1.3.0 → 90/719 in 3.3.3.1).

Alle voor F8 kritische entity-types en property-codes zijn aanwezig in **alle** versies,
met één uitzondering:

| Wijziging | Versie | Impact | Mitigatie |
|---|---|---|---|
| `RZ_BOUWW` (één veld, thermische massa) split in `RZ_BOUWW_VL` + `RZ_BOUWW_W` | vanaf 3.2.x (2022→2024) | 3.1.x-exports missen de gesplitste codes | fallback: als `_VL`/`_W` ontbreken, lees `RZ_BOUWW` en dupliceer naar beide |

**Aanbeveling:** parse tolerant — onbekende entity-types/property-codes overslaan (niet
falen), zodat nieuwere app-versies met extra velden blijven importeren. Log de app-versie
als provenance en waarschuw pas bij een onbekende `meta.Version` ≠ 2.

**Corpus-caveat:** alle 116 beschikbare `.uniec3`-bestanden zijn **woningen** (grondgebonden,
woonark, drijvende woning). De enige utiliteit ("clubgebouw") is PDF-only, geen export.
Het entity-model is generiek (`GEB_TYPEGEB` = `TGEB_GRWON` stuurt woning; utiliteit heeft
andere `TGEB_*` + rekenzone-structuur), maar utiliteit-import is **onbeproefd** → open item.

---

## 7. Implementatie-fasering (F8 stap 4)

| Fase | Deliverable | Inhoud |
|---|---|---|
| **4a. Parser-crate** | `uniec3-import` (nieuwe crate, of module in `openaec-project-shared`) | ZIP-uitpakken (bv. `zip` crate), BOM-strippen, `serde_json` deserialize van meta/buildings/entities/relations. Entity-index (`HashMap<GUID, Entity>`) + children-index uit relations. Dutch-komma f64-helper + `_NON`-fallback-helper. |
| **4b. Geometrie-mapper** | `entities → BengGeometry` | Loop `UNIT-RZ → BEGR → CONSTRD/CONSTRT`, resolve LIB-GUIDs, bouw `OpaqueConstructionDef`/`WindowDef`-libs (dedup op GUID), map enums (§5a). Herbruik de validatie-invarianten (gevel sluit op `BEGR_A`). |
| **4c. Installatie-mapper** | `entities → EnergyInput` + `shared.q_v10` | §5b. Enum-mapping-tabellen voor generator/emission/vent-systeem; `_NON`-fallbacks. Koeling optioneel. |
| **4d. Resultaat-extractie** | `summary.json + RESULT-* → UniecReference` | §5c. Struct voor certified-vergelijk; eerst `summary.json`, later per-functie. |
| **4e. Error-handling** | typed errors | corrupte ZIP, ontbrekende building, `meta.Version ≠ 2`, onbekende enum-codes (skip+warn, niet falen). Verzamel warnings i.p.v. hard-fail. |
| **4f. UI-importknop** | frontend | file-upload `.uniec3` → parse → ProjectV2 (`beng_geometry` + `energy`) + certified-referentie. |
| **4g. Vergelijkings-weergave** | frontend | eigen BENG-uitkomst naast Uniec-certified (per BENG-indicator + per energiefunctie), met residu-% zoals de golden-toleranties. |
| **4h. Validatie** | round-trip test | geïmporteerd Aalten/Gouda kruisgecheckt tegen `beng_geometry.input.json` + `expected.json` (deze analyse bewijst 28/28 + 29/29 — automatiseer als regressietest). |

**Open vragen voor stap 4:**
1. PV-Wp: productblad-totaal (`PV_WPPRDT`) vs veld-totaal (`aantal × Wp/paneel`) — welke hanteert de certificering?
2. Per-functie primair-energie som-definitie per `_CAT` (ENER vs +HULP) exact ijken op `expected.json`.
3. `CONSTRT_BESCH` zijbelemmering (`BELEMTYPE_ZIJ_*`) → V1 `Obstruction` kent alleen None/Minimal; benaderen als minimal (verlies-arm) of `Obstruction` uitbreiden?
4. Utiliteit-import onbeproefd (geen `.uniec3`-sample) → apart valideren zodra een utiliteit-export beschikbaar is.
5. Meerdere rekenzones / meerdere units (appartementen): cases hier zijn single-zone woningen; multi-`UNIT`/`UNIT-RZ` traversal verifiëren op een appartement-export.

---

## 8. Implementatie (fase 4a–4e, 13-07)

Backend-crate **`crates/uniec3-import`** gebouwd conform §7. Publieke API:

```rust
pub fn import_uniec3(bytes: &[u8]) -> Result<Uniec3Import, Uniec3ImportError>;
pub struct Uniec3Import { pub project: ProjectV2, pub certified: Uniec3CertifiedResults, pub warnings: Vec<String> }
```

**Modulestructuur:** `parse` (4a ZIP/BOM/serde + 4b `EntityIndex`), `geometry`
(4c), `installations` (4d), `results` (4e), `error` (typed, tolerant vs hard).

### Besluiten op de open vragen (met bewijs)

| # | Vraag | Besluit | Grondslag |
|---|---|---|---|
| 1 | PV-Wp productblad vs veld | **veld-totaal** `aantal_pnl × PV_WPPNL_NON / 1000` kWp | PM-besluit; `PV_WPPRDT`-afwijking (bv. 6736 vs 4100 Wp) als **warning** meegegeven, niet gebruikt. Empirische ijk tegen certified vergt de (nu rode) `compute_beng`-keten en is bewust uitgesteld — de definitiekeuze is gedocumenteerd, niet locked. |
| 2 | Per-functie primair-som | **Σ `RES_ENER_PRIM` per `_CAT`** (zónder hulpenergie), gesommeerd over alle instances | Empirisch op de golden: VERW 2550,7≈2551 · TAPW 1812,6≈1813 · KOEL 421,8≈422 · VENT 442,9≈443 (hulp-optel zou KOEL naar 436 tillen → fout). Unit-niveau-instances staan op 0 → geen dubbeltelling. |
| 3 | Zijbelemmering | `BELEMTYPE_ZIJ_*`/onbekend → `Obstruction::Minimal` + note | Analyse §5a; enum-uitbreiding = F8-V2-ticket. |
| 4 | Utiliteit | `GEB_TYPEGEB` zonder `WON`/`WOON` → `UtilityUnsupported` | `TGEB_GRWON` (grondgebonden) én `TGEB_WOONBB` (woonark/drijvende woning) zijn woningbouw; echte utiliteitscodes falen netjes. |
| 5 | Multi-zone | >1 `UNIT` of >1 `UNIT-RZ` → `MultiUnitUnsupported` | Nette, specifieke fout (geen stille eerste-keuze); V2-ticket. |

### Extra bevinding — twee kozijn-invoermodi

De corpus bevat twee Uniec-invoermodi voor kozijnen: **oppervlakte-per-merk**
(`LIBCONSTRT_AC` gevuld → één gedeelde `WindowDef`, het pad van de goldens) en
**oppervlakte-per-raam** (`AC` leeg → oppervlak op de plaatsing `CONSTRT_OPP`). De
mapper detecteert de modus per merk en synthetiseert bij de tweede een
plaatsing-eigen `WindowDef` (`opp / aantal`). Zonder deze split faalden vier
corpus-bestanden op een dangling `WindowDef`-referentie.

### Validatie-uitkomst

- **Round-trip (kernvalidatie):** geïmporteerd Aalten = **31/31** velden exact
  tegen de hand-fixture, Gouda = **35/35** (na fix: omtrek P ook op de
  vloer-op-kruipruimte, die het bestand wél draagt). Vergelijking op waarde
  (Rc/U/ggl/opp/oriëntatie), id-onafhankelijk. Certified matcht `expected.json`
  (BENG 1/2/3 + eisen + label + per-functie primair + PV + koudebehoefte).
- **CI-dekking zonder klantdata:** synthetische in-memory `.uniec3`-fixture
  (`tests/synthetic.rs`) dekt parsing + geometrie + installaties; round-trip
  skipt netjes als de gitignored `.uniec3`-bronnen ontbreken.
- **Variatie-smoke** (`tests/variation_smoke.rs`, `#[ignore]`) over 52
  corpus-bestanden (app 3.2.6.0 → 3.3.5.3, 2022–2025, incl. woonark/drijvende
  woning): **37 OK, 15 correct geweigerd** als multi-zone (2–3 `UNIT-RZ`, V2). Nul
  panics, nul onverwachte hard-errors.

**Resterend (F8-V2):** multi-zone/appartementen + utiliteit-traversal;
`ZONW_*`→`MovableSunShading`; zijbelemmering-enum.

## Fase 4f–4h — UI-ontsluiting (13-07)

De importer is ontsloten in de app; de contract-keten is symmetrisch web/desktop.

- **API-route** — `POST /api/v1/beng/import-uniec3`
  (`crates/isso51-api/src/handlers/uniec_import.rs`). Body = base64-JSON
  `{ file_base64 }` (géén multipart — houdt de client-dispatch gelijk aan de rest
  van de BENG-familie). Eigen router met een **8 MB** body-limit náást de 2 MB
  compute-router (een base64-`.uniec3` kan de compute-default overschrijden),
  zelfde per-IP rate-limit. `spawn_blocking`→`import_uniec3`; succes →
  `{ project, certified, warnings }`. `Uniec3ImportError` → **422** met de
  letterlijke `Display`-boodschap als `detail` (de multi-zone/utiliteit-afwijzing
  is directe user-feedback en moet ongewijzigd doorkomen); ongeldige base64 →
  **400**. Route-tests: geldig synthetisch archief (200), kapotte ZIP (422),
  multi-zone (422 + boodschap-check), ongeldige base64 (400), en een
  **Aalten-golden E2E** die skipt als het gitignored bestand ontbreekt.
- **Tauri-command** — `import_uniec3(file_base64)` in `src-tauri/src/commands.rs`
  (geregistreerd in `lib.rs`), identiek contract.
- **Frontend** — importknop + `.uniec3`-bestandskiezer in de BENG-tab, dispatch
  via `frontend/src/lib/uniecImport.ts`. Na import wordt de wire-`ProjectV2` via
  `splitV2ForStore` naar de V1-store + sidecar gesplitst en worden de
  BENG-invoerblokken (`energy` + `beng_geometry`) uit de top-level velden
  hersteld; een overschrijf-bevestiging verschijnt alléén als er al invoer staat.
  De certified referentie leeft additief als `projectStore.uniecReference`
  (persist-migratie + regressietest). Op de resultatenpagina staat een
  **vergelijkings-paneel**: onze `compute_beng` BENG 1/2/3 naast de certified
  Uniec-waarden met delta en een **indicatieve** tolerantie-kleuring (BENG 1
  ±6 %, BENG 2 ±10 %, BENG 3 ±3 pp). NL/EN i18n voor alle nieuwe strings.
  `q_v10;spec` reist mee door de V1↔V2-round-trip (`SharedExtra`) zodat de
  recompute dezelfde infiltratie ziet als de afgemelde export.
