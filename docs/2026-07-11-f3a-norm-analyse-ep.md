# F3a — Norm-analyse EP-laag: hernieuwbaar aandeel (BENG 3) + PV-saldering (BENG 2)

**Datum:** 2026-07-11 · **Normversie:** NTA 8800:2025+C1:2026 · **Bron-PDF:**
`Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\NTA 8800_2025+C1_2026 nl.pdf`
(paginanummers hieronder = PDF-paginalabel, gelijk aan het nummer rechtsonder op de pagina).

Deze notitie onderbouwt twee EP-laag-fixes:
1. **BENG 3 / hernieuwbaar aandeel** — omgevingswarmte van warmtepompen telde niet mee.
2. **BENG 2 / PV-saldering** — PV verlaagde het primaire energiegebruik niet (factor 0 i.p.v. 1,45).

---

## 1. Indicator-definities (§5.3.1, p. 71-73)

| Indicator | Formule | Nr. | Pagina |
|---|---|---|---|
| BENG 2 = EwePTot | `EPTot / Ag;tot` | (5.2) | 71 |
| BENG 3 = RERPrenTot | `EPrenTot / (EPTot + EPrenTot) × 100 %` | (5.3) | 72 |

waarin `EPTot` = karakteristiek primair-**fossiel** energiegebruik (§5.5, kWh/jr) en
`EPrenTot` = hernieuwbaar primair energiegebruik (§5.6, kWh/jr). **Belangrijk:** de RER-noemer
gebruikt de *gesaldeerde* `EPTot` — dus fix 2 (PV-saldering) werkt door in fix 1 (BENG 3).

---

## 2. BENG 2 — karakteristiek primair-fossiel + PV-saldering (§5.5, p. 82-95)

### 2.1 Kernformule (5.10, p. 84)

```
EPTot;mi = Σ_ci EP;del;ci − Σ_gi EP;exp;T;gi − Σ_gi EP;exp;el;gi − Σ EP;pr;nEPus;el − EP;BAT,out;tot
```

met per energiedrager `EP;del;ci = EEPdel;ci × fP;del;ci` (5.11) en voor elektriciteit
`EEPdel;el = EEPus;el − Epr;EPus;el` (5.15): afgenomen net-elektriciteit = gebruik minus
zelf-gebruikte eigen productie.

### 2.2 De primaire-energiefactoren van elektriciteit (tabel 5.2, p. 93-94)

| Kolom | Symbool | Waarde |
|---|---|---|
| Aangeleverd (net) | `fP;del;el` | **1,45** |
| Op eigen perceel gebruikt (zelfconsumptie) | `fP;pr;us;el` | **1,45** |
| Geëxporteerd | `fP;exp;el` | **1,45** |

### 2.3 Gevolg: de zelfconsumptie/export-splitsing valt weg voor het totaal

> ⚠️ **Correctie/bevestiging 2026-07-12 (F3d-8).** Deze paragraaf is bij F3d-8
> **geverifieerd tegen de volledige formulestructuur** §5.5.2–5.5.4 (formules 5.9–5.27,
> p. 85–93) en is **wiskundig correct** voor NTA 8800:2025+C1. De maandonafhankelijkheid
> volgt uit de identiteit `Max(0,a−b) − Max(0,b−a) = a − b` en geldt zolang
> `fP;del;el = fP;exp;el = 1,45` en er geen batterij ≥ 5 kWh is (zie
> `docs/2026-07-12-f3d8-norm-analyse-saldering.md` §3). Wat F3a **niet expliciet noemde**
> maar wél impliciet klopt: het zelfgebruik is maandelijks begrensd op `Min(PV, EEPus;el)`
> (bovengrens 5.25), waarna het overschot als export tegen dezelfde 1,45 wordt afgetrokken
> (5.26) — netto identiek. **De certified Uniec-waarde (BENG 2 = 27,48) weerlegt deze
> paragraaf NIET:** die tool crediteert maar ~64 % van de PV (maand-directgebruik, ouder-norm/
> bijlage-AB-model) en is een **normversie-artefact**. Onder 2025+C1 mág BENG 2 negatief
> zijn. **De EP-crate blijft ongewijzigd.**

Omdat alle drie de elektriciteitsfactoren **gelijk (1,45)** zijn, laat de
elektriciteitsterm van (5.10) zich per maand exact herschrijven. Met
`PV = Epr;EPus;el + Epr;nEPus;el + Eexp;el` (alle opgewekte PV wordt zelf-gebruikt of
geëxporteerd) en `Epr;nEPus;el = 0` voor de energieprestatie-indicatoren (huishoudelijk
verbruik `enEPus;el = 0 W/m²`, §5.5.4.2 p. 92):

```
elektriciteit-primair = (EEPus;el − Epr;EPus;el)·1,45 − Eexp;el·1,45 − Epr;nEPus;el·1,45
                      = (EEPus;el − PV)·1,45
```

De identiteit `min(PV,EEPus) + max(0,PV−EEPus) = PV` maakt dit **exact en maandonafhankelijk**:
de jaarsom is `(EEPus;el;jaar − PV;jaar)·1,45`, ongeacht de maandelijkse matching. De
zelfconsumptiefractie (5.22-5.26) hoeft dus niet expliciet berekend te worden zolang alle
drie de factoren 1,45 zijn en er geen batterij (§5.5.14a, ≥5 kWh) in het model zit.

**Implementatie-interpretatie:** in de EP-crate (die per drager het jaarlijkse
elektriciteitsgebruik optelt en PV als aparte term aftrekt) is de correcte saldering:
`EPTot = Σ(dienst-energie × fP;del) − PV × 1,45`. De crate rekende `PV × 0,0` → **bug**.
Fix: `fP;exp;el = 1,45` voor de PV-term (via `primary_factor(HernieuwbareElektriciteit)`).
`EPTot` mag negatief worden bij veel PV (§5.5.2 opmerking 11, p. 86) — geen clamp.

---

## 3. BENG 3 — hernieuwbaar primair energiegebruik EPrenTot (§5.6, p. 100-112)

### 3.1 Optelling (5.28/5.29, p. 102)

```
EPrenTot = Σ_mi Σ_si ( EPren;H + EPren;C + EPren;W + EPren;el )
```

verwarming (§5.6.2.1) + koeling (§5.6.2.2) + warm tapwater (§5.6.2.3) + lokaal opgewekte
hernieuwbare elektriciteit (§5.6.2.4).

### 3.2 Warmtepomp-omgevingswarmte — de ontbrekende term

**Verwarming (5.30/5.31, p. 102-103):**
```
EPren;H = QH;hp;in × fPren;renheat
QH;hp;in = QH;gen;gi;mi;out × (1 − 1/COP_H;gen;prac;mi;gi)      (5.31)
```
Voorwaarden (5.31/5.33, p. 102-103): `COP ≥ 1` **én** brontemperatuur `< 20 °C` **én** de
bron is geen ventilatieretourlucht. Anders `QH;hp;in = 0` (5.33). Lucht- en bodem-WP voldoen;
elektrische weerstand (`COP = 1` → term 0) en HR-ketel (geen WP) leveren niets.

**Warm tapwater (5.35/5.36, p. 106):** identiek met `ηW;gen;prac` i.p.v. COP.

**Omzetting naar het crate-model:** `QH;gen;out = QH;use × ηgen` met `ηgen = SCOP` (de crate
gebruikt de seizoens-COP als opwekkingsrendement, `nta8800-heating` §9). Substitutie in (5.31):
```
QH;hp;in = QH;use × SCOP × (1 − 1/SCOP) = QH;use × (SCOP − 1)
```
Fysisch: omgevingswarmte = geleverde warmte − elektrische input. Analoog voor tapwater:
`QW;ren;hp;in = QW;use × (SCOP_W − 1)`. `QH;use`/`QW;use` = `annual_q_h_use`/`annual_q_w_use`;
`SCOP`/`SCOP_W` = `breakdown.generation_efficiency` van de heating-/dhw-resultaten.

### 3.3 Koeling — omgevingskoude (5.34, p. 105)

Alleen koeling uit systemen met `EER ≥ 8` (vrije koeling, WKO, bodemkoeling) telt als
`rencold`: `EPren;C = QC;gen;out × fPren;rencold`. Compressiekoeling (EER < 8) → 0.
**Buiten F3a-scope** (koel-keten = F3b): de EP-crate krijgt een additief
`renewable_ambient_cold_mj`-veld dat F3b vult; `compute_beng` levert nu `0,0` met een note.

### 3.4 PV (5.39, p. 108)

```
EPren;el = Σ Eel;PV;out × fPren;renelect
```
De **volledige** PV-productie telt (zelfgebruik én export), niet alleen zelfconsumptie.

### 3.5 Primaire hernieuwbare energiefactoren (tabel 5.4, p. 109)

| Energiebron (ri) | `fPren;ri` |
|---|---|
| Hernieuwbaar opgewekte elektriciteit (renelect) | **1,45** |
| Omgevingswarmte (renheat) | **1,0** |
| Omgevingskoude (rencold) | **1,0** |
| Biomassa bmA / bmB / bmC | 1,0 / 0,5 / 0 |

**Implementatie-interpretatie:** `EPrenTot = renheat·1,0 + rencold·1,0 + PV·1,45 (+ biomassa)`.
Biomassa: V1 telt het brandstofverbruik bij `fPren = 1,0` (bmA-forfait) — overschat licht,
te verfijnen in F5 met vermogensklasse + geleverde warmte i.p.v. brandstof.

---

## 4. Wijzigingen en verwijzingen

| Wijziging | Bestand | Norm-ref |
|---|---|---|
| `fPren`-factor PV 0,0 → 1,45 | `nta8800-ep/src/calc/primary_energy.rs` | tabel 5.2 (p. 93), tabel 5.4 (p. 109) |
| RER-formule (5.3) i.p.v. saldo-benadering | `nta8800-ep/src/calc/ep_score.rs::renewable_share` | (5.3) p. 72, §5.6 |
| additieve velden `renewable_ambient_heat_mj` / `..._cold_mj` | `nta8800-ep/src/model/mod.rs` | (5.31)/(5.36)/(5.34) |
| WP-omgevingswarmte-doorgifte | `openaec-project-shared/src/beng/mod.rs::compute_beng` | (5.31) p. 103, (5.36) p. 106 |

Bestaande crate-tests met een normlezing die deze analyse weerlegt (PV-factor 0, saldo-RER)
zijn aangepast met bovenstaande formule-/paginaverwijzing; F0-golden-expected-waarden zijn
**niet** aangeraakt.

---

## 5. Review-napunten 11-07

### 5.1 CO₂-verrekening PV (tabel 5.3, §5.5.6.1, p. 96)

De operationele CO₂-bepaling (5.5.6.1) spiegelt 5.5.2: `mCO2` wordt bepaald met
`fP;del;ci` vervangen door `KCO2;ci`. Tabel 5.3 zet voor **elektriciteit** alle drie de
kolommen gelijk: `KCO2;del;el = KCO2;pr;us;el = KCO2;exp;el = 0,268 kg CO2eq/kWh`. Dus
zelf-gebruikte én geëxporteerde PV **vermijdt** net-CO₂ tegen de elektriciteitsfactor —
de norm zegt aftrek. De crate rekende `co2_factor(HernieuwbareElektriciteit) = 0,0` →
inconsistent na de primaire-energie-fix. **Fix:** factor → `0,0900 kg/MJ` (= de crate-
net-elektriciteitswaarde, tabel-5.3-gelijkheid del=exp). De absolute waarde is 2023-kader
(0,0900/MJ ≈ 0,324/kWh vs. norm 0,268/kWh) — actualisatie is out-of-scope; de *structuur*
(PV verlaagt de CO₂-indicator) volgt de norm. Test `renewable_fuels_have_low_co2` sluit PV
nu uit (PV's "factor" = vermeden net-emissie, geen brandstof-emissie).

### 5.2 Correctie: bijlage Z/AB bestaan nog wél in 2025+C1

Mijn eerdere claim "bijlage Z bestaat niet meer" was **onjuist**. In NTA 8800:2025+C1:2026:
- **Bijlage Z (normatief, p. 1132)** = "Vermelding beleidsfactoren" — een *index* van
  beleidsfactoren, **niet** de f_prim-getalswaarden.
- **Bijlage AB (informatief, p. 1147)** = "Bepaling ZEB-indicator" (EPBD IV, experimenteel),
  **niet** de reguliere CO₂-factoren.
- De getalswaarden staan in **tabel 5.2** (f_prim, §5.5.5, p. 93) en **tabel 5.3**
  (KCO2, §5.5.6, p. 96). Alle crate-doc-verwijzingen "bijlage Z/AB → tabel 5.2/5.3"
  bijgewerkt (doc-only + de twee `EpError`-messagestrings). De grep-anchor-const-namen
  (`NTA_8800_2025_BIJLAGE_Z/_AB`, `_TABEL_Z1/_AB1`) blijven ongewijzigd; alleen hun
  doc-comment is gecorrigeerd (audit-traceability behouden).
