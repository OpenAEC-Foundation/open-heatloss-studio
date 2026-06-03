/**
 * TypeScript-spiegel van `isso53_core::result::*` (ISSO 53 calculation
 * result). Rust serialiseert met `#[serde(rename_all = "camelCase")]`.
 *
 * Wordt verbruikt door `frontend/src/lib/isso53ReportBuilder.ts` om een
 * BM Reports JSON op te bouwen voor de PDF-generator. Backend-pipeline
 * (`calculate_v2` → `to_isso53_project` → `isso53_core::calculate`) levert
 * dit shape op zodra de frontend voor `norm === "isso53"` op die route
 * gaat zitten.
 */

/** Per-vertrek calculatie-resultaat (ISSO 53). */
export interface Isso53RoomResult {
  /** Room identifier (referentie naar `Project.rooms[].id`). */
  roomId: string;
  /** Ruimtenaam zoals ingevoerd. */
  roomName: string;
  /** Ontwerp-binnentemperatuur θ_i in °C. */
  thetaI: number;
  /** Transmissieverlies Φ_T in W. */
  phiT: number;
  /** Ventilatieverlies Φ_v in W. */
  phiV: number;
  /** Infiltratieverlies Φ_i in W (ISSO 53: apart van Φ_v). */
  phiI: number;
  /** Opwarmtoeslag Φ_hu in W. */
  phiHu: number;
  /** Systeemverliezen Φ_system in W. */
  phiSystem: number;
  /** Interne warmtewinsten Φ_gain in W (negatief = warmtebron). */
  phiGain: number;
  /** Totaal warmteverlies Φ_HL,i in W. */
  totalHeatLoss: number;
  /** H_T,ie naar buitenlucht in W/K. */
  hTExterior: number;
  /** H_T,ia naar aangrenzende verwarmde ruimten in W/K. */
  hTAdjacentRooms: number;
  /** H_T,iae naar onverwarmde ruimten in W/K. */
  hTUnheated: number;
  /** H_T,iaBE naar aangrenzende gebouwen in W/K. */
  hTAdjacentBuildings: number;
  /** H_T,ig naar de grond in W/K. */
  hTGround: number;
  /** H_v ventilatie-coëfficiënt in W/K. */
  hV: number;
  /** H_i infiltratie-coëfficiënt in W/K. */
  hI: number;
}

/**
 * Herkomst van de gebruikte infiltratie-rekenmethode (ISSO 53 hybride-beleid).
 * Spiegelt `isso53_core::result::InfiltrationMethodOrigin`
 * (`#[serde(rename_all = "camelCase")]`).
 * - `isso53Norm`: ISSO 53-norm-pad (tabel 4.5 / formule 4.31), norm-puur.
 * - `vabiCompat`: Vabi-compat power-law (NEN 8088-1, NTA 8800, Δp ≈ 3,14 Pa),
 *   bewust geen ISSO 53-norm; rapport markeert dit expliciet.
 */
export type InfiltrationMethodOrigin = "isso53Norm" | "vabiCompat";

/** Gebouw-totaal samenvatting (ISSO 53 — simpele optelling, geen kwadratische sommatie). */
export interface Isso53BuildingSummary {
  /** Totaal transmissie Φ_T,build in W. */
  totalTransmissionLoss: number;
  /** Totaal ventilatie Φ_V,build in W. */
  totalVentilationLoss: number;
  /** Totaal infiltratie Φ_I,build in W (apart van ventilatie). */
  totalInfiltrationLoss: number;
  /** Totaal opwarmtoeslag Φ_hu,build in W. */
  totalHeatingUp: number;
  /** Totaal systeemverliezen Φ_system,build in W. */
  totalSystemLosses: number;
  /** Totaal interne winsten Φ_gain,build in W. */
  totalInternalGains: number;
  /** Totaal gebouw-warmteverlies Φ_HL,build in W. */
  totalBuildingHeatLoss: number;
  /** Aansluitvermogen individueel Φ_source in W (formule 5.1). */
  connectionCapacityIndividual: number;
  /** Aansluitvermogen collectief Φ_source in W (formule 5.9). */
  connectionCapacityCollective: number;
  /** Schil-methode warmteverlies Φ_HL,shell in W (hoofdstuk 3). */
  shellHeatLoss: number;
  /** Toegepaste infiltratie-reductiefactor z (tabel 5.1). */
  infiltrationReductionFactorZ: number;
  /**
   * Gelijktijdigheidsfactor (K2) toegepast op Σ Φ_hu in het aansluitvermogen
   * (ISSO 53 §4.1/§5.1). `1,0` = 100% gelijktijdigheid (default, engine-aanname,
   * geen reductie). Maakt expliciet welke gelijktijdigheid is aangenomen.
   */
  heatingUpSimultaneityFactor: number;
  /**
   * Herkomst van de gebruikte infiltratie-rekenmethode (ISSO 53 hybride-beleid):
   * `isso53Norm` (norm-puur) of `vabiCompat` (Vabi-compat power-law). Toont
   * transparant welke conventie voor Δp is gebruikt.
   */
  infiltrationMethodOrigin: InfiltrationMethodOrigin;
}

/** Volledig ISSO 53 project-resultaat. */
export interface Isso53ProjectResult {
  rooms: Isso53RoomResult[];
  summary: Isso53BuildingSummary;
}
