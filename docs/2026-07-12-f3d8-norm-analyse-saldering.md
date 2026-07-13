# F3d-8 — Norm-analyse PV-saldering (BENG 2): waarom de EP-tak niet "over-netteert"

**Datum:** 2026-07-12 · **Normversie:** NTA 8800:2025+C1:2026 · **Bron-PDF:**
`Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\NTA 8800_2025+C1_2026 nl.pdf`
(PyMuPDF-extractie; paginanummers = PDF-paginalabel rechtsonder).

## 0. Kernconclusie (TL;DR)

| Vraag | Antwoord |
|---|---|
| Begrenst NTA 8800:**2025+C1** de PV-aftrek voor BENG 2? | **Nee.** §5.5.2 salderert PV-export **volledig** tegen `fP;exp;el = 1,45`. EPTot mág negatief. |
| Was de F3a-lezing ("maandonafhankelijk, EPTot = Σafname·1,45 − PV·1,45") fout? | **Nee — wiskundig correct** voor 2025+C1. De engine (−8,2 Gouda) is **norm-conform**. |
| Waarom geeft certified Uniec dan 27,48 i.p.v. ~0/negatief? | Uniec 3.3.x is een **oudere-norm-tool** die maar **~64 % van de PV** verrekent (maand-directgebruik-fractie), niet 100 %. Dit is een **normversie-verschil**, geen engine-bug. |
| Actie op de EP-crate? | **Geen** (anti-fudge). De flat `pv_yield × 1,45`-aftrek is exact gelijk aan de norm-maandmatching (bewijs §3). De Gouda/Aalten-BENG 2-golden blijft `#[ignore]` met normversie-motivatie. |

De F3d-4-hypothese ("EP-tak salderert op jaarbasis i.p.v. de norm-maandmatching")
is **onjuist**: onder 2025+C1 is jaarbasis ≡ maandmatching (identiteit §3). Maandmatching
implementeren verandert **niets** aan het 2025+C1-resultaat.

---

## 1. De volledige formulestructuur §5.5.2–5.5.4 (formules 5.9–5.27)

### 1.1 Jaar- en maandsom (5.9/5.10, p. 85)

```
EPTot      = Σ_mi EPTot;mi                                                        (5.9)
EPTot;mi   = Σ_ci EP;del;ci − Σ_gi EP;exp;T;gi − Σ_gi EP;exp;el;gi
             − Σ EP;pr;nEPus;el − EP;BAT,out;tot                                  (5.10)
```

Per energiedrager (5.11–5.14, p. 85):

```
EP;del;ci      = EEPdel;ci × fP;del;ci                                            (5.11)
EP;exp;el;gi   = Eexp;el;gi × fP;exp;el;gi                                        (5.13)
EP;pr;nEPus;el = Epr;nEPus;el × fP;pr;us;el                                       (5.14)
EP;BAT,out;tot = Min[Epr;el;ren;tot; EEPus;el] × 0,05 × fBAT;cor                  (5.14a)
```

`fBAT;cor = 1` alleen bij ≥ 5 kWh opslag (p. 87); Gouda/Aalten hebben geen batterij → term = 0.

### 1.2 Afgenomen elektriciteit (5.15, p. 87)

```
EEPdel;el = EEPus;el − Epr;EPus;el                                                (5.15)
```

`EEPus;el` = maandelijkse EP-relevante elektriciteit (5.20, §5.5.3): verwarming +
bevochtiging + ventilatoren + verlichting + koeling + ontvochtiging + tapwater +
hulpenergie. **Woningbouw:** `EL;ci = 0` (OPMERKING 3, p. 90) en huishoudelijk verbruik
zit **niet** in `EEPus;el`.

### 1.3 Zelfgebruik en export (5.22–5.27, p. 91–93) — hier zit de begrenzing

**Zelfgebruik-fractie (5.22/5.23):** met `EnEPus;el = 0` (verplicht 0 W/m² voor de
EP-indicatoren, formule 5.27, p. 93):

```
Epr;EPus;el  = Epr;us;el × EEPus;el / (EEPus;el + EnEPus;el) = Epr;us;el
Epr;nEPus;el = Epr;us;el × EnEPus;el / (EEPus;el + EnEPus;el) = 0
```

**Zelfgebruik met bovengrens (5.24/5.25, p. 92):**

```
Epr;us;el = Σ_gi Epr;el;gi         met bovengrens   Epr;us;el ≤ (EEPus;el + EnEPus;el)
```

⇒ **`Epr;us;el = Min(PV, EEPus;el)`** — het zelfgebruik is **maandelijks begrensd op de
EP-elektriciteitsvraag**. Dít is de begrenzing die F3a niet expliciet noemde.

**Export (5.26, p. 93):** voor één bron (Σ Epr;el;gi = PV):

```
Eexp;el;gi = Epr;el;gi − (Epr;el;gi / Σ Epr;el;gi) × Epr;us;el
           = PV − Epr;us;el = PV − Min(PV, EEPus;el) = Max(0, PV − EEPus;el)
```

**Cruciaal:** de export-term wordt in (5.10) **afgetrokken** tegen `fP;exp;el = 1,45`
(tabel 5.2). De begrenzing op zelfgebruik verschuift de PV dus alleen van "del-aftrek"
naar "exp-aftrek" — **beide tegen 1,45** — en verandert het totaal niet (§3).

### 1.4 De factoren (tabel 5.2, §5.5.5, p. 93–94)

| Elektriciteit | Symbool | Waarde |
|---|---|---|
| Aangeleverd (net) | `fP;del;el` | **1,45** |
| Zelf-gebruikt eigen productie | `fP;pr;us;el` | **1,45** |
| Geëxporteerd | `fP;exp;el` | **1,45** |

Bevestigt de F3a-lezing. `EPTot;mi` mág negatief worden bij PV-overschot (OPMERKING 11,
p. 87) — **geen clamp**.

---

## 2. Doorgerekend Gouda-voorbeeld (2467)

Ag = 133,1 m². Certified Uniec (`expected.json`):

| Post (primair, kWh) | Waarde |
|---|---|
| Verwarming | 6506 |
| Tapwater | 4208 |
| Koeling | 244 |
| Ventilatoren | 822 |
| **Σ functies** | **11 780** |
| PV-opbrengst (kWh el) | 8734 |
| BENG 2 certified | 27,48 kWh/m² → **EPTot = 27,48 × 133,1 = 3658 kWh** |

**Impliciete PV-verrekening certified** = 11 780 − 3658 = **8122 kWh primair**
= 8122 / 1,45 = **5601 kWh elektriciteit** = **64,1 % van 8734 kWh PV**.

**Wat 2025+C1 (volledige saldering) zou geven:** 8734 × 1,45 = 12 664 primair
→ EPTot = 11 780 − 12 664 = **−884 kWh** → BENG 2 = **−6,6** (engine: −8,2, verschil = demand-gaps).

Het gat = 3658 − (−884) = 4542 kWh primair = **3133 kWh × 1,45** = **exact de geëxporteerde
PV** (8734 − 5601). Certified crediteert die export **niet**; 2025+C1 wél.

### 2.1 Aalten (2522) — dezelfde vingerafdruk

Ag = 67,0 m². Σ functies = 2551 + 1813 + 422 + 443 = **5229**. BENG 2 = 24,71 × 67,0
= **1656 kWh**. PV-verrekening = 5229 − 1656 = 3573 primair = 2464 kWh el = **64,6 % van
3811 kWh PV**. Volledige saldering → EPTot = 5229 − 5526 = −297 → BENG 2 = −4,4.

**Twee onafhankelijke certified-cases crediteren ~64 % van de PV** (alleen zelfgebruik).
Dat is de handtekening van een **maand-directgebruik-fractiemodel**, niet van volledige
saldering. Volledige saldering zou beide BENG 2's **negatief** maken.

---

## 3. Bewijs: jaarbasis ≡ maandmatching onder 2025+C1

Netto elektriciteits-bijdrage per maand aan `EPTot;mi` (enige drager voor all-electric,
`EnEPus = 0`, geen batterij):

```
netto_el;mi = EP;del;el;mi − EP;exp;el;mi
            = Max(0, EEPus;mi − PV;mi)·1,45 − Max(0, PV;mi − EEPus;mi)·1,45
```

Met de identiteit `Max(0,a−b) − Max(0,b−a) = a − b`:

```
netto_el;mi = (EEPus;mi − PV;mi) · 1,45
```

Jaarsom = `(Σ EEPus;mi − Σ PV;mi)·1,45 = (EEPus;jaar − PV;jaar)·1,45`.

**De maandelijkse matching valt exact weg** zolang `fP;del;el = fP;exp;el = 1,45` (tabel 5.2)
en er geen batterij ≥ 5 kWh is. De engine-implementatie `EPTot = Σ(functies × fP) −
PV × 1,45` is hiermee **identiek** aan de norm-maandmatching. Maandprofielen toevoegen aan
`EpInputs` verandert het 2025+C1-resultaat **niet** (het zou alleen nodig zijn voor een
batterij-correctie (5.14a) of het ZEB-model, §4).

---

## 4. Waar het ~64 %-model wél in de norm staat: bijlage AB (ZEB, informatief)

NTA 8800:2025+C1 bevat het **directgebruik-fractiemodel** dat certified Uniec benadert —
maar **uitsluitend** voor de **nieuwe, experimentele ZEB-indicator** `EweP,ZEB;Tot`
(bijlage AB, **informatief**, EPBD IV), **niet** voor BENG 2.

Bijlage AB.0 (p. 1148) stelt het letterlijk:

> "Een ander verschil met de primaire-fossiele-energie-indicator EwePTot is dat in
> EweP,ZEB;Tot **niet meer volledig gesaldeerd wordt** voor lokaal opgewekte elektrische
> energie."

Het woord **"niet meer volledig gesaldeerd"** (t.o.v. EwePTot) bevestigt dat de reguliere
BENG 2 (`EwePTot`, §5.3.1.2/§5.5) **wél** volledig salderert.

Het ZEB-model (AB.2.3.2, p. 1153):

```
Epr;el,ren;directuse = Min[fdu;el,ren × Epr;el,ren;tot ; 0,3 × EEPus;el]     (AB.65)
   met bovengrenzen  ≤ EEPus;el  en  ≤ Epr;el,ren;tot                         (AB.67/68)
```

met maand-fracties `fdu;el,ren` (tabel AB.1, p. 1154): woningbouw jan/feb/nov/dec 0,75 …
jul/aug 0,15. Export krijgt **weegfactor 1** en `fP,ZEB;del;el = 1,35`, `fP,ZEB;exp;el,ren = 1`
(tabel AB.2, p. 1156) — **andere factoren** dan BENG 2. Dit is het mechanisme dat een
begrensd zelfgebruik (~50–65 % voor een warmtepompwoning) oplevert en dus 27,48 i.p.v.
−6,6 verklaart — maar het hoort bij een **andere indicator**.

---

## 5. Gevolg voor de goldens en de EP-crate

1. **Geen EP-crate-wijziging.** De 2025+C1-saldering is correct geïmplementeerd
   (`ep_score.rs::total_primary_energy_mj`, `primary_energy.rs`). Het ~64 %-model in de
   EP-tak proppen zou (a) 2025+C1 **schenden**, (b) anti-fudge schenden (golden gerekend
   onder oudere norm), en (c) onder correcte 2025+C1-wiskunde **niets** doen tenzij de
   export-term wordt weggelaten — dat is een **niet-2025+C1-regel fabriceren**.

2. **Golden blijft `#[ignore]`.** De BENG 2-afwijking Gouda/Aalten is (deels) een
   **normversie-saldering-verschil**, geen engine-bug. Bovendien faalt BENG 1 sowieso
   (−26 %/−37 % demand-onderschatting) — activering hangt niet alleen aan de saldering.

3. **Wil je certified Uniec tóch reproduceren?** Implementeer bijlage AB als **losse,
   additieve** `EweP,ZEB;Tot`-indicator (eigen factoren + tabel AB.1 + batterij), naast —
   niet in plaats van — de volledig-salderende BENG 2. Dat is een 2025+C1-gesanctioneerde
   informatieve indicator en fudge-vrij. **Buiten F3d-8-scope**; besluit bij PM.

---

## 6. Correctie op eerdere framing

- **F3a §2.3 is niet weerlegd.** De "maandonafhankelijkheid" is wiskundig juist voor
  2025+C1 (§3). De empirische afwijking t.o.v. certified is een **normversie-artefact**,
  geen bewijs dat F3a fout zat. Zie het ⚠️-blok in
  `docs/2026-07-11-f3a-norm-analyse-ep.md` §2.3.
- **F3d-4-README's** ("EP-tak salderert op jaarbasis i.p.v. norm-maandmatching") —
  herkaderd: onder 2025+C1 zijn jaarbasis en maandmatching **identiek**; het echte verschil
  is volledig (2025+C1) vs. partieel (certified/ouder) salderen.

---

## 7. F3d-8b — bijlage-AB ZEB-indicator geïmplementeerd + gemeten (13-07)

§4/§5.3 stelde voor de bijlage-AB ZEB-indicator (`EweP,ZEB;Tot`) als **losse,
additieve** informatieve output te bouwen — niet in het BENG-rekenpad, wél om te
toetsen of het directgebruik-fractiemodel de certified ~64 %-PV-credit reproduceert.
Dat is nu gedaan (`crates/openaec-project-shared/src/beng/zeb.rs`, additief veld
`BengResult.zeb_indicator`, wiring in `compute_beng`). Zie `zeb_measure` voor de
meting.

### 7.1 Implementatie (all-electric + PV, geen batterij/WKK)

Maandmodel AB.9/AB.10 met (dropping index mi):

```
directuse   = Min[fdu;el,ren × PV ; 0,3 × EEPus;el]   (AB.65, bovengrenzen AB.67/68)
EEPdel,ZEB  = EEPus;el − directuse                     (AB.15)
Eexp;el,ren = PV − directuse                            (AB.61)
EP,ZEB;mi   = EEPdel,ZEB × 1 × 1,35 − Eexp;el,ren × 1 × 1   (AB.11a/AB.13/AB.10)
EweP,ZEB    = ⌈Σmi EP,ZEB;mi / A_g⌉0,01                 (AB.9/AB.1)
```

Factoren uit **tabel AB.2** (p. 1156): `fP,ZEB;del;el = 1,35`, `fP,ZEB;weeg;el = 1`,
`fP,ZEB;exp;el,ren = 1`, direct gebruik = factor 0. Maand-fracties `fdu;el,ren` uit
**tabel AB.1** (p. 1153): woningbouw jan/feb/nov/dec 0,75 … jul/aug 0,15. Batterij
(AB.2.3.3) en WKK (`Epr;el,nren`) niet gemodelleerd → termen exact 0.

### 7.2 Meetresultaat (bridged geometrie, F6)

| Case | A_g | BENG 2 (2025+C1, volledig salderen) | ZEB-indicator (bijlage AB) | Certified Uniec | ZEB-delta | ZEB-zelfgebruik |
|---|---|---|---|---|---|---|
| Gouda 2467 | 133,1 | 8,90 | **20,82** | 27,48 | **−24,2 %** | 26,4 % |
| Aalten 2522 | 67,0 | 22,61 | **31,77** | 24,71 | **+28,6 %** | 27,3 % |

### 7.3 Conclusie — bijlage AB reproduceert certified óók niet

De hypothese uit §4 ("het ~64 %-directgebruik-model verklaart de certified 27,48")
is **empirisch weerlegd** voor de 2025+C1-bijlage-AB-parametrisatie:

1. **Zelfgebruik is ~26–27 %, niet ~64 %.** De `0,3 × EEPus;el`-cap (AB.65) domineert
   in de zomer (juli: `fdu = 0,15`, lage vraag, hoge PV → directgebruik ≈ 5 % van de
   PV). Het certified ~64 %-zelfgebruik uit §2 hoort bij een **ander**
   (ouder-norm-)directgebruik­model, niet bij tabel AB.1 + de 0,3-cap.
2. **De ZEB-indicator ligt niet consistent t.o.v. certified**: Gouda −24 %,
   Aalten +29 % (tegengesteld teken). De lagere ZEB-factoren (1,35/1 i.p.v. 1,45)
   verlagen; het lage zelfgebruik verhoogt t.o.v. volledige saldering — de netto
   uitkomst hangt af van de PV/vraag-verhouding en valt per case anders uit.
3. **Certified 27,48 / 24,71 is dus noch de 2025+C1-BENG 2 (8,90 / 22,61) noch de
   2025+C1-ZEB-indicator (20,82 / 31,77).** Het is een ouder-norm partieel-
   salderingsartefact met eigen parameters (Uniec 3.3.x), niet reproduceerbaar met
   één 2025+C1-grootheid.

### 7.4 Gevolg voor de goldens

- **Golden blijft `#[ignore]`** (anti-fudge): geen 2025+C1-grootheid haalt certified
  binnen tolerantie, dus niets om tegen te activeren zonder de fixture te fudgen. De
  `#[ignore]`-redenen van `uniec_gouda_2467` / `uniec_aalten_2522` /
  `gouda_beng_geometry_within_certified_tolerance` dragen nu de gemeten ZEB-gap.
- **De ZEB-indicator zelf is wél live** als additieve, norm-gereferentieerde output
  op elk `compute_beng`-resultaat (byte-additief: `#[serde(default,
  skip_serializing_if)]`), plus een transparantie-note in `BengResult.notes`.
