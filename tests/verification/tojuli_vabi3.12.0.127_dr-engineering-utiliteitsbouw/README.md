# TO-juli (NTA 8800 cooling) — Vabi 3.12.0.127 — DR Engineering Utiliteitsbouw

| Veld | Waarde |
|---|---|
| Norm | TO-juli / NTA 8800 cooling |
| Software | Vabi Elements 3.12.0.127 |
| Bron PDF | `tests/references/dr-engineering-koellast-utiliteitsbouw-2024.pdf` (1197 KB) |
| Gebouw | Utiliteitsbouw (kantoor / industrie — checken in PDF) |
| Status `expected.json` | ❌ **nog niet aangemaakt** |

## Open werk

1. **PDF inhoud verkennen** — pagina-structuur + welke ruimtes / zones zijn beschreven
2. `input.json` opbouwen volgens TojuliFullInputs schema (skelet uit Batch 2 NTA 8800 cooling V2-mapping)
3. `expected.json` invullen vanuit PDF
4. Auto-test toevoegen in `crates/nta8800-cooling/tests/`

## Afhankelijkheid

Vereist eerst Batch 2 NTA 8800 cooling V2-mapping (TojuliFullInputs skelet + ColdGeneratorData + emission/distribution + g-waarde + oriëntatie) in vabi-importer crate. Zie sessie 11 handoff "woensdag-prio".

## Doel

Tweede TO-juli fixture naast woningbouw — dekt utiliteitsbouw-pad (andere ventilatie-systemen, andere zone-typen).
