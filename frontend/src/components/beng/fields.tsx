/**
 * Gedeelde invoer-primitieven voor de BENG-tabs (installaties + geometrie).
 *
 * Uit `pages/Beng.tsx` gelicht zodat de F6 gevel-geometrie-editor exact dezelfde
 * styling en gedrag hergebruikt (één bron van waarheid voor het BENG-formulier).
 * De markup is byte-identiek aan de oorspronkelijke lokale definities.
 */
import type { ReactNode } from "react";

/** Basis-styling voor tekst/number/select-invoer (spiegelt TojuliFull). */
export const INPUT_CLASS =
  "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-on-surface focus:outline-none focus:ring-1 focus:border-primary focus:ring-primary";

/** Label + optionele hint rond een willekeurig invoer-element. */
export function LabeledField({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="font-medium text-on-surface">{label}</span>
      {children}
      {hint && <span className="text-xs text-on-surface-muted">{hint}</span>}
    </label>
  );
}

/** Numeriek invoerveld; `null` bij een lege string (norm-forfait/leeg). */
export function NumberField({
  label,
  unit,
  value,
  step,
  placeholder,
  onChange,
  hint,
}: {
  label: string;
  unit?: string;
  value: number | null | undefined;
  step?: number | string;
  placeholder?: string;
  onChange: (v: number | null) => void;
  hint?: string;
}) {
  return (
    <LabeledField label={unit ? `${label} [${unit}]` : label} hint={hint}>
      <input
        type="number"
        step={step ?? "any"}
        value={value ?? ""}
        placeholder={placeholder}
        onChange={(e) =>
          onChange(e.target.value === "" ? null : Number(e.target.value))
        }
        className={INPUT_CLASS}
      />
    </LabeledField>
  );
}

/** Keuzelijst over een string-union (label + serde-waarde). */
export function SelectField<T extends string>({
  label,
  value,
  options,
  onChange,
  hint,
}: {
  label: string;
  value: T;
  options: ReadonlyArray<{ value: T; label: string }>;
  onChange: (v: T) => void;
  hint?: string;
}) {
  return (
    <LabeledField label={label} hint={hint}>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value as T)}
        className={INPUT_CLASS}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </LabeledField>
  );
}

/** Tekst-invoerveld; lege string → `null` (wist het optionele veld). */
export function TextField({
  label,
  value,
  placeholder,
  maxLength,
  onChange,
  hint,
}: {
  label: string;
  value: string | null | undefined;
  placeholder?: string;
  maxLength?: number;
  onChange: (v: string | null) => void;
  hint?: string;
}) {
  return (
    <LabeledField label={label} hint={hint}>
      <input
        type="text"
        value={value ?? ""}
        placeholder={placeholder}
        maxLength={maxLength}
        onChange={(e) => onChange(e.target.value === "" ? null : e.target.value)}
        className={INPUT_CLASS}
      />
    </LabeledField>
  );
}
