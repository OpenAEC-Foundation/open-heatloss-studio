# ISSO 53 UI-spoor — werkpakket

**Datum:** 2026-05-23
**Status:** Concept ter goedkeuring
**Architectuurkeuze (PM, 2026-05-23):** **Optie A** — norm-keuze bij project-aanmaak (wonen vs utiliteit) → UI past zich aan
**Voorwaarde:** ISSO 53 rekenkern is klaar (commits `2ba43f2` / `31d7c1e` / `1d0256e`)

## Beslissingen (user, 2026-05-23)

| # | Vraag | Besluit |
|---|-------|---------|
| 1 | Bestaande projecten zonder `norm`-veld | **Silent migration** → default `Isso51` |
| 2 | Wisselen tussen 51/53 binnen bestaand project | **Toegestaan met waarschuwing** (data-conversie-modal) |
| 3 | PDF-rapport template | **Dezelfde template** voor 51 en 53 (alleen andere getallen + andere sectie-titels) |

## Faseplan

| Fase | Wat | LOC indicatie | Risico |
|------|-----|---------------|--------|
| **1** | Backend: `norm: Norm` veld op Project + silent migratie + dual-pipeline routing | ~100 backend | Laag |
| **2** | Frontend: norm-radio in Backstage/NewProject + norm-badge in topbar | ~120 frontend | Laag |
| **3** | Conditional rendering bestaande screens: ISSO 51 RoomFunction ↔ ISSO 53 GebruiksFunctie+RuimteType | ~500 frontend | **Middel** — meerdere screens, veel kleine wijzigingen |
| **4** | Wissel-flow met waarschuwingsmodal + data-conversie-mapping | ~150 frontend | Middel |
| **5** | Rapport-bouwer: norm-switch in sections, label-mapping, ISSO 53 voorbeeldrapport | ~200 mixed | Middel |

**Totaal:** ~1070 LOC verspreid over 5 fasen. Logische commit-checkpoints na elke fase.

---

## Fase 1 — Backend dual-pipeline (rust-developer)

### Data-model
Voeg toe in `src-tauri/src/...` (of dezelfde locatie waar Project leeft op de backend):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Norm {
    Isso51,
    Isso53,
}

impl Default for Norm { fn default() -> Self { Self::Isso51 } }
```

Veld op `Project`:
```rust
#[serde(default)]
pub norm: Norm,
```

`#[serde(default)]` zorgt voor **silent migration**: bestaande JSON zonder `norm`-veld krijgt `Isso51`.

### Routing
Backend-pipeline kiest crate o.b.v. `project.norm`:
- `Norm::Isso51` → `isso51_core::calculate(project_51)` (huidige route)
- `Norm::Isso53` → vertaal Project naar `isso53_core::Project` + `isso53_core::calculate`

Eerste implementatie: aparte velden per norm op het Tauri-niveau Project — `room.iso51_fields` / `room.iso53_fields` (alleen één gevuld o.b.v. norm). Voorkomt zware type-gymnastics.

Alternatief: één Project-type voor beide normen met optionele velden. Beslis op basis van isso53-core::Project signature versus huidige isso51 Project; rust-dev kiest pragmatisch.

### Acceptatie fase 1
- `cargo test` groen
- Bestaand v1-projectbestand opent zonder error en gedraagt zich als ISSO 51 (silent migration werkt)
- Nieuw project met `"norm": "isso53"` route via isso53-core

---

## Fase 2 — Norm-keuze UI (frontend-developer)

### Backstage / NewProject scherm
Voeg toe boven "Projectnaam"-veld:
```
[ ] Wonen (ISSO 51)
[ ] Utiliteit ≤ 4m (ISSO 53)
```
Default: ISSO 51. Niet meer wijzigbaar na keuze → toon de gekozen norm als read-only badge.

### Topbar/TitleBar
Permanente badge `ISSO 51` of `ISSO 53` rechts van projectnaam. Klikbaar → opent fase-4 wissel-modal.

### Acceptatie fase 2
- Nieuwe projecten krijgen norm uit UI gekozen
- Bestaande projecten tonen "ISSO 51" badge zonder modificatie nodig
- Visueel onderscheid duidelijk (kleur/icoon)

---

## Fase 3 — Conditional rendering (frontend-developer, grootste fase)

### Te beïnvloeden screens
| Screen | Wijziging | Geschatte regels |
|--------|-----------|-----------------|
| `ProjectSetup/AlgemeenTab` | Toon `BuildingShape` + `BuildingPosition` + `WindPressureType` voor 53 i.p.v. `BuildingType` voor 51 | 80 |
| `ProjectSetup/RuimteTab` | Ruimte-aanmaak: voor 51 `RoomFunction`, voor 53 dropdown `GebruiksFunctie` + afhankelijke `RuimteType` | 100 |
| `RoomTable` | Kolom "Functie" toont `GebruiksFunctie • RuimteType` voor 53 | 60 |
| `Resultaten` | Per-vertrek breakdown: voor 53 toon ook `phiHu` + `phiI` apart | 80 |
| `Modeller` | Out of scope fase 3 — alleen 51 ondersteund (TODO voor latere fase) | — |
| `Library/Materialen` | Geen wijziging — constructies zijn norm-onafhankelijk | 0 |

### Patroon
```tsx
{project.norm === "isso51" ? <Isso51FunctionPicker ... /> : <Isso53FunctionPicker ... />}
```

Hou per-norm-components klein en herbruikbaar — vermijd één mega-component met if/else overal.

### Acceptatie fase 3
- 51-project: UI identiek aan huidig
- 53-project: alle norm-specifieke velden zichtbaar, ISSO 51-velden verborgen
- Geen regressies in bestaande 51-flow

---

## Fase 4 — Wissel-flow met waarschuwing (frontend-developer)

### Trigger
Klik op norm-badge in topbar → modal:

> **Norm wisselen**
>
> Je staat op het punt dit project van **ISSO 51** naar **ISSO 53** te wisselen.
>
> ⚠️ Niet alle gegevens worden 1-op-1 overgenomen:
> - Ruimte-functies (Woonkamer, Slaapkamer, …) worden gemapt naar utiliteit-functies (Kantoor.Verblijfsruimte, …) — controleer per ruimte
> - Ventilatie-eis verandert van per-m² (Bouwbesluit-wonen) naar per-persoon × bezetting (tabel 4.10)
> - Bedrijfsbeperking-toeslag verandert van main-room-percentage naar specifieke toeslag P [W/m²]
>
> Een **back-up** van het project wordt opgeslagen als `{naam} (v ISSO 51 backup).json` in dezelfde map.
>
> [Annuleren] [Wissel naar ISSO 53]

### Data-conversie-mapping
Map huidige 51-velden naar 53-velden (best-effort defaults):

| ISSO 51 | → ISSO 53 |
|---------|-----------|
| `RoomFunction::LivingRoom` | `GebruiksFunctie::Kantoor` + `RuimteType::Verblijfsruimte` |
| `RoomFunction::Bedroom` | `GebruiksFunctie::Kantoor` + `RuimteType::Verblijfsruimte` |
| `RoomFunction::Bathroom` | `GebruiksFunctie::Kantoor` + `RuimteType::Badruimte` |
| Alle andere | `GebruiksFunctie::Kantoor` + `RuimteType::OnbenoemdeRuimte` |

User moet handmatig verfijnen na de wissel.

### Acceptatie fase 4
- Wissel werkt + back-up wordt opgeslagen
- Modal toont alle veranderingen voor user-akkoord
- Geen data-verlies (backup beschikbaar)

---

## Fase 5 — Rapport (rust-developer + frontend-developer)

### Backend (`isso53-report-builder`)
Bouw naast `isso51-report-builder` een `isso53-report-builder` met **dezelfde PDF-template** (`bm-reports` of huidige template-keuze in open-heatloss-studio):
- Voorblad: project-info + norm-aanduiding ("ISSO 53 — Warmteverliesberekening utiliteit")
- Inhoudsopgave
- Per vertrek: ΦT, ΦV, ΦI, ΦHu, ΦHL tabel
- Gebouw-totaal: aansluitvermogen individueel + collectief + shell
- Bijlage: constructies-overzicht (gedeeld met 51)

### Frontend
Knop "Genereer rapport" routed naar juiste backend o.b.v. `project.norm`. Geen UI-verandering verder.

### Acceptatie fase 5
- 51- en 53-rapport-PDFs gebruiken identieke styling/layout
- 53-rapport toont ISSO 53-specifieke termen (gebruiksfunctie, etc.)
- PDF-test: één 51-project + één 53-project rendert zonder error

---

## Volgorde + commit-strategie

1. **Fase 1** alleen, commit `feat(norm): backend dual-pipeline + silent migration`
2. **Fase 2** alleen, commit `feat(norm): norm-keuze UI in project-aanmaak + topbar-badge`
3. **Fase 3** in 2-3 sub-commits per screen, eindcommit `feat(53): conditional rendering bestaande screens`
4. **Fase 4** alleen, commit `feat(norm): wissel-flow met waarschuwing + data-mapping`
5. **Fase 5** alleen, commit `feat(report): ISSO 53 rapport-builder + frontend-routing`

Tussen elke fase een **handmatige smoke-test** door user op de echte app (`pnpm tauri dev` of vergelijkbaar). PM verifieert build + clippy + tests groen voor elke commit.

## Out of scope dit werkpakket

- ISSO 57 voorbereidingen (vertrek > 4m)
- Modeller-integratie ISSO 53 (separate spoor)
- IFCX `isso53::` namespace (separate spoor)
- BAG-data import voor utiliteit
- Quick-calc wizard
