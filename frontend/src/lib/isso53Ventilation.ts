/**
 * TypeScript-port van ISSO 53 tabel 4.10 — minimale ventilatie-eisen
 * (Bouwbesluit).
 *
 * Spiegelt 1:1 `crates/isso53-core/src/tables/ventilation_requirements.rs`:
 * de `VENTILATION_REQUIREMENTS_4_10`-data én de `requirement(functie, ruimte)`
 * match-arms (incl. de `(_, Vergaderruimte)`-catch-all en `_ => None`).
 *
 * Gebruikt voor de read-only referentie-minimums in de ISSO 53-vertrekkenrij.
 * De rekenkern blijft leidend op het handmatig "Vastgesteld"-veld; dit is
 * puur invul-hulp.
 *
 * Eenheden: `nieuwbouwDm3sPp` en `bestaandDm3sPp` in dm³/(s·persoon),
 * `personenPerM2` in personen/m². Alleen de nieuwbouw-waarde wordt door de
 * UI-helpers gebruikt.
 */
import type {
  Isso53GebruiksFunctie,
  Isso53RuimteType,
} from "../types/projectV2";

/** Eén regel uit ventilatie-eisentabel 4.10 (ISSO 53, PDF p.48-50). */
export interface VentilationRequirement {
  /** Beschrijving van de ruimte zoals in tabel 4.10. */
  description: string;
  /** Nieuwbouw-eis in dm³/s per persoon. `null` = geen eis ("-"). */
  nieuwbouwDm3sPp: number | null;
  /** Richtwaarde bezetting in personen/m². `null` = n.v.t. */
  personenPerM2: number | null;
  /** Eis bestaande bouw in dm³/s per persoon. `null` = geen eis ("-"). */
  bestaandDm3sPp: number | null;
}

/**
 * Tabel 4.10 — minimaal vereiste ventilatie-luchtvolumestromen.
 * ISSO 53 (PDF p.48-50). Exacte spiegel van de Rust-const
 * `VENTILATION_REQUIREMENTS_4_10`.
 */
export const VENTILATION_REQUIREMENTS_4_10: readonly VentilationRequirement[] = [
  // --- Bijeenkomstfunctie ---
  { description: "Eetruimte", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Bar", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Bedrijfsrestaurant", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Kantine", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Toeschouwersruimte", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Bibliotheek (bijeenkomst)", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Museum", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Bioscoop", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Concertzaal", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Schouwburg/theater", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Casino", nieuwbouwDm3sPp: 4.0, personenPerM2: 0.125, bestaandDm3sPp: 2.12 },
  { description: "Vergaderruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.05, bestaandDm3sPp: 3.44 },
  // --- Kantoorfunctie ---
  { description: "Kantoorruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.05, bestaandDm3sPp: 3.44 },
  { description: "Receptie", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.05, bestaandDm3sPp: 3.44 },
  // --- Celfunctie ---
  { description: "Cel niet voor dag- en nachtverblijf", nieuwbouwDm3sPp: 12.0, personenPerM2: 0.05, bestaandDm3sPp: 6.4 },
  { description: "Cel voor dag- en nachtverblijf", nieuwbouwDm3sPp: 12.0, personenPerM2: 0.05, bestaandDm3sPp: 6.4 },
  { description: "Andere ruimte (cel)", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  // --- Gezondheidszorgfunctie ---
  { description: "Patiëntenkamer", nieuwbouwDm3sPp: 12.0, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  { description: "Ontwaakkamer", nieuwbouwDm3sPp: 12.0, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  { description: "Intensive care", nieuwbouwDm3sPp: 12.0, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  { description: "Operatiekamer", nieuwbouwDm3sPp: 12.0, personenPerM2: 0.05, bestaandDm3sPp: 3.44 },
  { description: "Onderzoekruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.05, bestaandDm3sPp: 3.44 },
  { description: "Fysiotherapie", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.05, bestaandDm3sPp: 3.44 },
  { description: "Sectieruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  // --- Logiesfunctie ---
  { description: "Hotelkamer", nieuwbouwDm3sPp: 12.0, personenPerM2: 0.05, bestaandDm3sPp: 6.4 },
  // --- Onderwijsfunctie ---
  { description: "Lesruimte", nieuwbouwDm3sPp: 8.5, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  { description: "Collegezaal", nieuwbouwDm3sPp: 8.5, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  { description: "Werkplaats", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  { description: "Bureauruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.05, bestaandDm3sPp: 3.44 },
  { description: "Gymzaal", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  { description: "Aula", nieuwbouwDm3sPp: 6.5, personenPerM2: 0.125, bestaandDm3sPp: 3.44 },
  // --- Sportfunctie ---
  { description: "Sportzaal", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  { description: "Bowlingruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  { description: "IJsvloerspeelruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  { description: "Zwembad", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  // --- Industriefunctie ---
  { description: "Industrie algemeen", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  { description: "Verfspuitinrichting", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  { description: "Accuruimte", nieuwbouwDm3sPp: 6.5, personenPerM2: null, bestaandDm3sPp: 3.44 },
  // --- Winkelfunctie ---
  { description: "Apotheek", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Beautyshop", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Bibliotheek (winkel)", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Bloemist", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Kapper", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Postkantoor", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Supermarkt", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Warenhuis", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Slagerij", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Verkoopruimte", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
  { description: "Wasserette", nieuwbouwDm3sPp: 4.0, personenPerM2: null, bestaandDm3sPp: 2.12 },
];

/**
 * Zoekt een ventilatie-eis op via de letterlijke ruimte-omschrijving.
 * Spiegel van Rust `requirement_by_description` (case-insensitive match).
 */
export function requirementByDescription(
  description: string,
): VentilationRequirement | null {
  const lower = description.toLowerCase();
  return (
    VENTILATION_REQUIREMENTS_4_10.find(
      (r) => r.description.toLowerCase() === lower,
    ) ?? null
  );
}

/**
 * Mapt een (gebruiksfunctie, ruimtetype)-combinatie naar de tabel 4.10-regel.
 * Exacte spiegel van Rust `fn requirement` — inclusief de
 * `(_, Vergaderruimte)`-catch-all en `_ => None`. Retourneert `null` voor
 * combinaties zonder eis (berg-/technische/verkeers-/sanitaire ruimten).
 */
export function requirement(
  functie: Isso53GebruiksFunctie,
  ruimte: Isso53RuimteType,
): VentilationRequirement | null {
  // Vergaderruimte is een catch-all over alle gebruiksfuncties (vóór de
  // functie-specifieke arms uitgesplitst, net als in de Rust match-volgorde:
  // de `(_, Vergaderruimte)`-arm staat tussen Kantoor en Onderwijs in en wint
  // van geen enkele functie-specifieke Vergaderruimte-arm — die bestaan niet).
  let description: string | null = null;

  switch (functie) {
    case "kantoor":
      if (
        ruimte === "kantoorruimte" ||
        ruimte === "verblijfsruimte" ||
        ruimte === "verblijfsgebied"
      ) {
        description = "Kantoorruimte";
      } else if (ruimte === "receptie") {
        description = "Receptie";
      }
      break;
    case "onderwijs":
      if (
        ruimte === "lesruimte" ||
        ruimte === "verblijfsruimte" ||
        ruimte === "verblijfsgebied"
      ) {
        description = "Lesruimte";
      } else if (ruimte === "collegezaal") {
        description = "Collegezaal";
      } else if (ruimte === "werkplaats") {
        description = "Werkplaats";
      } else if (ruimte === "bureauruimte") {
        description = "Bureauruimte";
      }
      break;
    case "gezondheidszorg":
      if (
        ruimte === "patientenkamer" ||
        ruimte === "verblijfsruimte" ||
        ruimte === "verblijfsgebied"
      ) {
        description = "Patiëntenkamer";
      } else if (ruimte === "operatiekamer") {
        description = "Operatiekamer";
      } else if (ruimte === "onderzoekruimte") {
        description = "Onderzoekruimte";
      }
      break;
    case "bijeenkomst":
      if (ruimte === "eetruimte") {
        description = "Eetruimte";
      } else if (ruimte === "restaurant") {
        description = "Bedrijfsrestaurant";
      } else if (ruimte === "kantine") {
        description = "Kantine";
      }
      break;
    case "logies":
      if (
        ruimte === "hotelkamer" ||
        ruimte === "verblijfsruimte" ||
        ruimte === "verblijfsgebied"
      ) {
        description = "Hotelkamer";
      }
      break;
    case "sport":
      if (
        ruimte === "sportzaal" ||
        ruimte === "verblijfsruimte" ||
        ruimte === "verblijfsgebied"
      ) {
        description = "Sportzaal";
      }
      break;
    case "winkel":
      if (
        ruimte === "verkoopruimte" ||
        ruimte === "verblijfsruimte" ||
        ruimte === "verblijfsgebied"
      ) {
        description = "Verkoopruimte";
      } else if (ruimte === "supermarkt") {
        description = "Supermarkt";
      } else if (ruimte === "warenhuis") {
        description = "Warenhuis";
      }
      break;
    case "cel":
      if (ruimte === "verblijfsruimte" || ruimte === "verblijfsgebied") {
        description = "Cel voor dag- en nachtverblijf";
      }
      break;
    case "industrie":
      if (
        ruimte === "werkplaats" ||
        ruimte === "verblijfsruimte" ||
        ruimte === "verblijfsgebied"
      ) {
        description = "Industrie algemeen";
      }
      break;
  }

  // `(_, Vergaderruimte)`-catch-all: in de Rust match staat deze arm vóór de
  // functie-specifieke arms van álle andere functies dan Kantoor, dus elke
  // (functie, Vergaderruimte)-combi valt erop terug. Geen enkele functie-arm
  // hierboven matcht `vergaderruimte`, dus we vangen het hier af.
  if (description === null && ruimte === "vergaderruimte") {
    description = "Vergaderruimte";
  }

  return description === null ? null : requirementByDescription(description);
}

/** Rondt af op 1 decimaal (zelfde presentatie als het Vastgesteld-veld). */
function round1(value: number): number {
  return Math.round(value * 10) / 10;
}

/**
 * Minimale BBL-eis op basis van oppervlakte (nieuwbouw), in dm³/s:
 * `floorAreaM2 × personenPerM2 × nieuwbouwDm3sPp`.
 *
 * Retourneert `null` als de (functie × type)-combi geen eis in tabel 4.10
 * heeft, of als de regel geen bezettingsdichtheid (`personenPerM2`) kent
 * (oppervlakte-eis dan niet bepaalbaar).
 */
export function bblMinimumDm3s(
  functie: Isso53GebruiksFunctie,
  ruimte: Isso53RuimteType,
  floorAreaM2: number,
): number | null {
  const req = requirement(functie, ruimte);
  if (req === null || req.nieuwbouwDm3sPp === null || req.personenPerM2 === null) {
    return null;
  }
  return round1(floorAreaM2 * req.personenPerM2 * req.nieuwbouwDm3sPp);
}

/**
 * Minimale bezettings-eis op basis van het ingevoerde aantal personen
 * (nieuwbouw), in dm³/s: `personen × nieuwbouwDm3sPp`.
 *
 * Retourneert `null` als personen niet is ingevuld (`null`/`undefined`) of
 * als de (functie × type)-combi geen eis in tabel 4.10 heeft.
 */
export function bezettingMinimumDm3s(
  functie: Isso53GebruiksFunctie,
  ruimte: Isso53RuimteType,
  personen: number | null | undefined,
): number | null {
  if (personen === null || personen === undefined || !Number.isFinite(personen)) {
    return null;
  }
  const req = requirement(functie, ruimte);
  if (req === null || req.nieuwbouwDm3sPp === null) {
    return null;
  }
  return round1(personen * req.nieuwbouwDm3sPp);
}
