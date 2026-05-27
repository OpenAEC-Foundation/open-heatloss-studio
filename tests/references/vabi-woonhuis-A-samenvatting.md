# Vabi Woonhuis A — ISSO 51/53/57 (2017)

**Rekenkern:** Vabi 3.9.1.2 | **Datum:** 2023-Q2 | **Norm:** ISSO 51, 53, 57 (2017)

## Projectparameters

| Parameter | Waarde |
|---|---|
| Soort gebouw | Woning/woongebouw |
| Bouwwijze | Vrijstaand |
| Bruto inhoud | 594 m3 |
| Tijdconstante | 100,8 h |
| Basis theta_e | -10,0 C |
| Correctie tijdconstante | +1,0 K |
| **theta_e ontwerp** | **-9,0 C** |
| Thermische bruggen | Nieuw gebouw (toeslag 0,05 W/(m2.K)) |
| qv,10 | 1,000 dm3/(s.m2 Ag) |
| Ventilatiesysteem | C (alle ruimten) |
| Verwarming | Vloerverwarming (alle ruimten) |
| Nachtverlaging | 2K, opwarmtijd 2 uur |
| Gebouwzwaarte | Licht/middelzwaar |
| Bodemtemperatuur | 9,0 C |
| Bruto omtrek | 50,6 m |
| Bruto vloeroppervlakte | 140,1 m2 |

## Zonetotalen

| Component | W |
|---|---|
| Transmissie | 4.085 |
| Ventilatie | 4.523 |
| Opwarmtoeslag | 2.176 |
| **Ruimtetotaal** | **10.784** |
| Additioneel vloerverwarming | 1.936 |
| **Aansluitvermogen** | **12.564** |

## Aansluitvermogen detaillering

| Component | W |
|---|---|
| Transmissie naar buiten | 4.070 |
| Transmissie naar onverwarmd | 15 |
| Transmissie naar aangrenzend | 0 |
| Transmissie naar bodem | 0 |
| Infiltratie | 435 |
| Ventilatie | 3.932 |
| Bedrijfsbeperking | 2.176 |
| Additioneel vloerverwarming | 1.936 |
| Voorverwarmer ventilatie | 0 |
| **Aansluitvermogen** | **12.564** |
| Fractie z woningbouw | 0,7 |

## Ruimte-overzicht

| # | Naam | theta_i [C] | Trans. [W] | Vent. [W] | Opwarm [W] | Totaal [W] | W/m2 |
|---|---|---|---|---|---|---|---|
| 0.01 | Hal | 20 | 355 | 156 | 121 | 632 | 68 |
| 0.02 | Kast | 20 | -15 | 0 | 15 | 0 | 0 |
| 0.03 | Woonkamer | 22 | 895 | 1.182 | 466 | 2.543 | 71 |
| 0.04 | Inloopkast | 20 | -115 | 37 | 94 | 16 | 2 |
| 0.05 | Slaapkamer | 22 | 339 | 336 | 133 | 808 | 79 |
| 0.06 | Badkamer BG | 22 | 49 | 46 | 49 | 144 | 38 |
| 0.07 | Speelkamer | 22 | 263 | 347 | 137 | 748 | 71 |
| 0.08 | Leefkeuken | 22 | 778 | 928 | 366 | 2.073 | 74 |
| 0.09 | Portaal | 20 | 227 | 121 | 94 | 443 | 61 |
| 0.10 | Techniek | 20 | -26 | 27 | 46 | 47 | 13 |
| 0.11 | Toilet | 20 | 67 | 47 | 20 | 133 | 87 |
| 1.01 | Overloop | 20 | -112 | 30 | 84 | 2 | 0 |
| 1.02 | Slaapkamer 1 | 22 | 328 | 310 | 122 | 761 | 81 |
| 1.03 | Slaapkamer 2 | 22 | 410 | 380 | 150 | 941 | 81 |
| 1.05 | Slaapkamer 3 | 22 | 365 | 414 | 163 | 942 | 74 |
| 1.06 | Badkamer 1e | 22 | 261 | 161 | 130 | 552 | 55 |
| **Totaal** | | | **4.085** | **4.523** | **2.176** | **10.784** | **64** |

## Opwarmtoeslag per ruimte

Alle ruimten: 13,0 W/m2 (licht/middelzwaar gebouw, nachtverlaging 2K, opwarmtijd 2 uur).

## Kerngetallen ventilatie

- **Infiltratie:** 0,000170 m3/s per m2 buitenoppervlak (ruimten zonder mech. toevoer)
- **Ventilatie verblijfsgebieden:** 0,000900 m3/s per m2 vloeroppervlak (= 0,9 dm3/s per m2)
- **Correctiefactor binnenwand 20->22 C:** -0,069
- **Correctiefactor binnenwand 22->20 C:** +0,065
- **Reductiefactor z:** 1,0 (BG), 0,5 (overloop 1.01), 0,7 (slk2/slk3)

## Constructie U-waarden

| Constructie | U [W/(m2.K)] | dUtb |
|---|---|---|
| Wand buiten bestaand (Rc=2.5) | 0,43 | 0,05 |
| Wand buiten nieuw (Rc=5.4) | 0,18 | 0,05 |
| Wand binnen KZS 100mm | 2,78 | — |
| Dak hellend sandwichpaneel (Rc=6.4) | 0,15 | 0,05 |
| Dak plat beton (Rc=6.4) | 0,15 | 0,05 |
| Vloer BG bestaand (Rc=0.2) | 0,49 | — |
| Vloer BG beton (Rc=4.2) | 0,49 | — |
| Tussenvloer beton | 2,48 | — |
| Raam Alu HR++ (U=1,1) | 1,37 | 0,05 |
| Raam Alu HR+++ (U=0,6) | 0,92 | 0,05 |
