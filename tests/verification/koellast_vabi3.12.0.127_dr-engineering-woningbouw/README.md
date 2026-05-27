# Koellast — Vabi 3.12.0.127 — DR Engineering Woningbouw

| Veld | Waarde |
|---|---|
| Norm | EN 12831 / NEN 5060 TO2 (peak cooling load) |
| Software | Vabi Elements Koellast 3.12.0.127 |
| Bron PDF | `tests/references/dr-engineering-koellast-woningbouw-2024.pdf` (432 KB) |
| Gebouw | Woning, Ag_gekoeld = 191,7 m², V = 414,6 m³, 6 gekoelde ruimtes |
| Klimaat | NEN 5060 TO2 streng (extreme dag) |
| T_setpoint zomer | 24 °C alle ruimtes |
| Beschaduwing | aan, schakelniveau handzonwering 150 W/m² |
| Status `expected.json` | ✅ ingevuld vanuit PDF — engine implementatie pending |
| Status Rust-test | `#[ignore]` op alle echte tests in `crates/nta8800-cooling/tests/vabi_koellast_golden.rs` tot peak-cooling engine bestaat |

## Status (2026-05-27)

✅ **`expected.json` ingevuld** vanuit het Vabi Koellast PDF rapport. Bevat peak W per ruimte + gebouw-totaal + maand/tijdvak waarop het optreedt.

❌ **`input.json` ontbreekt** — eerdere 120 m² synthetische woning is verwijderd want komt niet overeen met de 191,7 m² woning uit de PDF. TBD wanneer peak-cooling engine bestaat en een passend heatloss-studio inputmodel uit het rapport gereconstrueerd kan worden.

❌ **Peak-cooling engine niet beschikbaar** — `crates/nta8800-cooling/` rekent annual cooling demand (NTA 8800 H.10) in MJ, niet peak W. Voor deze fixture is een aparte engine nodig (EN 12831 / NEN 5060 TO2).

## Peak koellast per ruimte (uit PDF)

| Ruimte | Vloer m² | Peak [W] | Maand | Tijdvak |
|---|---|---|---|---|
| 0.03 Woonkamer | 45,70 | 2074 | augustus | 10 |
| 0.04 Keuken / eetkamer | 40,75 | 1869 | september | 19 |
| 1.04 Slaapkamer 1 | — | 1118 | september | 14 |
| 1.03 Slaapkamer 2 | — | 590 | juli | 10 |
| 1.02 Slaapkamer 3 | 15,65 | 610 | juli | 8 |
| 1.08 Speelzolder | — | 1122 | juli | 19 |
| **Totaal gebouw** | **191,7** | **6420** | **augustus** | **14** |

## Verschil met NTA 8800 H.10 (annual)

| Aspect | Deze fixture (Koellast) | NTA 8800 H.10 (TO-juli) |
|---|---|---|
| Output-eenheid | W (vermogen) | MJ / kWh (energie per jaar) |
| Tijdresolutie | uur (tijdvak 8-20) | maand (12 waardes) |
| KPIs | Max koellast/ruimte, totaal gebouw 6420 W | Q_C;use [MJ], H_T [W/K], H_V [W/K], τ [h] |
| Klimaat | NEN 5060 ref TO2 streng (extreme dag) | De Bilt referentiejaar |
| Doel | Dimensioneren koelinstallatie | BENG-2 toets TO-juli ≤ 1,20 |
| Crate-engine | nog niet beschikbaar | `nta8800-cooling` (annual) |

## Files

- `expected.json` — peak W per ruimte + gebouw + maand/tijdvak (ingevuld vanuit PDF)
- `README.md` — dit bestand
- `reference.pdf` — niet in deze folder; staat in `tests/references/dr-engineering-koellast-woningbouw-2024.pdf`

Niet aanwezig (TBD):
- `input.json` — wacht op peak-cooling engine + nieuwe 191,7 m² project-reconstructie

## Volgende stappen

1. **Peak-cooling engine ontwerpen** — EN 12831 / NEN 5060 TO2 logic, apart van `nta8800-cooling` (annual TO-juli). Nieuwe crate `crates/peak-cooling/` voorgesteld.
2. **input.json reconstrueren** — 191,7 m² woning matchen aan de constructies/oriëntaties/beschaduwing zoals beschreven in p.5-23 van de Vabi PDF.
3. **`#[ignore]` weghalen** van tests in `crates/nta8800-cooling/tests/vabi_koellast_golden.rs` zodra engine + input.json klaar zijn (of verplaatsen naar de nieuwe `peak-cooling` crate).

## PDF-detail (24p Vabi Elements Koellast)

- p.1-2: project + uitgangspunten
- p.3-4: totalen gebouw + maandoverzicht (mei-sep, tijdvak 8-20, in W)
- p.5-21: per-ruimte resultaten (6 ruimtes × ~3 pagina's: koellast/uur, deelresultaten transmissie/zon/intern, constructies, schaduwfracties)
- p.22-23: plattegronden + isometrie
- p.24: bedrijfsgegevens DR Engineering
