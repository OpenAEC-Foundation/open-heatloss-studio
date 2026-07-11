# Gouda 2467 — certified Uniec BENG-crosscheck

Grondgebonden woning, **2467 Goejanverwelledijk 85 Gouda**. Certified met **Uniec 3.3.3.1 / BengCert**.

- **Invoer:** `input.oes.json` (kopie van `open-energy-studio/training-data/2467-goejanverwelledijk-gouda.oes.json`, John Heikens, LGPL-3.0). Bevat het volledige `project{}`-blok (1 rekenzone, 1 verwarming, 1 ventilatie, 1 koeling, 1 tapwater, 2 PV-velden, 3 constructies) + `meta`.
- **Expected:** `expected.json`, waarden EXACT uit `meta.uniecReference` (JSON-paden in `provenance`).

| Indicator | Certified | Limiet |
|---|---|---|
| BENG 1 | 95,86 | ≤ 96,4 |
| BENG 2 | 27,48 | ≤ 30,0 |
| BENG 3 | 83,7% | ≥ 50 |
| Label | A+++ | — |

Sub-totalen (primair, kWh): verwarming 6506 · tapwater 4208 · koeling 244 · ventilatoren 822 · PV-opbrengst 8734 · koelbehoefte 504.

Toleranties: BENG 1 ±6%, BENG 2 ±8%, BENG 3 ±3 pp. Zie `../README.md` voor de gedeelde kanttekening (geometrie is benadering, regressie-golden, WTW-pad ongedekt).
