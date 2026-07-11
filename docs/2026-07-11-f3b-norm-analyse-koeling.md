# F3b — Norm-analyse koel-keten: opwekkings-stap (H.10) + omgevingskoude (BENG 3)

**Datum:** 2026-07-11 · **Normversie:** NTA 8800:2025+C1:2026 · **Bron-PDF:**
`Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\NTA 8800_2025+C1_2026 nl.pdf`
(paginanummers hieronder = PDF-paginalabel rechtsonder op de pagina; extractie via
PyMuPDF, tekst-laag).

Deze notitie onderbouwt twee koel-fixes:
1. **Koel-eindenergie (H.10.5)** — de opwekkings-efficiëntie van *vrije koeling* was fout
   gemodelleerd (COP = 1,0 op het niet-vrije deel → koeling domineerde met 56 kWh/(m²·jr)).
2. **BENG 3 / omgevingskoude (rencold, §5.6.2.2)** — de vrije-koeling-koude telde niet mee in
   de hernieuwbaar-teller.

---

## 0. Diagnose-correctie vooraf

Het F2b/F3a-verslag stelde dat "de opwekkings-stap ontbreekt (deling door SEER)". Dat is voor
**compressie-** en **absorptiekoeling** onjuist: `calculate_cooling`
(`crates/nta8800-cooling/src/calc/monthly_use.rs`) deelt Q_C;nd al door
`η_em·η_dist·f_reg·COP` — de SEER-deling gebeurt dus wél. Het echte defect zit uitsluitend in
de **vrije-koeling-tak**: die rekende het niet-vrije deel `(1 − factor)` af tegen een
"nominale COP = 1,0" (`monthly_use.rs::compute_monthly_use`, regel ~131), dus
`E_C ≈ 0,6 · Q_C;gen;out` — bijna één-op-één elektriciteit. Vandaar de 56 kWh/(m²·jr) in de
smoke (synthetische WP-bodem-tussenwoning met `FreeCooling { factor: 0,4 }`).

---

## 1. Koel-eindenergie per opwekker-type (§10.5, p. 393-424)

### 1.1 Structuur (§10.5.1/§10.5.6.1)

De elektrische (of thermische) eindenergie voor koeling volgt uit de **koude die de opwekker
moet leveren** `Q_C;gen;pref` gedeeld door de energie-efficiëntie. Voor de forfaitaire methode
3 (geen fabrikantgegevens — ons geval):

| Opwekker (GEN_TYPE) | Formule | Nr. | Pagina |
|---|---|---|---|
| Compressie (COMP) | `E_C;el = Q_C;gen;pref / (EER · f_prpr)` | (5.76)/(10.76) | 417 |
| Absorptie (ABS) | `Q_H;C;abs = Q_C;gen;pref / (ζ_n · f_prpr)` | (10.77) | 417 |
| Externe koude (dc) | `Q_C;ext = Q_C;gen;pref / (η_C;gen;equiv;dc · f_prpr)` | (10.78) | 418 |

waarbij `Q_C;gen;pref = Q_C;nd / (η_em · η_dist · f_reg)` = de koude die de opwekker levert
(demand opgehoogd met afgifte-/distributie-/regelverliezen). In het crate-model is dit exact
`Q_C;use × COP` voor compressie/absorptie; de crate deed dit al goed.

### 1.2 Forfaitaire EER-waarden (tabel 10.29/10.30, p. 419)

| Opwekker | EER / ζ | Voorwaarde |
|---|---|---|
| Onbekende koudeopwekker in collectieve installatie | **3,00** | tabel 10.29 |
| Elektrisch aangedreven compressiekoelmachine | **3,00** | tabel 10.29 |
| Met gas aangedreven absorptiekoeling (ζ) | 0,80 | tabel 10.30 |
| Absorptie op externe warmte (ζ) | 0,70 × η_dh | tabel 10.30 |

`f_prpr;si = 1,0` bij de forfaitaire tabellen 10.29/10.30 (p. 419).

### 1.3 Vrije koeling — alleen pompenergie (§10.5.7.2.1, p. 422-424)

De sleutel-passage (p. 422): bij vrije koeling "waarbij de aan de rekenzone onttrokken energie
rechtstreeks aan het oppervlaktewater of bodemopslagsysteem wordt overgedragen … **wordt alleen
pompenergie toegerekend**". De elektrische hulpenergie (formule 10.86, p. 423):

```
W_fc;el;in;si,mi = Q_C;hr;out;si,mi / EER_fc;si,mi          (10.86)
```

met `Q_C;hr;out = Q_C;gen;pref` (de door de vrije koeling geleverde koude). De forfaitaire
EER_fc uit **tabel 10.34 (p. 424)**:

| Systeemtype vrije koeling | EER_fc |
|---|---|
| Koudeopslag open aquifer (woningen, ≥ 2013) | 23 |
| Koudeopslag open aquifer utiliteit (< 2013) | 16 |
| Koudeopslag open aquifer woningen (< 2013) | 14 |
| Oppervlaktewater | **10** |
| Koudeopslag **gesloten** systeem (bodemwarmtewisselaars) | **10** |
| Dauwpuntskoeling | 8 |

**Alle EER_fc ≥ 8.** De laagste (10 voor gesloten bodemwarmtewisselaar / oppervlaktewater) is de
conservatieve V1-forfait die past bij de smoke-woning (bodem-WP-bron).

### 1.4 Prioritering: vrije koeling + backup-chiller (§10.5.3 + tabel 10.15, p. 398-400)

Tabel 10.15 zet vrije koeling (WKO/oppervlaktewater/dauwpunt) op **hoogste** preferentie en
chillers op laagste. `βC;gen` (formule 10.49) bepaalt de energiefractie die de preferente
(vrije) opwekker dekt (tabel 10.16); het **restant** wordt door de volgende opwekker (chiller)
geleverd. Omdat Q_C;nd berekend is bij een *gehandhaafde* koelsetpoint (24 °C), gaat de norm
ervan uit dat de koudevraag volledig wordt gedekt — het niet-vrije deel loopt dus via een
backup-compressiekoelmachine (forfait EER = 3,0, tabel 10.29).

**V1-invoermodel-keuze:** het DTO kent alleen `free_cooling_fraction` (= de energiefractie
`factor` van de vrije opwekker), geen EER_fc-type-selectie en geen expliciete backup. De
norm-consistente V1-interpretatie:

```
E_C;el = Q_C;gen;out · [ factor / EER_fc  +  (1 − factor) / EER_backup ]
       = Q_C;gen;out · [ factor / 10       +  (1 − factor) / 3,0 ]
```

Dit vervangt de foute `(1 − factor) · Q_C;gen;out / 1,0`. Een "vrije-koeling-zónder-backup"-
variant (het restant blijft ongedekt → comfortverlies i.p.v. energie) is een V2-verfijning
zodra het DTO het opwekker-type + de aanwezigheid van een backup codeert.

---

## 2. BENG 3 — omgevingskoude rencold (§5.6.2.2, formule 5.34, p. 105-106)

```
E_Pren;C;mi;si = Σ Q_C;gen;out · f_Pren;rencold  (+ dc-termen)          (5.34)
```

met de **voorwaarde** (letterlijk, p. 105):

- "in het geval van **(vrije) koeling met een EER ≥ 8**": `Q_C;gen;out = Q_C;gen;pref` (de
  geleverde koude telt als rencold);
- "in het geval van (vrije) koeling met een **EER < 8**": `Q_C;gen;out = 0` (telt niet mee).

De primaire hernieuwbare factor `f_Pren;rencold = 1,0` (**tabel 5.4, p. 109**). Dus de rencold-
hoeveelheid = de door de vrije koeling (EER ≥ 8) geleverde **koude** (niet de elektriciteit):

```
Q_rencold = factor · Q_C;gen;out        (want EER_fc = 10 ≥ 8)
```

Het backup-chiller-deel (EER = 3 < 8) levert **geen** rencold. Compressie- en absorptiekoeling
(forfait EER 3,0 / ζ 0,80) vallen eveneens onder de drempel → rencold = 0.

De EP-crate consumeert dit al: `EpInputs.renewable_ambient_cold_mj × F_PREN_RENCOLD (=1,0)`
(`nta8800-ep/src/calc/ep_score.rs:139/190`). F3b hoefde alleen het veld te vullen.

---

## 3. Wijzigingen en verwijzingen

| Wijziging | Bestand | Norm-ref |
|---|---|---|
| Vrije-koeling twee-termen-formule (EER_fc + backup) i.p.v. COP = 1,0 | `nta8800-cooling/src/calc/monthly_use.rs` | (10.86) p. 423 + tabel 10.34 p. 424 + tabel 10.29 p. 419 + tabel 10.15 p. 399 |
| Forfait-constanten `EER_FREE_COOLING`, `EER_BACKUP_COMPRESSION`, `EER_RENCOLD_THRESHOLD` | `nta8800-cooling/src/calc/monthly_use.rs` | tabel 10.34 / 10.29 / (5.34) |
| `monthly_rencold_mj` + `annual_rencold_mj` op `CoolingResult` | `nta8800-cooling/src/result/cooling_result.rs` | (5.34) p. 105 |
| `annual_rencold_mj` doorgegeven in `TojuliResult` | `openaec-project-shared/src/tojuli.rs` | — (plumbing) |
| `renewable_ambient_cold_mj` gevuld; F3b-note verwijderd | `openaec-project-shared/src/beng/mod.rs` | (5.34) + tabel 5.4 p. 109 |

**Plaatsingskeuze opwekkings-stap:** in `nta8800-cooling` (niet in `compute_beng`), want H.10 =
de koudeopwekkings-methode; de crate modelleert `FreeCooling` al en bezit de sterkste
norm-refs. `compute_beng` neemt de gecorrigeerde `annual_q_c_use_mj`/`annual_rencold_mj` af
zonder H.10-logica te dupliceren.

De 3 crate-tests met de weerlegde normlezing (vrije koeling = 0 elektriciteit / COP 1,0) zijn
aangepast met bovenstaande formule-/paginaverwijzing; de **Bijlage-AA-golden** (`calculate_bijlage_aa`,
aparte code-pad) en de F0-goldens zijn onaangeroerd.

---

## 4. Smoke-resultaat (synthetische WP-bodem-tussenwoning, `FreeCooling { factor: 0,4 }`)

Zie §5 van de F3b-rapportage in de sessie-handoff. Kort: koeling daalt van **56,2** naar
**~22,5 kWh/(m²·jr)** (0,4 vrij @ EER 10 + 0,6 backup @ EER 3); rencold gaat van 0 naar
> 0 → BENG 3 stijgt. Het residu boven de RVO-referentie is de **F_sh = 1,0**-overschatting van
Q_C;nd (F3d-kalibratie, buiten F3b-scope) plus de 60 % backup-compressie-aandeel dat bij
`factor = 0,4` een reële energiepost is.
