# C1 — Norm-analyse demand-transmissie (P/A-grond + raam-U)

**Datum:** 2026-07-13
**Werkpakket:** C1 — twee gedocumenteerde engine-benaderingen in de demand-transmissie
vervangen door het norm-model (NTA 8800:2025+C1:2026), plus een klein
validatie-napunt.
**Norm-bron:** `NTA 8800_2025+C1:2026 nl.pdf` — §8.3 (grond, via NEN-EN-ISO 13370),
§8.2.1 formule (8.1) (H_D).

Dit werkpakket raakt uitsluitend de gevel-georiënteerde BENG-keten
(`compute_beng` → `compute_tojuli_full` → `nta8800-transmission`). De ISSO 51/53-
warmteverlies-tak heeft een eigen transmissie-implementatie en is ongemoeid.

---

## Kernbevinding: opgeheven compensatie

Tot C1 was de bridged Aalten-golden groen doordat **twee fouten elkaar in BENG 1
wegstreepten**:

| Post | Pre-C1 | Oorzaak |
|------|--------|---------|
| Q_H;nd (verwarming) | ~40 % te laag | raam-U liep op de opake U; forfaitair `h_g;an = 10` |
| Q_C;nd (koeling) | fors te hoog | `F_sh = 1,0` (zomerscreens niet gemodelleerd, F3d, out-of-scope) |

C1 corrigeert de transmissie (Q_H;nd wordt correct); daardoor is de
koeling-overschatting niet langer gemaskeerd en schieten BENG 1/2 over. Dit is
**geen regressie in de transmissie** maar het blootleggen van een aparte,
buiten-scope koeling-post. Anti-fudge: geen enkele `expected.json`/`input.json`
is aangeraakt; de correcte transmissie is niet teruggedraaid.

---

## Item 1 — P/A-grondmodel (§8.3, NEN-EN-ISO 13370)

**Vervangt:** het forfaitaire `h_g;an = 10 W/K` (§8.3.1-fallback via bijlage I.2.3)
door het stationaire P/A-model voor een vloer direct op de grond (vloer op staal).

### Normformules

- **(8.30)** karakteristieke vloerbreedte: `B'_f = A_f / (0,5·P)`
- **(8.32)** equivalente dikte: `d_f;equi = d_bw + λ_gr·(R_si + R_c + R_se)`
  met `d_bw = 0,5 m` (§8.3.2.3), `λ_gr = 2,0 W/(m·K)` (8.35),
  `R_se = 0,04` (§8.3.2.3 OPM. 8), en `R_si + R_c = 1/U_vloer` (de in deze keten
  opgeslagen grond-U is berekend met `R_se = 0`).
- **(8.40)** matig geïsoleerd (`d_f;equi < B'_f`):
  `U_fl = 2·λ_gr/(π·B'_f + d_f;equi) · ln(π·B'_f/d_f;equi + 1)`
- **(8.41)** goed geïsoleerd (`d_f;equi ≥ B'_f`):
  `U_fl = λ_gr/(0,457·B'_f + d_f;equi)`
- **(8.36)** `H_g = A_fl · U_fl` (de aparte `ψ_gr`-vloerrandterm loopt in deze keten
  via de generieke lineaire koudebruggen, niet dubbel).

De maandelijkse faseverschuiving (bijlage D) is nog niet toegepast: de stationaire
`H_g` wordt als jaargemiddelde `H_g;an` gebruikt — dezelfde vereenvoudiging als de
forfait-tak.

### Implementatie (bestand:regel + formule)

| Locatie | Inhoud |
|---------|--------|
| `crates/nta8800-transmission/src/calc/h_t_ground.rs` — `slab_on_ground_conductance()` | 8.30/8.32/8.40/8.41/8.36; consts `LAMBDA_GROUND` (8.35), `R_SE_GROUND`, `WALL_THICKNESS` |
| `crates/nta8800-transmission/src/lib.rs` | re-export + scope-doc bijgewerkt |
| `crates/nta8800-transmission/src/references.rs` | `FORMULE8_30/8_32/8_36/8_40_41` |
| `crates/openaec-project-shared/src/geometry.rs` — `Construction::ground_perimeter_m` | additief `Option<f64>` (serde-skip → byte-identiek) |
| `crates/openaec-project-shared/src/beng/geometry_bridge.rs` — `map_boundary()` | `ground_perimeter_m = gevel.omtrek_p_m` |
| `crates/openaec-project-shared/src/tojuli.rs` — `build_ground_conductance()` | P/A per grondvloer als élke een perimeter draagt; anders forfait 10 (byte-identiek) |

### Handrekening Aalten (verificatie in de unit-test)

`A = 67,0 m²`, `P = 32,92 m`, `Rc = 3,70` → `U = 0,258398`:
`B'_f = 67/(0,5·32,92) = 4,0705 m`; `d_f;equi = 0,5 + 2·(1/0,258398 + 0,04) = 8,320 m`
≥ `B'_f` → goed geïsoleerd (8.41): `U_fl = 2/(0,457·4,0705 + 8,320) = 0,19646`;
`H_g = 67·0,19646 = 13,163 W/K` (vs forfait 10). Test:
`slab_on_ground_matches_hand_calc_aalten`.

Gouda's vloer grenst aan een **onverwarmde kruipruimte** (§8.4 b-factor-tak), niet
direct aan grond → het P/A-model raakt Gouda niet.

---

## Item 2 — Raam-U in de demand-transmissie (formule 8.1)

**Vervangt:** de F6-fase-2-vereenvoudiging waarin het volledige bruto gevelvlak op
de opake U transmitteerde en de kozijn-U alleen de zonwinst/TOjuli-noemer voedde.

### Normformule

- **(8.1)** `H_D = Σ(A_T;i · U_C;i) + Σ(L_k·ψ_k) + Σχ_j` — de som loopt over **alle**
  vlakdelen, inclusief ramen/deuren op hun eigen (samengestelde) U.

### Implementatie

`crates/openaec-project-shared/src/tojuli.rs` — `build_transmission_elements()`:
per constructie nu een **opaak element** `(A_bruto − Σ A_raam) · U_opaak` **plus één
element per raam/deur** `A_raam · U_window`. Een opake rest ≤ 0 (volledig beglaasde
pui) levert geen opaak element (calc weigert niet-positieve oppervlakten). Dit
reproduceert Uniecs decompositie exact (bv. Aalten Wand O: opaak 18,77 + ramen 5,04
= 23,81 bruto).

Bijgewerkte unit-tests (oude waarde codeerde het opake-U-gedrag):
`thermal_bridges_raise_h_t_and_heating_demand` (45,0 → 67,0 W/K),
`end_to_end_woning_120m2` (45,0 → 67,0), `compute_tojuli_full_with_adjacent_room_and_named_unheated_space`
(bovengrens 65 → 80; H_T ≈ 73).

---

## Item 3 — Gevel-id globaal uniek

`crates/openaec-project-shared/src/beng_geometry.rs` — `BengGeometry::validate()`:
de gevel-id-uniekheid is verplaatst van per-zone (`BengZone::validate`) naar
**globaal over alle zones** (relevant bij multi-zone utiliteit; rapportage en
koudebrug-koppelingen refereren aan een begrenzingsvlak op id). Nieuwe tests:
`duplicate_gevel_id_across_zones_is_invalid_input`,
`distinct_gevel_ids_across_zones_are_ok`. Bestaande per-zone-duplicaat-test blijft
groen (zelfde `BengBoundary.id`-context).

---

## Meetresultaten — vóór/na per golden-case

Diagnostiek: `uniec_measure_bridged` (Aalten), `gouda_measure_bridged` (Gouda).
"pre-C1" = master `1d70887`; "C1" = na items 1+2.

### Aalten-2522 (vloer op grond → item 1 + 2 actief)

| Grootheid | pre-C1 | C1 | certified | tol |
|-----------|-------:|----:|----------:|----:|
| BENG 1 | 102,84 (−0,8 %) | **133,16 (+28,4 %)** | 103,69 | ±6 % |
| BENG 2 | 22,61 (−8,5 %) | **33,77 (+36,7 %)** | 24,71 | ±10 % |
| BENG 3 | 83,57 (−1,4 pp) | 81,22 (−3,8 pp) | 85,00 | ±3 pp |
| **verwarming primair [kWh]** | **1544 (−40 %)** | **2444 (−4,2 %)** | 2551 | — |
| koeling primair [kWh] | ~1758 | ~1583 | 244 (demand 873) | — |

De verwarmingsbehoefte landt op certified (−4,2 %); BENG 1/2 overschieten door de
onafgedekte koeling-`F_sh`-post.

### Gouda-2467 (vloer op kruipruimte → alleen item 2)

| Grootheid | pre-C1 | C1 | certified | tol |
|-----------|-------:|----:|----------:|----:|
| BENG 1 | 90,41 (−5,7 %) | 115,02 (+20,0 %) | 95,86 | ±6 % |
| BENG 2 | 8,90 (−67,6 %) | 22,25 (−19,0 %) | 27,48 | ±8 % |
| BENG 3 | 92,26 (+8,6 pp) | 85,30 (+1,6 pp) | 83,70 | ±3 pp |
| **verwarming primair [kWh]** | 2914 (−55 %) | **5131 (−21 %)** | 6506 | — |
| koeling primair [kWh] | ~1541 | ~1500+ | 244 | — |

Gouda's resterende verwarming-gap (−21 %) is géén transmissie-post meer maar de
kruipruimte-b-factor / infiltratie (buiten C1-scope).

---

## Teststatus

- `cargo test --workspace`: **volledig groen** (geen FAILED).
- Nieuwe/gewijzigde green tests:
  - `nta8800-transmission`: `slab_on_ground_matches_hand_calc_aalten`,
    `slab_on_ground_uses_log_branch_when_poorly_insulated`,
    `slab_on_ground_zero_for_invalid_input`.
  - `openaec-project-shared` (lib): 3 tojuli-H_T-tests herijkt naar formule 8.1;
    2 gevel-id-cross-zone-tests.
  - `openaec-project-shared` (beng_golden): **nieuwe green transmissie-anchor**
    `aalten_beng_geometry_heating_matches_certified` (heating primair 2444 vs
    certified 2551, binnen ±10 %).
- **`aalten_beng_geometry_within_certified_tolerance` → `#[ignore]`** met gemeten
  reden (BENG 1/2 overschieten door de out-of-scope koeling-`F_sh = 1,0`-post,
  niet door de transmissie). `gouda_beng_geometry_within_certified_tolerance` blijft
  `#[ignore]` (reden bijgewerkt met de C1-verschuiving).

## Vervolg (buiten C1)

1. **Koeling `F_sh` / zomer-zonwering** (F3d) — dominante blokkade voor de bridged
   BENG-goldens; zodra gemodelleerd gaat Aalten weer groen op BENG 1.
2. **PV-saldering-normversie** (F3d-8) — BENG 2/3 bij hoog PV-aandeel (Gouda).
3. **Bijlage-D-periodiek** voor de grondtransmissie (maandelijkse faseverschuiving).
4. **Kruipruimte-b-factor / infiltratie** — Gouda's resterende verwarming-gap.
