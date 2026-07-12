# EDR-attesteringsgoldens (ISSO 54 v2.0) — `beng_edr_epw`

Golden-laag afgeleid van de **EDR-testset voor EP-woningen** (ISSO 54, deelgebied
EDR attest energieprestatie), versie 2.0, vastgesteld door het CCvD van InstallQ
op **12-05-2022**. Rekent volgens **NTA 8800 (januari 2022)**.

Volledige analyse: `docs/2026-07-12-f3d5-edr-testset-analyse.md`.

## Wat deze laag wél en niet levert

Deze set is het **spiegelbeeld** van `beng_rvo_voorbeeldconcepten`:

- **Invoer:** volledig, normatief en deterministisch uit de PDF-tekst + figuur 1.
  EPW001 is een canonieke ProjectV2-invoer; elke variant is EPW001 + één delta.
- **Eindwaarden:** de resultaatgetallen staan in een **apart Excel-document
  ("Bijlage 2", p67), dat niet in ons bezit is**. Er staat géén resultaatgetal in
  de PDF zelf. Alle energie-eindwaarden (EP1/EP2/EP3/Q_H;nd/TOjuli/deelposten)
  zijn dus geblokkeerd op dat Excel — gemarkeerd met `blocked_on` in `expected.json`.
- **Uitzondering — nu al assertbaar:** `Ag` (96 m²) en `Als` (247,2 m²) staan
  expliciet in de EPW001-tekst (p5). Deze geometrie-golden staat los van het Excel
  én van de nog kapotte PV/energie-keten en is de eerste fase-2-activatie.

## Tolerantie

De officiële EDR-afkeurtolerantie is **±1,0%** (p67): een rekenprogramma wordt
afgekeurd als een resultaat >1% afwijkt. Attesteringsniveau — veel strakker dan
de ±10% RVO-starttolerantie. Zodra Bijlage 2 er is, geldt ±1% op de deelposten.

## Anti-fudge (hard)

Expected-waarden komen uitsluitend uit de bron (PDF-tekst voor Ag/Als; Bijlage 2
voor de rest). Nooit aanpassen aan wat de engine uitrekent. Geblokkeerde
grootheden houden `value: null` + `blocked_on`; ze mogen niet stilzwijgend met een
berekend getal gevuld worden. Het provenance-vangnet in `edr_golden.rs` bewaakt dit.

## Fixtures

| Map | Test | Delta t.o.v. EPW001 | Engine-keten |
|---|---|---|---|
| `epw001/` | EP-W001 | referentie | geometrie (Ag/Als), transmissie, HR107, D2+WTW |
| `epw002c/` | EP-W002c | detail thermische bruggen (ψ·L expliciet) | koudebrug-propagatie (F3d-4) |
| `epw004d/` | EP-W004d | hoofdgevel → Noord | zonwinst per oriëntatie |
| `epw101p/` | EP-W101p | ventilatie D2 → D1 (geen WTW) | ventilatie/WTW + infiltratie |
| `epw203f/` | EP-W203f | HR107 → elektrische WP buitenlucht | WP-opwekking (F3d-4) |
| `epw301a/` | EP-W301a | koelinstallatie toegevoegd (compressie) | koelvraag + koelopwekking |

**Geen PV-fixture:** de EDR-woningbouwset kent geen gebouwgebonden-PV-test (EPW001:
"geen gebouwgebonden productie van elektriciteit"). De PV-keten blijft op de
Uniec-crosscheck leunen. Zie analyse-doc §2.

## Geometrie-conventie (input.json)

`input.json` is ProjectV2-nabij (best-effort documentatie; het harnas valideert
alleen dat het geldige JSON is — reken-invoer voor een toekomstige
`edr_to_projectv2`). Conventie zoals `oes_to_projectv2` in `beng_golden.rs`:
`construction.area_m2` = **bruto** vlakoppervlak (incl. raamopening); ramen zijn
sub-elementen (`openings[]`). Zo telt de zuidgevel 43,2 m² bruto (waarvan 24 m²
raam → 19,2 m² dicht, conform tabel 1) en is Σ bruto-vlakken = A_ls = 247,2 m²
zonder dubbeltelling.
