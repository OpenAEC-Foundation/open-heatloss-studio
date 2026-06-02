# Audit 02 — Ventilatie- & infiltratieverlies (ISSO 53)

**Scope:** `isso53-core` ventilatie/infiltratie-keten. READ-ONLY broncode-audit tegen ISSO-publicatie 53 (2016).
**Norm-PDF:** `ISSO-publicatie 53 ... vertrekhoogten tot 4 meter.pdf` (95 p.)
**Datum:** 2026-06-02
**Verdict in één zin:** De kernformules en alle tabellen (4.5–4.11) zijn **norm-correct overgenomen**; er is **één echte conformiteitsfout** (natuurlijke-ventilatie `f_v` negeert Δθ_v uit tabel 2.3) en een handvol structurele aannames die zichtbaar gedocumenteerd horen te worden i.p.v. stilzwijgend in code.

---

## Kritieke conformiteitsfouten

Gesorteerd op impact. Geen enkele fout is een eenheidsfout of dubbeltelling — het zwaarst-gewijzigde `ventilation.rs` blijkt rekenkundig solide.

| # | bestand:regel | norm-clausule | norm-formule/waarde | code-waarde | numerieke impact | fix |
|---|---------------|---------------|---------------------|-------------|------------------|-----|
| K1 | `calc/ventilation.rs:154-156` | Formule **4.39** + tabel **2.3** (Δθ_v-kolom), PDF p.48 & p.22 | `f_v = (θ_i − Δθ_v − θ_e)/(θ_i − θ_e)`, waarbij Δθ_v afhangt van het verwarmingssysteem (wandverw. −0,5; lt-vloer+rad −0,5; BKA −0,5; vloerverw.-hoofd −0,5 bij R_c≥3,5) | Hardcoded `f_v = 1.0` voor álle natuurlijke/mechanisch-zonder-voorverwarming-systemen | Bij R_c≥3,5 en stralings-/vloer-/wandverwarming wordt Δθ_v=−0,5 K genegeerd → f_v overschat met ~0,5/30 ≈ **1,7%** op Φ_vent. Klein, maar systematisch en altijd één kant op (overschatting). Bij ht-radiatoren/luchtverwarming/plafond Δθ_v=0 → géén impact (meest voorkomende geval). | `f_v = 1.0 − Δθ_v/(θ_i − θ_e)` met Δθ_v-lookup uit tabel 2.3 op basis van `building.heating_system` + R_c-klasse. Δθ_v is **negatief** in de tabel, dus f_v wordt >1 (méér verlies). |
| K2 | `calc/ventilation.rs:108` & `:116-117` | Tabel 4.11 (PDF p.51) + tabel 4.10 (PDF p.48-50) | Bij ontbrekende bezetting/eis is er **geen** universele default; tabel 4.11 geeft per functie een waarde of "n.v.t." (industrie/sport/winkel → expliciete invoer verplicht) | Magic fallbacks `unwrap_or(0.05)` (dichtheid) en `unwrap_or(6.5)` (dm³/s·pp) | Voor functies zónder tabel 4.11-default (industrie, sport, winkel) verzint de code 0,05 pers/m². Tabel 4.10 koppelt die functies meestal aan `personen_per_m2: None`, dus de eis is dan 0 via de None-gate — fallback wordt zelden bereikt. Maar als ze wél geraakt wordt is het een niet-normwaarde zonder spoor in het rapport. | Vervang silent defaults door expliciete `OccupancyContext`/`default_occupancy()` (die bestaat al in `tables/occupancy.rs` maar wordt **niet** aangeroepen vanuit `ventilation.rs`!) en laat industrie/sport/winkel zonder invoer een waarschuwing/Err geven i.p.v. 0,05. |
| K3 | `calc/ventilation.rs:106` | Tabel 4.11 (PDF p.51) | Bezettingsdichtheid komt uit tabel 4.11 per (functie, context) | Dichtheid komt uit `req.personen_per_m2` (= tabel **4.10**-kolom) i.p.v. `tables/occupancy.rs::default_occupancy` (= tabel **4.11**) | Tabel 4.10 en 4.11 geven dezelfde getallen (0,05 / 0,125 / 0,3), dus numeriek **0 impact vandaag**. Architectonisch risico: er zijn twee bronnen voor dezelfde grootheid; de occupancy-module met sport/bezoekers/bedgebied-contexten wordt genegeerd, dus de 0,3-sportvariant en 0,125-bezoekersvariant worden **nooit** toegepast in de ventilatieberekening. | Route dichtheid via `occupancy::default_occupancy(functie, context)`; verwijder `personen_per_m2` als occupancy-bron uit tabel 4.10 of merk het expliciet als duplicaat. |

---

## Risico in net-toegevoegde code (`ventilation.rs`, +257 regels)

De uitbreiding voegde drie gates toe vóór de bestaande people-based afleiding. Regel-voor-regel beoordeeld op regressie/dubbeltelling:

| Aspect | bestand:regel | Oordeel |
|--------|---------------|---------|
| **`ventilation_q_v_established` override (Fase 3)** | `:83-85` | **Correct.** Vastgestelde toevoer-q_v overrulet de hele keten, negatief geclampt. Geen dubbeltelling — het is een vroege `return`. Geen norm-bezwaar: een gemeten/vastgestelde q_v is per definitie de q_v in formule 4.37. |
| **`has_mechanical_supply == Some(false)` gate** | `:91-93` | **Norm-correct maar let op semantiek.** ISSO 53 §4.7.2: ventilatie = "alle lucht die wordt **toegevoerd**". Een ruimte met enkel mechanische afvoer (systeem C) en natuurlijke toevoer heeft wél verse-luchttoevoer en dus Φ_vent > 0. De gate zet q_v=0 alleen als het veld expliciet `false` is; de gebruiker moet dit dus bewust zetten voor "geen toevoer". Risico: als de UI `has_mechanical_supply=false` zet voor een systeem-C-ruimte (natuurlijke toevoer aanwezig!) wordt ventilatieverlies onterecht 0. **Twijfelgeval — zie hieronder T1.** |
| **`requirement() == None` → q_v=0** | `:99-102` | **Correct + goede regressiefix.** Voorheen crashte dit de hele projectberekening (NotSupported). Berg-/techn. ruimten hebben geen personeneis → q_v=0 is norm-conform (geen Bouwbesluit-eis). Goed getest (`test_room_without_ventilation_requirement_yields_zero`). |
| **`max(explicit, area_based)` people-logica** | `:110-113` | **Verdedigbaar, licht conservatief.** ISSO 53 zegt: gebruik tabel 4.11-richtwaarde "indien aantal personen niet door opdrachtgever opgegeven". De norm impliceert dat een *opgegeven* aantal de richtwaarde **vervangt**, niet dat je het maximum neemt. De code neemt `max()`, wat een lagere opgegeven bezetting optrekt naar de area-based default → **overschatting** als de opdrachtgever bewust een lager (realistisch) aantal opgeeft. Conservatief voor warmteverlies (veilige kant), maar strikt genomen niet wat §4.7.2 voorschrijft. **Twijfelgeval T2.** |
| **Aggregatie in `room_load.rs:61`** | `room_load.rs:61-62` | **Correct, geen dubbeltelling.** Φ_HL,i = Φ_T + Φ_vent + Φ_i + Φ_hu − Φ_gain. Ventilatie en infiltratie zijn gescheiden termen (formule 4.1) — beide tellen mee, wat klopt: infiltratie (ongecontroleerde lekkage, formule 4.25) en ventilatie (gecontroleerde toevoer, formule 4.35) zijn in ISSO 53 expliciet aparte posten. `h_v` wordt correct doorgegeven aan heating-up (§4.8.3). |
| **WTW/voorverwarming f_v** | `:132-153` | **Correct, exact norm-conform.** `f_v = (θ_i − θ_t)/(θ_i − θ_e)` = formule 4.38. Default η=0,75 redelijk. Luchtverwarming (θ_t > θ_i) → f_v=0 = norm (PDF p.48 "f_v=0 voor luchtverwarming"). Geverifieerd tegen PDF-rekenvoorbeeld p.66 (η=0,8 → f_v=0,2). `clamp(0,1)` defensief correct. |

**Conclusie net-toegevoegde code:** geen eenheidsfout, geen dubbeltelling, geen verkeerde aggregatie. Twee gedrags­keuzes (T1 supply-gate semantiek, T2 max-people) verdienen norm-verificatie maar zijn beide conservatief/defensief.

---

## Twijfelgevallen (norm-clausule te verifiëren)

| # | bestand:regel | Twijfel | Aanbeveling |
|---|---------------|---------|-------------|
| T1 | `ventilation.rs:91-93` | `has_mechanical_supply=false` ⇒ q_v=0. Maar systeem C/A heeft *natuurlijke* toevoer met wél ventilatieverlies. De gate dekt enkel "mechanische" toevoer. | Verifieer met UI-team: wordt dit veld alleen gezet voor "ruimte zonder enige toevoer"? Zo niet → systeem-C-ruimten krijgen onterecht Φ_vent=0. Overweeg hernoemen naar `has_any_supply` of koppelen aan `system_type`. |
| T2 | `ventilation.rs:110-113` | `max(explicit, area_based)` vs. norm "richtwaarde indien niet opgegeven". | Verifieer of dit een bewuste 3BM-engineering­keuze is (conservatief) of een misinterpretatie. Documenteer in rapport-voetnoot als bewust. |
| T3 | `nen8088.rs` (hele module) + `infiltration.rs:130-156` `UnknownVabiCompat` | Dit pad gebruikt **NEN 8088-1 + NTA 8800 + power-law (Δp/10)^0,67, default Δp=3,14 Pa** — dit is **bewust géén ISSO 53**, het is een Vabi-reproductie-pad. De f_inf/f_type-waarden wijken af van ISSO 53 tabel 4.6/4.7 (bv. SystemA: 1,10 vs 0,80; SystemD: 1,00 vs 1,15). | **Geen ISSO 53-fout** — correct als apart, gemarkeerd pad. WEL: borg dat het rapport expliciet meldt "infiltratie via Vabi-compat (NEN 8088/NTA 8800), niet ISSO 53 §4.2" wanneer dit pad actief is, anders lijkt het rapport norm-conform terwijl het Vabi-conform is. Δp=3,14 Pa is een fit-waarde zonder norm-grondslag → voetnoot. |
| T4 | `infiltration.rs:163-171` `calculate_f_wind` | Formule 4.32 is in de PDF een afbeelding (niet tekstueel leesbaar); code-vorm `max[1;(0,01·(24+0,555·√(L²+B²)+4,5·H))^0,65]` is geverifieerd via het DR-rekenvoorbeeld (L=30,B=20,H=13 → 1,016) maar niet 1:1 tegen de norm-formule. | Tel als geverifieerd-via-voorbeeld. Bij beschikbaarheid van een OCR/tekstversie van 4.32 één keer letterlijk natrekken. |
| T5 | `infiltration.rs:75` infiltratie-`f_v = 1.0` hardcoded | Formule 4.27/4.30: infiltratie heeft óók een f_v met Δθ_v uit tabel 2.3. PDF p.65 rekenvoorbeeld bevestigt "f_v=1 (uit tabel 2.3 volgt Δθ_v=0)" — maar dat geldt alleen voor ht-radiatoren. | Zelfde issue als K1 maar dan voor infiltratie. Bij stralings-/vloerverwarming met R_c≥3,5 zou f_v≠1. Impact identiek ~1,7%, altijd overschatting. Fix samen met K1 (gedeelde Δθ_v-helper). |

---

## Geverifieerd correct (kort)

- **ρ·c_p = 1200 J/(m³·K)** (`formulas.rs:7`) — exact formule 4.36/4.37. Eenheid m³/s wordt consequent gebruikt (q_v people-pad deelt door 1000 om dm³/s→m³/s te converteren, `ventilation.rs:120`). **Geen m³/h↔m³/s verwarring.** ✓
- **Formule 4.35** Φ_vent = H_v·(θ_i−θ_e), **4.37** H_v = q_v·1200·f_v — letterlijk correct (`ventilation.rs:38,41`). ✓
- **Formule 4.38** WTW f_v=(θ_i−θ_t)/(θ_i−θ_e) — correct, getest tegen PDF p.66-voorbeeld. ✓
- **Tabel 4.10** (ventilatie-eisen, `ventilation_requirements.rs`) — alle gecontroleerde rijen matchen PDF p.48-50 exact: Kantoorruimte 6,5/0,05/3,44; Lesruimte 8,5/0,125/3,44; Patiëntenkamer 12/0,125/3,44; Operatiekamer 12/0,05; afvoer-constanten toilet 7 / douche 14 / keuken 3 dm³/s. Multi-waarde-cellen (0,125/0,3 etc.) correct gedocumenteerd met `// PDF:`-comment. ✓
- **Tabel 4.11** (`occupancy.rs`) — bezettingsdichtheden + contexten (sport 0,3 / cel-bezoekers 0,125 / bedgebied 0,125) matchen PDF p.51; n.v.t. voor industrie/sport/winkel correct als `None`. ✓ (kanttekening: module wordt niet aangeroepen, zie K3)
- **Tabel 4.5** q_is-matrix (`infiltration.rs:107-120`) — alle 6×5 cellen matchen PDF p.45-46 exact. Hoogte-/q_v10-klassegrenzen correct (≤3 / 3-6 / 6-20 / 20-30 / >30; <0,20 t/m >1,0). ✓
- **Tabel 4.6** f_type, **4.7** f_inf, **4.8** f_typ, **4.9** q_i,spec,reken — alle matchen PDF p.46-47 exact. ✓
- **Formule 4.31** q_is = f_wind·f_type·f_inf·(0,23·q_i,spec), **4.33** q_i,spec = f_typ·f_jaar·q_i,spec,reken, **4.34** f_jaar = 0,4+0,033·e^(0,05·(2060−J)) — alle drie letterlijk correct geïmplementeerd (`infiltration.rs:113-128, 173-178`). ✓
- **Formule 4.28/4.29** A_u (gevel, Known) vs A_g (vloeropp., Unknown) — correct onderscheiden. A_u filtert terecht op `VerticalPosition::Wall` (exterior vloer/dak van zwevend gebouw uitgesloten) — goede regressiebescherming, getest. ✓
- **z-reductiefactor** (`infiltration.rs:72`) — correct toegepast in formule 4.27 H_i = z·q_i·1200·f_v. ✓
