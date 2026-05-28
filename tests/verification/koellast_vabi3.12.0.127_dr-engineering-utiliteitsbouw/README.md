# Koellast — Vabi 3.12.0.127 — DR Engineering Utiliteitsbouw

| Veld | Waarde |
|---|---|
| Norm | EN 12831 / NEN 5060 TO2 (peak cooling load) |
| Software | Vabi Elements Koellast 3.12.0.127 |
| Bron PDF | `tests/references/dr-engineering-koellast-utiliteitsbouw-2024.pdf` (1197 KB, 91 p.) |
| Gebouw | 4-laags utiliteitsgebouw (atrium + kantoren W/Z/O per laag + trappenhuis + 2 gangen), Ag_gekoeld = 1815,1 m², V = 5436 m³, 27 gekoelde ruimtes |
| Klimaat | NEN 5060 TO2 streng (extreme dag) |
| T_setpoint zomer | 24 °C (kantoren Zuid 24,0 °C; West 24,9 °C; Oost 25,6 °C door zon-instraling) |
| Beschaduwing | aan, schakelniveau handzonwering **300 W/m²** (let op: hoger dan woningbouw's 150 W/m²) |
| Status `expected.json` | ingevuld vanuit PDF — engine implementatie pending |
| Status Rust-test | `#[ignore]` op alle echte tests in `crates/nta8800-cooling/tests/vabi_koellast_golden.rs` tot peak-cooling engine bestaat |

## Status (2026-05-28)

`expected.json` ingevuld vanuit het Vabi Koellast PDF rapport p.3-4. Bevat peak W per ruimte (27 ruimtes) + gebouw-totaal + maand/tijdvak waarop het optreedt.

`input.json` ontbreekt — peak-cooling engine moet eerst bestaan voordat een 1815,1 m² project-reconstructie uit de PDF zinvol is.

**Peak-cooling engine niet beschikbaar** — `crates/nta8800-cooling/` rekent annual cooling demand (NTA 8800 H.10) in MJ, niet peak W. Voor deze fixture is een aparte engine nodig (EN 12831 / NEN 5060 TO2).

## Gebouw-peak (uit PDF p.3)

| KPI | Waarde |
|---|---|
| Peak koellast totaal | **54 016 W** |
| Peak voelbaar | 46 715 W |
| Peak latent | 7 301 W |
| Peak maand | augustus |
| Peak tijdvak | 17 (17:00) |

## Peak koellast per ruimte (uit PDF p.3-4, "Maximale koellast per ruimte")

27 ruimtes verdeeld over 4 lagen (0./1./2./3.). Atrium alleen op laag 0/2/3 (niet 1). Kantoren W/Z/O per laag. Trappenhuis + 2 gangen per laag.

Per-ruimte peaks treden op verschillende tijdstippen op — sommatie van room-peaks (~61 549 W) is hoger dan de gebouw-peak (54 016 W op augustus-17), omdat het gebouw-peak op één moment is en de meeste kantoren hun individuele peak in juli halen.

Type-codes: **VKR** = verblijfskoelruimte (atrium / circulatie), **VG** = verblijfskoelgebied (kantoor).

## Negatieve "koellast" in PDF p.3 (Totalen-tabel)

In de Totalen-tabel op p.3 (gebouw-peak op augustus-17) geven Trappenhuis (0.06, 1.06, 2.06, 3.06) en Gangen (0.07, 0.08, 1.07, 1.08, 2.07, 2.08, 3.07, 3.08) negatieve waarden (-163 t/m -373 W). Dat is **warmtebehoefte i.p.v. koelbehoefte** op dat tijdstip: deze ruimtes hebben geen interne warmtelast + weinig zon en zouden op augustus-17 17:00 juist een beetje verwarming nodig hebben.

In de individuele "Maximale koellast per ruimte"-tabel (p.3-4) staan voor diezelfde ruimtes wél positieve waarden — dat zijn de momenten waarop ze hun eigen peak halen (typisch mei-tijdvak 15 of juli-tijdvak 15 met zon door dakraam / westgevel). Voor `expected.json` zijn de individuele peaks gebruikt, niet de negatieve waarden uit de Totalen-tabel.

## Verschil met NTA 8800 H.10 (annual TO-juli)

| Aspect | Deze fixture (Koellast) | NTA 8800 H.10 (TO-juli) |
|---|---|---|
| Output-eenheid | W (vermogen) | MJ / kWh (energie per jaar) |
| Tijdresolutie | uur (tijdvak 8-20) | maand (12 waardes) |
| KPIs | Max koellast/ruimte, totaal gebouw 54 016 W | Q_C;use [MJ], H_T [W/K], H_V [W/K], τ [h] |
| Klimaat | NEN 5060 ref TO2 streng (extreme dag) | De Bilt referentiejaar |
| Doel | Dimensioneren koelinstallatie | BENG-2 toets TO-juli ≤ 1,20 (alleen woningbouw) |
| Crate-engine | nog niet beschikbaar | `nta8800-cooling` (annual) — alleen woningbouw |

## Verschil met woningbouw-fixture

| Aspect | Utiliteitsbouw (deze fixture) | Woningbouw |
|---|---|---|
| Ag_gekoeld | 1815,1 m² | 191,7 m² |
| Volume | 5436 m³ | 414,6 m³ |
| Aantal ruimtes | 27 (VKR + VG) | 6 (woonkamer / keuken / 3 slaapkamers / speelzolder) |
| Peak totaal | 54 016 W | 6 420 W |
| Peak tijdstip | augustus tijdvak 17 | augustus tijdvak 14 |
| Schakelniveau zonwering | 300 W/m² | 150 W/m² |
| Negatieve waarden in totalen-tabel | ja (trappenhuis / gangen op aug-17) | nee |

## Files

- `expected.json` — peak W per ruimte (27) + gebouw + maand/tijdvak (ingevuld vanuit PDF)
- `README.md` — dit bestand
- `reference.pdf` — niet in deze folder; staat in `tests/references/dr-engineering-koellast-utiliteitsbouw-2024.pdf`

Niet aanwezig (TBD):
- `input.json` — wacht op peak-cooling engine + 1815,1 m² project-reconstructie

## Volgende stappen

1. **Peak-cooling engine ontwerpen** — EN 12831 / NEN 5060 TO2 logic, apart van `nta8800-cooling` (annual TO-juli). Nieuwe crate `crates/peak-cooling/` voorgesteld. Deze utiliteitsbouw-case is bewust groter (27 ruimtes, 4 lagen, atrium) dan de woningbouw-case zodat de engine direct getoetst wordt op zone-aggregatie en multi-laag oriëntatie.
2. **`input.json` reconstrueren** — 1815,1 m² gebouw matchen aan constructies/oriëntaties/beschaduwing zoals beschreven verderop in de PDF (p.5-91: per-ruimte detailpagina's).
3. **`#[ignore]` weghalen** van tests in `crates/nta8800-cooling/tests/vabi_koellast_golden.rs` zodra engine + input.json klaar zijn (of verplaatsen naar de nieuwe `peak-cooling` crate).

## PDF-detail (91 p. Vabi Elements Koellast)

- p.1-2: project + uitgangspunten
- p.3-4: totalen gebouw (per-ruimte op gebouw-peak) + per-ruimte max + maand/tijdvak overzicht
- p.5-90: per-ruimte detail (27 ruimtes × ~3 pagina's: koellast/uur, deelresultaten transmissie/zon/intern, constructies, schaduwfracties)
- p.91: bedrijfsgegevens DR Engineering
