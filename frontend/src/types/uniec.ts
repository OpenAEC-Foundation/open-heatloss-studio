/**
 * TypeScript-spiegel van `uniec3_import::Uniec3CertifiedResults` + de
 * import-response (`crates/uniec3-import/src/results.rs` +
 * `crates/isso51-api/src/handlers/uniec_import.rs`).
 *
 * Handgeschreven (geen schema-gen — zie de type-gen-valkuil in de
 * project-lessons). Houd deze velden 1:1 met de Rust-struct: alles optioneel,
 * `#[serde(skip_serializing_if = "Option::is_none")]` → een ontbrekend bron-veld
 * verschijnt niet in de JSON.
 */

import type { ProjectV2 } from "./projectV2";

/**
 * De door Uniec/BengCert **gecertificeerde** uitkomsten van één afgemeld
 * gebouw — het vergelijkingsobject naast de eigen `compute_beng`-uitkomst.
 *
 * Eenheden: BENG 1/2 in kWh/(m²·jr), BENG 3 in %, primaire energie in kWh,
 * oppervlakten in m².
 */
export interface Uniec3CertifiedResults {
  /** App-versie waaruit geëxporteerd is (bv. `"3.3.3.1"`), provenance. */
  app_version?: string | null;

  /** BENG 1 — energiebehoefte, kWh/(m²·jr). */
  beng1_kwh_m2_jr?: number | null;
  /** BENG 1-eis. */
  beng1_limit_kwh_m2_jr?: number | null;
  /** BENG 2 — primair fossiel energiegebruik, kWh/(m²·jr). */
  beng2_kwh_m2_jr?: number | null;
  /** BENG 2-eis. */
  beng2_limit_kwh_m2_jr?: number | null;
  /** BENG 3 — aandeel hernieuwbare energie, %. */
  beng3_pct?: number | null;
  /** BENG 3-eis. */
  beng3_limit_pct?: number | null;

  /** TOjuli-waarde. */
  tojuli?: number | null;
  /** TOjuli-eis. */
  tojuli_limit?: number | null;

  /** Energielabel (bv. `"A+++"`). */
  energy_label?: string | null;

  /** Primaire energie verwarming, kWh. */
  heating_primary_kwh?: number | null;
  /** Primaire energie warm tapwater, kWh. */
  hot_water_primary_kwh?: number | null;
  /** Primaire energie koeling, kWh. */
  cooling_primary_kwh?: number | null;
  /** Primaire energie ventilatoren, kWh. */
  fans_primary_kwh?: number | null;

  /** Opgewekte PV-elektriciteit, kWh. */
  pv_production_kwh?: number | null;
  /** Netto koudebehoefte, kWh. */
  cooling_demand_kwh?: number | null;

  /** Netto warmtebehoefte, kWh/(m²·jr). */
  warmtebehoefte_kwh_m2?: number | null;
  /** Vormfactor A_ls/A_g. */
  vormfactor?: number | null;
  /** Verliesoppervlak A_ls, m². */
  verlies_opp_m2?: number | null;
  /** Gebruiksoppervlak A_g, m². */
  gebruiks_opp_m2?: number | null;
}

/**
 * Resultaat van een `.uniec3`-import (web-route + Tauri-command leveren
 * hetzelfde contract). `project` is de wire-`ProjectV2` inclusief de
 * top-level `energy` + `beng_geometry` velden die de store-sidecars vullen.
 */
export interface Uniec3ImportResult {
  project: ProjectV2;
  certified: Uniec3CertifiedResults;
  warnings: string[];
}
