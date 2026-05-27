# TO-juli — Vabi 3.12.0.127 — DR Engineering Woningbouw (placeholder)

| Veld | Waarde |
|---|---|
| Norm | TO-juli / NTA 8800 H.10 (annual cooling demand) |
| Software | Vabi Elements 3.12.0.127 |
| Bron PDF | ⏳ wacht op aanvraag |
| Status `expected.json` | 🟡 PLACEHOLDER — nog niet aanwezig |
| Status Rust-test | n.v.t. (nog geen include_str! verwijzing — engine + fixture wachten op PDF) |

## Status (2026-05-27)

🟡 **PLACEHOLDER folder** — wacht op een Vabi BENG / TO-juli output-rapport voor een woning.

De eerder hier aanwezige content beschreef een Vabi Koellast-rapport (peak W), niet een TO-juli/BENG-rapport (annual MJ). Die content is verplaatst naar `koellast_vabi3.12.0.127_dr-engineering-woningbouw/`. Deze folder is opnieuw aangemaakt als plek voor de échte TO-juli ground-truth.

## Wat moet de aanvraag-PDF bevatten

| Indicator | Eenheid | Doel |
|---|---|---|
| `Q_C;nd` | MJ per maand (12 waardes) | Maandelijkse koudebehoefte |
| `Q_C;use` | MJ jaar + per maand | Koel-energie na COP (BENG-2 input) |
| `Q_H;nd` | MJ per maand | Warmtebehoefte (bijproduct, sanity check) |
| `H_T` | W/K | Transmissie-warmteoverdrachtscoëfficiënt |
| `H_V` | W/K | Ventilatie-warmteoverdrachtscoëfficiënt |
| `τ` | uur | Gebouw-tijdconstante |
| TO-juli waarde | — | BENG-2 toets, ≤ 1,20 vereist |
| Klimaat | — | De Bilt referentiejaar (alle uren) |

## Bron-opties (in volgorde van voorkeur)

1. **Vabi Elements BENG output-module** voor dezelfde 120-200 m² woning — exporteert Q_C;nd, Q_H;nd, H_T, H_V, τ maandelijks.
2. **GPR-Gebouw export** van een NTA 8800-conform model.
3. **Uniec voorbeeldproject** (XML/CSV-export van EP-online registratie).
4. **Nieman BENG-publicaties** (tussenwoning, hoekwoning, appartement) — Lente-akkoord 2020 brochure.

## Files

In te vullen wanneer aanvraag-PDF binnenkomt:

- `input.json` — heatloss-studio project gereconstrueerd uit de PDF
- `expected.json` — Vabi-output KPIs + maandelijkse arrays
- `reference.pdf` — bronrapport
- README (dit bestand) bijwerken naar status ✅

## Verbinding met Rust-tests

Wanneer ground-truth beschikbaar is:
- Nieuwe Rust-test `crates/nta8800-cooling/tests/vabi_tojuli_golden.rs` aanmaken (parallel aan de bestaande `vabi_koellast_golden.rs`)
- `include_str!("../../../tests/verification/tojuli_vabi3.12.0.127_dr-engineering-woningbouw/{input,expected}.json")`
- Tolerantie: 10% per KPI, 15% maandelijks (zie eerder placeholder-schema voor referentie)
