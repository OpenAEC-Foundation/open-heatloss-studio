# F3c — Norm-analyse TOjuli per oriëntatie (§5.7.2) + t_juli (§17.2)

**Datum:** 2026-07-11 · **Normversie:** NTA 8800:2025+C1:2026 · **Bron-PDF:**
`Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\NTA 8800_2025+C1_2026 nl.pdf`
(paginanummers hieronder = PDF-paginalabel rechtsonder op de pagina; extractie via
PyMuPDF, tekst-laag).

Deze notitie onderbouwt de vervanging van de **whole-zone screening** (F3-stub) door de
norm-conforme **per-oriëntatie**-bepaling van TOjuli, zodat de BENG-keten zónder actieve
koeling een pass/fail kan uitspreken. Ook fixeert ze de exacte `t_juli`-waarde.

---

## 0. Kernbevinding (samenvatting)

| Vraag | Antwoord | Bron |
|---|---|---|
| Wordt TOjuli per oriëntatie bepaald? | Ja — formule (5.40), per rekenzone én per oriëntatie; maatgevend = max over oriëntaties | §5.7.2, p. 115; Bbl 4.149b lid 1 |
| Welke oriëntatie-set? | 8 kompasrichtingen N, NO, O, ZO, Z, ZW, W, NW (géén "horizontaal") | Stap A, p. 116 |
| Wat splitst per oriëntatie? | A_T (Stap A), Q_C;nd;juli (Stap B), H_C;D;vert (Stap 1/2/5), zonwinst (Stap 1/2/5); **pro-rata naar A_T:** H_C;ve, H_gr;an, H_C;D;hor (Stap 3/4), interne winst, C_m;eff | Stap A/B + Stap 1-5, p. 116-118 |
| Wat is de maatgevende waarde? | max_or TOjuli;or,zi; getoetst per oriëntatie tegen 1,20 K | §5.7.2 + Bbl 4.149b lid 1 |
| t_juli? | **744 h** (juli), exact uit tabel 17.1 | §17.2, tabel 17.1, p. 690 |
| Kleine oriëntaties | A_T;or ≤ 3 m² → oriëntatie buiten beschouwing | Stap A + OPMERKING 3, p. 116/120 |
| Afronding | naar boven op veelvoud 0,01 | p. 120 |

---

## 1. Toepasselijkheid (§5.7.1, p. 114-115)

TOjuli hoeft **alleen** te worden bepaald voor rekenzones **zonder** actief koelsysteem van
voldoende capaciteit. Bij een actief koelsysteem (tabel 10.29/10.30/10.34 excl.
dauwpunts-ETA-koeling, WP met actieve koeling, externe koude, split-units) mag voor alle
oriëntaties `TOjuli;or,zi = 0` en is de zone geacht te voldoen. Dit is de reeds werkende
`ActivelyCooled`-shortcut — RVO verwacht bij de WP-goldens TOjuli = 0 (actieve koeling).

---

## 2. Formule (5.40) en de per-oriëntatie-opdeling (§5.7.2, p. 115-119)

### 2.1 Basisformule (p. 115)

```text
                (Q_C;nd;juli;or,zi − Q_C;HP;juli;or,zi) × 1000
TOjuli;or,zi = ────────────────────────────────────────────────────────────
               (H_C;D;juli;or,zi + H_gr;an;juli;or,zi + H_C;ve;juli;or,zi) × t_juli
```

Eenheid K; `TOjuli;or,zi ≥ 0`. Dit is reeds correct getranscribeerd in
`nta8800-ep/src/tojuli.rs::tojuli_orientation` (F1b) — die module blijft ongewijzigd.

### 2.2 Stap A — A_T;or,zi (formule 5.41, p. 116)

`A_T;or,zi` = som van de **geprojecteerde oppervlakten van de uitwendige
scheidingsconstructies** (naar buitenlucht/serre) **per oriëntatie**. Uitgesloten:
elementen naar AOR/AVR/grond/kruipruimte/water. **Horizontale** elementen worden **niet**
in deze som meegenomen — die worden apart beschouwd en **naar rato over de oriëntaties
verdeeld** (Stap 3/4). Oriëntaties: N, NO, O, ZO, Z, ZW, W, NW. Voor `A_T;or,zi ≤ 3 m²`
vervalt de bepaling (kleine geveldelen/dakkapel-wangen — OPMERKING 3, p. 120).

**Horizontaal vs oriëntatiegebonden — hellingsdrempel (randgeval schuin dak).**
§5.7.2 zegt "Horizontale elementen … naar rato over de oriëntaties verdeeld", maar
definieert "horizontaal" daar niet. De operationele definitie staat in **§7.6.6.4
(Vormfactor, p. 203):** een "horizontale constructie" is een vlak *"waarvan de hellingshoek
met de horizontaal kleiner is dan of gelijk is aan 5°"*. Gevolg voor de opdeling:

| Element | Helling t.o.v. horizontaal | Bucket |
|---|---|---|
| Verticale gevel | 90° | oriëntatiegebonden (Stap 2/5) |
| **Hellend dakvlak mét azimuth** (bv. 45° zuid) | > 5° | **oriëntatiegebonden** (Stap 2/5) — telt in `A_T;Zuid`, `H_C;D;Zuid`, zonwinst-Zuid |
| Plat dak / vloer | ≤ 5° | overig/horizontaal (Stap 3/4), pro-rata `A_T;or` |

Een **schuin dakvlak is dus géén pro-rata-element**: alleen (bijna-)platte vlakken (≤ 5°)
gaan naar de pro-rata-pool. De klassering hangt aan de **helling** (`slope_deg`), niet aan
`kind`. Een zuidgericht dakvlak draagt zo bij aan het oververhittingsrisico van de
zuid-oriëntatie (fysisch correct; dekt tevens een eerdere schuin-dak-blinde-vlek af).
Geïmplementeerd via `tojuli_orientation_bucket` + `HORIZONTAL_TILT_MAX_DEG = 5,0`.

### 2.3 Stap B — Q_C;nd;juli;or,zi (p. 116-117)

De koudebehoefte juli per oriëntatie volgens 7.2.2, waarbij de warmtebalans-componenten
per oriëntatie worden opgedeeld:

| Component | Opdeling per oriëntatie |
|---|---|
| Zonwinst Q_C;sol;juli;or,zi | toegewezen aan de **werkelijke** oriëntatie (Stap 1/2/5 vert. + Stap 3/4 hor. pro-rata) |
| H_C;D;juli (transmissie excl. begane-grondvloer) | vert.: **werkelijke** oriëntatie (Stap 2); hor.: **pro-rata A_T** (Stap 3/4); Stap 5 = som |
| H_C;ve;juli (ventilatie) | **pro-rata A_T** |
| H_gr;an;juli (grond) | **pro-rata A_T** |
| H_C;p (verticale leidingen) | op **nul** gesteld |
| Q_C;int, C_m;int;eff, terugwinbare verliezen | **pro-rata A_T** |

### 2.4 Stap 1-5 — H_C;D en zonwinst (p. 116-118)

- **Stap 1:** per element k → H_C;D;juli;k en Q_C;sol;juli;k.
- **Stap 2 (oriëntatiegebonden):** som per oriëntatie van de verticale/gevel-elementen op die
  oriëntatie → H_C;D;juli;vert;or,zi en Q_C;sol;juli;vert;or,zi.
- **Stap 3 (overig/horizontaal):** som van de horizontale elementen → H_C;D;juli;hor;zi.
- **Stap 4:** H_C;D;juli;hor;zi verdelen over oriëntaties **gewogen naar A_T;or,zi**.
- **Stap 5:** H_C;D;juli;or,zi = H_C;D;juli;hor;or,zi (Stap 4) + H_C;D;juli;vert;or,zi (Stap 2).

Lineaire thermische bruggen alleen splitsen als TOjuli ≠ 0 én er een grenswaarde geldt
(p. 117); ons model rekent forfaitair met 0 bruggen — geen impact.

### 2.5 Q_C;HP;juli;or,zi — booster-warmtepomp (5.41a-c, p. 118-119)

Stap i: `Q_C;nd;juli,zi = Σ_or Q_C;nd;juli;or,zi`. Stap ii:
`f_C;juli;or,zi = Q_C;nd;juli;or,zi / Q_C;nd;juli,zi`. Stap iii:
`Q_C;HP;juli;or,zi = Q_C;HP;juli,zi × f_C;juli;or,zi`. **Zonder booster-WP is dit 0** — ons
model kent geen booster-WP, dus `q_c_hp_juli_kwh = 0`.

### 2.6 Symbolen H-noemer + t_juli (p. 119)

H_C;D;juli;or,zi (directe transmissie excl. begane-grondvloer), H_gr;an;juli;or,zi (grond),
H_C;ve;juli;or,zi (ventilatie), `t_juli` = lengte maand juli volgens §17.2.

---

## 3. t_juli — §17.2, tabel 17.1 (p. 690)

Tabel 17.1 ("Lengte van de maand, t_mi") geeft voor **Juli t_mi = 744 h**. De reeds
gehanteerde `T_JULI_H = 744.0` is dus de **norm-exacte** waarde (31 d × 24 h), niet enkel een
benadering — én identiek aan `DE_BILT_MONTH_LENGTHS_HOURS[Juli] = 744.0`
(`nta8800-tables/src/climate/de_bilt.rs`), dus consistent met de demand-maandlengtes.
Actie: alleen de doc-comment bij de const bijwerken (bron = §17.2/tabel 17.1); waarde blijft.

---

## 4. Wat de keten wél/niet levert — implementatiekeuze (anti-fudge)

De **noemer** (H_C;D;or + H_gr;an;or + H_C;ve;or) is volledig uit de geometrie + de whole-zone
`TojuliResult` te reconstrueren en wordt **norm-conform** gebouwd:

- **H_C;D;vert;or** = Σ exterieur-verticale constructies[or] (A·U + ramen A·U) — **werkelijke**
  oriëntatie (Stap 2/5).
- **H_C;D;hor** = Σ exterieur-**horizontale** constructies (helling ≤ 5°, §7.6.6.4: plat dak;
  A·U + ramen) → **pro-rata A_T** (Stap 3/4). Hellende dakvlakken mét azimuth vallen onder
  H_C;D;vert;or (zie §2.2).
- **H_gr;an** = Σ grond-constructies (A·U, gedocumenteerde screening-vereenvoudiging i.p.v.
  het §8.3-grondmodel) → **pro-rata A_T**.
- **H_C;ve** = `tj.ventilation_h_v_w_per_k` (whole-zone, incl. WTW) → **pro-rata A_T**.

De **teller** `Q_C;nd;juli;or,zi` is het enige echte restant: de norm berekent die door de
§7.2.2-julibalans **per oriëntatie** opnieuw te draaien (oriëntatie-specifieke zonwinst +
pro-rata overige termen). Dat is demand-crate-werk (buiten F3c-scope). **Gekozen, expliciet
gedocumenteerde benadering:** de whole-zone `Q_C;nd;juli` (`tj.monthly_q_c_nd_mj[Juli]`) wordt
over de oriëntaties verdeeld naar het **toegelaten zonwinst-aandeel** per oriëntatie:

```text
S_or = Σ ramen[or] ( A_glas · g · I_juli(or) )      met A_glas = A_raam·(1 − frame_fractie)
f_C;or = (S_or + A_T-fractie · S_hor) / Σ (…)
Q_C;nd;juli;or = Q_C;nd;juli;whole · f_C;or
```

`I_juli(or)` = `climate.solar_irradiation[or][Juli]` (De Bilt, tabel 17.2, MJ/m²). Dit is de
**fysisch juiste** verdeelsleutel: de julikoudebehoefte wordt gedomineerd door de per
oriëntatie toegelaten zoninstraling — precies wat TOjuli beoogt te onderscheiden (zuid-zwaar
glas ⇒ zuid maatgevend). Een puur geometrische A_T-verdeling zou dat onderscheid uitmiddelen.
Terugval op A_T-fractie als `S_total = 0` (raamloze zone → julikoudebehoefte ≈ 0 → TOjuli ≈ 0).

**Restant voor F3d:** (1) norm-exacte per-oriëntatie §7.2.2-julibalans (echte per-oriëntatie
Q_C;nd i.p.v. zonwinst-gewogen verdeling van de whole-zone waarde); (2) **helling-afhankelijke
zoninstraling** — de zonwinst-proxy `S_or` gebruikt nu de *verticale* `I_juli(or)`; tabel 17.2
geeft `I_sol` per oriëntatie én hellingshoek β (90°/…/0°), dus een hellend dakvlak ontvangt in
juli méér dan de verticale waarde (onderschatting van zijn gewicht); (3) lineaire-bruggen-split;
(4) §8.3-grondmodel i.p.v. A·U voor H_gr. De `PerOrientation`-methode + `notes` markeren dit
expliciet in het resultaat.

---

## 5. Mapping norm → code

| Norm | Code (`openaec-project-shared`) |
|---|---|
| Formule 5.40 | `nta8800-ep::tojuli_orientation` (ongewijzigd) |
| Zone-toets (max, AT≤3, pass, actief-gekoeld) | `nta8800-ep::tojuli_zone` |
| Per-oriëntatie-inputs opbouwen | nieuw: `build_tojuli_orientation_inputs` in `beng/mod.rs` |
| t_juli = 744 h | `beng::T_JULI_H` (doc-bron → §17.2/tabel 17.1) |
| ActivelyCooled-shortcut | ongewijzigd |
| WholeZoneScreening | **vervangen** door `TojuliMethod::PerOrientation`, `pass = Some(...)` |
