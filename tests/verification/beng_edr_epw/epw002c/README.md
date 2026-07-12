# EP-W002c — Detailberekening thermische bruggen

EPW001 waarbij de forfaitaire koudebruggen worden vervangen door een
**detailberekening met expliciete ψ-waarden en lengtes** (ISSO 54 v2.0, §2.2.1
deeltest c, p7). Alle overige kenmerken gelijk aan EPW001.

## Expliciete lineaire koudebruggen (p7)

| Aansluiting | ψ [W/mK] | Lengte [m] |
|---|---|---|
| gevel-gevel | 0,10 | 21,6 |
| gevel-dak | 0,04 | 28,0 |
| vloer-gevel | −0,18 | 28,0 |
| kozijn-gevel | 0,05 | 40,0 |

De PDF toont bij gevel-gevel een doorhaling ("20,8 ~~21,6~~" → 21,6 is de
vastgestelde waarde); en bij kozijn-gevel 40 m.

## Waarom deze variant

Dit is de **enige EDR-woningtest met expliciete ψ·L-invoer**, en raakt exact de
F3d-4-fix (koudebrug-propagatie: Σ ψ·L bij H_D). Zodra Bijlage 2 beschikbaar is,
is dit de scherpste vangrail voor die tak (±1%).

## Status

Ag/Als ongewijzigd t.o.v. EPW001 (96 / 247,2). Alle energie-eindwaarden
geblokkeerd op Bijlage 2-Excel. Zie `../README.md`.
