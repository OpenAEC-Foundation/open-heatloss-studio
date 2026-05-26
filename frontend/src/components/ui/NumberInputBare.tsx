/**
 * Naakte numerieke invoer zonder label/unit-wrapper — bedoeld voor plekken
 * waar `<Input>` te zwaar is (tabel-rijen, compacte forms).
 *
 * Hetzelfde decimaal-commit-patroon als `Input` voor `type="number"` en
 * `EditableCell`: text-input + inputMode=decimal + draft-state + commit op
 * blur/Enter, accepteert zowel `,` als `.` als decimaalseparator.
 *
 * Voorkomt de "5.02"-bug van native `<input type="number">` op nl-NL Chrome
 * (waar `.` mid-typing tot een lege intermediate value leidt en de cursor
 * naar de decimaalplek springt).
 */
import {
  type FocusEvent,
  type InputHTMLAttributes,
  type KeyboardEvent,
  useEffect,
  useRef,
  useState,
} from "react";

interface NumberInputBareProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "type"> {
  value: number | string | null | undefined;
  /** Wordt aangeroepen op blur/Enter met de genormaliseerde string-waarde
   *  ("." als decimaalseparator). Empty string betekent leeg veld. */
  onCommit: (value: string) => void;
}

function normalizeDecimal(raw: string): string {
  return raw.replace(",", ".");
}

function isPartialDecimal(raw: string): boolean {
  if (raw === "") return true;
  return /^-?(\d+([.,]\d*)?|[.,]\d*|)$/.test(raw);
}

export function NumberInputBare({
  value,
  onCommit,
  onBlur,
  onKeyDown,
  ...rest
}: NumberInputBareProps) {
  const initial = value == null || value === "" ? "" : String(value);
  const [draft, setDraft] = useState<string>(initial);
  const [editing, setEditing] = useState(false);
  const lastCommittedRef = useRef<string>(initial);

  useEffect(() => {
    if (!editing) {
      const incoming = value == null || value === "" ? "" : String(value);
      setDraft(incoming);
      lastCommittedRef.current = incoming;
    }
  }, [value, editing]);

  const commit = (rawDraft: string) => {
    const normalized = normalizeDecimal(rawDraft.trim());
    if (normalized === lastCommittedRef.current) return;
    lastCommittedRef.current = normalized;
    onCommit(normalized);
  };

  return (
    <input
      {...rest}
      type="text"
      inputMode="decimal"
      value={editing ? draft : initial}
      onFocus={() => setEditing(true)}
      onChange={(e) => {
        if (!isPartialDecimal(e.target.value)) return;
        setDraft(e.target.value);
      }}
      onBlur={(e: FocusEvent<HTMLInputElement>) => {
        setEditing(false);
        commit(draft);
        onBlur?.(e);
      }}
      onKeyDown={(e: KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter") {
          commit(draft);
          (e.currentTarget as HTMLInputElement).blur();
        }
        onKeyDown?.(e);
      }}
    />
  );
}
