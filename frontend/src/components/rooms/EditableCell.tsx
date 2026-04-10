import { useCallback, useEffect, useRef, useState } from "react";

interface EditableCellProps {
  value: string | number;
  onChange: (value: string) => void;
  type?: "text" | "number";
  unit?: string;
  placeholder?: string;
  className?: string;
  /**
   * Optional display-only formatter. Applied to the non-editing rendering
   * of numeric values (bv. oppervlakte-afronding op 2 decimalen). Wordt
   * niet gebruikt tijdens editing zodat de gebruiker de volledige precisie
   * ziet en kan bewerken.
   */
  displayFormatter?: (value: number) => string;
}

export function EditableCell({
  value,
  onChange,
  type = "text",
  unit,
  placeholder,
  className = "",
  displayFormatter,
}: EditableCellProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(String(value));
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [editing]);

  useEffect(() => {
    if (!editing) {
      setDraft(String(value));
    }
  }, [value, editing]);

  const commit = useCallback(() => {
    setEditing(false);
    if (draft !== String(value)) {
      onChange(draft);
    }
  }, [draft, value, onChange]);

  const cancel = useCallback(() => {
    setEditing(false);
    setDraft(String(value));
  }, [value]);

  if (editing) {
    return (
      <input
        ref={inputRef}
        type={type}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          if (e.key === "Escape") cancel();
        }}
        step={type === "number" ? "any" : undefined}
        className={`w-full rounded border border-primary bg-[var(--oaec-bg-input)] px-1.5 py-0.5 text-sm text-on-surface
          outline-none focus:ring-1 focus:ring-primary ${className}`}
      />
    );
  }

  const rawDisplay = value === 0 && type === "number" ? "0" : String(value);
  const isEmpty = rawDisplay === "" || rawDisplay === "0";
  const displayValue =
    !isEmpty &&
    displayFormatter !== undefined &&
    type === "number" &&
    typeof value === "number" &&
    Number.isFinite(value)
      ? displayFormatter(value)
      : rawDisplay;

  return (
    <span
      onClick={() => setEditing(true)}
      className={`inline-block w-full cursor-text rounded px-1.5 py-0.5 text-sm
        hover:bg-[var(--oaec-hover)] ${isEmpty ? "text-on-surface-muted" : "text-on-surface"} ${className}`}
    >
      {isEmpty ? (placeholder ?? "\u2014") : displayValue}
      {unit && !isEmpty && (
        <span className="ml-0.5 text-xs text-on-surface-muted">{unit}</span>
      )}
    </span>
  );
}
