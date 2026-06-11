/**
 * Help — sectie "Formules".
 *
 * Per rekenonderdeel de hoofdformule + symboolverklaring + norm-referentie.
 * Bron: doc-comments in `crates/isso51-core/src/calc/*.rs` en
 * `docs/2026-05-12-isso51-norm-conformiteit-audit.md`. Formules in
 * unicode/subscript-notatie, consistent met de rest van de app.
 */
import { Card } from "../../components/ui/Card";
import { FormulaBlock, SymbolList } from "./primitives";

export function HelpFormules() {
  return (
    <div className="flex flex-col gap-4">
      <Card title="Transmissie (Φ_T)">
        <FormulaBlock
          formula="Φ_T = (H_T,ie + H_T,ia + H_T,io + H_T,ib + H_T,ig) × (θ_i − θ_e)"
          reference="ISSO 51:2023 §2.5.1–§2.5.5, Formule 4.2"
        />
        <p className="mb-2 text-sm text-on-surface-secondary">
          De specifieke warmteverliescoëfficiënt H_T wordt per begrenzingstype
          opgebouwd:
        </p>
        <FormulaBlock
          formula="H_T,ie = Σ (A_k × f_k × (U_k + ΔU_TB))"
          reference="Naar buitenlucht — Formule 4.3a. ΔU_TB = forfaitaire thermische-brugtoeslag (default 0,10 W/(m²·K), per element aanpasbaar)."
        />
        <FormulaBlock
          formula="H_T,ia = Σ (A_k × U_k × f_ia,k)   met   f_ia = (θ_i − θ_a) / (θ_i − θ_e)"
          reference="Naar aangrenzende vertrekken — §2.5.3, Formule 2.17/4.6. Voor plafonds/vloeren met Δθ₁/Δθ₂-stratificatiecorrectie uit Tabel 2.12."
        />
        <FormulaBlock
          formula="H_T,io = Σ (A_k × U_k × f_k)"
          reference="Naar onverwarmde ruimtes — Formule 4.10. b-factor f_k default 0,5 als geen expliciete temperatuurfactor is gezet."
        />
        <FormulaBlock
          formula="H_T,ib = c_z × Σ (A_k × U_k × f_b)   met   f_b = (θ_i − θ_b) / (θ_i − θ_e)"
          reference="Naar buurpanden — Formules 4.14–4.17. θ_b = 17 °C woonfunctie / 14 °C overig (erratum 2023); plafond/vloer met Δθ₁/Δθ₂-correctie."
        />
        <FormulaBlock
          formula="H_T,ig = 1,45 × G_w × Σ (A_k × f_g2 × U_e,k)"
          reference="Naar de grond — Formule 4.18. G_w = grondwaterfactor, U_e = equivalente U-waarde, f_g2 = temperatuur-reductiefactor."
        />
        <FormulaBlock
          formula="H_T,iw = Σ (A_k × U_k × f_w)   met   f_w = (θ_i − θ_w) / (θ_i − θ_e)"
          reference="Naar open water — GEEN norm-formule (woonboot-extensie). Watertemperatuur θ_w is een engineering-aanname, default 5 °C. Zie sectie Afwijkingen."
        />
        <SymbolList
          symbols={[
            ["A_k", "oppervlak van constructie-element k [m²]"],
            ["U_k", "warmtedoorgangscoëfficiënt [W/(m²·K)]"],
            ["f_k, f_ia, f_b", "temperatuur-/reductiefactoren [-]"],
            ["θ_i / θ_e", "ontwerp-binnen-/buitentemperatuur [°C]"],
            ["θ_a / θ_b / θ_w", "temperatuur aangrenzend vertrek / buurpand / water [°C]"],
            ["c_z", "zekerheidsfactor buurverlies [-]"],
            ["Δθ₁ / Δθ₂", "stratificatiecorrecties plafond/vloer, Tabel 2.12 (erratum 2023)"],
          ]}
        />
      </Card>

      <Card title="Ventilatie (Φ_v)">
        <FormulaBlock
          formula="H_v = 1,2 × q_v × f_v        Φ_v = H_v × (θ_i − θ_e)"
          reference="ISSO 51:2023 §2.5.7, §4.2.2 — erratum Formule 4.3. Factor 1,2 = ρ·c_p in W·s/(dm³·K), q_v in dm³/s."
        />
        <FormulaBlock
          formula="f_v = ((θ_i + Δθ_v) − θ_t) / (θ_i − θ_e)"
          reference="Erratum Formule 4.6a (buitenlucht-toevoer). Voor lucht uit een aangrenzend vertrek geldt 4.6b met θ_a in plaats van θ_t."
        />
        <FormulaBlock
          formula="H_v = 1,2 × ((a × q_v × f_v1) + (1 − a) × q_v × f_v2)"
          reference="Erratum Formule 4.7 (gemengde toevoer: fractie a buitenlucht, rest binnenlucht)."
        />
        <SymbolList
          symbols={[
            ["q_v", "ventilatievolumestroom [dm³/s]"],
            ["θ_t", "toevoerluchttemperatuur [°C] — bij WTW afgeleid uit het rendement (Tabel 2.14, erratum)"],
            ["Δθ_v", "verwarmingssysteem-correctie uit Tabel 2.12 [K]"],
            ["a", "fractie buitenlucht [0–1]"],
          ]}
        />
      </Card>

      <Card title="Infiltratie (Φ_i)">
        <FormulaBlock
          formula="H_i = 1,2 × q_i        Φ_i = z_i × H_i × (θ_i − θ_e)"
          reference="ISSO 51:2023 §2.5.6, §4.2.1 — erratum Formules E.5 en 4.1. z_i = 1,0 (z-tabellen vervallen in het erratum)."
        />
        <p className="mb-2 text-sm text-on-surface-secondary">
          De norm-conforme keten voor q_i (f_inf-keten) op gebouwniveau:
        </p>
        <FormulaBlock
          formula="qv₁₀ = q_i,spec(klasse) × f_type × f_y × A_g        q_i = qv₁₀ × (Δp_design / 10)^n_lea × f_inf"
          reference="ISSO 51:2023 Tabel 2.8 + NTA 8800 Tabel 11.13/11.14 + NEN 8088-1 Tabel 10, met power-law-conversie van referentiedruk 10 Pa naar ontwerpdruk."
        />
        <SymbolList
          symbols={[
            ["q_i,spec", "specifieke infiltratie per m² A_g uit Tabel 2.8 [dm³/(s·m²)]"],
            ["f_type", "uitvoeringsvariant-correctie, NTA 8800 Tabel 11.14 (1,0 / 1,2 / 1,4)"],
            ["f_y", "bouwjaarcorrectie, NTA 8800 Tabel 11.13 (0,7–3,0)"],
            ["Δp_design", "ontwerp-drukverschil [Pa] — Vabi-compatibel: 3,14 Pa; NTA 8800-strikt: 10 Pa"],
            ["n_lea", "power-law-exponent, default 0,67"],
            ["f_inf", "ventilatiesysteem-correctie, NEN 8088-1 Tabel 10 (1,10 voor systeem D)"],
          ]}
        />
        <p className="text-sm text-on-surface-secondary">
          Bij een gemeten qv₁₀ (blower-door) wordt de tabel-keten overgeslagen
          en alleen de power-law-conversie + f_inf toegepast. Het
          gebouwresultaat wordt naar vertrekken verdeeld naar rato van het
          gebruiksoppervlak.
        </p>
      </Card>

      <Card title="Opwarmtoeslag (Φ_hu)">
        <FormulaBlock
          formula="Φ_hu = P × A_g"
          reference="ISSO 51:2023 §2.5.8 (Formule 2.45) + §4.3 (Formule 4.15). P [W/m²] uit Tabel 2.10 op basis van afkoeling en gebouwzwaarte."
        />
        <SymbolList
          symbols={[
            ["P", "opwarmvermogen per m² uit Tabel 2.10 [W/m²]"],
            ["A_g", "gebruiksoppervlak van het vertrek [m²]"],
          ]}
        />
        <p className="text-sm text-on-surface-secondary">
          Afkoeling na 8 uur nachtverlaging: 2 K voor nieuwbouw (na 2015), 1 K
          bij Ū ≤ 0,50 W/(m²·K) (Afb. 2.6). Geen nachtverlaging, een
          zelflerende regeling (§4.3.2) of vloerverwarming in álle vertrekken
          → Φ_hu = 0. Bestaande bouw (Afb. 2.7) valt buiten de huidige scope
          en geeft een expliciete foutmelding in plaats van een gegokte
          waarde.
        </p>
      </Card>

      <Card title="Sommatie per vertrek en gebouw">
        <FormulaBlock
          formula="Φ_extra = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²)        Φ_HL,i = Φ_basis + Φ_extra"
          reference="ISSO 51:2023 erratum Formule 3.11 + §4.5.3 — niet-simultane verliezen worden kwadratisch gesommeerd, niet lineair opgeteld."
        />
        <SymbolList
          symbols={[
            ["Φ_basis", "altijd optredende verliezen: Φ_T,ie + Φ_T,io + Φ_T,ig + Φ_i (+ systeemverliezen in de verdeler-tak)"],
            ["Φ_vent", "ventilatieverlies-component [W]"],
            ["Φ_T,iaBE", "transmissie naar buurpanden [W]"],
            ["Φ_hu", "opwarmtoeslag [W]"],
          ]}
        />
        <p className="text-sm text-on-surface-secondary">
          Op gebouwniveau geldt voor Φ_vent: bij natuurlijke toevoer (systeem
          A/C) <code className="font-mono">Φ_vent = ΣΦ_v − ΣΦ_i</code>{" "}
          (Formule 3.3, infiltratie is deel van de toevoerlucht); bij
          mechanische toevoer (systeem B/D/E){" "}
          <code className="font-mono">Φ_vent = ΣΦ_v</code> (Formule 3.4). Het
          aansluitvermogen volgt §3.5.3: Φ_HL,build (Formule 3.12, schil) en
          Φ_HL,verdeler (Formule 3.13, inclusief systeemverliezen).
        </p>
      </Card>

      <Card title="Systeemverliezen ingebedde verwarming (Φ_verlies)">
        <FormulaBlock
          formula="Φ_verlies = f × Φ_HL,i"
          reference="ISSO 51:2023 §2.9 — Tabel 2.17 (vloerverwarming), Tabel 2.18 erratum (wandverwarming), §2.9.1 erratum (plafondverwarming)."
        />
        <p className="text-sm text-on-surface-secondary">
          Fractie f staffelt op de R_c van de constructie achter het
          verwarmingsvlak: 0,85 / 0,40 / 0,25 / 0,15 / 0,10 voor vloer- en
          wandverwarming (R_c ≤ 0,35 t/m &gt; 3,0 m²·K/W); plafondverwarming
          0,50, of 0,20 bij R_c ≥ 3,0. De circulaire afhankelijkheid
          (Φ_verlies hangt af van Φ_HL die Φ_verlies bevat) is algebraïsch
          opgelost: Φ_HL = (Φ_basis + Φ_extra) / (1 − f).
        </p>
      </Card>
    </div>
  );
}
