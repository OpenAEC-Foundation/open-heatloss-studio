# Plan — Ventilatiebalans-module (Open Heatloss Studio)

**Datum:** 2026-06-06 | **Status:** architect-plan, klaar voor gefaseerde delegatie
**Repo's:** `C:/Github/open-heatloss-studio` (Rust+React, hoofdwerk) · `C:/Github/pyrevit-gis2bim` (Revit-export)
**Beslissingen (user, 06-06):** norm-grondslag = **BBL + NEN 1087 + NTA 8800** · ventiel-bron = **Revit-import + handmatig bijwerken** · scope = **volledige port incl. units-database**

> ## ⚓ Architectuur-beslissing 06-06 — IFC/IFCX geparkeerd
> De vraag "IFC als basis i.p.v. eigen JSON" is overwogen en **bewust uitgesteld**. Bevinding: het IFCX-fundament werkt (`isso51-ifcx` roundtrip, `.ifcenergy`), maar de "herbruikbaar in Revit"-belofte vergt echte IFC-entiteit-spreiding (nu één `isso51::`-blob), een 2D→3D-geometrie-strategie, en een pyRevit IFCX-export — samen weken werk.
> **Besluit:** de hele tool (incl. ventilatie) eerst **áf bouwen in het huidige model** als *werkende, levende specificatie*. Pas wanneer alles werkt en de gewenste werking volledig helder is, komt er een **IFC-module ernaast** die langzaam migreert. Ventilatie wordt dus **pragmatisch** gebouwd (bestaand 2D calc-model + sidecar, Modeller-mode), niet IFCX-native. Ventielen later als `IfcAirTerminal` bij de IFC-migratie.
> **Gevolg voor dit plan:** fase 1-5 ongewijzigd, maar expliciet op het eigen model. Geen IFC-werk in scope.

---

## 1. Doel

Eén geïntegreerde ventilatiebalans-module in de warmteverlies-web-tool, die de standalone pyRevit `VentilatieBalans`-plugin vervangt/overneemt én een **visuele weergave op de plattegrond** toevoegt:

- **Pijl buiten→binnen** — natuurlijke/mechanische toevoer door de gevel
- **Inblaasventiel** met pijl (mechanische toevoer, systeem B/D)
- **Afzuigventiel** met pijl (mechanische afvoer, systeem C/D — keuken/bad/toilet)
- **Ruimte-op-ruimte overstroom** met pijl (toevoerruimte → afvoerruimte via binnendeur)
- **Spleet onder de deur** — berekening + indicator van de benodigde vrije doorlaat (doorstroomopening)

---

## 2. Architectuur-overzicht

```
┌─ Revit (pyRevit) ──────────────┐      ┌─ Open Heatloss Studio (web) ─────────────────┐
│ VentilatieBalans.pushbutton    │      │  Frontend — ventilatie-MODE in de Modeller   │
│  • DuctTerminals + posities    │ JSON │   • nieuwe tools place_supply/exhaust         │
│  • room gebruiksfunctie/zone   │─────▶│   • pijl/ventiel-laag in FloorCanvas          │
│  → ventilation-export (nieuw)  │      │   • spleet/overstroom op bestaande ModelDoor  │
└────────────────────────────────┘      │  Rust crates                                  │
                                         │   • ventilation-balance (NIEUW): BBL+NEN1087  │
                                         │   • nta8800-ventilation (bestaand): koppeling │
                                         └───────────────────────────────────────────────┘
```

**Drie norm-lagen, één datamodel:**

| Norm | Rol in de module | Grootheden |
|------|------------------|-----------|
| **Bouwbesluit/BBL** | Capaciteitseis per ruimte (toevoer/afvoer) | dm³/(s·m²) per gebruiksfunctie + minima (woon 0,7 / overig-verblijf 0,9 / toilet 7 / bad 14 / keuken 21 dm³/s) |
| **NEN 1087** | Overstroom/doorstroomopeningen (spleet onder deur), spui | debiet↔drukverschil doorstroomopening; vrije-doorlaat-dimensionering |
| **NTA 8800** | Energetische koppeling (TO-juli/warmteverlies) | q_V;ODA;req-keten, m³/h-debieten voeden bestaande `tojuli.rs` |

De BBL-balans levert per-ruimte debieten; die worden geaggregeerd naar de gebouw/zone-debieten die NTA 8800 (`mechanical_supply/exhaust_m3_per_h`) en ISSO 51 al consumeren. **Eén invoer, drie uitkomsten** — geen dubbele debiet-invoer meer (lost tevens TO-juli werkpakket B/C2-spanning op: zie `2026-05-21-tojuli-ventilatie-norm-werkpakket.md`).

---

## 3. Datamodel (Fase 1)

### 3.1 Gebouw-niveau — nieuw `VentilationTerminal[]` op `Project`

```typescript
// frontend/src/types/project.ts  (+ Rust mirror in openaec-project-shared)
interface VentilationTerminal {
  id: string;
  room_id: string;
  terminal_type: "supply" | "exhaust";      // toevoer / afvoer
  source: "revit" | "manual";               // herkomst (import vs handmatig)
  // Plaatsing voor visualisatie — wand-gebonden of vrij in ruimte:
  wall_index?: number;                       // edge-index in room.polygon (NEN-gevel of binnenwand)
  offset_mm?: number;                        // positie langs de wand-edge
  position_mm?: { x: number; y: number };    // óf vrije positie (plafondventiel)
  capacity_m3_per_h?: number;                // uit Revit-param of geschat
  family_name?: string;
  mark?: string;
}
```

### 3.2 Room-niveau — uitbreiden

| Veld | Status | Toevoegen |
|------|--------|-----------|
| `ventilation_function` | nieuw | gebruiksfunctie-classificatie (woon/keuken/bad/toilet/verkeer…) → BBL-norm-lookup |
| `required_supply_m3_per_h` | nieuw | berekend uit BBL-eis |
| `required_exhaust_m3_per_h` | nieuw | berekend uit BBL-eis |
| `air_source_room_id` | **bestaat** | overstroom-bron — nu eindelijk visualiseren (ruimte-op-ruimte pijl) |
| `ventilation_type_override` | nieuw | handmatige toevoer/afvoer/geen-override (zoals plugin) |

### 3.3 Persistentie
- Terminals + per-room velden in de **`sharedExtra`-sidecar** (zelfde patroon als TO-juli/KNMI-klimaat, zie `projectV2.ts`). Mee-serialiseren in `.heatloss.json` + `.ifcenergy` (let op de save→reopen-valkuil die in `8ccff9f` is gefixt).
- Reproduceerbaar: terminal-`id`'s stabiel houden over import-rondes (Revit `ElementId` als basis voor `source:"revit"`).

---

## 4. Rekenkern (Fase 2) — nieuwe crate `ventilation-balance`

> Port van de BBL-logica uit `VentilatieBalans.pushbutton/script.py` (NORMEN_BBL, `_bereken_ventilatie_eis`, `_bereken_overdruk_verdeling`) naar getypeerde, geteste Rust. **Norm-kritisch.**

| Module | Inhoud | Norm-bron (aanleveren aan agent) |
|--------|--------|----------------------------------|
| `tables/bbl_requirements.rs` | gebruiksfunctie → dm³/(s·m²) + minimum + default-type | Bouwbesluit/BBL afd. 3.6 — tabel uit `NORMEN_BBL` (al in plugin) |
| `calc/room_demand.rs` | per-ruimte eis = max(opp×spec, personen×pp, minimum) | BBL + ISSO 62 (pp-toeslag) |
| `calc/overflow.rs` | overdruk-verdeling: toevoer−afvoer per zone → overstroom naar afvoer-ruimtes naar rato oppervlakte | NEN 1087 §overstroom |
| `calc/door_gap.rs` | **spleet onder deur**: benodigde vrije doorlaat A uit overstroomdebiet q + toelaatbaar ΔP (~1 Pa). Orifice: `q = C_d·A·√(2ΔP/ρ)` → `A = q/(C_d·√(2ΔP/ρ))`. **C_d + ΔP-criterium uit NEN 1087 verifiëren** | **NEN 1087 — exacte formule/parameters aanleveren** |
| `calc/zone_balance.rs` | zone-balans + WTW/MV-unit-capaciteit-toets (port `_get_gecombineerde_eis`) | — |
| `aggregate.rs` | per-ruimte → gebouw/zone m³/h voor NTA 8800 + ISSO 51 | NTA 8800 §11.2 (pagina's in TO-juli werkpakket-doc) |

**Acceptatie:** unit-tests per module; ijking tegen een handberekening van een referentiewoning (de mockup-tabel in `mockups/pages/ventilation.html` als startpunt: woonkamer 25,38 / bad 14,0 / etc.).

> ⚠️ **Normdocumenten vereist vóór delegatie van fase 2:** NEN 1087 (doorstroomopening-formule + C_d + ΔP-criterium) en de BBL-tabel-bevestiging. NTA 8800-pagina's staan al in `2026-05-21-tojuli-ventilatie-norm-werkpakket.md` §Norm-referenties. Zonder NEN 1087-uittreksel is `door_gap.rs` een benadering, geen norm-exacte berekening.

---

## 5. Visualisatie (Fase 3) — nieuwe **ventilatie-mode in de bestaande Modeller**

> **Geen nieuwe canvas, geen losse component.** De Modeller (`pages/Modeller.tsx` + `components/modeller/FloorCanvas.tsx`) is al een volwaardige 2D/3D viewer/editor met room-geometrie, deuren, ramen, tool-modes, zoom/pan en PDF/IFC-onderlegger. Ventilatie wordt een **mode/laag hierin**. Konva is enkel de renderer eronder — geen technologie-keuze.

**Herbruikbare bouwstenen (geverifieerd in `Modeller.tsx`):**

| Bestaand | Hergebruik voor ventilatie |
|----------|----------------------------|
| `useModellerToolStore` (`tool`: select/draw_window/draw_door/…) | nieuwe tools `place_supply` / `place_exhaust` |
| `onAddWindow(roomId, wallIndex, offset, width)` + `findWallHit()` | identiek patroon om ventielen op een wand te plaatsen |
| `ModelDoor` (`wallIndex/offset/width/swing`) — **deuren bestaan al** | overstroom-pijl + spleet-onder-deur hangen direct aan een bestaand object — geen nieuwe geometrie |
| `deriveModelDoors/Rooms/Windows` | ventiel-overlay afleiden naast de bestaande afgeleide modellen |
| 2D/3D `viewMode`-toggle, `PropertiesPanel` | ventiel-eigenschappen (debiet/type) in het bestaande properties-panel |

**Render-laag:** nieuwe Konva-`<Group>` in `FloorCanvas.tsx` **na de walls, vóór de labels**, zichtbaar wanneer de ventilatie-mode actief is (analoog aan hoe windows/doors als groepen renderen). Coördinaat-transform voor een ventiel op `{wall_index, offset_mm}`:
```
a = room.polygon[wall_index]; b = room.polygon[(wall_index+1) % n]
t = offset_mm / hypot(b-a); p = a + t*(b-a)   // wereld-mm, deelt de bestaande zoom/pan-Group
```
**Spleet-onder-deur** itereert over de bestaande `doors` (per binnendeur: `door.width` + overstroomdebiet → benodigde vrije doorlaat → spleethoogte). **Overstroom-pijlen** lopen door de bestaande deur-objecten tussen `room.air_source_room_id` en de afvoer-ruimte.

**Visuele taal:** gevalideerd in de dummy `mockups/pages/ventilation-balance.html` (kleuren, ventiel-glyphs, gevelrooster-pijl, overstroom-stippelpijl, spleet-balk). Die dummy blijft als referentie; de echte rendering gebeurt in de Modeller.

| Pijl-type | Glyph + richting | Trigger |
|-----------|------------------|---------|
| Gevel-toevoer | pijl door buitenwand naar binnen | toevoer-ruimte, natuurlijk (rooster) of systeem A/B |
| Inblaasventiel | rond glyph + pijl de ruimte in | `terminal_type:"supply"`, mechanisch |
| Afzuigventiel | rond glyph + pijl uit de ruimte | `terminal_type:"exhaust"` (keuken/bad/toilet) |
| Ruimte-op-ruimte | pijl door binnendeur, bron→doel | `room.air_source_room_id` gezet |
| Spleet onder deur | balk-indicator bij deur + tekst "benodigd A cm² / aanwezig" | `door_gap.rs`-resultaat per binnendeur |

- Kleurcodering: toevoer (blauw/koel), afvoer (warm/rood), overstroom (neutraal). Hergebruik bestaande deur/raam-marker-patronen.
- Debiet-label bij elk ventiel (m³/h). Balans-indicator per ruimte (✓/tekort).

---

## 6. Ventiel-invoer (Fase 4) — import + handmatig

### 6.1 Revit-export (`pyrevit-gis2bim`)
- `VentilatieBalans.pushbutton` krijgt een **"Export naar web"**-knop → schrijft `ventilation.json` (terminals: id/positie/type/mark/flow-param + room gebruiksfunctie/zone), parallel aan de thermal-export. Hergebruikt de bestaande `AirTerminalData`/`RuimteData`-collectie (script.py:132-322).
- Positie: Revit feet → mm (×304,8). Wand-binding optioneel; anders vrije `position_mm`.
- Geen wijziging aan `json_builder.py` (thermische export) nodig — aparte ventilatie-JSON houdt de scheiding schoon.

### 6.2 Web-import + handmatig
- Import-pagina leest `ventilation.json` → `VentilationTerminal[]` (`source:"revit"`).
- Handmatige editor: klik op plattegrond → ventiel plaatsen/typen/debiet (`source:"manual"`), of Revit-ventiel corrigeren.
- Re-import merge-strategie: `source:"revit"` overschrijven, `source:"manual"` behouden.

---

## 7. Rapport + units (Fase 5)

- **Units-database:** port `ventilatie_units.json` (WTW/MV-units: fabrikant/model/capaciteit/rendement/geluid) + zone-toewijzing + capaciteitstoets (plugin `_setup_units_tab`, `ZoneUnitToewijzing`).
- **Rapport-sectie:** balans-tabel per ruimte/zone (eis vs. aanwezig, toevoer/afvoer, overstroom, spleet-doorlaat) via de openaec-reports renderer. Plattegrond-snapshot met pijlen als rapport-figuur (overweeg: Konva→PNG-export).

---

## 8. Volgorde & afhankelijkheden

| Fase | Agent | Blokkade | Norm-doc nodig |
|------|-------|----------|----------------|
| 1 Datamodel | rust + frontend (parallel, gedeelde struct-spec eerst) | — | — |
| 2 Rekenkern | rust-developer | **NEN 1087 + BBL-tabel-uittreksel** | ✅ kritisch |
| 3 Visualisatie | frontend-developer | fase 1 (datamodel) | — |
| 4 Import/handmatig | revit-bim-specialist (export) + frontend (import) | fase 1 | — |
| 5 Rapport/units | frontend + python-developer | fase 1-3 | — |

**Kritische pad:** Fase 1 → 2 (rekenkern) en 1 → 3 (visualisatie) lopen parallel na het datamodel. Fase 2 is geblokkeerd tot de **NEN 1087-formules** zijn aangeleverd (zoals het TO-juli-werkpakket de NTA 8800-pagina's aanleverde).

---

## 9. Risico's / open punten

1. **NEN 1087-grondslag** — exacte doorstroomopening-formule (C_d, ΔP-criterium) moet uit het normdocument; zonder = benadering. **Actie user:** normpagina's aanleveren (analoog aan NTA 8800-pagina's op de share).
2. **Nieuwe crate vs. uitbreiden** — `ventilation-balance` als aparte crate (BBL/NEN 1087) gekozen boven uitbreiden van `nta8800-ventilation` (andere norm-grondslag, schonere scheiding). Te bevestigen bij fase 1.
3. **Konva→PNG voor rapport** — haalbaarheid plattegrond-snapshot in rapport verifiëren (fase 5).
4. **Personen-data** — BBL pp-toeslag vergt bezetting per ruimte; komt die uit Revit (param `bezetting`) of handmatig in web? (plugin leest Revit-param.)
5. **Overlap TO-juli werkpakket B** — de aggregatie (fase 2 `aggregate.rs`) raakt `air_change_rate_per_h`-vervanging uit werkpakket B. Coördineren zodat niet dubbel gebouwd wordt.
