# C4 — Demand-keten-analyse (heating-gap vs certified)

**Datum:** 2026-07-13
**Werkpakket:** C4 — de oorzaak vinden van de heating-primair-gap die C3 blootlegde
(bij matched thermische massa zit onze verwarming 9–19 % ónder certified Uniec, en
de gap groeit met de massa) en die norm-conform fixen.
**Norm-bron:** `NTA 8800:2025+C1:2026 nl.pdf` — §7.8 (benuttingsfactoren, formules
7.46–7.57), §7.5.2.1 (interne warmtewinst, 7.21–7.24), §7.6 (zonwinst, 7.30–7.44),
§7.9 (rekentemperatuur + niet-continu verwarmen, 7.59–7.78).

---

## Samenvatting

| | Uitkomst |
|---|---|
| **Formule-audit benuttingsketen** | η_H;gn (7.46), a_H (7.51: a_H;0=1,0 / τ_H;0=15), τ (7.57) en γ_H (7.50) zijn **regel-voor-regel norm-correct**. Géén fout in de gain-utilization. |
| **Gevonden oorzaak** | Twee norm-omissies in de **zonwinst** (formule 7.32): (1) invalshoek-correctie `F_w = 0,90` (7.40) ontbrak; (2) hemelstralingsterm `Q_sky` (7.39, §7.6.5) werd niet afgetrokken. Beide blazen de zonwinst op → verwarming te laag én koeling te hoog (de dubbel-signatuur). |
| **Fix** | `crates/nta8800-demand/src/calc/solar_gains.rs` — `g_gl = F_w · g_gl;n` + per-raam `Q_sky` afgetrokken. |
| **Effect (Aalten)** | heating primair 2053 → **2168 kWh** (−19,5 % → −15,0 %); koudebehoefte-demand 1329 → **1036 kWh** (cert 873). Split beweegt correct richting certified. |
| **Restgap** | Blijft > ±10 %. Resterende kandidaten (buiten C4-scope): **opake zonwinst/Q_sky** (formule 7.33 — de demand-keten rekent alleen ramen) en de **onbewezen plafondkolom-massa** (D_m 110 i.p.v. 180, voetnoot c → ~+4pp). |

---

## 1. Formule-audit van de benuttingsketen (§7.8)

Regel-voor-regel getoetst tegen de gerenderde PDF-formules (p. 207–210):

| Code-locatie | Norm | Bevinding |
|---|---|---|
| `utilization::A_0_MONTHLY = 1.0`, `TAU_0_MONTHLY_HOURS = 15.0` | Formule 7.51: `a_H;0 = 1,0`, `τ_H;0 = 15 h` (expliciet in de PDF) | **Correct.** De C2-doc-aanname is bevestigd in de norm. |
| `utilization_heating`: `(1−γ^a)/(1−γ^(a+1))`, γ=1 → `a/(a+1)`, γ≤0 → 1 | Formules 7.46/7.47/7.48/7.49 | **Correct** (incl. de vier `als`-takken). |
| `a_parameter`: `a = a_0 + τ/τ_0` | Formule 7.51 | **Correct.** |
| `monthly_balance::gamma`: `γ_H = Q_gn/Q_ht` | Formule 7.50 | **Correct.** |
| `time_constant_hours`: `τ = (C_m/3600)/(H_tr+H_ve)` | Formule 7.57: `τ_H = (C_m;int;eff/3600)/(H_H;tr(excl.grfl) + H_g;adj + H_ve)` | **Correct** (C_m uit tabel 7.10, formule 7.45). |

**Conclusie:** de dynamica-keten (τ → a → η_H;gn → Q_H;nd) bevat géén formule-fout. De
C3-hypothese "η_H;gn te hoog" is daarmee **weerlegd voor de utilization-formule zelf** —
de over-crediting moet in een van de **invoertermen** (Q_H;gn of Q_H;ht) zitten.

Voor de volledigheid ook getoetst en correct bevonden dat de norm géén
demand-**verhogend** mechanisme bevat dat wij missen: §7.9.4.2 (temperatuurnivellering
woningbouw, formule 7.78) en §7.9.2 (a_H;red, niet-continu verwarmen) **verlagen** juist
de effectieve setpoint/Q_H;ht — verkeerde richting om onze te-lage verwarming te
verklaren, dus geen kandidaat.

---

## 2. Maand-decompositie Aalten @ D_m=180 (bridged, De Bilt)

τ = 27,96 h · a = 2,864 · H_tr = 81,4 · H_ve = 38,4 W/K. Energieën in MJ.

**Vóór C4 (D_m=180, forfaitaire zonwinst zonder F_w/Q_sky):**

| mnd | Q_H;ht | Q_int | Q_sol_H | γ_H | η_H | Q_H;nd | Q_C;ht | Q_sol_C | Q_C;nd |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 5297 | 808 | 318 | 0,21 | 0,991 | 4181 | 6581 | 710 | 0 |
| 4 | 3271 | 782 | 1972 | 0,84 | 0,801 | 1065 | 4513 | 2088 | 346 |
| 7 | 886 | 808 | 1990 | 3,16 | 0,309 | 23 | 2170 | 2066 | 1063 |
| 12 | 4900 | 808 | 219 | 0,21 | 0,991 | 3882 | 6184 | 536 | 0 |
| **jaar** | | | | | | **18485 (5135 kWh)** | | | **4783 (1329 kWh)** |

Diagnose: η_H is fysisch/normatief correct (winter ≈ 0,99 → alle winst benut; zomer
laag). De **Q_sol** is de enige overgebleven verdachte: hij voedt zowel de
verwarmings- (verlagend) als de koudebalans (verhogend). Certified koudebehoefte = 873
kWh; wij zaten op 1329 (+52 %) — hetzelfde te-veel-aan-winst dat de verwarming te laag
duwt.

---

## 3. Gevonden oorzaak — twee norm-omissies in de zonwinst (formule 7.32)

De zonwarmtewinst door een raam is (NTA 8800 §7.6.3, formule 7.32):

```
Q_H/C;sol;wi = g_gl · A_w · (1−F_F) · F_sh;obst · I_sol · 0,001 · t_mi  −  Q_sky;wi
```

Onze `solar_gains.rs` rekende `A_w · g · (1−F_F) · I_sol` — twee termen ontbraken:

### 3a. Invalshoek-correctie F_w (formule 7.40, §7.6.6.1.2)

```
g_gl;wi = F_w · g_gl;n     met F_w = 0,90
```

De ingevoerde `g_value` is de **loodrechte** `g_gl;n` (fixture Aalten: 0,40 = drievoudig
glas, tabel 7.4). De tijdgewogen effectieve zontoetreding is 10 % lager door schuine
inval. Uniec past dit intern toe; wij niet → zonwinst 11 % te hoog.

### 3b. Hemelstraling Q_sky (formule 7.39, §7.6.5)

```
Q_sky;k = F_sky · R_se · U_c · A · h_lr;e · Δθ_sky · 0,001 · t_mi   [kWh]
```

met `h_lr;e = 4,14 W/(m²K)`, `Δθ_sky = 11 K`, `R_se = 0,04 m²K/W` (C.2), `F_sky` uit
§7.6.6.4 (1,0 horizontaal / 0,75 hellend / 0,5 verticaal). Dit langgolvige
warmteverlies naar de koude hemel wordt **van elke raamwinst afgetrokken** en ontbrak
volledig. Voor Aalten ≈ 60 MJ/maand/zone, jaarrond ~200 kWh.

Beide termen verlagen de netto-zonwinst → **verwarming omhoog, koeling omlaag**: exact
de gemeten dubbel-signatuur.

### Implementatie

`crates/nta8800-demand/src/calc/solar_gains.rs`:
- `F_W_GLAZING = 0,90` in het effectief-oppervlak: `A·g·F_w·F_sh·(1−F_F)`.
- `sky_view_factor(tilt)` (§7.6.6.4) + constanten `R_SE`, `H_LR_E`, `DELTA_THETA_SKY_K`;
  per raam per maand `Q_sky` (in MJ via `MONTH_HOURS·WH_TO_MJ`) afgetrokken. Q_sky wordt
  óók afgetrokken bij een oriëntatie zonder klimaatprofiel (hangt niet van I_sol af).
- Beide termen zijn balans-onafhankelijk (gelden op H én C).

**Additiviteits-impact (bewust, gedocumenteerd):** `solar_gains` zit in de demand-crate
die óók de standalone TO-juli-/ISSO-51-callers (`compute_tojuli_full` → Tauri/isso51-api)
voedt. De zonwinst-correcties gelden norm-breed (§7.6 is niet BENG-specifiek), dus dit is
een correcte, gewenste gedragswijziging; **TOjuli-uitkomsten verschuiven mee** (iets
lagere zonwinst → iets lager oververhittingsrisico). Alle bestaande workspace-tests
blijven groen.

### Decompositie ná C4 (Aalten @ D_m=180)

Q_sol_H jan 318→204, jul 1990→1708; jaar-Q_H;nd 5135 → **5422 kWh**, jaar-Q_C;nd 1329 →
**1036 kWh**.

---

## 4. Meetmatrix vóór/ná C4 (bridged, % = afwijking t.o.v. certified)

**Aalten-2522** — cert BENG1 103,69 / BENG2 24,71 / BENG3 85,0 / heat 2551 / cool-demand 873 kWh:

| grootheid | C3 (vóór C4) | C4 (F_w + Q_sky) | cert | tol | status |
|---|---:|---:|---:|---:|---|
| BENG 1 | 96,47 (−7,0 %) | 96,39 (−7,0 %) | 103,69 | ±6 % | rood (≈ongewijzigd; split-neutraal) |
| BENG 2 | 15,48 (−37,4 %) | 14,73 (−40,4 %) | 24,71 | ±10 % | rood (aggregaat, te lage totaalvraag) |
| BENG 3 | 89,6 (+4,6pp) | 90,30 (+5,3pp) | 85,0 | ±3pp | rood |
| **heat primair [kWh]** | **2053 (−19,5 %)** | **2168 (−15,0 %)** | 2551 | ±10 % | rood, **maar +4,5pp dichter** |
| koudebehoefte-demand [kWh] | 1329 (+52 %) | **1036 (+19 %)** | 873 | — | **−33pp dichter** |
| koeling primair [kWh] | 749 | 584 (+38 %) | 422 | — | dichter |

**Gouda-2467** — cert BENG1 95,86 / BENG2 27,48 / BENG3 83,7 / heat 6506 kWh:

| grootheid | C3 (vóór C4) | C4 | cert | status |
|---|---:|---:|---:|---|
| BENG 1 | 83,18 (−13,2 %) | 81,80 (−14,7 %) | 95,86 | rood (aggregaat) |
| BENG 2 | 4,44 (−83,8 %) | 3,61 (−86,9 %) | 27,48 | rood (PV-normversie, F3d-8) |
| heat primair [kWh] | 4505 (−30,8 %) | 4761 (−26,8 %) | 6506 | rood, **+4pp dichter** |
| koeling primair [kWh] | 1591 | 1224 | 244 | dichter |

**Lezing:** de fix corrigeert de verwarming/koeling-**split** (beide richting certified,
in de fysisch juiste richting), maar is bij benadering **BENG-neutraal in het totaal**
(minder zonwinst = meer verwarming − minder koeling, dat middelt uit). De aggregaat-gap
(BENG 1/2 te laag) wordt dus **niet** door C4 gedicht — die wordt gedragen door een te
lage **totale** energiebehoefte.

---

## 5. Restgap — resterende kandidaten (buiten C4-scope)

1. **Opake zonwinst + opake Q_sky (formule 7.33).** De demand-keten rekent zonwinst
   **alleen voor ramen**; formule 7.33 kent ook een (kleine) zonwinst én een (grotere)
   `Q_sky` voor niet-transparante vlakken. Netto is dat voor een goed-geïsoleerd dak/gevel
   een **verlies** (Q_sky > zon-absorptie) → zou verwarming verder omhoog en koeling verder
   omlaag duwen. Schatting Aalten: dak F_sky=1,0 ~65 MJ/maand + gevels ~60 MJ/maand →
   ~300–400 kWh/jr extra netto verlies → heating ~+120 kWh primair. **Vereist een
   signatuurwijziging** van `calculate_demand` (opake vlakken meegeven), raakt de
   TO-juli-/ISSO-51-callers — een eigen werkpakket.
2. **Plafondkolom-massa (voetnoot c, D_m 110 i.p.v. 180).** C3 koos de open-plafond-kolom
   (D_m=180, voetnoot b default woningbouw). Aalten heeft echter een **zware
   beganegrondvloer (massief beton) onder lichte verdiepingsvloeren** — precies de
   voetnoot-c-situatie → gesloten-plafond-kolom **D_m=110**. Bij D_m=110 zit BENG 1 op
   +0,4 % (C3-matrix) en heating ~4pp gunstiger; gecombineerd met C4 zou heating naar
   ~−11 % gaan. Uniec exposeert D_m niet, dus onbewijsbaar — dit is een **C3/invoer-vraag**,
   niet de demand-keten.
3. **Q_H;ht (transmissie/ventilatie).** Bij D_m=55 matchte heating op −4,2 %; het
   massa-onafhankelijke deel is dus klein. Geen aanwijzing voor een systematische
   loss-onderschatting, maar niet uitgesloten als tweede-orde-bijdrage.

**Niet-fudge-verklaring:** C4 heeft één concrete, norm-referenteerbare fout gevonden en
gefixt (zonwinst-omissies 7.40 + 7.39). De resterende gap is eerlijk gemeten en toegewezen
aan bovenstaande kandidaten; `expected.json`/`input.json` en de toleranties zijn
onaangeraakt.

---

## 6. Teststatus

`cargo test --workspace`: **volledig groen** (0 failed). Gewijzigd:
- `nta8800-demand::calc::solar_gains` — nieuwe constanten (F_w, R_se, h_lr;e, Δθ_sky) +
  `sky_view_factor`; 7 bestaande unit-tests bijgewerkt naar de netto-formule (bruto −
  Q_sky), nieuwe helper `q_sky_mj`. 80 tests groen.
- `beng_golden.rs` — `#[ignore]`-redenen van `aalten_beng_geometry_heating_matches_certified`
  en `..beng2_matches_certified` bijgewerkt met de gemeten C4-stand. `expected.json`
  onaangeraakt.
