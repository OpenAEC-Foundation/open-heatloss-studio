/**
 * Deurspleet-rekenmodel — spleet onder de deur als doorstroomopening
 * (NEN 1087:2001 §5.1.3.2).
 *
 * Herbruikbaar, state-loos rekenmodel voor de losse deurspleet-tool
 * (`pages/DoorGapCalculator.tsx`); bewust volledig los van de
 * ventilatiebalans-state zodat latere integratie in de balans en de
 * rapport-sectie zonder herstructurering kan. De spleetformule zelf is één
 * bron van waarheid: {@link estimateDoorGapAreaCm2} uit
 * `ventilationBalance.ts` (orifice-benadering, C_d = 0,6, ρ = 1,2 kg/m³,
 * n = 0,5 — norm-verankering gedocumenteerd op die functie).
 *
 * **Eenheden:** dm³/s intern (project-conventie); m³/h-conversie gebeurt
 * uitsluitend aan de UI-rand via `flowToDisplay`/`flowFromDisplay`
 * (`types/ventilation.ts`) — nooit in deze module.
 */

import {
  DOOR_GAP_DELTA_P_PA,
  estimateDoorGapAreaCm2,
} from "./ventilationBalance";

// Re-export zodat tool-code met één import van dit rekenmodel toekan.
export {
  DOOR_GAP_DELTA_P_PA,
  DOOR_GAP_DELTA_P_OFFICE_PA,
} from "./ventilationBalance";

/**
 * Praktijkvuistregel: 12 cm² vrije doorlaat per dm³/s overstroomdebiet.
 *
 * **Reconciliatie met de exacte formule:** de orifice-benadering van
 * {@link estimateDoorGapAreaCm2} geeft bij Δp = 1 Pa (woonfunctie,
 * NEN 1087:2001 §5.1.3.2.7) exact `1/(0,6·√(2/1,2)) × 10 ≈ 12,9` cm² per
 * dm³/s. De praktijkvuistregel 12 cm² is daarvan een iets krappere afronding
 * naar beneden — handig als hoofdrekensom, maar de exacte waarde is leidend.
 */
export const RULE_OF_THUMB_CM2_PER_DM3S = 12;

/**
 * Boven deze spleethoogte (mm) is een spleet onder de deur praktisch niet
 * meer uitvoerbaar (loopdeur sleept niet, maar de kier oogt als een gat en
 * geeft geluid-/lichtlek) — het advies wordt dan "deurrooster overwegen".
 * Praktijkgrens, geen normwaarde.
 */
export const DOOR_GAP_GRILLE_THRESHOLD_MM = 20;

/**
 * Benodigde vrije doorlaat (cm²) voor een overstroomdebiet — dunne wrapper om
 * {@link estimateDoorGapAreaCm2} (zelfde formule, zelfde constantes; zie de
 * norm-verankering NEN 1087:2001 §5.1.3.2 aldaar).
 *
 * @param flowDm3s overstroomdebiet in dm³/s (≤ 0 → 0 cm²)
 * @param deltaPPa toelaatbaar drukverschil in Pa (default
 *   {@link DOOR_GAP_DELTA_P_PA} = 1 Pa woonfunctie; kantoor: 2 Pa via
 *   `DOOR_GAP_DELTA_P_OFFICE_PA`)
 */
export function requiredGapAreaCm2(
  flowDm3s: number,
  deltaPPa: number = DOOR_GAP_DELTA_P_PA,
): number {
  return estimateDoorGapAreaCm2(flowDm3s, deltaPPa);
}

/**
 * Doorlaat volgens de praktijkvuistregel
 * ({@link RULE_OF_THUMB_CM2_PER_DM3S} = 12 cm² per dm³/s) — voor de
 * vergelijking exact ↔ vuistregel in de tool.
 */
export function ruleOfThumbAreaCm2(flowDm3s: number): number {
  return flowDm3s <= 0 ? 0 : flowDm3s * RULE_OF_THUMB_CM2_PER_DM3S;
}

/** Invoer voor {@link gapHeightMm}. */
export interface GapHeightInput {
  /** Overstroomdebiet door deze ene deur in dm³/s. */
  flowDm3s: number;
  /** Vrije deurbreedte in mm (gangbaar binnendeur-maat: 880). */
  doorWidthMm: number;
  /** Drukverschil in Pa (default {@link DOOR_GAP_DELTA_P_PA} = 1 Pa). */
  deltaPPa?: number;
  /**
   * Optionele reductie (%) van de vrije doorlaat van de spleet — bv. door een
   * drempel, borstelstrip of hoogpolige vloerbedekking. De geometrische
   * spleet moet dan groter: `A = A_vrij / (1 − pct/100)`. Alleen toegepast
   * voor `0 < pct < 100`; waarden daarbuiten worden genegeerd.
   */
  freeAreaReductionPct?: number;
}

/** Resultaat van {@link gapHeightMm}. */
export interface GapHeightResult {
  /** Benodigde (geometrische) doorlaat in cm², incl. eventuele reductie. */
  areaCm2: number;
  /**
   * Spleethoogte in hele mm, **naar boven afgerond** (een te krappe spleet
   * haalt het debiet bij het Δp-criterium niet). 0 bij debiet ≤ 0 of
   * ongeldige deurbreedte.
   */
  heightMm: number;
}

/**
 * Spleethoogte onder de deur voor een overstroomdebiet:
 * `hoogte = doorlaat / deurbreedte`, naar boven afgerond op hele mm.
 *
 * Doorlaat via {@link requiredGapAreaCm2} (NEN 1087:2001 §5.1.3.2),
 * eventueel vergroot voor een gereduceerde vrije doorlaat
 * ({@link GapHeightInput.freeAreaReductionPct}).
 */
export function gapHeightMm({
  flowDm3s,
  doorWidthMm,
  deltaPPa = DOOR_GAP_DELTA_P_PA,
  freeAreaReductionPct,
}: GapHeightInput): GapHeightResult {
  let areaCm2 = requiredGapAreaCm2(flowDm3s, deltaPPa);
  if (
    freeAreaReductionPct !== undefined &&
    freeAreaReductionPct > 0 &&
    freeAreaReductionPct < 100
  ) {
    areaCm2 = areaCm2 / (1 - freeAreaReductionPct / 100);
  }
  if (areaCm2 <= 0 || doorWidthMm <= 0) {
    return { areaCm2: Math.max(0, areaCm2), heightMm: 0 };
  }
  // cm² → mm² (×100), gedeeld door de breedte (mm) → hoogte (mm), naar boven.
  const heightMm = Math.ceil((areaCm2 * 100) / doorWidthMm);
  return { areaCm2, heightMm };
}

/** Advies-uitkomst van {@link doorGapAdvice}. */
export type DoorGapAdvice = "ok" | "grille";

/**
 * Praktisch advies bij een berekende spleethoogte: boven
 * {@link DOOR_GAP_GRILLE_THRESHOLD_MM} (20 mm) is een spleet niet meer
 * realistisch uitvoerbaar → `"grille"` (deurrooster toepassen), anders `"ok"`.
 *
 * @param acoustic geluidswerende uitvoering gewenst → **altijd** `"grille"`,
 *   ongeacht de spleethoogte: een open spleet is akoestisch ongewenst
 *   (geluidlek tussen ruimtes), dus bij geluidseisen hoort een geluidswerend
 *   deurrooster — zie {@link proposeDoorGrille}.
 */
export function doorGapAdvice(
  heightMm: number,
  acoustic = false,
): DoorGapAdvice {
  if (acoustic) return "grille";
  return heightMm > DOOR_GAP_GRILLE_THRESHOLD_MM ? "grille" : "ok";
}

// ---------------------------------------------------------------------------
// Deurrooster-voorstel — indicatieve seed met generieke maatvoeringen
// ---------------------------------------------------------------------------

/**
 * Conservatieve netto-doorlaatfractie van een standaard (schoepen-)deurrooster:
 * netto vrije doorlaat ≈ 40% van het dagmaat-oppervlak (b × h).
 *
 * **Aanname, indicatief** — gangbare schoepenroosters halen ca. 40–60% vrije
 * doorlaat; we rekenen bewust met de onderkant van die bandbreedte zodat het
 * voorstel aan de veilige kant zit. Géén fabrikantdata — controleer altijd
 * het productblad van het gekozen rooster (zelfde indicatief-patroon als de
 * units-seed in `ventilationUnits.ts` / `data/ventilationUnits.json`).
 */
export const GRILLE_NET_AREA_FRACTION = 0.4;

/**
 * Conservatieve netto-doorlaatfractie van een **geluidswerend** deurrooster:
 * netto vrije doorlaat ≈ 25% van het dagmaat-oppervlak.
 *
 * **Aanname, indicatief** — geluidswerende (gelabyrintheerde/gedempte)
 * roosters leveren per cm² dagmaat aanzienlijk minder vrije doorlaat dan
 * open schoepenroosters; 25% is een conservatieve ondergrens. Géén
 * fabrikantdata — controleer het productblad (zie
 * {@link GRILLE_NET_AREA_FRACTION}).
 */
export const GRILLE_NET_AREA_FRACTION_ACOUSTIC = 0.25;

/** Eén generieke deurrooster-maat (dagmaat in mm). */
export interface DoorGrilleSize {
  widthMm: number;
  heightMm: number;
}

/**
 * Seed-lijst met **generieke, indicatieve** deurrooster-maatvoeringen
 * (gangbare-klasse afmetingen zoals 425×125 en 455×90), oplopend gesorteerd
 * op dagmaat-oppervlak. Bewust géén fabrikanten/artikelnummers — de netto
 * doorlaat volgt uit de conservatieve fracties
 * ({@link GRILLE_NET_AREA_FRACTION} / {@link GRILLE_NET_AREA_FRACTION_ACOUSTIC});
 * controleer altijd het productblad.
 */
export const DOOR_GRILLE_SEED: readonly DoorGrilleSize[] = [
  { widthMm: 245, heightMm: 90 },
  { widthMm: 345, heightMm: 90 },
  { widthMm: 425, heightMm: 90 },
  { widthMm: 455, heightMm: 90 },
  { widthMm: 425, heightMm: 125 },
  { widthMm: 455, heightMm: 150 },
];

/**
 * Indicatieve netto vrije doorlaat (cm²) van een seed-rooster:
 * `b × h × fractie`, met de conservatieve fractie voor standaard of
 * geluidswerende uitvoering (zie de doc-comments op de fractie-constantes).
 */
export function grilleNetAreaCm2(
  size: DoorGrilleSize,
  acoustic = false,
): number {
  const fraction = acoustic
    ? GRILLE_NET_AREA_FRACTION_ACOUSTIC
    : GRILLE_NET_AREA_FRACTION;
  // mm² → cm² (÷100).
  return ((size.widthMm * size.heightMm) / 100) * fraction;
}

/** Rooster-voorstel van {@link proposeDoorGrille}. */
export interface GrilleProposal {
  /** Voorgestelde generieke maat (uit {@link DOOR_GRILLE_SEED}). */
  size: DoorGrilleSize;
  /** Aantal roosters (1, 2, of meer wanneer zelfs 2× de grootste te klein is). */
  count: number;
  /** Indicatieve netto doorlaat per rooster in cm². */
  netAreaCm2PerGrille: number;
  /** Totale indicatieve netto doorlaat (`count × per rooster`) in cm². */
  totalNetAreaCm2: number;
  /** Geluidswerende uitvoering (bepaalt de gebruikte netto-fractie). */
  acoustic: boolean;
}

/**
 * Stel het kleinste deurrooster uit de seed voor waarvan de indicatieve
 * netto doorlaat de benodigde doorlaat haalt; past dat niet in één rooster,
 * dan het kleinste rooster dat het in 2× haalt. Haalt zelfs 2× de grootste
 * maat het niet, dan de grootste maat met het benodigde aantal (naar boven
 * afgerond) — het voorstel blijft dan bruikbaar als maatwerk-indicatie.
 *
 * De benodigde netto doorlaat is dezelfde berekende doorlaat als voor de
 * spleet ({@link gapHeightMm}, `areaCm2`) — het Δp-criterium verandert niet
 * door de uitvoeringsvorm. `null` bij doorlaat ≤ 0 (geen rooster nodig).
 *
 * **Indicatief** — generieke maatvoeringen + conservatieve netto-fracties,
 * geen fabrikantdata; controleer het productblad.
 */
export function proposeDoorGrille(
  requiredNetAreaCm2: number,
  acoustic = false,
): GrilleProposal | null {
  if (requiredNetAreaCm2 <= 0) return null;

  const toProposal = (size: DoorGrilleSize, count: number): GrilleProposal => {
    const per = grilleNetAreaCm2(size, acoustic);
    return {
      size,
      count,
      netAreaCm2PerGrille: per,
      totalNetAreaCm2: per * count,
      acoustic,
    };
  };

  // Kleinste maat die het in 1× haalt, anders kleinste die het in 2× haalt.
  for (const count of [1, 2]) {
    for (const size of DOOR_GRILLE_SEED) {
      if (grilleNetAreaCm2(size, acoustic) * count >= requiredNetAreaCm2) {
        return toProposal(size, count);
      }
    }
  }

  // Fallback: grootste maat × benodigd aantal (maatwerk-indicatie).
  const largest = DOOR_GRILLE_SEED[DOOR_GRILLE_SEED.length - 1]!;
  const count = Math.ceil(
    requiredNetAreaCm2 / grilleNetAreaCm2(largest, acoustic),
  );
  return toProposal(largest, count);
}
