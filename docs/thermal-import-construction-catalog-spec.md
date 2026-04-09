# Thermal Import — Construction Catalog refactor (Mini-spec)

**Datum:** 2026-04-09
**Status:** Draft
**Scope:** `crates/isso51-core/src/import/thermal.rs` backend + frontend import wizard
**Relatie:** Bug A uit review sessie 2026-04-09 (woonboot 3056 testmodel)

---

## Probleem

De huidige thermal import produceert **twee inconsistente views** van dezelfde data:

| Output veld | Inhoud | Frontend view |
|---|---|---|
| `project.rooms[].constructions` | gegroepeerd (phase 2, thermal.rs 473-534) | vertrekken-view |
| `construction_layers` | **flat, per raw surface** (thermal.rs 320-325) | constructies-view |

Gevolg in UI:
- **Vertrekken-view:** constructies samengevoegd (gebruiker ziet 4 groepen ipv ~39)
- **Constructies-view:** 83 losse schillen, geen relatie met wat in vertrekken-view staat

### Verergerd door ontbrekende `revit_type_name`

De huidige grouping key is `(revit_type_name, boundary_type, orientation)`. In de testexport (3056 woonboot) blijkt `revit_type_name` echter **in alle 83 constructies afwezig**, waardoor alle walls/floors op `"onbekend"` vallen. Resultaat: 83 unieke surfaces → slechts **4 groepen** in vertrekken-view:

```
(onbekend, Exterior, Wall), (onbekend, Exterior, Floor),
(onbekend, Ground,   Wall), (onbekend, Ground,   Floor)
```

De daadwerkelijke uniciteit zit in de **layer-samenstelling**: 39 unieke layer-fingerprints in dezelfde export.

---

## Doelen

1. **One source of truth:** unieke constructies worden één keer gedefinieerd; room-surfaces verwijzen ernaar via ID.
2. **Grouping op semantische inhoud:** layer fingerprint + SfB code, niet op (optionele) Revit-type naam.
3. **Behoud van adjacent room info:** interior boundaries moeten zien aan welke kamer ze grenzen (zie bug B, aparte fix).
4. **Consistente UI:** vertrekken-view en constructies-view kijken naar dezelfde lijst.

---

## Nieuwe output structuur

```rust
/// Resultaat van thermal import mapping.
pub struct ThermalImportResult {
    pub project: Project,
    pub warnings: Vec<String>,
    /// Unieke constructies (gedeeld tussen rooms), gegroepeerd op layer-fingerprint.
    pub construction_catalog: Vec<CatalogEntry>,
    /// Room polygons voor 3D viewer (ongewijzigd).
    pub room_polygons: Vec<RoomPolygon>,
}

/// Eén unieke constructie (layer-samenstelling) in de catalogus.
pub struct CatalogEntry {
    /// Catalogus ID, format "cat-{n}".
    pub id: String,
    /// SfB-gebaseerde description (bv "21_Stuc_KZS_PIR_Spouw_Klinker").
    pub description: String,
    /// Layer-samenstelling van interieur naar exterieur.
    pub layers: Vec<ThermalLayer>,
    /// Eerst aangetroffen Revit type naam (indien aanwezig), voor debugging.
    pub revit_type_name: Option<String>,
    /// Voor welke (BoundaryType, Orientation) combinaties wordt deze catalogus-entry gebruikt.
    pub used_for: Vec<(BoundaryType, ThermalOrientation)>,
    /// Totaal oppervlak in m² over alle voorkomens (som van alle surfaces).
    pub total_area_m2: f64,
    /// Aantal keer aangetroffen als losse surface in de ruwe export.
    pub surface_count: usize,
}
```

### Koppeling vanuit `Room.constructions`

Elk `ConstructionElement` in `Room.constructions` krijgt een nieuw veld:

```rust
pub struct ConstructionElement {
    // ...bestaande velden...
    /// Verwijst naar een CatalogEntry.id — None voor openings en legacy entries.
    pub catalog_ref: Option<String>,
}
```

De `description` blijft voor backwards-compat en leesbaarheid gevuld (zelfde string als catalog entry), maar de catalog ref is de echte identiteit.

### Verwijdert

- `pub construction_layers: Vec<ConstructionLayerInfo>` → vervangen door `construction_catalog`

---

## Grouping algoritme

```
PHASE 1 (per room, bestaand):
  Verzamel raw surfaces met (boundary_type, orientation, layers, adjacent_room_id, area)

PHASE 2 (per room, uitgebreid):
  Groepeer per key = (layer_fingerprint, boundary_type, orientation, adjacent_room_id_if_interior)
  - layer_fingerprint = gehashte string van [(material, thickness_mm, type) ...]
  - adjacent_room_id_if_interior = Some(id) voor AdjacentRoom/UnheatedSpace, anders None
  → room-lokale entries met totaal oppervlak + SfB-description

PHASE 3 (nieuw, na alle rooms):
  Bouw globale construction_catalog:
  - Key = (layer_fingerprint, sfb_code)
    [bewust GEEN adjacent_room_id in catalog key — catalog is per unieke constructie,
     adjacent info blijft op room-level ConstructionElement staan]
  - Entry.description = SfB-naam (eerste voorkomen)
  - Entry.used_for = alle unieke (boundary_type, orientation) combinaties
  - Entry.total_area_m2 = som over alle rooms
  - Entry.surface_count = totaal aantal raw surfaces
  → Elke ConstructionElement krijgt catalog_ref = Entry.id
```

### Layer fingerprint algoritme

```rust
fn layer_fingerprint(layers: &[ThermalLayer]) -> String {
    layers.iter()
        .map(|l| format!("{}|{:.1}|{:?}",
            l.material.trim().to_lowercase(),
            l.thickness_mm,
            l.layer_type))
        .collect::<Vec<_>>()
        .join("::")
}
```

Rationale: materiaal-naam + dikte + type is voldoende voor uniciteit. Lambda wordt bewust weggelaten (kan verschillen tussen Revit projects voor hetzelfde materiaal — dezelfde constructie zonder rekenwijziging).

---

## Frontend impact

### Huidige situatie
- Wizard stap "constructies" leest `construction_layers` → toont flat list.
- Wizard stap "vertrekken" leest `project.rooms[].constructions` → toont gegroepeerd.

### Nieuwe situatie
- Wizard stap "constructies" leest `construction_catalog` → toont unieke entries met:
  - SfB-description
  - Layer breakdown
  - Gebruikt in: N rooms, totaal X m²
  - Rc/U berekening editor (bestaand via LayerEditor)
- Wizard stap "vertrekken" leest `rooms[].constructions` → toont per room de ConstructionElement met:
  - `catalog_ref` → klikbare link naar catalog entry
  - `adjacent_room_id` voor interior boundaries (bug B)
  - Area (room-specifiek)

### LayerEditor integratie
Wijzigingen aan een catalog entry propageren automatisch naar alle room-surfaces die ernaar verwijzen (want zelfde ID). Dit is een **verbetering** ten opzichte van de huidige aanpak waarbij wijzigingen per-surface gebeuren.

---

## Migratie / breaking change

- **Breaking change** in JSON response van `POST /api/v1/import/thermal`
- Frontend moet mee — de import wizard is de enige consumer
- Bestaande projecten (opgeslagen via `/projects/save`) zijn **niet** geraakt: het nieuwe `catalog_ref` veld is optioneel op `ConstructionElement`, oude projecten blijven werken zonder ref
- API versie bump overwegen: `POST /api/v2/import/thermal`? Of in-place vervangen v1 — wizard is nog niet breed in gebruik

**Voorstel:** in-place vervangen. De thermal import pipeline is minder dan 24 uur oud en alleen door één gebruiker getest.

---

## Tests

### Rust (thermal.rs)
- Bestaande 14 grouping tests blijven valide maar worden uitgebreid
- Nieuwe tests:
  - `test_catalog_dedupes_surfaces_with_same_layers` — 5 walls met identieke layers → 1 catalog entry
  - `test_catalog_preserves_distinct_layer_thickness` — 2 walls, alleen isolatie-dikte verschillend → 2 catalog entries
  - `test_catalog_without_revit_type_name` — **regressie voor woonboot 3056** — revit_type_name = None, 39 unieke layer fingerprints → 39 catalog entries
  - `test_room_constructions_reference_catalog` — elk element heeft `catalog_ref` die bestaat in catalog
  - `test_interior_surfaces_keep_adjacent_room_id` — voor AdjacentRoom boundaries blijft adjacent_room_id bewaard op ConstructionElement (bug B)

### Integratie fixture
- Voeg nieuwe test fixture toe: `tests/fixtures/woonboot_3056_thermal.json` (sanitized versie van 3056 export)
- End-to-end test: fixture → map_thermal_import → assert 39 catalog entries, 0 plafonds (bekend data-probleem, geen assertion fail)

---

## Niet-doelen (out of scope)

- **Bug C (plafonds/daken ontbreken):** upstream probleem in PyRevit EAM scanner of Revit model. Aparte diagnose.
- **Rc/U-waarden:** gebruiker berekent via LayerEditor, backend blijft U=0 placeholder.
- **Historische projecten migreren:** bestaande .json projectbestanden hoeven niet geconverteerd.
- **openaec-cloud integratie:** ongewijzigd.

---

## Open vragen

1. Moet `CatalogEntry.used_for` ook openings opnemen (als aparte entries), of blijven openings buiten de catalog?
   → **Voorstel:** openings blijven buiten catalog. Ze worden per-surface gedefinieerd met U-waarde uit Revit.
2. Moet SfB-naam uniek zijn per catalog entry of mag er collision zijn?
   → **Voorstel:** geen collision-check. Twee constructies met identieke materialen maar verschillende dikten kunnen dezelfde SfB-naam hebben (bv beiden `21_Stuc_KZS_PIR`). Dat is informatief, geen ID.
3. Moet `catalog_ref` verplicht (`String`) of optioneel (`Option<String>`) zijn op `ConstructionElement`?
   → **Voorstel:** optioneel. Openings en handmatig-toegevoegde elementen hebben geen catalog ref.

---

## Implementatie volgorde (aanbeveling)

1. Nieuwe types in `thermal.rs` (CatalogEntry, aanvulling op ConstructionElement) — compileert nog zonder gebruik
2. Refactor `map_thermal_import` phase 2/3 naar layer-fingerprint grouping
3. Tests schrijven (regressie voor 3056)
4. Frontend wizard aanpassen (separate PR)
5. Spec updaten naar "Implemented" na merge
