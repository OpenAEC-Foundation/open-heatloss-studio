# ISSO 51:2023 basis-publicatie — tabel- en formule-verificatie

**Datum:** 2026-05-12
**Bron:** `Z:/50_projecten/7_3BM_bouwkunde/000_Documentatie/98_normen/ISSO-51 Warmteverliesberekening voor woningen d.d. 01-05-2023.pdf` (94 p., basis-publicatie 01-05-2023, ISSO Ontwikkeling B.V.)
**Bedoeld om Issue C uit `docs/2026-05-12-issue-diagnostiek.md` definitief af te sluiten.**

_Redactie 2026-07-02: letterlijke norm-transcripties vervangen door verwijzingen (auteursrecht ISSO); volledige transcripties lokaal bij 3BM._

---

## 0. TL;DR

| Bevinding | Status | Ernst |
|---|---|---|
| Tabel 2.8 in onze code is **fundamenteel verkeerd geïnterpreteerd** — geen qv10-klassen, maar gebouwtype-rijen | Bug | KRITIEK |
| Tabel 4.3 bestaat niet in de 2023-publicatie | Refactor nodig | HOOG |
| Tabel 2.6 is `f_type2` (winddrukcorrectie), NIET qv,10,spec | Audit-correctie | MIDDEL |
| Tabel 2.12 (14 verwarmingssystemen) — basis heeft slechts 13 rijen, niet 14 | Audit-correctie | LAAG |
| Formule 3.3 / 3.11 — geverifieerd tegen norm, audit-claims kloppen | OK | — |
| Vabi 0.317 — verklaarbaar via `q_i,spec = 1.0` (Tabel 2.8 rij 1) ÷ gevel-area-factor, NIET via 5e qv10-klasse | Issue C verklaard | — |

---

## 1. Tabel 2.8 — qi,spec (KRITISCH voor Issue C)

### 1.1 Verificatie tegen Tabel 2.8 (p.41)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.8, p.41 (bron lokaal: `Z:\...\98_normen\ISSO-51 ... 01-05-2023.pdf`)._

**Bevinding:** Tabel 2.8 is een **3-rijige gebouwtype-lookup** (sleutel = gebouwtype, niet qv10-klasse), eenheid dm³/(s·m²) gebruiksoppervlak (Ag), niet geveloppervlak. Het levert qi,spec als de **forfaitaire methode** voor woningen waarvan de luchtdichtheid (qv;10) NIET door meting bekend is. De uitkomst loopt via formule (2.35) `qi = fwind · finf · ftype2 · qv,10,spec · Ag`, waarbij qv,10,spec (NIET qi,spec) uit formule (2.37) volgt = `qi,spec · ftp · fjaar` (p.40). De drie gebouwtype-waarden staan — als losse punten om de code-discrepantie te tonen — in vergelijkingstabel §1.2.

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

### 2.1 Verificatie tegen Tabel 2.6 (p.39)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.6, p.39 (bron lokaal: `Z:\...\98_normen`)._

**Bevinding:** Tabel 2.6 levert de correctiefactor **f_type2** (winddrukverdeling/thermiek), NIET qv,10,spec. Grondgebonden gebouwen (kap of platdak) staan op 1,0; meerlaagse woongebouwen variëren van 1,0 (standaard) tot 0,30 (dubbele huidgevel met doorlopende tussenruimte), met tussenwaarden voor binnengalerij en onderbroken huidgevel. Dit is het bewijs dat het erratum-audit-doc de tabel-toewijzing verwart (zie §2.2).

### 2.2 Verdict — erratum-audit moet herzien

De erratum-audit `docs/2026-05-12-isso51-norm-conformiteit-audit.md` (en ondersteunend erratum-doc) beweren dat Tabel 2.6 de **qv,10,spec-waarden** levert. Dat klopt niet voor de 2023-publicatie. Tabel 2.6 is uitsluitend de **correctiefactor f_type2** voor winddrukverdeling, gebruikt in formule (2.35). De qv,10,spec-getalswaardes komen NIET uit een tabel met dat nummer, maar uit:

- **opgave van de energieadviseur/architect** (uit bouwaanvraag), of
- **formule (2.37)**: `qv,10,spec = qi,spec(Tab 2.8) · ftp(Tab 2.7) · fjaar(formule 2.38)`

Het erratum mag deze terminologie hebben rechtgetrokken, maar de **basis-publicatie 01-05-2023 zelf** kent géén "Tabel qv,10,spec".

---

## 3. Formule E.5 — context (pagina 90-91)

_Geverifieerd tegen ISSO 51:2023 Bijlage E, formule E.5, p.90-91 (bron lokaal: `Z:\...\98_normen`)._

**Bevinding:** Bijlage E behandelt de **warmtebalans van aangrenzende vertrekken** (woning achter belendende woning); E.5 is geen primaire-routeformule maar een **hulpformule voor de temperatuur θa van een aangrenzend pand**. De symboolverklaring bij E.5 definieert `qi;spec` expliciet als "specifieke luchtvolumestroom infiltratie per m² gebruiksoppervlak **conform tabel 2.8**" — daarmee is bevestigd dat Tabel 2.8 gebouwtype-categorieën levert (per m² Ag), niet qv10-klassen.

De hoofdroute voor infiltratie van de **eigen** woning loopt via formule (2.31)/(2.32) + (2.34)/(2.35) — daarin komt qi rechtstreeks of via qv;10-meting. Beide paden gebruiken **dezelfde Tabel 2.8 als forfaitaire fallback wanneer qv;10 onbekend is**.

---

## 4. Tabel 2.12 — verwarmingssystemen (pagina 48)

### 4.1 Verificatie tegen Tabel 2.12 (p.48)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.12, p.48 (Δθ/Δθa1/Δθ2/Δθa2/Δθv per verwarmingssysteem, ontwerpcondities, ruimtehoogte ≤ 4 m). Bron lokaal: `Z:\...\98_normen`; volledige tabel niet gereproduceerd (auteursrecht)._

**Bevinding:** de tabel telt **14 verwarmingssystemen** (3 lokale + 11 centrale) en heeft **vier Δθ-kolommen**, met een **Ū-discriminatie** (Ū > 0,5 vs Ū ≤ 0,5). De implementatie `crates/isso51-core/src/tables/temperature.rs` bevat alle 14 varianten (afzonderlijke enum-varianten voor vloerverwarming ≥27°C / <27°C en de vloer+wand-combinaties) — komt overeen. Afwijkingen: geen op aantal/waarden. Aandachtspunt: bevestig in een follow-up dat onze Δθ-functie de Ū-waarde van het constructie-pakket gebruikt om tussen de twee Ū-kolomparen te kiezen; anders mist de code de Ū-conditie.

### 4.2 Vergelijking met onze code

De audit-claim "1-op-1 erratum match met 14 rijen" is **correct**. Geen wijziging nodig in `crates/isso51-core/src/tables/temperature.rs`. Enige openstaande vraag is de Ū-kolomselectie (zie §4.1).

---

## 5. Tabel 4.3 — bestaat NIET in 2023-publicatie

Zoekopdracht "Tabel 4.3" geeft **0 hits** in het volledige PDF-tekstlaag. De code-verwijzing in `infiltration.rs:2` (`Tables 2.8, 4.3`) is dus **stale** — Tabel 4.3 is óf uit een vorige ISSO 51 editie (mogelijk 2017 of eerder), óf de auteur heeft de tabelnummering verkeerd onthouden.

**Aanbeveling:** verwijder de "Table 4.3"-verwijzing uit de docstring en hernoem `qi_spec_per_exterior_area` met een eerlijker bron (NEN 8088-1 direct, of een interne afspraak met expliciete onderbouwing).

---

## 6. Formules 3.3 en 3.11 — verificatie

### 6.1 Formule 3.3 (pagina 58)

_Geverifieerd tegen ISSO 51:2023 §3.2.3, formule 3.3, p.58 (bron lokaal: `Z:\...\98_normen`)._

**Bevinding:** op gebouwniveau geldt voor systemen met **natuurlijke toevoer (A en C)** dat het in rekening te brengen ventilatiewarmteverlies `Φvent = Φv − Φi` is (met clamp: Φvent = 0 indien negatief) — infiltratie is deel van de toevoerlucht en wordt afgetrokken tegen dubbeltelling. Voor **mechanische systemen (B, D, E)** geldt formule 3.4: `Φvent = Φv` (geen aftrek). De formule staat als afbeelding in de PDF; de rekenrelatie is bevestigd.

### 6.2 Formule 3.11 (pagina 61)

_Geverifieerd tegen ISSO 51:2023 §3.5.2, formule 3.11, p.61 (bron lokaal: `Z:\...\98_normen`)._

**Bevinding:** het extra vermogen Φextra is een **kwadratische sommatie** van drie niet-simultane deelverliezen — bedrijfsbeperking (Φhu,i), ventilatie (Φvent) en warmteverlies naar aangrenzend pand (ΦT,iaBE) — toegepast op **gebouwniveau** (§3.5.2, schil-vermogen). De norm merkt op dat deze kwadratische toerekening overdimensionering van het benodigde woningvermogen vrijwel wegneemt. Formule staat als afbeelding; componenten en kwadratische aard zijn bevestigd.

---

## 7. Andere relevante observaties

### 7.1 Formule 2.34 / 2.35 — meetgebaseerde route (pagina 37-38)

_Geverifieerd tegen ISSO 51:2023 §2.5.6, formules 2.34/2.35, p.37-38._

Meting-route: `qi = fwind · finf · ftype2 · qv;10`. Forfaitaire route: `qi = fwind · finf · ftype2 · qv,10,spec · Ag`. Onze code kent **geen** aparte `fwind`/`finf`/`ftype2` vermenigvuldigers in `infiltration.rs`. Snelle code-grep nodig: of die in een aanroepende laag (`calc/*`) zitten, of we slaan ze stilzwijgend over (zou een tweede norm-afwijking zijn).

### 7.2 Basisontwerpbuitentemperatuur (pagina 49)

_Geverifieerd tegen ISSO 51:2023 §2.6, p.49._ Basis-ontwerpbuitentemperatuur θe,0 = -10 °C, met een tijdconstante-afhankelijke correctie Δθe,τ (0–4 K, op halve graden). Onze `crates/isso51-core/src/tables/` heeft géén tijdconstante-correctie zichtbaar in de file-listing. Audit moet checken of `calc/*` deze correctie toepast — anders gebruiken we constant -10°C terwijl de norm tot -6°C kan oplopen voor zware gebouwen.

### 7.3 Tabel 2.5 — f_inf (pagina 39)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.5, p.39 (bron lokaal: `Z:\...\98_normen`)._ Vijf ventilatiesystemen (A t/m E) met f_inf-correctie, oplopend van 0,80 (systeem A) naar 1,10 (systeem D), plus 1,05 voor systeem E. Als onze code een `f_inf`-functie heeft (vermoed in `ventilation.rs` of `infiltration.rs`), check op systeem E.

### 7.4 Tabel 2.7 — f_tp (pagina 40)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.7, p.40._ Zeven liggings-situaties (kap-met-puntdak / -platdak / -hoekligging, etc.); waardes 1,0 — 1,4. Niet zichtbaar in onze tables-files. Mogelijk binnen `calc/*` of nog niet geïmplementeerd.

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
