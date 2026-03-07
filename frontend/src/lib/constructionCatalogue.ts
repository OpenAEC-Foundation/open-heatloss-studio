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

// ---------------------------------------------------------------------------
// Constructiebibliotheek
//
// Alle opbouwen met laagdetail.  Lagen-volgorde:
//   Wanden:  binnen → buiten
//   Daken:   plat dak  = buiten → binnen (top → bottom)
//            hellend dak = binnen → buiten
//   Vloeren: boven → onder (top → bottom)
//
// U-waarden zijn berekend uit de lagen incl. Rsi/Rse:
//   Wand horizontaal:   Rsi = 0.13, Rse = 0.04
//   Dak opwaarts:       Rsi = 0.10, Rse = 0.04
//   Vloer neerwaarts:   Rsi = 0.17, Rse = 0.04
//   Binnenwand:         Rsi = 0.13 aan beide zijden
//
// Bronnen lambda-waarden: materialsDatabase.ts (NEN-EN ISO 10456, DIN 4108-4)
// ---------------------------------------------------------------------------

export const CONSTRUCTION_CATALOGUE: CatalogueEntry[] = [

  // ===== WANDEN — Buitenwanden metselwerk =====

  {
    id: "spouwmuur-nieuwbouw",
    name: "Spouwmuur nieuwbouw (Rc\u22484.7)",
    category: "wanden",
    uValue: 0.19,
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
    id: "spouwmuur-standaard",
    name: "Spouwmuur standaard (Rc\u22483.5)",
    category: "wanden",
    uValue: 0.30,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "metselwerk-kalkzandsteen", thickness: 100 },
      { materialId: "isolatie-mineraal-minerale-wol-platen", thickness: 100 },
      { materialId: "spouw-spouw-niet-gevent-rd-0-17", thickness: 30 },
      { materialId: "metselwerk-b4-gevelklinkers", thickness: 100 },
    ],
  },
  {
    id: "buitenwand-metselwerk",
    name: "Spouwmuur bestaande bouw (Rc\u22482.5)",
    category: "wanden",
    uValue: 0.36,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "metselwerk-kalkzandsteen", thickness: 100 },
      { materialId: "isolatie-mineraal-minerale-wol-platen", thickness: 80 },
      { materialId: "spouw-spouw-niet-gevent-rd-0-17", thickness: 30 },
      { materialId: "metselwerk-b4-gevelklinkers", thickness: 100 },
    ],
  },
  {
    id: "spouwmuur-bestaand-na-isolatie",
    name: "Spouwmuur na-ge\u00EFsoleerd (Rc\u22481.3)",
    category: "wanden",
    uValue: 0.62,
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
    id: "spouwmuur-ongeisoleerd",
    name: "Spouwmuur onge\u00EFsoleerd (jaren \u201960)",
    category: "wanden",
    uValue: 1.46,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 15 },
      { materialId: "metselwerk-baksteen-1000-kg-m", thickness: 100 },
      { materialId: "spouw-spouw-niet-gevent-rd-0-17", thickness: 60 },
      { materialId: "metselwerk-b1-rood", thickness: 100 },
    ],
  },

  // ===== WANDEN — Buitenwanden houtskelet =====

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
    id: "buitenwand-houtskelet",
    name: "Houtskeletwand standaard (Rc\u22483.5)",
    category: "wanden",
    uValue: 0.28,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "plaatmateriaal-gipskartonplaat", thickness: 12.5 },
      { materialId: "folie-dampremmend-pe-folie-0-15mm", thickness: 0 },
      { materialId: "isolatie-mineraal-minerale-wol-platen", thickness: 110 },
      { materialId: "hout-osb", thickness: 12 },
      { materialId: "spouw-spouw-gevent-rd-0-09", thickness: 25 },
      { materialId: "plaatmateriaal-vezelcementplaat", thickness: 8 },
    ],
  },

  // ===== WANDEN — Buitenwanden overig =====

  {
    id: "buitenwand-etics-cellenbeton",
    name: "Buitenwand ETICS cellenbeton (Rc\u22483.5)",
    category: "wanden",
    uValue: 0.27,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "beton-cellenbeton-600", thickness: 200 },
      { materialId: "isolatie-kunststof-eps", thickness: 100 },
      { materialId: "afwerking-sierpleister-mineraal", thickness: 8 },
    ],
  },

  // ===== WANDEN — Binnenwanden =====

  {
    id: "binnenwand-kalkzandsteen",
    name: "Binnenwand kalkzandsteen 100mm",
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
  {
    id: "binnenwand-licht",
    name: "Binnenwand metalstud/gips",
    category: "wanden",
    uValue: 2.22,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_room",
    isBuiltIn: true,
    layers: [
      { materialId: "plaatmateriaal-gipskartonplaat", thickness: 12.5 },
      { materialId: "spouw-spouw-gevent-rd-0-09", thickness: 48 },
      { materialId: "plaatmateriaal-gipskartonplaat", thickness: 12.5 },
    ],
  },
  {
    id: "binnenwand-cellenbeton",
    name: "Binnenwand cellenbeton 100mm",
    category: "wanden",
    uValue: 1.52,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_room",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "beton-cellenbeton-600", thickness: 100 },
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
    ],
  },

  // ===== WANDEN — Woningscheidend =====

  {
    id: "woningscheidende-wand",
    name: "Woningscheidende wand (dubbel KZS)",
    category: "wanden",
    uValue: 1.64,
    materialType: "masonry",
    verticalPosition: "wall",
    boundaryType: "adjacent_building",
    isBuiltIn: true,
    layers: [
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
      { materialId: "metselwerk-kalkzandsteen", thickness: 100 },
      { materialId: "spouw-spouw-niet-gevent-rd-0-17", thickness: 40 },
      { materialId: "metselwerk-kalkzandsteen", thickness: 100 },
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
    ],
  },

  // ===== VLOEREN / PLAFONDS — Ge\u00EFsoleerd =====

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
    id: "betonvloer-geisoleerd",
    name: "Breedplaatvloer ge\u00EFsoleerd (Rc\u22483.0)",
    category: "vloeren_plafonds",
    uValue: 0.31,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "beton-cementdekvloer", thickness: 60 },
      { materialId: "isolatie-kunststof-eps", thickness: 100 },
      { materialId: "beton-breedplaatvloer", thickness: 200 },
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
    ],
  },
  {
    id: "houten-vloer-geisoleerd",
    name: "Houten vloer ge\u00EFsoleerd (Rc\u22483.2)",
    category: "vloeren_plafonds",
    uValue: 0.29,
    materialType: "non_masonry",
    verticalPosition: "floor",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "vloer-parket-massief", thickness: 15 },
      { materialId: "hout-osb", thickness: 18 },
      { materialId: "isolatie-mineraal-minerale-wol-platen", thickness: 100 },
      { materialId: "hout-naaldhout", thickness: 22 },
    ],
  },
  {
    id: "verdiepingsvloer-geisoleerd",
    name: "Verdiepingsvloer ge\u00EFsoleerd (Rc\u22482.5)",
    category: "vloeren_plafonds",
    uValue: 0.37,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "unheated_space",
    isBuiltIn: true,
    layers: [
      { materialId: "beton-cementdekvloer", thickness: 50 },
      { materialId: "isolatie-mineraal-minerale-wol-platen", thickness: 80 },
      { materialId: "beton-kanaalplaatvloer", thickness: 200 },
      { materialId: "afwerking-stucwerk-gips", thickness: 10 },
    ],
  },

  // ===== VLOEREN / PLAFONDS — Onge\u00EFsoleerd =====

  {
    id: "tussenvloer-beton",
    name: "Tussenvloer beton (onge\u00EFsoleerd)",
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
    id: "begane-grondvloer",
    name: "Begane grondvloer (onge\u00EFsoleerd)",
    category: "vloeren_plafonds",
    uValue: 2.75,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "ground",
    isBuiltIn: true,
    layers: [
      { materialId: "beton-cementdekvloer", thickness: 50 },
      { materialId: "beton-beton-gewapend", thickness: 200 },
    ],
  },
  {
    id: "betonvloer-ongeisoleerd",
    name: "Breedplaatvloer onge\u00EFsoleerd",
    category: "vloeren_plafonds",
    uValue: 2.54,
    materialType: "masonry",
    verticalPosition: "floor",
    boundaryType: "unheated_space",
    isBuiltIn: true,
    layers: [
      { materialId: "beton-cementdekvloer", thickness: 50 },
      { materialId: "beton-breedplaatvloer", thickness: 200 },
      { materialId: "afwerking-stucwerk-gips", thickness: 15 },
    ],
  },

  // ===== DAKEN — Ge\u00EFsoleerd =====

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
    id: "plat-dak-geisoleerd",
    name: "Plat dak ge\u00EFsoleerd (Rc\u22484.5)",
    category: "daken",
    uValue: 0.22,
    materialType: "masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "folie-overig-bitumen-sbs", thickness: 5 },
      { materialId: "isolatie-kunststof-pir-alu-bekleed", thickness: 95 },
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
    id: "hellend-dak-geisoleerd",
    name: "Hellend dak ge\u00EFsoleerd (Rc\u22483.8)",
    category: "daken",
    uValue: 0.25,
    materialType: "non_masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "plaatmateriaal-gipskartonplaat", thickness: 12.5 },
      { materialId: "folie-dampremmend-pe-folie-0-15mm", thickness: 0 },
      { materialId: "isolatie-mineraal-minerale-wol-platen", thickness: 130 },
      { materialId: "hout-naaldhout", thickness: 18 },
    ],
  },

  // ===== DAKEN — Onge\u00EFsoleerd =====

  {
    id: "plat-dak-ongeisoleerd",
    name: "Plat dak onge\u00EFsoleerd",
    category: "daken",
    uValue: 3.58,
    materialType: "masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "folie-overig-bitumen-sbs", thickness: 5 },
      { materialId: "beton-beton-gewapend", thickness: 150 },
      { materialId: "afwerking-stucwerk-gips", thickness: 15 },
    ],
  },
  {
    id: "hellend-dak-ongeisoleerd",
    name: "Hellend dak onge\u00EFsoleerd",
    category: "daken",
    uValue: 3.38,
    materialType: "non_masonry",
    verticalPosition: "ceiling",
    boundaryType: "exterior",
    isBuiltIn: true,
    layers: [
      { materialId: "plaatmateriaal-gipskartonplaat", thickness: 12.5 },
      { materialId: "hout-naaldhout", thickness: 18 },
    ],
  },

  // ===== KOZIJNEN / VULLINGEN (geen laag-detail) =====

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
    id: "buitendeur-geisoleerd",
    name: "Buitendeur (ge\u00EFsoleerd)",
    category: "kozijnen_vullingen",
    uValue: 1.5,
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
    id: "dubbel-glas",
    name: "Dubbel glas (oud)",
    category: "kozijnen_vullingen",
    uValue: 2.9,
    materialType: "non_masonry",
    verticalPosition: "wall",
    boundaryType: "exterior",
    isBuiltIn: true,
  },
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
