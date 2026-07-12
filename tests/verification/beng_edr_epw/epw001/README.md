# EP-W001 — EDR-referentiewoning

Canonieke EDR-referentietest (ISSO 54 v2.0, §2.1, p4-6). Vrijstaande grondgebonden
eengezinswoning, 2 bouwlagen in 1 rekenzone, plat dak, ramen alleen op zuid.
Bouwjaar 2021.

## Kentallen (provenance in `expected.json`)

| Grootheid | Waarde | Bron |
|---|---|---|
| Afmetingen (binnenmaats) | 8,0 × 6,0 × 5,4 m | p5 |
| Volume | 259,2 m³ | p5 |
| A_g | 96 m² | p5 |
| A_ls (= A_o) | 247,2 m² | p5 (vlak-breakdown) |
| A_ls/A_g | 2,575 | afgeleid |
| Perimeter BG-vloer | 28 m | p5 |
| Ramen | 4 × 6 m² (zuid), U=1,8, g=0,7, kozijnfractie 25% | p5 + fig.1 |
| Dichte constructies | Rc = 6,0; U_dak/gevel = 0,162; vloer op grond | tabel 1 (p5) |
| Dm (therm. massa) | 450 kJ/m²K | p5 (tabel 7.10) |
| Infiltratie | qv10;spec = 0,7 dm³/(s·m²); ftype = 1,4 | p6 (tabel 11.14) |
| Ventilatie | D2 gebalanceerd + WTW (kunststof tegenstroom), LUKA C, bypass 1,0 | p6 |
| Ruimteverwarming | HR107-combiketel η=0,95, LT 45/40, vloerverwarming | p6 |
| Warmtapwater | HR107-combi, CW5, geen voorraadvat | p6 |
| Koeling | geen | p6 |
| PV | geen | p6 |

Figuur 1 (pixmap-render, p4) bevestigde de tekstmaten exact: zuidgevel 8,0×5,4 m,
2 bouwlagen à 2,7 m, per laag 2 ramen 3,0×2,0 m (0,5 m van rand, 1,0 m tussenruimte,
0,5 m boven, 0,2 m onder de raamrij).

## Consistentiecheck A_ls

dak 48 + vloer 48 + zuid **bruto** 43,2 (dicht 19,2 + raam 24,0) + noord 43,2 +
oost 32,4 + west 32,4 = **247,2 m²**. ✔ (matcht Ao in de tekst; zuid-dicht 19,2
matcht tabel 1)

## Status

- **Assertbaar nu (niet Excel-geblokkeerd):** `geometry_expected` — Ag, Als,
  A_ls/A_g. Eerste fase-2-activatie via een `edr_to_projectv2`-builder.
- **Geblokkeerd op Bijlage 2-Excel:** alle `grootheden` (EP1/EP2/EP3/Q_H;nd/TOjuli
  + deelposten). `value: null` + `blocked_on`; nooit met engine-uitkomst vullen.

Zie `../README.md` voor de gedeelde tolerantie- en anti-fudge-afspraken en
`docs/2026-07-12-f3d5-edr-testset-analyse.md` voor de volledige analyse.
