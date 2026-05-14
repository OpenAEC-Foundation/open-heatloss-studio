# ADR-002 — Multi-calc Project architectuur

| | |
|---|---|
| **Status** | Geaccepteerd (2026-05-14) |
| **Context** | TO-juli (NTA 8800 H.10) toevoegen + uitbreidbaarheid voor ISSO 53, glaser-jaar, EP-score |
| **Scope** | Repo `open-heatloss-studio` — Project-model, backend storage, frontend store/UI |
| **Vervangt** | Project-model uit ISSO 51 V1 (`Project` in `isso51-core::model::building`) |

---

## Probleem

De huidige `Project`-struct in `isso51-core` is volledig ISSO 51-gericht: één building, één set rooms, één ventilation-config. Aankomende uitbreidingen vereisen:

1. **TO-juli volledig** (NTA 8800 H.10) voor woningen én utiliteit — eigen koelsysteem/distributie/emissie-inputs, monthly Q_C;nd via demand-pipeline.
2. **ISSO 53** (toekomstig) — luchtkanalen, eigen schema.
3. **Glaser jaarmethode** — vochthuishouding, eigen klimaat-binding.
4. **EP-score** (NTA 8800 H.5) — integratie van alle bovenstaande.

Elk van deze berekeningen heeft **specifieke inputs**, maar deelt **algemene project-data**: gebouwlocatie, adres, postcode, gebouwtype (woning/utiliteit + subtype), bouwjaar, gebruiksoppervlak. Het is ondoenlijk om die data per berekening opnieuw in te voeren.

Daarnaast: **gedeelde geometrie**. Een gebouw is fysiek één set wanden/vloeren/daken/ramen. ISSO 51 ziet het als `Room + ConstructionElement[]`. NTA 8800 ziet het als `Rekenzone + EFR + Window`. Het model en de invoer-UX moet één gemeenschappelijke geometrie aanbieden, met view-mappers per norm.

## Beslissing

### Drielagig Project-model

```
ProjectV2
├── shared:           Eénmalige cross-calc invoer (≈ ProjectInfo + locatie + gebouwtype)
├── geometry:         Gedeelde geometrie (kamers / constructies / ramen) — single source
└── calcs:            Map<CalcKey, CalcInputs> — per-norm specifieke inputs
    ├── "isso51"  → Iso51Inputs    (huidige Building/Ventilation/Room.heating fields)
    ├── "tojuli"  → TojuliInputs   (cooling system, distribution, emission, blinds-strategie)
    └── (later: "iso53", "glaser", "ep")
```

### Crate-allocatie

| Sectie | Crate | Reden |
|---|---|---|
| `SharedProject` | nieuw: `openaec-project-shared` | Crate-onafhankelijk, alle calcs gebruiken het |
| `SharedGeometry` | idem | Geometrie is generiek (Room/Wall/Floor/Roof/Window/Door) — geen ISSO 51-only |
| `Iso51Inputs` | `isso51-core` | Bestaande calc, eigen veld |
| `TojuliInputs` | `nta8800-cooling::project_input` | Cooling-specifieke inputs landen bij de calc |
| **View-mappers** | per calc-crate | bv. `isso51-core::project::from_shared(&SharedProject, &SharedGeometry, &Iso51Inputs) -> Project` |

Voor multi-norm projecten: één `ProjectV2` JSON, meerdere `calcs[*]` populated. Een calc-crate kan **alleen** runnen als alle vereiste fields ingevuld zijn (validatie aan calc-rand).

### Geometrie sharing

`SharedGeometry` = canoniek model:
- `Space[]` — verblijfsruimte/ruimte (mapt naar ISSO 51 Room en NTA 8800 EFR)
- `Construction[]` — wand/vloer/dak/raam/deur per Space, met boundary type, area, U-waarde, oriëntatie
- `Building` — verzameling Spaces + gebouwniveau eigenschappen

ISSO 51 view-mapper en NTA 8800 view-mapper transformeren dit naar hun calc-specifieke structs. Heuristieken voor `Room → EFR` aggregatie (NTA 8800 werkt op rekenzone-niveau, ISSO 51 op kamerniveau).

### Backend storage

- Eén `projects_v2` tabel met `data JSONB` voor de hele struct (snel uitbreidbaar, geen schema-migratie per nieuwe calc)
- `calc_type` indexkolom voor filtering ("heeft tojuli", "heeft isso51")
- Migratie van bestaande `projects` tabel: scripted reader die oude `Project`-JSON omzet naar `ProjectV2` met `shared` + `geometry` gevuld uit huidige Building/Rooms, `calcs.isso51` = rest

### Frontend

- `projectStore` opgesplitst in slices: `sharedSlice`, `geometrySlice`, `calcsSlice` (Zustand combine pattern)
- ProjectSetup-pagina → tabs: **Algemeen** (shared) · **Geometrie** (rooms/walls) · **ISSO 51** · **TO-juli** · placeholder voor toekomst
- Per calc-tab toont validatie ("Vul X in onder Algemeen om hier te kunnen rekenen")

### Backward-compatibility

- Bestaande `Project` JSON blijft serialiseerbaar (frozen interface) — calc-rand accepteert beide
- `ProjectV2 → Project` mapper bestaat (one-way; voor de bestaande ISSO 51 calc-call)
- Geen big-bang DB-migratie: dual-read in API tot alle clients geüpgraded

## Consequenties

### Voordelen

- Multi-calc projecten met één bron van waarheid voor geometrie
- Nieuwe calcs toevoegen = nieuwe `calcs[<key>]` variant + view-mapper, geen wereldwijde refactor
- TO-juli kan voor utiliteit volledig norm-conform omdat gedeelde geometrie + NTA 8800 view-mapper alle Rekenzone/EFR/Window data leveren

### Trade-offs

- Indirectie: calc-rand moet altijd door view-mapper (kleine performance kost, verwaarloosbaar voor onze maandelijkse berekeningen)
- Migratie-eenmalige investering (zie F2 in fasering)
- `ProjectV2` JSON is groter (alle inputs samen) — niet erg, MBs niet GBs

### Niet-doelen

- BCF Platform Project (apart product, andere repo) — geen poging tot DRY daar
- Multi-tenant project-sharing — buiten scope
- Real-time collaboratie — buiten scope

## Implementatie-fasering (zie orchestrator TaskList #6-#13)

| Fase | Wat | Effort |
|---|---|---|
| F1 | Deze ADR | 0,5d |
| F2 | Backend ProjectV2 schema + migratie + dual-read | 1-1,5d |
| F3 | Frontend ProjectSetup multi-tab + store-split | 1-2d |
| F4 | `nta8800-demand` crate (blocker H.10) | 3-5d |
| F5 | NEN 5060 klimaattabellen in `nta8800-tables` | 0,5-1d |
| F6 | Geometry mapper ISSO 51 → NTA 8800 | 1-2d |
| F7 | TO-juli UI volledig (H.10 + utiliteit) | 2d |
| F8 | PDF rapport TO-juli via BM Reports | 1d |

**Totaal:** 2-3 weken, gespreid over sessies + agent-delegaties.

### Legacy MVP-pad

De huidige `/tojuli` route (commit `29d3ed9`) gebruikt bijlage AA (woningen-only, simplified). **Beslissing:** behouden als `/tojuli/quick` — "expert mode snelle check voor woningen, niet norm-volledig". Volledig pad komt op `/tojuli` zelf via F7. Frontend-route + i18n updaten in F3.

## Open punten

- KNMI-locatie binding voor klimaatdata (postcode → station). Zie TODO-tools.md regel 193-195. Mogelijk parallel met F5 op te pakken.
- Multi-rekenzone projecten: vereist UX-werk in F3 om gebouw op te delen in zones. Voor MVP: één rekenzone = hele gebouw. Multi-zone uitstellen naar v1.1.
- Agent/Variant patroon analoog aan Vabi voor user-eigen project-varianten (basisontwerp vs renovatie). Uitstellen.

## Referenties

- TODO-tools.md regel 160-192 — NTA 8800 implementatie-roadmap
- Vabi-schema-reference.md — Aspect/Template patroon (vergelijkbaar concept voor variant-overrides, voor inspiratie bij v1.1)
- Project-registry.json — orchestrator-context
