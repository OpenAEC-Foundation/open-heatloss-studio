# ISSO 51:2023 Erratum — verificatie van tabel-nummers en formules

**Datum:** 2026-05-12
**Bron:** `C:/GitHub/warmteverliesberekening/tests/references/erratum-isso51-2023.pdf` (12 pagina's, ISSO publicatiedatum 1 september 2023)

_Redactie 2026-07-02: letterlijke norm-/erratum-transcripties vervangen door verwijzingen (auteursrecht ISSO); volledige transcripties lokaal bij 3BM._

> **Methodologische voorbehoud.** Dit document is een **erratum** — een lijst van wijzigingen op de basis-publicatie ISSO 51:2023. Het bevat alleen patches op specifieke tabellen, niet de volledige basis-tabellen. Tabel 2.12, 2.14, 2.16 en 2.18 worden volledig vervangen en zijn dus letterlijk overgenomen. Tabel 2.6 en 2.8 krijgen alleen tekstuele puntcorrecties — de waardes-rijen in die tabellen staan **niet in dit PDF** en moeten uit de basis-publicatie komen.

---

## 1. Tabel-mapping infiltratie (Issue C)

### 1.1 Tabel-verwijzingen die in het erratum voorkomen

| Tabel | Onderwerp (zoals genoemd in erratum) | Vindplaats erratum |
|---|---|---|
| 2.6 | bevat grootheid `qv,10,spec` ("toelichting q onder tabel" op p.2) | p.2 — alleen tekst-wijziging: "vervang constructeur door energieadviseur" |
| 2.8 | bevat grootheid `qi;spec` ("specifieke luchtvolumestroom infiltratie per m² gebruiksoppervlak", expliciet in Formule E.5 op p.12) | p.2 — "verwijder in omschrijving de tekst: afhankelijk van het bouwjaar/renovatiejaar"; p.12 — verwezen vanuit Formule E.5 |
| 2.9 | volumestromen ventilatie (verwijzing in formule 4.3 op p.8) | p.8 |
| 2.10 | luchtvolumestroom ventilatie-eisen (p.9) | p.9 |
| 2.11 | ontwerpbinnentemperatuur per ruimte-type (p.2) | p.2 — Bergruimte-voetnoot wijziging |
| 2.12 | Δθ1, Δθa1, Δθ2, Δθa2, Δθv per verwarmingssysteem | p.3 — VOLLEDIG vervangen |
| 2.14 | θt bij WTW/voorverwarming | p.4 — VOLLEDIG vervangen |
| 2.16 | c_z aangrenzende woningen | p.4 — alleen tekst-wijziging |
| 2.17 | (Rc-gerelateerd, vloerverwarming) | p.5 — voetnoot |
| 2.18 | f_wvw per Rc-klasse wand | p.5 — VOLLEDIG vervangen |
| 3.1 | fractie z per gebouwtype | p.6 — **VERWIJDERD** (volledige inhoud aanwezig vóór verwijdering) |
| 4.1 | fractie z per vertrek-type | p.8 — **VERWIJDERD** (volledige inhoud aanwezig vóór verwijdering) |

### 1.2 qi,spec-tabel — definitieve toewijzing

Geverifieerd tegen erratum p.12, formule E.5 onder Par. E.2.2: de symboolverklaring bij E.5 definieert `q_i;spec` als specifieke luchtvolumestroom infiltratie per m² gebruiksoppervlak **conform tabel 2.8** (eenheid dm³/(s·m²)). De rekenrelatie is `H_i = 1,2 · q_i;spec · z · ΣA_g`. Dit koppelt qi,spec definitief aan Tabel 2.8, niet aan Tabel 2.6.

**Conclusie tabel-toewijzing:**

| Vraag | Antwoord |
|---|---|
| Welke tabel bevat `qi,spec`? | **Tabel 2.8** (eenheid dm³/(s·m²)) |
| Wat bevat Tabel 2.6? | `qv,10,spec` — een **andere** grootheid (gerelateerd aan luchtdoorlatendheid bij 10 Pa drukverschil, niet de specifieke infiltratie-volumestroom voor warmteverlies-berekening) |
| Audit-claim "Tabel 2.8 bevat qi_spec" | **CORRECT** — onze code-implementatie verwijst naar de juiste tabel |
| Zoekagent-claim "infiltratie zit in Tabel 2.6" | **INCORRECT** — Tabel 2.6 bevat `qv,10,spec` (een verwante maar andere grootheid). De agent verwarde de twee q-symbolen |

### 1.3 qi,spec waardes — KAN NIET UIT ERRATUM WORDEN GEVERIFIEERD

Erratum p.2 wijzigt alleen de omschrijving van Tabel 2.8 ("verwijder: afhankelijk van het bouwjaar/renovatiejaar"). De waarderijen van Tabel 2.8 (de eigenlijke klassen 0.04/0.08/0.12/0.16) staan **niet in dit erratum** en blijven dus zoals in de basis-publicatie ISSO 51:2023.

**Implicatie voor onze 4-klassen-implementatie** (`crates/isso51-core/src/tables/infiltration.rs:34-45`):

| Klasse | Onze waarde [dm³/(s·m²)] | Erratum-tabel werkelijke waarde |
|---|---|---|
| 1 | 0.04 | NIET in erratum → moet uit basis-publicatie ISSO 51:2023 worden geverifieerd |
| 2 | 0.08 | idem |
| 3 | 0.12 | idem |
| 4 | 0.16 | idem |
| 5? (0.32 conform Vabi-fixture) | (niet aanwezig in onze code) | idem — kan niet uit dit PDF worden bevestigd of weerlegd |

### 1.4 Conclusie Issue C

- **Onze code wijst naar het juiste tabel-nummer (2.8).** Audit-claim staat overeind op dat punt.
- **De qi,spec-waardes** (4 vs 5 klassen, max 0.16 vs 0.32) **kunnen uit dit erratum NIET worden geverifieerd** — die staan in de basis-publicatie ISSO 51:2023, niet in het erratum. De 50% afwijking met Vabi blijft een **open vraag** tot we de basis-tabel hebben.
- **Aanbeveling:** raadpleeg de basis-publicatie ISSO 51:2023 (niet aanwezig in repo) of de officiële DR Engineering/Vabi documentatie om de exacte klassen-tabel vast te stellen.

---

## 2. Formule 3.11 — kwadratische sommatie

### 2.1 Bevinding (erratum p.7, Par. 3.5.2)

_Geverifieerd tegen erratum ISSO 51:2023 Par. 3.5.2, p.7._ Het erratum voegt het gelijkteken toe aan formule 3.11 en bevestigt de **kwadratische sommatie** `Φ_extra = √(Φ_vent² + Φ_T,iaBE² + Φ_hu,i²)` op gebouwniveau (§3.5.2, "warmteverliezen die niet altijd of niet gelijktijdig optreden").

### 2.2 Op welk niveau van toepassing?

| Niveau | Vindplaats in erratum | Conclusie |
|---|---|---|
| Gebouwniveau (Hoofdstuk 3) | Par. 3.5.2 — formule 3.11 expliciet **wel** | Formule 3.11 hoort bij **Hoofdstuk 3 (gebouw)** |
| Vertrekniveau (Hoofdstuk 4) | Par. 4.5.2 — verwijzingen naar Φ_vent / Φ_T,iaBE / Φ_hu, maar **geen eigen formule-nummer** voor kwadratische sommatie. Erratum past de _verwijzingen onder formule 4.22_ aan, dus er IS een formule 4.22 op vertrekniveau (kwadratische sommatie analoog) | Hoofdstuk 4 (vertrek) heeft analoge formule (4.22 of nabij) — exacte vorm niet in erratum |

### 2.3 Conclusie

- **Formule 3.11 = gebouwniveau** is letterlijk juist (Par. 3.5.2).
- Vertrekniveau heeft een eigen analoge formule (rond 4.22), niet expliciet hernoemd in erratum.
- **Audit-claim "gebouwniveau 3.11 ontbreekt in onze implementatie"** — als de implementatie alleen vertrek-sommatie heeft en geen gebouw-aggregatie, is dat een terecht bezwaar. Erratum bevestigt dat de norm dit op beide niveaus expliciet onderscheidt.

---

## 3. Formule 3.3 — Φ_vent = Φ_v − Φ_i

### 3.1 Bevinding (erratum p.7, Par. 3.2.3)

_Geverifieerd tegen erratum ISSO 51:2023 Par. 3.2.3, p.7._ Het erratum wijzigt formule 3.3 naar `Φ_vent = Φ_v − Φ_i` (gebouwniveau) en verlegt de paragraaf-verwijzing van 2.5.6 naar 3.2.1.

### 3.2 Vergelijkbare bepaling op vertrekniveau (p.9)

_Geverifieerd tegen erratum ISSO 51:2023 Par. 4.2.2, p.9._ In de context "centrale mechanische afvoer en toevoer via de verblijfsruimten" geldt op vertrekniveau `Φ_vent = Φ_v` (geen aftrek).

### 3.3 Op welk niveau?

| Niveau | Formule | Bron |
|---|---|---|
| **Gebouwniveau (§3.2.3)** | `Φ_vent = Φ_v − Φ_i` | p.7, expliciet Par. 3.2.3 |
| **Vertrekniveau (§4.2.2)** | `Φ_vent = Φ_v` (in centrale-mech-systeem context) | p.9, Par. 4.2.2 |

### 3.4 Conclusie

- **Audit-claim "Φ_vent = Φ_v − Φ_i is een gebouwniveau-correctie"** wordt door erratum bevestigd. Op gebouwniveau wordt het infiltratie-deel afgetrokken van het ventilatie-deel om dubbeltelling te voorkomen.
- Op vertrekniveau ontbreekt deze aftrek (in een specifiek system-type), waardoor de getalsmatige uitkomst van een vertrek-sommatie systematisch hoger uitkomt dan de gebouwberekening.
- **Verklaart een deel van de 50% Vabi-afwijking** als onze implementatie alleen vertrek-sommeert.

---

## 4. Wat de audit moet aanpassen

Concreet voor `docs/2026-05-12-isso51-norm-conformiteit-audit.md`:

| Audit-claim | Status na erratum-verificatie | Voorgestelde nieuwe tekst |
|---|---|---|
| "Tabel 2.8 bevat qi_spec" | **CORRECT** — bevestigd via Formule E.5 in erratum p.12 | Behouden |
| "Tabel 2.6 bevat infiltratie-volumestroom" (zoekagent-claim) | **INCORRECT** — Tabel 2.6 bevat `qv,10,spec` (luchtdoorlatendheid bij 10 Pa) | Verwijderen of corrigeren naar Tabel 2.8 |
| "qi_spec heeft 4 klassen (0.04–0.16)" | **NIET TE VERIFIËREN UIT ERRATUM** | Toevoegen disclaimer: "exacte klassen vereist verificatie tegen basis-publicatie ISSO 51:2023; niet aanwezig in erratum-PDF" |
| "Formule 3.11 = kwadratische sommatie op gebouwniveau" | **CORRECT** — Par. 3.5.2 erratum p.7 | Behouden |
| "Formule 3.3 (Φ_vent = Φ_v − Φ_i) is gebouwniveau-correctie" | **CORRECT** — Par. 3.2.3 erratum p.7 | Behouden, evt. uitbreiden met bronverwijzing (geen letterlijk citaat) |
| "Onze code mist gebouwniveau-aggregatie" | **Ondersteund door norm** — erratum bevestigt expliciet onderscheid §3.x (gebouw) vs §4.x (vertrek) | Behouden |

---

## 5. Onbeantwoorde vragen

1. **Tabel 2.8 waardes** — Het erratum toont alleen wijzigingen op de _omschrijving_ van Tabel 2.8, niet de waarderijen. Of er werkelijk 4 of 5 klassen zijn en wat de max-waarde is (0.16 of 0.32) kan **alleen uit de basis-publicatie ISSO 51:2023** worden beantwoord. Deze publicatie is **niet aanwezig** in `tests/references/`.
2. **Formule 4.22 (vertrek-equivalent van 3.11)** — Erratum verwijst naar deze formule (p.11: "Wijzig de verwijzing in de toelichting onder formule 4.22") maar geeft de exacte vorm niet weer. Aanname: analoog aan 3.11 met `Φ_extra,vertrek = √(...)`, maar dit is **niet bevestigd**.
3. **Tabel 2.6 volledige inhoud** — Erratum patcht alleen "vervang constructeur door energieadviseur" in toelichting. De grootheid `qv,10,spec` wordt genoemd, maar inhoud (waarden, klassen) staat alleen in basis-publicatie.
4. **Verband qv,10,spec ↔ qi,spec** — Of `qv,10,spec` (Tabel 2.6, bij 10 Pa) wordt omgerekend naar `qi,spec` (Tabel 2.8, ontwerp-conditie) via een formule of via aparte klassen-tabel is **niet expliciet in erratum** beschreven. Mogelijk relevant voor begrip Vabi-fixture (mogelijk gebruikt Vabi `qv,10,spec` × correctiefactor in plaats van directe Tabel 2.8 lookup).

**Aanbeveling vervolg:** verifieer de basis-publicatie ISSO 51:2023 (papier of digitaal via ISSO-portaal) op de volledige inhoud van Tabel 2.6 én 2.8 en op de exacte formulering van formule 4.22.
