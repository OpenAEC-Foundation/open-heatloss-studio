/**
 * ISSO 53 ventilatie-helpers voor de vertrekkenrij (`VentilationRow`).
 *
 * Twee referentie-minimums worden read-only naast het editbare q_v-veld
 * getoond:
 *   - **BBL-minimum**: uniforme verblijfsgebied-eis 0,9 dm³/s per m²
 *     vloeroppervlak ({@link isso53BblMinimumDm3s}).
 *   - **Bezettings-minimum**: `personen × per-persoon-tarief` uit ISSO 53
 *     tabel 4.10 ({@link isso53BezettingMinimumDm3s}).
 *
 * De per-persoon-tarieven en de (gebruiksfunctie × ruimtetype)-mapping
 * spiegelen 1:1 `crates/isso53-core/src/tables/ventilation_requirements.rs`
 * (de `VENTILATION_REQUIREMENTS_4_10`-data + de `requirement`-match incl. de
 * `(_, Vergaderruimte)`-catch-all en `_ => None`).
 *
 * De rekenkern blijft leidend op de uiteindelijk in `room.ventilation_rate`
 * doorgegeven waarde; deze helpers leveren puur invul-/referentiewaarden.
 */
import type {
  Isso53GebruiksFunctie,
  Isso53RuimteType,
} from "../types/projectV2";

/** ISSO 53 BBL-minimum ventilatie (verblijfsgebied): 0,9 dm³/s per m². */
export function isso53BblMinimumDm3s(floorAreaM2: number): number {
  return 0.9 * floorAreaM2;
}

/**
 * Per-persoon ventilatie-tarief (nieuwbouw) uit ISSO 53 tabel 4.10,
 * gekeyed op de letterlijke ruimte-omschrijving. Spiegel van de
 * `nieuwbouw_dm3_s_pp`-kolom in de Rust-const `VENTILATION_REQUIREMENTS_4_10`
 * (dm³/s per persoon).
 */
const RATE_PER_PERSON_NIEUWBOUW: Readonly<Record<string, number>> = {
  // --- Bijeenkomstfunctie ---
  Eetruimte: 4.0,
  Bedrijfsrestaurant: 4.0,
  Kantine: 4.0,
  // --- Kantoorfunctie ---
  Kantoorruimte: 6.5,
  Receptie: 6.5,
  Vergaderruimte: 6.5,
  // --- Gezondheidszorgfunctie ---
  Patiëntenkamer: 12.0,
  Operatiekamer: 12.0,
  Onderzoekruimte: 6.5,
  // --- Logiesfunctie ---
  Hotelkamer: 12.0,
  // --- Onderwijsfunctie ---
  Lesruimte: 8.5,
  Collegezaal: 8.5,
  Werkplaats: 6.5,
  Bureauruimte: 6.5,
  // --- Sportfunctie ---
  Sportzaal: 6.5,
  // --- Winkelfunctie ---
  Verkoopruimte: 4.0,
  Supermarkt: 4.0,
  Warenhuis: 4.0,
  // --- Celfunctie ---
  "Cel voor dag- en nachtverblijf": 12.0,
  // --- Industriefunctie ---
  "Industrie algemeen": 6.5,
};

/**
 * Mapt een (gebruiksfunctie, ruimtetype)-combinatie naar de tabel 4.10-
 * omschrijving. Exacte spiegel van Rust `fn requirement` — inclusief de
 * `(_, Vergaderruimte)`-catch-all en `_ => None`. Retourneert `null` voor
 * combinaties zonder eis (berg-/technische/verkeers-/sanitaire ruimten).
 */
function requirementDescription(
  functie: Isso53GebruiksFunctie,
  ruimte: Isso53RuimteType,
): string | null {
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

  // `(_, Vergaderruimte)`-catch-all: in de Rust match wint deze arm voor élke
  // gebruiksfunctie (geen functie-specifieke Vergaderruimte-arm bestaat),
  // dus we vangen het hier af na de functie-specifieke arms.
  if (description === null && ruimte === "vergaderruimte") {
    description = "Vergaderruimte";
  }

  return description;
}

/**
 * Per-persoon ventilatie-tarief (nieuwbouw) voor een (functie × type)-combi,
 * in dm³/s per persoon. `null` als de combinatie geen eis in tabel 4.10 heeft.
 */
export function isso53RatePerPersonDm3s(
  functie: Isso53GebruiksFunctie,
  ruimte: Isso53RuimteType,
): number | null {
  const description = requirementDescription(functie, ruimte);
  if (description === null) {
    return null;
  }
  return RATE_PER_PERSON_NIEUWBOUW[description] ?? null;
}

/**
 * Bezettings-minimum (nieuwbouw) in dm³/s: `personen × per-persoon-tarief`
 * uit ISSO 53 tabel 4.10.
 *
 * Retourneert `null` als:
 *   - personen niet is ingevuld (`null`/`undefined`) of geen eindig getal is, of
 *   - de (functie × type)-combinatie geen eis in tabel 4.10 heeft.
 */
export function isso53BezettingMinimumDm3s(
  functie: Isso53GebruiksFunctie,
  ruimte: Isso53RuimteType,
  personen: number | null | undefined,
): number | null {
  if (
    personen === null ||
    personen === undefined ||
    !Number.isFinite(personen)
  ) {
    return null;
  }
  const rate = isso53RatePerPersonDm3s(functie, ruimte);
  if (rate === null) {
    return null;
  }
  return personen * rate;
}
