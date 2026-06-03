# ISSO 51:2023 — opwarmtoeslag (Φ_hu) geverifieerde formule-referentie

> Voor de A1/A2-rewrite (Ronde 5). Formules + Tabel 2.10 geëxtraheerd uit de ISSO 51:2023-PDF
> (`Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\ISSO-51 ... 01-05-2023.pdf`),
> dubbel geverifieerd: tekstlaag + `pdf_extract_tables` + **visuele render p.44/45/70 (PM, 2026-06-03)**.
> Alle 50 Tabel-2.10-cellen visueel bevestigd. Bron-PDF is auteursrechtelijk — **niet in git**.
>
> **Scope-besluit gebruiker (2026-06-03): NIEUWBOUW-scope eerst.** Afkoeling = 2 K (woning ná 2015)
> resp. 1 K (Ū≤0,50). Bestaande-bouw afkoeling (Afb 2.7-grafiek) = gemarkeerde follow-up, NIET nu.

## Het probleem (audit K1/K2, rapport 04-isso51.md)

`isso51-core/src/calc/heating_up.rs` gebruikt het **ISSO 51:2017 / NEN-EN 12831-model**
`Φ_hu = f_RH × ΣA_metselwerk` (accumulerend oppervlak). De term `f_RH` bestaat **niet** in
ISSO 51:2023. De 2023-norm vervangt dit door `Φ_hu = P × A_g` (vloeroppervlak), met `P` uit Tabel 2.10.
De unit-test `test_isso51_example_room1_heating_up` (heating_up.rs) cementeert het 2017-model en moet weg.
Beide Vabi-fixtures hebben `night_setback=false` → Φ_hu=0 → de foute kern wordt **nooit getest** (V1).

---

## Form. 2.45 (§2.5.8, p.43) — schil / woningniveau

```
Φ_hu = P · A_g
```
- `Φ_hu` = toeslag voor bedrijfsbeperking [W]
- `P` = specifieke toeslag voor bedrijfsbeperking [W/m²] (uit Tabel 2.10)
- `A_g` = totale gebruiksoppervlak van de woning/woongebouw [m²]

**§3.3 schiltoepassing (p.59):** voor de schilberekening wordt Φ_hu bepaald **voor het grootste
verblijfsgebied** (dus `A_g` = vloeroppervlak van het grootste verblijfsgebied, niet de hele woning).

## Form. 4.15 (§4.3.1, p.70) — per vertrek, regeling per verblijfsgebied

```
Φ_hu,i = P · A_g
```
- `Φ_hu,i` = toeslag per vertrek i [W]
- `P` = specifieke toeslag [W/m²] volgens §2.5.8 (Tabel 2.10)
- `A_g` = gebruiksoppervlak; **per verblijfsruimte** in deze §4.3.1-methode ("per verblijfsruimte
  bepaald"). Let op: de norm-definitieregel zegt generiek "woning/woongebouw" maar de methode is
  per-vertrek → implementeer als `room.floor_area` per verblijfsruimte. (Audit K1 bevestigt:
  `A_g = room.floor_area` per-vertrek, woning-`A_g` alleen voor de schil §3.3.)

---

## Regeltype-branches (§4.3, p.69-72) — NIEUWBOUW-scope

Opmerking-box p.70 (letterlijk): **"Bij nieuwbouw wordt altijd uitgegaan van regeling per
verblijfsruimte of een zelflerende regeling."** → Nieuwbouw kent exact twee takken:

| §  | Regeltype | Φ_hu | In nieuwbouw-scope? |
|----|-----------|------|---------------------|
| 4.3.1 | Regeling per verblijfsgebied (thermostatische afsluiters / stooklijn) | `P · A_g` (Form. 4.15) | ✅ JA |
| 4.3.2 | Zelflerende regeling (buitentemp/opwarmkarakteristiek stuurt inschakeltijd) | **0** | ✅ JA |
| 4.3.3 | Kamerthermostaat (y-procentmethode, Form. 4.16/4.17, óf 5 W/m²) | y-methode | ❌ bestaande bouw — BUITEN scope (markeer als follow-up) |

Aanvullende achterwege-laat-regels (alle takken):
- Vloerverwarming in **alle** verwarmde vertrekken (ook verdiepingen) → `Φ_hu,i = 0` (p.70).
- Geen nachtverlaging/bedrijfsbeperking → `Φ_hu,i = 0` (p.69).
- Zelflerende regeling → `Φ_hu,i = 0` (p.70).

---

## Bepaling van P uit Tabel 2.10 (3 ingangen)

**P[afkoeling][zwaarte][opwarmtijd]** — uit Afb. 2.6 beslis-schema (p.44):

1. **Afkoeling (graden verlaging [K]):**
   - Woning **ná 2015** (nieuwbouw) → **2 K** (vast; Afb 2.6 + voetnoten 6/7 "bij nieuwbouw zakt
     de temperatuur niet zo ver").
   - **Ū ≤ 0,50 W/(m²·K)** → **1 K** (harde tekstregel p.44, overschrijft het bovenstaande;
     `Ū` = oppervlakte-gewogen gem. U incl. thermische bruggen over externe scheidingsconstructies
     + ramen/deuren + begane grondvloer — al beschikbaar als `u_bar` in `lib.rs:81`).
   - (Bestaande bouw, Ū>0,5 → Afb 2.7-grafiek = BUITEN scope.)
2. **Zwaarte gebouw:** `c_eff ≤ 70 Wh/K` → **ZL+L+M**; anders → **Z** (p.44). `c_eff` uit
   forfaitaire Tabel 2.1 of Form. 2.46 (`c_eff = C_eff / V`).
3. **Opwarmtijd [h]:** default **2 h** (Afb 2.6 richtwaarde, asterisk = aanbevolen). Instelbaar.

## Tabel 2.10 — Specifieke toeslag P [W/m² vloeroppervlak] (VISUEEL GEVERIFIEERD, p.45)

Periode = 8 uur nachtverlaging. Kolommen = (afkoeling [K] × zwaarte). Rijen = opwarmtijd [h].
Superscripts in de PDF zijn voetnoten (6,7 = nieuwbouw; 8-17 = "minder zinvolle toepassing") — geen cijfers.

| Opwarmtijd [h] \ Afkoeling [K] | 1 ZL+L+M | 1 Z | 1,5 ZL+L+M | 1,5 Z | 2 ZL+L+M | 2 Z | 2,5 ZL+L+M | 2,5 Z | 3 ZL+L+M | 3 Z |
|---|---|---|---|---|---|---|---|---|---|---|
| **0,5** | 14 | 18 | 22 | 27 | 29 | 35 | 37 | 44 | 44 | 53 |
| **1**   | 10 | 14 | 16 | 21 | 21 | 28 | 27 | 36 | 32 | 43 |
| **2**   | 7  | 11 | 10 | 17 | 13 | 22 | 17 | 28 | 21 | 33 |
| **3**   | 5  | 10 | 8  | 15 | 10 | 19 | 13 | 23 | 15 | 27 |
| **4**   | 4  | 9  | 6  | 13 | 8  | 17 | 11 | 21 | 13 | 25 |

Sanity (monotonie, alle 50 cellen): P ↑ met afkoeling, Z > ZL+L+M, P ↓ met langere opwarmtijd. ✓

**Nieuwbouw-scope gebruikt alleen de kolommen afkoeling = 2 K (Ū>0,5) en 1 K (Ū≤0,5).**
Bij default opwarmtijd 2 h: P = 13 (2K, ZL+L+M) / 22 (2K, Z) / 7 (1K, ZL+L+M) / 11 (1K, Z).

---

## Erratum-2023 (rapport 04 §erratum) — NIET wijzigen

De kwadratische sommatie `Φ_extra = √(Φ_hu² + Φ_vent² + Φ_T,iaBE²)` (Form. 3.11) is **al correct**
(`quadratic_sum.rs`). Φ_hu zelf blijft een lineair vermogen (`P·A_g`) dat als component in 3.11 meegaat.
A1/A2 raakt alléén de Φ_hu-bepaling, niet de sommatie.

## V1 — verplichte nieuwe fixture

Beide bestaande ISSO 51-fixtures hebben `night_setback=false` → Φ_hu=0. Voeg een fixture/unit-test
**mét nachtverlaging** toe die de nieuwe `P·A_g`-kern écht uitvoert (nieuwbouw, afkoeling 2K,
opwarmtijd 2h, bekende zwaarte → verifieer P tegen Tabel 2.10 hierboven). Anders blijft de kern ongetest.

## Open follow-ups (NA Ronde 5, markeren)
- **Bestaande-bouw afkoeling (Afb 2.7-grafiek)** → eigen regeltype + afkoeling-lookup of invoerveld.
- **Kamerthermostaat §4.3.3 (Form. 4.16/4.17 y-procentmethode)** → bestaande bouw.
- **K3** (Φ_HL,build 3.12 vs Φ_HL,verdeler 3.13 splitsing) — alleen bij embedded heating.
- **K4** (VabiCompat-aggregatie sluit Φ_T,iae uit) — bewuste keuze, documenteren.
- **example-fix** `vabi_import.rs` (`[[example]] required-features=["vabi-import"]` in Cargo.toml).
- **V3** stale comment `integration_test.rs:5-11` (claimt dat DR-test moet falen — achterhaald).
