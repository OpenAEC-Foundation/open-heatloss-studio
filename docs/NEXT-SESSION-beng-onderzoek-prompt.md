# Prompt voor de volgende sessie — BENG/NTA 8800 onderzoek & implementatieplan

> Plak dit als openingsbericht in de volgende sessie (model: Fable).

---

## Rol & noordster

Je zet het BENG-spoor van open-heatloss-studio voort. De eindvisie: **deze tool
(open-heatloss-studio, Rust-workspace) wordt de nieuwe "energy studio"** — één tool
die zowel de ISSO 51/53-warmteverliesberekening als de **NTA 8800-BENG-berekening**
doet.

De enige noordster die telt: **een tool die 100% te vertrouwen is.** Voor een
NTA 8800-tool betekent dat: elke uitkomst (BENG 1, BENG 2, BENG 3, TO-juli,
energielabel) moet bewezen zijn tegen **gezaghebbende referenties**. Features en
architectuur zijn ondergeschikt aan correctheid. De opdrachtgever heeft expliciet
gezegd: "desnoods alles opnieuw, maar 100% te vertrouwen" — sunk cost mag de juiste
weg niet in de weg staan.

**Deze sessie schrijf je nog geen productie-code.** Je doet onderzoek en levert een
plan. In drie fasen (zie Opdracht).

## Lees dit EERST — en verifieer tegen de echte code, vertrouw geen samenvatting blind

De vorige sessie (Opus) maakte één belangrijke **framing-fout**: die nam aan dat de
BENG-engine "greenfield naar Rust geport" moest worden vanuit de TS-tool van collega
John Heikens. **Dat klopt niet.** De werkelijkheid — en jouw startpunt:

1. **`C:\Github\open-heatloss-studio`** (Rust-workspace, branch `master`) heeft **al
   ~30.000 regels Rust NTA 8800**, door de opdrachtgever zelf begonnen 2026-04-24.
   Volledige modulaire crates: `crates/nta8800-model` (rekenzone / energy_function_room
   / geometrie / klimaat), `-tables`, `-transmission`, `-ventilation`, `-heating`,
   `-cooling`, `-dhw`, `-lighting`, `-pv`, `-ep`, `-demand`, `-humidity`, `-automation`,
   `-geometry`. Service-crates zijn af + honderden tests groen. `nta8800-ep::
   calculate_ep_score()` produceert primair energiegebruik + EP-label (A++++..G) +
   service-breakdown — maar neemt `EpInputs` (al-berekende service-energieën), **geen
   gebouwmodel**. **Alleen koeling is in `crates/isso51-api` gewired** (rest niet).
2. **`TODO.md`** in die repo heeft een uitgewerkt **"Sprint v1.0 — BENG/TO-juli/
   koellast strategie"**-blok met officiële golden-bronnen: **RVO Rekentool Bijlage AA
   NTA 8800 2025.04** (`.xlsm`, golden master koelbehoefte — Bijlage AA-module al af,
   ~1300 LOC in `nta8800-cooling`) en **RVO BENG-voorbeeldconcepten woningbouw 2021**
   (DGMR-PDF, 93 doorgerekende cases incl. TO-juli). Open punt: 3 BENG-fixtures
   (Tussenwoning M / Hoekwoning M / Vrijstaande M) als goldens — eindwaarden staan in
   de PDF.
3. **`C:\Github\derden\open-energy-studio`** — John's LGPL BENG-tool (Tauri + React +
   **TypeScript-engine**). Die engine is **~1-5% van gecertificeerde Uniec 3.3.7.0** op
   woningbouw (gemeten). In `training-data/*.oes.json` staan **3 projecten met
   `meta.uniecReference`** (Gouda 2467, Aalten 2522, Kijkduin 2786 — BENG 1/2/3,
   limieten, label uit certified Uniec/BengCert). Er staat een vitest-vangrail op
   branch `feat/beng-validation-ci` (`src/core/energy/__tests__/bengValidation.test.ts`).
   **Dit is een cross-check/validatie-set, geen port-bron.**
4. **Memory `project_beng_open_energy_studio.md`** (auto-geladen) — vat het bovenstaande
   samen, inclusief de gecorrigeerde framing en 5 gemeten gaten in John's TS-engine.
5. **`docs/2026-07-11-beng-integratie-model-mapping.md`** — model-mapping-doc van de
   vorige sessie. **Let op:** de model-inzichten (oriëntatie/g-waarde/volume die het
   model nodig heeft voor zonwinst) zijn geldig; de "scaffold nta8800-core"-fasering is
   achterhaald (de crates bestaan al). Behandel het kritisch.
6. **Norm-PDF's:** NTA 8800, ISSO 75, ISSO 82 heeft de opdrachtgever. Zoek ze op
   `Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\` (+ de RVO-bronnen uit
   TODO.md). De Read-PDF-tool is kapot → gebruik PyMuPDF via Bash (`get_text` +
   pixmap-render voor formules). Instrueer subagents daar expliciet over.

**Discipline (uit eerdere sporen, hard geleerd):**
- **Validatie-eerst.** Zet de gouden fixtures (officiële referentiewaarden) op vóór je
  de engine aanraakt — zoals de isso53 §6.2-golden. Zonder vangrail is elke
  engine-wijziging blind.
- **Anti-fudge, absoluut.** Gepubliceerde/officiële waarden (RVO, certified Uniec) zijn
  grondwaarheid. Pas NOOIT een expected-waarde aan om een verkeerde engine te laten
  kloppen. Haalt de engine een term niet, of spreekt een bron zichzelf tegen → STOP,
  documenteer, en zet die term op `null`/gedocumenteerd i.p.v. weg te masseren. (Dit
  precedent is streng gehandhaafd bij isso53 §6.1/§6.2 en de reports-engine.)
- **Geen aannames.** Verifieer elke claim (ook uit dit bericht) tegen de echte code en
  de norm-bron. De vorige sessie ging de mist in juist door een aanname.
- **Respecteer bestaande conventies** in de nta8800-crates (norm-referentie-conventie,
  crate-structuur). Bouw in de stijl die er al is.
- **Rust-workspace:** `cargo test --workspace` + `cargo clippy` moeten groen blijven;
  een struct-veld toevoegen kan consumers in andere crates breken (`serde(default)`).
- **Subagent-valkuil:** spawn write/rust-agents als `general-purpose` met
  `model: "sonnet"`-override (de gepinde agent-modellen zijn kapot); nooit background
  voor write-werk.

## Opdracht — onderzoek in drie fasen, dan een plan

### Fase A — Wat staat er nu (accurate inventarisatie)
Lees de Rust `nta8800-*`-crates end-to-end (niet alleen tellingen): per service-crate
wat is geïmplementeerd, getest, en tot welke normdiepte. Breng de datastroom in kaart:
`nta8800-model` (gebouw/rekenzone/geometrie) → welke services → `EpInputs` →
`nta8800-ep`. Benoem exact **wat ontbreekt voor een end-to-end BENG-run** (het
vermoeden: er is geen orchestrator die model → alle services → EpInputs → BENG 1/2/3 +
label + TO-juli ketent; alleen koeling is in de app gewired). Check ook de bestaande
goldens (`vabi_koellast_golden`, `bijlage_aa_test`, `small_house`) en wat ze dekken.

### Fase B — Normconformiteit
Toets de bestaande engine tegen de norm en tegen de gezaghebbende referenties:
- **NTA 8800 zelf** (+ ISSO 75 utiliteit / ISSO 82 woningbouw als bepalingsprotocol):
  waar wijkt de implementatie af van de norm-formules? Welke onderdelen hebben
  norm-referenties in doc-comments, welke niet?
- **RVO-referenties:** de Rekentool Bijlage AA (koelbehoefte) en de RVO
  voorbeeldconcepten (BENG 1/2/3 + TO-juli, 93 cases) — welke zijn al als golden
  ingebouwd, welke niet? Kwantificeer waar mogelijk de afwijking van de Rust-engine.
- **Certified Uniec** (John's 3 projecten) als tweede, onafhankelijke cross-check.
Lever een eerlijk "wat is bewezen conform / wat is onbewezen / waar zijn de gaten"-beeld.
Geen wensdenken — als iets niet gevalideerd is, zeg dat.

### Fase C — Hoe implementeren/uitbreiden we de normen
Op basis van A + B: een concreet, gefaseerd plan om tot **end-to-end, tegen officiële
referenties gevalideerde BENG 1/2/3 + TO-juli + label** te komen, en de tool tot "de
energy studio" te maken. Behandel expliciet:
- De **end-to-end orchestrator** (model → services → EpInputs → BENG-indicatoren) en
  het wiren van de resterende services in `isso51-api` + frontend.
- De **golden-validatiestrategie**: de RVO-cases (Tussenwoning/Hoekwoning/Vrijstaande M)
  + certified-Uniec-projecten als Rust-goldens, met gemotiveerde toleranties.
- Het **gedeelde gebouwmodel**: heeft `nta8800-model` alles, of moet er (zoals het
  model-mapping-doc suggereert) oriëntatie/g-waarde/volume bij? Verhoudt het zich tot
  het isso5x-model of blijft het gescheiden?
- Een eerlijke afweging **voortbouwen vs. herstructureren** — de opdrachtgever staat
  open voor "alles opnieuw" als dat de betrouwbaarheid geloofwaardiger maakt. Onderbouw
  je advies.

## Deliverable
Een helder onderzoeksrapport (Fase A + B) + een gefaseerd implementatieplan (Fase C),
ter goedkeuring. Tabellen/bullets waar het kan (de opdrachtgever leest liever
structuur dan proza). Nog geen productie-code deze sessie, tenzij de opdrachtgever na
het plan groen licht geeft.
