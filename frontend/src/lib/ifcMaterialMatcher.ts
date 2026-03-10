/**
 * IFC material name → materialsDatabase matcher.
 *
 * Matching strategy (layered):
 * 1. Exact: normalized name matches material.id or material.name
 * 2. Keyword overlap: tokenize IFC name, match against material keywords[]
 * 3. Category heuristic: pattern → category mapping
 * 4. Fallback: confidence "none"
 */
import {
  MATERIALS_DATABASE,
  type Material,
} from "./materialsDatabase";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type MatchConfidence = "exact" | "keyword" | "heuristic" | "none";

export interface MaterialMatch {
  material: Material | null;
  confidence: MatchConfidence;
  /** Matched IFC material name (for display). */
  ifcName: string;
}

// ---------------------------------------------------------------------------
// Category heuristic patterns (NL + EN)
// ---------------------------------------------------------------------------

const CATEGORY_PATTERNS: [RegExp, string[]][] = [
  // Masonry
  [/brick|baksteen|metselwerk|klinker|kalkzandsteen|kzs/i, ["metselwerk"]],
  // Concrete
  [/concrete|beton|cement|cementdekvloer|breedplaat|kanaalplaat/i, ["beton"]],
  // PIR/PUR insulation
  [/pir|pur|polyiso|polyurethaan|polyurethane/i, ["isolatie_kunststof"]],
  // EPS/XPS insulation
  [/eps|xps|polystyreen|polystyrene|styrofoam/i, ["isolatie_kunststof"]],
  // Mineral wool
  [/mineral.?wool|steenwol|glaswol|rockwool|stone.?wool|glass.?wool|minerale?.?wol/i, ["isolatie_mineraal"]],
  // Natural insulation
  [/cellulose|hennep|hemp|kurk|cork|vlas|flax|schapenwol|sheep/i, ["isolatie_natuurlijk"]],
  // Wood
  [/hout|wood|timber|naaldhout|loofhout|osb|multiplex|plywood/i, ["hout"]],
  // Gypsum boards
  [/gips|gypsum|plasterboard|gyproc|fermacell/i, ["plaatmateriaal"]],
  // Foils/membranes
  [/folie|film|membrane|membraan|damprem|vapor.?barrier|pe.?folie|bitumen/i, ["folie"]],
  // Plaster/finish
  [/stuc|plaster|pleister|sierpleister|render/i, ["afwerking"]],
  // Metal
  [/staal|steel|aluminium|aluminum|metaal|metal|zink|zinc|koper|copper/i, ["metaal"]],
  // Cavity
  [/spouw|cavity|air.?gap|luchtlaag/i, ["spouw"]],
  // Glass
  [/glas|glass/i, ["glas"]],
  // Floor finishes
  [/parket|parquet|laminaat|laminate|tegels|tiles|pvc.?vloer/i, ["vloer"]],
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Normalize a string for comparison: lowercase, strip accents, reduce whitespace. */
function normalize(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[_\-\/\\]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

/** Tokenize into words. */
function tokenize(s: string): string[] {
  return normalize(s).split(/\s+/).filter((t) => t.length > 1);
}

// ---------------------------------------------------------------------------
// Matching
// ---------------------------------------------------------------------------

/**
 * Match a single IFC material name to the best material from the database.
 */
export function matchIfcMaterial(ifcName: string): MaterialMatch {
  const normalizedName = normalize(ifcName);
  const tokens = tokenize(ifcName);

  // Strategy 1: Exact match on id or name
  for (const m of MATERIALS_DATABASE) {
    if (normalize(m.id) === normalizedName || normalize(m.name) === normalizedName) {
      return { material: m, confidence: "exact", ifcName };
    }
  }

  // Strategy 2: Keyword overlap scoring
  let bestMatch: Material | null = null;
  let bestScore = 0;

  for (const m of MATERIALS_DATABASE) {
    const haystack = [normalize(m.name), ...m.keywords.map(normalize)];
    let score = 0;
    for (const token of tokens) {
      for (const h of haystack) {
        if (h.includes(token)) {
          score += token.length;
          break;
        }
      }
    }
    if (score > bestScore) {
      bestScore = score;
      bestMatch = m;
    }
  }

  // Require at least 2 meaningful token matches or 5 chars matched
  if (bestMatch && bestScore >= 5) {
    return { material: bestMatch, confidence: "keyword", ifcName };
  }

  // Strategy 3: Category heuristic
  for (const [pattern, categories] of CATEGORY_PATTERNS) {
    if (pattern.test(ifcName)) {
      // Find first material in that category
      const candidate = MATERIALS_DATABASE.find(
        (m) => categories.includes(m.category),
      );
      if (candidate) {
        return { material: candidate, confidence: "heuristic", ifcName };
      }
    }
  }

  // Strategy 4: No match
  return { material: null, confidence: "none", ifcName };
}

/**
 * Match multiple IFC material names, returning an array of matches
 * in the same order.
 */
export function matchIfcMaterials(
  ifcNames: string[],
): MaterialMatch[] {
  return ifcNames.map(matchIfcMaterial);
}
