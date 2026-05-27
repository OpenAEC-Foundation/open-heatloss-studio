# Koellast — Vabi 3.11.2.23 — Woning B (koellastberekeningen.nl)

| Veld | Waarde |
|---|---|
| Norm | EN 12831 / NEN 5060:2008 ref TO2 streng (peak cooling load) |
| Software | Vabi Elements Koellast 3.11.2.23 + rekenkern Koellast 2.09 |
| Bron PDF | `tests/references/vabi-koellastberekeningen-woning-B-2024.pdf` |
| Gebouw | Woning, Ag_gekoeld = 182,6 m², V = 565,4 m³, 6 gekoelde ruimtes (VR) |
| Klimaat | NEN 5060:2008 ref TO2 streng |
| T_setpoint zomer | 24 °C alle ruimtes |
| Beschaduwing | aan op alle 4 typen; schakelniveau handzonwering 300 W/m² |
| Zondoorstraling | nee |
| Status `expected.json` | ✅ ingevuld vanuit PDF — engine implementatie pending |
| Status Rust-test | n.v.t. — peak-cooling engine bestaat nog niet |

## Status (2026-05-27)

✅ **`expected.json` ingevuld** vanuit het Vabi Koellast PDF rapport. Bevat peak W (voelbaar + latent + totaal) per ruimte + gebouw-totaal + maand/tijdvak.

❌ **`input.json` ontbreekt** — er bestaat geen heatloss-studio inputmodel voor deze woning. TBD wanneer peak-cooling engine bestaat en de constructies/oriëntaties uit het PDF gereconstrueerd kunnen worden.

❌ **Peak-cooling engine niet beschikbaar** — `crates/nta8800-cooling/` rekent annual cooling demand (NTA 8800 H.10) in MJ, niet peak W. Aparte engine nodig (EN 12831 / NEN 5060 TO2).

## Peak koellast per ruimte (uit PDF)

| Code | Ruimte | Type | T_int | Voelb [W] | Latent [W] | Totaal [W] | W/m² | Max maand | Tijdvak |
|---|---|---|---|---|---|---|---|---|---|
| 0.06 | Keuken       | VR | 24,0 | 1914 | 97 | 2010 | 47  | september | 19 |
| 0.08 | Woonkamer    | VR | 24,0 | 1781 | 29 | 1810 | 39  | juli      | 20 |
| 0.09 | Eetkamer     | VR | 24,0 | 2072 | 34 | 2106 | 48  | augustus  | 19 |
| 0.13 | Slaapkamer 1 | VR | 24,0 | 1764 | 34 | 1798 | 99  | augustus  | 20 |
| 1.02 | Slaapkamer 2 | VR | 24,0 | 1329 | 48 | 1377 | 99  | september | 15 |
| 1.05 | Slaapkamer 3 | VR | 24,0 |  993 | 34 | 1027 | 61  | juli      | 19 |
| **Totaal gebouw** | | | | **8633** | **261** | **8894** | | **augustus** | **20** |

## Bekende afwijkingen

- **Som per-ruimte voelbaar (9853 W) ≠ gebouw-piek voelbaar (8633 W).** Per-ruimte peaks treden niet allemaal in hetzelfde tijdvak op — gebouwpiek is sommatie in augustus 20:00, terwijl bv. keuken en eetkamer hun piek hebben in september 19 / augustus 19.
- **Zondoorstraling staat op nee** — opvallend voor een woning. Mogelijk vereenvoudiging in het PDF-voorbeeld; niet 1-op-1 toepassen op echte projecten.
- **Schakelniveau handzonwering 300 W/m²** is hoger dan de DR Engineering woningbouw fixture (150 W/m²). Resultaat: minder vroeg dichtgaan → hogere peaks bij gelijke oriëntatie.

## Files

- `expected.json` — peak W (voelbaar + latent + totaal) per ruimte + gebouw + maand/tijdvak
- `README.md` — dit bestand
- `reference.pdf` — niet in deze folder; staat in `tests/references/vabi-koellastberekeningen-woning-B-2024.pdf`

Niet aanwezig (TBD):

- `input.json` — wacht op peak-cooling engine + reconstructie van 182,6 m² woning uit PDF
