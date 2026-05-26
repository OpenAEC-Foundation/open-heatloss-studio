import {
  type ChangeEvent,
  type FocusEvent,
  type InputHTMLAttributes,
  type KeyboardEvent,
  forwardRef,
  useEffect,
  useRef,
  useState,
} from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  unit?: string;
  error?: string;
}

/**
 * Standaard tekst/numerieke invoerveld met optioneel label en unit-suffix.
 *
 * Voor `type="number"` schakelt deze component intern over op een
 * `<input type="text" inputMode="decimal">` met draft-state. Reden:
 * native `<input type="number">` met nl-NL locale + Chrome blokkeert
 * tussentijdse tekens als `.` of `,`. Bij een controlled component met
 * `value={number}` levert dat een verspringende cursor op — concreet
 * symptoom: "50.2" tikken werd "5.02".
 *
 * Het werkende patroon spiegelt `EditableCell.tsx`:
 *   - draft-state houdt de tekst-representatie vast tijdens typen
 *   - commit gebeurt op blur (en Enter), niet op elk keystroke
 *   - beide decimale separatoren (`,` en `.`) zijn geldig
 *   - lege string blijft leeg zodat null-detection bij callers werkt
 *
 * De externe API blijft identiek: callers krijgen `onChange(e)` met
 * `e.target.value` als string (bij commit) en de oorspronkelijke
 * `value`-prop blijft een number.
 */

/** Vervang de eerste komma door een punt zodat `Number(...)` deze parst. */
function normalizeDecimal(raw: string): string {
  return raw.replace(",", ".");
}

/** Tussentijds-validate: accepteer alleen tekens die in een decimaal getal
 *  passen (incl. een leidend minteken). Voorkomt dat plakwerk of een typo
 *  zoals letters in het veld terechtkomt. */
function isPartialDecimal(raw: string): boolean {
  if (raw === "") return true;
  // Optioneel teken, cijfers, optioneel één separator gevolgd door cijfers.
  // Sta ook "-", ".", "," los toe als prefix-state tijdens typen.
  return /^-?(\d+([.,]\d*)?|[.,]\d*|)$/.test(raw);
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      label,
      unit,
      error,
      id,
      className = "",
      type,
      value,
      onChange,
      onBlur,
      onKeyDown,
      ...props
    },
    ref,
  ) => {
    // Schaal de right-padding mee met unit-lengte zodat lange eenheden
    // zoals "dm³/(s·m²)" niet door de ingevulde waarde heen vallen.
    const unitPad = unit ? (unit.length <= 4 ? "pr-12" : unit.length <= 8 ? "pr-20" : "pr-28") : "";

    const isNumeric = type === "number";

    // ────────────────────────────────────────────── draft-state voor numerics
    // De draft is alleen relevant terwijl het veld focus heeft. Zo blijft het
    // veld synchroon met externe state-wijzigingen wanneer de gebruiker
    // ergens anders is.
    const [draft, setDraft] = useState<string>(() => (value == null ? "" : String(value)));
    const [editing, setEditing] = useState(false);
    const lastCommittedRef = useRef<string>(draft);

    useEffect(() => {
      if (!editing && isNumeric) {
        const incoming = value == null || value === "" ? "" : String(value);
        setDraft(incoming);
        lastCommittedRef.current = incoming;
      }
    }, [value, editing, isNumeric]);

    if (!isNumeric) {
      return (
        <div className="flex flex-col gap-1">
          {label && (
            <label htmlFor={id} className="text-xs font-medium text-on-surface-secondary">
              {label}
            </label>
          )}
          <div className="relative">
            <input
              ref={ref}
              id={id}
              type={type}
              value={value as InputHTMLAttributes<HTMLInputElement>["value"]}
              onChange={onChange}
              onBlur={onBlur}
              onKeyDown={onKeyDown}
              className={`w-full rounded-md border-[1.5px] bg-[var(--oaec-bg-input)] px-3 py-2 text-sm text-on-surface
                transition-colors placeholder:text-on-surface-muted
                focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20
                disabled:opacity-50 disabled:cursor-not-allowed
                ${unitPad}
                ${error ? "border-red-400" : "border-[var(--oaec-border)]"}
                ${className}`}
              {...props}
            />
            {unit && <span className="input-unit">{unit}</span>}
          </div>
          {error && <p className="text-xs text-red-400">{error}</p>}
        </div>
      );
    }

    // ─────────────────────────────────────────────────────────── numeric path
    const commit = (rawDraft: string) => {
      const normalized = normalizeDecimal(rawDraft.trim());
      if (normalized === lastCommittedRef.current) return;
      // Onveranderd doorgeven via een synthetisch event-shape; React's
      // ChangeEvent is een SyntheticEvent en heeft `target.value` als
      // string, wat alle bestaande callers verwerken via `numVal()`.
      if (onChange) {
        const fake = {
          target: { value: normalized },
          currentTarget: { value: normalized },
        } as unknown as ChangeEvent<HTMLInputElement>;
        onChange(fake);
      }
      lastCommittedRef.current = normalized;
    };

    const handleDraftChange = (e: ChangeEvent<HTMLInputElement>) => {
      const next = e.target.value;
      // Negeer ongeldige tussenstanden (bv. plakken van letters). Hiermee
      // blijft de cursor stabiel: de draft verandert alleen bij geldige
      // tekens, wat de "schiet naar decimalen"-bug uitsluit.
      if (!isPartialDecimal(next)) return;
      setDraft(next);
    };

    const handleFocus = () => {
      setEditing(true);
    };

    const handleBlur = (e: FocusEvent<HTMLInputElement>) => {
      setEditing(false);
      commit(draft);
      onBlur?.(e);
    };

    const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") {
        commit(draft);
        // Forceer dat onChange direct verwerkt wordt voor de bestaande input.
        (e.currentTarget as HTMLInputElement).blur();
      }
      onKeyDown?.(e);
    };

    return (
      <div className="flex flex-col gap-1">
        {label && (
          <label htmlFor={id} className="text-xs font-medium text-on-surface-secondary">
            {label}
          </label>
        )}
        <div className="relative">
          <input
            ref={ref}
            id={id}
            type="text"
            inputMode="decimal"
            // Tijdens edit toont draft, anders de externe value als string.
            value={editing ? draft : value == null || value === "" ? "" : String(value)}
            onFocus={handleFocus}
            onChange={handleDraftChange}
            onBlur={handleBlur}
            onKeyDown={handleKeyDown}
            className={`w-full rounded-md border-[1.5px] bg-[var(--oaec-bg-input)] px-3 py-2 text-sm text-on-surface
              transition-colors placeholder:text-on-surface-muted
              focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20
              disabled:opacity-50 disabled:cursor-not-allowed
              ${unitPad}
              ${error ? "border-red-400" : "border-[var(--oaec-border)]"}
              font-mono text-right
              ${className}`}
            {...props}
          />
          {unit && <span className="input-unit">{unit}</span>}
        </div>
        {error && <p className="text-xs text-red-400">{error}</p>}
      </div>
    );
  },
);

Input.displayName = "Input";
