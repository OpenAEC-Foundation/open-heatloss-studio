# Uniec 3 velden-inventarisatie — golden-case 2522 Woning Aalten

**Datum:** 2026-07-12
**Doel:** de volledige invoer van een certified Uniec 3-berekening inventariseren als
referentie voor het BENG-invoermodel van open-heatloss-studio. Aanleiding: het
BENG-invoermodel wordt heroriënteerd van **ruimte-georiënteerd** (warmteverlies-modeller,
binnen-oppervlakten) naar **gevel-georiënteerd, zo dicht mogelijk bij Uniec 3**, omdat
NTA 8800 buiten-oppervlakten per gevel vraagt. De Q_H;nd-kalibratiegap (BENG 1 −26 % op deze
case) is bewezen demand-model-breed; verkeerde geometrie-invoer is de hoofdverdachte
(zie `../tests/verification/beng_uniec_crosscheck/aalten-2522/README.md`).

## 1. Bron & capture-methode

| Aspect | Waarde |
|---|---|
| Berekening | **2522 Woning Aalten** — grondgebonden woning, nieuwbouw, bouwjaar 2024 |
| Golden-case | `tests/verification/beng_uniec_crosscheck/aalten-2522/` (identieke case) |
| Uniec-versie | **3.3.8.0** (waargenomen in de UI-footer; het certificaat in de golden-README noemt 3.3.2.1 / BengCert — versie-drift, zie kanttekening) |
| Capture | Geautomatiseerde Playwright-walk, **alleen-lezen** — geen invoer gewijzigd |
| Ruwe dump | `…/scratchpad/uniec_dump/walk_1530142/` → `fields.json` (20 pagina's: kind/label/value + `selectOptions` + tabelkoppen), `nav.json` (navigatiestructuur), `p01.png`–`p20.png` |
| Gearchiveerd in repo | `tests/verification/beng_uniec_crosscheck/aalten-2522/uniec_fields_capture.json` (kopie van `fields.json`) |

> **Capture-artefacten (belangrijk voor lezing):**
> 1. In grid-lay-outs (bibliotheek, begrenzing, constructie-ramen) hebben de md-selects een
>    **leeg `label`** — het kolomlabel staat als kop bóven het grid. De kolomstructuur is
>    gereconstrueerd uit de bijbehorende screenshot + de interne veld-codes.
> 2. De walk kreeg voor **Wand (O)**, **Wand (W)** en **Koeling 1** een **stale/fallback-view**
>    terug (respectievelijk de "Indeling gebouw"-pagina en de "Installaties"-overzichtstegels)
>    i.p.v. de detailpagina — vermoedelijk een traag-ladende Angular-route. Die drie pagina's
>    zijn hieronder aangevuld uit de betrouwbaar-gecapturede zustergevels + de screenshots
>    (p10, p12, p19). Vloer, Wand N, Wand Z en Dak N zijn wél volledig gecapturet.

## 2. Datamodel van Uniec — de boomstructuur

Uniec modelleert **gebouw-first en gevel-first**, niet ruimte-first. De hiërarchie:

```
Gebouw (2522 Woning Aalten)
├── Algemene gegevens          type gebouw, soort bouw, bouwjaar, opname, plaats
├── Bouwkundige bibliotheek    herbruikbare constructie- en kozijn-definities (Rc / U / g),
│                              elk met een interne code, gerefereerd door de gevels
├── Indeling gebouw(en)
│   └── Rekenzone "woning"      A_g = 67,00 m², bouwwijze vloer/wand, woningtype
│       └── Begrenzing "woning" 6 begrenzingsvlakken = de thermische schil
│           ├── vlak-type       VLAK_VLOER / VLAK_GEVEL / VLAK_DAK / kelderwand
│           ├── grenst-aan      VL_MV_GRSP / GVL_BTNL_{N..NW} / DAK_BTNL_* / AOS/AOR / water
│           ├── oriëntatie      8-punts kompas (in de grenst-aan-code) + HOR voor dak
│           ├── bruto opp [m²]  BUITEN-oppervlak per vlak
│           └── helling [°]
│       └── Constructies/{vlak} per begrenzingsvlak één constructiepagina:
│           ├── opake constructie  → referentie naar bibliotheek (Rc)
│           └── ramen/deuren[]     → per stuk: kozijnmerk-ref (bibliotheek) +
│                                    belemmering + zonwering + zomernachtventilatie
├── Luchtdoorlaten             gebouwhoogte, infiltratie-invoermethode, qv10, verticale leidingen
└── Installaties               Verwarming / Warm tapwater / Ventilatie / Koeling / PV
```

### Referentie-enums (Uniec interne codes uit de dump)

Deze codes zijn Uniecs eigen typologie en zijn het meest waardevolle deel van de capture —
ze zijn preciezer dan onze huidige `BoundaryKind`. Neem ze op als referentie.

**Vlak-type** (`BEGR_VLAK`): `vloer` · `vloer boven buitenlucht` · `gevel` · `dak` · `kelderwand`

**Vloer grenst-aan** (`BEGR_VLOER`, code `VL_MV_GRSP` = "op/boven mv; boven grond/spouw"):
`op/boven mv; boven kruipruimte` · `op/boven mv; boven grond/spouw (z ≤ 0,3)` ·
`op/boven mv; boven onverwarmde kelder` · `onder mv; boven kruipruimte` ·
`onder mv; boven grond/spouw (z ≤ 0,3)` · `onder mv; boven onverwarmde kelder` ·
`water` · `AVR` · `AOS forfaitair` · `AOR forfaitair`

**Gevel grenst-aan** (`BEGR_GEVEL`, codes `GVL_BTNL_{Z,ZW,W,NW,N,NO,O,ZO}`):
`buitenlucht, {Z,ZW,W,NW,N,NO,O,ZO}` · `sterk geventileerd` · `water` · `AVR` ·
`AOS forfaitair; {richting}` · `AOR forfaitair`

**Dak grenst-aan** (`BEGR_DAK`, codes `DAK_BTNL_*`): idem gevel + `buitenlucht; HOR` en
`AOS forfaitair; HOR`

(AVR = aangrenzende verwarmde ruimte; AOS = aangrenzende onverwarmde serre/ruimte;
AOR = aangrenzende onverwarmde ruimte. Deze forfaitaire begrenzingen ontbreken volledig in
ons model.)

## 3. Veldentabellen per pagina

### p01 Overzicht resultaten (read-only)
Rapportage-selectie via checkboxes: energieprestatie-indicatoren ✓, -eisen ✓,
risico op oververhitting ✓, energielabel ✓ (verbruiken/gebouwkenmerken/netto warmtebehoefte/
CO₂ uit). Geen invoer.

### p02 Energieprestatie (read-only resultaten)
Geen invoervelden in de capture.

### p03 Risico op oververhitting (TOjuli)
| Veld | Type | Opties | Aalten |
|---|---|---|---|
| aanvullende GTO-berekening | select (`RESULT-TOJULI_AANW_AANV_BER`) | `geen berekeningen aanwezig` · `GTO berekening volgens Omgevingsregeling en GTO ≤ 450` | geen berekeningen aanwezig |

### p04 Algemene gegevens
| Veld | Type | Opties / code | Aalten |
|---|---|---|---|
| omschrijving | tekst | — | 2522 Woning Aalten |
| plaats | search | — | Aalten |
| type gebouw | select (`GEB_TYPEGEB`) | grondgebonden woning · appartementengebouw · appartement · vakantiewoning · woonwagen · woonboot (2 varianten) | grondgebonden woning (`TGEB_GRWON`) |
| soort bouw | select (`GEB_SRTBW`) | nieuwbouw · bestaande bouw - niet gerenoveerd · bestaande bouw - gerenoveerd | nieuwbouw (`NIEUWB`) |
| bouwjaar | tekst | — | 2024 |
| eigendom | select (`GEB_EIGEND`) | koop · huur · combinatie · onbekend | koop (`GEBEIGEND_KOOP`) |
| opname | select (`GEB_OPN`) | detailopname | detailopname (`OPN_DETAIL`) |
| datum berekening | tekst | — | 31-10-2024 |
| opmerkingen | textarea | — | (leeg) |

### p05 Bouwkundige bibliotheek
Twee categorieën definities, elk met een interne UUID die de constructiepagina's refereren.

**Opake constructies** (`LIBCONSTRD_TYPE` / methode `VRIJE_INV` = vrije invoer):
| Omschrijving | Type | Rc [m²K/W] |
|---|---|---|
| Vloer | `LIBVLAK_VLOER` | 3,70 |
| Wand | `LIBVLAK_GEVEL` | 4,70 |
| Dak | `LIBVLAK_DAK` | 6,30 |

Type-opties: `vloer` · `vloer boven buitenlucht` · `gevel` · `dak` · `kelderwand` · `bodem`.
Methode-opties: `vrije invoer` · `beslisschema`.

**Kozijnmerken** — modus `oppervlakte per kozijnmerk invoeren` (`KOZKENM_OPP`).
Kolommen per merk: U [W/m²K] · ggl (g-waarde) · **oppervlakte [m²]** (totaal per merk).
Type-opties (`LIBCONSTRT_TYPE`): `raam` · `deur` · `paneel in kozijn`.
| Merk | Type | U | ggl | opp [m²] |
|---|---|---|---|---|
| A | raam | 1,3 | 0,40 | 4,12 |
| B | raam | 1,3 | 0,40 | 0,56 |
| C | raam | 1,3 | 0,40 | 0,36 |
| D | raam | 1,3 | 0,40 | 0,97 |
| D deurglas | raam | 1,3 | 0,40 | 0,53 |
| D deur | deur | 2,0 | 0,00 | 1,84 |
| E | raam | 1,3 | 0,40 | 0,91 |
| F | raam | 1,3 | 0,40 | 2,00 |
| G | raam | 1,3 | 0,40 | 3,59 |
| H | raam | 1,3 | 0,40 | 4,57 |
| I | raam | 1,3 | 0,40 | 1,20 |
| J | raam | 1,3 | 0,40 | 4,66 |
| dakraam | raam | 1,3 | 0,40 | 1,20 |

### p06 Indeling gebouw(en)
| Veld | Type | Opties / code | Aalten |
|---|---|---|---|
| energieprestatie berekenen | select | per gebouw / … | per gebouw |
| rekeneenheid | select (`RZFORM_CALCUNIT`) | — | rekenzone / per gebouw (`RZUNIT_GEB`) |
| bouwwijze vloer | select (`RZ_BOUWW_VL`) | — | massief beton (zeer zwaar) (`CONSTRM_FL_26`) |
| bouwwijze wand | select (`RZ_BOUWW_W`) | — | hsb, sfb of staalskeletbouw (licht) (`CONSTRM_W_11`) |
| woningtype | select (`UNIT_TYPEWON`) | — | vrijstaand met kap (`TWON_VRIJ_K`) |
| gebruiksfunctie | select | — | woning |
| rekenzone-omschrijving / aantal | tekst | — | "woning" / 1 |
| unit-omschrijving | tekst | — | "Woning" |
| **A_g gebruiksoppervlak [m²]** | tekst | — | **67,00** |

### p07 Begrenzing "woning" — de thermische schil (KERNPAGINA)
6 begrenzingsvlakken. Kolommen (uit screenshot p07 + codes): omschrijving · vlak-type ·
grenst-aan · **bruto BUITEN-opp [m²]** · helling [°].
| Vlak-omschr | Vlak-type | grenst-aan (code) | bruto opp [m²] | helling [°] |
|---|---|---|---|---|
| vloer | VLAK_VLOER | op/boven mv; boven grond/spouw z≤0,3 (`VL_MV_GRSP`) | 67,00 | n.v.t. |
| Wand | VLAK_GEVEL | buitenlucht, N (`GVL_BTNL_N`) | 21,96 | 90 |
| Wand | VLAK_GEVEL | buitenlucht, O (`GVL_BTNL_O`) | 23,81 | 90 |
| Wand | VLAK_GEVEL | buitenlucht, Z (`GVL_BTNL_Z`) | 39,86 | 90 |
| Wand | VLAK_GEVEL | buitenlucht, W (`GVL_BTNL_W`) | 23,81 | 90 |
| Dak | VLAK_DAK | buitenlucht, N (`DAK_BTNL_N`) | 69,30 | 15 |

Som gevels 109,44 + dak 69,30 = 178,74 m² verliesoppervlak (sluit aan op A_ls ≈ 177,6 in de
golden na raam-verrekening; vormfactor A_ls/A_g ≈ 2,65).

### p08 Constructie — vloer
| Veld | Waarde |
|---|---|
| constructie-ref (bibliotheek) | Vloer (Rc = 3,70) |
| omtrek van het vloerveld P [m] | 32,92 |
| opp [m²] (`CONSTRD_OPP`) | 67,00 |

### p09 Constructie — Wand (N)
Opaak: Wand (Rc = 4,70). Geplaatste kozijnmerken (elk: merk-ref · belemmering · zonwering ·
zomernachtventilatie):
| Kozijnmerk | belemmering (`CONSTRT_BESCH`) | zonwering | zomernachtvent. |
|---|---|---|---|
| D | minimale belemmering (`BELEMTYPE_MIN`) | geen zonwering (`ZONW_GEEN`) | niet aanwezig |
| D deurglas | zijbelemmering rechts (`BELEMTYPE_ZIJ_RECHTS`) — afstand 0,18 m / breedte 0,05 m / hoek 74° / hoogte ≥2,5 m | geen zonwering | niet aanwezig |
| D deur | n.v.t. (opake deur) | n.v.t. | niet aanwezig |
| E | minimale belemmering | geen zonwering | niet aanwezig |
| I | minimale belemmering | geen zonwering | niet aanwezig |

Belemmering-opties (`CONSTRT_BESCH`): `minimale belemmering` · `constante belemmering` ·
`constante overstek` · `zijbelemmering rechts/links/beide` · `volledige belemmering` ·
`overige belemmering` · `constante overstek & (zij)belemmering` · `eigen waarde beschaduwing`.
Zonwering-opties (`CONSTRT_ZONW`): `geen zonwering` · screens/jaloezieën/rolluiken (buiten,
per kleur) · gemetalliseerde weefsels (binnen) · uitval-/knikarmschermen · vaste lamellen.
Zomernachtventilatie (`CONSTRT_ZNVENT`): `niet aanwezig` · `aanwezig`.

### p10 Constructie — Wand (O)  ⚠ stale capture
Opaak: Wand (Rc = 4,70). Ramen niet betrouwbaar gecaptured (walk gaf Indeling-fallback).
Verifieer tegen screenshot p10. Begrenzing-opp 23,81 m² (uit p07).

### p11 Constructie — Wand (Z)
Opaak: Wand (Rc = 4,70). Geplaatste kozijnmerken: **H**, **C**, **J** (elk minimale
belemmering / geen zonwering / zomernachtvent. niet aanwezig, conform het N-patroon;
verifieer detail tegen screenshot p11).

### p12 Constructie — Wand (W)  ⚠ stale capture
Opaak: Wand (Rc = 4,70). Ramen niet betrouwbaar gecaptured. Verifieer tegen screenshot p12.
Begrenzing-opp 23,81 m² (uit p07).

### p13 Constructie — Dak (N)
Opaak: Dak (Rc = 6,30). Geplaatst kozijnmerk: **dakraam** (U 1,3 / ggl 0,40, opp 1,20 m²).

### p14 Luchtdoorlaten (infiltratie)
| Veld | Type | Opties / code | Aalten |
|---|---|---|---|
| buitenwerkse gebouwhoogte [m] | tekst | — | 5,00 |
| invoer infiltratie | select (`INFIL_INVOER`) | — | meetwaarde voor infiltratie - per gebouw (`INFIL_MWG`) |
| verticale leidingen thermische schil | select (`VLEIDING_INVOER`) | — | onbekend (`VLEIDINGL_ONBEKEND`) |
| **qv10 [dm³/(s·m²)]** per gebouw | tekst | — | **0,40** |

Dit is de gemeten luchtdichtheid uit de golden (`airTightness.qv10 = 0,40`). Ligt *onder* het
forfait (≈0,98 voor deze vrijstaande woning); injectie via `q_v10_spec_dm3_s_m2` verlaagt
Q_H;nd juist licht — de onderschatting komt dus niet van de infiltratie-invoer.

### p15 Installaties (overzicht)
Vijf systemen, elk `aantal identieke systemen` = 1, gekoppeld aan rekenzone "woning".
Tapwater: # badruimten 1 · # keukens 1.

### p16 Verwarming 1
| Veld | Opties / code | Aalten |
|---|---|---|
| type opwekker (`VERW-OPWEK_TYPE`) | — | warmtepomp - elektrisch (`_TYPE_A`) |
| invoer opwekker (`VERW-OPWEK_INVOER`) | forfaitair / eigen waarde | eigen waarde opwekkingsrendement (`_EIG_A`) |
| functie(s) van opwekker (`VERW-OPWEK_FUNCTIE`) | — | verwarming en warm tapwater (`_VT`) |
| gemeenschappelijk | — | niet-gemeenschappelijke installatie |
| bron warmtepomp | search | buitenlucht (afgifte water) |
| COP | tekst | **4,10** |
| warmtebehoefte systeem [kWh] | read-only | 6852 |
| hulpenergie [kWh] | read-only | 154 |
| type distributiesysteem | select | eenpijpssysteem |
| ontwerp aanvoertemperatuur | select | onbekend |
| afgifte | select | afgifte alleen oppervlakteverwarming |
| type afgiftesysteem hoofdvertrek | select | vloerverwarming |
| type ruimtetemperatuur-regeling | select | centrale regeling met naregeling per ruimte |
| ventilatoren afgifte | select | geen ventilatoren aanwezig |

### p17 Warm tapwater 1
| Veld | Opties / code | Aalten |
|---|---|---|
| type opwekker (`TAPW-OPWEK_TYPE`) | — | warmtepomp - elektrisch (`_TYPE_1`) |
| invoer opwekker (`TAPW-OPWEK_INV`) | forfaitair / … | forfaitair (`_FORF`) |
| voorraadvat (`TAPW-OPWEK_INDIR`) | — | warmtepomp met geïntegreerd voorraadvat (`_GEINT`) |
| functie(s) | — | warm tapwater |
| bron warmtepomp | search | buitenlucht (afgifte water) |
| COP | read-only | 1,40 |
| warmtebehoefte tapwater [kWh] | read-only | 1750 |
| circulatieleiding | select | geen circulatieleiding aanwezig |
| leidinglengte naar badruimte | select | 2 - 4 m |
| leidinglengte naar aanrecht | select | 4 - 6 m |

### p18 Ventilatie 1
| Veld | Opties / code | Aalten |
|---|---|---|
| ventilatiesysteem (`VENT_SYS`) | — | Dc. mechanische toe- en afvoer - centraal (`VENTSYS_MECHC`) |
| invoer ventilatiesysteem (`VENT_INVOER`) | forfaitair / … | forfaitair (`VENT_FORF`) |
| systeemvariant | search | D.2 centrale WTW-installatie zonder zonering, zonder sturing |
| fctrl | read-only | 1,00 |
| passieve koeling (`VENT_PKOEL`) | — | geen passieve koelregeling (`_GEEN`) |
| invoer WTW-toestel (`WARMTETERUG_INV`) | forfaitair / … | forfaitair (`WTWINV_FORF`) |
| type warmteterugwinning (`WARMTETERUG_TYPE`) | — | onbekende WTW (`WARMTETYPE_ONB`) |
| rendement WTW | read-only | 0,000 (forfait wordt toegepast) |
| invoer ventilatorvermogen | select | forfaitair ventilator vermogen |
| geïnstalleerde ventilatiecapaciteit | select | onbekend |
| luchtdichtheidsklasse kanalen | select | onbekend |

### p19 Koeling 1  ⚠ detail stale (uit screenshot p19)
Systeem: **compressiekoeling - elektrisch**. SEER/EER-detail niet in de capture — overnemen uit
`input.oes.json` van de golden.

### p20 PV 1
| Veld | Opties / code | Aalten |
|---|---|---|
| aangesloten achter meter van (`PV_INVOER`) | — | gebouw (`PVINVOER_GEB`) |
| invoer wattpiek (`PV_WATTPIEK`) | — | productspecifiek Wp/paneel (`PVWATTPIEK_PRDTPNL`) |
| PV gedeeld (`PV_GEM`) | — | niet gedeeld (`PVGEM_NIET`) |
| product | search | DMEGC DM410M10-54HBB |
| Wp per paneel | read-only | 410 |
| veroudering per jaar [%] | read-only | 0,50 |
| **oriëntatie** (`PV-VELD_ORIE`) | 8-punts kompas | **noord (`PVORIE_N`)** |
| bouwkundige integratie (`PV-VELD_BOUWINTRG`) | — | matig geventileerd (`PVINTGR_MATIGVENT`) |
| belemmering (`PV-VELD_BELEM`) | — | minimale belemmering (`BELEMTYPE_MIN`) |
| **aantal panelen** | tekst | **10** (→ 4,1 kWp) |
| **helling [°]** | tekst | **15** |

## 4. Mapping-analyse naar ProjectV2

Referentie: `crates/openaec-project-shared/src/{energy,shared,geometry}.rs`.

### 4a. Installaties → `energy.rs` (`EnergyInput`) — sterke 1:1 aansluiting

| Uniec-pagina | Uniec-waarde | ProjectV2-tegenhanger | Status |
|---|---|---|---|
| Verwarming: warmtepomp elektrisch, buitenlucht | — | `HeatingInput.generator = HeatPumpAir` | ✅ |
| Verwarming: COP 4,10 | — | `HeatingInput.cop = 4.10` | ✅ |
| Verwarming: afgifte vloerverwarming | — | `HeatingInput.emission = FloorHeating` | ✅ |
| Verwarming: eenpijps distributie / naregeling per ruimte | — | `distribution_efficiency` + `control_factor` (forfaitair) | ⚠ alleen als getal, geen distributietype-enum |
| Tapwater: warmtepomp, COP 1,40 | — | `DhwInput.generator = HeatPump`, `efficiency = 1.40` | ✅ |
| Tapwater: geïntegreerd voorraadvat | — | — | ❌ ontbreekt |
| Tapwater: circulatieleiding + leidinglengte-klassen (bad/aanrecht) | — | — | ❌ ontbreekt (distributieverlies-invoer) |
| Ventilatie: systeem D centraal WTW | — | `VentilationInput.system = D` | ✅ |
| Ventilatie: WTW forfaitair "onbekend" | — | `wtw_efficiency = None` → forfait | ✅ |
| Ventilatie: systeemvariant D.2 / zonering / sturing | — | — | ⚠ opgevangen in forfait |
| Koeling: compressiekoeling elektrisch | — | `CoolingInput.generator = Compression` (+ `seer`) | ✅ |
| PV: 4,1 kWp, noord, 15° | — | `PvInput.peak_power_kwp/azimuth_degrees/tilt_degrees` | ✅ (azimut N = 0°) |
| PV: integratie "matig geventileerd" + veroudering 0,50 % | — | `system_efficiency`/`shadow_factor` (benadering) | ⚠ geen expliciete integratie-/verouderingsinvoer |
| Automatisering (BACS) | — (niet in woning-detail) | `AutomationInput` | n.v.t. |

**Conclusie 4a:** het installatie-invoermodel is grotendeels compleet en gevel-neutraal — hier
zit de kalibratiegap **niet**. Kleine hiaten: tapwater-distributieverlies (circulatie +
leidinglengtes), voorraadvat-type, ventilatie-systeemvariant, PV-integratie/veroudering.

### 4b. Geometrie → `geometry.rs` (`SharedGeometry`) — structureel dichtbij, verkeerd gevuld

`SharedGeometry` is al **constructie-georiënteerd** (`Space` → `Construction` → `Opening`) en
`Opening` draagt zelfs al `movable_shading` + `obstruction` per raam — dat mapt fraai op Uniecs
per-kozijn zonwering/belemmering. De mismatch zit **niet in de schema-vorm** maar in drie
punten:

| # | Uniec-concept | ProjectV2 nu | Gap |
|---|---|---|---|
| 1 | Begrenzing per **gevel** op **rekenzone-niveau**, **buiten-oppervlakten** | `Construction` hangt onder `Space` (kamer); de studio vult `area_m2` met **binnen-oppervlakten** uit de warmteverlies-modeller | **Aggregatieniveau + oppervlaktedefinitie.** Hoofdverdachte Q_H;nd −26 %. |
| 2 | Tweelaags: **Bouwkundige bibliotheek** (Rc/U/g met code) ↔ **plaatsing** op gevel | U/lagen worden **inline** per `Construction`/`Opening` opgegeven; geen bibliotheek/kozijnmerk-concept | Geen hergebruik-/referentielaag; Uniec-kozijnmerk met totaal-opp per merk mist. |
| 3 | Begrenzing-typologie rijk: vloer-subtypes (op/onder mv × kruipruimte/kelder/grond), `AOS/AOR forfaitair`, `sterk geventileerd`, `water`, dak `HOR`, + omtrek P voor vloer-op-grond | `BoundaryKind` = 6 grove waarden; geen vloer-subtype, geen P (omtrek), geen forfaitaire AOS/AOR | Vloer-tot-grond (P/A-methode, P = 32,92 m) en onverwarmde-buffer-forfaits ontbreken. |

Kleiner: Uniec bindt **oriëntatie (8-punts) + helling aan het begrenzingsvlak**; ProjectV2 zet
`orientation_deg` (continu) + `slope_deg` op de losse `Construction`. De mapper
`orientation_from_degrees` (in `nta8800_view`) overbrugt dit al, maar de *invoer* zou aan het
gevel-vlak moeten hangen, niet aan elke constructie.

### 4c. Algemene gegevens → `shared.rs` (`SharedProject`) — grotendeels gedekt
`type gebouw` → `BuildingTypeShared::Woning`; `bouwjaar` → `construction_year`; `A_g` →
`gross_floor_area_m2`; `plaats` → `location`; woningtype "vrijstaand met kap" →
`ResidentialType::Detached`. Geen blokkers. **Bouwwijze** (massief beton / hsb — thermische
massa) heeft geen tegenhanger in `SharedProject` en is relevant voor TOjuli/dynamica.

## 5. Voorstel-blok — "BENG-geometrie-invoer v1" (spec, geen code)

Doel: een gevel-georiënteerde invoerlaag die 1:1 op Uniecs begrenzing + constructie + kozijnmerk
zit, zodat buiten-oppervlakten per gevel de bron van waarheid worden. **Aanbeveling:** introduceer
dit als een apart, additief invoerblok (bv. `beng_geometry` op `ProjectV2`), náást de bestaande
room-georiënteerde `SharedGeometry` — niet als vervanging, zodat de ISSO 51/warmteverlies-tak
ongemoeid blijft. De F2b-orchestrator vertaalt dit blok naar de rekenzone-geometrie voor
`compute_beng`.

Structuur:

```
BengGeometry
├── constructie-bibliotheek: OpaqueConstructionDef[]   { id, omschrijving, kind(vloer/gevel/dak/kelderwand), rc_of_u }
├── kozijn-bibliotheek:       WindowDef[]               { id, omschrijving, type(raam/deur/paneel), u, ggl }
└── rekenzone[]               { id, naam, a_g_m2, bouwwijze_vloer, bouwwijze_wand, woningtype }
    └── gevel[] (begrenzing)  {
          vlak_type            (VLAK_VLOER | VLAK_GEVEL | VLAK_DAK | KELDERWAND),
          grenst_aan           (enum ← §2 referentie-enums: buitenlucht/AOS/AOR/grond+subtype/water/…),
          oriëntatie           (8-punts kompas | HOR),   // alleen gevel/dak
          bruto_buiten_opp_m2,
          helling_deg,
          omtrek_p_m           (optioneel; verplicht bij vloer-op-grond),
          constructie_ref      → OpaqueConstructionDef.id,
          ramen[]              { kozijn_ref → WindowDef.id, aantal, belemmering(enum), zonwering(enum), zomernachtventilatie(bool) }
        }
```

Toelichting keuzes:
- **Kozijn-opp in de bibliotheek** (zoals Uniec `oppervlakte per kozijnmerk`) óf per plaatsing —
  Uniec doet het hier in de bibliotheek; voor BENG volstaat totaal-opp per merk, dus opp mag op
  `WindowDef`. Overweeg opp per plaatsing als één merk over meerdere gevels varieert.
- **`grenst_aan` als rijke enum** neemt Uniecs typologie over (incl. AOS/AOR forfaitair,
  sterk geventileerd, water, vloer-subtypes) — dit dekt gap #3 uit §4b.
- **Belemmering/zonwering per raam** hergebruikt de bestaande `nta8800_model::Obstruction` /
  `MovableSunShading` (al aanwezig op `Opening`) — geen nieuw type nodig.

Aalten-2522 als validatie-fixture voor dit blok: 1 rekenzone (A_g 67,00), 6 gevels
(vloer 67,00 / N 21,96 / O 23,81 / Z 39,86 / W 23,81 / dak 69,30), 3 opake defs (Rc 3,70/4,70/6,30),
13 kozijnmerken (A–J + deurglas + deur + dakraam), qv10 0,40.

## 6. Eindrapport — de 5 grootste gaten

1. **Geometrie-aggregatieniveau (hoofdverdachte Q_H;nd −26 %).** ProjectV2 hangt constructies
   onder `Space` (kamer) met **binnen-oppervlakten**; NTA 8800/Uniec vraagt **buiten-oppervlakten
   per gevel op rekenzone-niveau**. Dit is de kern van de heroriëntatie en de meest waarschijnlijke
   bron van de demand-onderschatting.
2. **Ontbrekende bibliotheek/referentie-laag.** Uniec scheidt constructie-/kozijndefinitie (Rc/U/g
   met code) van plaatsing en hergebruikt merken over gevels; ProjectV2 inlinet U per constructie.
   Zonder deze laag is de invoer niet Uniec-isomorf en foutgevoelig.
3. **`BoundaryKind` te grof.** Mist vloer-subtypes (op/onder maaiveld × kruipruimte/kelder/grond),
   de **omtrek P** voor vloer-op-grond (P/A-methode; Aalten P = 32,92 m), en de forfaitaire
   `AOS/AOR`/`sterk geventileerd`/`water`-begrenzingen. Raakt de transmissie- én bodemtak.
4. **Oriëntatie/helling op verkeerd niveau.** Uniec bindt 8-punts oriëntatie + helling aan het
   begrenzingsvlak; ProjectV2 zet continue azimut op elke losse constructie. Mapper bestaat, maar de
   invoerstructuur wijkt af — bron van dubbele/inconsistente invoer (zie de PV-noord provenance-gap
   in de golden).
5. **Tapwater-distributieverlies & installatie-detail.** `DhwInput` mist voorraadvat-type,
   circulatieleiding en leidinglengte-klassen (bad/aanrecht) die Uniec expliciet vraagt; idem
   ventilatie-systeemvariant en PV-integratie/veroudering. Kleiner dan #1–4, maar telt mee in de
   sub-totaal-afwijkingen (tapwater −7 %, ventilatoren +45 %).

## Bijlage — ruwe capture

- Ruwe dump: `…/scratchpad/uniec_dump/walk_1530142/` (`fields.json`, `nav.json`, `p01.png`–`p20.png`)
- Gearchiveerd in de golden: `tests/verification/beng_uniec_crosscheck/aalten-2522/uniec_fields_capture.json`
- Verifieer p10 (Wand O), p12 (Wand W) en p19 (Koeling detail) tegen de screenshots — die drie
  detailpagina's gaven een stale capture-view.
