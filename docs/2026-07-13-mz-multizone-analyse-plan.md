# MZ — Multi-zone `.uniec3`-import (V2): analyse + implementatieplan

**Datum:** 2026-07-13
**Ticket:** TODO.md F8-V2 (multi-zone), verwant: F8 (`docs/2026-07-13-f8-uniec3-formaat-analyse.md`), Zone-model ADR
**Status:** analyse afgerond — implementatie is een volgend pakket
**Scope:** waaróm de importer 15/52 korpus-bestanden weigert, wat de engine met meerdere
`BengGeometry.zones` doet, wat NTA 8800 eist bij meerdere rekenzones, en een gefaseerd plan.

**Analysebronnen:**
- Korpus `C:\Users\JochemK\Desktop\uniec\` — 5 multi-zone-bestanden geopend (Python, ZIP+JSON-graaf):
  woning-2176, woonark-2248, woning-1838, woning-2703, drijvende-woning-3003
- Engine: `uniec3-import/src/geometry.rs`, `openaec-project-shared/src/{beng/geometry_bridge.rs, beng/mod.rs, nta8800_view.rs, tojuli.rs}`
- Norm: NTA 8800:2025+C1:2026 §6.5 (rekenzone-indeling), §6.6 (A_g), §8.2.2 + §10.5 (per-zone demand), p.536 (tapwater 1 systeem)
- Certified: `summary.json` + `RESULT-*`-entities per korpus-bestand

---

## 1. Kernconclusie + scope-aanbeveling

**Aanbeveling: (c) beide, gefaseerd.** Niet (a)-alleen: de importer-guard weghalen zou de
15 multi-zone-bestanden wél laten importeren, maar de engine poolt ze dan tot één rekenzone —
dat is een **stille norm-afwijking** (de winstbenutting η en de tijdconstante τ worden dan over
de gepoolde schil berekend i.p.v. per rekenzone). Dat botst met de transparantie-huisregel.

| Fase | Wat | Norm-status | Omvang |
|------|-----|-------------|--------|
| **V2a** | Importer accepteert N rekenzones (1 UNIT); engine poolt tot één rekenzone (bestaand gedrag), + fix van 2 `zones.first()`-bugs; expliciete "multi-zone gepoold, indicatief"-note/warning | **Benaderend** (transmissie/A_g/ventilatie exact-lineair; η/τ gepoold) | Klein — importer ~30 regels + 2 engine-fixes + 1 golden |
| **V2b** | Demand per rekenzone (§8.2.2, eigen τ/C_m/Φ_int), sommeren, diensten op de som (§10.5, p.536) | **Norm-exact** (reproduceert certified binnen F8-tolerantie) | Middel — `compute_beng`-orchestratie + brug + tojuli per-zone + view |

**Wat NIET in scope:** meerdere `UNIT`-entiteiten (appartementen/meergezins) blijven een nette
`MultiUnitUnsupported`-fout — dat zijn aparte woonfuncties met eigen installaties en een eigen
BENG-toets per woning, een fundamenteel groter pakket. Het bestaande F8-V2-ticket klutst
"multi-zone/appartementen" samen; dit doc splitst ze: **multi-rekenzone (1 UNIT, dit pakket)**
versus **multi-UNIT (appartementen, apart)**.

---

## 2. Korpus-analyse — waarom een adviseur splitst

Alle 5 onderzochte multi-zone-bestanden hebben **exact hetzelfde patroon: 1 UNIT, meerdere
UNIT-RZ (rekenzones), één gedeelde installatieset.** De installaties (`VERW/TAPW/VENT/KOEL/PV`)
hangen zonder uitzondering op UNIT-niveau (`INSTALLATIE`), nooit per rekenzone. Infiltratie
(`INFILUNIT_QV`) is één waarde per UNIT.

| Case (projectnr.) | Gebouwtype | Rekenzones (RZ_OMSCHR → A_g m²) | Splits-reden | Installaties |
|---|---|---|---|---|
| 2176 | grondgebonden | verdiepingen 159 · Kelder 117,1 · begane grond 159 | verdieping-groepen + kelder | 1 set gedeeld |
| 2248 | woonark/drijvend | Water deel 86,1 · Bovenwater deel 82,11 | onder- vs bovenwaterlijn (grens water↔buitenlucht) | 1 set gedeeld |
| 2703 | grondgebonden | Woning 207 · kelder 4,0 | hoofdvolume + kleine kelder | 1 set (2 tapw-opwekkers, 1 systeem) |
| 3003 | drijvend | bak 82,99 · woning 119,82 | casco/ponton vs woonlaag | 1 set gedeeld |
| 1838 | grondgebonden | *1 rekenzone* (291,8) | — (single-zone, importeert nu al) | 1 set |

**Patroon:** de splits is **geometrisch/bouwkundig**, niet installatie- of functie-gedreven.
Terugkerende drijfveren: (1) een **kelder** met afwijkend grondcontact/constructie, (2) bij
**drijvende woningen** een casco-/waterdeel met een fundamenteel andere begrenzing (water i.p.v.
buitenlucht), (3) logische **verdieping-groepering**. Alle zones vallen binnen één woonfunctie
en dezelfde setpoint (§6.5.2: ≤ 4 K verschil, of dominante functie ≥ 90 %). Dat is precies het
geval waarvoor de gedeelde-installatie-aggregatie van de norm geldt (§10.5, p.536).

**Gevolg voor het ontwerp:** V2b hoeft géén per-zone installaties, géén per-zone infiltratie en
géén per-zone woonfunctie te modelleren. Alleen de **energiebehoefte** (Q_H;nd/Q_C;nd) moet per
rekenzone, daarna gesommeerd; de bestaande `EnergyInput` (project-breed, één set) consumeert de
som ongewijzigd.

---

## 3. Engine-staat — waar zitten de single-zone-aannames

De keten draagt meerdere zones verrassend ver, maar knijpt ze op één plek plat. Van bron naar
resultaat:

| # | Locatie (bestand:regel) | Gedrag bij N zones | Oordeel |
|---|---|---|---|
| 1 | `uniec3-import/src/geometry.rs:203-208` | **Harde afwijzing** `unit_rzs.len() > 1` → `MultiUnitUnsupported` | De poort. Weg in V2a. |
| 2 | `beng/geometry_bridge.rs:111-133` | Loopt correct over `beng.zones`, produceert **1 `Space` per zone** | ✅ multi-zone-klaar |
| 3 | `nta8800_view.rs:95-136` | Sommeert alle spaces → **1 `Rekenzone`** (Σ floor_area, alle constructions gepoold) | Poolt: lineair OK, η/τ NIET |
| 4 | `tojuli.rs:175,247,654` | Transmissie/zonwinst itereren álle `geometry.spaces` | ✅ som is correct |
| 5 | `beng/mod.rs:405` | `view.rekenzones.first()` — DHW/ventilatie op de (gepoolde) rekenzone | OK zolang view poolt |
| 6 | `beng/mod.rs:322,372` | Thermische massa + **Φ_int uit `zones.first()`** (`derive_internal_gains_woningbouw(first_zone.a_g_m2, 1.0)`) | **BUG bij N zones** — gebruikt alleen 1e zone-A_g |
| 7 | `beng/mod.rs:317,385` | `a_g_total = Σ zones` → op `gross_floor_area_m2` | ✅ totaal correct |

**Kern:** de brug (#2) en de demand-transmissie (#4) zijn al zone-agnostisch en **lineair**, dus
Σ A·U en Σ A_g kloppen ook bij pooling. De niet-lineaire posten zitten in #3/#6:
- **Winstbenutting η en τ** worden over de gepoolde schil bepaald. Voor een kelder (lage interne
  winst, hoge massa) + woonlaag (hoge winst) wijkt de gepoolde η af van de som van per-zone-η's.
  Dit is de bron van de V2a-benadering-onnauwkeurigheid.
- **Φ_int (#6)** is nu letterlijk fout bij meerdere zones: `first_zone.a_g_m2` i.p.v. `A_g;tot`.
  Formule 7.21 schaalt met A_g; op de eerste (vaak kleinste) zone levert dat een te lage interne
  winst. Dit moet in V2a al gefixt worden (gebruik `A_g;tot`), anders is zelfs de gepoolde
  uitkomst onnodig scheef.

---

## 4. Norm — aggregatie bij meerdere rekenzones (NTA 8800:2025+C1:2026)

De norm rekent expliciet **per rekenzone en sommeert**, hij voegt zones niet samen:

- **§6.5 Indeling in rekenzones** — elke klimatiseringszone valt uiteen in ≥ 1 rekenzone. §6.5.2:
  binnen één rekenzone verschilt de verwarmings-setpoint ≤ 4 K (of dominante functie ≥ 90 %).
  De korpus-splitsingen voldoen hieraan (alle woonfunctie, één setpoint).
- **§6.6.2 A_g;tot** — "de som van de gebruiksoppervlakten van alle rekenzones". → BENG-noemer =
  Σ A_g;zi. **Bevestigd op het bestand:** certified A_g = 435,10 = 159 + 117,1 + 159 (woning 2176);
  202,81 = 82,99 + 119,82 (drijvende woning 3003).
- **§8.2.2 + §10.5.2 (formule 10.19, p.377; 10.46/10.47, p.398)** — de netto warmte-/koudebehoefte
  van de thermische zone = Σ over de rekenzones van de **per-zone** bepaalde Q_H;nd;zi / Q_C;nd;zi.
  Elke zone krijgt een eigen maandbalans (§8.2.2) met eigen τ, interne winst en zonwinst.
- **Tapwater p.536** — "Bij toepassing van één warmtapwatersysteem voor de gehele woning wordt de
  nettowarmtebehoefte voor alle rekenzones bepaald en samengenomen." Distributieverliezen worden
  naar rato van A_g over de rekenzones verdeeld (p.286). → één gedeelde installatie (ons geval)
  bedient de **gesommeerde** behoefte.

**Conclusie:** de norm = per-zone demand → sommeren → diensten op de som → één BENG-triplet per
UNIT. Zones samenvoegen tot één rekenzone (huidige view, #3) is **geen** norm-conforme route; het
klopt alleen voor de lineaire posten. V2b implementeert de per-zone-som; V2a levert de lineaire
benadering met expliciete markering.

---

## 5. Certified kruiscontrole

`summary.json` levert per bestand **precies één** BENG 1/2/3-triplet + label + TOjuli, ongeacht
het aantal rekenzones — de certificering aggregeert dus over de zones heen tot woning-niveau:

| Case | BENG 1 | BENG 2 | BENG 3 | Label | A_g certified | Σ zones | Match |
|---|---|---|---|---|---|---|---|
| 2176 woning | 72,49 | 22,00 | 75,9 | A+++ | 435,10 | 159+117,1+159 | ✅ exact |
| 3003 drijvende woning | 100,13 | 41,30 | 71,9 | A+++ | 202,81 | 82,99+119,82 | ✅ exact |
| 2248 woonark | 103,98 | 11,47 | 91,5 | A+++ | (168,21) | 86,1+82,11 | ✅ |
| 2703 woning | 75,61 | 1,63 | 98,3 | A+++ | (211,0) | 207+4,0 | ✅ |

De certified `Uniec3CertifiedResults`-extractie (`results.rs`) werkt al voor multi-zone: A_g/A_ls/
vormfactor komen uit de gevulde `RESULT-ENERGIEGEBRUIK`-instance (gebouw-niveau, al geaggregeerd),
en de per-functie primair-energie sommeert over `RESULT-ENERGIEFUNCTIE`. Er is dus **geen** werk
aan de certified-kant nodig; het vergelijkingsobject klopt zodra de importer de zones accepteert.

---

## 6. Gefaseerd implementatieplan

### V2a — Importer accepteert multi-rekenzone; engine poolt (benaderend)

**Scope / bestanden:**
- `uniec3-import/src/geometry.rs` — verwijder de `unit_rzs.len() > 1`-afwijzing (regel 203-208);
  loop `map_zones` over álle UNIT-RZ (nu `vec![zone]`), map elke RZ → `BengZone` (id/naam/A_g/
  bouwwijze/gevels blijven per zone). **Behoud** de `units.len() > 1`-afwijzing (multi-UNIT).
  Voeg één warning toe: "N rekenzones — gepoolde (indicatieve) BENG, zie V2b".
- `uniec3-import/src/lib.rs:118-125` — `gross_floor_area` = Σ zones (al zo); `residential_subtype`
  uit `zones.first()` blijft acceptabel (woningtype is UNIT-breed).
- `beng/mod.rs:372` — **fix Φ_int**: `derive_internal_gains_woningbouw(a_g_total, 1.0)` i.p.v.
  `first_zone.a_g_m2`. Thermische massa (#6, regel 325) uit een A_g-gewogen of dominante zone
  kiezen i.p.v. blind `first_zone`; documenteer de keuze in de note.
- `beng/mod.rs:387-395` — breid de bestaande "Geometrie-bron"-note uit met een expliciete
  "meerdere rekenzones gepoold tot één; η/τ benaderend (V2b = per-zone)"-regel bij `zones.len() > 1`.

**Acceptatiecriteria:**
- Alle 15 multi-zone-korpusbestanden importeren zonder `Err` (variatie-smoke: 52/52 OK).
- `compute_beng` levert een resultaat met de gepoolde-benadering-note in `notes`.
- BENG 1/2/3 binnen een **ruimere, gedocumenteerde** tolerantie vs certified (niet de strakke
  F8-single-zone-tol); afwijking gemeten en vastgelegd per golden.

**Golden-strategie:** kies **woning-2176** (3 zones, kelder-patroon, recente app 3.3.6) als
multi-zone-golden. Zelfde pad-detectie als de F8-tests (`#[ignore]` + skip-if-absent, klantdata
gitignored). Leg de gepoolde delta t.o.v. certified vast als expliciete baseline; V2b moet die
delta naar de single-zone-tolerantie terugbrengen.

### V2b — Per-rekenzone demand + aggregatie (norm-exact)

**Scope / bestanden:**
- `nta8800_view.rs` / `geometry_bridge.rs` — stop het platslaan: houd de spaces/rekenzones
  gescheiden zodat elke zone een eigen maandbalans kan draaien (of introduceer een per-zone
  demand-lus die de bestaande view per zone aanroept).
- `beng/mod.rs` `compute_beng` — vervang de enkele demand-tak door een **lus over rekenzones**:
  per zone `compute_tojuli_full` met die zone's geometrie + eigen C_m/Φ_int/τ; sommeer
  Q_H;nd/Q_C;nd/Q_C;use maandprofielen (§8.2.2/§10.19). Diensten (heating/dhw/cooling/vent) op de
  **gesommeerde** demand met de ongewijzigde project-brede `EnergyInput` (p.536). A_g = Σ zones.
- TOjuli — per rekenzone bepalen; maatgevende = max over zones (§5.7.2 werkt per rekenzone).

**Aandachtspunten (uit §3/§4):** infiltratie is UNIT-breed (één `INFILUNIT_QV`) → géén per-zone
q_v10 nodig; het drukmodel verdeelt al over het gebouw. Distributieverliezen naar rato A_g (p.286)
— nu forfaitair, controleren of de per-zone-som dit al benadert.

**Acceptatiecriteria:**
- Multi-zone-golden (woning 2176) BENG 1/2/3 binnen de **reguliere F8-tolerantie** (zoals de
  single-zone Aalten-golden), niet de ruimere V2a-benadering-tol.
- Single-zone-goldens (Aalten/Gouda) blijven byte-identiek (de lus met N=1 = bestaand pad).

---

## 7. Openstaande beslissingen voor de PM

1. **V2a shippen of direct door naar V2b?** V2a ontsluit 15 bestanden snel maar met indicatieve
   cijfers (mits luid gemarkeerd, conform huisregel). Advies: V2a shippen als "import + indicatief",
   V2b als de norm-exacte follow-up — de importer-poort en de golden zijn dan al klaar.
2. **Thermische-massa-keuze bij pooling (V2a):** A_g-gewogen gemiddelde vs dominante zone. Advies:
   dominante zone (grootste A_g), documenteren in de note; V2b maakt het per-zone toch exact.
3. **Ticket-splitsing:** F8-V2 (TODO.md:301) opknippen in *multi-rekenzone (dit doc)* en
   *multi-UNIT/appartementen (apart, groter)*.

---

## 8. MZ-V2a opgeleverd (13-07) — import + indicatief

**Gewijzigd:**

| Bestand:regel | Wijziging |
|---|---|
| `crates/uniec3-import/src/geometry.rs` (`map_zones` + nieuwe `map_zone`) | `unit_rzs.len() > 1`-afwijzing weg; loopt over álle UNIT-RZ, één `BengZone` per RZ; multi-UNIT-guard behouden; indicatief-warning bij N > 1 |
| `crates/openaec-project-shared/src/beng/mod.rs` (bridged-tak, ~r319-400) | Φ_int op `a_g_total` (Σ zones) i.p.v. `first_zone.a_g_m2` (§6.6.2); thermische massa uit **dominante** zone (grootste A_g); `INDICATIEF (MZ-V2a)`-note + dominante-zone-vermelding bij `zones.len() > 1` |
| `crates/uniec3-import/tests/synthetic.rs` | CI-fixtures: 2-zone-import (pooled-warning) + multi-UNIT-reject |
| `crates/openaec-project-shared/src/beng/tests.rs` | Φ_int-som-regressie + indicatief-note + single-zone-géén-note |
| `crates/uniec3-import/tests/multizone_golden.rs` + `tests/verification/.../woning-2176/` | Golden (skip-if-absent) + README |

**Woning-2176-golden (3 zones, A_g 435,10):** import GROEN; gepoolde `compute_beng` vs certified:

| | Gepoold | Certified | Δ |
|---|---|---|---|
| BENG 1 | 63,41 | 72,49 | −9,08 |
| BENG 2 | 9,69 | 22,00 | −12,31 |
| BENG 3 | 88,15 | 75,90 | +12,25 |

Deltas zijn **indicatief** (geen tol-assert); MZ-V2b moet ze naar de F8-tolerantie terugbrengen.
De onderschatting is dezelfde familie als de bestaande single-zone gain-utilization-drift
(zie `beng_golden.rs` ignore-reason), plus de pooling-η/τ-benadering.

**Korpus-realiteit — corrigeert de planpremisse:** de smoke-verdeling is **0 multi-UNIT**,
**15 multi-RZ (alle binnen 1 UNIT)**, 37 single-zone. De veronderstelde "3 multi-UNIT" bestaan
niet in het korpus. Aalten/Gouda single-zone zijn byte-identiek (stash-geverifieerd:
B1 64,31/82,98/84,17, B2 15,38 vóór én ná de Φ_int-fix).

## 9. MZ-V2c (13-07) — drijvende woning: water-adjacency

De hersmoke na V2a bracht **47/52 OK**; de 5 uitval waren **geen** multi-zone-regressie maar
**drijvende woningen** (woonark-2248 ×4, drijvende-woning-3003 ×1). Diagnose op het bestand: hun
onderbouw grenst aan **open water**, gecodeerd als `BEGR_VLOER=VL_WATER` (vloer) en
`BEGR_GEVEL=GVL_WATER` (onderwaterlijn-gevel). Die codes kende `map_adjacency` niet:

- de water-**vloer** viel in de `other`-tak → default `VloerOpMaaiveldBovenGrond`, die via de
  P/A-methode een omtrek P eist; op water ontbreekt die → `GeometryValidation` faalde (harde
  weigering);
- de water-**gevel** viel in de oriëntatie-terugval → stil `Buitenlucht{Noord}` (importeerde
  wél, maar fysiek fout: een wand tegen water i.p.v. buitenlucht).

Het model draagt al een `BengAdjacency::Water` (bridge → `BoundaryKind::OpenWater`, telt mee in
A_ls, géén P/A-eis). Fix in `geometry.rs` `map_adjacency`: `VL_WATER` → `Water` (vloer-tak) en
`GVL_WATER` → `Water` (gevel/kelderwand-tak, vóór de oriëntatie-poging). Dit was dus een
**mapping-gap, geen ontbrekende brondata**. Resultaat: **smoke 52/52 OK**. Synthetische
`floating_home_water_floor_and_wall_map_to_water`-test dekt vloer + onderwaterlijn-gevel +
referentie-bovenwaterlijn-gevel (blijft buitenlucht-N).

## 10. MZ-V2b norm-analyse — zone-allocatie van C_m, Φ_int, ventilatie, TOjuli

Norm-verificatie vóór de per-zone-implementatie. Alle citaten uit
NTA 8800:2025+C1:2026 (nl-editie, PyMuPDF-extract; paginanr. = PDF-pagina).

### 10.1 Grondregel: per rekenzone rekenen, dan sommeren (geen pooling)

- **§6.6.2 (p. 158):** *"De totale gebruiksoppervlakte van de thermische zone (A_g;tot) wordt
  bepaald als de som van de gebruiksoppervlakten van alle rekenzones."* → BENG-noemer = Σ A_g;zi.
  Certified-bevestigd: 435,10 = 159 + 117,1 + 159 (woning 2176).
- **§8.2.2 + formule (10.19) (p. 377):** de koudebehoefte van de **thermische zone** `zt,j`
  = Σ over de rekenzones van de per-zone `Q_C;nd;zi;mi` (elke `Q_C;nd;zi` "bepaald zoals in
  8.2.2"). Idem voor `Q_H;nd`. De norm sommeert per-zone-uitkomsten; hij poolt de schil **niet**.

De V2a-pooling (Σ spaces → 1 rekenzone, dan één maandbalans) is dus alleen exact voor de
**lineaire** posten (Σ A·U, A_g, infiltratie-C_lea). De niet-lineaire winstbenutting η (§7.2.2,
formule 7.6: γ = Q_gn/Q_ht, dan η(γ,τ)) en de tijdconstante τ = C_m·A_g/(H_T+H_V) worden bij
pooling over de gecombineerde schil bepaald i.p.v. per zone → de V2a-`INDICATIEF`-afwijking.

### 10.2 Interne warmtewinst Φ_int per zone — de N_woon;zi-sleutel (kern-vondst)

Formule **(7.21) (p. 177)** is expliciet **zi-geïndexeerd**:

> `Q_H/C;int;dir;zi;mi = 180 · N_woon;zi · N_P;woon;zi · 0,001 · t_mi`

met `N_P;woon;zi = f(A_g;zi / N_woon;zi)` (formules 7.22–7.24, piecewise). De vraag was: komt
`N_P` per zone uit die zone's eigen A_g, of uit A_g;tot? Antwoord in **§6.6.6, formule (6.2b)
(p. 160) + OPMERKING 2 (p. 161):**

> `N_woon;zi = N_woon · A_g;zi / Σ A_g;zi`  — *"Indien de woonfunctie is verdeeld in
> verschillende zones, is N_woon;zi kleiner dan 1. Dit is de fractie van de rekenzone in de
> totale woningoppervlakte."*

Voor één woning over N rekenzones (N_woon = 1) geldt dus `N_woon;zi = A_g;zi / A_g;tot`, en
daarmee:

```
x_zi = A_g;zi / N_woon;zi = A_g;zi / (A_g;zi/A_g;tot) = A_g;tot   (voor ELKE zone)
```

**Gevolg:** `N_P;woon;zi = N_P(A_g;tot)` is voor elke zone gelijk (geëvalueerd op de héle
woningoppervlakte), en de per-zone **flux** Φ_int;zi [W/m²]
`= 180·N_woon;zi·N_P / A_g;zi = 180·N_P(A_g;tot)/A_g;tot` is **uniform over alle zones** en gelijk
aan de unit-brede flux die V2a al berekent via `derive_internal_gains_woningbouw(a_g_total, 1.0)`.
Σ_zi Q_int;zi = 180·N_P(A_g;tot)·Σ N_woon;zi = 180·N_P(A_g;tot) — identiek aan de unit-som.

**Implementatie-consequentie:** V2b geeft aan élke zone dezelfde `internal_gains`-flux (uit
A_g;tot). De demand-crate vermenigvuldigt die met de zone-eigen A_g;zi (`Q_int = Φ_int·A_g·t·0,0036`,
`internal_gains.rs:50`) → per-zone Q_int correct, som correct. **Anti-valkuil:** Φ_int naïef
per-zone uit A_g;zi berekenen (zónder de N_woon;zi-fractie) is fout — dat overschat kleine zones
(kelder) fors, want N_P(A_g;zi) ≠ N_P(A_g;tot).

### 10.3 Thermische massa C_m per zone (§7.7)

C_m volgt de bouwwijze en is **per rekenzone** verschillend (§7.7 tabel 7.10/7.11/7.12).
**§6.5 OPMERKING 4 (p. 157)** noemt dit expliciet als splits-drijfveer: *"delen van een gebouw
met een zeer uiteenlopende thermische massa [mogen] niet zonder meer samengenomen worden in één
rekenzone"* (kelder-beton vs. lichte woonlaag = precies het korpus-patroon). V2b leidt C_m per
zone af uit die zone's eigen `bouwwijze_vloer/wand`-codes (`dynamics::derive_thermal_mass`);
ontbrekende/onbekende code → `light_woning()`-default (per zone gemeld). V2a's dominante-zone-C_m
vervalt op dit pad.

### 10.4 Ventilatie + infiltratie: unit-breed, maar per-zone verdeeld

- **Infiltratie:** de importer levert één `q_v10;spec` [dm³/(s·m²)] uit `INFILUNIT_QV`
  (unit-breed, **per m²**). Omdat het een *specifieke* waarde is, geeft toepassing per zone met
  A_g;zi de juiste per-zone-C_lea; Σ = unit-totaal. Geen per-zone q_v10 nodig (plan §6).
- **Ventilatie-forfait:** `q_V;ODA;req` is in de norm **zi-geïndexeerd** (§11.2.2, o.a.
  `q_v;ODA;req;des;zi;mi`, p. 81). Toepassing per zone met A_g;zi is dus norm-conform. De
  woning-ondergrens is per zone **35·N_woon;zi** (formule **(11.64), p. 469**), niet de vlakke 35.
  Voor woning-2176 zijn alle zones > 70 m²: `f_τ = min(0,38+A_g·0,006; 0,8)` zit op de cap 0,8 én
  0,5·A_g;zi ≫ 35 → forfait exact lineair in A_g;zi (Σ = unit). De vlakke `.max(35)` in
  `nta8800_q_v_oda_req_m3_per_h` overschat alleen **zeer kleine** zones (bv. woning-2703 kelder
  4 m²); dat is een **gedocumenteerde restbenadering** op dit pad (zie §10.6), niet relevant voor
  de golden. Signatuurwijziging van `compute_tojuli_full` daarvoor = niet nodig.
- **Gedeelde installaties (VERW/TAPW/VENT/KOEL/PV):** hangen op UNIT-niveau → op de **som**.
  **Tapwater p. 536:** *"Bij toepassing van één warmtapwatersysteem voor de gehele woning [...]
  wordt de nettowarmtebehoefte voor alle rekenzones bepaald en samengenomen."* Distributieverliezen
  naar rato A_g (p. 286). V2b houdt de dienst-keten (heating/dhw/cooling/vent-aux/EP) dus
  ongewijzigd op A_g;tot + de gepoolde `Rekenzone` (unit-volume) — alleen de **demand** wordt per
  zone bepaald en gesommeerd.

### 10.5 TOjuli per zone (§5.7.2)

De TOjuli-toets werkt per rekenzone (formule 5.40 per oriëntatie). V2b bepaalt TOjuli **per zone**
op die zone's eigen schil + eigen `TojuliResult`, en neemt de **maatgevende = max over de zones**
(consistent met de bestaande per-oriëntatie-max). Bij een actief gekoelde zone blijft `TOjuli = 0`.

### 10.6 Restbenaderingen (eerlijk vermeld)

1. **Koudebruggen (Σψ·L)** zitten niet zone-geattribueerd in de BENG-invoer (ze reizen mee uit de
   ruimte-geometrie). V2b verdeelt ze **A_g-proportioneel** over de zones (length·frac); Σψ·L blijft
   exact behouden. Korpus multi-zone-bestanden dragen er geen → nul effect; bij N = 1 → frac = 1
   (identiek).
2. **Ventilatie-ondergrens 35 dm³/s** wordt per zone op de vlakke waarde geknipt i.p.v.
   35·N_woon;zi; alleen merkbaar bij zeer kleine zones (< ~70 m² unit-A_g of een mini-kelder).
3. **Drukmodel-gebouwhoogte** is per zone gelijk (uit unit-`num_storeys`); tweede-orde op de
   infiltratie-verdeling.

### 10.7 Architectuurkeuze (minimaal-invasief)

`compute_beng` krijgt bij `zones.len() > 1` een **per-zone demand-lus**: per zone een sub-`ProjectV2`
(geometrie = alleen die zone's `Space`, `gross_floor_area_m2 = A_g;zi`, eigen C_m, unit-flux Φ_int),
`compute_tojuli_full` per zone, dan de maandprofielen Q_H;nd/Q_C;nd/Q_C;use + H_T/H_V/rencold
**gesommeerd** tot één aggregaat-`TojuliResult`. De bestaande dienst-/EP-/BENG-staart draait
ongewijzigd op dat aggregaat + de gepoolde unit-`Rekenzone`. Bij `zones.len() ≤ 1` loopt exact het
bestaande enkelvoudige pad (N = 1 byte-identiek — geen refactor-risico op Aalten/Gouda). Geen
wijziging aan de service-crates of `compute_tojuli_full`-signatuur.

## 11. MZ-V2b opgeleverd (13-07) — per-zone demand + meting

**Gewijzigd:**

| Bestand:regel | Wijziging |
|---|---|
| `crates/openaec-project-shared/src/beng/mod.rs` (bridging-arm + `ZonePlan` + `compute_demand_multizone`) | Bij `zones.len() > 1`: per-zone `ZonePlan` (sub-geometrie = 1 `Space` + A_g-proportionele koudebruggen, eigen C_m §7.7); demand-lus sommeert Q_H;nd/Q_C;nd/Q_C;use + H_T/H_V/rencold; uniforme Φ_int uit A_g;tot; TOjuli per zone = max. `INDICATIEF (MZ-V2a)`-note → `MZ-V2b (norm-exact)`-note + per-zone-C_m-notes. Single-zone/non-beng = ongewijzigd pad. |
| `crates/uniec3-import/tests/multizone_golden.rs` | Golden woning-2176: assert V2b-note + **BENG 1 binnen F8-tol (±6 %)**; BENG 2/3 gerapporteerd (PV-saldering-normversie, niet geasserteerd). |
| `crates/uniec3-import/tests/variation_smoke.rs` | Corpus-brede `compute_beng`-smoke (`#[ignore]`): 52/52 zonder fouten, 15 multi-zone incl. mini-kelder + drijvende woningen. |
| `crates/openaec-project-shared/src/beng/tests.rs` | `multizone_emits_indicative_note` → `multizone_emits_v2b_note_and_per_zone_cm` (norm-exact-note + per-zone-C_m). |

**Woning-2176-golden (3 zones, A_g 435,10) — V2a-gepoold vs V2b-per-zone vs certified:**

| | V2a gepoold | V2b per-zone | Certified | Δ V2b | F8-tol | Status |
|---|---|---|---|---|---|---|
| BENG 1 | 63,41 | **68,99** | 72,49 | −4,8 % | ±6 % | ✅ **binnen tol** (geasserteerd) |
| BENG 2 | 9,69 | 11,08 | 22,00 | −49,6 % | ±10 % | ⚠️ PV-normversie (niet geasserteerd) |
| BENG 3 | 88,15 | 86,94 | 75,90 | +11,0 pp | ±3 pp | ⚠️ PV-normversie (niet geasserteerd) |

V2b brengt de energiebehoefte-indicator **BENG 1** van V2a's −12,5 % (buiten tol) naar −4,8 %
(**binnen** de reguliere F8-tolerantie) — precies wat het per-rekenzone-rekenen moet leveren. De
per-zone-splitsing verhoogt de behoefte (de gunstige gepoolde winstbenutting tussen kelder en
woonlaag vervalt), wat de V2a-onderschatting corrigeert.

**BENG 2/3 restgap = PV-saldering-normversie, geen multi-zone-fout.** BENG 2 blijft ~−50 % en
BENG 3 ~+11 pp — dezelfde discrepantie als de **single-zone** Aalten/Gouda-goldens (zie
`beng_golden.rs` `#[ignore]`-redenen): NTA 8800:2025+C1 §5.5.2 salderert PV-export **volledig**
tegen fP;exp;el = 1,45, terwijl certified Uniec 3.3.x maar ~64 % crediteert (ouder-norm partieel
salderen). Dat BENG 1 (zuivere demand) wél binnen tol valt terwijl BENG 2 −50 % is, bewijst dat de
restgap in de **installatie-/primair-energie-keten** zit, niet in de per-zone-demand. Anti-fudge:
`expected.json`/`summary.json` onaangeraakt; geen tol-verruiming.

**Byte-identiek N = 1:** Aalten `97,58 / 15,38 / 89,93` (heating 2172 kWh) en Gouda `82,98 / 4,27`
(heating 4777 kWh) — identiek aan de C4+C5a-stand vóór V2b (single-zone loopt het ongewijzigde
pad). **Corpus:** `compute_beng` draait 52/52 zonder fouten (37 single, 15 multi, incl.
woning-2703 4 m²-kelder en de drijvende woningen). `cargo test --workspace` volledig groen.
