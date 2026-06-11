/**
 * Help — sectie "Gebruik".
 *
 * Stapsgewijze uitleg van de werkflow, afgeleid uit de bestaande
 * pagina-flow (project → vertrekken → constructies → ventilatie →
 * resultaten → rapport) en de drie invoerroutes (handmatig, Revit
 * thermal-import, Vabi-import).
 */
import { Card } from "../../components/ui/Card";
import { Kbd, Step } from "./primitives";

export function HelpGebruik() {
  return (
    <div className="flex flex-col gap-4">
      <Card title="Werkflow in zes stappen">
        <ol className="flex flex-col gap-5">
          <Step nr={1} title="Project aanmaken of openen">
            Start via <Kbd>Bestand</Kbd> (Backstage) een nieuw project, of open
            een bestaand <Kbd>.ifcenergy</Kbd>-bestand (lokaal of vanaf de
            server). Op de pagina <Kbd>Project</Kbd> stel je de
            projectgegevens, het klimaat (ontwerp-buitentemperatuur θ_e),
            gebouwtype, infiltratie-instellingen en het standaard
            verwarmingssysteem in. De norm (ISSO 51 woningbouw of ISSO 53
            utiliteitsbouw) wissel je via{" "}
            <Kbd>Warmteverlies → Instellingen</Kbd>.
          </Step>
          <Step nr={2} title="Vertrekken en zones invoeren">
            Op de pagina <Kbd>Vertrekken</Kbd> voer je per ruimte de functie
            (bepaalt de ontwerp-binnentemperatuur θ_i volgens ISSO 51 Tabel
            2.11, handmatig te overschrijven), vloeroppervlak, hoogte en het
            verwarmingssysteem in. Het verwarmingssysteem bepaalt de
            Δθ-correcties uit Tabel 2.12 (erratum 2023) en is dus rekenkundig
            relevant — vloerverwarming geeft andere toeslagen dan radiatoren.
          </Step>
          <Step nr={3} title="Constructies koppelen">
            Op <Kbd>Constructies</Kbd> definieer je per vertrek de
            scheidingsconstructies: oppervlak, U-waarde en begrenzing
            (buitenlucht, aangrenzend vertrek, onverwarmde ruimte, buurpand,
            grond of water). U-waardes kun je kiezen uit de{" "}
            <Kbd>Bibliotheek</Kbd>, zelf invoeren, of opbouwen met de{" "}
            <Kbd>Rc-waarde</Kbd>- en <Kbd>U-waarde kozijn</Kbd>-calculators.
          </Step>
          <Step nr={4} title="Ventilatie instellen">
            Kies het ventilatiesysteem (A t/m E) en de debieten per vertrek.
            Bij systeem D met WTW wordt de toevoertemperatuur θ_t afgeleid uit
            het WTW-rendement (ISSO 51 Tabel 2.14, erratum). De{" "}
            <Kbd>Ventilatiebalans</Kbd>-pagina helpt bij het opstellen van een
            kloppende toevoer/afvoer-balans volgens BBL-minimums.
          </Step>
          <Step nr={5} title="Berekenen en resultaten controleren">
            De pagina <Kbd>Resultaten</Kbd> toont per vertrek de
            transmissieverliezen (Φ_T), ventilatie (Φ_v), infiltratie (Φ_i),
            opwarmtoeslag (Φ_hu) en het ontwerpvermogen Φ_HL,i, plus het
            gebouwtotaal (aansluitvermogen). Zie de sectie{" "}
            <em>Formules</em> voor de gehanteerde rekenregels.
          </Step>
          <Step nr={6} title="Rapport genereren">
            Via <Kbd>Rapport</Kbd> genereer je een PDF-rapport met
            uitgangspunten, klimaattabel, per-vertrek-resultaten en
            gebouwsamenvatting. Engineering-aannames die niet uit de norm
            komen (zoals de watertemperatuur θ_w) worden in het rapport
            expliciet met een asterisk en voetnoot gemarkeerd.
          </Step>
        </ol>
      </Card>

      <Card title="Drie invoerroutes">
        <ul className="flex flex-col gap-3 text-sm leading-relaxed text-on-surface-secondary">
          <li>
            <strong className="text-on-surface">Handmatig</strong> — vertrekken
            en constructies rechtstreeks invoeren via de pagina&apos;s{" "}
            <Kbd>Vertrekken</Kbd> en <Kbd>Constructies</Kbd>, eventueel
            ondersteund door de 2D-<Kbd>Modeller</Kbd> of een{" "}
            <Kbd>IFC</Kbd>-model als onderlegger.
          </li>
          <li>
            <strong className="text-on-surface">Revit thermal-import</strong>{" "}
            — exporteer vanuit Revit met de pyRevit-knop{" "}
            <em>RaycastExport</em> een thermal-JSON en open dat bestand via{" "}
            <Kbd>Bestand → Openen</Kbd>. De import-wizard (
            <Kbd>/import/thermal</Kbd>) opent automatisch en leidt je door het
            mappen van ruimtes, constructiepakketten en begrenzingen.
          </li>
          <li>
            <strong className="text-on-surface">Vabi-import</strong> — de
            rekenkern bevat een importer voor Vabi Elements-projecten
            (.vp), inclusief afleiding van de Vabi-compatibele
            infiltratieketen. De menukeuze in <Kbd>Bestand → Openen</Kbd> is
            momenteel uitgeschakeld terwijl de keten gevalideerd wordt tegen
            referentieprojecten; zie de sectie <em>Verificatie</em>.
          </li>
        </ul>
      </Card>

      <Card title="Eenheden-conventie">
        <p className="text-sm leading-relaxed text-on-surface-secondary">
          Luchtdebieten worden intern gerekend in{" "}
          <strong className="text-on-surface">dm³/s</strong> — de eenheid van
          ISSO 51:2023 (erratum: factor 1200 → 1,2 met q_v in dm³/s). In de
          invoervelden kun je schakelen naar{" "}
          <strong className="text-on-surface">m³/h</strong>{" "}
          (fabrikant-conventie); de omrekening is ÷ 3,6 van m³/h naar dm³/s.
          Temperaturen in °C, oppervlakten in m², U-waardes in W/(m²·K),
          vermogens in W.
        </p>
      </Card>
    </div>
  );
}
