# Norm-conformiteit-audit 01 — Transmissie & Grondverlies (ISSO 53)

**Scope:** `isso53-core` transmissie- en grondverlies-keten.
**Norm-bron:** ISSO-publicatie 53 (2016, PDF 95 p.), aanvullend NEN 1068.
**Datum:** 2026-06-02 · **Modus:** read-only, geen broncode gewijzigd.

Geauditeerde bestanden: `calc/transmission.rs`, `calc/ground.rs`, `calc/shell.rs`, `calc/source_capacity.rs`, `tables/thermal_bridge.rs`, `tables/ground_params.rs`, `tables/adjacent_unheated.rs`, `tables/temperature.rs`, `tables/temperature_stratification.rs`, `tables/building_type.rs`, `model/climate.rs`, `model/enums.rs`.

---

## Kritieke conformiteitsfouten

### K1 — ΔU_TB niet meegenomen in U_equiv-berekening (grond)
| Aspect | Detail |
|--------|--------|
| Bestand:regel | `calc/ground.rs:44-50` (call), `calc/ground.rs:105-162` (`calculate_u_equivalent`), `calc/ground.rs:154` (`term2`) |
| Norm-clausule | Formule 4.24 + variabelenlijst, **PDF p.44**; tabel 4.3 |
| Norm-formule | In 4.24 is de U-term expliciet `U_k = U + ΔU_TB` (variabelenlijst p.44: *"ΔU_TB = toeslag voor thermische bruggen volgens tabel 3.1"*). De norm verwerkt de koudebrug-toeslag dus ín de equivalente U van grondvlakken. |
| Code | `calculate_u_equivalent(element.area, perimeter, depth, element.u_value, is_wall)` — er wordt **`element.u_value` rauw** doorgegeven; ΔU_TB (`DELTA_U_TB_DEFAULT = 0.10` of `custom_delta_u_tb`) wordt niet opgeteld. De docstring op `ground.rs:86-91` claimt `U_k + ΔU_TB` maar de implementatie (`term2 = c2 · u_construction^n2`, regel 154) gebruikt de rauwe waarde. |
| Numerieke impact | Onderschatting van U_equiv → onderschatting H_T,ig. Bij een typische begane-grondvloer U=0,18 vs U_eff=0,28 schuift `term2 = 9,773·U^0,6027` van 9,773·0,18^0,6027 ≈ 3,57 naar 9,773·0,28^0,6027 ≈ 4,62 (≈ +29% op term2). Door de sterk negatieve exponent `b=-7,455` werkt dit niet-lineair door; in de praktijk enkele %-en op H_T,ig per ruimte. Klein absoluut (grondverlies is meestal <5% van totaal) maar systematisch te laag en strijdig met de norm-definitie. |
| Voorgestelde fix | Vóór de call `let u_k = element.u_value + delta_u_tb;` bepalen (zelfde ΔU_TB-prioriteit als exterior: custom > forfaitair > 0) en die doorgeven aan `calculate_u_equivalent`. Let op: voor `has_embedded_heating` blijft f_ig=0 dus dan irrelevant. |

### K2 — Inverse prioriteit ΔU_TB in schilmethode (custom-waarde genegeerd)
| Aspect | Detail |
|--------|--------|
| Bestand:regel | `calc/shell.rs:52-56` |
| Norm-clausule | Tabel 3.1 / formule 3.3 (PDF p.28); consistentie met detailmethode 4.3 |
| Norm/verwachting | ΔU_TB-prioriteit moet identiek zijn aan de detailmethode: **expliciete custom-waarde > forfaitair > 0** (zie `transmission.rs:70-77`). |
| Code | ```let delta_u_tb = if element.use_forfaitaire_thermal_bridge { DELTA_U_TB_DEFAULT } else { element.custom_delta_u_tb.unwrap_or(0.0) };``` — hier wint `use_forfaitaire_thermal_bridge` áltijd; een ingevulde `custom_delta_u_tb` wordt genegeerd zodra de forfaitair-vlag aanstaat, én omgekeerd kan een element met custom-waarde maar zonder vlag wél de custom krijgen. Dit is de **omgekeerde voorrangsregel** t.o.v. `transmission.rs`. |
| Numerieke impact | Inconsistentie tussen voorontwerp (schil) en definitief (detail). Een element met `custom_delta_u_tb = 0,02` (nieuwbouw, speciale voorzieningen) én forfaitair-vlag aan rekent in de schilmethode met 0,10 i.p.v. 0,02 → tot +0,08 W/(m²·K) per m² gevel te hoog. Op een gevel van 1.000 m²: ≈ +80 W/K → bij Δθ=30 K ≈ +2,4 kW overschatting in de voorontwerp-aansluitcapaciteit. |
| Voorgestelde fix | Logica spiegelen aan `transmission.rs:70-77`: `element.custom_delta_u_tb.unwrap_or_else(|| if use_forfaitaire { DELTA_U_TB_DEFAULT } else { 0.0 })`. |

### K3 — Temperatuurgelaagdheid (Δθ_1 / Δθ_2) ontbreekt in f_k en f_ia,k voor vloeren/plafonds
| Aspect | Detail |
|--------|--------|
| Bestand:regel | `calc/transmission.rs:155-160` (adjacent rooms), `calc/transmission.rs:176-186` (adjacent buildings); `calc/shell.rs:72-77` |
| Norm-clausule | Formules **4.14/4.15/4.16** (onverwarmd, PDF p.41) en **4.18/4.19/4.20** (aangrenzend pand, PDF p.42); tabel 2.3 (PDF p.21-22) |
| Norm-formule | De norm geeft per oriëntatie aparte formules: wand (4.14/4.18), vloer (4.15/4.19), plafond (4.16/4.20). De vloer- en plafondvarianten bevatten een Δθ_1- of Δθ_2-stratificatieterm (variabelenlijst p.41/p.42 noemt expliciet `Δθ_1` én `Δθ_2 volgens tabel 2.3`). |
| Code | Eén generieke factor `(θ_i − θ_adj)/(θ_i − θ_e)` voor álle oriëntaties; `vertical_position` (Wall/Floor/Ceiling) wordt voor adjacent-room/-building **niet** gebruikt. Geen Δθ-correctie. |
| Numerieke impact | Voor verticale wanden tussen ruimten/panden klopt de simpele factor (geen stratificatieterm in 4.14/4.18). Voor **vloer/plafond** tussen verdiepingen of naar bovengelegen woning mist de Δθ-term (±1 K). Bij Δθ_eff van bv. 11 K betekent ±1 K ≈ ±9% op die f-factor → enkele procenten op H_T,ia / H_T,iaBE van horizontale scheidingen. Bij gebouwen met veel inter-verdieping-overdracht of bovengelegen woningen relevant. |
| Voorgestelde fix | f_ia/f_k splitsen per `vertical_position` analoog aan `ground.rs::calculate_f_ig_auto`, met Δθ_1 (vloer/plafond-bovenzijde) resp. Δθ_2 (vloer-onderzijde) uit tabel 2.3. Vereist ook een Δθ_1-tabel (zie K4). |

### K4 — Δθ_1-tabel ontbreekt volledig; alleen Δθ_2 geïmplementeerd
| Aspect | Detail |
|--------|--------|
| Bestand:regel | `tables/temperature_stratification.rs:11-26` (alleen `delta_theta_2`) |
| Norm-clausule | Tabel 2.3, **PDF p.21-22** — kolom *Δθ_1 resp. Δθ_a1* |
| Norm-waarden (Δθ_1, eerste kolom) | Lokale verw. **+4**; Radiatoren ht/lucht **+3**; Radiatoren lt **+2**; Plafondverw. **+3**; Wandverw. **+2**; Plintverw. **+1**; Vloerverw.+ht **+3**; Vloerverw.+lt **+2**; Vloerverw. hoofdverw. **0**; Vloerverw.+wandverw. **+1**; Betonkernactivering **0**; Ventilatorgedreven **0,5**. |
| Code | Geen `delta_theta_1`-functie aanwezig. Δθ_1 wordt nergens toegepast (vereist voor formules 4.15/4.16/4.19/4.20 vloeren/plafonds en voor de vide-hoogtecorrectie, zie K5). |
| Numerieke impact | Zolang K3 niet gefixt is, is dit latent. Wordt blokkerend zodra horizontale scheidingen Δθ-correct moeten. Δθ_1 tot +4 K op een vloer/plafond-factor is een substantiële afwijking (op Δθ_eff ≈ 11 K is +4 K ≈ +36% op die f-term). |
| Voorgestelde fix | Tabel-functie `delta_theta_1(HeatingSystem)` toevoegen met bovenstaande 12 waarden; inzetten in de oriëntatie-gesplitste f-formules (K3). |

### K5 — Vertrekhoogte-correctie (vides > 4 m) niet geïmplementeerd — toeslag h/4 op Δθ
| Aspect | Detail |
|--------|--------|
| Bestand:regel | gehele stratificatie-keten; geen verwijzing naar `room.height` in `transmission.rs`/`ground.rs`/`temperature_stratification.rs` |
| Norm-clausule | Tabel 2.3, **voetnoot 2, PDF p.22** |
| Norm-regel | *"Bij toepassing van vides etc. waardoor een grotere hoogte ontstaat moet de waarde van Δθ_1 resp. Δθ_a1 worden vermenigvuldigd met h/4 waarbij h de totale hoogte [m] is."* |
| Code | Geen hoogte-afhankelijke schaling van Δθ. `room.height` wordt alleen voor volume (ventilatie, `shell.rs:87`) gebruikt, niet voor stratificatie. |
| Numerieke impact | Voor de doelgroep van deze publicatie (vertrekhoogten tot 4 m) is h/4 ≤ 1 → meestal geen effect. **Maar** bij vides/atria/sporthallen met h > 4 m wordt Δθ_1 te laag aangehouden: bij h=8 m mist een factor 2 op Δθ_1 (tot +4 K → +8 K). Substantiële onderschatting van vloer-/plafondverlies in hoge ruimten. Norm-titel begrenst weliswaar op 4 m, maar voetnoot 2 dekt expliciet de vide-uitzondering. |
| Voorgestelde fix | Bij Δθ_1/Δθ_a1-gebruik: `delta_theta_1 * (room.height / 4.0).max(1.0)` (alleen ophogen, niet verlagen). Afhankelijk van K3/K4. |

---

## Twijfelgevallen (te verifiëren tegen norm)

### T1 — Formule 4.24 machtsstructuur niet uit PDF-tekstlaag te bevestigen
- **Bestand:** `calc/ground.rs:152-158`, `tables/ground_params.rs:60-87`.
- **Status:** De parameters a/b/c1-c3/n1-n3/d in `GROUND_PARAMS_FLOOR`/`_WALL` matchen **exact** tabel 4.3 (PDF p.44, geverifieerd: Vloer 0,9671/-7,455/10,76/9,773/0,0265/0,5532/0,6027/-0,9296/-0,0203; Wand 0,799/-6,7951/0/26,586/0,1523/0/0,5012/-0,1406/-1,074). De **formule-vorm** `a·(c1·B'^n1 + c2·U^n2 + c3·(z+d)^n3)^b` staat in de PDF als afbeelding; OCR (300 dpi) kon de exponentstructuur niet betrouwbaar lezen (`a + ( C 2 + + + ...`). De codebase-comment (`ground_params.rs:17-22`) erkent dit en steunt op Vabi-fixture-validatie (commit 0f4293a).
- **Actie:** Bevestigen tegen een tweede bron (NEN 1068 bijlage of ISSO 53 rekenvoorbeeld met expliciete U_equiv-tussenwaarde). Worked example p.65 geeft U_equiv=0,177 bij U=2,43, B'=12,07 — narekenen met de code-formule om de structuur hard te verifiëren.

### T2 — `z + d` guard verwerpt ondiepe wanden/vloeren (z=0)
- **Bestand:** `calc/ground.rs:144-150`; tests `ground.rs:207-218`.
- **Status:** Voor wanden is `d = -1,074`; bij z < 1,074 m is `z + d ≤ 0` → `InvalidInput`. Voor een begane-grond-wand met geringe insteek (z=0,5 m) faalt de berekening. De norm clamp is `0 ≤ z ≤ 5` (p.44) en stelt **geen** ondergrens z ≥ |d|. De negatieve `d` is een regressie-parameter, geen fysieke diepte; bij z=0 hoort `(z+d)^n3` met n3 negatief een grote (maar eindige, want d≠0... voor wand z+d=-1,074 → negatieve basis met niet-integer exponent = NaN) waarde te geven.
- **Risico:** De norm-tabel is gefit voor wanden die werkelijk onder maaiveld liggen (z > |d|). Bij z=0 (wand niet in grond) is de formule fysisch niet bedoeld. De guard voorkomt NaN, maar geeft een **harde error** i.p.v. een fallback. Dit kan legitieme modellen blokkeren (bv. een grondwand opgegeven met z=0 door invoerfout/aanname).
- **Actie:** Verifiëren of ISSO 53 een minimale z voor wanden voorschrijft, of dat de tool i.p.v. error een z-clamp naar z_min (= net boven |d|) of een waarschuwing moet geven. Norm-tekst noemt alleen `0 ≤ z ≤ 5`, wat suggereert dat z=0 toegestaan moet zijn → huidige error is mogelijk te streng.

### T3 — Ceiling-grondvlak behandeld als wand (geen Δθ-correctie)
- **Bestand:** `calc/ground.rs:193-196`.
- **Status:** Een `VerticalPosition::Ceiling` grondvlak gebruikt de wand-formule 4.22 `(θ_i − θ_me)/(θ_i − θ_e)` zonder Δθ. Een plafond tegen grond is fysiek ongebruikelijk; de norm 4.6 noemt "wanden en vloeren". Worked example p.65 past op een horizontaal grondvlak wél Δθ_2 toe (f_ig=0,351 = (20−1−9)/28,5). Conservatief gelabeld in de code-comment, maar niet norm-exact.
- **Actie:** Bevestigen of ceiling-grondvlakken überhaupt voorkomen in de doelmodellen; zo ja, of 4.23 (Δθ_2) of een plafond-variant geldt.

### T4 — θ_b aangrenzend gebouw hardcoded 15 °C; `theta_b_adjacent_building`-veld ongebruikt
- **Bestand:** `calc/transmission.rs:178`, `calc/shell.rs:71`; veld `model/climate.rs:24`.
- **Status:** Default `theta_b = element.adjacent_temperature.unwrap_or(15.0)`. Conform p.42 (ch4 forfaitair: 15/5/θ_e), dus norm-correct voor de detailmethode. **Echter** het centrale `DesignConditions.theta_b_adjacent_building`-veld (default 15) wordt hier niet gebruikt — dode parameter / inconsistentierisico als een gebruiker dat veld zet maar per-element niets invult. Ch3 (p.30) noemt bovendien 10 °C voor "overige utiliteitsgebouwen en woningen", wat in ch4 niet als optie terugkomt.
- **Actie:** Beslissen of `theta_b_adjacent_building` als fallback vóór de hardcoded 15.0 moet komen. Geen norm-fout, wel een architectuur-/consistentiepunt.

### T5 — Shell-methode unheated fallback f_k = 0,8
- **Bestand:** `calc/shell.rs:65`.
- **Status:** `.unwrap_or(0.8)` als geen `unheated_space`/`temperature_factor` opgegeven. Tabel 4.2 kent geen enkele "default 0,8"; 0,8 is de bovenkant van het bereik (conservatief). Voor een grove voorontwerp-schatting verdedigbaar, maar het is een engineering-aanname zonder norm-grondslag.
- **Actie:** Conform rapportage-principe (zie MEMORY): aanname expliciet markeren in output i.p.v. stil 0,8 hanteren. Verifiëren of conservatief (hoog) hier gewenst is.

---

## Geverifieerd correct (kort)

| Onderdeel | Bestand | Norm-ref | Status |
|-----------|---------|----------|--------|
| ΔU_TB tabel 3.1 (0 / 0,02 / 0,05 / 0,15 / 0,10) | `thermal_bridge.rs:32-40` | tabel 3.1, p.28 | ✓ exact |
| DELTA_U_TB_DEFAULT = 0,10 ("overige situaties") | `thermal_bridge.rs:28` | p.28; worked ex. p.59 `(U+0,1)` | ✓ |
| H_T,ie = Σ A·(U+ΔU_TB)·f_k, f_k=1 exterior | `transmission.rs:64-86` | formule 4.3/4.13, p.38-40; ex. p.59 | ✓ |
| Grond-prefactor 1,45 | `ground.rs:17,70` | formule 4.21, p.43; ex. p.59/64/65 | ✓ |
| f_ig vloer = ((θ_i+Δθ_2)−θ_me)/(θ_i−θ_e) | `ground.rs:187-192` | formule 4.23, p.43; ex. p.65 (0,351) | ✓ |
| f_ig wand = (θ_i−θ_me)/(θ_i−θ_e) | `ground.rs:183-185` | formule 4.22, p.43 | ✓ |
| f_ig = 0 bij embedded heating (§4.6) | `ground.rs:61-62` | p.43 "f_ig,k = 0 voor verwarmd deel" | ✓ |
| f_gw = 1,0 / 1,15 | `ground.rs:12-13`; `ground_params.rs` | p.43 | ✓ (waarden correct; zie noot) |
| U_equiv-parameters tabel 4.3 (vloer+wand, 9 params elk) | `ground_params.rs:60-87` | tabel 4.3, p.44 | ✓ exact |
| B' = 2·A/O, clamp [2, 50] | `ground.rs:130`; `ground_params.rs:95-99` | p.43-44 | ✓ |
| z clamp [0, 5] m | `ground.rs:133`; `ground_params.rs:103` | p.44 | ✓ |
| U_equiv ≥ 0,1 W/(m²·K) | `ground.rs:161`; `ground_params.rs:91` | p.31 ("indien U_equiv<0,1 dan 0,1") | ✓ |
| Wand c1=n1=0 (B' geen invloed) | `ground_params.rs:77-87` | tabel 4.3 voetnoot 1, p.44 | ✓ |
| θ_me = 9 °C | `model/climate.rs:41-43` | p.43 | ✓ |
| θ_e,0 = -10 °C (basis) | `model/climate.rs:37-39` | formule 2.6, p.22 | ✓ basis (zie noot θ_e,τ) |
| Δθ_2 tabel 2.3 (alle 12 systemen) | `temperature_stratification.rs:11-26` | tabel 2.3 kolom Δθ_2, p.21-22 | ✓ exact (12/12) |
| HeatingSystem default = Radiatoren ht (Δθ_2=-1) | `enums.rs:248-249` | tabel 2.3 | ✓ |
| f_k tabel 4.2 (15 onverwarmde-ruimte-categorieën) | `adjacent_unheated.rs:19-37` | tabel 4.2, p.41-42 | ✓ exact (0,4/0,5/0,6/0,8; kelder 0,5/0,8; dak 1,0/0,9/0,7; verkeersr. 0,0/0,5/1,0; kruipr. 0,6/0,8/1,0) |
| f_type tabel 4.6 | `building_type.rs:25-34` | tabel 4.6, p.46 | ✓ (buiten transmissie-scope, niet diepgaand gecontroleerd) |
| f_typ tabel 4.8 | `building_type.rs:38-48` | tabel 4.8, p.47 | ✓ (idem) |
| θ_i tabel 2.2 (zorg 22, overig 20; bad 22/24; toilet/verkeer 18; techn./berg 10; stalling 5; garage→θ_e) | `temperature.rs:41-95` | tabel 2.2, p.20 | ✓ |
| Φ_T = H_T,total·(θ_i−θ_e); H_T,ia uitgesloten in bronvermogen 5.1/5.9 | `transmission.rs:48-49`; `source_capacity.rs:32-35,74` | formule 4.2 / 5.1 / 5.9 | ✓ |
| Collective sluit Φ_T,iaBE uit | `source_capacity.rs:91` | formule 5.9, p.64 | ✓ |

**Noten bij geverifieerd:**
- θ_e default -10 is de **basis**ontwerpbuitentemperatuur (formule 2.6). De norm-voorbeelden rekenen met de tijdconstante-gecorrigeerde θ_e (= θ_e,0 + Δθ_e,τ, formule 2.7, bv. -8,5 °C). Of die correctie elders in de pipeline gebeurt is buiten deze auditscope — aanbevolen apart te verifiëren.
- Geen tabel-interpolatie aanwezig in de geauditeerde keten (alle lookups zijn discrete enum-matches; ground-params zijn directe constanten). Daarmee zijn off-by-one-interpolatiefouten hier niet van toepassing.

---

## Prioritering (impact-gesorteerd)

1. **K2** — inverse ΔU_TB-prioriteit schil → kW-orde overschatting voorontwerp (kwantificeerbaar, eenvoudige fix).
2. **K5 + K3 + K4** — stratificatie (Δθ_1 / hoogtecorrectie) ontbreekt → significant voor vides/hoge ruimten en horizontale scheidingen; samenhangend cluster.
3. **K1** — ΔU_TB ontbreekt in U_equiv → systematische lichte onderschatting grondverlies + docstring-mismatch.
4. **T1** — formule-4.24-structuur hard verifiëren (parameters al exact).
5. **T2-T5** — randgevallen / architectuur-consistentie.
