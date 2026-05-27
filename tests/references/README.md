# Referentie-berekeningen voor validatie

Verzameling van warmteverliesberekeningen als referentiedata voor het testen van de ISSO 51 rekenengine.

## Bestanden

### 1. Vabi Woonhuis A (ISSO 51:2017)
- **PDF:** `vabi-woonhuis-A-isso51-2017.pdf` (lokaal, gitignored)
- **Samenvatting:** `vabi-woonhuis-A-samenvatting.md`
- **Norm:** ISSO 51, 53, 57 (2017)
- **Rekenkern:** Vabi 3.9.1.2
- **Type:** Vrijstaande woning, 16 vertrekken, vloerverwarming, systeem C
- **theta_e:** -9,0 C (basis -10 + 1K tijdconstantecorrectie)
- **Totaal:** 10.784 W (vertrekken), aansluitvermogen 12.564 W

### 2. DR Engineering Woningbouw (ISSO 51:2024)
- **PDF:** `dr-engineering-woningbouw-isso51-2024.pdf`
- **Samenvatting:** `dr-engineering-samenvatting.md`
- **Norm:** ISSO 51, 53, 57 (2024)
- **Rekenkern:** Vabi 3.12.0.127
- **Type:** Vrijstaande woning met garage, 14 vertrekken, radiatoren LT, systeem D met WTW
- **theta_e:** -8,0 C (basis -10 + 2K tijdconstantecorrectie)
- **Totaal:** 6.700 W (gebouw, kwadratische sommatie)

### 3. Vrijstaande woning (ISSO 51:2017)
- **PDF:** `vrijstaande-woning-isso51-2017.pdf`
- **Norm:** ISSO 51, 53, 57 (2017)
- **Rekenkern:** Vabi 3.8.1.14

### 4. Erratum ISSO 51:2023
- **PDF:** `erratum-isso51-2023.pdf`
- **Samenvatting:** `erratum-isso51-2023-samenvatting.md`
- **Alle correcties** op de originele ISSO 51:2023 publicatie

## Belangrijkste normdifferences (2017 vs 2023/2024)

| Aspect | 2017 | 2023/2024 |
|--------|------|-----------|
| theta_b aangrenzend (wonen) | 15 C | 17 C |
| theta_b aangrenzend (overig) | variabel | 14 C |
| Bodemtemperatuur | 9 C | 10,5 C |
| Thermische brug (nieuw, voorzien) | 0,05 W/(m2.K) | 0,02 W/(m2.K) |
| Infiltratie op basis van | geveloppervlak + Z-factor | Ag (gebruiksoppervlak) |
| Niet-gelijktijdige verliezen | lineaire sommatie | kwadratische sommatie |
| Vloerverwarming tussenvloer | verlies meegerekend | geen verlies binnen woning |
| Standaard theta_i verblijf | 20 C | 22 C |
| Zekerheidsklasse | opgeven | vervalt |
