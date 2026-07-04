# nta8800-core

Unified façade over de NTA 8800:2025+C1:2026 reken-crates — één
JSON-in/JSON-uit entry-point die de volledige energieprestatie-keten
orkestreert, naar het `isso51-core`/`isso53-core` API-patroon.

```text
Project (JSON)
  → transmissie (H.8) + ventilatie (H.11, §11.2.2 q_V;ODA;req-forfait)
  → warmte-/koudebehoefte (H.7)
  → verwarming (H.9) + koeling (H.10) + tapwater (H.13)
    + verlichting (H.14, utiliteit) + PV (H.16)
  → EP-score + energielabel (H.5) + BENG 1/2/3
Nta8800Result (JSON)
```

## API

```rust
// JSON in → JSON uit
let result_json = nta8800_core::calculate_from_json(&project_json)?;

// Typed
let result: Nta8800Result = nta8800_core::calculate(&project)?;

// JSON-schema van het invoer-model (schemars)
let schema = nta8800_core::project_schema();
```

## Voorbeeldberekeningen

20 geverifieerde fixtures in `tests/fixtures/` — 10 woningbouw + 10
utiliteit, samen dekkend: HR-ketel (100/104/107), lucht-/bodem-warmtepomp,
elektrische weerstand, stadswarmte, ventilatie-systemen A/C/D+WTW,
compressie-/absorptie-/vrije koeling, PV, douche-WTW en alle
tabel-11.8-gebruiksfuncties.

```bash
cargo run -p nta8800-core --example run_all
```

| fixture | label | BENG1 | BENG2 | BENG3 | pass |
|---|---|---|---|---|---|
| w05-nieuwbouw-wp-bodem-pv | A++++ | 44.8 | 15.5 | 100% | +++ |
| w04-hoekwoning-wp-lucht-wtw | A++++ | 40.9 | 22.6 | 0% | ++− |
| w01-tussenwoning-gas-hr107 | B | 79.1 | 104.4 | 0% | −−− |
| w02-vrijstaand-oud-gas-hr100 | F | 195.7 | 260.3 | 0% | −−− |
| u02-kantoor-groot-pv-koeling | A++++ | 58.1 | 42.6 | 65% | −−+ |

(volledige tabel via het example; pass = BENG 1/2/3 t.o.v. indicatieve
nieuwbouw-grenzen)

## Invoer-model (verkort)

```json
{
  "info": { "name": "Tussenwoning" },
  "building": {
    "usage_function": "woonfunctie",
    "floor_area_m2": 110.0,
    "thermal_mass": "heavy"
  },
  "envelope": [
    {
      "id": "gevel-zuid", "area_m2": 40.0, "u_value": 0.30,
      "boundary": { "type": "exterior" },
      "orientation_deg": 180.0, "tilt_deg": 90.0,
      "windows": [
        { "id": "raam", "area_m2": 8.0, "u_value": 1.4, "g_value": 0.6 }
      ]
    }
  ],
  "ventilation": { "system": { "type": "balanced", "wtw_efficiency": 0.85 } },
  "heating": {
    "emission": { "type": "floor_heating" },
    "generation": { "type": "heat_pump", "scop": 4.0 }
  },
  "dhw": { "generation": { "type": "heat_pump_dhw", "scop_dhw": 2.5 } },
  "pv": [ { "peak_power_kwp": 6.0, "tilt_degrees": 35.0,
            "azimuth_degrees": 180.0, "system_efficiency": 0.85,
            "inverter_efficiency": 0.96 } ]
}
```

Zie `examples/minimal.json` voor een compleet werkend voorbeeld en
`nta8800_core::project_schema()` voor het volledige JSON-schema.

## V1 scope

- Eén rekenzone voor het hele gebouw (multi-zone → V2)
- De Bilt referentie-klimaat (NEN 5060 via `nta8800-tables`)
- Ontbrekende luchtdebieten vallen terug op het §11.2.2
  `q_V;ODA;req`-norm-forfait
- Verlichting alleen utiliteit (H.14 kent geen woonfunctie-forfait)
- Gebouwautomatisering (H.15) + bevochtiging buiten de EP-optelling (V2)
- BENG-grenzen zijn indicatief (vast per functie); de formele
  compactheids-correctie op BENG 1 is V2
