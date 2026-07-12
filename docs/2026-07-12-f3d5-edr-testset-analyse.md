# F3d-5 — EDR-attesteringstestset (ISSO 54, v2.0) — extractie-analyse

**Bron:** `ISSO 54 - 12-05-2022.pdf` — "Testen EP-woningen (BRL 9501 NTA8800)",
"Energie Prestatie Rekenprogramma's — Testen voor het deelgebied EDR attest
energieprestatie", versie 2.0, vastgesteld door het CCvD van InstallQ op
**12-05-2022** (vervangt v2.0 nov-2021). 68 pagina's, rekent volgens **NTA 8800
(januari 2022)**.

> **Licentie:** de PDF blijft buiten de repo. Fixtures bevatten uitsluitend
> afgeleide invoergegevens + bronverwijzing (pagina/figuur/tabel), analoog aan de
> norm-tabellen.

---

## 1. Kernbevinding (bepaalt de hele fixture-strategie)

De EDR-set is het **spiegelbeeld** van de RVO-voorbeeldconcepten
(`beng_rvo_voorbeeldconcepten`):

| Laag | RVO-set | EDR-set (deze) |
|---|---|---|
| **Invoer** (geometrie/installaties) | zit in niet-publieke Bijlage 4-Excel → geblokkeerd | **volledig + normatief in de PDF-tekst** |
| **Verwachte uitkomst** (getallen) | gepubliceerd (p13) | zit in **apart Excel-document "Bijlage 2"** → **niet in ons bezit** |

De PDF-tekst (p67, Bijlage 2) zegt letterlijk: *"In een apart Excel-document,
bijlage 2, zijn zowel de uitkomsten van de standaard EDR-testen en de uitkomsten
van de realistische woningbouw en utiliteitsbouw gegeven."* **Er staat geen
enkel resultaatgetal in de PDF zelf.**

**Gevolg voor de goldens:**
- De **invoer** is de sterkste die we hebben: deterministisch, normatief,
  geen geometrie-giswerk (anders dan RVO Bijlage 4). EPW001 is een canonieke
  ProjectV2-invoer.
- De **eindwaarden** (EP1/EP2/EP3/Q_H;nd/TOjuli/deelposten) zijn geblokkeerd tot
  het Bijlage 2-Excel er is. Anti-fudge is hard: geen enkele expected-waarde mag
  ontstaan uit wat onze engine uitrekent.
- **Wél assertbaar zónder het Excel:** `Ag` en `Als` staan expliciet in de
  EPW001-tekst (p5). Dat maakt een **geometrie-golden** mogelijk (a_g, a_ls,
  A_ls/A_g-ratio) die volledig los staat van de nog kapotte PV/energie-keten —
  de aangewezen eerste fase-2-activatie.

**Officiële afkeurtolerantie:** ±1,0% (p67). *"Afkeuring vindt plaats wanneer
het berekende resultaat meer dan 1,0% afwijkt."* Dit is een attesteringsset:
veel strakker dan de ±10% RVO-starttolerantie. Zodra het Excel er is, geldt ±1%
op de deelposten — een scherpe engine-vangrail.

---

## 2. Structuur van de testset

Hoofdstuk 2 (woningen) is opgebouwd als **één referentietest (EPW001) + variant-
tests die uitsluitend een delta t.o.v. EPW001 beschrijven**. Elke variant-tekst
zegt "In het gebouw van deeltest EP-W001 wordt X gewijzigd". Dat maakt elke
fixture = EPW001-basis + gedocumenteerde delta.

| Blok | §  | Testreeks | Onderwerp | # deeltests (indicatief) |
|---|---|---|---|---|
| Bouwkundig (EPW0) | 2.2 | EPW002–014 | isolatie, ramen, oriëntatie, massa, vloerbegrenzing, infiltratie, overstek, belemmering, zonwering, gebruiksoppervlak, dakvorm | ~60 |
| Ventilatie (EPW1) | 2.3 | EPW101–104 | ventilatiesysteem (A/B/C/D/E), voorverwarming, zomernachtventilatie, overig | ~35 |
| Ruimteverwarming (EPW2) | 2.4 | EPW201–206 | afgifte, distributie, opwekking (ketel/WP/biomassa/WKK/warmtelevering), gemeenschappelijk/woongebouw | ~50 |
| Koeling (EPW3) | 2.5 | EPW301–303 | afgifte, distributie, opwekking (compressie/absorptie/vrije koeling) | ~18 |
| Tapwater (EPW4) | 2.6 | EPW401–407 | afgifte, distributie, douche-WTW, voorraadvat, zonneboiler, opwekking, gemeenschappelijk | ~45 |
| Utiliteit (EPU) | 3   | EPU… | idem voor utiliteitsbouw | n.v.t. voor woningbouw-golden |
| Realistisch | 4 / bijlage 4 | EPWRealB/D01… | 11 ingevulde opnameformulieren (ook in aparte bijlagen) | 11 |

**PV / gebouwgebonden elektriciteitsproductie:** de referentie stelt expliciet
*"Er is geen gebouwgebonden productie van elektriciteit aanwezig"* (p6). Een
scan van álle §2-koppen (2.2–2.6) toont **geen aparte PV-/zonnestroom-testreeks**
in de woningbouw-EDR-set. Het dichtst bij "eigen elektriciteitsproductie" komen
de **micro-WKK-tests (EPW204f/g/h)** — die produceren elektriciteit maar zijn
warmte-gedreven, geen PV. **Conclusie: de EDR-set dekt de PV-keten niet; de
PV-golden blijft leunen op de Uniec-crosscheck (F3d-4).** Dit corrigeert de
aanname in de taakstelling dat er een PV-variant te extraheren zou zijn.

---

## 3. Dekkingsmatrix (test ↔ engine-keten)

Welke deeltest oefent welke reeds-gebouwde tak van `compute_beng`:

| Deeltest | Delta t.o.v. EPW001 | Engine-keten | Waarom gekozen |
|---|---|---|---|
| **EPW001** | — (referentie) | **geometrie** (A_g/A_ls), H_T, H_V, forfaits, HR107, D2+WTW | canoniek; A_g/A_ls direct assertbaar |
| **EPW002c** | detailberekening thermische bruggen (ψ·L expliciet) | koudebrug-propagatie (Σψ·L in H_D) — **F3d-4** | enige test met expliciete ψ + lengtes; raakt exact de F3d-4-fix |
| **EPW004d** | hoofdgevel → Noord (gebouw draait mee) | zonwinst per oriëntatie (Q_sol), TOjuli | zuivere oriëntatie-delta; isoleert de azimut-tak |
| **EPW101p** | ventilatie D2 → D1 (geen WTW) | ventilatie/WTW + infiltratie (H_V) | contrast met EPW001's WTW; test η_wtw-uitschakeling |
| **EPW203f** | HR107 → elektrische WP buitenlucht (COP conform) | opwekking verwarming, WP-tak — **F3d-4** | lucht-WP is de F3d-4-relevante generator |
| **EPW301a** | koelinstallatie toegevoegd (compressie, vloerkoeling) | koelvraag + koelopwekking (Q_C, EER) | EPW001 koelt niet; bekende koel-bug (+506% in Uniec) |

De 5 varianten dekken samen: transmissie+koudebruggen, zon/oriëntatie,
ventilatie/WTW, warmtepomp-opwekking, koeling. Niet gedekt (bewust buiten fase-1):
tapwater-varianten, thermische massa, vloerbegrenzing, woongebouw/gemeenschappelijk,
realistische opnames.

---

## 4. Extractie-betrouwbaarheid per onderdeel

| Onderdeel | Bron | Kwaliteit | Toelichting |
|---|---|---|---|
| Geometrie EPW001 | p4 fig.1 (pixmap) + p5 tekst | **hoog** | maten in tekst én figuur, intern consistent (zie §5) |
| Constructies/Rc/U EPW001 | p5 tabel 1 (tekstlaag) | **hoog** | tabel volledig in tekstlaag geëxtraheerd |
| Ramen EPW001 | p5 tekst + fig.1 | **hoog** | 4× 6 m², U=1,8, g=0,7, kozijnfractie 25% |
| Installaties EPW001 | p5–6 tekst | **hoog** | HR107 η=0,95, D2+WTW, qv10=0,7, ftype=1,4, Dm=450, perim=28 |
| Variant-delta's | §2.2–2.6 tekst | **hoog** | delta's staan als lopende tekst, goed leesbaar |
| **Eindwaarden (alle tests)** | Bijlage 2 **Excel** | **ONBESCHIKBAAR** | niet in PDF; hard geblokkeerd |
| A_g / A_ls EPW001 | p5 tekst | **hoog, assertbaar** | Ag=96; Ao=247,2 met vlak-breakdown → Als |

Figuur-render (pixmap, p4) bevestigde de tekstmaten exact: zuidgevel 8,0×5,4 m,
2 bouwlagen à 2,7 m, per laag 2 ramen van 3,0×2,0 m (0,5 m van de rand, 1,0 m
tussenruimte, 0,5 m boven, 0,2 m onder). Geen figuur-only maat die niet ook in de
tekst stond → geen pixmap-afhankelijke onzekerheid.

---

## 5. EPW001 — geëxtraheerde kentallen (met interne consistentie-check)

- Afmetingen (binnenmaats, NEN 1068): 8,0 × 6,0 × 5,4 m; constructiedikte = 0.
- Volume V = 259,2 m³ (129,6 per bouwlaag). ✔ 8·6·5,4 = 259,2
- A_g = 96 m² (48 + 48). Perimeter BG-vloer = 28 m.
- Omhullend A_o = 247,2 m²:
  - dak 48,0 + vloer 48,0
  - zuid **bruto** 43,2 (= 8,0·5,4); waarvan raam 24,0 → dicht 19,2 (tabel 1)
  - noord 43,2 (dicht, geen ramen)
  - oost = west = 32,4 (= 6,0·5,4) elk
  - Σ = 48+48+43,2+43,2+32,4+32,4 = **247,2** ✔ → **A_ls = 247,2 m²**
  - A_ls/A_g = 247,2 / 96 = **2,575**
- Ramen: 4 × 6,0 m² = 24,0 m² (12 per bouwlaag), zuid. HR++ Ugl=1,2, ggl;n=0,7,
  houten kozijn Ukozijn=2,4, kozijnfractie 25% → **U-raam = 1,8** (formule 8.15).
- Dichte constructies: alle Rc = 6,0 m²K/W; U_dak/U_gevel = 0,162; vloer op grond
  (hfdst. 8 NTA 8800). Overgangsweerstanden: dak 0,14 / gevel 0,17 / vloer 0,21.
- Thermische massa: Dm = 450 kJ/m²K (tabel 7.10, "geen/open plafond").
- Infiltratie: qv10;spec;reken = 0,7 dm³/(s·m²) (tabel 11.14), ftype = 1,4
  (vrijstaand), gebouwhoogte 5,4 m, geen open verbrandingstoestellen.
- Ventilatie: **D2** (gebalanceerd, mechanische toe-/afvoer + WTW), tegenstroom-
  wisselaar kunststof, toevoerkanaal 1,0 m geïsoleerd, LUKA C (flea;du=1,05),
  bypass fbypass=1,0, ventilatorvermogen forfaitair, unit in verwarmde zone.
- Ruimteverwarming: individuele modulerende **HR107-combiketel** η_H;gen=0,95,
  installatiejaar 2021, LT-systeem 45/40, overal vloerverwarming (deklaag 1,8 cm,
  EN 442/1264), θroomaut = −0,5 K, twee-pijps waterzijdig ingeregeld, geen
  aanvullende pomp, geen warmtemeter.
- Warmtapwater: zelfde HR107-combi, tapklasse **CW5**, geen voorraadvat,
  uittapleidingen keuken 8,5 m / douche 5 m (Ø>10 mm), geen circulatieleiding,
  geen douche-WTW.
- Koeling: **geen** actieve koeling, geen zomernachtventilatie.
- PV: **geen** gebouwgebonden elektriciteitsproductie.
- Bouwjaar 2021.

---

## 6. h5 "Eisen aan de uitvoer" — welke grootheden Bijlage 2 publiceert (p53)

Dit bepaalt hoe diagnostisch de goldens worden zodra het Excel er is. Minimaal te
publiceren uitkomsten (afgezet tegen onze `BengResult`-velden):

| # | Grootheid | Symbool | Eenheid | `BengResult`-mapping |
|---|---|---|---|---|
| 1 | Energiebehoefte-indicator | EP1 / E;we;H+C;nd | kWh/m² | `beng1.value` |
| 2 | Primaire fossiele energie-indicator | EP2 / E;we;PTot | kWh/m² | `beng2.value` |
| 3 | Aandeel hernieuwbare energie | EP3 / RER;PrenTot | % | `beng3.value` |
| 4 | Netto warmtebehoefte | QH,nd;net | kWh/m² | (deelresultaat, nu niet apart geëxposeerd) |
| 5 | Max. temperatuuroverschrijding | TOjuli;max | – | `tojuli` |
| 6 | Gebruiksoppervlakte | Ag | m² | `a_g_m2` **(assertbaar nu)** |
| 7 | Oppervlakte thermische schil | Als | m² | `a_ls_m2` **(assertbaar nu)** |
| 8 | Energie verwarming | EH;ci | kWh | `service_breakdown_kwh_m2.heating · Ag` |
| 9 | Energie bevochtiging | Ehum;ci | kWh | – (n.v.t. woning) |
| 10 | Elektrische energie ventilatie | EV;ci | kWh | `…ventilation_aux · Ag` |
| 11 | Elektrische energie verlichting | EL;ci | kWh | – (forfait woning) |
| 12 | Energie koeling | EC;ci | kWh | `…cooling · Ag` |
| 13 | Energie warm tapwater | EW;ci | kWh | `…dhw · Ag` |
| 14 | Totale hulpenergie | Waux;tot | kWh | – |
| 15 | Standaard voor woningisolatie | – | kWh/m² | – |

De deelposten (8–14) maken de EDR-set — als het Excel er is — **diagnostisch
sterker dan de RVO-set** (alleen eindindicatoren): per-dienst-afwijking op ±1%.

---

## 7. Harnas-status (fase 1)

- Nieuw testbestand `crates/openaec-project-shared/tests/edr_golden.rs`, patroon
  van `beng_golden.rs`. Raakt `beng_golden.rs` niet aan (parallel-agent-territorium).
- 6 fixtures onder `tests/verification/beng_edr_epw/` (epw001 + 5 varianten),
  elk `input.json` (ProjectV2-nabij tussenschema) + `expected.json` + `README.md`.
- **Actief (draait in `cargo test`):** provenance-vangnet — elke fixture heeft
  bron (document/pagina/figuur/tabel), elke geometrie-expected (Ag/Als) een
  paginaverwijzing, en elke Excel-geblokkeerde grootheid een expliciete
  `blocked_on`-marker (kan niet stilzwijgend een verzonnen getal bevatten).
- **Rood/`#[ignore]`:** de reken-asserts, met twee onafhankelijke blokkades:
  (a) Bijlage 2-Excel ontbreekt (eindwaarden), (b) F3d-4 PV/energie-keten.
- Uitzondering: de **geometrie-golden** (Ag/Als) is niet Excel-geblokkeerd en is
  de eerste fase-2-activatiekandidaat.

---

## 8. Aanbevolen fase-2-volgorde

1. **Geometrie-golden activeren (nu mogelijk, niet Excel-geblokkeerd):** bouw een
   `edr_to_projectv2(EPW001)` (analoog aan `oes_to_projectv2`) en assert
   `a_g_m2 ≈ 96` en `a_ls_m2 ≈ 247,2` binnen ±1%. Valideert de geometrie-pijplijn
   los van energie/PV.
2. **Bijlage 2-Excel verwerven** (via InstallQ/ISSO of de attesterende partij).
   Zonder dit blijven alle energie-eindwaarden geblokkeerd. Dit is de kritieke
   externe afhankelijkheid — vermeld als projectblokkade richting orchestrator.
3. **Na F3d-4-merge (PV/koudebrug-fix):** transmissie/verwarming activeren met de
   Excel-eindwaarden op EPW001, EPW002c, EPW203f (±1%).
4. **Oriëntatie/ventilatie/koeling** (EPW004d, EPW101p, EPW301a) als laatste; die
   raken de zon- en koel-takken waar de engine nu nog het verst afligt.
5. **Uitbreiden** met tapwater- en massavarianten zodra de kern groen is.
