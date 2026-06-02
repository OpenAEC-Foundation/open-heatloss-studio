/**
 * ISSO 53 ontwerpbinnentemperatuur θ_i per gebruiksfunctie × ruimtetype.
 *
 * 1:1 TS-port van `crates/isso53-core/src/tables/temperature.rs`
 * (`design_indoor_temperature`), bron: ISSO 53 (2016) tabel 2.2, PDF p.20.
 *
 * Wordt door de chart-laag (`components/charts/deltaT.ts` +
 * `ConstructionLossChart.tsx`) gebruikt om in ISSO 53-modus de
 * ruimtetemperaturen uit de sidecar-`ruimteType` af te leiden i.p.v. de
 * ISSO 51 `room.function`-tabel — die in ISSO 53 systematisch fout is.
 */

import type {
  Isso53GebruiksFunctie,
  Isso53RuimteType,
} from "../types/projectV2";

/**
 * Marker die aangeeft dat de waarde gelijk is aan de
 * ontwerpbuitentemperatuur θ_e (ruimten buiten de thermische schil, garage).
 * Spiegelt de Rust-sentinel `TEMPERATURE_IS_EXTERIOR` (`f64::MIN`); de caller
 * moet deze marker vervangen door de actuele θ_e. We gebruiken hier
 * `Number.NEGATIVE_INFINITY` als sentinel zodat `=== TEMPERATURE_IS_EXTERIOR`
 * betrouwbaar matcht.
 */
export const TEMPERATURE_IS_EXTERIOR = Number.NEGATIVE_INFINITY;

/**
 * Geeft `true` als de gebruiksfunctie een gezondheidszorgfunctie is.
 * ISSO 53 tabel 2.2 onderscheidt zorg van alle overige functies.
 */
function isZorg(functie: Isso53GebruiksFunctie): boolean {
  return functie === "gezondheidszorg";
}

/**
 * Ontwerpbinnentemperatuur θ_i in °C volgens ISSO 53 tabel 2.2 (PDF p.20).
 *
 * Retourneert de minimale ontwerpbinnentemperatuur voor de combinatie
 * gebruiksfunctie × ruimtetype. Voor ruimten "buiten de thermische schil"
 * (garage) wordt {@link TEMPERATURE_IS_EXTERIOR} teruggegeven — de caller
 * vult dan θ_e in.
 */
export function design_indoor_temperature(
  functie: Isso53GebruiksFunctie,
  ruimte: Isso53RuimteType,
): number {
  const zorg = isZorg(functie);
  switch (ruimte) {
    // Verblijfsruimte / verblijfsgebied: 20 °C overig, 22 °C zorg.
    case "verblijfsruimte":
    case "verblijfsgebied":
    case "kantoorruimte":
    case "receptie":
    case "lesruimte":
    case "collegezaal":
    case "werkplaats":
    case "bureauruimte":
    case "patientenkamer":
    case "operatiekamer":
    case "onderzoekruimte":
    case "eetruimte":
    case "restaurant":
    case "kantine":
    case "vergaderruimte":
    case "hotelkamer":
    case "sportzaal":
    case "verkoopruimte":
    case "supermarkt":
    case "warenhuis":
      return zorg ? 22.0 : 20.0;
    // Badruimte: 22 °C overig, 24 °C zorg.
    case "badruimte":
      return zorg ? 24.0 : 22.0;
    // Toilet- en verkeersruimte: 18 °C (of warmtebalans).
    case "toiletruimte":
    case "verkeersruimte":
      return 18.0;
    // Technische, onbenoemde en bergruimte: 10 °C (of warmtebalans).
    case "technischeRuimte":
    case "onbenoemdeRuimte":
    case "bergruimte":
      return 10.0;
    // Stallingsruimte: forfaitair 5 °C (tabel 2.2 voetnoot 2).
    case "stallingsruimte":
      return 5.0;
    // Garage: buiten de thermische schil → θ_e.
    case "garage":
      return TEMPERATURE_IS_EXTERIOR;
  }
}
