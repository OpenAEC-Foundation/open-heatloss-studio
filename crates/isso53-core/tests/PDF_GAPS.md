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

### Test resultaat (2026-05-23 sessie 2, na WTW f_v fix)

| Component | ISSO53-Core | Vabi | Afwijking | Status |
|-----------|-------------|------|-----------|--------|
| **phiT (transmissie)** | 2918 W | 2919 W | -0.03% | ✅ **OPGELOST** §4.6 |
| **phiV (ventilatie)** | 543 W | \* | \* | ✅ **OPGELOST** formule 4.38 |
| **phiI (infiltratie)** | 3372 W | \* | \* | ❌ **GAP 2 BLIJFT OPEN** |
| **phiV+phiI combined** | 3915 W | 3080 W | +27% | ⚠️ Verbeterd van +109%, nog buiten 10% |
| **phiHu (opwarming)** | 2163 W | 2163 W | 0.0% | ✅ Perfect |
| **totalHeatLoss** | 8996 W | 8161 W | +10% | ⚠️ Door resterende gap 2 |

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

### **GAP 2 BLIJFT OPEN: Infiltratie Φ_I — root-cause NIET in formule 4.27/4.28**

**Wat NIET de fix is (onderzocht sessie 2, 2026-05-23 via PDF p.44-47):**

f_inf (tabel 4.7, ventilation_system.rs) hoort **alleen** in het Unknown-pad (formule 4.31). Voor Known-pad (formule 4.28 + tabel 4.5) noemt de norm f_inf niet — q_is volgt direct uit q_v10,kar-klasse × gebouwhoogte. Een vroeg voorstel om f_inf in Known-pad toe te passen is verworpen: bij systeem D is f_inf = 1,15 (verhoging), wat de gap zou verergeren i.p.v. oplossen.

**Hypothesen voor de werkelijke 27%-overschatting (toekomstig onderzoek):**

1. **z-factor in fixture klopt niet.** Tabel 4.4 zegt z = 1 voor één buitengevel; z = 0,5 voor twee tegenover elkaar liggende buitengevels. Bedrijfsruimte4 layout uit Vabi-rapport opnieuw narekenen.
2. **q_v10,kar-klasse fixture te hoog.** Fixture gebruikt `From040To060` — Vabi gebruikt mogelijk een lagere klasse voor "nieuw goed vakmanschap".
3. **Vabi gebruikt Unknown-pad (formule 4.31).** In dat geval is q_is = f_wind · f_type · f_inf · (0,23 · q_i,spec) met systeem D's f_inf = 1,15 — maar f_type voor "Gebouwen met meer lagen standaard" = 0,51 (tabel 4.6), wat het netto verlaagt.
4. **Vabi past A_g (gebruiksopp.) toe i.p.v. A_u (uitwendige scheidingsoppervlak)** — formule 4.28 zegt A_u, ons fixture gebruikt floor_area. Mogelijk verschil bij hoge ruimtes.

**Acceptatie open laten:** snapshot phiI = 3372 W vastgeklikt; `#[ignore]` op `phi_vi_combined_matches` blijft staan met motivatie "+27%, infiltratie-gap blijft open".

### Conclusie sessie 2

Twee van drie norm-bugs opgelost (§4.6 ground + formule 4.38 ventilation). De resterende infiltratie-gap (+27%) ligt niet in onze formule maar mogelijk in fixture-aannames of Vabi's keuze voor Unknown-pad. Vereist herinterpretatie van Vabi-rapport pagina's voor exacte input-replica, NIET een code-fix.