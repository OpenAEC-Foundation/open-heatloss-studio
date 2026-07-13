# C3 — Norm-analyse thermische massa + interne warmtewinst (gevel-BENG-keten)

**Datum:** 2026-07-13
**Werkpakket:** C3 — de twee resterende, aan de verwarming/koeling **gekoppelde**
invoerposten van de gevel-georiënteerde BENG-keten
(`compute_beng` → brug → `compute_tojuli_full` → `nta8800-demand`) norm-conform
maken:
- **C3a** — thermische massa: bouwwijze-codes → `C_m;int;eff;zi` (i.p.v. het
  hardcoded `ThermalMassInput::light_woning()`).
- **C3b** — interne warmtewinst woningbouw: formule 7.21 (i.p.v. het forfait
  3 W/m², `InternalGains::forfaitair`).

**Norm-bron:** `NTA 8800:2025+C1:2026 nl.pdf` — §7.5.2.1 (interne warmtewinst
woningbouw, formules 7.21–7.24, p. 176-177), §7.7 + tabel 7.10/7.11/7.12
(effectieve interne warmtecapaciteit, formule 7.45, p. 204). Beide takken raken
uitsluitend de BENG/TO-juli-keten; de ISSO 51/53-warmteverlies-tak gebruikt ze
niet en blijft ongemoeid. `beng/zeb.rs` onaangeraakt.

---

## Uitgangspunt: waar de twee posten in de keten zitten

`compute_tojuli_full` (`crates/openaec-project-shared/src/tojuli.rs`) draaide tot
C3 twee hardcoded defaults:

| Regel (vóór) | Post | Default |
|---|---|---|
| `let internal_gains = InternalGains::forfaitair(usage);` | Φ_int | woonfunctie 3 W/m² (tabel 7.6) |
| `let thermal_mass = ThermalMassInput::light_woning();` | C_m | `D_m = 55` (licht/licht/gesloten) |

De bouwwijze-codes (`BengZone.bouwwijze_vloer`/`bouwwijze_wand`) en A_g
(`BengZone.a_g_m2`) dragen de norm-invoer al, maar gingen bij de brug
(`beng_geometry_to_shared` → `SharedGeometry`) verloren — `SharedGeometry` codeert
geen bouwwijze. C3 leidt beide posten in `compute_beng` af uit de eerste
`BengZone` en geeft ze via twee **optionele** velden op `TojuliFullInputs` door;
`compute_tojuli_full` valt op de oude defaults terug wanneer ze `None` zijn.

**Additiviteit:** de afleiding gebeurt **alleen** in de bridged BENG-tak (aanwezig
`beng_geometry`-blok). Zonder dat blok blijven `thermal_mass`/`internal_gains`
`None` → `light_woning()`/`forfaitair` → byte-identiek gedrag. De standalone
`compute_tojuli_full`-callers (Tauri/isso51-api) krijgen de velden via
serde-default `None` en zijn dus onveranderd.

---

## C3a — bouwwijze → C_m (NTA 8800 §7.7, tabel 7.10/7.11/7.12)

Tabel 7.10 (p. 204, geverifieerd tegen de PDF) geeft `D_m;int;eff;zi` [kJ/(m²·K)]
per (vloer-massaklasse × wand-massaklasse × plafondkolom). Deze lookup is al
geïmplementeerd en getest in `nta8800_tables::thermal_capacity`
(`specific_heat_capacity` / `zone_heat_capacity`, formule 7.45). C3a levert enkel
de **klasse-afleiding uit de twee Uniec-bouwwijze-codes** + de plafondkolom-keuze.

### Uniec-code → massaklasse

Uniec `RZ_BOUWW_VL` (vloer) / `RZ_BOUWW_W` (wand) — optielabels uit de capture
(`uniec_fields_capture.json`, veld `RZ_BOUWW_VL`/`RZ_BOUWW_W`); de codes uit de
ruwe walk-dump (`fields.json`, `value`-veld). Confirmed = code+label samen in de
capture; de labels mappen 1-op-1 op de norm-massaklassen (tabel 7.11/7.12).

**Vloer (`FloorMassClass`)**

| Uniec-code | Label (tabel 7.11) | Klasse | Provenance |
|---|---|---|---|
| `CONSTRM_FL_11` | hsb, sfb, schuimbeton of hout (licht) | `Light` | label uit capture; code = +5-nummering |
| `CONSTRM_FL_16` | geïsoleerd aan binnenzijde (licht) | `Light` | label uit capture; code = +5-nummering |
| `CONSTRM_FL_21` | staal-beton, hout-beton of niet-massief beton (zwaar) | `Heavy` | **confirmed (Gouda-2467)** |
| `CONSTRM_FL_26` | massief beton (zeer zwaar) | `VeryHeavy` | **confirmed (Aalten-2522)** |
| `CONSTRM_FL_31` | eigen waarde Cm;int;eff - bijlage B | — (`None`) | bijlage-B-pad, niet ondersteund |

**Wand (`WallMassClass`)**

| Uniec-code | Label (tabel 7.12) | Klasse | Provenance |
|---|---|---|---|
| `CONSTRM_W_11` | hsb, sfb of staalskeletbouw (licht) | `Light` | **confirmed (Aalten+Gouda)** |
| `CONSTRM_W_16` | geïsoleerd aan binnenzijde (licht) | `Light` | label uit capture; code = +5-nummering |
| `CONSTRM_W_21` | dragend metselwerk (zwaar) | `Heavy` | label uit capture; code = +5-nummering |
| `CONSTRM_W_26` | betonnen kolom-ligger skeletbouw (zwaar) | `Heavy` | label uit capture; code = +5-nummering |
| `CONSTRM_W_31` | betonnen wand-vloer skeletbouw (zeer zwaar) | `VeryHeavy` | label uit capture; code = +5-nummering |
| `CONSTRM_W_36` | eigen waarde Cm;int;eff - bijlage B | — (`None`) | bijlage-B-pad, niet ondersteund |

De numerieke suffix van de niet-confirmed codes is afgeleid uit de optie-volgorde
(de gecapturede codes zijn `_11`, `_21`, `_26` → +5-stappen). De **klasse** zelf is
in elk geval eenduidig uit het norm-label (tabel 7.11/7.12), alleen de code-suffix
is inferentie. Onbekende of `eigen waarde`-code → `None` → terugval op de default
`light_woning()` (gedocumenteerd, geen stille zware aanname).

### Plafondkolom (voetnoot a/b/c van tabel 7.10)

- **Woningbouw** → default kolom *"geen of open plafond"* = `CeilingType::OpenOrNone`
  (voetnoot b, p. 204).
- **Utiliteitsbouw** → default kolom *"gesloten of verlaagd plafond"* =
  `CeilingType::ClosedOrSuspended` (voetnoot a).
- **Voetnoot c** (woningbouw, bovenzijde vloer zwaarder dan onderzijde vloer
  erboven → gesloten kolom) wordt **niet** geëvalueerd: het BENG-DTO codeert geen
  per-verdieping-vloerconstructie. Gedocumenteerde vereenvoudiging; spiegelt Uniec,
  dat met de enkelvoudige bouwwijze-selectie de default-kolom aanhoudt.

### Resultaat per fixture

| Case | vloer-code → klasse | wand-code → klasse | plafond | D_m [kJ/(m²·K)] |
|---|---|---|---|---:|
| Aalten-2522 | `CONSTRM_FL_26` → VeryHeavy | `CONSTRM_W_11` → Light | open (woning) | **180** (groep 2) |
| Gouda-2467 | `CONSTRM_FL_21` → Heavy | `CONSTRM_W_11` → Light | open (woning) | **180** (groep 2) |

Beide fixtures komen op D_m = 180 (tegen de default 55). Diagnostisch (C2-doc): de
`light_woning`-default (55) zette Aalten BENG 1 op +11,2 %; `zwaar_massief` (450) op
−14,5 % maar brak de heating-anchor. De norm-afleiding (180) ligt ertussen — hogere
C_m → hogere τ → hogere η_C;ht → lagere Q_C;nd, en hogere benuttingsgraad → lagere
Q_H;nd. Verwachte richting: **BENG 1 omlaag** t.o.v. de +11,2 %-stand.

---

## C3b — interne warmtewinst woningbouw (NTA 8800 §7.5.2.1, formule 7.21)

Geverifieerd tegen de PDF (p. 176-177, formule-afbeelding gerenderd):

```
Q_H/C;int;dir;zi;mi = 180 · N_woon;zi · N_P;woon;zi · 0,001 · t_mi     [kWh]   (7.21)
```

met het aantal bewoners per woonfunctie `N_P;woon;zi` uit de gemiddelde
gebruiksoppervlakte per woning `x = A_g;zi / N_woon;zi`:

| Band | N_P;woon | Formule |
|---|---|---|
| `x ≤ 30 m²` | `1` | (7.22) |
| `30 < x ≤ 100 m²` | `2,28 − 1,28/70 · (100 − x)` | (7.23) |
| `x > 100 m²` | `1,28 + 0,01 · x` | (7.24) |

> **Correctie op de opdracht-schatting:** de team-lead noemde voor Aalten
> "N_P ≈ 1,95". Dat is de uitkomst van de **>100**-formule (7.24: 1,28 + 0,01·67).
> Aalten (A_g = 67, N_woon = 1) valt echter in de **30–100**-band → formule 7.23:
> `N_P = 2,28 − 1,28/70·(100 − 67) = 1,677`. De gerenderde PDF-formule bevestigt
> deze bandtoewijzing ondubbelzinnig.

`N_woon` = aantal woonfuncties in de rekenzone (§6.6.7). Beide fixtures zijn één
grondgebonden woning (`woningtype = TWON_VRIJ_K` = vrijstaand met kap) → **N_woon =
1** (provenance: de `woningtype`-capture). Meervoudige N_woon (appartementgebouw)
is V2.

### Omzetting naar het bestaande `InternalGains`-model (W/m², constant)

`InternalGains` draagt een maandprofiel Φ_int in **W/m²**; de demand-crate rekent
`Q_int;mi = Φ_int · A_g · t_mi · 0,0036` [MJ]
(`calc/internal_gains.rs`). Gelijkstellen aan formule 7.21 (kWh → MJ ×3,6):

```
Φ_int · A_g · t_mi · 0,0036 = 180 · N_woon · N_P · 0,001 · t_mi · 3,6
                            = 180 · N_woon · N_P · 0,0036 · t_mi
→ Φ_int = 180 · N_woon · N_P / A_g          [W/m²]   (t_mi valt weg → constant)
```

De maandlengte `t_mi` valt weg omdat de demand-crate hem opnieuw aanbrengt; de
maandlengte-tabel (`MONTH_HOURS`, Σ = 8760 h) is identiek aan §17.2. Zo
reproduceert een **constant** Φ_int = 180·N_woon·N_P/A_g formule 7.21 exact door de
gevalideerde maandbalans, zonder een tweede rekenpad. Dezelfde Φ_int voedt de
verwarmings- én de koudebalans (formule 7.21 is `Q_H/C;int` — één winst voor beide).

### Resultaat per fixture

| Case | x = A_g/N_woon | band | N_P;woon | Φ_int = 180·N_P/A_g [W/m²] | vs forfait 3,0 |
|---|---:|---|---:|---:|---:|
| Aalten-2522 | 67,00 | 7.23 | 1,677 | **4,50** | +50 % |
| Gouda-2467 | 133,06 | 7.24 | 2,611 | **3,53** | +18 % |

Hogere interne winst → **Q_H;nd omlaag** (meer gratis warmte) én **Q_C;nd omhoog**
(meer af te voeren warmte in de zomer). Beide fixtures krijgen dus een lagere
verwarmings- en een hogere koudebehoefte; de heating-anchor (Aalten 2444 vs
certified 2551 kWh) en de Gouda-koeling verschuiven tegengesteld. De certified
Uniec-tool rekent óók met formule 7.21, dus de verwachte netto-beweging is
**richting** certified.

---

## Meet-strategie (gekoppelde balansen)

Beide correcties verschuiven verwarming én koeling tegengesteld; ze worden daarom
samen geïmplementeerd en de volledige matrix wordt vóór/na gemeten:
C3a-alleen, C3b-alleen, C3a+C3b samen — telkens Aalten BENG 1/2/3 + verwarming
primair en Gouda BENG 1 + koeling primair. De meetmatrix staat hieronder
(niet gefudged: een anker dat uit tolerantie loopt = bevinding, geen terugdraaiing).
Fixtures `expected.json`/`input.json` blijven onaangeraakt.

---

## Meetmatrix (compute_beng; % = afwijking t.o.v. certified)

**Aalten-2522** — cert BENG1 = 103,69, BENG2 = 24,71, BENG3 = 85,0, heat = 2551,
cool = 422 kWh:

| variant | BENG1 | BENG2 | BENG3 | heat prim | cool prim |
|---|---:|---:|---:|---:|---:|
| baseline (D_m 55, forfait 3,0) | 115,27 (+11,2 %) | 23,68 (−4,2 %) | 86,0 | 2444 (−4,2 %) | 907 |
| C3a massa open (D_m 180) | 99,19 (−4,3 %) | 15,90 (−35,7 %) | 89,8 | 2234 (−12,4 %) | 596 |
| C3a massa gesloten (D_m 110) | 106,20 (+2,4 %) | 19,31 (−21,8 %) | 88,0 | 2322 (−9,0 %) | 737 |
| C3b winst alleen (4,50 W/m²) | 114,06 (+10,0 %) | 23,94 (−3,1 %) | 85,5 | 2291 (−10,2 %) | 1077 |
| **C3a+C3b open (D_m 180) — shipped** | **96,47 (−7,0 %)** | **15,48 (−37,4 %)** | **89,6** | **2053 (−19,5 %)** | **749** |
| C3a+C3b gesloten (D_m 110) | 104,13 (+0,4 %) | 19,19 (−22,4 %) | 87,7 | 2153 (−15,6 %) | 898 |

**Gouda-2467** — cert BENG1 = 95,86, BENG2 = 27,48, BENG3 = 83,7, heat = 6506,
cool = 244 kWh:

| variant | BENG1 | BENG2 | heat prim | cool prim |
|---|---:|---:|---:|---:|
| baseline | 96,83 (+1,0 %) | 11,99 (−56,4 %) | 5131 (−21,1 %) | 1969 |
| C3a massa open (D_m 180) | 83,41 (−13,0 %) | 4,53 (−83,5 %) | 4672 (−28,2 %) | 1436 |
| C3a massa gesloten (D_m 110) | 88,96 (−7,2 %) | 7,62 (−72,3 %) | 4852 (−25,4 %) | 1667 |
| C3b winst alleen (3,53 W/m²) | 97,98 (+2,2 %) | 12,67 (−53,9 %) | 4990 (−23,3 %) | 2201 |
| **C3a+C3b open (D_m 180) — shipped** | **83,18 (−13,2 %)** | **4,44 (−83,8 %)** | **4505 (−30,8 %)** | **1591** |
| C3a+C3b gesloten (D_m 110) | 89,28 (−6,9 %) | 7,84 (−71,5 %) | 4695 (−27,8 %) | 1852 |

## Bevinding — de correcties leggen een demand-keten-fout bloot (buiten C3-scope)

De norm-correcte dynamica verlaagt Q_H;nd fors, maar **certified Uniec houdt de
verwarming juist hóóg** (2551 kWh) terwijl het zélf verplicht met een hogere massa
(D_m ∈ {110, 180}) en formule 7.21 rekent. Bij **gelijke massa** zit onze keten
9–12 % onder certified op de verwarming:

| certified D_m-aanname | onze heat @ die D_m (C3a) | certified heat | gap bij matched mass |
|---|---:|---:|---:|
| 110 (voetnoot c) | 2322 | 2551 | −9,0 % |
| 180 (voetnoot b) | 2234 | 2551 | −12,4 % |

Dat de gap **groeit** met de massa (baseline −4,2 % → C3a −12,4 %) wijst op een te
sterke gain-utilization (η_H;gn) of een te lage Q_H;ht in `nta8800-demand`: onze keten
crediteert de zonne-/interne winst agressiever dan certified. De oude
`light_woning`/forfait-defaults (minimale massa + laagste winst) minimaliseerden die
crediting en verborgen zo de gap → de C2-groene ankers waren een compensatie-artefact,
geen fysische match. **Ceiling-gevoeligheid:** voetnoot c (D_m 110) verschuift het
resultaat ~4 pp gunstiger maar sluit de gap niet; welke plafondkolom Uniec intern
kiest is niet uit de capture af te lezen (Uniec exposeert D_m niet). Vervolg =
demand-keten-analyse (`nta8800-demand::calc::time_constant` + utilization tegen
NTA 8800 §7.2.1.1), buiten C3-scope.

## Onderbouwing "certified past formule 7.21 + tabel 7.10 toe"

- Uniec 3 is een **BCT-gecertificeerde NTA 8800-tool** (de fixture is een echt Uniec
  3.3.x-certificaat). Formule 7.21 (§7.5.2.1, verplicht voor de woonfunctie) en
  tabel 7.10 (§7.7) zijn **verplichte** rekenstappen — er is voor woningbouw geen
  alternatieve interne-winst-route, en de warmtecapaciteit móét uit tabel 7.10 komen
  tenzij bijlage-B (eigen waarde) is gekozen.
- De capture bevestigt dat Uniec **niet** de bijlage-B-route gebruikt: `RZ_BOUWW_VL` =
  "massief beton (zeer zwaar)" en `RZ_BOUWW_W` = "hsb…(licht)" zijn tabeloptie-
  selecties, geen "eigen waarde Cm;int;eff - bijlage B". Die inputs vallen in tabel 7.10
  op D_m = 180 (open) of 110 (gesloten).
- **Grens van de claim:** Uniec exposeert de gekozen plafondkolom en de resulterende
  D_m niet in de walk, dus of het exact 180 of 110 is, is niet direct bewezen — wél dat
  het ≫ 55 (onze oude default) is. De bevinding hangt niet van die keuze af: bij beide
  waarden zit onze verwarming bij matched mass onder certified.
