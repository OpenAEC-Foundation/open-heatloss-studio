/**
 * WTW/MV-units — catalogus-loader + capaciteitstoets (pure functies).
 *
 * Port van het units-mechanisme uit de pyRevit `VentilatieBalans`-plugin
 * (`VentilatieBalans.pushbutton/script.py`):
 *   - `load_units_database()` (r.117-126) — JSON met `{wtw_units, mv_units}`;
 *   - `ZoneUnitToewijzing` (r.304-321) — totaalcapaciteit = Σ capaciteit × aantal;
 *   - `_get_gecombineerde_eis` (r.622-630) + capaciteits-status
 *     (`_update_zone_units_display`, r.1007-1019) — toets capaciteit ≥ eis.
 *
 * NB: het plugin-databestand `ventilatie_units.json` bestond nergens (de
 * plugin viel terug op een lege DB) — het mechanisme is geport, de seed-data
 * is nieuw en **indicatief** (`data/ventilationUnits.json`, zie
 * `_meta.verification_note` aldaar).
 *
 * **Eenheden:** catalogus-capaciteit in m³/h (fabrikant-conventie); de toets
 * rekent om naar dm³/s (÷ 3,6) — zelfde route als de plugin
 * (`get_totaal_capaciteit_dm3s`, r.320-321).
 */

import catalogJson from "../data/ventilationUnits.json";
import {
  m3hToDm3s,
  ventilationSystemOf,
  type VentilationState,
  type VentilationUnit,
  type VentilationUnitAssignment,
  type VentilationUnitType,
} from "../types/ventilation";

// ---------------------------------------------------------------------------
// Catalogus-loader
// ---------------------------------------------------------------------------

/** Eén ruw catalogus-record zoals in `data/ventilationUnits.json`. */
interface CatalogRecord {
  id: string;
  fabrikant: string;
  model: string;
  capaciteit_m3h: number;
  rendement?: number;
  geluid_db?: number;
}

interface VentilationUnitCatalog {
  wtw_units: CatalogRecord[];
  mv_units: CatalogRecord[];
}

// `resolveJsonModule` levert de JSON getypeerd uit; cast naar het expliciete
// contract zodat de helpers stabiel blijven als de JSON groeit. `_meta` wordt
// bewust genegeerd (zelfde patroon als `lib/productCatalog.ts`).
const catalog = catalogJson as unknown as VentilationUnitCatalog;

function toUnit(record: CatalogRecord, type: VentilationUnitType): VentilationUnit {
  return {
    id: record.id,
    type,
    fabrikant: record.fabrikant,
    model: record.model,
    capaciteitM3h: record.capaciteit_m3h,
    ...(record.rendement !== undefined ? { rendement: record.rendement } : {}),
    ...(record.geluid_db !== undefined ? { geluidDb: record.geluid_db } : {}),
    source: "catalog" as const,
  };
}

/**
 * Alle catalogus-units (WTW + MV), gesorteerd op fabrikant + model.
 * **Indicatief** — controleer fabrikantgegevens (zie `_meta` in de JSON).
 */
export function getCatalogUnits(): VentilationUnit[] {
  return [
    ...catalog.wtw_units.map((r) => toUnit(r, "wtw")),
    ...catalog.mv_units.map((r) => toUnit(r, "mv")),
  ].sort((a, b) => {
    const byFab = a.fabrikant.localeCompare(b.fabrikant, "nl");
    return byFab !== 0 ? byFab : a.model.localeCompare(b.model, "nl");
  });
}

/** Zoek één catalogus-unit op id. `undefined` wanneer niet gevonden. */
export function findCatalogUnit(id: string): VentilationUnit | undefined {
  return getCatalogUnits().find((u) => u.id === id);
}

/**
 * Voorkeurs-unit-type per ventilatiesysteem: D (balans/WTW) → `"wtw"`,
 * B/C (één mechanische kant) → `"mv"`, A (volledig natuurlijk) → `null`
 * (geen units van toepassing).
 */
export function preferredUnitType(
  system?: VentilationState["system"],
): VentilationUnitType | null {
  const sys = ventilationSystemOf({ system });
  if (sys.supplyMechanical && sys.exhaustMechanical) return "wtw";
  if (sys.supplyMechanical || sys.exhaustMechanical) return "mv";
  return null;
}

// ---------------------------------------------------------------------------
// Toewijzing — resolutie + totaalcapaciteit
// ---------------------------------------------------------------------------

/** Eén opgeloste toewijzing: unit-snapshot + aantal. */
export interface ResolvedUnitAssignment {
  unit: VentilationUnit;
  aantal: number;
}

/**
 * Los de toewijzingen op tegen de project-unitbibliotheek. Toewijzingen naar
 * een onbekende `unitId` (bv. een verwijderde unit) worden genegeerd —
 * zelfde defensieve houding als terminals van verwijderde ruimtes in
 * `aggregateVentilationBalance`.
 */
export function resolveUnitAssignments(
  units: VentilationUnit[] | undefined,
  assignments: VentilationUnitAssignment[] | undefined,
): ResolvedUnitAssignment[] {
  if (!units || !assignments) return [];
  const byId = new Map(units.map((u) => [u.id, u]));
  const out: ResolvedUnitAssignment[] = [];
  for (const a of assignments) {
    const unit = byId.get(a.unitId);
    if (!unit || a.aantal <= 0) continue;
    out.push({ unit, aantal: a.aantal });
  }
  return out;
}

/**
 * Totaal toegewezen capaciteit in m³/h: Σ capaciteit × aantal. Port van
 * `ZoneUnitToewijzing.get_totaal_capaciteit_m3h` (plugin r.317-318).
 */
export function totalAssignedCapacityM3h(
  resolved: ResolvedUnitAssignment[],
): number {
  return resolved.reduce((s, r) => s + r.unit.capaciteitM3h * r.aantal, 0);
}

// ---------------------------------------------------------------------------
// Capaciteitstoets
// ---------------------------------------------------------------------------

/**
 * De gecombineerde eis (dm³/s) waartegen de unit-capaciteit getoetst wordt.
 *
 * **Eis-keuze per systeem (gedocumenteerde ontwerpkeuze):**
 * De plugin (`_get_gecombineerde_eis`, r.622-630) neemt altijd
 * `max(toevoer-eis, afvoer-eis)` — de unit moet de maatgevende kant aankunnen.
 * De webtool ként het systeem (A–D) en maakt de vergelijking systeem-bewust:
 *
 *   - **Systeem D (WTW/balans):** de unit verzorgt toevoer én afvoer →
 *     eis = `max(toevoer, afvoer)` (maatgevende kant; identiek aan de plugin).
 *   - **Systeem C (MV-box):** alleen de afvoer is mechanisch, maar de
 *     MV-box moet in balans óók de via gevelroosters toegevoerde lucht
 *     verwerken (de overdruk-verdeling boekt het toevoer-overschot bij de
 *     afvoerruimtes, zie `computeOverflowDistribution`) →
 *     eis = `max(toevoer, afvoer)` (identiek aan de plugin).
 *   - **Systeem B:** alleen de toevoer is mechanisch (afvoer natuurlijk via
 *     kanalen/schacht, geen unit-afhankelijkheid) → eis = toevoer-eis.
 *   - **Systeem A:** volledig natuurlijk → geen units van toepassing, eis = 0
 *     ({@link checkUnitCapacity} markeert de toets als `applicable: false`).
 */
export function combinedRequirementDm3s(
  totalRequiredSupplyDm3s: number,
  totalRequiredExhaustDm3s: number,
  system?: VentilationState["system"],
): number {
  const sys = ventilationSystemOf({ system });
  if (!sys.supplyMechanical && !sys.exhaustMechanical) return 0; // A
  if (sys.exhaustMechanical) {
    // C en D: de afvoerkant moet minstens de totale toevoer-eis verwerken
    // (gebouwbalans) → maatgevende kant.
    return Math.max(totalRequiredSupplyDm3s, totalRequiredExhaustDm3s);
  }
  return totalRequiredSupplyDm3s; // B: alleen mechanische toevoer
}

/** Uitkomst van de capaciteitstoets (gebouwniveau). */
export interface UnitCapacityCheck {
  /** `false` bij systeem A (volledig natuurlijk) — geen units van toepassing. */
  applicable: boolean;
  /** Totaal aantal toegewezen toestellen (Σ aantal). */
  assignedCount: number;
  /** Σ capaciteit × aantal in m³/h. */
  totalCapacityM3h: number;
  /** Idem in dm³/s (÷ 3,6). */
  totalCapacityDm3s: number;
  /** Gecombineerde eis in dm³/s (zie {@link combinedRequirementDm3s}). */
  requiredDm3s: number;
  /** Capaciteit ≥ eis? (plugin-criterium, r.1011). */
  satisfied: boolean;
  /** Tekort in dm³/s (eis − capaciteit, geclamped op ≥ 0). */
  shortfallDm3s: number;
  /**
   * Marge t.o.v. de eis in %: `(capaciteit − eis) / eis × 100`.
   * Eis ≤ 0 → 0 (geen zinvolle marge).
   */
  marginPct: number;
}

/**
 * Capaciteitstoets: toegewezen totaalcapaciteit vs. de gecombineerde eis uit
 * de gebouwbalans. Port van de status-logica in de plugin
 * (`_update_zone_units_display`, r.1007-1019: OK wanneer
 * `totaal_cap >= gecomb_eis`, anders TEKORT), uitgebreid met marge% en een
 * systeem-bewuste eis-keuze (zie {@link combinedRequirementDm3s}).
 */
export function checkUnitCapacity(
  units: VentilationUnit[] | undefined,
  assignments: VentilationUnitAssignment[] | undefined,
  totalRequiredSupplyDm3s: number,
  totalRequiredExhaustDm3s: number,
  system?: VentilationState["system"],
): UnitCapacityCheck {
  const sys = ventilationSystemOf({ system });
  const applicable = sys.supplyMechanical || sys.exhaustMechanical;
  const resolved = resolveUnitAssignments(units, assignments);
  const totalCapacityM3h = totalAssignedCapacityM3h(resolved);
  const totalCapacityDm3s = m3hToDm3s(totalCapacityM3h);
  const requiredDm3s = combinedRequirementDm3s(
    totalRequiredSupplyDm3s,
    totalRequiredExhaustDm3s,
    system,
  );
  return {
    applicable,
    assignedCount: resolved.reduce((s, r) => s + r.aantal, 0),
    totalCapacityM3h,
    totalCapacityDm3s,
    requiredDm3s,
    satisfied: totalCapacityDm3s >= requiredDm3s,
    shortfallDm3s: Math.max(0, requiredDm3s - totalCapacityDm3s),
    marginPct:
      requiredDm3s > 0
        ? ((totalCapacityDm3s - requiredDm3s) / requiredDm3s) * 100
        : 0,
  };
}
