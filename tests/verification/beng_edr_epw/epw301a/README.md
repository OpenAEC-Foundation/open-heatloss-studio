# EP-W301a — Koelinstallatie toegevoegd

EPW001 (dat niet actief koelt) krijgt een **koelinstallatie**: individuele
elektrische compressiekoelmachine met **vloerkoeling** (ontwerptemperatuur 12/16).
EPW301a is de referentiesituatie van de koeltestreeks. ISSO 54 v2.0, §2.5
preambule + §2.5.1 deeltest a (p31).

## Delta

| Kenmerk | EPW001 | EPW301a |
|---|---|---|
| Koeling | geen | compressiekoelmachine (individueel, elektrisch) |
| Afgifte | — | vloerkoeling, 12/16 |
| Distributie | — | geïsoleerde leidingen, pomp aanwezig, geen warmtemeter |

## Waarom deze variant

Activeert de koelvraag (Q_C) en koelopwekking (EER); EPW001 heeft EC;ci = 0.
Diagnostisch belangrijk: in de Uniec-crosscheck ligt de koeling nu +506% (F_sh=1,0,
F3d-4 §engine-gaps) — een EDR-golden op ±1% dwingt die tak af zodra Bijlage 2 er is.

## Status

Ag/Als ongewijzigd (96 / 247,2). Energie-eindwaarden geblokkeerd op Bijlage 2-Excel.
Zie `../README.md`.
