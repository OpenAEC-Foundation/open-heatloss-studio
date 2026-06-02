# ISSO 53 — geverifieerde formule-referentie (voor calc-rewrite)

> Formules + tabelwaarden geëxtraheerd uit de ISSO 53-PDF (beeld-render, 200 dpi, 2026-06-02) omdat de tekstlaag alleen `[-] (4.x)`-placeholders gaf. Alleen functionele formules/data — geen norm-tekst gekopieerd. Bron: ISSO-publicatie 53, hoofdstuk 4. Gebruik dit voor A4/A5/A7-implementatie i.p.v. de PDF opnieuw te renderen.

## Tabel 2.3 — temperatuurcorrecties (p.21-22, max. hoogte 4 m)

| Verwarmingssysteem | Δθ₁ resp. Δθ_a1 [K] | Δθ₂ resp. Δθ_a2 [K] | Δθ_v (R_c<3,5) [K] | Δθ_v (R_c≥3,5) [K] |
|---|---|---|---|---|
| Lokale verwarming | +4 | -1 | 0 | 0 |
| Radiatoren/conv. ht + luchtverwarming | +3 | -1 | 0 | 0 |
| Radiatoren/conv. lt | +2 | -1 | 0 | 0 |
| Plafondverwarming | +3 | 0 | 0 | 0 |
| Wandverwarming | +2 | -1 | -1 | -0,5 |
| Plintverwarming | +1 | -1 | 0 | 0 |
| Vloerverwarming + ht radi/conv | +3 | 0 | 0 | 0 |
| Vloerverwarming + lt radi/conv | +2 | 0 | -1 | -0,5 |
| Vloerverwarming als hoofdverwarming | 0 | 0 | -1 | -0,5 |
| Vloerverwarming + wandverwarming | +1 | 0 | -1 | -0,5 |
| Betonkernactivering | 0 | 0 | -1 | -0,5 |
| Ventilatorgedreven conv./radi | 0,5 | 0 | 0 | 0 |

- **R_c** (voetnoot 4) = oppervlakte-gewogen gemiddelde R_c van de uitwendige scheidingsconstructies.
- **Voetnoot 2 (vide):** bij vides etc. die een grotere hoogte geven → Δθ₁ resp. Δθ_a1 **× (h/4)**, met h = totale hoogte [m].
- Δθ_v = 0 voor alle systemen met toevoertemperatuur hoger dan θ_i (bv. luchtverwarming). Indien systeem nog onbekend → Δθ_v = 0.
- Code-status: alleen Δθ₂ geïmplementeerd (`tables/temperature_stratification.rs::delta_theta_2`, 12/12 correct). Δθ₁, Δθ_v, Δθ_a1, Δθ_a2 ontbreken.

## A5 — waar Δθ₁ / Δθ₂ ingaan

| Formule | Toepassing | Vorm |
|---|---|---|
| 4.14 (wanden) | onverwarmde ruimte, wand | `f_k = (θ_i − θ_a)/(θ_i − θ_e)` (geen Δθ) |
| 4.15 (vloeren) | onverwarmde ruimte, vloer | `f_k = (θ_i + Δθ₁ − θ_a)/(θ_i − θ_e)` |
| 4.16 (plafonds) | onverwarmde ruimte, plafond | `f_k = (θ_i + Δθ₁ − θ_a)/(θ_i − θ_e)` |
| 4.5/4.6 | vloer boven buitenlucht / plat dak | Δθ₁ (en Δθ₂) idem patroon |
| 4.11/4.12 | aangrenzend vertrek (wand/plafond) | Δθ₁, Δθ_a1, Δθ₂, Δθ_a2 |
| 4.19/4.20 | aangrenzend gebouw (vloer/plafond) | Δθ₁, Δθ₂ |
| 4.22 (grond-wand) | grond | `f_ig = (θ_i − θ_me)/(θ_i − θ_e)` (geen Δθ) |
| 4.23 (grond-vloer) | grond | `f_ig = (θ_i + Δθ₂ − θ_me)/(θ_i − θ_e)` ✅ code correct |

Effect Δθ₁ op dak/vloer-boven-buitenlucht: bij radiatoren-ht (Δθ₁=+3), θ_i=20, θ_e=−10 → factor (20+3+10)/(20+10)=33/30 = **+10%**.

## A4 — formule 4.24 (U_equiv,k bepaling, §4.6, p.44)

```
U_equiv,k = a · b / ( c₁·(B')^n₁ + c₂·(U_k + ΔU_TB)^n₂ + c₃·(z)^n₃ + d )
```
Waarin:
- `U_k` = U-waarde wand/vloer in contact met grond [W/m²K]
- **`ΔU_TB`** = toeslag thermische bruggen volgens Tabel 3.1 [W/m²K] — **deze ontbreekt nu in `ground.rs:48` (rauwe `element.u_value` doorgegeven)**
- `B'` = geometrische factor = 2·A_vl/O, geclampt [2, 50]; voor wanden valt B'-term weg (c₁=0, n₁=0)
- `z` = diepte vloer onder maaiveld, 0≤z≤5 m (z>5 → z=5)
- `a,b,c,d,n` uit Tabel 4.3; resultaat clampen U_equiv ≥ 0,1

**Tabel 4.3 — parameters U_equiv:**

| | a | b | c₁ | c₂ | c₃ | n₁ | n₂ | n₃ | d |
|---|---|---|---|---|---|---|---|---|---|
| **Vloer** | 0,9671 | -7,455 | 10,76 | 9,773 | 0,0266 | 0,5532 | 0,6027 | 0,9296 | -0,0203 |
| **Wand** | 0,799 | -6,7951 | 0¹ | 26,586 | 0,1523 | 0¹ | 0,5012 | -0,1406 | -1,074 |

¹ Voor wanden c₁=n₁=0 (B'-term vervalt); B' mag niet 0 zijn (rekenkundige integriteit).

> Actie A4: voeg ΔU_TB toe aan `U_k` vóór `calculate_u_equivalent`, met dezelfde forfaitair/custom-prioriteit als `transmission.rs` (zie A6-fix). Verifieer tegelijk de bestaande `ground_params.rs`-implementatie tegen bovenstaande exacte 4.24 + Tabel 4.3 (de waarden waren eerder OCR-onleesbaar — nu bevestigd).

## A7 — formule 4.39 (f_v ventilatie/infiltratie)

```
H_v = q_v · 1200 · f_v            (4.37, NL-condities ρ·c_p=1200)
WTW/voorverwarming (4.38):  f_v = (θ_i + Δθ_v − θ_t)/(θ_i − θ_e)
Nat. toevoer / mech. toevoer zonder voorverwarming (4.39):
                            f_v = (θ_i + Δθ_v − θ_e)/(θ_i − θ_e)
```
- `θ_t` = toevoertemperatuur ventilatielucht (§4.7.3)
- `Δθ_v` uit Tabel 2.3 (kolom afhankelijk van R_c<3,5 vs ≥3,5)
- Code-status: `ventilation.rs` hardcodet f_v=1,0 voor de natuurlijke/geen-voorverwarming tak → klopt alleen als Δθ_v=0. Bij straling/vloer/wand-verwarming (Δθ_v=−1 of −0,5) → ~3% overschatting. Idem infiltratie (form. 4.30 gebruikt dezelfde Δθ_v).

## Tabel 4.2 — f_k onverwarmde ruimten (15 varianten, p.41) — voor UI-gap U2

| Onverwarmde ruimte | f_k |
|---|---|
| Vertrek/groep aangrenzende ruimten — 1 externe scheidingsconstructie/buitenwand | 0,4 |
| — 2 externe scheidingsconstructies, zonder buitendeur | 0,5 |
| — 2 externe scheidingsconstructies, met buitendeur | 0,6 |
| — 3 of meer externe scheidingsconstructies | 0,8 |
| Kelder — zonder ramen/deuren in externe scheidingsconstructies | 0,5 |
| Kelder — met ramen/deuren in externe scheidingsconstructies | 0,8 |
| Ruimte onder het dak — hoog geïnfiltreerd (bijv. pannendak zonder folielaag) | 1,0 |
| — overige niet-geïsoleerde daken | 0,9 |
| — geïsoleerde daken | 0,7 |
| Gemeenschappelijke verkeersruimte — interne ruimte zonder buitenwanden + ventilatievoud <0,5 | 0,0 |
| — vrij geventileerd (A_opening/V > 0,005) | 1,0 |
