/**
 * Selectors en helpers voor de project-brede U-waarde override voor
 * kozijnen (categorie `kozijnen_vullingen`).
 *
 * De override wordt gezet via `useProjectStore.setFrameUValueOverride`
 * en via `getEffectiveFrameUValue()` uitgelezen tijdens weergave of
 * vlak voor de backend-call. De onderliggende per-element `u_value` in
 * de store wordt NOOIT gemuteerd — de override werkt puur als selector.
 */

import type { ConstructionElement, Project } from "../types";
import type { ProjectConstruction } from "../components/modeller/types";

/** Categorie in de construction catalogue waarop de override van toepassing is. */
const FRAME_CATEGORY = "kozijnen_vullingen" as const;

/**
 * Controleert of een construction-element een kozijn is (window, door,
 * curtain_panel). Dit gebeurt via de link naar een `ProjectConstruction`
 * met categorie `kozijnen_vullingen`. Handmatig ingevoerde elementen
 * zonder `project_construction_id` vallen hier niet onder — de gebruiker
 * behoudt daar volle controle.
 */
export function isFrameConstruction(
  element: ConstructionElement,
  projectConstructions: readonly ProjectConstruction[],
): boolean {
  if (!element.project_construction_id) {
    return false;
  }
  const pc = projectConstructions.find(
    (c) => c.id === element.project_construction_id,
  );
  return pc?.category === FRAME_CATEGORY;
}

/**
 * Is de frame U-waarde override actief (numeriek, eindig, > 0)?
 */
export function isFrameOverrideActive(
  override: number | null | undefined,
): override is number {
  return (
    typeof override === "number" &&
    Number.isFinite(override) &&
    override > 0
  );
}

/**
 * Geeft de effectieve U-waarde voor een construction-element terug.
 *
 * - Als de project-brede `frameUValueOverride` actief is EN het element
 *   een kozijn is → return de override.
 * - Anders → return de per-element `u_value`.
 *
 * Deze helper muteert niets — de store blijft bron-van-waarheid voor
 * per-element waarden. Gebruik voor weergave en vlak voor de backend-
 * call (zie `prepareProjectForCalculation`).
 */
export function getEffectiveFrameUValue(
  element: ConstructionElement,
  project: Pick<Project, "frameUValueOverride">,
  projectConstructions: readonly ProjectConstruction[],
): number {
  if (
    isFrameOverrideActive(project.frameUValueOverride) &&
    isFrameConstruction(element, projectConstructions)
  ) {
    return project.frameUValueOverride;
  }
  return element.u_value;
}

/**
 * Bereid het project voor op verzending naar de rekenkern.
 *
 * Wanneer `frameUValueOverride` actief is, worden de `u_value`s van
 * alle kozijn-elementen door de override vervangen — maar ALLEEN op de
 * (deep-)gekopieerde payload, niet op de bron in de zustand store.
 *
 * Ook het `frameUValueOverride`-veld zelf wordt verwijderd uit de
 * payload zodat het niet meegestuurd wordt naar de backend (de Rust
 * core kent dit veld niet).
 */
export function prepareProjectForCalculation(
  project: Project,
  projectConstructions: readonly ProjectConstruction[],
): Project {
  const override = project.frameUValueOverride;
  // Deep clone zodat we niets in de store aanraken.
  const cloned: Project = structuredClone(project);
  // Frame-override-veld mag niet mee naar de backend.
  delete cloned.frameUValueOverride;

  if (!isFrameOverrideActive(override)) {
    return cloned;
  }

  for (const room of cloned.rooms) {
    for (const ce of room.constructions) {
      if (isFrameConstruction(ce, projectConstructions)) {
        ce.u_value = override;
      }
    }
  }
  return cloned;
}
