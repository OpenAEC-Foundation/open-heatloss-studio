# Vrijstaande woning L (G12) — RVO BENG-golden

BENG-referentie **Vrijstaande woning L** (grondgebonden, hellend dak, ruime dakkapel noordoost). Ag 181 m², Als/Ag 2,14, massief. BENG 1-eis ≤ 74,1 kWh/m²·jr.

Bron: `rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf` — eisen p.7 (tabel 5), resultaten p.14 (Bijlage 1, pagina 2/2). Zie `../README.md` voor de gedeelde versie-caveat, geometrie-oordeel en tolerantie-motivatie.

## Concepten (expected uit p.14)

| Concept | BENG 1 | BENG 2 | BENG 3 | TOjuli | Wp PV | EPC |
|---|---|---|---|---|---|---|
| WP-bodem C4c/BB+ | 67,3 | 28,2 | 66% | 0 | 0 | 0,38 |
| WP-buiten D2/BB+ | 65,0 | 28,3 | 62% | 1,23 | 2.300 | 0,39 |
| WP-bodem D5a/passief | 55,6 | 24,2 | 57% | 0 | 0 | 0,36 |

WP-buiten D2/BB+ heeft TOjuli 1,23 — **net boven** de eis 1,20 (PDF p.7: vrijstaande L voldoet bij BB+/D2-combinaties net niet aan TOjuli, 1,23). Nuttig als near-boundary/negatief-signaal voor de TOjuli-indicator. Deze case is de enige met gepubliceerde EPC-context (kolom EPC* p.14).

Volledige provenance per waarde: `expected.json`. Best-effort invoer: `input.json`.
