/**
 * Help — sectie "Afwijkingen".
 *
 * Gedocumenteerde afwijkingen t.o.v. Vabi Elements en/of de norm.
 * Bronnen: de README's per fixture in `tests/verification/`,
 * `docs/2026-05-12-isso51-norm-conformiteit-audit.md`,
 * `docs/2026-05-12-vabi-infiltratie-keten-reproductie.md` en
 * doc-comments in `crates/isso51-core/src/`.
 */
import { Card } from "../../components/ui/Card";

type Ernst = "hoog" | "middel" | "laag";

interface Afwijking {
  onderwerp: string;
  onsGedrag: string;
  vabiNorm: string;
  impact: string;
  ernst: Ernst;
}

/**
 * Geëxporteerd zodat de smoke-test kan asserteren dat de tabel gevuld is
 * zonder de DOM te hoeven parsen.
 */
export const AFWIJKINGEN: ReadonlyArray<Afwijking> = [
  {
    onderwerp: "Gebouwsommatie (aansluitvermogen)",
    onsGedrag:
      "Kwadratische sommatie op gebouwniveau (Φ_extra = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²), erratum Formule 3.11). Historisch telde de gebouwsamenvatting lineair op — het DR Engineering-referentieproject toonde toen ~8.121 W waar Vabi 6.700 W rapporteert.",
    vabiNorm:
      "Vabi 3.12 en het erratum 2023 sommeren kwadratisch op zone-/gebouwniveau (6.700 W bij DR Engineering).",
    impact:
      "Inmiddels conform; oudere berekeningen/rapporten van vóór de fix overschatten het aansluitvermogen (bij DR Engineering ~21%). Herbereken bestaande projecten.",
    ernst: "hoog",
  },
  {
    onderwerp: "Aggregatiemethode Φ_T,iae (onverwarmde ruimtes)",
    onsGedrag:
      "Default 'Vabi-compatibel': verlies via onverwarmde ruimtes (Φ_T,iae) telt NIET mee in het gebouw-Φ_basis. Instelbaar naar 'norm-strikt' (§3.5.1: wél meetellen).",
    vabiNorm:
      "Vabi volgt de markt-conventie (exclusief); de norm §3.5.1 schrijft inclusief voor.",
    impact:
      "Norm-strikt geeft tot ~17% hoger aansluitvermogen. De gekozen methode staat expliciet in het resultaat en rapport.",
    ernst: "middel",
  },
  {
    onderwerp: "Watertemperatuur θ_w (woonboot-begrenzing)",
    onsGedrag:
      "Begrenzingstype 'water' met θ_w = 5 °C als default — een engineering-aanname, geen normwaarde. Volledige ΔT telt, zonder b-factor-demping zoals bij grond.",
    vabiNorm:
      "ISSO 51 / NEN-EN 12831 kennen geen water-begrenzing; Vabi heeft deze categorie niet.",
    impact:
      "Alleen relevant voor projecten met water-begrenzingen (woonboten). In het rapport altijd zichtbaar met asterisk + voetnoot.",
    ernst: "laag",
  },
  {
    onderwerp: "Infiltratie-ontwerpdruk Δp_design",
    onsGedrag:
      "Methode 'Vabi-compatibel' gebruikt Δp = 3,14 Pa (empirische fit op Vabi-resultaten); methode 'NTA 8800-strikt' gebruikt 10 Pa (geen reductie).",
    vabiNorm:
      "De norm-keten rekent op referentiedruk 10 Pa; Vabi past intern een lagere ontwerpdruk toe die niet in de norm gedocumenteerd is.",
    impact:
      "Vabi-compatibel reproduceert Vabi-rapporten; norm-strikt geeft hogere infiltratieverliezen (bv. Φ_i 693 W Vabi-compat vs. 177 W norm-strikt bij het kantoor-referentieproject — methode-keuze is dominant).",
    ernst: "middel",
  },
  {
    onderwerp: "Default infiltratiemethode",
    onsGedrag:
      "Default is 'per m² buitenoppervlak' (Tabel 4.3, ISSO 51:2017-legacy). De erratum-conforme methode 'per m² gebruiksoppervlak' (Tabel 2.8, Formule E.5) is beschikbaar maar niet de default.",
    vabiNorm:
      "Erratum 2023 Formule E.5 schrijft q_i,spec per m² gebruiksoppervlak (Tabel 2.8) voor; Vabi 3.12 rekent daarmee.",
    impact:
      "Voor 2023/2024-projecten levert de default andere infiltratiegetallen dan Vabi — kies bewust de methode in de instellingen.",
    ernst: "middel",
  },
  {
    onderwerp: "H_T,ia bij onverwarmde tussenruimtes (intra-woning)",
    onsGedrag:
      "Norm-formule 2.17 letterlijk: f_ia = (θ_i − θ_a)/(θ_i − θ_e), zonder extra correctie.",
    vabiNorm:
      "Vabi rapporteert ~10% minder negatieve Φ_T,ia bij situaties als 'onverwarmd toilet tussen verwarmde vertrekken'. Geen norm-bron gevonden voor die correctie (onderzocht: Tabel 2.16, Tabel 2.12, Formule 2.17) — vermoedelijk Vabi-interne intra-zonefactor. Status: won't fix.",
    impact:
      "Alleen per-vertrek-verschil; geen effect op het gebouwtotaal (Φ_basis van zulke vertrekken wordt op 0 geclampt).",
    ernst: "laag",
  },
  {
    onderwerp: "b-factor onverwarmde ruimtes (f_k default)",
    onsGedrag:
      "Zonder expliciete temperatuurfactor valt H_T,io terug op f_k = 0,5.",
    vabiNorm:
      "Norm Tabel 4.1 differentieert per ruimtetype (bv. 0,8 bij binnenzijde-isolatie, 0,5 kruipruimte).",
    impact:
      "Stel voor afwijkende onverwarmde ruimtes de temperatuurfactor handmatig in; de default is conservatief noch altijd correct.",
    ernst: "laag",
  },
  {
    onderwerp: "R_si/R_se bij Rc-herleiding voor systeemverliezen",
    onsGedrag:
      "R_c wordt uit de U-waarde herleid met vaste R_si = 0,17 (wand/vloer) en 0,14 (plafond) + R_se = 0,04 (buiten) of 0,0 (grond/water).",
    vabiNorm:
      "NEN-EN ISO 6946: R_si = 0,13 (wand), 0,10 (opwaarts), 0,17 (neerwaarts).",
    impact:
      "Kleine verschuiving in de staffel-keuze (f-fractie) voor vloer-/wand-/plafondverwarming; alleen relevant bij ingebedde verwarming.",
    ernst: "laag",
  },
  {
    onderwerp: "Ventilatiesysteem E",
    onsGedrag:
      "Systeem E (lokaal D + centraal C per vertrek, erratum 2023) bestaat als keuze-label, maar zonder per-vertrek-routing; het rekent als mechanische toevoer.",
    vabiNorm: "Erratum 2023 introduceert systeem E met mengvorm per vertrek.",
    impact:
      "Voor echte systeem E-projecten kan de ventilatieverdeling per vertrek afwijken.",
    ernst: "middel",
  },
];

const ERNST_STYLE: Record<Ernst, string> = {
  hoog: "bg-red-600/15 text-red-400",
  middel: "bg-amber-600/15 text-amber-400",
  laag: "bg-blue-600/15 text-blue-400",
};

export function HelpAfwijkingen() {
  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm leading-relaxed text-on-surface-secondary">
        Onderstaande afwijkingen ten opzichte van Vabi Elements en/of ISSO
        51:2023 zijn gedocumenteerd in de verificatieprojecten en de
        norm-conformiteitsaudit. Uitgangspunt van de rekenkern: bij twijfel
        volgt de code de norm letterlijk; bewuste Vabi-compatibiliteit is
        altijd een expliciete, instelbare keuze.
      </p>

      <div className="overflow-x-auto rounded-lg border border-[var(--oaec-border)]">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b-2 border-[var(--oaec-border)] bg-surface-alt text-left text-xs font-semibold uppercase tracking-wider text-on-surface-secondary">
              <th className="px-3 py-2.5">Onderwerp</th>
              <th className="px-3 py-2.5">Ons gedrag</th>
              <th className="px-3 py-2.5">Vabi / norm</th>
              <th className="px-3 py-2.5">Impact</th>
              <th className="w-[80px] px-3 py-2.5">Ernst</th>
            </tr>
          </thead>
          <tbody>
            {AFWIJKINGEN.map((a) => (
              <tr
                key={a.onderwerp}
                className="border-b border-[var(--oaec-border-subtle)] align-top hover:bg-[var(--oaec-hover)]/50"
              >
                <td className="px-3 py-2.5 font-medium text-on-surface">
                  {a.onderwerp}
                </td>
                <td className="px-3 py-2.5 text-on-surface-secondary">
                  {a.onsGedrag}
                </td>
                <td className="px-3 py-2.5 text-on-surface-secondary">
                  {a.vabiNorm}
                </td>
                <td className="px-3 py-2.5 text-on-surface-secondary">
                  {a.impact}
                </td>
                <td className="px-3 py-2.5">
                  <span
                    className={`rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase ${ERNST_STYLE[a.ernst]}`}
                  >
                    {a.ernst}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Card title="Bronverwijzing">
        <ul className="list-disc pl-5 text-sm leading-relaxed text-on-surface-secondary">
          <li>
            Verificatieprojecten met bekende afwijkingen per fixture:{" "}
            <code className="font-mono text-xs">
              tests/verification/*/README.md
            </code>
          </li>
          <li>
            Norm-conformiteitsaudit ISSO 51:2023 (erratum-dekking):{" "}
            <code className="font-mono text-xs">
              docs/2026-05-12-isso51-norm-conformiteit-audit.md
            </code>
          </li>
          <li>
            Vabi-infiltratieketen-reproductie:{" "}
            <code className="font-mono text-xs">
              docs/2026-05-12-vabi-infiltratie-keten-reproductie.md
            </code>
          </li>
        </ul>
      </Card>
    </div>
  );
}
