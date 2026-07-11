# F3d — Norm-analyse beschaduwing & zonwering (zonwinst formule 7.33)

**Datum:** 2026-07-11 · **Normversie:** NTA 8800:2025+C1:2026 · **Bron-PDF:**
`Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\NTA 8800_2025+C1_2026 nl.pdf`
(paginanummers = PDF-paginalabel rechtsonder; extractie via PyMuPDF, tekst-laag).

Onderbouwt de vervanging van de hardcoded `F_sh = 1,0` in de zonwinst door een
norm-conforme, per-raam beschaduwings-/zonweringsbepaling. F3d-1 levert de
**beweegbare zonwering** (het dominante Q_C-reducerende mechanisme); de externe
belemmering (§17.3) is als restpunt F3d-2 gemarkeerd (zie §5).

---

## 0. Kernbevinding — twee onafhankelijke mechanismen

Formule (7.33) voor de zonwinst door een raam (p. 183) bevat **twee** losse
schaduw-/zonwering-ingangen die de norm strikt gescheiden houdt:

| # | Mechanisme | Symbool | Waar het aangrijpt | Norm | Forfait bij "niets bekend" |
|---|---|---|---|---|---|
| 1 | Externe belemmering (horizon, overstek, zij-/verticale lamellen) | `F_sh;obst;wi,k;mi` | multiplicatief op `I_sol` | §17.3, formule 7.33 | **≠ 1,0** — tabel 17.4 (H) / 17.5 (C), "minimale belemmering" |
| 2 | Beweegbare zonwering (screen, jaloezie, rolluik, uitval-/knikarmscherm) | via `g_gl;wi;mi` | reduceert de effectieve **g-waarde** | §7.6.6.1.4, formule 7.42/7.43 | geen zonwering ⇒ `g` onveranderd |

Cruciaal: **beide zijn asymmetrisch tussen warmte en koeling.**
- `F_sh;obst` heeft aparte tabellen: 17.4 (verwarming) vs 17.5 (koeling).
- Beweegbare zonwering: voor de **warmtebehoefte van woningen** geldt
  `f_sh;with = 0` (§7.6.6.1.4 lid 1, p. 197) — handbediende/juist-ingeregelde
  zonwering wordt bij warmtevraag niet ingezet. Zonwering reduceert dus vrijwel
  uitsluitend de **koel**-/zomerwinst.

Formule 7.33 (p. 183), raamterm:

```text
Q_H/C;sol;wi,k;mi = A_wi,k · (1 − F_fr;wi,k) · g_gl;wi,k;mi · F_sh;obst;wi,k;mi · I_sol;wi,k;mi · t_mi · 0,001
```

---

## 1. Mechanisme 2 — beweegbare zonwering (F3d-1, geïmplementeerd)

### 1.1 Formules (§7.6.6.1.4, p. 196-198)

Maandgemiddelde effectieve zontoetreding met beweegbare zonwering (7.42, p. 196):

```text
g_gl;wi;mi = (1 − f_sh;with;mi) · g_gl;wi + f_sh;with;mi · g_gl;sh;wi          (7.42)
g_gl;sh;wi = F_c · g_gl;wi                                                    (7.43, p. 198)
```

Ingevuld levert dat een **effectieve g-reductiefactor per maand**:

```text
r_mi = (1 − f_sh;with;mi) + f_sh;with;mi · F_c
```

- `F_c` (0..=1): forfaitaire reductiefactor totale ZTA — tabel 7.5 (screens/
  jaloezieën/rolluiken/gemetalliseerde weefsels, p. 199) of tabel 7.6 (uitval-/
  knikarmschermen per oriëntatie, p. 199). Voorbeelden tabel 7.5: buitenscreen
  zwart `F_c = 0,12`, onbekende kleur `0,20`, wit `0,25`; buitenjaloezie
  onbekend `0,10`; rolluik onbekend `0,11`. Op 2 decimalen naar boven afronden.
- `f_sh;with;mi`: gewogen inzetfractie — tabel 7.7 (handbediend, woningbouw,
  schakelcriterium 300 W/m², p. 200) of tabel 7.9 (automatisch, 150 W/m²,
  p. 201). Per oriëntatie (N…NW) × helling (verticaal 90° / schuin 45° /
  horizontaal 0°) × maand. **Nov–feb ≈ 0** voor woningbouw handbediend → geen
  reductie in de winter; piek in de zomer (bv. juli Zuid vert. `0,59`).

Voorwaarden (p. 197): alleen gebouwgebonden zonwering; binnenzonwering alleen
als onlosmakelijk deel van een geregeld klimatiseringssysteem.

### 1.2 Waar in de norm de eigenschap ligt → code-plek

De zonwering wijzigt `g_gl` **per raam** (7.42 werkt op `g_gl;wi`). Daarom:
- DTO: `Opening.movable_shading: Option<MovableSunShading>` (raam-/openingsniveau).
- Model: `nta8800_model::Window.movable_shading` + `MovableSunShading { f_c, control }`
  + `ShadingControl { ManualResidential, Automatic }`.
- Reken: `nta8800-demand::calc::shading` bevat de tabellen 7.7/7.9 en
  `movable_shading_g_factor()`; `solar_gains::monthly_solar_gains` past de
  maand-factor per raam toe. Default (geen zonwering) ⇒ factor 1,0 in elke maand
  ⇒ byte-identiek aan het gedrag vóór deze wijziging.

### 1.3 Bewuste V1-benadering (single-Q_sol)

De demand-keten voert **één gedeeld `Q_sol`-profiel** dat zowel de warmte- als
de koudebalans voedt (`calc/mod.rs`: `q_gn = Q_int + Q_sol`, gebruikt voor
`heating_demand` én `cooling_demand`). Daardoor kan de norm-regel
"`f_sh;with = 0` voor woning-warmtevraag" nog **niet** exact worden nageleefd:
de zonweringsreductie wordt symmetrisch op beide balansen toegepast.

Dat is acceptabel omdat de tabel-`f_sh;with` zelf ~0 is in nov–feb: het
reductie-effect landt vrijwel volledig in het koelseizoen (fysisch de juiste
vorm). De residuele overreductie van de schouderseizoen-warmtewinst (mrt–okt,
waar `f_sh;with > 0`) is het gedocumenteerde restpunt (§5, F3d-2 — vereist de
splitsing van `Q_sol` in een warmte- en een koel-variant).

---

## 2. Mechanisme 1 — externe belemmering (F3d-2, geanalyseerd, nog niet bedraad)

### 2.1 Formule & bepaling (§17.3, p. 697-708)

`F_sh;obst;wi,k;mi` volgt uit de situatiekeuze a) t/m g) (tabel 17.3, p. 707).
De referentiewoningen/BENG-voorbeeldconcepten hanteren situatie **a) "minimale
belemmering"** (17.3.2a, p. 698): geen belemmeringen > 20° belemmeringshoek,
geen overstekken < 45°. Dit is tevens de altijd-toepasbare conservatieve keuze
voor koeling (tabel 17.3, x=C).

**Belangrijk voor het BENG-anker:** ook bij minimale belemmering is
`F_sh;obst ≠ 1,0`. Tabel 17.4 (verwarming, p. 708) voor een verticaal (90°)
zuidraam:

| Maand | jan | feb | mrt | apr | mei | jun | jul | aug | sep | okt | nov | dec |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| F_sh;obst (Z, 90°) | 0,23 | 0,91 | 1,00 | 1,00 | 1,00 | 1,00 | 1,00 | 1,00 | 1,00 | 0,97 | 0,61 | 0,19 |

De lage winterwaarden (standaard-horizon blokkeert de lage winterzon) → tabel
17.4 **verhoogt** de warmtebehoefte, en is ≈1,0 in de zomer (weinig effect op
koeling). De koeltabel 17.5 (p. 709 e.v.) geeft eigen, hogere zomerwaarden.

### 2.2 Waarom nog niet bedraad

1. **H/C-tabelkeuze onmogelijk in single-Q_sol.** 17.4 (H) en 17.5 (C) zijn
   verschillend; met één gedeeld `Q_sol` is er geen correcte keuze. Symmetrische
   toepassing van 17.4 zou de zomer-koelwinst niet corrigeren, van 17.5 de
   winter-warmtewinst wegsnijden. → vereist de balans-splitsing (F3d-2).
2. **Transcriptie-omvang.** Tabellen 17.4/17.5 zijn 8 oriëntaties × 13
   hellingen × 12 maanden × 2 = ~2500 waarden; disproportioneel voor een
   niet-bedrade factor. Het `Opening.movable_shading`-veld is al toekomstvast;
   een `obstruction`-veld volgt bij F3d-2 samen met de balans-splitsing.

---

## 3. Mapping norm → code (F3d-1)

| Norm | Code |
|---|---|
| formule 7.42/7.43 (g-reductie) | `nta8800-demand::calc::shading::movable_shading_g_factor` |
| tabel 7.7 (handbediend woning) | `shading::FSH_WITH_MANUAL_RESIDENTIAL` |
| tabel 7.9 (automatisch) | `shading::FSH_WITH_AUTOMATIC` |
| `F_c` (tabel 7.5/7.6) | `MovableSunShading.f_c` (caller kiest forfait) |
| `f_sh;with` regime | `ShadingControl { ManualResidential, Automatic }` |
| zonwering op raamniveau | `Opening.movable_shading` → `Window.movable_shading` (identiteits-mapping in `map_window`) |
| whole-zone override (bestaand) | `TojuliFullInputs.shading_factor` — grove blunt-factor, **vermenigvuldigt** met de per-raam factor (voorrang gedocumenteerd op het veld) |

Hellingskolom-keuze (`shading.rs`): ≤ 22,5° → horizontaal, ≤ 67,5° → 45°, anders
verticaal. Norm-interpolatie tussen hellingshoeken = restpunt F3d-2.

---

## 4. Smoke — synthetisch WP-bodem-tussenwoning (`compute_beng`)

Handbediende buitenscreens op beide gevelramen (ZW 12 m², NO 6 m²),
`F_c = 0,20` (onbekende kleur, tabel 7.5), `ShadingControl::ManualResidential`.
PV = 0 kWp (matcht het F2b-anker). RVO-anker: B1 54,8 / B2 29,3 / B3 59 % /
TOjuli 0–1,2 K.

| Scenario | metriek | vóór | na | RVO-anker |
|---|---|---|---|---|
| FreeCooling | B1 (energiebehoefte) | 60,9 | **40,5** | 54,8 |
| FreeCooling | B2 (primair) | 41,8 | **33,4** | 29,3 |
| FreeCooling | B3 (%) | 52,1 | 51,9 | 59 |
| FreeCooling | Q_C;use (kWh) | 1956 | **1207** | — |
| zonder koeling | TOjuli;max (K) | 18,83 | **12,62** | 0–1,2 |

**Duiding.**
- **B2** beweegt correct richting het anker (41,8 → 33,4 vs 29,3): de
  zonwering knijpt de fors overschatte koudebehoefte af — precies het door
  F2b/F3c gesignaleerde motief.
- **B1** (= Q_H;nd + Q_C;nd) schiet dóór onder het anker (40,5 < 54,8). Dat is
  de signatuur van de single-Q_sol-benadering + de agressieve `F_c = 0,20`
  (donker scherm) op grote ZW-beglazing zónder externe belemmering. Een lichter
  forfait (wit `0,25`, uitvalscherm `0,35–0,50`) en de nog ontbrekende §17.3-
  belemmering (die de winter-warmtewinst juist verlaagt) trekken B1 terug omhoog.
- **TOjuli** verbetert fors (18,83 → 12,62 K) maar haalt de 1,2 K nog niet:
  zonwering alleen is onvoldoende. De resterende afstand vergt de §17.3-koeltabel
  (17.5) én de per-oriëntatie-koudebalans-verfijning (F3c-restant / F3d-2).

Conclusie: het mechanisme werkt en beweegt alle drie de indicatoren de goede
kant op; exacte anker-kalibratie vergt realistische per-raam invoer + de F3d-2-
verfijningen.

---

## 5. Restpunten voor F3d-2

1. **Balans-splitsing `Q_sol` → warmte-/koel-variant** in `nta8800-demand`, zodat
   (a) `f_sh;with = 0` voor woning-warmtevraag exact wordt nageleefd en (b) de
   H/C-`F_sh;obst`-tabellen (17.4 vs 17.5) elk op de juiste balans landen.
2. **Externe belemmering §17.3** bedraden: tabellen 17.4 (H) / 17.5 (C)
   "minimale belemmering" + `Opening.obstruction`-veld + mapper. Model/DTO zijn
   hier al op voorbereid (additief patroon identiek aan `movable_shading`).
3. **Norm-interpolatie hellingshoek** voor `f_sh;with` (lineair tussen 90°/45°/0°)
   i.p.v. de huidige bucket-forfait.
4. **`F_c`-forfaittabellen 7.5/7.6** in code (nu levert de caller `f_c` expliciet);
   optioneel een `SunShadingType`-enum met kleur/oriëntatie-lookup.
5. **Goldens activeren (F3d-2):** de `#[ignore]`-F0-goldens blijven ongewijzigd;
   activatie valt buiten F3d-1.
