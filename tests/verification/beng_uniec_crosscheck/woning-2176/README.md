# Woning 2176 — multi-rekenzone golden (MZ-V2a)

Vrijstaande woning met **drie rekenzones** (verdieping-groepen + kelder) binnen
één UNIT — de referentiecasus voor de multi-rekenzone-import (MZ-V2a).

| Kenmerk | Waarde |
|---|---|
| Rekenzones (UNIT-RZ) | 3 — 159,00 + 117,10 + 159,00 m² |
| A_g;tot | **435,10 m²** (Σ zones, = certified `RESULT-OPP_GEBROPP`) |
| Certified BENG 1 / 2 / 3 | 72,49 / 22,00 / 75,9 |
| Label | A+++ |
| App-versie | 3.3.6 |

## Bestand

Het bron-`.uniec3` is **klantdata** en daarom gitignored (`*.uniec3`). Het staat
alleen lokaal in deze map. De test `crates/uniec3-import/tests/multizone_golden.rs`
vindt het via een `*.uniec3`-glob in deze map (de klant-bestandsnaam staat dus niet
in de repo) en skipt netjes als het ontbreekt (CI).

## Waar dit voor pint

- **Importer:** 3 rekenzones geïmporteerd (geen `MultiUnitUnsupported`-afwijzing),
  per-zone A_g/naam/bouwwijze behouden, A_g;tot = 435,10.
- **Indicatief:** de gepoolde `compute_beng` levert een `INDICATIEF (MZ-V2a)`-note.
  De BENG-cijfers worden **gerapporteerd** (delta t.o.v. certified) maar **niet**
  op tolerantie geassert — dit resultaat is bewust indicatief; norm-exact
  per-rekenzone-rekenen is MZ-V2b (NTA 8800 §6.6.2 / §8.2.2).
