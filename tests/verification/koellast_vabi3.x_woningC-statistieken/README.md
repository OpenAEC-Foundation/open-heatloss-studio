# Koellast — Vabi (versie onbekend) — Woning C (statistieken-XLS)

| Veld | Waarde |
|---|---|
| Norm | EN 12831 / NEN 5060 TO2 (peak cooling load) |
| Software | Vabi Koellast — rekenkern-versie niet in XLS aanwezig (placeholder `vabi3.x`) |
| Bron XLS | `tests/references/vabi-koellast-statistieken-woning-C.xls` (sheets `Gebouw` + `Ruimte`) |
| Gebouw | Woning, Ag_gekoeld ≈ 85 m² (geschat), V ≈ 215 m³ (geschat), 3 gekoelde ruimtes |
| Klimaat | NEN 5060 TO2 (aanname — niet expliciet in XLS) |
| T_setpoint zomer | onbekend (geen kolom in statistieken-XLS) |
| Status `expected.json` | ✅ ingevuld vanuit XLS — A_g + areas zijn schattingen |
| Status Rust-test | n.v.t. — peak-cooling engine bestaat nog niet |

## Status (2026-05-27)

✅ **`expected.json` ingevuld** vanuit de Vabi statistieken-XLS. Bevat peak W (voelbaar + latent + totaal) per ruimte + gebouw-totaal + maand/tijdvak. **A_g en per-ruimte vloeroppervlakken zijn geschat** uit `peak_w / w_per_m2`.

❌ **`input.json` ontbreekt** — er bestaat geen heatloss-studio inputmodel voor deze woning, en het XLS bevat alleen statistieken (geen geometrie of constructies).

❌ **Peak-cooling engine niet beschikbaar** — zelfde situatie als de andere koellast-fixtures: `nta8800-cooling` doet annual (MJ), niet peak W.

❌ **Vabi rekenkern-versie onbekend** — XLS statistiek-export bevat geen versieveld. Folder-naam gebruikt `vabi3.x` als placeholder; updaten zodra het bron-PDF (indien aanwezig) of een Vabi-projectfile geraadpleegd kan worden.

## Peak koellast per ruimte (uit XLS)

| Code | Ruimte    | Voelb [W] | Latent [W] | Totaal [W] | W/m² | W/m³ | A_g [m²] (geschat) | Max maand | Tijdvak |
|---|---|---|---|---|---|---|---|---|---|
| 0.07 | Tuinkamer   | 3501 | 34 | 3535 | 121 | 31 | 29 | juli | 20 |
| 1.04 | Slaapkamer  |  781 | 34 |  815 |  28 | 17 | 29 | juli | 20 |
| 1.05 | Slaapkamer  |  966 | 48 | 1014 |  38 | 19 | 26 | juli | 10 |
| **Totaal gebouw** | | **5260** | **102** | **5362** | **63** | **25** | **≈85** | **juli** | **20** |

## Bekende afwijkingen

- **A_g + per-ruimte areas zijn geschat** uit de W/m² + W/m³ kolommen — niet rechtstreeks uit de XLS. Tuinkamer 3501 / 121 ≈ 28,9 m² → 29 m². Gebouw 5260 / 63 ≈ 83,5 m² → 85 m². **Niet als test-truth gebruiken**, alleen indicatief voor UI-controle.
- **Som per-ruimte voelbaar (5248 W) ≈ gebouw voelbaar (5260 W).** Verschil 12 W binnen afrondingsmarge — juli 20:00 is gelijktijdige piek voor 2 van 3 ruimtes (1.05 heeft eerder piek om 10:00).
- **Geen T_setpoint, geen beschaduwings-info, geen klimaat-set in XLS.** Aannames hierboven zijn extrapolatie van de andere Woning B fixture en moeten geverifieerd worden zodra een onderliggend PDF/projectfile beschikbaar komt.

## Files

- `expected.json` — peak W per ruimte + gebouw + maand/tijdvak + schattings-notes
- `README.md` — dit bestand
- `reference.xls` — niet in deze folder; staat in `tests/references/vabi-koellast-statistieken-woning-C.xls`

Niet aanwezig (TBD):

- `input.json` — wacht op peak-cooling engine + geometrie-reconstructie (niet in XLS aanwezig)
- Vabi-versie — placeholder `vabi3.x` updaten zodra bekend
