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

## 2. Mechanisme 1 — externe belemmering (F3d-2, geïmplementeerd)

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

### 2.2 Bevinding transcriptie — 17.5 is triviaal

Bij het uitlezen van de PDF (PyMuPDF, tekstlaag) bleek tabel **17.5 (koeling)**
géén per-oriëntatie/helling-blok te zijn maar één regel:

> **Elke oriëntatie · Elke maand · alle hellingen = 1,00** (p. 715).

Fysisch logisch: bij minimale belemmering blokkeert de standaard-horizon de
**hoge** zomerzon niet, dus de koelwinst wordt niet gereduceerd. Alleen tabel
**17.4 (verwarming)** kent reële waarden (< 1 in de winter, de lage winterzon
wordt wél geblokkeerd). Dit halveert de transcriptie-omvang én maakt de bedrading
eenvoudig: `F_sh;obst` grijpt uitsluitend op de **warmtebalans** aan.

Tabel 17.4 is per oriëntatie 12 maanden × 13 hellingen (90°…0° schuin-omhoog +
105°…180° schuin-omlaag). V1 bedraadt de drie buckets die met de bestaande
`f_sh;with`-bucketing overeenkomen: **90° (verticaal), 45°, 0° (horizontaal)**.
De 0°-kolom is per oriëntatie identiek 1,00 (een plat vlak kent geen
azimuth-afhankelijke horizonblokkering). De schuin-omlaag-kolommen (105°–180°)
en de tussenliggende hellingen (75/60/30/15°) zijn V2 (norm-interpolatie).

**Provenance & verificatie.** Tabel 17.4: PDF p. 708–714 (fysieke pagina-idx
707–713 + kop van 714). Tekstextractie kruisgecheckt met een pixmap-render
(matrix 2,2×) van de Oost-pagina (idx 711): 24 waarden — verticaal, 45° én
horizontaal — één-op-één gelijk aan de tekstextractie. De Zuid-verticaal-rij
`[0,23 0,91 1,00 … 0,97 0,61 0,19]` matcht bovendien de onafhankelijke
hand-transcriptie in §2.1. Tabel 17.5 (uniform 1,00) is direct van p. 715
overgenomen. F_c-forfaits (tabel 7.5/7.6) van p. 199.

### 2.3 Code-plek (F3d-2)

| Norm | Code |
|---|---|
| tabel 17.4 (H, minimale belemmering) | `shading::FSH_OBST_HEATING` |
| tabel 17.5 (C) = 1,00 | impliciet: `obstruction_g_factor(_, _, _, Cooling)` → 1,0 |
| factor `F_sh;obst;mi` per raam | `shading::obstruction_g_factor(obstruction, or, tilt, balance)` |
| belemmering op raamniveau | `Window.obstruction: Obstruction{None,Minimal}` ← `Opening.obstruction` (identiteits-mapping in `map_window`) |
| toepassing in 7.33 | `solar_gains::monthly_solar_gains(.., balance)` — multiplicatief met de zonwering |

**Motivatie tabelplek.** De 17.4-data staan in `nta8800-demand::calc::shading`,
naast de bestaande 7.7/7.9-tabellen — níét in `nta8800-tables`. Beide
beschaduwingsmechanismen (§7.6.6 zonwering, §17.3 belemmering) voeden dezelfde
formule (7.33) op dezelfde code-plek (`solar_gains`); co-locatie houdt het
beschaduwings-domein cohesief en vermijdt het splitsen van verwante forfaits over
twee crates. (Consistent met de bestaande F3d-1-keuze.)

## 2b. Balans-splitsing Q_sol → warmte/koeling (F3d-2 kern)

`calculate_demand` berekent nu **twee** zonwinst-profielen via
`monthly_solar_gains(.., SolarBalance::{Heating,Cooling})`:

| balanstak | beweegbare zonwering | §17.3-belemmering | voedt |
|---|---|---|---|
| `Heating` | **uit** (`f_sh;with = 0`, §7.6.6.1.4 lid 1 woningen) | tabel 17.4 | γ_H → η_H → Q_H;nd |
| `Cooling` | maandprofiel tabel 7.7/7.9 | tabel 17.5 = 1,00 | γ_C → η_C → Q_C;nd |

**Minst-invasieve norm-conforme plek.** De H.7-maandbalans hanteerde al aparte
benuttingsfactoren η_H/η_C; het enige gedeelde punt was `Q_gn = Q_int + Q_sol`.
De splitsing zit daarom precies daar: `Q_int` blijft gedeeld, `Q_sol` splitst in
een warmte- en een koelvariant, elk met zijn eigen γ. Geen wijziging aan de
benuttings-, τ- of koelformules. `DemandBreakdown.monthly_q_sol` rapporteert de
warmtebalans-variant (identiek aan koeling wanneer geen zonwering/belemmering).

**Default byte-identiek:** zonder `movable_shading` én zonder `obstruction` zijn
beide factoren 1,0 in elke maand voor beide takken ⇒ `Q_sol;H = Q_sol;C = Q_sol`
zoals voorheen. Vastgepind in `solar_gains::geen_zonwering_is_identiek_aan_voorheen`
(nu over beide balanstakken).

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

## 4b. Smoke F3d-2 — belemmering + balans-splitsing

WP-bodem-tussenwoning, PV = 0, handbediende buitenscreens `F_c = 0,20` op ZW/NO,
minimale belemmering. Draai: `cargo test -p openaec-project-shared f3d2_smoke --
--ignored --nocapture`.

| Stap | B1 | B2 | B3 | koeling kWh/m² | TOjuli (geen koeling) |
|---|---|---|---|---|---|
| baseline (geen zonwering) | 60,9 | 41,8 | 52,1 % | 22,48 | — |
| screens (split, geen belemm.) | 39,7 | 33,1 | 51,7 % | 13,87 | — |
| **screens + belemmering (F3d-2)** | **41,2** | **33,7** | **52,2 %** | **13,87** | **12,62 K** |
| RVO-anker | 54,8 | 29,3 | 59 % | — | ≤ 1,2 K |

**Duiding.**
- **B1** komt van de belemmering (§17.3 tabel 17.4): 39,7 → 41,2. De splitsing
  haalt de zonwering ván de warmtebalans (screens verhogen Q_H níét meer,
  `f_sh;with=0`), de belemmering verlaagt de winter-warmtewinst juist → netto
  duwt de belemmering B1 richting het anker. Het residu (41,2 vs 54,8) is de
  agressieve `F_c=0,20` op grote ZW-beglazing + het ongekalibreerde synthetische
  fixture (geen RVO-1:1-invoer), níét een normfout.
- **Koeling / B2 / TOjuli** wijzigen niet door de belemmering: tabel 17.5 is
  uniform 1,00 op de koudebalans. TOjuli (12,62 K) blijft dus de F3d-1-waarde;
  het afknijpen van TOjuli vergt de koel-zijdige verfijning (F3c-restant), niet
  §17.3.

## 5. Restpunten voor F3d-3

1. ~~Balans-splitsing `Q_sol` → warmte/koel-variant~~ — **klaar (F3d-2)**, zie §2b.
2. ~~Externe belemmering §17.3 bedraden~~ — **klaar (F3d-2)**: tabel 17.4 (H) in
   `FSH_OBST_HEATING`, 17.5 (C) = 1,00, `Window.obstruction`-veld + mapper.
3. **Norm-interpolatie hellingshoek** (lineair tussen 90°/45°/0°) voor zowel
   `f_sh;with` als `F_sh;obst`, plus de schuin-omlaag-kolommen (105°–180°) en de
   overige belemmerings-varianten (§17.3.4 e.v.: overstek, zijbelemmering,
   hoogtehoek). Nu: bucket-forfait + alleen "minimale belemmering".
4. **`F_c`-forfaittabellen 7.5/7.6** — **klaar (F3d-2)** als `SunShadingType`
   (tabel 7.5) + `awning_f_c` (tabel 7.6); de caller mag `f_c` nog steeds
   rechtstreeks op `MovableSunShading` zetten. Optioneel: de DTO een
   `SunShadingType` laten dragen i.p.v. een kale `f_c`.
5. **Goldens activeren (F3d-3):** de `#[ignore]`-F0-goldens blijven ongewijzigd
   in F3d-2; activatie + anker-kalibratie is F3d-3.
