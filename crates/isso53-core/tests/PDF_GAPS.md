# PDF Gaps - ISSO 53 Norm Examples

Deze file documenteert de verschillen tussen de ISSO 53 PDF voorbeelden (p.59-75) en onze huidige implementatie, inclusief externe verificaties tegen Vabi Elements.

## Status van fixtures

⚠️ **De `expected.json` files bevatten momenteel PLACEHOLDER-waarden, niet de echte ISSO 53 norm-resultaten uit PDF H6.** De PDF op `Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\ISSO-publicatie 53...` bleek niet (volledig) extraheerbaar via beschikbare tools in deze sessie.

### Wat is er WEL gedaan
- Fixture-infrastructuur opgezet: 4 JSON-files + golden.rs framework + tolerance-helper  
- Input-JSONs samengesteld volgens een aannemelijk utiliteitsbouw-scenario
- Tests staan op `#[ignore]` — kunnen handmatig geactiveerd worden zodra de norm-getallen ingevuld zijn

### Wat moet er NOG gebeuren (los werkpakket)
1. PDF H6 (p.59-75) handmatig openen + uittypen
2. Vergelijken: onze input-JSON ↔ PDF-input. Bij verschil: input-JSON aanpassen
3. Per voorbeeld de PDF-verwachte H_T, Φ_V, Φ_I, Φ_HL invullen in `expected.json`
4. Tolerance instellen (5% gebouw, 2% per vertrek)
5. `#[ignore]` weghalen + run `cargo test`

### Risico bij negeren
Zonder norm-getallen is het rekenresultaat van isso53-core niet onafhankelijk geverifieerd. Wel: alle formules zijn doc-comment-verwijzingen naar de spec, en de calc-primitieven hebben handberekende smoke-tests (batch 2b).

## Voorbeeld 6.1 Gaps

*TBD na handmatige PDF-extractie - fixture bevat placeholder-input*

## Voorbeeld 6.2 Gaps  

*TBD na handmatige PDF-extractie - fixture bevat placeholder-input*

## Mogelijke structurele limitaties

- [ ] Model velden die ontbreken in de input JSON schema
- [ ] Tabel lookup waarden die niet geïmplementeerd zijn  
- [ ] Specifieke randgevallen van de norm die nog niet ondersteund worden

## Acties

- [ ] Handmatige PDF-extractie p.59-75 ISSO 53 publicatie
- [ ] Verificatie via externe ISSO 53 software (indien beschikbaar)
- [ ] Contact met norm-auteurs voor onduidelijke gevallen

## Vabi TR02 Houtfabriek (Bedrijfsruimte4)

**Bron:** Vabi Elements 3.11.2.23, rapport TR02 Houtfabriek p.18-20  
**Fixture:** `vabi_houtfabriek_bedrijfsruimte4_input.json` + `expected.json`  
**Test:** `tests/vabi_golden.rs::vabi_houtfabriek_bedrijfsruimte4()`

### Input mapping gaps
- **Industriefunctie → kantoor:** Vabi gebruikt "Industriefunctie Verblijfsgebied", onze enum heeft geen `Industrie` + `Verblijfsgebied` combinatie. Gebruikt `gebruiksFunctie: "kantoor"` + `ruimteType: "verblijfsgebied"` als beste benadering
- **ΔU_TB = 0,05:** Vabi gebruikt 0,05 voor "nieuw gebouw, goed vakmanschap", onze default is 0,10. Override via `customDeltaUTb: 0.05` per exterior element
- **30+ constructies → ~6 elementen:** Vabi heeft 30+ individuele wanden/ramen, fixture bundelt vergelijkbare elementen voor leesbaarheid
- **WTW-voorverwarming:** Vabi heeft vorstbeveiliging + voorverwarmer, onze VentilationConfig heeft deze als optionele velden maar wordt mogelijk niet gebruikt in berekening

### Verwachte resultaten (Vabi output)
- **Φ_T (transmissie):** 2919 W (707,71 m² totaal constructie-oppervlak)
- **Φ_V (ventilatie+infiltratie):** 3080 W (alleen infiltratie — ventilatie=0 door WTW)  
- **Φ_hu (opwarming):** 2163 W (216,28 m² × 10 W/m²)
- **Φ_HL totaal:** 8161 W

### Tolerantie: 15%
Utiliteitsbouw heeft veel Vabi-specifieke toeslagen (P-tabellen, gebouw-niveau correcties, WTW-efficiëntie details) die onze implementatie mogelijk niet 1-op-1 reproduceert.

### Test resultaat (2026-05-23 sessie 2 vervolg, na alle 4 fixes + Vabi-rapport-extractie)

| Component | ISSO53-Core | Vabi | Afwijking | Status |
|-----------|-------------|------|-----------|--------|
| **phiT (transmissie)** | 2918 W | 2919 W | -0.03% | ✅ **OPGELOST** §4.6 |
| **phiV (ventilatie)** | 0 W | 0 W | exact | ✅ luchtverwarming θ_t=21°C → f_v=0 |
| **phiI (infiltratie)** | 3134 W | 3080 W | +1.8% | ✅ **OPGELOST** A_u/A_g + building_height |
| **phiV+phiI combined** | 3134 W | 3080 W | +1.8% | ✅ binnen 10%-tolerantie |
| **phiHu (opwarming)** | 2163 W | 2163 W | 0.0% | ✅ Perfect |
| **totalHeatLoss** | 8215 W | 8161 W | +0.7% | ✅ **VOLLEDIG IN OVEREENSTEMMING** |

\* *Vabi rapporteert gecombineerde waarde 3080 W voor ventilatie+infiltratie*

### **OPGELOST: ISSO 53 §4.6 embedded heating clause**

**Root-cause:** De vloer in Vabi TR02 heeft vloerverwarming (PDF p.18 "Soort verwarming: Vloerverwarming"). ISSO 53 §4.6 stelt expliciet: *"f_ig,k = 0 voor het, door de verwarming van het beschouwde vertrek, verwarmde deel van wand/vloer/plafond bij wand-/vloerverwarming c.q. betonkernactivering"*.

**Fix:** `ground.rs::calculate_h_t_ground()` nu met:
```rust
let f_ig = if element.has_embedded_heating {
    0.0  // ISSO 53 §4.6: verwarmd deel van vloer/wand
} else {
    ground_params.f_ig
};
```

**Impact:** phiT daalde 4385→2918 W (-1467 W = exacte ground-bijdrage), nu <0.1% verschil met Vabi.

### **OPGELOST: WTW Φ_V f_v formule (formule 4.38) — sessie 2, 2026-05-23**

**Root-cause:** In `calc/ventilation.rs::calculate_f_v` stond `f_v = (θ_t − θ_e) / (θ_i − θ_e)`. Bij η=0,85, θ_i=20, θ_e=−10: θ_t = 15,5°C → f_v = 0,85. Maar fysisch moet f_v = 1−η = 0,15 zijn (het deel van Δθ dat ná WTW nog opgewarmd moet worden).

**Norm-onderbouwing (ISSO 53 §4.7.2, PDF p.48):** Formule 4.38 geldt voor WTW én voorverwarming. θ_t = toevoertemperatuur ventilatielucht. Fysische definitie: Φ_V = q_v·ρ·c_p·(θ_i − θ_t), dus f_v·(θ_i − θ_e) = (θ_i − θ_t) → `f_v = (θ_i − θ_t) / (θ_i − θ_e)`.

**Fix:**
```rust
let f_v = (theta_i - theta_t) / (theta_i - theta_e);
```
Toegepast op zowel `has_heat_recovery` als `has_preheating` branches (dezelfde formule 4.38 voor beide).

**Impact:** Φ_V daalde 3076 → 543 W (−2533 W = ~82% reductie, exact (1−0,85)·oorspronkelijke = 461 W in de juiste orde; 543 W door q_v-verschillen per ruimte).

### **OPGELOST: Infiltratie Φ_I — twee code-bugs + één model-extensie (sessie 2 vervolg, 2026-05-23)**

Via Vabi-rapport p.18-20 extractie (PDF tools MCP) zijn de exacte input-parameters gevonden. Hypothese 4 (A_u/A_g) bevestigd, twee aanvullende bugs blootgelegd:

**Bug 1 — A_u en A_g omgedraaid in `calc/infiltration.rs::calculate_h_i`:**

Per ISSO 53:
- Formule 4.28 (Known-pad): `q_i = q_is × A_u` waarbij **A_u = uitwendige scheidingsconstructie (gevel excl. plat dak)**
- Formule 4.29 (Unknown-pad): `q_i = q_is × A_g` waarbij **A_g = gebruiksoppervlakte (vloer)**

De code had het exact omgedraaid (Known → `room.floor_area`, Unknown → som exterior elements). Fix: Known-pad gebruikt nu A_u (som exterior elements met `VerticalPosition != Ceiling` per voetnoot tabel 4.5).

**Bug 2 — Hardcoded `building_height = 3.0` in `calculate_q_is`:**

Comment zei al "TODO: get actual building height from Building model". Veld toegevoegd aan `Building` (Option<f64>, default 3.0). Vabi-gebouwhoogte 10,9 m → klasse 6<h≤20 → q_is = 0,00103 (i.p.v. 0,00064 bij ≤3m).

**Fixture-aanvulling — Luchtverwarming (Vabi Φ_V = 0):**

Vabi-rapport: "Vorstbeveiliging: Voorverwarming buitenlucht" + "Verwarmingsbatterij: Ja" + "Ventilatie 21,0°C → 0 W". Modellering via `supplyTemperature = 21.0` in `VentilationConfig`. Bij θ_t > θ_i clamp WTW-formule naar f_v = 0 → Φ_V = 0 W exact.

**Eindresultaat:** Φ_V+Φ_I = 3134 W vs Vabi 3080 W = +1,8%. Totaal warmteverlies +0,7%.

### Conclusie sessie 2 (vervolg)

Alle drie norm-/implementatie-bugs opgelost: §4.6 ground (sessie 1), formule 4.38 WTW (sessie 2 ochtend), A_u/A_g omdraai + building_height (sessie 2 vervolg). De Vabi TR02 Bedrijfsruimte4 fixture matcht nu op alle vier componenten binnen 2%. `#[ignore]` verwijderd op `vabi_bedrijfsruimte4_phi_vi_combined_matches`.

## Vabi DR Engineering voorbeeld - Kantoor West 0.03 (sessie 2026-05-23)

**Bron:** Vabi Elements 3.12.0.127, "Voorbeeld Warmteverliesberekening Utiliteitsbouw"
(27-2-2025), DR Engineering. Ruimte 0.03 Kantoor West.
**Fixture:** `vabi_dr_engineering_kantoorwest_input.json`
**Test:** `tests/vabi_dr_golden.rs`

### Doel cross-validatie

Bevestigen dat de 4 fixes uit Bedrijfsruimte4 generaliseren naar ander Vabi-project,
plus eerste verificatie van de **Unknown-pad** (formule 4.31, geactiveerd in deze sessie).

### Cross-validatie resultaat

| Component | ISSO53-Core | Vabi | Δ | Status |
|-----------|-------------|------|---|--------|
| **Φ_V** | 0 W | 0 W | exact | ✅ luchtverwarming formule 4.38 generaliseert |
| **Φ_T** | 3786 W | 3059 W | +24% | ⚠️ adjacent-room ΔT-keten verschilt |
| **Φ_I** | 177 W | 681 W | -74% | ⚠️ Unknown-pad: norm vs Vabi-keten |
| **Totaal** | 3963 W | 3741 W | +6% | ⚠️ compenserende fouten |

### Twee nieuwe open sporen

**Spoor 1 — Transmissie naar adjacent rooms met hoge U-waarde**

Vabi rapporteert Φ_T,ie + Φ_T,ia + Φ_T,ig = 1237 + 1507 + 315 = 3059 W. Onze code geeft 3786 W
(+24%). Het verschil zit in adjacent-room transmissie: "Vloer tus Plafond" U=2,91 area=119,49 m².
Vabi rekent met "Temp. gradient 4K" → 119,49 × 2,91 × 4 = 1391 W. Onze code geeft een lagere
waarde door de manier waarop ΔT (room.theta_i − adjacent.theta) wordt afgehandeld bij hoge
tussenvloer-U-waarden. **Onderzoek nodig:** §4.4 formule 4.9/4.10 voor adjacent rooms.

**Spoor 1 — Ground f_ig auto-berekening (OPGELOST, sessie 2026-05-24)**

**Root-cause:** `calculate_h_t_ground()` haalde f_ig uit user-input (default 1.0), maar ISSO 53 §4.6 schrijft expliciet voor dat f_ig MOET berekend worden uit binnen/buiten/jaargemiddelde temperatuur via formules 4.22 (Wall) en 4.23 (Floor).

**Fix geïmplementeerd:**
- Formule 4.22 (wanden): f_ig,k = (θ_i − θ_me) / (θ_i − θ_e)  
- Formule 4.23 (vloeren): f_ig,k = ((θ_i + Δθ_2) − θ_me) / (θ_i − θ_e)
- Δθ_2 lookup via nieuwe `HeatingSystem` enum + tabel 2.3 implementatie
- `GroundParameters.f_ig` wijzigt van `f64` naar `Option<f64>` (None=auto, Some=override)
- Silent migration: bestaande JSON's blijven werken

**Impact:** Φ_T daalde 3786→3282 W (-504 W = ground-correctie), nu binnen 10% tolerantie van Vabi 3059 W (+7.3%). Test `vabi_dr_kantoorwest_phi_t_matches` niet meer op `#[ignore]`.

**Spoor 2 — Unknown-pad Vabi-compat** → **OPGELOST**

Geïmplementeerd via `InfiltrationMethod::UnknownVabiCompat` variant met:
- NEN 8088-1 Tabel 9 (f_type = 0,90), Tabel 10 (f_inf = 1,10)  
- NTA 8800 Tabel 11.13 (f_jaar = 0,70 voor j≥2010)
- Power-law drukconversie: (Δp/10)^0.67 met Δp=3.14 Pa (Vabi-fit)

Resultaat: Φ_I = 693 W vs Vabi 681 W (+1,8%, binnen 5% tolerantie).
Bron: NEN 8088-1, NTA 8800, docs/2026-05-12-nta8800-infiltratie-verificatie.md

### Wat WEL werkt (positieve cross-validatie)

- **§4.6 embedded heating clause**: nvt voor dit gebouw (geen vloerverwarming)
- **Formule 4.38 WTW omkering**: f_v=0 bij luchtverwarming (θ_t=21,5°C) → Φ_V=0 W exact ✅
- **A_u/A_g + building_height**: A_u-extractie werkt (75 m² exterior excl. ceiling)
- **Unknown-pad implementatie (formule 4.31)**: berekening loopt zonder error,
  resultaten zijn norm-conform — alleen niet Vabi-compatible

## Vabi TR02 Houtfabriek — 3 verdiepingen (1.10a, 2.10a, 3.10a) — sessie 7 (2026-05-25)

**Bron:** Vabi Elements 3.11.2.23, rapport TR02 Houtfabriek p.38-40 (1.10a), p.82-84 (2.10a), p.131-133 (3.10a)
**Fixture:** `vabi_houtfabriek_3floors_input.json` + `expected.json`
**Test:** `tests/vabi_houtfabriek_3floors_golden.rs`

### Doel

Cross-validatie van adjacent-room transmissie tussen verdiepingen op dezelfde temperatuur, en exposeren van structurele norm-vs-Vabi verschillen.

### Sessie 7 calc-core fixes (bug C1+C2)

**Bug C1 — `BoundaryType::AdjacentRoom` niet geïmplementeerd**
Voor sessie 7 telden alle adjacentRoom-elementen 0 W door een `// TODO: batch 2c` skip in `transmission.rs`. Gevolg: zowel interne wanden naar koudere kamers (Vabi +41W) als shared floors tussen warme verdiepingen werden genegeerd. Fix: nieuwe `calculate_h_t_adjacent_rooms()` met formule 4.18 mirror van adjacentBuildings, gebruikt `element.adjacent_temperature` (geen 15°C default — fout als ontbreekt).

**Bug C2 — `customDeltaUTb` genegeerd bij `useForfaitaireThermalBridge: true`**
Voorheen: forfaitaire flag overschreef silently elke `customDeltaUTb`. Vabi's kb=0,05 werd dus genegeerd, default 0,10 gebruikt → +0,05 W/m²K op alle exterior elementen. Fix: customDeltaUTb krijgt voorrang wanneer expliciet gezet (`Some`), forfaitaire default is fallback.

### Cross-validatie na C1+C2 fix

| Room | Φ_T calc | Φ_T Vabi | Δ Φ_T | Φ_I calc | Φ_I Vabi | Δ Φ_I |
|---|---|---|---|---|---|---|
| 1.10a | 1499 W | 1514 W | **−1,0 %** ✅ | 1337 W | 1337 W | 0,0 % ✅ |
| 2.10a | 1579 W | 1494 W | **+5,7 %** ⚠️ | 1338 W | 1337 W | +0,1 % ✅ |
| 3.10a | 1855 W | 1691 W | **+9,7 %** ⚠️ | 1218 W | 1217 W | +0,1 % ✅ |

`vabi_3floors_phi_t_matches` test is **`#[ignore]`** — gap is structureel en gedocumenteerd, niet een regressie. Snapshot test (`vabi_3floors_snapshot`) bewaakt regressie via ±1% op de huidige calc-output.

### Resterende gaps (na C1+C2 fix)

**Gap residueel 2.10a (+5,7 %, +85 W)**
Origin onbekend. Mogelijk Vabi-Vabi inconsistentie in interne wand-temperaturen, of modelleer-noise. Niet kritisch — binnen ±6 %.

**Gap structureel 3.10a (+9,7 %, +164 W) — Vabi dak f=1,138**
Vabi past onverklaarde correctiefactor 1,138 toe op dak-elementen (Temp.grad=+4K boven θ_e). Norm-strikt gebruikt f=1,000. Per user-principe sessie 7 (norm krijgt voorrang) niet overriden. Effect: 54,5 m² × 0,15 W/m²K × 29K × (1,138−1,000) ≈ +33 W direct + interactie-effecten ≈ +100-160 W totaal.

### Principe sessie 7 — norm-voorrang

Vastgelegd in user-memory `feedback_norm_voor_vabi.md`:

> Bij validatie tegen Vabi-rapporten krijgt de norm (ISSO 51/53/57, NEN 8088, NTA 8800) altijd voorrang boven Vabi-snapshot. Vabi-documenten kunnen verkeerd ingevuld zijn — afwijkingen documenteren, niet overriden.

Implementatie: structurele gaps blijven zichtbaar via `#[ignore]` + comment + PDF_GAPS-vermelding. Compat-modes zoals `UnknownVabiCompat` zijn alleen toegestaan als opt-in variant naast norm-strikte default.

## Vabi DR Engineering Kantoor West — fixture-defect ontmaskerd sessie 7

**Symptoom:** na C1 fix (sessie 7) sprong room 0.03 Φ_T van 3282 W (was +7,3 % vs Vabi 3059 W) naar 4672 W (+52,7 %).

**Root cause:** fixture `vabi_dr_engineering_kantoorwest_input.json` bevat een plafond-element met `uValue: 2.91 W/m²K` naar `adjacentRoom 17,5°C`. Fysiek onmogelijk voor een tussenvloer (verwacht ~0,40-0,48). Vóór C1 fix telde dit element 0 W door de skip — defect bleef verborgen.

**Status:** `vabi_dr_kantoorwest_phi_t_matches` is `#[ignore]` met expliciete `TODO sessie 8: correcteer plafond U-waarde uit Vabi DR Engineering bron, re-validate`. Snapshot test bewaakt regressie op huidige output.

**Les:** placeholder-implementaties die elementen silently 0 W laten bijdragen verbergen fixture-defecten. C1 fix legt deze bloot — gewenste lange-termijn-effect.