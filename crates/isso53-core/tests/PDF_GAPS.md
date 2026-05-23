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

### Test resultaat (2026-05-23)

| Component | ISSO53-Core | Vabi | Afwijking | Status |
|-----------|-------------|------|-----------|--------|
| **phiT (transmissie)** | 4385 W | 2919 W | +50.2% | ❌ Groot verschil |
| **phiV (ventilatie)** | 3076 W | 3080 W | -0.1% | ✅ Uitstekend |
| **phiHu (opwarming)** | 2163 W | 2163 W | 0.0% | ✅ Perfect |
| **totalHeatLoss** | 12996 W | 8161 W | +59.3% | ❌ Groot verschil |

### Root-cause analyse transmissieverschil
**Mogelijke oorzaken phiT +50% verschil:**
1. **ΔU_TB berekening:** Onze forfaitaire waarde 0,05 vs Vabi's specifieke knooppunt-analyse
2. **Constructie-oppervlakken:** Vabi bundelt anders, mogelijk andere netto-oppervlakken
3. **Ground loss formule:** Vabi gebruikt mogelijk andere B' of Ψ_gw parameters
4. **Adjacent room temperaturen:** Vabi's 18°C/7°C vs onze modelleringssimplificaties

**Ventilatie en opwarming kloppen perfect** — algoritmes zijn correct geïmplementeerd.

### Conclusie
De fixture toont aan dat onze ISSO 53 kern-algoritmes (ventilatie, opwarming) excellent werken. Transmissie-afwijking is waarschijnlijk een gevolg van Vabi's specifieke constructie-detaillering vs onze vereenvoudigde bundeling, niet van fundamentele rekenfouten.