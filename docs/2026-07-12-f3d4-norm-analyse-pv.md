# F3d-4 — Norm-analyse PV-opbrengst (NTA 8800:2025+C1:2026 H.16 + Tabel 17.2)

**Datum:** 2026-07-12
**Scope:** correcte tilt/azimut-afhankelijkheid van de PV-opbrengst + koudebrug-propagatie.
**Bron:** NTA 8800:2025+C1:2026 (`Z:/…/98_normen/NTA 8800_2025+C1_2026 nl.pdf`), PyMuPDF-transcriptie met pixmap-verificatie bij twijfel.

## Kernbevinding — er is géén "tilt/azimut-correctiefactor" in H.16

De F3d-3-diagnose ging uit van "Tabel 16.2 = azimut-correctiefactoren" (zoals de fictieve `references.rs`-constanten suggereerden). **Dat klopt niet.** H.16 kent de volgende tabellen (PDF p. 680-681):

| Tabel | Werkelijke inhoud (PDF-pagina) | Waarden |
|---|---|---|
| 16.1 | Piekvermogen `Kpk` [W/m²] per zonnestroompaneeltype (p. 680) | 55-175 W/m² (mono/multi-Si, dunne film) |
| 16.2 | Opbrengstfactor `f_perf` naar bouwintegratie/ventilatie (p. 681) | 0,76 / 0,80 / 0,82 |
| 16.3 | Schaduwcorrectie `c_sh;PV` vs `F_sh;obst` (p. 681) | 1,00 → 0,75 |

Geen van deze drie draagt tilt of azimut. De opbrengstformule is (PDF p. 677-678):

- **(16.2)** `E_el;PV;out;i,mi = E_sol;mi · P_pk;i · f_perf;i · c_sh,PV;mi;i · f_prac,PV;i / I_ref`, met `f_prac = 0,95`, `I_ref = 1 kW/m²`.
- **(16.3)** `E_sol;mi = I_sol;mi · t_mi · F_sh;obst;mi / 1000` [kWh/m²].

De **volledige** hellingshoek- en oriëntatie-afhankelijkheid zit in de keuze van `I_sol;mi` uit **Tabel 17.2** — expliciet benoemd in OPMERKING 2 bij formule 16.3 (PDF p. 678): *"Tabel 17.2 geeft de totale zonnestraling (I_sol,mi) voor verschillende oriëntaties (γ) en hellingshoeken (β)."*

## Tabel 17.2 — I_sol;mi [W/m²] per β en γ (De Bilt, ρ = 0,2), PDF p. 690-693

- **Hellingshoeken β:** 0° (horizontaal), 30°, 45°, 60°, 90°, 135°, 180° (horizontaal omlaag).
- **Oriëntaties γ:** Z (180°), ZW (225°), W (270°), NW (315°), N (360°), NO (45°), O (90°), ZO (135°). β = 0° en β = 180° zijn oriëntatie-onafhankelijk ("–"-kolom).

**Selectie-/interpolatieregels (PDF p. 693, letterlijk):**
1. Tussenliggende oriëntatie → waarde bij de **dichtstbijzijnde** oriëntatie; exact tussen twee kolommen → de **hoogste** naastliggende waarde.
2. Tussenliggende hellingshoek → **lineair interpoleren** tussen de tabelwaarden.

**Steekproef-verificatie (getranscribeerd → PDF):**

| β | γ | Maand | I_sol [W/m²] |
|---|---|---|---|
| 0° | – (horizontaal) | jan | 28,0 |
| 30° | Z | jun | 211,2 |
| 30° | W | jul | 180,2 |
| 45° | Z | apr | 189,7 |
| 90° | Z | jan | 60,1 |
| 90° | N | jun | 73,0 |
| 135° | ZO | dec | 19,0 |
| 180° | – (omlaag) | dec | 4,2 |

Volledige transcriptie in `crates/nta8800-pv/src/tables/irradiation.rs` (provenance in de module-doc, per β-blok gemarkeerd op paginanummer).

## Waarom de oude V1-benadering fout was

`calculate_tilt_azimuth_factor` deed `f = cos(β − 35°) · cos((γ − 180°)/2)`, geschreven voor een 0-360-conventie met zuid = optimum. Na de `map_pv`-azimutnormalisatie (0-360 → −180..180; west 270° → −90°) werd de azimut-term `cos((−90 − 180)/2) = cos(−135°) < 0`, door de `.max(0.0)`-clamp op **0** gezet. Noord gaf zelfs mét wrap ~0. Gevolg: west- en noord-strings leverden niets.

De nieuwe implementatie ([`tables::tilt_azimuth_factor`]) vermenigvuldigt de horizontale maand-instraling met de tabel-verhouding `I_sol(β, γ, mi) / I_sol(0°, mi)` — norm-conform, maand-afhankelijk, en zonder clamp. Verificatie: noord-15° geeft nu een strikt positieve factor (~0,77 in de zomer), west-30° volgt `180,2 / 191,0` in juli.

## Koudebruggen — verliespunt-diagnose

Onafhankelijk van PV: de `thermalBridges` uit de Uniec-bron (ψ + lengte) kwamen nooit in H_T. **Verliespunt:** `crates/openaec-project-shared/src/tojuli.rs` gaf een harde `let thermal_bridges_linear = Vec::new();` door aan `calculate_transmission` — níet `zone.thermal_bridges_linear` uit de view. Zowel de TO-juli- als de BENG-keten lopen via `compute_tojuli_full`, dus dit ene punt blokkeerde beide. (De `nta8800_view`-Rekenzone draagt alleen ID-strings die de keten niet resolvt; dat was een tweede, niet-load-bearing gap.)

Fix: `SharedGeometry.thermal_bridges: Vec<ThermalBridge>` (ψ + lengte), gepropageerd naar `ThermalBridgeLinear` en opgeteld bij H_D (`Σ ψ·L`, formule 8.1, §8.2.3).
