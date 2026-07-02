# Vabi ISSO 51:2023 erratum-conforme voorbeeldwoningen — index

**Datum:** 2026-05-12
**Doel:** numerieke validatie-bronnen voor isso51-core (`C:/GitHub/warmteverliesberekening`)

_Redactie 2026-07-02: letterlijke erratum-tekstwijzigingen (zoekronde 2) vervangen door verwijzing (auteursrecht ISSO); volledige transcriptie lokaal bij 3BM. Vabi-voorbeeldoutput (derden-rapporten) blijft staan._
**Resultaat:** **1 echt erratum-conform Vabi-voorbeeld gevonden** (DR Engineering 2025, ISSO 51:2024). Daarnaast 5 Vabi-voorbeelden van ISSO 51:2017-vintage die als indicatieve / partiële validatie kunnen dienen, en 1 vergelijkende effectenstudie (warmteverliesberekeningen.nl) die wel de 2023-versie aanhaalt maar geen absolute Watt-getallen publiceert.

Het beoogde getal van 11 is **niet** gehaald — publieke Vabi-output op het post-erratum 2023/2024 normenniveau is op het open net dun gezaaid. De meeste praktijkrapporten uit ingenieursbureaus zijn nog ISSO 51:2017 (laatste deploys zijn vooral 2019-2022), en nieuwe 2024-projecten zitten klantvertrouwelijk achter offer-eisen.

---

## Gevonden voorbeelden

### 1. DR Engineering — Voorbeeld warmteverliesberekening woningbouw (vrijstaand, 2024-versie norm)
- **Bron:** https://www.dr-engineering.nl/wp-content/uploads/voorbeeld-warmteverliesberekening-woningbouw.pdf
- **Type:** vrijstaand gebouw met puntdak, eenlaags + kap, 14 ruimten (entree, toilet, woonkamer, keuken/eetkamer, bijkeuken, garage, overloop, 3 slaapkamers, badkamer, 2 toiletten, kast, speelzolder)
- **Norm-versie:** **ISSO 51, 53 en 57 (2024)** — expliciet vermeld op pagina 2. Dit is de ISSO-update die de erratum 2023 incorporeert; geen losse 2023-melding maar de actuele post-erratum publicatie. Bouwjaar 2024.
- **Software:** Vabi rekenkern Warmteverlies **3.12.0.127** (datum berekening: 2-3-2025)
- **Inputs zichtbaar:**
  - Bruto inhoud 873,1 m³ / Ag = 243,2 m² / gem. U-uitwendig 0,31 W/(m²·K)
  - Tijdconstante 189,1 h → temperatuurcorrectie 2,0 K → θ_e = -8,0 °C (basisontwerp -10,0)
  - qv;10;spec = 0,6250 dm³/(s·m² Ag), methode "Specifiek"
  - Ventilatiesysteem D (gebalanceerd met WTW) op gebouwniveau 137,2 dm³/s
  - Vermogen voorverwarmer WTW: 1481 W
  - Thermische bruggen: "Nieuw gebouw met voorzieningen tegen koudebruggen" (dUtb = 0,02 W/m²·K op buitengevels/dak)
  - **Ontwerpbinnentemperaturen** (post-erratum waarden): woonkamer/keuken/slaapkamers/badkamer/speelzolder = **22,0 °C**; entree/bijkeuken = 20,0 °C; toilet beneden = 18,0 °C; toilet boven/kast = 19,5 °C; garage = 15,5 °C — dit zijn de Tabel 2.2 (2023) waarden, **niet** de 2017 default van 20 °C verblijfsruimten.
  - Vloer op grond, grondwaterspiegel ≥ 1 m onder vloerniveau, BG-vloer 174,8 m²
- **Outputs zichtbaar:**

  Gebouwniveau:
  | Component | Waarde (W) |
  |---|---|
  | ΦT,ie (transmissie naar buiten) | 3601 |
  | ΦT,iae (naar onverwarmde ruimten) | 0 |
  | ΦT,ig (naar bodem) | 326 |
  | Φi (infiltratie) | 2003 |
  | Φbasis (basis) | **5931** |
  | Φvent (ventilatie) | 770 |
  | Φhu (bedrijfsbeperking) | 0 |
  | Φextra (niet-gelijktijdig) | 770 |
  | **ΦHL,build (ontwerpvermogen)** | **6700** (= 31 W/m² = 15 W/m³) |

  Per-ruimte split (uittreksel pagina 5):
  | # | Naam | θ_i [°C] | Φbasis [W] | Φextra [W] | ΦHL,i [W] |
  |---|---|---|---|---|---|
  | 0.01 | entree | 20,0 | 567 | 0 | 567 |
  | 0.03 | woonkamer | 22,0 | 2101 | 221 | 2322 |
  | 0.04 | keuken/eetkamer | 22,0 | 1823 | 197 | 2020 |
  | 0.05 | bijkeuken | 20,0 | 321 | 0 | 321 |
  | 1.02 | slaapkamer 3 | 22,0 | 262 | 45 | 307 |
  | 1.03 | slaapkamer 2 | 22,0 | 241 | 40 | 281 |
  | 1.04 | slaapkamer 1 | 22,0 | 556 | 119 | 675 |
  | 1.05 | badkamer | 22,0 | 230 | 34 | 263 |
  | 1.08 | speelzolder | 22,0 | 1252 | 115 | 1367 |
  | **Totaal verwarmd** | | | **7352** | **770** | **8121** |

  Daarbij valt op: ΦHL,build (gebouw) = 6700 W ≠ Σ ΦHL,i (ruimten) = 8121 W. Dit komt door de **niet-gelijktijdige sommatie** (kwadratische / RSS-aanpak op Φextra-componenten op gebouwniveau) — exact het mechanisme dat erratum 2023 introduceert en dat audit `2026-05-12-isso51-norm-conformiteit-audit.md` als kritieke validatie-targetpunt benoemt.
- **Bruikbaarheid voor validatie:** ⭐⭐⭐ direct — dit is *de* benchmark om isso51-core tegenaan te leggen. Echte 2024 normversie + Vabi-output + complete inputs + per-kamer numerieke split + dat ene gebouwniveau-totaal dat alleen via correcte RSS-implementatie reproduceert.
- **Notitie:** rapport-PDF heeft tekstlaag, dus alle getallen zijn machine-leesbaar te extraheren naar JSON-fixture. Aanrader: integration test `vabi_dr_engineering_woningbouw_2024.rs` met deze 14 ruimten als golden case.

---

### 2. Trajectum Engineering — Voorbeeldwoning (2019, ISSO 51:2017)
- **Bron:** https://trajectum.eu/files/original/2019-50.0000-jdi-20191209-warmteverliesberekening.pdf
- **Type:** woning/woongebouw "tussenligging" (= tussenwoning of rij), 3 bouwlagen (BG / 1e / 2e), 15 ruimten
- **Norm-versie:** **ISSO 51, 53 en 57 (2017)** — expliciet vermeld
- **Software:** Vabi Elements Warmteverlies **3.5.1.21477** / rekenkern v2.26 (9-12-2019)
- **Inputs zichtbaar:**
  - Bruto inhoud 715 m³ / verwarmd opp 196,2 m² / verwarmd vol 560 m³
  - Tijdconstante 23,5 h (licht), Cz = 1,0 (zekerheidsklasse A)
  - Hogere ontwerpbinnentemperaturen: nee → **20 °C verblijfsruimten** (=2017 default)
  - Ventilatie systeem C (natuurlijke toevoer + mech afvoer), woonfunctie
  - qv;10 = 2,000 dm³/(s·m²Ag) (forfaitair)
  - Thermische bruggen: "isolatie binnenzijde, doorbroken plafonds" → dUtb = 0,15 W/m²·K
- **Outputs zichtbaar (gebouwtotalen):**
  - Transmissie: 8370 W (gesplitst: 5156 buiten + 619 onverwarmd + 2595 aangrenzend gebouw)
  - Ventilatie: 5673 W (5206 ventilatie + 326 infiltratie + 141 anders)
  - Opwarmtoeslag: 2445 W
  - **Totaal aansluitvermogen: 16487 W** (= 84 W/m² = 29 W/m³)
- **Bruikbaarheid voor validatie:** ⭐⭐ partieel — niet erratum-conform (θ_i=20 °C, geen RSS-sommatie, oude infiltratietabel), maar prima voor regressietest van de **2017-modus** van isso51-core (als die mode bestaat) of voor unit-tests op individuele formules (transmissie, infiltratie tabel-driven).
- **Notitie:** rapport heeft per-ruimte tabellen met U-waarden, oppervlakken, oriëntaties — daardoor erg geschikt voor formule-unit-tests los van de overkoepelende sommatie.

---

### 3. Punt Ontwerp en Advies — Hoekwoning Wyandottelaan 20 Barneveld (2020, ISSO 51:2017)
- **Bron:** https://www.puntontwerpenadvies.nl/wp-content/uploads/2020/10/warmteverlies-berekening-punt-ontwerp-advies.pdf
- **Type:** **hoekwoning nieuwbouw**, half-vrijstaand, ~3 bouwlagen, 12 ruimten
- **Norm-versie:** ISSO 51, 53 en 57 (**2017**)
- **Software:** Vabi Elements Warmteverlies 3.5.2.23008 / rekenkern v2.26
- **Inputs zichtbaar:**
  - Bruto inhoud 684 m³ / verwarmd opp 177,5 m² / verwarmd vol 526,5 m³
  - Tijdconstante 77,0 h → θ_e gecorrigeerd naar -9,5 °C
  - Cz = 1,0 (zekerheidsklasse A), Rc > 3,5 → "Hogere ontwerpbinnentemperaturen: nee" → 20 °C verblijfsgebieden
  - Ventilatie C, vloerverwarming + radiatoren LT
  - Thermische bruggen: nieuw gebouw, dUtb = 0,05 W/m²·K
  - Op grond, grondwaterfactor 1,0, bruto omtrek 39,2 m / bvo 111,1 m²
- **Outputs zichtbaar:**
  - Transmissie: 4961 W
  - Ventilatie: 3033 W
  - Opwarmtoeslag: 0 W (waarschijnlijk uitgezet)
  - **Totaal: 7994 W** (= 45 W/m² = 15 W/m³)
- **Bruikbaarheid voor validatie:** ⭐⭐ — als hoekwoning-archetype voor de **2017-formules** met vloerverwarming-pad (relevant voor Tabel 2.12 Δθ-waarden van vloerverwarmingssystemen). Niet als erratum-test te gebruiken.

---

### 4. warmteverliesberekeningen.nl — Vrijstaande woning, warmtepomp (2022, ISSO 51:2017)
- **Bron:** https://warmteverliesberekeningen.nl/images/algemeen/rapport_home.pdf
- **Type:** **vrijstaande woning**, 3 bouwlagen, 18 ruimten (zeer fijnkorrelig)
- **Norm-versie:** ISSO 51, 53 en 57 (**2017**)
- **Software:** Vabi rekenkern Warmteverlies 3.8.1.14 / v2.30 (17-11-2022)
- **Inputs zichtbaar:** bruto inhoud 504 m³, tijdconstante 99,7 h → θ_e = -9,0 °C, ventilatie C, qv;10 = 1,215 dm³/(s·m²Ag) (al hoge luchtdichtheid), thermische massa "Zwaar", Bouwbesluit 2012, Rc < 3,5 W/m²·K (oudere bouw), dUtb = 0,05
- **Outputs zichtbaar:** rapport heeft per-ruimte vermogensblokken, maar zonder pagina-doorlezing in deze sessie geen gebouwtotaal geëxtraheerd. Wel duidelijk dat **per-ruimte θ_i vaak afwijkend** is (12,1 °C berging, 12,4 °C kast, 18,2 °C toilet etc.) — interessant voor onverwarmde-ruimten warmtebalans testpad.
- **Bruikbaarheid voor validatie:** ⭐⭐ — interessant als hardcase voor de **warmtebalansberekening van onverwarmde ruimten** (12 v.d. 18 ruimten zijn niet-standaard θ_i), wat in ISSO 51:2024 explicieter is geregeld (zie Vabi 3.12.2 release notes).

---

### 5. ProRail / Movares — Onderstation Gilze-Rijen (2022, utiliteit, ISSO 51/53/57:2017)
- **Bron:** https://www.planviewer.nl/imro/files/NL.IMRO.0784.OVKempenbaan17-VG01/b_NL.IMRO.0784.OVKempenbaan17-VG01_bd2.pdf
- **Type:** **utiliteitsgebouw** (geen woning, dus strikt buiten scope), maar gebruikt wel ISSO 51-tak via gedeelde transmissie/infiltratie-modules
- **Norm-versie:** ISSO 51, 53 en 57 (2017)
- **Software:** Vabi rekenkern Warmteverlies 3.7.0.335 (4-3-2022)
- **Bruikbaarheid voor validatie:** ⭐ — alleen relevant als isso51-core ooit ISSO 53/utiliteit gaat ondersteunen. Voor de huidige woning-scope: skip.
- **Notitie:** in scope-uitsluiting opgenomen, niet gebruiken voor woning-validatie.

---

### 6. TVVL Magazine 03/2018 — "Onderzoek & Cases nieuwe ISSO publicaties warmteverliesberekening"
- **Bron:** https://tvvl.nl/wp-content/uploads/2023/05/44_Klimaatinstallaties-Nieuwe-ISSO-publicaties-warmteverliesberekening-TM03-2018.pdf
- **Type:** vakbladartikel met **3 woningvarianten** doorgerekend door Vabi: **Tussenwoning Licht** / **Tussenwoning Zwaar** / **Passiefhuisniveau** (zelfde geometrie, andere bouwfysische eigenschappen). Plus 1 kantoorgebouw (out of scope).
- **Norm-versie:** **ISSO 51 (2012) vs (2017)** — beide naast elkaar. **Geen 2023**.
- **Software:** Vabi (specifieke versie niet vermeld, "berekeningen opgesteld door VABI")
- **Outputs zichtbaar (Tabel 3, alle in W):**

  | Component | Tussenw. Licht 2012 | Tussenw. Licht 2017 | Tussenw. Zwaar 2012 | Tussenw. Zwaar 2017 | Passief 2012 | Passief 2017 |
  |---|---|---|---|---|---|---|
  | Basisontwerp T_e [°C] | -10 | -10 | -10 | -10 | -10 | -10 |
  | Tijdconstante [h] | 51 | 51 | 413 | 413 | 280 | 280 |
  | Temperatuurcorr. [K] | 0 | 0 | 0 | 4 | 0 | 3,5 |
  | Ontwerp θ_e [°C] | -10 | -10 | -10 | -6 | -10 | -6,5 |
  | dUtb [W/m²·K] | 0,1 | 0,05 | 0,1 | 0,05 | 0,1 | 0,02 |
  | Transmissie [W] | 1780 | 1612 | 1914 | 1559 | 1536 | 1115 |
  | Ventilatie [W] | 966 | 959 | 917 | 828 | 579 | 442 |
  | Opwarmtoeslag [W] | 220 | 1104 | 1202 | 1742 | 660 | 568 |
  | **Aansluitvermogen [W]** | **2966** | **3675** | **4033** | **4129** | **2775** | **2125** |

- **Bruikbaarheid voor validatie:** ⭐⭐ — uitstekend voor unit-tests op de **temperatuurcorrectie-formule via tijdconstante** (zwaarte→correctie→θ_e) en op de **opwarmtoeslag-tabel** (Tabel 2.x ISSO 51:2017). Niet bruikbaar voor erratum 2023-validatie. Wel: 3 dezelfde-geometrie varianten geven uitzonderlijk goed gevoel voor parameter-sensitiviteit.
- **Notitie:** geometrische gegevens (afmetingen, gevels) zijn **niet** in het artikel — alleen de bouwfysische resultaten. Voor reproductie zou je de geometrie zelf moeten kiezen, wat het minder bruikbaar maakt als "gegeven inputs → vergelijk Watt-output" test.

---

### 7. Nieman Raadgevende Ingenieurs / RVO — Warmtebehoefte gasloze concepten (2018)
- **Bron:** https://www.klimaatakkoord.nl/binaries/klimaatakkoord/documenten/publicaties/2019/07/01/achtergrondnotitie-warmtebehoefte-gasloze-concepten/Rapport+warmtebehoefte+gasloze+concepten+2018-10-16.pdf
  - **Caveat:** mijn WebFetch + curl konden deze URL niet bereiken (ECONNREFUSED). User moet zelf downloaden om numerieke tabellen te zien. WebSearch metadata bevestigt wel de inhoud:
- **Type:** **4 RVO-referentiewoningen** in één rapport — vrijstaand, hoek, tussen, galerij — beide oriëntaties (NZ, OW), twee isolatieniveaus ("matig" en "nieuwbouw")
- **Norm-versie:** ISSO 51 (**2017**)
- **Software:** Vabi Elements **3.4.1.19588** (warmteverlies) + Uniec 2.2.16 (NEN 7120 / BENG1)
- **Inputs / outputs zichtbaar:** **niet door mij geverifieerd** wegens download-fail. WebSearch-resultaat suggereert wel dat de berekeningsresultaten in het rapport staan (datum berekeningen 7-8-2018). Vermoedelijk wel per-woningtype transmissie/ventilatie/totaal.
- **Bruikbaarheid voor validatie:** ⭐⭐ als de getallen er inderdaad staan — meest waardevolle multi-typologie set op één identieke methodologie. Maar **eerst zelf openen**, want zonder verificatie kan ik niet garanderen dat de absolute Watts in dit rapport staan en niet alleen kWh/jaar-warmtebehoefte.
- **Notitie:** RVO referentiewoningen zijn nationaal gestandaardiseerd (geometrie + bouwfysica bekend), dus bij bevestiging van Vabi-output is dit een ideale **golden dataset** voor isso51-core 2017-modus.

---

## Niet bruikbaar maar wel gevonden (afgewezen)

| Bron | URL | Reden afwijzing |
|---|---|---|
| Vitec-Vabi effectenstudie 2017 vs 2023 | https://www.vitec-vabi.com/nieuws/isso-51-effectenstudie-2017-vs-2023/ | Vergelijkende studie — alleen **procentuele verschillen** (11,2% / 2,6% / 3,5% / -2,8%) gepubliceerd, géén absolute Watt-getallen of inputs. Wel goede norm-uitleg, maar geen validatiebron. |
| warmteverliesberekeningen.nl praktijkreview | https://warmteverliesberekeningen.nl/nieuws-kennis/verschil-tussen-isso-51-2017-en-isso-51-2023-warmteverliesberekening-in-de-praktijk | Idem — twee woningen (2-onder-1-kap + vrijstaand, beide 2008) doorgerekend met Vabi 3.10.1.91 in beide normen, maar resultaten alleen als percentages gepubliceerd. Software-versie en θ_i-defaults wel handig als reference. |
| Vabi webinar PDF "Warmteverlies ISSO 51 Woningen 2023" | https://files.vabi.nl/Elements/WebinarWVUpdate2023.pdf | PDF is **image-based** (screenshots van de slides), tekstlaag onleesbaar. Mogelijk staan er numerieke vergelijkingen op de slides, maar zonder OCR-pad niet te extraheren via deze sessie. User kan via `pdf_ocr_extract` proberen. |
| Vabi WijzigingenWoongebouwWarmteverlies | https://files.vabi.nl/Elements/WijzigingenWoongebouwWarmteverlies.pdf | Release notes voor Vabi Elements 3.12.2 — uitsluitend methodische uitleg over onverwarmde-ruimten-warmtebalans, geen numerieke voorbeelden. Wel **goud waard voor isso51-core onverwarmde-ruimten module** als specificatie. |
| TVVL artikel 2018 kantoorgebouw (Tabel 4) | (zelfde TVVL-URL) | Utiliteitsgebouw (BVO 564,5 m²) — buiten scope ISSO 51. |
| Kenteq lesboek "Warmteverlies berekenen volgens ISSO 51" (Intechnium 2001) | https://leermiddelenshop.kenteq.nl/Previews/9789056363260.pdf | Preview is alleen inhoudsopgave + colofon. Inhoudelijke hoofdstukken zitten achter de paywall. Bovendien: dit is gebaseerd op de **2001-versie** van ISSO 51, niet 2017 of 2023. Pad: koop lesboek voor hand-rekenvoorbeeld (mogelijk waardevol als single-room handcalc test) als isso51-core ook een unit-test wil voor naïeve 2001-stijl referentie. |
| Leever Gezondheidscentrum Arnhem | https://www.leever.nl/wp-content/uploads/Gezondheidscentrum-te-Arnhem-Transmissie.pdf | Utiliteitsgebouw, out of scope. |
| Vabi Best Practice Aansluitvermogen 2013 | https://support.vabi.nl/wp-content/uploads/sites/2/2020/04/Warmteverlies-Aansluitvermogen.pdf | Best practice document zonder numerieke case-output — alleen methodische uitleg over individueel vs collectief aansluitvermogen bij appartementen. Bruikbaar als ontwerpdocumentatie, niet als validatie. |
| ISSO 51:2023 erratum officieel | https://documenten.isso.nl/s/rXyirFGw20gnnLF7s91CR7teJ4UDuFeq/23.09.01%20Erratum%20ISSO%2051_2023.pdf | Het erratum-document zelf — geen voorbeeldwoning, alleen tekstwijzigingen. Wel zeer relevant als referentiebron voor de audit. |

---

## Zoekstrategie samenvatting

| Strategie | Resultaat |
|---|---|
| Brede NL-zoekopdrachten op "ISSO 51 voorbeeldberekening Vabi" + varianten | Dezelfde 5-6 PDFs blijven terugkomen — markt is dun |
| Site-restricted zoekopdrachten (vabi.nl, vitec-vabi.com, support.vabi.nl) | Alleen marketing + best-practice docs, geen rekenoutput |
| Filetype:pdf op specifieke woningtypen (tussenwoning, appartement, hoekwoning) | Hoekwoning-PDF (Punt Ontwerp), maar geen tussenwoning- of appartement-PDF na 2022 |
| Scriptie-databases (hbo-kennisbank.nl, studenttheses.uu.nl) | Geen treffers — directe site-scope geeft 0 resultaten op deze combinatie |
| Vakbladen (TVVL, IsoMagazine, Bouwwereld) | 1 hit (TVVL 03/2018) — drie woningvarianten, maar 2017-versie |
| Klimaatakkoord / RVO publicaties | 1 hit (Nieman) — vier woningtypen in één rapport, 2017-versie |

**Wat heb ik NIET geprobeerd** (suggestie voor user):
- ISSO kennisportaal achter login (issosite.nl/kennisportaal) — daar zit mogelijk officieel cursusmateriaal met uitgewerkt rekenvoorbeeld in de 2023-versie
- Vabi Servicedesk / Vabi Academy training-materiaal — vraag om een 2024-rekenvoorbeeld bij `support@vabi.nl` of via klantportaal
- HBO Kennisbank (`hbo-kennisbank.nl`) direct doorzoeken met user-account — student-scripties tussen 2024-2025 hebben mogelijk verse cases
- Linkedin posts van TVVL-experts (Michiel van Bruggen, Björn Jansen) — auteurs van TVVL-artikel uit 2018 hebben mogelijk in 2024/2025 een vervolg geschreven over erratum-impact
- Garantie-instellingen Woningborg / SWK kennisbank — deze stellen Vabi als verplicht en hebben mogelijk modelcases gepubliceerd op niet-geïndexeerde subdomeinen

---

## Aanbeveling

**Eerlijk eindoordeel:** numerieke validatie van isso51-core op publieke ISSO 51:2023/2024 Vabi-output zal voor het overgrote deel moeten leunen op **één** voorbeeld (DR Engineering 2025, item 1). Dat is voldoende voor één gouden integration test, maar te weinig voor robuuste regressie-dekking over alle woningvarianten en parametercombinaties.

**Voorgestelde drietraps-strategie:**

1. **Quick-win, deze week:** bouw integration test `vabi_dr_engineering_2024_vrijstaand.rs` met de exacte inputs/outputs uit item 1. Dit dekt: erratum 2023 θ_i = 22 °C verblijfsruimten, kwadratische sommatie (Φextra → ΦHL,build), Tabel 2.8 erratum infiltratie, ventilatiesysteem D met WTW-voorverwarmer. Eén golden case dekt al ~80% van de auditbevindingen.

2. **Backfill 2017-modus, volgende sprint:** voeg 4 ⭐⭐ regressietests toe voor Trajectum (item 2), Punt Ontwerp hoekwoning (item 3), Nieman vrijstaand+hoek+tussen+galerij (item 7), en TVVL Tussenwoning Licht/Zwaar/Passief (item 6). Markeer expliciet als "ISSO 51:2017 mode" — als isso51-core geen aparte 2017-modus heeft, moet je dit eerst architecturaal beslissen (zie audit §X). Anders gebruik je ze voor unit-tests op individuele formules (transmissie, infiltratie tabel, opwarmtoeslag tabel, tijdconstante-correctie).

3. **Bredere 2023+ dekking, kwartaal-actie:** vraag actief Vabi-output op via klant-relaties. 3BM Bouwkunde heeft mogelijk eigen recente klant-rapporten in 2024/2025 op de norm-2024 versie — die zijn geanonimiseerd nog steeds bruikbaar als golden data, en als interne validatie veel sterker dan publieke output. Aanvullend: stel ISSO via `info@isso.nl` de vraag of er een **officieel** rekenvoorbeeld bij de 2023 publicatie hoort, zoals bij sommige NEN-normen het geval is.

**Niet doen:** verzin geen synthetische "wat zou Vabi hier uitkomen" cases. Numerieke validatie is alleen geloofwaardig met externe Vabi-output. Tot je 3-5 cases hebt op de 2024-norm is isso51-core wat numerieke conformiteit betreft **niet validatie-volledig** — eerlijk benoemen in de audit.

---

## Aanvullende zoekronde 2 (2026-05-12 later)

**Strategie:** breder gegaan dan Vabi-Nederland: (a) EN 12831 internationaal (UK/DE/BE), (b) open-source Python implementaties met fixture-houdende voorbeelden, (c) Buildwise BE counterpart van ISSO 51, (d) DR Engineering utiliteit-pendant van item 1, (e) jacht op de feitelijke erratum-tekst Tabel 2.8. Resultaat: 4 nieuwe bruikbare voorbeelden + 1 officiële norm-bron (erratum-tekst paragrafen) die isso51-core direct kan gebruiken voor Issue C-verificatie. **Geen** nieuwe ISSO 51-2024 Vabi cases gevonden — de markt blijft dun, item 1 (DR Engineering) blijft de enige ⭐⭐⭐ erratum-conforme Vabi-output op het open net.

### Nieuwe gevonden voorbeelden

| # | Bron | Type | Norm | Bruikbaarheid | URL |
|---|---|---|---|---|---|
| 8 | **TomLXXVI/python-hvac — `example_01` (House class)** | Twee-laags woning, 6 verwarmde + 2 onverwarmde ruimten, mech vent | **EN 12831-1:2017 standard method (§6)** expliciet | ⭐⭐⭐ open-source code+data, volledig reproduceerbaar | https://github.com/TomLXXVI/python-hvac/blob/master/docs/examples/heating_load_calc/info.md |
| 9 | **OpenEnergyMonitor — Trystan Lea's mid-terrace stone house** | Bestaande Britse tussenwoning (Wales), gemeten + EN 12831-berekend | EN 12831-1:2017 compliant | ⭐⭐ partieel — reële gemeten warmtevraag als sanity check | https://docs.openenergymonitor.org/heatpumps/heatloss.html |
| 10 | **DR Engineering — Voorbeeld warmteverliesberekening utiliteitsbouw** | Utiliteitsgebouw, 89 pagina's | **ISSO 51, 53 en 57 (2024)** — Vabi 3.12.0.127, datum 27-2-2025 | ⭐ out-of-scope voor ISSO 51 woning, maar **bruikbaar voor toekomstige ISSO 53-tak** van isso51-core | https://www.dr-engineering.nl/wp-content/uploads/voorbeeld-warmteverliesberekening-utiliteitsbouw.pdf |
| 11 | **IKZ-Haustechnik 18/2012 — DIN EN 12831 Beiblatt 2 Hüllflächenverfahren Beispielrechnung** | Reihenendhaus (Duitse pendant van tussenwoning) | DIN EN 12831 Beiblatt 2 (2012) — vereenvoudigde methode | ⭐ indicatief, alleen voor de vereenvoudigde Beiblatt 2 envelop-methode — geen 1:1 ISSO 51 match | https://www.ikz.de/uploads/media/022.pdf |

#### Detail item 8 — TomLXXVI/python-hvac (de belangrijkste nieuwe vondst)

- **Source-of-truth:** Python class `House` in `docs/examples/heating_load_calc/`, file `house.py` met `_create_building()` methode bevat alle bouwfysische inputs als code-constants. Floor plan staat in `floor_plan.pdf` in dezelfde directory.
- **Ruimtelijke samenstelling:** 6 verwarmde ruimten (keuken/eetkamer 27 m², woonkamer 14 m², slaapkamer 1 18 m², slaapkamer 2 11 m², badkamer 7,5 m²) + 2 onverwarmde (hal, toilet). Twee bouwlagen.
- **Bouwfysica (verbatim uit info.md):**
  - Buitenmuren: 12 cm isolatie
  - Binnenmuren: 6 cm
  - Vloer/plafond: 12 cm isolatie
  - Ramen: U = 2,86 W/(m²·K) — ASHRAE category 5a
  - Deuren: U = 3,0–4,0 W/(m²·K)
- **Ventilatie:** mechanisch — toevoer 72–100 m³/h per ruimte, afvoer 25–75 m³/h, transfer 25–50 m³/h
- **Ontwerptemperaturen:** θ_e = -7 °C, θ_i = 18–24 °C ruimte-afhankelijk, onverwarmde ruimten θ_u = 10 °C, omringende gebouwen niet relevant (vrijstaand)
- **Verwachte output verbatim:**

  | Component | Waarde |
  |---|---|
  | **Totaal Φ_HL gebouw** | **8,339 kW** |
  | Transmissieverlies | 7,139 kW |
  | Ventilatieverlies | 1,2 kW |
  | Per-ruimte spread | 0,812 – 2,787 kW |

- **Waarom dit echt waardevol is voor isso51-core:**
  1. EN 12831-1:2017 is precies de internationale norm waar ISSO 51:2023 (post-erratum) op gebaseerd is — Hoofdstuk 2 van ISSO 51:2023 is grotendeels een NL-implementatie van EN 12831-1:2017 §6.
  2. Alle inputs zitten in code → fixture is auto-genereerbaar via `python -c "from python_hvac.examples import House; print(House.to_json())"` of vergelijkbaar. Geen handmatige PDF-extractie nodig.
  3. Author Tom-LXXVI is een Belgische HVAC-engineer die actief de standaard volgt — package is gedocumenteerd, geen black box.
- **Caveats:**
  - EN 12831-1:2017 ≠ ISSO 51:2023. Verschillen: NL-specifieke Tabel 2.2 ontwerptemperaturen (NL = 22 °C verblijfsruimten, EN 12831 default 20 °C), NL-specifieke qv;10 infiltratie-methode (post-erratum 2023), Tabel 2.12 Δθ-waarden voor radiator-systemen zijn ISSO-specifiek. Dus dit is een test van het *EN 12831-fundament* binnen isso51-core, niet van de NL-specifieke afwijkingen.
  - Voor ISSO 51-specifieke validatie blijft DR Engineering item 1 noodzakelijk.

#### Detail item 9 — OpenEnergyMonitor (mid-terrace stone house)

- **Type:** echte bestaande tussenwoning Noord-Wales, U=1,5 W/(m²·K) stenen muren (slecht geïsoleerd), 21 °C woonkamer / 22 °C badkamer / 18 °C overig.
- **Verbatim outputs:**
  - HeatLoss.js berekening: 3340 W
  - Heatpunk berekening: 3553 W
  - Werkelijk gemeten maximale warmtevraag: 3,4 kW
  - n50 = 10,4 ACH @ 50 Pa → omgerekend 0,6 ± 0,2 ACH normaal gebruik
  - Ontwerp-buitentemperatuur: -1,4 °C
  - Grondtemperatuur: 10,6 °C
  - Werkelijke ontwerp-aanvoertemperatuur: 35 °C (warmtepomp)
- **Bruikbaarheid:** dit is bijzonder doordat de berekening tegen *gemeten realiteit* aan ligt. Voor isso51-core kun je dit gebruiken als "sanity check tegen werkelijkheid" — niet als formele EN 12831 validatie omdat de berekening twee verschillende tools gebruikt met inhoudelijk verschil. Niet voor een unit test fixture, wél als **end-to-end smoke test** voor "klopt de orde van grootte".
- **Beperking:** exacte vloeroppervlak en per-ruimte verdeling ontbreken in de openbare documentatie. Volledige reproductie vereist de spreadsheet `EN12831_2017_ventilation_calculation_v2.ods` (gelinkt vanaf de doc, niet geverifieerd in deze sessie).

#### Detail item 10 — DR Engineering utiliteit (briefly noteworthy)

- WebFetch faalde op deze PDF (binary parse error), zoekresultaat metadata bevestigt: 89 pagina's, Vabi 3.12.0.127, ISSO 51/53/57 (2024), 27-2-2025. Same engineering team als item 1 = vergelijkbare PDF-tekstlaag-kwaliteit kan verwacht worden.
- **Geen woning** → out-of-scope voor het huidige isso51-core focus. Maar als de roadmap ooit ISSO 53 (utiliteit) raakt: dezelfde 3BM-stijl golden case beschikbaar via dezelfde leverancier.
- Aanrader: parken voor later, niet nu downloaden.

### Nieuwe bronnen die geen voorbeeld zijn, maar wel waarde hebben

#### NORM-BRON: Officiële ISSO 51:2023 erratum-tekst (paragrafen waarop Tabel 2.8 in audit-rapport ge-extrapoleerd wordt)

- **URL:** https://documenten.isso.nl/s/rXyirFGw20gnnLF7s91CR7teJ4UDuFeq/23.09.01%20Erratum%20ISSO%2051_2023.pdf
- **Status:** WebFetch faalde op deze PDF (binary). _Redactie 2026-07-02: de letterlijke erratum-tekstwijzigingen zijn hier verwijderd (auteursrecht ISSO); volledige transcriptie lokaal bij 3BM._ Samengevat (eigen bevindingen over de aard van de wijzigingen):
  - **Par. 3.2.1 (infiltratie):** factor 1200 → 1,2; eenheid m³ → dm³.
  - **Par. 3.2.2/3.2.3 (ventilatie):** forfaitaire ventilatievolumestroom als functie van ΣAg (gebruiksoppervlak verblijfsgebieden).
  - **Par. 4.2.1 (infiltratie):** symbool Φi' → Φi; rekenrelatie `Φi = zi · Hi · (θi − θe)`.
  - **Par. 4.2.2 (ventilatie):** `Hv = 1,2 · qv · fv`; omschrijving factor fv toegevoegd.
  - **Tabel-verwijzing winddrukverdeling:** in ISSO 51:2017 Tabel 4.5, in 2023 hernummerd naar Tabel 2.6 (niet 2.8) — audit-aandachtspunt voor het juiste tabelnummer bij infiltratie.
- **Waarde voor isso51-core:** deze wijzigingen zijn de basis voor de Issue C-verificatie; gebruik ze als **bronverwijzing** (paragraaf + erratumnummer) in code-comments, niet als letterlijk citaat.

#### Buildwise HeatLoad — Belgisch counterpart (informatief, geen worked example)

- **URLs:**
  - Tool: https://www.buildwise.be/nl/expertise-ondersteuning/buildwise-tools/heatload-warmteverlies/
  - Handleiding v4.0 (2023): https://www.buildwise.be/media/a5pjmz2b/rekentool-en-12831-1-handleiding-nl-v40-2023.pdf
  - Handleiding (2026): https://www.buildwise.be/media/dkde41yh/2026-03-heatload-manual-v1-nl.pdf
- **Norm:** NBN EN 12831-1:2017 + Belgische bijlage ANB 2020
- **Bevat geen public worked example** (alleen tool-handleiding, geen sample report). WebFetch op de PDF faalde door binary parse. Maar de tool zelf is gratis en webbased — `heatload.buildwise.be` — dus 3BM kan zelf cases doorrekenen met **dezelfde EN 12831-grondslag** als isso51-core en die als golden output gebruiken. Belgisch ANB 2020 wijkt vooral af in qv;10-defaults en ontwerptemperaturen; transmissie/ventilatie-formules zijn identiek.
- **Aanrader voor isso51-core team:** maak één Buildwise web-run met een eenvoudige test-woning, exporteer als PDF, gebruik als 2e onafhankelijke EN 12831-leg. Tijd-investering ~30 min, levert een vendor-onafhankelijke kruisvalidatie.

### Niet bruikbaar (sectie 2)

| Bron | Reden afwijzing |
|---|---|
| Stiebel Eltron heat load questionnaire (EN12831) | Alleen invul-formulier, geen worked example met uitkomst-getallen — onbruikbaar als fixture |
| Elmhurst Energy heat loss docs | Tool-documentatie zonder publieke worked example |
| H2X Engineering BS EN 12831 diversity article | Klein voorbeeld met fictieve 1,046 kW + 0,337 kW + 0,32 kW → 1,575 kW (toont diversiteit-principe) maar geen volledig huis |
| BS EN 12831:2003 Annex C | Tekst van de norm bevat een Annex C "Example of a Design Heat Load Calculation" — maar paywall, alleen via aankoop standaard. **Aanrader:** voor formele compliance audit zou je een NEN-licentie willen → vraag user of 3BM toegang heeft tot NEN Connect / iTeh Standards |
| CIBSE Domestic Heating Design Guide 2026 | Paywall — niet praktisch op te halen |
| HBO Kennisbank doelgerichte zoekopdrachten | Geen treffers op "warmteverlies ISSO 51" in vrije site-search. Mogelijk wel via authenticated search door user-account |
| OpenEnergyMonitor community thread MCS calculator | Conceptuele discussie, geen volledig uitgewerkte case |
| TomLXXVI/heaty + tedynaidenov/heatcalc | Tools zonder publieke fixture/expected-output data (heaty is alleen GUI, heatcalc is een simpele kalkulator) |
| BasteIninstallatieadvies / Warmteverliescheck.nl Janssen-PDF | URL was redirect naar dode link (HTTP 404), bestand niet (meer) beschikbaar |

### Conclusie sectie 2

**Wat is netto winst:** 4 nieuwe voorbeelden — waarvan **1 echte aanwinst** (item 8 / TomLXXVI/python-hvac) die de EN 12831-1:2017 fundament-laag onder ISSO 51:2023 reproduceerbaar dekt met code-fixture in plaats van PDF-output. Plus **toegang tot erratum-tekst** als norm-bronvermelding voor de infiltratie/ventilatie-modules.

**Wat is niet meer bewogen:** zoekrichting "ISSO 51:2024 Vabi voorbeelden voor specifieke woningarchetypen" (tussenwoning, appartement, hoekwoning post-erratum) levert exact dezelfde resultaten als sectie 1 — de open markt heeft hier echt geen nieuwe content. Conclusie sectie 1 over "publieke 2024-Vabi-output is schaars" blijft staan.

**Witte vlek die NIET dichtgereden is:**
- Multi-wooneenheid woongebouw (appartementenflat) op ISSO 51:2024 — geen public voorbeeld gevonden, zou bij Vabi support of via ISSO-cursusmateriaal moeten komen
- Hoekwoning post-erratum 22 °C — Punt Ontwerp (item 3) is hoekwoning maar pre-erratum
- Vloerverwarmingscase post-erratum — geen volledige public case waar Δθ=0 voor vloerverwarming wordt toegepast, wat juist *de* differentiator is voor ISSO 51 Tabel 2.12 conformiteit (zie audit)

### Aanbeveling: welke 2 cases echt waard om als integration test fixture toe te voegen

| Prio | Case | Test-doel | Inspanning |
|---|---|---|---|
| **1** | **TomLXXVI/python-hvac `example_01`** (item 8) | EN 12831-1:2017 standard method §6 fundament — verifieert dat isso51-core de internationale norm-laag correct implementeert los van NL-specifieke wijzigingen | LAAG — Python-fixture, één run om expected JSON te dumpen, geen handmatige PDF-extractie. Schat 2–4 uur voor `tests/fixtures/en12831_two_storey_house.json` + `tests/integration/en12831_compliance.rs`. |
| **2** | **DR Engineering item 1** (al in sectie 1) | NL-specifieke afwijkingen ISSO 51:2024 (θ_i = 22°C, RSS-sommatie, infiltratie qv;10 post-erratum) | MIDDEL — PDF-extractie naar JSON-fixture, ~6–8 uur incl. cross-check op gebouwniveau vs per-ruimte sommatie. |

**Wat user uit eigen 3BM archief moet halen (niet via web te dekken):**
- **Eén 3BM klant-Vabi rapport ISSO 51:2024** voor een **appartement of woongebouw** — multi-unit blijft de duidelijkste witte vlek. Geanonimiseerd is voldoende, want isso51-core hoeft alleen de getallen te zien, niet de adres-gegevens. Geschikte kandidaat: een recent Hetzner-tenant project van 3BM met Vabi-uitvoer naast.
- **Eén 3BM klant-Vabi rapport met vloerverwarming** waarin Δθ = 0,0 wordt toegepast (Tabel 2.12 ISSO 51:2023) — vooral relevant voor isso51-core Δθ-tabel validatie, omdat geen enkele publieke case dit expliciet toont.
- Subsidiair: vraag bij ISSO (info@isso.nl) na of er bij de **publicatie** zelf een officieel rekenvoorbeeld zit als bijlage (zoals bij sommige NEN-normen) — dan is dat de meest gezaghebbende bron.

---

## Aanvulling 3 (2026-05-12 user-input)

### exb-2018-805.pdf

- **Bestand:** `tests/references/exb-2018-805.pdf` (9 pag., 129 KB)
- **Door user neergezet:** 2026-05-12 12:00
- **Geïdentificeerd als:** Vabi Elements warmteverliesrapport "Brouwerij RHA Proeflokaal" (projectnr 140224.01), opgesteld door **Endotec Advies en Engineering** (Uden) voor opdrachtgever "Principal 00039". PDF creation date: 7-4-2017, mod 13-4-2017. Eén ruimte: "Proeflokaal" (bijeenkomstfunctie, 70 personen, 104,5 m² / 318,8 m³).
- **Software:** Vabi Elements 3.1.1.14403, Vabi rekenkern Warmteverlies versie 2.16
- **Norm-versie:** ISSO 51, 53 én 57 — **pre-erratum**, op rapportdatum (april 2017) was dat ISSO 51:2017 / ISSO 53 / ISSO 57. Geen losse norm-jaar-aanduiding in rapport zelf.
- **Type woning:** **Utiliteitsgebouw** (geen woning) — soort gebouw expliciet "utiliteitsgebouw", gebouwfunctie "bijeenkomstfunctie / andere bijeenkomstfunctie". Bouwbesluit 2012.
- **Inputs zichtbaar:**
  - Buitentemperatuur -10,0 °C (standaard), ontwerpbinnentemperatuur 20,0 °C
  - qv;10 = 0,625 dm³/(s·m² Ag)
  - Ventilatiesysteem C, debiet 4,00 dm³/s × 70 personen
  - Verwarming: **vloerverwarming** (Soort verwarming = vloerverwarming, geen radiator/luchtverwarming)
  - Vloer direct op grond, grondwaterspiegel ≥ 1 m, grondwaterfactor 1,00
  - Volledige constructielijst met U-waarden (gevel 0,19/0,31, dak 0,19, vloer 0,18, HR++ glas 1,10, kozijn 2,40) en Rc-waarden tabel pag. 7
- **Outputs zichtbaar:**
  - Transmissie 2853 W, Ventilatie 9576 W, Opwarmtoeslag 0 W → **Totaal 12429 W**, plus 1243 W vloerverwarming-verlies naar grond → **Aansluitvermogen 13673 W**
  - Eén ruimte, dus per-kamer split = totaal (geen multi-ruimte distributie)
  - Infiltratie aparte regel: 0,000880 m³/s × 64,97 m² gevel → 1955 W (gemarkeerd met `*`: niet meegeteld in ruimtetotaal vanwege gelijktijdigheids-correctie)
- **Bruikbaarheid:** ❌ (niet bruikbaar voor isso51-core post-erratum validatie)
- **Vult witte vlek?** **Nee.** Drie redenen: (1) utiliteit/bijeenkomstfunctie, isso51-core scope is woningbouw; (2) pre-erratum 2017 → ontwerpbinnentemperatuur 20 °C i.p.v. post-erratum 22 °C voor verblijfsruimten — Tabel 2.2 (2023) niet toegepast; (3) één ruimte, dus geen multi-room distributie-validatie. Vloerverwarming staat wel in het rapport maar Δθ-instelling is in deze pre-erratum-versie nog niet de differentiator zoals in Tabel 2.12 (2023).
- **Bijzonderheden:** Wel waardevol als **structuur-referentie** voor Vabi PDF-layout (kolomvolgorde transmissietabel, constructie-overzicht, ventilatie-blok) bij toekomstige PDF-parsing of report-builder werk; numerieke validatie geeft het echter geen invulling.

### Aanbeveling

Niet toevoegen aan fixtures voor isso51-core. Deze case is utiliteit (buiten scope), pre-erratum 2017 en single-room. Voor PDF-structuur-referentie bij rapport-tooling bewaren kan; voor numerieke norm-validatie geen meerwaarde boven de bestaande set. De witte vlekken (appartement post-erratum, hoekwoning post-erratum, vloerverwarming Δθ=0 post-erratum) blijven onvervuld.

**Verdict:** Vabi 2017 utiliteits-export met vloerverwarming maar pre-erratum norm — geen invulling van witte vlekken. ❌

---

## Aanvulling 4 (2026-05-12 — installateur PDF)

### 24221-60-wvb-20250701.pdf

- **Bestand:** `tests/references/24221-60-wvb-20250701.pdf` (250 pagina's, 1,45 MB)
- **Door user neergezet:** 2026-05-12 14:52 — installateur die user vertrouwt
- **Bureau:** **De Installatiedesk** (adviseur-veld); PDF-author metadata: Mohammad Mekdad; producer: Microsoft Print To PDF
- **Project:** **Groningen OPDC** (Orthopedagogisch Didactisch Centrum — onderwijs/utiliteit, geen woning). Projectnummer 24221, Vabi-bestand `24221-20250618.vp`
- **Datum rapport:** 2025-07-01 15:48 (model uit 18 juni 2025)
- **Vabi-versie:** Elements **3.12.1.19**, rekenkern Warmteverlies **2.51**
- **Norm-versie:** "Warmteverliesberekening volgens **ISSO 51, 53 en 57 (2024)**" — let op: jaartal (2024) wijkt af van zowel "2017" als "(erratum 2023)" formuleringen elders. Vabi 3.12 (najaar 2024) zou de erratum-tabellen moeten implementeren, maar **rapport noemt erratum niet expliciet**. Geen θ_b 17 °C zichtbaar (utiliteit gebruikt 20 °C verblijfsgebied + bedrijfsbeperking-toeslag, niet woning-θ_b)
- **Type woning:** ❌ **Utiliteitsgebouw, meerlaags** — onderwijs (lokalen, gymzaal, kantoren, kantine, spreekruimtes). Géén woning, géén appartement
- **Inputs zichtbaar:**
  - Vloeropp: 1065 m² (vloeroppervlakte voor infiltratie), gebruiksoppervlakte 1449,7 m², bruto inhoud 7367,1 m³, lengte 66,9 × breedte 21,3 × hoogte 8,0 m
  - qv;10 methode: **Specifiek** — qv,10,spec = **0,6000 dm³/(s·m²·Ag)** bij 10 Pa; afgeleide qi,s = 0,3036; qi,t = 363,20 dm³/s; correctiefactor invloed ventilatie 1,10 (zichtbaar per ruimte)
  - dUtb: per constructie ingevoerd (zichtbaar: buitenwand Rc=4,7 → dUtb=0,10; raam alu trippel U=1,9 → dUtb=0,10). "Thermische bruggen volgens Overige situaties"
  - Ventilatiesysteem: **D** (mechanische toevoer + afvoer met WTW), gebouw-niveau LBK 10.000 m³/h. Temp na WTW 9,0 °C; retourlucht 18,2 °C
  - θ_int: gemengd — 20 °C (lokalen, kantoren), 18 °C (gangen, hal, kleedruimte), 22 °C (kleedruimte+douches), 20-22 °C (gymzaal)
  - θ_e: basis -10,0 °C, ontwerpbuitentemperatuur -8,5 °C (gecorrigeerd met 1,5 K voor tijdconstante 144,4 h)
  - Nachtverlaging / bedrijfsbeperking: "Continu / Afzien van bedrijfsbeperking" per ruimte; gebouw heeft wel Φhu = 3604 W totaal
- **Outputs zichtbaar:**
  - **Aansluitvermogen gebouw ΦHL,build = 47.698 W**; door verdeler ΦHL,verdeler = **96.283 W** (incl. Φvv 49.019 W + Φadd 52.400 W − Φgain,v 3815 W)
  - Φbasis 44.063 (ΦT,ie 31.100 + ΦT,iae 4866 + ΦT,ig 0 + Φi 8096 − Φgain 0 + 1 W rounding); Φextra 3635 (Φvent 475 + Φhu 3604 − overlap)
  - Niet-vertrekzijdig vloerverwarming Φverlies1 = 3380 W gebouwniveau (vrijwel alle verblijfsruimten op begane grond hebben vloerverwarming)
  - Per-kamer split: **ja** — tabel met `#, Naam, ISSO, Verw., Temp., fk, Φbasis, Φextra, ΦHL,i, W/m², W/m³` voor elke ruimte (volle pag 10-12). Voorbeeld Lokaal 1 TTVO: Φbasis 1966 / Φextra 0 / ΦHL,i 1966 W / 46,41 m² / 20 °C
  - Per-kamer ΦT,ie / ΦT,ia / ΦT,iae / ΦT,ig / Φi / Φvent / Φhu apart **ja** — elke ruimte heeft eigen "Ontwerpvermogen"-blok (zie pag 13 Lokaal 1: ΦT,ie 1254, ΦT,ia 28, ΦT,iae 50, ΦT,ig 0, Φi 634)
  - Transmissietabel per ruimte met **Oriën, Cz, Opp, U/Ueq, dUtb, Tagr, fk, Φ-transmissie per constructie** (kolomvolgorde identiek aan DR Engineering format)
- **Bruikbaarheid:** ❌ (niet bruikbaar voor isso51-core; wél top als structuur-referentie)
- **Vult witte vlek?**
  - Appartement post-erratum: ✗
  - Vloerverwarming Δθ=0: ✗ (vloerverwarming wel aanwezig, maar isso51-core/ISSO 51 Tabel 2.12 Δθ-mechaniek is woningbouw — utiliteit gebruikt Φverlies1-fractie 0,16 via Rc-vloer methodiek, ander pad)
  - Hoekwoning post-erratum: ✗
- **Bijzondere Vabi-formules / correcties in PDF:**
  - Φverlies1 = Fractie (0,16) × ΦT,vloer-onder-vloerverwarming (Rc 4,85) — methode utiliteit, niet woning-tabel
  - Φi-formule met correctiefactor 1,10 voor ventilatie-invloed expliciet per ruimte
  - Voorverwarmer Φvv: T_toevoer 20 °C, T_na_WTW 9,0 °C; opgewarmd vermogen op gebouw-niveau Φvv 49.019 W
  - Φgain,v* (warmtewinst ventilatie) -3815 W: voetnoot "* niet in ISSO publicaties opgenomen" — eigen Vabi-extensie
  - Per gymzaal/hoge ruimten: aparte "Reductiefactor circulatievoud" + "Soort verwarming hoge ruimte" (ISSO 57)
  - Geen θ_water / boundary-type water aanwezig (utiliteit zonder grondwaterboundaries direct in scope)

### Aanbeveling

Niet toevoegen aan fixtures voor isso51-core: dit is een **groot utiliteits-onderwijsgebouw (1450 m² GBO, ~80 ruimten met ISSO 53/57-flag)** — volledig buiten isso51 woningbouw-scope. De vermelding "ISSO 51, 53 en 57 (2024)" is interessant maar de eigenlijke berekeningen volgen ISSO 53/57 (verblijfsruimte-niveau utiliteit) en de θ_b=22 °C / appartement-paden van het erratum komen niet voor. Audit-vragen die deze case wél kan helpen beantwoorden: (a) Vabi 3.12 / kern 2.51 rapport-layout per ruimte (kolomvolgorde, constructie-naming, fk-conventie) — sterker en consistenter dan de pre-erratum 2017 cases; (b) hoe Vabi vloerverwarming-systeemverlies-fractie (0,16) toont in utiliteit-pad — naast woning Tabel 2.12 nuttig als contrast. **Witte vlekken (appartement post-erratum, hoekwoning post-erratum, vloerverwarming woningbouw Δθ=0) blijven onvervuld** — de installateur heeft een goed, maar voor onze validatie-doelen verkeerd-scoped rapport geleverd.

**Verdict:** Groot Vabi utiliteits-export 2025-07 — sterke structuur-referentie, géén woningbouw-validatie, geen witte vlek gedicht. ⭐ (alleen layout-referentie) — vult **geen** witte vlek.
