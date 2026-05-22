/**
 * Veilige reken-evaluator voor numerieke invoervelden.
 *
 * Doel: een gebruiker kan in een numeriek tabelveld een rekenexpressie typen,
 * Excel-stijl met een optioneel leidend `=`, bv. `=1,5*2,6` → `3,9`. Dekt
 * `+ - * /` en haakjes met correcte operator-prioriteit, en accepteert zowel
 * de komma als de punt als decimaalteken (NL-toetsenbord vs. numpad).
 *
 * Bewust GEEN `eval()` / `Function`-constructor: dit is een zuivere
 * recursive-descent parser zonder side-effects. Onveilige of onvolledige
 * invoer levert `null`, nooit een exception naar de caller.
 *
 * Grammatica (EBNF):
 *   expr   = term   { ("+" | "-") term } ;
 *   term   = unary  { ("*" | "/") unary } ;
 *   unary  = ["+" | "-"] unary | factor ;
 *   factor = number | "(" expr ")" ;
 */

type TokenType = "number" | "+" | "-" | "*" | "/" | "(" | ")";

interface Token {
  type: TokenType;
  value: number;
}

/**
 * Splits de ruwe invoer in tokens. Retourneert `null` zodra er een teken
 * voorkomt dat niet in de grammatica past.
 */
function tokenize(input: string): Token[] | null {
  const tokens: Token[] = [];
  let i = 0;

  while (i < input.length) {
    const ch = input.charAt(i);

    // Whitespace overslaan.
    if (ch === " " || ch === "\t" || ch === "\n" || ch === "\r") {
      i += 1;
      continue;
    }

    if (ch === "+" || ch === "-" || ch === "*" || ch === "/" || ch === "(" || ch === ")") {
      tokens.push({ type: ch, value: 0 });
      i += 1;
      continue;
    }

    // Getal: cijfers met komma of punt als decimaalteken.
    if ((ch >= "0" && ch <= "9") || ch === "," || ch === ".") {
      let numStr = "";
      let seenSeparator = false;
      while (i < input.length) {
        const c = input.charAt(i);
        if (c >= "0" && c <= "9") {
          numStr += c;
          i += 1;
        } else if (c === "," || c === ".") {
          // Maximaal één decimaalteken per getal.
          if (seenSeparator) return null;
          seenSeparator = true;
          numStr += ".";
          i += 1;
        } else {
          break;
        }
      }
      // Een losse separator zonder cijfers is geen geldig getal.
      if (numStr === "" || numStr === ".") return null;
      const value = Number(numStr);
      if (!Number.isFinite(value)) return null;
      tokens.push({ type: "number", value });
      continue;
    }

    // Onbekend teken → ongeldige invoer.
    return null;
  }

  return tokens;
}

/**
 * Recursive-descent parser. Werkt op een cursor over de tokenlijst.
 * Gooit bij syntaxfouten of deling door nul; de publieke functie vangt dit af.
 */
class Parser {
  private pos = 0;

  constructor(private readonly tokens: Token[]) {}

  parse(): number {
    const result = this.parseExpr();
    if (this.pos !== this.tokens.length) {
      throw new Error("Onverwachte tokens na einde expressie");
    }
    return result;
  }

  private peek(): Token | undefined {
    return this.tokens[this.pos];
  }

  private parseExpr(): number {
    let value = this.parseTerm();
    while (true) {
      const tok = this.peek();
      if (tok?.type === "+") {
        this.pos += 1;
        value += this.parseTerm();
      } else if (tok?.type === "-") {
        this.pos += 1;
        value -= this.parseTerm();
      } else {
        break;
      }
    }
    return value;
  }

  private parseTerm(): number {
    let value = this.parseUnary();
    while (true) {
      const tok = this.peek();
      if (tok?.type === "*") {
        this.pos += 1;
        value *= this.parseUnary();
      } else if (tok?.type === "/") {
        this.pos += 1;
        const divisor = this.parseUnary();
        if (divisor === 0) {
          throw new Error("Deling door nul");
        }
        value /= divisor;
      } else {
        break;
      }
    }
    return value;
  }

  private parseUnary(): number {
    const tok = this.peek();
    if (tok?.type === "-") {
      this.pos += 1;
      return -this.parseUnary();
    }
    if (tok?.type === "+") {
      this.pos += 1;
      return this.parseUnary();
    }
    return this.parseFactor();
  }

  private parseFactor(): number {
    const tok = this.peek();
    if (tok === undefined) {
      throw new Error("Onverwacht einde van expressie");
    }
    if (tok.type === "number") {
      this.pos += 1;
      return tok.value;
    }
    if (tok.type === "(") {
      this.pos += 1;
      const value = this.parseExpr();
      const closing = this.peek();
      if (closing?.type !== ")") {
        throw new Error("Ontbrekend sluithaakje");
      }
      this.pos += 1;
      return value;
    }
    throw new Error("Onverwacht token");
  }
}

/**
 * Evalueert een numerieke invoerstring naar een getal.
 *
 * @param raw - De ruwe invoer. Mag een kaal getal zijn (`12,5`), een
 *   rekenexpressie (`1,5*2,6`) of een Excel-stijl variant met leidend `=`.
 * @returns Het berekende getal, of `null` bij lege invoer, een ongeldige of
 *   onvolledige expressie, deling door nul, of een niet-eindig resultaat.
 */
export function evaluateNumericInput(raw: string): number | null {
  if (typeof raw !== "string") return null;

  let trimmed = raw.trim();
  if (trimmed === "") return null;

  // DoS-guard: de recursive-descent parser heeft geen dieptelimiet, dus
  // pathologische invoer (duizenden geneste haakjes, lange unaire-min-keten)
  // zou de JS-stack kunnen laten overlopen. Een numerieke expressie in een
  // UI-cel wordt nooit zo lang — kap ruim af op 100 tekens.
  if (trimmed.length > 100) return null;

  // Optioneel leidend `=` (Excel-stijl) strippen.
  if (trimmed.startsWith("=")) {
    trimmed = trimmed.slice(1).trim();
    if (trimmed === "") return null;
  }

  const tokens = tokenize(trimmed);
  if (tokens === null || tokens.length === 0) return null;

  try {
    const result = new Parser(tokens).parse();
    if (!Number.isFinite(result)) return null;
    return result;
  } catch {
    return null;
  }
}
