# Bijlage AA cross-validatie — sample case 1

**Status:** ✅ groen sinds 2026-05-28 (`golden_master_xlsm_cross_validatie` test). Onze engine matcht de RVO-rekentool xlsm binnen 0.07% (max 0.26 W op 377 W) voor sample case 1.

Deze instructie blijft staan als reproductie-handleiding wanneer xlsm of engine wijzigt en cross-validatie opnieuw moet draaien.

## Bestand

`tests/references/bijlage-aa-sample-case1-slaapkamer-zuid.xlsm` (gitignored, lokaal)

## Sample-case ontwerp

Minimale 1-slaapkamer-woning, gericht op alle 5 hoofd-formules (P_int, P_V, P_tr;ntr, P_sol, P_tr;gl) zonder edge-cases:

| Parameter | Waarde |
|---|---|
| Project | Sample Case 1 — Single Bedroom Zuid |
| Bouwjaar (B14, **dropdown-string!**) | `vanaf 2015` (→ f_iso = 2.2 W/m²) |
| Nageisoleerd | Nee |
| Gebouw-oriëntatie voorzijde | Zuid |
| Infiltratie q_v,eff,lea,in | 5 m³/h |
| Natuurlijke toevoer q_v,eff,vent,in | 0 m³/h |
| Mechanische toevoer q_v,mech,in | 20 m³/h |
| A_g rekenzone | 12 m² |
| Aantal woonfuncties | 1 |

| Ruimte 1 | Waarde |
|---|---|
| Naam | Slaapkamer 1 |
| Type | Andere verblijfsruimte |
| Vloeroppervlak A_vr | 12 m² |
| Voorgevel (Zuid) — Grenst aan buitenlucht | Ja |
| Voorgevel — hellingshoek | 90° |
| Voorgevel — lengte × hoogte | 3.5 × 2.6 = 9.1 m² |
| Voorgevel — Glasvlak type 1 A_w | 2.0 m² |
| Voorgevel — U-waarde glas | 1.2 W/m²·K (HR++ alu) |
| Voorgevel — g-waarde glas | 0.6 |
| Voorgevel — Beschaduwing | Minimale belemmering |
| Voorgevel — Zonwering | Geen |
| Voorgevel — Overstek / zijbelemmering | 0 (geen) |
| Andere 3 gevels + platdak | Grenst aan buitenlucht = Nee (default) |

## Stappen voor jou

### 1. Open Excel
```
"C:\Github\open-heatloss-studio\tests\references\bijlage-aa-sample-case1-slaapkamer-zuid.xlsm"
```

### 2. Macro's inschakelen
Excel zal vragen of macro's te activeren — **klik "Inhoud inschakelen"**. De tool gebruikt VBA UDF's (`o_Orientatie`) voor automatische gevel-rotaties.

### 3. Trigger volledige herberekening
`Ctrl+Alt+F9` (full recalc, omdat het bestand net geopend wordt en geen "dirty" markers heeft die F9 zou triggeren).

### 4. Lees outputs uit twee plekken

**A. Sheet "Ruimte 1" — per-ruimte resultaten** (regel 55-64):
- B53: Buitenluchttemperatuur op tijdstip max koellast θ_e [°C]
- B55: Koellastbijdrage transmissie ondoorzichtige delen P_tr;ntr;vr [W] (Voorgevel)
- B56: Koellastbijdrage zoninstraling transparante delen P_sol;vr [W]
- B57: Koellastbijdrage transmissie transparante delen P_tr;gl;vr [W]
- B58: Totaal koellastbijdrage per gevel [W]
- B60: Koellast door interne warmtelast P_int;calc [W]
- B61: Koellast door buitenluchttoetreding P_v;calc [W]
- B63: **Totaal koellastbijdrage [W]** ← hoofdgetal
- B64: **Koelbehoefte verblijfsruimte [W/m²]** ← hoofdgetal

**B. Sheet "Projectgegevens en Resultaten" — gebouw-totalen** (regel 37 en volgende):
- B33: Rekenwaarde internewarmtelast q_int;calc;zi [W/m²]
- B43: Koellast door interne warmtelast (Ruimte 1) [W]
- B44: Koellast door buitenluchttoetreding (Ruimte 1) [W]
- B45: Koellast door transmissie ondoorzichtige delen (Ruimte 1) [W]
- (en eventuele eindwaarde q_C totaal gebouw — scroll naar onder voor de regel `q_C` of `Koelbehoefte rekenzone`)

### 5. Koppel terug

Stuur me het volgende lijstje terug (kopiëren / screenshot / typed):

```
Ruimte 1 outputs:
  B53 (θ_e max) = ___ °C
  B55 (P_tr;ntr Voorgevel) = ___ W
  B56 (P_sol Voorgevel) = ___ W
  B57 (P_tr;gl Voorgevel) = ___ W
  B58 (Totaal per gevel Voorgevel) = ___ W
  B60 (P_int;calc) = ___ W
  B61 (P_v;calc) = ___ W
  B63 (Totaal koellastbijdrage) = ___ W
  B64 (Koelbehoefte verblijfsruimte) = ___ W/m²

Projectgegevens outputs:
  B33 (q_int;calc;zi) = ___ W/m²
  (eindwaarde q_C totaal als zichtbaar) = ___ W/m²
```

## Wat ik dan doe

Met die outputs:
1. Update de `XlsmGoldenMaster` struct-init in `golden_master_xlsm_cross_validatie` test (`crates/nta8800-cooling/tests/bijlage_aa_test.rs`).
2. Run `cargo test -p nta8800-cooling --test bijlage_aa_test golden_master_xlsm_cross_validatie -- --nocapture`.
3. Bij afwijking: diagnose welke formule fout zit in onze engine (zie 2026-05-28 sessie voor F_F=0.9 en B14-dropdown bugs).

## Gotchas (uit 2026-05-28 sessie)

1. **B14 moet dropdown-string zijn, niet integer.** VBA `o_F_Iso(i_Bouwjaar As String, ...)` matcht alleen op `"tot 1975"` / `"1975 t/m 1991"` / `"1992 t/m 2014"` / `"vanaf 2015"`. Integer 2020 in B14 → geen match → return 0 → P_tr;ntr fout.
2. **Excel COM via Python werkt niet** voor deze xlsm (Office Group Policy blokkeert macro-files via automation). Recalc moet handmatig in Excel UI.
3. **`scripts/recalc_bijlage_aa_xlsm.py`** is een poging die faalt — bewaard ter referentie. Werkende route is handmatig: Excel open → Ctrl+Alt+F9 → cellen kopiëren naar de test.
4. **`scripts/patch_b14_dropdown.py`** zet B14 correct via openpyxl, gebruik dit als de xlsm ooit gereset wordt.
5. **`scripts/extract_vba_f_iso.py`** extract VBA-source voor diagnose (oletools nodig: `pip install oletools`).

## Waarom deze case?

- **Minimaal** — 1 verblijfsruimte, 1 buitengevel, 1 raam
- **Niet-nul outputs** — voorgevel Zuid met raam → alle 5 hoofd-formules actief
- **Reproduceerbaar** — invoer staat hierboven, kun je ook in een vers tool-exemplaar handmatig invullen
- **Engine-coverage** — raakt P_int, P_V, P_tr;ntr, P_sol, P_tr;gl én capaciteits-aggregatie

Latere sample-cases kunnen edge-cases dekken (overstek, zonwering, meerdere ruimtes, woonvertrek vs overig, hoekgevel met N-O, etc.). Maar deze ene case is genoeg om de hoofdas te valideren.
