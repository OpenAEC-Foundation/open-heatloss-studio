# C2 — Norm-analyse koudebalans (koel-setpoint + §7.2.2-poort)

**Datum:** 2026-07-13
**Werkpakket:** C2 — de koudebehoefte `Q_C;nd` van de gevel-georiënteerde BENG-keten
(`compute_beng` → `compute_tojuli_full` → `nta8800-demand`) norm-conform maken.
**Norm-bron:** `NTA 8800:2025+C1:2026 nl.pdf` — §7.2.2 (koudebehoefte, formules
7.6/7.7), §7.2.3 (7.12/7.13 winst, 7.14/7.15/7.16 warmteoverdracht), §7.8.3
(η_C;ht, 7.52/7.55), §7.9.4 (setpoint, tabel 7.13), §7.9.3 + tabel 7.15 (a_C;red).

Dit werkpakket raakt uitsluitend de BENG/TOjuli-tak (`nta8800-demand`-koudebalans).
De ISSO 51/53-warmteverlies-tak (`isso51-core`/`isso53-core`) heeft geen koudebalans
en gebruikt deze keten niet — ongemoeid.

---

## Kernbevinding: de "F_sh = 1,0"-hypothese was fout — het is de koudebalans zelf

C1 liet Q_C;nd Aalten op ~2847 kWh staan vs certified 873 kWh; de werkhypothese in
de `#[ignore]`-reden was ontbrekende zomerzonwering (`F_sh = 1,0`). **De capture
weerlegt dat:** de certified Uniec-case draagt voor élk kozijn `ZONW_GEEN` (geen
zonwering) en `ZOMERNVENT_NAANW` (geen zomernachtventilatie) — zie
`aalten-2522/uniec_fields_capture_retry2.json` (`CONSTRT_ZONW = ZONW_GEEN` ×5) en de
`beng_geometry.input.json`-`_note`s. De certified tool rekent dus óók zonder
zonwering en komt tóch op 873 kWh. De gap zit niet in de zonwinst-invoer maar in de
**koudebalans-formule** zelf. Drie afwijkingen van NTA 8800 §7.2.2 gevonden:

| # | Afwijking | Norm | Status vóór C2 |
|---|-----------|------|----------------|
| 1 | `Q_C;ht` rekende tegen de **verwarmings**-setpoint (20 °C) | §7.3.2 form. (7.15): koeling tegen θ_int;set;C = **24 °C** (tabel 7.13) | fout |
| 2 | Geen **poort**: verlies-gedomineerde maanden hielden een residu-`Q_C;nd` | §7.2.2 form. (7.6): `(1/γ_C) > 2,0 → Q_C;nd = 0` | ontbrak |
| 3 | `a_C;red` (niet-continu koelen) | §7.9.3 + tabel 7.15: **woonfunctie `t_C;red;wknd = 0` → a_C;red = 1,0** | correct (geen effect voor woningen) |

Afwijking 3 blijkt géén lever voor woningen (tabel 7.15: woonfunctie heeft geen
weekend-onderbreking voor koeling). De η_C;ht-formule (7.52) is al correct
geïmplementeerd (`utilization_cooling`, γ^(−a)-vorm; a_C;0 = 1,0, τ_C;0 = 15 h — idem
verwarming). Blijven over: **afwijking 1 (setpoint) + 2 (poort)** — dat is de C2-scope.

---

## Gekwantificeerde gap-decompositie (bridged Aalten)

Per-maand-decompositie van de koudebalans (instrumentatie op `calc/mod.rs`, De Bilt-
klimaat, H_tr = 81,4 W/K, τ → a = 1,569). Q_C;nd in kWh/jr, certified = 873.

| Variant | Q_C;nd | Δ t.o.v. huidig | Mechanisme |
|---------|-------:|----------------:|------------|
| Huidig (20 °C, geen poort) | **2808** | — | fout |
| + setpoint 24 °C (afw. 1) | 1869 | −33 % | `Q_C;ht` groter → meer verlies verrekend |
| + poort alleen (afw. 2) | 2567 | −9 % | schouder-maanden `1/γ_C > 2` → 0 |
| **+ beide (C2)** | **1609** | **−43 %** | — |
| certified | 873 | | |

De twee mechanismen samen halen ~57 % van de gap weg. De **rest (1609 → 873)** is
niet de koudebalans-formule maar **thermische massa** (zie hieronder) — een aparte,
aan de verwarming gekoppelde invoer buiten C2-scope.

---

## Implementatie (bestand:regel + formule)

| Locatie | Inhoud |
|---------|--------|
| `crates/nta8800-demand/src/calc/monthly_balance.rs` — `cooling_demand_gated()` | §7.2.2 form. (7.6)+(7.7): poort `Q_C;ht/Q_C;gn > 2 → 0`, anders `Q_C;gn − η_C;ht·Q_C;ht` |
| `crates/nta8800-demand/src/calc/mod.rs` — `calculate_demand_with_cooling_ht()` | nieuwe entry met optionele `Q_C;ht`-profiel; `calculate_demand` delegeert met `None` (byte-identieke terugval). Koudebalans gebruikt `q_c_ht` en `cooling_demand_gated` |
| `crates/nta8800-demand/src/lib.rs` | re-export `calculate_demand_with_cooling_ht` |
| `crates/openaec-project-shared/src/tojuli.rs` | `cooling_indoor_temperature` (24 °C); tweede `calculate_transmission` op de koel-setpoint (`transmission_cooling`, form. 7.15); ventilatie-branch → closure `compute_ventilation(indoor)` zodat `ventilation_cooling` op 24 °C herberekend wordt; `cooling_heat_transfer = Q_C;tr + Q_C;ve`; doorgegeven aan `calculate_demand_with_cooling_ht` |

**Waarom `calculate_transmission`/`calculate_ventilation` hergebruiken i.p.v. het
20 °C-profiel schalen:** de warmteoverdracht voor koeling loopt over onverwarmde
buffers (b-factor, Gouda-kruipruimte), grond (jaargemiddeld ΔT, §8.3) en
koudebruggen — elk met een eigen temperatuur-afhankelijkheid. De transmissie-/
ventilatie-crates verrekenen dat al correct per formule 7.15/7.16; een handmatige
`(24−θ_e)/(20−θ_e)`-schaling zou de buffer- en grondtakken verkeerd doen. De closure
isoleert de setpoint als énige variabele; systeem-/debiet-context blijft identiek.

**Additiviteit:** `calculate_demand` (zonder `cooling_heat_transfer`) is identiek aan
het gedrag vóór deze uitbreiding **op de §7.2.2-poort na**: die geldt norm-correct
óók op de terugval-tak (vóór deze uitbreiding ontbrak hij — een bewuste,
norm-conforme gedragswijziging, geen byte-identieke terugval). Er zijn geen callers
buiten de TO-juli-/BENG-keten (die altijd `Some(Q_C;ht)` levert), en de bestaande
demand-crate-tests — die alleen `≥ 0`/monotonie asserten — blijven groen.
`beng/zeb.rs` (ZEB/saldering) onaangeraakt.

---

## Vóór/na per golden-case (bridged, F6-brug)

| Case | Grootheid | vóór C2 | ná C2 | certified | tol | status |
|------|-----------|--------:|------:|----------:|----:|--------|
| Aalten | BENG 1 | +28,4 % (133,16) | **+11,2 % (115,27)** | 103,69 | ±6 % | rood (massa-residu) |
| Aalten | BENG 2 | +36,7 % (33,77) | **−4,2 % (23,68)** | 24,71 | ±10 % | **GROEN** |
| Aalten | BENG 3 | −3,8 pp (81,22) | **+1,0 pp (86,04)** | 85,00 | ±3 pp | **GROEN** |
| Aalten | verwarming primair [kWh] | 2444 | 2444 (ongewijzigd) | 2551 | — | anchor blijft groen |
| Gouda | BENG 1 | +20,0 % | **+1,0 % (96,83)** | 95,86 | ±6 % | **binnen tol** |
| Gouda | BENG 2 | −19,0 % | −56,4 % (11,99) | 27,48 | ±8 % | rood (PV-normversie) |
| Gouda | koeling primair [kWh] | ~3334 | 1969 | 244 | — | massa-residu |

Nieuwe groene anchor: **`aalten_beng_geometry_beng2_matches_certified`** (BENG 2 op
certified — de C2-kernbelofte). De verwarmings-anchor
`aalten_beng_geometry_heating_matches_certified` bleef groen (2444 kWh, ongewijzigd):
C2 raakt uitsluitend de koudebalans.

De samengestelde goldens blijven `#[ignore]` met bijgewerkte, gemeten redenen:
- **Aalten** — BENG 1 +11,2 %: thermische-massa-residu (zie onder).
- **Gouda** — BENG 2/3: PV-saldering-normversie (F3d-8, ongewijzigde blokkade); BENG 1
  is nu wél binnen tol maar de golden toetst alle drie.

---

## Resterende gaps (vervolgtickets, buiten C2-scope) → TODO.md

1. **Thermische massa (F7.2) — dominante Aalten-BENG 1-residu.** De brug draait
   `ThermalMassInput::light_woning()`; Aalten heeft een massieve betonvloer
   (`bouwwijze_vloer` = massief beton zeer zwaar in de fixture). Hogere C_m → hogere
   τ → hogere a → hogere η_C;ht → lagere Q_C;nd. Diagnostisch: `zwaar_massief` zet
   BENG 1 op −14,5 % **én breekt de heating-anchor** (2119 vs 2551 kWh). De echte C_m
   ligt ertussen; de invoer is aan de verwarming gekoppeld → een eigen werkpakket met
   bouwwijze-→-C_m-mapping, niet de koudebalans.
2. **Interne warmtewinst woningbouw (form. 7.21).** De keten gebruikt het forfait
   3 W/m² (`InternalGains::forfaitair`); NTA 8800 §7.5.2.1 rekent
   `Q_int = 180 · N_woon · N_P;woon · t_mi` (Aalten: N_P ≈ 1,95 → ~6,6 W/m² in juli).
   Dat is hoger dan het forfait (verhoogt zowel koeling als — verlagend — verwarming);
   het koppelt aan de heating-anchor en is dus een aparte correctie.
3. **Gouda PV-saldering-normversie (F3d-8).** BENG 2/3 blokkade; zie
   `docs/2026-07-12-f3d8-norm-analyse-saldering.md`. Ongewijzigd door C2.

---

## Teststatus

`cargo test --workspace`: **volledig groen** (62 test-binaries, 0 failed). Nieuwe/
gewijzigde tests:
- `nta8800-demand` (`monthly_balance`): `gated_poort_kapt_verliesgedomineerde_maand_af`,
  `gated_koelmaand_gelijk_aan_ongepoort`, `gated_grens_precies_twee_telt_nog_mee`,
  `gated_geen_winst_geeft_nul`.
- `openaec-project-shared` (`beng_golden`): nieuwe groene anchor
  `aalten_beng_geometry_beng2_matches_certified`; `#[ignore]`-redenen Aalten/Gouda
  bijgewerkt met de gemeten C2-verschuiving. `expected.json`/`input.json`
  onaangeraakt.
