# C5 — Norm-analyse opake zonwinst (formule 7.33) + plafondkolom voetnoot c (tabel 7.10)

**Datum:** 2026-07-13
**Werkpakket:** C5 — twee restgap-kandidaten uit de C4-demand-keten-analyse
norm-conform afhandelen:
- **C5a** — de opake (niet-transparante) zonwinst + hemelstraling die de
  demand-keten tot nu toe alleen voor ramen rekende (formule 7.33, §7.6.3).
- **C5b** — de plafondkolom-keuze (open vs gesloten) van tabel 7.10, in het
  bijzonder of **voetnoot c** uit onze invoer afleidbaar is.

**Norm-bron:** `NTA 8800:2025+C1:2026 nl.pdf` — §7.6.3 (formules 7.31–7.33,
p. 183-184), §7.6.5 (formule 7.39, p. 188), §7.6.6.3 (α_sol, p. 203), §7.6.6.4
(F_sky, p. 203), §7.7 + tabel 7.10 (p. 204-205). PDF gerenderd via PyMuPDF
(tekst + pixmap voor tabel 7.10).

---

## Deel A — Opake zonwinst (formule 7.33)

### A.1 Letterlijke formule-transcriptie

**Formule 7.31 (§7.6.3) — som over transparant + opaak:**

```
Q_H/C;sol;dir;zi;mi = Σ_k Q_H/C;sol;wi,k;mi  +  Σ_k Q_H/C;sol;op,k;mi
```

De demand-keten rekende t/m C4 alleen de **eerste** som (ramen, formule 7.32).
C5a voegt de **tweede** som (opake vlakken, formule 7.33) toe.

**Formule 7.32 (ramen) — reeds geïmplementeerd (C4):**

```
Q_H/C;sol;wi,k;mi = g_gl;wi,k;H/C;mi · A_wi,k · (1 − F_fr;wi,k) · F_sh;obst;wi,k;mi
                    · I_sol;wi,k;mi · t_mi · 0,001  −  Q_sky;wi,k;mi
```

**Formule 7.33 (opake constructies) — NIEUW in C5a:**

```
Q_H/C;sol;op,k;mi = α_sol · R_se · U_c;op,k · A_c;op,k · F_sh;obst;op,k;mi
                    · I_sol;op,k;mi · t_mi · 0,001  −  Q_sky;op,k;mi
```

met (letterlijk uit p. 184):
- `α_sol` — dimensieloze absorptiecoëfficiënt voor zonnestraling, §7.6.6.3;
- `R_se` — warmteovergangsweerstand buitenzijde, C.2, in m²K/W;
- `U_c;op,k` — warmtedoorgangscoëfficiënt van het opake element, §8.2.2, W/(m²K);
- `A_c;op,k` — geprojecteerde oppervlakte van het opake element, §K.1.2, m²;
- "en met de overige variabelen beschreven in de vorige formule (waarbij index
  wi wordt vervangen door index op)."

**F_sh;obst voor opake vlakken (letterlijk, p. 185):**
> "Voor de dimensieloze beschaduwingsreductiefactor voor externe belemmeringen
> van niet-transparant element op,k, geldt: F_sh;obst;wi,k;mi = 1."

→ Voor opake vlakken **F_sh;obst = 1** (geen belemmeringsreductie). Er is óók
geen g-waarde en geen beweegbare zonwering: de opake term is dus
**balans-onafhankelijk** (identiek op de warmte- en koudebalans).

**Formule 7.39 (§7.6.5) — hemelstraling Q_sky, geldt voor elk schilelement:**

```
Q_sky;k;mi = F_sky;k · R_se;k · U_c;k · A_c;k · h_lr;e · Δθ_sky;mi · t_mi · 0,001   [kWh]
```

met `h_lr;e = 4,14 W/(m²·K)` en `Δθ_sky;mi = 11 K` (beide norm-vaste getalswaarden,
p. 189). Identiek aan de reeds voor ramen geïmplementeerde Q_sky-term, met de
opake `U`/`A`. OPMERKING (p. 189): deze term staat bewust in de **winst** (als
aftrek), niet in het verlies.

### A.2 Rekenwaarden

**α_sol (§7.6.6.3, letterlijk p. 203):**
> "De absorptiecoëfficiënt voor zonnestraling van het buitenoppervlak van de
> niet-transparante constructie bedraagt: α_sol = 0,6."

→ Forfaitair **α_sol = 0,60** voor élk opaak vlak, ongeacht kleur. Geen
kleur-afhankelijke aanname verzonnen.

**F_sky (§7.6.6.4, letterlijk p. 203):**
- F_sky = 1     — horizontale constructie (helling ≤ 5°);
- F_sky = 0,75  — hellende constructie (5° < helling ≤ 75°);
- F_sky = 0,5   — verticale constructie (helling > 75°);
- F_sky = 0     — overhellend (naar de grond gericht) / grens met serre.

Identiek aan de reeds bestaande `sky_view_factor` in `solar_gains.rs` (die is
dus hergebruikt).

**R_se = 0,04 m²K/W** — hergebruik van de bestaande `R_SE`-constante (C.2,
buitenzijde, exterieur). Consistent met `nta8800_view::surface_resistances`
(0,04 voor alle exterieur-vlakken).

### A.3 Implementatie (bestand:regel)

| Locatie | Wijziging |
|---|---|
| `crates/nta8800-demand/src/calc/solar_gains.rs` | `ALPHA_SOL = 0.6` (§7.6.6.3); `struct OpaqueElement { area, u_value, orientation, tilt }`; `fn monthly_opaque_solar_gains(elements, climate) -> MonthlyProfile<Energy>` (formule 7.33 netto = bruto-absorptie − Q_sky), balans-onafhankelijk. `sky_view_factor` hergebruikt. 4 nieuwe hand-berekende unit-tests. |
| `crates/nta8800-demand/src/calc/mod.rs` | `calculate_demand_with_cooling_ht` krijgt additief `opaque_elements: &[OpaqueElement]`; opake winst opgeteld bij `q_gn_heating` én `q_gn_cooling`; `breakdown.monthly_q_sol` = ramen + opaak (consistentie `Q_gn = Q_int + Q_sol`). `calculate_demand` geeft `&[]` door (byte-identiek). |
| `crates/nta8800-demand/src/lib.rs` | `pub use ... OpaqueElement`. |
| `crates/openaec-project-shared/src/tojuli.rs` | `build_opaque_solar_elements(geometry)` — exterieur-opake vlakken, `A_opaak = A_bruto − Σ A_opening` (geen dubbeltelling met de ramen), oriëntatie/helling via dezelfde mapper als de ramen; doorgegeven aan de demand-call. |

**Eenheden.** `climate.solar_irradiation` is per maand in MJ/m² (reeds
geïntegreerd, zoals bij de ramen). `α·R_se·U·A` is dimensieloos × m², dus
`(α·R_se·U·A) · I_sol[MJ/m²]` → MJ — dezelfde MJ-conventie als de raam-zonwinst.
`Q_sky` (in W) × maanduren × Wh→MJ → MJ. Geen apart rekenpad.

**Additiviteit.** De opake term loopt via de demand-crate die óók de standalone
TO-juli-/ISSO-51-callers voedt. Formule 7.33 is niet BENG-specifiek (§7.6 geldt
norm-breed), dus dit is een correcte, gewenste gedragswijziging; TO-juli-
uitkomsten verschuiven mee (iets meer zomer-zonwinst op opake vlakken → iets
hoger oververhittingsrisico). `calculate_demand` (alleen tests) blijft
byte-identiek via de lege slice.

### A.4 Verwachte richting (vóór meting)

De opake term is per vlak per maand `α·R_se·U·A·I_sol − F_sky·R_se·U·A·h_lr·Δθ·t`.
Beide delen schalen met `R_se·U` (klein: ~0,008 bij U=0,2) → de opake bijdrage is
**tweede-orde** t.o.v. de ramen (die met `g` ≈ 0,4 rekenen, zonder R_se·U-demping).

Seizoensteken:
- **Zomer** (hoog I_sol): op een zonbeschenen vlak overheerst de absorptie →
  **netto winst** → koudebehoefte omhoog.
- **Winter** (laag I_sol) en op naar-de-hemel-gerichte vlakken (dak, F_sky groot):
  Q_sky overheerst → **netto verlies** → warmtebehoefte omhoog.

Netto verwachting: een **kleine verhoging van zowel Q_H;nd als Q_C;nd**
(seizoenen overlappen niet), dus BENG 1/2 licht omhoog — richting de te-lage
certified-stand.

---

## Deel B — Plafondkolom voetnoot c (tabel 7.10)

### B.1 Tabel 7.10 — letterlijke transcriptie (p. 204-205)

**Tabel 7.10 — Forfaitaire waarden voor de specifieke interne warmtecapaciteit**
`D_m;int;eff;zi` [kJ/(m²·K)]

| Bouwwijze vloeren | Bouwwijze wanden | Gesloten of verlaagd plafond ᵃ˒ᶜ | Geen of open plafond ᵇ |
|---|---|---:|---:|
| Licht | Licht | 55 | 80 |
| Licht | Zwaar | 110 | 180 |
| Zwaar | Licht | 110 | 180 |
| Zeer zwaar | Licht | 110 | 180 |
| Zwaar | Zwaar | 180 | 360 |
| Licht | Zeer zwaar | 180 | 360 |
| Zwaar | Zeer zwaar | 250 | 450 |
| Zeer zwaar | Zwaar | 250 | 450 |
| Zeer zwaar | Zeer zwaar | 250 | 450 |

De kolom **"gesloten of verlaagd plafond"** geeft de **lagere** D_m; **"geen of
open plafond"** de **hogere** D_m.

**Voetnoten (letterlijk):**

> **ᵃ** Bij utiliteitsbouw moet worden uitgegaan van de kolom 'gesloten of
> verlaagd plafond' tenzij aan de volgende twee voorwaarden wordt voldaan:
> 1) er is sprake van een vrijhangend plafond in het verblijfsgebied dat ten
>    minste netto 15 % van de plafondoppervlakte, gelijkelijk verdeeld over het
>    plafond, open is uitgevoerd, én
> 2) er is geen sprake van de situatie geschetst onder c.

> **ᵇ** Bij woningbouw moet worden uitgegaan van de kolom 'geen of open plafond',
> behalve in situaties waarin sprake is van de situatie geschetst onder c.

> **ᶜ** Indien bij woningbouw de bovenzijde van een vloer in een zwaardere
> categorie valt dan de onderzijde van de vloer erboven, dan moet worden
> uitgegaan van de kolom 'gesloten of verlaagd plafond'.

> **OPMERKING 2** Voetnoot c gaat over het volgende: er kan sprake zijn van een
> zware vloer, terwijl de verdiepingen erboven een lichte vloerconstructie hebben,
> waardoor het plafond licht is. Het verschil in massa tussen vloer en plafond
> wordt in rekening gebracht door te rekenen met een gesloten of verlaagd plafond
> in plaats van geen of open plafond. Dit treedt bijvoorbeeld op als de begane
> grondvloer zwaar is, bijvoorbeeld doordat er een dekvloer op ligt, en de
> vloerconstructie erboven (aan de plafondzijde) licht is.

### B.2 Analyse — is voetnoot c uit onze invoer afleidbaar?

**Wat de conditie eist.** Voetnoot c is een **twee-vloer-vergelijking**: de
massaklasse van de *bovenzijde van een vloer* versus de *onderzijde van de vloer
erboven*. OPMERKING 2 maakt het concreet: een **zware begane-grondvloer** met
**lichte verdiepingsvloeren** erboven → gesloten kolom.

**Wat onze invoer draagt.** Het BENG-DTO (`BengZone.bouwwijze_vloer`) codeert
**één** vloer-bouwwijze-code per rekenzone (bv. `CONSTRM_FL_26` = massief beton,
zeer zwaar). Dat is één scalar voor de héle zone. Het codeert **niet**:
- de vloerconstructie **per verdieping**, noch
- de massaklasse van "de onderzijde van de vloer erboven" als apart gegeven.

**Gevolg.** Uit één vloer-massaklasse is niet af te leiden of de *bovenzijde van
een vloer zwaarder is dan de onderzijde van de vloer erboven*. Twee fysiek
verschillende gebouwen leveren dezelfde code:
1. zware BG-vloer + lichte verdiepingsvloeren → **voetnoot c van toepassing**
   (gesloten kolom);
2. massieve betonvloeren op élke verdieping → **voetnoot c NIET van toepassing**
   (bovenzijde = onderzijde erboven = zwaar, geen sprongt).

Beide worden in Uniec/ons DTO ingevoerd als `CONSTRM_FL_26`. De conditie is dus
**niet eenduidig afleidbaar** uit de beschikbare invoer.

**Wat Uniec doet is niet non-circulair vast te stellen.** De C3-capture kon de
door Uniec gekozen plafondkolom / resulterende D_m **niet** uitlezen (Uniec
exposeert D_m niet). Dat certified beter bij D_m=110 (gesloten) past (zie B.3) is
**geen bewijs** dat Uniec voetnoot c toepaste — het aannemen daarvan om de match
te verbeteren is precies de fudge die het werkpakket verbiedt.

### B.3 Gevoeligheid (gemeten, D_m 110 vs 180, met C4+C5a actief)

Diagnostisch gemeten door `ceiling_type` tijdelijk op `ClosedOrSuspended` te
zetten (daarna teruggedraaid; `expected.json` onaangeraakt):

| Case | grootheid | open (D_m 180, **shipped**) | gesloten (D_m 110) | certified |
|---|---|---:|---:|---:|
| Aalten | BENG 1 | 97,58 (−5,9 %) | **104,77 (+1,0 %)** | 103,69 |
| Aalten | BENG 2 | 15,38 (−37,8 %) | 18,87 (−23,6 %) | 24,71 |
| Aalten | BENG 3 | 89,93 (+4,9 pp) | 88,14 (+3,1 pp) | 85,0 |
| Aalten | heat primair [kWh] | 2172 (−14,9 %) | 2264 (−11,2 %) | 2551 |
| Gouda | BENG 1 | 82,98 (−13,4 %) | 88,39 (−7,8 %) | 95,86 |
| Gouda | heat primair [kWh] | 4777 (−26,6 %) | 4951 (−23,9 %) | 6506 |

De gesloten kolom trekt Aalten BENG 1 naar +1,0 % en heating naar −11,2 % — een
opvallend betere fit. **Juist daarom** is de anti-fudge-discipline hier bindend:
de betere fit is geen norm-bewijs.

### B.4 Besluit C5b

**Voetnoot c wordt NIET automatisch toegepast.** De open-plafond-default
(voetnoot b, woningbouw) blijft staan. In plaats van een kolomkeuze die niet uit
de norm-tekst + onze invoer volgt, wordt de **gevoeligheid gedocumenteerd** als
`BengResult.notes`-regel bij een zware/zeer-zware vloer (waar de conditie
plausibel is):

> `crates/openaec-project-shared/src/beng/mod.rs` — "Plafondkolom (C5b): tabel
> 7.10 voetnoot c … is NIET toegepast — die conditie … vereist
> per-verdieping-vloerconstructie die de enkelvoudige bouwwijze-code niet levert;
> de open-plafond-default (voetnoot b) blijft staan. Gevoeligheid: de gesloten
> kolom verlaagt D_m (bv. zeer-zwaar/licht 180 → 110 kJ/(m²·K)) … BENG 1 met orde
> ~7 kWh/(m²·jr) verhoogt."

`dynamics.rs`-`ceiling_type` blijft dus ongewijzigd (woningbouw → open).
Toekomstige route (V2): een per-verdieping-vloer-invoerveld dat de
twee-vloer-vergelijking wél mogelijk maakt.

---

## Deel C — Meetmatrix (compute_beng, bridged; % = afwijking t.o.v. certified)

Gemeten met `uniec_measure_bridged` / `gouda_measure_bridged`
(`--ignored --nocapture`). C5b is niet geïmplementeerd (zie B.4), dus de matrix
toont **C4 (vóór) → C5a (na)**; de D_m=110-kolom is de C5b-gevoeligheid (niet
shipped).

**Aalten-2522** — cert BENG1 103,69 / BENG2 24,71 / BENG3 85,0 / heat 2551 kWh:

| grootheid | C4 (vóór) | **C5a (shipped)** | C5a+D_m110 (C5b-gevoeligheid) | cert | tol | status |
|---|---:|---:|---:|---:|---:|---|
| BENG 1 | 96,39 (−7,0 %) | **97,58 (−5,9 %)** | 104,77 (+1,0 %) | 103,69 | ±6 % | rood (BENG1 alleen binnen ±6 %, aggregaat niet) |
| BENG 2 | 14,73 (−40,4 %) | **15,38 (−37,8 %)** | 18,87 (−23,6 %) | 24,71 | ±10 % | rood |
| BENG 3 | 90,30 (+5,3 pp) | **89,93 (+4,9 pp)** | 88,14 (+3,1 pp) | 85,0 | ±3 pp | rood |
| heat primair [kWh] | 2168 (−15,0 %) | **2172 (−14,9 %)** | 2264 (−11,2 %) | 2551 | ±10 % | rood |
| cooling primair [kWh/m²] | 8,72 | **9,30** | 11,42 | — | — | omhoog (zomer-absorptie) |

**Gouda-2467** — cert BENG1 95,86 / BENG2 27,48 / BENG3 83,7 / heat 6506 kWh:

| grootheid | C4 (vóór) | **C5a (shipped)** | C5a+D_m110 | cert | status |
|---|---:|---:|---:|---:|---|
| BENG 1 | 81,80 (−14,7 %) | **82,98 (−13,4 %)** | 88,39 (−7,8 %) | 95,86 | rood |
| BENG 2 | 3,61 (−86,9 %) | **4,27 (−84,5 %)** | 7,28 (−73,5 %) | 27,48 | rood (PV-normversie, F3d-8) |
| heat primair [kWh] | 4761 (−26,8 %) | **4777 (−26,6 %)** | 4951 (−23,9 %) | 6506 | rood |
| cooling primair [kWh] | 1224 | **1295** | 1522 | 244 | omhoog |

**Lezing.** C5a beweegt alle indicatoren de fysisch/normatief juiste kant op
(BENG 1/2 omhoog, richting de te-lage certified-stand), maar het effect is klein
(opake bijdrage `R_se·U`-gedempt): Aalten heating +4 kWh, BENG 1 +1,1 pp. Géén
anker/golden komt binnen tolerantie — alle blijven eerlijk `#[ignore]` met de
C5a-reststand. De resterende, veel grotere aggregaat-gap zit niet in de opake
zonwinst; de dominante hefboom blijft de plafondkolom-massa (C5b-gevoeligheid,
niet norm-afleidbaar) plus — voor Gouda — de PV-saldering-normversie (F3d-8).

## Deel D — Teststatus

`cargo test --workspace`: **volledig groen** (1550 tests). Nieuw/gewijzigd:
- `nta8800-demand::calc::solar_gains` — `OpaqueElement` + `monthly_opaque_solar_gains`;
  4 nieuwe hand-berekende unit-tests (zuidgevel, zuidelijk dakvlak 45°,
  ontbrekende oriëntatie = puur Q_sky-verlies, lege lijst). De hand-berekeningen
  staan letterlijk in het test-commentaar, onafhankelijk van de productieformule
  (QC-les C4: geen test-circulariteit).
- `nta8800-demand::calc::mod` — additieve `opaque_elements`-parameter; bestaande
  integratietests draaien via `calculate_demand` (lege slice) → onveranderd.
- `openaec-project-shared::beng` — C5b-gevoeligheidsnote; `beng`-tests groen.
- `beng_golden.rs` — `#[ignore]`-redenen bijgewerkt met de C5a-stand.
  `expected.json` onaangeraakt.
