import type { BoundaryType, MaterialType, VerticalPosition } from "../types";

export interface CatalogueLayer {
  materialId: string;
  /** Laagdikte in mm. */
  thickness: number;
}

export interface CatalogueEntry {
  id: string;
  name: string;
  category: CatalogueCategory;
  uValue: number;
  materialType: MaterialType;
  verticalPosition: VerticalPosition;
  boundaryType?: BoundaryType;
  isBuiltIn?: boolean;
  /** Optioneel: laag-detail voor Rc/U berekening. */
  layers?: CatalogueLayer[];
}

export type CatalogueCategory =
  | "wanden"
  | "vloeren_plafonds"
  | "daken"
  | "kozijnen_vullingen";

export const CATALOGUE_CATEGORY_LABELS: Record<CatalogueCategory, string> = {
  wanden: "Wanden",
  vloeren_plafonds: "Vloeren / plafonds",
  daken: "Daken",
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
    isBuiltIn: true,
  },
  {
    id: "binnenwand-licht",
    name: "Binnenwand (licht)",
    category: "wanden",
    uValue: 2.17,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_room",
    isBuiltIn: true,
  },
  {
    id: "woningscheidende-wand",
    name: "Woningscheidende wand",
    category: "wanden",
    uValue: 2.08,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_building",
    isBuiltIn: true,
  },
  {
    id: "buitenwand-houtskelet",
    name: "Buitenwand (houtskelet)",
    category: "wanden",
    uValue: 0.28,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
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
    isBuiltIn: true,
  },
  {
    id: "betonvloer-geisoleerd",
    name: "Betonvloer (ge\u00EFsoleerd)",
    category: "vloeren_plafonds",
    uValue: 0.35,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
  {
    id: "begane-grondvloer",
    name: "Begane grondvloer",
    category: "vloeren_plafonds",
    uValue: 0.29,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "ground",
    isBuiltIn: true,
  },
  // -- Daken --
  {
    id: "plat-dak-geisoleerd",
    name: "Plat dak (ge\u00EFsoleerd)",
    category: "daken",
    uValue: 0.22,
    materialType: "masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
  {
    id: "plat-dak-ongeisoleerd",
    name: "Plat dak (onge\u00EFsoleerd)",
    category: "daken",
    uValue: 3.5,
    materialType: "masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
  {
    id: "hellend-dak-geisoleerd",
    name: "Hellend dak (ge\u00EFsoleerd)",
    category: "daken",
    uValue: 0.25,
    materialType: "non_masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
  {
    id: "hellend-dak-ongeisoleerd",
    name: "Hellend dak (onge\u00EFsoleerd)",
    category: "daken",
    uValue: 3.0,
    materialType: "non_masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
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
    isBuiltIn: true,
  },
  {
    id: "dubbel-glas-hr",
    name: "Dubbel glas HR++",
    category: "kozijnen_vullingen",
    uValue: 1.1,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
  {
    id: "triple-glas",
    name: "Triple glas",
    category: "kozijnen_vullingen",
    uValue: 0.7,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
  {
    id: "buitendeur-hout",
    name: "Buitendeur (hout)",
    category: "kozijnen_vullingen",
    uValue: 2.78,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
  {
    id: "buitendeur-geisoleerd",
    name: "Buitendeur (ge\u00EFsoleerd)",
    category: "kozijnen_vullingen",
    uValue: 1.5,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
  },

  // -- Constructies met laag-detail (Bouwbesluit 2024 / RVO) --

  {
    id: "spouwmuur-nieuwbouw",
    name: "Spouwmuur nieuwbouw (Rc\u22484.7)",
    category: "wanden",
    uValue: 0.20,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "metselwerk-kalkzandsteen", thickness: 100 },
      { materialId: "isolatie-kunststof-pir", thickness: 110 },
      { materialId: "spouw-spouw-niet-gevent-rd-0-17", thickness: 40 },
      { materialId: "metselwerk-b4-gevelklinkers", thickness: 100 },
    ],
  },
  {
    id: "spouwmuur-bestaand-na-isolatie",
    name: "Spouwmuur bestaand (na-isolatie, Rc\u22481.3)",
    category: "wanden",
    uValue: 0.68,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "metselwerk-kalkzandsteen", thickness: 100 },
      { materialId: "isolatie-mineraal-minerale-wol-dekens", thickness: 50 },
      { materialId: "metselwerk-b4-gevelklinkers", thickness: 100 },
    ],
  },
  {
    id: "houtskeletwand-nieuwbouw",
    name: "Houtskeletwand nieuwbouw (Rc\u22485.0)",
    category: "wanden",
    uValue: 0.19,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "plaatmateriaal-gipskartonplaat", thickness: 12.5 },
      { materialId: "folie-dampremmend-pe-folie-0-15mm", thickness: 0 },
      { materialId: "hout-osb", thickness: 12 },
      { materialId: "isolatie-mineraal-minerale-wol-platen", thickness: 140 },
      { materialId: "hout-osb", thickness: 12 },
      { materialId: "spouw-spouw-gevent-rd-0-09", thickness: 25 },
      { materialId: "plaatmateriaal-vezelcementplaat", thickness: 8 },
    ],
  },
  {
    id: "begane-grondvloer-nieuwbouw",
    name: "Begane grondvloer nieuwbouw (Rc\u22483.7)",
    category: "vloeren_plafonds",
    uValue: 0.26,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "ground",
    isBuiltIn: true,
    layers: [
      { materialId: "beton-cementdekvloer", thickness: 60 },
      { materialId: "isolatie-kunststof-eps", thickness: 120 },
      { materialId: "beton-beton-gewapend", thickness: 200 },
    ],
  },
  {
    id: "tussenvloer-beton",
    name: "Tussenvloer beton",
    category: "vloeren_plafonds",
    uValue: 1.79,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "adjacent_room",
    isBuiltIn: true,
    layers: [
      { materialId: "beton-cementdekvloer", thickness: 60 },
      { materialId: "beton-kanaalplaatvloer", thickness: 200 },
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
    ],
  },
  {
    id: "plat-dak-nieuwbouw",
    name: "Plat dak nieuwbouw (Rc\u22486.3)",
    category: "daken",
    uValue: 0.15,
    materialType: "masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "folie-overig-bitumen-sbs", thickness: 5 },
      { materialId: "isolatie-kunststof-pir-alu-bekleed", thickness: 140 },
      { materialId: "folie-dampremmend-pe-folie-0-15mm", thickness: 0 },
      { materialId: "beton-beton-gewapend", thickness: 200 },
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
    ],
  },
  {
    id: "hellend-dak-nieuwbouw",
    name: "Hellend dak nieuwbouw (Rc\u22486.3)",
    category: "daken",
    uValue: 0.15,
    materialType: "non_masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "plaatmateriaal-gipskartonplaat", thickness: 12.5 },
      { materialId: "folie-dampremmend-pe-folie-0-15mm", thickness: 0 },
      { materialId: "isolatie-kunststof-pir-alu-bekleed", thickness: 140 },
      { materialId: "hout-naaldhout", thickness: 18 },
    ],
  },
  {
    id: "binnenwand-kalkzandsteen",
    name: "Binnenwand kalkzandsteen",
    category: "wanden",
    uValue: 2.87,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_room",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "metselwerk-kalkzandsteen", thickness: 100 },
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
    ],
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
