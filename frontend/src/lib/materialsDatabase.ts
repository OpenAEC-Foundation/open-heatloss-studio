/**
 * Materialendatabase voor constructie-opbouwen.
 *
 * Bron: DIN 4108-4, NEN-EN ISO 10456, fabrikantspecs.
 * Gemigreerd uit pyRevit materialen_database.json v2.0
 */

export type MaterialCategory =
  | "spouw"
  | "mortel"
  | "natuursteen"
  | "metselwerk"
  | "beton"
  | "plaatmateriaal"
  | "hout"
  | "isolatie_mineraal"
  | "isolatie_kunststof"
  | "isolatie_natuurlijk"
  | "folie"
  | "afwerking"
  | "vloer"
  | "metaal"
  | "kunststof"
  | "glas"
  | "overig";

export interface Material {
  id: string;
  name: string;
  category: MaterialCategory;
  /** Warmtegeleidingscoëfficiënt [W/(m·K)]. null voor folies/spouwen. */
  lambda: number | null;
  /** Dampweerstandsfactor [-]. */
  mu: number;
  /** Dichtheid [kg/m³]. null voor folies. */
  rho: number | null;
  /** Vaste Rd-waarde [m²·K/W] voor spouwen/folies. null = bereken via d/λ. */
  rdFixed: number | null;
  /** Zoektermen voor fuzzy search. */
  keywords: string[];
}

export const MATERIAL_CATEGORY_LABELS: Record<MaterialCategory, string> = {
  spouw: "Spouw",
  mortel: "Mortel",
  natuursteen: "Natuursteen",
  metselwerk: "Metselwerk",
  beton: "Beton",
  plaatmateriaal: "Plaatmateriaal",
  hout: "Hout",
  isolatie_mineraal: "Isolatie — mineraal",
  isolatie_kunststof: "Isolatie — kunststof",
  isolatie_natuurlijk: "Isolatie — natuurlijk",
  folie: "Folie / membraan",
  afwerking: "Afwerking",
  vloer: "Vloer",
  metaal: "Metaal",
  kunststof: "Kunststof",
  glas: "Glas",
  overig: "Overig",
};

/** Display-volgorde categorieën in MaterialPicker. */
export const MATERIAL_CATEGORY_ORDER: MaterialCategory[] = [
  "metselwerk",
  "beton",
  "hout",
  "plaatmateriaal",
  "isolatie_mineraal",
  "isolatie_kunststof",
  "isolatie_natuurlijk",
  "spouw",
  "folie",
  "afwerking",
  "vloer",
  "mortel",
  "natuursteen",
  "metaal",
  "kunststof",
  "glas",
  "overig",
];

/** Slug generator: lowercase, replace spaces/special chars with dashes. */
function slug(category: string, name: string): string {
  const base = `${category}-${name}`
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
  return base;
}

/** Map JSON categorie → TypeScript MaterialCategory. */
function mapCategory(cat: string): MaterialCategory {
  const mapping: Record<string, MaterialCategory> = {
    "Spouw": "spouw",
    "Mortel": "mortel",
    "Natuursteen": "natuursteen",
    "Metselwerk": "metselwerk",
    "Beton": "beton",
    "Plaatmateriaal": "plaatmateriaal",
    "Hout": "hout",
    "Isolatie - Mineraal": "isolatie_mineraal",
    "Isolatie - Kunststof": "isolatie_kunststof",
    "Isolatie - Natuurlijk": "isolatie_natuurlijk",
    "Folie - Dampremmend": "folie",
    "Folie - Miofol": "folie",
    "Folie - Pro Clima": "folie",
    "Folie - Overig": "folie",
    "Afwerking": "afwerking",
    "Vloer": "vloer",
    "Metaal": "metaal",
    "Kunststof": "kunststof",
    "Glas": "glas",
    "Overig": "overig",
  };
  return mapping[cat] ?? "overig";
}

// ---------- Inline database (uit pyRevit materialen_database.json v2.0) ----------

interface RawMaterial {
  categorie: string;
  naam: string;
  lambda: number | null;
  mu: number;
  rho: number | null;
  rd_vast: number | null;
  keywords: string[];
}

const RAW_MATERIALS: RawMaterial[] = [
  // Spouw
  { categorie: "Spouw", naam: "Spouw (niet gevent.) Rd=0.17", lambda: null, mu: 1, rho: null, rd_vast: 0.17, keywords: ["spouw", "lucht", "cavity", "air"] },
  { categorie: "Spouw", naam: "Spouw (gevent.) Rd=0.09", lambda: null, mu: 1, rho: null, rd_vast: 0.09, keywords: ["spouw", "geventileerd"] },
  { categorie: "Spouw", naam: "Spouw (sterk gevent.) Rd=0", lambda: null, mu: 1, rho: null, rd_vast: 0.0, keywords: ["spouw", "open"] },

  // Mortel
  { categorie: "Mortel", naam: "Cementmortel", lambda: 1.16, mu: 18, rho: 2000, rd_vast: null, keywords: ["cement", "mortel", "voeg"] },
  { categorie: "Mortel", naam: "Kalkmortel", lambda: 0.91, mu: 10, rho: 1800, rd_vast: null, keywords: ["kalk", "mortel"] },
  { categorie: "Mortel", naam: "Gipsmortel", lambda: 0.58, mu: 10, rho: 1400, rd_vast: null, keywords: ["gips", "mortel"] },
  { categorie: "Mortel", naam: "Isolerende mortel", lambda: 0.12, mu: 15, rho: 450, rd_vast: null, keywords: ["isolatie", "mortel"] },

  // Natuursteen
  { categorie: "Natuursteen", naam: "Basalt / Graniet", lambda: 4.34, mu: 10000, rho: 2900, rd_vast: null, keywords: ["basalt", "graniet", "natuursteen"] },
  { categorie: "Natuursteen", naam: "Kalksteen zacht", lambda: 1.94, mu: 30, rho: 2200, rd_vast: null, keywords: ["kalksteen", "natuursteen"] },
  { categorie: "Natuursteen", naam: "Zandsteen", lambda: 3.75, mu: 40, rho: 2600, rd_vast: null, keywords: ["zandsteen", "natuursteen"] },
  { categorie: "Natuursteen", naam: "Leisteen", lambda: 2.2, mu: 1000, rho: 2700, rd_vast: null, keywords: ["leisteen", "natuursteen"] },

  // Metselwerk
  { categorie: "Metselwerk", naam: "Kalkzandsteen", lambda: 1.44, mu: 15, rho: 1900, rd_vast: null, keywords: ["kalkzandsteen", "ks", "silka"] },
  { categorie: "Metselwerk", naam: "A2 (Poriso)", lambda: 0.56, mu: 8, rho: 1200, rd_vast: null, keywords: ["poriso", "a2", "isolatiesteen"] },
  { categorie: "Metselwerk", naam: "A3 (Isolatiesteen)", lambda: 0.41, mu: 8, rho: 1000, rd_vast: null, keywords: ["a3", "isolatiesteen"] },
  { categorie: "Metselwerk", naam: "B1 (Rood)", lambda: 0.72, mu: 10, rho: 1400, rd_vast: null, keywords: ["b1", "rood", "baksteen"] },
  { categorie: "Metselwerk", naam: "B4 (Gevelklinkers)", lambda: 1.065, mu: 50, rho: 1800, rd_vast: null, keywords: ["b4", "klinker", "gevel"] },
  { categorie: "Metselwerk", naam: "Baksteen 700 kg/m\u00B3", lambda: 0.39, mu: 8, rho: 700, rd_vast: null, keywords: ["baksteen", "licht"] },
  { categorie: "Metselwerk", naam: "Baksteen 1000 kg/m\u00B3", lambda: 0.57, mu: 10, rho: 1000, rd_vast: null, keywords: ["baksteen"] },
  { categorie: "Metselwerk", naam: "Baksteen 1400 kg/m\u00B3", lambda: 0.85, mu: 12, rho: 1400, rd_vast: null, keywords: ["baksteen"] },
  { categorie: "Metselwerk", naam: "Baksteen 1800 kg/m\u00B3", lambda: 1.16, mu: 16, rho: 1800, rd_vast: null, keywords: ["baksteen", "zwaar"] },
  { categorie: "Metselwerk", naam: "Poroton", lambda: 0.33, mu: 10, rho: 900, rd_vast: null, keywords: ["poroton", "thermosteen"] },
  { categorie: "Metselwerk", naam: "Snelbouw", lambda: 0.68, mu: 10, rho: 1350, rd_vast: null, keywords: ["snelbouw", "betonblok"] },

  // Beton
  { categorie: "Beton", naam: "Beton gewapend", lambda: 1.7, mu: 80, rho: 2400, rd_vast: null, keywords: ["beton", "gewapend", "constructief"] },
  { categorie: "Beton", naam: "Beton ongewapend", lambda: 1.3, mu: 70, rho: 2300, rd_vast: null, keywords: ["beton", "ongewapend"] },
  { categorie: "Beton", naam: "Cellenbeton 400", lambda: 0.16, mu: 6, rho: 400, rd_vast: null, keywords: ["cellenbeton", "ytong", "400"] },
  { categorie: "Beton", naam: "Cellenbeton 500", lambda: 0.24, mu: 8, rho: 500, rd_vast: null, keywords: ["cellenbeton", "ytong", "500"] },
  { categorie: "Beton", naam: "Cellenbeton 600", lambda: 0.28, mu: 8, rho: 600, rd_vast: null, keywords: ["cellenbeton", "ytong", "600"] },
  { categorie: "Beton", naam: "Cellenbeton 700", lambda: 0.32, mu: 10, rho: 700, rd_vast: null, keywords: ["cellenbeton", "ytong", "700"] },
  { categorie: "Beton", naam: "Cellenbeton 1000", lambda: 0.47, mu: 15, rho: 1000, rd_vast: null, keywords: ["cellenbeton", "ytong", "1000"] },
  { categorie: "Beton", naam: "Lichtbeton 1200", lambda: 0.55, mu: 10, rho: 1200, rd_vast: null, keywords: ["lichtbeton", "argex"] },
  { categorie: "Beton", naam: "Lichtbeton 1600", lambda: 0.80, mu: 15, rho: 1600, rd_vast: null, keywords: ["lichtbeton"] },
  { categorie: "Beton", naam: "Schuimbeton 400", lambda: 0.14, mu: 5, rho: 400, rd_vast: null, keywords: ["schuimbeton"] },
  { categorie: "Beton", naam: "Schuimbeton 600", lambda: 0.22, mu: 6, rho: 600, rd_vast: null, keywords: ["schuimbeton"] },
  { categorie: "Beton", naam: "Cementdekvloer", lambda: 1.4, mu: 30, rho: 2000, rd_vast: null, keywords: ["dekvloer", "cement", "afwerk"] },
  { categorie: "Beton", naam: "Anhydriet dekvloer", lambda: 1.2, mu: 20, rho: 2100, rd_vast: null, keywords: ["anhydriet", "dekvloer", "giet"] },
  { categorie: "Beton", naam: "Breedplaatvloer", lambda: 1.7, mu: 80, rho: 2400, rd_vast: null, keywords: ["breedplaat", "vloer", "prefab"] },
  { categorie: "Beton", naam: "Kanaalplaatvloer", lambda: 1.5, mu: 70, rho: 1800, rd_vast: null, keywords: ["kanaalplaat", "vloer", "hol"] },

  // Plaatmateriaal
  { categorie: "Plaatmateriaal", naam: "Gipskartonplaat", lambda: 0.25, mu: 8, rho: 900, rd_vast: null, keywords: ["gipskarton", "gips", "gyproc", "rigips"] },
  { categorie: "Plaatmateriaal", naam: "Gipskarton brandwerend", lambda: 0.25, mu: 8, rho: 1000, rd_vast: null, keywords: ["gipskarton", "brandwerend", "rf"] },
  { categorie: "Plaatmateriaal", naam: "Gipsvezelplaat (Fermacell)", lambda: 0.32, mu: 13, rho: 1150, rd_vast: null, keywords: ["fermacell", "gipsvezel"] },
  { categorie: "Plaatmateriaal", naam: "Cellulair glas (Foamglas)", lambda: 0.048, mu: 10000, rho: 120, rd_vast: null, keywords: ["foamglas", "cellulair", "glas"] },
  { categorie: "Plaatmateriaal", naam: "Cementgebonden plaat", lambda: 0.35, mu: 50, rho: 1200, rd_vast: null, keywords: ["cement", "plaat", "eternit"] },
  { categorie: "Plaatmateriaal", naam: "Vezelcementplaat", lambda: 0.35, mu: 30, rho: 1200, rd_vast: null, keywords: ["vezelcement", "eternit"] },
  { categorie: "Plaatmateriaal", naam: "Magnesiumoxideplaat", lambda: 0.21, mu: 15, rho: 1100, rd_vast: null, keywords: ["mgo", "magnesium"] },

  // Hout
  { categorie: "Hout", naam: "Naaldhout", lambda: 0.17, mu: 40, rho: 550, rd_vast: null, keywords: ["naaldhout", "vuren", "grenen", "den", "spar"] },
  { categorie: "Hout", naam: "Loofhout", lambda: 0.20, mu: 50, rho: 700, rd_vast: null, keywords: ["loofhout", "eiken", "beuken"] },
  { categorie: "Hout", naam: "Hardhout / Multiplex", lambda: 0.20, mu: 50, rho: 700, rd_vast: null, keywords: ["hardhout", "multiplex", "triplex"] },
  { categorie: "Hout", naam: "Tropisch hardhout", lambda: 0.24, mu: 100, rho: 900, rd_vast: null, keywords: ["tropisch", "iroko", "merbau", "azobe"] },
  { categorie: "Hout", naam: "OSB", lambda: 0.13, mu: 30, rho: 650, rd_vast: null, keywords: ["osb", "oriented strand"] },
  { categorie: "Hout", naam: "Spaanplaat", lambda: 0.14, mu: 15, rho: 650, rd_vast: null, keywords: ["spaanplaat", "particle"] },
  { categorie: "Hout", naam: "MDF", lambda: 0.14, mu: 20, rho: 750, rd_vast: null, keywords: ["mdf", "medium density"] },
  { categorie: "Hout", naam: "Hardboard (HDF)", lambda: 0.20, mu: 20, rho: 900, rd_vast: null, keywords: ["hardboard", "hdf"] },
  { categorie: "Hout", naam: "Zachtboard", lambda: 0.05, mu: 5, rho: 250, rd_vast: null, keywords: ["zachtboard", "houtvezel"] },
  { categorie: "Hout", naam: "Houtwolcement", lambda: 0.10, mu: 5, rho: 400, rd_vast: null, keywords: ["houtwol", "heraklith"] },
  { categorie: "Hout", naam: "CLT (Cross Laminated Timber)", lambda: 0.13, mu: 50, rho: 500, rd_vast: null, keywords: ["clt", "massief hout", "cross laminated"] },
  { categorie: "Hout", naam: "Accoya", lambda: 0.14, mu: 70, rho: 510, rd_vast: null, keywords: ["accoya", "gemodificeerd"] },
  { categorie: "Hout", naam: "Thermowood", lambda: 0.13, mu: 30, rho: 420, rd_vast: null, keywords: ["thermowood", "thermisch"] },

  // Isolatie - Mineraal
  { categorie: "Isolatie - Mineraal", naam: "Minerale wol (dekens)", lambda: 0.040, mu: 1, rho: 20, rd_vast: null, keywords: ["mineraal", "wol", "deken", "glaswol", "rockwool"] },
  { categorie: "Isolatie - Mineraal", naam: "Minerale wol (platen)", lambda: 0.035, mu: 1, rho: 50, rd_vast: null, keywords: ["mineraal", "wol", "plaat"] },
  { categorie: "Isolatie - Mineraal", naam: "Steenwol hoge dichtheid", lambda: 0.034, mu: 1, rho: 100, rd_vast: null, keywords: ["steenwol", "rockwool"] },
  { categorie: "Isolatie - Mineraal", naam: "Glaswol", lambda: 0.035, mu: 1, rho: 25, rd_vast: null, keywords: ["glaswol", "isover"] },
  { categorie: "Isolatie - Mineraal", naam: "Perlite", lambda: 0.05, mu: 2, rho: 100, rd_vast: null, keywords: ["perlite"] },
  { categorie: "Isolatie - Mineraal", naam: "Vermiculiet", lambda: 0.07, mu: 3, rho: 100, rd_vast: null, keywords: ["vermiculiet"] },

  // Isolatie - Kunststof
  { categorie: "Isolatie - Kunststof", naam: "EPS", lambda: 0.035, mu: 40, rho: 20, rd_vast: null, keywords: ["eps", "piepschuim", "tempex"] },
  { categorie: "Isolatie - Kunststof", naam: "EPS grijs (Neopor)", lambda: 0.032, mu: 40, rho: 18, rd_vast: null, keywords: ["eps", "grijs", "neopor"] },
  { categorie: "Isolatie - Kunststof", naam: "XPS", lambda: 0.034, mu: 150, rho: 35, rd_vast: null, keywords: ["xps", "roofmate", "styrodur"] },
  { categorie: "Isolatie - Kunststof", naam: "PUR", lambda: 0.026, mu: 50, rho: 35, rd_vast: null, keywords: ["pur", "polyurethaan"] },
  { categorie: "Isolatie - Kunststof", naam: "PIR", lambda: 0.023, mu: 50, rho: 35, rd_vast: null, keywords: ["pir", "iko", "recticel"] },
  { categorie: "Isolatie - Kunststof", naam: "PIR alu-bekleed", lambda: 0.022, mu: 50, rho: 35, rd_vast: null, keywords: ["pir", "alu", "dampgesloten"] },
  { categorie: "Isolatie - Kunststof", naam: "Resolschuim (phenol)", lambda: 0.020, mu: 30, rho: 45, rd_vast: null, keywords: ["resol", "phenol", "kingspan"] },
  { categorie: "Isolatie - Kunststof", naam: "PE-schuim", lambda: 0.04, mu: 10000, rho: 35, rd_vast: null, keywords: ["pe", "schuim", "ondervloer"] },

  // Isolatie - Natuurlijk
  { categorie: "Isolatie - Natuurlijk", naam: "Houtwol", lambda: 0.040, mu: 3, rho: 160, rd_vast: null, keywords: ["houtwol"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Houtvezelisolatie", lambda: 0.042, mu: 5, rho: 50, rd_vast: null, keywords: ["houtvezel", "pavatex", "steico"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Houtvezelplaat (droog)", lambda: 0.045, mu: 5, rho: 160, rd_vast: null, keywords: ["houtvezel", "plaat"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Cellulose (inblaas)", lambda: 0.040, mu: 2, rho: 50, rd_vast: null, keywords: ["cellulose", "inblaas"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Schapenwol", lambda: 0.040, mu: 2, rho: 25, rd_vast: null, keywords: ["schapenwol", "wol"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Hennep", lambda: 0.042, mu: 2, rho: 35, rd_vast: null, keywords: ["hennep", "thermo-hemp"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Vlas", lambda: 0.040, mu: 2, rho: 30, rd_vast: null, keywords: ["vlas"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Kurk (platen)", lambda: 0.045, mu: 10, rho: 120, rd_vast: null, keywords: ["kurk"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Kurk (granulaat)", lambda: 0.055, mu: 10, rho: 80, rd_vast: null, keywords: ["kurk", "granulaat"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Kokos", lambda: 0.045, mu: 2, rho: 75, rd_vast: null, keywords: ["kokos"] },
  { categorie: "Isolatie - Natuurlijk", naam: "Stro", lambda: 0.08, mu: 3, rho: 100, rd_vast: null, keywords: ["stro", "baal"] },

  // Folie - Dampremmend
  { categorie: "Folie - Dampremmend", naam: "PE-folie 0.15mm", lambda: null, mu: 50000, rho: null, rd_vast: 0.0, keywords: ["pe", "folie", "dampremmend"] },
  { categorie: "Folie - Dampremmend", naam: "PE-folie 0.20mm", lambda: null, mu: 80000, rho: null, rd_vast: 0.0, keywords: ["pe", "folie", "dampremmend"] },
  { categorie: "Folie - Dampremmend", naam: "PE-folie 0.30mm (AVS)", lambda: null, mu: 100000, rho: null, rd_vast: 0.0, keywords: ["pe", "avs", "dampremmend"] },

  // Folie - Miofol
  { categorie: "Folie - Miofol", naam: "Miofol 125S (dampremmend)", lambda: null, mu: 200000, rho: null, rd_vast: 0.0, keywords: ["miofol", "125s", "dampremmend"] },
  { categorie: "Folie - Miofol", naam: "Miofol 100S (dampremmend)", lambda: null, mu: 90000, rho: null, rd_vast: 0.0, keywords: ["miofol", "100s"] },
  { categorie: "Folie - Miofol", naam: "Miofol 125AV (dampdicht alu)", lambda: null, mu: 7500000, rho: null, rd_vast: 0.0, keywords: ["miofol", "125av", "alu", "dampdicht"] },
  { categorie: "Folie - Miofol", naam: "Miofol 150A (dampdicht gewapend)", lambda: null, mu: 10000000, rho: null, rd_vast: 0.0, keywords: ["miofol", "150a", "alu"] },
  { categorie: "Folie - Miofol", naam: "Miofol 200AK (zelfklevend)", lambda: null, mu: 10000000, rho: null, rd_vast: 0.0, keywords: ["miofol", "200ak", "zelfklevend"] },
  { categorie: "Folie - Miofol", naam: "Miofol Active (variabel)", lambda: null, mu: 1000, rho: null, rd_vast: 0.0, keywords: ["miofol", "active", "variabel", "klimaat"] },
  { categorie: "Folie - Miofol", naam: "Miofol 125G (dampopen gevel)", lambda: null, mu: 1400, rho: null, rd_vast: 0.0, keywords: ["miofol", "125g", "gevel", "dampopen"] },
  { categorie: "Folie - Miofol", naam: "Miofol 170AG (alu gevel)", lambda: null, mu: 500, rho: null, rd_vast: 0.0, keywords: ["miofol", "170ag", "gevel"] },
  { categorie: "Folie - Miofol", naam: "Miofol AVS 4 (vloerfolie)", lambda: null, mu: 500000, rho: null, rd_vast: 0.0, keywords: ["miofol", "avs", "vloer"] },

  // Folie - Pro Clima
  { categorie: "Folie - Pro Clima", naam: "Pro Clima INTELLO (variabel)", lambda: null, mu: 2500, rho: null, rd_vast: 0.0, keywords: ["proclima", "intello", "variabel", "klimaat"] },
  { categorie: "Folie - Pro Clima", naam: "Pro Clima INTELLO PLUS", lambda: null, mu: 2500, rho: null, rd_vast: 0.0, keywords: ["proclima", "intello", "plus"] },
  { categorie: "Folie - Pro Clima", naam: "Pro Clima DB+ (dampremmend)", lambda: null, mu: 10000, rho: null, rd_vast: 0.0, keywords: ["proclima", "db"] },
  { categorie: "Folie - Pro Clima", naam: "Pro Clima DA (damprem)", lambda: null, mu: 10000, rho: null, rd_vast: 0.0, keywords: ["proclima", "da"] },
  { categorie: "Folie - Pro Clima", naam: "Pro Clima SOLITEX MENTO (dampopen)", lambda: null, mu: 125, rho: null, rd_vast: 0.0, keywords: ["proclima", "solitex", "mento", "onderdak"] },
  { categorie: "Folie - Pro Clima", naam: "Pro Clima SOLITEX PLUS (dampopen)", lambda: null, mu: 100, rho: null, rd_vast: 0.0, keywords: ["proclima", "solitex", "plus"] },
  { categorie: "Folie - Pro Clima", naam: "Pro Clima SOLITEX WELDANO (dampopen)", lambda: null, mu: 125, rho: null, rd_vast: 0.0, keywords: ["proclima", "weldano"] },
  { categorie: "Folie - Pro Clima", naam: "Pro Clima SOLITEX FRONTA WA", lambda: null, mu: 80, rho: null, rd_vast: 0.0, keywords: ["proclima", "fronta", "gevel"] },

  // Folie - Overig
  { categorie: "Folie - Overig", naam: "EPDM", lambda: 0.17, mu: 10000, rho: 1200, rd_vast: null, keywords: ["epdm", "rubber", "dakbedekking"] },
  { categorie: "Folie - Overig", naam: "Bitumen APP", lambda: 0.23, mu: 50000, rho: 1100, rd_vast: null, keywords: ["bitumen", "app", "dak"] },
  { categorie: "Folie - Overig", naam: "Bitumen SBS", lambda: 0.23, mu: 25000, rho: 1100, rd_vast: null, keywords: ["bitumen", "sbs", "dak"] },
  { categorie: "Folie - Overig", naam: "PVC dakbedekking", lambda: 0.16, mu: 50000, rho: 1400, rd_vast: null, keywords: ["pvc", "dak"] },
  { categorie: "Folie - Overig", naam: "TPO/FPO dakbedekking", lambda: 0.20, mu: 25000, rho: 900, rd_vast: null, keywords: ["tpo", "fpo", "dak"] },
  { categorie: "Folie - Overig", naam: "Aluminium (pure folie)", lambda: null, mu: 10000000, rho: null, rd_vast: 0.0, keywords: ["alu", "aluminium", "folie"] },

  // Afwerking
  { categorie: "Afwerking", naam: "Tegelwerk keramisch", lambda: 1.3, mu: 200, rho: 2300, rd_vast: null, keywords: ["tegel", "keramisch", "vloer"] },
  { categorie: "Afwerking", naam: "Tegels natuursteen", lambda: 2.0, mu: 1000, rho: 2700, rd_vast: null, keywords: ["tegel", "natuursteen"] },
  { categorie: "Afwerking", naam: "Stucwerk gips", lambda: 0.51, mu: 10, rho: 1300, rd_vast: null, keywords: ["stuc", "gips", "pleister"] },
  { categorie: "Afwerking", naam: "Stucwerk kalk-cement", lambda: 0.87, mu: 20, rho: 1800, rd_vast: null, keywords: ["stuc", "kalk", "cement"] },
  { categorie: "Afwerking", naam: "Spachtelputz", lambda: 0.70, mu: 15, rho: 1400, rd_vast: null, keywords: ["spachtel", "putz", "pleister"] },
  { categorie: "Afwerking", naam: "Sierpleister mineraal", lambda: 0.80, mu: 20, rho: 1600, rd_vast: null, keywords: ["sierpleister", "korrel"] },
  { categorie: "Afwerking", naam: "Acrylaat pleister", lambda: 0.70, mu: 150, rho: 1600, rd_vast: null, keywords: ["acrylaat", "pleister", "kunststof"] },
  { categorie: "Afwerking", naam: "Silicaat pleister", lambda: 0.80, mu: 50, rho: 1700, rd_vast: null, keywords: ["silicaat", "pleister"] },
  { categorie: "Afwerking", naam: "Siliconen pleister", lambda: 0.80, mu: 80, rho: 1700, rd_vast: null, keywords: ["siliconen", "pleister"] },
  { categorie: "Afwerking", naam: "Leemstuc", lambda: 0.70, mu: 8, rho: 1500, rd_vast: null, keywords: ["leem", "stuc"] },
  { categorie: "Afwerking", naam: "Kalkverf", lambda: null, mu: 10, rho: null, rd_vast: 0.0, keywords: ["kalk", "verf"] },
  { categorie: "Afwerking", naam: "Latexverf", lambda: null, mu: 500, rho: null, rd_vast: 0.0, keywords: ["latex", "verf"] },
  { categorie: "Afwerking", naam: "Behang vinyl", lambda: null, mu: 20000, rho: null, rd_vast: 0.0, keywords: ["behang", "vinyl"] },
  { categorie: "Afwerking", naam: "Behang papier", lambda: null, mu: 10, rho: null, rd_vast: 0.0, keywords: ["behang", "papier"] },

  // Vloer
  { categorie: "Vloer", naam: "Parket massief", lambda: 0.17, mu: 50, rho: 700, rd_vast: null, keywords: ["parket", "massief", "vloer"] },
  { categorie: "Vloer", naam: "Laminaat", lambda: 0.17, mu: 20, rho: 850, rd_vast: null, keywords: ["laminaat", "vloer"] },
  { categorie: "Vloer", naam: "PVC vloer", lambda: 0.17, mu: 50000, rho: 1400, rd_vast: null, keywords: ["pvc", "vinyl", "vloer"] },
  { categorie: "Vloer", naam: "Linoleum", lambda: 0.17, mu: 1000, rho: 1200, rd_vast: null, keywords: ["linoleum", "marmoleum", "vloer"] },
  { categorie: "Vloer", naam: "Tapijt synthetisch", lambda: 0.06, mu: 5, rho: 200, rd_vast: null, keywords: ["tapijt", "vloer"] },
  { categorie: "Vloer", naam: "Ondervloer EPS", lambda: 0.035, mu: 40, rho: 20, rd_vast: null, keywords: ["ondervloer", "eps"] },
  { categorie: "Vloer", naam: "Ondervloer PUR", lambda: 0.030, mu: 50, rho: 50, rd_vast: null, keywords: ["ondervloer", "pur"] },
  { categorie: "Vloer", naam: "Ondervloer rubber", lambda: 0.17, mu: 10000, rho: 1000, rd_vast: null, keywords: ["ondervloer", "rubber"] },

  // Metaal
  { categorie: "Metaal", naam: "Aluminium", lambda: 200.0, mu: 10000000, rho: 2700, rd_vast: null, keywords: ["alu", "aluminium"] },
  { categorie: "Metaal", naam: "Staal", lambda: 50.0, mu: 10000000, rho: 7850, rd_vast: null, keywords: ["staal", "steel"] },
  { categorie: "Metaal", naam: "RVS", lambda: 15.0, mu: 10000000, rho: 8000, rd_vast: null, keywords: ["rvs", "roestvaststaal", "inox"] },
  { categorie: "Metaal", naam: "Koper", lambda: 380.0, mu: 10000000, rho: 8900, rd_vast: null, keywords: ["koper", "copper"] },
  { categorie: "Metaal", naam: "Zink", lambda: 110.0, mu: 10000000, rho: 7130, rd_vast: null, keywords: ["zink", "zinc"] },
  { categorie: "Metaal", naam: "Lood", lambda: 35.0, mu: 10000000, rho: 11300, rd_vast: null, keywords: ["lood", "lead"] },

  // Kunststof
  { categorie: "Kunststof", naam: "Polyetheen (PE)", lambda: 0.33, mu: 100000, rho: 950, rd_vast: null, keywords: ["pe", "polyetheen"] },
  { categorie: "Kunststof", naam: "Polypropyleen (PP)", lambda: 0.22, mu: 10000, rho: 910, rd_vast: null, keywords: ["pp", "polypropyleen"] },
  { categorie: "Kunststof", naam: "PVC hard", lambda: 0.17, mu: 50000, rho: 1400, rd_vast: null, keywords: ["pvc", "hard"] },
  { categorie: "Kunststof", naam: "PVC zacht", lambda: 0.14, mu: 100000, rho: 1300, rd_vast: null, keywords: ["pvc", "zacht"] },
  { categorie: "Kunststof", naam: "Polycarbonaat", lambda: 0.20, mu: 5000, rho: 1200, rd_vast: null, keywords: ["pc", "polycarbonaat"] },
  { categorie: "Kunststof", naam: "PMMA (Acrylaat)", lambda: 0.18, mu: 10000, rho: 1180, rd_vast: null, keywords: ["pmma", "acrylaat", "plexiglas"] },
  { categorie: "Kunststof", naam: "Nylon (PA)", lambda: 0.25, mu: 100, rho: 1150, rd_vast: null, keywords: ["nylon", "pa"] },
  { categorie: "Kunststof", naam: "Siliconen", lambda: 0.35, mu: 5000, rho: 1100, rd_vast: null, keywords: ["siliconen"] },

  // Glas
  { categorie: "Glas", naam: "Glas float", lambda: 1.0, mu: 10000000, rho: 2500, rd_vast: null, keywords: ["glas", "float", "ruit"] },
  { categorie: "Glas", naam: "Glasblokken", lambda: 0.8, mu: 10000000, rho: 1400, rd_vast: null, keywords: ["glasblok"] },

  // Overig
  { categorie: "Overig", naam: "Aarde droog", lambda: 0.52, mu: 2, rho: 1500, rd_vast: null, keywords: ["aarde", "grond"] },
  { categorie: "Overig", naam: "Aarde vochtig", lambda: 1.5, mu: 50, rho: 1800, rd_vast: null, keywords: ["aarde", "grond", "vochtig"] },
  { categorie: "Overig", naam: "Zand droog", lambda: 0.33, mu: 2, rho: 1500, rd_vast: null, keywords: ["zand"] },
  { categorie: "Overig", naam: "Grind", lambda: 0.81, mu: 2, rho: 1800, rd_vast: null, keywords: ["grind"] },
];

// ---------- Build indexed database ----------

const usedSlugs = new Set<string>();

export const MATERIALS_DATABASE: Material[] = RAW_MATERIALS.map((raw) => {
  let id = slug(raw.categorie, raw.naam);
  // Garandeer uniekheid
  if (usedSlugs.has(id)) {
    let suffix = 2;
    while (usedSlugs.has(`${id}-${suffix}`)) suffix++;
    id = `${id}-${suffix}`;
  }
  usedSlugs.add(id);

  return {
    id,
    name: raw.naam,
    category: mapCategory(raw.categorie),
    lambda: raw.lambda,
    mu: raw.mu,
    rho: raw.rho,
    rdFixed: raw.rd_vast,
    keywords: raw.keywords,
  };
});

const MATERIAL_INDEX = new Map(MATERIALS_DATABASE.map((m) => [m.id, m]));

// ---------- Visuele categorie-eigenschappen (voor Glaser-diagram) ----------

export interface CategoryVisual {
  /** Vulkleur voor laag-band in diagram. */
  color: string;
  /** SVG pattern-id voor arcering (optioneel). */
  patternId?: string;
}

/** Kleuren en arceringen per materiaalcategorie. */
export const MATERIAL_CATEGORY_VISUALS: Record<MaterialCategory, CategoryVisual> = {
  metselwerk:         { color: "#d4736a", patternId: "hatch-masonry" },
  beton:              { color: "#b0b0b0", patternId: "hatch-concrete" },
  isolatie_mineraal:  { color: "#fde047", patternId: "hatch-insulation" },
  isolatie_kunststof: { color: "#f9a8d4", patternId: "hatch-insulation" },
  isolatie_natuurlijk:{ color: "#86efac", patternId: "hatch-insulation" },
  hout:               { color: "#c68642", patternId: "hatch-wood" },
  spouw:              { color: "#bfdbfe" },
  folie:              { color: "#a5b4fc", patternId: "hatch-foil" },
  afwerking:          { color: "#e7e5e4" },
  plaatmateriaal:     { color: "#fbbf24", patternId: "hatch-wood" },
  mortel:             { color: "#d6d3d1" },
  natuursteen:        { color: "#9ca3af", patternId: "hatch-concrete" },
  vloer:              { color: "#d1d5db" },
  metaal:             { color: "#94a3b8", patternId: "hatch-metal" },
  kunststof:          { color: "#c084fc" },
  glas:               { color: "#67e8f9" },
  overig:             { color: "#e5e7eb" },
};

// ---------- Query functies ----------

/** Haal materiaal op basis van id. */
export function getMaterialById(id: string): Material | undefined {
  return MATERIAL_INDEX.get(id);
}

/** Alle materialen in een categorie. */
export function getMaterialsByCategory(category: MaterialCategory): Material[] {
  return MATERIALS_DATABASE.filter((m) => m.category === category);
}

/** Zoek materialen op naam + keywords (case-insensitive substring match). */
export function searchMaterials(query: string): Material[] {
  if (!query.trim()) return MATERIALS_DATABASE;
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
  return MATERIALS_DATABASE.filter((m) => {
    const haystack = [m.name, ...m.keywords].join(" ").toLowerCase();
    return terms.every((term) => haystack.includes(term));
  });
}
