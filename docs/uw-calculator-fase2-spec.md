# U_w Kozijn-calculator — Fase 2 spec (rekenlogica + UI)

**Datum:** 2026-05-21
**Status:** Approved (PM-reconstructie uit commit `7727e79` + NEN-EN-ISO 10077-1)
**Scope:** `frontend/` — pure rekenlogica + per-kozijn invoer-UI
**Relatie:** Fase 1 (`7727e79 feat(uw): add uw_breakdown data model`) leverde het datamodel

---

## Context — 3-fasen plan

| Fase | Inhoud | Status |
|---|---|---|
| 1 | Datamodel `UwBreakdown` + `Spacer`-enum op `ConstructionElement` (Rust + JSON-schema + TS-types) | ✅ `7727e79` |
| 2 | **Rekenlogica + UI-sectie** — dit document | ⬜ deze sprint |
| 3 | Rapport-integratie — `uw_breakdown` als onderbouwing in de PDF | ⬜ later |

## Architectuurbesluit — calc draait in frontend TS

`construction.rs:97` stelt expliciet: *"uitsluitend `u_value` is de rekeningang. `uw_breakdown` dient
als persistente onderbouwing"*. De `Spacer`-enum is in `isso51-core` **bewust** lokaal gemirrord
"om geen crate-dependency op `nta8800-tables` te introduceren". De Rust-core herberekent `uw_breakdown`
nooit — het is opslag + rapportbron.

→ **Fase 2 = puur `frontend-developer`-werk.** Geen Rust-wijzigingen, geen nieuw Tauri-command.

---

## Rekenmodel — NEN-EN-ISO 10077-1

Standaard-detailniveau: **uniform kozijn** — één `U_g`, één `U_f`, uniforme profielbreedte,
rooster van `c × r` identieke ruiten.

### Invoer (uit `UwBreakdown`)

| Veld | Eenheid | Betekenis |
|---|---|---|
| `width_mm` (W) | mm | raambreedte buitenwerks |
| `height_mm` (H) | mm | raamhoogte buitenwerks |
| `frame_width_mm` (f) | mm | uniforme profielbreedte (buitenkozijn + tussenprofielen) |
| `pane_columns` (c) | — | aantal ruit-kolommen, ≥ 1 |
| `pane_rows` (r) | — | aantal ruit-rijen, ≥ 1 |
| `u_g` | W/(m²·K) | glas-U-waarde (handmatig, glasleverancier) |
| `u_f` | W/(m²·K) | profiel-U-waarde (handmatig, profielfabrikant) |
| `spacer` | enum/null | randafstandhouder voor ψ_g-tabelwaarde; `null` = volledig handmatig |
| `psi_g` | W/(m·K) | effectieve ψ_g (tabel óf handmatig) |
| `psi_g_is_manual` | bool | `true` = handmatige override op de spacer-tabelwaarde |

### Afleiding (alles in mm, daarna → m / m²)

```
A_w   = W · H                                  (totale raam-oppervlakte)
A_g   = (W − (c+1)·f) · (H − (r+1)·f)          (totaal glasoppervlak)
A_f   = A_w − A_g                              (totaal profieloppervlak)
l_g   = 2 · [ r·(W − (c+1)·f) + c·(H − (r+1)·f) ]   (totale zichtbare glasrand-omtrek)
```

### Resultaat

```
U_w = (A_g·U_g + A_f·U_f + l_g·Ψ_g) / A_w
```

`a_g_m2`, `a_f_m2`, `l_g_m`, `u_w` worden berekend en **mee-gepersisteerd** in `uw_breakdown`
(gecachet, herberekenbaar). `u_w` wordt tevens naar `element.u_value` geschreven.

### Spacer ψ_g-tabel — NTA 8800 bijlage L (V1-defaults)

Bron: `crates/nta8800-tables/src/glazing_edge/mod.rs`. Inline TS-kopie (4 waarden, klein):

| `Spacer` | ψ_g W/(m·K) |
|---|---|
| `Aluminium` | 0,08 |
| `Stainless` | 0,06 |
| `WarmEdgePolymer` | 0,04 |
| `WarmEdgeFoam` | 0,02 |

Logica: `spacer` gezet & `psi_g_is_manual = false` → `psi_g` = tabelwaarde.
`psi_g_is_manual = true` → gebruiker-ingevoerde `psi_g` wint. `spacer = null` → volledig handmatig.

### Validatie (blokkeert berekening, toon inline fout)

- `W, H, f, u_g, u_f > 0`; `psi_g ≥ 0`; `c, r ≥ 1` (integer)
- `W > (c+1)·f` **én** `H > (r+1)·f` → anders "profiel te breed voor de ruit-indeling"
- `u_g`, `u_f` plausibiliteits-hint bij > 7 W/(m²·K) (geen harde blokkade)

### Worked examples (testankers voor `uwCalculator.test.ts`)

1. **1 ruit** — W=1200, H=1500, f=80, c=1, r=1, U_g=1,1, U_f=1,4, Aluminium (ψ=0,08):
   A_w=1,800 · A_g=1,3936 · A_f=0,4064 · l_g=4,760 → **U_w ≈ 1,379**
2. **2 ruiten naast elkaar** — idem, c=2, r=1:
   A_g=1,2864 · A_f=0,5136 · l_g=7,280 → **U_w ≈ 1,509**

---

## UI — per-kozijn modal vanaf de constructie-rij

Gekozen uit 3 opties (zie Explore-verkenning): `uw_breakdown` zit **per `ConstructionElement`**,
dus de calculator hoort per-element thuis — niet project-breed.

| Onderdeel | Detail |
|---|---|
| Trigger | Knop in `ConstructionRow.tsx`, **alleen** voor kozijnen (`isFrameConstruction()` true). Kozijnen hebben daar nu géén "Lagen"-knop (disabled `ConstructionRow.tsx:277-289`) — die plek is vrij. |
| Component | Nieuw `frontend/src/components/rooms/UwCalculatorModal.tsx` (modal-patroon analoog aan `ConstructionPicker`). |
| Velden | W, H, f, c, r, U_g, U_f, spacer-dropdown (4 + "handmatig"), ψ_g (read-only bij spacer, editbaar bij handmatig). |
| Live output | A_g / A_f / l_g / **U_w** updaten realtime; validatiefouten inline. |
| Opslaan | Schrijft volledige `uw_breakdown` op het element **én** `u_value = u_w`, via dezelfde per-element store-update als de inline U-waarde-edit. |
| Heropenen | Modal pre-fills uit bestaand `uw_breakdown` indien aanwezig. |

### Rekenlogica-bestand

`frontend/src/lib/uwCalculator.ts` — pure functies, geen React/store:
`deriveUwGeometry()`, `resolvePsiG(spacer, manual)`, `computeUw()`, `validateUwInput()`,
`SPACER_PSI_G`-constante. Volledig unit-getest in `uwCalculator.test.ts`.

### Randgeval — interactie met `frameUValueOverride`

`getEffectiveFrameUValue()` (`lib/frameOverride.ts`) geeft de **project-brede** override terug voor
álle kozijnen zodra die actief is — die maskeert de per-element `u_value`. Wanneer de gebruiker de
U_w-calculator opent terwijl een project-override actief is: **toon een waarschuwing** in de modal
("project-brede kozijn-override actief — deze U_w wordt pas gebruikt als de override uitstaat").
Niet auto-uitschakelen; de gebruiker beslist.

### i18n

Nieuwe sleutels in `frontend/src/i18n/locales/{nl,en}/common.json` onder namespace `uwCalculator.*`
(of een eigen `uw.json`-locale-bestand, consistent met bestaande structuur).

---

## Te wijzigen / nieuwe bestanden

| Bestand | Actie |
|---|---|
| `frontend/src/lib/uwCalculator.ts` | **nieuw** — pure rekenlogica + spacer-tabel |
| `frontend/src/lib/uwCalculator.test.ts` | **nieuw** — unit-tests incl. 2 worked examples |
| `frontend/src/components/rooms/UwCalculatorModal.tsx` | **nieuw** — invoer-modal |
| `frontend/src/components/rooms/ConstructionRow.tsx` | wijzig — U_w-knop voor kozijnen |
| `frontend/src/store/projectStore.ts` | wijzig — per-element update schrijft `uw_breakdown` mee |
| `frontend/src/i18n/locales/{nl,en}/common.json` | wijzig — `uwCalculator.*`-labels |

**Niet aanraken:** Rust-crates, JSON-schema, Tauri-commands, `frameOverride.ts`-logica (alleen lezen).

## Acceptatiecriteria

- `uwCalculator.test.ts` groen — beide worked examples ±0,01 W/(m²·K).
- Kozijn-rij toont U_w-knop; niet-kozijn-rijen niet.
- Modal berekent live, valideert, en schrijft bij opslaan `uw_breakdown` + `u_value` naar het element.
- Bestaand `uw_breakdown` herlaadt correct in de modal.
- Waarschuwing zichtbaar bij actieve `frameUValueOverride`.
- `npm run build` + lint schoon; bestaande projecten laden ongewijzigd (Fase 1 was al backward-compatible).

## Fase 3 (preview, niet nu bouwen)

`uw_breakdown` als onderbouwingstabel in de PDF — glas/profiel/rand-opbouw per kozijn,
via `reportBuilder.ts`. Aparte sprint.
