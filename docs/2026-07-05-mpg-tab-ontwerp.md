# MPG-tab — Ontwerpdoc

| | |
|---|---|
| **Status** | Concept (2026-07-05) |
| **Doel** | Indicatieve MPG-berekening (Milieuprestatie Gebouwen) als ontwerptool in vroege projectfasen |
| **Scope** | Nieuwe calc `"mpg"` binnen het ADR-002 multi-calc model + Ribbon-tab **MPG** |
| **Uitgangspunt** | Bepalingsmethode Milieuprestatie Bouwwerken (A2 / EN 15804+A2, verplicht sinds Bbl-wijziging 1-7-2026) |
| **Nadrukkelijk NIET** | Gevalideerd rekeninstrument voor Bbl-toetsing — zie [Positionering](#positionering) |

---

## Positionering

De tab is een **ontwerptool**: snel inzicht in de milieuprestatie tijdens schets/VO/DO, met variantenvergelijking en hotspot-analyse. Geen certificeerbare eindberekening.

- Datalaag: **NMD Cat. 3 Viewer-API (bèta)** — generieke, merkloze milieudata van Stichting NMD, inclusief de 30% ophoogfactor. Gratis, toegang op aanvraag.
- De API-voorwaarden verbieden gebruik voor gevalideerde Bbl-berekeningen → elke output (UI + PDF) draagt de disclaimer *"Indicatieve berekening op basis van generieke categorie 3-data — geen gevalideerde MPG conform Bbl"*, plus bronvermelding NMD (voorwaarde bèta-API).
- De 30% ophoogfactor op cat. 3 data maakt de uitkomst **conservatief** — dat is voor een ontwerptool een feature: de gevalideerde berekening valt vrijwel altijd gunstiger uit.
- Validatietraject bij Stichting NMD (toetsingsprotocol, betaald) is een expliciete *latere* optie; niets in dit ontwerp mag die route blokkeren.

---

## Fasemodel — detail groeit mee met het ontwerp

Kernprincipe: de gebruiker kiest per project (of per element) het invoerniveau. Grover niveau = bredere onzekerheidsband, nooit een blokkade.

| Fase | Invoerniveau | Databron | Onzekerheid |
|---|---|---|---|
| **Schets** | m² GO + gebouwtype → kengetal-MPG | Eigen benchmarkset (referentieprojecten, hardcoded starter-set) | Breed (±30–40%) |
| **VO** | Elementen (gevel/dak/vloer/kozijnen) met opbouw-presets | Cat. 3 profielen geaggregeerd per elementpreset | Middel (±15–25%) |
| **DO** | Per constructielaag materiaal + dikte (= bestaande Rc-lagen) | Cat. 3 API-profielen per productkaart | Smal (±10%, ophoogfactor domineert) |

De niveaus zijn mengbaar: dak op DO-niveau terwijl installaties nog op kengetal staan. De totaalscore toont dan de gecombineerde bandbreedte.

---

## Synergie met bestaand model

| Bestaand | Hergebruik |
|---|---|
| `SharedGeometry` — `Space[]` + `Construction[]` met `area_m2`, `openings` | Hoeveelheden (m² per constructie) gratis — grootste invoerlast van losse MPG-tools vervalt |
| `ConstructionLayer { material, thickness_mm, lambda }` | `thickness_mm × area_m2` = volume per materiaal → koppeling aan milieuprofiel |
| Constructie-bibliotheek (`Library`) | Uitbreiden met milieuprofiel-koppeling per opbouw → wordt de VO-presetlaag |
| Rc-calculator (ISO 6946) | Zelfde lagen-editor krijgt een milieukolom: één invoer stuurt Rc én MPG — de kern-USP |
| ADR-002 `calcs`-map | `"mpg" → MpgInputs`, view-mapper `mpg-core::from_shared(...)` zoals isso51/tojuli |
| PDF-generator | Extra rapportsectie "Milieuprestatie (indicatief)" |

**Wat geometrie níét dekt** (wel MPG-plichtig): fundering, installaties (verwarming/ventilatie/PV), trappen, afwerkingen, terreininrichting. Deze komen als **forfaitaire posten** in `MpgInputs` — op schets/VO-niveau kengetallen, op DO-niveau handmatige productregels.

---

## Berekening (Bepalingsmethode, A2)

```
per productregel:
  hoeveelheid (uit geometrie of handmatig)
  × milieuprofiel per eenheid (19 indicatoren, A2)
  × (1 + vervangingen)          vervangingen = ceil(L_gebouw / L_product) − 1
  → per indicator sommeren over alle regels
  → weegfactoren set-A2 (€/eenheid per indicator) → MKI (€)

MPG = MKI_totaal / (GO_m2 × L_gebouw)      [€/m²·jr]
```

- `L_gebouw`: 75 jaar woningbouw, 50 jaar utiliteit (Bepalingsmethode-default, overschrijfbaar).
- Levenscyclusmodules A1-A3, A4-A5, B, C en D conform productkaart; module D apart gerapporteerd.
- Grenswaarde als referentielijn in de UI (woningen/utiliteit). **Let op:** met de Bbl-wijziging per 1-7-2026 zijn de grenswaarden op A2-basis herijkt — actuele waarden verifiëren bij implementatie en als data (niet hardcoded in de rekenkern) opnemen.

---

## Architectuur

### Crate: `crates/mpg-core`

Zelfde discipline als `isso51-core`: puur Rust, geen I/O, geen async, JSON in/uit, schemas via schemars, doc comments verwijzen naar Bepalingsmethode-paragrafen.

```
mpg-core/
├── src/model/        MpgInputs, ProductRegel, Milieuprofiel (19 indicatoren), ElementPreset
├── src/calc/         hoeveelheden-extractie, vervangingscycli, MKI-weging, bandbreedte
├── src/tables/       weegfactoren set-A2, kengetallen benchmarkset, forfaitaire posten
└── src/lib.rs        calculate_from_json()
```

`Milieuprofiel` is een **snapshot in het projectbestand** (`.ifcenergy`): het project blijft reproduceerbaar rekenen zonder API of cache, en een NMD-data-update verandert nooit stilletjes een opgeslagen resultaat. Elke snapshot draagt `nmd_versie` + `opgehaald_op`.

### Datalaag: Cat. 3 cache (host, niet core)

```
Cat. 3 Viewer-API ──(sync-knop / eerste gebruik)──► mpg-nmd.db (SQLite, naast isso51.db)
                                                        │
frontend zoekt/koppelt profielen ◄──────────────────────┘
gekozen profiel → snapshot in project-JSON → mpg-core rekent offline
```

- API-client in `src-tauri` (zoals bestaande host-IO), niet in de core.
- Cache versie-gepind; UI toont NMD-dataversie + laatste sync.
- Zolang API-toegang loopt: starter-set van ±40 veelgebruikte cat. 3 profielen handmatig overgenomen uit de NMD Viewer als seed, zodat de bouw niet op de aanvraag wacht.

### `MpgInputs` (in `calcs["mpg"]`, per ADR-002)

```
MpgInputs
├── niveau per element:  Kengetal | Preset | Lagen
├── materiaal_koppeling: Map<material-string, ProfielRef>     // laag → milieuprofiel
├── element_presets:     Map<construction-id, PresetRef>      // VO-niveau override
├── forfaitair:          Vec<ProductRegel>                    // fundering, installaties, …
├── varianten:           Vec<Variant>                         // named overrides op koppelingen
├── go_m2 / l_gebouw:    defaults uit shared, overschrijfbaar
└── profiel_snapshots:   Vec<Milieuprofiel>                   // embedded NMD-data
```

De `material`-string in `ConstructionLayer` blijft ongewijzigd (frozen interface); de koppeling leeft volledig in `MpgInputs` — geen migratie van bestaande projecten nodig.

---

## UI — Ribbon-tab **MPG**

Groepen conform de geplande OpenAEC ribbon-migratie:

| Groep | Inhoud |
|---|---|
| **Score** | MPG-meter t.o.v. grenswaarde (live), MKI-totaal, bandbreedte-indicator |
| **Koppeling** | Materiaal→profiel matrix (alle unieke lagen-materialen uit het project, met zoek-popup naar cache), forfaitaire posten |
| **Varianten** | Variant aanmaken/dupliceren, vergelijkingsweergave (2–3 naast elkaar: ΔMPG én ΔRc/warmteverlies in één beeld) |
| **Analyse** | Hotspot top-5 (bijdrage per element/materiaal), module-uitsplitsing A/B/C/D |
| **Data** | NMD-sync, dataversie, niveau-schakelaar per element |

UX-principes:
1. **Nooit blokkeren** — ongekoppelde materialen krijgen automatisch het kengetal-fallback met brede band, en staan in een "nog te koppelen"-lijst.
2. **Bandbreedte i.p.v. schijnprecisie** — score als `0,62 ± 0,11`, geen 3 decimalen.
3. **Live herberekenen** — zelfde patroon als warmteverlies: elke wijziging in lagen/oppervlaktes werkt direct door.
4. Variantenvergelijking combineert MPG met de warmteverlies-uitkomst — de combinatie thermisch + milieu in één klik is wat geen bestaand instrument biedt.

---

## Rapport

Extra PDF-sectie (aan/uit-schakelbaar zoals bestaande secties): score + meter, hotspots, productregeltabel, NMD-dataversie, disclaimer + bronvermelding. Bandbreedte expliciet in de samenvatting.

---

## Stappenplan

| # | Stap | Afhankelijkheid |
|---|---|---|
| 1 | Toegang Cat. 3 Viewer-API aanvragen (parallel starten) | — |
| 2 | `mpg-core` scaffold: model + schemas + weegfactoren set-A2 + kengetallen-starter | — |
| 3 | Rekenkern + fixtures: één gepubliceerde referentie-MPG-berekening nabouwen (zelfde aanpak als ISSO 51/Vabi-fixtures) | 2 |
| 4 | Starter-set cat. 3 profielen (handmatig uit NMD Viewer) als seed-db | — |
| 5 | `MpgInputs` in `calcs`-map + view-mapper + Zustand-slice | 2 |
| 6 | UI: koppelingsmatrix + score-meter + hotspots | 3, 5 |
| 7 | API-client + SQLite-cache + sync (vervangt seed) | 1 |
| 8 | Varianten + rapportsectie | 6 |

Stap 2–6 zijn niet geblokkeerd door de API-aanvraag — dat is bewust.

## Open vragen

- [ ] Actuele A2-grenswaarden per gebouwfunctie na Bbl 1-7-2026 verifiëren (als datatabel, niet hardcoded).
- [ ] Welke gepubliceerde referentieberekening als fixture? (bijv. RVO/W-E voorbeeldwoning of SBK-referentiegebouw — beschikbaarheid checken.)
- [ ] Cat. 3 API: rate limits, licentievoorwaarden bulk-cache, en of module D meegeleverd wordt — check bij toegang.
- [ ] Eenheidconversie profiel ↔ geometrie (kg vs m² vs m³ vs stuks per productkaart) — dekkingsgraad functionele eenheden in cat. 3 data verkennen.
- [ ] Kengetallen-benchmarkset schets-niveau: bron kiezen (eigen projecten? publicaties?).
