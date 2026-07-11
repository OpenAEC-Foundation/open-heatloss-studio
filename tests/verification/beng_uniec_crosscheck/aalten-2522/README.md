# Aalten 2522 — certified Uniec BENG-crosscheck

Grondgebonden woning, **2522 Woning Aalten**. Certified met **Uniec 3.3.2.1 / BengCert**.

- **Invoer:** `input.oes.json` (kopie van `open-energy-studio/training-data/2522-woning-aalten.oes.json`, John Heikens, LGPL-3.0). Bevat het volledige `project{}`-blok (1 rekenzone, 1 verwarming, 1 ventilatie, 1 koeling, 1 tapwater, 1 PV, 3 constructies) + `meta`.
- **Expected:** `expected.json`, waarden EXACT uit `meta.uniecReference` (JSON-paden in `provenance`).

| Indicator | Certified | Limiet |
|---|---|---|
| BENG 1 | 103,69 | ≤ 123,39 |
| BENG 2 | 24,71 | ≤ 30,0 |
| BENG 3 | 85,0% | ≥ 50 |
| Label | A+++ | — |

Sub-totalen (primair, kWh): verwarming 2551 · tapwater 1813 · koeling 422 · ventilatoren 443 · PV-opbrengst 3811 · koelbehoefte 873.

Toleranties: BENG 1 ±6%, BENG 2 ±10% (ruimer dan Gouda: lagere absolute BENG 2 ⇒ hogere relatieve gevoeligheid), BENG 3 ±3 pp. Zie `../README.md` voor de gedeelde kanttekening.
