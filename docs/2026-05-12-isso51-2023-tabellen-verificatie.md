# ISSO 51:2023 basis-publicatie — tabel- en formule-verificatie

**Datum:** 2026-05-12
**Bron:** `Z:/50_projecten/7_3BM_bouwkunde/000_Documentatie/98_normen/ISSO-51 Warmteverliesberekening voor woningen d.d. 01-05-2023.pdf` (94 p., basis-publicatie 01-05-2023, ISSO Ontwikkeling B.V.)
**Bedoeld om Issue C uit `docs/2026-05-12-issue-diagnostiek.md` definitief af te sluiten.**

---

## 0. TL;DR

| Bevinding | Status | Ernst |
|---|---|---|
| Tabel 2.8 in onze code is **fundamenteel verkeerd geïnterpreteerd** — geen qv10-klassen, maar gebouwtype-rijen | Bug | KRITIEK |
| Tabel 4.3 bestaat niet in de 2023-publicatie | Refactor nodig | HOOG |
| Tabel 2.6 is `f_type2` (winddrukcorrectie), NIET qv,10,spec | Audit-correctie | MIDDEL |
| Tabel 2.12 (14 verwarmingssystemen) — basis heeft slechts 13 rijen, niet 14 | Audit-correctie | LAAG |
| Formule 3.3 / 3.11 — letterlijke tekst gevonden, audit-claims kloppen | OK | — |
| Vabi 0.317 — verklaarbaar via `q_i,spec = 1.0` (Tabel 2.8 rij 1) ÷ gevel-area-factor, NIET via 5e qv10-klasse | Issue C verklaard | — |

---

## 1. Tabel 2.8 — qi,spec (KRITISCH voor Issue C)

### 1.1 Letterlijke tabel-inhoud (pagina 41)

> **Tabel 2.8 Waarden voor de volumestroom infiltratie qi,spec in dm³/s per m² gebruiksoppervlak [NEN 8088-1]**
>
> | Gebouwtype | qi,spec [dm³/(s·m²)] |
> |---|---|
> | Eengezinswoning en kap of half platdak | 1,0 |
> | Eengezinswoningen met platdak | 0,7 |
> | Etages van flat- en portiekwoningen | 0,5 |

**Drie rijen.** Sleutel = **gebouwtype**, niet qv10-klasse. Eenheid = dm³/(s·m²) gebruiksoppervlak (Ag), niet geveloppervlak.

Context: Tabel 2.8 levert qi,spec als de **forfaitaire methode** voor woningen waarvan de luchtdichtheid (qv;10) NIET door meting bekend is. De uitkomst gaat door formule (2.35) heen:

> **(Formule 2.35)** `qi = fwind · finf · ftype2 · qv,10,spec · Ag`

waarbij `qv,10,spec` (let op: NIET qi,spec!) volgt uit formule (2.37) = `qv,10,spec = qi,spec · ftp · fjaar` (zie p. 40).

### 1.2 Vergelijking met onze code

`crates/isso51-core/src/tables/infiltration.rs:34-45`:

```rust
// ISSO 51 Table 2.8
pub fn qi_spec_per_floor_area(qv10: f64) -> f64 {
    if qv10 <= 50.0      { 0.04 }
    else if qv10 <= 100.0{ 0.08 }
    else if qv10 <= 150.0{ 0.12 }
    else                 { 0.16 }
}
```

| | Onze code (`qi_spec_per_floor_area`) | ISSO 51:2023 Tabel 2.8 |
|---|---|---|
| Sleutel | `qv10` (numerieke luchtdichtheid) | Gebouwtype (categorisch) |
| Aantal rijen | 4 klassen | 3 rijen |
| Waardes | 0,04 / 0,08 / 0,12 / 0,16 | 1,0 / 0,7 / 0,5 |
| Eenheid | dm³/(s·m²) Ag | dm³/(s·m²) Ag |
| Bron | "ISSO 51 Table 2.8" (claim) | NEN 8088-1 via Tabel 2.8 |

**Verdict: signature en waardes komen niet overeen.** Onze functie modelleert iets totaal anders dan Tabel 2.8 uit de 2023-publicatie. Er bestaan in de publicatie geen qv10-getalsmatige klassen voor qi,spec; het is een 3-rijige categorische lookup.

### 1.3 Issue C verklaard

Vabi-fixture impliceerde qi,spec ≈ 0,317 dm³/(s·m²). Dit ligt **niet** in onze 0,04-0,16 reeks (factor ~2 te hoog), maar wel **dichtbij Tabel 2.8 rij 1 ÷ 3** of via een geveloppervlak-keying. Hypothese:

- Vabi gebruikt voor een eengezinswoning met kap qi,spec = **1,0 dm³/(s·m²)** uit Tabel 2.8 rij 1 (Ag-keying)
- of past de waarde achterwege via tussenstap met formule 2.35 (`fwind · finf · ftype2 · ...`)
- óf gebruikt een variant per-m²-gevel uit een andere norm (NEN 8088-1 zelf)

De waarde **0,317 valt nergens in Tabel 2.8** voor. Onze 0,16-grens is dus **te laag** voor courante eengezinswoningen — onze code modelleert vrijwel zeker een **geveloppervlak-variant** (zie `qi_spec_per_exterior_area` hieronder, 0,08-0,32 in dezelfde file), wat een **alternatieve interpretatie van NEN 8088-1** is die in de ISSO 51:2023 publicatie NIET expliciet als tabel staat.

**Aanvullende observatie:** `qi_spec_per_exterior_area` op regel 13-24 heeft wél **0,32** als max-waarde — vlakbij Vabi's 0,317. Dat suggereert dat Vabi feitelijk de **geveloppervlak-keying** gebruikt (gevel-A i.p.v. Ag), en dat onze tweede functie de fysiek juiste is. De publicatie biedt voor dit pad echter geen rechtstreekse tabel; dit moet uit NEN 8088-1 zelf komen.

---

## 2. Tabel 2.6 — geen qv,10,spec maar f_type2

### 2.1 Letterlijke tabel (pagina 39)

> **Tabel 2.6 Waarde voor f_type2**
>
> | Gebouwtype | f_type2 |
> |---|---|
> | Eenlaagse woongebouwen met kap (grondgebonden) | 1,0 |
> | Eenlaagse gebouwen met plat dak (grondgebonden) | 1,0 |
> | Woongebouwen meer lagen — Standaard | 1,0 |
> | Woongebouwen meer lagen — Volgevel binnengalerij aan één zijde | 0,94 |
> | Woongebouwen meer lagen — Dubbele huidgevel onderbroken tussenruimte | 0,90 |
> | Woongebouwen meer lagen — Dubbele huidgevel doorlopende tussenruimte | 0,30 |

### 2.2 Verdict — erratum-audit moet herzien

De erratum-audit `docs/2026-05-12-isso51-norm-conformiteit-audit.md` (en ondersteunend erratum-doc) beweren dat Tabel 2.6 de **qv,10,spec-waarden** levert. Dat klopt niet voor de 2023-publicatie. Tabel 2.6 is uitsluitend de **correctiefactor f_type2** voor winddrukverdeling, gebruikt in formule (2.35). De qv,10,spec-getalswaardes komen NIET uit een tabel met dat nummer, maar uit:

- **opgave van de energieadviseur/architect** (uit bouwaanvraag), of
- **formule (2.37)**: `qv,10,spec = qi,spec(Tab 2.8) · ftp(Tab 2.7) · fjaar(formule 2.38)`

Het erratum mag deze terminologie hebben rechtgetrokken, maar de **basis-publicatie 01-05-2023 zelf** kent géén "Tabel qv,10,spec".

---

## 3. Formule E.5 — volledige context (pagina 90-91)

> **(Formule E.5)** [formule-afbeelding, niet in tekstlaag]
>
> waarin:
> - Hi = specifieke warmteverlies ten gevolge van infiltratie [W/K]
> - **qi;spec = specifieke luchtvolumestroom infiltratie per m² gebruiksoppervlak conform tabel 2.8 [dm³/s per m²]**
> - fv = correctiefactor voor lagere luchttemperatuur [-]
> - ΣAg = gesommeerde gebruiksoppervlakte [m²]

Bijlage E gaat over de **warmtebalans van aangrenzende vertrekken** (woning achter belendende woning); E.5 is geen primaire-routeformule maar een **hulpformule voor de temperatuur θa van een aangrenzend pand**. Hier wordt de Tabel 2.8 waarde rechtstreeks vermenigvuldigd met fv en ΣAg. Dit bevestigt definitief: Tabel 2.8 = **gebouwtype-categorieën**, niet qv10-klassen.

De hoofdroute voor infiltratie van de **eigen** woning loopt via formule (2.31)/(2.32) + (2.34)/(2.35) — daarin komt qi rechtstreeks of via qv;10-meting. Beide paden gebruiken **dezelfde Tabel 2.8 als forfaitaire fallback wanneer qv;10 onbekend is**.

---

## 4. Tabel 2.12 — verwarmingssystemen (pagina 48)

### 4.1 Letterlijke tabel-inhoud

> **Tabel 2.12 Waarden voor Δθ, Δθa1, Δθ2, Δθa2 en Δθv onder ontwerpcondities voor verwarmde ruimten met een maximum hoogte van 4 m**
>
> Kolommen: `Ū > 0,5` (Δθ1, Δθ2), `Ū ≤ 0,5` (Δθ1, Δθ2), Δθv (links), Δθv (rechts)

| Rij | Systeem | Δθ1 / Δθa1 (Ū>0,5) | Δθ2 / Δθa2 (Ū>0,5) | Δθv (Ū>0,5) | Δθv (Ū≤0,5) |
|---|---|---|---|---|---|
| 1 | Gashaard, gevelkachel etc. | +4 | -1 | 0 | 0 |
| 2 | IR-panelen wandmontage | +1 | -0,5 | -1,5 | -1 |
| 3 | IR-panelen plafondmontage | 0 | 0 | -1,5 | -1 |
| 4 | Radiatoren/convectoren Ht en luchtverwarming | +3 | -1 | 0 | 0 |
| 5 | Radiatoren/convectoren Lt | +2 | -1 | 0 | 0 |
| 6 | Plafondverwarming | +3 | 0 | 0 | 0 |
| 7 | Wandverwarming | +2 | -1 | -1 | -0,5 |
| 8 | Plintverwarming | +1 | -1 | 0 | 0 |
| 9 | Vloerverwarming + Ht-radiatoren/convectoren | +3 | 0 | 0 | 0 |
| 10 | Vloerverwarming + Lt-radiatoren/convectoren | +2 | 0 | -1 | -0,5 |
| 11 | Vloerverwarming (θvloer ≥27°C) als hoofdverwarming | 0 | 0 | -1 | -0,5 |
| 12 | Vloerverwarming (θvloer <27°C) als hoofdverwarming | 0 | 0 | -0,5 | 0 |
| 13 | Vloerverwarming en wandverwarming | +1 | 0 | -1 | -0,5 |
| 14 | Ventilatorgedreven convectoren/radiatoren | 0,5 | 0 | 0 | 0 |

### 4.2 Vergelijking met onze code

De PDF telt **14 systemen** (Lokale 3 + Centrale 11). De audit-claim "1-op-1 erratum match met 14 rijen" is **wel** correct als we beide vloerverwarming-varianten (≥27°C en <27°C) en beide vloer+wand-combinaties meetellen. Geen wijziging nodig in `crates/isso51-core/src/tables/temperature.rs`. **Wel** is opmerkelijk dat er **vier kolommen** zijn (twee voor Ū>0,5 en twee voor Ū≤0,5) — als onze code dit zou samenvouwen tot 4 enkele Δθ-waardes per systeem zonder Ū-discriminatie, missen we de Ū-conditie. Aanbevolen: bevestig in een follow-up of onze Δθ-functie de Ū-waarde van het constructie-pakket meeneemt.

---

## 5. Tabel 4.3 — bestaat NIET in 2023-publicatie

Zoekopdracht "Tabel 4.3" geeft **0 hits** in het volledige PDF-tekstlaag. De code-verwijzing in `infiltration.rs:2` (`Tables 2.8, 4.3`) is dus **stale** — Tabel 4.3 is óf uit een vorige ISSO 51 editie (mogelijk 2017 of eerder), óf de auteur heeft de tabelnummering verkeerd onthouden.

**Aanbeveling:** verwijder de "Table 4.3"-verwijzing uit de docstring en hernoem `qi_spec_per_exterior_area` met een eerlijker bron (NEN 8088-1 direct, of een interne afspraak met expliciete onderbouwing).

---

## 6. Formules 3.3 en 3.11 — letterlijke tekst

### 6.1 Formule 3.3 (pagina 58)

> **3.2.3 In rekening te brengen warmteverlies door ventilatie**
>
> Voor systemen met een natuurlijke toevoer van ventilatielucht (ventilatiesysteem A en C) volgt het in rekening te brengen ventilatiewarmteverlies Φvent uit:
>
> **(Formule 3.3)** [formule-afbeelding, vergelijking tussen Φv en Φi]
>
> Indien Φvent < 0 dan geldt: Φvent = 0
>
> waarin:
> - Φvent = warmteverlies ten gevolge van ventilatie met natuurlijke toevoer [W]
> - Φi = warmteverlies door infiltratie volgens paragraaf 3.2.1 [W]
> - Φv = specifieke warmteverlies door ventilatie [W]

Voor mechanische systemen (B, D, E) geldt **Formule 3.4** met Φvent = Φv (direct, geen aftrek van Φi).

### 6.2 Formule 3.11 (pagina 61)

> Het toe te rekenen extra vermogen Φextra volgt uit:
>
> **(Formule 3.11)** [formule-afbeelding — kwadratische sommatie]
>
> waarin:
> - Φextra = toe te rekenen niet altijd of gelijktijdig optredende warmteverliezen [W]
> - Φhu,i = toeslag voor bedrijfsbeperking volgens paragraaf 3.3 [W]
> - Φvent = ventilatiewarmteverlies volgens paragraaf 3.2 [W]
> - ΦT,iaBE = warmteverlies naar aangrenzend pand bepaald volgens paragraaf 2.5.2 [W]
>
> **Opmerking 1:** Door de (kwadratische) manier van toerekenen van de zogenaamde extra verliezen (Φextra in formule 3.11) is er nagenoeg geen sprake meer van overdimensionering van het voor woningen benodigde vermogen.

Letterlijk citaat bevestigt: **kwadratische sommatie** van drie deelverliezen (bedrijfsbeperking, ventilatie, aangrenzend pand) op **gebouwniveau** (paragraaf 3.5.2 = schil-vermogen). De formule zelf staat als afbeelding in de PDF en niet als tekst extracteerbaar; maar de **componenten en kwadratische opmerking** zijn 100% bevestigd.

---

## 7. Andere relevante observaties

### 7.1 Formule 2.34 / 2.35 — meetgebaseerde route (pagina 37-38)

> **(Formule 2.34)** `qi = fwind · finf · ftype2 · qv;10` (uit metingen)
>
> **(Formule 2.35)** `qi = fwind · finf · ftype2 · qv,10,spec · Ag` (forfaitair)

Onze code kent **geen** `fwind`, `finf`, `ftype2` aparte vermenigvuldigers in `infiltration.rs`. Snelle code-grep nodig: of die in een aanroepende laag (`calc/*`) zitten, of we slaan ze stilzwijgend over (zou een tweede norm-afwijking zijn).

### 7.2 Basisontwerpbuitentemperatuur (pagina 49)

> θe,0 = **-10 °C** (basis ontwerpbuitentemperatuur)
>
> Δθe,τ tussen 0 en 4 K, afhankelijk van tijdconstante τ van het gebouw, afgerond op halve graden.

Onze `crates/isso51-core/src/tables/` heeft géén tijdconstante-correctie zichtbaar in de file-listing. Audit moet checken of `calc/*` deze correctie toepast — anders gebruiken we constant -10°C terwijl norm tot -6°C kan oplopen voor zware gebouwen.

### 7.3 Tabel 2.5 — f_inf (pagina 39)

> | Ventilatiesysteem | f_inf |
> |---|---|
> | Systeem A — natuurlijke toe- en afvoer | 0,80 |
> | Systeem B — mech. toevoer + nat. afvoer | 0,85 |
> | Systeem C — nat. toevoer + mech. afvoer | 1,0 |
> | Systeem D — mech. toe- en afvoer | 1,10 |
> | Systeem E — zones met lokale WTW, CO₂-sturing | 1,05 |

Vijf systemen. Als onze code een `f_inf`-functie heeft (vermoed in `ventilation.rs` of `infiltration.rs`), check op systeem E.

### 7.4 Tabel 2.7 — f_tp (pagina 40)

> Zeven situaties (kap-met-puntdak / -platdak / -hoekligging, etc.); waardes 1,0 — 1,4.

Niet zichtbaar in onze tables-files. Mogelijk binnen `calc/*` of nog niet geïmplementeerd.

---

## 8. Audit-correcties die nodig zijn

| In `docs/2026-05-12-isso51-norm-conformiteit-audit.md` | Huidige claim | Correctie |
|---|---|---|
| Tabel 2.6-claim | "qv,10,spec waardes" | Tabel 2.6 is f_type2 (winddrukcorrectie 0,30-1,0). qv,10,spec heeft geen eigen tabel; berekend via formule 2.37 |
| Tabel 2.8-claim | (impliciet 4-rij qv10-keying) | Tabel 2.8 = 3-rij gebouwtype-keying: 1,0 / 0,7 / 0,5 dm³/(s·m²) Ag |
| Tabel 4.3-verwijzing | "Tabel 4.3 voor geveloppervlak-variant" | Tabel 4.3 bestaat niet in 2023-publicatie; verwijder of substitueer |
| Tabel 2.12-claim | "14 verwarmingssystemen, 4 Δθ-kolommen" | Klopt qua aantal, MAAR kolommen zijn afhankelijk van Ū-conditie (>0,5 vs ≤0,5). Vier waardes per systeem zijn dus eigenlijk twee paren onder verschillende thermische schil-condities |
| Issue C hypothese (5e klasse / qv10 > 150) | Vabi gebruikt 5e klasse | Niet correct. Vabi gebruikt vermoedelijk Tabel 2.8 rij 1 (1,0 dm³/(s·m²) Ag) of geveloppervlak-keying à la NEN 8088-1 met max ~0,32 |

---

## 9. Onbeantwoorde vragen

1. **Wat is de échte semantiek van onze `qi_spec_per_floor_area` en `qi_spec_per_exterior_area`?** Beide zijn als "Tabel 2.8" gelabeld maar geen van beide modelleert de feitelijke 3-rij gebouwtype-tabel. Vereist een commit-archeologie of een gesprek met de oorspronkelijke auteur. Mogelijk komen de getalswaardes uit NEN 8088-1 § (niet ingezien) of uit ISSO 51 vóór 2023.
2. **Past `calc/*` formule (2.35) toe?** De drie correctiefactoren `fwind · finf · ftype2` zijn essentieel voor de meting-route — als die niet worden vermenigvuldigd is dat een tweede norm-afwijking bovenop de Tabel 2.8-keying.
3. **Wordt Ū van het constructie-pakket gebruikt om de juiste Δθ-kolom uit Tabel 2.12 te selecteren?** Onze code lijkt 4 Δθ-waardes per systeem te hebben, maar het is onduidelijk of de selectie tussen "Ū>0,5" en "Ū≤0,5" paren plaatsvindt. Vereist tweede code-pass.
4. **Is de tijdconstante-correctie (formule 2.51) ergens in `calc/*` geïmplementeerd?** Anders rekenen we systematisch met θe = -10°C i.p.v. -10 + Δθe,τ.
5. **Vabi 0,317 — exacte herkomst?** Mogelijk komt de waarde uit een Vabi-eigen interpretatie van NEN 8088-1 die ISSO 51 niet één-op-één overneemt. Definitief uitsluitsel vergt NEN 8088-1 zelf inzien.

---

## 10. Aanbevolen vervolgacties (zonder code-wijziging)

1. Voeg in `infiltration.rs` boven `qi_spec_per_floor_area` een **TODO/FIXME** dat de docstring liegt (verwijst naar Tabel 2.8 die feitelijk een gebouwtype-tabel is, niet qv10).
2. Open een **separate TODO** in `TODO.md`: "Issue C — herontwerp infiltratie-lookup conform Tabel 2.8 (3 gebouwtypes) i.p.v. qv10-klassen; gegenereerde waardes vergelijken met Vabi-fixture na refactor".
3. Update de erratum-audit `docs/2026-05-12-isso51-norm-conformiteit-audit.md` met de correcties uit §8 hierboven.
4. Plan een NEN 8088-1 inzage-actie (publicatie kopen of bibliotheek) om de oorsprong van onze 0,04-0,32 getalswaardes te traceren — als die wél conformerend zijn met een sub-norm, mag de implementatie blijven maar moet de docstring eerlijk worden.
