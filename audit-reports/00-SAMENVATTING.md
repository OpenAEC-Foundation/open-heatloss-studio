# Audit warmteverlies-engine — geconsolideerde samenvatting

**Datum:** 2026-06-02
**Scope:** norm-conformiteit (ISSO 51:2023 + ISSO 53) van de rekenkernen `isso51-core` / `isso53-core`, verificatie-dekking, en UI-veld-dekking.
**Methode:** 4 norm-audit-agents (lezen ISSO-PDF's regel-voor-regel tegen Rust), 1 UI-dekkingsaudit, + Codex onafhankelijke cross-check. Baseline: alle bestaande tests groen (isso53: 105 unit + 13 golden; isso51: 163 unit + 3 golden), maar "groen" = binnen tolerantie.
**Conform-definitie (besluit gebruiker):** HYBRIDE — norm leidend als default; bewuste Vabi-compat alleen achter een expliciet gemarkeerd pad; beide getest.

Detailrapporten: `01-transmissie-grond.md`, `02-ventilatie-infiltratie.md`, `03-opwarmtoeslag-bronvermogen.md`, `04-isso51.md`, `05-ui-veld-dekking.md`.

---

## 1. Urgentie × effort matrix

| ID | Bevinding | Norm | Afwijking | Effort | Wanneer actief |
|----|-----------|------|-----------|--------|----------------|
| A1 | ISSO 51 opwarmtoeslag gebruikt 2017-model `f_RH × ΣA_metselwerk` i.p.v. 2023 `P × A_g` | ISSO 51:2023 §2.5.8/§4.3.1 | **2-4× fout** | Hoog | alleen bij nachtverlaging |
| A2 | ISSO 51 nacht-afkoeling Δt aan gebouwtype i.p.v. Ū/massa; mist "Ū≤0,5→1K" | ISSO 51:2023 Afb 2.7 | **~3× fout** (goed geïsoleerd) | Middel | alleen bij nachtverlaging |
| A5 | ISSO 53 Δθ_1-stratificatietabel ontbreekt + geen vide-correctie ×h/4 | ISSO 53 tab 2.3 | forse onderschatting atria/vides/hallen | Hoog | h>4m / vides |
| A6 | ISSO 53 ΔU_TB-prioriteit omgekeerd in `shell.rs` vs `transmission.rs` | ISSO 53 §3.1 | tot kW-orde (voorontwerp) | **Laag** | custom ΔU_TB |
| A3 | ISSO 53 §4.8.3-reductie `−H_v·Δθ` ook op natuurlijk geventileerde ruimten | ISSO 53 §4.8.3 | Φ_hu te laag/0 | Middel | nat. ventilatie + setback |
| A4 | ISSO 53 grond `U_k = U + ΔU_TB` mist (docstring liegt) | ISSO 53 §4.7 | lichte onderschatting | **Laag** | grondvloeren |
| A7 | ISSO 53 nat. ventilatie `f_v=1.0` hardcoded, negeert Δθ_v | ISSO 53 form 4.39 | ~1,7% overschatting | **Laag** | straling/vloer/wand-verw. |

## 1b. Codex cross-check — aanvullende criticals (orthogonaal aan agents)

Codex (gpt-5, read-only) vond bugs die de 4 norm-agents misten omdat die tabel-wáárden checkten i.p.v. downstream-paden. D1 en D2 zijn PM-geverifieerd aan de bron. Detail: `06-codex-crosscheck.md`.

| ID | Bevinding | Afwijking | Effort | Wanneer actief |
|----|-----------|-----------|--------|----------------|
| **D1** | `temperature.rs:21,93` sentinel `f64::MIN` voor `Garage` wordt door callers NIET vervangen door θ_e → `H×(f64::MIN−θ_e)` | **oneindig/astronomisch** Φ_T | **Laag** | garage-ruimte zonder custom_temp |
| **D2** | `ventilation.rs:116` altijd `VentilatieBouwfase::Nieuwbouw` | **~+89% Φ_V** bestaande bouw | Middel (+ UI/model-veld) | bestaande bouw |
| **D3** | `infiltration.rs:117` `Unknown`-methode negeert gebouwafmetingen → f_wind=1,0 i.p.v. ~1,29 | ~22% te lage infiltratie | Laag | Unknown-infiltratiepad |
| **D4** | `ground.rs:144` `U_equiv` weigert normale `z=0` grondvloer (test bevestigt fout gedrag) | vloeren falen tenzij u_equiv vooraf ingevuld | Middel | grondvloeren z=0 |
| **D5** | `shell.rs:88` voorontwerp-schil grove vaste aannames (0,5 ach) | tientallen % voorontwerp | Hoog | schilmethode |

**Extra verborgen-afwijking (Codex):** `vabi_dr_golden.rs` 10% tolerantie — expected 3059 W, snapshot 3165 W (+3,5%), nog ~190 W regressie zou slagen; `vabi_golden.rs:37` test Φ_V+Φ_I **gecombineerd** → fouten compenseren elkaar.

## 2. Stille-fout klasse (fout antwoord ZONDER error — apart gevaarlijk)

| ID | Bevinding | Effort |
|----|-----------|--------|
| B1 | `isso53 heating_up.rs:97` `unwrap_or(0.0)` bij ongeldige setback → Φ_hu verdwijnt geruisloos | Laag |
| B2 | `isso53 model/project.rs:27` `#[serde(default)]` → ontbrekend `heatingUp`-blok = Φ_hu=0 hele gebouw (third-party import ~10-28% te laag) | Laag |
| B3 | `isso53 ventilation.rs` magic `unwrap_or(0.05/6.5)` zonder rapport-spoor | Laag |

## 3. UI-veld-dekking (calc-input zonder invoerveld → stille default)

| ID | Veld | Gevolg bij ontbreken | Status |
|----|------|----------------------|--------|
| U1 | `source_zone_config` (gescheiden opwekker z=1.0) | Φ_source altijd z=0.5 → onderschat bronvermogen | niet gemapt |
| U2 | `unheated_space`-enum (15 norm-varianten tab 4.2) | valt op reductiefactor 0.5 i.p.v. juiste waarde | niet kiesbaar |
| U3 | Koudebrug-toggle + custom ΔU_TB | forfaitair altijd aan (raakt A6) | geen UI |
| U4 | Grond-params (u_equiv, f_gw, perimeter/diepte) | f_gw altijd 1.0, fallback op construction-U | alleen via thermal-import |
| U5 | Voorverwarming (`has_preheating`/temperatuur) | voorverwarmde lucht niet meegerekend | geen UI |
| U6 | Vide/vertrekhoogte >4m voor stratificatie (raakt A5) | per-vertrek-calc leest `room.height` niet | geen pad |

**Wezen (UI-veld zonder calc-effect — verwarrend):** `material_type` (claimt ΔU_TB-invloed die niet bestaat), `frost_protection` (isso53-mapper stuurt altijd null; wél isso51-relevant).

## 4. Verificatie-dekking — het grootste gat

| ID | Bevinding |
|----|-----------|
| V1 | **Beide ISSO 51-fixtures: `night_setback=false` → alle `phi_hu=0`. De foute opwarmtoeslag-kern (A1/A2) wordt NOOIT uitgevoerd door een test.** |
| V2 | ISSO 53 golden-toleranties (`3.10a +5.0%`, `1.10a #[ignore]`) kunnen A3/A5 maskeren |
| V3 | Geen enkele fixture mét nachtverlaging in beide normen |

## 5. Bewuste Vabi-keuzes (geen bug — onder hybride-beleid markeren/documenteren)

| ID | Keuze | Actie onder hybride-beleid |
|----|-------|----------------------------|
| C1 | `isso53 nen8088.rs` infiltratie via NEN 8088 power-law (Δp=3,14) = Vabi, niet ISSO 53 | expliciet markeren in rapport-output |
| C2 | `isso51 lib.rs:218-225` `VabiCompat`-aggregatie sluit Φ_T,iae uit | idem; aparte ISSO-conforme variant naast Vabi-variant |

## 6. Aanbevolen volgorde (volgende sessie, indien fixen)

0. **Landmine eerst:** D1 (garage-sentinel → oneindig verlies) — laag effort, kan elk garage-bevattend project laten ontsporen.
1. **Quick wins, hoog effect:** D3 + A6 + A4 + A7 (alle "Laag", directe norm-fout) + B1/B2/B3 (stille-fout → errors).
2. **Grote afwijkingen, common case:** D2 (+89% bestaande bouw ventilatie) + D4 (z=0 grondvloer).
3. **Grootste rekenfout dichten:** A1 + A2 samen (ISSO 51 opwarmtoeslag 2023) + V1/V3 (nieuwe fixture MÉT nachtverlaging die het écht test).
4. **A3** (ISSO 53 opwarmtoeslag-reductie) — na A1/A2, zelfde testaanpak.
5. **A5** (stratificatie Δθ_1 + vide) — grootste calc-uitbreiding; vergt nieuwe tabel + U6 UI-veld.
6. **UI-gaten U1-U5** parallel aan de bijbehorende calc-fixes.

> **A1/A2 HARD BEVESTIGD (2026-06-02) tegen ISSO 51:2023 PDF:**
> - Formule 4.15 (§4.3.1, p.70): `Φ_hu,i = P × A_g`, met P [W/m²] uit §2.5.8 en A_g = totale gebruiksoppervlak (vloeroppervlak).
> - Tabel 2.10 (§2.5.8, p.45): P geïndexeerd op (aantal graden verlaging × zwaarte gebouw ZL+L+M/Z × opwarmtijd).
> - Afb. 2.7 (p.44): afkoeling/graden uit Ū (opp.-gewogen gem. U incl. koudebruggen + bg-vloer); Ū≤0,50 → 1 K.
> - Code `heating_up.rs:41` gebruikt `f_RH × accumulating_area` (metselwerk-opp.) met Δt uit `building_type` = ISSO 51:**2017**-model. `f_RH` bestaat niet in de 2023-norm. De unit-test `test_isso51_example_room1_heating_up` codeert het 2017-model en houdt de fout groen.
> - Implementatiedetail voor de rewrite: scope van A_g (per-vertrek vloeropp. vs gebouwbreed verdeeld) + de regeltype-branches §4.3.1/4.3.2 (zelflerend → Φ_hu=0) / §4.3.3 (thermostaat → 5 W/m²) exact uitwerken.
>
> **A5 HARD BEVESTIGD (2026-06-02) tegen ISSO 53 PDF tab 2.3 (p.21-22):** code heeft alléén `delta_theta_2` (1 call-site `ground.rs:189`, form. 4.23 — correct). Ontbreekt volledig: **Δθ₁** (per systeem: lokaal +4, radi-ht +3, radi-lt +2, plafond +3, wand +2, plint +1, vloer+ht +3, vloer+lt +2, vloer-hoofd 0, vloer+wand +1, betonkern 0, ventilatorgedreven 0,5 — nodig in form. 3.4/3.5, 4.5/4.6, 4.11/4.12, 4.15/4.16, 4.19/4.20 voor vloeren-boven-buitenlucht/daken/plafonds/aangrenzend → ~+10% dak-transmissie bij radi-ht), **Δθ_v** (=A7-kolom: wand/vloer-lt/vloer-hoofd/vloer+wand/betonkern = -1 bij R_c<3,5, -0,5 bij R_c≥3,5), Δθ_a1/Δθ_a2, en de vide-correctie **Δθ₁×(h/4)** bij h>4m (voetnoot 2, letterlijk). Δθ₂-waarden zelf 12/12 correct. Vermoedelijke verklaring voor verborgen +5,0% op dak-zwaar vertrek 3.10a.
