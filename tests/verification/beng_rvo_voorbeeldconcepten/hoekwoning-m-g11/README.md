# Hoekwoning M (G11) — RVO BENG-golden

BENG-referentie **Hoekwoning M** (2-onder-1-kap / rijhoekwoning, hellend dak op 2e verdieping, dwarskap voorzijde). Ag 133 m², Als/Ag 1,87, massief. BENG 1-eis ≤ 66,2 kWh/m²·jr.

Bron: `rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf` — eisen p.7 (tabel 5), resultaten p.13 (Bijlage 1). Zie `../README.md` voor de gedeelde versie-caveat, geometrie-oordeel en tolerantie-motivatie.

## Concepten (expected uit p.13)

| Concept | BENG 1 | BENG 2 | BENG 3 | TOjuli | Wp PV |
|---|---|---|---|---|---|
| WP-bodem C4c/BB+ | 59,2 | 28,2 | 62% | 0 | 0 |
| WP-buiten D2/BB+ | 59,6 | 30,0 | 59% | 1,17 | 1.600 |
| WP-bodem D5a/passief | 49,7 | 24,3 | 50% | 0 | 0 |

Let op de randgevallen: WP-buiten D2/BB+ haalt BENG 2 = 30,0 (exact op de eis ≤30) en TOjuli 1,17 (net onder 1,20). Volgens de PDF (p.7) voldoet hoekwoning M bij de **passieve** pakketten net níet aan TOjuli (1,22) — ons D5a/passief-concept toont echter TOjuli 0 omdat het bodemkoeling heeft (bodemkoeling ⇒ automatisch TOjuli-conform). Deze twee near-boundary-waarden maken de case diagnostisch waardevol.

Volledige provenance per waarde: `expected.json`. Best-effort invoer: `input.json`.
