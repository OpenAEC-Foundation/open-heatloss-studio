# Tussenwoning M (G13) — RVO BENG-golden

BENG-referentie **Tussenwoning M** (levensloopbestendig, plat dak, 2 bouwlagen, 1e verdieping half zo groot). Ag 87 m², Als/Ag 2,03, massief. BENG 1-eis ≤ 70,9 kWh/m²·jr.

Bron: `rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf` — eisen p.7 (tabel 5), resultaten p.13 (Bijlage 1). Zie `../README.md` voor de gedeelde versie-caveat, geometrie-oordeel en tolerantie-motivatie.

## Concepten (expected uit p.13)

| Concept | BENG 1 | BENG 2 | BENG 3 | TOjuli | Wp PV |
|---|---|---|---|---|---|
| WP-bodem C4c/BB+ | 54,8 | 29,3 | 59% | 0 | 0 |
| WP-buiten D2/BB+ | 54,7 | 28,8 | 59% | 0,97 | 1.400 |
| WP-bodem D5a/passief | 45,6 | 23,2 | 52% | 0 | 200 |

Alle drie voldoen aan de eisen (BENG 1 ≤70,9; 2 ≤30; 3 ≥50%; TOjuli ≤1,20). Tussenwoning M voldoet volgens de PDF met álle concepten aan TOjuli.

Volledige provenance (paginanummer + rij + kolomwaarden) per waarde: `expected.json`. Best-effort invoer + ontbrekende geometrie: `input.json`.
