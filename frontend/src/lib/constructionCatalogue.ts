import type { BoundaryType, MaterialType, VerticalPosition } from "../types";

export interface CatalogueEntry {
  id: string;
  name: string;
  category: CatalogueCategory;
  uValue: number;
  materialType: MaterialType;
  verticalPosition: VerticalPosition;
  boundaryType: BoundaryType;
}

export type CatalogueCategory = "wanden" | "vloeren_plafonds" | "kozijnen_vullingen";

export const CATALOGUE_CATEGORY_LABELS: Record<CatalogueCategory, string> = {
  wanden: "Wanden",
  vloeren_plafonds: "Vloeren / plafonds",
  kozijnen_vullingen: "Kozijnen / vullingen",
};

export const CONSTRUCTION_CATALOGUE: CatalogueEntry[] = [
  // -- Wanden --
  {
    id: "buitenwand-metselwerk",
    name: "Buitenwand (metselwerk)",
    category: "wanden",
    uValue: 0.36,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
  },
  {
    id: "binnenwand-licht",
    name: "Binnenwand (licht)",
    category: "wanden",
    uValue: 2.17,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_room",
  },
  {
    id: "woningscheidende-wand",
    name: "Woningscheidende wand",
    category: "wanden",
    uValue: 2.08,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_building",
  },
  {
    id: "buitenwand-houtskelet",
    name: "Buitenwand (houtskelet)",
    category: "wanden",
    uValue: 0.28,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
  },
  // -- Vloeren / plafonds --
  {
    id: "betonvloer-ongeisoleerd",
    name: "Betonvloer (onge\u00EFsoleerd)",
    category: "vloeren_plafonds",
    uValue: 2.5,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "unheated_space",
  },
  {
    id: "betonvloer-geisoleerd",
    name: "Betonvloer (ge\u00EFsoleerd)",
    category: "vloeren_plafonds",
    uValue: 0.35,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "exterior",
  },
  {
    id: "begane-grondvloer",
    name: "Begane grondvloer",
    category: "vloeren_plafonds",
    uValue: 0.29,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "ground",
  },
  // -- Kozijnen / vullingen --
  {
    id: "enkel-glas",
    name: "Enkel glas",
    category: "kozijnen_vullingen",
    uValue: 5.8,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
  },
  {
    id: "dubbel-glas-hr",
    name: "Dubbel glas HR++",
    category: "kozijnen_vullingen",
    uValue: 1.1,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
  },
  {
    id: "triple-glas",
    name: "Triple glas",
    category: "kozijnen_vullingen",
    uValue: 0.7,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
  },
  {
    id: "buitendeur-hout",
    name: "Buitendeur (hout)",
    category: "kozijnen_vullingen",
    uValue: 2.78,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
  },
  {
    id: "buitendeur-geisoleerd",
    name: "Buitendeur (ge\u00EFsoleerd)",
    category: "kozijnen_vullingen",
    uValue: 1.5,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
  },
];

/** Group catalogue entries by category. */
export function getCatalogueByCategory(): Map<CatalogueCategory, CatalogueEntry[]> {
  const map = new Map<CatalogueCategory, CatalogueEntry[]>();
  for (const entry of CONSTRUCTION_CATALOGUE) {
    const list = map.get(entry.category) ?? [];
    list.push(entry);
    map.set(entry.category, list);
  }
  return map;
}
