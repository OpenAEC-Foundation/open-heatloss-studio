# EP-W203f — Elektrische warmtepomp (buitenlucht)

EPW001 waarbij de HR107-combiketel voor **ruimteverwarming** wordt vervangen door
een **elektrische lucht/water-warmtepomp** (bron buitenlucht, ontwerptemperatuur-
klasse 55/47, COP conform tabel 9.28). ISSO 54 v2.0, §2.4.3 deeltest f (p25).

## Delta

| Kenmerk | EPW001 | EPW203f |
|---|---|---|
| Verwarmingsopwekker | HR107-combiketel (η=0,95, gas) | elektrische WP buitenlucht |
| Ontwerptemperatuur | 45/40 | 55/47 |
| COP | n.v.t. | voldoet aan tabel 9.28 |

**Tapwater:** EPW203 test uitsluitend de verwarmingsopwekker; de tekst noemt geen
tapwater-wijziging. Of tapwater bij de HR107-combi blijft of naar de WP gaat is
niet expliciet en volgt uit Bijlage 2 — te documenteren bij fase-2-activatie
(zie `input.json` → `delta.dhw_note`).

## Waarom deze variant

De lucht-WP is de F3d-4-relevante generator. De overgang gas→elektrisch verschuift
EP2 (primair) en EP3 (aandeel hernieuwbaar) sterk t.o.v. de gasketel-referentie —
diagnostisch voor de opwekkings- en primaire-energie-tak. Getallen uit Bijlage 2.

## Status

Ag/Als ongewijzigd (96 / 247,2). Energie-eindwaarden geblokkeerd op Bijlage 2-Excel.
Zie `../README.md`.
