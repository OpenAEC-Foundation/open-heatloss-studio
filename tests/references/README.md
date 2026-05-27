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

## Koellast-berekeningen (Vabi Elements Koellast)

Alle PDF/XLS gitignored — privacy/copyright. Beschrijving als index voor lokale ontwikkeling.

### 5. DR Engineering Koellast Woningbouw (NEN 5060 TO2 streng)
- **PDF:** `dr-engineering-koellast-woningbouw-2024.pdf` (gitignored)
- **Rekenkern:** Vabi Elements Koellast 3.12.0.127
- **Gebouw:** Woning Ag_gekoeld 191.7 m², 6 gekoelde ruimtes
- **Peak:** 6420 W (augustus, tijdvak 14), T_setpoint 24°C, beschaduwing aan

### 6. Koellastberekeningen.nl Woning B (NEN 5060:2008 TO2 streng)
- **PDF:** `vabi-koellastberekeningen-woning-B-2024.pdf` (gitignored)
- **Rekenkern:** Vabi Elements Koellast 3.11.2.23 + rekenkern 2.09
- **Gebouw:** Woning Ag_gekoeld 182.6 m², inhoud 565.4 m³, 6 ruimtes (Keuken/Woonkamer/Eetkamer + 3 slpk)
- **Peak:** 8894 W (augustus, tijdvak 20), volledige invoer + materiaallagen + schaduwfracties
- **Detail:** 17 pp met per-ruimte invoer + maand-/uur-matrix — best gedetailleerde Vabi Koellast PDF

### 7. Statistieken-export Woning C (Vabi)
- **XLS:** `vabi-koellast-statistieken-woning-C.xls` (gitignored)
- **Andere case** dan pdf 6 — 3 ruimtes (Tuinkamer 3535W + 2 slpk = 5260W totaal voelbaar)
- Gebouw + Ruimte sheets, gestructureerde output voor parsing

### 8. DR Engineering Koellast Utiliteitsbouw (NEN 5060)
- **PDF:** `dr-engineering-koellast-utiliteitsbouw-2024.pdf` (gitignored)
- **Rekenkern:** Vabi Elements Koellast 3.12.0.127

### 9. Leever Utiliteit Horeca (historisch, NEN 5067:1985)
- **PDF:** `vabi-koellast-utiliteit-leever-2015.pdf` (gitignored)
- **XLS:** `vabi-koellast-utiliteit-leever-tabellen-2015.xls` (gitignored)
- **Rekenkern:** Vabi VA102 versie 5.35 (2015)
- **Gebouw:** Horeca utiliteit (R.A06), 169.5 m² vloer, NEN 5067 (oude norm)
- **Status:** Verouderde norm (vervangen door NEN 5060 TO2), structureel referentie

## BENG / TO-juli / NTA 8800 (Bijlage AA)

### 10. RVO Rekentool Bijlage AA NTA 8800 (2025-versie)
- **XLSM:** `rekentool-bijlage-aa-nta8800-2025.04.xlsm` (gitignored, ~580 KB)
- **Type:** Officiële RVO-rekentool — vereenvoudigde bepalingsmethode koelbehoefte
- **Sheets:** Toelichting, Projectgegevens en Resultaten, Ruimte 1-10, Tabellen, Tabel AA
- **Inzet:** **Golden master** voor toekomstige Bijlage AA-engine in `crates/nta8800-cooling`. Voor elke fixture-case: invoer in Rekentool → output naar `expected.json`

### 11. RVO BENG-voorbeeldconcepten woningbouw (DGMR rapport 2021)
- **PDF:** `rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf` (gitignored, ~840 KB)
- **Bron:** DGMR rapport B.2017.1387.02.R001 voor RVO, 26-3-2021
- **Inhoud:** 6 woningtypen × 15-18 concepten = ~93 doorgerekende cases (tussenwoning S/M, hoekwoning M/L, vrijstaand L/M, woongebouw)
- **KPIs in PDF:** BENG-1/2/3, **TO-juli per concept**, PV-vermogen, A_g, A_ls/A_g
- **Limitatie:** Bijlage 4 Excel met U-waardes/glaspercentages/oriëntatie zit NIET in PDF — wel reconstrueerbaar via Rekentool (zie 10)

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
