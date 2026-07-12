# BENG RVO-voorbeeldconcepten — eindwaarde-goldens (rood/`#[ignore]`)

Officiële BENG-eindwaarden uit de RVO-publicatie **BENG voorbeeldconcepten woningbouw** (DGMR, rapport B.2017.1387.02.R001 v003, 26-03-2021), als golden-vangrail vóór `compute_beng` (fase F2) bestaat.

Bron-PDF: `tests/references/rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf` (21 p.).

## Drie cases × drie concepten

| Case | Gebouwcode | Ag [m²] | Als/Ag | BENG 1-eis | Concepten |
|---|---|---|---|---|---|
| `tussenwoning-m-g13` | G13 | 87 | 2,03 | ≤ 70,9 | WP-bodem C4c/BB+ · WP-buiten D2/BB+ · WP-bodem D5a/passief |
| `hoekwoning-m-g11` | G11 | 133 | 1,87 | ≤ 66,2 | idem |
| `vrijstaande-l-g12` | G12 | 181 | 2,14 | ≤ 74,1 | idem |

Naamgeving-correctie t.o.v. de eerste opdrachtformulering: de derde referentie is **Vrijstaande L** (massief, Ag 181) — "Vrijstaande M Herten" bestaat wél in de PDF maar is een markt-case (gemengd licht, +5 kWh/m² eisophoging) en géén BENG-referentie. Zie plan §C3-beslispunt 3.

Eisen: PDF p.7, tabel 5. Resultaten: PDF p.13 (tussen/hoek) en p.14 (vrijstaand), Bijlage 1.

## Versie-caveat (waarom starttolerantie ±10%)

De referenties zijn gerekend met **NTA 8800:2020** via de validatietool **v1.49** (28-10-2020, incl. NEN-interpretatiedocument). Onze engine implementeert **NTA 8800:2025+C1:2026** (de geldende norm — plan-beslispunt 1, akkoord 11-07). Deze versie-delta plus een niet-deterministische geometrie-reconstructie (zie hieronder) rechtvaardigen een ruime starttolerantie:

- BENG 1 / BENG 2: **±10%**
- BENG 3: **±5 procentpunt**
- TOjuli: **±0,25** (absoluut; veel referentiewaarden zijn 0 bij bodemkoeling)

Elke aanscherping in F3 is winst; elke verruiming is verboden zonder normanalyse (anti-fudge, isso53 §6.1/§6.2-precedent). Toleranties staan per case in `expected.json.tolerance`.

## Geometrie-oordeel: RVO Referentiegebouwen 2017 als invoerbron

Beoordeeld: `tests/references/referentiegebouwen-beng-2017.pdf` (RVO/DGMR e2015137100r001v2, 101 p., gedownload in F0).

**Bevinding — deels bruikbaar, niet voldoende:**

- ✅ **Bevestigt de referentiewoningen mét exact matchende Ag**: tabel 1 (p.10) noemt nr. 4 "Woning M tussen" (Ag 87), nr. 2 "Woning M hoek" (Ag 133), nr. 3 "Woning L vrij" (Ag 181) — identiek aan tabel 1/5 van het 2021-rapport. Dit zijn dezelfde BENG-referentiegebouwen.
- ✅ **Kwalitatieve geometrie**: bouwlagen, daktype (plat vs hellend), tuinpui-oriëntatie (zuidwest), dakkapel-positie (NO bij vrijstaand), dwarskap (hoek). Par. 4.1–4.4, p.16–20.
- ❌ **GEEN numerieke per-gevel geometrie**: geen gevelvlakken (m²) per oriëntatie, geen raamoppervlak per gevel, geen kozijnfracties. Die zitten óók hier in een niet-gepubliceerde Bijlage 4 (p.11 en p.68 verwijzen expliciet naar "bijlage 4" / de "uittrekstaat van verliesoppervlaktes"); de PDF-body geeft alleen 3D-afbeeldingen (beeldpagina's 15/19).
- ⚠️ **Concepten wijken af**: de 2017-concepten (Rc/U/PV/installaties) verschillen van de 2021-B10/B12-pakketten. 2017 kan dus ook de envelope-invoer niet leveren — die komt uit 2021 tabel 3.

**Conclusie:** de per-gevel geometrie voor deterministische invoer-reconstructie **wacht op RVO "Bijlage 4" (Excel)** — voor beide PDF's dezelfde ontbrekende bron. Het 2017-PDF is bewaard als geometrie-context (typologie + Ag-bevestiging + oriëntatie), niet als invoerbron. Zie plan §C3-beslispunt 2 (user vraagt Bijlage 4 op bij RVO/DGMR).

## Wat bewust op `null`/ontbrekend staat

- **Alle per-gevel geometrie** in `input.json._missing` (gevelvlakken, ramen per oriëntatie, volume, verliesoppervlakte Als). De `input.json`-bestanden zijn best-effort documentatie, **niet** machine-inleesbaar door `compute_beng`.
- **Opwekker-kentallen** (COP/η WP, WTW-SFP, tapwater-rendement, PV-oriëntatie/tilt): niet in de PDF (Bijlage 4 tabbladen RAPPORT I/V).
- **Koudebrug-ψ-waarden**: uitgebreide methode, SBR-details type T2 (passief) — numeriek in Bijlage 4.
- **EPC** is alleen bij `vrijstaande-l-g12` als context meegenomen (PDF geeft EPC daar; bij tussen/hoek niet doorgerekend).

## Status

🔴 **Rood/`#[ignore]`** — **niet** door de engine geblokkeerd: `compute_beng` bestaat sinds F2 en is voor de Uniec-cases end-to-end aangesloten. Deze drie RVO-cases blijven `#[ignore]` op een **invoer-provenance-blokkade**: de per-gevel-geometrie (gevelvlakken m², ramen per oriëntatie, Als) staat niet in de publieke PDF's maar in de niet-gepubliceerde RVO Bijlage 4 (Excel). `input.json` is daarom documentatie-only en niet machine-inleesbaar. Zodra Bijlage 4 er is, verloopt de reconstructie analoog aan `oes_to_projectv2` (Uniec-harnas). Harnas: `crates/openaec-project-shared/tests/beng_golden.rs`.
