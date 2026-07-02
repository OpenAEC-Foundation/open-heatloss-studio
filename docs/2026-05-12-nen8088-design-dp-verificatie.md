# NEN 8088-1 — design-Δp verificatie voor Vabi-compatibiliteit

**Datum:** 2026-05-12
**Bron:** `C:/Users/JochemK/Desktop/NEN 8088-1+C1_2012_C2_2014 nl.pdf` (22 pp., NEN 8088-1+C1:2012/C2:2014)
**Status bron:** vervallen sinds NTA 8800 (2020+), maar Vabi-software gebruikt nog steeds deze rekenmethode voor warmteverlies-infiltratie
**Bedoeld om:** laatste open vraag uit `2026-05-12-nta8800-infiltratie-verificatie.md` te beantwoorden — herkomst Vabi-factor **0,461**

_Redactie 2026-07-02: letterlijke norm-transcripties (NEN 8088-1) vervangen door verwijzingen (auteursrecht NEN); volledige transcripties lokaal bij 3BM._

## Scope & beperking

Het beschikbare PDF is het **correctieblad** (NEN 8088-1+C1/C2), niet de oorspronkelijke NEN 8088-1:2011. Het correctieblad bevat alleen *vervangingen* op pagina-/paragraaf-niveau. Hierdoor staat de **volledige** formule-context (incl. waarde van `f_wind`) niet woordelijk in dit document, maar de gewijzigde formule (5.25) en alle vervangen tabellen (9, 10) staan er wél in.

## 1. Design-Δp in NEN 8088-1

**Geen expliciete design-Δp van 3,14 Pa (of ~3 Pa) gevonden.**

Het correctieblad vermeldt drukverschillen alleen in de context van **luchtdrukgestuurde toevoer-roosters** (∆p over rooster), in drie categorieën (≤ 1 Pa / 1–5 Pa / 5–10 Pa). Deze ∆p-waarden zijn rooster-classificaties voor het selecteren van een correctiefactor (NEN 8088-1 Tabel 3, p.4), **niet** een design-Δp voor infiltratie-berekening op gebouwniveau. Zoekterm `3,14` levert **0 hits**.

## 2. n_lea = 0,67

**`n_lea` als symbool en `0,67` als exponent komen NIET voor in dit correctieblad.**

Zoekterm `0,67` levert 2 hits, beide in Tabel 3 (pagina 5) als kolomwaarde voor ventilatie-correctiefactoren D.5a/D.5b (CO₂-sturing met meerdere zones) — totaal andere context. NEN 8088-1 hanteert **geen power-law `(Δp/10)^n_lea`-conversie**, maar een **lineaire** relatie via een vaste coëfficiënt (zie §3).

## 3. Conversie q_v10 → q_v;inf — volledige formule

_Geverifieerd tegen NEN 8088-1+C2, §5.8.1.1, formule 5.25, p.7 (bron lokaal: bureau-archief)._ De vervangen formule 5.25 luidt in de kern `q_ve;inf = f_wind · f_type2 · f_inf · (0,23 · q_v10;spec) · A_g`, met de opmerking dat `q_v10;spec · A_g` de gemeten waarde bij een blowerdoor-test is. Componenten:

| Symbool | Betekenis | Bron in correctieblad |
|---------|-----------|----------------------|
| `q_ve;inf` | toevoertemperatuur-gecorrigeerde infiltratiestroom [dm³/s] | formule (5.25) |
| `f_wind` | correctiefactor winddrukregime (afhankelijk van klimaatzone) | **niet** in correctieblad — staat in oorspronkelijke NEN 8088-1:2011 |
| `f_type2` | correctiefactor gebouwafhankelijke winddrukverdeling + thermiek | Tabel 9, pag. 7 — waarden 0,30 / 0,90 / 0,94 / **1,0** (standaard) |
| `f_inf` | correctiefactor ventilatievoorziening | Tabel 10, pag. 8 — zie §4 |
| `0,23` | **vaste lineaire conversiefactor** qv10 → qinf | hard-coded in formule (5.25) |
| `q_v10;spec` | specifieke luchtvolumestroom bij Δp = 10 Pa (blowerdoor) | gemeten in [dm³/(s·m²)] |
| `A_g` | gebruiksoppervlakte zone [m²] | bepaald volgens 5.3 |

**Cruciaal:** geen power-law, geen design-Δp parameter. De factor `0,23` is een vaste empirische conversie van blowerdoor-meting (10 Pa) naar gemiddelde infiltratie onder Nederlandse klimaatcondities. Equivalent met `(Δp_eff/10)^0,67 ≈ 0,23` geeft `Δp_eff ≈ 0,86 Pa` — een orde van grootte lager dan Vabi's afgeleide 3,14 Pa.

## 4. f_inf = 1,10 — herkomst

**JA, `f_inf = 1,10` komt direct uit NEN 8088-1 Tabel 10.** Geen ISSO 51:2023-specifieke factor.

_Geverifieerd tegen NEN 8088-1+C2 Tabel 10, p.8 (bron lokaal: bureau-archief; volledige tabel niet gereproduceerd, auteursrecht NEN)._ Tabel 10 geeft f_inf per ventilatievoorziening (A t/m E), oplopend van 0,80 (systeem A) naar 1,10 voor **systeem D (mechanische toe- en afvoer, gebalanceerd)**, met 1,05 voor E.1 (C+D met decentrale WTW + CO₂-sturing). Deze reeks is identiek aan ISSO 51 Tabel 2.5. Vabi gebruikt waarschijnlijk standaard **D-systeem** (gebalanceerd) → `f_inf = 1,10` is dan correct.

## 5. Eindconclusie

### Vabi-factor 0,461 verklaring — definitief?

**Nee, niet uit NEN 8088-1 alleen reproduceerbaar.** Wel kennen we nu alle losse bouwstenen. Twee plausibele decomposities:

| Hypothese | Berekening | Match Vabi 0,461? |
|-----------|-----------|--------------------|
| **A: lineair NEN 8088-1** | `f_wind · f_type2 · 1,10 · 0,23` met f_type2=1,0 → `f_wind · 0,253` | vereist `f_wind ≈ 1,82` (atypisch — gevelzone IV?) |
| **B: power-law NTA 8800** | `1,10 · (3,14/10)^0,67 = 1,10 · 0,461 ≈ 0,507` | **dichtbij 0,461 maar niet exact** |
| **C: hybride Vabi-eigen** | `1,10 · (Δp/10)^0,67`, fit op 0,461 → `Δp = 3,14 Pa` | bevestigt empirische fit, geen norm-onderbouwing |

De aanname uit `2026-05-12-nta8800-infiltratie-verificatie.md` dat Vabi een **NTA 8800-power-law** gebruikt met `n_lea = 0,67` en `Δp = 3,14 Pa` blijft een **best fit zonder expliciete normverwijzing**. NEN 8088-1 schrijft een fundamenteel andere methodologie voor (lineair, factor 0,23, met `f_wind` als klimaat-correctie). Mogelijk hanteert Vabi intern een hybride waarin NTA 8800-power-law gecombineerd is met `f_inf` uit NEN 8088-1 Tabel 10.

### Implementatie-aanbeveling

| Aspect | Voorgestelde default in `isso51-core` |
|--------|---------------------------------------|
| Conversie qv10 → q_inf | Power-law `q_inf = f_inf · q_v10 · (Δp/10)^n_lea` (NTA 8800-compatibel) |
| `n_lea` default | **0,67** (NTA 8800 Tabel 11.2), configurable per project |
| `Δp_design` default | **3,14 Pa** (Vabi-compatibele fit), configurable, label "Vabi-equivalent" |
| `f_inf` default | **1,10** (D-systeem gebalanceerd, meest voorkomend), configurable via dropdown met NEN 8088-1 Tabel 10 waarden |
| UI-disclaimer | Voetnoot: "design-Δp = 3,14 Pa is empirische Vabi-fit, niet expliciet uit NEN 8088-1 of NTA 8800; voor strikte NEN 8088-1 conformiteit gebruik lineair model met factor 0,23 en `f_wind` uit klimaatzone-tabel" |
| Toekomstige toggle | Methode-selector: `"vabi-fit"` (default) / `"nen8088-linear"` / `"nta8800-strict"` |

**Verdict:** Vabi 100% reproduceerbaar uit NEN 8088-1: **nee** — `n_lea = 0,67`, `Δp = 3,14 Pa` en de power-law-vorm staan niet in NEN 8088-1; Vabi mixt de NTA 8800-formulestructuur met de `f_inf`-tabel uit NEN 8088-1 en past een empirische `Δp`-fit toe.
