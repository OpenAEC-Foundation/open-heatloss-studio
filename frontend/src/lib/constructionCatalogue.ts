import type { BoundaryType, MaterialType, VerticalPosition } from "../types";
import type { ProjectConstruction } from "../components/modeller/types";
import { calculateRc } from "./rcCalculation";

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
// Korte laagnamen voor weergave in constructienamen
// ---------------------------------------------------------------------------

const SHORT_NAMES: Record<string, string> = {
  "afwerking-stucwerk-gips": "Stuc",
  "metselwerk-kalkzandsteen": "KZS",
  "isolatie-kunststof-pir": "PIR",
  "spouw-spouw-niet-gevent-rd-0-17": "Spouw",
  "spouw-spouw-gevent-rd-0-09": "Spouw(v)",
  "metselwerk-b4-gevelklinkers": "Klinker",
  "plaatmateriaal-gipskartonplaat": "Gips",
  "folie-dampremmend-pe-folie-0-15mm": "PE-folie",
  "hout-osb": "OSB",
  "isolatie-mineraal-minerale-wol-platen": "MW",
  "isolatie-mineraal-minerale-wol-dekens": "MWd",
  "plaatmateriaal-vezelcementplaat": "VCement",
  "beton-cellenbeton-600": "CB600",
  "isolatie-kunststof-eps": "EPS",
  "afwerking-sierpleister-mineraal": "Sierpl",
  "beton-cementdekvloer": "Dekvloer",
  "beton-beton-gewapend": "Beton",
  "beton-breedplaatvloer": "Breedpl",
  "beton-kanaalplaatvloer": "Kanaalpl",
  "hout-naaldhout": "Nhout",
  "vloer-parket-massief": "Parket",
  "folie-overig-bitumen-sbs": "Bitumen",
  "isolatie-kunststof-pir-alu-bekleed": "PIRalu",
  "metselwerk-baksteen-1000-kg-m": "Bak1000",
  "metselwerk-b1-rood": "B1",
};

/** Build a human-readable name from layer composition. */
export function buildLayerName(layers: CatalogueLayer[]): string {
  return layers
    .map((l) => {
      const short = SHORT_NAMES[l.materialId] ?? l.materialId;
      return l.thickness > 0 ? `${short} ${l.thickness}` : short;
    })
    .join(" | ");
}

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
    name: "Stuc 10 | KZS 100 | PIR 110 | Spouw 40 | Klinker 100",
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
    name: "Stuc 10 | KZS 100 | MW 100 | Spouw 30 | Klinker 100",
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
    name: "Stuc 10 | KZS 100 | MW 80 | Spouw 30 | Klinker 100",
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
    name: "Stuc 10 | KZS 100 | MWd 50 | Klinker 100",
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
    name: "Stuc 15 | Bak1000 100 | Spouw 60 | B1 100",
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
    name: "Gips 12.5 | PE-folie | OSB 12 | MW 140 | OSB 12 | Spouw(v) 25 | VCement 8",
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
    name: "Gips 12.5 | PE-folie | MW 110 | OSB 12 | Spouw(v) 25 | VCement 8",
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
    name: "Stuc 10 | CB600 200 | EPS 100 | Sierpl 8",
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
    name: "Stuc 10 | KZS 100 | Stuc 10",
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
    name: "Gips 12.5 | Spouw(v) 48 | Gips 12.5",
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
    name: "Stuc 10 | CB600 100 | Stuc 10",
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
    name: "Stuc 10 | KZS 100 | Spouw 40 | KZS 100 | Stuc 10",
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
    name: "Dekvloer 60 | EPS 120 | Beton 200",
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
    name: "Dekvloer 60 | EPS 100 | Breedpl 200 | Stuc 10",
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
    name: "Parket 15 | OSB 18 | MW 100 | Nhout 22",
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
    name: "Dekvloer 50 | MW 80 | Kanaalpl 200 | Stuc 10",
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
    name: "Dekvloer 60 | Kanaalpl 200 | Stuc 10",
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
    name: "Dekvloer 50 | Beton 200",
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
    name: "Dekvloer 50 | Breedpl 200 | Stuc 15",
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
    name: "Bitumen 5 | PIRalu 140 | PE-folie | Beton 200 | Stuc 10",
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
    name: "Bitumen 5 | PIRalu 95 | PE-folie | Beton 200 | Stuc 10",
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
    name: "Gips 12.5 | PE-folie | PIRalu 140 | Nhout 18",
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
    name: "Gips 12.5 | PE-folie | MW 130 | Nhout 18",
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
    name: "Bitumen 5 | Beton 150 | Stuc 15",
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
    name: "Gips 12.5 | Nhout 18",
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

// ---------------------------------------------------------------------------
// Unified lookup: project constructions + catalogue
// ---------------------------------------------------------------------------

/**
 * Resolve a construction by ID — searches project constructions first,
 * then falls back to the global catalogue.
 *
 * For project constructions, the U-value is calculated on-the-fly from layers.
 */
export function resolveConstruction(
  entryId: string,
  catalogueEntries: CatalogueEntry[],
  projectConstructions: ProjectConstruction[],
): CatalogueEntry | null {
  // Check project constructions first (proj- prefix)
  const projEntry = projectConstructions.find((c) => c.id === entryId);
  if (projEntry) {
    const rcResult = calculateRc(projEntry.layers, projEntry.verticalPosition);
    return {
      id: projEntry.id,
      name: projEntry.name,
      category: projEntry.category,
      uValue: Math.round(rcResult.uValue * 1000) / 1000,
      materialType: projEntry.materialType,
      verticalPosition: projEntry.verticalPosition,
      layers: projEntry.layers,
    };
  }

  // Fall back to catalogue
  return catalogueEntries.find((e) => e.id === entryId) ?? null;
}
