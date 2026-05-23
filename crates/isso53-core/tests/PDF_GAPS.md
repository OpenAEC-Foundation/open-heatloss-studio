# PDF Gaps - ISSO 53 Norm Examples

Deze file documenteert de verschillen tussen de ISSO 53 PDF voorbeelden (p.59-75) en onze huidige implementatie.

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